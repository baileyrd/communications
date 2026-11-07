//! Reproducible offline trials and telemetry import. This module never calls a model.
use crate::{
    adapter::{self, Arm, Context},
    codec::Encoding,
    golden::{self, LoadedCase, Score},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, hint::black_box, path::Path, time::Instant};

pub const WARMUP: u64 = 3;
pub const ITERATIONS: u64 = 20;
const MAX_IMPORT_BYTES: u64 = 16 * 1024 * 1024;
const CONTEXTS: [&str; 4] = ["human_model", "model_tool", "model_model", "model_data"];
const ARMS: [&str; 4] = ["conventional", "json", "cbor", "records"];

fn context(label: &str) -> Result<Context, String> {
    match label {
        "human_model" => Ok(Context::HumanModel),
        "model_tool" => Ok(Context::ModelTool),
        "model_model" => Ok(Context::ModelModel),
        "model_data" => Ok(Context::ModelData),
        _ => Err("unsupported offline context".into()),
    }
}
fn arm(label: &str) -> Result<Arm, String> {
    match label {
        "conventional" => Ok(Arm::Conventional),
        "json" => Ok(Arm::Semantic(Encoding::Json)),
        "cbor" => Ok(Arm::Semantic(Encoding::Cbor)),
        "records" => Ok(Arm::Semantic(Encoding::Records)),
        _ => Err("unsupported arm".into()),
    }
}

/// All questions/schema/instructions are common within a case. Expected answers
/// and original source are intentionally absent from the receiver's request.
pub fn request(case: &LoadedCase, ctx: &str, arm_name: &str) -> Result<Vec<u8>, String> {
    let envelope = adapter::render(context(ctx)?, arm(arm_name)?, &case.contract)?;
    let questions: Vec<_> = case
        .reference
        .questions
        .iter()
        .map(|q| json!({"id":q.id,"task":q.task,"prompt":q.prompt}))
        .collect();
    serde_json::to_vec(&json!({
        "harness_schema_version":golden::VERSION,
        "instructions":golden::INSTRUCTIONS,
        "response_schema":golden::RESPONSE_SCHEMA,
        "questions":questions,
        "context_envelope":serde_json::from_slice::<Value>(&envelope).map_err(|e| e.to_string())?
    }))
    .map_err(|e| e.to_string())
}

fn time<T>(mut operation: impl FnMut() -> Result<T, String>) -> Result<u64, String> {
    for _ in 0..WARMUP {
        black_box(operation()?);
    }
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        black_box(operation()?);
    }
    u64::try_from(start.elapsed().as_nanos()).map_err(|e| e.to_string())
}

/// Import-only telemetry. Values are caller-attested, never locally estimated.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Metrics {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub model_revision: Option<String>,
    pub settings: Option<Value>,
    pub usage_evidence: Option<String>,
    pub actual_input_tokens: Option<u64>,
    pub actual_output_tokens: Option<u64>,
    pub actual_cached_input_tokens: Option<u64>,
    pub retry_count: Option<u32>,
    pub trial_error: Option<String>,
    pub model_latency_ms: Option<f64>,
    pub end_to_end_latency_ms: Option<f64>,
    pub cost_usd: Option<f64>,
    pub cost_evidence: Option<String>,
    pub cpu_time_ns: Option<u64>,
    pub peak_rss_bytes: Option<u64>,
}
impl Metrics {
    fn validate(&self, origin: Origin) -> Result<(), String> {
        if origin == Origin::Synthetic
            && serde_json::to_value(self).unwrap() != serde_json::to_value(Self::default()).unwrap()
        {
            return Err("synthetic replay cannot carry model/resource telemetry".into());
        }
        if origin == Origin::ImportedReal {
            if [&self.provider, &self.model, &self.model_revision]
                .iter()
                .any(|s| s.as_ref().is_none_or(|s| s.trim().is_empty()))
                || !self.settings.as_ref().is_some_and(Value::is_object)
            {
                return Err("imported real trial needs provider/model/revision/settings".into());
            }
            let usage = self.actual_input_tokens.is_some()
                || self.actual_output_tokens.is_some()
                || self.actual_cached_input_tokens.is_some();
            if usage
                && self
                    .usage_evidence
                    .as_ref()
                    .is_none_or(|s| s.trim().is_empty())
            {
                return Err("actual tokens need caller-attested provider usage evidence".into());
            }
            if self.cost_usd.is_some()
                && self
                    .cost_evidence
                    .as_ref()
                    .is_none_or(|s| s.trim().is_empty())
            {
                return Err("cost needs billing or dated pricing evidence".into());
            }
        }
        for n in [
            self.model_latency_ms,
            self.end_to_end_latency_ms,
            self.cost_usd,
        ]
        .into_iter()
        .flatten()
        {
            if !n.is_finite() || n < 0.0 {
                return Err("negative/nonfinite metric".into());
            }
        }
        if self
            .actual_cached_input_tokens
            .is_some_and(|cached| self.actual_input_tokens.is_none_or(|input| cached > input))
        {
            return Err("cached tokens must be a subset of actual input tokens".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Synthetic,
    ImportedReal,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Replay {
    pub version: u32,
    pub trial_id: String,
    pub case_id: String,
    pub context: String,
    pub arm: String,
    pub request_sha256: String,
    pub origin: Origin,
    /// Raw text preserves malformed JSON and duplicate keys for scoring.
    pub response_json: String,
    pub metrics: Metrics,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Row {
    pub version: u32,
    pub harness_version: String,
    pub mode: String,
    pub trial_id: String,
    pub case_id: String,
    pub context: String,
    pub arm: String,
    pub suite_sha256: String,
    pub source_sha256: String,
    pub contract_sha256: String,
    pub cargo_lock_sha256: String,
    pub response_schema_sha256: String,
    pub instructions_sha256: String,
    pub request_sha256: String,
    pub response_sha256: Option<String>,
    pub payload_bytes: usize,
    pub adapter_envelope_bytes: usize,
    pub final_request_bytes: usize,
    pub schema_text_bytes: usize,
    pub response_schema_text_bytes: usize,
    pub source_digest_verified: bool,
    pub ledger_verified: bool,
    pub structural_roundtrip: bool,
    pub warmup_iterations: u64,
    pub iterations: u64,
    pub codec_encode_validate_total_ns: Option<u64>,
    pub codec_decode_validate_total_ns: Option<u64>,
    pub adapter_render_encode_validate_total_ns: Option<u64>,
    pub adapter_parse_decode_validate_total_ns: Option<u64>,
    pub request_render_total_ns: Option<u64>,
    pub timing_boundary: String,
    pub metrics: Metrics,
    pub score: Option<Score>,
    pub correct_assertions_per_1000_actual_input_tokens: Option<f64>,
    pub limitation: String,
}
fn base(
    case: &LoadedCase,
    ctx: &str,
    arm_name: &str,
    trial_id: String,
    mode: &str,
) -> Result<Row, String> {
    let a = arm(arm_name)?;
    let c = context(ctx)?;
    let bytes = adapter::render(c, a, &case.contract)?;
    let recovered = adapter::parse(c, a, &bytes)?;
    case.reference.validate_contract(&recovered)?;
    // Whole contract equality also preserves entity kinds, relation types and evidence.
    if recovered != case.contract {
        return Err("structural roundtrip failed".into());
    }
    let final_request = request(case, ctx, arm_name)?;
    Ok(Row {
        version:golden::VERSION, harness_version:env!("CARGO_PKG_VERSION").into(), mode:mode.into(), trial_id,
        case_id:case.reference.id.clone(), context:ctx.into(), arm:arm_name.into(),
        suite_sha256:golden::digest(golden::SUITE_BYTES), source_sha256:case.reference.source_sha256.clone(),
        contract_sha256:case.contract_sha256.clone(), cargo_lock_sha256:golden::digest(include_bytes!("../Cargo.lock")),
        response_schema_sha256:golden::digest(golden::RESPONSE_SCHEMA.as_bytes()), instructions_sha256:golden::digest(golden::INSTRUCTIONS.as_bytes()),
        request_sha256:golden::digest(&final_request), response_sha256:None,
        payload_bytes:a.payload(c, &case.contract)?.len(), adapter_envelope_bytes:bytes.len(), final_request_bytes:final_request.len(),
        schema_text_bytes:a.schema_bytes(), response_schema_text_bytes:golden::RESPONSE_SCHEMA.len(),
        source_digest_verified:true, ledger_verified:true, structural_roundtrip:true,
        warmup_iterations:0, iterations:0, codec_encode_validate_total_ns:None, codec_decode_validate_total_ns:None,
        adapter_render_encode_validate_total_ns:None, adapter_parse_decode_validate_total_ns:None, request_render_total_ns:None,
        timing_boundary:"Totals in ns, 3 warmups then 20 iterations each; includes allocations/drop and validation. Codec spans separate; adapter spans include codec/schema/transfer/envelope; request render includes adapter plus common questions/instructions/response schema and JSON serialization. Spans overlap, never sum them. Source/ledger loading/hash, roundtrip assertions, I/O, printing, build and inference excluded.".into(),
        metrics:Metrics::default(), score:None, correct_assertions_per_1000_actual_input_tokens:None,
        limitation:"Local offline requests only, no provider/HTTP/MCP/retrieval/hidden-state access or model execution. Replay scores are exact structured rubric checks; imported telemetry is caller-attested and unverified. Synthetic replay is never model evidence. Wall times are not CPU or end-to-end latency; one warmed process/fixed order, no inference efficiency conclusion.".into(),
    })
}

pub fn run(initial_only: bool) -> Result<Vec<Row>, String> {
    let mut rows = Vec::new();
    for case in golden::load()?
        .iter()
        .take(if initial_only { 3 } else { usize::MAX })
    {
        for ctx in CONTEXTS {
            for arm_name in ARMS {
                let c = context(ctx)?;
                let a = arm(arm_name)?;
                let mut row = base(
                    case,
                    ctx,
                    arm_name,
                    format!("offline/{}/{ctx}/{arm_name}", case.reference.id),
                    "offline_local",
                )?;
                if let Arm::Semantic(encoding) = a {
                    let payload = encoding.encode(&case.contract)?;
                    row.codec_encode_validate_total_ns =
                        Some(time(|| encoding.encode(black_box(&case.contract)))?);
                    row.codec_decode_validate_total_ns =
                        Some(time(|| encoding.decode(black_box(&payload)))?);
                }
                let envelope = adapter::render(c, a, &case.contract)?;
                row.adapter_render_encode_validate_total_ns =
                    Some(time(|| adapter::render(c, a, black_box(&case.contract)))?);
                row.adapter_parse_decode_validate_total_ns =
                    Some(time(|| adapter::parse(c, a, black_box(&envelope)))?);
                row.request_render_total_ns =
                    Some(time(|| request(black_box(case), ctx, arm_name))?);
                row.warmup_iterations = WARMUP;
                row.iterations = ITERATIONS;
                rows.push(row);
            }
        }
    }
    Ok(rows)
}

pub fn replay(input: &str) -> Result<Vec<Row>, String> {
    if input.is_empty() {
        return Err("replay contains no trials".into());
    }
    if input.len() as u64 > MAX_IMPORT_BYTES {
        return Err("replay exceeds 16 MiB".into());
    }
    let cases = golden::load()?;
    let mut ids = std::collections::BTreeSet::new();
    input
        .lines()
        .enumerate()
        .map(|(line, text)| {
            let trial: Replay = golden::from_json(text.as_bytes())
                .map_err(|e| format!("replay line {}: {e}", line + 1))?;
            if trial.version != golden::VERSION
                || trial.trial_id.trim().is_empty()
                || !ids.insert(trial.trial_id.clone())
            {
                return Err("unsupported version/empty or duplicate trial id".into());
            }
            trial.metrics.validate(trial.origin)?;
            let case = cases
                .iter()
                .find(|c| c.reference.id == trial.case_id)
                .ok_or("unknown replay case")?;
            let mode = match trial.origin {
                Origin::Synthetic => "synthetic_replay",
                Origin::ImportedReal => "imported_real_unverified",
            };
            let mut row = base(case, &trial.context, &trial.arm, trial.trial_id, mode)?;
            if row.request_sha256 != trial.request_sha256 {
                return Err("replay request digest mismatch".into());
            }
            row.response_sha256 = Some(golden::digest(trial.response_json.as_bytes()));
            if trial.metrics.trial_error.is_none() {
                row.score = Some(golden::score(
                    &case.reference,
                    trial.response_json.as_bytes(),
                ));
                if trial.origin == Origin::ImportedReal {
                    row.correct_assertions_per_1000_actual_input_tokens =
                        golden::assertion_efficiency(
                            row.score.as_ref(),
                            trial.metrics.actual_input_tokens,
                        );
                }
            }
            row.metrics = trial.metrics;
            Ok(row)
        })
        .collect()
}

/// A normal CSV with every JSONL field represented. Nested structures are compact
/// JSON in RFC4180-quoted cells. JSON null is literal null, never an empty/zero value.
pub fn exports(rows: &[Row]) -> Result<(String, String), String> {
    let values: Vec<Value> = rows
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
    let mut jsonl = String::new();
    let mut csv = String::new();
    if let Some(first) = values.first() {
        let keys: Vec<_> = first
            .as_object()
            .ok_or("row is not an object")?
            .keys()
            .collect();
        csv.push_str(
            &keys
                .iter()
                .map(|s| csv_cell(s))
                .collect::<Vec<_>>()
                .join(","),
        );
        csv.push('\n');
        for value in &values {
            jsonl.push_str(&value.to_string());
            jsonl.push('\n');
            let fields = keys
                .iter()
                .map(|key| {
                    let v = &value[*key];
                    csv_cell(&match v {
                        Value::String(s) => s.clone(),
                        _ => v.to_string(),
                    })
                })
                .collect::<Vec<_>>();
            csv.push_str(&fields.join(","));
            csv.push('\n');
        }
    }
    Ok((jsonl, csv))
}
fn csv_cell(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}

fn read_bounded(path: &str) -> Result<String, String> {
    use std::io::Read;
    let mut bytes = Vec::new();
    fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(MAX_IMPORT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_IMPORT_BYTES {
        return Err("replay exceeds 16 MiB".into());
    }
    String::from_utf8(bytes).map_err(|e| e.to_string())
}
fn save(rows: &[Row], jsonl: &str, csv: &str) -> Result<(), String> {
    if Path::new(jsonl) == Path::new(csv) {
        return Err("export paths must differ".into());
    }
    let (j, c) = exports(rows)?;
    fs::write(jsonl, j).map_err(|e| e.to_string())?;
    fs::write(csv, c).map_err(|e| e.to_string())?;
    eprintln!("wrote {} rows to {jsonl} and {csv}", rows.len());
    Ok(())
}
pub const HELP: &str = "Usage: communications-probe [golden check [--initial] | golden run JSONL CSV [--initial] | golden request CASE CONTEXT ARM | golden replay INPUT_JSONL OUTPUT_JSONL OUTPUT_CSV | golden sample]\nNo arguments: unchanged #792 smoke. Context: human_model|model_tool|model_model|model_data. Arm: conventional|json|cbor|records. sample prints one explicitly synthetic perfect replay record. No live/model calls.";
pub fn cli(args: &[String]) -> Result<(), String> {
    let arg: Vec<&str> = args.iter().map(String::as_str).collect();
    match arg.as_slice() {
        ["golden", "check"] | ["golden", "check", "--initial"] => {
            let cases = golden::load()?;
            let count = if arg.len() == 3 { 3 } else { cases.len() };
            for case in cases.iter().take(count) {
                let control = golden::score(
                    &case.reference,
                    case.reference.synthetic_perfect().to_string().as_bytes(),
                );
                if control.fidelity != Some(1.0) {
                    return Err("synthetic perfect control failed".into());
                }
                println!(
                    "{}",
                    json!({"case":case.reference.id,"source_sha256":case.reference.source_sha256,"contract_sha256":case.contract_sha256,"source_bytes":case.source.len(),"ledger_verified":true,"synthetic_control_only":control})
                );
            }
            Ok(())
        }
        ["golden", "run", j, c] | ["golden", "run", j, c, "--initial"] => {
            save(&run(arg.len() == 5)?, j, c)
        }
        ["golden", "request", id, ctx, a] => {
            use std::io::Write;
            let cases = golden::load()?;
            let case = cases
                .iter()
                .find(|c| c.reference.id == *id)
                .ok_or("unknown case")?;
            std::io::stdout()
                .write_all(&request(case, ctx, a)?)
                .map_err(|e| e.to_string())
        }
        ["golden", "replay", input, j, c] => {
            if Path::new(input) == Path::new(j) || Path::new(input) == Path::new(c) {
                return Err("do not overwrite replay input".into());
            }
            save(&replay(&read_bounded(input)?)?, j, c)
        }
        ["golden", "sample"] => {
            let cases = golden::load()?;
            let case = &cases[0];
            println!(
                "{}",
                serde_json::to_string(&Replay {
                    version: golden::VERSION,
                    trial_id: "synthetic-perfect-conflict".into(),
                    case_id: case.reference.id.clone(),
                    context: "human_model".into(),
                    arm: "json".into(),
                    request_sha256: golden::digest(&request(case, "human_model", "json")?),
                    origin: Origin::Synthetic,
                    response_json: case.reference.synthetic_perfect().to_string(),
                    metrics: Metrics::default()
                })
                .map_err(|e| e.to_string())?
            );
            Ok(())
        }
        ["--help"] | ["golden", "--help"] => {
            println!("{HELP}");
            Ok(())
        }
        _ => Err(HELP.into()),
    }
}

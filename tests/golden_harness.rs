use communications_probe::{
    adapter::{self, Arm, Context},
    codec::Encoding,
    golden::{self, Case, Score},
    harness::{self, Metrics, Origin, Replay},
};
use serde_json::{json, Value};
use std::{collections::BTreeMap, process::Command};

fn reference() -> Case {
    golden::load().unwrap().remove(0).reference
}
fn score(case: &Case, value: &Value) -> Score {
    golden::score(case, value.to_string().as_bytes())
}

#[test]
fn three_initial_then_six_pinned_cases_preserve_every_arm_and_metadata() {
    let cases = golden::load().unwrap();
    assert_eq!(cases.len(), 6);
    assert_eq!(
        cases
            .iter()
            .take(3)
            .map(|c| c.reference.id.as_str())
            .collect::<Vec<_>>(),
        ["conflict", "unknown", "ordering"]
    );
    for case in cases {
        assert_eq!(golden::digest(case.source), case.reference.source_sha256);
        let perfect = score(&case.reference, &case.reference.synthetic_perfect());
        assert_eq!(perfect.fidelity, Some(1.0));
        assert_eq!(perfect.correct_assertions, 2);
        assert_eq!(perfect.comprehension.correct, 1);
        assert_eq!(perfect.application.correct, 1);
        for ctx in Context::OFFLINE {
            for a in [
                Arm::Conventional,
                Arm::Semantic(Encoding::Json),
                Arm::Semantic(Encoding::Cbor),
                Arm::Semantic(Encoding::Records),
            ] {
                let bytes = adapter::render(ctx, a, &case.contract).unwrap();
                let recovered = adapter::parse(ctx, a, &bytes).unwrap();
                case.reference.validate_contract(&recovered).unwrap();
                assert_eq!(recovered, case.contract);
            }
        }
        let mut omitted = case.contract.clone();
        omitted.assertions.pop();
        assert!(case.reference.validate_contract(&omitted).is_err());
        let mut shared_bug = case.contract.clone();
        shared_bug.assertions[0].times.record_ms += 1;
        assert!(case.reference.validate_contract(&shared_bug).is_err());
        let mut extra = case.contract.clone();
        extra
            .evidence
            .insert("invented".into(), extra.evidence["e1"].clone());
        assert!(case.reference.validate_contract(&extra).is_err());
        let mut kinds = case.contract.clone();
        kinds
            .entities
            .values_mut()
            .for_each(|e| e.kind = "wrong".into());
        kinds.relations.values_mut().for_each(|r| {
            r.subject_kind = "wrong".into();
            if r.object_kind == "room" {
                r.object_kind = "wrong".into();
            }
        });
        assert!(case.reference.validate_contract(&kinds).is_err());
    }
}

#[test]
fn scorer_penalizes_omissions_inventions_contradictions_and_partial_metadata_separately() {
    let c = reference();
    let perfect = c.synthetic_perfect();
    let mut omitted = perfect.clone();
    omitted["reconstruction"].as_array_mut().unwrap().pop();
    let s = score(&c, &omitted);
    assert_eq!(s.missing_assertions, 1);
    assert_eq!(s.correct_assertions, 1);
    assert_eq!(s.fidelity, Some(0.75));
    let mut invented = perfect.clone();
    let mut extra = invented["reconstruction"][0].clone();
    extra["id"] = json!("unsupported");
    invented["reconstruction"]
        .as_array_mut()
        .unwrap()
        .push(extra);
    let s = score(&c, &invented);
    assert_eq!(s.invented_assertions, 1);
    assert_eq!(s.correct_assertions, 2);
    assert_eq!(s.fidelity, Some(0.75));
    let mut contradict = perfect.clone();
    contradict["reconstruction"][0]["object"]["value"] = json!(false);
    let s = score(&c, &contradict);
    assert_eq!(s.wrong_relationships, 1);
    assert_eq!(s.correct_assertions, 1);
    let mut partial = perfect.clone();
    partial["reconstruction"][0]["times"]["event_ms"] = Value::Null;
    partial["reconstruction"][0]["provenance"]["asserted_by"] = json!("bob");
    partial["reconstruction"][0]["uncertainty"] = json!({"kind":"unknown"});
    let s = score(&c, &partial);
    assert_eq!(
        (s.temporal_errors, s.provenance_errors, s.uncertainty_errors),
        (1, 1, 1)
    );
    assert_eq!(s.wrong_relationships, 0);
    assert_eq!(s.fidelity, Some(0.25));
    partial["reconstruction"][0]
        .as_object_mut()
        .unwrap()
        .remove("times");
    partial["reconstruction"][0]
        .as_object_mut()
        .unwrap()
        .remove("provenance");
    partial["reconstruction"][0]
        .as_object_mut()
        .unwrap()
        .remove("uncertainty");
    let s = score(&c, &partial);
    assert!(s.valid_structured_response);
    assert_eq!(
        (s.temporal_errors, s.provenance_errors, s.uncertainty_errors),
        (1, 1, 1)
    );
    let all_missing = json!({"reconstruction":[],"comprehension":{},"application":{}});
    let s = score(&c, &all_missing);
    assert_eq!(s.missing_assertions, 2);
    assert_eq!(s.fidelity, Some(0.0));
}

#[test]
fn answers_use_exact_types_unknown_null_and_explicit_task_rubrics() {
    let c = reference();
    let mut r = c.synthetic_perfect();
    r["comprehension"]["c"] = json!("bob");
    r["application"]["a"] = json!("false");
    r["application"]["invented"] = json!(true);
    let s = score(&c, &r);
    assert_eq!(s.comprehension.wrong, 1);
    assert_eq!(s.application.wrong, 1);
    assert_eq!(s.application.invented, 1);
    assert_eq!(s.fidelity, Some(0.25));
    r["comprehension"].as_object_mut().unwrap().remove("c");
    assert_eq!(score(&c, &r).comprehension.missing, 1);
    let unknown = golden::load().unwrap().remove(1).reference;
    let r = unknown.synthetic_perfect();
    assert!(r["comprehension"]["c"].is_null());
    assert_eq!(score(&unknown, &r).comprehension.correct, 1);
    let relocation = golden::load().unwrap().remove(4).reference;
    let mut r = relocation.synthetic_perfect();
    r["application"]["a"] = json!(["r2", "r1"]);
    assert_eq!(score(&relocation, &r).application.wrong, 1);
}

#[test]
fn malformed_duplicate_and_ambiguous_responses_have_no_valid_score() {
    let c = reference();
    for bytes in [
        b"not json".as_slice(),
        b"{}",
        br#"{"reconstruction":[],"reconstruction":[],"comprehension":{},"application":{}}"#,
        br#"{"reconstruction":[],"comprehension":{"c":1,"c":2},"application":{}}"#,
    ] {
        let s = golden::score(&c, bytes);
        assert!(!s.valid_structured_response);
        assert_eq!(s.malformed, 1);
        assert_eq!(s.fidelity, None);
        assert!(s.error.is_some());
    }
    let mut duplicate = c.synthetic_perfect();
    let first = duplicate["reconstruction"][0].clone();
    duplicate["reconstruction"]
        .as_array_mut()
        .unwrap()
        .push(first);
    assert_eq!(score(&c, &duplicate).malformed, 1);
    let mut unknown = c.synthetic_perfect();
    unknown["extra"] = json!(1);
    assert_eq!(score(&c, &unknown).malformed, 1);
    let mut bad = c.synthetic_perfect();
    bad["reconstruction"][0]["times"]
        .as_object_mut()
        .unwrap()
        .remove("event_ms");
    assert_eq!(score(&c, &bad).malformed, 1);
}

#[test]
fn efficiencies_and_empty_denominators_are_never_fabricated() {
    let mut c = reference();
    let s = score(&c, &c.synthetic_perfect());
    assert_eq!(
        golden::assertion_efficiency(Some(&s), Some(100)),
        Some(20.0)
    );
    assert_eq!(golden::assertion_efficiency(Some(&s), None), None);
    assert_eq!(golden::assertion_efficiency(Some(&s), Some(0)), None);
    assert_eq!(golden::assertion_efficiency(None, Some(100)), None);
    assert_eq!(
        golden::assertion_efficiency(Some(&Score::default()), Some(100)),
        None
    );
    c.facts.clear();
    c.questions.clear();
    let s = score(
        &c,
        &json!({"reconstruction":[],"comprehension":{},"application":{}}),
    );
    assert_eq!(s.fidelity, None);
    assert_eq!(golden::assertion_efficiency(Some(&s), Some(100)), None);
}

fn replay() -> Replay {
    let cases = golden::load().unwrap();
    let c = &cases[0];
    Replay {
        version: 1,
        trial_id: "synthetic-test".into(),
        case_id: c.reference.id.clone(),
        context: "human_model".into(),
        arm: "json".into(),
        request_sha256: golden::digest(&harness::request(c, "human_model", "json").unwrap()),
        origin: Origin::Synthetic,
        response_json: c.reference.synthetic_perfect().to_string(),
        metrics: Metrics::default(),
    }
}
fn import(r: &Replay) -> Result<Vec<harness::Row>, String> {
    harness::replay(&serde_json::to_string(r).unwrap())
}
#[test]
fn request_identity_replay_telemetry_and_error_boundaries() {
    let r = replay();
    let rows = import(&r).unwrap();
    assert_eq!(rows[0].mode, "synthetic_replay");
    assert_eq!(rows[0].score.as_ref().unwrap().fidelity, Some(1.0));
    assert_eq!(
        rows[0].correct_assertions_per_1000_actual_input_tokens,
        None
    );
    let mut bad = r.clone();
    bad.request_sha256 = "stale".into();
    assert!(import(&bad).unwrap_err().contains("digest"));
    let mut bad = r.clone();
    bad.context = "model_native".into();
    assert!(import(&bad).is_err());
    let mut bad = r.clone();
    bad.case_id = "absent".into();
    assert!(import(&bad).is_err());
    let mut bad = r.clone();
    bad.version = 99;
    assert!(import(&bad).is_err());
    let mut bad = r.clone();
    bad.metrics.actual_input_tokens = Some(100);
    assert!(import(&bad).unwrap_err().contains("synthetic"));
    let mut real = r.clone();
    real.origin = Origin::ImportedReal;
    assert!(import(&real).is_err());
    // Unit-test metadata only. These numbers are never exported as measured trials.
    real.metrics.provider = Some("unit-test".into());
    real.metrics.model = Some("not-a-model".into());
    real.metrics.model_revision = Some("test-only".into());
    real.metrics.settings = Some(json!({"temperature":0}));
    real.metrics.actual_input_tokens = Some(100);
    assert!(import(&real).is_err());
    real.metrics.usage_evidence = Some("unit-test counter only".into());
    let rows = import(&real).unwrap();
    assert_eq!(
        rows[0].correct_assertions_per_1000_actual_input_tokens,
        Some(20.0)
    );
    real.metrics.actual_input_tokens = Some(0);
    assert_eq!(
        import(&real).unwrap()[0].correct_assertions_per_1000_actual_input_tokens,
        None
    );
    real.metrics.actual_cached_input_tokens = Some(1);
    assert!(import(&real).is_err());
    real.metrics.actual_cached_input_tokens = None;
    real.metrics.cost_usd = Some(0.1);
    assert!(import(&real).is_err());
    real.metrics.cost_usd = None;
    real.metrics.model_latency_ms = Some(-1.0);
    assert!(import(&real).is_err());
    real.metrics.model_latency_ms = None;
    real.response_json = "malformed".into();
    let row = import(&real).unwrap().remove(0);
    assert_eq!(row.score.unwrap().malformed, 1);
    assert_eq!(row.correct_assertions_per_1000_actual_input_tokens, None);
    real.metrics.trial_error = Some("unit-test failed request".into());
    let row = import(&real).unwrap().remove(0);
    assert!(row.score.is_none());
    let line = serde_json::to_string(&r).unwrap();
    assert!(harness::replay(&format!("{line}\n{line}\n")).is_err());
}

// Minimal independent parser for the all-quoted exporter, including embedded newline.
fn csv_rows(s: &str) -> Vec<Vec<String>> {
    let mut rows = vec![];
    let mut row = vec![];
    let mut cell = String::new();
    let mut quoted = false;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' if quoted && chars.peek() == Some(&'"') => {
                chars.next();
                cell.push('"');
            }
            '"' => quoted = !quoted,
            ',' if !quoted => row.push(std::mem::take(&mut cell)),
            '\n' if !quoted => {
                row.push(std::mem::take(&mut cell));
                rows.push(std::mem::take(&mut row));
            }
            c => cell.push(c),
        }
    }
    assert!(!quoted);
    assert!(row.is_empty());
    rows
}
#[test]
fn jsonl_csv_are_losslessly_compatible_for_all_fields_and_special_characters() {
    let mut rows = import(&replay()).unwrap();
    rows[0].limitation = "comma, quote\" and\nnewline café".into();
    let (jsonl, csv) = harness::exports(&rows).unwrap();
    let parsed = csv_rows(&csv);
    assert_eq!(parsed.len(), 2);
    let values: BTreeMap<String, String> = parsed[0]
        .iter()
        .cloned()
        .zip(parsed[1].iter().cloned())
        .collect();
    let j: Value = serde_json::from_str(jsonl.trim()).unwrap();
    for (key, v) in j.as_object().unwrap() {
        let restored = match v {
            Value::String(_) => Value::String(values[key].clone()),
            _ => serde_json::from_str(&values[key]).unwrap(),
        };
        assert_eq!(&restored, v, "{key}");
    }
    let (j, c) = harness::exports(&[]).unwrap();
    assert!(j.is_empty() && c.is_empty());
}

#[test]
fn final_requests_count_schema_questions_escaping_and_do_not_leak_answers() {
    let cases = golden::load().unwrap();
    let c = &cases[0];
    for ctx in ["human_model", "model_tool", "model_model", "model_data"] {
        let mut common = None;
        for a in ["conventional", "json", "cbor", "records"] {
            let bytes = harness::request(c, ctx, a).unwrap();
            let v: Value = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(serde_json::to_vec(&v).unwrap(), bytes);
            assert!(v["questions"][0].get("expected").is_none());
            assert!(v.get("source").is_none());
            let fields = json!([v["questions"], v["instructions"], v["response_schema"]]);
            if let Some(ref prior) = common {
                assert_eq!(&fields, prior);
            } else {
                common = Some(fields);
            }
            assert_eq!(v["response_schema"], golden::RESPONSE_SCHEMA);
            assert!(bytes.len() > golden::RESPONSE_SCHEMA.len());
        }
    }
}

#[test]
fn cli_check_initial_expanded_and_usage_errors() {
    let binary = env!("CARGO_BIN_EXE_communications-probe");
    for (args, n) in [
        (vec!["golden", "check", "--initial"], 3),
        (vec!["golden", "check"], 6),
    ] {
        let out = Command::new(binary).args(args).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(String::from_utf8(out.stdout).unwrap().lines().count(), n);
    }
    let sample = Command::new(binary)
        .args(["golden", "sample"])
        .output()
        .unwrap();
    assert!(sample.status.success());
    let r: Replay = golden::from_json(&sample.stdout).unwrap();
    assert_eq!(import(&r).unwrap()[0].mode, "synthetic_replay");
    let bad = Command::new(binary)
        .args(["golden", "run"])
        .output()
        .unwrap();
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("Usage:"));
}

#[test]
fn cli_runs_full_matrix_exports_and_replays_without_model_measurements() {
    let binary = env!("CARGO_BIN_EXE_communications-probe");
    let dir =
        std::env::temp_dir().join(format!("communications-golden-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let j = dir.join("offline.jsonl");
    let c = dir.join("offline.csv");
    let out = Command::new(binary)
        .args(["golden", "run"])
        .arg(&j)
        .arg(&c)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = std::fs::read_to_string(&j).unwrap();
    let rows: Vec<harness::Row> = text
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert_eq!(rows.len(), 6 * 4 * 4);
    assert_eq!(csv_rows(&std::fs::read_to_string(&c).unwrap()).len(), 97);
    let cases = golden::load().unwrap();
    for row in rows {
        let case = cases
            .iter()
            .find(|c| c.reference.id == row.case_id)
            .unwrap();
        let request = harness::request(case, &row.context, &row.arm).unwrap();
        assert_eq!(request.len(), row.final_request_bytes);
        assert_eq!(golden::digest(&request), row.request_sha256);
        assert_eq!(row.iterations, harness::ITERATIONS);
        assert_eq!(row.warmup_iterations, harness::WARMUP);
        assert!(row.request_render_total_ns.is_some());
        assert!(row.structural_roundtrip && row.ledger_verified && row.source_digest_verified);
        assert!(row.score.is_none());
        assert!(row
            .correct_assertions_per_1000_actual_input_tokens
            .is_none());
        assert_eq!(
            serde_json::to_value(row.metrics).unwrap(),
            serde_json::to_value(Metrics::default()).unwrap()
        );
        assert_eq!(
            row.codec_encode_validate_total_ns.is_some(),
            row.arm != "conventional"
        );
    }
    let input = dir.join("input.jsonl");
    let mut trial = replay();
    trial.response_json = "not JSON".into();
    std::fs::write(&input, serde_json::to_vec(&trial).unwrap()).unwrap();
    let out = Command::new(binary)
        .args(["golden", "replay"])
        .arg(&input)
        .arg(&j)
        .arg(&c)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let row: harness::Row =
        serde_json::from_str(std::fs::read_to_string(&j).unwrap().trim()).unwrap();
    assert_eq!(row.mode, "synthetic_replay");
    assert_eq!(row.score.unwrap().malformed, 1);
    let denied = Command::new(binary)
        .args(["golden", "replay"])
        .arg(&input)
        .arg(&input)
        .arg(&c)
        .output()
        .unwrap();
    assert!(!denied.status.success());
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn checked_in_synthetic_controls_have_explicit_non_model_outcomes() {
    let rows = harness::replay(include_str!("../fixtures/golden/synthetic-replay.jsonl")).unwrap();
    let expected = [
        Some(1.0),
        Some(0.75),
        Some(0.75),
        Some(0.75),
        Some(0.25),
        None,
        Some(0.5),
    ];
    assert_eq!(rows.len(), expected.len());
    for (row, expected) in rows.iter().zip(expected) {
        assert_eq!(row.mode, "synthetic_replay");
        assert_eq!(row.score.as_ref().unwrap().fidelity, expected);
        assert_eq!(row.correct_assertions_per_1000_actual_input_tokens, None);
        assert_eq!(
            serde_json::to_value(&row.metrics).unwrap(),
            serde_json::to_value(Metrics::default()).unwrap()
        );
    }
    assert!(harness::replay("").is_err());
}

use communications_probe::{
    adapter::{self, Arm, Context},
    codec::Encoding,
    fixture, SCHEMA_VERSION,
};
use serde_json::json;
use std::{hint::black_box, time::Instant};

const WARMUP: u64 = 100;
const ITERATIONS: u64 = 1000;

// Includes construction, allocation, validation and destruction of each result.
// Fixture loading, evidence hashing, assertions, printing and build are outside.
fn time<T>(mut operation: impl FnMut() -> Result<T, String>) -> Result<u128, String> {
    for _ in 0..WARMUP {
        black_box(operation()?);
    }
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        black_box(operation()?);
    }
    Ok(start.elapsed().as_nanos())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let c = fixture();
    c.validate()?;
    c.verify_evidence("e1", include_bytes!("../fixtures/observation.txt"))?;
    let common = json!({
        "mode":"offline_codec_adapter_smoke", "case":"observation", "version":SCHEMA_VERSION,
        "iterations":ITERATIONS, "warmup_iterations":WARMUP, "assertions":c.assertions.len(),
        "evidence_digest_matches":true, "exact_structural_roundtrip":true,
        "model_input_tokens":null, "model_output_tokens":null, "model_cost_usd":null,
        "model_latency_ms":null, "end_to_end_latency_ms":null, "cpu_time_ns":null,
        "peak_memory_bytes":null, "semantic_fidelity_score":null,
        "correct_assertions_per_1000_input_tokens":null,
        "limitation":"One smoke fixture, one warmed process per command; no model or endpoint; structural preservation does not measure understanding. Adapter timings include codec work; do not add them to codec times."
    });
    for encoding in Encoding::ALL {
        let bytes = encoding.encode(&c)?;
        assert_eq!(encoding.decode(&bytes)?, c);
        let encode = time(|| encoding.encode(black_box(&c)))?;
        let decode = time(|| encoding.decode(black_box(&bytes)))?;
        let mut row = common.clone();
        row.as_object_mut().unwrap().extend(
            json!({
                "kind":"codec", "encoding":encoding, "payload_bytes":bytes.len(),
                "encode_validate_total_ns":encode, "decode_validate_total_ns":decode,
                "encode_validate_mean_ns":encode / u128::from(ITERATIONS),
                "decode_validate_mean_ns":decode / u128::from(ITERATIONS)
            })
            .as_object()
            .unwrap()
            .clone(),
        );
        println!("{row}");
    }
    for context in Context::OFFLINE {
        for arm in [
            Arm::Conventional,
            Arm::Semantic(Encoding::Json),
            Arm::Semantic(Encoding::Cbor),
            Arm::Semantic(Encoding::Records),
        ] {
            let payload = arm.payload(context, &c)?;
            let bytes = adapter::render(context, arm, &c)?;
            assert_eq!(adapter::parse(context, arm, &bytes)?, c);
            let render = time(|| adapter::render(context, arm, black_box(&c)))?;
            let parse = time(|| adapter::parse(context, arm, black_box(&bytes)))?;
            let text_bytes = match arm {
                Arm::Semantic(Encoding::Cbor | Encoding::Records) => payload.len().div_ceil(3) * 4,
                _ => payload.len(),
            };
            let mut row = common.clone();
            row.as_object_mut().unwrap().extend(json!({
                "kind":"adapter", "context":context, "arm":arm,
                "payload_bytes":payload.len(), "rendered_payload_text_bytes":text_bytes,
                "schema_text_bytes":arm.schema_bytes(), "schema_mode":"cold_included_in_final_envelope",
                "final_envelope_bytes":bytes.len(), "envelope_expansion_bytes":bytes.len() - payload.len(),
                "render_encode_validate_total_ns":render, "parse_decode_validate_total_ns":parse,
                "render_encode_validate_mean_ns":render / u128::from(ITERATIONS),
                "parse_decode_validate_mean_ns":parse / u128::from(ITERATIONS),
                "timing_boundary":"Full local translation: encode/validate + schema + text/base64 + envelope; parse checks duplicate keys, decodes/validates and reconstructs canonical envelope. Includes allocations/drop. Not network or inference."
            }).as_object().unwrap().clone());
            println!("{row}");
        }
    }
    for context in Context::OFFLINE.into_iter().chain([Context::ModelNative]) {
        println!(
            "{}",
            json!({"kind":"capability","capability":context.capability()})
        );
    }
    Ok(())
}

use communications_probe::{fixture, Contract, SCHEMA_VERSION};
use serde_json::json;
use std::hint::black_box;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let contract = fixture();
    contract.validate()?;
    contract.verify_evidence("e1", include_bytes!("../fixtures/observation.txt"))?;
    let compact = contract.canonical_json()?;
    let pretty = serde_json::to_vec_pretty(&contract)?;
    let iterations = 10_000;
    // Warm-up excludes initial allocation/cache effects, but not allocator cost in timed loops.
    for _ in 0..100 {
        black_box(contract.canonical_json()?);
        black_box(Contract::from_json(&compact)?);
    }
    let start = Instant::now();
    for _ in 0..iterations {
        black_box(contract.canonical_json()?);
    }
    let encode_ns = start.elapsed().as_nanos();
    let start = Instant::now();
    for _ in 0..iterations {
        let decoded = Contract::from_json(black_box(&compact))?;
        black_box(decoded);
    }
    let decode_ns = start.elapsed().as_nanos();
    let decoded = Contract::from_json(&compact)?;
    assert_eq!(decoded, contract);
    println!(
        "{}",
        json!({
            "mode":"offline_architecture_smoke", "case":"observation", "version":SCHEMA_VERSION,
            "representation":"semantic_json", "assertions":contract.assertions.len(),
            "wire_bytes":compact.len(), "pretty_json_control_bytes":pretty.len(),
            "iterations":iterations, "warmup_iterations":100,
            "validate_clone_sort_encode_total_ns":encode_ns,
            "decode_validate_total_ns":decode_ns,
            "validate_clone_sort_encode_mean_ns":encode_ns / iterations,
            "decode_validate_mean_ns":decode_ns / iterations,
            "exact_roundtrip":true, "evidence_digest_matches":true,
            "model_input_tokens":null, "model_output_tokens":null,
            "model_latency_ms":null, "end_to_end_latency_ms":null,
            "cpu_time_ns":null, "peak_memory_bytes":null,
            "model_cost_usd":null, "adapter_overhead_ns":null,
            "semantic_fidelity_score":null,
            "limitation":"one synthetic smoke fixture; JSON formatting control only; no model, adapter, CBOR or comprehension experiment"
        })
    );
    Ok(())
}

use communications_probe::{
    adapter::{self, Arm, Context},
    codec::Encoding,
    golden, harness,
};
use serde_json::{json, Value};
use std::process::Command;

#[test]
fn expanded_suite_preserves_originals_and_checks_every_extraction_and_wire_arm() {
    let old = golden::load().unwrap();
    let cases = golden::load_expanded().unwrap();
    assert_eq!(cases.len(), 60);
    for (a, b) in old.iter().zip(&cases) {
        assert_eq!(
            serde_json::to_value(&a.reference).unwrap(),
            serde_json::to_value(&b.reference).unwrap()
        );
        assert_eq!(a.source, b.source);
        assert_eq!(a.contract, b.contract);
        assert_ne!(a.suite_sha256, b.suite_sha256);
        assert_eq!(
            harness::request(a, "model_tool", "cbor").unwrap(),
            harness::request(b, "model_tool", "cbor").unwrap()
        );
    }
    for case in cases {
        assert_eq!(
            case.suite_sha256,
            golden::digest(golden::EXPANDED_SUITE_BYTES)
        );
        assert_eq!(
            golden::score(
                &case.reference,
                case.reference.synthetic_perfect().to_string().as_bytes()
            )
            .fidelity,
            Some(1.0)
        );
        for context in Context::OFFLINE {
            for arm in [
                Arm::Conventional,
                Arm::Semantic(Encoding::Json),
                Arm::Semantic(Encoding::Cbor),
                Arm::Semantic(Encoding::Records),
            ] {
                let bytes = adapter::render(context, arm, &case.contract).unwrap();
                assert_eq!(adapter::parse(context, arm, &bytes).unwrap(), case.contract);
            }
        }
        let mut broken = case.contract.clone();
        broken.assertions.remove(0);
        assert!(case.reference.validate_contract(&broken).is_err());
    }
}

#[test]
fn expanded_cli_selection_capabilities_and_null_boundaries() {
    let binary = env!("CARGO_BIN_EXE_communications-probe");
    let out = Command::new(binary)
        .args(["expanded", "check"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(String::from_utf8(out.stdout).unwrap().lines().count(), 60);
    let out = Command::new(binary)
        .args(["expanded", "capabilities"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let caps: Vec<Value> = String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    assert_eq!(caps.len(), 5);
    assert!(caps.iter().all(|c| c["live_endpoint"] == false));
    assert_eq!(caps[4]["context"], "model_native");
    assert_eq!(caps[4]["offline_roundtrip"], false);
    for arm in ["conventional", "json", "cbor", "records"] {
        let out = Command::new(binary)
            .args([
                "expanded",
                "trial",
                "boundary-join",
                "model_data",
                arm,
                "expanded/test",
            ])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let row: harness::Row = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(
            row.suite_sha256,
            golden::digest(golden::EXPANDED_SUITE_BYTES)
        );
        assert_eq!(row.iterations, 20);
        assert!(row.score.is_none() && row.response_sha256.is_none());
        assert!(serde_json::to_value(row.metrics)
            .unwrap()
            .as_object()
            .unwrap()
            .values()
            .all(Value::is_null));
    }
    for args in [
        ["expanded", "trial", "absent", "model_data", "json", "id"],
        [
            "expanded",
            "trial",
            "boundary-join",
            "model_native",
            "json",
            "id",
        ],
        [
            "expanded",
            "trial",
            "boundary-join",
            "model_data",
            "bad",
            "id",
        ],
        [
            "expanded",
            "trial",
            "boundary-join",
            "model_data",
            "json",
            " ",
        ],
        [
            "golden",
            "trial",
            "boundary-join",
            "model_data",
            "json",
            "id",
        ],
    ] {
        let out = Command::new(binary).args(args).output().unwrap();
        assert!(!out.status.success() && out.stdout.is_empty());
    }
}

#[test]
fn expanded_import_uses_same_unverified_and_fractional_boundary_with_new_case() {
    let case = golden::load_expanded().unwrap().pop().unwrap();
    let mut replay = json!({"version":1,"trial_id":"expanded-test-only","case_id":case.reference.id,
        "context":"model_model","arm":"json","request_sha256":golden::digest(&harness::request(&case,"model_model","json").unwrap()),
        "origin":"synthetic","response_json":case.reference.synthetic_perfect().to_string(),"metrics":harness::Metrics::default()});
    let dir = std::env::temp_dir().join(format!("expanded-replay-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let input = dir.join("input.jsonl");
    let j = dir.join("out.jsonl");
    let c = dir.join("out.csv");
    let run = |r: &Value| {
        std::fs::write(&input, r.to_string()).unwrap();
        Command::new(env!("CARGO_BIN_EXE_communications-probe"))
            .args(["expanded", "replay"])
            .arg(&input)
            .arg(&j)
            .arg(&c)
            .output()
            .unwrap()
    };
    assert!(run(&replay).status.success());
    let row: Value = serde_json::from_str(std::fs::read_to_string(&j).unwrap().trim()).unwrap();
    assert_eq!(row["mode"], "synthetic_replay");
    assert_eq!(row["score"]["fidelity"], 1.0);
    replay["origin"] = json!("imported_real");
    replay["metrics"]["provider"] = json!("unit-test");
    replay["metrics"]["model"] = json!("not-real");
    replay["metrics"]["model_revision"] = json!("test");
    replay["metrics"]["settings"] = json!({"temperature":0.0,"nested":{"p":0.95}});
    replay["metrics"]["model_latency_ms"] = json!(0.25);
    assert!(run(&replay).status.success());
    let row: Value = serde_json::from_str(std::fs::read_to_string(&j).unwrap().trim()).unwrap();
    assert_eq!(row["mode"], "imported_real_unverified");
    assert_eq!(row["metrics"]["model_latency_ms"], 0.25);
    assert!(row["correct_assertions_per_1000_actual_input_tokens"].is_null());
    replay["request_sha256"] = json!("stale");
    assert!(!run(&replay).status.success());
    std::fs::remove_dir_all(dir).unwrap();
}

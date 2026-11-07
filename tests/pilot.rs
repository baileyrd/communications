use std::process::Command;

#[test]
fn isolated_trial_cli_matches_requests_and_keeps_telemetry_unmeasured() {
    let binary = env!("CARGO_BIN_EXE_communications-probe");
    let case = communications_probe::golden::load().unwrap().remove(0);
    for arm in ["conventional", "json", "cbor", "records"] {
        let output = Command::new(binary)
            .args([
                "golden",
                "trial",
                "conflict",
                "model_tool",
                arm,
                "pilot/test",
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        let row: communications_probe::harness::Row =
            serde_json::from_slice(&output.stdout).unwrap();
        let request = communications_probe::harness::request(&case, "model_tool", arm).unwrap();
        assert_eq!(row.trial_id, "pilot/test");
        assert_eq!(
            row.request_sha256,
            communications_probe::golden::digest(&request)
        );
        assert_eq!(row.final_request_bytes, request.len());
        assert_eq!((row.iterations, row.warmup_iterations), (20, 3));
        assert_eq!(
            row.codec_encode_validate_total_ns.is_some(),
            arm != "conventional"
        );
        assert!(row.request_render_total_ns.is_some());
        assert!(row.structural_roundtrip && row.ledger_verified);
        assert!(row.score.is_none());
        assert!(row
            .correct_assertions_per_1000_actual_input_tokens
            .is_none());
        assert!(serde_json::to_value(row.metrics)
            .unwrap()
            .as_object()
            .unwrap()
            .values()
            .all(serde_json::Value::is_null));
    }
    for args in [
        [
            "golden",
            "trial",
            "unknown-case",
            "model_tool",
            "json",
            "id",
        ],
        ["golden", "trial", "conflict", "model_native", "json", "id"],
        ["golden", "trial", "conflict", "model_tool", "bad-arm", "id"],
        ["golden", "trial", "conflict", "model_tool", "json", " "],
    ] {
        let output = Command::new(binary).args(args).output().unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
    }
}

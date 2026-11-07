use communications_probe::{fixture, Contract, Object, Relation, Uncertainty, SCHEMA_VERSION};
use serde_json::{json, Value};

fn wire() -> String {
    String::from_utf8(fixture().canonical_json().unwrap()).unwrap()
}

fn rejected(value: Value, expected: &str) {
    let bytes = serde_json::to_vec(&value).unwrap();
    let error = Contract::from_json(&bytes).unwrap_err();
    assert!(
        error.contains(expected),
        "{error:?} should contain {expected:?}"
    );
    // Direct Serde ingress applies the same validation.
    assert!(serde_json::from_slice::<Contract>(&bytes).is_err());
}

#[test]
fn duplicate_json_keys_fail_at_every_object_boundary() {
    let original = wire();
    for (from, to) in [
        ("\"version\":", "\"version\":\"0.1-draft\",\"version\":"),
        (
            "\"observer\":",
            "\"observer\":{\"kind\":\"person\"},\"observer\":",
        ),
        (
            "\"kind\":\"person\"",
            "\"kind\":\"person\",\"kind\":\"person\"",
        ),
        (
            "\"subject_kind\":\"valve\"",
            "\"subject_kind\":\"valve\",\"subject_kind\":\"valve\"",
        ),
        ("\"locator\":", "\"locator\":\"elsewhere\",\"locator\":"),
        ("\"id\":\"a1\"", "\"id\":\"a1\",\"id\":\"a1\""),
        (
            "\"type\":\"boolean\"",
            "\"type\":\"boolean\",\"type\":\"boolean\"",
        ),
        (
            "\"kind\":\"confidence\"",
            "\"kind\":\"unknown\",\"kind\":\"confidence\"",
        ),
        (
            "\"asserted_by\":",
            "\"asserted_by\":\"observer\",\"asserted_by\":",
        ),
        ("\"event_ms\":", "\"event_ms\":null,\"event_ms\":"),
    ] {
        assert!(original.contains(from), "missing test target {from}");
        let error = Contract::from_json(original.replacen(from, to, 1).as_bytes()).unwrap_err();
        assert!(error.contains("duplicate JSON key"), "{error}");
    }
    // Escaped names are compared after JSON unescaping.
    let escaped = original.replacen("\"v1\":", "\"\\u00761\":{\"kind\":\"valve\"},\"v1\":", 1);
    assert!(Contract::from_json(escaped.as_bytes())
        .unwrap_err()
        .contains("duplicate JSON key"));
}

#[test]
fn duplicate_references_and_assertions_fail_without_deduplication() {
    let mut value = serde_json::to_value(fixture()).unwrap();
    value["assertions"][0]["provenance"]["evidence"] = json!(["e1", "e1"]);
    rejected(value, "duplicate evidence reference");
    let mut value = serde_json::to_value(fixture()).unwrap();
    value["assertions"][1]["id"] = json!("a1");
    rejected(value, "duplicate/empty assertion id");
}

#[test]
fn semantic_validation_runs_during_decode() {
    let cases = [
        ("subject", json!("missing"), "missing subject"),
        ("predicate", json!("missing"), "missing relation"),
        (
            "object",
            json!({"type":"entity","value":"missing"}),
            "missing object entity",
        ),
        (
            "object",
            json!({"type":"text","value":"true"}),
            "type mismatch",
        ),
        (
            "uncertainty",
            json!({"kind":"confidence","permille":1001}),
            "confidence exceeds",
        ),
        (
            "provenance",
            json!({"asserted_by":"missing","method":"reported","evidence":["e1"]}),
            "missing provenance entity",
        ),
        (
            "provenance",
            json!({"asserted_by":"observer","method":"reported","evidence":[]}),
            "missing evidence references",
        ),
        (
            "provenance",
            json!({"asserted_by":"observer","method":"reported","evidence":["missing"]}),
            "dangling evidence reference",
        ),
    ];
    for (field, bad, error) in cases {
        let mut value = serde_json::to_value(fixture()).unwrap();
        value["assertions"][0][field] = bad;
        rejected(value, error);
    }
}

#[test]
fn schema_and_malformed_inputs_fail_closed() {
    let mut value = serde_json::to_value(fixture()).unwrap();
    value["version"] = json!("future");
    rejected(value, "unsupported version");
    for bad in [b"{".as_slice(), b"null", b"[]", b"{}", b"\xff"] {
        assert!(Contract::from_json(bad).is_err());
    }
    assert!(Contract::from_json(format!("{} {{}}", wire()).as_bytes()).is_err());
    for bad in [
        json!({"kind":"certain"}),
        json!({"kind":"confidence","permille":-1}),
        json!({"kind":"confidence","permille":0.5}),
        json!({"kind":"unknown","extra":true}),
    ] {
        let mut value = serde_json::to_value(fixture()).unwrap();
        value["assertions"][0]["uncertainty"] = bad;
        assert!(Contract::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    let mut value = serde_json::to_value(fixture()).unwrap();
    value["assertions"][0]["times"]["record_ms"] = json!(18446744073709551615_u64);
    rejected(value, "invalid value");
}

#[test]
fn unused_relation_kinds_and_malformed_identifiers_are_validated() {
    let mut c = fixture();
    c.relations.insert(
        "unused".into(),
        Relation {
            subject_kind: "ghost".into(),
            object_kind: "boolean".into(),
        },
    );
    assert!(c
        .validate()
        .unwrap_err()
        .contains("undeclared subject kind"));
    c.relations.get_mut("unused").unwrap().subject_kind = "valve".into();
    c.relations.get_mut("unused").unwrap().object_kind = "ghost".into();
    assert!(c.validate().unwrap_err().contains("undeclared object kind"));
    for id in ["", " ", " v1", "v1\n"] {
        let mut c = fixture();
        c.entities.insert(id.into(), c.entities["v1"].clone());
        assert!(c.canonical_json().is_err());
    }
    let mut c = fixture();
    c.entities.get_mut("v1").unwrap().kind = "boolean".into();
    assert!(c.validate().is_err());
}

#[test]
fn evidence_verification_uses_exact_bytes_and_never_locator_identity() {
    let mut c = fixture();
    let original = include_bytes!("../fixtures/observation.txt");
    c.verify_evidence("e1", original).unwrap();
    assert!(c
        .verify_evidence("e1", b"changed")
        .unwrap_err()
        .contains("digest mismatch"));
    assert!(c
        .verify_evidence("missing", original)
        .unwrap_err()
        .contains("missing evidence"));
    c.evidence.get_mut("e1").unwrap().locator = "unfetched:elsewhere".into();
    c.verify_evidence("e1", original).unwrap();
    for digest in ["A".repeat(64), "0".repeat(63), "g".repeat(64)] {
        c.evidence.get_mut("e1").unwrap().sha256 = digest;
        assert!(c.canonical_json().is_err());
    }
}

#[test]
fn roundtrip_preserves_literal_variants_conflicts_unicode_and_time_extremes() {
    let mut c = fixture();
    for (id, kind, object) in [
        ("a3", "text", Object::Text("e\u{301} ≠ é\n\\\"".into())),
        ("a4", "integer", Object::Integer(i64::MIN)),
        ("a5", "integer", Object::Integer(i64::MAX)),
        ("a6", "boolean", Object::Boolean(false)),
    ] {
        c.relations.insert(
            id.into(),
            Relation {
                subject_kind: "valve".into(),
                object_kind: kind.into(),
            },
        );
        let mut a = c.assertions[0].clone();
        a.id = id.into();
        a.predicate = id.into();
        a.object = object;
        a.times.event_ms = None;
        a.times.record_ms = i64::MAX;
        a.times.interpretation_ms = i64::MIN;
        a.uncertainty = Uncertainty::Confidence { permille: 0 };
        c.assertions.push(a);
    }
    let mut conflict = c.assertions[0].clone();
    conflict.id = "a7".into();
    conflict.object = Object::Boolean(false);
    conflict.uncertainty = Uncertainty::Confidence { permille: 1000 };
    c.assertions.push(conflict);
    let canonical = c.canonical_json().unwrap();
    let decoded = Contract::from_json(&canonical).unwrap();
    assert_eq!(decoded, c);
    assert_eq!(decoded.version, SCHEMA_VERSION);
    assert_eq!(decoded.canonical_json().unwrap(), canonical);
    c.assertions.reverse();
    assert_eq!(c.canonical_json().unwrap(), canonical);
}

#[test]
fn unknown_or_missing_nested_fields_are_rejected() {
    for pointer in [
        "",
        "/entities/v1",
        "/relations/open",
        "/evidence/e1",
        "/assertions/0",
        "/assertions/0/object",
        "/assertions/0/provenance",
        "/assertions/0/times",
    ] {
        let mut value = serde_json::to_value(fixture()).unwrap();
        value
            .pointer_mut(pointer)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("extra".into(), json!(true));
        assert!(Contract::from_json(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    let mut value = serde_json::to_value(fixture()).unwrap();
    value["assertions"][0]["times"]
        .as_object_mut()
        .unwrap()
        .remove("record_ms");
    rejected(value.clone(), "missing field");
    value["assertions"][0]["times"]
        .as_object_mut()
        .unwrap()
        .insert("record_ms".into(), json!(0));
    value["assertions"][0]["times"]
        .as_object_mut()
        .unwrap()
        .remove("event_ms");
    rejected(value, "missing field");
}

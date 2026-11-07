use communications_probe::{fixture, Contract, Object, Uncertainty};

#[test]
fn preserves_all_fields_and_verifies_original_bytes() {
    let c = fixture();
    let decoded: Contract = serde_json::from_slice(&c.canonical_json().unwrap()).unwrap();
    assert_eq!(decoded, c);
    assert!(c.evidence["e1"].matches(include_bytes!("../fixtures/observation.txt")));
    assert!(!c.evidence["e1"].matches(b"valve is definitely open"));
}

#[test]
fn order_independent_encoding_does_not_collapse_assertions() {
    let c = fixture();
    let mut other = c.clone();
    other.assertions.reverse();
    assert_eq!(c.canonical_json().unwrap(), other.canonical_json().unwrap());
    other.assertions[0].id = "a1".into();
    assert!(other.validate().is_err());
}

#[test]
fn rejects_dangling_and_mistyped_relationships() {
    let c = fixture();
    let mut bad = c.clone();
    bad.assertions[0].subject = "missing".into();
    assert!(bad.validate().is_err());
    let mut bad = c.clone();
    bad.assertions[0].object = Object::Entity("r1".into());
    assert!(bad.validate().is_err());
    let mut bad = c.clone();
    bad.assertions[1].object = Object::Entity("missing".into());
    assert!(bad.validate().is_err());
    let mut bad = c;
    bad.assertions[0].predicate = "missing".into();
    assert!(bad.validate().is_err());
}

#[test]
fn rejects_invalid_confidence_provenance_and_digest() {
    let c = fixture();
    let mut bad = c.clone();
    bad.assertions[0].uncertainty = Uncertainty::Confidence { permille: 1001 };
    assert!(bad.validate().is_err());
    let mut bad = c.clone();
    bad.assertions[0].provenance.asserted_by = "missing".into();
    assert!(bad.validate().is_err());
    let mut bad = c.clone();
    bad.assertions[0]
        .provenance
        .evidence
        .insert("missing".into());
    assert!(bad.validate().is_err());
    let mut bad = c;
    bad.evidence.get_mut("e1").unwrap().sha256 = "not a digest".into();
    assert!(bad.validate().is_err());
}

#[test]
fn unknown_event_time_and_confidence_remain_unknown() {
    let mut c = fixture();
    c.assertions[0].times.event_ms = None;
    c.assertions[0].uncertainty = Uncertainty::Unknown;
    let decoded: Contract = serde_json::from_slice(&c.canonical_json().unwrap()).unwrap();
    assert_eq!(decoded, c);
    assert_ne!(
        decoded.assertions[0].times.record_ms,
        decoded.assertions[0].times.interpretation_ms
    );
}

#[test]
fn unknown_version_or_fields_fail_closed() {
    let mut c = fixture();
    c.version = "99".into();
    assert!(c.canonical_json().is_err());
    let mut value = serde_json::to_value(fixture()).unwrap();
    value["unrecognized"] = true.into();
    assert!(serde_json::from_value::<Contract>(value).is_err());
}

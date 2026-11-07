use communications_probe::{
    adapter::{self, Arm, Context, MAX_ENVELOPE_BYTES},
    codec::{Encoding, MAX_PAYLOAD_BYTES},
    fixture, Contract, Method, Object, Relation, Uncertainty,
};
use serde_json::{json, Value};

fn all_variants() -> Contract {
    let mut c = fixture();
    for (id, kind, object, method) in [
        (
            "a3",
            "text",
            Object::Text("e\u{301} ≠ é\n\\\"\0".into()),
            Method::Observed,
        ),
        ("a4", "integer", Object::Integer(i64::MIN), Method::Inferred),
        ("a5", "integer", Object::Integer(i64::MAX), Method::Reported),
        ("a6", "boolean", Object::Boolean(false), Method::Observed),
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
        a.provenance.method = method;
        c.assertions.push(a);
    }
    let mut conflict = c.assertions[0].clone();
    conflict.id = "a7".into();
    conflict.object = Object::Boolean(false);
    conflict.uncertainty = Uncertainty::Confidence { permille: 1000 };
    conflict.times.event_ms = Some(i64::MIN);
    c.assertions.push(conflict);
    let mut alias = c.evidence["e1"].clone();
    alias.locator = "unfetched:Unicode/é\n".into();
    c.evidence.insert("e2".into(), alias);
    c.assertions[0].provenance.evidence.insert("e2".into());
    c
}

#[test]
fn every_codec_preserves_exact_fields_variants_extremes_and_canonical_repeatability() {
    let c = all_variants();
    for encoding in Encoding::ALL {
        let bytes = encoding.encode(&c).unwrap();
        assert_eq!(encoding.decode(&bytes).unwrap(), c);
        assert_eq!(
            encoding.encode(&encoding.decode(&bytes).unwrap()).unwrap(),
            bytes
        );
        let mut reversed = c.clone();
        reversed.assertions.reverse();
        assert_eq!(encoding.encode(&reversed).unwrap(), bytes);
        // No evidence retrieval; the digest continues to match exact original bytes.
        encoding
            .decode(&bytes)
            .unwrap()
            .verify_evidence("e1", include_bytes!("../fixtures/observation.txt"))
            .unwrap();
    }
}

#[test]
fn all_offline_contexts_and_matched_baselines_preserve_same_complete_facts() {
    let c = all_variants();
    for context in Context::OFFLINE {
        assert!(context.capability().offline_roundtrip);
        assert!(!context.capability().live_endpoint);
        for arm in [
            Arm::Conventional,
            Arm::Semantic(Encoding::Json),
            Arm::Semantic(Encoding::Cbor),
            Arm::Semantic(Encoding::Records),
        ] {
            let envelope = adapter::render(context, arm, &c).unwrap();
            assert!(std::str::from_utf8(&envelope).is_ok());
            let recovered = adapter::parse(context, arm, &envelope).unwrap();
            assert_eq!(recovered, c, "{context:?} {arm:?}");
            assert_eq!(adapter::render(context, arm, &recovered).unwrap(), envelope);
            assert!(envelope.len() > arm.payload(context, &c).unwrap().len());
        }
    }
}

#[test]
fn smoke_reference_content_is_not_a_renderers_shared_omission() {
    // Existing reference fixture is independent of every codec/adapter renderer.
    let expected: Value =
        serde_json::from_slice(include_bytes!("../fixtures/contract.json")).unwrap();
    assert_eq!(
        expected["assertions"][0]["provenance"]["method"],
        "reported"
    );
    assert_eq!(expected["assertions"][0]["uncertainty"]["permille"], 800);
    assert_eq!(
        expected["assertions"][0]["times"]["event_ms"],
        1700000000000_i64
    );
    assert_eq!(
        expected["assertions"][0]["times"]["record_ms"],
        1700000001000_i64
    );
    assert_eq!(
        expected["assertions"][0]["times"]["interpretation_ms"],
        1700000002000_i64
    );
    for context in Context::OFFLINE {
        for arm in [
            Arm::Conventional,
            Arm::Semantic(Encoding::Json),
            Arm::Semantic(Encoding::Cbor),
            Arm::Semantic(Encoding::Records),
        ] {
            let decoded = adapter::parse(
                context,
                arm,
                &adapter::render(context, arm, &fixture()).unwrap(),
            )
            .unwrap();
            assert_eq!(serde_json::to_value(decoded).unwrap(), expected);
        }
    }
}

#[test]
fn every_truncation_trailing_bytes_and_oversize_payload_fail() {
    for encoding in Encoding::ALL {
        let bytes = encoding.encode(&fixture()).unwrap();
        for end in 0..bytes.len() {
            assert!(
                encoding.decode(&bytes[..end]).is_err(),
                "{encoding:?} end={end}"
            );
        }
        let mut trailing = bytes;
        trailing.push(0);
        assert!(encoding.decode(&trailing).is_err());
        assert!(encoding
            .decode(&vec![0; MAX_PAYLOAD_BYTES + 1])
            .unwrap_err()
            .contains("limit"));
    }
}

fn cbor(value: &impl serde::Serialize) -> Vec<u8> {
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(value, &mut bytes).unwrap();
    bytes
}

#[test]
fn cbor_is_real_named_field_cbor_and_rejects_tags_indefinite_nonpreferred_and_duplicate_keys() {
    use ciborium::value::Value as C;
    let bytes = Encoding::Cbor.encode(&fixture()).unwrap();
    // Independent known CBOR prefix: map(5), text(7) "version", text(9) "0.1-draft".
    assert_eq!(&bytes[..19], b"\xa5\x67version\x690.1-draft");
    let value: C = ciborium::de::from_reader(bytes.as_slice()).unwrap();
    let C::Map(entries) = value else {
        panic!("not CBOR map")
    };
    let mut duplicate = entries.clone();
    duplicate.push(entries[0].clone());
    assert!(Encoding::Cbor
        .decode(&cbor(&C::Map(duplicate)))
        .unwrap_err()
        .contains("duplicate"));
    let mut indefinite = bytes.clone();
    indefinite[0] = 0xbf;
    indefinite.push(0xff);
    assert!(Encoding::Cbor.decode(&indefinite).is_err());
    let mut long_map = vec![0xb8, 5];
    long_map.extend_from_slice(&bytes[1..]);
    assert!(Encoding::Cbor.decode(&long_map).is_err());
    let mut tagged = vec![0xc0];
    tagged.extend_from_slice(&bytes);
    assert!(Encoding::Cbor.decode(&tagged).is_err());
    let mut wrong_key = entries;
    wrong_key[0].0 = C::Integer(0.into());
    assert!(Encoding::Cbor.decode(&cbor(&C::Map(wrong_key))).is_err());
    assert!(Encoding::Cbor.decode(&[0x9f; 300]).is_err());
}

#[test]
fn cbor_rejects_unknown_duplicate_nested_keys_refs_and_invalid_contracts() {
    use ciborium::value::Value as C;
    let value: C =
        ciborium::de::from_reader(Encoding::Cbor.encode(&fixture()).unwrap().as_slice()).unwrap();
    fn duplicate_at(v: &mut C, path: &[&str]) {
        if let C::Map(entries) = v {
            if path.is_empty() {
                entries.push(entries[0].clone());
            } else {
                duplicate_at(
                    &mut entries
                        .iter_mut()
                        .find(|(k, _)| k == &C::Text(path[0].into()))
                        .unwrap()
                        .1,
                    &path[1..],
                );
            }
        } else if let C::Array(entries) = v {
            duplicate_at(&mut entries[0], path);
        } else {
            panic!("bad test path")
        }
    }
    for path in [
        vec![],
        vec!["entities"],
        vec!["entities", "v1"],
        vec!["relations", "open"],
        vec!["evidence", "e1"],
        vec!["assertions"],
        vec!["assertions", "object"],
        vec!["assertions", "provenance"],
        vec!["assertions", "uncertainty"],
        vec!["assertions", "times"],
    ] {
        let mut duplicate = value.clone();
        duplicate_at(&mut duplicate, &path);
        assert!(
            Encoding::Cbor
                .decode(&cbor(&duplicate))
                .unwrap_err()
                .contains("duplicate"),
            "{path:?}"
        );
    }
    for (pointer, bad, error) in [
        ("/version", json!("future"), "unsupported version"),
        ("/assertions/0/subject", json!("missing"), "missing subject"),
        (
            "/assertions/0/uncertainty/permille",
            json!(1001),
            "confidence exceeds",
        ),
        (
            "/assertions/0/provenance/evidence",
            json!(["e1", "e1"]),
            "duplicate evidence",
        ),
        (
            "/assertions/0/provenance/method",
            json!("invented"),
            "unknown variant",
        ),
        ("/assertions/0/times/event_ms", json!(0.5), "integer"),
    ] {
        let mut bad_value = serde_json::to_value(fixture()).unwrap();
        *bad_value.pointer_mut(pointer).unwrap() = bad;
        let err = Encoding::Cbor.decode(&cbor(&bad_value)).unwrap_err();
        assert!(err.contains(error), "{err} expected {error}");
    }
    for path in [
        "",
        "/entities/v1",
        "/relations/open",
        "/evidence/e1",
        "/assertions/0",
        "/assertions/0/object",
        "/assertions/0/uncertainty",
        "/assertions/0/provenance",
        "/assertions/0/times",
    ] {
        let mut bad = serde_json::to_value(fixture()).unwrap();
        bad.pointer_mut(path)
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert("extra".into(), json!(true));
        let error = Encoding::Cbor.decode(&cbor(&bad)).unwrap_err();
        assert!(error.contains("extra"), "{path}: {error}");
    }
}

#[test]
fn invalid_in_memory_contracts_cannot_be_encoded_or_rendered() {
    let c = fixture();
    let mut invalids = Vec::new();
    let mut bad = c.clone();
    bad.assertions[0].provenance.evidence.clear();
    invalids.push(bad);
    let mut bad = c.clone();
    bad.assertions.push(c.assertions[0].clone());
    invalids.push(bad);
    let mut bad = c.clone();
    bad.assertions[0].uncertainty = Uncertainty::Confidence { permille: 1001 };
    invalids.push(bad);
    let mut bad = c;
    bad.version = "future".into();
    invalids.push(bad);
    for bad in invalids {
        for encoding in Encoding::ALL {
            assert!(encoding.encode(&bad).is_err());
        }
        for context in Context::OFFLINE {
            for arm in [
                Arm::Conventional,
                Arm::Semantic(Encoding::Json),
                Arm::Semantic(Encoding::Cbor),
                Arm::Semantic(Encoding::Records),
            ] {
                assert!(adapter::render(context, arm, &bad).is_err());
            }
        }
    }
}

#[test]
fn records_enforce_framing_versions_bounds_unknown_tags_and_unique_ids() {
    let bytes = Encoding::Records.encode(&fixture()).unwrap();
    assert_eq!(&bytes[..4], b"CSR\x01");
    let mut bad = bytes.clone();
    bad[3] = 2;
    assert!(Encoding::Records
        .decode(&bad)
        .unwrap_err()
        .contains("framing/version"));
    let mut bad = bytes.clone();
    bad[8] = b'9';
    assert!(Encoding::Records
        .decode(&bad)
        .unwrap_err()
        .contains("unsupported version"));
    let mut bad = bytes.clone();
    bad[4..8].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(Encoding::Records
        .decode(&bad)
        .unwrap_err()
        .contains("limit"));
    // Offset after magic + length-prefixed "0.1-draft" is first section count.
    let count = 4 + 4 + "0.1-draft".len();
    let mut bad = bytes.clone();
    bad[count..count + 4].copy_from_slice(&4097_u32.to_le_bytes());
    assert!(Encoding::Records
        .decode(&bad)
        .unwrap_err()
        .contains("limit"));
    let mut bad = bytes.clone();
    bad[count + 4] = 0xff;
    assert!(Encoding::Records.decode(&bad).unwrap_err().contains("tag"));
    let mut bad = bytes.clone();
    bad[count + 4 + 1 + 4] = 0xff;
    assert!(Encoding::Records.decode(&bad).is_err());
    // Duplicate the complete first entity (observer/person) before the next.
    let first = count + 4;
    let end = first + 1 + 4 + "observer".len() + 4 + "person".len();
    let mut duplicate = bytes.clone();
    duplicate.splice(end..end, bytes[first..end].iter().copied());
    duplicate[count..count + 4].copy_from_slice(&4_u32.to_le_bytes());
    assert!(Encoding::Records
        .decode(&duplicate)
        .unwrap_err()
        .contains("duplicate record"));
    let mut c = fixture();
    c.evidence.get_mut("e1").unwrap().locator = "x".repeat(65536);
    assert_eq!(
        Encoding::Records
            .decode(&Encoding::Records.encode(&c).unwrap())
            .unwrap(),
        c
    );
    c.evidence.get_mut("e1").unwrap().locator.push('x');
    assert!(Encoding::Records.encode(&c).unwrap_err().contains("65536"));
    let mut c = fixture();
    for n in 0..4093 {
        c.entities
            .insert(format!("extra{n}"), c.entities["v1"].clone());
    }
    assert_eq!(
        Encoding::Records
            .decode(&Encoding::Records.encode(&c).unwrap())
            .unwrap(),
        c
    );
    c.entities
        .insert("one_too_many".into(), c.entities["v1"].clone());
    assert!(Encoding::Records.encode(&c).unwrap_err().contains("4096"));
}

#[test]
fn adapters_reject_ambiguous_metadata_extra_fields_and_corrupted_payloads() {
    for context in Context::OFFLINE {
        for arm in [
            Arm::Conventional,
            Arm::Semantic(Encoding::Json),
            Arm::Semantic(Encoding::Cbor),
            Arm::Semantic(Encoding::Records),
        ] {
            let bytes = adapter::render(context, arm, &fixture()).unwrap();
            assert!(adapter::parse(context, arm, &bytes[..bytes.len() - 1]).is_err());
            let mut value: Value = serde_json::from_slice(&bytes).unwrap();
            value["extra"] = json!(true);
            assert!(adapter::parse(context, arm, &serde_json::to_vec(&value).unwrap()).is_err());
            let text = String::from_utf8(bytes).unwrap();
            let duplicate = text.replacen('{', "{\"extra\":0,\"extra\":1,", 1);
            assert!(adapter::parse(context, arm, duplicate.as_bytes())
                .unwrap_err()
                .contains("duplicate"));
        }
    }
    for encoding in Encoding::ALL {
        let arm = Arm::Semantic(encoding);
        let bytes = adapter::render(Context::ModelTool, arm, &fixture()).unwrap();
        for (field, bad) in [
            ("adapter_version", json!(2)),
            ("encoding", json!("future")),
            ("transfer", json!("tensor")),
            ("schema", json!("")),
            ("payload", json!("%%%")),
        ] {
            let mut v: Value = serde_json::from_slice(&bytes).unwrap();
            v["params"]["arguments"][field] = bad;
            assert!(
                adapter::parse(Context::ModelTool, arm, &serde_json::to_vec(&v).unwrap()).is_err()
            );
        }
        assert!(adapter::parse(Context::ModelData, arm, &bytes).is_err());
        assert!(adapter::parse(Context::ModelTool, Arm::Conventional, &bytes).is_err());
    }
    let mut v: Value = serde_json::from_slice(
        &adapter::render(Context::ModelTool, Arm::Conventional, &fixture()).unwrap(),
    )
    .unwrap();
    v["params"]["arguments"]["assertions"][0]["provenance"] = json!({});
    assert!(adapter::parse(
        Context::ModelTool,
        Arm::Conventional,
        &serde_json::to_vec(&v).unwrap()
    )
    .is_err());
    assert!(adapter::parse(
        Context::HumanModel,
        Arm::Conventional,
        &vec![b' '; MAX_ENVELOPE_BYTES + 1]
    )
    .unwrap_err()
    .contains("limit"));
}

#[test]
fn model_native_is_explicitly_unsupported_for_all_arms() {
    assert!(!Context::ModelNative.capability().offline_roundtrip);
    for arm in [
        Arm::Conventional,
        Arm::Semantic(Encoding::Json),
        Arm::Semantic(Encoding::Cbor),
        Arm::Semantic(Encoding::Records),
    ] {
        assert!(adapter::render(Context::ModelNative, arm, &fixture())
            .unwrap_err()
            .contains("Unsupported/not testable"));
        assert!(adapter::parse(Context::ModelNative, arm, b"{}")
            .unwrap_err()
            .contains("Unsupported/not testable"));
    }
}

#[test]
fn typed_record_documented_bytes_and_scalar_validation() {
    let mut c = fixture();
    c.entities.remove("r1");
    c.relations.remove("located_in");
    c.assertions.truncate(1);
    // An independent literal construction of the documented CSR/1 wire layout.
    fn text(b: &mut Vec<u8>, s: &str) {
        b.extend_from_slice(&(s.len() as u32).to_le_bytes());
        b.extend_from_slice(s.as_bytes());
    }
    fn count(b: &mut Vec<u8>, n: u32) {
        b.extend_from_slice(&n.to_le_bytes());
    }
    let mut expected = b"CSR\x01".to_vec();
    text(&mut expected, "0.1-draft");
    count(&mut expected, 2);
    expected.push(0x10);
    text(&mut expected, "observer");
    text(&mut expected, "person");
    expected.push(0x10);
    text(&mut expected, "v1");
    text(&mut expected, "valve");
    count(&mut expected, 1);
    expected.push(0x11);
    text(&mut expected, "open");
    text(&mut expected, "valve");
    text(&mut expected, "boolean");
    count(&mut expected, 1);
    expected.push(0x12);
    text(&mut expected, "e1");
    expected.extend_from_slice(&[
        0x4e, 0xcc, 0x52, 0xe6, 0xee, 0x2c, 0x4e, 0x8d, 0xfc, 0x30, 0x35, 0xee, 0x23, 0x6e, 0xb8,
        0xb1, 0x47, 0x6b, 0xbb, 0x8b, 0x83, 0x6d, 0x83, 0x4f, 0xec, 0xb5, 0x32, 0xda, 0xc6, 0xb3,
        0x79, 0x89,
    ]);
    text(&mut expected, "fixtures/observation.txt");
    let assertion_count = expected.len();
    count(&mut expected, 1);
    let assertion_start = expected.len();
    expected.push(0x13);
    text(&mut expected, "a1");
    text(&mut expected, "v1");
    text(&mut expected, "open");
    let object = expected.len();
    expected.extend_from_slice(&[0x21, 1]);
    let uncertainty = expected.len();
    expected.push(1);
    expected.extend_from_slice(&800_u16.to_le_bytes());
    text(&mut expected, "observer");
    let method = expected.len();
    expected.push(1);
    let refs = expected.len();
    count(&mut expected, 1);
    text(&mut expected, "e1");
    let event = expected.len();
    expected.push(1);
    expected.extend_from_slice(&1700000000000_i64.to_le_bytes());
    expected.extend_from_slice(&1700000001000_i64.to_le_bytes());
    expected.extend_from_slice(&1700000002000_i64.to_le_bytes());
    assert_eq!(Encoding::Records.encode(&c).unwrap(), expected);
    assert_eq!(Encoding::Records.decode(&expected).unwrap(), c);
    for (offset, bad) in [
        (object, 0xff),
        (object + 1, 2),
        (uncertainty, 2),
        (method, 3),
        (event, 2),
    ] {
        let mut bytes = expected.clone();
        bytes[offset] = bad;
        assert!(Encoding::Records.decode(&bytes).is_err(), "offset {offset}");
    }
    let mut bad = expected.clone();
    bad[uncertainty + 1..uncertainty + 3].copy_from_slice(&1001_u16.to_le_bytes());
    assert!(Encoding::Records
        .decode(&bad)
        .unwrap_err()
        .contains("confidence exceeds"));
    let mut bad = expected.clone();
    let mut extra_ref = Vec::new();
    text(&mut extra_ref, "e1");
    bad.splice(event..event, extra_ref);
    bad[refs..refs + 4].copy_from_slice(&2_u32.to_le_bytes());
    assert!(Encoding::Records
        .decode(&bad)
        .unwrap_err()
        .contains("duplicate evidence"));
    let mut duplicate_assertion = expected.clone();
    duplicate_assertion.extend_from_slice(&expected[assertion_start..]);
    duplicate_assertion[assertion_count..assertion_count + 4].copy_from_slice(&2_u32.to_le_bytes());
    assert!(Encoding::Records
        .decode(&duplicate_assertion)
        .unwrap_err()
        .contains("duplicate/empty assertion"));
}

#[test]
fn adapter_nested_ambiguity_and_envelope_identity_fail_closed() {
    let arm = Arm::Semantic(Encoding::Json);
    let encoded = adapter::render(Context::ModelTool, arm, &fixture()).unwrap();
    let original: Value = serde_json::from_slice(&encoded).unwrap();
    for (pointer, bad) in [
        ("/id", json!(2)),
        ("/method", json!("other")),
        ("/jsonrpc", json!("1.0")),
        ("/params/name", json!("other")),
    ] {
        let mut v = original.clone();
        *v.pointer_mut(pointer).unwrap() = bad;
        assert!(adapter::parse(Context::ModelTool, arm, &serde_json::to_vec(&v).unwrap()).is_err());
    }
    let mut v = original;
    let payload = v["params"]["arguments"]["payload"]
        .as_str()
        .unwrap()
        .replacen("\"event_ms\":", "\"event_ms\":null,\"event_ms\":", 1);
    v["params"]["arguments"]["payload"] = json!(payload);
    assert!(
        adapter::parse(Context::ModelTool, arm, &serde_json::to_vec(&v).unwrap())
            .unwrap_err()
            .contains("duplicate")
    );
    let bytes = adapter::render(Context::HumanModel, Arm::Conventional, &fixture()).unwrap();
    let mut v: Value = serde_json::from_slice(&bytes).unwrap();
    let text = v["messages"][0]["content"]
        .as_str()
        .unwrap()
        .replace("\"event_ms\":", "\"event_ms\":null,\"event_ms\":");
    v["messages"][0]["content"] = json!(text);
    assert!(adapter::parse(
        Context::HumanModel,
        Arm::Conventional,
        &serde_json::to_vec(&v).unwrap()
    )
    .unwrap_err()
    .contains("duplicate"));
    let mut v: Value = serde_json::from_slice(&bytes).unwrap();
    let extra = v["messages"][0].clone();
    v["messages"].as_array_mut().unwrap().push(extra);
    assert!(adapter::parse(
        Context::HumanModel,
        Arm::Conventional,
        &serde_json::to_vec(&v).unwrap()
    )
    .is_err());
}

//! Lossless JSON ingress: reject duplicates before constructing maps or tagged enums.
use crate::{Assertion, Contract, Entity, Evidence, Relation};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireContract {
    version: String,
    entities: BTreeMap<String, Entity>,
    relations: BTreeMap<String, Relation>,
    evidence: BTreeMap<String, Evidence>,
    assertions: Vec<Assertion>,
}

impl<'de> Deserialize<'de> for Contract {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = UniqueValue::deserialize(deserializer)?;
        let raw: WireContract = serde_json::from_value(value.0).map_err(de::Error::custom)?;
        let contract = Self {
            version: raw.version,
            entities: raw.entities,
            relations: raw.relations,
            evidence: raw.evidence,
            assertions: raw.assertions,
        };
        contract.validate().map_err(de::Error::custom)?;
        Ok(contract)
    }
}

pub(crate) fn unique_references<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeSet<String>, D::Error> {
    let references = Vec::<String>::deserialize(deserializer)?;
    let mut result = BTreeSet::new();
    for reference in references {
        if !result.insert(reference) {
            return Err(de::Error::custom("duplicate evidence reference"));
        }
    }
    Ok(result)
}

pub(crate) struct UniqueValue(pub(crate) Value);

impl<'de> Deserialize<'de> for UniqueValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(UniqueVisitor)
    }
}

struct UniqueVisitor;
impl<'de> Visitor<'de> for UniqueVisitor {
    type Value = UniqueValue;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("contract JSON with unique object keys and integer numbers")
    }

    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut values = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate JSON key: {key}")));
            }
            values.insert(key, map.next_value::<UniqueValue>()?.0);
        }
        Ok(UniqueValue(Value::Object(values)))
    }

    fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element::<UniqueValue>()? {
            values.push(value.0);
        }
        Ok(UniqueValue(Value::Array(values)))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::String(value.into())))
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Bool(value)))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(value.into())))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Number(value.into())))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }

    // CBOR's null enters through visit_none; JSON null uses visit_unit.
    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        self.visit_unit()
    }
}

// An empty struct variant rejects extra fields; Serde's internally tagged unit
// variant otherwise ignores them even with deny_unknown_fields.
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum WireUncertainty {
    Unknown {},
    Confidence { permille: u16 },
}

impl From<WireUncertainty> for crate::Uncertainty {
    fn from(value: WireUncertainty) -> Self {
        match value {
            WireUncertainty::Unknown {} => Self::Unknown,
            WireUncertainty::Confidence { permille } => Self::Confidence { permille },
        }
    }
}

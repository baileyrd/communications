//! Replay-only JSON ingress permits finite fractional telemetry/settings.
//! Semantic contracts and scored responses retain their integer-only ingress.
use serde::{
    de::{self, DeserializeOwned, MapAccess, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use serde_json::Value;
use std::fmt;

pub(super) fn from_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    let value: ReplayValue = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    serde_json::from_value(value.0).map_err(|e| e.to_string())
}

struct ReplayValue(Value);
impl<'de> Deserialize<'de> for ReplayValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(ReplayVisitor)
    }
}
struct ReplayVisitor;
impl<'de> Visitor<'de> for ReplayVisitor {
    type Value = ReplayValue;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("replay JSON with unique object keys and finite numbers")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut values = serde_json::Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom(format!("duplicate JSON key: {key}")));
            }
            values.insert(key, map.next_value::<ReplayValue>()?.0);
        }
        Ok(ReplayValue(Value::Object(values)))
    }
    fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
        let mut values = Vec::new();
        while let Some(value) = seq.next_element::<ReplayValue>()? {
            values.push(value.0);
        }
        Ok(ReplayValue(Value::Array(values)))
    }
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(ReplayValue(Value::String(value.into())))
    }
    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(ReplayValue(Value::Bool(value)))
    }
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(ReplayValue(Value::Number(value.into())))
    }
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(ReplayValue(Value::Number(value.into())))
    }
    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
        serde_json::Number::from_f64(value)
            .map(|number| ReplayValue(Value::Number(number)))
            .ok_or_else(|| E::custom("nonfinite replay number"))
    }
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(ReplayValue(Value::Null))
    }
}

//! Validated, bounded transport boundary. See docs/WIRE-FORMATS.md.
mod records;

use crate::Contract;
use serde::{Deserialize, Serialize};

pub const MAX_PAYLOAD_BYTES: usize = 1_048_576;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Encoding {
    Json,
    Cbor,
    Records,
}

impl Encoding {
    pub const ALL: [Self; 3] = [Self::Json, Self::Cbor, Self::Records];

    pub fn encode(self, contract: &Contract) -> Result<Vec<u8>, String> {
        contract.validate()?;
        let bytes = match self {
            Self::Json => contract.canonical_json()?,
            Self::Cbor => {
                let mut sorted = contract.clone();
                sorted.assertions.sort_by(|a, b| a.id.cmp(&b.id));
                let mut bytes = Vec::new();
                ciborium::ser::into_writer(&sorted, &mut bytes).map_err(|e| e.to_string())?;
                bytes
            }
            Self::Records => records::encode(contract)?,
        };
        bound(bytes.len())?;
        Ok(bytes)
    }

    /// Exactly one payload. CBOR and records accept only this probe's emitted
    /// ordered profile; alternatives are rejected, never silently normalized.
    pub fn decode(self, bytes: &[u8]) -> Result<Contract, String> {
        bound(bytes.len())?;
        let contract = match self {
            Self::Json => Contract::from_json(bytes)?,
            Self::Cbor => {
                let mut remaining = bytes;
                let contract: Contract =
                    ciborium::de::from_reader_with_recursion_limit(&mut remaining, 32)
                        .map_err(|e| e.to_string())?;
                if !remaining.is_empty() {
                    return Err("trailing CBOR bytes".into());
                }
                contract
            }
            Self::Records => records::decode(bytes)?,
        };
        contract.validate()?;
        if self != Self::Json && self.encode(&contract)? != bytes {
            return Err("noncanonical probe profile".into());
        }
        Ok(contract)
    }
}

pub(crate) fn bound(len: usize) -> Result<(), String> {
    if len > MAX_PAYLOAD_BYTES {
        Err("payload exceeds 1 MiB limit".into())
    } else {
        Ok(())
    }
}

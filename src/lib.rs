//! Minimal versioned semantic contract, independent of transport and model adapters.
pub mod adapter;
pub mod codec;
mod decode;
pub mod golden;
pub mod harness;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const SCHEMA_VERSION: &str = "0.1-draft";

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Contract {
    pub version: String,
    pub entities: BTreeMap<String, Entity>,
    pub relations: BTreeMap<String, Relation>,
    pub evidence: BTreeMap<String, Evidence>,
    pub assertions: Vec<Assertion>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    pub kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relation {
    pub subject_kind: String,
    /// "boolean", "text", "integer", or the target entity kind.
    pub object_kind: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub sha256: String,
    /// A retrieval hint, never proof that retrieved bytes match the digest.
    pub locator: String,
}

impl Evidence {
    /// Content verification is separate from validating a reference's shape.
    pub fn matches(&self, bytes: &[u8]) -> bool {
        format!("{:x}", Sha256::digest(bytes)) == self.sha256
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assertion {
    pub id: String,
    pub subject: String,
    pub predicate: String,
    pub object: Object,
    pub uncertainty: Uncertainty,
    pub provenance: Provenance,
    pub times: Times,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Object {
    Entity(String),
    Boolean(bool),
    Text(String),
    Integer(i64),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    from = "decode::WireUncertainty"
)]
pub enum Uncertainty {
    Unknown,
    /// Source-reported confidence, not a calibrated probability.
    Confidence {
        permille: u16,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub asserted_by: String,
    pub method: Method,
    #[serde(deserialize_with = "decode::unique_references")]
    pub evidence: BTreeSet<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    Observed,
    Reported,
    Inferred,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Times {
    /// UTC epoch milliseconds. null means unknown, not record time.
    #[serde(deserialize_with = "Option::<i64>::deserialize")]
    pub event_ms: Option<i64>,
    pub record_ms: i64,
    pub interpretation_ms: i64,
}

fn identifier(value: &str) -> bool {
    !value.is_empty() && value.trim() == value && !value.chars().any(char::is_control)
}

fn require(ok: bool, message: String) -> Result<(), String> {
    if ok {
        Ok(())
    } else {
        Err(message)
    }
}

impl Contract {
    pub fn validate(&self) -> Result<(), String> {
        require(self.version == SCHEMA_VERSION, "unsupported version".into())?;
        require(!self.assertions.is_empty(), "no assertions".into())?;
        for (id, e) in &self.entities {
            require(
                identifier(id) && identifier(&e.kind),
                "invalid entity identifier/kind".into(),
            )?;
            require(
                !["boolean", "text", "integer"].contains(&e.kind.as_str()),
                "entity kind collides with literal kind".into(),
            )?;
        }
        let kinds: BTreeSet<_> = self.entities.values().map(|e| e.kind.as_str()).collect();
        for (id, r) in &self.relations {
            require(
                identifier(id) && identifier(&r.subject_kind) && identifier(&r.object_kind),
                "invalid relation field".into(),
            )?;
            require(
                kinds.contains(r.subject_kind.as_str()),
                format!("undeclared subject kind in {id}"),
            )?;
            require(
                kinds.contains(r.object_kind.as_str())
                    || ["boolean", "text", "integer"].contains(&r.object_kind.as_str()),
                format!("undeclared object kind in {id}"),
            )?;
        }
        for (id, e) in &self.evidence {
            require(
                identifier(id) && !e.locator.trim().is_empty(),
                "empty evidence identifier/locator".into(),
            )?;
            require(
                e.sha256.len() == 64
                    && e.sha256
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
                "invalid SHA-256 reference".into(),
            )?;
        }
        let mut ids = BTreeSet::new();
        for a in &self.assertions {
            require(
                identifier(&a.id) && ids.insert(&a.id),
                "duplicate/empty assertion id".into(),
            )?;
            let subject = self.entities.get(&a.subject).ok_or("missing subject")?;
            let relation = self.relations.get(&a.predicate).ok_or("missing relation")?;
            let object_kind = match &a.object {
                Object::Entity(id) => &self.entities.get(id).ok_or("missing object entity")?.kind,
                Object::Boolean(_) => "boolean",
                Object::Text(_) => "text",
                Object::Integer(_) => "integer",
            };
            require(
                subject.kind == relation.subject_kind && object_kind == relation.object_kind,
                format!("type mismatch in {}", a.id),
            )?;
            if let Uncertainty::Confidence { permille } = a.uncertainty {
                require(permille <= 1000, "confidence exceeds 1000".into())?;
            }
            require(
                self.entities.contains_key(&a.provenance.asserted_by),
                "missing provenance entity".into(),
            )?;
            require(
                !a.provenance.evidence.is_empty(),
                "missing evidence references".into(),
            )?;
            for id in &a.provenance.evidence {
                require(
                    self.evidence.contains_key(id),
                    "dangling evidence reference".into(),
                )?;
            }
        }
        Ok(())
    }

    /// Decode exactly one JSON document and validate before exposing the contract.
    /// Duplicate object keys and evidence references fail rather than losing data.
    pub fn from_json(bytes: &[u8]) -> Result<Self, String> {
        serde_json::from_slice(bytes).map_err(|e| e.to_string())
    }

    /// Verify caller-supplied original bytes. This never fetches a locator.
    pub fn verify_evidence(&self, id: &str, bytes: &[u8]) -> Result<(), String> {
        let evidence = self.evidence.get(id).ok_or("missing evidence")?;
        require(
            evidence.matches(bytes),
            format!("evidence digest mismatch for {id}"),
        )
    }

    /// Stable contract encoding: sorted maps, assertion ids, and evidence-reference sets.
    /// This is deliberately not a claim of RFC 8785 or cross-format canonicalization.
    pub fn canonical_json(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        let mut sorted = self.clone();
        sorted.assertions.sort_by(|a, b| a.id.cmp(&b.id));
        serde_json::to_vec(&sorted).map_err(|e| e.to_string())
    }
}

pub fn fixture() -> Contract {
    Contract::from_json(include_bytes!("../fixtures/contract.json"))
        .expect("checked-in smoke fixture")
}

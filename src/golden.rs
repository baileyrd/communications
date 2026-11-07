//! Offline golden references and exact structured scoring. No model clients.
use crate::{
    decode::UniqueValue, Assertion, Contract, Method, Object, Provenance, Times, Uncertainty,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const VERSION: u32 = 1;
pub const SUITE_BYTES: &[u8] = include_bytes!("../fixtures/golden/suite.json");
pub const RESPONSE_SCHEMA: &str = include_str!("../docs/GOLDEN-RESPONSE-SCHEMA.txt");
pub const INSTRUCTIONS: &str = "Reconstruct every supplied assertion using its original id and exact typed fields. Preserve conflicting reports independently; do not resolve them. Answer the comprehension and application questions using only supplied facts. Use the structured response schema; unknown answers are explicit null. Do not fetch evidence locators. Return one JSON object only.";

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Duplicate keys fail before typed deserialization, including answer objects.
pub fn from_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    let value: UniqueValue = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    serde_json::from_value(value.0).map_err(|e| e.to_string())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Suite {
    pub version: u32,
    pub cases: Vec<Case>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub id: String,
    pub source_sha256: String,
    pub facts: Vec<Fact>,
    pub entity_kinds: BTreeMap<String, String>,
    pub relation_kinds: BTreeMap<String, [String; 2]>,
    pub questions: Vec<Question>,
}

/// Hand-authored source ledger, with field names distinct from wire extraction.
/// Never generated from a codec, adapter, or a model response.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Fact {
    pub id: String,
    pub subject: String,
    pub relation: String,
    pub value: Object,
    pub by: String,
    pub method: Method,
    pub evidence: BTreeSet<String>,
    #[serde(deserialize_with = "Option::<i64>::deserialize")]
    pub event_ms: Option<i64>,
    pub recorded_ms: i64,
    pub interpreted_ms: i64,
    #[serde(deserialize_with = "Option::<u16>::deserialize")]
    pub confidence_permille: Option<u16>,
}

impl Fact {
    fn matches(&self, a: &Assertion) -> bool {
        self.id == a.id
            && self.subject == a.subject
            && self.relation == a.predicate
            && self.value == a.object
            && self.by == a.provenance.asserted_by
            && self.method == a.provenance.method
            && self.evidence == a.provenance.evidence
            && self.times() == a.times
            && self.uncertainty() == a.uncertainty
    }
    fn times(&self) -> Times {
        Times {
            event_ms: self.event_ms,
            record_ms: self.recorded_ms,
            interpretation_ms: self.interpreted_ms,
        }
    }
    fn uncertainty(&self) -> Uncertainty {
        self.confidence_permille
            .map_or(Uncertainty::Unknown, |permille| Uncertainty::Confidence {
                permille,
            })
    }
    fn provenance(&self) -> Provenance {
        Provenance {
            asserted_by: self.by.clone(),
            method: self.method.clone(),
            evidence: self.evidence.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Question {
    pub id: String,
    pub task: Task,
    pub prompt: String,
    pub expected: Value,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Task {
    Comprehension,
    Application,
}

pub struct LoadedCase {
    pub reference: Case,
    pub contract: Contract,
    pub contract_sha256: String,
    pub source: &'static [u8],
}

fn files(id: &str) -> Result<(&'static [u8], &'static [u8]), String> {
    macro_rules! file {
        ($id:literal) => {
            (
                include_bytes!(concat!("../fixtures/golden/", $id, ".txt")).as_slice(),
                include_bytes!(concat!("../fixtures/golden/", $id, ".contract.json")).as_slice(),
            )
        };
    }
    Ok(match id {
        "conflict" => file!("conflict"),
        "unknown" => file!("unknown"),
        "ordering" => file!("ordering"),
        "provenance" => file!("provenance"),
        "relocation" => file!("relocation"),
        "quoted" => file!("quoted"),
        _ => return Err("unknown compiled fixture".into()),
    })
}

pub fn load() -> Result<Vec<LoadedCase>, String> {
    let suite: Suite = from_json(SUITE_BYTES)?;
    if suite.version != VERSION || suite.cases.is_empty() {
        return Err("unsupported/empty golden suite".into());
    }
    let mut ids = BTreeSet::new();
    suite
        .cases
        .into_iter()
        .map(|reference| {
            if !ids.insert(reference.id.clone()) {
                return Err("duplicate case id".into());
            }
            let (source, bytes) = files(&reference.id)?;
            if digest(source) != reference.source_sha256 {
                return Err(format!("source digest mismatch: {}", reference.id));
            }
            let contract = Contract::from_json(bytes)?;
            contract.verify_evidence("e1", source)?;
            reference.validate_contract(&contract)?;
            Ok(LoadedCase {
                reference,
                contract,
                contract_sha256: digest(bytes),
                source,
            })
        })
        .collect()
}

impl Case {
    /// An omitted fact in the extraction/encoder cannot disappear from the oracle.
    pub fn validate_contract(&self, c: &Contract) -> Result<(), String> {
        c.validate()?;
        let entities: BTreeMap<_, _> = c
            .entities
            .iter()
            .map(|(id, e)| (id.clone(), e.kind.clone()))
            .collect();
        let relations: BTreeMap<_, _> = c
            .relations
            .iter()
            .map(|(id, r)| (id.clone(), [r.subject_kind.clone(), r.object_kind.clone()]))
            .collect();
        if entities != self.entity_kinds || relations != self.relation_kinds {
            return Err("extraction differs from reference entity/relation kinds".into());
        }
        let ids: BTreeSet<_> = self.facts.iter().map(|f| &f.id).collect();
        if self.facts.is_empty()
            || ids.len() != self.facts.len()
            || c.assertions.len() != self.facts.len()
            || self
                .facts
                .iter()
                .any(|f| !c.assertions.iter().any(|a| f.matches(a)))
        {
            return Err(format!(
                "extraction differs from independent ledger: {}",
                self.id
            ));
        }
        // Fixtures use exactly one original; locators are hints and are never opened.
        if c.evidence.len() != 1
            || c.evidence.get("e1").is_none_or(|e| {
                e.sha256 != self.source_sha256
                    || e.locator != format!("fixture:golden/{}.txt", self.id)
            })
        {
            return Err("fixture evidence identity mismatch".into());
        }
        let mut questions = BTreeSet::new();
        if self.questions.is_empty()
            || self
                .questions
                .iter()
                .any(|q| q.id.is_empty() || q.prompt.is_empty() || !questions.insert(&q.id))
            || !self.questions.iter().any(|q| q.task == Task::Comprehension)
            || !self.questions.iter().any(|q| q.task == Task::Application)
        {
            return Err("invalid golden question set".into());
        }
        Ok(())
    }

    /// Explicit synthetic scoring control built from the ledger, never an inference.
    pub fn synthetic_perfect(&self) -> Value {
        let assertions: Vec<_> = self
            .facts
            .iter()
            .map(|f| {
                serde_json::json!({
                    "id":f.id,"subject":f.subject,"predicate":f.relation,"object":f.value,
                    "times":f.times(),"provenance":f.provenance(),"uncertainty":f.uncertainty()
                })
            })
            .collect();
        let answers = |task| {
            self.questions
                .iter()
                .filter(|q| q.task == task)
                .map(|q| (q.id.clone(), q.expected.clone()))
                .collect::<BTreeMap<_, _>>()
        };
        serde_json::json!({"reconstruction":assertions,"comprehension":answers(Task::Comprehension),"application":answers(Task::Application)})
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Response {
    reconstruction: Vec<Reconstruction>,
    comprehension: BTreeMap<String, Value>,
    application: BTreeMap<String, Value>,
}
/// Missing fields are scoring losses; wrong shapes/unknown fields are malformed.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Reconstruction {
    id: String,
    subject: Option<String>,
    predicate: Option<String>,
    object: Option<Object>,
    times: Option<Times>,
    provenance: Option<Provenance>,
    uncertainty: Option<Uncertainty>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AnswerScore {
    pub expected: usize,
    pub correct: usize,
    pub missing: usize,
    pub wrong: usize,
    pub invented: usize,
}
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Score {
    pub valid_structured_response: bool,
    pub malformed: usize,
    pub expected_assertions: usize,
    pub correct_assertions: usize,
    pub missing_assertions: usize,
    pub invented_assertions: usize,
    pub wrong_relationships: usize,
    pub temporal_errors: usize,
    pub provenance_errors: usize,
    pub uncertainty_errors: usize,
    pub comprehension: AnswerScore,
    pub application: AnswerScore,
    pub total_penalties: usize,
    pub fidelity: Option<f64>,
    pub error: Option<String>,
}

fn answers(case: &Case, task: Task, actual: &BTreeMap<String, Value>) -> AnswerScore {
    let expected: BTreeMap<_, _> = case
        .questions
        .iter()
        .filter(|q| q.task == task)
        .map(|q| (&q.id, &q.expected))
        .collect();
    let mut score = AnswerScore {
        expected: expected.len(),
        ..AnswerScore::default()
    };
    for (id, value) in &expected {
        match actual.get(*id) {
            None => score.missing += 1,
            Some(answer) if answer == *value => score.correct += 1,
            Some(_) => score.wrong += 1,
        }
    }
    score.invented = actual
        .keys()
        .filter(|id| !expected.contains_key(id))
        .count();
    score
}

pub fn score(case: &Case, response_bytes: &[u8]) -> Score {
    let mut score = Score {
        expected_assertions: case.facts.len(),
        ..Score::default()
    };
    let response = (|| {
        let response: Response = from_json(response_bytes)?;
        let mut ids = BTreeSet::new();
        if response
            .reconstruction
            .iter()
            .any(|r| r.id.is_empty() || !ids.insert(&r.id))
        {
            return Err("duplicate/empty response assertion id".into());
        }
        Ok::<_, String>(response)
    })();
    let response = match response {
        Ok(r) => r,
        Err(error) => {
            score.malformed = 1;
            score.error = Some(error);
            return score;
        }
    };
    score.valid_structured_response = true;
    for fact in &case.facts {
        let Some(actual) = response.reconstruction.iter().find(|r| r.id == fact.id) else {
            score.missing_assertions += 1;
            continue;
        };
        let relationship = actual.subject.as_ref() == Some(&fact.subject)
            && actual.predicate.as_ref() == Some(&fact.relation)
            && actual.object.as_ref() == Some(&fact.value);
        let times = actual.times.as_ref() == Some(&fact.times());
        let provenance = actual.provenance.as_ref() == Some(&fact.provenance());
        let uncertainty = actual.uncertainty.as_ref() == Some(&fact.uncertainty());
        score.wrong_relationships += usize::from(!relationship);
        score.temporal_errors += usize::from(!times);
        score.provenance_errors += usize::from(!provenance);
        score.uncertainty_errors += usize::from(!uncertainty);
        score.correct_assertions += usize::from(relationship && times && provenance && uncertainty);
    }
    score.invented_assertions = response
        .reconstruction
        .iter()
        .filter(|r| !case.facts.iter().any(|f| f.id == r.id))
        .count();
    score.comprehension = answers(case, Task::Comprehension, &response.comprehension);
    score.application = answers(case, Task::Application, &response.application);
    score.total_penalties = score.missing_assertions
        + score.invented_assertions
        + score.wrong_relationships
        + score.temporal_errors
        + score.provenance_errors
        + score.uncertainty_errors
        + score.comprehension.missing
        + score.comprehension.wrong
        + score.comprehension.invented
        + score.application.missing
        + score.application.wrong
        + score.application.invented;
    let denominator = case.facts.len() + score.comprehension.expected + score.application.expected;
    score.fidelity = (denominator != 0)
        .then(|| denominator.saturating_sub(score.total_penalties) as f64 / denominator as f64);
    score
}

pub fn assertion_efficiency(
    score: Option<&Score>,
    actual_input_tokens: Option<u64>,
) -> Option<f64> {
    match (score, actual_input_tokens) {
        (Some(s), Some(n))
            if s.valid_structured_response
                && s.fidelity.is_some()
                && s.expected_assertions > 0
                && n > 0 =>
        {
            Some(s.correct_assertions as f64 * 1000.0 / n as f64)
        }
        _ => None,
    }
}

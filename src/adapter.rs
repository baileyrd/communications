//! Offline render/parse boundaries, never network clients or protocol endpoints.
use crate::{
    codec::{Encoding, MAX_PAYLOAD_BYTES},
    decode::UniqueValue,
    Contract,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Serialize;
use serde_json::{json, Value};

pub const MAX_ENVELOPE_BYTES: usize = 4 * MAX_PAYLOAD_BYTES;
const CONTRACT_SCHEMA: &str = include_str!("../docs/CONTRACT-SCHEMA.txt");
const RECORD_SCHEMA: &str = include_str!("../docs/RECORD-SCHEMA.txt");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Context {
    HumanModel,
    ModelTool,
    ModelModel,
    ModelData,
    ModelNative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Arm {
    Conventional,
    Semantic(Encoding),
}

#[derive(Debug, Serialize)]
pub struct Capability {
    pub context: Context,
    pub offline_roundtrip: bool,
    pub live_endpoint: bool,
    pub limitation: &'static str,
}

impl Context {
    pub const OFFLINE: [Self; 4] = [
        Self::HumanModel,
        Self::ModelTool,
        Self::ModelModel,
        Self::ModelData,
    ];

    pub fn capability(self) -> Capability {
        Capability { context: self, offline_roundtrip: self != Self::ModelNative, live_endpoint: false,
            limitation: match self {
                Self::HumanModel => "Offline user-message text; no model comprehension measured.",
                Self::ModelTool => "Offline tools/call JSON-RPC-shaped envelope; no MCP session, discovery or endpoint.",
                Self::ModelModel => "Offline sender/receiver message boundary; no agent or model execution.",
                Self::ModelData => "Offline retrieved-document envelope; no search, storage or locator retrieval.",
                Self::ModelNative => "Unsupported/not testable: no authorized documented tensor/hidden-state interface.",
            }
        }
    }
}

impl Arm {
    /// Bytes of the exact schema text carried in a cold semantic envelope.
    /// Conventional arms use ordinary labelled text or structured JSON; no
    /// separate schema is transmitted. This does not establish comprehension.
    pub fn schema_bytes(self) -> usize {
        self.schema().len()
    }
    fn schema(self) -> String {
        match self {
            Self::Conventional => String::new(),
            Self::Semantic(Encoding::Records) => format!("{CONTRACT_SCHEMA}\n{RECORD_SCHEMA}"),
            Self::Semantic(Encoding::Cbor) => format!("{CONTRACT_SCHEMA}\nCBOR: RFC 8949 data model; same named fields and variant labels as JSON; Ciborium 0.2.2 emitted ordering profile only. Payload is standard padded base64. Decode CBOR to the contract before use."),
            Self::Semantic(Encoding::Json) => CONTRACT_SCHEMA.into(),
        }
    }

    pub fn payload(self, context: Context, c: &Contract) -> Result<Vec<u8>, String> {
        if context == Context::ModelNative {
            return Err(context.capability().limitation.into());
        }
        match self {
            Self::Conventional if context == Context::HumanModel => fact_sheet(c),
            Self::Conventional => Encoding::Json.encode(c),
            Self::Semantic(encoding) => encoding.encode(c),
        }
    }
}

/// Encode plus context translation, allocating a complete cold UTF-8 envelope.
pub fn render(context: Context, arm: Arm, c: &Contract) -> Result<Vec<u8>, String> {
    let payload = arm.payload(context, c)?;
    let body = match arm {
        Arm::Conventional if context == Context::HumanModel => {
            Value::String(String::from_utf8(payload).map_err(|e| e.to_string())?)
        }
        Arm::Conventional => serde_json::from_slice(&payload).map_err(|e| e.to_string())?,
        Arm::Semantic(encoding) => {
            let (transfer, content) = if encoding == Encoding::Json {
                (
                    "utf8",
                    String::from_utf8(payload).map_err(|e| e.to_string())?,
                )
            } else {
                ("base64", STANDARD.encode(payload))
            };
            json!({"adapter_version":1,"encoding":encoding,"schema":arm.schema(),"transfer":transfer,"payload":content})
        }
    };
    let envelope = match context {
        Context::HumanModel | Context::ModelModel => {
            let content = if let Value::String(text) = body {
                text
            } else {
                body.to_string()
            };
            json!({"messages":[{"role":"user","content":content}]})
        }
        Context::ModelTool => {
            json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"accept_facts","arguments":body}})
        }
        Context::ModelData => json!({"documents":[{"id":"observation","content":body}]}),
        Context::ModelNative => return Err(context.capability().limitation.into()),
    };
    let bytes = serde_json::to_vec(&envelope).map_err(|e| e.to_string())?;
    envelope_bound(bytes.len())?;
    Ok(bytes)
}

/// Reject duplicate JSON keys throughout, ambiguous metadata, wrong context/arm,
/// and any fields the corresponding renderer cannot preserve. Every path ends
/// in contract validation, including conventional text and JSON arms.
pub fn parse(context: Context, arm: Arm, bytes: &[u8]) -> Result<Contract, String> {
    if context == Context::ModelNative {
        return Err(context.capability().limitation.into());
    }
    envelope_bound(bytes.len())?;
    let envelope = unique_json(bytes)?;
    let body = match context {
        Context::HumanModel | Context::ModelModel => {
            let text = envelope
                .pointer("/messages/0/content")
                .and_then(Value::as_str)
                .ok_or("missing message text")?;
            if context == Context::HumanModel && arm == Arm::Conventional {
                Value::String(text.into())
            } else {
                unique_json(text.as_bytes())?
            }
        }
        Context::ModelTool => envelope
            .pointer("/params/arguments")
            .ok_or("missing tool arguments")?
            .clone(),
        Context::ModelData => envelope
            .pointer("/documents/0/content")
            .ok_or("missing document content")?
            .clone(),
        Context::ModelNative => unreachable!(),
    };
    let c = match arm {
        Arm::Conventional if context == Context::HumanModel => {
            parse_fact_sheet(body.as_str().ok_or("missing fact sheet")?)?
        }
        Arm::Conventional => {
            Encoding::Json.decode(&serde_json::to_vec(&body).map_err(|e| e.to_string())?)?
        }
        Arm::Semantic(encoding) => {
            let content = body
                .get("payload")
                .and_then(Value::as_str)
                .ok_or("missing semantic payload")?;
            let payload = if encoding == Encoding::Json {
                content.as_bytes().to_vec()
            } else {
                STANDARD.decode(content).map_err(|e| e.to_string())?
            };
            encoding.decode(&payload)?
        }
    };
    // Exact reconstruction also checks adapter version, transfer, encoding,
    // schema, envelope ids/method/roles, extra messages and unknown fields.
    if render(context, arm, &c)? != bytes {
        return Err("unknown, mismatched or noncanonical offline envelope".into());
    }
    Ok(c)
}

fn envelope_bound(n: usize) -> Result<(), String> {
    if n > MAX_ENVELOPE_BYTES {
        Err("envelope exceeds 4 MiB limit".into())
    } else {
        Ok(())
    }
}

fn unique_json(bytes: &[u8]) -> Result<Value, String> {
    serde_json::from_slice::<UniqueValue>(bytes)
        .map(|v| v.0)
        .map_err(|e| e.to_string())
}

const SECTIONS: [(&str, &str); 5] = [
    ("Version", "version"),
    ("Entities", "entities"),
    ("Relations", "relations"),
    ("Evidence", "evidence"),
    ("Assertions", "assertions"),
];

fn fact_sheet(c: &Contract) -> Result<Vec<u8>, String> {
    let json = Encoding::Json.encode(c)?;
    let value: Value = serde_json::from_slice(&json).map_err(|e| e.to_string())?;
    let mut text = String::new();
    for (label, field) in SECTIONS {
        text.push_str(&format!("{label}: {}\n", value[field]));
    }
    crate::codec::bound(text.len())?;
    Ok(text.into_bytes())
}

fn parse_fact_sheet(text: &str) -> Result<Contract, String> {
    let mut lines = text.lines();
    let mut fields = Vec::new();
    for (label, field) in SECTIONS {
        let line = lines.next().ok_or("missing fact-sheet section")?;
        let raw = line
            .strip_prefix(&format!("{label}: "))
            .ok_or("unknown fact-sheet section")?;
        // Preserve raw nested keys until duplicate-checking Contract ingress.
        fields.push(format!("\"{field}\":{raw}"));
    }
    if lines.next().is_some() {
        return Err("extra fact-sheet section".into());
    }
    Encoding::Json.decode(format!("{{{}}}", fields.join(",")).as_bytes())
}

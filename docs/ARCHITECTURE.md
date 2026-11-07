# M01 minimal semantic contract — #791

Status: #791 builds on merged #790 (PR #2), retaining one independent crate,
one smoke fixture and one JSON wire form. The vocabulary remains versioned
`0.1-draft`; this is a minimal experiment contract, not a production standard.
Independent review and merge of this increment remain coordinator gates.

The experiment asks whether a shared semantic contract can carry matched facts
across mechanisms with less total cost and equal understanding. A smaller binary
payload alone cannot answer that question. The core must remain independent of
MCP, model APIs, retrieval storage, and Rusty Mill's workspace.

## Draft semantic boundary

`src/lib.rs` gives this vocabulary executable validation:

| Field | Meaning and invariant |
| --- | --- |
| `version` | Exact `0.1-draft`; unknown versions fail validation. |
| `entities` | Local id → kind. Empty, padded or control-containing ids/kinds fail. Literal-kind names are reserved. |
| `relations` | Predicate → subject and object kinds. Kinds must exist in entities (or a literal object kind); assertions must match both. |
| `assertions` | Unique id, subject, typed predicate and typed object: entity reference, boolean, text, or integer. No last-writer-wins deduplication. |
| `uncertainty` | Explicit unknown or reported confidence in [0,1000] permille. It is not inferred certainty or a calibrated probability. |
| `provenance` | Existing asserting entity, observed/reported/inferred method, nonempty set of valid evidence references. Attribution is distinct from truth. |
| `times` | Required separate event (explicitly nullable), record, interpretation times in UTC epoch milliseconds. Unknown event time stays null. No invented cross-clock ordering constraint. |
| `evidence` | Lowercase SHA-256 of exact original bytes plus retrieval locator. Digest is authoritative; locator is a hint. Retrieval and byte verification are separate from reference validation. |

Maps and evidence-reference sets are sorted; assertions sort by id before compact
JSON encoding. Integers avoid floating-point spelling and NaN ambiguity. This is
a stable encoding for this Rust probe, **not** RFC 8785, a standardized universal
format, or a claim of cross-language canonical equivalence. Unicode and strings
retain their exact content. Conflicting assertions with different ids remain
separate and retain their own provenance and uncertainty.

The fixture is a source report that a valve is probably open and is in a room.
The contract does not promote the report to a verified physical observation.
It contains no model-generated data. A digest detects changed evidence; it does
not prove authorship, authenticity, availability, or that an assertion is entailed.
Evidence is not fetched by this probe, and locators are never opened by the CLI.

## Dependency direction and future ports

`evidence bytes → reviewed extraction → contract → wire encoder → mechanism adapter`

The core owns data and invariants. Encoders own wire syntax. Adapters own endpoint
envelopes and any rendering necessary for a model to consume those bytes. A future
runner measures each boundary separately and records the final model-visible
request. Decoders must validate before handing data to consumers. A source fact
ledger is independent of all renderers so identical omissions cannot pass simply
because two adapters share a bug.

CBOR and compact typed-record encodings are #792, after #791 stabilizes the
contract. JSON remains a readable control. A small Protobuf runtime may later be
an optional **fourth** baseline; it does not replace CBOR. MCP is one contextual
adapter, not the contract or proof of a model-native interface. No hidden-state
or arbitrary tensor interface is assumed available.

## Validated ingress and canonical form

`Contract::from_json(&bytes)` and direct Serde deserialization of `Contract`
reject duplicate JSON object keys at every nesting level before constructing
maps or tagged enums. Escaped key aliases count as duplicates. Duplicate evidence
references are rejected before set construction. Validation runs during decode:
unknown schema versions, fields/variants, missing fields, dangling references,
type mismatches, duplicate assertion ids, invalid uncertainty and malformed
digests fail. An unknown event time must be explicit `null`, not omitted. Integer
fields reject floats and overflow. JSON whitespace and object/set/assertion order
may differ on ingress; canonical bytes normalize order, not semantic content.

Use `canonical_json()` for validated output; ordinary Serde serialization is a
low-level representation operation. Public Rust fields permit construction and
editing, so call `validate()` before consuming an edited value. `verify_evidence`
checks supplied original bytes against the referenced SHA-256 and reports missing
references or mismatches. It never retrieves a locator. The same locator may name
different snapshots; the digest defines original content identity. Digest aliases
at different locators are allowed. No mutation or retrieval store is implemented.

Regression tests include every object boundary, evidence-set duplication, all
object variants, confidence boundaries, unknown confidence/event time, extreme
integer/time values, Unicode preservation, conflicting assertions and canonical
idempotence. Conflicting claims remain independent assertions; no truth resolution
or implicit deduplication occurs.

## Deliberate limits

JSON ingress buffers a duplicate-checked value before typed construction. It uses
Serde JSON's default recursion limit and has no explicit byte/collection quotas;
callers must bound input size before exposing it to untrusted traffic. No schema
migration, cross-language canonicalization, units, intervals, negation calculus,
authentication, evidence availability or semantic entailment is claimed. This
is a structural contract, not a universal ontology. Kind names remain local opaque
strings; unused relation kinds must still be represented by an entity in this
minimal vocabulary. Separate per-kind registries are outside this slice.

CBOR/typed-record encodings (#792), golden cases, extraction, adapters and semantic
fidelity/efficiency evaluation remain downstream. Exact structural round-trip
preservation does not establish model understanding or efficiency.

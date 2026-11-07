# M01 executable architecture proposal — #790

Status: draft for the user's architectural review. This probe is intentionally
one crate, one smoke fixture, one JSON wire form. It establishes an executable
boundary without claiming the downstream #791 contract is complete or stable.

The experiment asks whether a shared semantic contract can carry matched facts
across mechanisms with less total cost and equal understanding. A smaller binary
payload alone cannot answer that question. The core must remain independent of
MCP, model APIs, retrieval storage, and Rusty Mill's workspace.

## Draft semantic boundary

`src/lib.rs` gives this proposed vocabulary executable validation:

| Field | Meaning and invariant |
| --- | --- |
| `version` | Exact `0.1-draft`; unknown versions fail validation. |
| `entities` | Local id → kind. Empty ids/kinds fail. Literal-kind names are reserved. |
| `relations` | Predicate → subject and object kinds. Assertions must match both. |
| `assertions` | Unique id, subject, typed predicate and typed object: entity reference, boolean, text, or integer. No last-writer-wins deduplication. |
| `uncertainty` | Explicit unknown or reported confidence in [0,1000] permille. It is not inferred certainty or a calibrated probability. |
| `provenance` | Existing asserting entity, observed/reported/inferred method, nonempty set of valid evidence references. Attribution is distinct from truth. |
| `times` | Separate event (nullable), record, interpretation times in UTC epoch milliseconds. Unknown event time stays null. No invented cross-clock ordering constraint. |
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

## Deliberate limits and review questions

This is not a production untrusted-input API. Serde can accept duplicate JSON map
keys by retaining the last occurrence; the probe does not define a hardened
decoder, input-size limits, numeric units, intervals, negation logic, schema
migrations, or cross-language canonicalization. Unknown struct fields are refused,
but that alone does not establish strict canonical parsing. These are #791 review
items; the CLI only reads the checked-in smoke fixture. Full fixtures, extraction,
reconstruction/comprehension/application scoring and benchmark adapters remain
downstream. Exact round-trip is structural preservation, not semantic fidelity.

Architectural feedback requested through Mill Board #790: is this draft vocabulary
and separation of source-reported confidence from unknown uncertainty a useful
starting point for #791? It remains reversible until independent review.

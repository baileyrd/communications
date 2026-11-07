# Local wire profiles — #792

All three profiles carry the complete `0.1-draft` contract defined in
[architecture](ARCHITECTURE.md). `codec::Encoding` validates before encoding and
before returning any decoded contract. Input and emitted payloads are limited to
1 MiB. Exactly one document/frame is accepted; trailing bytes fail. There is no
retrieval, compression, encryption, authentication or error-correcting checksum.
A corruption that changes one valid fact into another valid fact is not detectable
without an external trusted digest/signature. Evidence SHA-256 checks caller-supplied
original bytes separately; it does not authenticate the contract itself.

## JSON control

`Encoding::Json` retains #791's duplicate-checking `Contract::from_json` ingress
and `canonical_json` output. All object keys and evidence references must be unique.
Unknown fields/variants, missing fields, floats and invalid contracts fail. JSON
input may use alternate whitespace/object/assertion ordering. Output sorts maps,
reference sets and assertion ids, preserving exact field content and Unicode.
This is the probe's repeatable encoding, not RFC 8785 canonical JSON.

`Contract::from_json` remains a low-level API without the transport byte cap;
use `Encoding::Json.decode` at the bounded transport boundary. JSON uses Serde's
default recursion limit. Valid JSON can contain insignificant whitespace; no
extension or schema migration is implemented.

## CBOR profile

Genuine CBOR maps/text/arrays/integers/booleans/null are encoded by pinned
`ciborium = 0.2.2`. This is a named-field control using the same field names and
externally visible enum labels as JSON, rather than a CBOR byte string containing
JSON. Signed integer extremes and null event times round-trip exactly. Digests
remain lowercase hex text here. No float, CBOR byte-string field, tag or undefined
value belongs to this vocabulary.

The decoder first rejects duplicate keys at **every** map boundary and duplicate
reference sets, then applies the same typed contract validation as JSON. CBOR
recursion is capped at 32 levels. It consumes exactly one value and compares the
original bytes with a validated re-encoding. Only this crate's emitted profile is
accepted: struct keys follow declaration order, BTree maps/references sort by Rust
string ordering, and assertions sort by id; integers/lengths use Ciborium's preferred
emission. Indefinite lengths, tags, nonpreferred scalar/length encodings and alternate
ordering fail, even where generic CBOR would permit them. They are never normalized
silently. This is **not** a claim of generic CBOR interoperability, RFC deterministic
ordering, or universal cross-language canonicalization. The independent known CBOR
prefix and all contract values are exercised in the tests.

## CSR/1 typed records

[Exact wire grammar](RECORD-SCHEMA.txt) defines every byte, tag, section and bound.
The four-byte `CSR\x01` magic includes the framing revision; a length-prefixed
contract version follows. Four ordered count-prefixed sections contain entity,
relation, evidence and assertion records. Every original field is represented;
strings are length-prefixed exact UTF-8, digest bytes are the reversible 32-byte
form of the contract's 64 lowercase hex digits, and booleans, object variants,
uncertainty, provenance methods and nullable event times have explicit tags.
Integers have fixed-width little-endian two's-complement representation. There is
no dictionary derived from the fixture and no implicit provenance/time default.

Lengths and counts are checked **before** allocating/copying announced content:
65,536 bytes per string, 4,096 per section/reference collection, 1 MiB total.
Unknown tags/framing/schema version, invalid UTF-8/scalars, duplicates, out-of-order
sections/identifiers/assertions, truncated data and trailing bytes fail. Re-encoding
checks the emitted local ordering profile after semantic validation. These quotas
are limits of this bounded wire form, not new semantic limits on all contracts.
This is a custom local typed-record experiment, not standardized, not Protobuf,
and not claimed to be a generally optimal compression scheme.

Tests include an independent literal CSR/1 byte vector, every truncation of the
smoke payload in all three formats, CBOR duplicate keys at every object boundary,
invalid scalar tags/booleans/confidence, unknown fields, bounds, exact text and all
four object variants. They are focused regressions for #792, not #793's golden
comprehension/application suite. No Rusty Mill code was copied.

## Offline adapters

`adapter::{render, parse}` composes a codec with four contexts. `Context::capability`
also reports model-native unsupported/not testable. The four supported boundaries
are local serializers/parsers only, with no model/API/MCP/retrieval transport:

| Context | Conventional arm | Semantic arms |
| --- | --- | --- |
| Human ↔ model | User message containing a labelled fact sheet; each value is compact JSON to preserve types and full metadata | User message containing a semantic packet |
| Model ↔ tool | `tools/call` JSON-RPC-shaped request with structured fact arguments | Same request with semantic packet arguments |
| Model ↔ model | User message containing structured JSON facts | Same message with semantic packet content |
| Model ↔ data | Retrieved-document-shaped object with structured JSON facts as content | Same document with semantic packet content |

The conventional human sheet has five labelled sections (version, entities,
relations, evidence, assertions); the other conventional arms use ordinary named
JSON facts. They intentionally share the contract's vocabulary and all metadata:
JSON is already idiomatic for structured tool arguments and document/message
content, so the conventional structured payload and semantic JSON payload coincide.
This is an offline matched structural control, not an independently validated
natural-language extraction or a representative workload for all mechanisms.
No assertion/provenance/time/uncertainty/evidence field is removed to make an arm
smaller. The source fixture is unchanged and separately checked against every arm.

A semantic packet carries adapter version 1, encoding, transfer, full schema text
and payload. JSON is a UTF-8 string; binary CBOR/CSR is standard padded base64
using pinned `base64 = 0.22.1`. Messages stringify the packet and consequently
include another escaping layer. All such bytes count. The exact contract and CSR
schema text delivered in packets is checked in, not an uncounted external lookup.
Schema bytes are raw UTF-8 before escaping; the final envelope byte count includes
the escaped form. Conventional arms transmit no separate schema. No schema cache
or amortized run is claimed. A future model pilot must test that these schemas are
sufficient and hold any additional common instructions/questions/settings equal.

The parser duplicate-checks outer and nested JSON, recovers the payload, validates
the contract, and reconstructs the exact envelope to reject unknown/mismatched
schema, version, transfer, envelope ids/roles/methods, extra messages and silently
lost fields. Thus it accepts only exact local renderer output, not arbitrary
MCP/OpenAI/A2A/retrieval inputs. Human and model-model semantic messages deliberately
share the same message shape; context is selected by the caller. Envelopes are
bounded at 4 MiB. There is no live protocol handshake, discovery/tool-definition
exchange, query, evidence fetch, binary attachment, agent messaging or hidden-state
access. Outside-probe protocol overhead remains unmeasured. Base64 bytes are not
zero tokens, and a round trip in Rust does not prove a model can decode a packet.

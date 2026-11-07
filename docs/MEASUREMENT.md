# Measurement architecture and fair comparison

This is a plan for #792–#794; only the offline JSON smoke in `src/main.rs` runs now.
No model endpoint is configured and no paid calls are authorized. Null means
not measured, never zero. The hypothesis is not yet tested.

## Comparison design

Use one immutable source input and an independently reviewed reference fact ledger
per case. Match entities, relationships, positive/negative assertions, provenance,
each time field, uncertainty and evidence references in every representation.
Use the same comprehension and application questions, scoring rubric and output
schema. Compare the mechanism's normal envelope/rendering (conventional arm) with
the same facts via semantic JSON, CBOR, and compact typed records (universal arms).
Keep the conventional arm idiomatic; do not add artificial verbosity. Report schema
instructions and shared context bytes/tokens separately, with both cold and
amortized costs when schemas are reused. Never compare enriched semantic content
against a conventional arm stripped of uncertainty or provenance.

For each run pin endpoint/provider, exact model/revision, temperature, top_p,
seed when supported, output budget, tool definitions, instructions, schema,
tokenizer/revision when available, cache mode and adapter version. Model settings
are currently **unconfigured**, not silently defaulted. A suggested first pilot is
temperature 0, top_p 1 and a fixed output cap; record provider acceptance and any
unsupported parameters. Hold settings equal within a comparison. Randomize or
counterbalance arm order, repeat trials and report sample counts, raw distributions
and uncertainty intervals. Cache hits and errors must be visible, not discarded.

## Measurement boundaries

| Metric | Boundary and required evidence |
| --- | --- |
| Wire bytes | Count actual UTF-8/binary payload bytes and final envelope bytes separately, including base64 expansion and schema overhead. Compression, if enabled, is a separate arm. |
| Encode/decode | Monotonic wall time around the named operation, warm-up and allocation/validation policy stated. Setup and I/O outside microbench loops. No wall time → CPU time inference. |
| Actual input/output tokens | Provider usage on the exact model-visible request/response, including tool/schema/system envelopes. Record cached tokens separately. Local tokenizer estimates must be labeled estimates and cannot replace actual usage. |
| Model latency | First request send → final response receive (and time to first token if supported), separately from local translation and retries. |
| End-to-end latency | Source extraction → final consumer-usable result, with extraction, encoding, network, inference, decoding, validation and reconstruction spans. Record failed attempts/retries. |
| Compute/money | CPU duration and peak RSS measured independently of wall time. Provider usage × dated pricing with cached/reasoning charges or actual billed totals. Unavailable measurements remain null. No monetary conclusion from bytes alone. |
| Translation overhead | Each adapter render/parse/validation span and request/response size expansion; measure both directions. Include schema lookup and evidence retrieval when required. |
| Semantic fidelity | Independent ledger comparison for reconstruction, comprehension, application; penalize invented claims, missing facts, wrong relationships, lost time/provenance/uncertainty. Exact JSON round-trip is only structural control. |
| Assertion efficiency | Correct reference assertions / 1,000 actual input tokens only when input usage and independent scores exist; zero-denominator or missing values yield null. |

CBOR bytes cannot be sent as hidden states to ordinary text model APIs. A model
adapter may decode them back into text or send a supported binary attachment; its
overhead and final token usage still count. Mark inaccessible model-native tensor
or hidden-state interfaces **not testable**. A proxy must be labeled and never
treated as equivalent access.

## Five future contexts

Human↔model uses normal text plus questions; model↔tool uses an actual supported
tool-call envelope (MCP-style is a context, not automatic endpoint support);
model↔model holds sender/receiver settings and intermediate payloads fixed;
model↔retrieval counts retrieval and context assembly; model-native representation
is not testable without a documented authorized interface. The architecture does
not claim any of these model experiments have run.

#793 owns the three initial golden cases, then six, including immutable inputs,
fact ledgers and question/rubric sets. The checked-in observation is an architecture
smoke fixture, not that golden suite. #794 owns the actual comparative pilot.

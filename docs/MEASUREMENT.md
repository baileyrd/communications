# Measurement architecture and fair comparison

The bounded #792 smoke in `src/main.rs` runs three codecs and sixteen offline
adapter/control combinations on the unchanged observation fixture. Five capability
rows distinguish offline boundaries from unsupported model-native access. #793
now adds the [golden harness](GOLDEN-HARNESS.md); #794 adds a [repeated offline pilot](PILOT.md); real-model experiments remain
unmeasured. No model endpoint is configured and no paid calls are authorized. Null means unmeasured, never zero.
The comprehension/efficiency hypothesis is not tested.

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

## Context boundary and future live experiments

Human↔model uses normal text plus questions; model↔tool uses an actual supported
tool-call envelope (MCP-style is a context, not automatic endpoint support);
model↔model holds sender/receiver settings and intermediate payloads fixed;
model↔retrieval counts retrieval and context assembly; model-native representation
is not testable without a documented authorized interface. The offline render/parse boundaries and matched controls are documented in
[wire formats](WIRE-FORMATS.md); none of these model experiments has run.

#793 delivers three initial golden cases expanded to six with immutable sources,
separate reference ledgers and structured reconstruction/question rubrics. The
original observation remains a separate architecture smoke fixture. #794 owns the
actual comparative pilot. See [golden measurement/import boundaries](GOLDEN-HARNESS.md)
for final requests with common questions/output schema, named overlapping local
timing spans, caller-attested telemetry and exact scoring limits. Local runs have
null model scores; only explicitly tagged synthetic replay has scorer controls.

## Delivered #792 local measurements

`cargo run --release --locked > docs/results/codec-adapter-smoke.jsonl` emits three
codec rows, sixteen adapter rows and five capability rows. The matched source is
`fixtures/contract.json` and original bytes `fixtures/observation.txt`, preserved
unchanged from #790/#791. Fixture loading, validation setup and evidence hashing
happen before all timing. Round-trip equality is asserted outside timing and the
independent checked-in reference is compared in regression tests. No golden-case
extraction or live input is measured.

Each operation has 100 untimed warm-up iterations followed by 1,000 timed iterations
using `Instant` and `black_box`. Results are totals and truncated integer means in
nanoseconds, not CPU time. Encoding includes validation, sorting/cloning where
used, all allocations and destruction. Decoding includes parse, allocations,
contract validation, and destruction. CBOR/CSR decode also includes validated
re-encoding and byte comparison to enforce the strict local profile. JSON decode
retains the duplicate-checked value and typed construction cost. No evidence
hashing, source retrieval, I/O, printing, dependency compilation or model execution
is inside these spans. Warm-up does not exclude allocator cost from timed work.

Adapter render times the **complete local translation**, including the selected
codec, validation, full cold schema construction, UTF-8/base64 rendering, JSON
envelope and drop. Adapter parse includes outer/nested duplicate checks, payload
transfer decoding, codec decode/validation, full envelope reconstruction/comparison
and drop. These combined measurements are intentional; they are not disjoint
additional overhead to add to codec times, nor is subtracting independent means a
measurement of isolated translation cost. The raw rows name these boundaries.

Payload bytes are counted before transfer. Rendered payload text bytes count UTF-8
or padded base64 before JSON escaping. Schema text bytes count the exact included
UTF-8 schema before escaping. Final envelope bytes count **all actual emitted UTF-8
bytes**, including schemas, version/encoding/transfer metadata, base64, quotes,
backslashes and nested message escaping. Envelope expansion is final minus raw
payload, not compression. Schema is repeated in each semantic request; there is
no cache, amortized result or uncounted external schema. Conventional arms have no
separate schema; they use an ordinary labelled fact sheet or structured JSON with
the same vocabulary/facts. The structured conventional payload and semantic JSON
payload coincide. These controls test structural content parity, not how a model
would choose to write the facts or how well it comprehends either form.

Only the final **local offline** envelope is measured. An actual provider's system
prompts, questions, tool definitions/discovery, protocol/session/HTTP envelopes,
tokenization, retries, inference and responses are unavailable. Their metrics are
null/unmeasured; no estimate from byte counts is substituted. A future pilot must
record and compare those real costs, full settings and schema sufficiency.

One warmed process, fixed order, one tiny fixture and one total per operation do
not supply a confidence interval, comparative performance guarantee, memory/CPU
measurement or isolated-system assurance. Results can vary with scheduling and
allocator/cache state. Binary-byte reduction is only a transport observation;
base64 plus schema can make the final envelope larger than conventional JSON.


## #794 repetition boundary

`golden trial CASE CONTEXT ARM TRIAL_ID` executes one selected warmed observation.
The standard-library pilot driver shuffles all 96 combinations with seed 794 in
each of five repetitions and runs one fresh release child per trial (480 total).
Inner timing operations retain 3 warm-ups/20 iterations, including allocations,
validation and drop. All setup, child startup, fixture checks, hashing and I/O are
outside timing. Fixed operation order inside each child and shared host caches
remain limitations. Raw failed attempts are retained without replacement.

Compact raw JSONL/CSV hold identity/order/status/timing; a 96-record invariant
matrix holds matching request/fixture hashes, bytes and null telemetry. Join by
case/context/arm. Audit restores CSV exactly, regenerates requests, checks hashes,
coverage and byte/timing/null arithmetic. Summary distributions describe batch
means, n=5 per combination and n=120 heterogeneous pooled batches per arm, not
independent inner iterations; P95 uses nearest rank. No confidence intervals or
CPU/model/end-to-end interpretations are supplied. See [report/artifacts](PILOT.md).

## #795 60-case extension

[Expanded benchmark](EXPANDED-BENCHMARK.md) fixes 960 matched cells and five serial
fresh-child batch samples per cell (4,800 attempts, seed795). Per-cell distributions
preserve case/context heterogeneity; the separate pooled arm summary is descriptive.
Inner 20-operation loops are not independent samples. Five batches support local
variability description, not stable tails, population confidence or real-model
significance. All prior timing boundaries/null rules remain. No model interface,
cross-model interoperability, MCP session or retrieval experiment is inferred from
local envelopes. Native hidden-state access is explicitly unsupported. References
and expected answers originate outside tested renderers; synthetic common-author
transcription remains a limit requiring independent technical review.

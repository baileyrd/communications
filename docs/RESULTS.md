# #795 60-case offline expansion — 2026-11-07

**Full #795 acceptance remains incomplete.** The [expanded benchmark report](EXPANDED-BENCHMARK.md)
delivers 60 authored synthetic cases (134 facts/120 explicit question rubrics),
4,800/4,800 successful fresh-child local attempts and zero execution errors.
Four contexts × conventional/JSON/genuine CBOR/CSR/1 × five batches per cell,
seed795. All 960 matched cells preserve their complete contracts/ledgers.
No real or imported model response, MCP session, model exchange, retrieval or
native-state call ran. Actual accuracy/usage/cost/model latency stay null.

| Arm | Payload bytes, min–max | Complete cold local request bytes, min–max |
| --- | ---: | ---: |
| Conventional | 860–2,058 | 2,951–4,542 |
| JSON | 866–2,058 | 4,652–6,844 |
| CBOR | 681–1,593 | 4,752–6,041 |
| CSR/1 | 318–863 | 5,494–6,257 |

For every one of the 240 case/context pairs, all three semantic arms have larger
complete local requests than their matched conventional control, despite smaller
CBOR/CSR payloads. These exact counts include common questions/instructions/output
schema and cold transport schema/base64/escaping. No provider/protocol overhead,
model tokenization, schema caching or understanding claim follows. Conventional
human text is a labelled fact sheet; other conventional payloads are structured
JSON with the same facts. These are authored representation controls, not a
representative natural-language baseline.

| Arm | Complete request render, pooled batch means: n | Min ns | Median ns | P95 ns | Max ns |
| --- | ---: | ---: | ---: | ---: | ---: |
| Conventional | 1,200 | 20,049.25 | 32,542.525 | 50,180.6 | 338,086.9 |
| JSON | 1,200 | 15,526.4 | 21,507.65 | 32,475.45 | 179,896.1 |
| CBOR | 1,200 | 12,889.45 | 14,999.6 | 23,170.9 | 169,163.45 |
| CSR/1 | 1,200 | 12,789.25 | 14,946.8 | 22,099.8 | 293,824.9 |

This table pools heterogeneous cases/contexts for descriptive inspection; it is
not an iid significance comparison. [Per-cell statistics](results/expanded/summary-by-cell.json)
retain five fresh-child batch observations per cell, not 100 independent inner
operations. Each has 3 untimed warm-ups and a 20-operation measured batch per named
span. With n=5, nearest-rank P95 is the maximum; no stable-tail or population
confidence claim is justified. Shared host/OS caches and fixed operation order
within each child limit conclusions. Spans overlap and must not be summed or
converted to CPU/model/end-to-end latency. No performance gate is introduced.

[Raw JSONL](results/expanded/trials.jsonl), [equivalent CSV](results/expanded/trials.csv),
[lossless invariant matrix](results/expanded/matrix.json), [pre-run plan](results/expanded/plan.json),
[input/output/environment manifest](results/expanded/manifest.json),
[per-case source/reference/extraction digests](results/expanded/case-manifest.json),
[capability matrix](results/expanded/capabilities.json) and
[validation](results/expanded-validation.log) retain the evidence. The independent
audit verifies every source/reference/extraction, order/id, matrix/export join,
request hash/bytes, statistic, failure denominator and null boundary. See the
[full design, coverage taxonomy, authorship limits and reproduction commands](EXPANDED-BENCHMARK.md).

All 42 prior Rust integration tests and three Python regressions are retained;
three new Rust tests plus five Python regressions cover expanded loading/scaling,
selection, capabilities, all structural arms, import compatibility, statistics and
export/failure accounting. Original six fixtures and smoke/golden/pilot evidence
remain unchanged. The historical pilot's source-pinned audit runs in its immutable
input context and regenerates exact requests with the current binary; this avoids
misrepresenting old timings as measurements of new sources.

Full original-card live interfaces/cross-model interoperability remain untested.
No authorized provider endpoint, exact model pair or bounded call/budget/data-sharing
grant is configured; these are the concrete missing prerequisites. Native hidden
states are unsupported. A vector-specific board comment is excluded pending
verified owner scope. Independent exact-head review/CI and unresolved acceptance
assessment remain coordinator gates; Issue #1 stays open and #795 is not Done.

The following sections preserve earlier measured checkpoints and limitations.

---

# #794 repeated offline pilot — current result

See the [pilot report](PILOT.md): 480/480 successful local attempts, six cases ×
four offline contexts × four arms × five fresh processes; seeded order, raw
JSONL/CSV, fixed matrix, exact hashes and descriptive local timing distributions.
Smaller CBOR/CSR payloads still produce larger cold local requests than matched
conventional controls. The model comprehension/inference-efficiency hypothesis
is untested; model accuracy, tokens, cost and latency remain null. No real model
calls occurred. Independent review and CI remain coordinator gates.

The sections below preserve original historical measurements and limitations.
Their references to #794 being undelivered apply to those earlier checkpoints.

---

# #793 offline golden harness — 2026-11-07

PR #5 review correction: replay imports now use separate finite-number JSON
ingress with recursive duplicate-key checks, accepting fractional telemetry and
nested settings/`0.0`. The integer-only contract/fixture/scorer ingress is
unchanged. Four additional regressions verify positive CLI/JSONL/CSV paths,
error-specific negative/evidence/synthetic validation, strict integer fields and
nested duplicate-key rejection (41 integration tests total). The failure was
reproduced before the fix. [Correction validation](results/golden-correction-validation.log)
records required checks and the golden evidence audit. Original measured outputs
below are retained; requests, fixtures, scoring rubrics and timing operations are
unchanged. Test-only invented fractional telemetry is never model evidence.

Six pinned synthetic cases × four offline contexts × four matched arms produce
96 local trials. All source hashes, independent ledger/extraction comparisons and
complete structural round trips pass. The first three cases were checked before
the expanded six-case run. This delivers the bounded #793 offline harness; it does
not complete M01, Issue #1 or #794's real model pilot.

[Raw JSONL](results/golden-offline.jsonl), [compatible CSV](results/golden-offline.csv),
[first-three checks](results/golden-initial-check.jsonl), [six-case checks](results/golden-check.jsonl),
[synthetic replay JSONL](results/golden-synthetic-replay.jsonl),
[replay CSV](results/golden-synthetic-replay.csv), [exact example request](results/golden-request-example.json),
[hashes](results/golden-sha256.txt), [environment](results/golden-environment.txt)
and [final validation](results/golden-validation.log) retain the evidence.
[Commands and exact scoring/timing/import boundaries](GOLDEN-HARNESS.md) explain
how to reproduce and interpret it.

| Arm | Payload bytes, min–max over six cases/contexts | Complete local request bytes, min–max |
| --- | ---: | ---: |
| Conventional | 870–941 | 2,995–3,236 |
| Semantic JSON | 876–941 | 4,694–5,118 |
| Semantic CBOR | 687–748 | 4,783–4,931 |
| Semantic CSR/1 | 328–388 | 5,522–5,650 |

Final request bytes count the entire context envelope, common questions,
instructions and output schema, cold transport schemas, escaping and base64.
Compact binary payloads do not make these complete local requests smaller than
the matched conventional controls. Provider/HTTP/session/tool-definition envelopes
are not available, so these are local request sizes, not endpoint measurements.
The conventional structured payload and semantic JSON payload use identical facts;
conventional human text is a labelled fact sheet. No representative natural-language
baseline or protocol interoperability is claimed.

Named encode/decode, full adapter render/parse and full request-render wall-time
totals are in the raw rows: 3 untimed warm-ups and 20 timed iterations per operation,
with allocation/drop and validation inside. Setup, source/ledger checking, evidence
hashing, round-trip assertions, I/O, build and model execution are outside. Spans
overlap (adapter includes codec; request includes adapter); do not sum them. One
warmed release process, fixed order, tiny cases and no repeated trials/confidence
intervals limit interpretation. Wall times are not CPU or end-to-end latency.

All 96 offline rows have **null** model scores, actual input/output/cached tokens,
provider/model/settings, monetary cost, model/end-to-end latency, CPU duration,
peak RSS and assertion-per-token efficiency. No estimate substitutes for usage.
The separate synthetic scoring controls demonstrate the rubric only:

| Synthetic response | Fidelity | Penalties / invalidity |
| --- | ---: | --- |
| Perfect | 1.00 | 0 |
| One omitted assertion | 0.75 | 1 omission |
| One invented assertion | 0.75 | 1 invention |
| Contradictory typed value | 0.75 | 1 wrong relationship |
| Lost provenance, changed event time, lost confidence | 0.25 | 3 category penalties |
| Malformed text | null | malformed=1; invalid structured response |
| Wrong comprehension and application answers | 0.50 | 2 wrong answers |

These response controls are explicitly synthetic; their telemetry and
assertion-per-token efficiency are null. The reference ledger and expected answers
are hand-authored independently of the encoder; they still need independent
technical review. Exact JSON answer scoring is intentionally strict and cannot
evaluate unrestricted paraphrases, source entailment or physical truth. Imported
real telemetry is a future caller-attested/unverified boundary; **no real trial is
included or run**. The comprehension/efficiency hypothesis remains untested.

Final validation includes 37 integration tests (26 retained, 11 focused golden
regressions), formatter, Clippy with warnings denied, unchanged release smoke,
three-case and six-case checks, 48/96-row matrices, replay/export/request commands,
independent CSV/hash/result validation and whitespace checks. The new tests cover
perfect, omitted, invented, contradictory, partial, malformed and wrong answers;
duplicate/unknown data, independent clocks/provenance/uncertainty, zero/missing
usage/score denominators, import errors, telemetry constraints, all arms and CLI
exports. See the exact final commands and exit codes in the validation log.

Initial environment setup needed the existing Cargo/Rustup homes and PATH to be
selected explicitly. An initial compile caught an export borrow conflict and an
unused import; both were fixed before final validation. No new dependencies,
Rusty Mill imports, license choice, packages, access settings or downstream work.
Remote CI and independent exact-head review are separate coordinator gates.

---

The following #792 and #790/#791 sections are historical run records. Their
statements about #793 being undelivered apply to those earlier checkpoints.

# #792 offline codec/adapter result — 2026-11-07

Three codecs and sixteen adapter/control combinations preserve the complete same
original smoke fixture, including assertion identity, provenance, independent times,
uncertainty and evidence metadata. Exact evidence-byte hashing also passes. These
are structural checks, not comprehension/application scores or an efficiency proof.

[Raw JSONL](results/codec-adapter-smoke.jsonl), [validation](results/codec-adapter-validation.log),
[environment](results/codec-adapter-environment.txt), [boundaries](MEASUREMENT.md)
and [formats/adapters](WIRE-FORMATS.md) make the run reproducible.

```sh
cargo fmt --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo run --release --locked > docs/results/codec-adapter-smoke.jsonl
git diff --check
```

One release process on Rust 1.98.1/Linux x86_64; 100 warm-ups and 1,000 timed
iterations per named operation. Totals are in the JSONL; means below truncate
integer nanoseconds. Allocations, validation and result destruction are timed.
CBOR/CSR decode includes validated re-encoding/profile comparison. Fixture loading,
original-evidence hashing, I/O and compilation are outside all timed loops.

| Codec | Payload bytes | Encode/validate mean ns | Decode/validate mean ns |
| --- | ---: | ---: | ---: |
| json | 994 | 3477 | 9120 |
| cbor | 779 | 3604 | 19959 |
| records | 372 | 1151 | 5216 |

Every final envelope below is UTF-8. Semantic CBOR is 1,040 base64 bytes and
semantic CSR/1 is 496 base64 bytes before JSON wrapping. Cold schemas are actually
included in each semantic envelope; raw schema bytes and escaping/expansion are
counted in the JSONL. No schema-cache or warm/amortized cost is claimed.

| Context | Arm | Final envelope bytes | Full render mean ns | Full parse mean ns |
| --- | --- | ---: | ---: | ---: |
| human_model | conventional | 1186 | 17542 | 33268 |
| human_model | semantic json | 3091 | 11639 | 29921 |
| human_model | semantic cbor | 2855 | 8076 | 31428 |
| human_model | semantic records | 3532 | 7483 | 18240 |
| model_tool | conventional | 1086 | 18075 | 43795 |
| model_tool | semantic json | 2797 | 7153 | 23432 |
| model_tool | semantic cbor | 2880 | 7019 | 31452 |
| model_tool | semantic records | 3556 | 5251 | 14310 |
| model_model | conventional | 1197 | 17006 | 41081 |
| model_model | semantic json | 3091 | 12451 | 32707 |
| model_model | semantic cbor | 2855 | 8659 | 33517 |
| model_model | semantic records | 3532 | 7515 | 17752 |
| model_data | conventional | 1041 | 17798 | 43844 |
| model_data | semantic json | 2752 | 7207 | 22381 |
| model_data | semantic cbor | 2835 | 6788 | 31250 |
| model_data | semantic records | 3511 | 5469 | 13597 |

Adapter render/parse times include codec work, cold schema construction, transfer
rendering, envelope handling and validation/reconstruction. They are combined local
**translation** costs; do not add them to codec means or subtract independent means
to infer an isolated adapter-only cost. Conventional human content is a labelled
fact sheet; the other conventional arms use structured JSON. The structured
conventional and semantic JSON payloads coincide; semantic packets add explicit
encoding/transfer/schema metadata. Full content is equal across all arms.

Compact transport bytes do not predict the final request size: every cold semantic
envelope here is larger than its conventional control. The result does not support
a model inference-efficiency claim. Fixed order, one tiny case, warmed caches and
no confidence intervals also prevent general comparative performance conclusions.

All model tokens, monetary cost, semantic fidelity, assertion-per-token efficiency,
CPU duration, memory, model latency and end-to-end latency remain **null**. No
model, MCP server/session, retrieval operation, binary attachment or hidden-state
interface was exercised. Model-native is explicitly unsupported/not testable.
Outside-probe protocol overhead and model comprehension of schemas/base64 are
unmeasured. A Rust round trip proves neither model understanding nor calibration.

Local validation: 26 integration tests (14 retained, 12 new), formatter, Clippy
with warnings denied, release harness and whitespace checks. The new tests cover
every object variant and method, independent literal CBOR/CSR bytes, all smoke
payload truncations, duplicate/unknown nested fields and records, scalar tags,
ordering, bounds, invalid contract edits and all offline adapter/baseline pairs.
The initial regression run exposed CBOR null arriving through visit_none; this
was fixed. A Clippy chunks_exact warning was corrected before final checks.
The initial shell lacked Cargo on PATH; the existing configured toolchain was
used explicitly. Final commands and exit codes are retained in the log.

No #793 golden suite or #794 model pilot is delivered. rusty_protobuf remains an
optional deferred fourth baseline; no Rusty Mill code/dependency or license choice
changed. Local validation is not remote CI or independent approval. The coordinator
must inspect the actual PR head, its CI and independent review before merge.

## Historical #790/#791 measurements

# Historical #790 offline architecture smoke result — 2026-11-07

The probe succeeds on one synthetic observation with two assertions. This establishes
an executable architecture boundary and exact structural preservation. It does
**not** establish semantic understanding or support the efficiency hypothesis.

Resumed-run [raw JSONL](results/offline-smoke.jsonl), generated with
`cargo run --release --locked`, Rust 1.98.1, Linux 6.18.57 x86_64 GNU/Linux,
managed virtual cloud environment. Dependencies are pinned in `Cargo.lock`.

| Measurement | Recorded value |
| --- | --- |
| Compact semantic JSON payload | 994 bytes |
| Pretty JSON formatting control, same object | 1,583 bytes |
| Warm-up iterations | 100 |
| Timed iterations per operation | 10,000 |
| Validate + clone + sort + encode, mean wall time | 2,987 ns |
| Same operation, total | 29,876,927 ns |
| Decode + validate + drop, mean wall time | 3,928 ns |
| Same operation, total | 39,288,981 ns |
| Exact contract equality after round-trip | true |
| Original evidence digest matches | true |

The encoding boundary includes validation, cloning/sorting and allocation. The
decode boundary includes parsing, allocation, validation and destruction. Evidence
digest verification and fixture construction are outside both timed loops. Means
are truncated integer nanoseconds. This is one warmed process and one data point,
without a confidence interval, CPU-time measurement or isolated-system guarantee.
Timings vary; an earlier warm process produced 3,706/3,807 ns respectively. These
are calibration values, not performance budgets or regression gates.

The byte difference is JSON whitespace only, not a conventional-vs-universal
comparison or a CBOR result. Model input/output tokens, model and end-to-end
latency, CPU time, peak memory, model cost, adapter overhead and fidelity scores
are explicitly null. No endpoint/model invocation, paid call, or pricing estimate
was made. No five-context pilot or golden-case suite was run.

Communications validation: 6 integration tests pass, formatter passes, Clippy with
warnings denied passes. Reference-source checks: 19 codec and 51 Protobuf tests
pass in isolated copies; other application suites were not run. See the exact
commands and results in [validation log](results/validation.log).

Initial `cargo test --offline` failed because the selected dependency cache lacked
`block-buffer v0.10.4`. Normal locked Cargo fetching from permitted package hosts
succeeded; tests then passed. This was dependency availability, not a test failure.
Remote CI is checked separately on the published PR head; local checks alone do
not establish remote CI success. See Mill Board #790 for current run evidence.
The [resumed validation log](results/resumed-validation.log) records checks on
the final source prepared against the actual README baseline.


#791 changes validated decode to reject duplicate keys/references and perform
semantic validation during deserialization. The table above records the original
#790 decoder and must not be presented as timings for the new decoder. Current
#791 validation and smoke output are in [contract validation](results/contract-validation.log).
No comparative performance or semantic-fidelity claim follows from either run.

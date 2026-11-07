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

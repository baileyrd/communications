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

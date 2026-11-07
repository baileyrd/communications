# Verified Rusty Mill reuse assessment

Inspected current `main` at **70013fd26cb1214d5543379f20d79c6d239df972** on
2026-11-07. The mounted checkout was older
(`53b2f5286913f6db68dc93faea3c1d0938a2c0f9`); 53 selected source/test/license
files fetched at the current commit were byte-identical to it. The mount was
read-only throughout. [Source manifest](source-manifest.json) pins file SHA-256s.
Paths below are relative to
[that source commit](https://github.com/Rusty-Mill/rusty_mill/tree/70013fd26cb1214d5543379f20d79c6d239df972).

| Candidate | Actual source/test evidence | License and dependency cost | Decision and reason |
| --- | --- | --- | --- |
| `crates/foundation/rusty_codec` | `src/binary.rs`: `serialize(&[u8])` writes a 4-byte LE u32 length then bytes; `deserialize` copies those bytes and allows trailing bytes. `roundtrip_binary` and two bound/signature tests; TOML parser is separate. | Package MIT OR Apache-2.0; root LICENSE-MIT names David Bailey (2026), root Apache text present. No dependencies; alloc/no_std. | **Reference/reject as semantic codec.** It is a byte framer, despite the `rusty_bincode` label in `src/lib.rs`; it does not serialize arbitrary Rust structs, define CBOR, or compress semantic content. No import. |
| `crates/libs/protocol/rusty_protobuf` | `src/{reader,writer,varint,wire}.rs`; bounds-checked zero-copy wire reader, length-fixing writer, default nesting limit 100; hand-written schemas. `tests/{bounds,limits,golden,writer}.rs` cover hostile bytes, type mismatches, nesting, scalar/repeated fields. Recorded Python protobuf fixtures in `tests/fixtures/mod.rs`. Unknown fields can be skipped and lost; repeated scalar semantics are last-wins. | Explicit MIT OR Apache-2.0; root texts verified. No runtime dependencies; no code generation. Schema implementation is still required. | **Possible later reuse/adapt**, optional fourth transport baseline only. Do not replace CBOR or import now. Explicit duplicate/unknown-field handling would need mapping to the contract's stricter rules. |
| `crates/libs/protocol/rusty_mcp_proto` and `docs/research/MCP-NATIVE-PLAN.md` | `src/lib.rs`, `rpc.rs`, `codec.rs`, tools/content/lifecycle modules implement JSON-RPC/MCP vocabulary, no transport. `tests/rpc.rs` checks ids, requests, responses, malformed messages; `tests/oracle.rs` is rmcp comparison. Unknown members of known types may be dropped. The plan contains dated proposals as well as progress, not independent proof of runtime support. | MIT OR Apache-2.0. Production path dependency `rusty_json` with std; tests add workspace rmcp plus serde/serde_json. Full server/client/TLS stack is separate. | **Reference** envelope boundaries and test cases for future model↔tool context. Do not import the whole MCP stack or bind the semantic core to MCP vocabulary. No native model representation access established. |
| `crates/apps/nexus/crates/nexus-memory/src/semantic.rs` | `SemanticEntry` is key/content text (or serialized JSON), tags, session, stored time. `search` uses lowercase substring matching; `store` replaces matching keys. Inline tests cover retrieval, last-writer-wins, case-insensitive search, limits, deletion and tags. Optional SQLite backing. | Inherits workspace MIT OR Apache-2.0. `Cargo.toml` pulls Nexus kernel/plugin crates, database/pool, chrono, uuid, HTTP, serde and other workspace dependencies. | **Reference/reject as contract.** Neither typed relationships nor assertion-level event/interpretation time and evidence/uncertainty semantics are supplied. Importing it would couple independent Communications to an app/storage stack and replacement semantics would lose history. |
| `crates/apps/rusty_multimodal_db` benchmark material | `docs/specifications/storage/STORAGE-003-benchmark-suite.md` describes matched seeded datasets, setup outside timed operations, rotation of lookup targets, Criterion groups and optional hardware counters. Current `benches/generic_production.rs` and `src/{bench_support,generic_spike/order_bench_support}.rs` actually show matched dataset helpers, fixed seed/rotating targets, black_box and groups. Historical `workloads.rs`/`cache_events.rs` named in the specification are absent in current benches; current Cargo targets are `generic_production` and `server`, feature-gated. | Package declares **MIT**, not the workspace's dual license. Storage types, mmap/server/research features and Criterion dependencies make whole-harness import costly. | **Adapt methodology/reference only.** Use identical inputs and explicit timing boundaries; do not import storage backends or claim the historical 36-case suite is presently available. Hardware counters are unavailable in this probe. |

No Rusty Mill code or substantial text was copied into Communications; no
attribution-bearing code is redistributed. If later reused, preserve the selected
license text and copyright notice, verify any per-file exceptions and third-party
fixtures, and pin/review the dependency boundary. The repository owner still needs
to choose Communications' own license before redistribution expectations are set.

## Checks performed

The selected source snapshots were copied outside the incomplete snapshot
workspace to `/tmp/communications-reference-tests`, without changing their code:

```sh
cargo test --manifest-path /tmp/communications-reference-tests/rusty_codec/Cargo.toml --offline
cargo test --manifest-path /tmp/communications-reference-tests/rusty_protobuf/Cargo.toml --offline
```

Both exit 0: **19** codec tests; **51** Protobuf tests (7 unit, 44 integration).
The fixtures are recorded oracle data; the Python oracle was not rerun. An initial
attempt inside `/workspace/reference-main` failed because its root workspace
manifest listed members not fetched in this partial inspection snapshot; isolated
copies resolved that tooling issue. MCP, Nexus and database suites were inspected
but **not run**: their workspace/application graphs were not imported or built.

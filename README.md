# communications

Rust-first M01 experiment for a shared semantic contract across communication
contexts, contributing to [Issue #1](https://github.com/baileyrd/communications/issues/1).
#790 established the architecture/reuse probe; merged #791 added the validated
contract; merged #792 adds JSON, genuine CBOR, CSR/1 and four offline context
boundaries. #793 adds six pinned golden cases, independent reference ledgers,
structured reconstruction/comprehension/application scoring, replay/import and
compatible JSONL/CSV exports. The full M01 milestone is still incomplete.

```sh
cargo fmt --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo run --release --locked
git diff --check
```

With no arguments, the CLI retains the #792 smoke and emits JSONL: three codec rows, sixteen adapter rows (four contexts × four
arms), and five capability rows. It uses the original synthetic two-assertion smoke
fixture; verifies the exact evidence digest, complete structural round trips and
final UTF-8 envelopes; measures payload/envelope/schema bytes and local wall time.
Binary payloads use counted base64 in text envelopes. Actual model tokens, cost,
semantic fidelity, CPU time, memory and model/end-to-end latency remain null.
No model or network calls occur in the experiment.

```rust
use communications_probe::{codec::Encoding, adapter::{self, Arm, Context}, fixture};
fn main() -> Result<(), String> {
let facts = fixture();
let wire = Encoding::Cbor.encode(&facts)?;
let recovered = Encoding::Cbor.decode(&wire)?; // validated, strict local CBOR profile
let envelope = adapter::render(Context::ModelTool, Arm::Semantic(Encoding::Cbor), &facts)?;
let parsed = adapter::parse(Context::ModelTool, Arm::Semantic(Encoding::Cbor), &envelope)?;
Ok(())
}
```

These are offline envelopes, not working MCP/model/retrieval endpoints. Model-native
tensors/hidden states are unsupported/not testable. Structural preservation is not
comprehension or efficiency evidence; a compact binary payload can become a larger
text envelope once base64, schema and escaping are included. #794's model pilot and the optional Protobuf baseline remain deferred.


Run the offline golden suite and explicit synthetic scoring controls:

```sh
cargo run --release --locked -- golden check --initial
cargo run --release --locked -- golden check
cargo run --release --locked -- golden run /tmp/golden.jsonl /tmp/golden.csv
cargo run --release --locked -- golden sample > /tmp/synthetic.jsonl
cargo run --release --locked -- golden replay /tmp/synthetic.jsonl /tmp/replay.jsonl /tmp/replay.csv
```

The full run emits 96 rows (six cases × four contexts × four arms). Each counts
actual final local request bytes with the same questions and output schema,
checks reference facts and structural preservation, and measures named local
operations. Model/resource telemetry and model scores remain null. Replay scores
use explicit expected JSON answers; synthetic controls are never model results.
`golden request CASE CONTEXT ARM` emits the exact request for inspection/hash
verification. Imported real telemetry is caller-attested and labeled unverified.
See [commands, fixtures, rubric and import/export schema](docs/GOLDEN-HARNESS.md).

Read [architecture](docs/ARCHITECTURE.md), [wire/adapters and limits](docs/WIRE-FORMATS.md),
[reuse evidence](docs/REUSE.md), [measurement boundaries](docs/MEASUREMENT.md),
[results](docs/RESULTS.md) and [checkpoint](docs/PROJECT-STATUS.md).
Dependencies are locked; new encoding dependencies are exactly pinned. The checked
validation uses Rust 1.98.1 on Linux x86_64.

No repository license has been selected by the owner. `publish = false` prevents
accidental package publication; it does not grant a license. No Rusty Mill code is
copied into this package or imported as a dependency.

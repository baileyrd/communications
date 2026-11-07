# communications

Rust-first M01 experiment for a shared semantic contract across communication
contexts, contributing to [Issue #1](https://github.com/baileyrd/communications/issues/1).
#790 established the architecture/reuse probe; merged #791 added the validated
contract. This #792 slice adds JSON, genuine CBOR and custom typed records, four
offline adapter boundaries, matched conventional controls and measured local
encoding/translation costs. The full M01 milestone is still incomplete.

```sh
cargo fmt --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo run --release --locked
git diff --check
```

The CLI emits JSONL: three codec rows, sixteen adapter rows (four contexts × four
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
text envelope once base64, schema and escaping are included. #793's golden cases,
#794's model pilot and optional Protobuf baseline remain deferred.

Read [architecture](docs/ARCHITECTURE.md), [wire/adapters and limits](docs/WIRE-FORMATS.md),
[reuse evidence](docs/REUSE.md), [measurement boundaries](docs/MEASUREMENT.md),
[results](docs/RESULTS.md) and [checkpoint](docs/PROJECT-STATUS.md).
Dependencies are locked; new encoding dependencies are exactly pinned. The checked
validation uses Rust 1.98.1 on Linux x86_64.

No repository license has been selected by the owner. `publish = false` prevents
accidental package publication; it does not grant a license. No Rusty Mill code is
copied into this package or imported as a dependency.

# communications

Shared repository for communications and collaboration.

## M01 architecture probe

Executable, provisional architecture and reuse slice for Mill Board **#790**,
contributing to [M01 issue #1](https://github.com/baileyrd/communications/issues/1).
This draft architecture slice is prepared for independent review.

```sh
cargo fmt --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo run --release --locked
```

The CLI emits one JSONL row for a synthetic two-assertion smoke fixture. It checks
typed references, provenance, uncertainty, distinct times, exact JSON round-trip,
and the original evidence's SHA-256. It measures bytes and local wall-clock work;
unavailable model, cost, memory, adapter, and comprehension metrics remain null.
There are no API calls. The original input and its content address are checked in.
Dependencies are locked; the measured toolchain is Rust 1.98.1 on Linux x86_64.

Read [architecture](docs/ARCHITECTURE.md), [reuse evidence](docs/REUSE.md),
[measurement plan](docs/MEASUREMENT.md), and [results](docs/RESULTS.md).
[Status](docs/PROJECT-STATUS.md) records remaining work and the publication handoff.
The full contract, adapters, golden-case harness, and pilot belong to #791–#794.

No repository license has been selected by the owner. `publish = false` prevents
accidental package publication; it does not grant a license. No Rusty Mill code is
copied into this package. Its sources were inspected and tested separately.

# Communications execution guide

Read `CHATGPT_WORKFLOW.md` and `docs/PROJECT-STATUS.md` before work. Mill Board is
the authority for scope, dependencies and execution progress; GitHub's actual
diff, head-specific tests, independent review and merge establish delivery.
Draft architecture documents are proposals, not owner-approved decisions.

Keep the contract independent of model, MCP, storage and Rusty Mill dependencies.
Preserve evidence and measurement boundaries. Never fabricate model measurements.
Respect the current card's scope; #790 is not the full M01 milestone.

Run `cargo fmt --check`, `cargo test --locked`,
`cargo clippy --all-targets --locked -- -D warnings`,
`cargo run --release --locked`, and `git diff --check` from this root.
Update relevant docs with behavioral or measurement changes. No merge, formal
approval, spending or credential/access changes are authorized for this worker.

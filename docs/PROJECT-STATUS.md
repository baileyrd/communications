# Communications implementation checkpoint

2026-11-07: #790 and #791 are Done. Verified current main at the start of #792 is
`f6cbefffdcfe43efb81748e0c29233b63aea9dcc` (PR #3 merged). PR #2 delivered the
architecture/reuse probe; PR #3 delivered the fail-closed semantic contract.
Do not repeat the historical #791 worker checkpoint or restart completed work.
Mill Board is authoritative for scope, dependencies and execution progress.

#792 implementation is on `codex/m01-792-codecs-adapters`, based on that verified
main, in a separate Communications checkout. It retains the exact smoke source
and #791 contract, and adds validated JSON, a strict genuine CBOR profile, custom
CSR/1 typed records, four offline context render/parse boundaries, matched
conventional controls, explicit model-native unsupported capability, byte counts
and measured combined local translation spans. See [wire profiles](WIRE-FORMATS.md),
[measurement boundaries](MEASUREMENT.md), [results](RESULTS.md),
[raw JSONL](results/codec-adapter-smoke.jsonl) and
[validation](results/codec-adapter-validation.log).

Cloud thread `01a1b74b-9c23-711e-8d0f-cd6c9595f017`, initial turn
`01a1b74b-a6b0-77ff-97b6-7687603dd9f8`. Refresh #792 comments for the actual
published PR, exact branch/base/head, verified tree and current head-specific CI.
The worker prepares one focused draft PR referencing #792 and Issue #1, and does
not approve, merge or mark Done. Independent technical review, required CI and
unchanged reviewed head remain coordinator gates. Corrections update the same
branch/PR. Local pass reports are not merge or remote-CI claims.

Known board defect: fixed authenticated `ChatGPT` actor cannot claim because the
store requires lowercase names. No claim retry/identity override was made.
Supported Planning → Architecting → Building transitions succeeded. Concrete cloud
ids are recorded in comments; incompatible Orch/assignment IDs are not invented.
Connected GitHub tools are used for publication; previously forbidden `gh api`
routes are not retried. The mounted Rusty Mill repository remains read-only.

Not delivered: #793's three-then-six golden suite, #794's model pilot or any model
comprehension/fidelity claim; live model/MCP/retrieval endpoints; model-native
hidden states/tensors; optional fourth Protobuf baseline; durable dispatch #798;
M02–M04. No credentials/access/security changes, spending, publication, unrelated
deployment or license choice. No Rusty Mill code or workspace dependency imported.

Next: coordinator obtains genuinely independent exact-head review of #792 and
required head-specific CI; verifies merge before Done/advancement. Resume by
refreshing current main, open PRs, #792 comments/execution and dependency-eligible
board cards, then `AGENTS.md`, `CHATGPT_WORKFLOW.md` and this checkpoint. Preserve
this execution and reconcile any existing branch/PR instead of competing with it.

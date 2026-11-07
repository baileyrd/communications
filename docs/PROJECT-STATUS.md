# Communications implementation checkpoint

2026-11-07: #790, #791 and #792 are Done. Verified current main at the start of
#793 is `50a4621d2683856287cf0052d41fb0a01356c5fb` (PR #4 merged), with tree
`c611cb633abc6d58617f6ea73015f0797b40d945`. PR #2 delivered architecture/reuse;
PR #3 delivered the validated independent contract; PR #4 delivered genuine CBOR,
CSR/1 and four offline adapter boundaries. This supersedes the historical #792
awaiting-review checkpoint. Do not restart completed cards. Mill Board is
authoritative for scope, dependencies and execution progress.

#793 implementation is on `codex/m01-793-golden-harness`, based on that verified
main in a separate Communications checkout. It adds six pinned golden sources,
independently authored entity/relation/fact ledgers, separate manual contract
extractions, explicit structured reconstruction/comprehension/application rubrics,
deterministic scoring, strict replay/import, complete local requests with common
questions/output schema, compatible JSONL/CSV and local timing measurements.
The first three cases were validated first; the same schema expands to six, with
no three-versus-six deferral. Existing smoke originals/evidence/results and the
#791/#792 contract, codecs and adapter implementations remain unchanged.

PR #5 independent review of initial head
`7cd665a2d27f22f3a9f1c0073031432b2f3c57b7` found that the shared integer-only
JSON visitor rejected valid fractional replay telemetry/settings. The same branch
now has separate replay-only finite-number ingress with recursive duplicate-key
checks; core contract, fixture and scored-response ingress is unchanged. Four
new regressions cover fractional CLI/export success, exact validation errors,
integer restrictions and nested duplicate keys (41 total integration tests).
See [correction validation](results/golden-correction-validation.log). The prior
local timing artifacts are retained; the correction does not change requests,
fixture content, scoring rubrics or measured codec/adapter operations. Refresh
#793/PR #5 for the correction commit/tree and new-head CI; independent re-review
is required and no previous review is approval of the changed head.

See [harness commands/rubric/import/export/boundaries](GOLDEN-HARNESS.md),
[architecture](ARCHITECTURE.md), [measurement](MEASUREMENT.md),
[results](RESULTS.md), [raw golden JSONL](results/golden-offline.jsonl),
[CSV](results/golden-offline.csv), [synthetic controls](results/golden-synthetic-replay.jsonl),
[validation](results/golden-validation.log) and [hashes](results/golden-sha256.txt).
Model telemetry and model scores are null in all 96 local rows. The seven replay
rows are explicitly synthetic scorer controls, never actual model output.
Imported future telemetry is caller-attested/unverified, not evidence that this
worker ran a model. No paid/model/credential/endpoint calls occurred.

Cloud thread `01a1b762-6075-72bb-8484-5115e845903d`, initial turn
`01a1b762-69c7-7097-a34a-811e592b535d`. Refresh #793 comments for the actual
published draft PR, exact base/branch/head/tree and live head-specific CI. The
worker prepares exactly one draft PR referencing #793 and related Issue #1,
without closing the full M01 issue. Independent exact-head technical review,
required CI and an unchanged reviewed head remain coordinator gates; local pass
reports do not establish them. Corrections stay on the same branch/PR. This
checkpoint is implementation evidence, not approval/merge/Done evidence.

Known board claim defect: authenticated `ChatGPT` does not satisfy the store's
lowercase actor constraint. No claim retry or identity override was made. Supported
Planning → Architecting → Building transitions succeeded. Concrete cloud ids are
in comments; incompatible Orch/assignment ids are not invented. Connected GitHub
tools publish the draft; previously forbidden `gh api` routes are not retried.
Mounted Rusty Mill is read-only reference and has no code/workspace dependency here.

Not delivered: automated source extraction; actual model comprehension, application,
inference efficiency or endpoint comparisons; #794's live comparative pilot;
MCP sessions, retrieval, model-native hidden states/tensors; optional Protobuf;
restart-safe durable dispatch #798; full M01 or Issue #1 completion; M02–M04.
No security/access/credential changes, spending, package publication, deployment,
license selection or unrelated work. The hypothesis remains untested by a model.

Next: coordinator reviews the actual #793 draft head independently and checks
required head-specific CI before any merge/Done/advancement. Do not launch #794
from this worker. Resume by refreshing current main, open PRs, #793 execution and
comments, dependencies, then `AGENTS.md`, `CHATGPT_WORKFLOW.md` and this checkpoint.
Preserve and reconcile the existing branch/PR/execution rather than duplicate it.

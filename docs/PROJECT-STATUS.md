# Communications implementation checkpoint

2026-11-07: #790–#793 verified Done by coordinator. Current #794 base main is
`ae9dafe30526bc6925a23031e5b9a3fe39ae254a` (merged PR #5), tree
`535c3c8611bbeb552a876785061169fd7f22282f`. #793's fractional replay correction
was independently re-reviewed and exact-head CI passed before merge. This
supersedes the historical pending-correction checkpoint. Preserve completed work.
Mill Board is authoritative for scope, dependencies and execution progress.

#794 implementation branch: `codex/m01-794-offline-pilot`, separate Communications
checkout; mounted Rusty Mill untouched. Cloud thread/execution
`01a1b781-0ecc-76c5-87d1-1cd98f4c303c`, initial turn
`01a1b781-180b-767f-be0a-c3fb6a7a7946`. Actual ids are recorded in board comments;
no incompatible Orch/assignment values invented. Supported in-progress transitions
succeeded; known lowercase-store claim defect was not retried or impersonated.

Delivered locally for review: one selected-trial CLI using existing timing code;
a standard-library seeded serial fresh-process driver; five repetitions over six
cases/four contexts/four arms; 480/480 successful attempts, zero execution errors;
compact raw JSONL/CSV, 96-row invariant matrix, manifest/input/output hashes,
descriptive distributions, audit and focused regression tests. All fixtures and
original smoke/golden artifacts are unchanged. See [pilot report](PILOT.md),
[measurement](MEASUREMENT.md), [results](RESULTS.md),
[validation](results/pilot-validation.log). All 41 prior integration tests retained;
one new Rust test and three Python tests cover selected CLI/schedule/export/
aggregation/error-denominator boundaries. Current required checks are recorded
in the validation log; local success is not independent approval or remote CI.

Runtime metadata reported no configured secret/endpoint-variable/outbound identity.
No already-authorized free model endpoint was established. Explicit offline-only
fallback used; no model/paid calls, imported model records, credentials, grants,
security changes or endpoint integration. Model fidelity, actual tokens/cost,
model/end-to-end latency, CPU/RSS and assertion-per-token efficiency remain null.
Model-native remains unsupported/not testable. Smaller binary payloads produce
larger cold local requests here; understanding/inference hypothesis is untested.

No #795 five-layer work, new ontology, source extraction, model judges, optional
Protobuf, production endpoints, deployment, publication or license decision.
Full M01/Issue #1 completion and restart-safe dispatch #798 are not established.

Next: refresh #794 comments, actual branch/PR/head/tree and CI. Worker publishes
exactly one draft PR referencing #794 and Issue #1; independent exact-head review,
required CI and unchanged reviewed head remain coordinator gates. Worker must not
approve/merge/mark Done or launch downstream cards. Corrections stay on this same
branch/PR. Do not duplicate this execution or restart completed cards.

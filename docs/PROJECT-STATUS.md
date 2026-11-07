# Communications draft checkpoint

2026-11-07: this focused #790 branch is based on actual remote main
`d29af7a6dc3eb7aa5e07b77da8414611f586992f` (the owner-requested README). Its
three-line introduction is preserved. This contributes to
[M01 issue #1](https://github.com/baileyrd/communications/issues/1); it does not
complete that issue, this card or any downstream dependency.

Implemented: provisional typed semantic architecture, validation, SHA-256 evidence
check, stable probe JSON encoding, one synthetic fixture, 6 integration tests,
offline JSONL smoke, measurement plan and source-pinned reuse assessment.
No Rusty Mill code was edited or imported. See [results](RESULTS.md), the exact
[validation log](results/validation.log), and [architecture limits](ARCHITECTURE.md).

Mill Board is the shared execution source of truth. Cloud thread
`01a1b71a-fe07-7182-9e33-465f5afb6033`, initial turn
`01a1b71b-0763-76c5-8d2a-e3f75be96409`. The resumed turn's identifier was not
exposed to the worker at preparation time; the coordinator can record it from
task metadata. Refresh #790's comments for live PR/head/CI status.

Publication access changed after the first local checkpoint
`d19c851d58bea599cdbebdbe28a5dc142dfbc8d2`. The collaborator invitation was
accepted and the requested README baseline established. On resume, connected
GitHub tools independently verified identity `dottiusmaximus`, push=true,
pull=true and triage=true, current main and no open PRs. Publication uses those
supported connected tools. An ordinary public Git fetch of main also succeeded.
Earlier `gh api` probes had returned `Forbidden` without response details; their
rejecting layer and separate CLI identity remain unknown. They were not retried,
and no credentials, permissions or network policy were changed by this worker.
The prior private artifact is not needed to reconstruct or publish this branch.

Board claiming remains separately blocked: fixed authenticated actor `ChatGPT`
is rejected with `invalid input: with a rusty_tick store, actor names must be
lowercase`. No actor override or repeated claim attempt is justified by the
access change. The coordinator moved the card to Blocked after the first local
run; subsequent transitions must follow the actual board rules. Comments,
not an invented execution-link identifier, record worker progress.

Remaining: remote head-specific CI, the user's architectural and independent
technical review, corrections and verified merge. Local success or PR creation
does not establish completion. No merge or formal approval is authorized for this
worker. #791–#794 (full contract, encodings/adapters, golden suite, pilot) and
M02–M04 remain unstarted by this worker. Owner license choice remains open.
The draft vocabulary and uncertainty boundary await architectural review.

Resume: refresh Mill Board #790–#797, current GitHub main, active PRs and exact
head-specific CI; read `CHATGPT_WORKFLOW.md`, `AGENTS.md` and this checkpoint.
Correct the existing PR branch when needed; do not start competing work or
release downstream dependencies until independently verified review and merge.

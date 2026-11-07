# Communications implementation checkpoint

2026-11-07: #790 is Done. GitHub PR #2 merged into main at
`1b6997845aea7d0770cc8e3101d1759ec866f66f`; #791 starts from that verified
checkpoint, preserving its architecture, fixture, evidence digest and README.
Mill Board remains authoritative for scope, dependencies and execution progress.

#791 implements the minimal versioned semantic contract on branch
`codex/m01-791-contract`: validated, duplicate-rejecting JSON ingress; typed
entities, assertions and relationships; provenance; separate explicit event,
record and interpretation times; uncertainty; exact original-byte verification;
and deterministic validated canonical serialization. The existing `0.1-draft`
wire version remains readable. Malformed inputs previously silently normalized
or accepted can now fail. No new dependencies, Rusty Mill edits or imports,
model/MCP/storage adapters, paid calls or access changes were introduced.

See [architecture and limits](ARCHITECTURE.md), [contract validation](results/contract-validation.log)
and `tests/contract.rs`. Existing #790 measurement logs remain historical;
[results](RESULTS.md) distinguishes them from the stricter #791 decoder.
Round trips prove structural preservation, not model semantic fidelity.

Cloud thread `01a1b73d-a74d-76a0-80a3-37dda7fe9013`, initial turn
`01a1b73d-b259-7662-ada8-6c051c347297`. Refresh #791 comments for the actual
published PR/head and head-specific CI. No merge, formal approval or Done is
claimed by this worker. Independent technical review, required CI and unchanged
reviewed head are coordinator gates; corrections must update the same PR.

Known board defect: fixed authenticated `ChatGPT` actor cannot claim because the
store requires lowercase names. No identity override or repeated claim attempt
was made. Supported Planning → Architecting → Building transitions succeeded.
Comments record concrete execution identifiers; incompatible Orch IDs are not
fabricated. Connected GitHub tools provide publication; previously forbidden
`gh api` routes were not retried.

Next: coordinator review and verified merge of #791, then select the next
Mill Board dependency-eligible card. #792 encodings/adapters and #793 golden suite
remain outside this increment, as do the pilot and M02–M04. Owner license choice
remains open. Resume with current main, active PRs, board #791–#797, `AGENTS.md`,
`CHATGPT_WORKFLOW.md` and this checkpoint; preserve work already completed.

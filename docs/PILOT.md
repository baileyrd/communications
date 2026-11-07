# #794 repeated offline pilot — 2026-11-07

**The model comprehension/inference-efficiency hypothesis remains untested.**
The offline fallback completed 480/480 local attempts with zero execution errors:
six pinned synthetic cases × four offline contexts × conventional/JSON/CBOR/CSR/1
× five repetitions. Every trial preserved the complete contract and independent
reference ledger. Actual model semantic accuracy, correct assertions per 1,000
actual input tokens, input/output/cached tokens, cost, model/end-to-end latency,
CPU duration and peak RSS are **null**. No real or imported model trials occurred.
Model-native tensors/hidden states remain unsupported/not testable.

Runtime configuration was inspected through safe metadata: current observations,
enforced restricted policy, no secret bindings, runtime endpoint variables or
outbound identities. Repository clients are offline only. No already-authorized
free endpoint was established; provider/action/budget are unselected/unapproved.
A future live pilot needs a named documented endpoint, credential binding, allowed
network destination, model revision/settings and an explicit call/budget grant.
No secret values, installed coding assistant or model label were treated as access.
No paid/model calls, credentials, grants or security settings changed.

## Reproduce and audit

```sh
cargo build --release --locked
python3 scripts/test-offline-pilot.py
# Output directory must be new; preserves the committed run and earlier evidence.
python3 scripts/offline-pilot.py run /tmp/communications-pilot-new
python3 scripts/offline-pilot.py audit /tmp/communications-pilot-new
python3 scripts/offline-pilot.py audit docs/results/pilot
python3 scripts/validate-golden-results.py
```

[Manifest](results/pilot/manifest.json) records exact commands, Rust 1.98.1,
Python 3.12.14, Linux x86_64, input/source hashes, executable hash and output hashes.
The executable hash identifies this run, not a portable-build requirement.
[Raw JSONL](results/pilot/trials.jsonl) and [equivalent CSV](results/pilot/trials.csv)
retain every attempted trial, actual order, repetition, exit status, error and all
five named timing totals. [Invariant matrix](results/pilot/matrix.json) stores the
96 fixed case/context/arm records once; join on `case_id/context/arm` to recover
request sizes/hashes, verified structure, boundaries and null telemetry for each
successful raw trial. Trial fields override the matrix's omitted trial/timing
fields. This compact artifact set avoids duplicating long invariant strings.
[Summary](results/pilot/summary.json) is regenerated from raw trials by the audit;
[validation](results/pilot-validation.log) records final checks.
Original smoke/golden evidence and all fixtures are unchanged.

Five repetitions use serial fresh release child processes, one per matrix cell.
Python `random.Random(794)` shuffles all 96 cells separately per repetition (MT19937;
actual sequence is persisted, reproducibility is pinned to the recorded Python).
Each child loads/checks the fixtures before timing, then runs the same fixed
operation order: codec encode/decode, adapter render/parse, request render. Each
operation has 3 untimed warm-ups and a single total for 20 timed iterations.
Fixture/source/ledger/hash work, process startup, build, I/O and printing are
excluded. Warm-up and validation include the existing allocation/drop policy.
This is fresh-process **warmed operation** measurement, not cold process latency.

Audit verifies 480 unique ids, exact seeded order and per-repetition coverage,
all CSV/JSONL values, hashes, regenerated exact requests, matching questions and
response schema across arms, payload/base64/schema/envelope/request arithmetic,
nonnegative batch times and null/unmeasured boundaries. Failed attempts remain
raw records and in attempted denominators; none occurred here. Fatal child errors
carry exit/stdout/stderr or timeout details; timings are unavailable if a child
cannot emit a complete observation. No failed attempt is retried or replaced.

## Payloads and cold local requests

Ranges cover the 24 distinct case/context combinations per arm; bytes are
invariant across repetitions. Complete requests include all common instructions,
questions and output schema, cold transport schemas, base64 and nested escaping.

| Arm | Raw payload bytes | Raw cold schema UTF-8 bytes | Adapter envelope bytes | Complete local request bytes |
| --- | ---: | ---: | ---: | ---: |
| conventional | 870–941 | 0 | 923–1144 | 2995–3236 |
| json | 876–941 | 1466 | 2622–3038 | 4694–5118 |
| cbor | 687–748 | 1660 | 2711–2840 | 4783–4931 |
| records | 328–388 | 2876 | 3455–3580 | 5522–5650 |

CBOR base64 content occupies 916–1,000 bytes; CSR/1 occupies 440–520 bytes,
using `4 * ceil(payload_bytes / 3)` before JSON escaping. Semantic JSON transfers
876–941 UTF-8 bytes before escaping. Schema bytes in the table are raw text,
not an estimate of their fully escaped contribution. Message contexts stringify
the whole packet and add another escaping layer. Full envelope/request counts
include those actual bytes plus keys/quotes/metadata; these nested totals must
not be added together. Every semantic cold request is larger than its matched
conventional control. No cached-schema/amortized measurement was performed.
This contradicts a **cold local request byte-saving claim for these six tiny
cases**, while supporting only the narrow observation that CBOR/CSR raw payloads
are smaller. Neither observation answers the model hypothesis. Conventional
controls use the same vocabulary/facts/metadata: labelled human fact sheet and
structured JSON elsewhere; they are not representative natural-language baselines.
Provider/HTTP/session/tool discovery/retrieval overhead and tokenization are absent.

## Repeated local wall times

Below are descriptive distributions of **20-operation batch means**, in ns per
operation. Each non-null cell has n=120 batches (six cases × four contexts × five
fresh processes), five batches per exact combination. These pool heterogeneous
cases/contexts; they are not 2,400 independent operation samples. Raw rows permit
per-combination analysis. P95 uses nearest rank, `ceil(.95*n)`; medians average the
two middle values. Values below round to nearest ns; raw totals remain exact.
No confidence interval, performance guarantee or inference generalization is made.
Scheduler/cache/allocator differences and fixed within-child operation order remain.

| Arm | Span | n | Min | Median | P95 | Max |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| conventional | adapter render | 120 | 14,816 | 17,837 | 26,331 | 28,782 |
| conventional | adapter parse | 120 | 30,365 | 43,053 | 56,664 | 132,128 |
| conventional | complete request render | 120 | 22,067 | 32,861 | 48,195 | 78,618 |
| json | codec encode/validate | 120 | 2,850 | 3,106 | 4,700 | 9,128 |
| json | codec decode/validate | 120 | 8,071 | 8,704 | 12,823 | 24,714 |
| json | adapter render | 120 | 6,226 | 10,632 | 16,615 | 24,684 |
| json | adapter parse | 120 | 20,813 | 30,081 | 39,676 | 99,108 |
| json | complete request render | 120 | 15,706 | 21,119 | 28,518 | 34,215 |
| cbor | codec encode/validate | 120 | 2,130 | 2,287 | 4,486 | 14,310 |
| cbor | codec decode/validate | 120 | 16,983 | 18,875 | 27,296 | 58,312 |
| cbor | adapter render | 120 | 5,582 | 7,495 | 11,249 | 16,290 |
| cbor | adapter parse | 120 | 27,540 | 32,982 | 41,677 | 47,059 |
| cbor | complete request render | 120 | 13,288 | 15,111 | 24,678 | 33,608 |
| records | codec encode/validate | 120 | 1,007 | 1,085 | 1,670 | 5,887 |
| records | codec decode/validate | 120 | 4,484 | 4,834 | 7,134 | 16,842 |
| records | adapter render | 120 | 4,730 | 7,011 | 8,330 | 11,965 |
| records | adapter parse | 120 | 13,110 | 16,899 | 24,585 | 30,842 |
| records | complete request render | 120 | 13,050 | 15,405 | 20,693 | 169,386 |

Standalone codec spans include validation, allocations/drop and strict decode
profile checks. Adapter render includes encoding/schema/transfer/envelope;
adapter parse includes decoding/validation and exact envelope reconstruction.
Request render includes adapter plus common questions/instructions/schema and JSON
serialization. These spans **overlap**: never sum them or subtract independently
measured means to invent isolated costs. They are local elapsed wall times,
not CPU time, model latency or source-to-consumer latency. No benchmark threshold
is enforced. Fresh processes share one host and OS caches; order randomization
does not establish isolated or independent hardware samples.

## Structural and synthetic controls; limits

All 480 local structural checks passed (480 attempted denominator); no model
responses were requested or scored. The existing seven separately labelled
synthetic scorer controls are re-audited: perfect 1.00; omitted, invented and
contradictory assertions each 0.75; lost provenance/time/confidence 0.25;
wrong comprehension/application 0.50; malformed JSON has null fidelity and one
malformed error. Thus 1/7 synthetic controls is deliberately malformed; it is
retained, not a model failure or an offline execution failure. These hand-authored
controls check regression behavior, not semantic accuracy of any model.

Exact structural preservation does not prove source entailment, physical truth,
free-form understanding or sufficient schema comprehension by a receiver model.
Fixture/reference authorship remains synthetic/manual. No extraction, real MCP
session, retrieval, agent exchange, model judge or model-native interface ran.
No actual usage or independent model-response scores exist, so semantic accuracy
and assertion efficiency stay null. Full M01/Issue #1 completion, independent
exact-head review, remote CI and merge remain coordinator gates. #795 is untouched.

## #795 extension and historical audit

The [60-case expansion](EXPANDED-BENCHMARK.md) extends this bounded workflow with
explicit expanded suite selection and per-cell descriptive statistics. All of
this report's six-case measured artifacts remain unchanged. Its manifest pins
source-file bytes, so use `python3 scripts/audit-pinned-pilot.py` on newer code:
it verifies every original input against the merged #794 commit and runs the
unchanged audit with the current binary for request equivalence. The direct
historical command above is valid in the pinned input context; new source bytes
must not be represented as the historical measurement.

# Offline golden harness — Mill Board #793

This bounded M01 implementation is a reproducible local experiment plus an import
boundary. It never invokes a model or opens an evidence locator. The six sources
are authored synthetic scenarios. The first three use the same schema as the
three expansion cases; no three-versus-six capability is deferred.

| Order | Case | Reconstruction distinction | Comprehension / application |
| --- | --- | --- | --- |
| 1 | conflict | Opposed reports, distinct reporters, confidence vs unknown | Identify reporter / refuse certain physical-state resolution |
| 2 | unknown | Unknown event time and destination label; observed seal | Preserve null / refuse unsupported delivery scheduling |
| 3 | ordering | Event order differs from record order; integer report | Select later event / apply event cutoff |
| 4 | provenance | Report and inference remain separate | Identify inferer / filter by lab attribution |
| 5 | relocation | Entity relationship changes over two events | Read location at event / sort by event rather than record time |
| 6 | quoted | Exact Unicode, quotes/newline, negative clocks, zero confidence | Preserve exact text / distinguish zero from unknown |

`fixtures/golden/*.txt` are the immutable originals. `suite.json` pins each exact
SHA-256, the independently authored reference entity/relation kinds, fact ledger,
questions, expected JSON answers and task rubrics. Separate `*.contract.json`
files are manual contract extractions. The ledger is not computed by an encoder,
round-trip decoder or model. Fixture loading hashes the originals without fetching
their locators and compares **every** extracted assertion field, the complete
entity/relation catalogues and evidence identity against this separate reference.
A shared renderer omission cannot erase the expected fact. Changing golden data
requires an explicit source/ledger/extraction review; hashes establish identity,
not authenticity or entailment. These authored references still require the
coordinator's independent review; no human gold-label certification is claimed.

`golden::load` owns fixture loading/reference validation. `golden::score` owns
structured response scoring; it does not use the encoder. `harness` owns trial
requests, timings, telemetry import and exports. Existing core validation, three
codecs and four adapter boundaries remain unchanged. There is no new dependency.
The original `fixtures/contract.json`, `fixtures/observation.txt`, smoke evidence
and old result artifacts are preserved unchanged.

## Commands

Run from the repository root with the locked Rust toolchain dependencies:

```sh
cargo run --release --locked -- golden check --initial
cargo run --release --locked -- golden check
cargo run --release --locked -- golden run /tmp/golden.jsonl /tmp/golden.csv --initial
cargo run --release --locked -- golden run /tmp/golden-six.jsonl /tmp/golden-six.csv
cargo run --release --locked -- golden request conflict human_model json > /tmp/request.json
cargo run --release --locked -- golden sample > /tmp/synthetic.jsonl
cargo run --release --locked -- golden replay /tmp/synthetic.jsonl /tmp/replay.jsonl /tmp/replay.csv
```

`check --initial` reports the first three cases; `check` reports six. Each validates
the independent reference and a perfect **synthetic scorer control**, not a model.
`run --initial` emits 48 rows, `run` 96: cases × four contexts × conventional,
semantic JSON, genuine CBOR and CSR/1. CLI tests run the complete 96-row matrix.
The default command without arguments retains the original #792 smoke behavior.
`--help` lists all commands. Output directories must already exist. Files named as
outputs are overwritten; use distinct local output paths. This CLI is for trusted
local files, not a public file service. Input replay files are bounded at 16 MiB,
UTF-8, one strict JSON object per nonempty line, with unique trial ids. Blank lines
are errors. Identical input/output path strings are rejected; the caller must also
avoid filesystem aliases/symlinks to inputs.

Contexts are `human_model`, `model_tool`, `model_model`, `model_data`. Arms are
`conventional`, `json`, `cbor`, `records`. Each uses #792's complete matched facts:
a labelled sheet for conventional human text, idiomatic structured JSON for other
conventional contexts. Structured conventional and semantic JSON payloads coincide;
semantic envelopes also carry cold schemas and transfer metadata. This is a
structural control, not a claim that these represent every mechanism's natural
language baseline. No MCP session, model-to-model execution, search/retrieval,
provider endpoint or hidden-state access is claimed. Model-native remains
unsupported/not testable; Protobuf remains deferred.

`request` prints **exactly** the measured final local request bytes with no trailing
newline. It combines the entire context envelope with common instructions,
questions (without expected answers) and [response schema](GOLDEN-RESPONSE-SCHEMA.txt).
All arms receive the same questions and output rubric. Source originals and the
answer ledger are not included. Binary payloads use counted padded base64. Final
request length includes schema text, common instructions/questions, nested
strings, escaping and base64 overhead. Provider/HTTP/tool-discovery/session
wrappers are not available and are not part of this local request measurement.
The exact request digest is pinned in each result and required on replay.

## Structured scoring rubric v1

The [strict schema](GOLDEN-RESPONSE-SCHEMA.txt) is included in every request. A
response has reconstruction assertions and separate comprehension/application
answer maps. Assertion ids identify source claims, including both sides of a
conflict; list order is immaterial. Every correctly reconstructed assertion must
match subject, predicate, typed object, all three independent clocks, full
provenance (asserting entity, method, evidence-reference set) and uncertainty.
The response reconstructs assertion metadata; it does not independently reproduce
the entity/relation/evidence catalogues, prove source entailment, or resolve truth.
Catalogue preservation is checked separately by the structural reference control.

For each missing expected assertion add one omission. Each extra id adds one
invention, even if its value copies a known fact. For an existing id, count one
wrong-relationship penalty if subject/predicate/object differs, one temporal
penalty if any clock differs/is lost, one provenance penalty if any attribution
or evidence reference differs/is lost, and one uncertainty penalty if confidence
or unknown status differs/is lost. Missing assertions are counted once rather
than in all four field categories. Correct assertions require every field.

Each expected question contributes one answer unit within its task. Missing keys,
wrong JSON values/types and invented keys are reported separately. JSON arrays
are order-sensitive; unknown answers are explicit JSON null. There is no fuzzy
matching, semantic judge, synonym acceptance or partial credit for prose. Each
question's exact expected value in the ledger is its rubric. This deliberately
bounded score may reject semantically equivalent wording and cannot establish
free-form model understanding.

`total_penalties` sums all assertion-category and answer errors. Fidelity is
`max(0, expected_units - total_penalties) / expected_units`, where units are
reference assertions plus comprehension and application questions. Categories
can overlap for a single damaged assertion; extra inventions reduce the score.
The denominator is pinned to the reference, never to how many answers were
returned. Zero expected units gives null. Correct counts and task breakdowns are
reported alongside this deliberately strict aggregate, so it is not mistaken for
an accuracy probability.

Malformed JSON, duplicate keys anywhere, duplicate/empty assertion ids, unknown
fields, invalid enum/object shapes, missing required top-level sections and
missing required nested time fields produce `malformed=1`, an error string,
`valid_structured_response=false` and **null fidelity**. They cannot be passing
scores or assertion-efficiency inputs. A wholly absent metadata group is accepted
as a category loss. Unknown/invented claims can be represented without requiring
the response to be a valid Contract. The scorer is separate from strict contract
round trips; empty reconstruction/answer maps are valid but score omissions.

## Import/replay contract

`golden sample` prints a clearly tagged perfect synthetic record with all telemetry
null. Checked-in [synthetic replay](../fixtures/golden/synthetic-replay.jsonl)
contains perfect, omitted, invented, contradictory, partial, malformed and wrong
answer controls, not actual model output. The replay envelope is strict:

- `version`: 1; `trial_id`: nonempty unique id.
- `case_id`, `context`, `arm`: a compiled case and supported labels.
- `request_sha256`: SHA-256 of `golden request` for that exact case/context/arm.
- `origin`: `synthetic` or `imported_real`.
- `response_json`: raw response **string**, preserving malformed JSON/duplicate keys.
- `metrics`: the complete nullable telemetry shape printed by `sample`.

Synthetic records must carry entirely null telemetry and always have null
assertion-per-token efficiency. For future imported real trials, provider, model,
revision and settings object are required. Token fields are provider-reported
`actual_input_tokens`, `actual_output_tokens`, `actual_cached_input_tokens`;
nonempty `usage_evidence` must identify the attested provider usage record if any
is supplied. No local token estimate can substitute. Cached tokens are a subset
of reported input tokens. `cost_usd` requires `cost_evidence` (actual billing or
dated pricing/calculation source); absent cost remains null. Evidence fields are
labels supplied by the importer and are **never fetched**. Retry count, trial
error, model latency, end-to-end latency, CPU time and peak RSS are separate
nullable fields. Resource/timing values supplied by a future importer must have
their boundaries and measurement method recorded in its settings/evidence; the
local runner never derives CPU time from wall time or fills them automatically.
Replay metadata uses its own finite-number, duplicate-key-checking JSON ingress,
so fractional settings (including nested values and `0.0`), latency and small
costs are accepted. Semantic contracts, golden fixtures and scored responses
retain their separate integer-only numeric ingress. Typed replay version, token,
retry and resource-count fields still reject decimal representations.
Negative/nonfinite timing/cost, inconsistent cached usage and missing metadata
are rejected. A failed trial with `trial_error` retains its record and telemetry
but receives no score or efficiency.

Imported telemetry is caller-attested, not authenticated or verified by this
program; output mode explicitly says `imported_real_unverified`. Importing a
record does not prove a call happened. No real trial is shipped in this PR and
no endpoint credentials or paid calls are authorized by this command.
`correct_assertions_per_1000_actual_input_tokens` is correct complete ledger
assertions ×1000 / actual input tokens. It is null for local or synthetic runs,
missing usage/scores, failed or malformed responses and zero input tokens. A
valid scored response with no correct assertions and positive usage yields zero.

## Reproduction and export boundaries

Rows pin harness/package/schema versions, source/contract/suite/Cargo.lock hashes,
common response-schema/instruction hashes and exact final request hash. Replay
also records the raw response hash. Model settings are unconfigured/null in all
local/synthetic records; imported settings are retained without defaults. No seed,
provider revision, pricing or cache assumption is fabricated. The original suite
and request generation are deterministic; wall-time values vary.

Golden timing uses 3 untimed warm-ups then 20 timed iterations **per named
operation**, one warmed process in fixed case/context/arm order, using `Instant`
and `black_box`. Rows report total nanoseconds; divide by `iterations` for a mean.
Codec encode includes validation/sorting/allocations/drop; decode includes
parsing/allocations/validation/drop and strict CBOR/CSR re-encoding/profile checks.
Conventional payloads use adapter translation; standalone codec times are null
there. Adapter render/parse includes codec work, cold schema, transfer, envelope
creation/reconstruction and destruction. Request render includes adapter render
plus common instructions/questions/output schema and final JSON serialization.
These spans **overlap**; do not add them or subtract means to infer isolated costs.
Fixture loading, source/reference validation, evidence hashing, round-trip checks,
request hashing, build, I/O, printing and model execution are outside timed loops.
Replay performs no benchmark timings: iteration counts are zero, times null.
Local times are neither model latency nor complete source-to-consumer latency.
CPU duration and peak RSS remain null; no inference, money or performance claims
follow from one fixed-order process without repetitions/confidence intervals.

JSONL has one typed Row per trial. CSV has the same top-level columns in sorted
order, with all cells quoted using doubled quotes and LF record endings. Nested
metrics/scores are compact JSON cells, strings are raw strings, numbers/booleans
retain their JSON lexical forms, and null is literal `null` rather than zero or
empty. Use the Row field types (or JSONL) to interpret cells. RFC4180-style CSV
readers support the quoting, including embedded newlines. The regression test
independently parses CSV and restores every JSONL field, including quotes,
commas, newline, Unicode and null. [Results](RESULTS.md) link the recorded outputs,
hashes, environment and final validation commands. An optional Python 3 standard-library
[audit script](../scripts/validate-golden-results.py), run after the release commands,
independently reads both exports, regenerates each exact request and verifies
input/output hashes and synthetic outcomes. It is not a runtime/model integration
dependency; the Rust tests also check the matrix and CSV independently.
Independent exact-head review
and head-specific CI remain separate gates; this harness does not complete M01,
Issue #1, or #794's live comparative pilot.


## #794 bounded repeated pilot

The existing `golden run` remains a single fixed-order 96-row smoke measurement.
The new `golden trial CASE CONTEXT ARM TRIAL_ID` uses the same requests/timing code
for one selected observation. A standard-library Python driver schedules 480
serial fresh children with a seeded shuffled order and records errors alongside
successful timings. It never invokes an endpoint. See [pilot report](PILOT.md)
for commands, compact export/matrix join schema, sample units, audit and findings.
The original smoke/golden result files and fixtures are preserved unchanged.

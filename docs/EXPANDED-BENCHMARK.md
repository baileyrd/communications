# #795 — 60-case offline benchmark expansion

This delivers the original card's bounded dataset, local transport/context
comparison and repeatability work. Full #795 acceptance is **incomplete**:
no actual model interface, two-model interoperability experiment, MCP session,
retrieval experiment or native hidden-state interface ran. Issue #1 remains open.
No semantic-contract authority, ontology, codec profile or scorer changed.

## Authored coverage and reference independence

[Expanded suite](../fixtures/golden/expanded-suite.json) contains the original six
references unchanged followed by 54 new scenario designs: 60 sources, 134 facts,
120 comprehension/application questions. Every expected JSON value is an explicit
rubric; the existing exact typed scoring rules apply. Requests omit expected
answers and original source. [Coverage catalogue](../fixtures/golden/coverage.json)
records the distinction and overlapping taxonomy tags for every new case.

These are authored synthetic scenarios, not an empirical population or certified
human gold labels. The source reports, reference facts/catalogues and expected
answers were authored outside the production renderer/codec. Contract extractions
are separate checked-in files. The author used a common working specification
for source/ledger/extraction transcription; their agreement cannot exclude a
shared authoring error. The source-notation audit independently compares every
new source statement/catalogue with its ledger and extraction. Independent
technical review must still assess source meaning, questions and expected answers;
passing synthetic scorer controls is never independent model-response evidence.

| Taxonomy | New cases tagged | Meaningful variation and examples |
| --- | ---: | --- |
| Directed relations | 17 | Inversion, self-edge, nontransitive chain, cycle, one-to-many, many-to-one, predicate identity; combined selection across six facts |
| Conflicts | 9 | Boolean, text, integer, three location reports, equal/high/zero/unknown confidence, distinct or identical reporters; no implicit overwrite |
| Provenance | 16 | Observed/reported/inferred distinctions, self-attribution, same values with separate reporters, inference without event, event/method filters |
| Evidence identity | 3 | Multiple distinct claims share one full immutable original; same content with separate assertion ids; digest never establishes truth |
| Uncertainty | 12 | Zero vs unknown vs 1000, threshold 499/500, high-confidence dispute, unknown observation, conjunction with time |
| Time | 15 | Equal/inverted/negative/extreme clocks, event ties, unknown-event sorting, separate interpretation times, three-event ordering |
| Unknown information | 16 | Absent relationship vs false, literal null vs unknown event, missing translation/causality, unreported schedule/state |
| Exact text | 13 | Empty/whitespace, numeric/boolean/JSON/instruction-like text, control characters, long literal |
| Unicode | 5 | NFC/NFD distinction, RTL marks, joined/modifier emoji, nonbreaking space, non-ASCII identifiers |
| Integer | 9 | Signed extrema, zero/negative, disagreements, text vs integer, predicate/time/confidence selection |
| Combined boundaries | 15 | Signed clocks plus empty text/zero confidence/entity link, distinct ids with identical metadata, extreme integer/time values |

Counts overlap and describe the 54 additions, not sampling weights. Every addition
has its own task distinction; this is not 54 scalar substitutions of one template.
All stay within the existing entity/boolean/text/integer objects, three provenance
methods, explicit unknown/confidence and separate signed-millisecond clocks.
Opaque local names `item`/`agent` do not add a universal ontology. Sources are
explicit fact reports, so representativeness is of these supported distinctions,
not ordinary human language. Blind spots include natural ambiguity, independently
collected human labels, realistic long documents, multiple evidence originals per
case, entailment, calibrated confidence, adversarial traffic near byte/count
limits, units/intervals/negation calculus and any live interface. The codec tests
continue to cover additional multiple-evidence/bounds cases outside this dataset.

## Reproduce

```sh
cargo build --release --locked
target/release/communications-probe expanded check
target/release/communications-probe expanded capabilities
target/release/communications-probe expanded request boundary-join model_tool cbor
python3 scripts/test-expanded-pilot.py
python3 scripts/expanded-pilot.py fixtures
python3 scripts/expanded-pilot.py run /tmp/communications-expanded-new
python3 scripts/expanded-pilot.py audit /tmp/communications-expanded-new
python3 scripts/expanded-pilot.py audit docs/results/expanded
```

The output directory must be new. `expanded trial CASE CONTEXT ARM TRIAL_ID`
reuses the existing selected-trial runner and timing operations.
`expanded replay INPUT_JSONL OUTPUT_JSONL OUTPUT_CSV` reuses the same import,
fractional telemetry and duplicate-key validation/scoring boundaries. Imported
records remain `imported_real_unverified`; no imported real record is delivered.
Original `golden` and no-argument smoke commands retain their behavior.

## Design fixed before measurement

60 cases × four offline contexts × four matched arms = 960 cells. Five repetitions
produce 4,800 attempts, one serial fresh release child per attempt. Python
`random.Random(795)` MT19937 shuffles all 960 cells once per repetition. The complete
pre-run plan and actual sequence/ids are retained. A sample is one warmed batch
of 20 timed operations in one child, with three untimed warm-ups **per operation**.
Inner iterations are not independent observations. No failures/timeouts are dropped
or retried into success; timeout is 60 seconds, and errors/exit statuses stay in
the attempted denominator. Fresh children share the host and OS caches.

Five batches per cell support a small descriptive variability check for these
exact tasks on this machine. They do not provide stable tail estimates, meaningful
population confidence, independent hardware replication or evidence of model
accuracy. P95 nearest rank is the maximum with n=5. Per-cell min/median/P95/max
preserve heterogeneity; separate pooled-by-arm distributions mix cases/contexts
and are explicitly descriptive. There is no statistical significance or inference
efficiency claim. Stronger live-model acceptance still needs genuine repeated
observations and a prespecified model/task analysis.

Named spans remain codec encode/decode, full adapter render/parse and complete
request render. Spans overlap and must not be summed; wall time cannot establish
CPU, model or end-to-end latency. Allocations/drop and structural validation are
inside the measured operations. Fixture loading/reference checks/evidence hashing,
startup/build/I/O and request hashing are outside. Exact payload, schema/base64,
escaped envelope and full cold local request bytes are counted; provider/protocol
bytes and schema amortization remain unmeasured. All arms use the same facts,
questions, instructions, output schema and explicit rubric for a given case.

## Five-layer capability and acceptance accounting

| Layer | Delivered local boundary | Actual model interface / acceptance |
| --- | --- | --- |
| human-model | Text message fact sheet and JSON/CBOR/CSR semantic envelopes | No receiver model; comprehension/application untested |
| model-tool | Local tools/call-shaped JSON-RPC envelope | No MCP connection, discovery, tool execution or model-tool integration |
| model-model | Local serialized message boundary | No sender/receiver pair; cross-model interoperability untested |
| model-data | Local document envelope | No retrieval, search, locator fetching or model-data experiment |
| model-native | Explicit unsupported capability record | No documented authorized tensor/hidden-state API; not testable |

All four transport arms pass structural preservation; CBOR is genuine Ciborium
CBOR, CSR/1 remains the existing local profile. This verifies local compatibility
with this Rust implementation, not cross-language/model interoperability.
Safe runtime metadata inspection found no configured secrets, endpoint variables
or outbound identities; no available authorized benchmark endpoint was established.
No secret values were read, no credentials/settings changed, no model downloaded,
no model calls made and no spending or external dataset sharing initiated.

A real multi-model comparison needs a chosen authorized provider endpoint,
two exact compatible model revisions, documented accepted request/response
interfaces, a bounded call/budget grant and permission to send these synthetic
fixtures, then genuine run/usage evidence under matched settings. Provider/action
are currently **unselected**, not an invented OpenAI/other-provider configuration.
Record actual failures, responses, token usage/cost and measured latency rather
than filling these from offline timings. Native access needs its own documented
interface; embedding vectors are not hidden states. The vector-specific board
comment is preserved but excluded pending verified owner scope. No vector engine,
embeddings or replacement architecture is implemented by this PR.

## Evidence and audits

[Run manifest](results/expanded/manifest.json) pins all Rust sources, fixtures,
Python scripts, schemas and Cargo files; executable/environment versions; design,
commands and output hashes. The run was built on base
`bcac61c82540c2a7f18d584ebad85ffde869f4e4` with the exact modified source hashes
listed in the manifest. Final tested Git head/tree and remote CI are recorded in
the PR/board; the manifest's base is not falsely relabelled as a later commit.
[Case manifest](results/expanded/case-manifest.json) independently pins each source,
canonical authored reference object (including all questions/expected values)
and extraction. [Plan](results/expanded/plan.json) fixes every order/id before run.

[Raw JSONL](results/expanded/trials.jsonl) and [equivalent CSV](results/expanded/trials.csv)
retain every attempt's timing/exit/error fields. Join each successful row to
[matrix](results/expanded/matrix.json) by `case_id/context/arm` to restore the
remaining exact request sizes/hashes, metadata, verified structural flags and
null unmeasured model/resource fields. Trial fields override any common fields.
Failed attempts remain raw records without pretending a successful matrix result.
[Per-cell summary](results/expanded/summary-by-cell.json) and
[explicitly pooled summary](results/expanded/summary-pooled-by-arm.json) expose
success/attempt denominators and distributions. [Capability records](results/expanded/capabilities.json)
include the fifth unsupported layer.

The expanded audit independently checks all new source statements, reference and
extraction catalogues/fields/hashes; original-six equivalence; id/matrix/order
completeness; CSV restoration; exact regenerated request bytes/hashes; schema,
base64/escaping arithmetic; matched questions without answers; null metrics;
capabilities; output/input manifests; and statistics using separate arithmetic
from the producer. Hash equality proves identity, not independent authorship or
source entailment. Rust tests check every 60×4×4 structural round trip plus selected
CLI/replay/import/capability behavior, without treating scorer controls as models.

Historical smoke/golden/pilot files and original six fixtures remain byte-for-byte
unchanged. The unchanged #794 pilot audit pins old *source files* too, so running
it directly on changed sources intentionally rejects the new source hashes.
`python3 scripts/audit-pinned-pilot.py` executes that same audit with all original
inputs verified against immutable merged commit `bcac61c82540c2a7f18d584ebad85ffde869f4e4`,
while using the current release binary to regenerate every exact old request.
CI fetches that history, retains all original tests/golden audit, and adds the
expanded tests/audit. No historical timing is claimed to measure current code.

See [results](RESULTS.md) for measured findings and
[validation log](results/expanded-validation.log) for actually executed checks.
Independent review, live exact-head CI and unresolved acceptance assessment remain
coordinator gates. A useful partial draft PR does not make #795 Done.

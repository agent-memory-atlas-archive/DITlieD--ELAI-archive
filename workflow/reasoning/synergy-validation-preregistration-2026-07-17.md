# Autoresearch-synergy validation preregistration

VERSION: 1.1  
STATUS: LOCKED AFTER INDEPENDENT PREREGISTRATION AUDIT; implementation may start in the frozen order, but B-SYN-0 live calls remain spend-gated  
DATE: 2026-07-17  
REPOSITORY ROOT: `/home/user/teikoku/elai`  
BASE HEAD: `a89f55bea1b6aa669c5c79aadca6291b5390df88`  
PROOF_SPINE: none (measurement-validation class; explicitly authorized by the user on 2026-07-17)

This document freezes the hypotheses, inputs, arms, seeds, sample sizes, power floors, cost caps,
decision rules, invalidation rules, and stopping rules for B-SYN-0, B-SYN-1, and B-SYN-2. It
implements the design in
`workflow/handoff/autoresearch-synergy-validation-plan-2026-07-17.md`. The three families are
independent. A result in one family cannot promote or suppress another.

No result was inspected while choosing the B-SYN-2 injected-scenario outcomes or the B-SYN-0
corpus labels. Existing P-25 and b0v3 full-run results were necessarily inspected because B-SYN-1
is a calibration replay over those named archives; their original verdicts and sealed bytes remain
the load-bearing truth.

## 1. Global protocol

### 1.1 Order and authority

1. Independently audit this entire preregistration and run the frozen plan gate.
2. Lock this file by BLAKE3 digest. Any substantive repair creates version 1.1 or later and a new
   digest; the old version remains archived.
3. Implement, run, independently check, and close B-SYN-1.
4. Implement, run, independently check, and close B-SYN-2.
5. Build and independently audit B-SYN-0 without making a model call.
6. Present the exact B-SYN-0 call and token ceilings to the user. Live execution requires a new,
   explicit spend authorization.
7. After authorization, make one upstream-aware health probe outside the benchmark, run the
   preregistered 12-call smoke exactly once, independently check it, then run the 200-call scored
   experiment exactly once.

The main agent owns decisions. Implementers do not render authoritative test or audit verdicts.
Independent checkers rederive every family verdict without importing the producer.

### 1.2 Common deterministic choices

- Canonical JSON: UTF-8, keys sorted, separators `,` and `:`, final LF only where a file format
  requires one.
- Content digests: BLAKE3 for new benchmark artifacts. Existing SHA-256 archive digests are retained
  as historical source identities and are never rewritten.
- Paired bootstrap: percentile interval, 5,000 samples, seed `20260712`; lower index 2.5%, upper
  index 97.5%. The report distinguishes paired observations from bootstrap resamples.
- Prefix rule: preserve the sealed archive order and use the first K rows. No resampling,
  replacement, sorting by outcome, or cherry-picking is permitted.
- Missing, malformed, non-finite, duplicate, or schema-incompatible evidence is `INVALID`, never a
  negative mechanism result.
- A checker mismatch is `INVALID` and withholds scientific interpretation until repaired under a
  version bump.
- No benchmark may modify the two historical source archives.
- No stale `F:\...` path is executable input. Historical Windows strings may be reported as stale
  metadata, but all live reads and writes resolve under `/home/user/teikoku/elai`.
- Unrelated dirty `.elai_cc` files are outside scope and must remain byte-untouched.
- No commit or push is authorized.

### 1.3 Verdict vocabulary

- `SUPPORTED`: every preregistered support condition and guard passes.
- `REFUTED`: a valid, adequately resolving comparison excludes the minimum useful effect, or a
  deterministic hard guard such as a locked cost cap fails.
- `NULL-ADEQUATE`: valid and adequately resolving, but neither support nor refutation rule fires.
- `NULL-UNDERPOWERED`: valid, but the interval does not exclude the minimum useful effect and its
  half-width exceeds that effect.
- `INVALID`: evidence, instrument, corpus, freeze, or checker integrity failed.
- `OPERATIONAL-FAILURE`: the environment could not complete the protocol. This is not a scientific
  verdict.

`NULL-UNDERPOWERED` never overrides a deterministic cost-cap refutation. Reports therefore carry
both comparison-level power verdicts and the family-level verdict. This prevents a wide quality CI
from erasing a real spend breach while still exposing an underpowered quality claim.

## 2. B-SYN-1 — refutation power floor and quantified-kill compatibility

### 2.1 Hypotheses

H1: adding a power block leaves every untouched P-25 and b0v3 family verdict unchanged.  
H2: deterministic K=15 and K=7 prefixes reduce, and never increase, the count of adequately
powered statistical refutations; at least one comparison becomes `NULL-UNDERPOWERED`.  
H3: an optional canonical quantified `KILL-EDGE` tail survives the current validate-and-fold path
exactly, while legacy v1.0 and v1.1 artifacts without the tail remain valid.

This family tests replay and wire-format compatibility. It does not make the quantified tail
load-bearing in production before the family verdict; production structured parsing remains
permission-ladder gated.

### 2.2 Frozen source inputs

P-25:

- corpus: `workflow/benchmarks/p25-exo-lean-2026-07-17/corpus.json`
- scored run: `workflow/benchmarks/p25-exo-lean-2026-07-17/scored/run.json`
- expected schema: `p25-lean-run-v1`, K=30, seed `20260712`
- scored-run SHA-256:
  `0cbc6704d6469a4cc08cf37869aa6c679f6283104313d74f30f80972c1e1e55a`
- native full-K family verdicts: Family A `REFUTED`, Family B `REFUTED`
- deterministic guards: Family A token-ratio cap 1.3; Family B token-ratio cap 2.5

b0v3:

- archive: `workflow/benchmarks/research-creativity-b0v3-2026-07-14/`
- expected K=30, bootstrap seed `20260712`, bootstrap samples 5,000
- manifest id:
  `2e53c72e21cb32c888b6c3679fc020d6aa92329c7061b294ca3a550832ace908`
- `results.json` SHA-256:
  `b0d1bc8bbb0bb2f7c22ebc91f5dde86f768e881bb68c8bb0c63e785ffbc44796`
- `bootstrap.json` SHA-256:
  `4b08229cbd4b1611143fd53b8e92590c50c3fcc04d069049f0cf70cbddb68232`
- native full-K family verdict: `REFUTED`
- deterministic guard: G3 cost non-inferiority failed

The replay levels are exactly K=`30`, `15`, and `7`; quarter-K means `floor(30/4)`.

### 2.3 Endpoint effect floors

| archive | endpoint | preregistered minimum useful effect |
|---|---|---:|
| P-25 | P1 distinct full-pass payloads per task | 0.5 |
| P-25 | P2 best evaluator value | 1/6 |
| b0v3 | P1 validated-recovery delta | 0.05 |
| b0v3 | P2 best-execution delta | 0.005 |

The b0v3 P2 floor is the existing archived sensitivity MDE. The b0v3 P1 floor equals the existing
absolute non-inferiority margin. The P-25 P1 floor is half an additional full-pass payload per task;
the P2 floor is one evaluator check.

### 2.4 Power-block computation

For each treatment-control endpoint and K:

1. Reconstruct paired task deltas from source task rows.
2. Compute the frozen paired-bootstrap mean and CI95.
3. Record K, paired count, bootstrap count, seed, mean, bounds, width, half-width, effect floor,
   vector digest, and native guard state.
4. Preserve the archive's original endpoint result as `native_endpoint_state`. Separately classify
   minimum-useful-effect resolution in this exact precedence:
   - `SUPPORTS-MINIMUM-EFFECT` when CI95 lower is at least the minimum useful effect.
   - `REFUTES-MINIMUM-EFFECT` when CI95 upper is strictly below the minimum useful effect.
   - `NULL-UNDERPOWERED` when half-width is greater than the minimum useful effect.
   - otherwise `NULL-ADEQUATE`.

The upper-bound rule answers the scientific question directly: can the observed interval exclude
an effect large enough to matter? Width remains a mandatory diagnostic and cannot be omitted. The
precedence makes the four states mutually exclusive; a small positive interval wholly below the
useful-effect floor refutes usefulness rather than simultaneously supporting a merely positive
effect.

Family replay records two fields:

- `native_family_verdict`: exact recomputation under the archived decision grammar;
- `power_adjusted_family_verdict`: a deterministic cap failure remains `REFUTED`; otherwise, a
  native statistical refutation remains `REFUTED` only when every row that triggered that native
  refutation is `REFUTES-MINIMUM-EFFECT`, and becomes `NULL-UNDERPOWERED` otherwise.

The frozen comparison sets are P-25 Family A's three controls times P1/P2, P-25 Family B's two
controls times P1/P2, and b0v3's four controls times P1/P2 plus G1/G2a/G2b/G3. A
`native_refutation_trigger` is exactly the row the archived producer uses: P-25 CI95 upper below
zero; b0v3 primary point estimate at or below zero; or an archived guard below its boundary.

### 2.5 Monotonicity

Raw prefix classifications are always reported and may be non-monotone: a smaller nested prefix can
legitimately shift its mean or have lower sample variance. The gate uses a conservative monotone
power envelope:

- K=30 effective state equals its raw state;
- a K=15 row effectively `REFUTES-MINIMUM-EFFECT` only when raw K=30 and K=15 both refute;
- a K=7 row effectively `REFUTES-MINIMUM-EFFECT` only when raw K=30, K=15, and K=7 all refute;
- otherwise a raw smaller-K refutation is reported but its effective state is
  `NULL-UNDERPOWERED`.

The effective refutation count is therefore non-increasing by construction. Prefix digests and raw
intervals remain visible so this conservative envelope cannot hide a non-monotone observation.

At least one comparison across the two archives must be `NULL-UNDERPOWERED` at K=15 or K=7.

### 2.6 E-2 canonical tail compatibility

The canonical positive fixture line is:

`KILL-EDGE: C7 <- experiment:fixture-b3 delta=-0.031 band=0.045 verdict=within-noise`

The synthetic artifact contains a complete valid candidate card and required sections. The
producer must:

1. run the current `conjecture_triage.py --validate` path;
2. run the current `conjecture_triage.py --fold` path;
3. recover exactly `killer_type=experiment`, `killer_ref=fixture-b3`, decimal delta `-0.031`,
   non-negative band `0.045`, and verdict `within-noise` from the folded kill-edge bytes;
4. prove the folded candidate remains `refuted`;
5. validate and fold one legacy v1.0 and one legacy v1.1 fixture without a quantified tail.

Exit code alone is insufficient: the checker must compare the recovered structured values and exact
canonical tail. The benchmark may use an isolated parser adapter for this recovery but may not edit
the production triage/fold parser before the B-SYN-1 verdict.

### 2.7 B-SYN-1 gate

`SUPPORTED` requires all of:

- source digests and schemas match;
- full-K replay reproduces all three native family verdicts with zero flips;
- all K=30 means and CI bounds match the archived values within `1e-12`;
- monotonicity rules pass;
- at least one K=15/K=7 row is `NULL-UNDERPOWERED`;
- quantified and both legacy grammar fixtures pass exact validate/fold round-trip;
- the independent checker rederives identical canonical output and exits 0.

Any source mutation, digest mismatch, native full-K flip, or producer/checker disagreement is
`INVALID`. A deterministic assertion failure with intact inputs is `REFUTED`.

### 2.8 Cost and stopping rule

Live model-call cap: zero. Network-call cap: zero. The run executes once after test/audit green.
Repairs require a version bump and replay; source evidence is never regenerated.

## 3. B-SYN-2 — Pareto-per-task specialist retention

### 3.1 Hypotheses

H1: at equal evaluation budget, Pareto-per-task retention preserves injected later-winning
specialists more often than greedy/argmax and deterministic round-robin by at least 0.25, with both
paired CI95 lower bounds above zero.  
H2: Pareto is non-inferior on final best-at-fixed-budget, with lower CI95 bound at least `-0.005`
against both controls.  
H3: on null scenarios with no task-rank reversal, Pareto produces no positive policy win.  
H4: the 1.5x arm is always flagged `spend_leak` and is never eligible to support H1 or H2.

### 3.2 Frozen scenario population

Candidate identities are the 45 signatures returned by
`.claude/tools/frontier_strategy_bench.py::strategy_space`, sorted by signature for identity and
ranked by their early score for policy decisions.

The known-answer conformance fixture uses seven candidates and six retention slots: six
higher-aggregate generalists plus one lower-aggregate candidate that uniquely wins a task and
becomes the later best. It proves selector wiring only and is excluded from every effect estimate.

Scored specialist scenario transforms:

- `single-pocket`
- `dual-pocket`
- `late-heavy`
- `risk-flip`

Each scenario has 32 paired seeds:

`seed_i = 20260712 + i * 9973`, for `i=0..31`.

This yields 128 paired specialist observations. Each scored row begins with the unmodified real
per-instance score matrix for all 45 DSL signatures. Before any policy runs, BLAKE3 over
`scenario_id|seed` selects one candidate from early aggregate ranks 7 through 20 and a pocket task
set of size 1, 2, 4, or 6 respectively. The transform applies this fixed, policy-blind perturbation
to that candidate only:

- pocket task scores add `+0.15`;
- non-pocket task scores add `-0.02`;
- values are clipped to `[0,1]`;
- later pocket weights are fixed at 4, 8, 16, and 32 for the four named transforms;
- candidate identities stay unchanged and no policy sees later weights before retention.

The transform never consults F1/F2/F3 selections and does not guarantee that the candidate becomes
a per-task winner, is retained by F3, or becomes the later best. Every row remains in the
denominator. For P2, a row is one only when the designated candidate both would be the unique later
best if evaluated and survives the policy; non-responsive injections are zero for every arm. This
is the scored efficacy population. The forced seven-candidate fixture cannot support H1 or H2.

Null scenarios:

- `null-monotone`
- `null-tie`

Each null scenario uses the same 32 seeds, candidates, slot count, and evaluation budgets. There is
no rank reversal: per-task order is consistent with aggregate order, and all equal-budget arms must
reach the same final best. Null rows are analyzed separately and never pooled with specialist rows.

Scenario bytes, task labels, later weights, specialist ids, and seed list are sealed by BLAKE3 before
the first benchmark execution.

### 3.3 Arms and budgets

- F1 `greedy_argmax`: retain the six highest early aggregate scores.
- F2 `deterministic_round_robin`: cycle through the five rank families in fixed lexical order,
  taking each family's highest unselected early aggregate candidate until six slots are filled.
- F3 `pareto_per_task`: collect per-task winners, apply deterministic set-cover dominance pruning,
  then fill remaining slots by tasks-won weight with seed-bound tie-breaking.
- F4 `pareto_spend_leak`: identical to F3 but with nine retained/evaluated slots.

The common initial 45-candidate score matrix is computed once per scenario row and costs exactly 45
logical evaluator invocations shared by all arms. The
policy-controlled later budget is six candidates across three later stages = 18 evaluations for
F1/F2/F3, and nine across three stages = 27 for F4. Thus F4 is exactly 1.5x on the controlled budget.
Calls are logical deterministic evaluator invocations; model and network calls are both zero.

F1/F2/F3 must have byte-identical scenario inputs, seed sets, score matrices, and later-stage count.
Policies may differ only in the retained candidate ids.

### 3.4 Metrics and power

- P1: per-seed final best weighted score among retained candidates.
- P2: per-seed specialist survival indicator.
- Guards: exact replay digest equality; invariants; equal F1/F2/F3 budgets; F4 `spend_leak=true`;
  null-policy win count zero.

P2 power line: an unrestricted paired binary difference lies in `{-1,0,1}` and has conservative SD
1. With N=128, the normal-approximation 80%-power two-sided MDE at alpha 0.05 is
`(1.96 + 0.84) * 1 / sqrt(128) = 0.2475`. The locked material floor 0.25 is just above this
conservative MDE. The preregistered gate still uses the empirical paired bootstrap, not this
approximation.

P1 non-inferiority margin is `-0.005`, matching the existing frontier validation margin.
All F3-minus-control intervals use the common bootstrap settings in section 1.2.

### 3.5 B-SYN-2 gate

`SUPPORTED` requires all of:

- the forced seven-candidate conformance fixture passes for F1/F2/F3;
- F3-minus-F1 P2 mean at least 0.25 and CI95 lower strictly above zero;
- F3-minus-F2 P2 mean at least 0.25 and CI95 lower strictly above zero;
- F3-minus-F1 and F3-minus-F2 P1 CI95 lower bounds both at least `-0.005`;
- every null scenario has P1 delta exactly zero within `1e-12`, CI includes zero, and no fabricated
  specialist-survival win;
- F1/F2/F3 each consume exactly 18 controlled evaluations per row;
- every F4 row consumes exactly 27 and is flagged `spend_leak=true`;
- repeated execution produces byte-identical canonical results;
- the independent checker rederives the verdict and exits 0.

An F3 win with any equal-budget breach is `INVALID`, never support. A valid failure of any support
condition is `REFUTED`.

### 3.6 Cost and stopping rule

Live model-call cap: zero. Network-call cap: zero. There are 192 scored rows: 128 specialist plus
64 null. Deterministic evaluator caps are:

- common matrices: `192 * 45 = 8,640`;
- controlled later evaluations:
  `192 * (18*3 equal-budget arms + 27 spend-leak arm) = 15,552`;
- total: 24,192 logical evaluator invocations.

The scored run executes once after the smoke and freeze checks. No seed replacement is allowed.

## 4. B-SYN-0 — novelty-instrument discrimination

### 4.1 Hypotheses

H1: the instrumented judge distinguishes known SAME from known DISTINCT with accuracy at least 0.90.  
H2: its non-UNCERTAIN rate on known pairs exceeds the deterministic fallback by at least 0.50, with
paired CI95 lower bound above zero.  
H3: the no-fixed-point scrambled verdict file fails closed with zero emitted verdicts.  
H4: normalized A1/A4 verdict agreement is at least 0.80.  
H5: the instrument stays inside the locked call and prompt/output envelopes.

### 4.2 Corpus

K=50 pairs, exactly 10 in each sealed hidden-label class:

1. known-SAME;
2. known-DISTINCT;
3. paraphrase;
4. compositional extension, label DISTINCT;
5. unrelated, label DISTINCT.

Sources are existing `workflow/reasoning/conjecture-run-*.md` artifacts and the frontier archive
fold. Every pair records stable source ids and exact source-text digests. Labels live only in the
sealed evaluator corpus, never in prompt-visible data. Pair ids are lexical after build.

Corpus seed: `20260717`. No-fixed-point permutation seed: `20260718`. Arm-order seed:
`20260712`. Bootstrap seed and count follow section 1.2.

The known-answer primary subset is N=20. At N=20, a one-sided exact binomial test of true accuracy
0.90 against chance 0.50 rejects at alpha 0.05 at 15 or more correct answers and has power
approximately 0.989. The stricter point gate requires at least 18/20 correct.

### 4.3 Arms and same-cost construction

Every pair consumes exactly one bounded completion call per arm:

- A1 `INSTRUMENTED`: judge response is parsed and used.
- A2 `BASELINE`: an A1-byte-identical shadow call is journaled but ignored; the current
  bin-coordinate fallback supplies the analyzed verdict.
- A3 `SCRAMBLED_INSTRUMENT`: an A1-byte-identical shadow call is journaled but ignored; the sealed
  A1 verdict rows are relabeled by the frozen no-fixed-point permutation and passed through the
  canary gate.
- A4 `ORDER_SWAP`: same candidate bytes and contract as A1, with candidate presentation order
  reversed; SAME/DISTINCT is normalized back to canonical pair order before comparison.

Temperature is zero. Model choice is frozen at build time to the live free completion lane and
reported in the spend-signoff brief. Maximum output is 256 tokens. The prompt begins with a
no-tools instruction and contains no repository paths. Output is exactly one compact JSON object
from the closed vocabulary `SAME|DISTINCT|UNCERTAIN`.

Within each pair, all four prompt byte lengths must be identical and at most 5,000 bytes. A1/A2/A3
prompt bytes must be identical; A4 differs only in candidate order. Deterministic space padding is
allowed only outside candidate fields and is sealed before calls.

### 4.4 Smoke, calls, checkpoints, and spend ceiling

Smoke pair count is exactly three, selected before labels are hidden as the lexical first
known-SAME, known-DISTINCT, and compositional-extension pair. Smoke calls: `3 pairs * 4 arms = 12`.
Smoke evidence is compatibility-only and is not reused in the scored result.

Scored calls: `50 pairs * 4 arms = 200`.

Total benchmark ceiling: 212 calls. One upstream-aware health probe is required and recorded outside
the benchmark, making the maximum external completion count 213. The health probe uses the same
5,000-byte prompt and 256-output-token ceiling. No retry may exceed these counts.
A transport failure stops the run; completed call records are resumed by pair/arm key and never
resampled.

Each call is journaled atomically before the next begins with pair id, arm, request digest, response
bytes, response digest, parser status, reported usage, latency, and attempt id. A call with a durable
valid response is immutable. Repair after smoke requires a preregistration version bump and offline
smoke replay; the smoke is never re-called.

Hard envelopes:

- one call per pair/arm;
- prompt at most 5,000 bytes and equal-length within pair;
- maximum output 256 tokens per call;
- at most 212 benchmark calls;
- monetary maximum on the declared free lane: USD 0;
- observed usage is reported but cannot override byte equality because the bridge input meter is a
  known estimate.

The spend-signoff brief must report the chosen model, 12 smoke calls, 200 scored calls, 212 benchmark
calls, one external health call, 54,528 maximum external generated tokens, 1,065,000 maximum
external prompt bytes, and USD 0 expected lane charge. The benchmark-only subtotals remain 54,272
tokens and 1,060,000 bytes. No call occurs before explicit user authorization of that brief.

### 4.5 Metrics and gates

P1 accuracy: fraction correct on the 20 known pairs; invalid/UNCERTAIN counts incorrect. Gate:
at least 0.90 and one-sided exact-binomial p-value at most 0.05 against 0.50.

P2 non-UNCERTAIN lift: per-known-pair A1 indicator minus A2 indicator. Gate: mean at least 0.50 and
paired-bootstrap CI95 lower strictly above zero.

Guards:

- A3 instrument strength exactly `FAILED`;
- A3 emitted-verdict count exactly zero;
- A1/A4 normalized agreement at least 0.80 across all 50 pairs;
- A1 and A4 valid-output rate each at least 0.95; invalid known rows already count against P1/P2;
- all corpus, prompt, request, response, checkpoint, and result digests verify;
- every cost envelope in section 4.4 passes;
- independent checker rederives the verdict without calls and exits 0.

`SUPPORTED` requires P1, P2, and every guard. A valid miss is `REFUTED`. Canary leakage, corpus-label
leakage, prompt path leakage, call-cap breach, checker mismatch, or a single emitted A3 verdict is
`INVALID`.

### 4.6 External-validity boundary

Support would establish discrimination only on this sealed ELAI-archive corpus and authorize the
E-1 harness leg for its next gated implementation step. It would not prove future
out-of-distribution novelty judgment or reopen Q-10/P-25 creativity mechanisms.

## 5. Implementation and audit moves

MOVE 1: independently audit and lock this preregistration before code.
EXPECTED_OBSERVATION: frozen plan gate exit 0; auditor marks all ten plan points pass and confirms
the statistical and permission-ladder rules are executable without discretionary choices.
ON_FAIL: revise only the preregistration, increment its candidate version, repeat the audit, and
make no benchmark-code edit.

MOVE 2: build B-SYN-1 as an archive-specific replay producer plus independent checker, leaving the
historical producers and production KILL-EDGE parsers unchanged.
EXPECTED_OBSERVATION: a sealed offline artifact contains K30/K15/K7 power blocks, exact grammar
round-trips, zero full-K verdict flips, and a checker exit-0 receipt.
ON_FAIL: if source bytes drift, classify INVALID and stop; if logic fails with intact sources,
repair under a version bump and rerun the offline replay.

MOVE 3: build B-SYN-2 as a bench-lane-only scenario runner plus independent checker.
EXPECTED_OBSERVATION: the artifact reports all four arms, exact budgets, paired intervals, null
rows, spend-leak rows, and deterministic replay digests.
ON_FAIL: if F3 differs by anything except retained ids, classify INVALID; if valid gates miss,
record REFUTED and do not author the S-1 bench policy.

MOVE 4: build and audit B-SYN-0 without calling the model.
EXPECTED_OBSERVATION: corpus, prompt/request manifests, checkpoint recovery, canary permutation,
offline fake-client smoke, and independent checker all pass with live-call count zero.
ON_FAIL: repair only instrument code/corpus under a version bump; do not loosen parsing or call the
lane.

MOVE 5: after explicit spend authorization, run the external health probe, one smoke, and one full
run.
EXPECTED_OBSERVATION: 213 or fewer total external completion calls, durable call-by-call evidence,
and an independently replayable final verdict.
ON_FAIL: an upstream/transport failure records `OPERATIONAL-FAILURE` and resumes only immutable
completed keys; a compatibility defect records `INVALID` and withholds the full run.

FORK: if the current KILL-EDGE fold does not preserve the exact quantified tail bytes, B-SYN-1 H3
is REFUTED; do not edit the production parser inside the benchmark to manufacture support.

FORK: if the 45-strategy identities contain behavioral aliases, retain identity by signature and
record vector aliases; do not deduplicate away the preregistered candidate set.

FORK: if the free completion lane or model differs at B-SYN-0 build time, update only the
spend-signoff brief and verify the lane is still USD 0. A paid or unknown-price lane requires a new
preregistration version and exact currency ceiling before authorization.

RECON_NEEDED: the B-SYN-0 builder must verify the live completion model id, endpoint, health route,
and usage fields without making a model call; the one inference probe remains spend-gated.

RECON_NEEDED: the preregistration auditor must verify the P-25 task-level P1/P2 reconstruction
against `.claude/tools/p25_exo_lean.py` and b0v3 reconstruction against
`.claude/tools/research_creativity_analysis.py`.

ABORT: any proposal to inspect B-SYN-2 outcomes before sealing scenarios, alter labels after a
B-SYN-0 response, replace a failed seed/call, exceed a cap, expose repository paths to the model,
let F4 support a policy claim, edit historical archive bytes, self-stamp an independent audit, or
make E-1/E-2/S-1/S-2 production-load-bearing before its family verdict.

VERIFY: an independent auditor runs the plan gate, focused unit/integration tests, Python wiring
checks, maintainability checks, real CLI smokes, source-digest checks, producer/checker comparison,
and final archive replay. Missing evidence is BLOCK.

REDTEAM: the successful attack was that an exit-0 E-2 check could falsely pass while the current
greedy regex swallowed the entire quantitative tail into `killer_ref`; the patch requires exact
structured-value recovery from folded bytes. The successful power-floor attack was that a wide
quality CI could erase a deterministic spend refutation; the patch separates comparison power from
family hard guards. The failed spend attack is F4: its larger budget is always ineligible and must
raise `spend_leak`.

## 6. Wiring contract

WIRING_CONTRACT:

- B-SYN-1 producer consumes only sealed P-25/b0v3 artifacts and emits one canonical replay file;
  its independent checker consumes source artifacts plus producer output and rederives all rows.
- B-SYN-2 producer consumes the sealed scenario manifest and the existing 45-strategy identity
  space; its independent checker separately implements scenario and verdict recomputation.
- B-SYN-0 producer consumes a sealed label-hidden corpus and a completion client; fake and live
  clients use the same entrypoint. Its independent checker consumes only frozen call archives.
- No checker imports its producer module.
- Every CLI exits 0 only for a checker-clean valid result; scan/config failure exits 2.

PRODUCTION_HOOK: each benchmark's real CLI is the producer entrypoint, its on-disk canonical result
is the off-process artifact, and its separately implemented checker CLI is the consumer and
authoritative replay gate.

INTEGRATION_SMOKE B-SYN-1: an independent auditor drives the real replay CLI on both full archives
and all three prefixes, then drives the independent checker.

INTEGRATION_SMOKE B-SYN-2: an independent auditor drives the real scenario CLI on the known-answer
canary, policy-blind scored population, null population, and F4 leak arm, then drives the
independent checker.

INTEGRATION_SMOKE B-SYN-0: before live use, an independent auditor drives the real CLI with a
deterministic fake completion client through malformed output, fail-closed canary, checkpoint
interruption/resume, and all four arms.

End of preregistration candidate.

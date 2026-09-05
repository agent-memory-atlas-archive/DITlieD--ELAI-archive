# Preregistration: research-conditioned creativity ladder v1.0 (2026-07-12)

STATUS: LOCKED-BEFORE-CODE. No evidence run has started. Any change to an arm, primary endpoint,
comparison, margin, task set, model, candidate cap, or decision rule after the first model call requires a
new preregistration version and a new artifact directory. Smoke/tuning runs must be labeled non-evidence.

Owning design:
`workflow/reasoning/research-coupled-creativity-design-2026-07-12.md`.

## 1. Hypothesis

A bounded relational `ResearchInspirationPacket` built from verified source evidence will increase
execution-valid coverage of improving mechanisms and best execution value versus direct generation, a
source-blind matched target scaffold, an equally grounded verified-evidence projection, and a schema-valid
relation-shuffled packet, without materially reducing grounding validity or cost-normalized frontier yield
versus the best non-relational control.

The hypothesis is not that research, citations, prompting, or more tokens make text look more novel. The
binding effect is target-side execution-valid mechanism recovery.

## 2. Frozen exclusions

The evidence implementation must not contain:

- archive-conditioned cycles or candidate-parent feedback;
- mutate/recombine/reframe/analogize operator rotation;
- ProbeUtility, `--discriminate`, `probe`, or `/conjecture --cycles`;
- generator access to raw prior-conjecture or rejection prose;
- a harness archive -> L-24 or verified-case bridge;
- an LLM judge in a primary endpoint or pass/fail rule;
- a source-support edge applied to a target predicate or relation-sharedness claim;
- human best-of-N source/packet selection or unmatched packetizer/generator budgets across causal controls;
- a post-run change to the validation margin, confidence rule, or non-inferiority margin;
- target answer/hidden instance data in source packets.

Violation disposition: INVALID RUN, not NULL or REFUTED.

## 3. Packet contract frozen for all stages

Required fields:

`packet_version`, `packet_id`, `target_problem_id`, `source_domain`, `source_problem`,
`source_mechanism`, `object_mappings`, `shared_relations`, `invariants`, `disanalogies`,
`transfer_hypothesis`, `predicted_observation`, `target_falsifier`, `source_evidence_refs`,
`source_support_edges`, `contradiction_refs`, `source_quality`, `freshness`, `evidence_scope`, `packetizer_id`,
`packetizer_version`.

Rules:

- `evidence_scope` is exactly `SOURCE_ONLY`.
- canonical JSON field order is irrelevant to `packet_id`; blake3 hashes canonical values.
- unknown fields fail closed.
- every `shared_relations` entry is typed as
  `{relation_id,source_predicate,proposed_target_predicate}`; a blended free-form relation that cannot keep
  the source-supported and target-unvalidated halves separate fails closed.
- evidence refs and contradiction refs must resolve to the frozen source manifest.
- every source-mechanism claim and every `shared_relations` source-side predicate must have at least one
  `source_support_edges{source_id,evidence_ref}` entry resolving to a positively verified source ref;
  a contradiction ref cannot be its only support. The target-side predicate and assertion that the relation
  transfers are never supported by a source edge.
- the packetizer may read target public configuration but never hidden instances, executor results, prior
  candidates, or arm outcomes.
- raw source excerpts are stored beside the packet for audit but are excluded from the
  `RELATIONAL_PACKET` generation prompt.
- `TRANSFER-EDGE` relation ids must resolve to the packet.

## 4. Stage 0 — instrument qualification

Before any scored arm-generation model call:

1. freeze the source pool, public task templates, source selector, packetizer/scaffold prompts, model,
   tokenizer/padding renderer, retry policy, call counts, token caps, and relation-shuffle algorithm/seed;
   build and freeze matched packets/scaffolds using only target public configuration plus the frozen source
   pool, accepting the first schema-valid output under the retry policy (no human best-of-N/cherry-picking;
   exhaustion invalidates the task set), then derive and freeze the no-fixed-point relation-shuffled packets;
2. generate hidden instances exactly once from manifest-bound seeds **after** packets are frozen, then seal
   them from packet/source authors, selection/packetizer calls, and every generation prompt;
3. exhaust the full executable strategy space and prove the evaluator has at least one improving strategy
   above the frozen validation margin for every task; any failure invalidates/versions the instrument — no
   selective seed resampling or task replacement;
4. property-test packet canonicalization, invalid evidence refs, dangling/missing source-support edges,
   unknown fields, empty mappings, empty relations, missing disanalogy marker, generator-authored
   status/evidence attempts, and the relation shuffle; prove all shuffled ids resolve, serialized shape/token
   envelope stays inside tolerance, and no shuffled mapping signature equals the matched packet;
5. verify no target DSL token, hidden item id, hidden score, or evaluator output occurs in packet/scaffold
   fields beyond the public closed-vocabulary declaration shared by every arm;
6. run a pre-evidence sensitivity simulation from the exhaustive oracle/task distribution, freeze the
   minimum-detectable paired effect report, and either accept K=30 or version this preregistration before
   calls; K may never expand in response to observed arm outcomes;
7. freeze task, hidden-instance, source, scaffold, packet, parser, evaluator, padding-renderer, and prompt
   hashes in `manifest.json`; verify post-freeze mutation fails before the first scored arm call.

Any failure blocks B-CREATIVITY-0.

## 5. B-CREATIVITY-0 — executable transfer probe

### 5.1 Population and task construction

- K=30 paired tasks.
- Each task uses the existing exactly enumerable 45-strategy DSL from
  `.claude/tools/frontier_strategy_bench.py` or a byte-for-byte behavior-compatible successor.
- Hidden instances are generated only after source packets are frozen.
- Each task has at least 20 hidden instances and a non-empty set of strategies whose execution delta is
  greater than `VALIDATION_MARGIN` against the frozen two-strategy baseline portfolio.
- Packet/source authors do not receive hidden instances or `frontier_scan` output.

### 5.2 Five arms

All arms receive the same target public config, closed DSL, output grammar, model, candidate cap, and
generation-token cap.

1. `DIRECT`: target only.
2. `MATCHED_SCAFFOLD`: a source-blind structured target scaffold (target roles/constraints, predicted
   observation, and falsifier) produced by the same packetizer model, one-call budget, and output cap; it
   contains no source facts, mappings, shared relations, or transfer hypothesis.
3. `VERIFIED_EVIDENCE`: the same matched target scaffold plus the same bounded verified source
   mechanisms/facts, provenance, and support edges as treatment, but no object mappings, shared relations,
   transfer hypothesis, or disanalogies.
4. `RELATIONAL_PACKET`: the complete valid packet, excluding raw excerpts.
5. `SHUFFLED_PACKET`: the current task's complete packet with object-mapping targets and relation-linked
   transfer clauses permuted by the frozen no-fixed-point bijection. Source facts/evidence, target id,
   resolvable ids, field counts, and rendered token envelope remain fixed; only the relational alignment is
   wrong by construction.

`VERIFIED_EVIDENCE` is rendered by stripping relational fields from the already-built matched packet, so
source selection/retrieval/packetizer calls and their costs are identical to treatment, not free or rerun.
`SHUFFLED_PACKET` is derived from that same packet. `MATCHED_SCAFFOLD` uses the frozen source-blind matched
call and explicitly controls the extra target-decomposition compute. The generator never sees an arm name,
treatment/control label, expected direction, or gate threshold; every arm uses the same neutral wrapper.

Candidate cap: exactly 6 free-form markdown cards per task/arm. Post-hoc parser rejects invalid cards.
One model generation call per task/arm. The packetizer/scaffold builders use the same model family, one-call
budget, and output cap. `MATCHED_SCAFFOLD`, `VERIFIED_EVIDENCE`, `RELATIONAL_PACKET`, and
`SHUFFLED_PACKET` non-target prompt envelopes are matched within +/-2% input tokens using a frozen neutral
padding renderer; actual unpadded/padded tokens are both reported. Research, selection, scaffold, packet,
generation, and execution costs remain separate and no arm gets unreported free preprocessing.

Arm order is randomized per task with seed `20260712` and recorded in the manifest. Model and maximum
tokens are frozen in the first evidence manifest; the planned default is the same bridge/model class used
by B-3 unless unavailable before any evidence call. An unavailable model causes a preregistration version
bump, not silent substitution.

### 5.3 Deterministic candidate lifecycle

For every parsed candidate:

1. compute exact DSL mechanism signature;
2. execute against all hidden instances;
3. record invariant validity and normalized score;
4. mark improving iff invariant-valid and delta > frozen `VALIDATION_MARGIN`;
5. deduplicate by exact signature within the arm/task;
6. write a blake3 execution receipt over task hash, signature, per-instance outputs, and evaluator version.

### 5.4 Metrics (unfused)

Primary endpoint P1 — `validated_recovery`:

`unique improving signatures discovered / all improving signatures in the enumerated space`.

Primary endpoint P2 — `best_execution_delta`:

best invariant-valid candidate normalized-score delta over the frozen baseline.

Binding guard G1 — `validity`:

parsed, invariant-valid candidates / requested candidates.

Binding guard G2a — `source_support_rate`:

positively verified source refs plus complete source-mechanism/relation-source-predicate support edges /
required source support. Arms with no source input report N/A rather than a synthetic 1.0.

Binding guard G2b — `target_grounding_rate`:

resolvable target DSL/anchor/falsifier refs / required target refs.

Binding guard G3 — `cost_normalized_frontier_yield`:

unique improving signatures divided by normalized total research + packetization + generation + execution
cost. Tokens, calls, latency, and executions remain separate report columns. For the locked relative-change
non-inferiority calculation, each bootstrap replicate uses denominator `max(control_yield, 1e-12)` so a
zero-yield evidence control is defined rather than silently dropped.

Secondary diagnostics only:

- Uniqueness (unique valid signatures / valid signatures);
- raw candidate diversity;
- total improving count;
- exact source/candidate lexical overlap;
- object/relation coverage;
- packet use rate;
- parse error and failed-call counts;
- any blind human or cross-family novelty/analogy judgments.

No secondary metric can promote a failed primary gate.

### 5.5 Statistics

- Paired task-level deltas.
- 5,000-sample percentile bootstrap CI95 with seed `20260712` for each endpoint/comparison.
- Report means, medians, every task delta, and the complete bootstrap vector hash.
- No dropping parse failures, failed calls, or zero-result tasks. A transient call retry follows the frozen
  bridge policy; exhausted calls score zero candidates and stay in the denominator.

### 5.6 Joint decision rule

`SUPPORTED` only if all eleven conditions hold:

1. P1 `RELATIONAL_PACKET - VERIFIED_EVIDENCE` CI95 lower bound > 0;
2. P1 `RELATIONAL_PACKET - DIRECT` CI95 lower bound > 0;
3. P1 `RELATIONAL_PACKET - MATCHED_SCAFFOLD` CI95 lower bound > 0;
4. P1 `RELATIONAL_PACKET - SHUFFLED_PACKET` CI95 lower bound > 0;
5. P2 `RELATIONAL_PACKET - VERIFIED_EVIDENCE` CI95 lower bound > 0;
6. P2 `RELATIONAL_PACKET - DIRECT` CI95 lower bound > 0;
7. P2 `RELATIONAL_PACKET - MATCHED_SCAFFOLD` CI95 lower bound > 0;
8. P2 `RELATIONAL_PACKET - SHUFFLED_PACKET` CI95 lower bound > 0;
9. G1, G2a, and G2b one-sided non-inferiority lower bounds are >= -0.05 versus `VERIFIED_EVIDENCE`;
10. G3 one-sided non-inferiority lower bound is >= -0.05 relative change versus the per-bootstrap best of
    `DIRECT`, `MATCHED_SCAFFOLD`, and `VERIFIED_EVIDENCE`;
11. zero manifest/hash/hidden-answer/support-edge/token-envelope violations.

`NULL` if the run is valid but any primary improvement condition fails without a material harm condition.

`REFUTED` if P1 or P2 point estimate is <= 0 versus any binding causal control
(`DIRECT`, `MATCHED_SCAFFOLD`, `VERIFIED_EVIDENCE`, or `SHUFFLED_PACKET`), or a grounding/cost guard falls
below its locked margin.

Raw diversity or judge novelty with no joint P1+P2 win is explicitly NULL/REFUTED, never PARTIAL support.

Disposition:

- SUPPORTED -> B-CREATIVITY-1 may start.
- NULL/REFUTED -> no tool adoption, no Rust type, no runtime plan execution. Keep the benchmark and result.
- INVALID -> fix the instrument under a new preregistration version; do not interpret outcomes.

## 6. B-CREATIVITY-1 — live planning-tool value

This stage is forbidden until B-CREATIVITY-0 is SUPPORTED.

### 6.1 Corpus

- K=30 paired ELAI improvement tasks.
- Every task freezes real target anchors from `elai.map`/live paths, a source-evidence set, an independently
  built packet, a closed micro-executor kind, and hidden expected observations.
- Executor kinds are limited to `grep_presence`, `parser_fixture`, `bounded_simulation`, and
  `benchmark_delta`; a task requiring a full `.elai_cc` build or live deployment is excluded before the run.
- At least 20 tasks must use a target-side executable micro-experiment; at most 10 may use a deterministic
  static acceptance predicate. Report strata separately.

### 6.2 Arms and budgets

The same five arms and all B-CREATIVITY-0 generation controls apply. Each arm generates the same number
of v1.1 candidate cards. The normal deterministic triage runs before micro-execution. Refuted candidates
remain in the artifact and score zero value.

### 6.3 Primary endpoints

P1 — `execution_validated_mechanism_yield`: unique candidates per task that pass anchor, demarcation,
packet/provenance, and target micro-execution checks, deduplicated by the frozen mechanism fingerprint.

P2 — `best_target_value_delta`: best target micro-executor improvement over the task's strong direct
baseline, normalized inside each task before pairing.

Guards: real-anchor rate, source-evidence validity, target-receipt validity, token/call/latency cost, and
plan-gate pass rate. Literature/feature-map novelty and blind plan preference are secondary.

### 6.4 Decision rule

Use the same eleven-condition joint rule as B-CREATIVITY-0, replacing Recovery with
`execution_validated_mechanism_yield` and best execution delta with `best_target_value_delta`. For condition
9, B1's real-anchor, source-support, target-receipt, and plan-gate pass rates replace B0's G1/G2 rates;
condition 10 uses B1 total cost-normalized mechanism yield. Condition 11 remains zero manifest/leakage/
support-edge/token-envelope violations.

- SUPPORTED -> canonical harness skill adoption may start; B-CREATIVITY-2 may be authored/executed.
- NULL/REFUTED -> keep packet/bench utilities only; do not change paper-to-plan, idea-intake, or runtime.
- A B0 win plus B1 loss is `SYNTHETIC-ONLY`, not partial production support.

## 7. B-CREATIVITY-2 — ELAI explore/grounding value

This stage is forbidden until B-CREATIVITY-0 and -1 are both SUPPORTED and the L-10/L-23/L-24 seams
exist in the live checkout.

### 7.1 Design

- K=30 paired explore tasks from the L-10 evaluation supply; fewer than 30 valid fixtures delays/invalidates
  the run rather than lowering K after inspection.
- Arms: normal explore control and `RELATIONAL_PACKET` explore treatment; `MATCHED_SCAFFOLD` and
  `VERIFIED_EVIDENCE` ablations are required, and `SHUFFLED_PACKET` remains a negative arm where safe.
- Same model, grounding/retrieval availability, effort budget, candidate cap, seed schedule, and target
  validators.
- Observe-only: no routing/default behavior change during evidence collection.

### 7.2 Primary endpoint

`receipt_valid_cluster_coverage`: unique semantic meaning clusters whose representative conjecture carries
at least one target grounding receipt and an executable falsifier.

Binding gates:

1. receipt-valid cluster coverage `RELATIONAL_PACKET - normal explore` CI95 lower bound >0;
2. coverage `RELATIONAL_PACKET - MATCHED_SCAFFOLD` CI95 lower bound >0;
3. coverage `RELATIONAL_PACKET - VERIFIED_EVIDENCE` CI95 lower bound >0;
4. coverage `RELATIONAL_PACKET - SHUFFLED_PACKET` CI95 lower bound >0;
5. best target verified-outcome `RELATIONAL_PACKET - normal explore` CI95 lower bound >0;
6. best outcome `RELATIONAL_PACKET - MATCHED_SCAFFOLD` CI95 lower bound >0;
7. best outcome `RELATIONAL_PACKET - VERIFIED_EVIDENCE` CI95 lower bound >0;
8. best outcome `RELATIONAL_PACKET - SHUFFLED_PACKET` CI95 lower bound >0;
9. PS-3 hallucination delta non-inferior within -0.05 versus normal explore;
10. N-16 exogenous-grounding ratio does not fall below its measured floor;
11. cost-normalized verified-cluster yield non-inferior within -0.05 relative change versus the
    per-bootstrap best of normal explore, `MATCHED_SCAFFOLD`, and `VERIFIED_EVIDENCE` (zero denominator uses
    the same `1e-12` rule);
12. negative retrieval tests prove packets cannot enter code-fix, auditor, critic, or verified-case paths.

Only all-twelve SUPPORTED permits a production explore consumer. Any lane-wall failure is REFUTED regardless
of creativity metrics.

## 8. Frozen artifact layout

Each evidence run writes:

```text
workflow/benchmarks/research-coupled-creativity-<stage>-<date>/
  manifest.json
  preregistration.blake3
  source-manifest.json
  packet-manifest.json
  task-manifest.json
  arm-order.json
  cache/
  scaffolds/
  packets/
  raw/
  executions/
  results.json
  bootstrap.json
  report.md
```

`manifest.json` binds the exact tool modules, fixtures, prompts, packet schema, model, tokenizer, neutral
padding renderer, bridge policy, seeds, candidate cap, token cap, evaluator, validation margin, and this preregistration. Evidence code/fixtures are
frozen before the first model call; post-run files must hash-identically to the manifest.

## 9. Required report language

Permitted on support: "On the locked paired benchmark, research-conditioned relational packets improved
execution-valid mechanism recovery/value under the stated scope."

Forbidden until B-CREATIVITY-2 support: "ELAI is more creative", "ELAI discovers", "grounded creativity
is solved", or any generalization beyond the measured tasks/models.

Required on null/refutation: state the exact failed joint condition and preserve all negative artifacts.

End of research-coupled-creativity-preregistration-2026-07-12.md.

================================================================================
## VERSION 2 (2026-07-13) — padding-renderer amendment after INVALID run 1
================================================================================

Run 1 (workflow/benchmarks/research-creativity-b0-2026-07-13/, manifest 7718aa1c...) returned
JOINT VERDICT INVALID: condition 11 fired on 30/30 tasks because the frozen NEUTRAL padding
renderer equalized a whitespace-token PROXY, while the real kimi-k2-7 tokenizer input spread
across the four matched non-target arms was 5-13% per task, outside the +/-2% tolerance of
section 5.2 ("matched within +/-2% input tokens"). Per section 9, the arm outcomes of run 1
are not interpreted; all artifacts are preserved as evidence of the instrument defect.

CHANGED in v2 (the ONLY change):
- The padding renderer measures REAL input tokens via the frozen bridge (max-tokens-1
  measurement calls whose usage.input_tokens is authoritative) during Stage-0 preparation,
  and pads each shorter non-target arm prompt with the neutral pad token until all four
  matched arms are within +/-2% REAL input tokens of the longest, re-measuring after each
  pad step. Iterations are capped at PAD_MATCH_MAX_ITERS=4 per prompt; failure to converge
  is fail-closed instrument INVALID (no tolerance widening). Measurement calls are part of
  the frozen preparation call budget, are cached/auditable like all bridge calls, occur
  BEFORE prompt freezing, and never touch hidden instances. Both unpadded and padded real
  token counts are sealed per arm. The post-call real-token re-check of condition 11 is
  unchanged and remains the enforcement backstop.

UNCHANGED: every arm definition, both primary endpoints, all comparisons, margins, K=30,
task construction and stride, model (kimi-k2-7), candidate cap, generation token cap,
parser, executor, bootstrap procedure (5,000 samples, seed 20260712), arm-order seed,
guards G1/G2a/G2b/G3, all eleven joint conditions and the decision rule, the source-pool
and packet/scaffold semantics, and every exclusion. The corpus base seed qualification
procedure is unchanged (whole-corpus, frozen pre-call).

Run 2 executes from zero in a NEW artifact directory under this v2. No preparation or
scored artifact of run 1 is reused (fresh prompts, fresh freeze, fresh hidden instances).

================================================================================
## VERSION 3 (2026-07-13) — envelope-meter hardening after INVALID run 2
================================================================================

Run 2 (workflow/benchmarks/research-creativity-b0v2-2026-07-13/, manifest 9b78f200...) returned
INVALID: the v2 real-token pad-match sealed all 30 tasks within tolerance (max 1.23%), but the
kimi-k2-7 lane's usage.input_tokens meter proved NON-DETERMINISTIC and occasionally physically
impossible (readings exceeding prompt character count, impossible for BPE; audit cycle-4, 16/16
byte-identity of sent vs sealed prompts proven). One implausible prep reading also poisoned one
task's pad loop (T03). 4/30 scored-time re-check violations; conditions 1-10 additionally failed
on the measured data. Per section 9 the outcomes are not interpreted.

CHANGED in v3 (the ONLY changes; both are meter robustness, neither relaxes what condition 11
protects — equal arm budgets — because budgets are the PROMPT BYTES SENT):
1. Preparation measurement: every pad-decision reading is the MEDIAN OF 3 bridge measurement
   calls, and every reading must fall in the physical plausibility band [prompt_chars/8,
   prompt_chars/2]; an out-of-band median after 3 further re-measurements is a fail-closed
   instrument INVALID for the run (never silently accepted, never padded on).
2. Condition-11 token-envelope leg at scored time: the violation test becomes (i) the byte-hash
   of every prompt actually sent must equal its sealed manifest hash (any mismatch = violation),
   AND (ii) the sealed padded real-token spread (the prep-frozen, plausibility-banded quantity)
   must be within the unchanged +/-2% tolerance. The scored-time usage.input_tokens is still
   recorded per arm and any plausible deviation from its sealed value is reported as METER-DRIFT
   (INFO) in report.md, but a drifted meter reading over byte-identical prompts is no longer a
   violation: the bytes are the budget. All other condition-11 legs (hidden leakage, support
   edges, artifact hashes) are unchanged.

UNCHANGED: everything else from VERSION 2 verbatim — all arms, endpoints, comparisons, margins,
K, seeds, model, caps, parser, executor, bootstrap, guards, decision rule, source-pool semantics,
exclusions, and the v2 pad-to-match procedure itself (now fed by median-of-3 banded readings).

Run 3 executes from zero in a NEW artifact directory under this v3; nothing from runs 1-2 is
reused. Runs 1-2 remain archived as INVALID instrument-defect evidence.

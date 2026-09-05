# DeepReason steal analysis + improvement design — 2026-07-08

Sources read this session, in full or head:
- repo: C:/Users/user/Downloads/DeepReason-main/DeepReason-main (github.com/AHepi/DeepReason)
  - docs/harness-spec-v1.3.md (524 lines, the normative spec)
  - docs/CAN_LLMS_EXPLORE.md (the research write-up, all numbers from committed replayable logs)
  - README.md, mini/README.md, mini/minireason/{loop,rotate,gate,checks}.py (the measured-20% build, ~880 lines)
  - reddit thread archive (r/ClaudeAI 1upqzun) incl. OP's own claims
- ELAI context: phase-index-MASTER head, B-PLAN-SUMMARY, C/D/E/F/G/H/I/K/L/M/N SUMMARY_INDEX (full), J-SUMMARY_INDEX head (259KB file, premise + J-1..J-3 detail; J-83..J-86 GRC-v3 + context tracks known via K-15/K-17 recaps)
- skills loaded: spec-eval (AS-SPECIFIED mode active), paper-to-plan, idea-intake, oracle read for routing

Mode: spec-eval AS-SPECIFIED. ELAI is judged on its spec; shipped-vs-planned is sequencing only and
appears solely in the readiness note at the end. This file is the PHASE-4-shaped steal artifact;
plan authoring (phase 5) is deliberately NOT done this session — verdicts below name the target
units so a later session can author with the feature-map ledger check (paper-to-plan phase 4 rule:
re-check docs/thefinalglance/ELAI-FEATURE-MAP.md REJECTED/ALREADY-TRIED ledger before any NEW file).

---

## 1. What DeepReason actually is

A Popperian conjecture-criticism harness. One core invariant (spec §0), quoted because it IS ELAI's
own doctrine: "The harness is deterministic and carries all epistemology. The LLM is a bounded pure
function: pack -> schema-validated JSON. The LLM MUST NOT hold graph state, adjudicate, or control
flow." Model proposes; deterministic harness disposes. Everything lands in an append-only JSONL
event log that replays byte-for-byte; state is a materialized view; nothing is deleted; token
accounting is verified (caught a real 1% spend leak).

Mechanics that matter:
- Candidates are "skeletons": {claim, mechanism, scope, forbidden[]} where each forbidden case is a
  falsifier THE CANDIDATE ITSELF DECLARES, compiled at registration into an executable commitment
  (predicate:/program: run mechanically, rubric: judged offline). Forbid nothing => refuted by a
  program on arrival. Demarcation (falsifiability) made structural, zero LLM cost.
- Two-pass adjudication: Dung grounded extension over attack edges, then support-DAG cascade.
  Refuting a premise suspends dependents (orphaned != false). Warrants carry attackable validity
  nodes; refute a standard (rubric) and every verdict issued under it falls and its targets
  reinstate — case-law closure, computed not curated.
- Capture control (§11, the novel part): two failure surfaces named — GENERATOR capture (variation
  collapse into a basin) and ADJUDICATOR capture (criticism ritualizes while statuses stay formally
  correct). Schools = persistent conditioning regimes (islands in conjecture, panmixia in
  criticism); anti-relapse gate (hash -> embedding-neighbor -> battery-equivalence) blocks ONLY
  re-proposals of refuted-equivalents; negative case law lives at the gate and is NEVER rendered
  into prompts; Verbalized Sampling (each call returns a distribution of K candidates with
  typicality estimates); Pareto retention over (hard-to-vary, reach, coverage); λ = windowed
  fraction of verdicts grounded in program/observation vs rubric, with a floor and a brake.
- MiniReason = the measured 20%: mechanical checks only, refuted-relapse gate + gate-block-rate
  orbit detector, stance rotation (decay 5) + problem turnover (K=8), offline judge with
  order-swap + verbosity penalty + degraded-control gate. Everything else was A/B-cut on
  measurement (2-judge trial, paraphrase screens, embedding convergence detection, complement
  directive, Dung reinstatement — all refuted-or-placebo at strong-generator regimes).

Measured findings (his own logs, honest caveats carried: n=1 per condition, single seed,
scale-blind 128-dim hashing embedder, self-diversity metric only):
- F1: the novelty fade is MODEL-INTERNAL repertoire exhaustion, not prompt echo. Blind arm (never
  sees its own output) decayed the same as control (0.888 vs 0.846 late/early).
- F2: a distinct, expensive failure exists — refuted-attractor ORBITING (re-proposing near-copies
  of a dead idea; 54 gate blocks, 4.3x token burn per registered idea). The clean detector is a
  zero-embedding counter: gate blocks per window. It separated healthy/orbiting perfectly on all 15
  roots; the embedding detector could not (medians 0.645 vs 0.671).
- F3: what works is STRUCTURAL — fast stance rotation (late/early 0.973 vs 0.846) and problem
  turnover (the only novelty-RAISING force, 1.12). "Avoid the likely continuation" complement
  directives = placebo (steered away from the prompt, gained zero novelty — moves output relative
  to the PROMPT, not out of the model's REPERTOIRE).

## 2. Correcting the frame: did it "beat Fable"?

No — and the OP does not claim it did. His words: DeepReason on DeepSeek V4 Flash was "marginally
better than Fable 5" (vanilla) ON HARD OPEN EXPLANATORY QUESTIONS, n=1-class evidence, judged by
his own instruments (which he says measure "best-argued against a rubric," not "correct"). The real
result is: A CHEAP MODEL INSIDE A DETERMINISTIC EPISTEMOLOGY HARNESS ≥ A FRONTIER MODEL RAW, at ~1%
of the cost, on the question class the harness fits. That is not "DeepReason > Fable"; it is
"harness > model" — which is ELAI's founding thesis (J DD-wave J-39..J-47 determinism doctrine;
L-20 premise: the codebuff read was "~80% CONFIRMATION of ELAI's harness>model thesis"). DeepReason
is the second independent confirmation, this time in the open-ended reasoning domain where ELAI's
corpus is thinnest. That is exactly why it is worth stealing from, and exactly why it is not a
threat.

## 3. How to improve HIS results (the part he asked the community for)

Ranked by leverage. Items 3.1-3.2 are the user's own idea ("guide the model deterministically
without flooding it with a big prompt + prompt caching / KV reuse") made concrete — and they are
the fix for his stated blocker (models <50B can't follow the template).

### 3.1 Replace template-following with grammar-constrained decoding (the <50B fix)
His small-model failure is 100% an output-format problem: the conjecturer must emit
schema-valid JSON (candidates list + typicality) whose content field is ANOTHER JSON skeleton
EMBEDDED AS A STRING. That double-encoded shape is the worst case for weak models — his own code
admits it: loop.py's Candidate._coerce validator exists because "models asked for skeleton content
often emit the skeleton as a JSON object instead of an embedded string." The repair loop
(call.py) then burns retries (retry-exhausted spend was 8.4% of a live run).
Fixes, in order:
1. FLATTEN the schema (candidate.content as a nested object, not an embedded string). Free.
2. GRAMMAR-MASK the decode: compile the skeleton schema to a CFG/JSON-schema mask and constrain
   generation so the output is valid BY CONSTRUCTION — the instructions move out of the prompt and
   into the logit mask, costing zero context tokens and zero adherence failures. This is ELAI's
   E-35 XGrammar substrate / C-26 DecodingMode / N-13 OutputBoundary resolver (provider strict >
   native tool > local CFG > regex > parse-validate-retry) applied to his conjecturer role.
   Small local models (7B) hold the semantic load only; the format is enforced deterministically.
3. DECOMPOSE the elicitation for the weakest models: ask claim / mechanism / forbidden cases as
   separate short constrained calls sharing one cached prefix, instead of one giant templated
   emission. Many small calls with a hot prefix < one big call with retries.
[grounding: repo code read (loop.py:29-35, mini/README "schema repair"); ELAI E-35/C-26/N-13 spec;
XGrammar/constrained decoding are corpus-verified units, not training memory]

### 3.2 Cache-stable pack layout + prefix-KV reuse (the cost/scale fix)
His pack contract re-renders per call: stance directive + recent-survivors neighbourhood are
interleaved with the static problem text (loop.py:_prompt), so every rotation and every new
survivor busts any provider prefix cache — and rotation happens every ≤5 calls BY DESIGN. Fix with
ELAI's cache discipline: frozen prefix = role instruction + problem + skeleton contract (byte-
stable across the whole run); volatile tail = stance directive + neighbourhood + VS directive,
strictly AFTER the last cache breakpoint (H-2 breakpoints, I-10 prefix-invariance + hit-rate hard
guard, L-16/L-17 pure-fold projection with enumerated recompute causes). On a local sidecar, J-36
frozen-head prefix-KV reuse gives the same effect without a provider: freeze the head KV once per
problem, re-decode only the tail per cycle. Net effect: a MiniReason cycle's marginal cost drops to
the tail tokens, which makes hundreds of cycles on a local 7B economically trivial — the regime his
attractor research actually needs (his n=1 problem is a budget problem).
[grounding: loop.py read; H-2/I-10/L-16/L-17/J-36 spec entries read this session]

### 3.3 Replace the scale-blind embedder with NLI meaning-clustering
His 128-dim hashing embedder couldn't tell within-problem from cross-problem distance (0.645 vs
0.671) — his novelty metric is nearly blind, so every late/early ratio understates real effects.
Fix: cluster candidates BY MEANING with a small deterministic-argmax NLI cross-encoder
(entailment-equivalence), i.e. semantic entropy over meaning clusters, not cosine over hashes.
That is ELAI's L-4 (semantic-entropy predictor) + F-12 (DeterministicNliJudge, local ONNX,
seed-stable) design verbatim. Paraphrases of one idea cluster together (not false novelty);
genuinely distinct mechanisms separate. His "exhaustion shows up as semantic clustering, not
verbatim repetition" finding predicts this instrument would sharpen every result he has.
[grounding: CAN_LLMS_EXPLORE instrument caveat; L-4/F-12 spec]

### 3.4 Ground-truth novelty problem sets (his open ask #3)
He cannot measure exploration past the TRAINING distribution, only within-run self-diversity. The
machinery to close it exists in ELAI's eval corpus discipline: post-cutoff / held-out task stores
with temporal splits (D-6), 13-gram contamination filters (E-54, which replaced the bypassable
9-gram), and commit-mined private corpora (L-20) that are contamination-resistant by construction.
Concrete protocol: take problems whose accepted resolution post-dates the model cutoff (e.g.
post-cutoff mechanism discoveries, held-out synthetic domains), run the harness, score whether any
survivor's MECHANISM matches the held-out answer — novelty measured as reach into verifiably
unseen territory, not self-distance.
[grounding: CAN_LLMS_EXPLORE "where I need help" §3; D-6/E-54/L-20 spec]

### 3.5 Evidence-driven stance selection instead of round-robin
His rotation is a fixed 8-stance cycle. Two upgrades: (a) classify each cycle's candidates as
structure-new vs parameter-tweak (stripped-AST/skeleton-role EditKind, ELAI N-14) and rotate toward
the stance family that historically converts on THIS problem class; (b) treat stance selection as
a bandit (Thompson sampling over per-stance novelty-yield, ELAI D-5) with the orbit detector as the
exploration trigger. Round-robin survives as the cold-start prior. Also: his own spec (§11.1
schools, §9 foreign-reviewer) says cross-FAMILY schools are the real lever when one family's basins
collapse together — MiniReason dropped families entirely; even two families (his own DeepSeek +
anything else) would test the "shared-model gravity" residue he names honestly in §17.
[grounding: rotate.py read; spec §11; N-14/D-5 entries]

### 3.6 Statistical floor
Everything is n=1 single-seed by his own disclosure. ELAI's I-38 ≥5-seed paired floor + E-9
paired-bootstrap CI is the minimum for any of his comparative claims (rotation vs control,
blind vs echo). Cheap once 3.2 makes cycles cheap.

### 3.7 Judge hardening he already specced but hasn't run
His §10.4 audits (paraphrase-invariance, premise-deletion, planted-flaw calibration,
self-preference/verbosity probes) are excellent and mostly UNWIRED in mini. ELAI's convergent
machinery: I-24 AST-preserving canaries (style-bias detection + scorer neutralization), N-15
framing-invariance with weight-zeroing, D-14 inverted-framing sycophancy probe, H-31 judge
firewall. The one thing to add that he lacks: on a detected biased judge, ZERO ITS WEIGHT for the
run (I-24/N-15 pattern) instead of merely logging the bias.

## 4. Steal analysis: what DeepReason adds to ELAI

### 4.1 Convergence map first (what ELAI already owns — do NOT re-author; GD-1)
- Append-only replayable log, state-as-view, verified token accounting = SSB (3B) + RRA (G-2) +
  I-18 chain. ELAI's is stronger (blake3 chain, attestation, redaction).
- Model-proposes / harness-disposes = the determinism doctrine (J-39..J-47) + TD-3. Identical.
- Stance rotation on stagnation = anti-spiral 4-pattern + 5-persona switch (C-10/B-22b). His
  personas: 8 epistemic stances; ELAI's: 5 problem-solving personas. Same mechanism.
- Verbalized-sampling-shaped candidate spread = I-4 best-of-N (partial — see steal S5).
- Cross-family criticism/judging = D-13 adversary + E-6 + J-45. His foreign-reviewer rule is D-13.
- Paraphrase-flip / planted-flaw / self-preference judge audits = I-24 + D-23 + N-15 + D-14.
- Dead-idea re-proposal pruning = I-8 dead-end hashing (AST/approach hash, prune before LLM call).
- Problem turnover / never-loop-a-dry-problem = I-8 voluntary abort + I-26 plateau collapse +
  anti-spiral. Convergent.
- "Measures never adjudicate, only steer attention" = ELAI's advisory-vs-gate split (TD-3,
  elai_status_query advisory boundary, K-5 LLM-never-picks-the-tier). Same invariant, both sides.
- λ-style "verdicts must bottom out in exogenous anchors" = GRC grounding levels + R4 off-agent
  artifacts + D-17 refusal SLA. Partially convergent (see steal S6).
- Skeleton forbidden-cases ≈ plan-standard EXPECTED_OBSERVATION/ON_FAIL + D-12 MEASURABLE_PROCEDURE
  + CSQ (2D) + L-5 do-test. Partially convergent (see steal S1 — the delta is real).

### 4.2 Genuine steals (verdicts in paper-to-plan phase-4 form)

S1. Claimant-authored falsifiers compiled to executable checks + a demarcation gate.
  VERDICT: EXTENSION(D-12, L-1, L-5, GRC-v3 J-83).
  The delta vs ELAI: D-12 blocks claims WITHOUT a measurable procedure; L-1/L-3 extract claims and
  round-trip them against ELAI's OWN checkers. DeepReason inverts the authorship: the GENERATOR
  must emit, with every design-claim/hypothesis/diagnosis, the specific machine-checkable condition
  that would refute it — and the harness compiles that falsifier into a runnable predicate and
  executes it at zero LLM cost ("free criticism" — the only criticism that measured cost-positive
  at low base error in his ablations). For ELAI: extend the L-1 ClaimSet with an optional
  falsifier field elicited AT GENERATION TIME (grammar-enforced via N-13, so it costs the weak lane
  nothing), compile predicate-class falsifiers through the D-11 checker registry, and add a
  DEMARCATION verdict to L-2's ClaimRisk: a Causal/Behavior claim that forbids nothing scores
  risk-floor High (it has an empty attack surface). Hypothesis-class GRC steps (debug forks,
  architecture choices) get the same contract: no falsifier, no Converge.
  Touches: claim/ (L-1/L-2), reasoning GRC-v3 (J-83), D-11, sandboxed predicate eval (his
  checks.py AST-guard, ELAI already has stronger sandboxing).

S2. Gate-block-rate orbit detector as a 5th anti-spiral pattern.
  VERDICT: EXTENSION(C-10, I-8).
  The measured insight: counting GATE REFUSALS naming the same refuted target over a window
  separated healthy from orbiting runs PERFECTLY (0 vs 54/36) with no embeddings and no tuning,
  where the embedding detector failed. ELAI's C-10 detects Spinning via identical content hashes —
  a STRICTER condition that misses near-copy orbiting around a refuted idea; I-8 prunes isomorphic
  dead ends but doesn't emit a stagnation signal from the refusal RATE. Steal: count I-8 dead-end
  prunes + L-7 gate blocks + anti-relapse refusals per window, keyed by refuted-target; rate ≥
  floor => StagnationPattern::Orbiting(target) => persona/stance rotation names the attractor it
  is escaping. ~50 lines on existing counters, journal-recomputable (GD-2).

S3. Negative case law at the gate, never in the prompt (doctrine refinement).
  VERDICT: EXTENSION(I-5 doctrine line, .claude rules).
  Two of his measured/argued findings sharpen I-5: (a) complement directives ("avoid X / be
  different") are a PLACEBO for diversity — they move output relative to the prompt, not out of the
  repertoire; (b) rendering refuted/banned content into prompts PRIMES the very content it bans and
  grows without bound — enforce tabu at the registration gate instead. ELAI's I-5 already
  hard-excludes failed-sibling tracebacks and carries a ≤30-token contrastive constraint. The
  refinement: keep the contrastive constraint ONLY for typed error-avoidance (failure class +
  location), NEVER as a diversity instruction; diversity interventions are structural (stance
  rotation, problem turnover, tail-weighted VS) and bans are enforced by deterministic gates
  (I-8 hash, S2 orbit), not prompt text. One paragraph added to the I-5/context doctrine; prevents
  a whole class of future prompt-bloat "fixes."

S4. Evaluator-refutation => verdict-collapse sweep (case-law closure).
  VERDICT: EXTENSION(F-5, N-8) — medium value, genuinely absent.
  His closure rule: every rubric verdict cites its standard; refute the standard and every verdict
  under it falls and every target reinstates, computed in one adjudication pass. ELAI's analogue
  gap: SSB verdict rows do not carry a first-class edge back to the evaluator VERSION that produced
  them, so when a checker/judge/rule is later found miscalibrated (N-8 kill-your-own-mutant, I-24
  bias detection, a frozen-tool bugfix), there is no mechanical "re-audit everything that verdict
  vouched for" sweep — invalidation is manual. Steal: (a) verdict rows carry evaluator-id +
  version-hash (cheap, additive); (b) a registered evaluator-refutation event (failed mutant
  corpus, bias detection, gate bugfix) triggers an F-5-style staleness cascade over the verdicts
  it issued: quarantine (advisory re-audit queue), never delete (append-only preserved). Fits
  N-8's governance substrate as a sixth view: "which live verdicts stand on since-refuted
  evaluators."

S5. Verbalized Sampling contract + tail-weighted selection under stagnation.
  VERDICT: EXTENSION(I-4, anti-spiral).
  I-4 fans out N candidates by re-sampling; VS elicits a DISTRIBUTION in one call (K candidates,
  each with a self-assessed typicality), then — only when the stagnation/orbit flag is up —
  spends criticism budget on the candidates the model itself marks ATYPICAL. One call instead of
  N for the propose leg (cheaper), and a principled diversity lever that is prompt-shape, not
  prompt-content (composes with S3's ban on complement directives). Typicality is advisory-only
  (TD-3): it steers budget, never adjudicates.
  [VERIFY at authoring: the Verbalized Sampling paper his spec §11.6 cites — confirm the arXiv id
  and its measured diversity-recovery claim before citing it in a plan file; from training,
  unsearched.]

S6. λ exogenous-grounding-ratio monitor with a floor.
  VERDICT: EXTENSION(D-17, K-1 dossier, N-8).
  Windowed fraction of accepted claims/verdicts whose support bottoms out in an exogenous anchor
  (program check, off-agent artifact, retrieval evidence, human ruling) vs model-judged rubric.
  ELAI enforces off-agent evidence per-gate (R4) but has no AGGREGATE closed-loop alarm: a long
  session can drift toward rubric/advisory-heavy operation with every individual gate still green.
  His honest framing survives: this detects STALLED/closed-loop dynamics, not wrong-but-stable
  consensus — which is exactly why it is a floor + brake (force retrieval/research/escalation),
  never a pass signal. One SSB-derived tile + one threshold; journal-recomputable.

S7. Adjudicator-capture metrics (criticism ritualization).
  VERDICT: EXTENSION(N-8 governance substrate) — small, novel observability angle.
  His graph-native set: attack-target entropy (is criticism probing new surfaces or re-litigating
  the same nodes), criticism debt (accepted artifacts with never-evaluated commitments),
  reinstatement-rate band, validity-node-attack rate ≈ 0 (nobody ever challenges a test = D3 dead
  in practice). ELAI translation: entropy over auditor finding-classes per window (rubber-stamp
  drift detection beyond the existing anti-rubber-stamp minimum), fraction of shipped units whose
  declared VERIFY legs never actually ran, rate at which gates themselves get challenged/amended.
  Feeds N-8's marketable governance number.

S8. The explore-mode use case itself (the flagship steal).
  VERDICT: NEW surface, ~90% composition of planned units — the one candidate for a plan file.
  ELAI has no lane for "hard open explanatory question -> map of rival explanations with their
  refutation status." B-39 /reason REFLECT refines ONE answer; L-9 research profile grounds
  token-perfect claims; the idea-framer critic loop attacks ONE framing. A `/reason --profile
  explore` (or `elai explore`) mode composes: VS-shaped conjecture spread (S5 on I-4) + stance
  rotation (anti-spiral personas) + claimant-authored falsifiers compiled to checks (S1) +
  refuted-relapse gate + orbit detector (S2, I-8) + problem turnover (I-26/I-8) + cross-family
  judging for survivors (D-13, order-swap + verbosity penalty per his judge.py = I-24/N-15
  machinery) + the survivors map as the deliverable (which explanations survived, which died and
  why, what evidence would decide) — all journaled (SSB), replayable (RRA), rendered (K-11/N-7).
  THE DIFFERENTIATOR: run the conjecturer on the LOCAL 7B with grammar-masked skeleton output
  (§3.1) and a frozen-prefix/KV-reused pack (§3.2) — i.e., ship the thing OP explicitly could not:
  the harness working on small models, for cents, offline. That is a demo no competitor and not
  even DeepReason itself has, and it exercises E-35/N-13/H-2/I-10/J-36 on a real workload.

### 4.3 Explicit REJECTS (with reasons, for the ledger)
- Dung grounded-semantics adjudicator + reinstatement as a general ELAI mechanism: REJECT for the
  code lane (ELAI's verdicts are execution-grounded, not argument-graph-grounded; his own mini cut
  it as unnecessary at strong-generator regimes). S4 takes the one valuable consequence (closure).
- Untyped-artifact ontology (everything is bytes + interface): REJECT — elegant for a philosophy
  engine, wrong for ELAI where typed SSB payloads + schema locks are the safety substrate.
- Embedding-based convergence detection: REJECT — his own data shows it failing; ELAI's L-4 NLI
  meaning-clustering is the sound replacement where semantic distance is needed at all.
- Schools as persistent lineage-conditioned populations: DEFER — the cheap 80% is stance rotation
  (shipped as anti-spiral personas) + cross-family dispatch (D-13/E-6). Full lineage islands +
  reseed machinery only earn their keep on long-horizon exploratory runs; revisit if S8 becomes a
  heavy-use surface. (His spec §17 admits all thresholds are empirical per family and domain.)
- 2-judge trial protocol / paraphrase screens in the hot loop: REJECT — his own A/B cut them;
  ELAI's judge audits run offline/periodic (I-24/N-15), same conclusion independently reached.

## 5. Recommended shape (not authored this session)

NOT a new letter. Per K-10/N doctrine and the N-2 anti-fragmentation rule, these are extensions:
- S1 falsifier-authored-claims + demarcation -> the L-T1 claim track (L-1/L-2 amendment or one new
  L file); S5 VS contract -> I-4 amendment noted in the same file.
- S2 orbiting detector + S3 doctrine line -> anti-spiral/C-10 amendment (tiny).
- S4 evaluator-closure + S6 λ-floor + S7 ritual metrics -> one N-8-adjacent unit (governance).
- S8 explore profile -> ONE new plan file (J or L family), the only NEW surface; its live leg
  doubles as the E-35/N-13 + H-2/I-10 + J-36 integration smoke on the local lane.
PROOF_SPINE (PSR-2): S1/S8 feed PS-3 (hallucination delta / grounded-claims, L-10's corpus gains a
falsifier-carrying claim class); S2 feeds PS-6-adjacent waste reduction but honestly `none` unless
measured — declare at authoring and get the user ack where `none`.
Before authoring: run the feature-map REJECTED-ledger check; re-grep live anchors (W-7); verify the
VS paper citation (S5 note); ingest nothing — the repo is local and read.

Cost note: everything above except S8 is S-to-M effort composition. S8 is M-L and is the only
piece with demo/GTM value on its own.

## 6. Honest limits carried forward
- His numbers: n=1, single-seed, scale-blind instrument, self-diversity not training-distribution
  novelty. Treat every DeepReason figure as design rationale (ID-5): ELAI measures its own numbers
  or claims nothing.
- His §17 residue applies to ELAI's version too: capture detection catches STALLED dynamics, not
  wrong-but-stable consensus; exogenous anchors (retrieval, program checks, human ruling) are the
  only lever against shared blind spots — which ELAI's R4/off-agent doctrine already centers.
- The "beat Fable" frame: marginal, one problem class, his own instruments. What is actually
  validated is the shared thesis; the useful transfer is the six mechanisms + the small-model
  serving trick, not the headline.

## IMPLEMENTATION-READINESS NOTE (sequencing only; not part of the design verdict)
Per the indexes (confirm against live code before relying): L-1/L-2/L-4/L-7 CGG are PLANNED; E-35
XGrammar substrate + sidecar LogitsProcessor chain SHIPPED-class; anti-spiral C-10 SHIPPED; I-8/I-4
planned/partial; H-1/H-2 caching ACTIVATION-planned with the RuntimeAdapter split-brain named; J-36
prefix-KV is a planned fork; N-8/N-13 authored 2026-07. Status stamps drift (I-39) — re-grep at
edit. None of this discounts the design above.

---

PHASE 5 OUTCOME (authored 2026-07-08, paper-to-plan, main-LLM in-band): 3 plan files created, fewest-files
across two families per the recommended shape (user routed S8 to L not J; J too big).
- L-22 (docs/thefinalglance/L-plan/L-22_claimant_authored_falsifiers_demarcation.md): S1 claimant-authored
  falsifiers + demarcation gate (new delta HG-5) + S5 Verbalized-Sampling contract (arXiv:2510.01171
  WebSearch-verified). PROOF_SPINE feeds PS-3.
- L-23 (docs/thefinalglance/L-plan/L-23_reason_explore_profile_conjecture_criticism.md): S8 `/reason
  --profile explore` (the one NEW surface, KEYSTONE), wiring S2 gate-block-rate orbit detector + S3
  negative-case-law doctrine (new delta family EX-1..EX-4). PROOF_SPINE feeds PS-3; the S2 leg is
  PROOF_SPINE none (user-acked 2026-07-08).
- N-16 (docs/thefinalglance/N-plan/N-16_governance_closure_grounding_floor_ritual.md): S4 evaluator-
  refutation verdict-collapse + S6 lambda exogenous-grounding floor + S7 adjudicator-ritual metrics,
  EXTENSION(N-8). PROOF_SPINE feeds PS-2 (transitive, declared).
Indexes updated: L-0_INDEX (title 23 sessions + L-T5 track + HG-5/EX deltas + table + constants + session
log), L-SUMMARY_INDEX (L-T5 track + L-22/L-23 entries), N-0_INDEX (title 16 sessions + N-16 table row +
constants), N-SUMMARY_INDEX (N-16 entry). Feature map upserted: L/N plan-family counts, L-T5 ledger clause,
the Section-4 DEEPREASON-STEAL block (all 8 verdicts + the 5 rejects), header date -> 2026-07-08. REJECTS
logged to the feature-map ledger: Dung adjudicator, untyped-artifact ontology, embedding convergence,
schools/lineage (DEFER), 2-judge-trial, the "beat Fable" frame. No commit (docs/ + workflow/ gitignored).

COLLAB-WAVE OUTCOME (authored 2026-07-11, from the live exchange — handoff deepreason-steal-and-collab.md
improvements (a)-(d)): 1 plan file created, fewest-files (all four land as legs of ONE L-T5 file; the
never-amend-old-plan-files rule rules out L-23/D-17/I-31 in-place edits, so the siblings ride as EXTENSION legs).
- L-24 (docs/thefinalglance/L-plan/L-24_conjecture_memory_middle_tier.md): (c) the conjecture-memory middle
  tier — status-explicit unproven conjectures between the journal and the VCG, EXPLORE lane only (J-96..J-98
  UNCHANGED, type-level lane wall, new deltas EX-5/EX-6; statuses = pure journal fold per DeepReason spec §4,
  orphaned != false, no absorbing status, quarantine never delete; composes N-16 evaluator-version edges;
  K-19 SF-1..SF-4 binding) + (a) the L-10 explore-arm NLI meaning-cluster novelty column under CGG ON/OFF
  (leg A) + (b) typed UnblockCondition {tried/missing/route} on D-17/L-7 refusals composed from receipts
  (leg B) + (d) the FD-1..FD-3 flywheel diversity guard as the I-30/I-31 contract-first pre-gate (leg D).
  PROOF_SPINE feeds PS-3; leg D none (user-acked 2026-07-11). plan_gate_check.py PASS (P1-P10).
Indexes updated: L-0_INDEX (title 24 sessions, L-T5 = L-22..L-24, LS-6 source, EX-5/EX-6, sessions table,
constants row at LIVE 581/272, session log), L-SUMMARY_INDEX (title + track premise + L-24 entry). Feature map
upserted: header 07-11b, L family L-1..L-24, CONJECTURE-MEMORY MIDDLE TIER entry beside the VCG block,
I-30/I-31 pre-gate clauses, Section-4 DEEPREASON-COLLAB verdicts C1-C4. Spec overview: L-section exploratory-
reasoning clause extended (design language, no ids). No commit (docs/ + workflow/ gitignored).

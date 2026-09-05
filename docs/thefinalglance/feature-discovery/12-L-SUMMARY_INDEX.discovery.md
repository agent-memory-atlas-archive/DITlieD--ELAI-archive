# ELAI L-plan exhaustive feature discovery

## Scope, evidence, and notation

This is a summary-layer discovery of L-SUMMARY_INDEX.md, not an implementation claim and not a substitute for inspecting the original L plan files. The audited source is 264 lines and was read in full. ELAI-SPEC-OVERVIEW.md lines 367-399 were used only to orient the L-plan boundary. No original L plan or grounding-map content was used to fill gaps.

- **[E] Explicit**: directly stated by the audited summary.
- **[I] Inferred**: a composition or consequence that follows from multiple explicit statements, but is not itself stated as a requirement.
- **[A] Ambiguous/missing**: named or suggested by the summary but underspecified there; deeper inspection is required.
- Source anchors use **L:n** for line n of L-SUMMARY_INDEX.md and **O:n** for line n of ELAI-SPEC-OVERVIEW.md.

Coverage:

- Summary lines audited: **264/264**.
- Detailed plan sections audited: **23/24 named sessions**: L-1 through L-14 and L-16 through L-24.
- Summary omission: **L-15 is named in the title as “receipt-minting repair controller” but has no body section**. Its existing original plan is a mandatory deep-inspection candidate.
- Tracks covered: L-T1 (L-1..L-10), L-T2 (L-11..L-14), title-only L-15, L-T3 (L-16..L-19), L-T4 (L-20..L-21), and L-T5 (L-22..L-24).
- Atomic inventory records below: **412**, plus cross-track workflows, invariants, status, and source manifests.

## End-state architecture described by the summary

1. **Claim trust loop [E].** Any backend output is structurally converted to typed claims; round-trip, oscillation, semantic-entropy, counterfactual, and demarcation evidence feed ClaimRisk; risk selectively buys more evidence; a policy gate then allows, caveats, verifies, blocks, or abstains. Contradicted beliefs are quarantined, while proven resolutions may compound into verified cases. (L:6-79, L:240-259)
2. **Generated-output quality loop [E].** Plan-time responsibility limits, generation-time critique/retry, write-time deterministic checks, and audit-time frozen authority enforce maintainability for every model. Rules begin in shadow and earn blocking authority from measured error rates. (L:83-88)
3. **Familiar user surface on the same trust engine [E].** Skills, typed rules/hooks, a one-shot builder, artifact execution, MCP access, and receipt verification expose existing machinery without bypassing admission, quality, or autonomy gates. (L:90-112)
4. **Immutable context substrate [E].** An append-only transcript is projected through a pure fold into a provider view; a named freeze-epoch state machine controls prefix recomputation; losslessness, time travel, recovery, and ambient recall become projection behavior rather than raw-history mutation. (L:117-204)
5. **Measured model orchestration [E].** Real commits supply contamination-resistant eval tasks; ELAI and vendor CLI arms run under one reproducible envelope; model capability and role deterministically select verification intensity, with unknown models receiving the strongest checks. (L:206-226)
6. **Exploratory reasoning loop [E].** Generators attach executable falsifiers to conjectures, exploration returns rival explanations rather than one collapsed answer, an embedding-free orbit detector rotates stance, and a lane-isolated conjecture store preserves unresolved ideas without contaminating proven-only code work. (L:228-262)
7. **Planning/runtime boundary [E].** The separate improvement-conjecture archive has no FDL controller, ProbeUtility scheduler, multi-cycle operator allocator, or bridge into L-23/L-24. Q-10 only proposes a future one-way SOURCE_ONLY research packet under independent target-side verification. (O:393-408)

## Exhaustive atomic inventory

### L-T1 — Causal Grounding Gate

#### L-1 — typed claim extraction substrate [foundation; HG-3 black-box]

- **L-1.01 [E]** Accept model outputs of at least answer, plan, patch, and explanation form from any backend. (L:13)
- **L-1.02 [E]** Run a deterministic structural extractor over those outputs. (L:13)
- **L-1.03 [E]** Emit ClaimSet as a vector of Claim objects carrying kind, source span, and references. (L:13)
- **L-1.04 [E]** Support ClaimKind values File, Symbol, Dependency, Api, Behavior, Causal, and Reachability. (L:13)
- **L-1.05 [E]** Recognize claims from cited paths, symbols, package names, and signatures. (L:13)
- **L-1.06 [E]** Never ask the emitting model to identify or type its own claims. (L:13)
- **L-1.07 [E]** Supply the shared intake consumed by L-3, L-4, and L-5. (L:13)
- **L-1.08 [E]** Lift the implicit claim object out of D-10 grounding-sweep rather than reimplement grounding-sweep. (L:14)
- **L-1.09 [E]** Tag claims sourced from untrusted material at extraction through G-1 taint. (L:14)
- **L-1.10 [E]** Preserve a nonexistent-file claim as kind File with refs-unresolved instead of silently dropping it. (L:16)
- **L-1.11 [E]** Run extraction in the orchestrator, outside the worker/model that emitted the output. (L:13, L:16)
- **L-1.12 [A]** Journal either ClaimSetExtracted or a reused grounding-verification row; the exact schema choice is deferred to edit time. (L:15)
- **L-1.13 [A]** Optionally expose a claims tile on the trust panel; W-4 may instead justify no frontend. (L:15)

#### L-2 — unified ClaimRisk score + convergence fuse [KEYSTONE; HG-2]

- **L-2.01 [E]** Generalize G-7 claim_score from the agent’s own output to claims from any backend. (L:20)
- **L-2.02 [E]** Represent risk with one ClaimRisk object. (L:20)
- **L-2.03 [E]** Consume P1 evidence-round-trip residue from L-3. (L:20)
- **L-2.04 [E]** Consume P2 oscillation from J-85 without rebuilding J-85. (L:20)
- **L-2.05 [E]** Consume P3 semantic entropy from L-4. (L:20)
- **L-2.06 [E]** Require agreement by at least two mechanistically distinct predictors for a High or Critical verdict. (L:20)
- **L-2.07 [E]** Cap a claim with only one firing predictor at Medium. (L:23)
- **L-2.08 [E]** Use cheap unresolved-reference or P1-residue signals to reach Medium before firing expensive predictors. (L:20)
- **L-2.09 [E]** Route the expensive-predictor budget through J-84’s effort classifier. (L:20)
- **L-2.10 [E]** Produce Low, Medium, High, or Critical tiers for L-7 policy. (L:20, L:22)
- **L-2.11 [E]** Remain valid when a backend supplies no log probabilities; logprob is optional. (L:23)
- **L-2.12 [E]** Make the score recomputable from journal evidence. (L:23)
- **L-2.13 [E]** Retain abstention-aware confident-wrong penalties and decorrelated predictor retention from H-39/H-40. (L:21)
- **L-2.14 [A]** Either emit ClaimRiskScored with the four tier kinds or generalize G-7 rows; schema and optional trust/context risk tile are edit-time decisions. (L:22)

#### L-3 — evidence round-trip predictor (P1) [activation over arbitrary claims]

- **L-3.01 [E]** Re-ground every typed claim against evidence and measure whether its cited content can be reconstructed. (L:27)
- **L-3.02 [E]** Reuse D-10/D-11 FILE, SOURCE, TEST, and CosmosCheck grounding modes. (L:27-28)
- **L-3.03 [E]** Enforce H-53 closed-set citation membership. (L:27-30)
- **L-3.04 [E]** Validate citations against the DKG through H-32. (L:27-28)
- **L-3.05 [E]** Treat an unreconstructable file, symbol, line, or source as high residual. (L:27)
- **L-3.06 [E]** Treat a present file/symbol citation as a low-residual PASS. (L:30)
- **L-3.07 [E]** Treat a nonexistent symbol as a high-residual FAIL and reject its closed-set citation marker. (L:30)
- **L-3.08 [E]** Never emit an out-of-evidence citation as supported. (L:30)
- **L-3.09 [E]** Activate shipped grounding methods over arbitrary L-1 claims instead of reimplementing any checker. (L:27-29)
- **L-3.10 [E]** Feed P1 output into L-2 with no dedicated frontend. (L:29)

#### L-4 — semantic-entropy inter-seed predictor (P3) [net-new predictor; ID-5]

- **L-4.01 [E]** Resample N alternative answers or explanations over I-4’s SessionDag best-of-N substrate. (L:34)
- **L-4.02 [E]** Cluster resamples by entailment-equivalent meaning. (L:34)
- **L-4.03 [E]** Use F-12 DeterministicNliJudge locally for clustering. (L:34-35)
- **L-4.04 [E]** Do not use embedding cosine for meaning clusters. (L:34)
- **L-4.05 [E]** Do not use an LLM judge or cloud call for meaning clustering. (L:34-35)
- **L-4.06 [E]** Emit semantic entropy across the meaning clusters. (L:34)
- **L-4.07 [E]** Interpret high semantic entropy as high claim risk. (L:34)
- **L-4.08 [E]** Route a subset of samples through D-13’s cross-family adversary. (L:34-35)
- **L-4.09 [E]** Emit CrossFamilyDiverge when a cross-family adversary contradicts same-model agreement. (L:36-37)
- **L-4.10 [E]** Emit SeedsAgree for one meaning cluster and SeedsDiverge for multiple meaning clusters. (L:36-37)
- **L-4.11 [E]** Cluster paraphrases of the same true answer together; wording variance is not semantic disagreement. (L:37)
- **L-4.12 [E]** Keep deterministic ELAI_SEED behavior through the local judge. (L:35)
- **L-4.13 [E]** Gate completion on an ELAI gold-set measurement, never a paper AUROC. (L:34)
- **L-4.14 [E]** Feed P3 into L-2 with no dedicated frontend. (L:36)
- **L-4.15 [E]** Admit the honest residual that same-family resampling can share a confidently wrong belief; cross-family testing mitigates but does not eliminate it. (L:34, L:114-115)

#### L-5 — counterfactual do-test on model claims [KEYSTONE; CANT-LIE; HG-1]

- **L-5.01 [E]** Trigger an interventional check when a model emits a Causal or Behavior claim. (L:41)
- **L-5.02 [E]** Permit an intervention that reverts only the claimed cause A. (L:41)
- **L-5.03 [E]** Permit an intervention that disables the implicated flag. (L:41)
- **L-5.04 [E]** Permit an intervention that patches only the claimed function. (L:41)
- **L-5.05 [E]** Permit an intervention that adds a trace to the claimed path. (L:41)
- **L-5.06 [E]** Compare the effect B before and after the intervention. (L:41)
- **L-5.07 [E]** Bind executable outcomes to F-4’s blake3 stdout hash so the model cannot forge them. (L:41-42)
- **L-5.08 [E]** Reuse C-28/I-20 mutation or C-29 metamorphic relations as alternative perturb-and-observe mechanisms. (L:41-42)
- **L-5.09 [E]** Label a claim causal only when an executable/interventional outcome passes. (L:41, L:44)
- **L-5.10 [E]** Label a claimed cause not-causal when reverting it does not change the effect. (L:44)
- **L-5.11 [E]** Score static, temporal, and import-edge evidence at weaker tiers and never call it causal. (L:41, L:44)
- **L-5.12 [E]** Use J-16 blast radius to bound the perturbation cone and G-1 to preserve taint. (L:42)
- **L-5.13 [E]** Drive existing anti-fabrication and mutation machinery on the claim; do not reimplement either engine. (L:42)
- **L-5.14 [A]** Prefer existing test-integrity/mutation/metamorphic rows; add CounterfactualProbed only if reuse cannot express the result. (L:43)

#### L-6 — risk-as-curiosity targeted-effort router [HG-4]

- **L-6.01 [E]** Convert High or Critical ClaimRisk into a curiosity reward that automatically buys more evidence. (L:48)
- **L-6.02 [E]** Target the extra effort at the exact weak claim rather than running a blanket verification pass. (L:48)
- **L-6.03 [E]** Allow mid-task agentic retrieval through J-38. (L:48-49)
- **L-6.04 [E]** Allow J-103 graduated read-ladder escalation on a miss. (L:48-49)
- **L-6.05 [E]** Allow a targeted test or trace. (L:48)
- **L-6.06 [E]** Allow I-7 local-to-cloud escalation. (L:48-49)
- **L-6.07 [E]** Use B-29 impact information to decide where evidence effort should look. (L:49)
- **L-6.08 [E]** Re-score the claim through L-2 after evidence arrives. (L:48)
- **L-6.09 [E]** Bound all targeted effort through J-84; do not permit an unbounded loop. (L:49-51)
- **L-6.10 [E]** Spend zero targeted effort on Low-risk claims. (L:51)
- **L-6.11 [E]** Make a newly grounded resolution eligible for J-96 Verified Case Graph minting without changing model weights. (L:48)
- **L-6.12 [A]** Reuse retrieval/escalation rows and an existing retrieval UI where possible; no new event is expected. (L:50)

#### L-7 — hallucination policy gate + abstention integration [CANT-WRECK; HG-4]

- **L-7.01 [E]** Allow Low-risk claims. (L:55)
- **L-7.02 [E]** Attach a caveat or require a citation for Medium-risk claims. (L:55, L:58)
- **L-7.03 [E]** Force L-6 targeted effort before a High-risk claim can act. (L:55)
- **L-7.04 [E]** Block the edit/action or abstain on Critical risk. (L:55, L:58)
- **L-7.05 [E]** Treat missing evidence as high risk rather than a silent pass. (L:55)
- **L-7.06 [E]** Insert the gate into AutonomousGate EvidenceCheck on every transition. (L:55-57)
- **L-7.07 [E]** Compose conformal refusal through D-16. (L:55-56)
- **L-7.08 [E]** Compose I-8 voluntary infeasibility acknowledgement and credit it through the abstention score. (L:55-56)
- **L-7.09 [E]** Compose the D-17 honest-refusal SLA. (L:55-56)
- **L-7.10 [E]** Journal the verdict as an SSB row so RRA can reproduce it. (L:55)
- **L-7.11 [E]** Run observe-only first and measure false-positive/over-block rates on at least three real runs before block-by-default. (L:55)
- **L-7.12 [E]** Default the gate on behind ELAI_HALLUCINATION_GATE_DISABLE. (L:57)
- **L-7.13 [E]** Under the disable switch, observe and record but do not block. (L:58)
- **L-7.14 [E]** Never silently allow an unverifiable High claim. (L:58)
- **L-7.15 [E]** Never let an LLM override the deterministic risk tier. (L:58)
- **L-7.16 [A]** Prefer RefusalDecision/EvidenceCheck rows; otherwise add HallucinationGateAction, with optional reuse of refusal/mode panels. (L:57)

#### L-8 — working-memory hygiene / prune spurious connections [CANT-WRECK]

- **L-8.01 [E]** Detect a repo-owner assumption contradicted by a symbol search. (L:62)
- **L-8.02 [E]** Detect a package assumption contradicted by the lockfile. (L:62)
- **L-8.03 [E]** Detect an “unused” assumption contradicted by a call-graph hit. (L:62)
- **L-8.04 [E]** Quarantine contradicted claims from the live claim/belief graph before downstream poisoning. (L:62)
- **L-8.05 [E]** Revalidate candidate claims against the current commit and AST through J-21. (L:62-65)
- **L-8.06 [E]** Use F-5’s staleness quarantine cascade. (L:62-63)
- **L-8.07 [E]** Use I-5’s pristine context injector to exclude failed-path traceback. (L:62-63)
- **L-8.08 [E]** Carry working beliefs through J-83/I-11 BeliefState. (L:62-63)
- **L-8.09 [E]** Do not re-inject a quarantined false claim downstream. (L:65)
- **L-8.10 [E]** Never prune a still-valid claim. (L:65)
- **L-8.11 [E]** Never delete the historical record; quarantine is reversible evidence retention. (L:65)
- **L-8.12 [E]** Make quarantine state recomputable from the journal. (L:65)
- **L-8.13 [E]** Provide no standalone frontend because this is runtime hygiene. (L:64)

#### L-9 — research/math deep-think grounding profile

- **L-9.01 [E]** Add a research profile selector to the existing /reason surface rather than a new reasoning engine. (L:69-71)
- **L-9.02 [E]** Run N=4 reasoners through B-39 Chat Reflect. (L:69)
- **L-9.03 [E]** Route the profile through GRC Mode A. (L:69-71)
- **L-9.04 [E]** Use D-11 constructive checkers including sympy, z3, and compile checks. (L:69-70)
- **L-9.05 [E]** Include E-53’s Formal Verification Lane. (L:69-70)
- **L-9.06 [E]** Ground against Cosmos-ingested papers from B-42, J-33 MinerU, and paper-ingestor. (L:69-70)
- **L-9.07 [E]** Round-trip every factual claim against ingested paper bytes through P1 and the H-53 closed set. (L:69-72)
- **L-9.08 [E]** Measure P3 semantic entropy across the four reasoners. (L:69)
- **L-9.09 [E]** Flag divergent derivations. (L:69, L:72)
- **L-9.10 [E]** Let the model propose and the symbolic checker adjudicate. (L:69)
- **L-9.11 [E]** Catch a wrong arithmetic step through the checker, not model self-judgment. (L:72)
- **L-9.12 [E]** Abstain with “unverified by constructive check” when sympy/z3 cannot confirm a derivation; never assert it as proven. (L:69, L:72)
- **L-9.13 [E]** Treat the feature as configuration plus a thin selector with net-zero event growth. (L:70-71)
- **L-9.14 [A]** Optionally show a research-profile badge in ReasoningDrawer. (L:71)

#### L-10 — CGG hallucination-delta bench [whole-plan done-gate]

- **L-10.01 [E]** Add the CLI command elai bench hallucination-delta. (L:76, L:78)
- **L-10.02 [E]** Run paired CGG gate-ON and gate-OFF arms over a held-out claim corpus. (L:76)
- **L-10.03 [E]** Measure unsupported-claim rate per arm. (L:76, L:79)
- **L-10.04 [E]** Measure contradicted-claim rate per arm. (L:76)
- **L-10.05 [E]** Measure reviewer-reaching rate per arm. (L:76)
- **L-10.06 [E]** Run under ELAI_SEED and make results journal-recomputable. (L:76)
- **L-10.07 [E]** Use receipts, not pass@k, on the outcome axis. (L:76)
- **L-10.08 [E]** Ship a deterministic fixture tier now. (L:76, L:79)
- **L-10.09 [E]** Gate the real-backend/live-model arm on RunPod. (L:75-76)
- **L-10.10 [E]** Report only ELAI-measured numbers, never a paper AUROC. (L:76)
- **L-10.11 [E]** Phrase the result as fewer unsupported claims per 100 outputs, never as a leaderboard rank. (L:76)
- **L-10.12 [E]** Mark CGG complete only when the ON/OFF delta is positive and recomputable. (L:76)
- **L-10.13 [E]** Emit one artifact for each arm. (L:79)
- **L-10.14 [E]** Include a deliberately broken gate-off-but-claim-savings arm whose delta must collapse, establishing the gate as the cause. (L:79)
- **L-10.15 [E]** Refuse to emit live cost/reviewer-reaching numbers until the live arm actually runs. (L:79)
- **L-10.16 [E]** Reuse SAFEDELTA/QDELTA/convergence runner shapes instead of forking a new runner family. (L:77-78)
- **L-10.17 [A]** Add at most one HallucinationDeltaArm kind and optionally an audit/cost tile. (L:78)

### L-T2 — Output Quality + Familiarity Surface

#### L-11 — output-quality enforcement system [KEYSTONE; OQ-1..OQ-5]

- **L-11.01 [E]** Apply the quality system to generated code from every model. (L:85)
- **L-11.02 [E]** Enforce quality through four distinct seams rather than one monolithic gate. (L:85)
- **L-11.03 [E]** Add one net-new frozen checker, .claude/tools/solid_proxy_check.py. (L:85, L:87)
- **L-11.04 [E]** Detect an SRP god object deterministically only when fan-out and size jointly cross the proxy. (L:85)
- **L-11.05 [E]** Block the SRP god-object finding in ACTIVE and warn in SHADOW. (L:85, L:88)
- **L-11.06 [E]** Detect DIP cross-layer concrete-vs-trait relationships through elai.map and issue WARN. (L:85)
- **L-11.07 [E]** Detect an ISP fat trait and issue WARN. (L:85)
- **L-11.08 [E]** Route OCP/LSP assessment to cross-family J-89 advisory only; never deterministically block it. (L:85, L:88)
- **L-11.09 [E]** Add a god-object MiniCritic METRIC predicate. (L:85)
- **L-11.10 [E]** At seam 1, constrain plan-time responsibility through C-31 file_plan budgets. (L:85-86)
- **L-11.11 [E]** At seam 2, run C-32 MiniCritic plus StructuralRetry during generation. (L:85)
- **L-11.12 [E]** Pair generation-time self-correction with J-55 so the mechanism applies across model modalities. (L:85-86)
- **L-11.13 [E]** Retry a weak-model god object inside the generation loop. (L:88)
- **L-11.14 [E]** At seam 3, enforce deterministic MR-1/MR-2 floors through VibeGate F-1 at write time. (L:85)
- **L-11.15 [E]** At seam 4, give audit-time authority to the frozen checker, J-65, and J-23’s generated-multilang lane. (L:85)
- **L-11.16 [E]** Keep the final verdict auditor-owned, frozen under R8, and unavailable to executor self-grading. (L:85, L:88)
- **L-11.17 [E]** Enter every quality rule into the rules-engine in SHADOW first. (L:85)
- **L-11.18 [E]** Promote a rule to BLOCK only after fpr <5%, tpr >80%, over at least three runs. (L:85)
- **L-11.19 [E]** Pass a clean module. (L:88)
- **L-11.20 [E]** Make outcomes recomputable under ELAI_SEED. (L:88)
- **L-11.21 [A]** Prefer an existing mini-critic/maintainability row, otherwise add OutputQualityVerdict; optional GatesPanel/vibesec tile is deferred to edit time. (L:87)

#### L-12 — skill front-door + self-improving skills [FS-1/FS-2/FS-4]

- **L-12.01 [E]** Give E-43 log_skill_invocation a production caller. (L:92)
- **L-12.02 [E]** Add elai skill add. (L:92)
- **L-12.03 [E]** Add elai skill improve as an on-demand curator operation. (L:92)
- **L-12.04 [E]** Scan the whole skill directory with skillspector before admission. (L:92)
- **L-12.05 [E]** Pass imports through J-102 admission. (L:92-93)
- **L-12.06 [E]** Pass skill imports/invocations through VibeGate. (L:92)
- **L-12.07 [E]** Block an injection-payload skill hidden under tests/. (L:95)
- **L-12.08 [E]** Accept Claude Code/agentskills.io v1 SKILL.md verbatim. (L:92)
- **L-12.09 [E]** Retrieve exactly BM25 top-1 skill into context_budget’s Rules slot. (L:92)
- **L-12.10 [E]** Gate retrieval/permission by F-6 SkillTrustTier S1-S4. (L:92-93)
- **L-12.11 [E]** Promote an episodic playbook to SKILL.md after at least k SSB-verified outcomes. (L:92, L:95)
- **L-12.12 [E]** Demote a skill when evaluation regresses. (L:92)
- **L-12.13 [E]** Derive promotion from verified outcome count, never an LLM opinion. (L:92, L:95)
- **L-12.14 [E]** Record benign skill admission and invocation. (L:95)
- **L-12.15 [E]** Treat skill improvement as a DATA edit, not arbitrary executable self-modification. (L:92)
- **L-12.16 [A]** Reuse skill_system_event/import logs and optionally add SkillPromoted; expose actions in SkillsPanel.svelte. (L:94)

#### L-13 — user-authorable rules & hooks surface [FS-1/FS-2]

- **L-13.01 [E]** Expose the wired rules engine as user-authorable rules-as-DATA. (L:99)
- **L-13.02 [E]** Support AST, GREP, SCHEMA, METRIC, and COMPOSITE predicates. (L:99)
- **L-13.03 [E]** Preserve the rules lifecycle and mutable=0 immune tier. (L:99)
- **L-13.04 [E]** Expose PreToolUse, PostToolUse, Stop, and PreCompact hook axes. (L:99)
- **L-13.05 [E]** Add elai rule add, list, test, promote, and retire commands. (L:99, L:101)
- **L-13.06 [E]** Create a rules-engine DATA row in SHADOW when a user adds a rule. (L:99)
- **L-13.07 [E]** Register the rule against a hook axis through J-52 path-keyed injection. (L:99-100)
- **L-13.08 [E]** Represent a user rule as a typed predicate plus verdict. (L:99)
- **L-13.09 [E]** Reject executable user-rule content. (L:99, L:102)
- **L-13.10 [E]** Pass rules through J-102 admission and content scanning. (L:99-100)
- **L-13.11 [E]** Reject any rule that weakens a mutable=0 immune check. (L:99, L:102)
- **L-13.12 [E]** Permit user rules only to add checks, never subtract safety. (L:99)
- **L-13.13 [E]** Keep hooks at the IPC layer and invisible to the acting agent. (L:99)
- **L-13.14 [E]** Record would-WARN without blocking while a rule is in SHADOW. (L:102)
- **L-13.15 [E]** Refuse ACTIVE promotion until the false-positive bound clears. (L:99, L:102)
- **L-13.16 [E]** Make firings recomputable from the journal. (L:102)
- **L-13.17 [E]** Add authoring to RulesDrawer/ToolsHooksDrawer. (L:100-101)

#### L-14 — one-shot verified builder front-door [FS-3/FS-4]

- **L-14.01 [E]** Add elai build "<idea>" as a one-shot CLI front door. (L:106)
- **L-14.02 [E]** Add a dashboard one-field build flow. (L:106)
- **L-14.03 [E]** Drive PIPELINE FACTORY L1 to L2 with production defaults. (L:106)
- **L-14.04 [E]** Invoke E-12 full-stack code generation and templates/web. (L:106-107)
- **L-14.05 [E]** Require the generated output to pass L-11 quality gates. (L:106-109)
- **L-14.06 [E]** Wrap every successful output through I-40 so it ships an RRA receipt. (L:106-108)
- **L-14.07 [E]** Allow elai rra verify to validate the emitted receipt. (L:109)
- **L-14.08 [E]** Offer elai artifact run under Tier-C via J-63. (L:106-107)
- **L-14.09 [E]** Expose build as a J-2 MCP verb for Claude Code/Codex clients. (L:106-108)
- **L-14.10 [E]** Report quality findings when generation fails L-11 and never silently mark it DONE. (L:109)
- **L-14.11 [E]** Ship a deterministic fixture build now. (L:109)
- **L-14.12 [E]** Gate live cloud/local builds on RunPod/J-63. (L:109)
- **L-14.13 [E]** Suppress “$20=$200”, “cheaper”, or equivalent value claims until SAFEDELTA/QDELTA measures them. (L:106, L:109)
- **L-14.14 [E]** Reuse pipeline, RunArtifactEmitted, receipt, quality, and autonomy rows with expected net-zero schema growth. (L:107-108)
- **L-14.15 [E]** Use ProjectSelector/Intro for the one-field frontend. (L:108)

#### L-15 — receipt-minting repair controller

- **L-15.01 [E]** The summary title names L-15 as a “receipt-minting repair controller.” (L:1)
- **L-15.02 [A]** No L-15 body, behavior, dependency, status, interface, event, test, or interaction is present in the audited summary. Nothing beyond the exact title is safe to infer. The existing original L-15_repair_controller_and_evidence_first_reorder.md must be inspected. (L:1, L:111-117)

### L-T3 — Context Core

#### L-16 — immutable transcript + pure fold-view context core [KEYSTONE; CC-1..CC-4]

- **L-16.01 [E]** Make the append-only SessionDag plus H-49 CAS transcript the single source of truth. (L:135, L:140-142)
- **L-16.02 [E]** Introduce ContextProjection fold(log, budget, policy, freeze) -> view. (L:135-137)
- **L-16.03 [E]** Make fold pure and forbid it from mutating the log. (L:136)
- **L-16.04 [E]** Treat E-25 ladder tiers as a fold strategy/parameter. (L:137, L:140-142)
- **L-16.05 [E]** Treat H-6 masking/read-back as a fold strategy/parameter. (L:137)
- **L-16.06 [E]** Treat the 3E allocator as the budget parameter. (L:137, L:140)
- **L-16.07 [E]** Treat H-50 context policy as the policy parameter. (L:137, L:140-142)
- **L-16.08 [E]** Replace conversation.rs compact_session prefix mutation with per-turn projection. (L:138, L:143-144)
- **L-16.09 [E]** Convert recover_stubs into recomputation with a larger budget. (L:138-139, L:143-144)
- **L-16.10 [E]** Make HD-2 cache stability a type property of projection. (L:139)
- **L-16.11 [E]** Keep conversation state as the immutable log and call fold for provider input. (L:143-144)
- **L-16.12 [E]** Default projection on behind ELAI_CONTEXT_PROJECTION_DISABLE. (L:145-146)
- **L-16.13 [E]** Retain the old mutating path for one release as rollback. (L:145-146)
- **L-16.14 [E]** Produce a byte-identical view prefix across non-epoch turns. (L:147-148)
- **L-16.15 [E]** Require turn-2 cache_read to cover that stable prefix. (L:147-148)
- **L-16.16 [E]** Recompute a previously skeletonized span verbatim in a later turn without reading an SSB recovery stub. (L:148)
- **L-16.17 [E]** Keep raw log byte length monotonically non-decreasing. (L:149)
- **L-16.18 [E]** Reject an attempted raw-log mutation. (L:149)
- **L-16.19 [E]** Fail the byte-identity assertion if fold reads wall-clock time. (L:149-150)
- **L-16.20 [A]** Prefer ContextReduction and optionally add ContextProjected after schema re-grep. (L:144-145)

#### L-17 — freeze-epoch controller [CC-2/CC-3]

- **L-17.01 [E]** Store a frozen view in FreezeState. (L:154)
- **L-17.02 [E]** Store a boundary fingerprint in FreezeState. (L:154)
- **L-17.03 [E]** Store freeze counters in FreezeState. (L:154)
- **L-17.04 [E]** Implement pure evaluate_freeze returning either Reuse or Recompute(cause). (L:154-156)
- **L-17.05 [E]** Enumerate FirstCall as a recompute cause. (L:155)
- **L-17.06 [E]** Enumerate ColdGapTtl as a recompute cause. (L:155)
- **L-17.07 [E]** Enumerate TailOverflow as a recompute cause. (L:155)
- **L-17.08 [E]** Enumerate BoundaryMismatch as a recompute cause. (L:155)
- **L-17.09 [E]** Enumerate HistoryRewound as a recompute cause. (L:155)
- **L-17.10 [E]** Enumerate PolicyChanged as a recompute cause. (L:156)
- **L-17.11 [E]** Enumerate PressureCeiling as a recompute cause. (L:156)
- **L-17.12 [E]** Append tail content without recomputing the frozen prefix. (L:156, L:163-164)
- **L-17.13 [E]** Journal every recompute cause. (L:156-157)
- **L-17.14 [E]** Hard-fail any unexplained prefix change. (L:157, L:164)
- **L-17.15 [E]** Return Reuse with unchanged prefix hash and a covering cache_read when boundaries are unchanged. (L:162)
- **L-17.16 [E]** Return and journal Recompute(ColdGapTtl) after TTL crossing. (L:162-163)
- **L-17.17 [E]** Never report a cache miss as a hit. (L:164)
- **L-17.18 [A]** Reuse I-10 CacheEconomicsEvent with a cause variant; optionally show the last epoch cause in the gauge. (L:158-161)

#### L-18 — projection corollaries: losslessness + time-travel collapse [CC-1/CC-4]

- **L-18.01 [E]** Run check_lossless_compaction against the projected view. (L:168)
- **L-18.02 [E]** Assert every K-12 critical atom by exact match. (L:168)
- **L-18.03 [E]** Fail closed when a view would omit a critical atom. (L:168-170)
- **L-18.04 [E]** Widen the projection budget after such failure because no raw data was dropped. (L:169-170, L:178)
- **L-18.05 [E]** Preserve K-12 R1 structurally through immutable history. (L:169-170)
- **L-18.06 [E]** Implement undo as recomputing the view at an earlier log index. (L:170-171)
- **L-18.07 [E]** Implement redo as recomputing the view at a log index. (L:170)
- **L-18.08 [E]** Implement branch as recomputing/forking at a log index. (L:170-171)
- **L-18.09 [E]** Walk undo-tree HEAD indices rather than storing diffs. (L:170-171)
- **L-18.10 [E]** Re-express recover_stubs as project-at-span with a larger budget. (L:171-173)
- **L-18.11 [E]** Keep the view-level atom gate auditor-owned and frozen. (L:174-175)
- **L-18.12 [E]** Preserve every must_preserve atom verbatim in every later view. (L:176)
- **L-18.13 [E]** Recompute an undo target byte-identically. (L:177)
- **L-18.14 [E]** Fork a branch without copying transcript bytes by using CAS. (L:177)
- **L-18.15 [E]** Reuse HandoffFaithfulnessGated/EditHistoryEvent with net-zero event growth. (L:174-175)

#### L-19 — ambient page-in budget ladder [CC-2]

- **L-19.01 [E]** Give a healthy window band the full page-in budget. (L:182)
- **L-19.02 [E]** Halve page-in budget in the warning band. (L:182)
- **L-19.03 [E]** Quarter page-in budget in the critical band. (L:182-183, L:192)
- **L-19.04 [E]** Reduce overflow recall to hints only. (L:183)
- **L-19.05 [E]** Treat a tool re-touch as tier-0 recall signal. (L:183)
- **L-19.06 [E]** Treat a claim on a folded path as tier-1 recall signal. (L:183-184)
- **L-19.07 [E]** Treat a critical atom resurfacing as tier-2 recall signal. (L:184)
- **L-19.08 [E]** Inject matching recall content as append-only tail cards. (L:184-185)
- **L-19.09 [E]** Never rewrite the frozen prefix during recall. (L:184, L:191-193)
- **L-19.10 [E]** Track recall-card residency TTL in passes. (L:185)
- **L-19.11 [E]** Cyclically refold expired cards at the next epoch. (L:185, L:192)
- **L-19.12 [E]** Encode failed-path recall as I-5 contrastive constraints rather than raw pasted traceback. (L:185-188)
- **L-19.13 [E]** Do not page in a low-relevance span. (L:193)
- **L-19.14 [E]** Re-touching a folded file must page it into the tail while leaving the prefix hash unchanged. (L:191-192)
- **L-19.15 [A]** Reuse ContextReduction and optionally show “recalled N (TTL k)” in the gauge. (L:189-190)

### L-T4 — Measured Competitor / Eval Supply

#### L-20 — commit-mined eval task supply + competitor-arm paired harness

- **L-20.01 [E]** Add elai eval mine-commits over a repository’s real commit history. (L:216, L:218)
- **L-20.02 [E]** Deterministically exclude merge commits. (L:216)
- **L-20.03 [E]** Deterministically exclude dependency-only commits. (L:216)
- **L-20.04 [E]** Deterministically exclude documentation-only commits. (L:216, L:219)
- **L-20.05 [E]** Deterministically exclude formatting-only commits. (L:216)
- **L-20.06 [E]** Type accepted tasks with failure_taxonomy. (L:216-217)
- **L-20.07 [E]** Offer an optional advisory-only hard-commit screen. (L:216)
- **L-20.08 [E]** Emit J-56 corpus rows containing task_prompt, base_sha workspace, and blake3 reference_diff. (L:216, L:219)
- **L-20.09 [E]** Gain contamination resistance by mining private repositories. (L:216)
- **L-20.10 [E]** Apply E-54’s 13-gram contamination filter to public rows. (L:216)
- **L-20.11 [E]** Add CompetitorArm kinds for ELAI, claude_code, codex_cli, and opencode. (L:216)
- **L-20.12 [E]** Reuse cli_common.rs:330 and its hand-parsed stream-json bridge; never create another process spawner. (L:216-217)
- **L-20.13 [E]** Run all arms on the same task rows and under an identical journaled envelope. (L:216)
- **L-20.14 [E]** Capture per-arm cost through E-49. (L:216)
- **L-20.15 [E]** Deterministically gate build results. (L:216)
- **L-20.16 [E]** Deterministically gate tests. (L:216)
- **L-20.17 [E]** Use diff-overlap as advisory-weighted evidence. (L:216)
- **L-20.18 [E]** Keep any LLM judge advisory-only. (L:216)
- **L-20.19 [E]** Require at least five paired seeds through I-38. (L:216)
- **L-20.20 [E]** Report an internal paired delta under a reproducible envelope, never a leaderboard rank. (L:216)
- **L-20.21 [E]** Gate any public comparative claim through KD-1. (L:216)
- **L-20.22 [E]** Emit observe-only bench-lesson rows for J-10, I-12, and J-96 while preserving H-31/K-19 firewalls. (L:216)
- **L-20.23 [E]** Add elai bench competitor and the commit_miner/competitor_arm_runner integration. (L:218)
- **L-20.24 [E]** Emit per-arm SuiteRun and cost rows. (L:219)
- **L-20.25 [E]** Fail a named arm when its vendor CLI is absent; never silently skip it. (L:219)
- **L-20.26 [E]** Fail a contract test if a leaderboard-rank string appears in the report. (L:219)
- **L-20.27 [A]** Live ELAI anchors are plan-asserted and must be re-grepped at edit time under W-7; the paired-arm frontend remains an edit-time choice. (L:215, L:218)

#### L-21 — weak-lane verification-intensity policy

- **L-21.01 [E]** Make a model-matched reviewer mandatory for free/weak model lanes. (L:223)
- **L-21.02 [E]** Fan out reviewer, typecheck, and tests in parallel during the weak model’s turn. (L:223)
- **L-21.03 [E]** Keep review optional on strong/frontier lanes. (L:223, L:226)
- **L-21.04 [E]** Resolve verification intensity from ModelCapabilityProfile tier × role. (L:223)
- **L-21.05 [E]** Store mini_critic_passes in the policy row. (L:223)
- **L-21.06 [E]** Store structural_retry_budget in the policy row. (L:223)
- **L-21.07 [E]** Store mid-turn scoped compile/test fan-out in the policy row. (L:223)
- **L-21.08 [E]** Store reviewer_dispatch as mandatory or optional. (L:223)
- **L-21.09 [E]** Store fanout_cap in the policy row. (L:223)
- **L-21.10 [E]** Keep the table as DATA beside I-51’s catalog and resolve it purely. (L:223)
- **L-21.11 [E]** Parameterize the C-32 critique/retry loop. (L:223-224)
- **L-21.12 [E]** Parameterize task_runner mid-turn checks. (L:223)
- **L-21.13 [E]** Parameterize B-41 reviewer marking. (L:223-224)
- **L-21.14 [E]** Keep reviewer output advisory to the executor; never transfer auditor authority. (L:223)
- **L-21.15 [E]** Fail closed to the highest verification intensity for unknown profiles. (L:223, L:226)
- **L-21.16 [E]** Validate that policy rows can only add verification and reject any weak-lane floor reduction at load time. (L:223, L:226)
- **L-21.17 [E]** Roll out observe-first, then retain ELAI_VERIFICATION_INTENSITY_DISABLE as a kill switch. (L:223)
- **L-21.18 [E]** Make DISABLE=1 byte-identical to current behavior. (L:226)
- **L-21.19 [E]** Measure weak-lane verified-outcome rate and tokens-per-correct with the table ON versus OFF. (L:223)
- **L-21.20 [E]** Journal at least two critic passes, scoped-check fan-out, and reviewer=mandatory on a Qwen-7B lane. (L:226)
- **L-21.21 [E]** Make the resolved policy recomputable from profile plus table. (L:226)
- **L-21.22 [E]** Reuse existing routing and critic systems rather than forking either. (L:224)
- **L-21.23 [A]** Live anchors require W-7 re-grep; the RuntimePanel intensity badge is optional. (L:222, L:225)

### L-T5 — DeepReason Steal / Exploratory Reasoning

#### L-22 — claimant-authored falsifiers + demarcation + Verbalized Sampling

- **L-22.01 [E]** Require the generator to emit a machine-checkable falsifier alongside each design claim, hypothesis, or diagnosis. (L:242)
- **L-22.02 [E]** Express the refutation surface as forbidden conditions. (L:242)
- **L-22.03 [E]** Compile a falsifier into a runnable predicate at registration. (L:242)
- **L-22.04 [E]** Execute the predicate in D-11’s checker sandbox at zero LLM cost. (L:242-243)
- **L-22.05 [E]** Add an optional falsifier field to the L-1 ClaimSet. (L:242-243)
- **L-22.06 [E]** Elicit the field during generation under N-13/E-35 grammar masking. (L:242)
- **L-22.07 [E]** Avoid adherence-retry cost on sub-50B/weak lanes by making the skeleton valid by construction. (L:242)
- **L-22.08 [E]** Give a Causal or Behavior claim that forbids nothing a High ClaimRisk floor. (L:242, L:245)
- **L-22.09 [E]** Refuse Converge for a hypothesis-class GRC-v3 step with no falsifier. (L:242, L:245)
- **L-22.10 [E]** Add a propose_distribution contract returning K candidates in one distribution call. (L:242, L:245)
- **L-22.11 [E]** Attach a typicality tag to each proposed candidate. (L:242, L:245)
- **L-22.12 [E]** Use Verbalized Sampling in place of I-4’s N independent resample calls for this propose leg. (L:242-243)
- **L-22.13 [E]** Tail-weight candidates only while the anti-spiral orbit flag is active. (L:242, L:245)
- **L-22.14 [E]** Keep typicality advisory and outside deterministic authority. (L:242)
- **L-22.15 [E]** Permit a causal claim with a compiled falsifier to score at its evidenced tier. (L:245)
- **L-22.16 [E]** Fail a contract if a forbids-nothing causal/behavior claim scores below High. (L:245)
- **L-22.17 [E]** Fail a contract if Rubric-only evidence sets causal_supported=true. (L:245)
- **L-22.18 [E]** Forbid a second free-form model call that reconstructs a missing falsifier after generation. (L:245)
- **L-22.19 [E]** Reuse L-1/L-2, checker, grammar-mask, GRC, and anti-spiral machinery without reimplementation. (L:243)
- **L-22.20 [A]** Live anchors remain plan-asserted and require W-7 re-grep; the falsifier chip on ClaimRisk is optional. (L:241, L:244)

#### L-23 — /reason --profile explore [KEYSTONE; new surface]

- **L-23.01 [E]** Add /reason --profile explore as a profile on the existing reason engine. (L:247-251)
- **L-23.02 [E]** Accept hard open explanatory questions. (L:249)
- **L-23.03 [E]** Return a map of rival explanations rather than one answer. (L:249)
- **L-23.04 [E]** Include each rival’s refutation status. (L:249, L:252)
- **L-23.05 [E]** Include the evidence gaps that would decide among survivors. (L:249, L:252)
- **L-23.06 [E]** Spread conjectures through L-22 Verbalized Sampling. (L:249)
- **L-23.07 [E]** Rotate stance through anti-spiral personas. (L:249)
- **L-23.08 [E]** Run claimant-authored falsifiers without an LLM criticism call. (L:249)
- **L-23.09 [E]** Add a fifth StagnationPattern value, Orbiting. (L:249, L:251)
- **L-23.10 [E]** Detect orbiting from the S2 gate block rate with zero embeddings. (L:249)
- **L-23.11 [E]** Rotate persona and name the attractor after a refuted-equivalent proposal reaches the threshold. (L:252)
- **L-23.12 [E]** Use I-26/I-8 problem turnover when exploration cannot progress. (L:249-250)
- **L-23.13 [E]** Judge surviving explanations cross-family through D-13/J-45. (L:249-250)
- **L-23.14 [E]** Apply I-24/N-15 order swapping and verbosity penalty to survivor judgment. (L:249-250)
- **L-23.15 [E]** Keep negative case law/tabu at the gate and never render it into the conjecture prompt. (L:249, L:252)
- **L-23.16 [E]** Do not adopt complement directives; their measured effect was placebo. (L:249)
- **L-23.17 [E]** Provide a local-7B conjecturer arm. (L:249)
- **L-23.18 [E]** Make the local skeleton grammar-valid by construction through E-35/N-13. (L:249-250)
- **L-23.19 [E]** Reuse an H-2/I-10 frozen prefix and J-36 KV cache. (L:249-250)
- **L-23.20 [E]** Use the local arm as an integration smoke test across mask, frozen-prefix, and KV-reuse systems. (L:249)
- **L-23.21 [E]** Ship a fixture arm now and sidecar-gate the live 7B arm. (L:252)
- **L-23.22 [E]** Replay the fixture byte-identically under ELAI_SEED with an RRA. (L:252)
- **L-23.23 [E]** Include every refuted rival with its killing falsifier in the survivor map. (L:252)
- **L-23.24 [E]** Fail if exploration collapses to one answer. (L:252)
- **L-23.25 [E]** Fail if an embedding-based orbit detector appears. (L:252)
- **L-23.26 [E]** Reject Dung/argument-graph adjudication and fail if such a symbol appears. (L:250, L:252)
- **L-23.27 [E]** Render the survivor map through K-11/N-7. (L:250-251)
- **L-23.28 [A]** Live anchors remain plan-asserted and must be re-grepped under W-7. (L:248)

#### L-24 — conjecture-memory middle tier + collaboration riders

- **L-24.01 [E]** Add a memory tier between the append-only journal and J-96..J-98 Verified Case Graph. (L:256)
- **L-24.02 [E]** Preserve unproven conjectures as first-class retrievable objects. (L:256)
- **L-24.03 [E]** Compute status as a pure journal fold rather than hand-curating it. (L:256)
- **L-24.04 [E]** Support live, refuted, pending, and suspended statuses. (L:256)
- **L-24.05 [E]** Suspend dependents when one of their premises is refuted. (L:256, L:259)
- **L-24.06 [E]** Do not treat an orphaned dependent as false. (L:256)
- **L-24.07 [E]** Make no lifecycle status absorbing. (L:256)
- **L-24.08 [E]** Quarantine contradicted nodes rather than deleting them. (L:256)
- **L-24.09 [E]** Preserve provenance on conjectures. (L:256)
- **L-24.10 [E]** Attach criticisms to the evaluator id and evaluator version hash defined by N-16. (L:256-257)
- **L-24.11 [E]** Collapse the support of claims vouched for by an evaluator that is later refuted. (L:256)
- **L-24.12 [E]** Store declared missing tests as the exact unrun L-22 falsifiers that would settle a conjecture. (L:256)
- **L-24.13 [E]** Restrict unproven-conjecture retrieval to the explore lane through a type-level lane token. (L:256)
- **L-24.14 [E]** Journal every retrieval with a Retrieved audit row. (L:256, L:259)
- **L-24.15 [E]** Add a negative contract test that makes ConjectureNode structurally unretrievable into code-fix or auditor paths. (L:256, L:259)
- **L-24.16 [E]** Add an explore-arm novelty column to L-10. (L:256)
- **L-24.17 [E]** Compute novelty with L-4/F-12 NLI meaning clusters, never cosine over hashes. (L:256)
- **L-24.18 [E]** Measure novelty in a 2×2 CGG ON/OFF × retrieval ON/OFF ablation. (L:256)
- **L-24.19 [E]** Add typed refusal unblock conditions with tried, missing, and route fields. (L:256)
- **L-24.20 [E]** Derive unblock conditions deterministically from the GRC receipt trail, never model prose. (L:256)
- **L-24.21 [E]** Treat a D-17/L-7 refusal without an unblock condition as an SLA violation. (L:256, L:259)
- **L-24.22 [E]** Add an FD-1..FD-3 diversity pre-gate to the I-30/I-31 training flywheel. (L:256)
- **L-24.23 [E]** Train on reasoning moves rather than only surviving conjectures. (L:256)
- **L-24.24 [E]** Filter labels through the N-16 S6 lambda/exogenous-grounding floor. (L:256)
- **L-24.25 [E]** Put the evaluator-version edge on every dataset row. (L:256)
- **L-24.26 [E]** Land flywheel code only with I-30 after D-026 unlocks; do not create wire-dark machinery before a consumer exists. (L:256)
- **L-24.27 [E]** Leave J-96..J-98 unchanged as a locked proven-only boundary and edit none of its files. (L:256-257)
- **L-24.28 [E]** Allow the conjecture store to ship independently of the L-23 loop. (L:255)
- **L-24.29 [E]** Emit lifecycle rows Registered, Criticized, Refuted, Suspended, and Retrieved. (L:259)
- **L-24.30 [E]** Recompute the lifecycle/status map byte-identically under ELAI_SEED. (L:259)
- **L-24.31 [E]** In the two-round fixture, keep B live with missing tests while A becomes Suspended and is excluded after premise P is refuted. (L:259)
- **L-24.32 [E]** Fail if code-lane retrieval returns any ConjectureNode. (L:259)
- **L-24.33 [E]** Fail if a never-criticized node reports live. (L:259)
- **L-24.34 [E]** Fail if status is set by hand. (L:259)
- **L-24.35 [E]** Fail if a D-17 refusal lacks tried[] or missing[]. (L:259)
- **L-24.36 [E]** Add status to the L-23 survivor-map rendering. (L:258)
- **L-24.37 [A]** Add ConjectureLifecycle after re-reading the current ssb.rs baseline; live anchors remain plan-asserted and require W-7/W-9 checks. (L:255, L:258)

## Workflows and cross-feature interactions

### Claim extraction, evidence, effort, and action

1. A backend emits an answer/plan/patch/explanation. L-1 structurally extracts typed claims, retains unresolved references, and adds taint. (L:11-16)
2. L-3 establishes P1 by re-grounding claims; J-85 supplies P2; L-4 establishes P3 from meaning clusters and a cross-family subset. L-5 supplies stronger interventional evidence for causal/behavior claims, and L-22 adds the falsifier/demarcation signal. (L:18-44, L:240-245)
3. L-2 fuses independent evidence. Cheap signals gate expensive checks; one signal cannot exceed Medium; two distinct signals may reach High/Critical. (L:18-23)
4. L-6 spends targeted retrieval/test/trace/escalation effort only on High/Critical claims, then loops back to L-2. (L:46-51)
5. L-7 maps the final tier to allow, caveat/citation, forced verification, block, or abstain and journals the gate. (L:53-58)
6. L-8 removes later-refuted assumptions from live working belief without erasing evidence. A grounded closure may mint J-96 verified memory; unresolved exploratory conjectures instead remain in the separate L-24 store. (L:48, L:60-65, L:254-259)

### Generated code from idea to verified artifact

1. L-14 takes an idea through PIPELINE FACTORY and E-12. (L:104-108)
2. L-11 applies plan-time, generation-time, write-time, and audit-time quality seams. Weak lanes can receive stronger L-21 retry/check/reviewer settings. (L:83-88, L:221-226)
3. Failed quality reports findings and cannot become DONE; success is wrapped in an RRA receipt and may run under Tier-C or through MCP. (L:104-109)
4. The same user-created skills and rules enter through L-12/L-13 admission and safety gates; none can bypass immutable checks. (L:90-102)

### Immutable context projection and recall

1. L-16 appends all history to SessionDag/CAS and derives each model view through pure fold. (L:133-150)
2. L-17 reuses the frozen prefix unless one of seven named causes triggers a journaled epoch recompute. Tail growth alone does not move the prefix. (L:152-164)
3. L-18 verifies critical atoms in the derived view, widening the budget on failure, and implements undo/redo/branch/recovery as projection at another index or budget. (L:166-178)
4. L-19 appends recall cards to the tail under pressure-sensitive budgets and expires/refolds them by TTL. (L:180-193)

### Evaluation and claims about value

1. L-20 mines real commits into hashed, base-SHA-bound tasks, then runs ELAI and competitor CLIs under an identical envelope. (L:214-219)
2. L-21 varies verification effort by model capability/role while L-20/K-15 measures verified outcomes and tokens per correct. (L:221-226)
3. L-10 independently ablates the hallucination gate. L-24 extends its explore arm with a grounding/retrieval novelty matrix. (L:74-79, L:256)
4. Public comparative/value claims remain absent until the appropriate ELAI-measured paired deltas exist. No paper metric, live-cost estimate, or leaderboard language may substitute. (L:76, L:106, L:216, L:219)

### Exploratory reasoning and isolated conjecture memory

1. L-22 elicits candidate conjectures and their executable falsifiers in one grammar-constrained generation call. (L:240-245)
2. L-23 iterates rival conjectures, gate-side refutation memory, stance rotation, orbit detection, turnover, and cross-family survivor judgment. (L:247-252)
3. L-24 stores unresolved/refuted/suspended conjectures with computed lifecycle, missing tests, provenance, and evaluator edges. (L:254-259)
4. Only the explore lane may retrieve unproven conjectures. Verified Case Graph remains proven-only, and refuted content influences the gate rather than the generation prompt. (L:256-257)

## Constraints, invariants, and negative contracts

- **HG-1 causal means interventional [E].** Static, temporal, import-edge, or rubric evidence cannot set causal_supported; only an executable perturb-and-observe pass can. (L:41-44, L:245)
- **HG-2 convergence, not one signal [E].** High/Critical needs at least two mechanistically distinct predictors; one predictor caps at Medium. (L:20, L:23)
- **HG-3 black-box [E].** Claims are extracted outside the model and risk scoring must work without logprobs on any backend. (L:13, L:20, L:23)
- **HG-4 abstain or verify [E].** Missing/unverifiable High evidence never silently passes; unchecked math is abstained. (L:55-58, L:69-72)
- **HG-5 demarcation [E].** A Causal/Behavior claim with no falsifier has a High floor, and a hypothesis cannot converge without one. (L:242-245)
- **GD-1 compose, do not restate [E].** L repeatedly forbids duplicated grounding, mutation, runners, routing, critic, hook, skill, and case-graph machinery. (L:14, L:28, L:42, L:77, L:93, L:100, L:107, L:217, L:224, L:243, L:250, L:257)
- **GD-2 reproducibility [E].** Risk, gates, context views, lifecycle, fixture outputs, and measurements derive from journaled data and ELAI_SEED. (L:23, L:55, L:65, L:76, L:88, L:95, L:102, L:147-150, L:203-204, L:226, L:252, L:259)
- **GD-3 receipts, not proxy prestige [E].** Measurement axes use receipts/verified outcomes rather than pass@k or leaderboard rank. (L:76, L:216, L:219)
- **GD-4 observe first [E].** Hallucination policy, quality rules, and verification intensity must bound error/over-block behavior before active blocking. (L:55, L:85, L:99, L:115, L:223)
- **GD-5 honest residue [E].** The claim gate catches coverage-gap hallucination, not all confidently shared wrong beliefs; cross-family evidence is mitigation, not elimination. (L:7, L:34, L:115)
- **TD-3 deterministic authority [E].** LLM judgments, typicality, hard-commit screening, and reviewers remain advisory where stated; pure tables/checkers/gates own authoritative verdicts. (L:34-35, L:55, L:69, L:85, L:92, L:99, L:216, L:223, L:242)
- **R8 separation/monotonicity [E].** The claimant/executor cannot self-grade; immutable checks cannot be weakened; policy rows and user rules may only add verification. (L:13, L:55, L:85, L:99, L:223)
- **CC-1 immutable raw history [E].** Raw transcript bytes are append-only and attempted mutation fails. (L:123-129, L:135-150)
- **CC-2 pure view/prefix stability [E].** Identical fold inputs produce byte-identical views; recall and tail growth do not rewrite the frozen prefix. (L:127-129, L:147-150, L:152-164, L:180-193)
- **CC-3 named epoch causes [E].** Every prefix change must have one enumerated, journaled cause; unexplained change hard-fails. (L:154-164)
- **CC-4 structural losslessness [E].** Critical atoms exact-match in all later views; too-tight budgets widen instead of losing history. (L:168-178)
- **EX-1 negative case stays gate-side [E].** Refuted/tabu ideas never appear in the conjecture prompt. (L:249, L:252, L:257)
- **EX-2 no embedding orbit detector [E].** Orbiting is measured by gate-block rate, not vector similarity. (L:249, L:252)
- **EX-3 preserve rival explanations [E].** A single collapsed answer fails the explore contract. (L:252)
- **EX-5 lane isolation [E].** Any unproven ConjectureNode returned to code-fix/auditor retrieval fails. (L:256, L:259)
- **EX-6 computed lifecycle [E].** Hand-set status fails; never-criticized cannot be live. (L:256, L:259)
- **No deletion of contradicted knowledge [E].** Working beliefs and conjectures are quarantined, never erased. (L:62-65, L:256)
- **No fabricated measurements [E].** Live metrics refuse to emit before their live arm runs; external single-seed/paper numbers are rationale only. (L:76, L:79, L:262)

## Dependencies, sequencing, status, and deferred work

| ID | Track / wave | Explicit dependency | Nature and status stated by summary |
|---|---|---|---|
| L-1 | T1 / W1 | none | BUILD; strong grounding; foundation |
| L-2 | T1 / W1 | L-1 | BUILD; KEYSTONE; G-7/D-3 shipped but DARK |
| L-3 | T1 / W2 | L-1 | BUILD-on-shipped; D-10/H-53 shipped |
| L-4 | T1 / W2 | L-1, L-2 | BUILD; meaning cluster net-new, I-4 shipped |
| L-5 | T1 / W2 | L-1 | EXTENSION; KEYSTONE; F-4/C-28/C-29 shipped |
| L-6 | T1 / W3 | L-2 | BUILD over J-38/I-7 substrate |
| L-7 | T1 / W3 | L-2, L-6 | BUILD-compose; D-16/AutonomousGate shipped; observe-first |
| L-8 | T1 / W3 | L-1 | BUILD-compose; F-5/I-5 shipped, J-21 planned |
| L-9 | T1 / W4 | L-2, L-3, L-4 | EXTENSION/profile; B-39/D-11/B-42 shipped |
| L-10 | T1 / W4 | measures L-1..L-7 | BUILD-instrument; fixture now, live arm BENCH-PENDING/RunPod-gated |
| L-11 | T2 / W1 | none | BUILD+COMPOSE; KEYSTONE; some checks shipped, J-65/J-23 planned |
| L-12 | T2 / W2 | none | ACTIVATION+COMPOSE; E-43 DARK, F-6 wired, I-12 planned |
| L-13 | T2 / W2 | none | ACTIVATION+COMPOSE; rule/hooks wired, J-52 planned |
| L-14 | T2 / W3 | L-11 | ACTIVATION+COMPOSE; fixture now, live build gated |
| L-15 | title only | unknown | [A] Receipt-minting repair controller; all other summary fields absent |
| L-16 | T3 / W1 | none | BUILD KEYSTONE plus compact_session revision; default-on with one-release rollback |
| L-17 | T3 / W2 | L-16 | BUILD |
| L-18 | T3 / W2 | L-16 | BUILD-compose; auditor-owned/frozen |
| L-19 | T3 / W3 | L-16, L-17 | BUILD |
| L-20 | T4 / W1 | existing runner/corpus/CLI bridges | EXTENSION+BUILD-instrument; live anchors plan-asserted/re-grep |
| L-21 | T4 / W2 | J-51/J-55/C-32/H-7/B-41 substrate | EXTENSION; observe-first; live anchors plan-asserted/re-grep |
| L-22 | T5 / W1 | L-1, L-2 | BUILD-on-shipped+EXTENSION; live anchors plan-asserted/re-grep |
| L-23 | T5 / W2 | L-22 | NEW surface, about 90% composition; fixture now/live-7B gated |
| L-24 | T5 / W2 | L-22, L-23; store loop-independent | BUILD+three extension legs; flywheel rider waits for D-026/I-30 |

Additional status and sequencing facts:

- **[E]** The source describes L as approximately 85% activation/unification and 15% net-new in T1; T3 similarly describes about 15% projection/state-machine novelty and 85% re-expression; T5 is about 90% composition. (L:7, L:129-131, L:233-235)
- **[A]** The summary gives two inconsistent lists of the four “genuine/net-new builds”: the premise names L-2, L-4, L-5, L-6, while the T1 cross-cutting section names L-1, L-2, L-4, L-5. Original plans/grounding map must resolve whether L-1 or L-6 is counted as the fourth. (L:7, L:115)
- **[E]** T1 is not done until L-10 produces a positive, recomputable ELAI delta. (L:76, L:115)
- **[E]** T3 is not done until a real run shows ELAI-measured byte identity plus cache_read; CWD’s published number is not an acceptance metric. (L:203-204)
- **[E]** T5’s flagship acceptance demo intentionally uses the local lane; external DeepReason figures are single-seed design rationale only. (L:261-262)
- **[E]** Rejected from T5: Dung argument-graph adjudication, untyped-artifact ontology, and embedding convergence. Schools/lineage is deferred. (L:237-238, L:250-252)
- **[E]** Planning-archive FDL controller, ProbeUtility scheduler, multi-cycle operator allocator, and archive-to-runtime bridge are outside the specification. (O:393-399)
- **[E]** Q-10 research coupling is only proposed, one-way, SOURCE_ONLY, and must beat source-blind/matched-scaffold/verified-evidence/relation-shuffled controls before reaching planning skills or later the explicit explore lane. (O:401-408)

## UI, CLI/API, configuration, events, and operations index

### CLI/API surfaces

- elai bench hallucination-delta (L-10); /reason --profile research (L-9); elai skill add/improve (L-12); elai rule add/list/test/promote/retire (L-13); elai build "<idea>", elai artifact run, elai rra verify, and the J-2 MCP build verb (L-14); elai eval mine-commits and elai bench competitor (L-20); /reason --profile explore (L-23). (L:69-79, L:90-109, L:214-219, L:247-252)

### Frontend surfaces

- Explicit or edit-time candidates are the claims tile, ClaimRisk tile, retrieval/escalation surface, refusal/mode panels, research profile badge, hallucination-delta tile, GatesPanel/vibesec quality tile, SkillsPanel actions, RulesDrawer/ToolsHooksDrawer authoring, ProjectSelector/Intro one-field builder, context epoch/recall gauge, paired-arm tile, RuntimePanel intensity badge, falsifier chip, and survivor-map status dimension. Many are explicitly “decide at edit” or permit justified N/A and must not be presented as committed UI. (L:15, L:22, L:36, L:43, L:50, L:57, L:64, L:71, L:78, L:87, L:94, L:100-101, L:108, L:161, L:190, L:218, L:225, L:244, L:251, L:258)

### Configuration and policy data

- ELAI_HALLUCINATION_GATE_DISABLE keeps observation/journaling but disables blocking. (L:57-58)
- ELAI_CONTEXT_PROJECTION_DISABLE selects the legacy mutating path during a one-release rollback window. (L:145-146)
- ELAI_VERIFICATION_INTENSITY_DISABLE restores byte-identical current behavior. (L:223, L:226)
- ELAI_SEED binds deterministic fixtures, quality, benches, and lifecycle replay. (L:35, L:76, L:88, L:109, L:252, L:259)
- User rules are lifecycle-governed DATA; verification intensity is a TOML policy table; skills use SKILL.md plus trust-tier/admission policy. (L:92-102, L:223-225)

### Journal/event candidates

- ClaimSetExtracted, ClaimRiskScored with LowRisk/MediumRisk/HighRisk/CriticalRisk, SemanticEntropyMeasured with SeedsAgree/SeedsDiverge/CrossFamilyDiverge, CounterfactualProbed, HallucinationGateAction, OutputQualityVerdict, SkillPromoted, UserRuleRegistered, ContextProjected, CacheEconomicsEvent cause variant, HallucinationDeltaArm, and ConjectureLifecycle are explicit candidates, not all mandatory new rows. The plan repeatedly prefers reuse and targets net 0 or net 0..+1 schema growth per feature. (L:15, L:22, L:36, L:43, L:57, L:87, L:94, L:101, L:144-145, L:160, L:175, L:218-219, L:258-259)

## Deeper-source manifest

### Mandatory L originals not inspected in this pass

All paths exist. Line counts are supplied only to help allocate deep-reading work.

| Plan | Path | Lines | Why inspect |
|---|---|---:|---|
| L doctrine/index | docs/thefinalglance/L-plan/L-0_INDEX.md | 273 | Canonical constants, sequencing, doctrine HG/CC/OQ/FS/EX, K-10 override |
| L-1 | docs/thefinalglance/L-plan/L-1_typed_claim_extraction_substrate.md | 102 | Extraction grammar, Claim schema, tests |
| L-2 | docs/thefinalglance/L-plan/L-2_claimrisk_score_convergence_fuse.md | 106 | Risk math, thresholds, predictor contracts |
| L-3 | docs/thefinalglance/L-plan/L-3_evidence_round_trip_predictor.md | 100 | Residual calculation and grounding-mode detail |
| L-4 | docs/thefinalglance/L-plan/L-4_semantic_entropy_inter_seed_predictor.md | 105 | NLI clustering and entropy details |
| L-5 | docs/thefinalglance/L-plan/L-5_counterfactual_do_test_on_model_claims.md | 103 | Intervention sandbox and causal tiers |
| L-6 | docs/thefinalglance/L-plan/L-6_risk_as_curiosity_targeted_effort_router.md | 103 | Router selection/budget policy |
| L-7 | docs/thefinalglance/L-plan/L-7_hallucination_policy_gate_abstention.md | 104 | Tier-to-action thresholds and rollout |
| L-8 | docs/thefinalglance/L-plan/L-8_working_memory_hygiene_prune_spurious.md | 98 | Quarantine lifecycle and revalidation |
| L-9 | docs/thefinalglance/L-plan/L-9_research_math_deepthink_grounding_profile.md | 99 | Research profile/checker routing |
| L-10 | docs/thefinalglance/L-plan/L-10_cgg_hallucination_delta_bench.md | 105 | Corpus, metrics, fixture/live contracts |
| L-11 | docs/thefinalglance/L-plan/L-11_output_quality_enforcement_system.md | 121 | Proxy thresholds and four-seam wiring |
| L-12 | docs/thefinalglance/L-plan/L-12_skill_front_door_self_improving_skills.md | 107 | Import, retrieval, curator detail |
| L-13 | docs/thefinalglance/L-plan/L-13_user_authorable_rules_hooks_surface.md | 108 | Predicate schema and lifecycle operations |
| L-14 | docs/thefinalglance/L-plan/L-14_one_shot_verified_builder_front_door.md | 110 | Builder pipeline/defaults/receipt flow |
| **L-15** | **docs/thefinalglance/L-plan/L-15_repair_controller_and_evidence_first_reorder.md** | **201** | **Mandatory: wholly absent from the summary body** |
| L-16 | docs/thefinalglance/L-plan/L-16_immutable_transcript_pure_fold_view_core.md | 137 | Projection types and migration |
| L-17 | docs/thefinalglance/L-plan/L-17_freeze_epoch_controller_prefix_invariance.md | 49 | Cause-state-machine details |
| L-18 | docs/thefinalglance/L-plan/L-18_projection_corollaries_lossless_timetravel.md | 48 | View gate/time-travel contracts |
| L-19 | docs/thefinalglance/L-plan/L-19_ambient_pagein_budget_ladder.md | 48 | Recall relevance/TTL/budgets |
| L-20 | docs/thefinalglance/L-plan/L-20_commit_mined_eval_supply_competitor_arm.md | 57 | Commit filter, harness envelope, arm failure |
| L-21 | docs/thefinalglance/L-plan/L-21_weak_lane_verification_intensity.md | 55 | Policy table floors and consumers |
| L-22 | docs/thefinalglance/L-plan/L-22_claimant_authored_falsifiers_demarcation.md | 121 | Falsifier grammar, demarcation, VS |
| L-23 | docs/thefinalglance/L-plan/L-23_reason_explore_profile_conjecture_criticism.md | 128 | Explore loop/orbit/map tests |
| L-24 | docs/thefinalglance/L-plan/L-24_conjecture_memory_middle_tier.md | 199 | Store lifecycle, isolation, riders |

### Grounding and correction sources

- docs/thefinalglance/L-plan/_grounding/L-CGG-GROUNDING-MAP.md — 140 lines; corrections/live-anchor map for L-T1. Named at L:2.
- docs/thefinalglance/L-plan/_grounding/L-OQF-GROUNDING-MAP.md — 63 lines; corrections/live-anchor map for L-T2. Named at L:2.
- **[A]** No corresponding summary-named grounding-map path is supplied for L-T3, L-T4, or L-T5. Their entries say CWD/codebuff/DeepReason sources were read and ELAI anchors are live or plan-asserted; locate any session-specific grounding artifacts during deep inspection. (L:134, L:153, L:167, L:181, L:215, L:222, L:241, L:248, L:255)

### Referenced local orientation, provenance, and grounding candidates

- docs/thefinalglance/ELAI-SPEC-OVERVIEW.md — orientation source; relevant L boundary at lines 367-408.
- docs/thefinalglance/ELAI-FEATURE-MAP.md — live-anchor source explicitly named at L:2.
- workflow/reasoning/steal-codebuff-2026-07-03.md — L-T4 provenance, explicitly named at L:208.
- workflow/reasoning/TEIKOKU-CODEBUFF-STEAL-PLAN-2026-07-03.md — nearby codebuff planning artifact found locally; not named verbatim by the summary, so [A] candidate.
- workflow/reasoning/steal-deepreason-2026-07-08.md — L-T5 provenance, explicitly named at L:230.
- workflow/handoff/deepreason-steal-and-collab.md — likely source for the “live collab exchange/handoff CURRENT STATUS” named at L:255; path association is [I] from the filename and must be verified.
- docs/specs/guide/steps/phase-2D.txt — likely CSQ phase-2D source cited at L:42; exact association is [I].
- .claude/agents/paper-ingestor.md and .codex/agents/paper-ingestor.toml — local paper-ingestor definitions referenced by name at L:69-70; inspect the runtime-relevant variant.

### External or location-unspecified sources to locate

- arXiv:2606.27326, cited as the T1 hallucination/coverage paper. (L:7)
- arXiv:2510.01171, cited for Verbalized Sampling and described as WebSearch-verified. (L:241-242)
- context-warp-drive-main, including foldFreeze.ts and foldRecall.ts, cited as L-T3 external prior art. No local path is given in the summary. (L:129, L:153, L:181, L:197-204)
- The codebuff repository, including base2.ts and its commit/CLI harness sources. No checkout path is given in the summary. (L:208-223)
- github.com/AHepi/DeepReason, harness-spec-v1.3, CAN_LLMS_EXPLORE, and MiniReason sources. No checkout path is given in the summary. (L:230-238)
- The paper notes/source containing LS-2, WV-2, SAC3, and paper sections 4.1.ii/4.2/4.3. The summary gives identifiers but no local path. (L:7, L:13, L:20, L:34, L:41, L:48)
- The “live collab exchange” underlying L-24. A likely local handoff file is listed above, but the summary does not state its exact path. (L:236-237, L:255)

### Cross-plan dependencies requiring original-plan trace

- B-22, B-29, B-37, B-39, B-41, B-42; C-10, C-28, C-29, C-31, C-32; D-3, D-7, D-10, D-11, D-13, D-16, D-17, D-27; E-5, E-12, E-25, E-31, E-35, E-43, E-49, E-53, E-54; F-1, F-4, F-5, F-6, F-12; G-1, G-2, G-6, G-7; H-2, H-6, H-7, H-31, H-32, H-39, H-40, H-49, H-50, H-53; I-4, I-5, I-7, I-8, I-10, I-11, I-12, I-20, I-24, I-26, I-27, I-30, I-31, I-38, I-40, I-51; J-2, J-10, J-16, J-21, J-23, J-27, J-33, J-36, J-38, J-45, J-51, J-52, J-53, J-55, J-56, J-63, J-65, J-80, J-83, J-84, J-85, J-89, J-96..J-98, J-102, J-103, J-107; K-10, K-11, K-12, K-15, K-18, K-19; N-7, N-13, N-15, N-16. These identifiers are explicit dependencies/compositions, but their full behavior is outside this summary and must come from their own discovery/original files.

## Ambiguities that must survive compilation

1. **L-15 absence [A].** The title promises a receipt-minting repair controller, while the summary body jumps from L-14 to L-T3/L-16. Do not infer repair stages from the filename; deep-read the 201-line original.
2. **Net-new four-item discrepancy [A].** L:7 counts L-2/L-4/L-5/L-6; L:115 counts L-1/L-2/L-4/L-5.
3. **Schema/UI choices [A].** Numerous “OR reuse,” “decide at edit,” and “or N/A” statements mean events and panels are candidates, not guaranteed features.
4. **Plan-asserted live anchors [A].** L-20 through L-24 repeatedly require W-7 re-grep before implementation; the summary is not code-grounding proof.
5. **Counts and thresholds [A].** N, k, risk thresholds, orbit floor, TTL values, pressure boundaries, J-84 budgets, and many proxy cutoffs are symbolic or example-only here. Exact constants belong to originals/L-0.
6. **Example versus normative threshold [A].** N=5 and the 1200-line/25-method god-object are observable examples; only original plans can establish whether they are fixed defaults or fixtures. (L:37, L:88)
7. **L-24 rider lettering [A].** The summary lists riders A, B, and D while calling them “three collab riders”; there is no rider C in the summary. Preserve the letters and inspect the original.
8. **External source paths [A].** CWD, codebuff, and DeepReason source checkouts are not located by the summary.

## Summary-level completeness conclusion

Within L-SUMMARY_INDEX.md, all detailed entries, their build/compose/integration/observable/negative clauses, cross-track premises, and stated exclusions are represented above. The only named L session not discoverable at this layer is L-15. Therefore a super-feature compilation may safely consume this file for L-1..L-14 and L-16..L-24, but must hold L-15 open until its original plan is independently inspected.

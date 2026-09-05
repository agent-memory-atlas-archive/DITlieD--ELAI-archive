STEAL ANALYSIS — AgenticSTS / ProgramAsWeights / agentic-abstention
Date: 2026-07-04. Mode: spec-eval AS-SPECIFIED per user instruction ("taking the old plans as done"):
every B..M plan treated as BUILT; shipped-vs-planned status never feeds a verdict below.
Sources (all file:line-grounded by 3 opus explorers, clones read, papers WebFetched):
  workflow/papers/agenticsts-summary.md          (github.com/AlayaLab/AgenticSTS, arXiv:2607.02255, Apache-2.0 agent / AGPL mod / CC-BY data)
  workflow/papers/programasweights-summary.md    (github.com/programasweights/programasweights-python, arXiv:2607.02512, MIT)
  workflow/papers/agentic-abstention-summary.md  (github.com/lhannnn/agentic-abstention, arXiv:2606.28733, Apache-2.0, Meta AbstentionBench derivative)
ELAI context: ELAI-FEATURE-MAP.md (line refs below) + ELAI-SYSTEMS-OVERVIEW.md. Section-4 rejected
ledger checked per candidate before any NEW/EXTENSION verdict.

================================================================================
META-VERDICT
================================================================================
~70% of the three repos' portable surface is CONFIRMATION of designs ELAI already specifies
(bounded slot-budgeted context, first-class abstention, evidence-gated playbooks, circuit-breaker
multi-backend routing, cheap-first gate layering). The two genuine holes they expose:
  H1. ELAI's learned-artifact ADMISSION seams (memory facts, playbook bullets, cases, skills) have
      dedup/poisoning/fitness legs but NO overlap-check against the IMMUTABLE instruction surface
      and NO counterfactual-replay fitness AT ADMISSION TIME (H-43 proves fitness on frozen tests
      for DGM candidates; nothing replays the ORIGINATING failure with the candidate injected
      before the write is admitted).
  H2. ELAI scores WHETHER an abstention was correct (H-39, +1/0/-t/(1-t)) but nowhere scores WHEN
      it happened. A run that grinds 9 steps of budget before abstaining scores identically to one
      that stops at step 1. No abstention-warranted task supply exists to measure this against.
Both holes get a new J plan file (J-109, J-110). Everything else is riders/rejections below.

================================================================================
1. AgenticSTS (arXiv:2607.02255) — bounded typed-memory contract testbed in Slay the Spire 2
================================================================================
S1 BOUNDED TYPED-MEMORY CONTRACT (retriever.py:444/129/713 — fresh prompt per decision from 5 named
   layers, per-decision-type token budget, capped top-k, flat prompt at any run length).
   ELAI: CONTEXT BUDGET ALLOCATOR [map:689, 8 fixed slots + floors/steal + breakpoints] + H-50
   regime policy + J-25 typed digest + CL-1..CL-4.
   VERDICT: CONFIRMATION. Same design, externally validated with a per-layer ablation. Their
   doctrine note "retrieve summaries + triggered guides, NEVER nearest-neighbor raw-log RAG"
   matches VCG trigger-signature retrieval (map:1546-1555). No new plan.
S2 WRITE-GATE CASCADE (write_gate.py:627/323/43-52): exact-match->update; REJECT if candidate
   cosine-overlaps an IMMUTABLE operator/rules span >=0.70 (StaticSpanIndex, fingerprint-stale
   rebuild); vs existing entries >=0.85 auto-reject, [0.70,0.85) split on trigger-Jaccard >=0.60
   MERGE else COEXIST; ambiguous band -> batched LLM judge (advisory); embed-fail degrades to
   lexical, NEVER blind-accept; every decision logged.
   ELAI: E-5 Mem0 ADD/UPDATE/DELETE/NOOP [map:696], I-13 write-time injection scan [map:962],
   C-48 SafeHarbor TrustTier [map:966], J-21 semantic validation, I-12 evidence-gated playbook
   curator [map:964]. NONE checks a candidate against the immutable instruction surface
   (CLAUDE.md/rules/operator prompts) — "don't re-learn what the rules already say" is absent —
   and none has the two-threshold merge/coexist band keyed on trigger context.
   VERDICT: EXTENSION(E-5/I-12/I-13/L-12) -> J-109 leg A.
S3 PRE-WRITE A/B COUNTERFACTUAL GATE (prewrite_ab.py:19/244): byte-exact ORIGINAL failing prompt
   fetched from the run log, candidate skill injected, re-decide N=3 on the strategic tier,
   independent judge, admit iff zero-harmful AND >=ceil(2/3) hits — a skill is written only if it
   demonstrably flips the originating mistake.
   ELAI: H-43 execution-grounded fitness (CodeRM pass-rate over frozen tests) [map:1490], J-97
   gated reuse, K-19 SF-1..SF-4, L-5 counterfactual do-test on CLAIMS. The do-test intervenes on
   claims; H-43 scores DGM candidates against frozen test suites. Neither replays the ORIGINATING
   failure with the candidate injected at admission time. ELAI is uniquely positioned to do this
   cheaply: the SSB/RRA journal already stores the byte-exact prompt (AgenticSTS had to build a
   log fetch; ELAI gets fetch_prompt_a for free).
   VERDICT: EXTENSION(H-43/J-96/J-97/I-12) -> J-109 leg B. Composes with S2 as one admission
   pipeline; SF-1..SF-4 + H-31 auditor firewall bind it.
S4 FOUR-TIER HEALTH-AWARE ROUTER (llm_router.py:197/439): HARD vs SOFT failure classes,
   per-(credential-profile,provider,model) circuit breaker CLOSED/OPEN/HALF_OPEN, peer-first
   fallback, credential-profile isolation.
   ELAI: B-37 PerRoleRuntime + fallback chains [WIRED, map:1849], J-54 FreePool quota/self_heal,
   per-model breakers in crates/knowledge.
   VERDICT: CONFIRMATION. Rider on J-54: adopt the HARD(switch-now)/SOFT(bounded-retry) failure
   dichotomy + credential-profile breaker key if not already in its self_heal table.
S5 ROLE-TIERED DISPATCH + DECISION BATCHING (~67 strategic calls/run; one strategic call emits a
   multi-action plan executed without re-calling; no-LLM fast tier).
   ELAI: J-84 2-axis effort classifier (trivial=ZERO) [map:360], architect/editor routing (H-7),
   deterministic-harness doctrine. VERDICT: CONFIRMATION.
S6 ABLATABLE NAMED LAYERS (--no-skills/--no-memory/--no-llm): CONFIRMATION of the wire-dark flag
   doctrine + GD-3 gate-ON-vs-OFF ablation framing. Their honest "directional not significant,
   Fisher p~=0.37, N=10" labelling is exactly ID-5/KD-1 discipline — cite as external precedent.
LICENSE NOTE: steal ideas only from the Apache-2.0 AgenticSTS/ tree; the C# mod is AGPL — no code
   port from it (no code is ported anyway; mechanisms re-implemented in Rust).

================================================================================
2. ProgramAsWeights (arXiv:2607.02512) — compile a spec into a tiny frozen-base LoRA program
================================================================================
P1 PERSISTED PREFIX-KV COLD-START CACHE (runtime_llamacpp.py:153-196: eval the stable prefix once,
   serialize seq state to disk, reload keyed by prefix hash).
   ELAI: the sidecar PrefixKVCache (server.py:1819) is BYPASSED on the streaming path
   (runtime_client.rs:547) — H-50 optional leg / J-95 G-wires already plan the unbypass [map:1844].
   VERDICT: RIDER on J-95/H-50 (recorded, not re-authored — the unit exists): when unbypassing,
   ALSO persist the prefix KV state to disk keyed by blake3(prefix tokens + model pin) so
   cold-start after a sidecar restart is eliminated too, not just intra-process. PAW proves the
   pattern llama.cpp-native; the transformers equivalent is a past_key_values serialization.
   Volatile-time/cache doctrine (HD-2) unaffected — the persisted prefix is byte-stable by
   construction.
P2 PER-TASK COMPILED LoRA ADAPTERS for recurring fuzzy micro-decisions (classify/route/repair).
   Section-4 ledger: "LoRA fine-tuning: DEFERRED entirely (catastrophic forgetting 5-19pp)"
   [map:3157] + "LoRA-swap: REJECT for the runtime (not ONNX-CPU portable)" [map:3516]. PAW's
   frozen-base variant dodges the forgetting objection (base never trained) BUT the compile step
   is their HOSTED PAID service (compiler/mapper training code absent from the repo — verified),
   and local training is D-026 GPU-deferred.
   VERDICT: REJECT now (re-proposing a ledger entry would need new evidence; the hosted-service
   dependency is disqualifying on its own — GD-5). Record as PRIOR-ART RIDER on I-30/I-31: when
   the GPU unlock fires, the frozen-interpreter + tiny-task-adapter shape and
   convert_peft_to_paw.py packaging are the reference implementation.
P3 .paw ARTIFACT FORMAT (paw_format.py:255-330: magic + versioned JSON metadata + safetensors,
   validate-at-load with size/rank caps + code-injection grep) and P4 HYBRID PROGRAM = auditable
   pseudo-program TEXT + weight delta.
   ELAI: RRA bundle (msgpack+zstd+blake3), J-102 config-ingress admission, auditable-artifact
   doctrine. VERDICT: CONFIRMATION + rider on I-30 (if ELAI ever ships adapters, package as
   text-half-auditable + validated-at-load single file; the text half stays greppable/diffable,
   satisfying the wiring/audit doctrine).
P5 FuzzyBench numbers (73.78% vs 32B 68.70%): as-reported, in-distribution, eval harness NOT in
   repo — tier-2 at best, never an ELAI number (ID-5). No action.

================================================================================
3. agentic-abstention (arXiv:2606.28733) — do agents know when to STOP instead of act
================================================================================
A1 TYPED THREE-WAY ACTION (ANSWER/ABSTAIN/TOOL_CALL at every turn, action_space.py:14-30).
   ELAI: I-8 Aletheia acknowledge_infeasible tool [map:565], D-16 conformal refusal, L-7
   policy-gate abstain, J-84 very-hard=escalate-or-abstain [map:365]. VERDICT: CONFIRMATION —
   ELAI's design already treats abstention as a first-class terminal outcome.
A2 ABSTENTION TIMELINESS METRICS (evaluation.py:89-160; SPL = oracle_step /
   max(first_abstain_step, oracle_step); Timely Recall = pass@0; computed over
   abstain-warranted samples only). Paper's measured point: agents abstain too late or never
   (<40% timely recall everywhere) — lateness, not absence, is the dominant failure.
   ELAI: H-39 scores abstention CORRECTNESS [map:326]; J-107 counts abstention-rate [map:2451].
   NOTHING scores lateness/budget-burned-before-stop. The metric is exactly ELAI's gate shape:
   off-agent, recomputable from the SSB trajectory, agent never authors it.
   VERDICT: EXTENSION(H-39 + 1.13 bench family) -> J-110 leg A.
A3 READ-TIME vs DELAYED ABSTENTION TAXONOMY + instruction-level task mutation (exactly one of
   false_premise / contradiction / underspecified_intent injected so a careful agent abstains
   BEFORE acting; __delayed_execution/__delayed_observation variants discoverable only after).
   ELAI: FEASIBILITY_PROBE [WIRED, map:2338] + B-40 critic + B-38 ambiguity score cover the
   read-time GATE; what's missing is the TASK SUPPLY to measure them: no abstention-warranted
   split exists in the eval corpus (J-56/L-20 mine real solvable tasks only).
   VERDICT: EXTENSION(J-56/L-20) -> J-110 leg B: a deterministic mutation leg that derives
   abstention-warranted variants from mined tasks, giving A2's metric a denominator and
   FEASIBILITY_PROBE/B-38/B-40 their first measured number.
A4 CONVOLVE STOPPING-RULES PLAYBOOK (Reflect->Compress->Reuse; bulleted playbook appended to
   context, no weight updates; claimed 26.7->57.4% AbsRec@1 from ~20 trajectories — but the
   DISTILLER IS NOT IN THE REPO, consumer-side only; claim unverifiable from code).
   ELAI: J-10 insight backprop [map:578] + I-12 evidence-gated playbook curator [map:964] are
   this exact family, with stronger admission (>=k SSB-verified outcomes vs their LLM-judge).
   VERDICT: CONFIRMATION of J-10/I-12. Rider on I-12: add a STOPPING-RULES bullet category
   distilled specifically from late-abstention failures (flagged by J-110's metric), admitted
   through J-109's counterfactual gate — the three units compose into a closed loop
   (measure lateness -> distill stopping rule -> prove it flips the originating failure -> admit).
   Cross-size transfer ("distill on small, apply on large", index.html:245) recorded as an
   UNVERIFIED paper claim — never load-bearing.
A5 LEXICAL ABSTENTION-KEYWORD PREFILTER (abstention_keywords.py): CONFIRMATION of the T1/T2
   cheap-deterministic-first gate layering. Their own file comment concedes real systems need
   more than keyword search. No action.

================================================================================
NEW PLAN FILES (authored this session, J family; treats all prior plans as done per spec-eval)
================================================================================
J-109 LEARNED-ARTIFACT ADMISSION HARDENING (AgenticSTS S2+S3): one admission pipeline at the
  memory/playbook/case/skill write seams — leg A immutable-surface span index + trigger-keyed
  merge/coexist dedup band; leg B pre-write counterfactual replay fitness (byte-exact originating
  prompt from the SSB, candidate injected, N re-decides, zero-harmful + supermajority, off-agent
  verdict). EXTENDS E-5/I-12/I-13/H-43/J-96/J-97; BOUND by K-19 SF-1..SF-4 + H-31 firewall.
J-110 ABSTENTION TIMELINESS + ABSTENTION-WARRANTED EVAL SUPPLY (agentic-abstention A2+A3): leg A
  SPL/timely-recall computed off the SSB trajectory, feeding H-39 promotion + the 1.13 bench
  family; leg B deterministic abstention-warranted task mutation extending J-56/L-20 corpus rows
  with a should_abstain split; stopping-rules playbook category recorded as the I-12 rider above.
Split into two files because the scopes share no subsystem (1.5/1.7 self-improvement/memory vs
1.13 eval) — fewest-files rule honored within each.

RIDERS RECORDED (no new files): J-95/H-50 prefix-KV disk persistence (P1); I-30/I-31 PAW
frozen-base adapter + packaging prior-art (P2/P3/P4); J-54 HARD/SOFT failure dichotomy +
credential-profile breaker key (S4); I-12 stopping-rules category (A4).
REJECTED (ledger-consistent): per-task compiled LoRA now (P2 — LoRA DEFERRED + hosted-compile
dependency); CONVOLVE-as-new-unit (A4 — J-10/I-12 already specify the family); LLM-judge as the
admission authority anywhere (TD-3 — judges stay advisory in J-109's band).
arXiv IDs verified by the explorers against live pages this session: 2607.02255, 2607.02512,
2606.28733 (each resolved + abstract matched the repo).

OUTCOME: J-109 + J-110 authored at docs/thefinalglance/J-plan/; indexes + feature map upserted.

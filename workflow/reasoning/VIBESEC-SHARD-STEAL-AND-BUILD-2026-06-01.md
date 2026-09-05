ELAI x VIBESEC x SHARD - WHAT TO STEAL AND WHAT TO BUILD
Authored 2026-06-01. A grounded synthesis answering: from the VibeSec landscape and from SHARD
plus the prior-art systems that beat SHARD, what mechanisms can ELAI borrow ("steal") and what
net-new capabilities should it build. Companion to:
  workflow/reasoning/ELAI-DIRECTION-2026-06-01.md (the strategy this serves)
  workflow/research/shard-prior-art/00-COMPARISON-shard-vs-priorart.md
  workflow/reasoning/VIBESEC-RESEARCH-2026-06-01-LANDSCAPE.md
  workflow/reasoning/VIBESEC-TOOLS-MATRIX-2026-06-01.md
  workflow/reasoning/VIBESEC-PAPERS-2026-06-01.md

HOW THIS WAS GROUNDED (provenance, read before trusting any claim below)
  Ten candidate proposals were drafted from the five research files above + a full read of ELAI's
  phase index (pre-0..4M) and the B/C/D/E plan summaries (161 planned sessions). Each draft was then
  put through a two-stage adversarial verification workflow (run wf_b959963a-db6, 20 Sonnet agents):
  stage 1 ground-checked the mechanism against the cited research file and mapped the exact ELAI
  integration point by reading the real crates + grepping elai.map and the plan summaries; stage 2
  was a hostile auditor instructed to REFUTE each proposal on novelty (is it already planned?),
  grounding (is the number real?), and feasibility (does ELAI's architecture allow it?). Every
  number and plan-overlap below carries the verdict that survived that refutation. Where the
  adversarial pass corrected a draft claim, the correction is kept and the original is flagged. This
  doc obeys the grounded-by-default HARD RULE: anything not survivable is labelled UNVERIFIED.

==============================================================================
0. EXECUTIVE VERDICT
==============================================================================
The headline: ELAI's 161-session plan set is large and already security-heavy (E-30 destructive-verb
argv gate, E-37 prompt injection, E-43 skill scanning, E-11 slopsquat block, E-38 EU AI Act,
E-15 sandbox, E-39 cost ceiling, E-53 formal verification), yet NONE of the ten mechanisms below are
fully covered by an existing plan. Six are genuinely new; four are partially covered but with a real,
named residual gap. All ten survived adversarial refutation as worth doing. All ten are "feasible but
HARD" - none trivial, none impossible. Every one needed a scoping correction (the revised scope is
recorded per item); none should be shipped as originally drafted.

The single most valuable move across all ten, and the one that serves ELAI-DIRECTION's two open
release risks (R1 "one real measured number", R2 "distribution"): produce the FIRST published
before/after insecure-code-rate number on ELAI's own pre-write gate (item C2). The whole VibeSec
market sells the posture and nobody publishes the number (tools-matrix J #6, landscape 7.3). That one
number is simultaneously R1 (a measured number on ELAI's own engine), R2 (the anti-Replit demo: "ELAI
physically blocked the SQLi/secret/slop before it was written, here is the gate and the before/after
rate"), and a first-in-category defensible claim no competitor holds.

The unifying product concept is VibeGate (item C1): one deterministic, fail-closed, in-process
write-gate that, on the same PreToolUse event, blocks the four VibeSec axes (code-vuln, hallucinated
dependency, exposed secret, malicious skill). ELAI already has every substrate (rules-as-data engine
with AST/GREP/COMPOSITE predicates, the mutable=0 immune-rule column, PreToolUse hooks, the SSB
journal that makes the gate auditable and reproducible). The four axes today exist as four disconnected
single-axis mechanisms; no competitor unifies them and the one OSS tool that tried (CodeGate) is dead.
That unification IS ELAI's defensible VibeSec position, and it is on-thesis: a fully journaled,
deterministic, schema-validated write-gate is "a security control you can audit and reproduce."

What to STEAL (borrow a shipped mechanism, adapt to ELAI):
  - SVEN / CodeGuard+ constrained-secure-decoding numbers as the target ceiling for a code-vuln gate (C2)
  - AgentDojo Tool Filter least-privilege "commit-before-untrusted-observation" (C3)
  - SHARD time-decay staleness + quarantine + the cross-subsystem handle_stale_memory self-healing
    cascade; Mem0 A.U.D.N.; Graphiti bi-temporal contradiction invalidation (C5)
  - SHARD T1-T4 skill trust ladder + Voyager critic-gated acquisition loop (C6)
  - Springdrift outcome-learning CBR ranking; the proven-complete safety-calculus methodology (C7)
  - SCF symbolic IntentNode pre/postcondition conflict detection (C8)
  - Socket Firewall at the install boundary as defense-in-depth (C9, ops-level)

What to BUILD (net-new for the category, ELAI uniquely positioned):
  - VibeGate: the unified four-axis fail-closed write-gate (C1)
  - The first measured pre-write insecure-rate number on ELAI's own engine (C2)
  - CaMeL-style capability/Dual-LLM data-plane separation for untrusted-content agents (C3)
  - A test-integrity / output-grounding "cant-lie" gate that binds claimed PASS to an
    orchestrator-captured artifact hash (C4) - this is the literal under-built half of the north star
  - A per-iteration security-monotonicity gate for the autonomous REFINE / auto-advance / self-mod
    loops (C10)

==============================================================================
1. GROUNDING DISCIPLINE - CORRECTIONS THE ADVERSARIAL PASS FORCED (read this)
==============================================================================
These are the draft claims that did NOT survive refutation. They are recorded so the next session
does not re-introduce them, per the project's "when wrong, state it" rule.

  1.1 CaMeL token overhead (C3). Draft cited "~2.82x input / ~2.73x output". VERDICT: OVERSTATED.
      Those multipliers are SECONDARY-only (MarkTechPost/DeepMind-attributed; the arXiv:2503.18813
      PDF did not render in the research pass). ELAI's IPC payloads are far more verbose than the
      AgentDojo task payloads the multiplier was measured on, so true overhead on ELAI's wire could
      be 4-5x, which would blow the 32K local context ceiling. The 77%/84% provable-security pair and
      the Tool Filter 57.7%->6.84% ASR / 73% utility ARE primary-verified and safe to cite.

  1.2 Springdrift numbers (C7). Draft cited "7,056 proposition pairs, 100% coverage" and "+10.7% P@4"
      with grounding_confidence VERIFIED. VERDICT: OVERSTATED. The springdrift dossier itself marks
      these SYNTHETIC / n=1 self-reported / UNREPRODUCED; the P@4 confidence interval overlaps the RAG
      baseline ("positive but not strongly separated"); the safety-calculus axiom constants were not
      fetched from source ("claimed from paper only"). The git-whole-state mechanism IS confirmed
      real. Treat the calculus completeness and CBR uplift as design hypotheses, never as targets.

  1.3 "API_REGISTRY_CHECK is shape-only" (C9). Draft claimed ELAI's registry check is string-shape.
      VERDICT: WRONG (corrected by ground-check). D-11's checkers/registry/registry_check.py is a
      real HTTP existence + publish-date + age checker for PyPI and npm. The true gap is narrower:
      invocation timing (it fires at the PR boundary, not at write-time) and missing cargo coverage.
      Caveat: D-11 is PLANNED, not shipped, so it is a prerequisite, not a primitive to rewire today.

  1.4 The "below 15%" target and the "30-commit GitGuardian replay corpus" (C1/C2). VERDICT:
      OVERSTATED / UNVERIFIED. The Veracode 45% is across 80 OWASP-Top-10 tasks; inferring that XSS +
      Log-Injection "account for the majority" of it from two per-class failure rates is a
      non-sequitur. The "GitGuardian 3.2%" is a population statistic, not a named reproducible corpus.
      Any measurable claim must define its own synthetic benchmark, not borrow these as if they were
      replay sets.

  1.5 Voyager "3.3x items" (C6). VERDICT: non-transferable. It is a Minecraft tech-tree number on a
      frozen 2024-04 codebase; it is zero evidence for coding-task skill-reuse rates. Any ELAI reuse
      target must be measured from ELAI's own runs.

  1.6 SCF "deliberately low-precision" framing (C8). VERDICT: OVERSTATED. The 27.9% precision is a
      limitation of SCF's simulation-only evaluation (0 stars, empty tests/, no live LLM, no framework
      adapter), not an intentional design choice. The mechanism (IntentNode pre/postconditions, 3
      conflict types) is portable, but its precision number has never been validated outside its own
      synthetic scenario set.

==============================================================================
2. THE CROSS-CUTTING ARCHITECTURAL INSIGHT (the single most useful finding)
==============================================================================
The adversarial pass surfaced one tension that recurs across almost every proposal and should shape
how all of this is built:

  ELAI's enforcement substrate is a synchronous, in-process, deterministic, SSB-journaled hook. That
  substrate is PERFECT for axes that are pure in-memory computation (AST pattern match, regex/GREP on
  a candidate diff, set-comparison) - those are genuinely fail-closed, sub-millisecond, replayable
  under ELAI_SEED, and journaled for audit. That is ELAI's real moat: a write-gate you can reproduce.

  The same substrate FIGHTS every axis that needs (a) network I/O at gate time (registry existence
  lookup), (b) a subprocess at gate time (mcp-scan, semgrep, a CWE classifier), or (c) a model run
  that does not yet exist (the Qwen secure-rate benchmark, RunPod-pending). Each of those either
  breaks the fail-closed guarantee (a 500ms timeout with fail-open silently passes exactly when a
  slopsquatted import is most dangerous - air-gapped build, flaky registry), or breaks determinism
  (non-deterministic network result in a replay-journaled gate), or breaks latency (a subprocess on
  every Edit stalls the FSM mid-EXECUTING).

  THE DESIGN RULE THAT FALLS OUT: split every gate into Tier 1 (in-process, zero-latency,
  genuinely fail-closed - ship now) and Tier 2 (network / subprocess / model-dependent - move the
  expensive check OUT of the per-write path: cache the registry/existence result at plan-start, run
  the CWE classifier on the delta as a post-write async check feeding the auditor, run the secure-rate
  benchmark on RunPod offline). State which guarantee each tier actually delivers; never let a
  fail-open Tier-2 axis advertise the fail-closed promise. This single rule resolves the recurring
  feasibility objection on C1, C2, C9, C10 and is the most reusable conclusion in this document.

  A second cross-cutting note: "all ten are HARD" is itself a finding. None of these is a quick win;
  the cheap wins in this space (E-30 argv allowlist, E-37 spotlighting, E-43 mcp-scan) are already
  planned. What remains is the genuinely hard, genuinely defensible whitespace - which is exactly why
  it is uncontested (CodeGate died, nobody publishes the number, no coding product ships CaMeL).

==============================================================================
3. THE SHARD VERDICT FOR ELAI (blueprint, not adoption)
==============================================================================
From 00-COMPARISON-shard-vs-priorart.md, re-confirmed: SHARD is a composition-interface BLUEPRINT and
a borrow-list target, not a system to adopt or a novelty to claim. Its flashiest paper claims
(semantic conflict detection, self-improving agents) are spec-only in the repo (review() is
string-equality; improvement.py is a 404; skill similarity is Jaccard with a "replace with embeddings"
TODO), and its headline semantic-detection result is self-undercut (E2 semantic 7/10 < its own
syntactic 9/10). So "better than SHARD" means: take SHARD's genuine shipped primitives (time-decay
staleness + quarantine, T1-T4 trust ladder, the cross-subsystem self-healing cascade, the code-enforced
rate-limit/scope-guard/rollback layer) AND lift the stronger shipped mechanism from each pillar leader:
  memory + semantic conflict     -> Mem0 A.U.D.N. + Graphiti bi-temporal invalidation        (C5)
  symbolic semantic conflict     -> SCF IntentNode pre/postconditions                          (C8)
  skill acquisition              -> Voyager critic-gated embedding-retrieval loop              (C6)
  safe self-improvement          -> DGM archive search UNDER guards + a real semantic review   (C8, self-mod)
  composed-runtime safety/audit  -> Springdrift proven-complete calculus + git whole-state + CBR (C7)
Assemble them under SHARD's composition framing (governance must COMPOSE because failure modes are
multiplicative) and ELAI's own staleness/trust governance. ELAI already has more shipped than SHARD on
most of these axes; the value is the missing CROSS-SUBSYSTEM wiring (C5) and the missing learning loops
(C6, C7-CBR), not re-implementing SHARD.

==============================================================================
4. THE TEN PROPOSALS (full detail; each carries its survived verdicts)
==============================================================================
Legend per item: CLASS = STEAL/NEW/HYBRID. NOVELTY = GENUINELY_NEW / PARTIALLY_COVERED (vs existing
plans). FEAS = feasibility after refutation. All ten recommendation = REVISE (worth doing, with the
recorded scope change). Effort is post-refutation (drafts that hid scope were bumped).

------------------------------------------------------------------------------
PART A - VIBESEC PREVENTION-IN-THE-LOOP (the write-time / runtime gates)
------------------------------------------------------------------------------

C1. VibeGate - unified fail-closed multi-axis in-process write-gate
  CLASS HYBRID | NOVELTY GENUINELY_NEW (vs E-30 argv-only, E-37 prompt-injection, E-43 skill soft-gate)
  | FEAS HARD | EFFORT L
  Steal/build: one PreToolUse handler that fans a file-write across four axis-evaluators - code-vuln
  (AST/GREP security-domain predicates), dependency-existence, secret-exposure (GREP on the diff),
  malicious-skill (mcp-scan) - and returns BLOCK fail-closed if any fires, backed by a "vibesec" rule
  domain seeded mutable=0 so per-project rule edits cannot disable security.
  Grounded by: tools-matrix J #1 (no tool unifies the four axes), J #4 (fail-closed + Claude Code
  issue #37210 where a deny was silently ignored for Edit - so per-tool contract tests are
  mandatory), CodeGate archived 2025-06-05 (the one prior unified local gate died; study why -
  false-positive UX friction is the likely killer). ELAI ground truth (code-read): the rules engine
  already has ALWAYS_INCLUDED_DOMAINS = [engineering, security] (rule_engine.rs:187) and the mutable
  INTEGER column (schema.rs:46, default 1) with a tested mutable=0 path (lifecycle.rs:871); there are
  no mutable=0 rows in the security domain yet, and the four axes are not wired into one compound
  check.
  Integration: crates/plugins/src/hooks.rs (run_pre_tool_use) -> compound VibeGate evaluator ->
  crates/rules RuleEngine; seed mutable=0 security rows in schema.rs; SSB action types
  vibesec_{code_vuln,dep_hallucinated,secret,skill}_blocked.
  REVISED SCOPE (the correction that makes it shippable): two tiers. TIER 1 (ship now, genuinely
  fail-closed, pure Rust, no network/subprocess): code-vuln AST/GREP + secret GREP on the candidate
  diff, with the four per-tool contract tests (Edit/Write/NotebookEdit/Bash) the #37210 caveat
  mandates, and a blake3 integrity check of the seed rows at startup (mutable=0 is not truly immune if
  the SQLite is user-writable). TIER 2 (design-gate first): the dependency-existence and mcp-scan axes
  are network/subprocess and must follow the section-2 rule - cache registry results at plan-start,
  make mcp-scan a per-session not per-write check, and document which guarantee each axis actually
  delivers. Measure false-positive rate on >=3 real orchestrate runs before enabling Tier 2 outside
  development. Drop the "below 15%" and "GitGuardian replay corpus" framings (section 1.4).
  Measurable: 0 secrets and 0 known vuln-class AST matches written to disk across a defined synthetic
  corpus, with a measured and bounded false-positive rate on real runs.

C2. Code-vuln-class pre-write gate + THE FIRST MEASURED INSECURE-RATE NUMBER  (the strategic keystone)
  CLASS HYBRID | NOVELTY GENUINELY_NEW (vs E-12 post-write semgrep, C-26/E-35 decoding substrate,
  E-9/E-49 functional-only benchmarks) | FEAS HARD | EFFORT L (split below)
  Steal/build: populate the always-on security rule domain with curated vuln-class patterns
  (SQLi/XSS/weak-crypto/missing-authz) that BLOCK at write; optionally add a secure-decoding layer on
  the sidecar; and CRUCIALLY produce the first published before/after insecure-rate number.
  Grounded by (all VERIFIED-abstract/table): SVEN (CCS 2023, arXiv:2302.05319) retrain-free prefix
  steering 59.1%->92.3% secure; CodeGuard+ (arXiv:2405.00218) constrained decoding 43.2%->76.0%
  secure-pass@1, beating GPT-4, no correctness loss; SeCodePLT (arXiv:2410.11096, NeurIPS 2025, 5.9k
  samples/44 CWEs, GPT-4o 55% secure); BaxBench (o1 62% correct, ~half exploitable); SUSVIBES (61%
  correct / 10.5% secure). tools-matrix J #6 + landscape 7.3: NO tool publishes a measured pre-write
  reduction number. CAVEAT (kept): the SVEN/CodeGuard+ numbers are CodeGen-2.7B; they do NOT transfer
  to Qwen2.5-Coder-7B-abliterated without an independent run.
  Integration: new crates/rules/rules/security/*.toml vuln-class ruleset in the existing security
  domain; gate via the PreToolUse path; benchmark extension to crates/cli/src/benchmark.rs (E-9
  harness) adding a --suite seccodeplt mode that reports insecure-rate alongside functional pass@1.
  REVISED SCOPE (split): C2a (effort M) - ship the vuln-class ruleset + write-time BLOCK, scoped to
  Python/TypeScript output first to bound false-positive surface, make NO performance claim yet. C2b
  (gated on C-26/E-35 shipped + RunPod) - run SeCodePLT/CodeGuard+ on Qwen2.5-Coder-7B-abliterated,
  produce the first measured before/after insecure-rate; the optional constrained-decode sidecar layer
  lives entirely here.
  WHY THIS IS THE KEYSTONE: it is ELAI-DIRECTION R1 (a real measured number on ELAI's own engine),
  R2 (the anti-Replit distribution demo), and a first-in-category claim, in one deliverable. Do not
  ship the gate with a projected/transferred number; gate the headline on the measured Qwen result.

C9. Hallucinated-dependency existence+age+provenance gate at WRITE-time + install-boundary wrap
  CLASS HYBRID | NOVELTY PARTIALLY_COVERED (D-11 ships the real HTTP checker; E-11 idea-framing block;
  E-41 PR-leg dep-verify) | FEAS HARD | EFFORT M
  Steal/build: move D-11's real registry_check.py (HTTP existence + publish-date + age, PyPI+npm) from
  the PR boundary to ALSO fire as a PreToolUse hook the instant an import/dep line is written; add a
  cargo (crates.io) check; wrap Socket Firewall at the install boundary as defense-in-depth.
  Grounded by (VERIFIED): Spracklen USENIX 2025 Distinguished Paper (19.7% hallucinated, 205,474
  unique fake names, 43% recur on all 10 reruns); Importing Phantoms (inverse hallucination/HumanEval
  correlation); Socket Firewall (free, cargo+pip/uv+npm transitive) explicitly CANNOT close the
  freshly-registered-slop gap (a new fake name has no CVE) - that gap is exactly the codegen-time
  existence+age slot tools-matrix J #2 calls uncovered. Correction from section 1.3: the checker code
  is real, not shape-only; the gaps are trigger-timing and cargo.
  Integration: extend registry_check.py with _check_cargo() (crates.io/api/v1/crates/{pkg}); new
  PreToolUse hook firing only when the Edit/Write path matches an import/use/Cargo.toml pattern AND
  the name misses a local LRU existence cache (TTL ~1h) - the cache is what keeps the network call off
  the hot per-write path (section 2 rule). Make the staleness threshold (730d) per-ecosystem
  configurable (stable Rust crates routinely go years without a publish - a hard 730d fails real deps).
  REVISED SCOPE: reframe as an extension to D-11 + E-41 (prereq: D-11 ships), one baby step for
  cargo + the cached PreToolUse trigger. Move the Socket Firewall wrap to an ops runbook, not a plan.
  Measurable: 0 hallucinated import lines reach disk on a 50-sample 404-confirmed slop suite; 0
  false-positives on the 20 most-used real ELAI deps.

C10. Per-iteration security-monotonicity gate (the autonomous-loop blind spot)
  CLASS NEW | NOVELTY GENUINELY_NEW (vs B-39 REFINE, B-22 auto-advance, D-4 tripwires, E-7 Two-Gate -
  all correctness/score-domain only) | FEAS HARD | EFFORT L (split below)
  Build: at each REFINE / auto-advance / self-mod iteration, fingerprint the generated DELTA by
  CWE-class and BLOCK/escalate if iteration N introduces a vuln class absent at N-1 - a security
  monotonicity invariant, distinct from correctness regression.
  Grounded by (VERIFIED): security degrades +37.6% in critical vulns after just 5 AI-improvement
  iterations (Shukla, arXiv:2506.11022, 400 samples); a standalone LLM injects ~11x more
  vulnerabilities than humans via APR (135 vs 12, v3; CWE-78/95/502/22 dominant; more-autonomous
  frameworks inject MORE; Sajadi arXiv:2507.02976 v3 - cite v3, NOT the stale v1 185/9x/20). ELAI runs
  exactly these loops, and every existing canary is correctness-only.
  Integration: SecurityMonotonicityGate(delta, prior_cwe_classes) wired pre-REFINE-commit
  (chat_reflect / live_cli.rs:~1685) and pre-auto-advance (task_runner.rs:~3996); SSB
  security_class_regression_blocked.
  REVISED SCOPE (the hidden cost the refutation exposed): the gate struct is 1-2 days; the CWE-class
  CLASSIFIER it calls does not exist and building one calibrated for Rust (ELAI's dominant language,
  where semgrep CWE rules are sparse) is itself L-effort. Split: phase 1 (L) build + calibrate the
  classifier against ELAI's own historical REFINE deltas to a measured FP<5% on 50 clean patches per
  language; phase 2 (S-M) wire it in. Run it as a post-write async check feeding the auditor, not a
  blocking per-write hook (section 2 rule), to avoid stalling the live REFLECT pipeline.

C3. CaMeL capability / Dual-LLM data-plane defense for untrusted-content agents
  CLASS NEW | NOVELTY GENUINELY_NEW (vs E-37 input-marking, E-24 MCP pin, C-11 OS isolation) | FEAS
  HARD | EFFORT XL (split below)
  Steal/build: ELAI's Tier-C isolation + visibility filter defend the CONTROL plane (which tools/paths
  an agent may touch). They do NOT defend the DATA plane - an injected instruction inside repo
  content, a web page, a skill file, or an MCP tool response can still steer a model that holds both
  untrusted input and privileged tools (the lethal trifecta). Adopt the capability/Dual-LLM pattern
  for the researcher / web-research / MCP-consuming agents.
  Grounded by: CaMeL (arXiv:2503.18813, SaTML 2026) - privileged LLM emits code over symbolic
  variables with capability metadata, a non-LLM controller enforces info-flow so untrusted data can
  never alter control flow; 77% AgentDojo tasks WITH provable security (84% undefended) -
  ABSTRACT-VERIFIED. AgentDojo Tool Filter (arXiv:2406.13352): commit to the minimal tool set BEFORE
  observing untrusted data cut ASR 57.7%->6.84% at 73% utility - TABLE-VERIFIED. Design Patterns
  (arXiv:2506.08837) Dual-LLM / Plan-Then-Execute. Token-overhead caveat: see section 1.1 (secondary
  only; must be measured on ELAI's verbose IPC before committing).
  Integration: the researcher agent becomes the quarantined tool-less LLM, returning typed SymbolicVar
  structs (not raw strings) over IPC; a new crates/orchestrator/src/capability_flow.rs validates no
  untrusted-derived SymbolicVar reaches a privileged sink; Tool Filter maps directly onto ELAI's
  existing per-phase FSM tool gating. SSB capability_flow_blocked / untrusted_var_quarantined.
  REVISED SCOPE (split): phase 1 (MEDIUM, ship first, primary-verified, FSM-native, no IPC schema
  change) - add Tool Filter: lock the researcher's tool set to a per-task minimal subset before any
  untrusted byte is observed; measure ELAI's own prompt-injection ASR on the researcher+web pipeline
  against an AgentDojo-style fixture, before and after. Phase 2 (XL, conditional) - full CaMeL
  SymbolicVar enforcement, gated on (a) primary-source token-overhead from the PDF, (b) a measured
  overhead on ELAI's actual IPC format proving the 32K ceiling survives, (c) acceptance of the IPC
  schema breaking change. No coding product ships CaMeL/Tool-Filter as the gate's reasoning model -
  that is the correct differentiator framing to keep.

C4. Test-integrity + output-grounding gate - the literal "cant lie to you" half of the north star
  CLASS HYBRID | NOVELTY PARTIALLY_COVERED (C-27 DeterminismDrift+ELAI_SEED, C-28 mutation, C-22
  stdout_hash on the feasibility path) | FEAS HARD | EFFORT M
  Build: the "cant wreck your system" half is well covered (FSM gates, PreToolUse, sandbox). The "cant
  lie" half is under-built. Bind every claimed test outcome to a deterministic, SSB-journaled,
  orchestrator-captured artifact so a fabricated PASS is detectable.
  Grounded by: the Replit incident (AI Incident DB #1152, CONFIRMED: ~1,206+1,196 records wiped, 4,000
  fabricated users, FAKED unit-test results, FALSE rollback claim) is the canonical lie-class failure.
  RedCode (arXiv:2411.07781, NeurIPS 2024, VERIFIED) - unsafe ops in natural language get LOWER
  rejection than in code, so the harness cannot trust the agent's self-report. memory
  feedback_verify_semantics: "PROVEN requires output content to match input semantics; a 511-byte Vite
  splash != ELAI built a TODO app." ELAI ground truth (code-read): TestOutcomePayload journals
  {test_name, passed, duration_ms} but has NO stdout_hash; the auditor runs tests inside its own LLM
  Bash tool, so its PASS narrative is not bound to an orchestrator-captured subprocess output. (C-22
  already adds stdout_hash on the FeasibilityProbe path - so the pattern exists; the gap is
  TestOutcomePayload specifically.)
  Integration: add stdout_hash: Option<String> (blake3 of truncated subprocess stdout, the
  feasibility_probe.rs:168 pattern) to TestOutcomePayload; gate so the auditor cannot emit passed=true
  without a matching orchestrator-captured hash row in the same superstep; emit SemanticGroundingFinding
  at AUDITING->MEMORIZING requiring >=1 non-LLM signal (exit 0 + hash present) before PROVEN.
  REVISED SCOPE: scope to the single concrete field + gate; PRE-RESOLVE the blocking design question
  first - the auditor must run tests through an orchestrator-managed process slot (so there IS a stdout
  handle to hash), otherwise the field is dead. Split the semantic output-grounding check to a separate
  proposal (it needs task-specific spec-pinned expected outputs and is not deterministically
  specifiable yet). Pre-req on C-27 + C-28 (they already shrink the gap to single-run sessions).
  Measurable: inject a baby step whose test process exits 1 while the auditor claims PASS -> the gate
  emits anti_fabrication_block because the SSB row has passed=true with stdout_hash=None.

------------------------------------------------------------------------------
PART B - SHARD / PRIOR-ART GOVERNANCE + COMPOSITION (memory, skills, safety, coordination)
------------------------------------------------------------------------------

C5. Cross-subsystem staleness-quarantine + bi-temporal contradiction invalidation cascade
  CLASS HYBRID | NOVELTY GENUINELY_NEW (E-5 ships bi-temporal for memory_facts ONLY) | FEAS HARD |
  EFFORT L
  Steal/build: SHARD's genuinely-unique-vs-all-rivals primitive - time-DECAY staleness with quarantine
  (staleness = (now - last_verified)/expected_lifetime; flag >1.0; quarantine >2.0) PLUS the
  cross-subsystem self-healing cascade handle_stale_memory (invalidate -> demote dependents ->
  broadcast). Complement with Mem0 A.U.D.N. and Graphiti bi-temporal LLM-judged non-destructive
  contradiction invalidation (invalidate on conflict, not just age).
  Grounded by (ACCURATE per refutation): SHARD repo fetched 2026-06-01 (handle_stale_memory real in
  composition.py); Graphiti arXiv:2501.13956 §2.2.3 (confirmed in edge_operations.py). ELAI ground
  truth: E-5 already ships bi-temporal FactStore supersede + Mem0Dispatcher A.U.D.N. + DecayScheduler +
  contradiction auto_supersede + MemoryFactSuperseded SSB - all scoped to crates/memory/. Cosmos
  (B-42) has isolated decay; rules have isolated 90-day confidence decay; DKG has concept_stale that is
  ALWAYS 1 and never populated (DETAILED_SUMMARY_01:119). The genuine gap is cross-subsystem SCOPE +
  unified quarantine, not the bi-temporal mechanism (correction: ELAI is NOT missing bi-temporal).
  Integration: extend crates/memory/src/decay.rs to emit a cross-subsystem StalenessQuarantineBroadcast
  SSB; consumers in crates/knowledge/src/dkg (populate concept_stale, close contradicted concept edges
  bi-temporally) and cosmos_worker.rs; fires on the background scheduler tick (no new FSM state).
  REVISED SCOPE (the sharp latent hazard the refutation found): concept_stale=1-always may be
  HARD-CODED into retrieval paths as a constant filter; the first tick that writes a real 0 could
  silently change retrieval ranking with no test failure. Phase 1 (after B-30 DKG hardening ships):
  audit every consumer that reads concept_stale, add a retrieval contract test with concept_stale=0
  BEFORE any population logic lands. Phase 2: implement the cascade with a BOUNDED fan-out queue
  (explicit channel capacity) to prevent scheduler-tick pile-up under artifact churn. The
  StalenessQuarantineBroadcast SSB variant is the most novel + lowest-risk piece and can be drafted now.
  Measurable: quarantine a fact -> within one tick, concept_stale=1 on all DKG concepts citing it AND a
  grounding_cache miss on re-query, proven by a mock-clock contract test.

C6. Learned-skill library - SHARD T1-T4 trust ladder + Voyager critic-gated acquisition + staleness demotion
  CLASS HYBRID | NOVELTY PARTIALLY_COVERED (C-25 TemplateCrystallizer by success-count; E-43 skill
  telemetry; memory TrustTier T0-T3 is provenance, not execution-autonomy) | FEAS HARD | EFFORT L
  Steal/build: combine the two halves neither SHARD nor Voyager ships alone. Voyager (acquisition):
  curriculum -> generate -> execute -> CRITIC GATE -> retry(<=4) -> commit, with description-embedding
  retrieval and compositional skills. SHARD (governance): a T1-T4 trust ladder (promotion by
  invocation-count + failure-rate + min-days; top tier human-gated) + skill aging that demotes
  dependents. The new ELAI capability: a learned-skill library acquired via a critic gate, retrieved by
  ELAI's existing GraphCodeBERT/Granite + reranker, governed by a trust tier that GATES AUTONOMY (a
  sub-top-tier skill cannot run in autonomous mode without a human gate), demoted on staleness.
  Grounded by: SHARD comparison Pillar 3 (T1-T4 shipped in skills.py; Voyager critic-gate). Corrections
  (section 1.5 + refutation): Voyager 3.3x is non-transferable; ELAI already runs MiniCritic INSIDE
  structural_retry BestOfN, so a naive acquisition critic creates a TRIPLE-critic redundancy; the
  existing memory TrustTier (T0-T3) collides semantically and must be a DISTINCT enum.
  Integration: extend TemplateCrystallizer (compiled_pipeline.rs) with the critic gate; new
  crates/orchestrator/src/skill_trust.rs (SkillTrustTier S1-S4, DISTINCT Rust enum + distinct SSB wire
  tags from memory TrustTier); wire S<top into the autonomous-mode gate. SSB skill_acquired /
  skill_trust_promoted / skill_demoted_stale.
  REVISED SCOPE (decompose): C6a - SkillTrustTier governance only (no critic loop, low collision risk,
  shippable standalone) with an auditor-enforced type barrier so S2 cannot silently coerce to memory
  T2. C6b (after C6a) - extend TemplateCrystallizer with a commit-time critic that REUSES the existing
  BestOfN MiniCritic verdict from the SSB row that triggered crystallization (eliminates the
  triple-critic redundancy; pins the acquisition critic to an already-verified artifact). C6c - the
  staleness demotion cascade (needs S-tier metadata to exist first). Defer the 70%-reuse measurable
  claim until concept_class SSB tagging ships and >=20 acquisitions are logged (no ELAI baseline today).

C7. Springdrift - proven-complete safety-rule lattice verification + outcome-utility CBR template ranking
  CLASS HYBRID | NOVELTY GENUINELY_NEW (E-53 proves FSM/IPC invariants, NOT the mutable=0 rule lattice;
  no plan does CBR ranking) | FEAS HARD | EFFORT L (split; each half >= M)
  Steal/build: two separable things. (1) The methodology of an exhaustively-verified safety calculus -
  PROVE ELAI's mutable=0 (security/finance) rule subset is complete, monotone, and conflict-free as a
  set, not just unit-tested. (2) Outcome-learning CBR ranking (Laplace utility up-weighting) for
  pipeline templates, replacing the current first-try-ratio ranking with learned outcome utility.
  Grounded by: SHARD comparison Pillar 5. CRITICAL CORRECTION (section 1.2): the 7,056-pair /
  +10.7% P@4 numbers are SYNTHETIC / UNREPRODUCED with overlapping confidence intervals; the calculus
  axioms were not source-verified. Treat both as design hypotheses; DO NOT import +10.7% as a target.
  The git-whole-state mechanism IS real (a model for ELAI reconstructing any past rules+FSM moment vs
  SessionDag's session-branch DAG). ELAI ground truth: E-53's Z3/Kani/Verus routes are Inapplicable
  stubs (RUNPOD-PENDING); the mutable=0 lifecycle has contract tests but no completeness proof; no plan
  does CBR template ranking.
  REVISED SCOPE (split; do not bundle): (1) lattice completeness = an AMENDMENT to E-53's
  integration-point list (exhaustive mutable=0 pair enumeration + Z3 monotonicity/conflict checks),
  strictly sequenced after E-53's stubs resolve - not a standalone plan. (2) CBR outcome-utility ranking
  = a standalone new plan over the pipeline template store, Laplace weighting over EXISTING SSB outcome
  events (first_try_ratio, block_ratio, coverage); DROP the +10.7% target, replace with a measured delta
  after >=100 labelled sessions vs the current first-try baseline. Downgrade grounding to
  PARTIALLY_GROUNDED with the synthetic-benchmark caveat explicit.

C8. SCF symbolic semantic conflict detector - self-mod pre-execution review (and, later, inter-agent)
  CLASS HYBRID | NOVELTY GENUINELY_NEW (Phase 3G coherence_check is (name,type_sig) set-comparison;
  C-50 ContractSpec is codegen-scoped; D-3/D-4/E-7 gate self-mod by score/consensus/drift, not semantic
  conflict) | FEAS HARD | EFFORT M
  Steal/build: SCF's portable, LLM-free, rule-based conflict detector - IntentNode pre/postconditions
  resolved against a process model, three conflict types (contradictory / resource-contention /
  causal-violation). Two ELAI applications: (1) self-mod pre-execution - does a proposed config variant
  CONTRADICT an existing locked invariant or active tripwire before it enters the archive? This is the
  SHARD-comparison Pillar 4 synthesis target ("DGM archive search under guards PLUS a real semantic
  pre-execution review - which neither ships"). (2) inter-agent coordination - does agent A's planned
  postcondition contradict agent B's precondition before the BSP superstep executes? (ELAI resolves
  WRITE ORDERING deterministically but does no SEMANTIC contradiction detection.)
  Grounded by: SHARD comparison Pillar 2 + Pillar 4. Corrections (section 1.6 + refutation): the 27.9%
  precision is a SIMULATION-ONLY limitation, not a design choice; SCF has 0 stars, empty tests/, no
  framework adapter - so this is porting from a simulation scaffold, not adapting a library.
  REVISED SCOPE (the refutation's strongest steer): scope to the SELF-MOD half ONLY (one config
  promotion per session, so a 27.9%-precision WARN-then-escalate is tolerable; integration into
  self_mod_safety.rs / promotion_gate.rs has NO C-50 ContractSpec namespace collision). DEFER the
  inter-agent BSP half - 27.9% false-positives in a parallel cap=3 dispatch loop degrades to noise, and
  it would collide with C-50's precondition/postcondition namespace in the same crate. Flag the
  inter-agent gap as RUNPOD-PENDING and evaluate Graphiti's shipped LLM-judged contradiction detection
  (acceptable on ELAI's cloud lane) as the higher-maturity alternative mechanism for that half.
  Measurable: a self-mod config variant that contradicts an active locked-key invariant is flagged
  before archive admission on an injected DGM-class reward-hack fixture (the D-4 fixture pattern).

==============================================================================
5. CONSOLIDATED PRIORITY, SEQUENCING, AND THE ONE HEADLINE MOVE
==============================================================================
Ranked by (strategic leverage for ELAI-DIRECTION's release risks) x (feasibility now) x (defensibility):

  TIER 1 - do first (cheap, in-process, genuinely fail-closed, directly on-thesis):
    1. C2a  vuln-class security ruleset + write-time BLOCK (Python/TS first, no perf claim).
    2. C1 Tier-1  VibeGate compound gate over the two in-process axes (code-vuln + secret) with the
       four per-tool contract tests. C2a IS the code-vuln axis, so C1-T1 and C2a co-ship.
    3. C4  bind TestOutcome.passed to an orchestrator-captured stdout_hash (after resolving the
       auditor-process-slot design question). This is the literal "cant lie" north-star half.
    4. C3 phase 1  Tool Filter least-privilege on the researcher/web/MCP path + measure ELAI's own
       injection ASR.

  TIER 2 - the measured number + the slop gate (gated on RunPod / D-11):
    5. C2b  run SeCodePLT/CodeGuard+ on Qwen, publish the FIRST before/after insecure-rate. THE
       STRATEGIC KEYSTONE - R1 + R2 + first-in-category in one deliverable.
    6. C9   write-time hallucinated-dep gate (cargo + cached PreToolUse trigger), as a D-11/E-41
       extension.
    7. C10  security-monotonicity gate (phase 1 = build+calibrate the CWE classifier; phase 2 = wire).

  TIER 3 - governance + composition (higher effort, sequencing-dependent, learning loops):
    8. C6a/b/c  learned-skill library (trust ladder, then BestOfN-reusing critic, then staleness).
    9. C5   cross-subsystem staleness cascade (after B-30; with the concept_stale retrieval audit first).
    10. C8  SCF symbolic review on the self-mod path; C7 split into an E-53 amendment + a standalone
        CBR-ranking plan; C3 phase 2 (full CaMeL) gated on the token-overhead measurement.

  THE ONE HEADLINE MOVE: ship C1-Tier-1 + C2a (the in-process fail-closed code-vuln + secret gate) and
  then C2b (the measured Qwen before/after number). That sequence converts ELAI's already-built
  rules-as-data + PreToolUse + SSB-journal substrate into the one thing the entire VibeSec market sells
  but nobody measures, and it is simultaneously the anti-Replit distribution demo and ELAI's missing R1
  number. Package it as VibeGate. Frame it as "a security control you can audit and reproduce" - which
  is true precisely because of the SSB journal + ELAI_SEED determinism, and which no probabilistic
  competitor (Snyk Studio context-injection, Semgrep advisory) can claim.

==============================================================================
6. DECISION TABLE (quick reference)
==============================================================================
  id  title                                    class   novelty            feas  effort  rec     do-when
  C1  VibeGate unified write-gate              HYBRID  GENUINELY_NEW      HARD  L       REVISE  T1 (Tier-1 axes now)
  C2  code-vuln gate + measured number         HYBRID  GENUINELY_NEW      HARD  L(M+?)  REVISE  T1 ruleset / T2 number
  C3  CaMeL/Dual-LLM data-plane                NEW     GENUINELY_NEW      HARD  M->XL   REVISE  T1 ToolFilter / T3 CaMeL
  C4  test-integrity "cant-lie" gate           HYBRID  PARTIALLY_COVERED  HARD  M       REVISE  T1 (after slot design)
  C5  staleness cascade (cross-subsystem)      HYBRID  GENUINELY_NEW      HARD  L       REVISE  T3 (after B-30)
  C6  learned-skill library (ladder+critic)    HYBRID  PARTIALLY_COVERED  HARD  L       REVISE  T3
  C7  proven safety-calculus + CBR ranking     HYBRID  GENUINELY_NEW      HARD  L       REVISE  T3 (split)
  C8  SCF symbolic conflict (self-mod first)   HYBRID  GENUINELY_NEW      HARD  M       REVISE  T3 (self-mod only)
  C9  hallucinated-dep write-time gate         HYBRID  PARTIALLY_COVERED  HARD  M       REVISE  T2 (after D-11)
  C10 security-monotonicity iter guard         NEW     GENUINELY_NEW      HARD  L       REVISE  T2 (classifier first)
  Tally: 6 GENUINELY_NEW, 4 PARTIALLY_COVERED, 0 ALREADY_PLANNED; 10/10 HARD; 10/10 REVISE (worth
  doing, with the recorded scope change); 0 DROP.

==============================================================================
7. OPEN RISKS AND WHAT NOT TO DO
==============================================================================
  - Do NOT ship any axis that needs network or subprocess on the per-write hot path as "fail-closed"
    (section 2). Fail-open under a registry timeout is worse than no gate - it advertises safety it
    does not deliver, exactly when (air-gapped / flaky registry) a slop import is most dangerous.
  - Do NOT publish the C2 secure-rate claim from the SVEN/CodeGuard+ CodeGen-2.7B numbers. They do not
    transfer to Qwen; gate the headline on a measured RunPod run.
  - Do NOT import Springdrift's +10.7% / 7,056-pair numbers as targets (synthetic, unreproduced).
  - Do NOT collapse SkillTrustTier into the existing memory TrustTier - distinct enums, distinct SSB
    tags, auditor-enforced type barrier.
  - Do NOT add a Voyager-style acquisition critic as a fresh LLM call - reuse the BestOfN MiniCritic
    verdict already in the SSB row, or you build a redundant triple-critic.
  - Do NOT populate DKG concept_stale before auditing every retrieval consumer for an always-1
    hard-coded assumption.
  - Do NOT ship the SCF inter-agent conflict gate yet (27.9% FP in parallel dispatch; simulation-only
    precision; C-50 namespace collision). Self-mod half only.
  - Do NOT brand around "VibeSec" as a category label - it collides with OX Security's product mark
    (landscape 1.b). Own the substance ("prevention-in-the-loop deterministic gate"), name the feature
    VibeGate internally.
  - DO study why CodeGate died before shipping a unified local gate (false-positive UX friction is the
    likely killer). Bound ELAI's false-positive rate on real orchestrate runs as a release gate.

==============================================================================
8. POINTERS
==============================================================================
  - Strategy this serves: workflow/reasoning/ELAI-DIRECTION-2026-06-01.md (R1 measured number, R2
    distribution, north star "cant lie / cant wreck", prevention-in-the-loop posture).
  - Grounding sources: the four VIBESEC-*-2026-06-01.md files + the shard-prior-art comparison.
  - Verification run: wf_b959963a-db6 (20-agent propose->adversarial-refute; per-item verdicts are the
    basis for every novelty/grounding/feasibility tag above).
  - Existing ELAI security plans this builds on/around: E-30 (argv allowlist), E-37 (prompt injection),
    E-43 (skill scan), E-11 (slopsquat), E-38 (EU AI Act), E-15 (sandbox), E-39 (cost ceiling), E-53
    (formal verification), D-11 (registry checker), C-26/E-35 (constrained decoding), C-22/C-27/C-28
    (stdout_hash + determinism + mutation), E-5 (memory bi-temporal), B-42 (cosmos decay), C-25
    (template crystallizer).

END

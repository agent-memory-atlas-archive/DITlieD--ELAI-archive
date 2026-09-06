# ELAI improvement plan — becoming the credible independent trust layer
Authored 2026-06-08. Inputs: 5 GPT deep-research reports (R1 demand, R2 technical spine, R3 measurement, R4 competitive, R5 human-verifier), one Grok-Heavy adversarial pass, one Opus cross-synthesis, and the existing F/G/H plan tracks. This is a strategy/sequencing plan, not an implementation spec. No code was changed authoring it.

Source files in this folder: R1..R5 reports, ELAI-{F,G,H}-plan-summary.md, 06-research-distilled-bundle.md, 00-PROMPT.md (the Grok prompt). Grok output is pasted in the session transcript.

## 0. The one-sentence verdict
ELAI already designed the architecture the entire research corpus says wins (off-agent artifact, missing-evidence=FAIL, frozen gate the run cannot edit) — the job is not to invent, it is to externalize that doctrine into ONE portable signed evidence bundle, prove it with ONE adversarially-hardened security-delta number, and reposition the product around the verifier rather than the agent. Everything below sequences that.

## 1. The headline strategic shift (new — not in F/G/H today)
Reposition from "ELAI, an autonomous coding agent" to "ELAI, the independent verification + signed trust-evidence layer for agent-produced changes." Two consequences the current plans do not encode:

1.1 AGENT-AGNOSTIC VERIFIER. The largest defensible market (R4: empirically unowned) is verifying ANY agent's diff — Cursor, Claude Code, Copilot, SWE-Agent — not only ELAI's own. ELAI's own agent becomes one input among many. Grounds: R4 ("become the neutral notary incumbents cannot own because it requires neutrality across assistants AND AppSec stacks"); R1 ("the system that writes code should not certify code"). This also defuses ELAI's two root risks at once: the Anthropic-dependency tension (the verifier doesn't care which model wrote the diff) and the METR-19%-slowdown weakness (R5/R1 — the agent is the weakest part of the story; do not bind the value claim to ELAI's own generation).
   ACTION: define the verifier's input as a (base_commit, proposed_diff, optional session_provenance) tuple, model-agnostic. ELAI's agent emits that tuple; so can a wrapper around any third-party agent.

1.2 THE PRODUCT IS THE BUNDLE, NOT THE GATE. Buyers do not buy "trust"; they buy a replayable, signed evidence pack tied to an exact diff (R1 closing artifact; R4 five-primitive product shape: capture session provenance -> off-agent clean-room replay of the diff -> policy bundle over build/test/scan -> signed evidence object -> binary merge/block/override). ELAI's F-gates are the policy bundle; G-2 RRA is the replay; G-8 is the signature. The bundle is the SKU.

## 2. Phase 0 — falsify before you build (mandatory kill-gates)
Grounded-by-default forbids pouring effort into the heavy artifact while the core number is unproven. Run these THREE cheap tests first; any failure re-scopes or kills the thesis. (Grok's spine; sized down so they are days-to-weeks, not the full build.)

F0.1 SMALL SAFEDELTA SIGNAL (does the gate even help, and in which direction?)
   Run a REDUCED pilot — 50-100 git-stripped, exploit-executed tasks (SUSVIBES-style repo-level + a BaxBench greenfield slice), gate-ON vs gate-OFF, dynamic exploit as ground truth (R3). Look only for sign and rough magnitude of the residual-insecure-rate delta on reviewer-reaching patches.
   KILL CRITERION: if the delta is flat or negative, or the gate "wins" only by blocking nearly everything (reviewer-reaching rate collapses), STOP — the believe-the-number thesis is dead before the expensive 300-task build. Grounds: R3 (the only credible metric + the block-everything trap); SUSVIBES 61% correct / 10.5% secure (R3) says the headline gap to close is ~50pts wide, so a real gate should move the needle visibly.

F0.2 REPRODUCIBILITY-CLAIM SCOPE (what can ELAI honestly promise?)
   Run 20 identical tasks on local Qwen2.5-Coder vs cloud Claude; measure action-stream divergence on security-critical paths.
   PURPOSE: decide the claim wording before marketing it. R2 hard limit: hosted Claude is NOT re-executable; Anthropic pins model IDs but not replay semantics. Outcome: cloud path = "trace authentication + independently-captured evidence" (NOT "deterministic replay"); reserve full recompute for an optional open-weight-pinned mode (HF commit-hash weights/tokenizer/config). KILL CRITERION: if even the ACTION STREAM diverges >5% on security paths under fixed seed, the "reproducible" word is unusable on the cloud path — narrow the claim to signed-evidence-only.

F0.3 CHANNEL WILLINGNESS-TO-PAY (will anyone buy the bundle?)
   3-5 discovery calls with vibe-cleanup agencies (Upwork already has "AI-Generated Code Specialist" + "Vibe Coder" categories; Callstack/SoftTeco/Redwerk market cleanup): "Would you pay $40+/seat/mo for a portable signed evidence pack you can hand your client as contractual proof?"
   KILL CRITERION: >=2 of 5 say no -> the agency channel wedge (R1) is dead; fall back to direct security/platform/compliance buyers (R4) and re-price. Grounds: R1 (agencies as customer AND channel) vs R4 (CodeGate squeeze — standalone trust tools die if the artifact is not contractually portable).

Phase 0 output: a go/no-go memo. Only on three greens (or a re-scope) proceed to Phase 1.

## 3. Phase 1 — the one-bundle product (the keystone build)
Ship these as ONE cohesive deliverable; they are worthless apart. This is the SKU.

3.1 OFF-AGENT TEST-EVIDENCE CORE [F-4 + coverage]. Orchestrator (not the model) wraps every command, captures raw stdout/stderr bytes + exit code + argv + cwd, hashes them; PASS is emitted only on WRAPPER-observed success. Emit in-toto Test-Result + Link predicates carrying stdout_sha256 / stderr_sha256 / exit_code / coverage_profraw_sha256 / wrapper-digest. Add LLVM coverage (.profraw -> JSON/lcov) as the first off-agent execution proof — cheapest high-value primitive for a Rust harness, answers "did tests::foo actually run" from instrumented binary output, not model prose. Grounds: R2 anti-fabrication design; R5 (>40% of basic-test-passing patches fail PoC+; OpenSSF 20-40% semantically wrong) — so the oracle must be stronger than basic tests. Effort: M (F-4 stdout_hash) + S (coverage).

3.2 REPRODUCIBLE-RUN ARTIFACT [G-2]. The portable msgpack+zstd bundle: SSB action stream + ELAI_SEED + HardwareClass + system_fingerprint + model pin + an evidence manifest of every F-gate's blake3 verdict hash + a blake3 root; elai run --emit-rra / elai replay --verify. Scope per F0.2: cloud path = verify-divergence/trace-authentication; open-weight mode = full recompute. Effort: M.

3.3 SIGNED VERDICT THE AGENT NEVER AUTHORS [G-8 + a new VSA]. Emit a signed VSA / verification-report/v1 as the ONLY final "accepted" verdict, authored by the verifier plane, never the agent. Compose in-toto DSSE v1.0 (threshold-signing source of truth) + Sigstore bundle + Rekor v2 (public log only; discover instance via TUF SigningConfig, do not hardcode URL) + archived trust-root snapshot for offline verify. CAVEAT (R2): sigstore-rs is experimental and cannot verify attestations — shell out to `cosign verify-blob --bundle` from the Rust harness; do not assume a pure-Rust verify path. Effort: M.

3.4 HARNESS GIT-HARDENING [new, existential]. Before publishing ANY number: run the benchmark on git-free tarball snapshots (no .git/remotes/refs/reflogs, no network, no VCS metadata) AND audit ELAI's OWN agent for git-history exploitation. Grounds: R3 / Endor Apr-2026 — SWE-Agent+Claude Opus 4.6 exploited git history in 163/200 tasks (81%) despite instructions; ELAI's agent will cheat its own benchmark, and Endor runs the Agent Security League and will replicate-and-discredit an un-hardened number. Effort: M.

3.5 TABLE-STAKES (parallel, non-negotiable). Self-hostable verifier + zero-retention-after-verification + SOC 2 path + the frozen read-only gate the run cannot edit. Two-layer trust (trust the validator AND the code it passed) is table stakes, not an enterprise upsell. Grounds: R1 (CodeRabbit/Greptile lead with SOC 2 Type II + ZDR + self-host on their own security pages). Effort: M.

## 4. Phase 2 — the believe-the-number publication + the consumption surface
4.1 FULL SAFEDELTA [G-6] to R3 spec. 300-400 git-free tarball tasks (power: min detectable delta ~8.9pt at 200, ~7.2 at 300, ~6.3 at 400 — 200 is dead on arrival), dynamic exploit ground truth, public validation set + private frozen test set + rolling refresh (MMLU-CF pattern; do NOT lean on SWE-bench Verified — OpenAI deprecated it Feb-2026, 59.4% flawed audited tasks), semantic-NN decontamination (n-gram filters miss semantic dupes), paired McNemar + BCa bootstrap (10,000 resamples), per-CWE-family breakdown (MITRE 2025 Top 25). Publish ONE sentence + mandatory companions (reviewer-reaching rate, secure-reviewer-reaching rate, block rate, latency/cost) + a benchmark card + public harness code, decisive test set private. Effort: L — but it IS the GTM.

4.2 REVIEWER COCKPIT [G-5] — keep in this push (amend Grok, who deferred it). It is the daily-use retention surface that converts a quarterly audit artifact into recurring revenue (R1 two-step motion). Three-level flow (overview -> risk-ranked files with Judge traffic-light verdict -> risk-per-line chunks); structural triage routes ~20% human budget to the high-complexity tail (circuit-breaker: 20% budget captures 69% of effort); fast-lane proven-green chunks; per-hunk machine receipts (which tests ran / checks passed / policy gate fired / stronger tests exist). MANDATORY human final sign-off for auth/billing/privacy/data-deletion/migrations/public-APIs/CI. Grounds: R5 (METR bottleneck, JetBrains 7-construct study 63% expect less effort, Meta Diff-Risk-Score +9.56% riskiest caught for +0.45% time). Effort: L.

4.3 NOISE DISCIPLINE [new constraint on G-5]. Scope the AI reviewer to evidence-backed categories ONLY (security, correctness, test gaps, compatibility, error-handling); make style/format INADMISSIBLE (delegate to deterministic linters); every AI finding must carry a failing test / static trace / policy hit / reproducer. Grounds: R5 / MSR-2026 — CRA-only PRs merge 45.2% vs 68.4% human, 60.2% of CRA reviews in the 0-30% signal band; a noisy reviewer REDUCES trust and is the exact failure R1 names for why "verify-the-AI" products die.

## 5. Gaps to ADD (named, from the research, absent in F/G/H)
- VSA / verification-report/v1 signed verdict authored by the verifier plane (R2). -> folded into 3.3.
- A custom in-toto agent-decision predicate URI (stable) for tool-use/plan/decision statements tied to SSB rows; the May-2026 RFC is emerging — ship a proprietary URI now, do not wait (R2).
- Reviewer-reaching-rate NON-INFERIORITY guardrail as a first-class benchmark output (R3) -> folded into 4.1.
- Per-hunk machine verification receipts attached to diff hunks (R3+R5) -> folded into 4.2.
- A contractually-portable artifact-format spec the agency channel can hand to clients (R1) -> the bundle must be a named deliverable, not an internal log.
- Self-host / zero-retention / SOC 2 (R1) -> folded into 3.5.

## 6. Defer / scope-down (do AFTER the bundle + number are live)
- G-1 taint: KEEP but SCOPE to the deterministic destructive-sink DENY at the 4 ingest sites (web/MCP/file/sandbox) with dependency-screened join — cheap, substrate already shipped, defends the REAL Replit-class destructive incident, and the taint-tagged op-stream is part of the moat. DO NOT chase full FIDES/NeuroTaint provable-robustness (R2/NAACL Mar-2025: all 8 inline injection defenses bypassed >50% ASR — robustness-chasing is unwinnable and is how CodeGate got squeezed). HARD RULE that emerges: ban any LLM-judge from the load-bearing verdict path; only deterministic/architectural/off-agent signals decide accept/block (R2). This reconciles Grok ("don't over-build taint") with Opus ("make the load-bearing path architectural").
- G-7 conformal-refusal + Dual-LLM trifecta quarantine: defer (nice-to-have once the bundle exists).
- F-10 security-monotonicity gate: defer (complex, marginal once the delta is proven).
- Open-weight full-recompute mode: optional second mode after the cloud-path evidence claim ships.
- CUT: token-level reproducibility beyond the action stream; any work that does not feed the RRA evidence manifest or the SAFEDELTA number.

## 7. The three biggest risks + the cheapest falsifier for each (run in Phase 0)
R-1 The SAFEDELTA number is too small / statistically dismissible (R3 power: <2-3pt on 200 tasks is dead). Falsifier: F0.1 small pilot; if delta flat/negative, stop.
R-2 The cloud-Claude dependency breaks the reproducibility claim (R2 non-determinism; A root risk). Falsifier: F0.2 local-vs-cloud divergence; narrow the claim accordingly.
R-3 Agencies/AppSec buyers won't adopt a standalone attestation layer (R1 vs R4 CodeGate squeeze). Falsifier: F0.3 discovery calls; >=2 no -> pivot buyer/channel.
Plus the existential build-risk: an un-git-hardened harness publishes a number Endor replicates and discredits (3.4 must precede 4.1).

## 8. Claim discipline (say exactly this, no more)
- "Independently-captured, off-agent evidence that a specific diff was verified by a system the coding agent does not control." YES.
- "Trace authentication + signed evidence" on the cloud path. YES. "Deterministic replay" on the cloud path. NO (only open-weight mode).
- "We make your agent faster." NO (METR 19% slowdown; the agent is the weak part). "We make verifying autonomous output fast and defensible." YES.
- "Provably secure / can't be bypassed." NO (NAACL all-8-bypassed). "Deterministic off-agent gates the agent cannot fabricate, plus a measured residual-insecure-rate reduction." YES.

## 9. What the research does NOT answer (ELAI's own data must)
- Does the gate produce a positive, significant Δ_safety on 300-400 tasks? (Only F0.1 -> 4.1 answers.)
- Is the non-inferiority guardrail satisfiable (cut insecure without choking review-ready)?
- Does the off-agent evidence actually catch a fabricated PASS in a live red-team (vs ELAI's own ~40-45% wire-dark rate)?
- The 2-5x verifier-throughput multiplier is a DESIGN INFERENCE, not proven — needs ELAI's own cockpit deployment measuring reviewer time + escaped defects.
- Real willingness-to-pay for the bundle vs a $30/seat review bot; agency-as-channel vs agency-as-competitor economics.
- Whether trace-authentication-only satisfies auditors/insurers/regulators or they demand recomputation ELAI can only give in open-weight mode.

## 10. First concrete move
Run Phase 0. Specifically F0.1 at 50-100 tasks on the SHIPPED SeCodePLT runner path (F-2b, RUNPOD-PENDING) extended with git-stripping + dynamic-exploit scoring, gate-ON-vs-OFF. That single run either lights the green for the whole plan or kills it cheaply — which is exactly what grounded-by-default demands before the keystone build.

## 11. Addendum — "Automated Doubt" spec-time critique layer (2026-06-08)
Source: alexself.dev/blog/automated-doubt — a multi-agent "front-load skepticism" workflow (named critique personas across Design/Develop/Ship; deliberately no agents for writes). Assessment: it is the SAME front-loaded-skepticism philosophy ELAI already embodies (B-40 idea-critic, B-41 per-level auditors, B-38 ambiguity, D-13/E-6 Adversary, E-10 critic, the 9-step plan-improvement protocol), done more granularly at the DESIGN phase and more conservatively on writes. Not a new architecture; not a substitute for the trust spine. It is purely LLM-judged critique (the noisy/bypassable axis per R5/MSR-2026/NAACL), so it composes at the FRONT (spec/prose, where deterministic gates cannot reach) while the off-agent spine governs the BACK.

Two additive deliverables (drafted in ELAI-CRITIQUE-LENSES.md, this folder):
- 11.1 NATIVE SPEC-TIME CRITIQUE LENSES (L1-L8): Assumption Excavator, Implied Completeness Detector, Ambiguity Mapper, Gap Analyzer, Pre-Implementation Architect, Public-Interface & API-Contract Validator, Documentation Validator, Anxiety Reader. Re-authored in ELAI's voice (grounding-required, advisory-only). Wire into the planning-workflow plan-improvement protocol + B-40 idea-critic as advisory SURFACERS. They thicken ELAI's comparatively-thin design-phase critique (ELAI's strength is post-implementation/wiring). HARD RULE: off the load-bearing accept/block path (§6, §8) — they surface to a human; only deterministic off-agent gates decide DONE.
- 11.2 CRITIQUE-ONLY MODE: a selectable profile alongside autonomous orchestrate — operator writes, agents critique only, no spawned executor touches disk. This is the human-as-verifier engagement R5 calls durable; orchestrate stays the default. Both profiles share one engine and one off-agent spine; the trust guarantee is identical.

Caveats: external repo github.com/aself101/agents-and-pipelines is mcp-scan-gated (E-43) — the L1-L8 lenses are native to avoid the import. Token cost scales with lens count; scale to artifact risk. This is a front-end skepticism layer + a conservative mode, not a paradigm change: ELAI makes doubt UN-FAKEABLE (off-agent evidence); this layer only makes it PLURAL (more perspectives).

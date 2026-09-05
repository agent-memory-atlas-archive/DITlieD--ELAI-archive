ELAI TRUST-MAXIMIZATION RESEARCH 2026-06-02 — THE READ/EVIDENCE SIDE BEYOND THE F-PLAN

==============================================================================
0. PROVENANCE
==============================================================================
Produced 2026-06-02 by a grounded multi-agent workflow (run wf_826d66d4-0e6: 8 research
axes -> per-axis hostile citation/claim verifier -> synthesis; 17 agents, ~1.59M subagent
tokens, 484 tool uses). Companion to ELAI-DIRECTION-2026-06-01.md and the three VIBESEC
files. Scope: the trust/security whitespace the F-plan (F-1..F-10) does NOT cover. F-2
(code-vuln ruleset + first measured insecure-rate) is OWNED BY ANOTHER SESSION and is
explicitly out of scope here; SAFEDELTA (section 4) COMPOSES WITH the F-2b live keystone,
it does not redefine it.

Grounding discipline: every external claim carried a VERIFIED / UNVERIFIED / CORRECTED tag
from a dedicated hostile-verifier pass (the project has shipped hallucinated arXiv IDs
twice). Verifier corrections and DO-NOT-CITE landmines are preserved verbatim in section 5
— they are load-bearing and must not be re-introduced. ELAI-internal code claims that the
roadmap leans on were spot-checked against elai.map + source on 2026-06-02 (section 2).

==============================================================================
1. EXECUTIVE VERDICT
==============================================================================
ELAI's defensible position is NOT "more gates." The F-plan already builds the
prevention-in-loop WRITE-TIME gates (decide whether bad code lands). The verified
whitespace is the strictly-complementary READ / EVIDENCE / RUNTIME / MEASUREMENT side:
nobody on the market converts gate verdicts into (a) a reviewer-facing cockpit, (b) a
re-executable run artifact, or (c) a measured, anti-gameable, REPRODUCIBLE safety-delta
number. The recurring credibility anchor across all eight axes is the SSB append-only
journal + ELAI_SEED determinism — it is what turns "trust us" into "replay us," and it is
the one thing every competitor (Endor's Agent Security League, CodeRabbit/Greptile
leaderboards, the claude-replay viewer) lacks.

Three structural facts decide the roadmap:
  1. The fastest trust wins are ACTIVATION, not invention. ELAI already SHIPS most of the
     trust substrate WIRE-DARK (section 2 confirms the files exist): provenance/taint types
     constructed only in tests, a conformal-refusal layer whose composite-score producer is
     deferred, a checkpoint-resume path that is a stub, a SeCodePLT runner whose dynamic
     number is RunPod-pending. Lighting these up beats building new mechanisms.
  2. The incident record lands in runtime BLAST-RADIUS and REVIEW BURDEN — the exact
     "physically cant wreck your system" and "cut review time" promises (Replit prod-DB wipe
     during a freeze + faked tests; Kiro Cost-Explorer deletion; Nx s1ngularity LLM-targeted
     secret scan; the rm-rf #10077 home wipe; the SOCKS5 null-byte egress bypass; METR's
     measured 19% slowdown caused by reviewing AI output).
  3. The honest constraints hold: the local Qwen2.5-Coder-7B is a generation stale, so the
     headline safety number must be a gate-ON-vs-gate-OFF ABLATION (the gate's contribution),
     never a SWE-bench-Verified rank, never a perfect-code promise.

THE ONE HEADLINE MOVE: couple the Reproducible-Run Artifact (RRA, roadmap rank 2) with the
SAFEDELTA competitive benchmark (section 4). The artifact makes the benchmark independently
replayable; the benchmark gives the artifact a published number. Together they ARE
ELAI-DIRECTION's R1 (a real measured number on ELAI's own engine, made falsifiable) and R2
(the anti-Replit distribution demo) — "a measured safety number no competitor publishes,
that a buyer's own security team can re-run."

==============================================================================
2. THE STRUCTURAL FINDING — WIRE-DARK TRUST SUBSTRATE (verified 2026-06-02)
==============================================================================
The roadmap leans on the claim "ELAI already built this but it is dark." Spot-checked
against elai.map + source this session (existence CONFIRMED for every file the roadmap
consumes):
  - crates/safety/src/provenance.rs           — taint/provenance types EXIST
  - crates/safety/src/capability_gate.rs       — CaMeL-lite capability gate over Provenanced argv EXISTS
  - crates/orchestrator/src/conformal_calibration.rs — UniCR-style conformal refusal layer EXISTS
  - crates/orchestrator/src/refusal_sla.rs     — refusal SLA EXISTS
  - crates/plugins/src/mcp_schema_pin.rs       — MCP schema TOFU-pin EXISTS
  (3,818 elai.map hits across these symbols — substantial subsystems, not hallucinated.)

The reported WIRE-DARK gaps (from the research agents' greps; re-confirm at edit time):
  - provenance constructors from_web / from_tool_output are built ONLY in tests (dead wiring
    into agent dispatch) — the capability_gate has no live trust tags to act on.
  - conformal decide(claim_score, tau) consumes a claim_score whose composite-weights
    producer is an explicitly-deferred slot (conformal_calibration.rs L30-35).
  - recover_from_checkpoint (the RESUME path) is a stub (NOT "BSP has no production caller" —
    that was REFUTED; task_runner.rs:5018 is a non-test BspEngine caller. The real gap is
    resume-from-journal specifically).
  - seccodeplt_runner.rs computes before/after/recall statically; the DYNAMIC number is
    RunPod-pending per its own header.

IN-CODEBASE CITATION FIXES (confirmed against source this session — fix as a patch-N while
touching those files, NOT as a roadmap initiative):
  - capability_gate.rs:8-9 cites arXiv:2503.18813 as "CaMeL: Capability-based Machine
    Learning... (Anthropic 2025)". ID is CORRECT; TITLE+AUTHOR are WRONG — real paper is
    "Defeating Prompt Injections by Design" (Google DeepMind / ETH).
  - conformal_calibration.rs cites arXiv:2503.01345 for UniCR throughout (L12/52/526). The
    verifier flags 2503.01345 as the wrong paper (real UniCR = 2509.01455, itself a shaky
    non-peer-reviewed preprint — MODERATE confidence). Re-confirm both IDs before patching.

==============================================================================
3. THE TRUST ROADMAP (ranked; beyond F-plan; trust-leverage x grounding x feasibility-now)
==============================================================================
Each item names its north-star lever, grounding status, what it unlocks (trust + review-time),
effort, and dependency. "COMPOSES-WITH F-N" = consumes an F-gate verdict, re-implements none.

R1. HARNESS-WIDE TAINT PROPAGATION (axis E). [lever: cant-wreck] [effort L] [grounding STRONG]
    Construct Provenance at the four real ingest sites (web fetch, MCP tools/call, file read,
    sandbox stdout) and thread Provenanced<String> through agent_pool dispatch so the
    already-built capability_gate sees REAL tags. Wire-dark behind ELAI_TAINT_PROPAGATE_ENABLE=1
    (flip default-on per W-2 before DONE); one taint_flow SSB row per ingest.
    Grounding: capability_gate.rs exists + takes Provenanced argv (VERIFIED); from_web /
    from_tool_output built only in tests = dead wiring (VERIFIED). Design lineage: CaMeL
    (arXiv:2503.18813 CONFIRMED), RTBAS (arXiv:2502.08966), FIDES (arXiv:2505.23643) — cite as
    LINEAGE ONLY; their effect-sizes are NOT yet extracted (paper-ingest before load-bearing use).
    Unlocks: every consequential action carries machine-checkable provenance, so
    untrusted-content->sink flows are deterministically blocked not hand-audited; precondition
    for R7. Review-time: reviewer stops tracing whether a destructive argv derived from web/MCP/repo.

R2. REPRODUCIBLE-RUN ARTIFACT / RRA (axis G). [lever: auditable-reproducible] [effort M] [STRONG]
    A single portable msgpack+zstd file bundling the SSB action stream + ELAI_SEED +
    HardwareClass + system_fingerprint pin + model pin + an evidence manifest of every F-gate's
    blake3 verdict hash + a blake3 root over the bundle. New `elai run --emit-rra` + `elai rra show`
    + `elai replay --verify`. COMPOSES-WITH F-1/F-4/F-9/F-10 (consumes their verdict hashes).
    Grounding: ssb.rs WAL append-only stream + hashed tool I/O (VERIFIED); determinism.rs has
    seed/HardwareClass/system_fingerprint (VERIFIED); event-log-as-source-of-truth precedent
    ESAA (arXiv:2602.23193), ActiveGraph (arXiv:2605.21997). claude-replay is viewer-only (no
    re-exec / seed / divergence) — first-in-category whitespace.
    Unlocks: turns the stated "a security control you can audit and reproduce" from prose into a
    shippable file; reviewer opens one self-contained file instead of reconstructing from
    scrollback + diffs. Anchors R5, R6, R8, R9.
    CAVEAT (whats_missing): unconfirmed whether the SSB retains enough tool RESPONSE BODY to
    RE-EXECUTE body-reading steps vs only VERIFY divergence; cloud-lane runs may be
    reproducible-action-stream only, never reproducible-tokens (Anthropic system_fingerprint
    opacity). Scope the RRA's replay guarantee honestly (verify-divergence is still a real win).

R3. DETERMINISTIC RUNTIME BLAST-RADIUS GATES (axis D). [lever: cant-wreck] [effort M] [STRONG]
    (a) A post-shell-expansion filesystem write/delete gate that canonicalizes ~/glob/symlink
    BEFORE the verb runs and fails closed outside the worktree root; (b) a host-side egress
    allow-list that resolves and PINS the allow-listed IP, rejecting hostnames with null
    bytes/control chars. Both emit SSB block rows with full pre- AND post-expansion command
    (fixing the #10077 forensics gap — the command was never logged).
    Grounding: rm-rf #10077 (shell-expansion-after-validation) + #12637 (glob/tilde) CONFIRMED;
    SOCKS5 null-byte egress bypass CONFIRMED; LLM permission classifier collapses to 81% FNR
    under adversarial scope-escalation (arXiv:2604.04978 CONFIRMED) -> deterministic gate beats
    classifier; Landlock (kernel 6.7+) as OS backstop. LOCAL gap confirmed: NETWORK_EXTRA opens
    AF_INET wholesale, dispatcher.rs has no egress layer.
    Unlocks: removes "could this cleanup wipe my home dir / phone home?" from manual review
    entirely; THE anti-Replit demo (R2 of ELAI-DIRECTION).
    CAVEAT: seccomp+Landlock is Linux-only; Windows AppContainer / macOS Seatbelt parity for the
    post-expansion FS gate + worktree is UNRESEARCHED (egress proxy + credential broker are
    OS-agnostic). Scope non-Linux separately.

R4. LIVE MCP RUG-PULL / LINE-JUMP RE-PIN GATE (axis E). [lever: cant-wreck] [effort M] [STRONG]
    On every notifications/tools/list_changed, recompute the blake3 schema-pin and BLOCK
    fail-closed on mismatch, requiring explicit operator re-approval (current mcp_schema_pin is
    TOFU-only at first connect). New mcp_rug_pull_blocked SSB row with before/after description diff.
    Grounding: OWASP MCP03:2025 (runtime schema change w/o operator confirm) CONFIRMED;
    Trail-of-Bits line-jumping CONFIRMED; Invariant Labs WhatsApp sleeper rug-pull CONFIRMED;
    extends existing mcp_schema_pin.rs. (CORRECTION: mcp-scan is Invariant Labs, NOT Snyk.)
    Cheapest high-grounding move; depends on nothing.

R5. REVIEWER COCKPIT + DIFF-SURFACE-TO-REVIEW (DSR) METRIC (axis A). [lever: verified-before-seen]
    [effort M] [STRONG] [depends R2]
    Per-hunk view rendering the proof tokens ELAI already emits (F-4 stdout_hash-bound passing
    test, grounding-sweep citation, write-gate verdict) so a reviewer skims VOUCHED hunks.
    DSR = (lines needing full attention) / (total changed lines), computed deterministically from
    SSB events — the headline review-time number. Silent-by-default: surface a finding ONLY when
    backed by a deterministic gate verdict or a failing test (no LLM-opinion noise).
    Grounding: METR (arXiv:2507.09089) — reviewing/editing AI output is THE confirmed time sink,
    19% slowdown — CONFIRMED; code-review-agent PRs sit at 0-30% signal in 60.2% of cases
    (arXiv:2604.03196) CONFIRMED; naive LLM review is style-biased and misses SQLi/XSS
    (arXiv:2509.13650) CONFIRMED; consumes F-4 stdout_hash already in-repo.
    Unlocks: THIS is the review-time lever — bulk-accept the vouched band, spend attention only on
    un-vouched surface. DSR is recomputable + copy-resistant, defeating the Greptile-vs-CodeRabbit
    opposite-winner problem (no neutral leaderboard exists).

R6. SAFEDELTA COMPETITIVE BENCHMARK (axis H). [lever: auditable-reproducible] [effort XL]
    [grounding STRONG] [depends R2; F-2b LIVE keystone MUST land first]
    Full spec in section 4. Sequenced BEHIND F-2b (the dynamic single-harness extension of the
    existing SeCodePLT runner) — no competitor claim before ELAI's own single-harness dynamic
    floor is measured.

R7. PER-CHANGE CONFIDENCE PRODUCER + DUAL-LLM TRIFECTA QUARANTINE (axes B + E). [lever:
    honest-about-limits + cant-wreck] [effort L] [STRONG] [depends R1]
    Fuse token log-prob + self-consistency dispersion + oracle-free generated-test
    execution-agreement into the scalar claim_score the existing conformal decide(claim_score, tau)
    ALREADY consumes (the deferred composite-weights slot). When one context simultaneously holds
    a Trusted secret + Untrusted content + an egress channel, route the untrusted span through a
    quarantined sub-LLM with no tool grants (Dual-LLM).
    Grounding: post-hoc Platt rescaling is model-size-agnostic — the 7B IS a viable calibration
    target, ECE 0.32->0.03 (Spiess arXiv:2402.02047 CONFIRMED); oracle-free execution signal
    (CodeHalu arXiv:2405.00253 + Task-Abstention arXiv:2605.17029 CONFIRMED); Dual-LLM (Design
    Patterns arXiv:2506.08837 CONFIRMED). UniCR (2509.01455) guarantee is MODERATE confidence only.
    Unlocks: makes the already-shipped conformal abstention layer ACTUALLY FIRE on real changes
    (the "honest stop"); structurally prevents untrusted content steering a secret-holding
    tool-caller. Review-time: feeds the confidence-triage band (read low-confidence hunks first).

R8. SIGNED IN-TOTO PROVENANCE ATTESTATION + ARTICLE-50 VERIFIER (axis C). [lever:
    auditable-reproducible] [effort L] [STRONG] [depends R2]
    A custom in-toto v1 predicate (subject bound by blake3/sha256 digest; carries model id,
    ELAI_SEED, an SLSA-VSA-style PASSED/verifiedLevels rollup of every F-gate, evidence hashes,
    SSB event id), emitted to a git commit trailer (digest only) + sidecar file + new SSB
    action_type; signed via a local blake3-Merkle proof (offline default) or opt-in cosign
    keyless. New `elai verify-attestation` re-checks OFFLINE.
    Grounding: in-toto Statement v1 + custom-predicate freedom CONFIRMED; SLSA VSA
    verificationResult/verifiedLevels "trust without re-running" purpose CONFIRMED; Sigstore
    Rekor-v2 GA CONFIRMED; EU AI Act Art-50(2) marking + the Code-of-Practice cryptographic/logging
    route CONFIRMED. POSITION AS "designed to support Article-50 marking", NOT "certified compliant"
    (final Code of Practice June 2026; source code is not an enumerated example).
    Unlocks: a portable receipt a CI / auditor / regulator verifies WITHOUT re-running ELAI —
    extends F-4's anti-fabrication from inside-the-run to off-machine. One-command verified/not.

R9. SSB/LOG WRITE-TIME SECRET+PII REDACTION GATE + DB-FILE TTL (axis F). [lever:
    auditable-reproducible + honest-about-limits] [effort M] [grounding MODERATE] [depends R2]
    A LOCAL-ONLY (no cloud egress, unlike ggshield/Nightfall) fail-closed redactor on the
    single-writer SSB commit path and the hook-log sink, replacing detected secrets/PII with
    stable blake3-keyed tokens so append-only rows stay valid. Retention enforced by purging whole
    expired SESSION DBs (the SSB PANIC-on-DELETE invariant forbids row deletes).
    Grounding: ssb.rs payload carries raw prompt + PANIC-on-DELETE -> redaction MUST be write-time
    (VERIFIED); OWASP AI Agent Cheat Sheet "audit memory before persistence" + "redact log data"
    CONFIRMED; GitGuardian 24,008 MCP-config secrets + 3.2% Claude-Code leak rate CONFIRMED.
    CORRECTIONS REQUIRED before any external claim: EU penalty tier for Art-12 logging is
    EUR 15M/3% (NOT 35M/7% — that is Art-5 prohibited practices); the PRvL Presidio
    precision/recall numbers are REFUTED — re-source from Presidio's own docs.
    Unlocks: ELAI's own auditable journal becomes PROVABLY secret-safe — a precondition for
    shipping the RRA/attestation (R2/R8) to a client without it being a secrets-exposure liability.

==============================================================================
4. SAFEDELTA — THE COMPETITIVE SAFETY BENCHMARK (the explicit deliverable)
==============================================================================
NAME: SAFEDELTA (Safety-Delta Harness Eval).

WHAT IT MEASURES: the MARGINAL safety contribution of ELAI's fail-closed in-loop gates =
the residual-insecure-rate of code that survives the gate and reaches a human, head-to-head
against (a) the IDENTICAL model in ELAI with the gate DISABLED (the ablation control that
isolates the gate from the model) and (b) competitor harnesses with no in-loop write-gate, on
an identical git-stripped held-out task set — plus a parallel review-burden axis. It measures
PREVENTION REACHING THE REVIEWER, not raw model capability, and never claims the run was
correct (determinism != correctness, DFAH arXiv:2601.15322 CONFIRMED).

SUITES (all IDs verifier-CONFIRMED):
  - SUSVIBES-200 (arXiv:2512.03262, 61% func / 10.5% secure) — primary functional+security,
    real-OSS tasks reconstructed from NL, git-stripped.
  - BaxBench-392 (arXiv:2502.11844) — DYNAMIC backend exploit execution (black-box malicious
    queries + white-box secret/password checks); resists static pattern-gaming.
  - SeCodePLT CWE corpus (arXiv:2410.11096) — static-recall floor across 44 CWE classes, scored
    from the real evaluate_vibegate write-gate via the existing seccodeplt_runner.
  - AgentDojo-97/629 (arXiv:2406.13352) — prompt-injection attack-success sub-axis, paired with
    the E-37 prompt-armor classifier + R1 taint wiring.
  - MCP Security Bench / MSB (arXiv:2510.15994) — optional MCP-attack sub-axis for R4.

COMPETITOR MATRIX: ELAI gate-ON | ELAI gate-OFF (ablation) | Cursor+frozen-model | Claude Code |
GitHub Copilot | Cline.

PROTOCOL: each harness drives the SAME task via a thin per-harness runner shim invoking its
real CLI/API (not a reimplementation); identical prompt + frozen model + git history stripped.
Capture each harness's accepted diff; execute BaxBench black/white-box exploits + SeCodePLT
dynamic labels against the diff; for ELAI additionally record the gate's pre-write Block
verdicts. Score 5 axes: (1) residual-insecure-rate post-acceptance (dynamically exploited),
(2) secret-leak rate, (3) slopsquat/hallucinated-dep block rate scored vs OWASP Top10:2025 A03
Software Supply Chain Failures, (4) injection attack-success-rate, (5) review-burden (triage
findings per accepted PR).
  FROZEN-MODEL FAIRNESS: because Qwen2.5-Coder-7B is a generation stale, the PRIMARY published
  framing is the gate-ON-vs-gate-OFF ABLATION (same model, same harness); the
  same-model-different-harness comparison is SECONDARY (model held identical behind each
  harness's cloud lane).
  CREDIBILITY ANCHOR: every SAFEDELTA run pins ELAI_SEED, records the full run as append-only
  SSB events (per-task model-output hash + gate verdict + exploit result + blake3 corpus hash),
  and emits a SIGNED run manifest published alongside the number — so a buyer's own security team
  re-runs the exact comparison. This SSB+seed reproducibility is the answer to "that's just
  marketing"; Endor / CodeRabbit / Greptile / claude-replay offer nothing equivalent.

ANTI-GAMING: (a) git-history strip + a tool-call/file-access cheat detector flagging any harness
reading the pre-fix commit — cloned from Endor's Agent Security League finding that
SWE-Agent+Opus-4.6 exploited git history in 163/200 = 81% of tasks; (b) DYNAMIC exploit execution
(BaxBench), so a harness cannot satisfy a pattern matcher without producing actually-secure code;
(c) held-out CWE split + blind suite rotation so ELAI's own ruleset cannot overfit the corpus;
(d) the gate-OFF ablation proving the delta comes from the GATE not the MODEL; (e) SAFEDELTA's
own reported PASS bound to a blake3 stdout_hash (reusing F-4) so it cannot fabricate its result.

HEADLINE METRIC: SAFETY DELTA = (strongest_comparator_residual_insecure_rate -
ELAI_gate_ON_residual_insecure_rate) on the identical held-out git-stripped task set, with a 95%
bootstrap CI and a per-CWE breakdown — phrased to a buyer as "N fewer dynamically-exploitable
diffs per 100 tasks reach a reviewer." NEVER a SWE-bench-Verified rank.

REVIEW-TIME AXIS: human-triage findings per accepted PR, alongside the DSR ratio (R5), measured
via a within-subjects screen-recorded clustered-SE study (METR arXiv:2507.09089 design) comparing
time-to-first-approve through the cockpit vs a plain diff, with escaped-defect-rate as a guardrail
so speed is never silently traded for safety. Speed-AND-safety simultaneity is the HYPOTHESIS the
study TESTS, not a pre-claimed result (industry-wide unmeasured — section 6).

STATISTICAL CAVEAT (whats_missing): with 200-392 tasks and small per-CWE cells, whether the
headline-delta CI is tight enough for a significance claim is UNRESOLVED; may require pooling
SUSVIBES+BaxBench+SeCodePLT (whose cross-harness methodological defensibility is itself unverified).

==============================================================================
5. VERIFIER LANDMINES — DO-NOT-CITE / CORRECT-BEFORE-SHIP (preserve verbatim)
==============================================================================
- PRvL (arXiv:2508.05545) Presidio 0.83/0.88/0.85, passport-0.33-F1, GPT-4o recall>0.9 numbers
  are VERIFIER-REFUTED (not in that paper). The qualitative "PII detection is high-recall /
  low-precision" framing is fine; re-source the numbers from Presidio docs.
- EUR 35M/7% is the Art-5 PROHIBITED-PRACTICES tier. Art-12 logging / Art-50 marking carry
  EUR 15M/3% (Art-99). Do NOT pair 35M/7% with a logging-compliance argument.
- CyberSecEval 3 (arXiv:2408.01605) is OFFENSIVE-cyber-focused — NOT the insecure-code-GENERATION
  rate source. The autocomplete/instruct insecure-code suite is CyberSecEval 1/2 (arXiv:2404.13161).
- "CVE-2026-30623 in Anthropic's MCP SDK" is WRONG — that CVE is LiteLLM. Anthropic's SDK has no
  CVE (by-design stdio stance). Say "the stdio command-injection family; Anthropic declined a
  protocol fix."
- The Thinking Machines nondeterminism experiment used Qwen3-235B-A22B-Instruct-2507, NOT Qwen3-8B
  (the 80-unique / token-103 / 1000-completions figures are correct).
- Do NOT ground a BspEngine-wiring move on "no production caller" — REFUTED (task_runner.rs:5018 is
  a non-test caller). The accurate gap is recover_from_checkpoint (the RESUME path) is a stub.
- Kiro "two-person approval optional" and the "breaking any one lethal-trifecta leg defeats the
  class" phrasing are author INFERENCE / a stronger restatement — not sourced incident fact;
  Amazon disputes the Kiro attribution.
- Martian "Code Review Bench" independence / CodeRabbit "#1" is vendor-amplified (Greptile's own
  benchmark names the opposite winner). Use the divergence as the ARGUMENT FOR a reproducible
  metric, not as a competitive ranking.
- GitHub Copilot review 71%-actionable / 29%-silent / 5.1-comments / April-2025-GA figures could
  NOT be confirmed — borrow the noise-suppression PATTERN, do NOT cite the numbers.
- mcp-scan distribution channel (pip/npm/cargo) UNCONFIRMED — must pass anti-slopsquatting
  registry verification before the whole-directory-scan move is wired.
- Do NOT add Vault/SPIFFE/cosign/sigstore/mcp-scan as dependencies on assumption — each must pass
  anti-slopsquatting verification (name+repo+version+date+license) first; the credential broker +
  local-Merkle signing paths are hand-rolled minimal, not product dependencies.

==============================================================================
6. WHAT IS MISSING / OPEN QUESTIONS (do not over-claim)
==============================================================================
- COST modality not researched: no axis measured tokens/$ per prevented vulnerability or the
  cloud-lane escalation cost of routing local-7B abstentions to Sonnet/Opus. Only cost anchor is a
  general-purpose cascade-abstention paper (arXiv:2502.09054, 13% cost / 5% error cut).
- The escaped-defect-rate-vs-review-time-reduction tradeoff curve is UNGROUNDED industry-wide: no
  primary source measures whether confidence-sorted review improves speed AND safety
  simultaneously. ELAI could be FIRST to publish it — but it is a hypothesis the METR-style study
  must TEST, not an assertion.
- UNVERIFIED carried forward: UniCR (arXiv:2509.01455) is a non-peer-reviewed preprint (MODERATE);
  FIDES (2505.23643) + RTBAS (2502.08966) effect-sizes NOT extracted (lineage only;
  RTBAS uses dependency-screeners, NOT literal taint-tracking); EU AI Act Art-12 >=6-month log
  retention is NOT in the Article-12 text itself (trace before advertising); the Digital-Omnibus
  delay (~Dec 2027, in trilogue) is a live caveat on the Aug-2-2026 deadline.
- Capture-completeness for replay (R2): unconfirmed whether the SSB retains enough tool RESPONSE
  BODY to RE-EXECUTE body-reading steps vs only VERIFY divergence.
- Cloud-lane reproducibility (R2): Anthropic's system_fingerprint may be stampable only as a
  reproducible-action-stream, never reproducible-tokens.
- Cross-platform OS gap (R3): the seccomp+Landlock layer is Linux-only; Windows/macOS parity for
  the post-expansion FS gate + worktree is unresearched.

==============================================================================
7. RECOMMENDED SEQUENCING + THE ONE RISK THAT MATTERS
==============================================================================
SEQUENCE (cheap-and-high-grounding first; the headline last):
  Wave 1 (activation, low risk, high grounding): R4 (MCP re-pin, no deps) + R1 (taint propagation,
    unlocks R3/R7) + the patch-N citation fixes (section 2).
  Wave 2 (the trust spine made shippable): R2 (RRA) -> then R3 (blast-radius gates), R5 (cockpit+DSR),
    R8 (attestation), R9 (journal redaction) all hang off R2.
  Wave 3 (the headline, gated): R7 (confidence producer, needs R1) + R6/SAFEDELTA (needs R2 + the
    RunPod-pending F-2b live keystone). No external safety-delta claim ships before ELAI's own
    single-harness dynamic floor is measured.

THE ONE RISK: false-positive / alarm-fatigue friction is what killed the closest prior art
(CodeGate, archived 2025-06-05). Every gate in waves 1-2 must bound its FP rate on >=3 real
orchestrate runs before default-on, and the cockpit (R5) must be silent-by-default (surface only
gate-backed findings). A reviewer who learns to ignore ELAI's signals loses the entire
review-time thesis — the metric (DSR / escaped-defect-rate) is also the FP early-warning.

DO-NOT-DO (binding): no SWE-bench-Verified win claim; no perfect/correct-code promise (a green
replay proves REPRODUCED, not RIGHT — DFAH); no post-hoc scanner or CodeRabbit/Greptile-style
review leaderboard (saturated/funded; Endor already does harness-vs-harness); no re-statement of
any F-1..F-10 (every move above fills a different gap or composes by consuming a verdict); ship
attestation as design-intent ("designed to support Article-50 marking"), never as certification.

==============================================================================
8. POINTERS
==============================================================================
  - This run's full per-axis research + verifier verdicts: workflow run wf_826d66d4-0e6 (temp
    output may be GC'd; this file is the durable digest).
  - Predecessors: ELAI-DIRECTION-2026-06-01.md (strategy), VIBESEC-{RESEARCH,TOOLS-MATRIX,PAPERS}
    -2026-06-01.md (the 2026 market/threat/paper grounding), docs/thefinalglance/F-plan/ (the 10
    write-time gates this file is complementary to).

End of ELAI-TRUST-MAXIMIZATION-2026-06-02.md.

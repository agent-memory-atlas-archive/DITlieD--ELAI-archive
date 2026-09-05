# LIA TRUST KERNEL — standalone build/test/validate/publish plan

STATUS: PLANNED (standalone program, NOT an ELAI letter unit; runs BEFORE the ELAI benchmark plan
7-BENCHMARK-READINESS-PLAN.md and shares its corpus-hardening discipline. Authored 2026-07-18 from
the direction-sweep corpus workflow/reasoning/direction-sweep-2026-07-17/ (1..7), the trust-kernel
chat note /home/user/teikoku/brain/migrated-notes/"ELAI trust kernel chat.md" (taken skeptically,
corrections applied below), and ELAI-SPEC-OVERVIEW.md read in full.)

PROOF_SPINE: feeds PS-1 (the kernel's cross-harness gate-ON-vs-OFF interception/detection numbers
are the outward, publishable form of the PS-1 safety-delta ablation; its corpus-hardening and
paired-ablation protocol are shared with the ELAI W1 kill-gate) and transitively PS-3 (the
unsupported-claim detection corpus is reusable as the hallucination-delta substrate).

================================================================================
0. WHAT LIA TRUST KERNEL IS (one paragraph) + THE FUNNEL THESIS
================================================================================

LIA Trust Kernel is an open-source, harness-agnostic trust substrate you put between ANY coding
agent (Claude Code, Codex, Devin first; Cursor/Gemini CLI later) and the user's machine and data.
It covers the whole LLM failure surface at the BOUNDARY grain: it blocks destructive actions and
out-of-scope writes (cant-wreck — the GPT-5.6-Sol / GLM-5.2 file-deletion incident class), detects
fabricated test results and false claims (cant-lie), verifies claims against real evidence — files,
symbols, test runs, command exits, dependency registries, cited sources (cant-fake-sources), flags
unsupported agreement (sycophancy), stops secret leakage, and records everything in a tamper-evident
append-only journal with signed receipts an auditor can re-verify offline. It deliberately does NOT
plan, decompose, repair, route models, or decide what to build — that interior intelligence is LIA/
ELAI proper. THE FUNNEL: after LIA, a user has receipts proving exactly how often their agent lied,
agreed without evidence, fabricated a pass, or reached for a destructive command — and a measured
appetite for the harness where those things are structurally impossible. LIA measures and blocks;
ELAI prevents, repairs, and finishes. "The wrapper is the on-ramp, not the product" (I-40:34-35;
2026-07-10 addendum: kernel-first engineering, native-first product identity, wrapper-assisted
distribution).

THE ONE-SENTENCE BOUNDARY (from the chat note, kept): open-source everything needed to
independently establish trust in an agent action; keep private everything that decides which
actions to take, how to solve the task, and how to recover until the task is finished.

GROUNDING-SYSTEMS DECISION (owner asked, decided here): YES — ELAI's grounding systems are folded
in, but ONLY the deterministic VERIFICATION half, re-authored clean-room (never code-copied; the
engine stays closed and the OSS repo stays license-clean): the six binary verification methods
(file / source / test / command / api / api-registry existence+support checks, Phase-2 GRC's
verification leg), the claim↔evidence data model (claim type, evidence_required, verdict:
verified|refuted|unsupported), and the unsupported-agreement (sycophancy) detector + fixtures.
KEPT CLOSED (the ELAI-only intelligence): automatic claim-risk prediction, semantic-entropy /
counterfactual selection, GRC reasoning budgets, plan-grounding sweeps, automated repair, claim
extraction beyond the simple deterministic extractor, DKG/retrieval, mode routing. The public
kernel answers "is this supplied claim supported?"; ELAI answers "what needs checking, how much
effort, and what do we do when it fails."

// DECISION (naming, reversible): project name "LIA Trust Kernel", repo/package `lia-trust`,
binary `lia`. "Kernel" is kept as the internal/product noun (it is honest: a small permanent core,
Hermes narrow-waist sense) but NEVER used in a security claim ("kernel-enforced" is banned copy —
P0-5: hook/MCP interception alone is not a security boundary). Public one-liner (R4's unow
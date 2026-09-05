TEIKOKU HARNESS FIX — interactive/autonomous modes, guide-up-front instead of reject-after
Date: 2026-07-05
Scope: TEIKOKU AS A GENERAL base-cc HARNESS (its own gate spine + its interactive and autonomous modes). NOT the ELAI-backlog walk — that is the separate file (teikoku-merge-addon-M8-M9-2026-07-05.md).
Grounding: teikoku handoffs (gate-redesign, general-orchestrator, build, glm-walk); ELAI regime-aware policy (docs/thefinalglance/ELAI-SYSTEMS-OVERVIEW.md §5) and I-5 provide-not-prove (§5/§13). Every named seam (reflect_gate.py, scope_ack_gate.py, project_memory_gate.py, read_full_gate.py, worker_adapter._worker_env, _load_skill_context, the gate-manifest, the conductor /spawn route) is taken FROM the handoffs, not from live source. CONFIRM against ~/teikoku/.claude/hooks/*.py + ~/teikoku/teikoku/conductor/*.py before editing. The gate spine is FROZEN (R8): any gate change needs an out-of-band re-lock (command in the gate-redesign handoff HOW TO PICK UP).
Boundary: this is a FIX PLAN, not applied edits. The distro is owner-hold + uncommitted; do not touch it without owner go.


0. THE DIAGNOSIS (grounded)

TEIKOKU's gate spine (gate-redesign handoff) is deliberately high-friction, and the friction is REMEDIATION — deny-after-attempt:
  - reflect_gate.py: PreToolUse "*", DENIES a tool call if no substantive visible reasoning since the last tool result. "The distro is now high-friction by design: reflect_gate roadblocks every action."
  - scope_ack_gate.py: fail-CLOSED, DENIES every tool until a visible ack names >=1 concrete file AND a non-truncating Read of each is in the transcript.
  - project_memory_gate.py: injects memory.md as additionalContext BUT also has a PreToolUse-deny fallback that DENIES the first project mutation until memory.md is proven read.
  - read_full_gate.py: DENIES a truncating Read while a full-read flag is set.
  - commit_gate.py: DENIES commit/push unless a sentinel is present.

For INTERACTIVE (human-in-the-loop chat) this is friction. For AUTONOMOUS (glm/opus workers) it was CATASTROPHIC: the glm-walk handoff recorded a worker that burned 12.7M tokens / 172 turns / 0 edits because scope_ack_gate blocked every tool call ("No tool action is possible"). The applied fix was a BLANKET bypass: the 4 meta-cognitive gates now skip when TEIKOKU_AUTONOMOUS=1.

That bypass is the tell. The gates were the WRONG SHAPE (reject) for autonomous, and turning them fully OFF throws away their intent (the worker SHOULD have read the file / known the scope — it just shouldn't be made to PROVE it under a fail-closed deny). The workers run now; the open problem is that they run UN-guided (no pre-read files, no handed scope), which is exactly why File 1's M9 exists. This file fixes the harness so it DELIVERS the guidance the reject-gates were clumsily demanding.

Same thesis as the deterministic-guidance work: kill wrongness as early as possible — CONSTRAIN (unemittable) > GUIDE (provided up front) > and only then REMEDIATE. TEIKOKU's spine is stuck in REMEDIATE for the meta-cognitive class.


PART A — TWO FROZEN GATE PROFILES, NOT ONE SPINE + A BYPASS FLAG

Interactive and autonomous need OPPOSITE gate policies (grounded: ELAI regime-aware policy, overview §5 — cloud and local need opposite CONTEXT policies; the same structural point applies to GATE policy here). Today TEIKOKU has ONE spine and a scatter of env flags (TEIKOKU_AUTONOMOUS, TEIKOKU_CHAT_TURN, REFLECT_MODE) doing an ad-hoc split.

FIX. Declare TWO named gate profiles, resolved deterministically from the session's mode (the general-orchestrator handoff shows Mode is already a first-class axis, independent of Orchestration) x role, and FREEZE BOTH in the gate-manifest (an unfrozen profile makes the split gameable):
  - SAFETY FLOOR (identical + fail-closed in BOTH profiles): scope_write_gate (host /mnt/f + outside-~/teikoku deny), frozen_gate_guard, bash_destructive_gate, commit_gate, comment_gate, plus any VibeGate/dep/secret class. These are Regime-1 CONSTRAINS — they make destructive/out-of-scope actions unemittable, which is correct regardless of who is driving. The glm-walk fix already KEEPS these on for workers; this formalizes it as the floor.
  - META-COGNITIVE gates (reflect, scope_ack, project_memory-deny-leg, read_full): differ by profile per PART B.
Replace the TEIKOKU_AUTONOMOUS=1 blanket bypass with "profile = AUTONOMOUS", so the split is a declared, frozen policy, not a per-hook if-flag that a future hook can forget to honor.


PART B — CONVERT THE META-COGNITIVE GATES FROM REJECT TO PROVIDE (the core fix)

Each meta-cognitive gate checks that the model DID something and DENIES until it did. Invert: the harness PROVIDES that thing up front, so there is nothing to reject (I-5 provide-not-prove; give the constraint, don't demand proof of it).

  - scope_ack_gate (denies tools until an ack names + reads files):
    AUTONOMOUS: DROP the gate. The dispatch PRE-READS the task's declared files (the write-set + the PRODUCTION_HOOK seam file from File 1's M9) into the worker's context and states the scope as an up-front block. The worker already HAS the files and the scope; there is no ack to gate on.
    INTERACTIVE: keep it only as a GUIDANCE nudge (PART C), not a fail-closed deny.

  - project_memory_gate (injects memory.md as additionalContext + denies first mutation until read):
    BOTH: keep the INJECTION (that is the guide). DROP the deny-until-read fallback for AUTONOMOUS (redundant remediation once the content is injected). CAVEAT from the handoff: additionalContext-on-print is UNVERIFIED-LIVE; if base-cc does not honor additionalContext on a --print UserPromptSubmit, prepend memory.md to the worker's effective_prompt directly — the chat lane already does exactly this for skills via _load_skill_context (same seam, proven reaches base-cc stdin).

  - read_full_gate (denies a truncating Read while the full-read flag is set):
    AUTONOMOUS: DROP. The task declares which files matter; pre-read them fully at dispatch. Nothing to gate.
    INTERACTIVE: KEEP. A human said "read X fully"; honoring that is guidance and the deny is cheap because the human is present to see and correct it.

  - reflect_gate (denies every action lacking visible reasoning):
    AUTONOMOUS: OFF, or mutation-only + ADVISORY (a journaled WARN, NEVER a fail-closed deny). The fail-closed every-action deny is exactly what produced the 12.7M-token / 0-edit spin — a worker with no visible-reasoning habit hits the wall on turn 1 and never recovers.
    INTERACTIVE: REFLECT_MODE=mutation-priority (already a knob) as a nudge, not an every-action deny (PART C).

Result: for autonomous workers, the reject-loops are GONE and replaced by up-front provision (files pre-read, scope handed, memory injected). This is the harness twin of File 1's M9 — File 1 says WHAT to hand the worker; PART B makes the harness hand it instead of gating on proof of it.


PART C — INTERACTIVE MODE'S OWN REMEDIATION WASTE

Even with a human present, reflect_gate's every-action fail-closed deny is the user's ORIGINAL complaint (rules-as-rejection burns context) applied to TEIKOKU's own chat: a turn that trips the gate is denied and re-costs the turn. The guide version:
  - Shape the behavior via the PROMPT/context, not a deny: inject one guidance line ("name your target file + why before a mutation") that the model follows naturally (Regime 2). A gate the model satisfies ~95% of the time by being nudged is cheaper than one that denies ~5% and re-costs the turn.
  - Reserve HARD deny for the SAFETY floor only.
  - Keep reflect as mutation-priority ADVISORY: it may journal a WARN when a mutation lands with no visible reasoning (useful signal for the human), but it does not fail-close the tool.
This keeps interactive mode's intent (a careful, reasoned human session) without the deny-burns-a-turn tax.


PART D — AUTONOMOUS MODE IS HALF-WIRED (a wire-dark finding on TEIKOKU itself)

Grounded in the build handoff KEY OWNER NUANCE: the conductor's recursive /spawn route (real depth/fanout/$10-ceiling caps, spawns NEW CLI sessions, visible on the process canvas) EXISTS, but base-cc's /auto-orchestrate skill does NOT call it — today a spawned worker recurses via IN-PROCESS Agent-tool subagents, bounded only by the SKILL's own caps (parallel 3 / 30-dispatch), NOT the conductor's depth/fanout/$10 caps, and NOT as new sessions. So "autonomous mode" does not actually run through the capped, ceilinged, OBSERVABLE conductor path it advertises.

This is the SAME class as the ELAI wiring problem, one level up: the /spawn PRODUCER exists, the CONSUMER (a caller from the skill) does not — wire-dark, on TEIKOKU's own harness.

FIX. The adapted auto-orchestrate skill dispatches recursion via the conductor /spawn seam (POST /spawn, depth+1, caps + ceiling enforced server-side) instead of in-process Agent subagents. Then autonomous mode's caps are REAL, its cost ceiling actually binds worker-spawned work (the build handoff notes worker-completion cost is not yet fed the ledger — this closes that too), and its recursion is visible on the process canvas (the general-orchestrator handoff's whole point). This is a correctness fix orthogonal to the reject-vs-guide thesis, but it belongs in "fix autonomous mode" and it is a genuine wire-dark defect, not a design opinion.


SEQUENCING (highest value first)

1. PART B (provide-not-reject for autonomous). Directly kills the reject-loop tax and lets workers run GUIDED (pre-read files + handed scope), which is the harness half of File 1's M9 pass-rate lift. Highest value.
2. PART A (two frozen profiles) — the container PART B needs; replaces the ad-hoc TEIKOKU_AUTONOMOUS flag with a declared, frozen policy + safety floor.
3. PART D (wire /auto-orchestrate through /spawn) — a correctness/observability fix; makes autonomous caps + cost real. Independent of 1-2, can run in parallel.
4. PART C (interactive reflect as nudge) — polish; lowest risk, do last.

All gate changes touch the FROZEN spine (R8) -> each needs the out-of-band re-lock. Profiles must be frozen too.


HONEST LIMITS / ANTI-SYCOPHANCY

  - The TEIKOKU_AUTONOMOUS bypass ALREADY made workers run (they edit real code now). PART B is NOT "un-break workers" — it is "run them GUIDED so their gate pass-rate rises," which is a quality/yield fix, not an availability fix. Do not frame it as fixing 0-edits; that is already fixed.
  - The provide-up-front inversions assume the task carries a well-formed contract to provide FROM (the write-set + seam). For the ELAI walk that is File 1's M9; for arbitrary registered projects (the general-intake path) there may be no such contract, in which case the autonomous profile falls back to "pre-read whatever the objective names" + safety floor, and the meta-cognitive guarantees are weaker. State that; do not claim universal provision.
  - additionalContext-on-print is unverified-live (handoff). The effective_prompt-prepend fallback (proven via _load_skill_context) is the safe path; prefer it over relying on additionalContext until a live --print check confirms.
  - None of this is verified against live source. Confirm every hook + the /spawn route + the dispatch prompt builder before editing, and re-lock the frozen manifest for any gate change.
  - Interactive gate friction is partly INTENTIONAL (a careful human session). PART C lowers it to a nudge; if the owner WANTS the hard deny in interactive as a discipline, keep it — that is a preference, not a bug. Confirm with the owner before softening interactive reflect.

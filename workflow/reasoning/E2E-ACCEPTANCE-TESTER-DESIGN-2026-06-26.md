E2E Acceptance Tester (the "fake human") - design

Date: 2026-06-26
Author: main session, grounded in: the wiring-enforcement rule stack (.claude/rules/wiring-enforcement.md
R1-R10, production-caller-gate.md, tracing-test-assert.md, maintainability-review.md), the two conductor
explorations (teikoku-p4-explore-devin-conductor.md, teikoku-p4-explore-codex-conductor.md), and the
Autonomous Finish System design (AUTONOMOUS-FINISH-SYSTEM-2026-06-18.md + the DEVIN variant). Every claim
below cites a file already read this session.

Problem (user's words): no time to manually use+test ELAI. Want a system that, per plan / per feature,
checks it actually works as intended on the dashboard, runs benchmarks, and decides if the feature is
really wired+working - so wire-dark dies for real and manual testing stops. Must be model-agnostic (test
any model, any feature) and use the air-gapped Chinese-proxy model as the judge (it tests the compiled
artifact, never sees ELAI source, returns only results, scanned for injection on the WSL<->Windows boundary).

================================================================================
0. THE ONE REFRAME - this is a MISSING WAVE, not a new system
================================================================================

The "fake human" is the semantic-acceptance layer (call it L5) on top of the wiring doctrine's already-
load-bearing runtime legs. It is NOT a parallel build. Grounding chain:

  wiring-enforcement.md admits the static legs (L1/L2) cannot close wire-dark for Rust+tokio+dyn Trait;
  "the runtime leg (Layer 3/4) is the only sound backstop." L3 = drive the real binary, coverage hit.
  L4 = drive the real entrypoint, assert the off-agent SSB row landed. Both prove the code RAN. NEITHER
  proves the feature DID WHAT THE HUMAN WANTED. That gap is exactly where manual testing time goes and
  where wire-dark survives at the FEATURE grain (the symbol is called, but the behaviour is wrong/partial).

  L5 ACCEPTANCE = fire the plan's declared trigger through the REAL entrypoint, collect the observed
  evidence, and have an independent judge score it against the plan's ACCEPTANCE block. The judge does NOT
  invent intent - it is handed observable_effect / trigger / acceptance as the rubric (TaskSpec already
  carries these verbatim, AUTONOMOUS-FINISH-SYSTEM §3). A feature whose plan has no checkable
  observable_effect simply cannot earn a green (unknown=fail, same schema as visual-ui-loop.md). That is
  the forcing function: no acceptance rubric -> no pass -> wire-dark cannot be declared done.

The conductor already runs Wave 1 IMPLEMENT, Wave 2 WIRE-SWEEP (every DARK symbol -> WIRE task), Wave 3
INTEGRATION (R10 cross-seam smoke), Wave 4 AUDIT, Wave 5 POLISH (AUTONOMOUS-FINISH-SYSTEM §5). Wave 2 + the
frozen 6-verifier gate ARE the wire-dark machinery (L2-L4). What is absent is a wave that judges feature
behaviour. So we ADD Wave 3.5 ACCEPTANCE (between INTEGRATION and AUDIT), not a new program.

================================================================================
1. WHAT ALREADY EXISTS (this is ~80% assembly, grounded per file)
================================================================================

From the LIVE Devin conductor (F:\.ELAI_workflow\conductor, teikoku-p4-explore-devin-conductor.md):
  - durable SQLite WAL task graph + dependency-frontier scheduler + TTL leases + crash-safe reclaim
    (conductor_db.py, scheduler.py, leases.py)
  - the off-agent frozen gate runner that already runs the 6 ELAI verifiers + cargo (gate_runner.py;
    smoke proves wire-dark BLOCK and wired PASS for both Rust and Python)
  - serialized integrator + post-merge re-gate (integrator.py)
  - the wave engine + loop-until-dry + epoch compaction (conductor_loop.py)
  - control plane (pause/drain/emergency-stop) + localhost dashboard (control_plane.py, dashboard_server.py)
  - a DORMANT codex/proxy worker lane: worker_adapter.build_codex_command (worker_adapter.py:451-496)
    shells `wsl -d <distro> codex exec --ephemeral ... --proxy` against a per-account CODEX_HOME. This is
    the air-gapped judge transport, already wired, currently unused (default harness=devin-cli).
  - SSB append-only event log = the evidence store the judge scores against.

From the idle Codex conductor (WSL codex-vm /root/.conductor-codex, teikoku-p4-explore-codex-conductor.md):
  - guard.py: a BIDIRECTIONAL safety scanner. check_prompt() scans text SENT to the model for injection
    (~17 patterns); check_output() scans what the model RETURNS for dangerous commands (rm -rf, dd, git
    push --force, curl|bash ... ~30), exfiltration (curl/wget/scp/nc to URL/IP, http.server ... ~9), and
    instruction-override echoes. Returns GuardResult{safe, reasons, score}; a hit -> BLOCKED. This is the
    exact WSL<->Windows boundary scanner the user asked for, already built and stdlib-only (drop-in).
  - a live, dry-run-green codex-exec-via-bmapi-proxy worker lane (gpt-5.5, 2 accounts) running inside
    codex-vm - i.e. "an air-gapped model behind the Chinese proxy with injection scanning on the boundary"
    already exists end-to-end; it has just never judged anything (it was built as a BUILDER).

From ELAI + the .claude harness:
  - the plan files' WIRING_CONTRACT / DETERMINISTIC_CHECK / ACCEPTANCE blocks = the per-feature rubric.
  - CDP screenshot capture (dashboard-v2/screenshots/cdp_*.mjs, :9222 against the Vite dev server :1420)
    + the schema-constrained visual judge (.claude/tools/visual/ui-diff.mjs + ax-scaffold.mjs +
    visual-ui-loop.md: pass|fail|unknown, unknown=fail, mandatory evidence region).
  - B-37 multi-runtime backend (RuntimeAdapter / PerRoleRuntime) - lets the run-under-test be ANY model,
    which is how "test any model" is satisfied (parameterize ELAI's runtime config, judge the result).

What is MISSING and must be built is only L5: the acceptance driver + the source-stripped judge packet +
the judge dispatch + the verdict gate. Everything else is reuse.

================================================================================
2. TOPOLOGY DECISION - one conductor, harvest two assets (NOT a merge, NOT two conductors)
================================================================================

DECISION: build L5 into the LIVE Devin conductor. Do NOT merge the whole Codex conductor in, and do NOT
run two conductors pipelined.

Reasoning (grounded): both conductors are the SAME finish-loop design (both explore docs concur). The
Devin one is live and complete; the Codex one is a redundant SUBSET that has never run a real turn. Merging
two copies of one design buys nothing. The Codex conductor's only UNIQUE assets are (a) guard.py and (b)
the codex/proxy worker-lane wiring - and the Devin conductor ALREADY has a dormant copy of (b). So:

  - HARVEST guard.py from the Codex conductor into the Devin conductor as the boundary scanner on the
    judge lane (small, stdlib, the explore doc calls it drop-in).
  - REACTIVATE the dormant build_codex_command lane in the Devin conductor as the AIR-GAPPED JUDGE lane,
    dispatched into codex-vm against the bmapi proxy (the user's Chinese-proxy model; cross-family vs the
    free GLM/kimi builders, which is the right call per D-13 FalsifiabilityCrossFamily).
  - RETIRE the rest of the Codex conductor, or keep it parked as a reference / escalation sandbox. It is
    not deleted (no-removal-without-confirmation), just not the engine.

This honours BOTH of the user's constraints that pull in opposite directions: "everything in one place"
(one control plane, one dashboard, one task DB = the Devin conductor) AND "air-gap the judge" (the judge is
a worker lane that physically runs in codex-vm behind the proxy, with a source-stripped packet). The air-gap
is provided by the VM + the packet shape, NOT by running a second whole system.

================================================================================
3. THE ACCEPTANCE WAVE (L5) - where it slots and how it runs
================================================================================

New task type ACCEPTANCE, new wave 3.5, runs AFTER a feature's IMPLEMENT+WIRE+INTEGRATE are green on
merged head (so the static gate already passed - L5 is additive, never a replacement).

Per-feature pipeline (deterministic harness; only the final verdict is a model call):
  A. SELECT a feature = one plan file's observable_effect (feature grain, not symbol grain). Skip if
     unchanged since its last ACCEPTANCE green (budget discipline - do not re-judge every wave).
  B. DRIVE the real entrypoint (NO model needed here, pure script):
       - CLI/IPC features: fire the plan's trigger through `elai <subcommand>` / the IPC handler.
       - dashboard features: drive the Vite dev server over CDP (:9222 <-> :1420), capture pinned
         screenshots (deviceScaleFactor 1, fromSurface true, font-render-hinting=none per visual-ui-loop.md).
       - benchmark features: run the bench harness (SAFEDELTA / swe_bench_pro_elai...), capture numbers.
  C. COLLECT evidence Windows-side, by id: read the SSB rows the trigger should have emitted (the judge
     never supplies these - they are read from ssb.db by the conductor, so a malicious judge cannot
     fabricate a row that was never emitted), the screenshot PNGs, the bench result JSON.
  D. BUILD THE SOURCE-STRIPPED JUDGE PACKET (the load-bearing air-gap control, NEW work): {feature_id,
     ACCEPTANCE rubric verbatim, the collected evidence (rows as redacted text, screenshots as images,
     bench numbers)}. It contains NO source, NO worktree, NO repo paths beyond the rubric. This is a
     DIFFERENT context-builder mode than the build workers get (they get a full .elai_cc worktree; the
     judge gets artifact+rubric+evidence only).
  E. PROMPT-GUARD the packet (guard.py check_prompt) before it leaves Windows.
  F. DISPATCH to the air-gapped judge (codex-vm + bmapi proxy). The judge returns a STRICT schema:
       {feature_id, verdict: pass|fail|unknown, evidence_refs: [hash], rationale: <bounded text>}.
       unknown is treated as FAIL (a feature the judge cannot confidently accept is not accepted).
  G. OUTPUT-GUARD the return (guard.py check_output) on the boundary; a hit -> the verdict is discarded
     and the feature is held NEEDS_REVIEW (never silently passed).
  H. RECORD a NON-LOAD-BEARING verdict: an SSB acceptance row + a dashboard tile. A red verdict marks the
     feature reopened (re-queues an IMPLEMENT/WIRE/FIX task). A green verdict is DISPLAY only - it does not
     ship code and does not weaken any gate (the static gate already decided shippability).

================================================================================
4. TRUST + INJECTION MODEL (the WSL<->Windows boundary)
================================================================================

Threat: the judge model (untrusted, behind the Chinese proxy) returns crafted output that injects the
Windows side, or the proxy exfiltrates ELAI IP.

Controls, layered (matches the project's own fail-closed doctrine):
  1. SOURCE-STRIPPED PACKET (§3.D): the judge sees compiled artifact + rubric + evidence, never source.
     This is the primary IP control - exfiltration has nothing to exfiltrate.
  2. guard.py BIDIRECTIONAL SCAN (§3.E/G): injection out, dangerous-command/exfiltration/override in.
  3. STRUCTURAL CONTAINMENT, not substring trust: the rationale field is rendered to the user as DATA, it
     is NEVER concatenated into another agent's prompt. (Project OWASP finding: substring taint-binding
     misses paraphrased injection - so do not rely on scanning the text to make it safe to re-inject;
     rely on never re-injecting it.)
  4. EVIDENCE READ WINDOWS-SIDE BY ID (§3.C): the judge can only say pass/fail on rows that actually
     exist in ssb.db; it cannot manufacture evidence.
  5. NON-LOAD-BEARING VERDICT (§3.H): the worst a fully-compromised judge can do is mark features red
     (false reopen = wasted worker time) or green (caught anyway by the independent static L1-L4 gates
     that already ran). It can NEVER cause code to ship or a gate to be weakened. Blast radius is bounded
     to "annoying", never "ships bad code". THIS is what makes using an untrusted external model safe.
  6. NETWORK: the codex-vm judge reaches the proxy + the compiled-app API only; no source tree, no ssb.db
     WRITE, no build-conductor control (mirrors the existing worktree allowed-write-set + no-commit rules).

================================================================================
5. NEW vs REUSE (the build ledger)
================================================================================

REUSE (already built, cited above): the wave engine, scheduler, leases, gate runner, integrator, control
plane, dashboard, SSB log (Devin conductor); the dormant codex/proxy worker lane (Devin conductor);
guard.py (Codex conductor, harvest); the ACCEPTANCE/DETERMINISTIC_CHECK rubric (plan files); CDP capture +
visual-ui-loop schema judge (.claude/tools/visual); B-37 runtime backend (for "any model under test").

NEW (build):
  N1. ACCEPTANCE task type + Wave 3.5 in the wave engine (re-judge only changed features).
  N2. The acceptance DRIVER per entrypoint family (CLI/IPC; dashboard-via-CDP; bench). Start CLI/IPC.
  N3. The SOURCE-STRIPPED judge context-builder mode (the air-gap control).
  N4. The judge DISPATCH on the reactivated codex/proxy lane + the strict return schema + unknown=fail.
  N5. guard.py harvested + wired as the boundary gate on the judge lane.
  N6. The NON-LOAD-BEARING verdict row (SSB action_type) + a dashboard acceptance tile.
  N7. Judge fallback ladder + (optional) skip-if-unchanged. The proxy is used as much as needed (no cost
      cap); the ONLY budgeting is a fallback: when the proxy credits run out, the judge falls back to 1 GLM
      taken from the conductor's free pool. Both the proxy judge and the GLM fallback get the SAME
      source-stripped packet + the SAME guard.py boundary scan, so the air-gap holds on either. Caveat: the
      GLM fallback is SAME-FAMILY as the glm-5.2 builder in the pool, so when it judges GLM-built work it is
      a weaker (correlated) check - acceptable as a temporary fallback, and mitigated by preferring to route
      the GLM judge at features NOT built by GLM. Skip-if-unchanged-since-last-green is kept only as an
      optional regression-efficiency, not a cost necessity.

================================================================================
6. FIRST SLICE + BUILD ORDER (smallest honest loop first)
================================================================================

  Slice 1 (recommended start, fully scriptable, no Tauri-binary problem):
    CLI/IPC feature -> drive real `elai` entrypoint -> assert the SSB row landed (this is L4 today) ->
    build the source-stripped packet -> dispatch to the air-gapped judge -> guard.py both directions ->
    non-load-bearing verdict row + tile. Proves the whole L5 loop on the easy entrypoint.
  Slice 2: add the dashboard-via-CDP driver (dev server :1420) + the visual-ui-loop schema judge for
    UI features. (Driving the FULLY PACKAGED Tauri binary headlessly is the hard part - DEFER it; the
    dev-server CDP leg covers the UI you watch.)
  Slice 3: add the benchmark driver (SAFEDELTA / swe-bench) feeding the same verdict ledger; no judge
    model needed for the numeric leg, only for the "is this number acceptable vs the plan target" call.
  Slice 4: "test any model" - parameterize the run-under-test through B-37 so the acceptance wave can run
    ELAI configured on model M and judge whether feature F still holds. Local models slot in here later
    when they are usable; today the judge is the proxy and the run-under-test is the cloud/free pool.

================================================================================
7. RESIDUAL HARD PROBLEMS (honest, not oversold)
================================================================================

  - THE ORACLE PROBLEM: a vague ACCEPTANCE block -> judge returns unknown -> unknown=fail. This is correct
    fail-closed behaviour but it means many plans need their ACCEPTANCE blocks tightened before they can be
    L5-judged. That tightening is itself worthwhile (it is the rubric wire-dark detection needs) but it is
    real upfront work, not free.
  - PACKAGED-TAURI HEADLESS DRIVING is genuinely hard (Tauri webview != plain browser). The dev-server CDP
    leg is tractable now; the packaged-binary leg is deferred and should not be promised early.
  - JUDGE COST: gpt-5.5 via the bmapi proxy is paid but used as much as needed (user decision 2026-06-26 -
    no cost cap). The only budgeting is the credit-exhaustion FALLBACK to 1 GLM from the free pool (N7).
    Note this raises the per-feature run count: E5 (pass^k) runs each acceptance test k times, so judge
    calls = features * k - fine under "use it freely", but it is latency, so the optional skip-if-unchanged
    keeps re-judging to changed features when throughput matters.
  - PROXY RESILIENCE: the codex/proxy lane needs the 524/stream-disconnect handling the proxy-gap note
    describes; the Devin conductor's transient-provider classifier (worker_adapter classify rate-limit +
    transient 5xx) already covers this for the codex lane if reused.

================================================================================
8. EVALS GROUNDING (Anthropic "Demystifying evals for AI agents" + LangSmith evaluation-concepts)
================================================================================

Both reads converge on the same discipline; mapped to L5 acceptance, the load-bearing additions:

  E1. VERIFY THE VERIFIER - the single most important add. Anthropic: "reference solutions prove
      solvability" - create a known-good output that passes all graders to confirm the GRADER is sound.
      So every acceptance test ships TWO planted fixtures: a known-PASS golden evidence set the judge MUST
      accept, and a known-FAIL set it MUST reject. If the judge mis-scores either, the JUDGE is broken (or
      gamed), not the feature. This is exactly the repo's own H-27 gate_self_validation.rs meta-gate
      (drive every gate with its own planted mutant, require 100% kill) applied to L5. Without it, a
      gameable/compromised judge silently greens everything - and our judge is an untrusted external model,
      so E1 is mandatory, not optional. It is also the defence that makes the air-gapped proxy SAFE to
      trust: we never trust the judge, we trust that it killed its planted mutants this run.

  E2. GRADE THE OUTCOME, NOT THE PATH (Anthropic: "Grade what the agent produced, not the path it took";
      outcome = final environment state, never the agent's self-report). L5 already does this: it reads the
      SSB row by id (the outcome), never the worker's "done". Do NOT add rigid step-sequence checks - they
      penalise valid approaches the plan author did not anticipate.

  E3. GIVE THE JUDGE A WAY OUT + GRADE DIMENSIONS SEPARATELY (Anthropic: instruct it to return "Unknown"
      when it lacks info; "Grade dimensions separately rather than having one judge evaluate everything").
      So decompose the ACCEPTANCE block into per-CRITERION judgments, each pass|fail|unknown (unknown=fail),
      one judge call per dimension - not one verdict for the whole feature. The LangSmith feedback shape
      {key, score, comment} is the per-criterion record.

  E4. BALANCED / NEGATIVE CASES (Anthropic: test both when a behaviour SHOULD and SHOULD NOT occur, or the
      agent learns to over-fire). Each feature gets a "fires when it should" case AND a "does NOT fire when
      it shouldn't" case. Catches the over-trigger failure a one-sided test would reward.

  E5. RELIABILITY METRIC = pass^k, NOT pass@k (Anthropic). Agent/feature behaviour is non-deterministic, so
      run each acceptance test k trials from a CLEAN worktree (E6) and require pass^k (ALL k pass) for a
      green. pass@k (at-least-one) hides flakiness; pass^k is the customer-facing reliability bar and is
      exactly the flaky-feature class manual testing catches.

  E6. ISOLATION PER TRIAL (Anthropic: start each trial from a clean environment; leftover files/caches
      cause correlated infra failures). Fresh worktree/build per trial - the conductor already does this.
      The source-stripped packet (no git history, no repo) also closes the Anthropic "agent cheats via
      cached git artifacts" exploit for the judge.

  E7. THE TWO-EXPERT RUBRIC BAR (Anthropic: "a good task is one where two domain experts would
      independently reach the same pass/fail verdict"). This is the concrete bar for the oracle problem
      (§7): an ACCEPTANCE block too vague to clear it is NOT L5-ready - tighten it before judging. A 0%
      acceptance rate across a family is, per Anthropic, "most often a signal of a broken task, not an
      incapable agent" - suspect the driver/rubric/judge before the code.

  E8. SEED FROM REAL FAILURES, DO NOT REVERSE-ENGINEER LATE (Anthropic: start with 20-50 tasks from real
      manual testing + bug reports). Seed the first acceptance set from gotchas.md, the FINISH punch lists,
      and recent bug reports - not invented from scratch. Watch for saturation: once a family clears ~80%,
      add harder cases or you are only measuring regressions.

  E9. FEW-SHOT + ITERATED JUDGE PROMPT (LangSmith: few-shot input/output/grade examples improve LLM-judge
      reliability; "design evaluators expecting careful review of scores and prompt tuning"). The judge
      prompt carries a few {evidence, rubric, verdict} examples and is tuned by reading scores, not fired
      once and trusted.

  E10. PAIRWISE FOR "TEST ANY MODEL" (LangSmith pairwise eval). To compare ELAI on model A vs model B for
       feature F (the §6 Slice-4 axis), pairwise-judge the two evidence sets - relative judgement ("which
       evidence better satisfies the rubric") is more reliable than two absolute scores.

  E11. OFFLINE NOW, ONLINE LATER (LangSmith offline=pre-deploy/dataset/regression vs online=production/
       reference-free/monitoring). L5 is an OFFLINE evaluator on the conductor's merged head (pre-ship
       regression over the feature "dataset"). A later ONLINE leg can score real ELAI production runs with
       the same reference-free judge.

  E12. CALIBRATE AGAINST HUMAN JUDGEMENT (Anthropic + LangSmith both: "you won't know if your graders work
       unless you read the transcripts and grades from many trials"; annotation queues with prescribed
       rubrics). Keep every judge transcript; the user periodically spot-reviews verdicts to confirm the
       judge tracks human judgement. This is the one irreducibly-human leg and it is cheap (spot-check, not
       every feature).

================================================================================
9. ONE-LINE ANSWER
================================================================================

The "fake human" is Wave 3.5 ACCEPTANCE: a deterministic harness drives the real feature and collects
off-agent evidence, then an air-gapped cross-family judge (the Chinese-proxy model in codex-vm, guard.py on
both sides of the boundary, fed a source-stripped packet) scores that evidence against the plan's ACCEPTANCE
rubric with pass|fail|unknown (unknown=fail) and a NON-LOAD-BEARING verdict. Built into the live Devin
conductor by harvesting guard.py + reactivating its dormant codex/proxy lane - not a second conductor, not a
merge. ~80% of the parts already exist; the new work is the acceptance driver, the source-stripped judge
packet, the judge dispatch+schema, and the verdict tile.

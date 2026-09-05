E2E Acceptance Tester (L5 "fake human") - build plan + as-built record

Date: 2026-06-26
Author: main session (implementation)
Design source: workflow/reasoning/E2E-ACCEPTANCE-TESTER-DESIGN-2026-06-26.md
Grounding: every file/line below was read this session before it was written against.

0. WHAT THIS IS
The L5 ACCEPTANCE layer (the "fake human") added to the LIVE Devin conductor at
F:\.ELAI_workflow\conductor. Per the design's topology decision it is built INTO the Devin
conductor by harvesting guard.py from the codex conductor and reactivating the dormant
codex/proxy worker lane as the air-gapped judge transport. Not a second conductor, not a merge.

1. THE EXACT PATTERN THIS MIRRORS (grounded)
The conductor already has 3 read-only stages built on one repeated shape:
  - a seed_X_tasks() that creates TaskSpecs for wave X            (verify_runtime.seed_verify_tasks)
  - a frozen decision function, NO worker judgment                (verify_runtime.assess_verify)
  - an injectable probe so the catch is testable w/o the binary   (entrypoint_runner / artifact_probe)
  - a config.limits.X_stage flag, default OFF (ships dark)         (conductor_config.Limits.verify_stage)
  - seeded in run_backlog_waves behind that flag                   (conductor_loop.py:1166)
  - NOT in WAVE_ORDER; leased via the saturate-borrow path         (waves.py:22-25)
L3/L4 VERIFY = drive real entrypoint + assert the off-agent SSB artifact landed (proves it RAN).
L5 ACCEPTANCE = drive real entrypoint + collect evidence by id + an independent air-gapped judge
scores that evidence against the plan's ACCEPTANCE rubric (proves it DID WHAT THE HUMAN WANTED).
L5 is additive on top of VERIFY; it never replaces a static gate. The verdict is NON-LOAD-BEARING.

2. KEY GROUNDED FACTS THAT SHAPED THE DESIGN
  - models.TaskSpec already carries acceptance, deterministic_check, verifier, production_hook =>
    the per-feature rubric already exists; nothing new on the plan side for the rubric itself.
  - ELAI SSB = ssb.db sqlite (tables ssb, ssb_files, seq-keyed) under .elai_cc/sessions/ =>
    the evidence the judge scores is READ CONDUCTOR-SIDE BY ID from ssb.db, never supplied by the
    judge (a compromised judge cannot fabricate a row that was never emitted).
  - worker_adapter.build_codex_command (worker_adapter.py:685) shells
    `wsl -d codex-vm codex exec --ephemeral --json --sandbox <ro|ww> --config <CODEX_HOME>/config.toml`
    => the air-gapped judge transport already exists; it is the dormant codex lane.
  - worker_slots CHECK is harness IN ('devin-cli') (migrations/001_initial.sql:53) => reactivating
    the codex lane needs the CHECK widened to allow 'codex'. (Code already branches on harness=='codex'.)
  - events table is append-only (triggers events_no_update/no_delete) => the natural ledger for a
    non-load-bearing verdict that must never be silently rewritten.
  - guard.py (codex-vm /root/.conductor-codex/conductor/guard.py) is stdlib-only, bidirectional
    (check_prompt injection-out / check_output dangerous-command+exfiltration+override-in) => drop-in.

3. TOPOLOGY DECISION (faithful to design §3): CONDUCTOR-SIDE STAGE, not a leased worker wave.
A leased worker runs inside a .elai_cc worktree and would SEE SOURCE, breaking the air-gap. The design
is explicit: "deterministic harness; only the final verdict is a model call." So the acceptance DRIVER,
evidence collection, packet build, and verdict record are all conductor-side code. The judge is the
ONLY model call, and it runs in a SCRATCH dir against a source-stripped packet. ACCEPTANCE is a
wave/task-type identity (WAVE_ACCEPTANCE) for the durable ledger + skip-if-unchanged, but it is NOT put
on the worker scheduler frontier; run_acceptance_pass() executes it directly in the backlog pass.

4. THE NEW MODULES (build ledger; maps to design N1-N7 + E1-E12)
  guard.py            (N5)  harvested + adapted; bidirectional regex scanner, stdlib-only.
  acceptance.py       (N1/N2/E1) the harness: AcceptanceContract parse (per-criterion rubric from the
                      ACCEPTANCE/observable_effect/trigger blocks), the CLI/IPC driver (slice 1: drive
                      `elai <subcommand>`, collect SSB rows by id via an INJECTABLE ssb_probe), the
                      frozen per-criterion assessment (unknown=fail), the E1 verify-the-verifier planted
                      PASS+FAIL fixtures (judge mis-scores either -> JUDGE_BROKEN -> discard, never pass),
                      candidate selection (DONE+VERIFIED plans, checkable rubric, skip-if-unchanged).
  acceptance_packet.py (N3) the SOURCE-STRIPPED judge context-builder mode: {feature_id, rubric verbatim,
                      redacted evidence text + image refs + bench numbers}, NO source, NO worktree, NO
                      repo paths. blake3 evidence hashes become the judge's evidence_refs.
  acceptance_judge.py (N4) guard.check_prompt -> dispatch (INJECTABLE judge_fn; production = codex/proxy
                      lane in a scratch dir, WIRE-DARK until the proxy is wired live) -> strict per-
                      criterion JSON schema (pass|fail|unknown, unknown=fail) -> guard.check_output
                      (hit -> discard -> NEEDS_REVIEW) -> fallback ladder (proxy -> 1 GLM, same packet +
                      same guard). E3 per-dimension judging, E4 negative cases honored by the contract.
  migrations/007_acceptance_verdicts.sql (N6) the non-load-bearing verdict store (feature_id, plan_id,
                      criterion, verdict, judge_model, evidence_refs_json, rationale, judged_commit,
                      run_at; load_bearing column literally 0). Skip-if-unchanged keys on judged_commit.

5. WIRING (each new public fn has a conductor-side caller in the same changeset)
  waves.py            + WAVE_ACCEPTANCE constant + alias (identity only; NOT in WAVE_ORDER).
  conductor_config.py + Limits.acceptance_stage (default False) + from-dict parse.
  conductor_loop.py   + run_acceptance_pass() called in run_backlog_waves behind acceptance_stage, after
                      the verify_stage seed (acceptance runs on VERIFIED plans).
  conductor_db.py     + record_acceptance_verdict() / last_acceptance_commit() / acceptance_verdicts().
  worker_adapter.py / migrations + harness CHECK widened to allow 'codex' (codex lane reactivation).
  dashboard_server.py + an acceptance tile reading acceptance_verdicts (display only).

6. TRUST MODEL (design §4, all preserved)
  source-stripped packet (primary IP control) + guard.py bidirectional + structural containment
  (rationale rendered as DATA, never concatenated into another prompt) + evidence read Windows-side by
  id + NON-LOAD-BEARING verdict (worst a fully-compromised judge does is a false reopen / false green,
  and a false green is caught by the independent static L1-L4 gates that already ran). Blast radius is
  bounded to "annoying", never "ships bad code".

7. EVALS DISCIPLINE WIRED (design §8)
  E1 verify-the-verifier planted fixtures (mandatory, the defence that makes an untrusted judge safe).
  E3 grade dimensions separately (per-criterion judge records). E4 balanced/negative cases (a
  "fires-when-it-should" + a "does-NOT-fire-when-it-shouldnt" criterion). unknown=fail throughout
  (E3 "give the judge a way out", scored as fail). E5 pass^k and E6 isolation are honored by running
  the deterministic driver k times from a clean collection; the judge call is k as well. E7 two-expert
  bar => a rubric too vague to yield a per-criterion verdict returns unknown=fail (forcing function to
  tighten ACCEPTANCE blocks). E8 seed from real failures: candidate selection prefers plans named in
  gotchas/FINISH punch lists (left as a selection hook).

8. RESIDUAL (honest)
  - Slice 1 only (CLI/IPC + SSB evidence). dashboard-via-CDP (slice 2) and bench (slice 3) drivers are
    registered as driver-family stubs with a clear NotImplemented seam, not faked.
  - The production codex/proxy judge dispatch is WIRE-DARK[E2E-ACCEPTANCE] until the proxy is wired
    live (same posture as verify_runtime's WIRE-DARK production probe); the injectable judge_fn makes
    the whole loop testable + provable now. The default-OFF acceptance_stage flag carries a
    wire-dark-allowlist intent (kill-switch/experiment, promotion = slice-2/3 + live proxy).
  - Packaged-Tauri headless driving is deferred per design §7.

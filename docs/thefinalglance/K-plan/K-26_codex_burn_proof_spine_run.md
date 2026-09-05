# K-26 — codex-credit burn run: proof-spine usage accrual + K-2 build (hard 5-day window)

PROOF_SPINE: feeds PS-2 (gate-tax ceilings locked from real `elai run` journals), PS-4 (K-2 routing-economics build; J-28 live leg finished + committed), PS-5 (TestOutcome red>green pairs accrue toward the case-reuse journal suite). Bonus: discharges J-28's deferred R10 live multi-turn smoke (it was deferred ONLY for lack of a live backend; gpt-5.5 is one).

DEADLINE: the bmapi codex-proxy credits expire 2026-07-15 (pool dashboard 2026-07-10: account-a ~5.4k/8k monthly bar, "resets in 5d 9h"; account-b is a second pool of the same provider). Every live-model leg here burns those credits and ZERO Anthropic tokens. After 07-15 the same legs re-run unchanged on another strong provider (the SP-3B smoke is already parameterized via ELAI_PROVIDER_LIVE_SMOKE_MODEL — "the gate outlives the credits").

STATUS: authored 2026-07-10, gate-checked + red-teamed (REDTEAM block at bottom). Not yet executed.

## WHY / GROUNDING (verified 2026-07-10, do not re-derive)

- gpt-5.5 is a NATIVE ELAI provider: `ProviderKind::Gpt55Proxy`, `detect_provider_kind("gpt-5.5")`, auth env `BMAPI_API_KEY` + `BMAPI_BASE_URL` (default `https://bmapi.020212.xyz/v1`) — crates/providers/src/client.rs:14-39, providers/mod.rs:142-145, providers/openai_compat.rs:24-78. Shipped at d91ec95. Construction live-verifies the model against bmapi `/v1/models`.
- TWO codex accounts exist on the codex-vm WSL distro: `/root/.codex-accounts/account-a` and `/root/.codex-accounts/account-b`. Both config.toml files are byte-identical in the provider block (base_url `https://bmapi.020212.xyz`, wire_api=responses); the per-account key lives in each dir's `auth.json` (root, 0600, NEVER copied into any repo). ELAI consumes ONE key at a time via env; rotation = swap `BMAPI_API_KEY`.
- The auditor-authored live smoke ALREADY EXISTS: crates/tools/tests/gpt55_proxy_live_smoke.rs (SP-3B). Gated `ELAI_PROVIDER_LIVE_SMOKE=1`, `#[ignore]`'d, drives the REAL entrypoint (`execute_tool("Agent")` -> ProviderClient::from_model -> real bmapi HTTP) and asserts an off-agent `PromptHardeningApplied` SSB row lands. Run recipe is in the file header.
- PS-2 and PS-5 are USAGE-GATED, not plan-gated (proof-spine ruling 2026-07-05; workflow/handoff/k18a-ps5-observe-arm.md). Their MEASURED numbers accrue only from real `elai run` journal rows. The 2026-07-09 user decision made `elai run` the DEFAULT lane for small self-contained .elai_cc fixes (dogfooding adopted).
- CONTAMINATION GUARD (user veto, 2026-07-09, never re-litigate): only ELAI's OWN runtime work enters ELAI's journal. No TEIKOKU/GLM/harness task is ever bridged into ssb.db. TEIKOKU codex-lane workers may be used as BUILD labor (MOVE 8) but their runs never touch ELAI's journal.
- J-28 (token ledger, PS-4 live leg) is BUILT + audited green except one UNVERIFIED one-line fix; 20 files dirty in .elai_cc, uncommitted. Full state: workflow/handoff/k18a-ps5-observe-arm.md CURRENT STATUS.
- RED-TEAM CORRECTION (2026-07-10, non-author auditor, verified live): `elai run` has NO model-selection seam today. `CliAction::Run` carries no model field (args.rs:193-217, parse_run_subcommand_args has zero model hits); `--provider` is a closed local|cloud|hybrid enum and `--provider cloud` is HARDCODED Anthropic-only (cloud_mode.rs:42 DEFAULT_CLOUD_PROVIDER="anthropic"; :345-350 rejects any other provider, "openai + byoe arrive in a follow-up" = the B-27 gap). The ONLY wired gpt-5.5 production entry is the Agent-tool handler's per-call `model` arg (crates/tools/src/lib.rs:4389, ProviderClient::from_model) — the exact path SP-3B smokes. Consequence: PS-2/PS-5 journal accrual is MODEL-AGNOSTIC (rows come from `elai run` driving the FSM, whoever the model is); the codex credits only decide who PAYS. MOVE 3 therefore forks (FORK-5).
- auth.json field name: `OPENAI_API_KEY` (verified; export its VALUE as BMAPI_API_KEY — the names differ).
- No gpt-5.5 row exists in the J-28 pricing catalog (usage_ledger.rs grep: zero hits) — VERIFY-6's fixture-only labeling is load-bearing.
- The SSB journal table is `ssb` (schema.rs:3294), not `actions`; action_type values `evidence_gate_evaluated` / `test_outcome` / `usage_turn_event` are real.

## BOX RULES (binding on every move)

- ONE cargo invocation at a time, ALWAYS `-j 1`. This box OOM-kills concurrent rustc (5 dead re-verify attempts on 2026-07-09). Check `Get-Process cargo,rustc` is empty before every cargo.
- The auditor agent is the sole test runner; it runs cargo via the PowerShell tool FOREGROUND, one at a time (sub-agents inherit the registered session FSM and the AUDITING Bash-hook blocks their cargo — the PowerShell tool is not hook-gated).
- Keys: sourced per-run from codex-vm auth.json, exported as env for the process only. NEVER written to any file in any repo. NEVER echoed into logs or commit messages.
- ONE commit per unit, maintainer voice, lowercase i, no AI trailer, no arrows. Nothing unaudited is ever committed.
- Every `elai run` dogfood task must be a REAL .elai_cc task (fix, wire, panel) — never a synthetic no-op prompt to farm rows. A farmed row corrupts PS-2/PS-5 permanently.

## MOVE 0 — preflight + recon (no credits spent except two 1-token probes)

Actions: (a) confirm box uncontended (`Get-Process cargo,rustc` empty); (b) read the account-a key: `wsl -d codex-vm cat /root/.codex-accounts/account-a/auth.json`, take the `OPENAI_API_KEY` field's VALUE, `$env:BMAPI_API_KEY = "<that value>"` (the names differ — this is an export-under-a-different-name, not a copy of the field name); (c) settle RECON-1..2 below; (d) probe each account with one minimal completion (curl the bmapi `/v1/models` or a max_tokens=1 chat call) to confirm both keys are live.
EXPECTED_OBSERVATION: no cargo/rustc processes; auth.json parses and yields the OPENAI_API_KEY value; both probes return HTTP 200; RECON-1..2 each resolve to a written one-line answer in workflow/reasoning/K-26-decisions.md.
ON_FAIL: probe 401/403 = that account's key is dead or rotated — re-read auth.json (the codex CLI may have refreshed it), retry once; still failing = drop that account from the plan and note it (counter-move: single-account run). Probe 429 = account rate-limited — wait 60s, retry once; persistent = start on the OTHER account (FORK-3 applies from MOVE 2 onward).
RECON_NEEDED-1: the blast radius of wiring a gpt-5.5 route into `elai run` (MOVE 3 FORK-5 Route A). Settle: `python .claude/tools/impact_radius.py --symbols validate_cloud_config,DEFAULT_CLOUD_PROVIDER --depth 2` + read cloud_mode.rs:330-360 + the dispatcher BackendMode::Api construction site. If the touched-file count > 5 or the change is structural (new trait plumbing, not a validation widen), Route A is OFF — take Route B, do not improvise.
RECON_NEEDED-2: account-b live health (it 429'd in the 2026-07-06 era per memory). Settle: the MOVE-0(d) probe on account-b's key.
(RECON-3 settled by red-team: auth.json field = OPENAI_API_KEY; export its value as BMAPI_API_KEY. RECON-4 settled: no gpt-5.5 pricing row exists; VERIFY-6 labeling mandatory.)

## MOVE 1 — finish J-28 (PS-4 prerequisite; carried verbatim from the k18a handoff)

Actions, in order: (1) `cargo test -p elai-cli -j 1` foreground via PowerShell from FSM=IDLE — the bare-usage regression test must pass (the 2 pre-existing unrelated fails, startup_banner Win tempdir race + bench_convergence seeded flake, are NOT J-28); (2) memory-writer agent (memory-bank updates + retry mcp_memorize topic=plan-template-bench-runner if RAG is healthy); (3) FEATURE-MAP upsert (J-28 entries: UsageTurnEvent 272, usage_ledger, `elai usage`, SpendAccumulator; PLUS the still-pending F-4 caveat from s-k4-exec — [WIRED] label kept, add "zero rows until dogfooded"); (4) ONE commit of the 20 J-28 files ONLY; auth_broker.rs is a SEPARATE tiny commit (it must never fold into the J-28 commit — auditor INFO-flagged); (5) push both.
EXPECTED_OBSERVATION: cargo exit 0 with `parse_args(["usage"])==Usage{json:false}` green; git log shows two new commits (J-28 unit, auth_broker one-liner); working tree clean except gitignored paths.
ON_FAIL: cargo OOM/killed again = the box is contended — stop all other builds, retry once with `cargo test -p elai-cli --lib -j 1` then the specific test bin split; a REAL test failure in the bare-usage regression = the one-line args.rs fix is wrong — re-read args.rs ~:7641 exemption list, fix in one EXECUTING pass, re-audit (auditor, not main LLM).
VERIFY: off-agent — the auditor re-runs the failing-then-passing test and reports the exit code; commit hashes exist in `git log -2 --oneline`.

## MOVE 2 — SP-3B provider live smoke on account-a (first credit spend, ~1 task)

Actions: per the smoke header: `$env:ELAI_PROVIDER_LIVE_SMOKE="1"; $env:BMAPI_API_KEY="<account-a key>"; cargo test -p elai-tools --test gpt55_proxy_live_smoke -- --ignored --nocapture --test-threads=1`.
EXPECTED_OBSERVATION: test passes; output shows a real bmapi round trip; the smoke's own SSB query finds the PromptHardeningApplied row keyed by this run's agent_id (db-not-found or zero rows = FAIL by the smoke's design, never SKIP).
ON_FAIL: model-presence check fails at construction = bmapi `/v1/models` doesn't list gpt-5.5 for this key — cause: pool model set changed; counter-move: list `/v1/models` with the key, pin ELAI_PROVIDER_LIVE_SMOKE_MODEL to a listed model, note the drift. HTTP 429 storm = FORK-3 (rotate to account-b). Compile error = J-28's dirty tree broke elai-tools — MOVE 1 must complete first (this move is ordered after it on purpose).
VERIFY: off-agent — the smoke's SSB row assertion IS the artifact (agent did not author it); exit code 0.

## MOVE 3 — the dogfood engine: real .elai_cc fixes through `elai run` (main accrual, all 5 days)

Dogfooding is the ONLY thing that buys PS-2/PS-5 MEASURED numbers — no build substitutes for it. The accrual is MODEL-AGNOSTIC (the journal rows come from `elai run` driving the FSM); the codex credits only decide who pays. Route the model per FORK-5.
FORK-5 (the seam fork, settled by RECON-1 at MOVE 0): `elai run` cannot select gpt-5.5 today (see GROUNDING: B-27 Anthropic-only validation at cloud_mode.rs:345-350).
  Route A (preferred IF RECON-1 says the widen is small, <= 5 files, non-structural): dogfood task #0 = wire a gpt-5.5/openai-compat route into `elai run`'s cloud path — widen the B-27 provider validation to accept a model whose detect_provider_kind resolves (ProviderClient::from_model already handles gpt-5.5; the validation reject is the blocker). PRODUCTION_HOOK: crates/orchestrator/src/cloud_mode.rs provider validation (:345-350) + the dispatcher BackendMode::Api client-construction site; smoke = the SP-3B live smoke re-run PLUS one real `elai run --provider cloud` turn on gpt-5.5 landing SSB rows. Bootstrap cost: task #0 itself runs on the existing default lane; every task after it is credit-funded. This unit follows the full walking-skeleton shape (producer + consumer + smoke same changeset) and is audited like any code unit.
  Route B (RECON-1 says structural, or task #0 goes 2 failed attempts — D-9 cap): dogfood on the EXISTING lane (local sidecar or anthropic default) — PS-2/PS-5 rows accrue identically — and burn the codex credits through the per-call Agent-tool leg instead (subagent spawns with model:"gpt-5.5", the SP-3B-proven path: use gpt-5.5 subagents as the executor labor inside dogfood sessions) + MOVE 8. Do NOT force Route A past the cap; the numbers do not need gpt-5.5.
Seed queue (all real, all grounded in prior audits/handoffs; pull more from gotchas.md + the auditor WARN backlog as these drain):
  0. (Route A only) the gpt-5.5 `elai run` seam widen, per FORK-5 above.
  1. crates/cli/src/usage_cmd.rs print_json: hand-built format! JSON (no escaping) -> serde_json::to_string (J-28 auditor WARN).
  2. Dashboard panel for RecurrenceHitRateMeasured (SSB row shipped at 270; no panel — queued K-18a follow-on).
  3. Dashboard tile for GateTaxMeasured (row shipped at 271; no tile).
  4. startup_banner Windows tempdir race (pre-existing flaky test, documented in 2 audit rounds).
  5. bench_convergence seeded flake (same).
  6. auth_broker SSB-emit seam: broker_resolve_anthropic_auth_with_ssb still driven with writer=None at runtime_cmd.rs:395 (the I-50 allowlist row's registered seam) — wire the real writer.
Per task: (a) run it through the REAL entrypoint `elai run` on the FORK-5 route's lane (account-a key funds whichever leg is credit-funded); (b) let ELAI's own FSM/gates/tests drive; (c) after each task, verify journal accrual (VERIFY below); (d) log one line per task in workflow/reasoning/K-26-decisions.md (task, outcome, rows delta, which lane paid).
EXPECTED_OBSERVATION: per task — `elai run` completes (PASS or an honest gate BLOCK; early stumbles are DATA per the dogfooding decision, surfaced not hidden); ssb.db row counts strictly increase for EvidenceGateEvaluated and (on tasks with a test cycle) TestOutcome; UsageTurnEvent rows carry nonzero token counts.
ON_FAIL: `elai run` crashes/wedges on a real task = a genuine product bug — that IS the dogfooding payoff: capture it, file it to gotchas.md, fix it as the NEXT dogfood task (through `elai run` if it can still drive, else directly + honestly note the journal gap). Zero new journal rows after a completed run = an emit-path bug (R2 dead evidence path) — STOP the burn, diagnose the emit seam before any further runs (rows that silently don't land make the whole burn worthless). 429/quota = FORK-3.
VERIFY: off-agent, after every task — `sqlite3 .elai_cc/sessions/ssb.db "SELECT action_type, COUNT(*) FROM ssb WHERE action_type IN ('evidence_gate_evaluated','test_outcome','usage_turn_event') GROUP BY action_type"` (table name `ssb` verified against schema.rs:3294; action_type values verified real) — counts monotonically increase across tasks. The journal rows are ELAI-authored regardless of the backend model, and the model pin is recorded per row: honest.

## MOVE 4 — PS-2 leg: gate-tax on real journals + ceiling lock

Trigger: after >= 5 completed dogfood tasks (enough fire-counts that the composed number is not a RunnerPresent zero).
Actions: `.\target\debug\elai.exe bench gate-tax --seed 1234 --json` (offline, no credits); read coding_task_gate_tax + audit_gate_tax off the report; then surface BOTH ceiling values to the user (FORK-1) — ceilings are USER-SET from the first real run per K-4 (ABORT-5 there: fail-closed while CEILING-PENDING).
EXPECTED_OBSERVATION: runner_status flips RunnerPresent -> a measured nonzero coding_task_gate_tax computed from real journal fire-counts; a GateTaxMeasured SSB row lands; `--enforce-ceiling` still exits nonzero (CEILING-PENDING) until the user locks values.
ON_FAIL: still RunnerPresent/zero after 5+ tasks = the journal rows aren't the ones Leg-B counts (query_by_type mismatch) — diagnose against K-4's Leg-B query list before running more tasks; do NOT fabricate a ceiling.
FORK-1: ceiling values are the USER's call. Trigger: the first measured report. Present the two numbers + a suggested ceiling (1.5x measured); the user locks or adjusts; then wire the locked values and re-run `--enforce-ceiling` expecting exit 0.
VERIFY: off-agent — the GateTaxMeasured row in ssb.db (sqlite3 count delta) + the bench exit codes.

## MOVE 5 — PS-5 leg: journal probe + conditional --suite journal build

Trigger: after >= 8-10 dogfood tasks with red>green test cycles.
Actions: re-run the R-A probe (exact query trail + re-check command: workflow/handoff/k18a-journal-suite.md) counting distinct bug-class pairs in ssb.db.
FORK-2: probe result decides the route. >= 10 distinct pairs -> BUILD `--suite journal` per the staged Wave-3 spec (journal loader read-only from the db, deterministic ~50/50 held-out split, floor re-tuned on the tune half — NEVER the fixture 11.6 — report hit_rate@k + false_match_rate@k on held-out, emit RecurrenceHitRateMeasured with suite="journal"). < 10 pairs -> keep dogfooding (MOVE 3 continues); PS-5 stays honestly RUNNER_PRESENT this window and the probe re-runs next window. Do NOT lower the pair threshold to force the build.
PRODUCTION_HOOK: crates/cli/src/bench_case_reuse.rs — the existing `elai bench case-reuse --suite journal` rejection path becomes the real dispatch (the CLI verb, parser arm, and beacon emit already exist from K-18a; this unit swaps the rejection for the loader+split+report and reuses emit_beacon_and_rra).
EXPECTED_OBSERVATION (build route): suite=journal runs green on the accrued db; same-seed byte-identical report; RecurrenceHitRateMeasured row with suite="journal" lands; held-out hit-rate reported with CI.
ON_FAIL (build route): loader finds pairs but the classifier's trigger substrings don't match real failure text = the fixture-tuned classifier is corpus-biased — record the mismatch as a K-18 Task-2 finding, do not hand-tune labels to make the number.
VERIFY: off-agent — auditor runs the suite + production_caller_check + the byte-identity check; the SSB beacon row is the artifact.

## MOVE 6 — K-2 routing-economics build (PS-4, the one remaining buildable spine unit)

Actions: (1) pre-flight `python .claude/tools/plan_gate_check.py --file docs/thefinalglance/K-plan/K-2_routing_economics_bench.md` — it is a legacy plan and will FAIL; (2) run the 9-step plan-improvement protocol on K-2 (inject the wargame blocks, re-ground its anchors against the LIVE tree — J-28's usage_ledger/UsageTurnEvent at schema 272 now exist and are its live-leg input; anchor drift is CERTAIN, grep before trusting any line number in it); (3) re-gate to PASS; (4) execute via the standard orchestrate FSM cycle (context-compiler + integration-explorer parallel cap 3 -> executor -> auditor with the full off-agent wire stack -> memory-writer), ONE commit at end. Fleet cost announced to the user before dispatch (fan-out rule).
PRODUCTION_HOOK: crates/cli/src/args_bench.rs — `elai bench routing-economics` parser+dispatch arm, mirroring the shipped gate-tax/case-reuse precedent (bench_cmd wrapper + a bench_routing_economics.rs handler + an SSB beacon action_type via the walled-off schema-bump path). The refined K-2 plan names the final seam; this is the grounded default.
EXPECTED_OBSERVATION: K-2 gate-PASS 10/10 after refine; build audited green; `elai bench routing-economics` produces a $/verified-task report recomputable from the journal; beacon row lands.
ON_FAIL: refine reveals K-2 needs live cloud-vs-hybrid runs this window cannot fund honestly = ship the runner + fixture arm only, leave the live arm usage-gated like PS-2/PS-5 (RunnerPresent is an honest verdict; a fabricated economics number is not).
VERIFY: off-agent — auditor's checker suite (production_caller_check 0 DARK, gate_freeze OK, maintainability 0 BLOCK) + the R10 smoke driving the real CLI verb + the SSB row.
PS-4 HONESTY CAVEAT (VERIFY-6): bmapi credit runs are free — any $/verified-task arm computed off gpt-5.5 usage is labeled fixture/plumbing, never the headline cloud-economics number. The headline number needs real-priced lanes (per RECON-4's catalog answer).

## MOVE 7 — account rotation

FORK-3: trigger = account-a returns 429/quota-exhausted twice in a row, OR its dashboard shows < 500 credits remaining. Route: swap `$env:BMAPI_API_KEY` to the account-b key (from `/root/.codex-accounts/account-b/auth.json`), re-run the MOVE-0(d) probe, continue the current move. Both accounts hit the same bmapi base; nothing else changes. Log the swap (timestamp + which move) in K-26-decisions.md — the journal's model pin does not distinguish accounts, so the decisions log is the account ledger.
EXPECTED_OBSERVATION: post-swap probe 200; the interrupted task re-runs clean.
ON_FAIL: account-b also dead (RECON-2 said it 429'd before) = the burn is over early — finish MOVE 4/5/6 legs that need no credits (gate-tax and the K-2 fixture arm are offline), write the handoff.

## MOVE 8 — surplus burn (optional, only if credits remain after MOVE 3-6 are saturated)

Route surplus credits at, in order of preference: (a) MORE dogfood volume (never wasted — every task compounds PS-2/PS-5); (b) the J-28 R10 live multi-turn/compaction smoke as its own dedicated run (a long multi-turn `elai run` session on the FORK-5 lane that crosses the compaction threshold — this discharges the deferred T7 cross-check WITH its WIRE-DARK[J-28] stub if compaction fires); (c) TEIKOKU codex-lane workers (codex_provider.py, per-account CODEX_HOME) as BUILD labor on L-1/L-2 (the CGG track start, PS-3) — greenfield, expect the capability wall the glm walk hit; cap it at 2 attempts per plan (D-9), ABORT the lane on slop, and remember: TEIKOKU worker output lands via its own gates + git, NEVER via ELAI's journal (contamination guard).
EXPECTED_OBSERVATION: (b) compaction fires mid-run and usage_ledger growth rows reconcile with per-turn sums.
ON_FAIL: (c) codex workers produce NEEDS_REVIEW slop = stop the lane, credits go back to (a).

## INTEGRATION_SMOKE

Two smokes, both driving REAL entrypoints on HARD inputs (R10):
1. After MOVE 2 (early): the SP-3B live smoke (real Agent dispatch -> real bmapi HTTP -> off-agent SSB row). This is the seam proof that provider + SSB + credits all compose before the main burn starts.
2. Before DONE (late): one long multi-turn `elai run` session on the FORK-5 lane driving a real multi-step .elai_cc task end to end (plan -> test -> execute -> audit path), long enough to fire secondary paths (compaction per MOVE 8(b) if reachable), asserting: task-scoped SSB rows landed (EvidenceGateEvaluated + TestOutcome + UsageTurnEvent for this session id), `elai usage` renders the session rollup, and `elai bench gate-tax` recomputes over a journal that includes this run. A toy single-turn happy-path run does NOT satisfy this.

## ABORT (stop and hand off, never improvise)

ABORT-1: both accounts dead/exhausted before MOVE 3 saturates — finish the offline legs, write the handoff to workflow/handoff/k18a-ps5-observe-arm.md (this campaign appends there; the topic is the spine numbers).
ABORT-2: journal rows silently not landing after completed runs (R2 dead-evidence-path) — no further burn until the emit seam is diagnosed; farmed-looking rows are worse than no rows.
ABORT-3: cargo OOM twice in a row on the serial protocol — stop builds, dogfood-only until the box is uncontended.
ABORT-4: any temptation to bridge TEIKOKU/GLM/harness work into ELAI's journal — categorically vetoed, no exception this window.
ABORT-5: a schema bump needed anywhere outside MOVE 5/6's planned beacon path — walled-off change, own session, not this campaign.

## FORKS

FORK-1 (MOVE 4): ceiling values — user locks from the first measured report.
FORK-2 (MOVE 5): journal pair count >= 10 -> build the suite; < 10 -> keep accruing, no build.
FORK-3 (MOVE 0/7): account health -> which key serves BMAPI_API_KEY; deterministic trigger = 2x 429 or < 500 credits.
FORK-4 (MOVE 6): K-2 refine outcome -> full build vs runner+fixture-arm-only (trigger: whether an honest live arm is fundable this window).
FORK-5 (MOVE 3): the `elai run` gpt-5.5 seam — Route A (wire it, RECON-1 says <= 5 files non-structural) vs Route B (existing lane + per-call Agent-tool gpt-5.5 subagents; trigger: RECON-1 structural OR task #0 fails twice, D-9 cap).

## VERIFY (campaign-level, all off-agent)

VERIFY-1: J-28 commit hashes in git log; the regression test's exit 0 (auditor-run).
VERIFY-2: SP-3B smoke exit 0 + its SSB row (the smoke authors neither).
VERIFY-3: monotone ssb.db row-count deltas per dogfood task (sqlite3, exact query pinned at first run).
VERIFY-4: GateTaxMeasured + (conditional) RecurrenceHitRateMeasured(suite=journal) + (conditional) the K-2 beacon rows in ssb.db.
VERIFY-5: frozen-checker exit codes on every code unit (production_caller_check, gate_freeze_check, maintainability_check) — auditor-run, never executor-self-run as authority.
VERIFY-6: any economics number computed this window carries the free-credit label (PS-4 honesty caveat).
VERIFY-7: proof_spine_status.py at campaign end — expected end-state: PS-2 MEASURED-or-ceiling-locked, PS-5 RUNNER_PRESENT-with-accruing-journal (or MEASURED if FORK-2 built), PS-4 runner shipped, PS-6 unchanged MEASURED. (Advisory read only — never a done-gate.)

## OUT OF SCOPE (declared so the executor never drifts)

- PS-1 (safedelta): hard-gated on ELAI_E2E_POD_ENDPOINT (RunPod) — safedelta_runner.rs:91-97 fail-closed. Codex credits cannot substitute; separate spend, separate session.
- PS-3 (L-10): needs L-1..L-9 built; only MOVE 8(c) touches its start, best-effort.
- Any harness (.claude/) fix — the 3-fix harness unit from the k18a handoff stays queued separately.

REDTEAM (2026-07-10, non-author adversarial-auditor mode=plan, sonnet, tool-backed; verdict on v1 = PLAN-BLOCK; all patches applied in-place above; immutable record):
- ATTACK 1 SUCCEEDED (the BLOCK): "dogfood through `elai run` on gpt-5.5" had no implemented seam. Verified: CliAction::Run has no model field (args.rs:193-217; parse_run_subcommand_args zero model hits); --provider is a closed local|cloud|hybrid enum; cloud_mode.rs:345-350 hard-rejects any provider but "anthropic" (B-27); cfg.default_model is cost-estimate-only, never provider selection; the only wired gpt-5.5 production entry is the Agent-tool per-call model arg (tools/lib.rs:4389). v1's RECON-1 presupposed a seam exists. PATCH: FORK-5 (Route A widen-the-validation as dogfood task #0 with a named PRODUCTION_HOOK, bounded by RECON-1 blast-radius + D-9 2-attempt cap / Route B existing-lane accrual + per-call gpt-5.5 subagent burn), plus the GROUNDING correction that PS-2/PS-5 accrual is model-agnostic and credits only decide who pays.
- ATTACK 2 SUCCEEDED (WARN): VERIFY-3's sqlite query named table `actions`; the live table is `ssb` (schema.rs:3294). PATCH: query pinned to `FROM ssb` now, not "adjust on first run"; action_type values confirmed real (test_outcome :3301, evidence_gate_evaluated :3349, usage_turn_event :5487).
- ATTACK 3 FAILED (plan survives): account topology — both /root/.codex-accounts/{account-a,account-b} exist, auth.json present, config.toml provider blocks byte-identical (diff), base_url + wire_api as claimed.
- ATTACK 4 FAILED (and settled RECON-3): tried to show the auth.json recon was redundant; it is not — the field is OPENAI_API_KEY, not BMAPI_API_KEY, so the export-under-a-different-name step is genuinely necessary.
- ATTACK 5 FAILED — the strongest surviving attack: tried to show RECON-4 (pricing row) was defensive padding; grep of usage_ledger.rs found ZERO gpt-5.5 pricing rows, so VERIFY-6's fixture-only labeling is load-bearing, not theater. The plan's caution was warranted.
- ATTACK 6 FAILED: bench_case_reuse.rs:68 rejects --suite journal exactly as claimed (matching test :487); K-2 plan file exists. Anchors live.
- ATTACK 7 FAILED: the dirty auth_broker.rs is a 3-line comment-only deletion; the load-bearing WIRE-DARK[I-50] marker lives at runtime_cmd.rs:391-395 and seed task #6 stays correctly scoped; the MOVE-1 commit-split instruction is unaffected.

AUTONOMOUS ORCHESTRATE RUN — ELAI cold-crate wiring / integration (phase-F-deconflicted)

Paste this entire message as the first message of a fresh `claude` session whose CWD is /workspace/ELAI. The session's native CLAUDE.md, .claude/rules, .claude/agents, .claude/skills and .mcp.json load automatically.

=== WHAT YOU ARE ===
You are an autonomous EXECUTOR running the orchestrate skill in SAME-PROJECT mode (target == host == ELAI). You adapt .claude/skills/orchestrate (SKILL.md + protocol.md per-sub-phase loop) with the override policies below. You close cold-crate wiring/integration debt: config structs that are currently env-only or hardcoded, and missing deterministic test coverage, in crates that are NOT part of phase F and are clean on the live tree. Optimize for THOROUGHNESS and coverage — work through the full cold-crate wiring backlog. This is a deliberate deep-improvement run.

Load these skills before starting: orchestrate, planning-workflow, impact-check. Do NOT load sidecar-protocol (no model inference in this run).

=== DECONFLICTION (LOAD-BEARING — READ FIRST) ===
Another live session, s-60824 ("F-plan continuation orchestrate"), owns ~6000 lines of uncommitted phase-F work on this same tree. You must NOT collide with it.

D1. FIRST ACTION, before any edit: run `git status --porcelain` and `git diff --stat HEAD` from /workspace/ELAI. Build your own HOT (dirty) vs COLD (clean) crate/file map from the live output. Re-derive it; do not trust any snapshot blindly.
D2. HARD-AVOID crates (hot, s-60824's): orchestrator, cli, contracts, dashboard-v2, rules, plugins, and the python checkers. Never edit a file under these. Specifically never touch ssb.rs, task_runner.rs, context_budget.rs, fsm.rs, evidence.rs, autonomous_gate.rs, compiled_pipeline.rs, hook_dispatcher.rs, and the cli loaders args.rs / main.rs / live_cli.rs / runtime_config.rs / knowledge_stack.rs / safety_cmd.rs. This HARD-AVOID list is an AUTHORITATIVE OVERRIDE — these crates and files (including the cli loaders knowledge_stack.rs and runtime_config.rs) are HOT and forbidden EVEN IF git status shows them clean. The D1 live re-derivation may only ADD crates to the avoid set; it never removes one named here.
D3. COLD-CRATE SCOPE (empirically verified via `cargo check -p` on the live tree — these are cargo PACKAGE names, use with -p):
   TIER-1 (fully insulated; pull only clean elai-core / elai-ipc-types; cannot be broken by concurrent HOT-crate edits): elai-core, elai-providers, elai-safety.
   TIER-2 (SAFE-but-coupled; W1-W4 live here; transitively depend on dirty elai-rules/elai-plugins (knowledge) and elai-orchestrator/elai-plugins (tools), so their scoped cargo check CAN fail through no fault of yours if a concurrent session breaks those HOT crates — droppable-at-runtime by V0): elai-knowledge, elai-tools.
   DROPPED: elai-eval-corpus (direct dirty-orchestrator dep, no W item targets it — only collapse surface, no payoff). elai-memory (zero-dirty + compiles, but its src/ IS the future F-5 surface — see W6).
   Edit ONLY inside a crate confirmed (a) zero-dirty in git status AND (b) compiles standalone via `cargo check -p <package>` in V0. Drop and note any candidate that fails either.
D4. CONFIG SPLICE BOUNDARY: many wiring items have their consumer in a cold crate but their final config-loader line in a HOT cli file (runtime_config.rs / knowledge_stack.rs). Do the COLD half only: add the serde-Deserialize config struct/field + make the crate read it (env still wins for now) + the crate's own unit test. STOP before the cli loader splice. Register the deferred splice as a stub `// TODO[wiring.<crate>.<field>]: cli loader splice deferred until s-60824 commits` with a row in your stubs.md (will_resolve_in = post-s-60824).
D5. NEVER run `git commit`, `git add`, `git stash`, `git checkout --`, `git reset`, or `git restore`. The tree holds s-60824's uncommitted work; any of these clobbers it. This OVERRIDES orchestrate POLICY 4 + protocol step 11. Accumulate edits; hand off uncommitted.
D6. SESSION SOVEREIGNTY: mcp_session_register for YOUR session, then mcp_session_list to read s-60824's locks. Claim only the cold files you edit. Never mutate another session's FSM/registry.

=== CANDIDATE WORK ITEMS (grounded from a prior survey; re-verify each against live source) ===
W1. crates/knowledge/src/hyde.rs — add an `nl_only` field to the HyDE config struct defaulting to the ELAI_HYDE_NL_ONLY env value; knowledge-crate unit test asserts env-override precedence.
W2. crates/knowledge/src/vector/indexer.rs (EXACT path; the sibling crates/knowledge/src/artifacts/indexer.rs is a DECOY, not the target) — add a `code_vec.enabled` config field (default = ELAI_CODE_VEC env) for the GraphCodeBERT secondary index; crate test.
W3. crates/knowledge/src/deep.rs + pipeline.rs — replace hardcoded DEEP-tier constants (max_candidates, rrf_floor, rrf weights) with a serde-Deserialize KnowledgeDeepConfig (Default = current consts); crate test.
W4. crates/tools/src/routing_rules.rs + lib.rs — NOTE: RoutingConfig (enabled + allow_override_prefix), a runtime_routing_config() accessor, and unwrap_or(OVERRIDE_PREFIX) fallbacks ALREADY EXIST (.unwrap_or is a safe Option method, not banned). Verify check_routing / strip_override_prefix already honor RoutingConfig (they do); add ONLY the missing thin check_routing_with_config(cmd, &RoutingConfig) wrapper + a crate test asserting config override beats the OVERRIDE_PREFIX const. Skip entirely if already covered.
W5. crates/eval-corpus/src — add deterministic CPU-only unit tests + fixtures for the scorecard/result parsers and fixture loaders. EXCLUDE every TODO[E-54.x] marked RUNPOD-PENDING and anything needing SearXNG / a live sidecar / the (nonexistent) sandbox crate.
W6. crates/safety ONLY — add deterministic unit tests for public functions lacking #[cfg(test)] coverage (verified safety/src/ files, NONE are F-5: capability_gate.rs, destructive_verb_allowlist.rs, injection_classifier.rs, prompt_armor.rs, provenance.rs, spotlight.rs, tool_description_pin.rs, trust_dialog.rs). crates/memory is REMOVED from free-edit scope: its src/ IS the future F-5 surface (verified F-5 files: decay.rs, decay_scheduler.rs, contradiction.rs, evolution.rs, defense.rs, mem0_dispatcher.rs — note quarantine_broadcast.rs does NOT exist anywhere in the workspace). If you add memory tests at all, HARD-RESTRICT to this non-F-5 allowlist ONLY: fact.rs, trust.rs, view.rs, notes.rs, retrieval.rs, store.rs, extractor.rs, core_block.rs. This collision is with a FUTURE F-5 session, NOT s-60824.
Treat W1-W6 as the seed backlog; the discovery phase may add more cold-crate items it finds. Each item touches 1-3 files and is independent.

=== OVERRIDE POLICIES (precedence over native skill defaults) ===
P1. PARALLEL CAP 3 per Agent-tool batch.
P2. SKIP prompt-engineer / prompt-transform.
P3. ONE auditor cycle per sub-phase; on BLOCK apply bubble-up retry budget (3 retries), no AUDITING<->EXECUTING ping-pong.
P4. NO COMMIT (see D5). Accumulate; hand off uncommitted.
P5. AUTONOMOUS = true. Do not ask the user. Best grounded non-lazy option, continue. Only the 4 tripwires interrupt. ADDED tripwire T5: if a needed edit lands on a HOT file or the cli splice boundary, do NOT surface — defer + register stub + document, move on.
P6. SCOPED VERIFICATION ONLY (overrides orchestrate steps 9-10 workspace gates). The full workspace does NOT build right now (s-60824 contamination), so NEVER run cargo build/test --workspace, cargo clippy --workspace, cargo public-api workspace diff, or the real-model test. Per cold crate touched:
    Push-Location /workspace/ELAI\.elai_cc
    cargo check -p <crate> --all-features         # compile gate (cargo check is permitted)
    cargo test  -p <crate> <new_test_names>       # run ONLY the new tests you authored in this crate (see TEST POLICY)
    Pop-Location
   TEST POLICY (explicit permission for THIS run): scoped `cargo test -p <crate>` restricted to the new tests you authored is PERMITTED. Full-crate and workspace test runs are NOT. If you prefer compile-only, downgrade to `cargo check -p <crate>` and mark the test as authored-but-unrun in the handoff. Foreground all cargo runs (no run_in_background).
P7. NO SIDECAR. No model inference, no sidecar boot.
P8. COLD-CRATE MANIFEST GUARD: if a wiring struct needs serde derive (#[derive(Deserialize)]), FIRST confirm the cold crate already enables serde with the `derive` feature; do NOT add or change any dependency or feature without anti-slopsquatting verification, and call out any Cargo.toml edit explicitly in the handoff (it dirties a manifest the operator hands off).

=== FLOW ===
V0 — COLD-CRATE BASELINE (do this before authoring the plan):
  Run `cargo check -p <package>` for each D3 candidate (TIER-1 + TIER-2) on the current tree. Keep only the crates that compile clean standalone. NOTE: cargo check -p on TIER-2 elai-knowledge/elai-tools intentionally compiles their dirty HOT deps (elai-rules / elai-plugins / elai-orchestrator); a hot-dep compile failure there is EXPECTED contamination, not a tooling error on your part — drop the crate and record why. If V0 drops BOTH elai-knowledge AND elai-tools, the W1-W4 seed items are ALL out of scope — record this in orchestrate-final.md as RUN_SCOPE_COLLAPSED and proceed with TIER-1 only (W6 safety tests + any cold items discovery finds in core/providers); do NOT report a no-op on the named backlog as a success. The surviving set is your real scope.
Phase 0 — DISCOVERY + PLAN AUTHORING (no existing plan file exists):
  0.1 mcp_fsm_get_state; set mcp_think budget complex (8 / 1500) per topic (prompt-engineer skipped).
  0.2 ONE batch (cap 3): context-compiler scoped to the surviving cold crates + integration-explorer-elai for each config consumer's wiring (where the field is read, what the env path is). Before any edit batch, run impact-check / impact-mapper on the target files (blast radius). Canonical ELAI dispatch: subagent_type=general-purpose, "Read /workspace/ELAI\.claude\agents\<name>.md and follow it strictly. <args>".
  0.3 Author a scoped plan at workflow/sessions/<your_sid>/cold-wiring-plan.md: Phase N > sub-Phase > Task, one sub-phase per work item (W1..Wk + any discovered), each with a WIRING_CONTRACT (produces_symbols = the new config struct/field; consumes = env path; observable_effect = "field present + env override test passes; cli splice deferred"), DETERMINISTIC_CHECK (the crate test command), ACCEPTANCE, and an explicit "cli-splice deferred" note per D4.
Phase 1..k — ORCHESTRATE LOOP (protocol.md step 8 per sub-phase):
  PARALLEL BATCH (context-compiler + integration-explorer-elai scoped to the item) -> impact-mapper if not already cached -> SKIP prompt-engineer -> EXECUTING (cold-half only; register the deferred cli stub) -> AUDITING (auditor 1 cycle; test policy = scoped cargo check + the new test only) -> WIRING_CONTRACT_VALIDATE via wiring-contract-validator (cold-half contract) -> MEMORIZING (memory-writer scoped to item) -> per-sub-phase scoped verification (P6) -> advance.
FINAL:
  Re-run `cargo check -p <crate>` for every touched cold crate (all green). Confirm zero edits landed in any HOT crate (git diff --name-only against your map). NO commit (D5). Write orchestrate-final.md + SESSION_HANDOFF.md listing: every file changed (uncommitted), every deferred cli-splice stub (with the exact loader line to add post-s-60824), and the dropped candidates from V0 with reasons. End in FSM IDLE.

=== GROUNDING + HARD RULES (native, restated) ===
Grounded-by-default. Anti-stale: blake3 not blake2/sha2; rand>=0.9; tracing not log; thiserror/anyhow not Box<dyn Error>; native async fn in traits not async-trait; no new deps without confirmation (anti-slopsquatting verify any crate name first). No .unwrap()/.expect()/try!/as! in non-test source (? / ok_or / propagation). Zero-lazy-code: every stub registered with a babystep id + stubs.md row. Token discipline: delegate reads to agents. Foreground all cargo runs. Anti-spiral auto-fires; observe it.

START: emit your session id, mcp_fsm_get_state, register session, run D1 git status, run V0 cold-crate baseline, then Phase 0.

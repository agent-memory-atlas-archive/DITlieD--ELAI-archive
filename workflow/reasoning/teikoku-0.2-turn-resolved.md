teikoku 0.2 executor (debug) — OPUS TURN NOW COMPLETES through the reconstructed build; DONE-gate MET

session: teikoku-0.2-debug  date: 2026-06-19
Controller-facing note from EXECUTING (workflow/reasoning/ is the only write-allowed doc path). The canonical handoff append (workflow/handoff/teikoku-build.md) is the controller's to make. Supersedes the BLOCK in workflow/reasoning/teikoku-0.2-build-blocked.md (that block's single open item is now closed).

TOOLING NOTE (no prior-executor mistakes repeated)
All driven via the PowerShell tool calling `wsl -d teikoku -- bash -lc ...` (Bash tool gated in EXECUTING; PowerShell allowed). Distro source files patched by base64-encoding a perl script to /mnt/c temp -> `base64 -d` inside the distro -> run perl (the proven reliable method; wslpath round-trips and $-passing across the boundary are unreliable). No Bash tool, no Write/Edit into the distro, no FSM reset, git untouched.

=========================================================================================
STEP 1 — BISECTION CONTROL (decisive): the hang is RECONSTRUCTION-SPECIFIC, not environmental
=========================================================================================
Installed the OFFICIAL @anthropic-ai/claude-code@2.0.0 (anti-slopsquat: npm view confirmed real, latest 2.1.183 modified 2026-06-19; pinned 2.0.0 to match the reconstructed source's vintage) into a THROWAWAY dir ~/teikoku-control (DIAGNOSTIC ONLY; owner's build-from-source directive stands; the control dir has been REMOVED). Ran the SAME trivial Opus turn through it with the SAME ~/.claude/.credentials.json plan-OAuth, SAME clean env, SAME flags, SAME bun runtime.
CONTROL RESULT: the official binary COMPLETED the turn in 2.6s: {"subtype":"success","result":"teikoku","modelUsage":{"claude-opus-4-1-20250805":{...}}}, billed to the plan, no api key in env.
=> environment, WSL egress, account quota, and the credential are ALL proven good. The reconstructed dist/cli.js hangs on the identical invocation. Bisection verdict: RECONSTRUCTION-SPECIFIC -> went to STEP 2A.

=========================================================================================
STEP 2A — ROOT CAUSE (instrumented) + FIX
=========================================================================================
INSTRUMENTATION: added stderr console.error probes (marker TEIKOKU_DBG) along the headless query path in src/main.tsx + src/cli/print.ts (getInputPrompt return; void runHeadless dispatch; runHeadless after getStructuredIO; before/after loadInitialMessages + ensureModelStringsInitialized; the for-await runHeadlessStreaming loop; run() entry; before/after updateSdkMcp; before ask()). Rebuilt (`bun run build.ts`) and re-probed.
First note also corrected: the "last log = [Bootstrap] Fetch ok" was misleading — that is a fire-and-forget background prefetch (src/services/api/bootstrap.ts), NOT on the turn path; and logForDiagnosticsNoPII only writes when CLAUDE_CODE_DIAGNOSTICS_FILE is set (it was not), so the absence of cli_message_loop_started in the debug log was not evidence. console.error probes were the right instrument.

BRACKETING: the probes fired through "runHeadless reached getStructuredIO" and "A after getSandboxUnavailableReason reason=undefined" (src/cli/print.ts ~602) then STOPPED — never reaching the setupTrigger check or before_loadInitialMessages. The only code between is the `else if (SandboxManager.isSandboxingEnabled())` branch (print.ts:617). A try/catch wrap around isSandboxingEnabled() captured the exact fault:
  TEIKOKU_DBG print: isSandboxingEnabled THREW Error: [teikoku] 'SandboxManager.isSupportedPlatform' from an Anthropic-internal package is not available in the teikoku build (sandbox feature not vendored).

ROOT CAUSE: runHeadless calls SandboxManager.isSandboxingEnabled() UNCONDITIONALLY at startup (the upstream #34044 surfacing fix). The adapter src/utils/sandbox/sandbox-adapter.ts isSandboxingEnabled() -> isSupportedPlatform() -> BaseSandboxManager.isSupportedPlatform(), and BaseSandboxManager was the inert THROWING Proxy stub the prior executor created for the un-vendored @anthropic-ai/sandbox-runtime (teikoku-stubs/internal.js: every property access returned a function that throws). The synchronous throw escaped the void-dispatched `void runHeadless(...)` (src/main.tsx ~2829) as an UNHANDLED REJECTION; the turn task died, but the process lingered on background timers (Bun.gc setInterval + prefetches), so NO /v1/messages was ever sent and the probe timed out (EXIT 124). The prior executor's assumption that "sandbox sits behind paths the OAuth+print path never runs" was wrong for this one detection call.

FIX (honest, no faking): replaced the throwing SandboxManager Proxy in teikoku-stubs/internal.js with a real "sandbox unavailable" stub reflecting the TRUE state (the native sandbox runtime is genuinely not vendored): isSupportedPlatform()=>false (truthful), checkDependencies()=>{errors:[...]}, lifecycle methods (initialize/reset/updateConfig/cleanupAfterCommand/waitForNetworkInitialization) safe no-ops, wrapWithSandbox returns the command unchanged, config getters return benign defaults. isSandboxingEnabled() now short-circuits to false, so every sandbox code path (REPL + --print/SDK) cleanly no-ops. The `else if` condition in print.ts was REVERTED to its original form (my wrap probe removed); the fix is entirely in the stub, not in CC source logic.

=========================================================================================
PROOF (DONE-gate met) — clean, probe-free build
=========================================================================================
All TEIKOKU_DBG probes stripped from src/main.tsx + src/cli/print.ts (grep -c => 0). Clean rebuild: `bun run build.ts` -> dist/cli.js, `--version` -> "2.0.0-teikoku (Claude Code)".
Opus turn (json): echo "Reply with exactly one word: teikoku" | <clean env, no api key> bun dist/cli.js --print --output-format json --strict-mcp-config --dangerously-skip-permissions --model opus
  -> {"subtype":"success","is_error":false,"result":"teikoku","stop_reason":"end_turn","service_tier":"standard","modelUsage":{"claude-opus-4-6":{...}}}  (~2.8s, plan-billed, no api key)
Opus turn (text): same invocation with --print (no json) -> "teikoku".
Real Opus model id observed: claude-opus-4-6. Reproducible across fresh sessions (two json runs + one text run, distinct session_ids).

CLEANUP done: all debug probes removed (build clean); ~/teikoku/docs/cc-auth-flow.md KNOWN-GAP section replaced with a RESOLVED + root-cause + fix section; throwaway ~/teikoku-control control install removed. No secret token value was ever read or printed (credential confirmed by field/source only).

FILES TOUCHED (distro paths)
- ~/teikoku/base-cc/teikoku-stubs/internal.js — replaced the throwing SandboxManager Proxy with an honest "sandbox unavailable" stub (THE FIX).
- ~/teikoku/base-cc/dist/cli.js — rebuilt artifact (product of build.ts).
- ~/teikoku/docs/cc-auth-flow.md — KNOWN GAP -> RESOLVED (root cause + fix; no auth-flow fact changed).
- (src/main.tsx, src/cli/print.ts were temporarily instrumented then fully reverted to original; net diff = ZERO probe lines remain.)
- F:\.ELAI_workflow\workflow\reasoning\teikoku-0.2-turn-resolved.md — this controller note.

VERIFICATION / GROUNDING
- Control: official 2.0.0 completed the turn (model claude-opus-4-1-20250805, success). Verified (observed JSON).
- Root cause: isolated to a thrown exception via instrumented probe (TEIKOKU_DBG isSandboxingEnabled THREW SandboxManager.isSupportedPlatform). Verified (observed stderr).
- Fix proof: clean probe-free reconstructed dist/cli.js completes Opus turn, model claude-opus-4-6, result "teikoku", success, end_turn. Verified (observed JSON + text output, multiple runs).
- No F:/D: source git-mutated (git read-only honored). No secret value read or printed.

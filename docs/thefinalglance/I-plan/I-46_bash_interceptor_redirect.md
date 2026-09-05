# I-46: bash interceptor + command fixup — redirect bare grep/cat/find to the proper tools instead of just validating

- Track 20 | lever CHEAPER + CANT-WRECK | grounding STRONG (seam) / MEDIUM (gajae semantics, SL-10) |
  nature EXTENSION | effort S-M | WAVE 2

- Premise: STEAL-LIST B1 ("top tool steal"). ELAI's bash_validation.rs VALIDATES commands (allow/deny);
  gajae's interceptor additionally REWRITES the call: a bare `grep -r foo src/` becomes the registered
  search tool (cheaper, journaled, permission-scoped), a known-broken invocation gets a deterministic
  fixup, and an allowed-prefix table keeps the rewrite surface bounded. Validation rejects; interception
  recovers — fewer dead tool calls, fewer raw-output context floods, every redirect journaled.

## Steal/build
- Steal the three-part shape from gajae (SL-10 binding read: read bash-interceptor.ts +
  bash-command-fixup.ts + bash-allowed-prefixes.ts before EXECUTING):
  (a) INTERCEPT: classify the bash command; if a registered ELAI tool covers it (grep/find/cat/head/tail
  -> search/read tools), REFUSE-WITH-REDIRECT carrying the exact substitute tool call;
  (b) FIXUP: a small deterministic rewrite table for known-broken shapes (wrong flag order, missing
  quoting) — applied only when the rewrite is semantics-preserving and the table entry is frozen;
  (c) ALLOWED-PREFIXES: the bounded list of prefixes that pass through untouched.
- EXCLUDED: no LLM-driven rewriting (deterministic table only); no auto-EXECUTION of the substitute (the
  agent re-issues the redirected call — keeps the supervisor/audit trail honest); no expansion of what is
  ALLOWED (interception narrows, never widens, the existing validation).

## Existing ground truth
- crates/tools/src/bash_validation.rs SHIPPED (SL-2); the redirect absence is STEAL-LIST-asserted —
  executor re-greps the file first (W-7).
- E-30 argv allowlist shipped (per-call shape); I-35 (PLANNED) adds sequence; I-46 adds SUBSTITUTION — a
  fourth, disjoint dimension. CLAUDE.md harness analog: the harness itself nags bare grep/cat — ELAI's
  runtime gets the mechanical version.

## REVISED scope (W-10 tiered)
- SHIP NOW: interceptor + fixup table + prefixes, wired into the existing bash tool validation path;
  redirect coverage for the read/search family.
- DEFER: redirects into recipe runners (waits on I-53); per-project custom prefix config.

## Integration
- crates/tools/src/bash_validation.rs (the seam — extend, do not fork) + the bash tool dispatch site;
  the fixup/prefix tables ship as R8-frozen repo artifacts (a run must not edit its own rewrite rules).
  frontend_pair: redirect rows visible in the existing tool-call drawer (no new component).

## WIRING_CONTRACT
- produces_symbols: tools::bash_interceptor (classify/redirect/fixup) + frozen fixup + prefix tables +
  BashRedirected SSB rows.
- consumes_symbols: tools::bash_validation (the existing verdict path — interceptor runs BEFORE it),
  the tool registry (substitute targets), E-30 argv allowlist (unchanged, downstream).
- reachable_from: the bash tool execution path in crates/tools (PRODUCTION dispatch — every agent bash
  call passes it). Test-file callers do not satisfy (R7).
- stub_ban: NO LLM call inside the interceptor; NO silent auto-execution of substitutes; NO bypass flag
  that skips interception (R3); NO regex-only "looks like grep" matching without argv parsing; NO
  .unwrap() on prod paths.
- observable_effect:
  trigger: "a live agent session issues `grep -rn pattern crates/` as a bash call"
  expect_ssb: "BashRedirected(Redirected) naming the substitute tool call; a fixup fixture lands
  BashRedirected(FixedUp); a non-allowlisted novel prefix lands BashRedirected(PrefixBlocked)"
  expect_artifact: "the bash command did NOT execute (no process spawn in the journal window); the
  NEGATIVE arm — an allowed-prefix command — executes untouched with zero redirect rows. A redirect row
  with a spawned grep process in the same window is the broken arm and FAILS (R2)."

## Grounding
- STEAL-LIST B1 (Tier 1); SL-2, SL-10. gajae files: packages/coding-agent/src/tools/bash-interceptor.ts,
  bash-command-fixup.ts, bash-allowed-prefixes.ts.
- CLAUDE.md token-discipline (the redirect exists to stop raw-output context floods at the runtime layer).

## COMPOSES-WITH
- tools bash_validation (extends), E-30 argv (downstream, unchanged), I-35 sequence enforcement (disjoint
  dimension), I-53 recipe runners (future redirect targets), H-4 tool search (discovery of substitutes).
  Restates none. SSB: +BashRedirected (Kind: Redirected/FixedUp/PrefixBlocked). baseline+1 from LIVE
  221/532 (SL-1); re-read at edit (W-9).

End of I-46.

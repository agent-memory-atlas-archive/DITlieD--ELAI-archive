Liberty token + context lifecycle plan

target repo: F:\.ELAI_workflow\.liberty\  (Liberty, its OWN local git repo; NOT .elai_cc)
author: main LLM (grounded from source reads 2026-06-16); implementer: one opus agent; verifier: main LLM off-agent after return.
do NOT push. no auto-commit. .liberty/.env holds keys and is gitignored - never stage it.

WHY / GROUNDED STARTING STATE (verified by reading the source, do not re-derive, but DO re-read each cited file before editing it)
- Per-message token capture already exists and is ACTUAL API-reported, after every acompletion:
    main loop: liberty/agent_loop.py:245-253 (response.usage -> prompt_tokens/completion_tokens -> overflow_cb.on_usage)
    subagent:  liberty/subagent/subagent_session.py:357-362 (accumulates into self.usage.input_tokens/output_tokens)
    callback:  liberty/context/context.py ContextOverflowCallback.on_usage (captures last_api_prompt_tokens), on_llm_start prints "~XK/YK".
  MISSING: no unified, queryable, per-message live token surface. `liberty quota` (cli.py:167) is KEY-level daily usage, not a per-message meter.
- Context window is resolved ONCE and pinned for the life of the loop:
    main:      agent_loop.py:146 self._resolved_model = resolve_model(model)  (model is a ROUTER GROUP name)
    callback:  context/context.py:252 ContextOverflowCallback.__init__ resolves _context_window once
    subagent:  subagent_session.py:279 builds its own ContextOverflowCallback once at construction
  The litellm Router swaps the actual deployment per call under that pinned window. Free-model windows ARE known and explicit
  in liberty/catalog.py (8_192 / 32_768 / 131_072 / 1_048_576), registered into resolve_model via _register_all_model_configs
  (catalog.py:331) WITHOUT a network call. Unknown/unregistered models fall to DEFAULT_CONTEXT_TOKENS=200_000 (context.py:67) = UNSAFE;
  dynamically-synced models default to 4_096 (catalog_sync.py:538) = safe.
- Reactive context-overflow recovery (catch -> force_compaction -> compact in place -> retry same step) ALREADY EXISTS in BOTH loops
  (agent_loop.py:229-240, subagent_session.py:347-355). This is "feature B"; it is DONE, only widen its trigger (see Deliverable 3).
- is_context_overflow_error EXCLUDES anything matching rate.?limit|tpm|tpd|rpm|rpd (context.py:212). So a TPM "single request too big"
  rejection is routed to the self-heal WAIT path (self_heal.py), never to compaction, even though compaction is the only thing that fixes it.
- Orchestration self-heal parity is wired: orchestrator.py:414 passes router=router into GeneralSubagentSession (WG-2 closed).
  Subagents already get their own overflow_cb + reactive/proactive compaction. Remaining holes: pinned-window bug (above) and no unified token total.

SCOPE (do ONLY this; R-3 scope contract). Every deliverable below must apply UNIFORMLY to the main loop AND the subagent loop.
Files in play: liberty/context/context.py, liberty/context/compaction.py (read-only ref), liberty/agent_loop.py,
liberty/subagent/subagent_session.py, liberty/self_heal.py, liberty/cli.py, liberty/catalog.py, liberty/orchestrator.py,
a NEW liberty/usage_ledger.py, and tests/. Do NOT touch the codex/bridge lanes, router internals, or unrelated modules.

DELIVERABLE 1 - UNIFIED REAL-TIME TOKEN LEDGER + `liberty usage` (ELAI J-28/J-29 design, built fresh)
  1a. NEW liberty/usage_ledger.py: a process-wide UsageLedger (one per run, threaded in, NOT a global singleton unless a singleton is
      the only way to aggregate subagents - prefer an explicit object passed down). Public API (keep it small, <3 verbs):
        record(*, source: str, model: str, prompt_tokens: int, completion_tokens: int, tagged: Literal["actual","estimate"]) -> None
        snapshot() -> a frozen dataclass: per-source totals + grand total {prompt, completion, total, last_message_*}.
      source = "main" for the main loop, "sub:<run_id>" for each subagent. Must be asyncio-safe (single-process; a plain object is fine,
      no lock needed if all calls are on the event loop thread - confirm and document that assumption).
  1b. WIRE record() at the two existing capture sites, AFTER each acompletion:
        agent_loop.py ~245-253: when response.usage present -> tagged="actual"; when absent -> estimate via estimate_messages_tokens
          on the sent messages + extracted output -> tagged="estimate". Always record exactly once per message.
        subagent_session.py ~357-362: same, source="sub:<run_id>".
      The estimate fallback is REQUIRED (free providers omit usage; Pollinations especially). Tag it so the surface shows actual vs est.
  1c. SURFACE: after each message, emit ONE concise live line via the existing print path (or logging) showing this-message + session-total
      tokens and the actual|est tag. Keep it one line; do not spam. The orchestrator must thread ONE ledger into all subagents so the
      grand total is real-time across main + every subagent (orchestrator.py passes it alongside router= at the GeneralSubagentSession call).
  1d. CLI: add `liberty usage` (cli.py: cmd_usage + add_parser + dispatch dict) that prints the snapshot for the current/most-recent run.
      Decide and document the source of truth for a finished run (read back from the per-session transcript usage entries that already exist
      via session_mgr.append_message(..., usage=...), OR persist a small usage.json next to the session). Prefer reading the transcript that
      already records usage; only add a file if the transcript cannot answer it. Do NOT duplicate the key-level `liberty quota`.

DELIVERABLE 2 - FEATURE A: PER-CALL MODEL-AWARE WINDOW (proactive, fallback-chain minimum)
  2a. Stop pinning the window for the whole run. Before each acompletion, the effective context_window used for needs_compaction must be the
      MINIMUM context_window across the deployments the router could serve THIS call from (the group + its fallback chain). Source the numbers
      from the catalog (catalog.py model windows / models_by_group), NOT litellm.get_model_info (already avoided). Provide a helper, e.g.
      liberty/catalog.py: min_context_window_for_group(group: str) -> int  (min over the group's models; if a model is unregistered, treat as
      the safe small default, NOT 200_000). Document the chosen rule.
  2b. Apply it at BOTH ContextOverflowCallback sites: let on_llm_start (or a small setter called before it) use the current call's group-min
      window instead of the constructor-pinned one. Constructor still takes an initial window; the per-call value overrides it when a router
      is wired. When NO router (bare model string), behaviour is unchanged (single concrete model = its own window).
  2c. HARDENING: give the overflow/Pollinations tier (providers_extra.py model) an explicit conservative context_window in the catalog so it
      no longer resolves to the unsafe 200_000 default. Pick a documented conservative number from the provider; if genuinely unknown, use the
      same small safe default the sync path uses (4_096-class), and write down why.

DELIVERABLE 3 - "LITERAL API LIMIT" HANDLING (the TPM / request-too-big gap + widen detection)
  3a. Distinguish, on an API error, three cases BEFORE deciding wait-vs-compact:
        (i)  context-window overflow  -> compact + retry  (already works; keep)
        (ii) rate limit by FREQUENCY (RPM/RPD) -> self-heal wait/swap (already works; keep)
        (iii) rate limit by SIZE (TPM / tokens-per-minute / "request too large for tokens-per-min") -> try ONE compaction THEN retry; only if
              still rejected, fall through to the self-heal wait/swap path.
      Implement (iii) as a narrow, well-tested classifier (a new helper alongside is_context_overflow_error, e.g. is_token_rate_or_size_error)
      - do NOT broaden is_context_overflow_error itself (its rate-limit exclusion is load-bearing for case ii). Bound the extra compaction by
      the same max_compactions budget so it cannot loop.
  3b. Widen the finite overflow-pattern list (context.py:200-210) with the common free-provider phrasings you can verify (groq/mistral/gemini/
      openrouter 400/413 "request too large", "reduce the length", "max tokens", etc). Verify each phrasing against the provider's real error
      text where possible; if you cannot verify a phrasing, do NOT add it on a guess (anti-slop). Note which were verified vs omitted.

DELIVERABLE 4 - ORCHESTRATION PARITY CONFIRMATION
  Confirm (and fix if needed) that Deliverables 1-3 apply to subagents identically: the ledger is threaded into every GeneralSubagentSession,
  Feature A's per-call window applies to the subagent overflow_cb, and the TPM/compaction handling lives in the subagent loop too. Self-heal
  router parity is already wired (orchestrator.py:414) - just verify it, do not rebuild it.

NON-GOALS (do NOT do): no streaming rewrite; no new dependencies (anti-slop: if you think you need one, STOP + handoff); no router-internals
changes; no touching codex/bridge lanes; no commit/push; no edits to tests that judge your own output beyond ADDING new tests.

WIRING / DONE-GATE (every new public fn must have a real production caller in the SAME change; wiring-enforcement R6/R7)
- record() called from both capture sites; snapshot() consumed by the live line + `liberty usage`; min_context_window_for_group consumed by
  both overflow callbacks; is_token_rate_or_size_error consumed by both loops' except handlers. No new pub fn left dark (no WIRE-DARK unless a
  consumer genuinely lands in a named later step - then mark it `# WIRE-DARK[liberty]:` + a WIRE-DARK.md row).

TESTS (fake backend only; no live VM/provider/network - mirror the existing tests/ style)
- ledger: actual-path records actual; usage-absent path records estimate tagged "estimate"; subagent records aggregate into the grand total.
- feature A: a run whose group-min window is small triggers needs_compaction earlier than the constructor window would; bare-model path unchanged.
- deliverable 3: a TPM/size error string -> classified as (iii) -> one compaction attempted before any wait; an RPM/RPD string -> still (ii),
  NOT compacted; a context-overflow string -> still (i). Widened patterns: each added phrasing detected.
- parity: subagent loop exercises the same three behaviours.

SELF-VERIFY BEFORE RETURNING DONE (R-5; the auditor/main-LLM re-runs these off-agent, your self-run is necessary-not-sufficient)
  from F:\.ELAI_workflow :
    python .claude/tools/python_wire_check.py --files <comma-list of every .py you changed>   (0 dark, or registered WIRE-DARK)
    python .claude/tools/maintainability_check.py --files <comma-list> --json                  (0 BLOCK)
  from F:\.ELAI_workflow\.liberty with the venv python (.venv\Scripts\python.exe):
    -m pytest tests/ -q     (report pass/skip/fail counts; the pre-existing test_quota.py::test_daily_cap_exhaustion failure is known-unrelated)
  Return STATUS + FILES TOUCHED (one line each) + VERIFIED (exact commands + counts) + NOTES. Anything you did not run = UNVERIFIED, never implied done.

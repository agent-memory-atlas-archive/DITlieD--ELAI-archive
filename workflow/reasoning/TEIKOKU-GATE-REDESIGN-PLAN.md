TEIKOKU GATE REDESIGN — rules as gates, CLAUDE.md as carrots

status: plan draft, 2026-06-23, authored from the two harness indexes (HARNESS-INDEX-ELAI.md + HARNESS-INDEX-LITEGAME.md) + the 3 TEIKOKU files (master plan + the two handoffs) + live distro greps.
grounding: every claim here is from a read file or a live wsl grep, cited inline. No training-memory claims about base-cc hook behavior.
goal (owner's words): make hard rules a deterministic GATE (a stick the model cannot ignore), stop carrying enforcement as .md prose in every session, leave CLAUDE.md as behavioral CARROTS, push reference depth into teikoku's own RAG. Specific new sticks asked for: cant comment on code, cant commit without owner approval, and a "read = read the WHOLE file" hook.

================================================================
0. THE ONE-PARAGRAPH THESIS
================================================================
A rule is text in the context window; the model can override it (training priors, token-prediction). A hook returns a block decision and the tool call does not run, full stop. So every HARD rule that is mechanically checkable should be a hook (removed from prose AND enforced), every JUDGMENT rule stays a terse carrot in CLAUDE.md (cannot be enforced, must be in context), and every REFERENCE doc moves to RAG (retrieved on trigger, zero always-on cost). Three buckets, one routing rule. liteGAME already proved this exact architecture; teikoku has the substrate but has NOT wired it. This plan ports liteGAME's gate spine into teikoku, re-implemented for WSL/base-cc, and adds the three new sticks.

================================================================
0b. LOCKED DECISIONS (grill, 2026-06-23) — authoritative; the body below is detail
================================================================
GLOSSARY (pinned): "gate" = a base-cc PreToolUse/PostToolUse hook in ~/teikoku/.claude/settings.json (NOT the FSM PreToolUse gate, NOT the finance deflation/leakage gates). "runtime" = base-cc sessions INSIDE the teikoku distro (the interactive SIGNAL chat main model + every spawned worker). "reflect/roadblock" = the every-action visible-text reasoning gate.

D1  SCOPE = teikoku runtime ONLY (distro chat + workers). settings.json -> ~/teikoku/.claude/. The F: build-context (ELAI sessions building teikoku) keeps its own existing hooks, untouched, and is the OUT-OF-BAND repair path.
D2  FAIL-MODE = ALL gates fail-CLOSED. A hook error denies the action. If a gate bricks the distro, recover from the ungated F: VS Code session via wsl. (The per-row "fail-open" labels in §4/§8 are SUPERSEDED — read them all as fail-CLOSED.)
D3  REFLECT GATE = every-action roadblock; VISIBLE TEXT only (a hidden thinking block does NOT satisfy it); substance bar = assistant text since the last tool result >= ~120 chars AND references the concrete target (file/symbol/command/last-result). filler that names nothing fails.
D4  COMMIT GATE = deny git commit/push unless a one-shot DASHBOARD button sentinel is set (model cannot self-approve); also rejects AI-attribution trailers + non-maintainer message prefixes.
D5  COMMENT GATE = BLOCK provenance/AI-tell comments on added code lines; WARN on net-new comment density.
D6  READ-EVERYTHING = on "read/look at/go through/check X", read the WHOLE file in chunks (verify line count + byte size first); warn+confirm ONLY above ~15k lines / ~2MB.
D7  GRC = DROP the mcp_think mandate AND liteGAME's evidence-gate entirely; the reflect gate replaces both. mcp_think survives only as an optional fork-ledger tool (no mandate, no blockquote).
D8  PROMPT-TRANSFORM = conditional prompt-engineer WHEN-clause (HEAVY/fuzzy only), not mandatory-always.
D9  PYTHON_WIRE_CHECK = fix the .memory-venv RecursionError now + R8 re-lock (folded into the freeze ceremony).
D10 UNITY PACK = port the FULL liteGAME unity set (~48 GAME-tagged files) into the "unity game" toggle now; dedup vs teikoku's existing AOR unity gates (toban001/mono_wire/unity_symbol).
D11 FREEZE = new hook scripts + settings.json go into gate-manifest.txt, hash-locked (R8); a run cannot weaken its own gates; legit changes need the re-lock ceremony.
D12 ROUTING = HOOKABLE -> python hook (deleted from prose); JUDGMENT -> terse CLAUDE.md carrot; REFERENCE -> teikoku memory_semantic RAG (retrieve on trigger).

OPEN-FOR-GROUNDING (the G-1 probe — MUST pass before ANY gate is built):
  (1) does base-cc fire PreToolUse hooks on a `--print` turn? (kill-question for the whole plan)
  (2) does the PreToolUse payload expose transcript_path? (the reflect gate is dead without it; if absent it degrades to a counter-based roadblock, blind to whether the text is real reasoning)

================================================================
1. WHAT TEIKOKU ALREADY HAS (do NOT rebuild — N-02 improve+wrap)
================================================================
Verified in the distro + handoffs:
- Self-hosted RAG: conductor/memory_store.py (grep floor) + memory_semantic.py (sqlite-vec vec0 + nomic-embed-text-v1.5, CPU/offline, separate memory_vec.db) + handoff_indexer.py (auto-indexes handoffs). Explicitly "grounded against the Windows local-rag concept, NOT a port." => the reference-doc store already exists; reuse it, do NOT touch the Windows local-rag server.
- Frozen-gate machinery: .claude/tools/gate_freeze_check.py + .claude/gate-manifest.txt + .claude/gate-manifest.lock (8 files frozen by sha3_256; R8 — a run cannot edit its own gate). Proven in the AOR audit verdict.
- Deterministic checkers (off-agent, auditor-invoked): maintainability_check.py, production_caller_check.py, python_wire_check.py (BROKEN — RecursionError on .memory-venv symlink loop, frozen so needs owner re-lock to fix), toban001_check.py, mono_wire_census.py, unity_symbol_census.py.
- A PostToolUse hook ALREADY WRITTEN (AOR WAVE-0): tools/aor/change_recorder.py -> FILE_CHANGED rows in the P2 event-log; changed_files() is the deterministic FILES-TOUCHED source.
- AOR "Session Orientation Brief": tools/aor/teikoku_orient.py fuses git + grep-map + event-log + gate-verdict into a non-drift brief (a context-saving mechanism in itself).
- A 43-line CLAUDE.md already (mostly carrots + pointers). So "CLAUDE.md is already carrots" is ~80% true for teikoku; the bloat lever is elsewhere (see §10).
- session_toggles.unity_preset + active_packs_for_session + context_catalog.unity_mcp_subset + the NewSessionMenu "unity game" toggle (AOR UNITY wave) — the pack-activation seam already exists.
- conditional-routing.md (lazy-load dispatch table) on both source harnesses — rulebook is already trigger-loaded, not always-on.

================================================================
2. WHAT TEIKOKU LACKS (the actual gap)
================================================================
- NO wired settings.json hooks in the distro. `ls ~/teikoku/.claude/settings*.json` = empty (verified). base-cc FULLY supports PreToolUse(can block)/PostToolUse/PostToolUseFailure/PermissionRequest (base-cc/src/skills/bundled/updateConfig.ts:140-142). change_recorder.py exists as a tool but is NOT registered as a live hook. There is NO PreToolUse BLOCKING hook at all in teikoku today. Every "hard rule" is hopeful prose — and the handoffs document the cost (workers reset the FSM repeatedly, one git reset --hard x8, one wrote to the controller's handoff out of scope, one targeted the wrong repo).
- The F: ELAI global rag hooks (pre_tool_check.py / post_tool_check.py / stop_check.py / record_evidence.py / pre_compact_check.py at C:\Users\user\.claude\rag\hooks) are PYTHON and are the right pattern, BUT they are tuned for the F: ELAI FSM + Windows paths and "MISGATE this WSL-distro build" (teikoku-build.md FSM/GATE GOTCHA). They are a design donor, not a drop-in.
- liteGAME's gate spine is the gold standard (10 project hooks + 5 global) BUT the project gates are .ps1 (unity-verify-gate.ps1, bash-dialect-gate.ps1, pre-checkin-gate.ps1, session-*.ps1). teikoku/base-cc runs in WSL (Linux) — PowerShell is not there. The LOGIC ports; the .ps1 shell does NOT. Every adopted gate must be re-implemented in python (or bash).

Net: teikoku has the RAG, the frozen-checker discipline, and the event-log; it is missing the live hook layer entirely, and neither donor's hooks run as-is in WSL.

================================================================
3. THE THREE-TIER ROUTING RULE (applied to the merged rule set)
================================================================
For every rule in the merged ELAI+liteGAME set, route by the index CLASS:
  HOOKABLE  -> a base-cc hook in ~/teikoku/.claude/settings.json (python). Prose DELETED from the worker system prompt; the gate IS the rule.
  JUDGMENT  -> stays in CLAUDE.md as ONE terse carrot line. Cannot be enforced, must be in context.
  REFERENCE -> indexed into teikoku RAG (memory_semantic), retrieved on trigger via the conditional-routing map. Zero always-on cost.

Index totals to route: ELAI 71 HOOKABLE / 18 JUDGMENT / 89 REFERENCE; liteGAME 40 HOOKABLE / 12 JUDGMENT / 65 REFERENCE. After dedup (the two sets are ~70% the same rule under different names — see both indexes' DUPLICATES sections) the merged unique set is roughly: ~30 hookable rule-families, ~12 carrots, ~60 reference docs.

================================================================
4. THE HOOK SET TO WIRE (~/teikoku/.claude/settings.json, python, WSL)
================================================================
All hooks are python (NOT .ps1), live in ~/teikoku/.claude/hooks/, read base-cc's JSON hook stdin, exit 0=allow / nonzero+deny-decision=block. NOTE (D2): ALL gates are fail-CLOSED — the per-row "fail-mode" column below is SUPERSEDED; read every "fail-open" as "fail-CLOSED". "from" = donor logic.

PreToolUse (the sticks — can BLOCK):
  matcher        | hook                      | check                                                                                 | from / new | fail-mode
  Bash           | commit_gate.py            | deny `git commit`/`git push` unless an owner-approval sentinel set this session        | NEW (your ask) | fail-CLOSED
  Bash           | bash_destructive_gate.py  | deny git reset --hard/checkout-paths/clean/stash-drop/rebase/push --force for workers  | liteGAME pre_tool_check destructive denylist + agent-dispatch-preamble R-1/R-2 | fail-CLOSED
  Bash           | no_cargo_test_gate.py     | deny `cargo test`/full-suite by a worker (auditor-only); allow cargo check             | ELAI CLAUDE.md "main never runs tests" | fail-CLOSED
  Bash           | bun_not_node_gate.py      | deny new `node`/`npm`/`npx` (suggest bun/bunx)                                         | bun-not-node.md (both) | fail-open
  Edit|Write     | comment_gate.py           | on .rs/.cs/.py/.ts diff, deny added 3rd-person/provenance comments (// per, // added by, agent/workflow refs); WARN on net-new comment density | NEW (your ask) + liteGAME code-comments-maintainer-voice + pre-checkin-gate | fail-open (WARN-heavy)
  Edit|Write     | scope_write_gate.py       | deny writes outside the session's assigned project worktree / TaskSpec write-set; deny any /mnt/f|F:\ path | liteGAME project-boundary gate + teikoku CLAUDE.md project-boundary | fail-CLOSED
  Edit|Write     | frozen_gate_guard.py      | deny edits to any file in gate-manifest.txt unless an out-of-band re-lock ceremony     | wiring-enforcement R8 (both) | fail-CLOSED
  Edit|Write     | zero_lazy_gate.py         | deny added TODO/pass/placeholder/except:pass/.unwrap() outside #[cfg(test)] unless a registered stub marker + stubs.md row | token-discipline zero-lazy + stub-tracking (both) | fail-open
  Read           | read_full_gate.py         | when the "full-read" flag is set (UserPromptSubmit), deny a Read with a truncating limit; force full/chunked read | NEW (your ask) | fail-open
  Read|Grep|Edit|Write|Bash | reflect_gate.py       | ROADBLOCK: deny a tool call if the transcript shows NO reasoning since the last tool result; forces 1-3 sentences of reasoning between actions | NEW (owner-designed, REPLACES mcp_think + evidence-gate — see §8) | fail-open

PostToolUse (record / lint — cannot block, can warn + feed the ledger):
  Edit|Write     | change_recorder.py        | FILE_CHANGED rows -> event-log (ALREADY EXISTS, just register it)                      | AOR | n/a
  Edit|Write     | syntax_lint.py            | ruff/py_compile + svelte-check-lite + the zero-lazy re-scan                            | ELAI post_tool_check | n/a

UserPromptSubmit:
  *              | full_read_flag.py         | if the user message contains read/look at/go through/check X -> set the per-turn full-read flag consumed by read_full_gate | NEW (your ask) | n/a

Stop / PreCompact:
  Stop           | stop_gate.py              | block mid-workflow Stop in a non-terminal FSM state; enforce post-write self-verify    | ELAI stop_check + liteGAME session-gate | fail-open
  PreCompact     | precompact_handoff.py     | autosave a SESSION_HANDOFF snapshot before truncation                                  | ELAI pre_compact_check | n/a

================================================================
5. THE THREE NEW STICKS (your explicit asks) — detailed
================================================================
5a. CANT COMMIT WITHOUT ME (commit_gate.py, PreToolUse:Bash, fail-CLOSED)
  - Match `git commit` or `git push` in the command. Deny by default with an actionable message: "commit blocked — owner approval required. Set the approval sentinel (or press <X> in the dashboard) to authorize this one commit."
  - Approval mechanism (D4, LOCKED): a one-shot sentinel file the SIGNAL DASHBOARD writes when you click an "approve commit" control — out-of-band of the model, so a worker CANNOT produce it to self-approve. One-shot (consumed on the commit) so it can't leak to the next. (Typed-phrase approval was rejected: the phrase passes through the model, which could emit it to self-approve.) This realizes the no-auto-commit memory + commit-cadence "one changeset per task" as a HARD gate instead of a hope.
  - Bonus (free, since we're gating the message anyway): reject a commit message carrying a Co-Authored-By / AI-attribution trailer (CLAUDE.md hard rule) and a non-maintainer-voice prefix (feat:/chore:/[Phase N]) — message-style lint.

5b. CANT COMMENT ON CODE (comment_gate.py, PreToolUse:Edit|Write)
  - Honest scope: a blanket "no comments" hard-block is noisy (legitimate comments exist). Recommend two tiers: BLOCK the provenance/AI-tell class (// per X, // added by, "as requested", agent/workflow/ticket refs — liteGAME's code-comments-in-maintainer-voice rule already defines this list) and WARN on net-new comment density above a threshold. If you truly want zero new comments, flip the WARN to a BLOCK behind a per-session toggle.
  - Diff-grounded: only inspect ADDED lines in the Edit/Write payload, language-scoped to .rs/.cs/.py/.ts/.svelte.

5c. READ = READ THE WHOLE FILE (full_read_flag.py + read_full_gate.py)
  - Two pieces, because a hook fires on tool calls, not on your message:
    (1) UserPromptSubmit hook detects read/look at/go through/check + a path/filename in your message and sets a per-turn "full-read" flag (a session state file).
    (2) PreToolUse:Read hook, while the flag is set, DENIES a Read that passes a truncating `limit`/`offset` (the lazy partial read) and instructs: "full-read mode — read the entire file; verify line count + byte size first, then read in <=25k-token chunks until EOF." It cannot FORCE N sequential reads (no hook can), but it refuses the lazy partial, which produces the same outcome.
  - This is exactly the friction i hit by hand on the 996-line teikoku-build.md this session; the gate would have made it automatic.
  - CEILING (D6, LOCKED): full read for normal files, no ceiling; ONLY above ~15k lines / ~2MB does the gate pause once — "this file is N lines / M MB, ~T tokens to read fully, confirm?" — to stop a catastrophic context blow on a giant log. Not a prefix-and-stop; a one-time confirm, then the whole file.

================================================================
6. WHAT CLAUDE.md BECOMES (carrots only, ~12 lines added to the existing 43)
================================================================
The JUDGMENT rules that CANNOT be a hook and CANNOT be reliably retrieved — they must sit in context as terse one-liners:
  grounded-by-default; anti-sycophancy (no opening affirmation, no auto-agree); two-approaches-before-recommending; abstention-over-guessing; completeness (a correct answer missing caveats is wrong); confidence-calibration (Verified/High/Uncertain); ask-don't-assume; main-LLM-is-orchestrator-not-implementer; don't-fold-under-pushback-if-correct; search-before-speaking. 
Everything else that is currently prose in the source CLAUDE.md files becomes either a hook (§4) or a RAG doc (§7). teikoku's CLAUDE.md stays ~one screen.

================================================================
7. THE RAG REFERENCE LAYER (reuse teikoku memory_semantic)
================================================================
- Index every REFERENCE doc (rulebook depth, skill how-tos, agent definitions, tool catalogs, the Project-PATHs / sidecar / SCHEMA_VERSION / posture pointer blocks) into teikoku's existing memory_semantic store, tagged by trigger.
- conditional-routing.md is no longer a "read this .md file" table — it becomes the RETRIEVAL MAP: trigger -> RAG query. When a trigger fires, the runtime pulls the doc from RAG instead of loading a file into the system prompt. The AOR orient brief already does fuse-on-demand context; this extends it to the rule corpus.
- This is the bucket that actually removes the .md from always-on context: the docs still EXIST (so they can be indexed and retrieved), but they are never injected unless their trigger fires.

================================================================
8. THE REFLECTION GATE (NEW — owner-designed: a roadblock between actions)
================================================================
Owner verdict (2026-06-23): mcp_think is NOT working as intended and not good enough; liteGAME's "deny an edit with no fresh Read" evidence-gate is redundant. DROP BOTH. The real problem the owner observes: the LLM fires 10 reads, or 10 actions, in a row with NO thinking in between. The fix is a ROADBLOCK between actions that forces VISIBLE reasoning.

reflect_gate.py  (PreToolUse, matcher = Read|Grep|Glob|Edit|Write|Bash|MCP-action tools — every tool, D3). fail-CLOSED (D2).
SUBSTANCE BAR (D3, LOCKED): the visible assistant text since the last tool result must be >= ~120 chars AND reference the concrete target (the file/symbol/command being acted on, or the last result). Filler that names nothing fails. This is a FORM check, not a quality check (quality is unhookable).

MECHANISM (deterministic, transcript-grounded, un-fakeable):
- The hook reads the TAIL of the live session transcript (base-cc writes ~/.claude/projects/<slug>/<uuid>.jsonl — confirmed in teikoku-native handoff; the PreToolUse payload must expose transcript_path — VERIFY in G-1).
- It asks ONE question: since the last tool RESULT, did the assistant emit a SUBSTANTIVE reasoning text block? (heuristic: an assistant text block >= MIN_CHARS, not pure boilerplate, ideally referencing the last result or the next target.) It reads ACTUAL emitted text, so the model cannot fabricate it and cannot skip it.
- If NO reasoning since the last result -> DENY this tool call ONCE with: "Reflection roadblock. Before {tool} on {target}, state in 1-3 sentences: (1) what the last result showed, (2) why THIS is the right next step, (3) what you expect it to return. Then re-issue." The denial forces a text turn; once the model reasons, the retry passes (the transcript now shows the reasoning).

TRIGGER POLICY (one knob, REFLECT_MODE — a session toggle):
- every-action (OWNER-CHOSEN DEFAULT, 2026-06-23): reasoning REQUIRED before EVERY tool call (read, grep, edit, write, bash — anything). Truest to "a roadblock between every tool use." Owner accepts the friction/latency: a reasoned text turn precedes each action. Reasoning window = "since the last tool RESULT or the turn-start user message" — so the model's opening response to the user covers its FIRST action; only CHAINING tool->tool with no intervening text is blocked. That is every-action done sanely, not an infinite first-call wedge.
- mutation-priority (the loosened option, for a trusted autonomous wave): reasoning before every MUTATION + before the (K+1)th read in a burst (K=3). Lower friction.
- per-session toggle, so interactive runs strict (every-action) and a long autonomous run can drop to mutation-priority.
REASONING FORM (OWNER-CHOSEN, 2026-06-23): VISIBLE TEXT only. A hidden extended-thinking block does NOT satisfy the gate — the reasoning must be a visible assistant text block in the transcript (model-agnostic for the free kimi/glm lanes that emit no thinking, and auditable by the owner).

LOOP SAFETY: never deny the same tool twice in a row (after a reasoned retry it always passes); per-session denial cap. fail-mode (D2): transcript GENUINELY shows no reasoning -> hard deny (the roadblock works); transcript UNREADABLE (a hook error) -> fail-CLOSED (deny) -> distro halts -> recover from the ungated F: session via wsl (owner-accepted edge case). No silent hole: a broken roadblock stops the distro rather than vanishing.

WHY IT BEATS BOTH PRIOR DESIGNS:
- mcp_think: self-issued (the model chooses to call it, can skip it), a verbose blockquote ritual, easy to game. This gate needs NO special tool call and NO blockquote.
- liteGAME evidence-gate: counts READS as a proxy for thinking — but reading 10 things IS the behavior to stop. This gate measures REASONING (visible transcript text), not reads.

HONEST LIMITS: it enforces that reasoning HAPPENED and is VISIBLE, not that it is CORRECT (quality is JUDGMENT, not hookable). Residual gaming: a model could emit filler to clear the substance bar — mitigated by MIN_CHARS + the target-reference check, not eliminated. Calibration: too aggressive = latency + nag; REFLECT_MODE/K tune it. This REPLACES the entire GRC/mcp_think apparatus in CLAUDE.md — that prose block is DELETED and the reasoning discipline becomes a gate instead of a hope. (mcp_think can stay available as a pure opt-in decision-ledger tool, but is no longer mandated or echoed.)

================================================================
9. DESIGN CHOICES — RESOLVED (grill 2026-06-23)
================================================================
- fail-mode: ALL gates fail-CLOSED (D2). Recovery out-of-band from the ungated F: session. (No per-row split.)
- commit-approval sentinel: dashboard one-shot button (D4); typed-phrase rejected (self-approval hole).
- .ps1 -> python: STILL TRUE and now larger — every adopted liteGAME gate (incl. the full unity set, D10) is PowerShell and must be re-implemented in python for WSL. Logic ports, shell does not. The roslyn UnityBannedApiAnalyzer is compiled C# and runs as-is (needs the .NET toolchain in the unity context). The asset-pipeline gates carry their own conformance fixtures.
- prompt-transform: conditional prompt-engineer WHEN-clause (D8).
- python_wire_check.py RecursionError: fixed now + R8 re-lock, folded into the freeze ceremony (D9).
- still genuinely build-time (not grill decisions): the exact MIN_CHARS value, the read-everything ceiling numbers, and the reflect-gate burst-K (only used if REFLECT_MODE is ever loosened to mutation-priority) — tune empirically in G-2/G-4.

================================================================
10. CONTEXT MATH (what is actually saved)
================================================================
Honest accounting:
- teikoku's CLAUDE.md is already 43 lines, so removing sticks from IT is a small win. The real always-on cost in the teikoku RUNTIME is the toggled rulebook/rules prose injected into each worker's cached system prompt (teikoku-chat-memory work) + whatever judgment prose stays.
- Moving the ~30 hookable rule-families out of prose into hooks removes them from every worker system prompt (they're enforced by the runtime, not recited).
- Moving the ~60 reference docs to RAG removes them from any always-on path (retrieved only on trigger).
- Deleting the mcp_think/GRC mandate (owner: not working, not good enough) removes the largest single always-on block + the per-response blockquote tax; the new Reflection Gate (§8) replaces it with RUNTIME cost (a transcript read per tool call), not context cost.
- The hooks add RUNTIME cost (each tool call runs a hook — a few ms of python), not CONTEXT cost. That is the correct trade: spend cheap runtime determinism to buy back context window + real enforcement.
- The bigger prize than tokens: the handoffs prove prose-only enforcement FAILED (FSM resets, git reset --hard x8, wrong-repo writes). Hooks make that class structurally impossible — which is what agent-dispatch-preamble.md only CLAIMS today.

================================================================
11. BUILD SEQUENCE (how to actually ship this in the teikoku distro)
================================================================
This is a real multi-file implementation in ~/teikoku, base-cc-native, WSL-pathed. Suggested phasing (each its own C-02 cycle or a scoped agent, per the teikoku contract):
  G-1  Hook harness + settings.json scaffold: ~/teikoku/.claude/settings.json + hooks/ dir + a base-cc hook smoke (prove a trivial PreToolUse deny actually blocks a tool call in a base-cc --print turn — the ONE unverified assumption; if base-cc --print does not fire PreToolUse, the whole plan re-scopes). VERIFY FIRST.
  G-2  The three owner sticks: commit_gate, comment_gate, full_read_flag+read_full_gate (highest user value, lowest risk).
  G-3  The safety sticks: bash_destructive_gate, scope_write_gate, frozen_gate_guard (the data-loss-prevention set the handoffs justify).
  G-4  Reflection Gate + posture cleanup: build reflect_gate.py (the §8 transcript-grounded roadblock); DELETE the mcp_think/GRC mandate from CLAUDE.md entirely (owner: not working, not good enough); register change_recorder + syntax_lint as PostToolUse. (No liteGAME evidence-gate, no record_evidence ledger — dropped per owner.)
  G-5  RAG reference migration: index the REFERENCE corpus into memory_semantic; convert conditional-routing to a retrieval map; trim CLAUDE.md to carrots.
  G-6  Unity pack gating (D10 — FULL PORT, a sizable sub-project): port the FULL liteGAME unity set (~48 GAME-tagged files: verify-before-write, scan_unity_patterns, validate_lifecycle, uss-taste-lint, ui-diff/vlm-prep, roslyn banned-API, unity_map extractor, the asset pipeline + image utilities) into the "unity game" toggle so they load only when active_packs includes unity. .ps1 -> python; roslyn runs as compiled C#; pipeline gates bring their conformance fixtures. Dedup against teikoku's existing AOR unity gates (toban001/mono_wire/unity_symbol). Likely its own multi-cycle effort.
  G-7  Dedup pass: collapse the ELAI/liteGAME duplicate rule-families (both indexes list them) to one shared core + packs; drop the dead residue (codebase_mapper/query_bm25/find_bm25_new with hardcoded F: paths; the node-not-bun canvas hook).
  Gate-freeze: add the new hooks + checkers to gate-manifest.txt and lock, so a future run cannot weaken its own gate (R8).

TWO THINGS TO VERIFY BEFORE ANY BUILD (G-1 kill-questions):
  (1) Does base-cc fire PreToolUse hooks on a `--print` headless turn (the transport every teikoku chat + worker uses)? base-cc SUPPORTS hooks (verified in source), but whether they fire in --print vs interactive is unconfirmed. If they don't fire in --print, the whole plan re-scopes.
  (2) Does the PreToolUse hook payload expose the session transcript_path? The Reflection Gate (§8) needs it to detect reasoning. If not exposed, the gate falls back to a WEAKER counter-based roadblock (deny every Nth consecutive tool_use to force a text turn) — still a roadblock, but blind to whether the text is actual reasoning.
Both are answered by one cheap probe: a trivial deny-hook + settings.json in the distro, one base-cc --print turn that tries a tool, inspect the hook payload. Do this before a line of gate logic is written.

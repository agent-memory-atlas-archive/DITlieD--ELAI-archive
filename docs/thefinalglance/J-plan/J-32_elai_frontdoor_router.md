# J-32: elai front-door router — chat = ELAI, not a model; total-function lane routing with journaled, calibrated, chip-confirmed dispatch

STATUS: PLANNED (WAVE CFW-1)

- Track J | lever STEERING+CANT-WRECK | grounding STRONG (every substrate claim code-verified 2026-06-12) | nature BUILD-on-shipped | effort M-L | WAVE CFW-1 | Tier-2 dep: lm_format_enforcer install on the remote sidecar (ops checkbox; T1 fallback ships dark of it)

## Premise

Today the chat IS a model: free text goes straight to the streaming path (commands.svelte.ts:337
submit -> _runStreamingPath -> start_chat_stream at commands.rs:5839 -> sidecar/Anthropic SSE).
The only routing is a frontend-only TS regex hint (classifyIntent, intent.ts:89, 10 intents) plus
the RESEARCH chip; the Rust classify_intent Tauri command is registered-but-uncalled
(commands.rs:5451) and the CLI classifier (classifier.rs:146) is test-only. No Rust-side routing
of a free-text message to subsystems exists (verified 2026-06-12); no front-door/intent-router
plan unit exists in H/I/J (grep: zero hits — GD-1 clear). J-32 makes the chat surface ELAI
itself: no /commands needed, any message routes to a LANE or to plain chat, sessions become
transcript hygiene only, and the user-facing identity is "ELAI" with role badges (E-3 doctrine),
never a model name.

## Design doctrine (the hard question answered)

"How can a closed set cover anything a user types?" — TOTAL-FUNCTION ROUTING: the closed set
enumerates LANES (entrypoints), not meanings, and CHAT is the default lane. Every message resolves
to exactly one of {execution lane, CHAT, CLARIFY} — the function is total by construction. The
front door is a DISPATCHER, not a comprehender: deep understanding is each lane's own machinery
(B-40 IdeaFramer + AmbiguityScorer for tasks — SHIPPED, ipc_gateway.rs:232; the research
orchestrator for research), so the router stays simple by design. Mislabeling INTO chat is
harmless (you get an answer, not an action); mislabeling INTO execution is chip-visible and needs
Enter. The probabilistic leg can only pick a label from the closed set and point at byte-spans of
the user's own words — it never authors instructions, never rewrites the message, never gates a
deterministic feature (T0 bypass always exists). This is TD-3 applied to the chat surface: the
LLM proposes a label; deterministic systems and the user dispose.

## Steal/build (6 legs)

1. TIER LADDER. T0: slash commands = deterministic bypass, kept forever. T1: deterministic
   matchers — promote the SHIPPED TS classifyIntent regex set (intent.ts:89) into a Rust
   FrontDoorRouter consumed by BOTH the dashboard (start_chat_stream seam) and the CLI (live_cli
   run_turn seam); the TS classifier demotes to an instant-UI-hint only (one classifier of
   record, in Rust, journal-visible). T2: a local-sidecar 3B classification call at temp 0,
   LMFE-constrained to a closed JSON schema (E-35 substrate: DecodingMode::JsonSchema,
   decoding.rs:73; the canonical sidecar gates constrained decode on ELAI_LMFE_INSTALLED and 503s
   without it — UNTIL the ops install lands, T2 degrades to T1 + CLARIFY chip: fail-closed to
   ASKING, never guessing). T2 output: { lane: enum, confidence: f32, arg_spans: [byte-ranges] }.
   ARG SPANS, NOT PARAPHRASE: the lane receives the user's literal bytes; rewrite is structurally
   absent and contract-tested.
2. CONFIRMATION. Generalize the SHIPPED research chip (pendingResearchChip,
   commands.svelte.ts:87; Enter/Esc/Tab at CommandWindow.svelte:244-254) into THE intent chip for
   all mutating lanes ("ELAI will start: <lane> on <scope> — Enter run / Esc cancel / Tab edit").
   Read-only lanes (CHAT / EXPLORE / status / cached research) auto-run chipless. Risk mapping
   rides the SHIPPED RiskLevel machinery (default_risk_level, gate.rs:238; contract-tested over
   all FSM transitions): LOW auto, MEDIUM chip, HIGH always-confirm.
3. AUTONOMOUS = APPROVE-ONCE. Rides the SHIPPED GateMode (B-22, autonomous_gate.rs;
   GateMode::from_cli_args; wired at runtime_setup.rs:896). In Autonomous, MEDIUM chips become
   non-blocking notice cards (journaled, scrollable); HIGH still escalates (shipped
   TreeAutoAdvancePolicy.high_risk_pauses + the four escalation triggers). The "approve once at
   the start" artifact = the H-37 plan-approval card (SHIPPED-DARK behind
   ELAI_PLAN_APPROVAL_GATE_ENABLE; task_runner.rs:5953/6824 + PlanApprovalCard.svelte): the first
   message frames the plan, ONE approval card, then it runs. J-32 activates H-37 on this flow
   (W-2: the dark flag stops being a terminal state).
4. ANSWER-SURFACE CONTRACT (what the user sees, the three stages):
   (a) ACK card, immediate + deterministic: lane + scope + an "Open in canvas ->" deep-link that
   switches CanvasWindow to the lane's mode (system/plan/research — shipped mode flag) and
   focuses the relevant node via the shipped focus bus. (b) PROGRESS: the canvas is the live
   workspace (shipped SSB overlay); the chat renders compact digest cards off phase_transition /
   bubble-up rows it already has — never raw token spam from worker agents. (c) RESULT card:
   outcome + RECEIPTS — files touched, gate verdicts, audit verdict, evidence row ids (the G-5
   cockpit principle: every claim joins a journal row, GD-2). The CHAT lane keeps today's
   streaming reply verbatim.
5. CALIBRATION FLYWHEEL. Every chip decision (Enter / Esc / Tab-edit) journals on the
   IntentRouted row = a free labeled example. The T2 confidence threshold starts as a fixed
   conservative default and graduates to a D-16 conformal tau computed from the accrued tuples
   (conformal_calibration.rs decide():697 — SHIPPED; route through the I-23 facade when it ships,
   one tau, no drift). Abstention over guessing is the explicit failure mode.
6. SESSION SEMANTICS. New session = clean transcript only. Lane/model selection is per-message
   via the shipped PerRoleRuntime + CloudModeRouter (runtime_setup.rs:772); the chat identity is
   ELAI, role badge shown, model hidden by default (E-3).

## Existing ground truth (code-verified 2026-06-12; re-grep at edit, W-7)

- intent.ts:89 classifyIntent SHIPPED (production caller commands.svelte.ts:340); research chip
  SHIPPED (CommandWindow.svelte:244 Enter/Esc/Tab); Rust classify_intent registered, uncalled
  (commands.rs:5451); CLI classifier.rs:146 test-only.
- E-35 DecodingMode SHIPPED (decoding.rs:73; consumers sidecar_client.rs:461,
  sidecar_xgrammar_hook.rs); canonical sidecar = HF + LMFE, gated ELAI_LMFE_INSTALLED (503
  without); XGrammar lives on the SGLang backend only. Plain FreeText classification works today.
- B-22 AutonomousGate/GateMode/TreeAutoAdvancePolicy SHIPPED (interactive default;
  runtime_setup.rs:896); Phase 3A RiskLevel SHIPPED + contract-tested (gate.rs:238/640).
- H-37/H-38 SHIPPED-DARK (commands_h37.rs:68, PlanApprovalCard.svelte, contract tests; default
  OFF). D-16 conformal SHIPPED with CLI consumer; I-23 facade PLANNED-ONLY.
- B-39 REFLECT: CLI-wired (live_cli.rs:1749), dashboard streaming path not — the CHAT lane is
  where that wiring lands naturally (rider below).
- B-40 IdeaFramer + AmbiguityScorer SHIPPED. B-37 PerRoleRuntime + RuntimeOrchestrator +
  CloudModeRouter wired (RouteDecisionEvent emit dark behind ELAI_ROUTE_DECISION_SSB_ENABLE).

## REVISED scope (W-10 tiered)

Tier-1: Rust FrontDoorRouter (T0/T1 + chip generalization + risk mapping + ACK/RESULT cards +
IntentRouted rows) — fully deterministic, ships without any model dependency, default-ON.
Tier-2: T2 constrained classification (gated on the LMFE ops install), conformal tau graduation,
canvas deep-link focus on SYSTEM-mode nodes (J-31), cloud-fallback classification (REQUIRES the
E-20 redaction mask first; recorded, not in Tier-1).

## Integration

- NEW crates/orchestrator/src/front_door.rs (~300 LOC): lane enum, T1 matchers ported from
  intent.ts, T2 sidecar call via DecodingMode::JsonSchema, risk mapping, span extraction.
- Seams: commands.rs:5839 (start_chat_stream entry) + the CLI live_cli run_turn entry — ONE
  router, two consumers (the B-14 audit's split-brain closed).
- frontend_pair: intent-chip generalization in commands.svelte.ts / CommandWindow.svelte + ACK /
  notice / RESULT cards + the canvas deep-link button (cross_lang_diff + svelte-check in-unit).
- SSB: +IntentRouted (Pattern-2 single-variant; Kind: Classified / Confirmed / Rejected /
  Bypassed / Clarified). baseline+1 from LIVE 241/550 (grepped 2026-06-12; re-grep at edit, W-9;
  full 6-part variant discipline, golden-lock 9-tuple lockstep). Payload: lane, tier, confidence,
  arg_span offsets, decision, risk level — NEVER the rewritten text (there is none).

## observable_effect (done-gate, W-1)

Typing "fix the failing test in sum.rs" (no slash, interactive) lands IntentRouted(Classified
{lane=fix, tier}) + the chip; Enter dispatches the IDENTICAL journal-row sequence the explicit
/fix control run produces (GD-2 diff = zero rows differ beyond ids/timestamps); Esc lands
Rejected + ZERO dispatch rows. Autonomous fixture: a MEDIUM lane auto-dispatches with a notice
card; a HIGH destructive fixture ("wipe the old sessions") still blocks on confirm despite
autonomous mode. NEGATIVE (R2): sidecar down -> CLARIFY chip, never a silent guess (R1
fail-closed); a broken arm where the router paraphrases args FAILS the contract test (arg_spans
must be byte-offsets into the original message, asserted on the journaled row); a mutating lane
dispatched chipless in interactive mode FAILS; a lane value outside the enum is unrepresentable
(closed set, serde deny_unknown_fields per J-22 discipline).

## Riders (recorded, not in-scope)

- Wiring B-39 REFLECT into the dashboard CHAT lane = the natural follow-on once the lane exists;
  recorded, not bundled.
- Multi-intent messages ("do X then Y"): the task lane's decomposition owns sequencing; the
  front door does NOT split messages (keep the door dumb).
- Voice input rides the existing dictation path unchanged; the router sees text.
- Cloud-fallback classification (E-20 redaction-masked) = Tier-2 checkbox, never default.

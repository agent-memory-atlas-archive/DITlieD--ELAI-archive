# ELAI dashboard user flows

Input document for the dashboard DESIGN.md. Synthesized 2026-08-12 from the
"User-facing surfaces and flows" section of
`docs/thefinalglance/ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md` (lines 51955-60945,
76/76 deep ledgers). Per-family dense extracts live in
`surface-digests/01..07-*.md` beside this file; every claim here carries atom IDs
that resolve there and in the master list.

The owner-accepted user-flow model is **section 1.1** (design decision dated 2026-08-16). It supersedes the
2026-08-12 "session board + tabs" ruling. Read section 1.1 before any older IA text
in this file.

[Companion plan](PLAN.md) · [Surface digests](surface-digests/) ·
[Master feature list](../../../docs/thefinalglance/ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md)

## 0. How to read this file

- Statuses are **plan-recorded**, not runtime proof. Vocabulary: shipped/DONE/WIRED,
  planned/BUILD, mixed/partial, wire-dark/DARK, DEFERRED, superseded,
  REJECTED-by-measurement. Per-leg banners beat coarse headers.
- Newer-plan precedence: explicit markers beat everything; else later letter family
  wins (B oldest → Q newest); higher plan number wins inside a family.
- Owner-ruling precedence: an owner ruling beats plan-recorded IA, and a later owner
  ruling supersedes an earlier one. Section 1.1 is the current ruling. A superseded
  ruling stays in this file only as a marked history note.
- Wire-dark honesty (INV-C07): the UI must never imply a live guarantee for a dark
  path. Many surfaces below exist as designs behind default-off flags.
- A huge share of proposed UI is "ride an existing panel OR justified N/A" (W-4/W-6
  doctrine). Those are **candidates**, not committed windows. Section 9 separates them.

## 1. What the dashboard is

One desktop app: `crates/dashboard-v2` — Tauri 2.10 + Svelte 5 runes + Vite 5
[B0-002..004, INV-F01]. The product stance is a **calm control room**: nine windows,
about sixty drawers, one command palette, outlined-only chrome, no solid CTAs
[E33-003/004, B12]. Under the 2026-08-16 ruling that control room is **three
dockable panels**, not nine fixed windows: the window and drawer lists below are
component inventory (section 4.5). Everything it shows is a projection of the
append-only SSB journal (INV-B01); numbers must recompute from the journal under
`ELAI_SEED` (INV-B03/GD-2). Window layout is local-only (`localStorage` key
`elai.dashboard-v2.workspace.v1`, no cross-device sync) while user-global knowledge
lives in `~/.elai/cosmos/` [INV-F03, SUM-0189]. Named dockable layouts replace that
single workspace key, and that storage shape is net-new (section 9).

Four audiences reach the same substrate through different render targets [KD-2, N-7]:
- **Operator (solo dev)** — the dashboard itself plus CLI.
- **Non-coder / one-shot user** — `elai build` one-field door (L-14), fleet
  Attention cards (P-1/P-5), plain-language diagnosis.
- **Reviewer** — G-5 cockpit (vouched vs unvouched hunks, DSR headline).
- **Buyer / org admin** — PR check-runs + Evidence section (K-3/K-11), compliance
  export (K-6), fleet rollup (M-3). PR is the buyer view; dashboard is the deep view.

## 1.1 Owner-accepted user-flow model (owner ruling 2026-08-16)

This is the target model. It supersedes the 2026-08-12 session-first window ruling
quoted at the end of this section. Sections 2 to 4.5 keep the older project-first IA
only as **component inventory**, never as the target shell. Open items live in
decision #17.

**Identity.** ELAI is one persistent user-facing conversation and the orchestration
layer between the user and the models. The conversation is a deterministic read
projection of the journals (INV-B01, INV-B03/GD-2). It is never shared model context,
so no run reads another run's context through it.

**Orchestration session.** A user-created orchestration session is a small durable
container for one goal, a default scope, and the approved decisions shared by work
under that goal. It is not a chat, not a workspace, and not an execution unit.

**Execution run.** A normal user request creates a **new isolated execution run**.
Only an anchored reply or an explicit run target updates an existing run. Ambiguous
targeting must ask one clarification and dispatch nothing, which is the J-32
fail-closed CLARIFY contract [J32-E009/E011].

**Run ownership.** Each execution run owns exactly one FSM driver, its authoritative
SSB journal, its context, its budget, its resolved scope or worktree, its
cancellation path, and its receipt. One run never gets two concurrent drivers.
Several non-conflicting runs can operate beneath one orchestration session. Unknown
or conflicting scope must queue or refuse. Heuristic dependency evidence can never
authorize parallelism and can never open a gate: Sound edges only, heuristic stays
advisory [K-23 Sound vs Heuristic; I-2; J-60].

**Supervisor journal.** Visible cross-run ordering comes from one **pointer-only,
single-writer supervisor journal**. Run journals stay authoritative and the
supervisor journal holds no run state. A pointer carries the orchestration session
id, the run id, the run-journal row sequence or id, a content hash, and the event
kind. This is net-new engineering; the nearest substrates are the phase-3B SSB
single-writer journal and the K-23 `workspace.ssb.db` global journal (section 9).

**Hierarchy.** The user-facing hierarchy is orchestration session > execution run >
stable work unit > agent attempt. A work unit is a stable name for a piece of work.
An attempt is a disposable try under that work unit.

**Panels.** The three primary dockable panels are `Sessions`, `ELAI`, and `Work`
(section 4.1). This replaces per-session workspace tabs.

**Minimal controls.** Selecting work and addressing ELAI is the primary steering
path, not a control per row (section 4.4, section 6 F3).

**Review boundary (owner ruling 2026-08-17).** Every execution-run result requires
user review before it can enter the accepted working state. A clean verified result
is not auto-accepted. It stays isolated and rises into Attention as **Review
required**. Accept is one command: it records the human verdict and starts the normal
deterministic merge-back flow. A clean merge updates the accepted working state. An
overlap or stale-target conflict halts atomically and leaves the accepted target
unchanged. The effective GateMode at that conflict decides the next step: Interactive
raises Needs user for manual resolution; Autonomous starts a bounded sequential loop
of isolated resolution attempts. Every successful resolution candidate still returns
as Review required before another Accept. Equivalent conflicts or the existing
anti-spiral limit stop the loop and raise Needs user. Decline keeps the result out.
Retry creates a new attempt under the same work unit. Accept never means commit,
push, deploy, publish, or any other irreversible external action.

> **HISTORY, SUPERSEDED 2026-08-16: owner ruling 2026-08-12 "session board +
> tabs".** What survives: ELAI is the layer between models and the user, a harness
> over the whole PC that works on any project, several at once, or none (research,
> fleet, browse); and the scope ladder, now read as run scope resolution (rootless,
> a worktree of a project, a whole project root, or a multi-root K-23 workspace).
> What is superseded: the window model recorded that day, meaning home as a command
> center with a live session board, each session as a switchable workspace tab with
> its own windows, canvas, drawers and per-session layout persistence, the
> per-session `workspace.v1` key, and the dual-panel pair pinning a second session's
> monitor. One persistent ELAI conversation plus the `Work` database replaces all of
> it. ProjectSelector survives only as a scope-picking component inside session
> creation, never as the app's front page [supersedes B12-006 as entry IA].

## 2. Onboarding: install to first task

Steps 3 and 7 carry the accepted entry path (section 1.1). Window names in steps 4 to
6 are component inventory (section 4.5), not a target route.

1. **Install** — B-43 installer is CLI-only (HTTPS fetch, blake3 binary verify,
   `credentials.json` 0600, `elai_cc.toml` template with VRAM + B-37 runtime
   defaults). Packaged Tauri app belongs to B-45 [B005-F08]. At install end the
   installer offers license activation in one line [M1-E022].
2. **License (optional)** — `elai license activate <file|key>` / `status` /
   `deactivate`. Tiers come from `tiers.toml` (Free = everything through family L
   except Team-keyed features). Expiry → 14-day **ExpiredGrace banner** →
   DegradedToFree. Banner placement/copy is open [M-1, M1-A002]. There is
   deliberately **no SettingsWindow license line** (superseded) [M1-C001].
3. **First request, not first project**: the entry surface is the ELAI conversation.
   One line creates an orchestration session: ELAI proposes the session name, the
   goal, and the scope, and the user accepts or edits the proposal. The **L-14
   one-field "build from an idea"** input is that same entry line, not a separate
   screen [L14-A002]. **ProjectSelector** survives as **component inventory** for the
   scope step only: New Project (name + parent picker, template picker
   Svelte/React/Astro/Vanilla from `templates/web/manifest.toml`), Open Existing
   (native folder dialog), Recent (max 10) [P4K-E029..E034]. It asked for the project
   and nothing else, with no shell or layout wizard [B12-006/024], and it never
   returns as the front page [B12-006 superseded as entry IA].
4. **First-run copy** — three short paragraphs in CommandWindow: what ELAI does,
   does not do, and its moats. maintainer voice, no em dash [E-23].
5. **IdeaFramingWindow** — for a new idea the first task passes framing:
   framed idea + TradeoffCard radio answers; FRAMING → PLANNING; critic rounds N/3
   visible in Monitor [E-B40-017, B41].
6. **Runtime/keys** — RuntimePanel is the BYOK home: pick backend → masked key →
   optional base-url → **live test call** → atomic save only on success (J-30).
   VRAM tier auto-detect with `--vram-tier` override [P4D-E049]. First-run
   onboarding is named as a consumer of this panel, not a separate route [J30-I001].
7. **Panels, not a project workspace**: after the first send the three panels mount
   and stay (section 4.1). Index build kicks off with a progress panel, resumable,
   with lock-contention banners [B31-043]. Layout is local-only named layouts, not a
   per-session tab set [INV-F03, SUM-0189 superseded].

Commercial spine around this: signed updates `elai update check|apply|rollback|channel`
(boot-time non-blocking check), `elai support bundle` (redacted, prints contents
before you share), org policy drift boot warning (M-2), fleet report (M-3),
`elai-verify` standalone binary so a buyer can verify receipts on a clean machine
(M-5/M-6).

## 3. The front door: one input, routed (J-32)

Identity rule: the chat is **ELAI**, showing action-oriented role badges (E-3),
never a model name. One classifier of record routes both dashboard chat and CLI:

- **T0** slash commands always bypass. **T1** deterministic Rust FrontDoorRouter
  (default-ON). **T2** local-3B constrained classifier; until it ships, ambiguity
  fail-closes into a **CLARIFY chip** — never a guess [J32-E009/E011].
- **Risk tiers**: LOW auto-runs chipless (read-only lanes: CHAT/EXPLORE/status/
  cached research). MEDIUM shows the **intent chip**: "ELAI will start: <lane> on
  <scope> — Enter run / Esc cancel / Tab edit". HIGH always confirms [J32-E013..E015].
- **Autonomous mode** (GateMode): approve-once via the H-37 PlanApprovalCard;
  MEDIUM becomes non-blocking journaled notice cards; HIGH still pauses
  [J32-E016/E017]. Card layout for those notices is open [J32-A003].
- **Card lifecycle**: deterministic **ACK card** (lane + scope + "Open in canvas →"
  deep-link) → **PROGRESS** as compact digest cards from phase transitions (never
  raw worker token spam; the canvas carries the live view) → **RESULT card** with
  receipts: files touched, gate verdicts, audit verdict, evidence row ids
  [J32-E018..E020].
- Chip Enter/Esc/Tab feed a labeled-example calibration flywheel; Esc must produce
  zero dispatch [J32-E021/E035].
- **Voice** (J-111/J-112, default-off `ELAI_VOICE_ENABLE`): push-to-talk local ASR
  enters the same router as text; MEDIUM+ gets a spoken readback of the routed
  intent and waits for a confirm token; HIGH additionally requires a non-voice
  confirm. Outbound, the NotificationDispatcher narrates completion / needs-input /
  status digests with deterministic phrasing; text stays canonical on the dashboard.
- Adjacent chips: **Oracle research chip** (auto-detected research intent, one-key
  confirm, never silent) [B0-047..052]; **grill offer chip** on high ambiguity —
  offered, never auto-run (J-79); **OneShotClarifier** asks at most 3 yes/no
  questions once (E-4).
- CommandWindow mechanics: Ctrl+Enter submits, Enter is newline. Stop is the same
  run cancellation action exposed from `Work`, not a second cancellation path;
  skeleton bubble + CSS cursor + "reading the project…" cue pace perceived latency
  (E-2); `/` opens the slash dialog grouped Workflow / Project / Help [B14, C-6].

**Run targeting (owner ruling 2026-08-16).** The front door now also decides which
execution run a message belongs to:

- A normal message creates a **new isolated execution run** inside the targeted
  orchestration session.
- Only an anchored reply to a run's message, or an explicit run target, updates that
  run.
- Ambiguous targeting asks **one** clarification and dispatches nothing. This reuses
  the CLARIFY contract: fail closed, never guess [J32-E009/E011].

**Composer controls (minimal by rule).** Three controls only: the orchestration
session target, a `Start now | Plan first` selector, and Send. `Start now` is the
default and resets to `Start now` after every submission. `Plan first` creates
the run in PLANNING and delays execution until the H-37 plan approval gate.
Everything else stays conversational text or a contextual menu.

**Conversation load control.** Routine run starts and progress fold into **one
updating digest message per orchestration session**. Questions, failures, and final
results stay individual messages. Every terminal result message carries **Review
required** until the human verdict lands. The card grammar survives inside the digest
[J32-E018..E020]. The complete trail lives in `Work` (section 4.4), never only in the
conversation.

## 4. Shell anatomy

### 4.1 The three primary panels (owner ruling 2026-08-16)

- `Sessions`: the orchestration sessions list (section 4.2).
- `ELAI`: the one persistent conversation and the composer (section 4.3).
- `Work`: the journal-derived operations database over every orchestration session
  (section 4.4).
- Positions are **defaults only**. The user can dock, move, resize, stack, collapse,
  detach into an OS window, and save named layouts. Single-instance focus-existing
  stays [P4K-E026].
- This replaces per-session workspace tabs. Layout persistence is local-only and the
  named-layout store is net-new [INV-F03, SUM-0189 superseded; section 9].
- The shell chrome is global because per-session workspaces are gone. The command
  palette and the Attention count are app-level. Run-specific StatusBar and usage
  fields follow the selected run and show an honest no-selection state. Canvas and
  drawers ride the panel that hosts them.
- Minimal controls is a core goal: three panels, one composer, one Attention count.
  Every added control needs a reason.

### 4.2 Sessions panel

A session row carries the goal line, the resolved default scope (rootless, a
worktree, a project root, or a multi-root K-23 workspace), the live run count, and
its Attention count. Creation is conversational (section 2 item 3). The durable
session lifecycle is **Active | Parked | Archived** (owner ruling 2026-08-17).
**Park is quiescent**: the transition is allowed only when every execution run under
the session is terminal, already paused, or cancelled. Park never sends a bulk Pause
or Cancel. While Parked, new user requests and scheduled wakes for that session are
withheld and dispatch nothing. Scheduled trigger causes remain journaled as withheld
work for deterministic handling after reactivation; they are not silently dropped.
The session, its conversation messages, Work rows, and Attention items remain visible
and readable. Reactivate returns it to Active and restores admission only. It does
not auto-resume paused runs or dispatch withheld wakes. Instead, reactivation seals
one immutable **withheld-wake batch per schedule per Park interval**, covering only
previously unbatched trigger rows through its journal watermark. Each batch is a
Needs user Attention item with the schedule, missed count, first and last trigger
time, and covered trigger-row range. **Run once** records one decision and creates
one normal isolated run that executes the TaskSpec once against current state, with
the batch receipt attached as trigger provenance. It does not replay each tick.
**Dismiss** records one decision and creates no run. Both actions resolve only the
sealed range, are idempotent, and preserve every original trigger row. Ticks after
the reactivation watermark follow the normal Active schedule path. Autonomous mode cannot decide a withheld batch. Automatic parking and
replay-every-wake are prohibited.

**Archive is cold storage** (owner ruling 2026-08-17). Archive is allowed only from
Parked when every run is terminal or cancelled, the session has zero unresolved
Attention items, and every withheld-wake batch has a Run once or Dismiss decision.
A paused or otherwise resumable run blocks Archive. The single-writer transition is
atomic and has no side effect beyond lifecycle state. Archived sessions are removed
from the default Sessions list, composer targets, and `Work` views, including Board,
Timeline, and Canvas. Past ELAI messages remain in the one conversation's immutable
chronological scrollback and search; the archived session digest freezes. Global
search or explicit `include archived` opens the session's complete read-only Work
history, receipts, and journals. Archive never hides a later safety or Attention item;
such an item resurfaces globally while the session remains Archived. Schedule
definitions remain stored, but Archived suspends schedule evaluation: no trigger rows
or wake batches accrue during the Archived interval. **Restore** is manual, returns
Archived to Parked, and restores visibility only. It does not dispatch, resume, or
backfill work. Schedules become eligible only after a later Parked to Active
transition, starting at the first due point strictly after that recorded Active
transition. Automatic Archive and automatic Restore are prohibited. Archive never
deletes data.

**Planned lifecycle contract, net-new.** The authoritative session registry must
append `SessionLifecycleChanged { session_id, from, to, actor=user, row_seq,
transition_at }` for every transition and enforce this closed matrix:
Active to Parked, Parked to Active, Parked to Archived, and Archived to Parked. Parked to Archived must atomically verify zero nonterminal
runs, zero unresolved Attention, and zero undecided wake batches. The scheduler
admission hook must treat Archived as suspended, distinct from Parked-withheld, and
append no trigger row. A later Parked to Active transition uses that recorded Active transition
time as the lower bound and selects the first schedule due point strictly after it;
no Archived interval backfill exists. Default session and Work projections exclude
Archived. The conversation projection retains immutable prior messages. Explicit
archived reads remain read-only, while safety or Attention rows override default
hiding globally. Missing transition guards, visibility override, scheduler
suspension, no-backfill rule, or restore-to-Parked enforcement is a wire-dark failure
[INV-C07]. This product lifecycle is unrelated to F-13's planned schema-bump file
copy. F-13 preserves stale database bytes before migration and does not provide an
orchestration-session Archive state, lifecycle transition, scheduler suspension, or
Restore flow.

Archive semantics are now closed.

**Planned wake-batching contract, net-new.** O-2 currently plans Cron/Manual fire
rows and explicitly defers missed-run detection. It does not define orchestration-
session ownership, Park intervals, withheld wakes, batch sealing, or decisions. This
flow therefore requires an O-2/session-registry extension and must render wire-dark
until that extension ships:

- Each scheduled TaskSpec binds to an `orchestration_session_id`.
- The authoritative single-writer session registry owns lifecycle admission. Before
  the existing scheduler tick dispatches, it reads that session's lifecycle. Parked
  appends `ScheduledWakeWithheld { session_id, schedule_id, park_interval_id,
  trigger_row_seq, trigger_at, trigger_cause_hash }` and creates no run.
- `park_interval_id` is the Park transition row id. Reactivate is one atomic registry
  transaction: capture `seal_watermark`, seal all previously unbatched withheld rows
  at or below that sequence by `(session_id, schedule_id, park_interval_id)`, append
  Active, then release admission. Later ticks therefore take the normal Active path.
- `WakeBatchSealed` carries immutable `batch_id`, session and schedule ids, Park
  interval id, missed count, first and last trigger time, first and last trigger row
  sequence, and seal watermark. The unique batch key is
  `(session_id, schedule_id, park_interval_id)`.
- `WakeBatchDecision { batch_id, RunOnce | Dismiss, decision_row_seq,
  resulting_run_id? }` is unique by batch id. RunOnce enters the normal isolated-run
  creation path once and binds the batch receipt to the run. Dismiss creates no run.
- The pointer-only supervisor journal references these authoritative registry and run
  rows for visible ordering. It does not duplicate lifecycle, wake, batch, or run
  state.

Missing ownership, rows, watermark, unique key, or production admission hook is a
wire-dark failure, not an inferred implementation [O-2, K-14, INV-C07].

Accept is the sole
merge-back trigger, and section 6 F4 fixes conflict behavior by GateMode. A registry
above the per-run journals is net-new; nearest substrates
are C-23/E-B9 session branches, C-17 auto-resume, and the K-23 `workspace.ssb.db`
journal (section 9).

### 4.3 ELAI panel

The persistent conversation, the composer, and the per-session digest message
(section 3). It is a deterministic read projection, so every message re-derives from
the journals and the same conversation renders under any layout. It is never shared
model context.

### 4.4 Work panel: the operations database

`Work` is a **derived operations database**, never an authoritative store. Every
column is a journal fold, and nothing writes execution state through a `Work` row
[INV-B01].

- **Default open**: Table view, all Active and Parked work across every orchestration
  session, grouped by orchestration session. Archived work requires global search or
  explicit `include archived`.
- **Default visible properties**: Run, orchestration session, Scope, Phase, Agents,
  State, Last meaningful update. **State and Phase are read-only journal folds**. The
  model never sets status [TD-3, J-60 §Steal/build 2].
- **Built-in views**: All work, Attention, Board, Timeline, Canvas. `All work` means
  Active plus Parked by default; Archived is opt-in. Safety and Attention overrides
  still surface globally from Archived sessions.
- **Saved views**: the user can save custom views with filters, grouping, sorting,
  columns, density, and opening style. The global **Attention count cannot be
  hidden**.
- **Drill-down**: opening a run shows its **stable work units** first. Opening a work
  unit reveals its **agent attempts**. Healthy attempts stay collapsed into counts,
  which is what keeps forty or more concurrent agents readable.
- **Escalation**: needs-input, failed, stalled, conflicting, expensive, or completed
  results awaiting review rise into **Attention**. `Review required` is a read-only
  journal fold: a terminal result exists and no human verdict follows it. Every
  finished execution run stays isolated until the user reviews it (section 6 F4).
  Nothing else interrupts (section 7).
- **Actions**: select the work, then address ELAI. Avoid persistent row buttons.
  Secondary actions live in contextual menus. Review, Pause, and Cancel are commands
  to the single-writer engine, never direct writes through a `Work` row. Their
  journal rows make `Work` re-derive. Pause and Cancel must stay reachable because
  they are safety controls.
- **Honesty**: honest empties and explicit unknowns apply here as everywhere, and a
  wire-dark leg never renders as live state [INV-C07, J29, J31].

### 4.5 Component inventory (older project-first shell, not the target IA)

Read this list as parts, not as the shell. Every entry predates section 1.1.

- **Workspace** owns everything after ProjectSelector: AppShell → 48px three-button
  **IconRail** + Sidebar → primary/secondary **Panel** pair with draggable
  PanelDivider (min 280px, double-click 50/50) → **WindowHost** → **StatusBar**
  (24px) → drawer host → **CommandPalette** (Ctrl+.) [B12, P4K-E037..E045]. The
  five-icon sidebar and Tier-0 shell are retired [CF-014].
- **Windows (nine today)**: CommandWindow, ActivityWindow (orbit: PLAN / MAP /
  RESEARCH constellations), PlanWindow (Concept→Phase→Step→BabyStep, eight
  BabyStep blocks), MonitorWindow (5s poll: FSM, supervision, spend, auto-advance,
  inline interventions), ExploreWindow (Ctrl+8 Q&A), ReviewWindow (per-hunk
  Accept/Decline/Retry; Ctrl+Shift+A/R only when review is primary),
  SettingsWindow (Cloud, Runtime, tool-gate, index, canary), IdeaFramingWindow,
  AuditTimelineWindow (auditor modes + cascade). **Candidates**: GraphWindow
  (Ctrl+G; A/B/C placement undecided [B33-E04]), UsageWindow (J-29), CockpitWindow
  or extended ReviewWindow (G-5), CanvasWindow (H-10, intended to replace the orbit
  at H-16).
- **Drawers (~60, consolidation deferred [B12])**, grouped: knowledge/retrieval
  (KnowledgeDrawer, staleness quarantine, artifacts), reasoning (ReasoningDrawer:
  budgets, grounding sweep, feasibility, MiniCritic, symbolic conflicts, Darwinian
  admits), orchestration (agents, evidence gates, ambiguity, stagnation, sandbox),
  memory (MemoryFactsDrawer: facts + trust + contradictions + pin/forget + history
  toggle [F-12, H-33]), sessions (BranchTree fork/merge/close, RecoveryModal),
  trust/security (VibeGateDrawer, SecurityRulesPanel, TaintFlowDrawer,
  BlastRadiusDrawer, McpRugPullDrawer, ConfidenceDrawer, AttestationDrawer,
  RedactionPanel, ModelDriftPanel, DepsSecurityPanel, SecurityMonotonicityDrawer,
  IncidentsDrawer, SafetyDrawer), observability (TranscriptDrawer, SpendDrawer,
  QueryDrawer read-only SQL, failure-modes MAST), research (ResearchDrawer,
  CosmosDrawer, VisionRAG).

### 4.6 Global chrome, shortcuts, recovery

Mostly shipped and still current. Under section 4.1 this chrome is app-global, and
"window" here means the panel that hosts the view.

- **Cross-window chrome (E-33, SHIPPED)**: **ActivityRibbon** at the WindowHost top
  edge (FSM substate, active role, last tool, token count, last-diff hint — persists
  across all windows; top placement is the authoritative decision),
  **WindowFooterMode** (static FSM/agent/cap footer on every window),
  **SnackBarPermission** above the input (Approve / Reject / Always-allow, deduped),
  **SkeletonList** (no spinners), **ConfidenceTag** (needs-evidence / low / ok /
  verified — shape+text+color, numbers only in tooltip), Wong signal palette,
  reduced-motion preference (`accessibility.motion`, stops orbit and particles).
- **StatusBar**: FSM badge + EXECUTING pulse, context-budget gauge (amber 70% / red
  85%), handoff threshold, gate-mode badge always visible, active-profile badge
  (I-42) [B2, patch-4G.1]. Run-specific fields follow the selected run. With no
  selection, they show an explicit no-selection state instead of a false aggregate
  [section 1.1].
- **Responsive**: <1000px hamburger sidebar; <800px secondary panel collapses
  [SUM-0196]. Both thresholds describe the retired panel pair, so narrow and mobile
  layout is open (decision #17). **Shortcuts**: Ctrl+1..7 windows, Ctrl+8 Explore,
  Ctrl+N/W/D, Ctrl+. palette, Ctrl+, settings, Ctrl+G graph, Ctrl+Shift+A/R review
  bulk, Ctrl+Shift+H window recovery (C-7 RecoveryModal; note B-26 earlier removed a
  misleading Ctrl+Shift+H claim — C is newer and wins), Esc.
- **Recovery inventory**: the shipped closed-view stack and RecoveryModal restore a
  view by type [C-7]. They do not restore session tabs. Named layouts are the target
  panel-model recovery surface (section 4.1).

## 5. The canvas arc (the one big migration)

**Framing (owner ruling 2026-08-16): canvas is detail, not the overview.** Canvas is
the dependency and steering view for one execution run or one work unit, reached from
a `Work` row, from the `Canvas` view, or from a message. The cross-run overview is the
`Work` table (section 4.4). Semantic zoom follows the hierarchy: run and work-unit
nodes first, then TAB descends into one agent attempt, deixis-only and never
auto-expanded [J-60 §Steal/build 1, 3, 6]. Three constraints hold: node state is a
journal-derived fold and the model never sets it [J-31 §Steal/build 2-3; J-60
§Steal/build 2; TD-3], steering is the closed J-61 verb set, and the canvas executes
nothing [J-31 §Steal/build 7; J-60 §Steal/build 5; J-61 §Steal/build 5; O-4].

Today ActivityWindow is a three.js **orbit** (PLAN constellation, MAP rings,
RESEARCH phase bars; captions derived without an LLM; V1 text feed retired)
[E-B13]. Wave-2 replaces it with a 2D **Canvas** workbench:

- H-10 canvas shell (code topology, PageRank, dead-code islands) → H-11 live agent
  overlay + VS Code-style FileExplorer column (project-root scoped) → H-12 PLAN and
  RESEARCH become flat frames with a **CanvasModeBar**; mode is a plain doc flag
  {code, plan, research, arch, design}; exactly one window owns the SSB
  subscription → H-13 the model drives the canvas through closed ops
  (CanvasOpApplied, ten kinds) → H-14 DESIGN page nodes (real app pages in a
  controlled iframe) → H-15 model-spawned interactive views with taint-at-handoff →
  H-16 delete the orbit and the three.js deps entirely (only after Canvas-as-default
  ships clean).
- **Altitudes** extend the same canvas: SYSTEM (J-31: intent-side nodes bound to
  plan units/contracts/MCP pins; lifecycle DRAFT→FRAMED→BUILDING→WIRED→VERIFIED→
  DRIFTED; zero-evidence nodes stay grey), PROCESS (J-60: read-only FSM/agent view,
  TAB descends into a worker), MEMORY/CASE (J-98: proof-born case nodes only),
  WORKSPACE (K-23: repos as nodes, Sound vs Heuristic edges), WORKFLOW (O-4:
  TaskSpec editor+monitor, "n8n with receipts", run-now enqueues through the normal
  FSM — the canvas never executes anything).
- **Steering** is a closed verb set (J-61): CONNECT, CUT-EDGE, REORDER, REBIND,
  AGENT-CONTROL(pause|redirect|kill). No node creation on canvas; new nodes come
  from chat. Every gesture is a journaled redirect intent to the single-writer
  engine.

Design consequence: ACK cards, usage badges, route explanations, browse panes,
device frames, fleet ports and drift rows all deep-link into canvas modes, so the
canvas is the shared "open it live" destination for one run or one work unit. `Work`
stays the cross-run overview, and `Canvas` is one of its views (section 4.4).

## 6. Core flows

- **F1 Submit a request**: target an orchestration session, keep `Start now` or pick
  `Plan first`, type or `/`, Send → front door (section 3) → a **new execution run**,
  unless the message is an anchored reply or names a run target → stream cards
  (Status / Intervention / Error / Disambiguation / Steering) → the run's next FSM
  state. `Start now` can reach EXECUTING. `Plan first` stays in PLANNING until H-37
  approval. Stop invokes that run's one cancellation path [B14]. Routine starts and
  progress fold into the session digest; the run row in `Work` carries the full trail.
- **F2 Watch it run**: `Work` is the watch surface: Table for everything, Attention
  for what needs a human, Board and Timeline for shape, Canvas for one run's
  dependencies (section 4.4). Live per-run detail is the ActivityWindow orbit (today)
  or the Canvas (target): active-node halo, budget ring, evidence-gate pips,
  BubbleUpStrip captions, shooting-star events; PROGRESS digests in chat stay compact
  [E-B13, J32-E019]. Healthy agent attempts stay collapsed into counts.
- **F3 Intervene / approve**: producers raise `intervention_request` → Monitor/orbit
  card + SnackBarPermission: Approve / Reject / Modify / Redirect / Take over →
  `submit_intervention_response` → optional webhook + browser notification
  [B005-F01/F04]. Plan-level: H-37 PlanApprovalCard before TESTING_FIRST. Spiral
  escalation card after 3 repeats (retry-hint / substitute / skip / abort)
  [P3C-E056]. Persona-exhaustion InterventionCard [P4G-E028]. Every one of these also
  raises the run into **Attention**, so nothing depends on the user watching a stream.
  Steering is: select the work, then address ELAI. Pause and Cancel stay reachable
  from the row's contextual menu and the selected-run conversation controls. The
  older E-4 permanent `/pause` `/redirect` `/cancel` bar is superseded by this
  minimal-control rule.
- **F4 Review every result (owner ruling 2026-08-17)**: completion emits a terminal
  result row. Until a later human-verdict row exists, `Work` derives **Review
  required** and raises the run in Attention. The result stays in its isolated run
  scope or worktree. Opening it uses the ReviewWindow grammar: per-hunk
  Accept/Decline/Retry(+note), empty and SkeletonList states, with bulk keys guarded
  to the active review surface [C-7]. Accept, Decline, and Retry submit commands to
  the single-writer engine; they never mutate `Work` directly. **Accept** is one
  command that records the verdict and starts deterministic merge-back. Non-overlap
  applies in journal order. An overlap or stale-target conflict emits a conflict row,
  halts the merge, and leaves the accepted target unchanged [I-2]. Resolution follows
  the effective GateMode recorded at the conflict:
  - **Interactive**: raise a Needs user Attention item and open the exact three-way
    overlap for manual resolution. The user's resolved candidate is a new isolated
    attempt under the same work unit and returns through normal gates and review.
  - **Autonomous**: run a bounded sequential loop of isolated resolution attempts
    under the same work unit. Each attempt receives the accepted base, the isolated
    candidate, and the conflict artifact. No attempt can write the accepted target.
    A failed candidate can feed the next attempt through the existing retry policy. The first candidate that
    passes its normal gates returns as Review required, and the user must Accept it
    before another merge-back attempt. Equivalent-conflict recurrence or the existing
    anti-spiral limit stops the loop and raises Needs user.
  There is no parallel resolution race. No mode can accept its own result.
  **Decline** keeps
  the original result out of the accepted working state. **Retry** records the verdict
  and creates a new attempt under the same work unit. Accept does not commit, push,
  deploy, publish, or bypass any later approval. Deep trust view: G-5 cockpit with
  vouched/unvouched bands, per-hunk badges, DSR headline, H-52 anchored findings, and
  per-requirement chips Satisfied/Unverified/Regressed (I-37). External: K-3 check-run
  + anchored PR comments; K-11 `## Evidence` section. A hunk with no evidence renders
  **UNVOUCHED, never silent clean**.
- **F5 Explore / ask**: ExploreWindow Q&A with file refs (backend arriving with
  Oracle/B-26); `/research`, `/ask`, oracle chip; `/reason --profile research`
  (checker-verified steps in ReasoningDrawer, L-9) and `--profile explore`
  (survivor map: surviving explanations + refuted-with-falsifier + evidence gaps,
  L-23/L-24); E-48 research orchestrator with tiered search, contradiction
  surfacing (never silent pruning) and dead-citation drops.
- **F6 Plan drill**: PlanWindow tree ↔ orbit/canvas PLAN mode; cursor highlight;
  DAG repair log; executable pipeline; C-29 intent panel (add NL/assertion pairs,
  Confirm enabled at ≥3); GraphWindow impact overlay with Mermaid/PNG export
  [B7, B33].
- **F7 Memory**: MemoryFactsDrawer = facts + trust score + validity badges
  (Valid / Stale-excluded / TTL-expiring) + semantic-contradiction adjudication
  (SurfaceToUser default, never auto-close) + include-superseded toggle + per-row
  pin/forget [E-5, F-12, H-31..33]. CLI `elai memory list|inspect|forget|pin|
  provenance`. `/continue` lists the decisions that will influence resumption
  before acting. Cross-session recall conflicts require **user adjudication** (E-1).
- **F8 Cost and usage**: live per-turn gauge in chat + CLI (1s poll, "in N / out M"
  for local lanes, never fake $0.00) → UsageWindow rollups per model/task/session/
  lane + cache effectiveness (J-28/J-29) → CloudPanel (mode selector local/hybrid/
  cloud, daily/monthly caps, warn 80% / block 100%, air-gap disables cloud) +
  TokenTrackerPanel once H-9 lights the writers (today it shows honest zeros at
  best) → layered caps with typed-confirmation override (E-39) → K-24 credit
  billing (balance, holds, out-of-credits refusal) when enabled. Cost estimate
  surfaces before the first cloud call [C-5].
- **F9 Orchestration sessions, runs, and time**: `Sessions` holds the durable goal
  containers; `Work` holds the runs beneath them (sections 4.2, 4.4). BranchTree
  fork/merge/close + `/session` verbs + `/continue` handoff picker become scope and
  resumption tools, not workspace switches [E-B9, C-23]; H-35 "Rewind to here" per
  turn and H-49 time travel (undo/redo/history/jump, file-manifest diffs, Files-Only
  vs Files+Conversation) act on one run. The context gauge drives the compaction
  ladder per run, and 85% budget auto-handoff is imminent-visible, because each run
  owns its own context and budget [E-23, E-25, H-50, section 1.1].
- **F10 Trust visibility**: every blocked write or call lands in a drawer with the
  rule, path and citation (VibeGate, taint, blast radius, MCP rug-pull with
  before/after diff and no auto re-pin, deps slopsquat, incidents with Allow/Deny/
  RequireHumanApproval trust dialogs) [F, G, E-30]. RRA viewer (seed, pins, gate
  hashes, replay verdict Verified/Divergence/Tampered) + `elai replay --verify`;
  AttestationDrawer + offline `elai verify-attestation`; SafeDelta and QDELTA
  benchmark tiles; ModelDriftPanel. N-7 makes a human render target **mandatory**
  for every verdict class: PR comment | chat ACK | canvas badge | brief row | org
  rollup.
- **F11 Settings and runtime**: SettingsWindow hosts CloudPanel, RuntimePanel
  (per-role backend, fallback chain, g/y/r health, Test button, BYOK, transport,
  free-pool badge "non-reproducible"), tool-gate inspector, index schema/migration,
  canary drift, webhook URL, uxPrefs (ribbon toggle, motion) [C-4/C-35, B-4, E-33].
  CLI `/runtime`, `/mode autonomous|interactive` (applies at next gate), `/model`.
- **F12 One-shot build (L-14)**: `elai build "<idea>"` or the one-line ELAI entry
  (section 2 item 3) → pipeline + E-12 production templates → L-11 quality gate (fail =
  findings, not DONE) → **always** an RRA receipt via I-40 wrap → output = app +
  one-line receipt "verified: <receipt> (elai rra verify)" — present, not an audit
  wall. Also exposed as a J-2 MCP verb for Claude Code/Codex users.
- **F13 Research machine (O)**: user edits typed `[[research.topics]]` (steal-scan,
  leaderboard, discovery, monitor) → hourly Cosmos tick → result-bearing runs rise in
  **Attention** and retain the intervention-card grammar. Empty TaskSpec results
  auto-close their transient card and leave the journal row; they never transition
  the orchestration session to Archived. Digests upsert the external **Markdown brain vault** (Obsidian/GDrive is the primary
  reading surface; namespace-claims guard writes) → canvas WORKFLOW mode edits/monitors
  TaskSpecs with run-now. If the owning orchestration session is Parked, each trigger
  cause is journaled but withheld. Reactivation coalesces the sealed missed range into
  one Needs user item per schedule; it never replays every missed tick [O-2, K-14].
- **F14 Personal software fleet (P)**: `elai fleet register|list|tick` health
  ticks → a red tick raises one plain-language **Attention** item (which app, what
  broke, one sentence) with a gated propose-fix; ceremony profiles
  (`elai profile set toy`) shrink gate tax for toy tools; `elai app export|import`
  hands verified apps to friends (import refuses a failed receipt); privacy grants
  per tool×namespace; `elai workflow from-goal` proposes an inert TaskSpec graph the
  human confirms; `elai browse` co-browse pane is dumb glass with a journaled input
  trail; device frames for e2e; `elai doctor` stays CLI-only.
- **F15 Commercial (M)**: activate → status (tiers.toml names) → ExpiredGrace
  banner → DegradedToFree; org policy drift boot warning; fleet report with
  `--verify`; signed update apply/rollback; support bundle with pre-share content
  print; `elai-verify` + EEF conformance for third parties.

## 7. Notification model

**One queue: Attention (owner rulings 2026-08-16 and 2026-08-17).** Needs-input,
failed, stalled, conflicting, expensive, and completed-but-unreviewed runs rise into
Attention, and the global Attention count cannot be hidden. Every result requires
review. Healthy in-progress attempts stay collapsed as counts in `Work`. Routine
starts and progress fold into the per-session digest, so notification volume does
not scale with agent count. Attention is itself a journal fold, so it re-derives and
never holds state.

**Deterministic Attention order (owner ruling 2026-08-17).** An item takes the
highest-priority class that matches it:

1. **Safety**: a safety-critical block, unsafe-side-effect risk, or security decision.
2. **Needs user**: any closed event that cannot proceed without a human, including
   clarification, permission or approval, Review required, interactive conflict
   adjudication, exhausted autonomous resolution, a withheld-wake batch, and blocked
   decisions.
3. **Blocked-work impact**: items with no current human decision whose blockage
   holds back other work.
4. **Active cost**: work that continues to consume measured money or scarce compute.
5. **Age**: every other Attention item, oldest first.

Within a class, sort by blocked descendant count descending, measured active cost
rate descending, first-Attention journal sequence ascending, then stable item id.
An item matching several classes uses the highest one. Missing fields sort last and
never promote an item. The model never ranks Attention; closed event kinds and
journal-derived numbers compute the tuple.

Below that queue, the existing ladder is layered, mostly quiet, always journal-backed:

1. **Attention cards**: intervention, plan approval, spiral/persona escalation,
   goal violation (Q-1), research, and fleet entries reuse one card grammar inside
   `Work`. Only result-bearing entries surface. Empty transient cards auto-close and
   leave their journal rows. This is card cleanup, never orchestration-session Archive.
2. **SnackBarPermission** above the input for permission events (E-33).
3. **Toasts**: completion + auditing live today; needs-input toast is a named gap
   to activate (the current handler is a no-op) [H-36]. Once activated it is the OS
   mirror of an Attention item, not a second queue. Re-anchor toast (E-4),
   intent-pair errors (C-29).
4. **Banners**: ExpiredGrace (M-1), index lock/resume (B-31), quarantine (J-102),
   consent banner for browser notifications, boot warnings (M-2 drift).
5. **OS layer**: Browser Notification API + webhook fire-and-forget [B005-F04];
   native notify-send/osascript/PowerShell behind `ELAI_NOTIFY_ENABLE`; terminal
   OSC9 unconditional [H-36].
6. **Voice narration** (J-112): completion, needs-input, step progress, on-demand
   digests; deterministic phrasing; text canonical.
7. **Status chrome**: StatusBar badges, ActivityRibbon, WindowFooterMode, canvas
   halos and badges — ambient state, not interruptions.

## 8. Standing visual rules (already decided upstream)

- Tokens (B-11): teal `#1B7A6E`-family on charcoal `#0f1318`; JetBrains Mono
  (+ Inter for display titles only); Lucide 16px (20px sidebar); 1px borders, teal
  only when active/focused; 6px panel / 3px control corners; no shadows; 8px
  spacing base; type scale 10/12/13/15/18 only; panel switch 80ms out / 120ms in;
  3s breathing progress dot; no bounce/shake.
- Signal colors are the Wong palette; brand teal/purple is chrome-only (E-33).
- Outlined buttons only; approve = outlined teal, destructive/neutral variants stay
  outlined [B14, E33-004].
- Bans (E33-022): spinners (use SkeletonList), visible numeric percentages
  (ConfidenceTag shapes instead), hue-only signals, unpausable motion >5s,
  Tauri unwraps.
- Svelte 5 runes only, no legacy `$:` (INV-F01). Window state local-only (INV-F03).
  Closed enums for every stage vocabulary the UI renders (INV-F04). Human-visible
  SSB needs a frontend pair or a one-line W-6 N/A (INV-F05).
- Honest empties everywhere: explicit "observability gate off" / "no rate for
  <model>" / grey zero-evidence nodes — never silent empty or fake $0 [J29, J31].
- Layout state stays local-only: named dockable layouts replace the single workspace
  key [INV-F03, SUM-0189 superseded]. The token values above stand. The exact visual
  style of the three panels, their default widths, and `Work` row density are open
  (decision #17).

## 9. Committed vs planned vs candidate

- **Live or asserted-shipped core** (treat as the design baseline of shipped parts;
  the target IA is section 1.1): Workspace shell
  + IconRail + palette + StatusBar [B12/B2], CommandWindow streaming [B-14], orbit
  ActivityWindow [B-13], PlanWindow [B-7], Monitor/Explore/Review/Settings windows,
  MemoryFactsDrawer read path (E-5), E-33 chrome pack (ribbon, footer, skeleton,
  confidence, snack, motion), NotificationDispatcher completion toasts, KG-2
  github_pr plumbing, B-39 `/reason` + ReasoningDrawer.
- **Committed BUILD next** (design them now): J-32 front door cards + chips,
  J-28/J-29 usage + gauge, H-37 approval card, H-36 needs-input toast, H-33 memory
  governance, H-35 rewind, canvas H-10..H-16 + altitudes J-31/J-60/J-98 + O-4
  WORKFLOW + J-61 verbs, G-5 cockpit + K-3/K-11 PR evidence, F/G security drawer
  set, M-1 license flow + banner, L-14 build door, O vault plus Attention cards,
  P-1 fleet Attention cards.
- **Candidates ("ride existing OR N/A", decide at edit)**: the long tile lists in
  digests 05-07 — L-1/L-2 claim tiles, K bench tiles, N measurement/governance
  tiles, Q tiles (mostly explicitly no-UI), P panel identities. Do NOT invent a
  window per feature; pick the host panel per W-4 when each ships.
- **Default-off experiments** (design the empty/off state honestly): K-23 workspace,
  K-24 billing, voice, LoRA gate panel (read-only, no Enable button), darwinian
  drawers, FIM streaming, many `ELAI_*_ENABLE` allowlist flags.
- **Net-new from the 2026-08-16 and 2026-08-17 rulings** (no shipped substrate, so
  design them honestly): the pointer-only supervisor journal, the `Work` projection
  and its saved views, the orchestration-session registry, named dockable layouts,
  lifecycle admission, the withheld-wake batch contract, and Cold Archive lifecycle
  contract in section 4.2. F-13's planned schema-bump database copy is a distinct
  forensic mechanism and does not ground this product lifecycle. Nearest
  existing substrates are the phase-3B SSB single-writer journal, the K-23
  `workspace.ssb.db` global journal, C-17 auto-resume, and O-2's planned TaskSpec fire
  path. O-2 explicitly defers missed-run detection and defines none of the Park/batch
  records. No surface may imply a live guarantee for these legs [INV-C07].

## 10. Superseded traps (do not design these)

- **Session board + tabs and per-session workspaces** (owner ruling 2026-08-12,
  superseded 2026-08-16): no session tab strip, no per-session window set, no
  per-session layout key, no pinned second-session monitor in a panel pair. One
  persistent conversation plus `Work` replaces all of it [section 1.1].
- **Implicit session deletion or Archive backfill.** The lifecycle enum is Active |
  Parked | Archived. No lifecycle state deletes journals, Work rows, conversation
  history, receipts, or schedules. Archived schedules emit no ticks, and Restore does
  not backfill the cold interval. Destructive deletion cannot hide behind Archive.
- **Archive vocabulary collision.** `Archive` and `Archived` refer only to the manual
  orchestration-session lifecycle. Empty result cards may auto-close, but no transient
  card cleanup can change session lifecycle.
- **Park as bulk run control.** Park never pauses or cancels runs. Non-quiescent Park
  requests fail closed and identify the runs that must finish, pause, or cancel first.
- **Archiving unresolved or resumable work.** Archive requires Parked, zero
  nonterminal runs, zero unresolved Attention, and zero undecided wake batches. A
  paused run blocks Archive. Safety and Attention can never be hidden by default
  Archive filtering.
- **Automatic parking, automatic wake release, replay-every-wake, or dropped wakes.**
  Only the user parks a session. Scheduled triggers that arrive while Parked are
  journaled as withheld work, never dispatched and never silently discarded.
  Reactivation restores admission, seals one review batch per schedule, and never
  auto-resumes paused runs. Run once creates exactly one normal isolated run for the
  sealed range. Dismiss creates none. Original rows remain immutable, and later ticks
  use the normal Active path.
- **`Work` as a state store.** It is a read projection. Nothing writes execution state
  through a `Work` row, and State and Phase stay read-only journal folds [INV-B01,
  J-60 §Steal/build 2].
- **Two drivers on one execution run.** One run, one FSM driver, one authoritative
  journal, one cancellation path [section 1.1].
- **Auto-accepting a clean result.** Verification proves that the result satisfies
  its gates. It does not supply the user's acceptance verdict. Every result stays
  isolated in Review required until Accept, Decline, or Retry [owner ruling
  2026-08-17].
- **Heuristic dependency evidence used to authorize parallelism or open a gate.**
  Sound edges only; heuristic stays advisory with a warning badge [K-23].
- **Attention ranking by model judgment.** The queue uses the closed deterministic
  tuple in section 7. An LLM score, prose urgency, or missing metric cannot reorder
  safety or user-action items.
- **A separate Merge button after Accept.** Accept already records the verdict and
  starts deterministic merge-back. A second action repeats the same decision and is
  prohibited by the minimal-control rule [owner ruling 2026-08-17].
- **Partial or conflict-winning merge.** An overlap or stale-target conflict halts
  atomically and leaves the accepted target unchanged. Interactive mode raises Needs
  user. Autonomous mode can run only a bounded sequential loop of isolated resolution
  attempts, never choose a winner or mutate the accepted target. It never resolves by
  arrival order or a model choice [I-2].
- **Autonomous conflict acceptance.** Autonomous mode can try to resolve a conflict,
  but the resolution result still returns as Review required. No mode can accept its
  own resolution.
- **Canvas as the global overview, or canvas as an execution engine** [J-31
  §Steal/build 7; J-60 §Steal/build 5; J-61 §Steal/build 5; O-4].
- **A control per row.** Persistent row buttons are the trap that minimal controls
  exists to prevent; use selection plus ELAI, with contextual menus for the rest
  (section 4.4).
- **A permanent live-run control bar.** E-4's older `/pause` `/redirect` `/cancel`
  bar is superseded. The selected-run ELAI controls and `Work` contextual menu expose
  Pause and Cancel through the run's one cancellation path.
- **A second dashboard inbox or notification queue.** Attention is the queue. Research
  and fleet results reuse its card grammar, and the OS layer only mirrors it. An
  external vault or TaskSpec sink is data routing, not another dashboard queue
  (section 7).
- V1 text ActivityFeed, Jump-to-latest pill, FilterStrip — retired [E-B13-028].
- Five-icon sidebar, Tier-0 shell, `primary_shell` TOML, first-run shell question
  [B12, CF-014]. ChatWindow name → CommandWindow [B0-004].
- SettingsWindow license-status line [M1-C001]. Lattice UI (F-7). GraphWindow as a
  mandatory 8th window (needs explicit user confirmation) [B33].
- "Zero secrets" copy before G-9 is wired; CloudPanel zeros as if measured [H-9].
- B-CREATIVITY explore-conditioning UI — REJECTED-by-measurement, gate ladder
  halted [Q-10]. MoE local-default routing [E-31]. Ollama promoted paths (demoted,
  WARN) [E-26].
- Summary-claims that lost to deep ledgers: e.g. `elai benchmark --suite seccodeplt`
  spelling, ablation auto-pruning of roles (E-10), unconditional cockpit tiles for
  I-5/I-7/I-25/I-30.

## 11. Open design decisions DESIGN.md must answer

1. Graph home: GraphWindow vs orbit/canvas MAP vs KnowledgeDrawer flyout [B33-E04].
2. Diff review container: feed-inline vs modal; single vs multi-file nav [P4K-A003].
3. InterventionCard: standalone component vs card variant; Monitor-inline vs
   extracted [B14-A01, B005-F04-005].
4. RuntimePanel placement: panel vs SettingsWindow section [P4M-A002].
5. Cockpit: new CockpitWindow vs extended ReviewWindow [G-5].
6. Cost surface consolidation: UsageWindow vs CloudPanel vs SpendDrawer vs Monitor
   gauge vs CostPanel — four+ coexisting cost views need one hierarchy [J29-E003].
7. Drawer consolidation strategy for ~60 drawers [B12]; which drawers become
   canvas-badge-first once N-7 lands.
8. Canvas default timing: when workspace primary flips activity→canvas; galaxy
   toggle disposition [H-12, H-16, DELTA-007-006]. Scoped by section 5: canvas is run
   and work-unit detail plus one `Work` view, never the app default surface.
9. Notice-card layout for Autonomous MEDIUM [J32-A003]; notification level enum +
   focus behavior [V-A002].
10. ExpiredGrace banner placement/copy/dismiss [M1-A002].
11. L-14 one-field builder layout + receipt presentation [L14-A002/A003].
12. Memory altitude name: memory vs case vs vcg [J98-A001].
13. Sidebar icon count and the four-question-tab IA remap [B12-023, B-26]. Component
    inventory only: the target shell has three dockable panels and no sidebar
    (section 4.1).
14. MeasurementPanel vs G-5 tile for the spine index [N1-A003].
15. Billing surface: dedicated tile vs CloudPanel ride [K24-A002].
16. Plus the three dormant PLAN.md §5 decisions: 10px font floor, gallery vs
    Storybook, markup rewrite vs re-point.
17. **Unified conversation shell (owner ruling 2026-08-16, supersedes the 2026-08-12
    "session board + tabs" ruling).** The model is DECIDED and recorded in section 1.1:
    one persistent ELAI conversation as the orchestration layer, orchestration sessions
    as small durable goal containers, one new isolated execution run per normal
    request, one FSM driver and one authoritative journal per run, a pointer-only
    single-writer supervisor journal for visible cross-run ordering, the hierarchy
    orchestration session > execution run > stable work unit > agent attempt, and the
    three dockable panels `Sessions`, `ELAI`, `Work` (sections 4.1 to 4.4).
    **DECIDED, retained from 2026-08-12: the scope ladder**, now read as run scope
    resolution: rootless (research/fleet/browse) | a worktree of a project (the
    parallel story, isolated via I-2 COW worktrees / C-23 E-B9 session branches /
    J-60 worktree diffs) | a whole project root | a multi-root K-23 workspace. Old
    ProjectSelector fields become the scope step of session creation.
    **DECIDED 2026-08-17: every execution result requires review before acceptance.**
    A clean verified result stays isolated as Review required. Accept records the
    verdict and starts deterministic merge-back as one action. A clean merge updates
    the accepted working state. An overlap or stale-target conflict halts atomically
    and leaves the accepted target unchanged. Interactive mode asks the user to
    resolve the exact overlap manually. Autonomous mode runs a bounded sequential
    loop of isolated resolution attempts. The first gated candidate returns as Review
    required. Equivalent-conflict recurrence or the anti-spiral limit raises Needs
    user. No mode can accept its own resolution. Decline keeps the result out; Retry
    creates a new attempt under the same work unit. Accept never implies commit,
    push, deploy, publish, or another irreversible external action.
    **DECIDED 2026-08-17: conflict resolution follows GateMode.** Interactive is
    manual. Autonomous runs the bounded resolution loop above. Both preserve the
    accepted target until a reviewed candidate is accepted.
    **DECIDED 2026-08-17: deterministic Attention ordering.** Safety ranks first.
    Any closed event that needs the user ranks second. Blocked-work impact ranks
    third, active measured cost fourth, and age fifth. Within a class, use blocked
    descendant count, active cost rate, first-Attention journal sequence, then stable
    item id. Missing fields sort last. The model never ranks the queue (section 7).
    **DECIDED 2026-08-17: session lifecycle has three durable states.** The closed
    state set is Active | Parked | Archived.
    **DECIDED 2026-08-17: Park is a user-triggered quiescent admission state.** All
    runs must be terminal, paused, or cancelled before Park. Park has no bulk Pause or
    Cancel side effect. It blocks new requests and scheduled dispatch while preserving
    withheld trigger causes in the journal. Conversation history, Work rows, and
    Attention remain visible. Reactivate returns to Active and restores admission
    only; it does not auto-resume paused runs or auto-dispatch withheld wakes.
    **DECIDED 2026-08-17: withheld wakes coalesce and require review.** Reactivation
    seals one immutable batch per schedule per Park interval at a journal watermark,
    covering only previously unbatched trigger rows. The batch shows missed count,
    trigger-time range, and covered row range as a Needs user Attention item. Run
    once executes the TaskSpec once against current state in one normal isolated run,
    with the batch receipt as trigger provenance. Dismiss creates none. Both decisions
    are idempotent and preserve the original trigger rows. Newer ticks follow the
    normal Active path. Autonomous mode cannot decide a batch. Replay-every-wake and
    silent discard are prohibited. This is a net-new O-2/session-registry extension:
    the production scheduler admission check, Park interval id, withheld row, atomic
    seal watermark, immutable batch receipt, and unique decision record are required
    as specified in section 4.2. O-2 currently defers missed-run detection, so this
    leg stays wire-dark until implemented. Automatic parking is prohibited.
    **DECIDED 2026-08-17: Archive is manual reversible cold storage.** Archive is
    allowed only from Parked with zero nonterminal runs, unresolved Attention, or
    undecided wake batches. Archived sessions leave default Sessions, composer, and
    Work surfaces, but remain searchable and read-only. Prior ELAI messages remain in
    immutable scrollback. Safety or Attention overrides hiding. Schedules emit no
    trigger rows while Archived. Restore returns to Parked, restores visibility only,
    and never resumes or backfills work. A later Active transition starts schedules
    at the first due point strictly after that transition. Archive and Restore are
    never automatic, and Archive never deletes data.
    The planned session-registry lifecycle row, transition guards, archived scheduler
    suspension, default-filter override, and restore-to-Parked contract are defined in
    section 4.2 and remain wire-dark until implemented.
    **STILL OPEN for DESIGN.md:** (a) exact visual style of the three panels;
    (b) default panel widths; (c) mobile and narrow layout, since the old <1000px and
    <800px rules describe the retired panel pair [SUM-0196]; (d) `Work` row density;
    (e) engineering shape of the net-new pieces listed in section 9.
    **HISTORY, SUPERSEDED:** the 2026-08-12 window decisions (session board + tabs,
    per-session workspaces and layout key, pinned second-session monitor, command-center
    home with a live session board) are superseded. Full note in section 1.1.

## 12. Design implications (the primitives)

The hundreds of proposed surfaces collapse into ~10 primitives the design system
must nail once:

1. **Card** — ACK/PROGRESS/RESULT, intervention, approval, notice, Attention, sprint
   contract, clarifier. One card grammar with lane color, receipts row, action row.
2. **Chip/badge** — intent chip, confidence tag, status/profile badges, taint and
   provenance badges, verdict chips. Shape+text+color, never hue-only.
3. **Tile/gauge** — cockpit tiles, spend gauges, context gauge, governance gauges.
   Journal-recomputable numbers, honest empty state, warn/block bands.
4. **Drawer** — list+detail with per-row actions (pin/forget/ratify/promote),
   evidence columns, kill-switch-off state.
5. **Dockable panel**: `Sessions`, `ELAI`, `Work`, plus detached panels; ribbon and
   footer; skeleton loading; named local layouts (section 4.1). The nine-window host
   list stays component inventory (section 4.5).
6. **Canvas node/edge** — lifecycle-colored nodes, evidence-colored edges,
   disconnected = visible, closed-verb steering.
7. **Banner/toast/snack** — the section 7 ladder.
8. **Receipt line** — the one-line "verified: … (elai rra verify)" pattern reused
   by build, PR, export, update.
9. **Database row and saved view**: the `Work` table. Journal-fold properties,
   read-only State and Phase, grouping, filters, sorting, columns, density, opening
   style, collapsed healthy counts, contextual menus instead of row buttons, and the
   built-in All work / Attention / Board / Timeline / Canvas views (section 4.4).
10. **Digest message**: one updating message per orchestration session that folds
    routine starts and progress, while questions, failures, and results stay
    individual messages (section 3).

Every flow in section 6 is a composition of these ten. The DESIGN.md should
specify tokens + these primitives + the per-panel compositions, in that order
(section 4.5 keeps the older per-window compositions as inventory).

## 13. Sources

- Master list: `docs/thefinalglance/ELAI-EXHAUSTIVE-SUPER-FEATURE-LIST.md`
  (HEAD legend + SYNERGY + surfaces section L52366-60929).
- Digests: `workflow/design/dashboard-design-system/surface-digests/01..07-*.md`.
- Companion plan (tokens/gates/AI harness): `workflow/design/dashboard-design-system/PLAN.md`
  + `design-lint-baseline.json` (335 findings).
- The design decisions dated 2026-08-16 are preserved in section 1.1. Earlier
  decisions dated 2026-08-12 remain as superseded design history. Private handoff
  metadata was omitted.

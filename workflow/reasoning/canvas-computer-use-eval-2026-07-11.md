# canvas / computer-use / e2e-testing eval — ELAI vs 6 external repos — 2026-07-11

mode: spec-eval (AS-SPECIFIED — design judged as if built; shipped-status quarantined to readiness notes)
question: is ELAI's canvas + testing surface (e2e on a website/game/user-made workflow node) better than
agent-zero / openclaw / omniparser / open-computer-use / trycua-cua / simular agent-s, and what do we steal?
protocol: read one thing -> write one entry -> repeat. entries below are append-only.

---

## ENTRY 1 — ELAI's own surface, as specified (source: ELAI-SPEC-OVERVIEW.md, read in full)

What ELAI actually specifies on the axis in question:

a) CANVAS (H-10..H-16): a 2D generative canvas the MODEL drives via a typed op stream
   (createNode/connect/updateNode/annotate, CanvasDispatcher with mode-namespace gating +
   sanitizeAction that REJECTS dangling-id ops), every op journaled as an SSB row
   (CanvasOpApplied), taint-gated deixis on click-to-hand-back (H-15: a web-sourced node's
   content is flagged at the handoff). Modes: CODE-GRAPH / PLAN / RESEARCH / ARCHITECTURE /
   design-mode with click-to-edit. The stated moat: the op-stream is journaled + reproducible +
   taint-tagged — i.e. the canvas is not a drawing, it is an auditable event log with a render.

b) BROWSER E2E (E-14, H-SUMMARY line 138): AXTree-FIRST Playwright snapshot-to-disk +
   axe-core accessibility gate + constrained-decoding VLM as Tier-2 ADVISORY only + hybrid
   SSIM-first visual diff + asymmetric veto + multi-context SaaS-journey runs +
   Cloudflare/Turnstile detect-and-skip-never-bypass. Same doctrine as the harness-side
   visual-ui-loop: deterministic diff decides WHERE to look BEFORE any vision call; counts come
   from AXTree/DOM never a VLM (J perception gateway doctrine).

c) NON-CODE / GAME-ADJACENT VALIDATION (4I media validator): render-capture-evaluate loop for
   images/3D/audio/UI, deterministic Tier-1 checks with VETO power, VLM Tier-2 advisory only.

d) WORKFLOW NODES — the user's open question "do we even have this feature?" — YES, in spec,
   three layers: O-2 scheduled TaskSpec substrate (typed cron/event -> inbox, "n8n with
   receipts"), O-4 canvas WORKFLOW mode (TaskSpec rows projected as nodes+edges, closed-enum
   node palette, gaps rendered VISIBLY DISCONNECTED = wire-dark made visual, NO separate canvas
   runtime — dispatch through the normal FSM), P-9 connector/stage-PACK format (declarative
   ConnectorSpec, ~80% of connectors are a data-diff not a build), P-10 ingress adapters +
   credential vault, P-11 `elai workflow from-goal` (intent -> closed-enum-constrained TaskSpec
   graph on canvas, human-confirmed before ANYTHING runs, RRA receipt binding intent->graph).
   The deliberate anti-n8n stance (O A3): NO open/freeform user nodes — that forfeits receipts.

DESIGN READ: ELAI's differentiator on this axis is not perception or breadth — it is that every
action (canvas op, browser step, workflow trigger) is a typed, journaled, replayable, gated
event. None of the six comparators even attempt that property (to be confirmed per-repo below).

READINESS NOTE (index-derived, confirm against live code; NOT part of the verdict): canvas
H-10..13 partially shipped (J-60 monitor exists); E-14 browser E2E PLANNED; O/P workflow layer
PLANNED, executes post-proof-spine per PSR-1/GATE M-0.

---

## ENTRY 2 — the "exploration vs execution mode" idea (source: user's screenshot, X post evayzh/2075374512401711203 — post itself not fetched, content taken from the screenshot text)

The screenshot's core claim: the industry split for browser automation is
(1) EXPLORATION mode — agentic, LLM reads AXTree, figures the flow out, expensive; and
(2) EXECUTION mode — after one successful exploration, EMIT A RAW PLAYWRIGHT SCRIPT of the exact
steps and replay it forever at zero reasoning cost + ~100% reliability. Browser-Use does the
hybrid (local model parses DOM, Playwright executes natively).

Mapping onto ELAI's spec: ELAI already has the GENERAL form of this — the compiled-pipeline +
template crystallizer (C-plan: "cache a recurring pipeline shape") and the verified case graph
(J: mint a typed case node only on a PROVEN fix, reuse behind an execution-grounded filter,
default-off) and O-2 TaskSpecs (a crystallized flow IS a TaskSpec). What ELAI does NOT yet
specify is the SPECIFIC browser-lane instance: "agentic browser exploration run -> emit a
deterministic Playwright script AS a TaskSpec stage/artifact -> replay it as the execution mode,
with the RRA receipt binding script to the exploration run that minted it." That is a genuinely
cheap, genuinely on-thesis steal (it is MORE deterministic than what we have specced for E-14,
which is agentic-per-run with deterministic gates around it). VERDICT: strongest single design
idea from the user's input; candidate new plan unit (extension of E-14 + O-2, lands in open
letter per CLOSED-FAMILIES rule).

---

## ENTRY 3 — "workflows that install programs (blender, ps) based on user needs" (user idea)

As specified, ELAI's closed-enum node vocabulary (O A3 / P-11 ABORT-1) FORBIDS the model
inventing arbitrary nodes — the honest-gap render + `elai connector new` (P-9 LANE A) is the
sanctioned path: an unavailable capability renders disconnected and the CONNECTOR (not the
workflow run) is authored through the gated pack lane. "Download and drive Blender/Photoshop"
would be: a P-9 code-pack connector wrapping the app's CLI/API + (for GUI-only apps) a
computer-use stage. Agent-zero-style "apt-get anything inside my container" is the OPPOSITE
design pole (maximally open, zero receipts). ELAI can match the CAPABILITY without matching the
openness IF connector authoring is cheap enough — which is exactly what P-9's "80% of connectors
are a data-diff" bets on. The steal question per-repo becomes: what makes their self-extension
FEEL instant, and can a gated pack lane reach that feel. (Ungrounded conjecture flag: whether
P-9's 80% claim holds for desktop apps like Blender — GUI apps need the computer-use stage,
which is the weakest specced part of ELAI today; see comparison entries.)

---

## ENTRY 4 — TEIKOKU asset studio plan + workflow brainstorm (sources: workflow/reasoning/TEIKOKU-ASSET-STUDIO-PLAN.md read to L120, TEIKOKU-ASSET-STUDIO-WORKFLOW-BRAINSTORM.md read to L90)

The other project already converged, independently, on the SAME architecture ELAI's O/P layer
specifies: durable control plane; providers/local tools as replaceable execution backends behind
a one-file adapter contract (capabilities/submit/poll/fetch/cost/license_tag/health); every op a
job row with manifest + cost + license; lineage graph; human review gates as durable decisions;
staged golden path with resumability. Two principles in the brainstorm are directly portable to
ELAI's connector doctrine:

a) "Blender and Unity are downstream EXECUTION STAGES, not manual file-transfer gaps" — driven
   headless (blender --background --python recipe.py, Unity via MCP), NEVER by GUI computer-use.
   This settles the user's "workflows download blender/ps on demand" idea the right way: the
   sanctioned form is a P-9 code-pack connector wrapping the app's headless/CLI/API surface
   (deterministic, journalable, replayable), and computer-use GUI driving is the LAST resort,
   not the mechanism.

b) "provider websites without a supported API are isolated OPERATOR LANES, not load-bearing
   infrastructure" (AssetHub had no public API -> import lane only). Ported to ELAI: a
   computer-use stage that drives a GUI is an operator lane — allowed, taint-tagged, journaled,
   but never a dependency a workflow's correctness rests on. This is the honest version of what
   agent-zero/openclaw-class systems do implicitly and unsafely.

Also relevant: the brainstorm's Temporal citation (durable human-in-the-loop waits) matches
P-11's confirm-before-run gate; and its capybara-game lesson ("the largest multiplier was
game-specific EDITOR TOOLS") supports P-1's app-fleet thesis (bespoke tools per user need).

---

## ENTRY 5 — the strategic frame for the whole comparison (source: I-SUMMARY_INDEX.md I-40)

I-40 specifies `elai wrap -- <agent cmd>`: a proxy/MCP wrapper that runs an EXTERNAL agent's
tool traffic through ELAI's taint tagging + write-gates + RRA emission, plus the .rra format as
an open spec with a conformance checker. This inverts the comparison the user asked for: ELAI
does not need to beat agent-zero/cua/agent-s at computer-use BREADTH — as designed it can
CONTAIN them. Any comparator that wins on raw capability (cua's sandboxed VMs, agent-s's GUI
skill) is, per I-40, a candidate WRAPPED BACKEND / operator lane rather than a rival: their
actions become taint-tagged journaled rows under ELAI's gates. So each per-repo verdict below
answers three separate questions: (1) does its DESIGN beat ELAI's on this axis, (2) what do we
steal, (3) is it a wrap/vendor candidate rather than a competitor. Pending: six repo summaries.

---

## ENTRY 6 — OmniParser (source: docs/.source/omniparser-SUMMARY.md, agent-read, HIGH reliability; arXiv:2408.00203 verified)

Not a competitor at all: a screen-parsing MODEL pipeline (YOLO detect + OCR + Florence-2
captions -> element list with bbox/interactivity/content/source), no planner, no workflow
surface. Consumed via Set-of-Marks: the VLM returns a Box ID integer, the harness owns geometry.
Measured: ScreenSpot 73.0% vs raw GPT-4V 16.2%; captions+IDs lift grounding 70.5->93.8%;
ScreenSpot Pro 39.6 vs GPT-4o-alone 0.8.

(1) Design vs ELAI: orthogonal. It answers the ONE case ELAI's AXTree-first doctrine (E-14,
J perception gateway) cannot: canvas-rendered UIs (games, WebGL, Figma-class), native apps with
dead UIA, streamed screens — exactly the surfaces the user's "test a game" ask needs. When a DOM
/AXTree exists it is dominated by it (its own paper barely beats GPT-4V+HTML), which CONFIRMS
ELAI's doctrine rather than challenging it.
(2) Steals (all cheap, all doctrine-compatible):
  - S1 ID-indirection ported onto AXTREE nodes (VLM picks an integer, never coordinates) — the
    mechanism behind the 93.8%, works without any pixel model. Strongest single steal.
  - S4 source-tagged provenance per element (axtree_exact | ocr_text | vlm_caption | pixel_yolo)
    — provenance IS the trust boundary; slots straight into taint/grounding rules.
  - S2 interactability flag filtering the VLM candidate set; S3 advisory-only captions for
    unnamed canvas icons; S5 asymmetric 80%-containment OCR-into-element label fusion;
    S6 the pinned FastAPI perception-service seam (base64 in -> provenance-tagged elements out).
  - S8 anti-pattern: never let context-free per-crop captions stand in for element identity.
(3) Wrap/vendor: YES — as the Tier-3 pixel FALLBACK leg of the perception gateway, fired only
when AXTree is empty/provably incomplete; captions advisory, counts never from it. License note:
code MIT but icon_detect weights AGPL — matters for productization; Florence caption weights MIT.
Not bit-deterministic (fp16/NMS variance) — pin weights+thresholds, treat captions as the least
stable field.

VERDICT: complement, not comparator. This is the missing limb for game/canvas e2e testing, and
it composes with (not against) the fail-closed doctrine.

---

## ENTRY 7 — agent-zero (source: docs/.source/agent-zero-SUMMARY.md, agent-read, MEDIUM reliability — reviewed source at v2.3 HEAD, 18.4k stars, MIT, company-backed)

The user's guess "agent-zero could be what elai canvas is trying to do" — checked, and NO:
agent-zero has NO node/workflow-graph surface at all. Its "Canvas" is a desktop/document/browser
VIEWER (Xpra/XFCE streamed into the WebUI); composition is agentic (call_subordinate + parallel
fan-out) and FILE-driven (drop tools/<name>.py, hot-loaded with zero registration). Nothing in
the six-repo set so far occupies the O-4/P-11 typed-node-canvas slot.

Design read per the three questions:
(1) Does its design beat ELAI's on this axis? On TRUST, no — and it is the cleanest possible
    foil: execution is agentic-EVERY-time (no record->replay), there is no action journal (only
    debounced shadow-git FILE snapshots), no evidence layer proving the agent's claims, and
    self-extension is maximal ("apt-get install zip" is literally in the tool prompt; a
    plugin_installer gated by an LLM scanner — an in-boundary gate our doctrine rejects). It is
    the maximally-open pole; ELAI's spec is the maximally-receipted pole. On CAPABILITY
    ERGONOMICS it beats ELAI's spec today in three honest places: (a) self-extension feel
    (agent installs what it needs, disposable container absorbs the blast), (b) the
    DOM-annotation ref-FIRST browser (stable refs incl. frame chains, vision opt-in only —
    they independently converged on our AXTree-first doctrine), (c) an extension-hook loop
    (~10 named hook points) that keeps the core loop closed while features attach.
(2) Steals (ranked): shadow-git time travel with debounced snapshots + preview-before-destroy
    (maps to H-49 reversibility; proven git-as-journal); the block-until-verdict pre-tool gate
    shape (_infection_check — fail-closed HOLD on tool exec until an off-path verdict lands;
    same shape as our gates, worth comparing implementations); the skill=SKILL.md+scripts/
    convention ("explore agentically, crystallize the repeatable part into a deterministic
    script") — the exploration->execution bridge EXISTS there only as a manual convention with
    no auto-compiler and no assertions, which is precisely the gap ELAI's journaled TaskSpec
    version would close; the __spaceBrowserPageContent__ ref-capture JS contract as the
    substrate a deterministic web-E2E recorder emits assertions against; the refuse-on-safety-
    check computer_call mapping; the external-CLI adapter-status contract.
(3) Wrap candidate? Yes in principle (`elai wrap` around its tool traffic), but it is a full
    product, not a component — the realistic relationship is steal-from, not embed.

VERDICT: agent-zero is the strongest COMPETITOR of the six on breadth-of-capability and
polish-of-openness, and it validates ELAI's two core bets by ABSENCE: no receipts (no journal,
no replay, no off-agent evidence) and no typed workflow surface. Its "n8n appeal" is zero; its
"do anything on a computer" appeal is high. ELAI's canvas/workflow design is differentiated,
not matched. Maturity line: agent-zero ships today at 18.4k stars; ELAI's equivalents are at
spec/partial — a timing gap, not a design gap.

---

## ENTRY 8 — open-computer-use (source: docs/.source/open-computer-use-SUMMARY.md, agent-read, MEDIUM reliability — reviewed source; 25 stars, MIT, toy-scale)

A weekend-scale open "Operator" clone: KVM Windows 11 VM + OmniParser V2 set-of-marks perception
+ pyautogui OS input + a pure ReAct loop (toolChoice:'required'). No planner, no task graph, no
saved routines, no verification (task_done is the model's unverified self-report), FAILSAFE off,
ungated arbitrary PowerShell. Its "memory" flag is literally wire-dark (written, never read) —
our vocabulary, their bug.

(1) Design vs ELAI: no contest on any axis we care about; it is a live demonstration of every
    failure our doctrine names (in-boundary self-report as done-gate, no off-agent artifact,
    ungated self-extension). Its one genuinely strong side: the EVIDENCE trail — every action
    carries before-screenshot + model reasoning + result + a 20fps cursor-composited per-action
    SCREEN VIDEO. Rich audit-for-humans, zero replay-for-machines.
(2) Steals:
    - Per-action bounded screen-recording attached to the journal row (main.py:95-150) — visual
      proof of what a computer-use action DID; composes perfectly with SSB rows and the RRA
      (a video hash per action is an off-agent artifact a narrated claim can be diffed against).
      Best single steal from this repo.
    - The per-action journal SCHEMA (intent + situation_analysis + before-shot + result) — lets
      a verifier diff stated-intent vs observed-outcome.
    - Fallback-mode tool gating (env unhealthy -> tool surface collapses to wait-only) — a
      structural fail-closed pattern, same family as our gate doctrine.
    - Async long-job model (start/poll/feed-stdin/kill) as the shape for long-running workflow
      nodes (installers, builds).
    - Curated install catalog (tools_config.json) — the SEED of the allowlisted install
      mechanism the user's "download blender on demand" idea needs: named {tool, mirror, alias}
      rows, but gated (the repo doesn't gate; we would).
    - (box_id indirection — already credited to OmniParser in ENTRY 6.)
(3) Wrap candidate: no — nothing to embed; it's a reference of what the operator-lane MVP looks
    like, plus a list of anti-patterns to invert.

VERDICT: not a competitor; a cautionary tale with two good ideas (action-videos-as-evidence,
allowlist-able install catalog).

---

## ENTRY 9 — openclaw (source: docs/.source/openclaw-SUMMARY.md, agent-read, MEDIUM reliability — 22,435-file monorepo at HEAD 2026-07-10, MIT, Clawdbot/Moltbot lineage, multi-org security team)

THE REAL PEER. Anti-sycophancy demands this said plainly: of the six, openclaw is the only one
that independently built large parts of what ELAI specifies, and in places it is AHEAD of our
spec in shipped design detail:
  - Lobster workflow DSL: typed, constrained pipelines run as ONE deterministic tool call,
    "pipelines are data — log, diff, replay, review", approval checkpoints that halt on side
    effects and return a durable RESUME TOKEN (approve+resume never re-runs earlier steps),
    runtime-enforced (not per-script) timeouts/caps/allowlists. This is O-2/O-4's execution
    semantics, shipped — minus the canvas render and the closed-enum synthesis.
  - qa-lab: BOTH modes of the exploration/execution split — agentic scenario packs AND
    `qa jsonl-replay` deterministic replay against mock providers — PLUS a zero-unknown
    confidence-report gate with seeded NEGATIVE-CONTROL canaries proving the gate detects drift
    (their version of our R2 "dead evidence path must scream" + verify-the-verifiers), PLUS a
    visual-task mode where a structured vision verdict must cite evidence and a missing mp4
    recording FAILS the task even if the driver passed (recording as REQUIRED off-agent
    artifact).
  - Trajectory bundles: versioned JSONL (traceSchema/schemaVersion/monotonic seq + manifest)
    emitted identically from runtime/transcript/export "so external tools can replay it" —
    their GD-2.
  - Metadata-only audit ledger: bounded (30d/100k), redaction:"metadata_only", explicit
    `unknown` terminal status never a silent pass.
  - Fail-closed GUI binding: a coordinate action must echo the frameId + display identity of
    the screenshot it was computed from; display reconnect fails closed. "An action must cite
    the exact observation it was computed from" — an invariant worth stealing verbatim.
  - Gated self-extension: ClawHub registry install-on-demand behind security.installPolicy +
    user approval; agent self-authoring exists but only as PENDING PROPOSALS via skill_workshop.
    This is the shipped middle path between agent-zero's ungated apt-get and our P-9 pack lane.

Where ELAI's design still clearly wins (like-for-like, both as designed):
  a) Trust boundary: openclaw EXPLICITLY declares prompt injection without a policy bypass a
     non-vulnerability (SECURITY.md triage line). ELAI's taint propagation + data-plane tool
     filter + Dual-LLM quarantine treat exactly that as in-scope. For adversarial-input safety
     the two designs are not comparable.
  b) Off-agent evidence discipline: openclaw's confidence gates live INSIDE the same product
     boundary (their qa gates are their own code, not frozen against the run); no equivalent of
     the frozen gate-manifest, off-agent artifact doctrine, RRA hash-binding, seeded-entropy
     byte-reproducibility, or SAFEDELTA ablation.
  c) Workflow synthesis: no workflow-from-goal, no closed-enum node vocabulary, no
     visibly-disconnected gap rendering, no canvas at all (composition is DSL/text).
  d) Deterministic replay is transcript-grain, not byte-recomputable (no entropy quarantine).

Steals (high-value, ranked): (1) Lobster's resume-token approval semantics into O-2/P-11
(durable halt-on-side-effect + resume-without-rerun is BETTER specified than our confirm gate —
extend P-11 MOVE 3 with it); (2) confidence-self-test negative-control canaries into our gate
corpus (Q's evidence-corruption corpus is the same idea — theirs is shipped prior art to cite);
(3) frameId/display-identity fail-closed binding into any future computer-use stage;
(4) metadata-only redacted ledger shape for the exported/shareable journal view (G-9 adjacent);
(5) Task Flow's revision-counter optimistic concurrency for canvas workflow state;
(6) bounded-window "arming" of dangerous capabilities (auto-expiring, multi-factor AND) as the
runtime-grain sibling of wiring-enforcement R9.

Wrap candidate: no — it is a full gateway product; relationship is steal-from + benchmark-against.
Maturity line: openclaw ships all of the above today; ELAI's equivalents are spec/partial — a
timing gap, but HERE the gap matters competitively because it occupies adjacent "trustworthy
personal agent" positioning. It is the comparator to track.

---

## ENTRY 10 — trycua/cua (source: docs/.source/cua-SUMMARY.md, agent-read, HIGH reliability — official company SDK, ~19.5k stars, MIT, HEAD 2026-07-10)

Computer-use INFRASTRUCTURE, not an assistant: sandboxes/VMs across 6 provider types (lume
Apple-Si VMs, Docker, Windows Sandbox, QEMU, cloud), a pluggable perception->reason->act agent
loop (model-string regex picks the loop; `grounding+thinking` composes two models), and
cua-bench — a gym-style benchmark with FIRST-CLASS record->replay.

(1) Design vs ELAI: the loop/infra layer is excellent engineering but trust-agnostic — in-loop
    safety gating is literally a commented-out TODO (agent.py:808/839, safety checks
    auto-acknowledged); isolation IS the VM boundary. So on the trust axis, no contest — but on
    the EXECUTION-SEMANTICS axis cua has already shipped the exploration/execution split the
    user's screenshot described: Tracing records step:before/step:after (action repr + full
    state snapshot + screenshot) into a typed HF Dataset; replay_env deterministically re-walks
    the rows and RE-SCORES from a recorded programmatic evaluate_task verdict. Their
    "determinism is a replay property, not a live-run property" is exactly our doctrine's
    distinction, independently converged.
(2) Steals (ranked):
    - Record->replay trajectory format + replay-re-scoring (environment.py + replay_env) — THE
      grounding prior art for our "browser exploration mints a deterministic script/TaskSpec"
      steal; theirs proves the shape works, ours adds the RRA hash-binding + gates.
    - Programmatic per-task verifier (evaluate_task returns a score, re-derivable on replay) —
      maps 1:1 onto our off-agent artifact doctrine, and is the missing "assertion primitive"
      every other repo lacked.
    - dry_run={"before","after"} golden-frame capture — a built-in primitive our visual-ui-loop
      would use directly.
    - Composed grounding with the desc2xy cache (thinking model names the element in NL, a
      small local grounding model resolves coords, resolutions memoized) — the right shape for
      ELAI's cheap-local + expensive-cloud split on GUI surfaces; the cache is deterministic
      memoization of perception.
    - OperatorNormalizerCallback — pure-function deterministic repair of malformed action JSON
      before execution (no LLM round-trip); same philosophy as H-13's sanitizeAction, worth
      diffing against ours.
    - Callback lifecycle bus (~15 typed hooks) as the seam pattern; screenshot-with-crosshair
      action audit (trivial, high legibility); the model->loop regex registry.
    - License flag: their SoM path drags ultralytics AGPL — their a11y-tree path is the clean
      alternative (supports our AXTree-first choice again).
(3) Wrap/vendor: STRONGEST candidate of the six. cua sandboxes (esp. Windows Sandbox + lume +
    Docker) are exactly the disposable execution substrate ELAI's computer-use operator lane
    and 4F feasibility sandbox need; MIT, actively maintained, MCP-exposed. Using cua as a
    VENDORED provider while supplying our own fail-closed in-loop gates (which they stubbed!)
    is a perfect division of labor: they bring VMs, we bring the trust spine.

VERDICT: not a competitor on trust; the best infrastructure partner/steal-source of the six.
Their shipped record->replay + programmatic-verifier stack is independent confirmation that
ELAI's replay/evidence direction is where serious builders converge.

---

## ENTRY 11 — simular Agent-S (source: docs/.source/agent-s-SUMMARY.md, agent-read; arXiv 2410.08164 / 2504.00906 / 2510.02250 verified; 12k stars, Apache-2.0)

The CAPABILITY sota: OSWorld 66% single-run, 72.6% with Behavior Best-of-N — first past the
~72.36% human baseline. Screenshot-first perception (no a11y tree), a SEPARATE swappable
grounding model (UI-TARS class) resolving NL element descriptions to coordinates, actions
emitted as pyautogui python and exec()'d. Agentic every time; no replay, no journal, no undo;
success is judged COMPARATIVELY post-hoc (narrate N rollouts, VLM picks the winner), not gated
absolutely. S3's design lesson: they DELETED S2's Manager/DAG hierarchy — "no hierarchy for
less inference time" — and spent the budget on Best-of-N selection instead.

(1) Design vs ELAI: on capability, ahead of anything ELAI specs for desktop GUI (ELAI's honest
    gap: the spec has NO desktop computer-use lane at all — browser E2E is E-14's ceiling; see
    synthesis). On trust: absent by their own shape — S2's KnowledgeBase saves EVERY run's
    distilled summary with NO success gate before commit; that unverified-save is precisely the
    gap ELAI's verified case graph (J) exists to close. Independent confirmation, again, that
    our differentiators are the un-shipped part of the field.
(2) Steals (ranked):
    - S2 KnowledgeBase retrieve/distill/fuse loop (narrative + episodic tiers, embedding
      retrieval, fusion into the next plan) as the CONCRETE prior art for the J verified case
      graph — port the loop, gate the SAVE behind our execution-grounded filter. An upgrade,
      not a copy; also citable prior art in the plan file.
    - bBoN behavior-narrative + comparative judge — but narrated from JOURNAL ROWS (we have
      ground-truth action/effect deltas; they need screenshots). Composes with I's
      evidence-ranked search: receipts pick first, narrative judge breaks ties.
    - mark_action + get_zoomed_image annotated before/after evidence crops — a "prove the
      click landed where intended" primitive for the visual-ui-loop evidence-region discipline.
    - S2's plan->DAG->toposort->action-queue compile step as a canvas-backend shape (nodes ARE
      the DAG) — though S3's own retreat from hierarchy is a caution against deep planning
      layers; ELAI's O-4 nodes are TaskSpecs, not planner output, which sidesteps that.
    - @agent_action decorator registry = enumerable frozen action vocabulary (audit-friendly).
    - flush_messages image-dropping context rule for image-heavy loops (H context-lifecycle).
(3) Wrap candidate: partially — the grounding-model-as-a-service pattern (UI-TARS behind an
    endpoint) is the piece to vendor if ELAI ever ships the desktop operator lane.

VERDICT: capability leader, trust vacuum. Steal its memory loop (gated) and its evidence-crop
recipe; do not copy its hierarchy history (they already un-shipped it).

---

## ENTRY 12 — SYNTHESIS (all six read; verdicts converge)

THE COMPARISON, answered as-specified:

1. On the axis ELAI actually claims — journaled, replayable, fail-closed, typed-workflow
   execution with receipts — NONE of the six matches the spec. Five of six have no off-agent
   evidence discipline at all; the sixth (openclaw) has real fail-closed instincts (confidence
   gates, metadata ledger, replay JSONL, frame-token binding) but explicitly scopes prompt
   injection OUT of its trust model, keeps its gates inside its own product boundary (nothing
   frozen against the run), and has no byte-reproducibility. ELAI's canvas design (journaled
   taint-tagged op-stream) and workflow design (closed-enum nodes, visibly-disconnected gaps,
   confirm-before-run, RRA-bound synthesis) are UNOCCUPIED territory across all six repos.
   The user's hunch that agent-zero might already be "ELAI canvas" is settled: it is not;
   nobody is.

2. Where the six beat the CURRENT spec (real gaps, design-grade, not status-grade):
   a) DESKTOP/GAME OPERATOR LANE: ELAI specs browser E2E (E-14) and media validation (4I) but
      NO general desktop computer-use stage. agent-s proves the capability ceiling, cua ships
      the sandbox substrate, omniparser ships the perception fallback for canvas/game surfaces.
      A workflow node "drive this GUI app / test this game build" has no ELAI home today.
   b) EXPLORATION->EXECUTION CRYSTALLIZATION: cua (record->replay->re-score), openclaw
      (lobster + jsonl-replay), agent-zero (skill scripts convention) all converge on the
      screenshot's thesis; ELAI has the general machinery (TaskSpecs, template crystallizer,
      verified case graph) but no specced browser/GUI instance of "agentic run mints a
      deterministic replayable artifact."
   c) SELF-EXTENSION ERGONOMICS: agent-zero (ungated), open-computer-use (catalog), openclaw
      (installPolicy-gated registry) — the field norm is install-on-demand; ELAI's P-9 pack
      lane is the gated answer but nothing specs the CATALOG (named tools, mirrors, aliases,
      license tags) that makes it feel instant.
   d) RESUME-TOKEN APPROVAL SEMANTICS: openclaw's lobster is a better-specified confirm gate
      than P-11 MOVE 3 (durable halt token, approve+resume never re-runs prior steps).

3. PRIORITIZED STEAL LIST (targets named per plan family; authoring new files needs the
   PSR-2 PROOF_SPINE ack — most of these are spine-exempt product surface):
   P1. "Explore-then-crystallize" browser/GUI lane: agentic E-14 exploration run emits a
       deterministic Playwright script AS an O-2 TaskSpec stage + RRA binding script->minting
       run; replay = execution mode. Prior art: cua replay_env, openclaw jsonl-replay,
       agent-zero scripts/ convention. EXTENSION(E-14 + O-2), lands in open letter.
   P2. Lobster resume-token semantics into the P-11/O-2 confirm gate (durable approval token,
       no re-run on resume). EXTENSION(P-11).
   P3. Set-of-Marks ID-indirection on AXTREE nodes + per-element provenance tags
       (axtree_exact|ocr|vlm_caption|pixel) into E-14/J-1 perception gateway. EXTENSION.
   P4. Desktop operator lane (the honest gap): cua sandbox as vendored substrate + omniparser
       as Tier-3 pixel fallback + openclaw frameId/display fail-closed binding + per-action
       bounded screen-recording (open-computer-use) + crosshair/zoom evidence crops (agent-s)
       — all under our gates (cua's own in-loop gating is a stubbed TODO). NEW unit; needs
       user ack (spine: none, product surface).
   P5. Verified case graph hardening: port agent-s S2 KnowledgeBase retrieve/distill/fuse,
       gate the save; add bBoN narrative-judge-over-journal-rows as tie-breaker.
       EXTENSION(J case graph unit; J CLOSED -> lands in open letter, cross-linked).
   P6. Gate-corpus prior art: openclaw confidence-self-test negative-control canaries cited
       into Q's evidence-corruption corpus unit.
   P7. Gated install catalog for P-9 (named {tool, source, alias, license_tag} rows +
       installPolicy) — the "download blender on demand" front door done fail-closed.
   P8. Smaller: metadata-only redacted ledger view (openclaw, G-9 adjacent); fallback-mode
       tool-surface collapse (open-computer-use); shadow-git preview-before-destroy details
       (agent-zero, H-49); desc2xy grounding cache + OperatorNormalizer diff vs H-13
       sanitizeAction (cua); dry_run golden frames (cua) for visual-ui-loop.

4. n8n/comfyui APPEAL: the six confirm the moat is real — nobody has a typed node canvas with
   receipts — but appeal needs three things the spec should absorb: (i) P1's crystallization
   (the "record my flow" feel n8n users expect), (ii) P7's catalog (capability-on-demand feel),
   (iii) P2's resume tokens (approval UX that doesn't restart work). With those, "n8n with
   receipts" is not just safer, it is operationally NICER than n8n's open-node model.

READINESS NOTE (separate from the verdict, index-derived, confirm against live code): canvas
partially shipped (J-60 monitor; H-13 op-stream per index); E-14, O-2/O-4, P-9/10/11 PLANNED,
post-proof-spine per PSR-1. Every comparator ships today. Timing gap, not design gap — except
2(a), which is a genuine design-scope gap in ELAI regardless of timing.

---

## ENTRY 13 — OpenClaw SOURCE-LEVEL VERIFICATION (deep pass, 2 opus agents read the actual TypeScript, not the docs; sources: docs/.source/openclaw-DEEP-lobster-qa.md + openclaw-DEEP-audit-capability.md)

Context: the openclaw-vs-elai comparison file (workflow/reasoning/openclaw-vs-elai-comparison-2026-07-11.md)
is DOC-grounded and already carries the design-level map + 20-item steal list. This entry records only
what SOURCE reading changed, corrected, or added — because OpenClaw is largely agent-generated at high
velocity, so a doc claim is a hypothesis until the code confirms it. Grades: VERIFIED-IN-SOURCE /
PARTIAL / DOC-ONLY-NOT-FOUND / STUBBED.

IS IT VIBE-CODED? (the user's actual question, answered from source, not vibes)
Both deep passes independently reached the SAME verdict: the four subsystems read as deliberate,
defensively-typed, TEST-COVERED code (every source file has a sibling .test.ts), NOT slop. No TODO/stub/
placeholder on any fail-closed path in the files read; per-branch comments explain the fail-closed choice;
state mutation is serialized. The audit ledger and the computer-use frame-binding are MORE rigorous than
the docs imply. The ONE genuine vibe-code tell found: the trajectory envelope's `schemaVersion:1` is a
hand-copied inline literal in 8+ emit sites with no shared builder (runtime.ts + export.ts ×6) — "one
shape everywhere" is enforced only by a shared TYPE, so a schema bump that misses a copy silently diverges.
So: Steinberger's ~100-Codex-instance velocity produced a codebase that is test-disciplined at the unit
grain; the unevenness is architectural/semantic (below), not sloppy. Steal designs, still never their code.

FOUR SOURCE-LEVEL CORRECTIONS to the doc-grounded steal list (these DOWNGRADE confidence on prior steals):

C1 — LOBSTER'S MARQUEE FEATURE IS NOT IN THE REPO. extensions/lobster/src/lobster-core.d.ts is a
   `declare module "@clawdbot/lobster/core"` TYPE STUB; the resume-replay engine (runToolRequest/
   resumeToolRequest, the step executor, the DSL parser) is an external npm package NOT vendored in a
   shallow clone. What ships in-repo is a WRAPPER (timeout/output-cap/cwd-jail). The headline
   "approve+resume never re-runs prior steps" is DOC-ONLY-NOT-FOUND — unverifiable and absent from
   visible code; on `resume` the wrapper makes ONE pass-through call to the external package
   (lobster-runner.ts:392-411). CONSEQUENCE: comparison-file steals S1/S6 and my P2 (Lobster resume-token
   semantics into P-11/O-2) rest on a property we CANNOT confirm exists as described. The IDEA is still
   worth designing toward, but cite it as an UNVERIFIED external claim, not "openclaw shipped this" — and
   a subprocess-model ELAI port (real SIGKILL, real pipe backpressure, real step-checkpoint store) would
   be STRONGER than their contingent, cooperative enforcement anyway.

C2 — LOBSTER'S "FAIL-CLOSED SANDBOX" IS PARTIAL. cwd is genuinely jailed (a correct cross-platform
   path-jail, case-fold-before-compare on win32 — lobster-runner.ts:141-165, worth copying verbatim), but
   full process.env is copied into the workflow context with NO allowlist (:279) — secrets in env are not
   sandboxed. Timeouts/output-caps are real but COOPERATIVE (a promise race + a Writable cap that only
   bite if the external runtime honors ctx.signal / propagates stdout write errors), not hard kills. Doc
   naming over-reads the teeth.

C3 — QA-LAB'S "PASS" IS A TRAP (new load-bearing nuance). The confidence-report default verdict is
   fail-closed on UNKNOWNS ONLY, not on classified failures: a green default report means "every required
   lane is classified / zero unknowns", NOT "everything works" — it can contain known product-bugs. Only
   `--strict-global-pass` (OFF by default, cli.runtime.ts:1223) makes classified fails block. A consumer
   reading a green report as "no bugs" is wrong unless that flag is set. Design lesson for ELAI: their
   taxonomy cleanly SEPARATES unknown (block) from classified-failure (route) — that is unknown=FAIL with
   a richer downstream router, and it is a genuinely good pattern (steal below), but the DEFAULT gate
   posture is a footgun to not copy.

C4 — QA-LAB JSONL-REPLAY IS NOT A VCR. It re-drives recorded INPUT transcripts (cumulative growing prefix,
   turn N against turns 1..N — a good pattern) through a deterministic SYNTHETIC provider (mock-openai, a
   3938-line generator; the default cell runner THROWS if no runtime is injected), NOT a replay of captured
   real-provider responses. So "openclaw replays trajectories" = replays INPUTS through a mock, not OUTPUTS.
   cua's replay_env (ENTRY 10) is the closer prior art for genuine record->re-score. Correct the framing.

FIVE CONFIRMED-REAL mechanisms (VERIFIED-IN-SOURCE) — these UPGRADE confidence, steal with the grade:

V1 — verify-the-verifier is REAL and load-bearing (the strongest thing in either QA subsystem).
   confidence-self-test builds 7 synthetic KNOWN-BAD artifacts and runs each through the SAME production
   detectors the real gate uses (not a re-implementation), and the self-test summary is itself a REQUIRED
   fail-closed lane — a missing/undetected canary drives the WHOLE gate to unknown->fail
   (confidence-report.ts:1139-1270 + evaluateSelfTestSummary:697-716). This is exactly ELAI's R2 (dead
   evidence path must scream) + verify-the-verifiers + Q's evidence-corruption corpus, SHIPPED. Port as:
   the "checker still works" proof is a fail-closed INPUT to the run, feeding the real code path. LOW port
   risk (design/discipline, language-agnostic).

V2 — AUDIT LEDGER REDACTION IS STRUCTURAL, not scrubbing (strongest steal of the deep pass). The durable
   record type has NO content field at all (audit-event-types.ts:34-56; the DDL has the same fixed columns)
   — raw content cannot leak into a field that doesn't exist. Plus hash-don't-store (toolCallId kept as
   `sha256:...`, agent-event-audit.ts:46-54) and whitelist-or-"unknown" for the tool-name field (:36-44).
   Maps directly to ELAI G-9 redaction: define the SSB/audit-export row as a CLOSED metadata struct;
   content has nowhere to go — far stronger than a denylist scrubber that can miss a field. LOW port risk.

V3 — BUT THE TRAJECTORY BUNDLE IS **NOT** METADATA-ONLY (critical correction — do not conflate). The
   audit ledger is structurally metadata-only; the trajectory bundle in the SAME repo deliberately carries
   finalPromptText / assistantTexts / submittedPrompts / tool text (export.ts:809-859). The comparison
   file's S8 ("metadata-only audit projection") is correct ONLY for the audit ledger, NOT the trajectory.
   Two subsystems, OPPOSITE privacy postures, similar "trace/audit" vocabulary — the exact conflation trap
   an LLM porting this would fall into. ELAI lesson: keep the shareable/export projection (metadata-only,
   structural) STRICTLY separate from the full debug bundle (content-carrying), by TYPE, and name them so
   they can't be confused.

V4 — COMPUTER-USE FRAME BINDING IS REAL AND STRONG (best-engineered of the four subsystems, maps to the
   desktop operator lane P4). A coordinate action is authorized only while the EXACT image the model saw
   is still in context: frame identity = sha256 of the delivered image bytes, epoch-invalidated when
   compaction drops the screenshot from context (computer-tool.ts:488-551); fail-closed on no-frame /
   node-switch / screenIndex-mismatch / geometry-change (:724-748), never a retarget to display 0; plus
   held-button target affinity (a down pins the node until a confirmed up). This is un-gameable by a
   stale/guessed token and is the concrete model for an ELAI operator lane: bind every actuator command to
   a content-hash of the observation it was issued against, reject when that observation is no longer
   provably in the model's context. MEDIUM port effort (entangled with their contextEpoch plumbing;
   reimplement against ELAI's compaction/observation model). Two supporting steals: two-phase arm commit
   (journal-the-grant BEFORE exposing the capability, so a crash mid-arm leaves no grant — phone-control
   index.ts:748-757) and dual expiry (a lease TTL checked BOTH on a 15s sweep AND inline at dispatch, never
   trust the sweeper alone).

V5 — CLAWHUB HAS NO BUILT-IN DANGEROUS-CODE GATE BY DEFAULT (sharpens the competitive line). Source
   confirms: the built-in install-time dangerous-code scan was REMOVED (plugins-install-command.ts:125);
   installPolicy is a real fail-closed gate ONLY when an operator wires an external decision command, and
   is DISABLED by default (install-policy.ts:437-444 -> undefined -> install proceeds); the regex pattern
   scanner that exists is ADVISORY-only (audit/workshop surfaces), orphaned from the install path. So
   out-of-box, `openclaw skills install` runs NO content gate. Combined with SECURITY.md scoping
   prompt-injection out, this is the sharpest ELAI positioning exhibit: a ~250K-star agent gateway whose
   plugin registry (ClawHub) is the same class of ecosystem the ToxicSkills/ClawHavoc study (cited in our
   .claude/rules/skill-import-policy.md — 1,184 malicious skills) found compromised, shipping with no
   default install-time code gate. This is exactly what ELAI's frozen-gate + skillspector /skill-audit +
   wire-checker doctrine exists to beat; do NOT port their regex scanner (signature-match with no teeth =
   the pattern our gate doctrine already rejects).

TWO NET-NEW STEALS the deep pass surfaced (not in the comparison file's S1-S20):
N1 — missing-artifact=FAIL with an in-manifest explicit escape (a missingVerdict may ONLY be
   environment-blocked or optional-gap — confidence-report.ts:744-760): structurally identical to ELAI's
   WIRE-DARK allowlist row (absence-of-evidence defaults to FAIL; the only pass path is a declared,
   reasoned manifest row). Independent convergence on our doctrine.
N2 — skipBackfill cross-lane dependency (a lane with skipped rows clears ONLY if a NAMED sibling lane
   passed, else it is a global failure — confidence-report.ts:856-909): threaded-contract / R10 thinking
   baked into the artifact model — you cannot skip a check and silently pass; the skip must be covered by
   a passing sibling. Maps onto wiring-enforcement R10's threaded-required-input rule.

NET EFFECT ON THE VERDICT: unchanged in direction, sharper in detail. OpenClaw's fail-closed INSTINCTS are
real and test-covered where they exist (V1-V4, N1-N2 are genuine and independently converge on ELAI
doctrine), which makes it a better STEAL SOURCE than the doc pass suggested — but its two headline
trust-relevant claims degrade under source reading (C1 lobster resume unverifiable, C4-adjacent: the
trajectory is content-carrying not redacted, V5 install path ungated by default), which keeps the wedge
fully intact: shipped fail-closed instinct, but no frozen off-agent gate, no replay-verify, no default
install-code gate, injection out of scope. The comparison file's steal list stands with these grade
annotations: S15/V1 confirmed-strong, S8/V2 confirmed but scope it to the audit ledger ONLY (V3), S1/S6/P2
carry the C1 "external-unverifiable" caveat, and the frame-binding/arming steals (P4) are the
best-engineered, highest-fidelity ports available for a future ELAI computer-use operator lane.

AMENDMENT (a second, independent opus source-read of lobster+qa landed as docs/.source/openclaw-DEEP-lobster-qalab.md — it CORROBORATES every load-bearing point above; two independent reads agreeing raises confidence on C1/C3/V1. Three details it adds):
 - C1 refined: the actual double-resume guard is NOT in lobster at all — it is OpenClaw's managed-TaskFlow
   OPTIMISTIC CONCURRENCY (a stale expectedRevision -> revision_conflict, every mutation revision+1;
   task-flow-registry.ts:515-521,413). But resumeFlow guards on REVISION ONLY with NO STATUS GATE
   (:566-588) — so a completed flow can be resumed again at that layer; the "never re-runs prior steps"
   safety, wherever it lives, is not this guard. This makes the comparison file's S6 "revision-counter
   optimistic concurrency" steal the REAL, verified mechanism (good, portable) while confirming the
   resume-token step-skipping (S1/P2) is the unverifiable external part. Steal the revision-conflict store;
   treat the step-skip as an idea to re-derive.
 - V1 strengthened: a failed self-test sets exitCode=1 — it HARD-BLOCKS (not warns), same for the
   confidence-report (cli.runtime.ts:1233-1235,1253-1254). The verify-the-verifier isn't just a required
   lane, it fails the process. And the wrapper force-converts the engine's `needs_input` status to an ERROR
   (lobster-runner.ts:183-190) — i.e. the interactive-pause/resume path the resume-token story needs is not
   even wired in the in-repo wrapper, reinforcing C1.
 - Peripheral steal: an AJV content-addressed compile-cache monkey-patch (sha256-keyed schema -> compiled
   validator, 512-entry FIFO; lobster-ajv-cache.ts) — a clean idiom if ELAI ever compiles per-node schemas
   on the workflow canvas hot path.

---

## ENTRY 14 — Claude Code desktop in-app browser (source: x.com/ClaudeDevs/status/2075635283211772279, full tweet JSON fetched via syndication CDN 2026-07-11; 9to5mac.com/2026/07/10 coverage; HIGH reliability on the announcement text, mechanics beyond it are vendor-doc-thin)

The feature (announced 2026-07-10): Claude Code on desktop embeds an in-app browser pane.
Claude can pull up docs, designs, any site; read, click through, and interact the same way it
does with local dev servers. Sandboxed: a clean browser profile separate from the user's
personal browser (no saved logins/history); session persistence is a user choice. Positioned
for "building and testing your app" + identity-free sites.

Mechanism read: this is an INTERACTIVE AGENTIC surface — the model drives a real embedded
browser per-run, the user watches. Nothing in the announcement or docs claims: a journaled
action stream, a replayable artifact, an exploration->script crystallization, off-agent
evidence for "the test passed", provenance on perception, or any gate between the model's
claim and the verdict. Isolation is the whole trust story (profile separation + sandbox),
which is the same story every one of the six repos in ENTRY 12 tells.

Like-for-like at design level:
 - Their sandbox/clean-profile ≈ P-12's disposable sandbox — parity on isolation, but P-12
   additionally binds every coordinate action to the frame_hash of the exact observation it was
   computed from and journals it taint-tagged. No CC analog.
 - Their "click through and interact" ≈ E-14's agentic browser E2E — but E-14 is AXTree-first
   with axe-core as a deterministic gate, VLM strictly Tier-2 advisory, SSIM-first diff deciding
   where to look, and SSB rows as the pass artifact. CC's loop has no evidence layer at all:
   "it worked" is the model's own narration over pixels (the exact in-boundary artifact our
   wiring doctrine rejects).
 - Their per-run agentic driving vs P-14 explore-then-crystallize: CC re-reasons every run
   (cost + nondeterminism forever); P-14 mints a deterministic Playwright script RRA-bound to
   the exploration run that produced it, so replay is the zero-reasoning execution mode. CC has
   no execution mode, only exploration mode (the ENTRY 2 dichotomy — CC ships the expensive half).
 - Their dev-loop use case ("interact with your local dev server") vs H-14 design mode: CC gives
   a DOM click; H-14 resolves a click to the element's AST RANGE via opaque ids (click-to-edit
   lands in source, not in the render). Deeper binding, same gesture.

Where THEIR design is genuinely better (anti-sycophancy, two items):
 1. A general-purpose co-browse pane. ELAI specs no surface where user+model interactively
    browse an ARBITRARY site together — E-14 is test-scoped, canvas RESEARCH mode is
    research-scoped, P-12 is an operator lane of last resort. Partly deliberate (an unjournaled
    free-browse surface forfeits receipts; web content enters ELAI as taint-tagged ingress),
    but the convenience is real and the gap is honest: ELAI's answer would have to be a
    journaled co-browse lane (every nav/click an SSB row, content taint-tagged at deixis —
    H-15's machinery already covers the handoff half), not a naked webview.
 2. Session-persistence as a one-bit user choice is clean UX for the identity problem; ELAI's
    equivalent (sandbox lifecycle + credential vault P-10) is stronger but heavier ceremony for
    the "just read these docs" case.

VERDICT: ahead on the axis that is the wedge — receipts. CC's in-app browser is the field-norm
design (agentic driving + isolation, zero evidence discipline) landed in a first-class daily
surface. ELAI's specced surface is categorically stronger on verification (journaled ops,
provenance-tagged perception, crystallized replay, fail-closed gates) and has ONE real
design-level hole CC exposes: no general co-browse surface. Candidate: a thin "browse node"
lane = embedded pane + journaled nav/click stream + taint-at-deixis, composing H-15 + G-1 —
an extension idea, not a new subsystem.

READINESS NOTE (sequencing, not verdict; index-derived, confirm against live code): CC's pane
ships today in the desktop app; ELAI's E-14/P-12/P-13/P-14 are PLANNED (P wave gated post
proof-spine per PSR-1). A timing gap, not a design gap.

---

## OUTCOME (paper-to-plan, 2026-07-11)
This eval's steal list P1..P8 was authored into ELAI plan files as Track COMPUTER-USE-STEAL (P-plan, open letter):
P-12 desktop/GUI operator lane (NEW, spine none), P-13 perception provenance + Set-of-Marks id-indirection (EXT
E-14/J-1), P-14 explore-then-crystallize record->replay lane (feeds PS-5), P-15 verified case-graph hardening (EXT
J-96, feeds PS-5), P-16 gated install catalog (EXT P-9). All plan_gate_check.py PASS (exit 0). Doctrine + tables in
P-0_INDEX.md (THIRD WAVE), entries in P-SUMMARY_INDEX.md, design in ELAI-SPEC-OVERVIEW.md P computer-use wave.

## OUTCOME 2 (co-browse gap session, 2026-07-11)
ENTRY 14's one design-level hole (the Claude Code in-app browser, no general co-browse surface in B..Q) was authored
as P-23_journaled_cobrowse_lane.md (CU-4 addendum, Track COMPUTER-USE-STEAL; EXT H-15 + E-14, depends P-13;
PROOF_SPINE none (product), owner-commissioned this session). Mechanism decision (GRC ledger, topic
cc-inapp-browser-vs-elai): engine-owned Playwright + CDP screencast + ONE InputProxy (pane = dumb glass, journal-then-
dispatch, bypass cannot compile) over a native Tauri multiwebview embed (unstable flag + unjournaled-input risk,
grounded). plan_gate_check.py PASS (exit 0). Upserts: P-0_INDEX (title 23 sessions, CU-4 addendum block, table row,
PROOF_SPINE line, SSB rows cobrowse_session/cobrowse_input), P-SUMMARY_INDEX (full entry), ELAI-SPEC-OVERVIEW (P
computer-use wave paragraph), ELAI-FEATURE-MAP (P-23 entry + header date + a STALENESS NOTE: P-9..P-22 were never
upserted to the feature map by their authoring sessions — backfill open).

## OUTCOME 3 (smartphone follow-up, same session 2026-07-11)
The user's "what about smartphone/website canvas interactive stuff, maybe H?" resolved to: website = H-14 (specced:
design.page live previews, click->AST-range, recast writeback); smartphone = H-14's EXPLICIT deferral (H-14:7 "RN/Expo
deferred to a later separately-scoped slice"; :184 notes the current gig's locked stack is RN) with zero other mobile
coverage in B..Q (corpus grep). Authored P-24_mobile_design_test_lane.md (CU-5 addendum, EXT H-14 + P-12/P-13/P-14 +
E-14, effort L): Rung 1 DeviceProfile matrix on E-14/H-14/P-23; Rung 2 Expo-web click-to-edit through the H-14 proxy
(Expo web production-grade SDK 54+, grounded); Rung 3 native emulator as a P-12 backend + mobile_axtree P-13 tag +
P-14 crystallize to an actuation-only Maestro flow (verdicts stay on the off-agent evidence leg; Maestro grounded
mobile-dev-inc, license RECON at edit). Typed render_source per row kills rung laundering. plan_gate_check.py PASS
(exit 0). Upserts: P-0_INDEX (24 sessions, CU-5 block, table row, PROOF_SPINE line, SSB rows emulator_session +
EXTEND fields), P-SUMMARY_INDEX (full entry), ELAI-SPEC-OVERVIEW (P wave paragraph), ELAI-FEATURE-MAP (entry + header).

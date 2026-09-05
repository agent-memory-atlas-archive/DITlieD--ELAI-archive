J-PLAN PRE-READING REASONING LOG - 2026-06-10
Purpose: incremental per-file reasoning before authoring the J plan (Section B of
workflow/reasoning/harness-redesign-recommendations-2026-06-10.md = the six new items).
Method per user instruction: read ONE file, write reasoning here, then next file.
This file doubles as compaction-resilience scratchpad for this session.

================================================================================
FILE 1/14: docs/specs/guide/steps/phase-index-MASTER.txt (1057 lines, read fully)
================================================================================

WHAT IT IS: single-file master index of the base spec phases (pre-0 through 4M),
generated 2026-05-17. Each entry = goal + key deliverables + dependencies. The 38
individual phase-N.txt files hold the full PRE_GATE/TASK/DETERMINISTIC_CHECK/
ADVERSARIAL/ACCEPTANCE blocks.

KEY TAKEAWAYS FOR J PLAN:
1. The base system architecture J items must compose with: 9-crate workspace
   (core/providers/knowledge/reasoning/orchestrator/tools/plugins/rules/cli),
   SSB single-writer SQLite WAL event log (typed action_types, append-only),
   FSM 9 states with evidence gates, BSP supersteps, Tier C process isolation
   (4A, 15-msg IPC), DecompositionTree (4E ext), AutonomousGate (4G),
   self-mod DGM archive (4H), Tauri dashboard (4K), RuntimeAdapter multi-backend (4M).
2. Established plan-unit conventions J files must follow: TDD red/green ordering
   (test step before implement step), GROUNDING block classifying claims,
   deterministic checks (ClockProvider/EntropySource, ELAI_SEED), metrics toml
   per phase, adversarial test blocks, entropy quarantine.
3. Spec anchor convention: plan files cite "Spec references:" into phase-N.txt
   files. J items that touch orchestrator/SSB/FSM should anchor to phase-3A/3B/
   3C/4A/4G/4H; harness-side items anchor to .claude/ rules instead (workflow
   tooling, not .elai_cc code) - need to check where harness-redesign items land:
   .elai_cc runtime vs .claude harness. The redesign doc title says "harness",
   so likely MOST J items are .claude/ workflow-tooling items, NOT .elai_cc crates.
   That changes the wiring-contract shape: harness items' observable effects are
   hook logs / session artifacts, not SSB rows. WAIT for file 11 to confirm.
4. Numbering: base phases are 0-4M and are CLOSED (4M added 2026-05-10 was the
   last). Letter plans B..I extend the system. J continues that lineage; J files
   do NOT add base phases.
5. SSB action_type additions have a golden lock (action_type_lock) + SCHEMA_VERSION
   bump discipline - any J item adding SSB events must state baseline+N and verify
   live numbers at build time (live numbers move; CLAUDE.md 191/464 already stale,
   I-plan memory says 221/532 as of 2026-06-10 - MUST grep live values before
   authoring J SSB claims).

NO J-RELEVANT CONFLICTS FOUND: base spec leaves harness-side (.claude/) concerns
entirely to the workflow root; J items targeting the harness do not collide with
base phases.

================================================================================
FILE 2/14: docs/thefinalglance/B-plan/B-PLAN-SUMMARY.md (1112 lines, read fully)
================================================================================

WHAT IT IS: master index of B-1..B-42 (45 files), 2026-05-17 snapshot. 8 parts:
MVP completion, dashboard V2, harness hardening, final product (4F-4J+cloud+
focus tunnel), knowledge extensions, src-tauri wiring, ouroboros ports, chat
REFLECT/framing/audits/cosmos.

KEY TAKEAWAYS FOR J PLAN:
1. PLAN-FILE FORMAT PRECEDENT (strongest signal so far): each B-N has objective,
   prereqs, deliverables, status, rationale; per-plan SSB action_type table with
   schema bump; "hard rules invoked" cross-cut; file ownership map (who owns
   which elai_cc.toml block); cross-plan dependency graph; recommendation order;
   per-part session estimates. A J-SUMMARY_INDEX should mirror: TOC, status
   legend, per-J entry, schema-bump table, dependency graph, recommended order.
2. DECISION-DOC CONVENTION: "every B-N produces workflow/reasoning/B-N-decisions.md"
   - J files must require workflow/reasoning/J-N-decisions.md at execution time.
3. RELEVANT SHIPPED SUBSTRATE J items can compose with (avoid re-planning):
   - B-22/B-22b autonomous gate + StagnationDetector + 5 personas (the .claude
     anti-spiral skill + persona agents ALREADY exist harness-side too).
   - B-37 RuntimeAdapter 11 backends (any J item about model routing composes).
   - B-39 REFLECT pipeline, B-40 idea-framer + critic loop, B-41 per-level
     auditor modes, B-42 cosmos artifact tier - all SHIPPED.
4. STILL-PLANNED B ITEMS THAT MIGHT OVERLAP A "harness redesign" J item:
   B-17 (ground vacuous gates), B-18/B-19/B-20 (Tier C sandbox + hook/tool
   unification), B-23 (self-mod), B-27 (cloud mode), B-28 (focus tunnel),
   B-29..B-35 (knowledge extensions). If a Section B six-item overlaps one of
   these, J must REFERENCE not duplicate (one-line pointer + delta only).
5. SCHEMA DISCIPLINE CONFIRMED: every B-N that adds SSB variants logs the bump
   in a cross-cut table. J index needs the same table even if J is mostly
   harness-side (harness items may add ZERO SSB variants - that is fine and
   should be stated explicitly to avoid the wiring gate's R9 confusion).
6. DISTINCTION SHARPENING: B-plan = .elai_cc runtime work. The harness-redesign
   doc (file 11) is about the WORKFLOW HARNESS (.claude/ rules, agents, skills,
   hooks, tools). J plan will likely be the first letter plan whose primary
   target is the harness root, not .elai_cc. Precedent for harness-side work
   inside plans exists (B-22b ships .claude/agents/persona-*.md), so a J file
   may ship BOTH harness artifacts and (optionally) .elai_cc surfaces.

================================================================================
FILE 3/14: docs/thefinalglance/C-plan/C-SUMMARY_INDEX.md (815 lines, read fully)
================================================================================

WHAT IT IS: index of C-1..C-59 (60 files incl index), 2026-05-17. C-plan =
"close the B-plan + guide + phase gaps with per-session cloud-Sonnet smoke
proofs" (C-1..C-33), then paper-grounded improvements (C-35..C-53, 19 verified
arXiv papers), then RunPod benchmark publication (C-54..C-59).

KEY TAKEAWAYS FOR J PLAN:
1. PER-SESSION ENTRY FORMAT refined here vs B: each C-N lists Closes / Pre-reqs /
   Parallel-safe-with / Adds (types+files) / Smoke (one concrete command + expected
   observable). The "Smoke:" line IS the wiring-contract trigger in miniature.
   J files should carry the same: one observable trigger per item.
2. WAVE STRUCTURE precedent: C uses dependency waves (W1 no-prereq parallel-safe
   ... W9 capstone). J with ~6-10 items can use a flat 2-wave order instead -
   keep it simple per user's "small in file amount" instruction.
3. ALREADY-PLANNED items the J six-items might collide with (CRITICAL overlap
   watchlist for file 11):
   - C-35 tool-gating + lazy MCP schema loading (if a harness item proposes
     deferred-tool discipline, the RUNTIME analog exists at C-35).
   - C-41 skills-driven workflow layer INSIDE elai (skills registry, 9 skills).
   - C-47 case-based tool-use telemetry; C-49 experience distillation for
     prompt-engineer (harness-analog: distilled per-skill experience files).
   - C-48 SafeHarbor memory trust tiers (harness-analog: memory provenance).
   - C-27 wiring contract enforcement runtime leg (already partially shipped
     via the 2026-06-03 wiring-fix-elai 7-wave session per memory).
   - C-40 GitHub PR adapter; C-39 MCP bridge CLI.
   Any J item resembling these must cite and DELTA, not duplicate.
4. CITATION DISCIPLINE: C-35..C-59 verified every arXiv ID via abs-page fetch
   and excluded out-of-window or dominated papers. J files citing papers must
   carry IDs already verified by the source docs (file 11/12) or mark them
   VERIFY-AT-EXECUTION; per feedback_paper_citation_verify memory, 2 bad cites
   slipped into B-40/B-42 era plans - do not introduce fresh unverified IDs.
5. STATUS REALITY CHECK: C-plan was all PLANNED at 2026-05-17; the FINISH
   master index (file 14) + wire-remediation handoff (file 13) hold the current
   truth of what shipped since. Do not trust C-SUMMARY status fields.

================================================================================
FILE 4/14: docs/thefinalglance/D-plan/D-SUMMARY_INDEX.md (311 lines, read fully)
================================================================================

WHAT IT IS: D-1..D-30 (2026-05-19), 5 waves. D = self-improvement substrate
(typed ReasoningTemplate archive, PPM verifier sidecar, multi-verifier
consensus, tripwires, BAI promotion, held-out store, crypto archive, L3 frozen
surfaces) + inference-time grounding modes (mode routing A-E, checker registry,
operationalization predicate, Adversary role, sycophancy probe, NoveltyClass,
UniCR conformal refusal, refusal SLA, ObservableEffect) + benchmarks + dashboard.

KEY TAKEAWAYS FOR J PLAN:
1. ENTRY FORMAT here adds two J-relevant fields beyond C's: "Closes:" (which
   upstream design section this discharges) and "Cites:" with TIER labels (T1)
   + explicit "Cloud test:" acceptance. Also a "paper grounding paths" section
   listing which papers paper-ingestor must pull BEFORE EXECUTING. J files
   citing papers should name the ingest dependency the same way.
2. OVERLAP WATCHLIST ADDITIONS for the six items:
   - D-13 Adversary role (cross-family verification) - harness analog already
     exists as the gpt-bridge second-opinion capability; a J item about
     cross-model checking must cite D-13 + the adversary agent (.claude/agents
     has adversary in the Agent tool list per session env).
   - D-14 sycophancy probe / D-16 UniCR refusal / D-17 refusal SLA - any J item
     about calibration/abstention composes here.
   - D-9 L3 frozen surfaces = runtime analog of harness R8 freeze-the-gate.
   - D-10 mode-routing predicate routed ORCHESTRATOR-side (LLM has no input) -
     this is the design principle "deterministic routing outside the model",
     reusable for any J harness item about dispatch/routing.
3. SEQUENCING DOCTRINE: D explicitly blocks D-1 until named C items DONE.
   J should declare its dependency surface the same way (likely: depends on
   the SHIPPED .claude wiring-gate harness from 2026-06-02/03 sessions + on
   nothing else heavyweight, since harness items are mostly self-contained).
4. H-plan Tracks 6/7 (per memory) already determined that several D items
   (D-10/D-11/D-16/D-17, D-5/D-29, D-6) are SHIPPED or extended - status in
   this index is stale; the FINISH index is the truth source.

================================================================================
FILE 5/14: docs/thefinalglance/E-plan/E-SUMMARY_INDEX.md (616 lines, read fully)
================================================================================

WHAT IT IS: E-1..E-55 (2026-05-19), from the 55-report grounded sweep (~770 web
fetches). Each E-N = one report's recommendation shipped as a session, with
"Closes:" report mapping + observable-effect smoke + closure map onto B/C/D.

KEY TAKEAWAYS FOR J PLAN (overlap watchlist is the main yield):
1. HARNESS-ADJACENT E ITEMS the six new items must be diffed against:
   - E-22: deferred MCP schema loading flag + SKILL.md export + microsandbox -
     runtime analog of harness lazy-loading ideas.
   - E-25: Claude Code architecture steals - 6-tier compaction ladder,
     prompt_decision hook primitive, session forking, pre-edit checkpoints,
     anti-pattern guards. ANY J item stealing Claude-Code-harness mechanics
     must check E-25 first (it is the runtime port of those same mechanics;
     H-49 time-travel and H-50/51 context lifecycle already extended it).
   - E-43: agent skill systems (frontmatter compliance, per-skill eval dirs,
     import gate) - SHIPPED per CLAUDE.md E-43 pointer; .claude/rules/
     skill-import-policy.md exists. A J item about skills must be a DELTA.
   - E-29: OTel observability on SSB; E-30 PreToolUse argv safety floor
     (harness has its own pre-tool hooks - distinct layers, same doctrine).
   - E-10: hostile critic of ELAI design (ablation harness; shipped per E-10
     pointer in CLAUDE.md with critic_decision_event SSB).
   - E-9/E-49: eval/metrics methodology (paired bootstrap, attribution tuple) -
     if a J item adds harness-side measurement, reuse this methodology rather
     than inventing one.
2. FORMAT NOTE: E entries add "Closes: <report>" provenance + explicit
   "Parallel-safe with:" + closure-map table (Augments B / Extends C / Extends
   D). J files should carry an analogous provenance line pointing at the
   harness-redesign doc section + ext-repo steal-list entry each item came from.
3. DOCTRINE REUSABLE FOR J: every E-N proves itself with an observable_effect
   smoke (CLI command > SSB row > artifact). For harness-side J items the
   equivalent observable chain is: trigger (slash command / hook event) >
   off-agent artifact (hook-executions.log row / session file / tool output)
   > assertion. Missing evidence = FAIL per wiring-enforcement R1.
4. DEFERRED-pattern: E-47 ships SPEC.md ONLY with a CI-lint ship-gate token.
   Useful precedent if any of the six items is research-grade: a J file can
   legitimately ship a spec + gate without code.

================================================================================
FILE 6/14: docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md (287 lines, read fully)
================================================================================

WHAT IT IS: F-1..F-13 (2026-06-01/02), VibeSec/SHARD steal-and-build. Per
memory project_fplan_tier1_run: ALL 13 units SHIPPED, committed 551d1aa
(RunPod/sidecar live checkboxes deferred). F-plan's differentiator = ENFORCED
done-gate: wiring_contract_check.py --run-trigger PASS, W-1..W-10 doctrine.

KEY TAKEAWAYS FOR J PLAN:
1. F-PLAN ENTRY FORMAT IS THE STRONGEST TEMPLATE for J: each F-N carries
   (a) cluster + classification + adversarial verdicts (novelty vs existing
   plans, grounding accuracy, feasibility), (b) "Existing ground truth
   (code-read)" with file:line anchors, (c) REVISED scope after adversarial
   pass, (d) Integration point, (e) observable_effect as the done-gate.
   The six J items came out of a similar adversarially-verified doc (file 11),
   so the J files should mirror this structure: verdicts -> ground truth ->
   revised scope -> integration -> observable_effect.
2. W-DOCTRINE TO INHERIT: W-1 (done = wiring gate PASS not build-green),
   W-2 (default-off env gate banned as terminal state; deliberate deviations
   recorded), W-4 (frontend pair when dashboard-surfaceable), W-5 (live
   end-to-end trigger; RunPod-gated items ship a deterministic fixture harness
   now + live checkbox later), W-10 (tiered split keeps network/subprocess off
   the fail-closed hot path). For harness-side J items the analog of W-4 is
   "rule/skill/hook registration pair" (a tool without its conditional-routing
   or skill trigger entry is wire-dark in the harness sense).
3. OVERLAP WATCHLIST ADDITIONS: F-6 learned-skill trust ladder (if a J item
   touches skill/memory trust, S1-S4 tiers exist); F-13 archive-before-
   destructive-change pattern (reusable doctrine).
4. STATUS: F COMPLETE = the six J items were proposed (2026-06-10) AFTER
   F shipped, so the harness-redesign doc already had F's substrate in view.
   Expect the six items to be harness-side gaps F/G/H/I did NOT cover.

================================================================================
FILE 7/14: docs/thefinalglance/G-plan/G-SUMMARY_INDEX.md (226 lines, read fully)
================================================================================

WHAT IT IS: G-1..G-10 (2026-06-02), trust-maximization READ/EVIDENCE/RUNTIME/
MEASUREMENT initiatives. Doctrine adds GD-1..GD-5 on top of W-1..W-10.
Per memory: all 9 detail files authored; G-2 rra.rs + G-8 attestation.rs
later confirmed SHIPPED (I-plan Track 20 memory).

KEY TAKEAWAYS FOR J PLAN:
1. NEW DOCTRINE LAYERS J SHOULD INHERIT (they generalize beyond G):
   - GD-1 COMPOSE-NOT-RESTATE: every J item must name what it composes with
     and re-implement nothing; restating an existing plan item = P0.
   - GD-2 recomputable numbers: any metric a J item emits must be recomputable
     from its journal/artifact source.
   - GD-4 FP-bounding before default-ON: any new harness gate added by J must
     bound false-positive rate on >=3 real runs before enforcement default-ON.
   - GD-5 honest-scope: no fail-open/pending path advertises a guarantee.
2. ACTIVATION-NOT-INVENTION pattern: G-1/G-7/G-10 all light up SHIPPED but
   wire-dark substrate (provenance.rs constructors test-only, conformal
   consumer without producer). If a J item proposes a "new" capability that
   actually exists wire-dark in the harness (.claude/tools etc.), the J file
   must frame it as activation, with grep-confirmed ground truth lines.
3. OVERLAP WATCHLIST ADDITIONS: G-2 RRA (reproducible run artifact) and G-8
   attestation - if a J item adds harness-side run journaling/replay, compose
   with these (and with H-49 time-travel). G-10 model-drift canary pattern is
   reusable for harness-side "model behavior changed" checks.
4. CONFIRMED FORMAT DECISION: G mirrors F format and says so in line 2.
   J-SUMMARY_INDEX will declare "Mirrors F/G format" the same way; J files
   carry the same per-entry fields + doctrine block (JD-* or inherit W/GD).

================================================================================
FILE 8/14: docs/thefinalglance/H-plan/H-SUMMARY_INDEX.md (1039 lines, read fully)
================================================================================

WHAT IT IS: H-1..H-54 across Tracks 1-4 + 6-11 (Track 5 odysseus RESERVED,
unauthored, starts H-55). T1 efficiency (caching/output/routing activation),
T2 canvas, T3 trading-trust gates, T4 UX+user-memory, T6 grounded-reasoning,
T7 DGM verification, T8 reversibility H-49, T9 context lifecycle H-50/51,
T10 review-anchoring H-52, T11 external-system steals H-53/54.

KEY TAKEAWAYS FOR J PLAN:
1. TRACK-NUMBERING CONSTRAINT: H tracks run 1-11 with Track 5 reserved; I-plan
   (file 9) uses Tracks 12-20. J plan should NOT reuse "Track" numbering unless
   it claims the next free track numbers (21+). Simpler: J keeps its own letter
   numbering (J-1..J-N) without track machinery - 6-10 files don't need tracks.
2. SCHEMA DISCIPLINE EVOLUTION: H files each state "LIVE baseline X/Y...
   re-read the live integer at edit (W-9)" because every snapshot goes stale.
   J files that add SSB variants must use the same "baseline+N ship-order-
   cumulative, confirm live const at authoring" formula. Latest stated here:
   224/501 (2026-06-05); I-plan memory says 221/532 (2026-06-10) - CONFLICTING,
   so I MUST grep the live consts before writing any J number.
3. STRONGEST OVERLAP-RISK ITEMS for harness-redesign six:
   - H-34 MCP defer-loading / tool-search emulation (token tax on tool schemas)
     and H-4 Tool Search Tool - if a six-item proposes harness-side deferred
     tool loading, the HARNESS ALREADY HAS IT (this very session: deferred
     tools + ToolSearch). Runtime side is H-4/H-34.
   - H-35 /rewind + H-49 time-travel; H-50/H-51 context lifecycle + sleep-time
     handoff (wires .claude/rules/handoff-protocol.md INTO the runtime).
     A six-item about session continuity/handoff must compose with H-51.
   - H-31/H-32/H-33 two-tier user memory + governance (auditor-objectivity
     firewall doctrine: auditor/critic/adversary get ZERO memory injection).
     A six-item about harness memory must respect the same firewall doctrine.
   - H-52 finding-anchoring; H-53 citation pre-binding - "verbatim snippet,
     never a line number" doctrine reusable for any harness reviewer item.
   - Track 10/11 set the precedent for EXTERNAL-STEAL plans: clone to
     docs/.source, map, explore, verdict anti-sycophantic, steal-the-shape.
     File 12 (ext-repo-steal) follows this; J items sourced from it inherit.
4. ENTRY-FORMAT FINAL FORM (Tracks 4+): adds "Premise:" line, "Existing ground
   truth (code-read date)" with file:line, "REVISED scope (W-10)", explicit
   DEFAULT-ON-with-kill-switch vs default-OFF-with-allowlist-row decision per
   item, and broken-arm R2 test in observable_effect. J files adopt this full
   form. Also: "Detail: <path to its own H-N file>" cross-link.
5. DOCTRINE FAMILIES so far: W-1..10 (F), GD-1..5 (G), HD-1..5 (H T1/2),
   TD-1..5 (T3), RD-1..4 (T6), VD-1..5 (T7), RV-1..4 (T8), CL-1..4 (T9).
   J adds JD-* only if the six items genuinely need a new family; otherwise
   inherit. Given the six are HARNESS-side, a small JD family for harness
   wiring (harness analog of W-1: "done = hook/skill/rule actually fired in a
   live session, logged off-agent") is likely justified.

================================================================================
FILE 9/14: docs/thefinalglance/I-plan/I-SUMMARY_INDEX.md (1049 lines, read fully)
================================================================================

WHAT IT IS: I-1..I-54, Tracks 12-20 (2026-06-10, TODAY). T12 fault-isolation/
supervisor plane, T13 evidence-ranked search, T14 cache-invariant middleware,
T15 deterministic gates/anti-gaming, T16 routing/budgets/measurement, T17
open-weight flywheel, T18 deterministic-over-prompt, T19 design-critique +
claim discipline, T20 competitor-steal ports. "First letter authored AFTER the
wiring bedrock was reached (wire-remediation 2026-06-07: no clean flips
remain)." LIVE SCHEMA baseline stated: 221/532 at 2026-06-10.

KEY TAKEAWAYS FOR J PLAN:
1. J IS AUTHORED THE SAME DAY as I Tracks 19/20. The harness-redesign doc +
   ext-repo-steal doc (files 11/12) are 2026-06-10 artifacts too. CRITICAL:
   I-plan Track 20 already consumed docs/.source/STEAL-LIST.md - so file 12
   (ext-repo-steal-recommendations) is either (a) the HARNESS-side residue of
   the same steal exercise, or (b) a different repo set. Must diff carefully:
   any J item that duplicates I-45..I-54 is a GD-1 P0.
2. HEAVY OVERLAP WATCHLIST (runtime side, for the six items):
   - I-12 evidence-gated playbook curator EXPORTS to CLAUDE.md/AGENTS.md/
     skills - "learned content interop" - if a six-item proposes harness
     memory/rules learning, I-12 is the runtime sibling.
   - I-39 plan-state-from-repo (kill hand-maintained status drift; "every
     index's SCHEMA snapshot went stale within days") - if a six-item proposes
     harness-side plan-status derivation, compose with I-39 (it is the elai
     CLI leg; the harness leg would be bn_status_report.py evolution).
   - I-41 critique lenses (advisory-only, anchored-or-dropped) - doctrine for
     any harness-side LLM-judged surface: ADVISORY, never load-bearing.
   - I-48 prompts-as-reviewed-file-artifacts - harness analog would be agent
     .md prompt hygiene; harness already does this (.claude/agents/*.md).
   - I-52 checkpoint/rewind tool; I-54 structural AST edit tool (harness
     sibling named: impact-mapper).
3. TRACK 19's METHOD is the closest model for J: "the genuinely-uncovered
   residue after cross-referencing source docs against B..I" + recorded
   declines so they are not re-litigated. J-SUMMARY_INDEX should carry the
   same: a DECLINES/ALREADY-COVERED section listing what of the harness-
   redesign doc + ext-repo doc was NOT turned into J files and why.
4. NUMBERING DECISION FIRMED: tracks 12-20 are taken by I; if J used tracks
   it would start at 21. But J is small (6-10 items) - SKIP track machinery,
   plain J-1..J-N + waves. Precedent: F (13 items, no tracks) + G (10 items,
   no tracks) both skip tracks. J mirrors F/G structure.
5. SCHEMA BASELINE CONFLICT RESOLVED: I-plan (2026-06-10) says 221/532;
   H Track 7 (2026-06-05) said 224/501. 221/532 is NEWER per the index header.
   Still grep the live consts at J-authoring time IF any J item adds SSB
   variants - though harness-side items likely add ZERO (state that
   explicitly per R9/W-6 reasoning).

================================================================================
FILE 10/14: docs/thefinalglance/PLAN-FOLDER-TEMPLATE.md (265 lines, read fully)
================================================================================

WHAT IT IS: the canonical authoring spec for any successor plan folder,
grounded 2026-06-10 against the live H-plan folder. BINDING for J.

DIRECT REQUIREMENTS EXTRACTED FOR J:
1. MANDATORY CONTENTS: J-0_INDEX.md (doctrine + master index, 17 sections),
   J-SUMMARY_INDEX.md (per-session entries, 6 sections), J-N_<snake_slug>.md
   one per session, _grounding/J-<TOPIC>-GROUNDING-MAP.md per track authored
   BEFORE detail files. N sequential, index is the allocator.
2. FILE-COUNT PLAN vs USER CONSTRAINT (~10 files max): 6 detail files +
   J-0_INDEX + J-SUMMARY_INDEX + 1 grounding map = 9 files. FITS. One track
   ("HARNESS-REDESIGN") -> one grounding map.
3. DETAIL FILE FORM: full form preferred (Objective / effort / prereqs /
   parallel-safe / Source / Context / Existing state / Reads (mandatory) /
   Tasks (last task = decisions doc) / WIRING_CONTRACT (produces, consumes,
   reachable_from, stub_ban, frontend_pair-or-justified-N/A,
   observable_effect with trigger+expect+negative arm) / DETERMINISTIC_CHECK
   bash block / ACCEPTANCE checkboxes / done-gate clause-by-clause / tests /
   files touched / citations / open questions / Next / literal "End of J-N."
   terminator). Compressed form allowed for steal/research tracks if all 5
   contract fields present - J items ARE redesign/steal-shaped, so the
   compressed form is legitimate, but I will keep the full skeleton where it
   adds enforcement value and compress prose.
4. NON-NEGOTIABLES THAT BITE J SPECIFICALLY:
   R-a ground every anchor (grep-confirmed or labelled PLAN-asserted),
   R-b compose-never-restate, R-c live-baseline-never-hardcoded,
   R-d vertical slices (producer+consumer+smoke in one unit),
   R-e fail-closed observable_effect with a negative arm,
   R-f kill-switch-not-enable-flag, R-g honest residue, R-h token discipline,
   R-i update the allocator, R-j deltas are earned (default: no new family).
5. HARNESS-SIDE ADAPTATION REQUIRED (deviation to declare in J-0_INDEX):
   wiring_contract_check.py --run-trigger and SSB rows are .elai_cc runtime
   machinery. For J items whose deliverables are .claude/ harness artifacts,
   the observable_effect off-agent artifact is instead: hook-executions.log
   rows / workflow/sessions/{id}/ artifacts / tool outputs with exit codes /
   git-diff-able file states. The J doctrine must define this mapping ONCE in
   J-0_INDEX (the harness analog of expect_ssb) so detail files can cite it.
   Items with BOTH harness + .elai_cc legs keep the standard SSB contract for
   the runtime leg.

================================================================================
FILE 11/14: workflow/reasoning/harness-redesign-recommendations-2026-06-10.md
(103 lines, read fully) - THE SOURCE DOC
================================================================================

WHAT IT IS: 2026-06-10 recommendations from the 7 user ideas (U1-U7), grounded
in a companion per-file reading log (harness-redesign-reading-2026-06-10.md,
[eN] cites) + vlm-vision-research-summary.md + Screenshot_42.png. Section A =
4 ideas already designed (execute H-35/H-49/I-52, H-50/H-51/G-2, H-1..9/H-34/
B-28/C-35/I-10, I-5/I-6/I-11/I-14, C-13/B-19/E-30/I-35/I-46). Section B = the
SIX genuinely new items. Section C = sequencing. Section D = honesty notes.

THE SIX ITEMS (J plan core), with my first-pass classification:
N-1 PERCEPTION GATEWAY (U7): 4-leg image-ingestion pipeline (preprocess /
    hybrid composer / 2-call perception scaffold / VLM router) + harness /see
    skill. DUAL-LEG (mostly .elai_cc + thin harness skill). Largest item.
    Doc already proposes "J-1 or H-55+". Composes B-24/E-36/E-16/H-13.
N-2 ELAI-AS-MCP-SERVER (U1): expose task/session/fsm/rra/intervention verbs as
    a long-lived MCP server. .ELAI_CC-side. Security preconditions: C-42
    authz + E-24 pin + G-4 re-pin + E-30/G-3 gates. Reuses B-25 capability
    list minus Empire transport. Novelty verified by the doc (B-25 deferred,
    C-39 is the opposite direction, I-40 wraps OTHERS).
N-3 HARNESS UNDO + PER-SESSION FILE BACKUPS (U3 harness half): PreToolUse
    pre-image backup (content-addressed, deduped) + .claude/tools/undo.py
    (list/diff/restore, RV-1-style divergence refusal) + last-good prune.
    HARNESS-side. Mirrors H-49 semantics without the CAS machinery.
N-4 HARNESS HANDOFF FILE LEDGER (U5 rest): machine-derived FILES TOUCHED
    ledger from PostToolUse change-records via .claude/tools/handoff_ledger.py
    + handoff-protocol.md template extension. HARNESS-side. Mirrors H-51's
    journal-derived DECISIONS/FILES discipline.
N-5 LITEGAME MCP + HOOK PORT (U5 first clause): port local-rag MCP + N-3 hook/
    tool + N-4 ledger + F-1-style compound PreToolUse to D:\.liteGAME_workflow.
    CROSS-ROOT. Doc itself says "a half-day port, not a plan" + caveat: verify
    the 7 unverified liteGAME compat files first.
N-6 INCREMENTAL-EVAL SKILL (U6): formalize the read-one/write-verdict/reset
    pattern as .claude/skills/incremental-eval. HARNESS-side, zero code.
    "this very session is the validation run" - and indeed THIS session is
    running exactly that protocol.

SEQUENCING FROM SECTION C (J must honor): N-6 + N-4 first (cheapest), N-3
second, N-1 at slot 7 (legs 1+3 first), N-2 at slot 8 (AFTER C-42/G-4
confirmed live), N-5 at slot 9 (after N-3/N-4 stabilize).

HONESTY NOTES THAT BIND J (Section D): plan statuses in B..E summaries STALE -
check FINISH/ + git log before citing status; combined token numbers
UNGROUNDED until measured (HD-1); U7's 30-60%/40%/-78% figures are from the
research summary's sources NOT elai measurements - N-1's done-gate must
produce ELAI-measured deltas (ID-5); all six follow W-1/W-2/GD-2/GD-4.

GROUNDING NOTE FOR J AUTHORING: the [eN] entries live in the companion
harness-redesign-reading-2026-06-10.md; J files cite the recommendations doc
+ companion as Source rather than re-deriving. file:line anchors quoted in
the doc (file_ops.rs:272/300, task_runner.rs:7871, runtime_builder.rs:306,
conversation.rs:22, task_runner.rs:3898) are PLAN-asserted for J purposes;
executors re-grep per W-7/R-a.

================================================================================
FILE 12/14: workflow/reasoning/ext-repo-steal-recommendations-2026-06-10.md
(127 lines, read fully)
================================================================================

WHAT IT IS: rev-2 steal verdicts over 7 external repos (cloned under
D:\.liteGAME_workflow\workflow\external-skills\repos\). DIFFERENT repo set
from I-Track-20 (lazycodex/gajae/roach-pi) - no I-45..I-54 collision.
Tier 1 direct steals: skillspector (NVIDIA, the missing whole-dir skill
scanner; static-only --no-llm; resolves skill-import-policy.md item 1's
unresolved-scanner gap + F-1 Tier-2 product reimplementation source) and
impeccable (41-rule deterministic design lint; dashboard-smoker checklist +
H-14 prior-art). Tier 2 prior-art pointers: codegraph (B-29/B-30/C-30/H-10),
headroom (I-14/I-40/I-12 - note headroom/cli/wrap.py is the exact `elai wrap`
deployment shape I-40 ships), Understand-Anything (2 canvas-mode concepts for
H-12). Declines recorded: agentmemory, Agent-Reach.

J-PLAN RELEVANCE ANALYSIS:
1. NO COLLISION WITH SECTION B SIX: none of the six items duplicates a steal;
   conversely, the steals all have EXISTING homes (policy amendment, F-1
   follow-up, plan pointer lines, smoker checklist) - they are chores +
   references, not new plan sessions... with ONE exception worth weighing:
   the doc's SUGGESTED NEXT ACTIONS (skillspector smoke + policy amendment,
   impeccable verify + smoker checklist add, ~9 one-line pointer insertions
   into consuming plan files) are pending-user-yes harness chores totalling
   <2h. OPTION: bundle them as one small J item (J-7 "ext-steal intake
   closeout") so the 2026-06-10 docs are FULLY discharged by J; keeps file
   count at exactly 10. DECIDE at the design step.
2. N-5 SYNERGY: the repos are cloned in the liteGAME root; skillspector is
   ALSO the scanner the skill-import-policy needs in ANY root the harness is
   ported to. The N-5 (liteGAME port) J file should name skillspector
   --no-llm as the import gate for ported skills.
3. STEAL METHOD discipline (binding for J files referencing external source):
   reimplementation-from-source, never summary-only (W-3 violation);
   license+attribution noted; idea-not-dep default (SLC-3); the harness MAY
   run a tool directly where noted (skillspector/impeccable CLI use).
4. N-3 GROUNDING BONUS: headroom/cache/compression_store.py (reversible
   originals store) is prior-art for N-3's content-addressed backup store -
   add as a read-if-useful pointer, not a dependency.

================================================================================
FILE 13/14: workflow/handoff/wire-remediation.md (231 lines, read fully)
================================================================================

WHAT IT IS: the topic-keyed handoff for the wire-dark remediation effort
(5 sessions, 2026-06-05..06-07b). CURRENT STATUS: wiring BEDROCK reached
("zero clean wiring flips remain"); verified static tally D 16w/11d/3empty,
E 1w/6d/48partial (map-stale inflated), F 9w/3d/2p, G 8w/2d; remaining
backlog is exhaustively GPU-bench / new-dep / structural-feature-eng /
intentional-defer. Huge UNCOMMITTED mixed tree. H-32..H-52 was being driven
live on 2026-06-07 by a concurrent session.

KEY TAKEAWAYS FOR J PLAN:
1. SCHEMA CONSTANT CHURN PROVEN: the handoff itself logs 234/511 -> 236/513 ->
   240/517 -> 209/520 -> 522 live -> I-plan says 221/532. CONCLUSION for J:
   never quote a live number except as "LIVE x/y at <date> (file:line),
   re-read at edit (W-9)". For J authoring I will grep ssb.rs SCHEMA_VERSION +
   action_type_lock TOTAL_VARIANTS once, date-stamp it, and mark re-read.
2. HARNESS BUG THE J PLAN MUST NOT IGNORE (N-3 adjacency): "the stale-F:-
   drive-hook bug is REAL - your FSM gate is effectively a no-op for F:
   paths" (pre_tool_check.py _SRC_PREFIXES lacks the F: prefixes; an agent
   auto-fixed it and the controller REVERTED it as an R8 violation pending
   the user's deliberate call). The N-3 J file (harness backups) lives in the
   same hook family - it must record this as a NAMED open question (fix is
   user-deliberate, not auto-included), and N-3's own hook must not repeat
   the R8 mistake (it is a NEW hook authored out-of-band, not an edit to the
   frozen gate).
3. EXECUTION-CONTEXT NOTES J-0_INDEX should carry: worker agents = sonnet;
   serial-per-unit on shared files; SSB 6-part variant discipline (partial
   add = 91-error breakage precedent); cargo check --workspace not -p
   (false-green cache); orchestrator integration tests do not link (ort rlib)
   so smokes go in-crate #[cfg(test)].
4. STATUS-CITATION RULE for J files: H-32..H-52 and the D/E/F/G statuses are
   MOVING - J files cite "per wire-remediation handoff 2026-06-07b" + insist
   the executor re-check plan-status fragments + git log at execution
   (Section D of the redesign doc says the same).
5. N-2 PRECONDITION STATUS CHECK: N-2 requires C-42 authz + G-4 re-pin
   confirmed LIVE. Static tally says G "8 wired / 2 dark" - G-4's status must
   be re-verified at execution; the J-2 file lists this as a blocking
   pre-requisite check, not an assumption.

================================================================================
FILE 14/14: workflow/sessions/s-plan-audit-2026-06-04/FINISH/_MASTER-INDEX.md
(292 lines, read fully)
================================================================================

WHAT IT IS: the 213-plan completion+wiring audit rollup (2026-06-04 baseline
60/213=28% wired) with three superseding update layers: 06-07 feature-
finishing NOW (~94/213 optimistic), 06-07b verified static re-audit (stricter:
D trust-runtime still dark-by-default under R7/R9), and the 06-06 definitive
"no clean wiring flips remain" finding. 4 themes: allowlist (done), wire
callers (exhausted), benchmarks (GPU), not-started (9 C harnesses).

KEY TAKEAWAYS FOR J PLAN:
1. STATUS GROUND TRUTH for J files = plan-status/<L>/<id>.md fragments +
   this index's latest layer, NOT any summary index. J files citing a
   prior-plan status say "per FINISH 06-07b static re-audit" + re-check rule.
2. R7/R9 HONESTY STANDARD J inherits: "None->Some under default-OFF flag" is
   STILL wire-dark until a non-test caller fires + allowlist row exists. For
   J's harness items the analog: a tool/skill/hook that exists but is never
   invoked by a live session = wire-dark; the J done-gate must show a real
   invocation artifact (JD-1).
3. WAVE-E GPU note: J-1's heavy legs (VLM router, perception measurements on
   real fixtures) belong to the same RunPod batch discipline (pod_stop after).
4. UNTRACKED-GAP PRECEDENT (phase-3L): the audit's one genuinely untracked
   foundation gap got folded into a WAVE, not a new letter. Reinforces:
   J stays SMALL - only the six+1 genuinely-new items; everything else points.

================================================================================
SECTION B DEEP ANALYSIS - THE SIX ITEMS (the user's question)
================================================================================

Per-item reasoning: novelty re-verification against all 14 files, composition
surface, risks, scope corrections, and the J mapping. GRC decision recorded
via mcp_think topic=j-plan (call 1/10).

--- N-1 PERCEPTION GATEWAY -> J-1 ---
NOVELTY RE-VERIFIED: B-24 validates OUTPUT media (Tier1 deterministic + Tier2
VLM advisory); E-16 adds AXTree/anti-sycophancy to that validator; E-36
ingests DOCUMENTS into grounding; H-13 captures canvas screenshots INTO
media_validator. Nothing in B..I conditions INBOUND images before a model
reasons over them, and nothing implements the 2-call perception scaffold.
GENUINELY NEW - confirmed.
COMPOSITION: gateway sits in front of every image-bearing dispatch; routes
documents to the E-36 leg; UI-hybrid leg independently confirms E-14 AXTree
doctrine; pixel-diff hard-limit routing already encoded as B-24 Tier-1 veto;
H-13 canvas_screenshot becomes a consumer. Heavy CV/VLM legs live on the
python sidecar (HARDWARE-HEAVY rule: remote 3090, flag before boot).
RISKS/CORRECTIONS: (a) all figures (30-60% recovery, ~0->40% scaffold, -78%
router cost) are EXTERNAL - the done-gate must produce ELAI-measured deltas
(ID-5/HD-1); (b) 2-call scaffold doubles call count on visual tasks -
mitigated by leg-4 router + image prompt caching, and scaffold fires only on
image-bearing turns; (c) harness /see leg needs pillow (NOT stdlib) - skill
must verify-before-use and degrade to pass-through (JC-4).
SCOPE: legs 1+3 first (deterministic preprocess + scaffold - model-free/cheap)
then legs 2/4. SSB: +1 PerceptionScaffoldEvent (baseline+1 from LIVE 221/532).
Effort L (legs 1+3 = M). WAVE 3.

--- N-2 ELAI-AS-MCP-SERVER -> J-2 ---
NOVELTY RE-VERIFIED: B-25 ElaiNeuron sketched 4 capabilities but is DEFERRED
on Empire (C-34 external blocker); C-39 built the INVERSE direction (CLI as
MCP client; mcp_client.rs/mcp_glue.rs exist in the live tree per git status);
I-40 wraps OTHER agents' traffic; H-34/C-35 defer external schemas INTO elai.
"elai exposes its own control plane" has no lane. GENUINELY NEW - confirmed.
COMPOSITION: verbs ride existing primitives ONLY: task_submit/status
(task_runner), fsm_get_state (FSM), session fork/rewind (A-1 chain H-35/H-49/
I-52), plan_status (I-39), rra_show (G-2), intervention_respond (B-6 SSB
pair). Capability list = B-25's minus Empire transport. Journal stays the
single source of truth: prefer ZERO new SSB variants (reuse C-39 cli_mcp_*
rows if shipped; else +1) - net 0..+1.
RISKS: this is an RCE surface by construction. The four preconditions (C-42
authz DENY-ALL, E-24 blake3 pin, G-4 rug-pull re-pin, E-30/G-3 destructive
gates) are a BLOCKING PRE_GATE with live status re-verification (FINISH
06-07b shows G 8 wired/2 dark - G-4 specifically must be re-confirmed).
Transport starts stdio-local-only (no TCP) to keep the attack surface at
"same user, same box". Concurrent-session safety: every verb passes the same
FSM gates an interactive user would; DENY-ALL on unlabeled verbs.
Effort L. WAVE 4 (deliberately late, after the security spine re-check).

--- N-3 HARNESS UNDO + PER-SESSION FILE BACKUPS -> J-3 ---
NOVELTY RE-VERIFIED: grep/glob this session: .claude/tools/ has 26 tools, NO
undo.py; hooks = rag/hooks/pre_tool_check.py (+pre_compact etc. per
CLAUDE.md). H-49/I-52/E-25 checkpoints are all .elai_cc-side. git is no
substitute (root gitignores everything but .elai_cc). GENUINELY NEW for the
harness - confirmed.
MECHANISM: NEW PreToolUse hook file (registered in settings, NEVER an edit to
the frozen pre_tool_check.py - JD-4/R8) copies the pre-image of Edit/Write/
NotebookEdit targets to workflow/sessions/{id}/backups/{hash-prefix}/
{relpath}@{seq} (content-addressed dedup) + .claude/tools/undo.py
(list/diff/restore/prune; RV-1-style divergence refusal: on-disk != recorded
post-image -> refuse + show diff). Last-good = last version preceding an
audit PASS read from the session state PostToolUse already records;
deliberately last-PASS-wins, NOT D-5 bandit machinery.
CORRECTIONS vs the redesign doc (JC-1/JC-2): zstd is NOT stdlib python ->
gzip above size threshold; blake3 is NOT stdlib -> hashlib.blake2b keyed
(stdlib) with an explicit anti-stale deviation note + upgrade path when the
queued py-blake3 dep (E-8 list) is ever approved. House convention "tools are
stdlib-only" wins; the blake3 hard rule is a Rust-crate rule.
RISKS: hook latency per write (bounded: size threshold + dedup short-circuit
on identical hash); backup growth (prune verb + last-good policy); the
stale-F:-hook bug (FSM gate no-op for F: paths) is ADJACENT - named open
question, fix stays user-deliberate. Effort S-M. WAVE 2.

--- N-4 HARNESS HANDOFF FILE LEDGER -> J-4 ---
NOVELTY RE-VERIFIED: handoff-protocol.md's FILES TOUCHED is narrator-
recalled; PostToolUse change-recording to session state EXISTS (proven by the
wire-remediation incident editing state.json changes[]); no handoff_ledger.py
exists. GENUINELY NEW - confirmed. The G-5 DSR principle (numbers from the
journal, not the narrator) and H-51 (elai-side digest must match journal
rows) are the doctrine sources.
MECHANISM: .claude/tools/handoff_ledger.py derives per-file rows (path, edit
count, last-good backup seq from J-3, audit verdict if any) from the session
state/hook log; handoff-protocol.md FILE TEMPLATE gains a LEDGER line in the
FILES TOUCHED section requiring machine-derived output when the tooling is
present. Degrades honestly: no session state -> emits NOTHING + a visible
"ledger unavailable" line (R1 analog: missing evidence is named, not faked).
RISKS: state schema variance across sessions/roots (tool reads both
state.json and hook-executions.log shapes; fail-visible). Effort S. WAVE 1.

--- N-5 LITEGAME MCP + HOOK PORT -> J-5 ---
NATURE CORRECTION: the redesign doc itself says "a half-day port, not a
plan". J-5 is therefore a PORT CHECKLIST session (compressed form), not a
build: (0) verify the 7 unverified liteGAME compat files FIRST (memory:
project_litegame_orchestrate_compat), (1) local-rag MCP registration (safe
multi-root per the 2026-06-05 paths.py resolver fix), (2) J-3 hook+tool,
(3) J-4 ledger, (4) F-1-style compound PreToolUse shape rather than stacked
hooks, (5) skill imports in that root gated by skillspector --no-llm (J-7's
scanner resolution applies in every root).
RISKS: cross-root drift (the D:-ghost class of bug); port=copy+adapt, never
shared mutable config. Effort S-M (half-day). WAVE 5 (after J-3/J-4
stabilize here).

--- N-6 INCREMENTAL-EVAL SKILL -> J-6 ---
NOVELTY RE-VERIFIED: 11 skills globbed; none encodes the read-one/
write-verdict/reset protocol. The user re-states it as a MAJOR RULE every
time (including THIS session, which is itself the validation run).
GENUINELY NEW - confirmed. Zero code.
MECHANISM: .claude/skills/incremental-eval/SKILL.md with trigger (N>=3
artifacts to read/evaluate/compare), protocol (ONE artifact -> bounded
anchored verdict block appended to ONE topic-keyed workflow/reasoning file ->
next; per-cycle context = {task spec, verdict-file index, next artifact};
head-only re-reads), anti-patterns (batch-read, unbounded verdicts,
full-file re-reads). E-43 convention compliance (JC-5): frontmatter passes
agentskills_validator.py + ships eval/positive.jsonl + eval/negative.jsonl
like the existing 11.
COMPOSITION: token-discipline + handoff-protocol one-file rules; I-5/I-6/
I-11 are the runtime siblings (cite, do not restate). Effort S. WAVE 1.

--- THE +1: J-7 EXT-STEAL INTAKE CLOSEOUT ---
WHY ADDED (my synthesis, not Section B): file 12's SUGGESTED NEXT ACTIONS are
"pending your yes" chores (<2h) with no owner: skillspector smoke +
skill-import-policy.md item-1 amendment (scanner RESOLVED; rug-pull stub
non-coverage stays G-4's), impeccable npm-verify + trial + dashboard-smoker
checklist line, ~9 GD-1 pointer lines into consuming plan files (F-1, B-29/
B-30/E-22, I-12/I-14/I-40, H-12/H-14). Folding them into J gives them a
wiring contract + done-gate and fully discharges BOTH 2026-06-10 docs in one
plan. Keeps J at exactly 10 files. The J-0_INDEX marks J-7's source as the
ext-repo doc, distinct from Section B.

--- CROSS-CUTTING DECISIONS (recorded in mcp_think call 1) ---
D1 STRUCTURE: 10 files = J-0_INDEX + J-SUMMARY_INDEX + _grounding/
   J-HARNESS-GROUNDING-MAP + J-1..J-7. Single track, no track numbering
   (F/G precedent; tracks 12-20 are I's).
D2 WAVES (Section C order honored): W1 J-6+J-4; W2 J-3+J-7; W3 J-1 (legs
   1+3); W4 J-2 (post security-precondition re-check); W5 J-5.
D3 DOCTRINE: inherit W-1..10 + GD-1/2/4/5 (+HD-1/HD-2 where relevant) by
   reference; ADD JD-1..JD-4 (earned per R-j - the harness has no SSB/
   wiring_contract_check machinery, so the harness-side evidence mapping
   must be defined once):
   JD-1 harness done-gate = a REAL session event produced an off-agent
        artifact (hook log row / backup file / ledger output / skill
        invocation in transcript); "the file exists" is NOT done; missing
        artifact = FAIL.
   JD-2 harness evidence is derived, never recalled: ledgers/backups come
        from hook-recorded state, not the narrator (G-5 DSR analog).
   JD-3 new harness gates ship observe-only first; nothing blocks until
        friction/FP is observed across >=3 real sessions (GD-4 analog).
        J-3 backups and J-4 ledger are non-blocking by construction.
   JD-4 frozen-gate respect (R8 analog): J items NEVER edit existing
        registered hooks; new hooks are new files; the stale-F:-hook fix
        stays a user-deliberate act (named open question in J-3).
D4 SSB: LIVE baseline 221/532 grep-confirmed this session (ssb.rs:2981 /
   action_type_lock.rs:601, 2026-06-10). J-1 +1; J-2 0..+1 prefer-reuse;
   J-3..J-7 zero. Re-read at edit (W-9).
D5 SECTION A IS NOT J: A-1..A-5 are execution pointers to existing plan
   files; J-0_INDEX carries them as a NOT-J-SESSIONS section so they are
   never re-authored (GD-1).

End of pre-reading log. J plan authored next at docs/thefinalglance/J-plan/.

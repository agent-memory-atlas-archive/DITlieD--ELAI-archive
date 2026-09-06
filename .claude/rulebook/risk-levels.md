Risk levels per decomposition level (HARD RULE)

Loaded by: idea-intake skill, planning-workflow skill, designer agent.

Classifies each decomposition-tree node by blast radius so workflow decides what auto-advances vs what blocks for human approval.


THE TABLE

Decomposition level | Default risk | Auto-advance ? | Notes
---------------------|--------------|----------------|------
BabyStep             | LOW          | YES            | Single-deliverable, single-file scope. Auditor decision is sufficient.
Step                 | LOW          | YES            | Multiple baby steps under one logical group. Same scope class as a baby step.
Phase                | MEDIUM       | NO (confirm)   | Touches multiple files, possibly cross-module. Requires user confirmation before starting.
Concept              | HIGH         | NO (re-direct) | Cross-system, architectural. Requires user re-direction or approval to start, again to finish.
Idea                 | HIGH         | NO (always)    | The whole tree. Always asks human at intake and at completion.


WHAT "AUTO-ADVANCE" MEANS

INTERACTIVE mode: auto-advance = no human confirmation needed. TreeNavigator transitions to next node when exit criteria met.

AUTONOMOUS mode (when added later): auto-advance default for LOW. MEDIUM still requires human checkpoint at boundary. HIGH always pauses for human.


PROMOTION RULES (when default risk is too low)

A baby step upgrades LOW > MEDIUM if any of:
- touches > 2 files (default for baby steps is 1-2)
- exceeds 200 LOC of new code
- touches a file in a HIGH-risk system (auth, payments, persistence migrations, anything in App Store compliance scope per .claude/rules/app-store-compliance.md)
- parent step has already redecomposed once (baby step came from BubbleUp redecomposition, area is fragile)

A step upgrades LOW > MEDIUM if:
- > 4 baby steps (suggests it should be a phase)
- any baby step upgraded to MEDIUM under rules above

A phase upgrades MEDIUM > HIGH if:
- spans more than one concept (decomposition error, should split)
- touches > 8 files
- modifies a system in app-store-compliance (privacy manifest, ATT, IAP, push)


DEMOTION RULES (rarer)

Phase demotes to LOW if purely test scaffolding (only writes test files, no production code). Mark in decomposition tree as "Risk: LOW (demoted because test-scaffolding-only)."

Concept demotes to MEDIUM only with user approval. Default is always HIGH.


HUMAN APPROVAL FORMS

LOW > no approval needed, just exit criteria met.

MEDIUM > "Phase {id} '{title}' is about to start. Touches: [files]. Estimated baby steps: {N}. Risk: MEDIUM ({reason}). Proceed ? [y / scope-down / abort]"
   - y: continue to first baby step of the phase
   - scope-down: open the phase for editing, user can split or trim
   - abort: skip this phase, mark blocked, ask if to redecompose or escalate

HIGH > "Concept {id} '{title}' is about to start. This is a HIGH-risk node. {reasoning}. Confirm direction ? [y / discuss / abort]"
   - y: continue
   - discuss: open dialogue about scope, alternatives, sequencing
   - abort: drop the concept

At idea level the human approval is the prompt-engineer hand-back: user reviews the transformed prompt + initial concept tree before any planning begins.


INTERACTION WITH BUBBLE-UP

When a BubbleUp event escalates baby step > step level, step's risk is reassessed:
- step was LOW > becomes MEDIUM (redecompose fired, area harder than expected)
- step was already MEDIUM > becomes HIGH and human approval required for the redecompose

Prevents silent re-spirals at increasingly broken decompositions.


INTERACTION WITH UI_CHECK

UI changes inherit risk from screen complexity, not from being "UI."
- single text-color tweak on existing screen: LOW
- new screen: MEDIUM at minimum
- navigation overhaul (changes how multiple screens connect): HIGH

UI_CHECK runs at any risk level, visual acceptance bar tightens at higher levels:
- LOW: 1 screenshot, default device, default appearance
- MEDIUM: 2-3 screenshots covering obvious variants (light/dark or default/large type)
- HIGH: full variant set (light + dark, default + large type, baseline + ProMotion device if frame-rate-sensitive)


INTERACTION WITH APP STORE COMPLIANCE

Any baby step touching items from .claude/rules/app-store-compliance.md (privacy manifest, ATT, StoreKit, push entitlements, account deletion, China ICP) auto-upgrades to MEDIUM regardless of file count. App Review rejection cost is non-trivial to recover from.

Any concept touching 3+ compliance items auto-upgrades to HIGH regardless of other heuristics.

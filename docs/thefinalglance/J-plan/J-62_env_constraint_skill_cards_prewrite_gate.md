# J-62: environment-constraint skill cards + pre-write mandatory-read gate — the frontend-design SKILL.md analog ELAI lacks (the missing/React-mismatched dashboardRules.md authored correctly as a Svelte5+Tauri-v2 env-card), plus a PreToolUse read-before-write gate that COMPOSES J-52 LEG 3's path-fingerprint injector

STATUS: PLANNED (WAVE FABLE-STEAL-1; nature EXTENSION over the WIRED context_budget Rules slot + hook_dispatcher PreToolUse + vibegate F-1 + the E-43 skill system; COMPOSES J-52 LEG 3, restates none)

- Track J (Fable-5 system-prompt steal) | lever SMARTER-ORCHESTRATION (capability-multiplier-not-weights) + CANT-MISBUILD | grounding STRONG on the ELAI seams (code-grounded this session); ILLUSTRATIVE on the Fable source (leaked prompt, see Citations) | nature EXTENSION | effort M | WAVE FABLE-STEAL-1 | parallel-safe with J-63 (disjoint files) | composes J-52 LEG 3 + conditional-routing.md + context_budget Rules slot + vibegate F-1 + hook_dispatcher PreToolUse + E-43 skill system + design-taste.md + visual-ui-loop.md
- Premise: a spec-eval of the Fable 5 chat system prompt found that a large fraction of its "one-shot vibe coding" is HARNESS, not weights. The single load-bearing mechanism: "Reading the relevant SKILL.md is a required first step before writing any code, creating any file, or running any other computer tool" (Fable prompt computer_use/skills + additional_skills_reminder), because skills "encode environment-specific constraints (available libraries, rendering quirks, output paths) that aren't in Claude's training data." The frontend-design skill is what makes Fable's UI output match the sandbox first-try. ELAI has the TASTE half (design-taste.md = what good looks like) and the LOOP half (visual-ui-loop.md = screenshot-compare critique) but is MISSING the env-constraint half: the component/state/API/build doctrine for dashboard-v2. Worse, the grounded find this session — dashboardRules.md DOES NOT EXIST anywhere in .claude (find + rg both empty) yet visual-ui-loop.md AND design-taste.md both reference it ("Pairs with ... dashboardRules.md", "Cross-ref dashboardRules.md (components/state)") AND flag that where it is described it is "written in React terms" while the real stack is Svelte 5 + Vite + Tauri v2. So two shipped rule files assume an env-card that is absent and framework-mismatched. J-62 authors that env-card correctly, generalizes it to a small env-card REGISTRY, and adds the one net-new mechanism sliver Fable has that ELAI does not: a PRE-write mandatory-read gate (read-before-you-touch-the-file), distinct from J-52 LEG 3's POST-edit injection.

## Steal/build (2 legs)

LEG 1 — ENVIRONMENT-CONSTRAINT SKILL CARDS [S1, the high-leverage piece, BUILD]. Source: Fable available_skills (frontend-design at /mnt/skills/public/frontend-design/SKILL.md = "design tokens and styling constraints for this environment"; the React-library/r128-three.js/Tailwind-core-only constraints in artifact_usage_criteria). ELAI port: author a tight, single, must-read-before-you-touch-the-file env-card per build surface, the missing component/state/API/build doctrine the other two rule files already assume. The CANONICAL first card is the dashboard-v2 card (the frontend-design analog, framework-CORRECT this time): Svelte 5 runes ($state/$props/$derived/$effect, no legacy stores-by-default), Vite build, Tauri v2 IPC (#[tauri::command] + ts-rs binding + invoke() symmetry — the cross_lang_diff.py drift pair), the real component inventory (WindowHost/GraphWindow/the panel family/tokens.css — assemble-from-existing not green-field), the design-token surface (oklch(), prefers-reduced-motion, @font-face — pointer to design-taste.md, do NOT restate it), the capture/verify harness (the CDP screenshots + ax-scaffold + visual-ui-loop, pointer not restate), and the dashboard-handoff skill cross_lang_diff requirement. The card is the environment-specific layer that is NOT in any model's training data, exactly the Fable thesis. The REGISTRY pattern: a card per surface class — dashboard-v2 (this one, first), the python sidecar lane (server.py contract / endpoint+Rust-client pair / the FORCE_SDPA/4bit boot facts), the Rust-crate lane (the CLAUDE.md anti-stale stack: blake3/rand0.9/tracing/no-unwrap-in-prod/native-async-trait) — authored ONLY as measured pain arises (GD-4, not all at once).

LEG 2 — PRE-WRITE MANDATORY-READ GATE [S2, the net-new mechanism sliver, EXTENSION composing J-52]. Source: Fable "required FIRST step before writing any code, creating any file" — a BEFORE-write blocking discipline. ELAI today: conditional-routing.md is a by-CONVENTION main-LLM rule-load ("load only when true"); J-52 LEG 3 mechanizes it as a POST-edit (PostToolUse) path-fingerprint injection into the Rules slot. The net-new is the PRE-write (PreToolUse) BLOCKING-read timing: on the FIRST Write/Edit of a session to a file matching an env-card's path fingerprint, the gate ensures the matching env-card is resident in the CONTEXT BUDGET ALLOCATOR Rules/Skill slot BEFORE the write is accepted; if absent, inject-then-proceed (default) or, for a card marked mandatory, a single bounded re-dispatch so the card is in-context for the generation, never a hard-kill. CRITICAL COMPOSITION (GD-1, P0 if violated): J-62 does NOT re-implement the path->card fingerprint resolver or the Rules-slot dedup — those are J-52 LEG 3 (path_rule_inject.rs::PathRuleInjector). J-62 reuses PathRuleInjector and adds (a) the env-card SOURCE class to its priority-ordered source list (.claude/skills/<surface>-env/ cards alongside .claude/rules), and (b) the PreToolUse read-before-write lifecycle wrapper (a different hook event than J-52's PostToolUse). If the action-type-lock review rules that J-52 LEG 3's PostToolUse injection is sufficient and the pre-write timing is not worth a distinct seam, LEG 2 degrades to "register the env-cards as a J-52 source" and J-62 stands on LEG 1 (the cards) alone — stated honestly as the fallback.

## HARD boundaries (negative space)
- LEG 2 NEVER reduplicates J-52 LEG 3's PathRuleInjector / fingerprint map / Rules-slot dedup (GD-1 P0). It reuses it and adds the source class + the pre-write lifecycle. A J-62 that re-implements path-keyed injection is a BLOCK.
- The env-card NEVER restates design-taste.md (look/taste) or visual-ui-loop.md (the critique loop) — it POINTERS them and owns only the component/state/API/build doctrine (the missing dashboardRules.md slot). Duplicating taste content is a token-waste bug (token-discipline) and a GD-1 violation.
- NO hard-kill on a missing card: default is inject-then-proceed; a mandatory card gets ONE bounded re-dispatch, never a crash (W-2 graceful).
- NO new ELAI_*_ENABLE: the gate is default-ON behind a kill-switch ELAI_ENV_CARD_GATE_DISABLE (W-2/R-f); a disabled gate LOGS the disable, never a silent no-op.
- NO blocking before the friction rate is bound: the gate ships OBSERVE-ONLY first (JD-3/GD-4) — it injects the card and emits the SSB row, but the mandatory-re-dispatch arm activates only after the inject/no-op FP rate is observed across >=3 real dashboard-v2 edit sessions.
- The env-cards are HARNESS docs (.claude/skills), framework-CORRECT (Svelte 5 + Tauri v2, never the React framing the absent dashboardRules.md carried) — a card that copies the React API is a BLOCK.

## Existing ground truth (grounded this session; PLAN-asserted anchors, executor re-greps per W-7)
- J-52 LEG 3 PLANNED — docs/thefinalglance/J-plan/J-52_returntime_perEdit_enforcement_hooks.md: path_rule_inject.rs::PathRuleInjector (edited-path fingerprint -> .claude/rules priority resolver -> Rules-slot dedup, PostToolUse). THIS is LEG 2's reuse target; J-62 extends its source list + adds the pre-write lifecycle. Sequence J-62 AFTER or WITH J-52 (LEG 2 depends on PathRuleInjector existing).
- CONTEXT BUDGET ALLOCATOR WIRED — crates/orchestrator/src/context_budget.rs (8 fixed slots incl Rules; SlotWeights; the env-card lands in the Rules/Skill slot, deduped by J-52's resolver). Do NOT re-define SlotWeights (GD-1).
- VIBEGATE F-1 WIRED — crates/orchestrator/src/vibegate.rs:1 ("F-1 VibeGate — compound evaluator on the PreToolUse path"), evaluate_query:68. The PreToolUse seam LEG 2's read-before-write wrapper rides; do NOT route through a new PreToolUse evaluator (extend the existing compound path).
- HOOK SYSTEM WIRED — crates/orchestrator/src/hook_dispatcher.rs (PreToolUse + dispatch_post_tool_use:1971). LEG 2 is a PreToolUse extension; J-52 LEG 3 is the PostToolUse sibling.
- E-43 SKILL SYSTEM WIRED-dark — skill_system_event (ELAI_SKILL_SYSTEM_ENABLE), .claude/tools/agentskills_validator.py, the 19 .claude/skills/ packages. The env-cards are NEW skill packages under this system (frontmatter-validated by agentskills_validator.py); skill-import-policy.md does NOT apply (locally authored, git-provenanced).
- design-taste.md + visual-ui-loop.md WIRED (rule files) — both reference the ABSENT dashboardRules.md and flag the React mismatch (grep-confirmed this session: find dashboardRules.md = empty; rg dashboardRules = only the two referencing files). LEG 1's card is the framework-correct replacement for that referenced-but-missing doctrine.
- dashboard-handoff skill + cross_lang_diff.py WIRED — the ts-rs Tauri<->Svelte drift gate the card must point at.

## REVISED scope (W-10 tiered)
Tier-1 (SHIP NOW): LEG 1 the dashboard-v2 env-card authored as a .claude/skills env-card package (frontmatter-validated) + the env-card REGISTRY convention (a card per surface, dashboard-v2 first); LEG 2 the PreToolUse read-before-write wrapper reusing J-52's PathRuleInjector, OBSERVE-ONLY (inject + emit, no mandatory re-dispatch yet).
Tier-2 (DEFER): the python-sidecar and Rust-crate env-cards (authored on measured pain, GD-4); the mandatory-re-dispatch arm (activates after the FP rate is bound across >=3 sessions); a learned card-relevance score (Tier-1 is the static J-52 fingerprint map). All recorded as riders.

## Integration
- NEW (harness): .claude/skills/dashboard-v2-env/SKILL.md (the env-card: component/state/API/build doctrine, framework-correct Svelte5+Tauri-v2, pointers to design-taste.md/visual-ui-loop.md, NOT a restatement) + eval/positive.jsonl + eval/negative.jsonl (E-43 per-skill eval convention) + frontmatter validated by .claude/tools/agentskills_validator.py. NEW: a one-line entry in conditional-routing.md pointing the Dashboard/frontend trigger at the env-card (the by-convention pointer LEG 2 then mechanizes).
- MODIFY (runtime, small): crates/orchestrator/src/path_rule_inject.rs (J-52) — add the .claude/skills env-card source class to the priority resolver (~15 LOC); crates/orchestrator/src/vibegate.rs OR hook_dispatcher.rs PreToolUse path — the read-before-write wrapper that calls PathRuleInjector before accepting the first matching Write/Edit (~30 LOC). NO new module unless the executor finds the wrapper does not fit the F-1 compound path (material fork -> R-4).
- SSB: +EnvCardGated (Pattern-2 single-variant; Kind: CardInjected / AlreadyResident / NoMatchingCard / MandatoryReDispatch). baseline+1; SCHEMA_VERSION COMPUTED (auto-follows SSB_ACTION_TYPES.len(), ssb.rs:1508); full 9-tuple lockstep; re-grep SCHEMA_VERSION + the variant total at edit (W-9). If the lock review prefers reuse of J-52's PathRuleInjected event with a wider Kind, collapse to that (net 0 new) — executor's call at lock time; the env-card decision must stay journal-distinct (R1) either way.
- frontend_pair: fold the env-card-injection count into the J-52 HookEnforcementPanel tile (do NOT author a second panel — GD-1); #[tauri::command] + ts-rs + cross_lang_diff.py PASS if a new field is added.

## WIRING_CONTRACT
produces_symbols:
  - .claude/skills/dashboard-v2-env/SKILL.md (the framework-correct component/state/API/build env-card) [LEG 1]
  - the env-card REGISTRY convention (.claude/skills/<surface>-env/ + conditional-routing.md pointer) [LEG 1]
  - the PreToolUse read-before-write wrapper in vibegate.rs/hook_dispatcher.rs that ensures the matching env-card is resident before the first write [LEG 2]
  - the .claude/skills env-card source class added to J-52's path_rule_inject.rs::PathRuleInjector priority resolver [LEG 2]
  - EnvCardGated SSB action_type (or the collapsed J-52 PathRuleInjected wider-Kind variant)
consumes_symbols:
  - crates/orchestrator/src/path_rule_inject.rs::PathRuleInjector (J-52 LEG 3 fingerprint resolver + Rules-slot dedup — EXTEND the source list, do NOT re-implement) — GD-1
  - crates/orchestrator/src/context_budget.rs SlotWeights Rules/Skill slot (the card lands here) — GD-1
  - crates/orchestrator/src/vibegate.rs::evaluate_query (F-1 PreToolUse compound path the wrapper rides) — GD-1
  - crates/orchestrator/src/hook_dispatcher.rs (PreToolUse dispatch) — GD-1
  - .claude/rulebook/design-taste.md + .claude/rules/visual-ui-loop.md (POINTERED by the card, never restated) — GD-1
  - .claude/tools/agentskills_validator.py (frontmatter gate for the new skill package); .claude/tools/cross_lang_diff.py (ts-rs drift the card teaches)
reachable_from:
  - the PreToolUse path on the FIRST real Write/Edit to a dashboard-v2 file (.elai_cc/crates/dashboard-v2/**) in a session -> EnvCardGated::CardInjected, the card resident in the Rules slot on the generating turn  [PRODUCTION PreToolUse seam]
  - a #[cfg(test)] caller NEVER satisfies this (R7); the SSB row + the card-resident-in-slot is the off-agent artifact
stub_ban:
  - NO re-implementing J-52's path-fingerprint injector / Rules-slot dedup (GD-1 P0 — reuse PathRuleInjector)
  - NO env-card that restates design-taste.md / visual-ui-loop.md content (pointer only; duplication = BLOCK)
  - NO React-framed dashboard card (must be Svelte 5 + Vite + Tauri v2 framework-correct)
  - NO hard-kill on a missing card (inject-then-proceed default; mandatory card = ONE bounded re-dispatch; W-2 graceful)
  - NO new ELAI_*_ENABLE (default-ON behind ELAI_ENV_CARD_GATE_DISABLE; a disabled gate LOGS, never silent no-op)
  - NO blocking arm before the FP rate is bound across >=3 real sessions (OBSERVE-ONLY first, JD-3/GD-4)
  - NO .unwrap()/.expect() on prod paths; NO blake2/sha2 (blake3 for the card fingerprint, reusing J-52's)
frontend_pair:
  fold env-card-injection counts into the J-52 HookEnforcementPanel tile (NO second panel — GD-1); #[tauri::command] + ts-rs + cross_lang_diff.py PASS if a field is added. Justified: "how often did the env-card fire / no-op" is the believe-the-number surface for whether the Fable mechanism is actually conditioning generations.
observable_effect:
  trigger: "the first real Write/Edit of a session to a .elai_cc/crates/dashboard-v2/** file with no dashboard-v2 env-card resident in the Rules slot"
  expect_ssb: "EnvCardGated::CardInjected carrying the edited path + the injected card id (or AlreadyResident on a later edit; NoMatchingCard for an unregistered surface) — byte-recomputable from the journal under ELAI_SEED (GD-2)"
  expect_artifact: "the dashboard-v2 env-card resident in the CONTEXT BUDGET Rules slot on the generating turn; the HookEnforcementPanel count incremented. NEGATIVE that must FAIL: a dashboard-v2 file generated with no card ever injected and no SSB row (gate must fire); a second edit re-injecting the card into a slot that already holds it (dedup must prevent, AlreadyResident); the gate a silent no-op when ELAI_ENV_CARD_GATE_DISABLE is set (must log the disable); a J-62 module that re-implements the J-52 fingerprint resolver (GD-1 BLOCK at audit)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-62_env_constraint_skill_cards_prewrite_gate.md --run-trigger
test -f .claude/skills/dashboard-v2-env/SKILL.md                                      # the env-card exists
python .claude/tools/agentskills_validator.py .claude/skills/dashboard-v2-env        # frontmatter valid
rg -n "Svelte 5|\$state|\$props|tauri::command|ts-rs|invoke\(" .claude/skills/dashboard-v2-env/SKILL.md  # framework-correct, not React
rg -c "design-taste|visual-ui-loop" .claude/skills/dashboard-v2-env/SKILL.md          # pointers present (>=2), not a restatement
rg -n "PathRuleInjector" .elai_cc/crates/orchestrator/src/vibegate.rs .elai_cc/crates/orchestrator/src/hook_dispatcher.rs  # reuses J-52, no re-impl
rg -n "EnvCardGated|PathRuleInjected" .elai_cc/crates/orchestrator/src/ssb.rs         # the action type (or collapsed)
rg -c "\.unwrap\(\)|\.expect\(|blake2|sha2" .elai_cc/crates/orchestrator/src/path_rule_inject.rs  # MUST be 0 on the modified lines
rg -c "ELAI_.*_ENABLE" .elai_cc/crates/orchestrator/src/vibegate.rs                   # MUST be 0 new (kill-switch only)
test -f workflow/reasoning/J-62-decisions.md
```

## ACCEPTANCE
- [ ] LEG 1: .claude/skills/dashboard-v2-env/SKILL.md authored, frontmatter-valid, framework-correct (Svelte 5 + Vite + Tauri v2, NOT React), owns component/state/API/build doctrine, POINTERS design-taste.md + visual-ui-loop.md (does not restate); eval/positive.jsonl + negative.jsonl present.
- [ ] LEG 1: the env-card REGISTRY convention documented + conditional-routing.md Dashboard/frontend trigger points at the card.
- [ ] LEG 2: the PreToolUse read-before-write wrapper REUSES J-52 PathRuleInjector (no re-impl), adds the .claude/skills source class, injects the card before the first matching write; OBSERVE-ONLY (no mandatory re-dispatch arm yet).
- [ ] gate default-ON behind ELAI_ENV_CARD_GATE_DISABLE; a disabled gate logs the disable.
- [ ] SSB +1 EnvCardGated (or collapsed into J-52 PathRuleInjected wider Kind, net 0); SCHEMA_VERSION + variant total re-grepped at edit, 9-tuple lockstep.
- [ ] frontend: counts folded into the J-52 HookEnforcementPanel (no second panel); cross_lang_diff.py PASS if a field added.
- [ ] cargo check --workspace GREEN; auditor PASS on -p elai-orchestrator only.
- [ ] No new dependency.
- [ ] depends J-52 (PathRuleInjector must exist); sequence WITH or AFTER J-52.

## WIRING DONE-GATE (clause by clause)
- W-1 missing evidence = FAIL: the EnvCardGated row is required; a gate that fired but logged nothing FAILS. W-2 kill-switch-not-enable: ELAI_ENV_CARD_GATE_DISABLE, default-ON. W-3 no opt-in on the gate. W-4 measure-the-objective: the artifact is the card resident in the slot, not the LLM claiming it read it. W-5 sound-reference: reachability is the real PreToolUse path on a real dashboard-v2 edit. W-6: HookEnforcementPanel field. W-7: anchors re-grepped (PLAN-asserted above). W-8 gate frozen: J-62 does NOT edit the wiring checkers / J-52's frozen gate; it composes them. W-9: SCHEMA re-grep at edit. W-10 integration: an independent integration smoke drives a real dashboard-v2 edit through the live PreToolUse entrypoint and asserts the card landed. GD-1: composes PathRuleInjector/context_budget/vibegate/design-taste/visual-ui-loop, restates none (the env-card is the NET-NEW doctrine, not a copy). GD-2: row recomputable under ELAI_SEED. GD-4: OBSERVE-ONLY until the inject/no-op FP rate is bound on >=3 real sessions. GD-5: honest residue below.

## Tests to add
- crates/contracts/tests/j62_env_card_gate_contract.rs (~70 LOC): (a) first dashboard-v2 edit, no card resident -> CardInjected + card in Rules slot; (b) second edit, card resident -> AlreadyResident (no double-inject, via J-52 dedup); (c) edit to an unregistered surface -> NoMatchingCard; (d) ELAI_ENV_CARD_GATE_DISABLE set -> logs disable, no-op (not silent); (e) the wrapper calls PathRuleInjector, does not re-implement it.
- a harness check (JD-1): a real dashboard-v2 edit session produces the EnvCardGated row + the card visible in the live transcript Rules slot.

## Files touched
- CREATE (harness): .claude/skills/dashboard-v2-env/SKILL.md + eval/positive.jsonl + eval/negative.jsonl; workflow/reasoning/J-62-decisions.md
- MODIFY (harness): .claude/rules/conditional-routing.md (one pointer line)
- MODIFY (runtime): crates/orchestrator/src/path_rule_inject.rs (~15, add the env-card source class — J-52 file); crates/orchestrator/src/vibegate.rs OR hook_dispatcher.rs (~30, the pre-write wrapper); crates/orchestrator/src/ssb.rs (+1 action type lockstep, or collapse into J-52); crates/orchestrator/tests/action_type_lock.rs (+1 if new); the J-52 HookEnforcementPanel tile (+1 field) + its #[tauri::command] + ts-rs binding

## Citations / Grounding
- FABLE SOURCE (ILLUSTRATIVE, UNVERIFIED PROVENANCE): C:/Users/user/Downloads/CLAUDE-FABLE-5.md — a leaked / jailbreak-reconstructed Claude Fable 5 chat system prompt (jailbreak-fable-main.zip + "fable 5 card.txt" co-located in Downloads = jailbreak-extraction signature; treat as illustrative, NOT ground truth). Load-bearing lines: computer_use/skills ("Reading the relevant SKILL.md is a required first step before writing any code"), additional_skills_reminder ("skills encode environment-specific constraints ... that aren't in training data"), available_skills (frontend-design = "design tokens and styling constraints for this environment"), artifact_usage_criteria (the per-env library/version constraints). The PATTERN (mandatory env-skill read before codegen) is INDEPENDENTLY CORROBORATED by Anthropic's public Skills feature + Agent SDK, so the design survives the leak's unverified provenance. The design below is grounded in ELAI's OWN code seams, not the leak.
- ELAI ground truth (grounded this session 2026-06-21, PLAN-asserted, re-grep W-7): J-52 path_rule_inject.rs (PathRuleInjector, the reuse target); context_budget.rs (Rules slot, SlotWeights, SCHEMA_VERSION=DDL_EPOCH(-309)+SSB_ACTION_TYPES.len() at ssb.rs:1503/1508); vibegate.rs:1/68 (F-1 PreToolUse compound evaluator); hook_dispatcher.rs (PreToolUse + dispatch_post_tool_use:1971); E-43 skill system + agentskills_validator.py + the 19 .claude/skills packages; design-taste.md + visual-ui-loop.md (both reference the ABSENT dashboardRules.md, React-mismatch flagged — find/rg confirmed dashboardRules.md does not exist).
- doctrine: GD-1 compose-not-restate (J-62's gate REUSES J-52 LEG 3, P0 if re-implemented); JD-3/GD-4 observe-only-first; W-2 kill-switch-not-enable; E-43 per-skill eval convention; skill-import-policy.md N/A (locally authored).
- spec-eval source-of-record: the Fable-steal verdict in this session (decision-ledger topic fable-orchestration-steal); reproduce in workflow/reasoning/J-62-decisions.md at execution.
- SSB: +1 (or collapsed +0 into J-52); SCHEMA_VERSION COMPUTED -> baseline ~268 / variant-total ~577 per the J-0_INDEX live re-grep; NEVER hard-code, re-grep BOTH at edit (W-9/R-c).
- CLAUDE.md: blake3 only, no .unwrap in prod, scoped -p tests, W-2 kill-switch, no new deps, ID-5 no fabricated stats.

## Open questions / risks
- Pre-write timing vs J-52 post-edit: if the action-type-lock / auditor review judges J-52 LEG 3's PostToolUse injection sufficient (the card lands one edit late, acceptable because the Rules slot re-fills each turn), LEG 2 degrades to "register env-cards as a J-52 source" and J-62 ships on LEG 1 alone. Decide at executing; a material relocation of the gate seam is a handoff fork (R-4), not a silent move.
- The dashboard-v2 env-card risks drifting from the real component inventory as dashboard-v2 evolves; mitigate by keeping the card pointer-heavy (point at the live component dir + tokens.css) rather than enumerating components inline (HD-1 measured-not-copied analog: cite the live source, do not snapshot it).
- Whether the env-card belongs under .claude/skills (E-43, frontmatter+eval) or .claude/rulebook (the design-taste sibling). Lean skills (it is a read-before-act capability with eval positives/negatives, the Fable shape); decide at executing.

## Riders (recorded, not in-scope)
- python-sidecar env-card + Rust-crate env-card (Tier-2; authored on measured pain, GD-4).
- The mandatory-re-dispatch (blocking) arm of LEG 2 (Tier-2; activates after FP rate bound on >=3 sessions).
- Learned card-relevance scoring (Tier-2; Tier-1 is the static J-52 fingerprint map).
- Auto-regenerating the env-card from the live dashboard-v2 component graph (rider on the codegraph/canvas family).

End of J-62.

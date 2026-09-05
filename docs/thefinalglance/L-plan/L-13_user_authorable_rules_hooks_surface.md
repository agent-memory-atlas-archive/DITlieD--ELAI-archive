# L-13: User-Authorable Rules & Hooks Surface — the CC-hook familiarity, done ELAI's safer (rules-as-DATA) way

STATUS: PLANNED (WAVE 2 of L-T2; M; ACTIVATION + COMPOSE)

Objective: let a user "keep working like they did in Claude Code / Codex" — add a hook / a PreToolUse rule that fires automatically — but on ELAI's safer substrate: rules are DATA evaluated by the WIRED T1/T2/T3 engine, never arbitrary shell code, and every user rule passes the SAME trust gates (FS-2). ELAI's HOOK SYSTEM [WIRED] (PreToolUse/PostToolUse/Stop/PreCompact, IPC-level, agents never see hooks — stronger than CC's user-shell hooks) and RULES ENGINE 3F [WIRED] (rules-as-DATA in SQLite, predicates AST/GREP/SCHEMA/METRIC/COMPOSITE, lifecycle PROPOSED->...->ACTIVE) already exist, but there is no USER-FACING way to register one — it is an internal seed-rule substrate. L-13 exposes it: `elai rule add` / a RulesDrawer authoring flow that writes a rules-engine DATA row OR a J-52 path-keyed PostToolUse rule, admission-gated (J-102) and lifecycle-gated (SHADOW->ACTIVE), so a user-authored rule cannot bypass VibeGate/wiring and cannot block until its FP rate is bounded. HARD BOUNDARY: a user rule is DATA + a predicate selection, never executable code (FS-2); it enters SHADOW/observe-only and promotes only on a measured FP bound (GD-4). GD-1: do NOT re-author the rules engine / hook system / J-52 — expose them.

Estimated effort: ~1 session (M). In-process + harness; no model; no RunPod.
Pre-requisites (re-grep live per W-7): RULES ENGINE 3F (rules-as-DATA SQLite, predicate kinds, lifecycle, mutable=0 immune tier); HOOK SYSTEM T1/T2/T3 (PreToolUse/PostToolUse/Stop/PreCompact dispatch); J-52 PATH-KEYED DYNAMIC RULE INJECTION (the PostToolUse path-fingerprint hook into the context_budget Rules slot); J-27 protected-blocks/webcache hook shape; J-102 config-ingress admission; VibeGate F-1; RulesDrawer.svelte + ToolsHooksDrawer.svelte (existing dashboard surfaces).
Parallel-safe with: L-11/L-12/L-14 (disjoint). Soft-dep J-52 (the path-keyed injection mechanism) + J-102 (admission); fall back to a direct rules-engine row + skillspector-class scan and record residue.
Source: output-quality-and-familiarity-surface-2026-06-27.md PART 2 (FINDING-5) + V-4/V-6 + L-OQF-GROUNDING-MAP TS-8/TS-16/TS-17/TS-18/TS-24 + NN-4 + FS-1..FS-4. Doctrine: L-0_INDEX.md (FS-1..FS-4, GD-4, TD-3, R8, mutable=0).

## Context
Why this matters: a CC migrant's muscle memory is "drop a hook in settings.json so X runs before every edit." ELAI's hooks are compiled Rust T1/T2/T3 and its rules are an internal SQLite substrate — both stronger (IPC-level, agent-invisible, lifecycle-governed) but with NO user front door. The familiarity fix is FS-1: expose rules/hooks as user-authorable DATA. The reason ELAI's version is SAFER than CC's (and worth switching to): a CC hook is arbitrary shell the agent can be tricked into triggering; an ELAI user rule is a typed predicate (AST/GREP/SCHEMA/METRIC/COMPOSITE) + a verdict (ALLOW/WARN/BLOCK/ESCALATE) evaluated out-of-agent, it passes J-102 admission + VibeGate (FS-2), and it cannot block-by-default until its FP rate is bounded (GD-4) — so a user cannot foot-gun the agent into a wedge. The mutable=0 immune tier protects the security floor: a user rule can ADD checks, never WEAKEN a frozen one (R8).

## Existing state
- RULES ENGINE 3F [WIRED]: rules-as-DATA SQLite, predicates AST/GREP/SCHEMA/METRIC/COMPOSITE, verdicts ALLOW/WARN/BLOCK/ESCALATE, lifecycle PROPOSED->CANDIDATE->SHADOW->ACTIVE->RETIRED (fpr<5%+tpr>80%+causal>50%), mutable=0 immune tier, >=33 seed rules. [MAP 1.7]
- HOOK SYSTEM T1/T2/T3 [WIRED]: PreToolUse/PostToolUse/Stop/PreCompact intercept IPC in orchestrator, agents never see hooks. NO SubagentStop axis. [MAP 1.8]
- J-52 PATH-KEYED DYNAMIC RULE INJECTION [PLANNED]: PostToolUse path-fingerprint hook injects the matching local rule into the context_budget Rules slot. [MAP 1.8]
- J-27 harness protected-blocks + webcache [PLANNED]: stdlib hook shape. [MAP 1.14]
- J-102 config-ingress admission [PLANNED]: blake3 load-time manifest, fail-closed. [MAP 1.8]
- RulesDrawer.svelte + ToolsHooksDrawer.svelte [WIRED]: existing dashboard surfaces. [grep]

## Reads (mandatory before coding)
1. _grounding/L-OQF-GROUNDING-MAP.md (TS-8/16/17/18/24, NN-4, FS-1..FS-4); output-quality-and-familiarity-surface-2026-06-27.md PART 2 + A-3.
2. the rules-engine 3F (the rules table schema + predicate kinds + lifecycle + mutable=0 immune tier); the HOOK SYSTEM dispatch (PreToolUse/PostToolUse/Stop/PreCompact); J-52 path-keyed injection; J-102 admission; RulesDrawer.svelte + ToolsHooksDrawer.svelte.
3. F-0 W-1; G-0 GD-1/GD-4; CLAUDE.md TD-3, R8; .claude/rules/conditional-routing.md (the path-keyed rule precedent J-52 mechanizes).

## Tasks
1. NN-4a `elai rule add` (~60 LOC): a user authors a rule as DATA — a predicate (AST/GREP/SCHEMA/METRIC/COMPOSITE) + a scope (path glob / tool / FSM state) + a verdict (ALLOW/WARN/BLOCK/ESCALATE) — written as a rules-engine row in lifecycle=SHADOW (observe-only); never executable code (FS-2). Validate the predicate against the engine's schema; reject a rule that targets a mutable=0 immune-tier check to WEAKEN it (R8).
2. NN-4b user hook registration (~50 LOC): register the rule against a hook axis (PreToolUse/PostToolUse/Stop/PreCompact) via the J-52 path-keyed injection mechanism (the rule is injected into the context_budget Rules slot on a path match) — the CC "hook fires before edit" familiarity, but as a typed rule on the IPC seam. SubagentStop axis if J-52's SubagentStop leg has shipped, else PostToolUse + note residue.
3. admission + promotion (~30 LOC): a user rule passes J-102 admission (blake3 manifest, fail-closed) + a content scan (dangerous-verb / injection-phrase, reusing the F-1/J-102 deterministic legs) before it is live; it stays SHADOW (observe-only, records would-fire) and promotes to ACTIVE/BLOCK only on the engine's fpr<5%/tpr>80% over the user's own runs (GD-4) — a user cannot ship a hard-blocking rule on day one.
4. `elai rule list/test/promote/retire` (~40 LOC): the lifecycle surface — list user rules + their SHADOW fire-rate, test a rule against a fixture, promote (only if the FP bound clears), retire. FS-1: the user manages rules as DATA.
5. RulesDrawer/ToolsHooksDrawer authoring UI (~surface, decide at edit): add-rule form + per-rule SHADOW fire-rate + promote/retire actions.
6. workflow/reasoning/L-13-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - `elai rule add/list/test/promote/retire` CLI arms (+ RulesDrawer authoring actions)
  - the user-rule registration against the hook axes (via J-52 path-keyed injection)
consumes_symbols:
  - the RULES ENGINE 3F rules table + predicate kinds + lifecycle + mutable=0 immune tier (WRITE rows, never re-implement the engine)
  - the HOOK SYSTEM PreToolUse/PostToolUse/Stop/PreCompact dispatch; J-52 path-keyed injection; J-102 admission; VibeGate F-1 deterministic legs; context_budget Rules slot
reachable_from:
  - the CLI/dashboard > `elai rule add` > J-102 admission > rules-engine SHADOW row [PRODUCTION user authoring]
  - the hook dispatch > the user rule fires (SHADOW observe / ACTIVE block) [PRODUCTION enforcement]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO user rule as executable code (FS-2 — DATA predicate + verdict only)
  - NO user rule that WEAKENS a mutable=0 immune-tier check (R8); NO user rule live before J-102 admission + content scan PASS
  - NO user rule blocking-by-default before its FP bound clears (GD-4 — SHADOW first)
  - NO LLM authoring/approving a user rule's verdict (TD-3); NO re-implementation of the rules engine / hook system / J-52 (GD-1 P0); blake3 not sha2
frontend_pair:
  RulesDrawer.svelte / ToolsHooksDrawer.svelte authoring + SHADOW-fire-rate + cross_lang_diff PASS, OR justified N/A.
observable_effect:
  trigger: "elai rule add a GREP PreToolUse rule (WARN on a project-specific banned pattern); then trigger an edit that matches it; then try elai rule add a rule that targets a mutable=0 immune check to disable it"
  expect_ssb: "reuse the rules-engine rule-fired / lifecycle rows; +0..1 (UserRuleRegistered Kind) if a new variant is needed; baseline+0..1 from LIVE ~268/577 (W-9)"
  expect_artifact: "the user rule fires in SHADOW (records would-WARN, does not block) and is recomputable from the journal (GD-2); the immune-tier-weakening rule is REJECTED at add (R8); promotion to ACTIVE is refused until the FP bound clears (GD-4)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-13_user_authorable_rules_hooks_surface.md --run-trigger
rg -n "rule_add|user_rule|register_rule" .elai_cc/crates   # the user authoring seam exists, non-test caller
rg -ci "exec|eval|shell|Command::new" <the user-rule path>  # MUST be 0 — a user rule is DATA, never code (FS-2)
test -f workflow/reasoning/L-13-decisions.md
```

## ACCEPTANCE
- [ ] FS-1: a user adds/manages a hook/rule as DATA via CLI/drawer, no recompile.
- [ ] FS-2: a user rule is a typed predicate + verdict, never executable code; it passes J-102 admission + a content scan before going live.
- [ ] R8: a user rule cannot WEAKEN a mutable=0 immune-tier check (rejected at add).
- [ ] GD-4: a user rule ships SHADOW/observe-only and promotes to BLOCK only on fpr<5%/tpr>80% over the user's runs.
- [ ] TD-3: the LLM never authors/approves a rule verdict; the engine evaluates the DATA deterministically.
- [ ] GD-1: the rules engine / hook system / J-52 are composed, not re-implemented.
- [ ] cargo check --workspace GREEN; auditor PASS on -p. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = a user-added rule actually fires in the live hook dispatch (SHADOW record), not cargo-green.
- W-2: kill-switch on the user-rule surface, not a new enable graveyard; the engine is already live.
- W-3: exposes BUILT rules-engine/hook code, not a summary-only port.
- W-4: RulesDrawer/ToolsHooksDrawer frontend_pair + cross_lang_diff, or justified N/A.
- W-5/W-10: in-process/harness; no RunPod.
- W-7: rules-engine schema / hook dispatch / J-52 / J-102 anchors re-grepped at edit.
- W-9: prefer-reuse rule-fired/lifecycle rows; +0..1 from LIVE baseline.
- R8: no immune-tier check weakened; gate-manifest unchanged.

## Tests to add
- crates/contracts/tests/l13_user_rules_contract.rs (~90 LOC): (a) a user GREP rule fires in SHADOW and records would-WARN without blocking; (b) a rule weakening a mutable=0 check is rejected at add (R8); (c) a rule is rejected if it contains executable content (FS-2); (d) promotion to ACTIVE is refused until the FP bound clears (GD-4); (e) the fire is recomputable from the journal (GD-2).

## Files touched
- CREATE/MODIFY: the `elai rule add/list/test/promote/retire` CLI arms (args.rs + a rule_cmd module, ~150 LOC); the user-rule->hook-axis registration (~50 LOC via J-52)
- MODIFY: RulesDrawer.svelte / ToolsHooksDrawer.svelte authoring actions (+ Tauri command pair)
- CREATE: crates/contracts/tests/l13_user_rules_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/L-13-decisions.md

## Citations / Grounding
- output-quality-and-familiarity-surface-2026-06-27.md PART 2 FINDING-5 + A-3 + V-4/V-6.
- ELAI ground truth (L-OQF-GROUNDING-MAP TS-8/16/17/18/24, NN-4): rules-engine 3F, HOOK SYSTEM T1/T2/T3, J-52 path-keyed injection, J-27, J-102, RulesDrawer/ToolsHooksDrawer (re-grepped at edit, W-7).
- CLAUDE.md: conditional-routing.md (the path-keyed precedent), TD-3, R8 (mutable=0 immune tier), blake3-only, no .unwrap.

## Open questions / risks
- SubagentStop axis: NO SubagentStop hook exists today (J-52 plans it). If J-52's SubagentStop leg has not shipped, register user rules on PostToolUse + note the residue; do not invent a new hook axis here.
- J-52 path-keyed injection not yet shipped: fall back to a direct rules-engine row evaluated by the existing T1/T2/T3 dispatch (no context-slot injection) and record the residue.
- the immune-tier protection is the whole safety story: verify at edit that the mutable=0 set is genuinely write-protected from the user-rule path (a user rule that re-defines an immune check id must be rejected, not silently shadowed).

## Next
L-13 gives the user the hook/rule familiarity safely. L-12 (skills) + L-13 (rules/hooks) together are the "extend ELAI like you did in CC/Codex" surface; L-14 is the lazy one-shot front door. The user-rule lifecycle reuses the same SHADOW->ACTIVE machine L-11's SOLID proxies use — one promotion path, two consumers.

End of L-13.

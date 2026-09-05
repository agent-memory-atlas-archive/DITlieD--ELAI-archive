# L-12: Skill Front-Door + Self-Improving Skills — light E-43, wire the curator, keep the CC/agentskills workflow

STATUS: PLANNED (WAVE 2 of L-T2; M; ACTIVATION + COMPOSE)

Objective: give the user the skill ergonomics they know from Claude Code / Codex / agentskills.io — drop or edit a SKILL.md, ELAI uses it — AND the thing CC does NOT have: ELAI can IMPROVE a skill on measured evidence (the episodic->skill->rule ladder). ELAI already has the plumbing: E-43 AGENT SKILL SYSTEM [DARK] (agentskills.io-v1 validator + `elai skills-mgmt` + per-skill eval JSONLs + skillspector import gate), F-6 LEARNED-SKILL LIBRARY [WIRED] (SkillTrustTier S1-S4 + Voyager critic-gated acquisition), I-12 PLAYBOOK CURATOR [PLANNED] (promote on >=k SSB-verified outcomes). The front door is UNLIT (E-43 log_skill_invocation has no production caller). L-12 lights it: a production caller for skill invocation + a user-facing `elai skill add` flow gated by skillspector + J-102 admission, and the I-12 curator wired so a proven playbook is promoted to a skill. HARD BOUNDARY: every user skill passes the SAME trust gates (FS-2) — skillspector whole-dir scan + J-102 config-ingress admission + VibeGate F-1 before a SKILL.md is usable; a skill is DATA + a gated tool-permit, never arbitrary code. GD-1: do NOT re-author E-43/F-6/I-12 — wire them.

Estimated effort: ~1 session (M). In-process + harness; no model training; no RunPod. The acquisition/curator measured arms reuse existing SSB-verified-outcome rows.
Pre-requisites (re-grep live per W-7): E-43 skill_system_event + agentskills_validator + `elai skills-mgmt` + log_skill_invocation (the unlit caller) + skill-import-policy/skillspector; F-6 SkillTrustTier S1-S4 + skill_permits_autonomous; I-12 curator promote-on-k; J-102 config-ingress admission (blake3 manifest); VibeGate F-1; context_budget Rules slot (skill injection).
Parallel-safe with: L-11/L-13/L-14 (disjoint scope). Soft-dep J-102 (the admission gate) — if J-102 not yet shipped, fall back to skillspector + the E-43 import-policy alone and record the residue.
Source: output-quality-and-familiarity-surface-2026-06-27.md PART 2 (FINDING-6) + V-3 + L-OQF-GROUNDING-MAP TS-13/TS-14/TS-15/TS-24 + NN-3 + FS-1..FS-4. Doctrine: L-0_INDEX.md (FS-1..FS-4, GD-4, TD-3, R8).

## Context
Why this matters: the advanced/trust user is the wedge, but the lazy user — and any CC/Codex migrant — expects skills, hooks, MCP to "just work" the way they already work. The skill door is the most familiar of these and it is built-but-dark. Lighting it is ACTIVATION not invention. The differentiator over CC is FS-1 + the curator: CC skills are static files a human edits; ELAI skills sit on the WIRED rules/eval substrate, are TRUST-TIERED (F-6 S1-S4 gating autonomy), and can be IMPROVED automatically — a playbook that passes >=k SSB-verified outcomes (I-12) is promoted into a skill, and a skill that regresses on its per-skill eval JSONL is demoted. FS-2 is the safety spine: a skill is the #1 prompt-injection surface (Snyk ToxicSkills: 13.4% critical), so every user skill passes skillspector (whole-dir, catches the tests/-dir payload class) + J-102 load-time admission + VibeGate before use. The "ask ELAI to improve a skill" loop is the I-12 ladder run on demand.

## Existing state
- E-43 AGENT SKILL SYSTEM [DARK(ELAI_SKILL_SYSTEM_ENABLE)]: agentskills.io-v1 frontmatter validator + `elai skills-mgmt` + per-skill eval JSONLs + skill-import-policy (skillspector); log_skill_invocation NO production caller. The unlit door. [MAP 1.8]
- F-6 LEARNED-SKILL LIBRARY [WIRED]: SkillTrustTier S1-S4 gates autonomy, Voyager critic-gated acquisition, skill_permits_autonomous in production. [MAP 1.8]
- I-12 PLAYBOOK CURATOR [PLANNED]: promote on >=k SSB-verified outcomes, export to CLAUDE.md/AGENTS.md/skills. [MAP 1.5]
- J-102 config-ingress admission [PLANNED]: blake3 load-time manifest over SKILL.md surface, FAIL-CLOSED quarantine. [MAP 1.8]
- skill-import-policy + skillspector [WIRED, .claude]: whole-directory scan, workflow/skill-imports.log row. [CLAUDE.md]

## Reads (mandatory before coding)
1. _grounding/L-OQF-GROUNDING-MAP.md (TS-13/14/15/24, NN-3, FS-1..FS-4); output-quality-and-familiarity-surface-2026-06-27.md PART 2 + A-3.
2. E-43 skill system (skill_system_event, agentskills_validator, log_skill_invocation, `elai skills-mgmt`); F-6 SkillTrustTier; I-12 curator interface; J-102 admission; .claude/rules/skill-import-policy.md + skill-audit.py (skillspector).
3. F-0 W-1; G-0 GD-1/GD-4; CLAUDE.md TD-3, R8; agentskills.io v1 spec (the SKILL.md frontmatter contract E-43 validates).

## Tasks
1. NN-3a light the door (~40 LOC): give E-43 log_skill_invocation a real production caller at the skill-dispatch seam (a skill named/retrieved in a task is invoked + recorded); flip the dark path to ON behind the existing ELAI_SKILL_SYSTEM_ENABLE (kill-switch, not a new flag). Observe-only first (GD-4).
2. NN-3b user `elai skill add <dir>` flow (~60 LOC): validate frontmatter (agentskills_validator) -> skillspector whole-dir scan (skill-audit.py) -> J-102 admission (blake3 manifest, fail-closed) -> VibeGate F-1 -> on PASS register the skill + a workflow/skill-imports.log row; on any HIGH/CRITICAL finding BLOCK (FS-2). CC/agentskills SKILL.md format accepted verbatim (FS-4).
3. skill RETRIEVAL into context (~30 LOC): a BM25 top-1 skill retrieval over the F-6 trust tiers into the context_budget Rules slot (the H5 shape J-12 also uses), trust-tier-gated (a sub-S-threshold skill needs the F-6 human gate for autonomy). Reuse, do not re-impl.
4. wire I-12 curator (~50 LOC): the episodic->skill->rule ladder — a playbook with >=k SSB-verified outcomes is PROMOTED to a skill (write a SKILL.md + eval JSONL); a skill that regresses on its eval JSONL is DEMOTED (F-6 tier drop). This is "ask ELAI to improve a skill" + the auto path. TD-3: the promotion verdict is the SSB-outcome count, never an LLM "this skill is good" call.
5. `elai skill improve <name>` (~25 LOC): run the I-12 curator on demand over a named skill's recent outcomes; surface the proposed edit + eval delta for user confirm (FS-1 — the user edits DATA, no recompile).
6. workflow/reasoning/L-12-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - the E-43 log_skill_invocation production caller (the lit door)
  - `elai skill add` / `elai skill improve` CLI arms (or dashboard SkillsPanel actions)
  - the I-12 curator promote/demote wiring (episodic->skill->rule)
consumes_symbols:
  - E-43 skill_system_event + agentskills_validator + `elai skills-mgmt` (GENERALIZE/wire, never re-implement)
  - F-6 SkillTrustTier + skill_permits_autonomous (the trust gate); I-12 curator promote-on-k; J-102 admission; skillspector/skill-import-policy; context_budget Rules slot; VibeGate F-1
reachable_from:
  - the skill-dispatch seam > log_skill_invocation [PRODUCTION skill use]
  - the CLI/dashboard > `elai skill add` > skillspector+J-102+VibeGate [PRODUCTION user import]
  - the post-outcome tick > I-12 curator promote/demote [PRODUCTION self-improvement]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO skill usable before skillspector + J-102 + VibeGate PASS (FS-2 — the injection surface)
  - NO LLM "this skill is good/bad" promotion verdict (TD-3 — SSB-verified-outcome count only)
  - NO re-implementation of E-43/F-6/I-12/skillspector (GD-1 P0); NO new enable-flag graveyard (reuse ELAI_SKILL_SYSTEM_ENABLE kill-switch, W-2); blake3 not sha2
frontend_pair:
  SkillsPanel.svelte actions (add/improve/trust-tier + eval delta; SkillsPanel exists — MAP 1.10) + cross_lang_diff PASS, OR justified N/A.
observable_effect:
  trigger: "elai skill add <a dir with a benign SKILL.md> then run a task that retrieves it; then elai skill add <a dir with a tests/-dir injection payload>"
  expect_ssb: "reuse skill_system_event / log_skill_invocation + a skill-imports.log row; +0..1 (SkillPromoted Kind if the curator path fires); baseline+0..1 from LIVE ~268/577 (W-9)"
  expect_artifact: "the benign skill is admitted + invoked + recorded; the injection-payload skill is BLOCKED by skillspector/J-102 with the finding logged (FS-2); a playbook with >=k verified outcomes is promoted to a SKILL.md (I-12) and the verdict is recomputable from the outcome count (GD-2/TD-3)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/L-plan/L-12_skill_front_door_self_improving_skills.md --run-trigger
rg -n "log_skill_invocation" .elai_cc/crates   # now has a NON-test production caller
python .claude/tools/skill-audit.py <a fixture skill dir>   # skillspector runs on `skill add`
rg -ci "llm.*promote|promote.*model_says|gpt.*skill_good" .elai_cc/crates   # MUST be 0 (TD-3)
test -f workflow/reasoning/L-12-decisions.md
```

## ACCEPTANCE
- [ ] FS-1: a user adds/edits a skill as DATA (SKILL.md) with no recompile; `elai skill improve` proposes a DATA edit + eval delta.
- [ ] FS-2: no skill is usable before skillspector whole-dir + J-102 admission + VibeGate PASS; a tests/-dir payload is BLOCKED.
- [ ] FS-4: CC/agentskills.io v1 SKILL.md frontmatter is accepted verbatim (format-compatibility).
- [ ] the curator promotes a playbook to a skill on >=k SSB-verified outcomes and demotes on eval regression — the verdict is the outcome count, never an LLM call (TD-3).
- [ ] GD-1: E-43/F-6/I-12/skillspector composed, not re-implemented. GD-4: the lit path ships observe-only first.
- [ ] cargo check --workspace GREEN; auditor PASS on -p. No new dependency.

## WIRING DONE-GATE (clause by clause)
- W-1: done = the live skill-dispatch seam invokes + records a real skill (not cargo-green); `elai skill add` actually runs skillspector.
- W-2: kill-switch ELAI_SKILL_SYSTEM_ENABLE (existing), no new enable graveyard.
- W-3: lights BUILT E-43 code, not a summary-only port.
- W-4: SkillsPanel frontend_pair + cross_lang_diff, or justified N/A.
- W-5/W-10: in-process/harness; no RunPod.
- W-7: E-43 log_skill_invocation / F-6 / I-12 / skillspector anchors re-grepped at edit.
- W-9: prefer-reuse skill_system_event; +0..1 (SkillPromoted) from LIVE baseline.
- R8: no gate file weakened; J-102/skillspector stay frozen.

## Tests to add
- crates/contracts/tests/l12_skill_front_door_contract.rs (~90 LOC): (a) log_skill_invocation fires from a production dispatch, not a test; (b) `skill add` rejects a skill with a skillspector HIGH finding (FS-2); (c) a valid agentskills.io v1 SKILL.md is accepted (FS-4); (d) the curator promotes only at >=k outcomes, demotes on eval regression (TD-3); (e) the promotion verdict is recomputable from the outcome count (GD-2).

## Files touched
- MODIFY: .elai_cc/crates/.../the skill-dispatch seam (log_skill_invocation caller, ~40 LOC); the I-12 curator wiring (~50 LOC); the context_budget Rules-slot skill retrieval (~30 LOC)
- CREATE/MODIFY: the `elai skill add`/`improve` CLI arms (args.rs + a skill_cmd module, ~85 LOC) + SkillsPanel.svelte actions
- CREATE: crates/contracts/tests/l12_skill_front_door_contract.rs (~90 LOC)
- CREATE: workflow/reasoning/L-12-decisions.md

## Citations / Grounding
- output-quality-and-familiarity-surface-2026-06-27.md PART 2 FINDING-6 + A-3 + V-3. agentskills.io v1 spec (the frontmatter contract); Snyk ToxicSkills (13.4% critical — the FS-2 rationale, design-rationale only, ID-5).
- ELAI ground truth (L-OQF-GROUNDING-MAP TS-13/14/15/24, NN-3): E-43 skill_system_event/log_skill_invocation, F-6 SkillTrustTier, I-12 curator, J-102 admission, skillspector (re-grepped at edit, W-7).
- CLAUDE.md: skill-import-policy.md, TD-3, R8, blake3-only, no .unwrap.

## Open questions / risks
- J-102 not yet shipped: fall back to skillspector + E-43 import-policy as the admission floor and record the residue (the load-time blake3 re-scan-on-modify is the J-102 piece; without it, re-scan on every `skill add` only). Named honestly.
- curator promotion threshold k: ELAI-measured (the per-skill eval JSONL pass-rate + the I-12 >=k), never a guess (ID-5); seeded conservatively + observe-only (GD-4).

## Next
L-12 lights the skill door and the self-improvement loop. L-13 is the rules/hooks sibling (the other "extend ELAI" surface); L-14 is the one-shot builder that a skill can ride. The curator's promotions feed the Verified Case Graph (J-96) episodic->skill->rule ladder — the same compounding-state moat.

End of L-12.

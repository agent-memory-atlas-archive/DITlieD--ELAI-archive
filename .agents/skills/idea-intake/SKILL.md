---
name: idea-intake
description: Layer A intake protocol. Triggers when the user provides an idea, app brief, hirer description, feature request, or asks "build me X" / "we need Y" / "the boss wants Z" / hands you an uploaded brief file. Recursively decomposes the idea into Concepts ] Phases ] Steps ] BabySteps with grounded research at every level via a chain of specialized agents (idea-framer + idea-critic loop ] researcher per concept ] concept-decomposer per concept ] per-level auditor ] researcher per phase if needed ] phase-decomposer per phase ] per-level auditor ] step-decomposer per step ] per-level auditor ] babystep-author per baby step ] per-level auditor) with quality gates at every level. The output is a decomposition tree the planning-workflow skill walks for execution.
license: internal
compatibility: "Requires .elai_cc/ layout, local-rag MCP server, FSM state machine"
metadata:
  version: "1.0.0"
  author: "elai"
---

Layer A is intake. Layer B (planning-workflow skill) is execution. Both are required for full work. Intake produces the tree, planning-workflow drives the tree.

This skill orchestrates the agent chain directly from the main LLM session. It does NOT delegate orchestration to a single big designer agent. Each level of the tree gets its own specialized agent invocation, with parallel batching where siblings are independent. The main LLM holds the orchestration state, surfaces user checkpoints, and assembles the final tree.

Trigger conditions and skip rules: per .Codex/rules/session-protocol.md tier classification (IDEA tier fires this skill).


THE AGENT CHAIN

The full chain of agent invocations (with parallelism, critic loop, per-level audit gates, and checkpoints):

```
Step 1.  regen-map.mjs                                        (1 process)
Step 2.  idea-framer                                          (1 agent)
         > workflow/sessions/{id}/idea-frame.md
Step 2a. idea-critic + idea-framer LOOP                       (max 3 rounds; exits on CRITIQUE PASSED or WARN-ONLY)
         > workflow/sessions/{id}/idea-critique-r{round}.md per round
Step 2b. plan-auditor mode=concepts                           (1 agent, 1 cycle; BLOCK > re-spawn idea-framer ONCE, no re-audit)
Step 3.  USER REVIEW CHECKPOINT (concept list approved ?)
Step 4.  researcher per concept (mode=concept)                (N agents PARALLEL, batch of 4)
         > workflow/sessions/{id}/research/C{n}.md per concept
Step 5.  concept-decomposer per concept                       (N agents PARALLEL, batch of 4)
         > workflow/sessions/{id}/concepts/C{n}.md per concept
Step 5a. plan-auditor mode=phases per concept                 (N agents PARALLEL, batch of 4, 1 cycle each;
                                                               BLOCK > re-spawn the failing concept-decomposer ONCE, no re-audit)
Step 6.  USER REVIEW CHECKPOINT (concept tree approved ? optional end-session-and-resume)
Step 7.  researcher per phase (mode=phase, CONDITIONAL)       (M agents PARALLEL where needed, batch of 4)
         > workflow/sessions/{id}/research/C{n}.P{m}.md per phase with open questions
Step 8.  phase-decomposer per phase                           (M agents PARALLEL within concept, sequential across concepts, batch of 4)
         > workflow/sessions/{id}/concepts/C{n}/P{m}.md
Step 8a. plan-auditor mode=steps per phase                    (M agents PARALLEL, batch of 4, 1 cycle each;
                                                               BLOCK > re-spawn failing phase-decomposer ONCE, no re-audit)
Step 9.  step-decomposer per step                             (K agents PARALLEL, batch of 4)
         > workflow/sessions/{id}/concepts/C{n}/P{m}/S{k}.md
Step 9a. plan-auditor mode=babysteps per step                 (K agents PARALLEL, batch of 4, 1 cycle each;
                                                               BLOCK > re-spawn failing step-decomposer ONCE, no re-audit)
Step 10. babystep-author per baby step                        (J agents MASSIVELY PARALLEL, batch of 4)
         > workflow/sessions/{id}/babysteps/{babystep_id}.md
Step 10a. plan-auditor mode=babystep-detail                   (J agents PARALLEL, batches of 4, 1 cycle each;
                                                               BLOCK > re-spawn failing babystep-author ONCE, no re-audit)
Step 11. assemble decomposition-tree.md                       (main LLM)
Step 12. plan-level audit (auditor mode=plan, whole tree)     (1 agent, 1 cycle, final sanity check)
Step 13. USER APPROVAL CHECKPOINT (final tree)
Step 14. hand off to planning-workflow (Layer B)
```


WHY ONE AGENT PER NODE PER LEVEL

A single designer agent doing all 200 nodes in one context window loses quality past ~30K tokens of working memory. Specialized per-level agents with fresh per-invocation context produce better-grounded, better-scoped output. Parallel batching keeps wall clock manageable. See user discussion 2026-05-09.

WHY CRITIC LOOP + PER-LEVEL AUDITS

Without critic-loop: idea-framer's output is taken at face value, and weak premises (stale stack assumptions, App Store policy conflicts, feasibility issues) cascade through 200 baby steps before the final mode=plan whole-tree audit catches anything. Catching errors at the level they were introduced is cheaper than rebuilding 50 baby steps later. The critic-framer loop catches idea-level weakness before research starts; the per-level auditors catch decomposition-quality issues at each layer of the tree. Both run with a 1-cycle policy (no re-audit after fix) — cascades surface at the next level or at the final whole-tree audit.

Cost: ~130-180 agent invocations per Emil-sized intake (1 framer + 1-3 critics + 1 concepts-audit + 5 researchers + 5 concept-decomposers + 5 phases-audits + ~5 phase-researchers + 15 phase-decomposers + 15 steps-audits + 45 step-decomposers + 45 babysteps-audits + 135 babystep-authors + ~17 babystep-detail-audit batches + 1 final plan-audit). Sonnet pricing: ~$30-50 per intake (~50% over the prior ~$20-30). Wall clock: 45-90 min with parallel batching. Worth it for HIGH-leverage tasks where a wrong concept caught late costs hours of rework.


PROTOCOL — DETAILED

Step 0 — Tier confirmation.

Verify task is IDEA-tier per session-protocol.md. If unsure: "this is going to require full decomposition (Concepts > Phases > Steps > BabySteps with research at each level, ~200 agent calls). Confirm before i start ?" If user says no, downgrade to FOCUSED or HEAVY.

Step 0a — App scope.

Confirm CURRENT_APP. Read workflow/sessions/{session_id}/current-app.txt. If empty or stale, run session-protocol app selection.

If idea is for a NEW app not in APPS, ask user to register it in .Codex/rules/project-config.md first (slug, status=discovery, platforms, stack, build_path).

Step 0b — FSM transition.

mcp_fsm_transition("PLANNING") if not already there.

Step 0c — Capture artifacts.

Read what user provided:
- file path: Read it
- pasted text: capture verbatim into workflow/sessions/{session_id}/idea-source.md
- inline description: paraphrase into idea-source.md and confirm with user

Lock the source text. Future sessions resuming the plan know exactly what was input.

Step 1 — Regenerate codebase map.

Before any agent fires, regen the per-app map for fresh data. Per .Codex/rules/codebase-map.md.

```
node .Codex/tools/regen-map.mjs --app {current_app} --folder {APPS.{current_app}.folder}
```

For greenfield (no .projects/{folder}/ folder yet): script writes a STUB mapIndex.md and exits 0. Continue intake — the C0 bootstrap concept will create the project skeleton.

Step 2 — Spawn idea-framer.

Spawn .Codex/agents/idea-framer.md with: raw_idea (path to idea-source.md), session_id, current_app, mode=intake (or rescope if BubbleUp triggered, or rescope-from-critic if step 2a is re-spawning after a critic BLOCK).

Wait for return. The agent writes workflow/sessions/{session_id}/idea-frame.md and returns one line + warnings.

Read warnings. If:
- USER QUESTION: surface each open question to user, wait for answers, re-spawn idea-framer with answers folded in.
- HIGH-RISK FLAG: note for downstream HIGH-risk handling. Do not abort yet.
- STACK CONFLICT: surface to user. The idea may need to change stack OR be re-scoped.
- GREENFIELD: confirmed, C0 was prepended.

Step 2a — idea-critic + idea-framer loop (max 3 rounds).

Spawn .Codex/agents/idea-critic.md with: session_id, current_app, idea_frame_path, idea_source_path, round=1, prior_critique_path=none.

Wait for return. The critic writes workflow/sessions/{session_id}/idea-critique-r{round}.md and returns one line + verdict.

Branch on verdict:
- CRITIQUE PASSED: exit loop, advance to step 2b.
- WARN-ONLY: exit loop, advance to step 2b. Note warns for surfacing at user checkpoint (step 3).
- BLOCK and round < 3: re-spawn idea-framer in mode=rescope-from-critic with critique_path and prior_frame_path. Wait. Then re-spawn idea-critic with round=round+1 and prior_critique_path pointing to last round's critique. Loop again.
- BLOCK and round = 3: STOP. Surface to user with: "After 3 critic-framer rounds, BLOCK findings remain on the framed idea. Top BLOCK: {top_block_finding}. Options: (a) ManualEdit — you revise the idea-source.md and we restart from step 2; (b) AcceptWithBlock — proceed knowing the BLOCK is unresolved (not recommended); (c) Abort — end session." Wait for user choice.

The critic-framer pair runs at most 3 rounds. Each round costs 1 critic + 1 framer (rescope) = 2 agent invocations. Worst case: 3 rounds = 6 extra invocations on top of the initial framer.

Step 2b — Plan-auditor mode=concepts.

Once the critic loop converges (CRITIQUE PASSED or WARN-ONLY), spawn .Codex/agents/auditor.md in mode=concepts with: session_id, current_app, idea_frame_path, idea_critique_path (the final round's critique).

Wait for return. The auditor evaluates: disjoint concepts, coverage of idea-source goal, sizing, critic-coverage, stack baseline (anti-stale), slop risk in research questions, hard D-rule violations, risk levels.

Branch on verdict:
- AUDIT PASSED: advance to step 3.
- WARN: document the WARNs in idea-frame.md NOTES, advance to step 3 (no re-spawn).
- BLOCK: re-spawn idea-framer in mode=rescope-from-critic ONCE with the audit findings folded into a synthetic critique file. Do NOT re-audit (1 cycle policy). Advance to step 3 with the revised frame. If user catches the BLOCK at step 3 checkpoint, the user can request another revision.

Step 3 — User concept-list review (checkpoint).

Show user the list of concepts from idea-frame.md (just the concept names + 1-line goals + risk flags). Ask: "These are the {N} concepts the framer identified. Approve, or want to adjust before we research and decompose ?"

If user approves: continue.
If user requests changes: relay edits to idea-framer (re-spawn in mode=intake with edits as additional input). Re-checkpoint.
If user wants to abort: mcp_fsm_reset to IDLE.

Step 4 — Concept-level research (PARALLEL).

Dispatch one researcher agent per concept in batches of 4 (rate-limit headroom for the 5-hour window). If concept count > 4, run multiple sequential batches of 4.

Each researcher gets:
- mode = "concept"
- question = the concept's research question from idea-frame.md
- session_id, current_app
- node_id = the concept_id (e.g. "C1")
- cache_key = slug like "research-{app_slug}-{concept-name}-{stack-version}-{year-month}"
- breadth = 7
- depth = 1 (researcher promotes to 2 if HIGH-risk + contradictions detected)
- prior_attempts = empty on first run

Each researcher operates per mode=concept floor: minimum 4 WebSearches, minimum 8 WebFetches, paper-ingestor mandatory if algorithm/paper claim, source-fetcher mandatory if SDK internals needed.

Wait for ALL researchers to complete. Read their one-line returns + warnings.

Aggregate warnings:
- UNVERIFIED PACKAGES across all concept research: if ANY, STOP. User must approve or substitute. Do not proceed with unverified packages in the plan.
- CONTRADICTIONS: list per-concept. If user-blocking, surface for direction. Otherwise note in tree.
- FLOOR-NOT-MET: list per-concept. Plan can continue but is shallower in those areas — caveat to user.
- HIGH-RISK CHECKPOINT: any concept escalated to HIGH gets a user checkpoint at step 6.

Step 5 — Concept decomposition (PARALLEL).

Dispatch one concept-decomposer agent per concept in batches of 4. Each gets:
- concept_id
- session_id, current_app
- idea_frame_path
- research_path = workflow/sessions/{id}/research/{concept_id}.md
- sibling_summaries = list of {concept_id: 1-line goal} (already known from idea-frame, all siblings exist by now)

Wait for all. Read returns + warnings:
- PHASE-LEVEL RESEARCH NEEDED: list of phases that emerged with open questions. These trigger conditional phase-research at step 7.
- PHASE SIZING SPLIT: any phase auto-split. Note in tree.
- DECISION SOURCE FALLBACK: any concept where research returned FALLBACK. Surface for user awareness.
- CONCEPT-OVERFLOW: any concept that produced > 12 phases. Surface for user — concept needs splitting.

Step 5a — Plan-auditor mode=phases per concept (PARALLEL).

Dispatch one auditor agent per concept in mode=phases, in batches of 4. Each gets:
- session_id, current_app
- mode=phases
- target_concept_path = workflow/sessions/{id}/concepts/{concept_id}.md
- target_research_path = workflow/sessions/{id}/research/{concept_id}.md

Wait for all. Read returns + verdicts.

For each concept:
- AUDIT PASSED or WARN-ONLY: advance.
- BLOCK: re-spawn THAT concept's concept-decomposer ONCE with the audit findings folded into the input. Do NOT re-audit (1 cycle policy). The next level (step 8a mode=steps) will catch any cascading issues.

Step 6 — User concept-tree review checkpoint (CRITICAL).

This is the single most important checkpoint. Show user:
1. The MERMAID VIEW of the partial tree (idea > concepts > phases). Pull from each concept's decomposition file.
2. Per-concept summary: name, risk, DecisionSource, top 3 phases.
3. Total estimated baby step count (sum of estimated_baby_steps from all phase entries).
4. Any HIGH-risk concept's reasoning.
5. Any unresolved contradictions from research.

Ask: "Concept tree decomposed. {N_concepts} concepts, {N_phases} phases, est. {N_babysteps} baby steps total. Estimated agent cost from here: ~{N} more invocations. Approve to drill into phases > steps > baby steps NOW, or end this session and resume the deep decomposition in a fresh session ?"

Two paths:
- Continue inline: proceed to step 7. Same session.
- End-session-and-resume: write workflow/sessions/{session_id}/intake-checkpoint.md with the current state (idea-frame + concepts + research + concept decompositions + tree-skeleton). User starts a new session. New session reads the checkpoint and resumes from step 7.

The end-session option exists because steps 7-10 are the most token-heavy (phase decomp + step decomp + ~135 babystep-author calls). Letting the user start fresh keeps each session's working memory clean. Cache hits on prior research mean step 7 is cheap on resume.

If user requests changes to the concept tree: relay to concept-decomposer (re-spawn affected concept's decomposer, or re-spawn idea-framer for fundamental scope change). Re-checkpoint.

Step 7 — Phase-level research (CONDITIONAL, PARALLEL).

For each phase that has open research questions (per concept-decomposer's PHASE-LEVEL RESEARCH NEEDED warnings), dispatch researcher in batches of 4.

Each researcher gets:
- mode = "phase"
- question = the phase's open research question
- node_id = the phase_id (e.g. "C1.P2")
- cache_key = slug derived from question
- breadth = 5
- depth = 1
- prior_attempts = the parent concept's cache_key (researcher leverages prior cache hits)

mode=phase floor: minimum 2 WebSearches, minimum 4 WebFetches.

Wait for all. Aggregate warnings same as step 4.

Step 8 — Phase decomposition.

Dispatch phase-decomposer per phase. Parallelism rules:
- Within a single concept: phases run in PARALLEL (sibling phases are independent within a concept), batches of 4.
- Across concepts: phases run SEQUENTIALLY (concept C2's phases may depend on contracts from C1's phases — wait for C1 phase decomposition before starting C2).

Each agent gets:
- phase_id, session_id, current_app
- concept_path = workflow/sessions/{id}/concepts/{concept_id}.md
- phase_research_path (if exists)
- sibling_summaries within the concept

Wait for each concept's batch. Read returns. Aggregate warnings (including PHASE-OVERFLOW for phases that produced > 10 steps).

Step 8a — Plan-auditor mode=steps per phase (PARALLEL).

For each phase, dispatch one auditor in mode=steps, in batches of 4. Each gets:
- session_id, current_app
- mode=steps
- target_phase_path = workflow/sessions/{id}/concepts/{concept_id}/{phase_id}.md
- parent_concept_path, parent_research_path

Wait for all. For each phase:
- AUDIT PASSED or WARN-ONLY: advance.
- BLOCK: re-spawn THAT phase's phase-decomposer ONCE with audit findings. No re-audit.

Step 9 — Step decomposition (PARALLEL).

All steps run in parallel in batches of 4 (5-hour rate limit headroom). For an Emil-sized intake (~45 steps), that is ~12 sequential batches of 4.

Each step-decomposer gets: step_id, session_id, current_app, phase_path, sibling_summaries within phase.

Wait for all. Read returns. Aggregate warnings (including STEP-OVERFLOW for steps that produced > 8 baby steps).

Step 9a — Plan-auditor mode=babysteps per step (PARALLEL).

For each step, dispatch one auditor in mode=babysteps, in batches of 4. Each gets:
- session_id, current_app
- mode=babysteps
- target_step_path = workflow/sessions/{id}/concepts/{concept_id}/{phase_id}/{step_id}.md
- parent_phase_path, parent_concept_path

Wait for all. For each step:
- AUDIT PASSED or WARN-ONLY: advance.
- BLOCK: re-spawn THAT step's step-decomposer ONCE with audit findings. No re-audit.

Step 10 — Baby step authoring (MASSIVELY PARALLEL).

The biggest batch. Up to ~135 baby step files written in parallel (in batches of 4 to respect rate limit). Each file is ~100 lines of dense 10-block content. For 135 baby steps that is ~34 sequential batches of 4.

Each babystep-author gets: babystep_id, session_id, current_app, step_path, babystep_spec (the row from step decomposition).

Wait for all. This is the longest wall-clock stage (~25-40 min for 135 baby steps in batches of 4).

Aggregate warnings:
- AMBIGUITY: list of baby steps with unresolved integration points. Surface for user — they may need to clarify scope OR this triggers a decomposition fix.
- MAP STALE: rare, but if reported, re-run regen-map and have those baby step authors retry.
- SCOPE-VIOLATION: a baby step that touches > 2 files. Re-spawn step-decomposer for the parent step to fix.

Step 10a — Plan-auditor mode=babystep-detail per baby step (PARALLEL).

For each baby step, dispatch one auditor in mode=babystep-detail, in batches of 4. Each gets:
- session_id, current_app
- mode=babystep-detail
- target_babystep_path = workflow/sessions/{id}/babysteps/{babystep_id}.md
- parent_step_path, parent_phase_path, parent_concept_path

Wait for all. For each baby step:
- AUDIT PASSED or WARN-ONLY: advance.
- BLOCK: re-spawn THAT baby step's babystep-author ONCE with audit findings. No re-audit. The final mode=plan whole-tree audit at step 12 catches any cascading issues.

This is the second-longest wall-clock stage (~20-30 min for 135 baby steps in batches of 4 audit invocations).

Step 11 — Assemble decomposition-tree.md.

Main LLM reads each concept/phase/step file (small — ~5-10 lines each at this aggregation level), writes the master workflow/sessions/{session_id}/decomposition-tree.md per workflow/memory-bank/apps/_TEMPLATE/plans/DECOMPOSITION-TREE-TEMPLATE.md.

Tree contents:
- IDEA header from idea-frame.md
- Concept rows pointing to concepts/{id}.md
- Phase rows nested under concepts, pointing to detail files
- Step rows nested under phases, pointing to detail files
- Baby step rows nested under steps, pointing to babysteps/{id}.md
- Mermaid view assembled from all node ids
- PROGRESS section initialized (cursor = first leaf, all status=pending)

Initialize workflow/sessions/{session_id}/tree-cursor.txt to the first baby step in depth-first order. The planning-workflow skill reads this when execution starts.

Step 12 — Plan-level audit (whole-tree final sanity check).

Spawn auditor agent (.Codex/agents/auditor.md) with mode="plan" and the decomposition-tree.md path.

This is the FINAL whole-tree audit, after all per-level audits have already run. It catches anything the per-level audits missed (cascading issues, cross-level contradictions, integration-contract gaps that only become visible at the whole-tree level).

Auditor evaluates: feasibility, completeness, domain rule compliance, testability, scope realism, stub closure (every forward stub has resolver), UI coverage (every UI-touching baby step has BLOCK 8 populated), risk levels (HIGH risks have research-grounded justification).

1 audit cycle (per workflow's 1-cycle policy). If BLOCK findings:
- read findings
- surface to user with the failing nodes + recommended fix per finding
- options: (a) AutoFix — re-spawn the relevant level-N decomposer ONCE per BLOCK; (b) ManualEdit — user edits the tree directly; (c) Abort
- after AutoFix: do NOT re-audit. Next user checkpoint (step 13) is the safety net.

If AUDIT PASSED or WARN-ONLY: advance to step 13.

Step 13 — Final user approval.

Show user:
1. Final Mermaid view of the FULL tree.
2. Summary: N concepts, N phases, N steps, N baby steps. Total estimated execution sessions.
3. Highest-risk concept with reasoning.
4. The first baby step's title (entry point of execution).
5. Audit verdict (passed / passed with WARN).

Ask: "Decomposition tree saved at workflow/sessions/{session_id}/decomposition-tree.md. Approve to start execution, or discuss any concept ?"

Wait for user approval. Do not transition to TESTING_FIRST without explicit approval.

Step 14 — Hand off to planning-workflow.

Once approved: mcp_fsm_transition("TESTING_FIRST"). The planning-workflow skill takes over per its SECTION 1 (TREE-DRIVEN EXECUTION). The TreeNavigator walks the tree from the cursor.


RESCOPE MODE (BubbleUp ReplanConcept)

When BubbleUp escalates to ReplanConcept (HIGH-risk failure exhausts retries):
1. User picks ReplanConcept from escalation prompt.
2. idea-intake skill re-fires with mode=rescope, focus_concept={id}.
3. Skill spawns idea-framer with mode=rescope, focus_concept={id} — re-frames just that concept.
4. Skill spawns researcher (mode=concept, NEW cache_key) — re-researches.
5. Skill spawns concept-decomposer for the affected concept.
6. Skill spawns phase-decomposer / step-decomposer / babystep-author for the new subtree.
7. The rest of the tree (other concepts) is preserved untouched.
8. Audit + approve only the rewritten subtree.
9. Hand back to planning-workflow with the cursor pointing into the new subtree.


GREENFIELD HANDLING

For status=discovery apps with no .projects/{slug}/ yet:
- step 1 regen-map writes STUB mapIndex.md
- idea-framer auto-prepends C0: "Project bootstrap" concept (per agent spec)
- C0 phases: P1 Expo init, P2 EAS profile, P3 navigation skeleton, P4 test pipeline, P5 first TestFlight build
- After C0 baby steps complete, subsequent regen-map calls produce real maps
- Only after C0 complete does C1 (the first feature) start


CONSTRAINTS

- never start EXECUTING without idea-intake completing for IDEA-tier tasks
- never decompose past 4 levels (Concept > Phase > Step > BabyStep)
- never produce a tree with > 200 baby steps total (system-level safety net; soft target whatever the idea decomposes to naturally) — split into multiple sequential plans if exceeded
- never produce a tree with > 12 concepts (was 7; soft target 2-9 is typical, 12 is the ceiling above which the idea is too big for one tree)
- never ship a tree with unverified packages
- never silently override user-edited concept lists
- never skip any of the agents per level (idea-framer, idea-critic, concept-decomposer, phase-decomposer, step-decomposer, babystep-author, plus the per-level auditor between layers) — each level produces critical context for the next
- never run more than 4 agents in any single parallel batch (5-hour rate limit headroom; was 8)
- never run more than 3 critic-framer rounds (round 3 BLOCK escalates to user)
- never re-audit after a per-level audit BLOCK fix (1-cycle policy; cascades caught at next level or final mode=plan audit)
- the user is the only entity allowed to approve checkpoints


COST DISCIPLINE

For Emil-sized intake (1 idea + 5 concepts + 15 phases + 45 steps + 135 baby steps), with critic loop + per-level audits:
- 1 idea-framer: ~$0.05
- 1-3 idea-critic + 0-2 idea-framer-rescope (loop average ~2 critics + 1 rescope): ~$0.30
- 1 plan-auditor mode=concepts: ~$0.10
- 5 concept-researchers: ~$2-5
- 5 concept-decomposers: ~$0.50
- 5 plan-auditors mode=phases: ~$0.50
- ~5 phase-researchers (conditional): ~$1-2
- 15 phase-decomposers: ~$1
- 15 plan-auditors mode=steps: ~$1.50
- 45 step-decomposers: ~$3
- 45 plan-auditors mode=babysteps: ~$4.50
- 135 babystep-authors: ~$15-25
- ~17 plan-auditors mode=babystep-detail (in batches of 4 baby steps each): ~$5-8
- 1 plan-auditor mode=plan (final whole-tree): ~$0.50
Total: ~$35-52 per intake (~50% above prior $23-37 baseline). Wall clock: 45-90 minutes with parallel batches of 4.

For tasks that dont warrant this depth: drop to FOCUSED tier and use planning-workflow's legacy single-plan flow.


ANTI-SLOPSQUATTING REINFORCEMENT

Even though researcher + concept-decomposer + phase-decomposer + step-decomposer + babystep-author all verify packages, idea-intake does a final cross-cutting check at step 13 before user approval:
- read workflow/sessions/{id}/research/*.md
- grep for "PACKAGES MENTIONED" sections
- confirm every package has verified URL
- if any unverified: BLOCK final approval

Belt-and-suspenders defense. Hallucinated packages in a 200-baby-step plan would cascade into 50+ broken baby steps.


END IDEA-INTAKE SKILL

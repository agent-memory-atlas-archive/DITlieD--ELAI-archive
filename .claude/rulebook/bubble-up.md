BubbleUp escalation policy (HARD RULE)

Loaded by: baby step execution (planning-workflow skill, post-idea-intake plans), and idea-intake skill's per-level audit gates.

On failure, decide deterministically whether to retry, redecompose, or escalate. Prevents infinite retry loops at wrong granularity. Direct port of ELAI's BubbleUpHandler (Phase 4E).

Two modes:
- EXECUTION-TIME BubbleUp (from THE FOUR LEVELS onward): baby step fails AUDITING / DETERMINISTIC_CHECK / UI_CHECK / PRE_GATE / TDD during tree walk. Full retry budgets per level.
- PLANNING-TIME BubbleUp (next section): per-level audit fires BLOCK during idea-intake. 1 fix-spawn at failing level, no re-audit, cascades caught at next level or final mode=plan whole-tree audit.


PLANNING-TIME BUBBLEUP (idea-intake skill, per-level audit gates)

When a per-level auditor (mode=concepts | mode=phases | mode=steps | mode=babysteps | mode=babystep-detail) returns BLOCK during idea-intake:

1. Read the audit findings.
2. Identify the failing node (concept / phase / step / baby step).
3. Re-spawn the level-N decomposer for that node ONCE, passing the audit findings:
   - mode=concepts BLOCK > re-spawn idea-framer in mode=rescope-from-critic with audit findings folded into a synthetic critique
   - mode=phases BLOCK > re-spawn the failing concept's concept-decomposer with audit findings
   - mode=steps BLOCK > re-spawn the failing phase's phase-decomposer with audit findings
   - mode=babysteps BLOCK > re-spawn the failing step's step-decomposer with audit findings
   - mode=babystep-detail BLOCK > re-spawn the failing baby step's babystep-author with audit findings
4. Wait for the re-spawned decomposer's revised output.
5. Do NOT re-audit. Advance to next step in idea-intake protocol.
6. Cascading issues caught at one of:
   - next level's audit (e.g. concept-decomposer fix's bad phases caught at mode=steps audit)
   - final mode=plan whole-tree audit at step 12 of idea-intake
   - user at CHECKPOINT 3 (final tree approval)

Critic-loop exception (idea-framer + idea-critic): at IDEA level only, the critic-framer pair runs as a true loop (max 3 rounds) BEFORE plan-auditor mode=concepts fires. Each round produces a fresh critique; framer revises. Round 3 BLOCK escalates to user via ESCALATE-IF-ROUND-3 in critic's return format. The plan-auditor mode=concepts after the loop is the additional 1-cycle gate with its own re-spawn policy per step 3.

WARNs: WARN-ONLY does NOT trigger re-spawn. Documented in failing node's NOTES section (or idea-frame.md NOTES for mode=concepts). User acts on WARNs at checkpoints.

Counting toward 50-event safety cap: planning-time re-spawns DO count toward bubble-up-history.md 50-event cap (see SAFETY: 50-EVENT TOTAL CAP). Planning-time events should be rare (2-5 per intake worst case); cap mostly catches runaway execution loops.

Other failure modes during idea-intake (researcher fails floor, system error): regular tool error, surface to user, NOT BubbleUp events.


THE FOUR LEVELS

Level | Retry budget | Action on exhaustion
------|--------------|---------------------
BabyStep | 3 retries | redecompose into smaller baby steps
Step     | 2 retries | redecompose step (different approach)
Phase    | 1 retry   | replan phase (research alternatives)
Concept  | 0 retries | replan concept, REQUIRES HUMAN APPROVAL

A "retry" at any level means a fresh attempt against the SAME spec. A "redecompose" or "replan" creates a NEW spec at the same level.


DECISION TABLE (when a baby step fails AUDITING / DETERMINISTIC_CHECK / UI_CHECK / PRE_GATE / TDD)

1. baby step attempt count: read counter from workflow/sessions/{id}/mistakes.md, count rows for this baby step id.
2. if attempt count < 3: increment, BubbleUpAction = RetryBabyStep. Uses Repair Context Profile (load mistakes.md before RepoMap).
3. if attempt count == 3: BubbleUpAction = RedecomposeBabyStep. Replace failing baby step with N smaller baby steps under same parent step. Reset attempt counter for new children.
4. if RedecomposeBabyStep already tried (parent step has 2+ baby step children from redecompose): BubbleUpAction = bubble to Step level.
5. step-level: read step retry count from session log. if step retries < 2: BubbleUpAction = RedecomposeStep, replace step with new children. Reset.
6. if step retries == 2: BubbleUpAction = bubble to Phase level.
7. phase-level: if phase retries < 1: BubbleUpAction = ReplanPhase. Spawn researcher agent for phase question, redo phase. Reset.
8. if phase retries == 1: BubbleUpAction = bubble to Concept level.
9. concept-level: BubbleUpAction = Escalate. STOP execution. Surface to user with full failure history. User decides: ReplanConcept (new approach), DropConcept (skip and continue), Abort (whole tree dies).


THE FIVE ACTIONS

- RetryBabyStep        — fresh attempt at same spec, mistakes injected as forbidden patterns
- ResearchAlternative  — pause execution, spawn researcher with refined question, then RetryBabyStep with new context
- RedecomposeBabyStep  — replace failing baby step with N smaller children
- RedecomposeStep      — replace failing step with N new children (different decomposition)
- ReplanPhase          — restart the phase from research, possibly different concept of phase entirely
- ReplanConcept        — escalate to user, the concept itself is wrong
- Escalate             — give up gracefully, present history, ask user

Plus one additive action borrowed from ELAI:
- DiagnoseAndRetry — before incrementing attempt counter, spawn auditor in DEAD_END_SCAN mode to determine if the issue is fixable or fundamental. If fundamental, jump straight to ReplanConcept. If fixable, treat as normal retry. Use ONCE per baby step path before exhausting retries.


REPAIR CONTEXT PROFILE (loaded on every retry)

Different from first-attempt context. Does NOT show the executor its own failed reasoning.

First attempt context (normal):
- TaskSpec, RepoMap, RecentChanges, RelevantInterface, GroundingClaims

Retry context (different shape):
- TaskSpec (same)
- MistakeBook entries for this baby step (replaces RepoMap in priority)
- Forbidden_diffs list (blake3 hashes of prior attempts that failed in build / audit)
- StructuralPatternsToAvoid (extracted from MistakeBook structural_pattern field)
- GroundingClaims (re-ground if any prior attempt hit feasibility failure)
- RecentChanges (same)

Planning-workflow builds retry context when invoking executor on attempt 2+. Executor doesnt see "your previous attempt was X." It sees "do not produce diff matching hash Y, do not use pattern Z."


INTEGRATION WITH AUDITING

Auditor in AUDITING returns BLOCK / WARN / DEAD END. Mapping:

- AUDIT PASSED  > exit baby step normally, advance TreeNavigator
- BLOCK         > BubbleUp decision table fires (start at BabyStep level)
- WARN          > log but proceed if exit criteria still met
- DEAD END (auditor declares unsalvageable) > skip RetryBabyStep, jump to ReplanPhase OR ReplanConcept per category:
   - data_contract / module_boundary > ReplanPhase
   - algorithm / platform_compliance > ReplanConcept


INLINE FEASIBILITY CHECK INTEGRATION

During EXECUTING, if executor encounters a novel claim not in GROUNDING (e.g. "this API exists" with no source confirmation), fire a max-2-per-phase inline feasibility check:
1. quick verification: read the SDK / docs / code path that proves or disproves the claim.
2. outcomes:
   - Pass: continue with original plan
   - Fail: emit ReplanRequest. Treat as BubbleUp at the level the wrong claim was decided. Concept research > ReplanConcept. Phase research > ReplanPhase.
   - Blocked: not enough info. Spawn researcher with refined question, then ResearchAlternative.


HUMAN ESCALATION FORMAT

When BubbleUpAction = Escalate, surface to user in this exact form:

"BubbleUp escalation: Concept C{n} '{title}' has exhausted all retry budgets.
History:
  - C{n}.P{a}.S{b}.B{c}: failed {N} times, MistakeBook: [3 most-recent structural patterns]
  - ...
Auditor latest verdict: [BLOCK reason | DEAD END category]
Researcher latest finding: [if any]
Options:
  1. ReplanConcept: re-research C{n} with [suggested new question]
  2. DropConcept: mark C{n} as skipped, continue with C{n+1}
  3. Abort: stop execution, save SESSION_HANDOFF
Pick 1, 2, or 3."

Do NOT make this decision unilaterally. The whole point of HIGH-risk escalation is human judgement.


SAFETY RULE

Single session may not exceed 50 BubbleUp events total across all baby steps. If 50 reached, force Abort + SESSION_HANDOFF. Prevents infinite loops with high-frequency retries.


DISPATCH PROTOCOL (concrete per-action wiring)

Wiring contract between AUDITING outcome and orchestration commands. Planning-workflow invokes this when auditor returns BLOCK / DEAD END / UI_FAILED. Main LLM executes the dispatch, no agent does it.

Inputs the dispatcher reads:
- workflow/sessions/{id}/mistakes.md — per-baby-step attempt history
- workflow/sessions/{id}/bubble-up-history.md — per-step / per-phase / per-concept escalation count (lazy-created on first BubbleUp event)
- the auditor's findings (chat result of last auditor invocation)
- workflow/sessions/{id}/decomposition-tree.md — to find parent ids and update node statuses
- workflow/sessions/{id}/babysteps/{id}.md — to find UI-touching flag, files, integration spec
- the codebase map (workflow/memory-bank/apps/{slug}/map/) — fresh

The dispatch decision (matching DECISION TABLE above):

```
1. read mistakes.md, count rows where babystep_id == current_id
2. read bubble-up-history.md, count rows where target_id == current_id and action in (RedecomposeBabyStep, RedecomposeStep, ReplanPhase)

    bs_attempts = mistakes_count
    bs_redecomposes = bubble_history.count(target=current_bs, action=RedecomposeBabyStep)

    if bs_attempts < 3 and not is_dead_end:
        action = RetryBabyStep  (or DiagnoseAndRetry if first failure ever for this baby step)
    elif bs_attempts == 3 and bs_redecomposes == 0:
        action = RedecomposeBabyStep
    else:
        # bubble to step
        step_redecomposes = bubble_history.count(target=parent_step, action=RedecomposeStep)
        if step_redecomposes < 2:
            action = RedecomposeStep
        else:
            # bubble to phase
            phase_replans = bubble_history.count(target=parent_phase, action=ReplanPhase)
            if phase_replans < 1:
                action = ReplanPhase
            else:
                # bubble to concept
                action = Escalate  # always asks user at concept level
```

DEAD END override: if auditor returned "DEAD END: <category>", skip RetryBabyStep entirely. Map by category:
- data_contract / module_boundary > ReplanPhase
- algorithm / platform_compliance > ReplanConcept (which goes through Escalate first for user approval)


PER-ACTION DISPATCH (exact commands)

ACTION 1: DiagnoseAndRetry (optional, runs ONCE per baby step before retry budget exhausted)

When: FIRST audit failure on a baby step where cause is unclear (not obviously a typo / wrong API name / contract mismatch).
Steps:
1. Spawn auditor agent (.claude/agents/auditor.md) in DEAD_END_SCAN mode by including in the prompt: "Run a dead-end scan on baby step {id}. The audit returned BLOCK on findings: {paste findings verbatim}. Decide: is this fixable in another EXECUTING pass, or is the approach fundamentally broken (data_contract / algorithm / module_boundary / platform_compliance)? Return one line: FIXABLE | DEAD_END:<category>."
2. If auditor returns FIXABLE: continue with RetryBabyStep (dont increment attempt counter for this scan).
3. If DEAD_END: jump to ReplanPhase or ReplanConcept per category mapping above.
4. Append to bubble-up-history.md: action=DiagnoseAndRetry, target=babystep_id, outcome=FIXABLE|DEAD_END:<cat>.

ACTION 2: RetryBabyStep

When: bs_attempts < 3.
Steps:
1. Append a new row to workflow/sessions/{id}/mistakes.md with: id, babystep_id, attempt=bs_attempts+1, structural_pattern (from auditor findings, NO variable names per CTIM-Rover guardrail, describe the failure shape not symbol names), diff_hash (blake3 of failed diff if available, else "no-diff"), failure_signal (build|test|audit|ui|feasibility), observed_evidence (1-line auditor quote), forbidden=true.
2. Append to bubble-up-history.md: action=RetryBabyStep, target=babystep_id, attempt=bs_attempts+1, outcome=pending.
3. Call mcp_fsm_transition("EXECUTING").
4. Reload Repair Context Profile per REPAIR CONTEXT PROFILE section. Next EXECUTING pass uses mistakes.md instead of repo map.
5. Re-execute baby step's BLOCK 3 (TASK).
6. After implementation, transition to AUDITING. Cycle repeats until AUDIT PASSED or budget exhausted.

ACTION 3: ResearchAlternative

When: at any retry, if failure pattern suggests original approach is suboptimal (e.g. third-party library has a known issue, alternative pattern preferred per recent docs).
Steps:
1. Compose a refined research question: "the original approach using {X} failed at {Y} with {Z}. What alternatives handle {Z}, and what are their tradeoffs ?"
2. Spawn researcher agent (.claude/agents/researcher.md) with mode=feasibility (light) or mode=phase (medium) depending on scope. node_id = "{babystep_id}/alternative". cache_key includes "alt-" prefix.
3. Wait for researcher report.
4. If researcher recommends switching approach: update baby step's BLOCK 3 / BLOCK 6, then RetryBabyStep with modified spec.
5. If researcher confirms original approach is correct (failure is implementation, not approach): continue with RetryBabyStep using original spec.
6. Append to bubble-up-history.md: action=ResearchAlternative, target=babystep_id, outcome=approach_changed|approach_kept.

ACTION 4: RedecomposeBabyStep

When: bs_attempts == 3 AND bs_redecomposes == 0.
Steps:
1. Mark failing baby step in decomposition-tree.md as status=redecomposed.
2. Spawn step-decomposer agent (.claude/agents/step-decomposer.md) with: step_id (parent step), session_id, current_app, phase_path, sibling_summaries, AND a redecompose flag with payload: "Baby step {failing_id} failed after 3 attempts. Failure pattern: {structural_pattern from mistakes.md}. Replace it with N smaller children that decompose differently. The other baby steps in this step keep their specs."
3. Wait for step-decomposer return. Read new baby step list.
4. For each NEW baby step, spawn babystep-author (.claude/agents/babystep-author.md) in parallel batch (max 8). Wait for all.
5. Update decomposition-tree.md with new baby steps as siblings of failed one (which stays marked redecomposed for history).
6. Update tree-cursor.txt to point at FIRST new baby step.
7. Reset mistakes.md attempt count for new baby steps (they are fresh).
8. Append to bubble-up-history.md: action=RedecomposeBabyStep, target=failing_babystep_id, new_children=[list of new ids], outcome=pending.
9. Call mcp_fsm_transition("PLANNING") then mcp_fsm_transition("TESTING_FIRST") to begin first new child.

ACTION 5: RedecomposeStep

When: BubbleToStep AND step_redecomposes < 2.
Steps:
1. Mark failing step in decomposition-tree.md as status=redecomposed.
2. Spawn phase-decomposer agent (.claude/agents/phase-decomposer.md) with: phase_id (parent phase), session_id, current_app, concept_path, sibling_summaries, AND a redecompose flag with payload: "Step {failing_step_id} failed after exhausting baby step retries (last failure pattern: {worst_pattern}). Replace with N new steps that decompose differently. Other steps in the phase keep their specs."
3. Wait for return. Read new step list.
4. For each NEW step, spawn step-decomposer (parallel batch max 8). Wait.
5. For each baby step under new steps, spawn babystep-author (parallel batches max 8). Wait.
6. Update decomposition-tree.md.
7. Update tree-cursor.txt to FIRST new step's first baby step.
8. Reset mistakes.md attempt count for new baby steps.
9. Append to bubble-up-history.md: action=RedecomposeStep, target=failing_step_id, new_children=[list], outcome=pending.
10. Transition to PLANNING > TESTING_FIRST for new baby step.

ACTION 6: ReplanPhase

When: BubbleToPhase AND phase_replans < 1, OR DEAD_END:data_contract / module_boundary.
Steps:
1. Mark failing phase in decomposition-tree.md as status=replanned.
2. Compose phase-level research question from accumulated failure history. Include: original phase goal, what was tried, what failed, structural pattern across failures.
3. Spawn researcher agent (.claude/agents/researcher.md) with mode=phase, node_id="{phase_id}-replan", cache_key includes "replan-" prefix. Wait for report.
4. Spawn concept-decomposer (.claude/agents/concept-decomposer.md) for parent concept with rescope flag + new research report. Decomposer rewrites just THIS phase's spec, other phases untouched. Wait.
5. Spawn phase-decomposer for rewritten phase (now has new spec). Wait.
6. Spawn step-decomposer for each new step in rewritten phase (parallel). Wait.
7. Spawn babystep-author for each new baby step (parallel batches). Wait.
8. Update decomposition-tree.md.
9. Update tree-cursor.txt to FIRST new baby step in rewritten phase.
10. Reset mistakes.md for new baby steps.
11. Append to bubble-up-history.md: action=ReplanPhase, target=phase_id, outcome=pending.
12. Transition to PLANNING > TESTING_FIRST.

ACTION 7: ReplanConcept (always goes through Escalate first)

When: BubbleToConcept, OR DEAD_END:algorithm / platform_compliance, OR user picked ReplanConcept from Escalate prompt.
Steps:
1. If reaching via bubble cascade (not user-requested directly): first run Escalate (action 8) with ReplanConcept as recommended option. User decides.
2. If user picks ReplanConcept (or user-requested directly):
   a. Mark failing concept in decomposition-tree.md as status=replanned.
   b. Invoke .claude/skills/idea-intake/SKILL.md with mode=rescope, focus_concept={concept_id}. Orchestrates: idea-framer (rescope, focused on concept), researcher (concept mode, fresh cache_key), concept-decomposer, phase-decomposer per new phase, step-decomposer per new step, babystep-author per new baby step.
   c. OTHER concepts (siblings) remain untouched. Their decomposition-tree rows stay status=pending or status=done.
   d. After rescope completes (with user approvals at concept-list + concept-tree checkpoints), planning-workflow resumes execution at FIRST new baby step in rewritten concept's first phase.
   e. Reset mistakes.md for all new baby steps.
   f. Append to bubble-up-history.md: action=ReplanConcept, target=concept_id, outcome=pending.

ACTION 8: Escalate

When: BubbleToConcept (default), OR if user manually invokes after seeing repeated retries.
Steps:
1. Aggregate failure history. Read:
   - workflow/sessions/{id}/mistakes.md filtered to this concept's baby steps
   - workflow/sessions/{id}/bubble-up-history.md filtered to this concept and descendants
   - latest auditor verdict (from chat context)
   - any researcher reports spawned for ResearchAlternative actions
2. Write structured report to workflow/sessions/{session_id}/bubble-up-escalation-{timestamp}.md with sections:
   - WHAT FAILED: failing baby step / step / phase / concept
   - HOW IT FAILED: attempts with structural patterns (no variable names, just shapes)
   - WHAT WAS TRIED: actions at lower levels (RetryBabyStep counts, RedecomposeBabyStep counts, etc)
   - LATEST AUDITOR VERDICT: auditor's last BLOCK / DEAD END output
   - LATEST RESEARCH FINDINGS: if any ResearchAlternative spawned researcher, paste report's CONVERGE section
3. Surface to user in this exact form (matches HUMAN ESCALATION FORMAT above with concrete payloads):

   "BubbleUp escalation: {Level} {target_id} '{title}' has exhausted retry budgets.
   History: {top 5 failure patterns from mistakes.md}
   Auditor latest verdict: {paste last BLOCK or DEAD END line}
   Researcher latest finding: {if any, 1-line CONVERGE summary}
   Options:
     1. ReplanConcept: re-research and re-decompose this concept (idea-intake rescope mode). Estimated cost: ~$5-15 + 30-60 min.
     2. DropConcept: mark this concept as skipped, continue with the next concept in tree-cursor order.
     3. Abort: stop execution, save SESSION_HANDOFF, end session.
     4. ManualFix: user fixes the underlying code OR the baby step spec by hand, then asks me to retry from the failing baby step. (No new agents spawned, user owns the fix.)
   Pick 1, 2, 3, or 4."

4. Wait for user response. Do NOT auto-pick.
5. Execute user's choice:
   - 1 ReplanConcept: jump to ACTION 7 step 2 (skip user-confirmation since this IS the user response).
   - 2 DropConcept: mark concept status=skipped in decomposition-tree.md, advance tree-cursor.txt to first baby step of next pending concept, transition to PLANNING > TESTING_FIRST.
   - 3 Abort: write SESSION_HANDOFF in workflow/memory-bank/apps/{current_app}/activeContext.md per planning-workflow's context-exhaustion handoff protocol, end session.
   - 4 ManualFix: pause execution, wait for user "ready to retry," jump back to RetryBabyStep with mistakes.md + bubble-up-history.md preserved (user's manual fix counts as the "fix" without new agent invocation).
6. Append to bubble-up-history.md: action=Escalate, target={level}_{id}, user_choice=1|2|3|4, outcome=routed.


bubble-up-history.md FORMAT (created lazily on first BubbleUp event)

Path: workflow/sessions/{id}/bubble-up-history.md
Format: one line per event, fields separated by ` | `:

[id] | [timestamp] | [action] | [target_id] | [target_level: BabyStep|Step|Phase|Concept] | [outcome: pending|approach_changed|approach_kept|FIXABLE|DEAD_END:cat|routed] | [extra: e.g. attempt_n, new_children_ids, user_choice]

EXAMPLES

bu-001 | 2026-05-09T14:35:12Z | RetryBabyStep | C1.P2.S1.B1 | BabyStep | pending | attempt_2
bu-002 | 2026-05-09T14:42:08Z | DiagnoseAndRetry | C1.P2.S1.B1 | BabyStep | FIXABLE | -
bu-003 | 2026-05-09T15:01:55Z | RedecomposeBabyStep | C1.P2.S1.B1 | BabyStep | pending | new_children=[C1.P2.S1.B1a,C1.P2.S1.B1b]
bu-004 | 2026-05-09T16:20:31Z | Escalate | C1 | Concept | routed | user_choice=1


SAFETY: 50-EVENT TOTAL CAP

If bubble-up-history.md exceeds 50 events in one session, force Abort regardless of state. Surface "BubbleUp safety cap reached. Probable infinite-retry loop. Saving SESSION_HANDOFF and aborting." Do NOT auto-pick from Escalate options.

---
name: grill
description: Relentless one-question-at-a-time interrogation that drives every unresolved decision out of a plan, design, or fuzzy idea BEFORE prompt-transform reformats it and any code or formal plan is written. USER-INVOKED ONLY — triggers when the user types /grill, /grill <topic>, says "grill me", "grill this plan/design", "stress-test this", "interrogate me on X", "poke holes in this", or "what am i missing". Walks the decision tree branch by branch via AskUserQuestion (one question, recommended answer first), resolves dependencies in order, pins overloaded terms, surfaces code-vs-claim contradictions from the live codebase (codegraph-elai + elai.map + ELAI-FEATURE-MAP), routes unverified external claims to grounding, and hands a resolved-decisions block to prompt-transform / planning-workflow / idea-intake. Read-only on code, works in any FSM state, never auto-fires, never transitions the FSM.
license: MIT
compatibility: "Designed for Codex. Works on Codex with explicit $grill invocation."
metadata:
  version: "1.0.0"
  author: "elai"
---

Grill is the ambiguity killer that runs BEFORE a plan, not after. The user has a half-formed plan, design, or idea; grill interrogates it until every decision branch is resolved, every overloaded term is pinned, and no claim contradicts what the code already does. The output is a clean decision set prompt-transform and planning-workflow can build on, not a vibe.

This exists because a plan written over unresolved ambiguity bakes the ambiguity into code, and ELAI then spends a full FSM cycle (context-compiler, explorers, executor, auditor) implementing the wrong resolution. The cheapest place to kill a wrong assumption is a question; the most expensive is a shipped, wired, audited feature that solved the wrong problem.

Where grill sits in the ELAI flow: grill (de-fuzz the idea) -> prompt-transform (reformat the now-clean task) -> grounding sweep (token-discipline.md) -> planning-workflow / idea-intake / b-plan-runner / paper-to-plan. Grill is the user-facing front-end that idea-critic (agent-to-agent, internal) and a bare AskUserQuestion (single tool call, no method) do not provide.


HARD CONSTRAINT — USER-INVOKED ONLY

NEVER auto-fire grill off a keyword or off the shape of a task. Grill runs ONLY when the user explicitly asks for it in the current message. prompt-transform is the skill that auto-fires before implementation; grill is NOT — an interrogation the user did not ask for is friction, and ELAI's whole anti-sycophancy posture is undercut if the assistant starts cross-examining unprompted. If a task looks fuzzy and you think grilling would help, OFFER it in one line ("want me to grill this before i plan it ?") and wait. Do not start interrogating.


WHEN TO INVOKE

- user types `/grill` or `/grill <topic>`
- user says "grill me", "grill this", "stress-test this design", "interrogate me on X", "poke holes in this plan", "what am i missing"
- user hands a fuzzy brief and explicitly asks to be questioned before planning

When NOT to invoke:
- the task is already spec'd (deliverables + paths + acceptance criteria present) — go straight to prompt-transform / planning, no grilling
- the question is a pure fact lookup — that is oracle, not grill
- the user gave a direct command to implement something concrete — do not stall them with questions, confirm the one genuine fork if any and proceed
- a plan/paper/repo link with a clear "build this" — that is paper-to-plan / orchestrate, not grill (unless the user explicitly asks to grill it first)
- the surface's LOOK/UX is not yet decided (no mock, brief, or reference, "i dont know what this should look like", "design this with me") — that is divergent design, run cook first (.Codex/skills/cook/SKILL.md): cook diverges on real renders the user reacts to, THEN borrows this grill engine to converge. Grilling an undecided look forces convergence before the user has reacted to anything and railroads them into the first idea, the opposite of discovery.


THE METHOD

1 MAP THE DECISION TREE FIRST (silent, GRC). Before asking anything, lay out the branches the plan depends on: the core mechanic / data model / ownership decisions, then the dependent decisions that hang off each. Do this reasoning through mcp_think (GRC is mandatory in this repo; built-in thinking is disabled — a hand-written tree is not allowed, the tree comes from an actual mcp_think call with grounding). You are walking a tree, not firing a checklist. Resolve a parent decision before its children, because the child only matters under one branch of the parent.

2 CODEBASE-FIRST, ALWAYS. Before asking the user ANY question, check whether the code already settles it. Use codegraph-elai (codegraph_explore for "how does X work" / call chains, codegraph_callers / codegraph_callees / codegraph_impact for blast radius — Rust + TS, the pre-built index, answer directly), .Codex/tools/elai.map (grep for DEF/CALL/IMPL_TRAIT/SCIP_REF), the ELAI-FEATURE-MAP.md (feature status + composition edges), or a scoped Read. A question the code already answers is a question you do not ask — you state the finding instead ("the orchestrator already routes this through CloudModeRouter::route, so i'm assuming you keep that path — yes ?"). Asking what the code settles wastes the user's attention and reads as not having looked.

3 ONE QUESTION AT A TIME. Use AskUserQuestion, single question, never a batch. Each question carries a RECOMMENDED answer as the first option (label it "(Recommended)") with your reasoning in its description, plus the real alternatives. The recommendation is grounded: from the code you just read, an established project convention (AGENTS.md hard rule, systemPatterns.md, a decisions doc), or a named tradeoff — never a guess. If the recommendation depends on external API / library / 2026-ecosystem behavior you have not verified, say so and route it to grounding (step 7), do not assert it (grounded-by-default, E-38 posture).

4 PIN OVERLOADED TERMS ON SIGHT. The moment the user uses a term that maps to more than one thing in this codebase, stop and pin it. ELAI is dense with overloaded vocabulary:
- "agent" — an orchestrator/plan agent (.Codex/agents/*.md), an Agent-tool subagent, a runtime agent_pool worker, or an agent_template ?
- "session" — an FSM session (CLAUDE_CODE_SESSION_ID), a RAG/local-rag session, or a git session ?
- "gate" — a wiring gate (R1-R9), the FSM PreToolUse gate, a promotion_gate, or a finance deflation/leakage gate ?
- "claim" — an SSB claim_emitted row, a user-facing factual claim (D-14 sycophancy), or a grounding claim ?
- "context" — the LLM context window, a context-package.md, or ContextPolicy (H-50 regime) ?
- "memory" — memory-bank/, the external memory path, the user-memory crate, or KV cache ?
Terminology drift is where two people agree out loud and build two different things. A glossary conflict against an established project term (systemPatterns.md, ELAI-FEATURE-MAP.md, a decisions doc) is surfaced immediately, not deferred.

5 SURFACE CONTRADICTIONS DIRECTLY. When a user claim contradicts the code, a decisions doc, the spec, or a decision already made earlier in this same grill, name it flatly and make them choose. "You just said the sidecar runs locally, but AGENTS.md + REMOTE-SIDECAR.md say it's canonical on the remote 3090 and must never run locally — which is real for this task ?" Do not smooth it over, do not silently pick one. Anti-sycophancy applies hard: do not accept an answer that creates an inconsistency just because the user gave it confidently. Push back with the evidence (cite the file:line / the rule), then let them resolve it.

6 STRESS-TEST WITH EDGE CASES. For each resolved branch, invent the edge case that forces a boundary. "What happens if the gate's evidence db is absent — FAIL or SKIP ?" (wiring R2 says FAIL). "Two sessions hold the same FSM file — who wins ?" "The flag is default-OFF and never promoted — is that a graveyard (R9) or a registered kill_switch ?" The edge case the user has not thought about is the one that becomes a wire-dark feature or a fail-open gate. A branch is not resolved until its obvious boundary case is decided.

7 ROUTE UNGROUNDED CLAIMS TO GROUNDING, DO NOT GUESS. If resolving a branch depends on a fact about an external API, library, paper, or 2026-ecosystem behavior that is not verified (grounded-by-default, AGENTS.md), mark it OPEN-FOR-GROUNDING rather than inventing an answer. After the grill those become: oracle (quick factual web grounding), paper-ingestor (mode=paper for an arXiv claim, mode=source for an SDK), spec-grounder (a "Spec references:" claim), WebSearch (version/registry/anti-slopsquatting), or a gpt-bridge second opinion (a hard design fork, one diversity vote, never a grounding source). "I don't know, routing to grounding" beats a confident wrong answer baked into the tree.


STOP CONDITION (confidence test, then two-pass replay)

The stop test is checkable, not a vibe: "can i predict the user's reaction to the next three questions i would ask ?" If yes, you have shared understanding — stop. If no, the largest remaining gap is your next question. This ~95% confidence bar replaces "i ran out of questions". If after several rounds confidence is NOT rising, that is a foundational gap, not a detail gap — stop and flag it rather than grinding more questions.

When the confidence test passes: every branch is resolved, no resolved decision depends on an unresolved one, no overloaded term is still ambiguous, and every contradiction surfaced has been chosen. Then do ONE final sweep: replay the resolved decision list back to the user in order and ask "this is what i'll plan against, anything wrong or missing ?". The user must give an explicit yes — "whatever you think" / "sounds good" / silence are NOT acceptance, re-ask with concrete options. One clean replay with an explicit yes ends the grill. A new fork in the replay resolves and replays again.

ANTI-SPIRAL. If the same decision flips back and forth across three questions, or the user keeps deferring the same branch, STOP grilling that branch — it is a genuine open question, not a resolvable one. Mark it DEFERRED with the options laid out, note what would resolve it (a spike, a measured run, an owner call), and move on. Do not loop. Endless interrogation is its own failure mode (the anti-spiral skill + token-discipline.md anti-spiral apply — same-branch flip x3 = stop).


OUTPUT (handed to prompt-transform / planning-workflow / idea-intake)

```
GRILL RESULT — {topic}

RESOLVED DECISIONS
- {decision}: {chosen answer} (because {grounding: code ref / AGENTS.md rule / convention / tradeoff})
- ...

GLOSSARY (terms pinned this session)
- {term} = {the one thing it means here}

CONTRADICTIONS RESOLVED
- claimed {X}, code/spec/rule said {Y} ({file:line}), user chose {Z}

EDGE CASES DECIDED
- {boundary case}: {behavior}

DEFERRED (genuine open questions, not blocking)
- {decision}: options {A/B}, resolves via {spike / measured run / owner call}

OPEN-FOR-GROUNDING (verify before the plan locks)
- {claim} > {oracle / paper-ingestor / spec-grounder / WebSearch / gpt-bridge}
```

The RESOLVED DECISIONS + GLOSSARY are the spec prompt-transform reformats and a plan is written against. OPEN-FOR-GROUNDING items must be grounded (per the grounding sweep in token-discipline.md) before TESTING_FIRST. The glossary is pure vocabulary — it never holds implementation detail, it only fixes what each word means.


CONSTRAINTS

- read-only on code. Grill never writes a project file, never transitions the FSM, never session-claims, never dispatches an executor. It is an interrogation; the prompt-transform + plan that follow do the writing.
- one question at a time, recommended answer first, grounded. A batch of questions or a recommendation pulled from training instinct both defeat the point.
- GRC-bound: the silent decision-tree mapping (step 1) and the confidence test come from actual mcp_think calls with grounding, never a hand-written blockquote (AGENTS.md GRC hard rule).
- never grill a task that is already spec'd. If the user already gave deliverables + paths + acceptance criteria, grilling is friction — hand it to prompt-transform.
- do not fold under a confident-but-inconsistent answer, and do not invent a fact to avoid a question — route it to grounding.


END GRILL SKILL

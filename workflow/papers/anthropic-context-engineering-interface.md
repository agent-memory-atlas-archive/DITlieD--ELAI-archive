Source: Effective Context Engineering for AI Agents (Anthropic engineering blog)
URL: https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents
Published: 2025-09-29
Authors: Prithvi Rajasekaran, Ethan Dixon, Carly Ryan, Jeremy Hadfield (Anthropic Applied AI team)
Reliability: MEDIUM-HIGH (Anthropic first-party, written by the team that built Claude Code and Claude agents; describes production techniques in use at Anthropic; not peer-reviewed)

CORE THESIS:
Context is a finite resource with diminishing marginal returns ("attention budget"). Good context engineering means finding the SMALLEST set of high-signal tokens that maximizes the probability of desired behavior. For long-horizon agent tasks, three complementary techniques address context pollution: compaction (summarize-and-restart), structured note-taking (agent-authored external memory), and sub-agent architectures (isolated subtask contexts returning condensed summaries of 1,000-2,000 tokens). The guiding principle is constant: minimize tokens, maximize signal.

MECHANISMS

MECHANISM 1: System Prompt "Right Altitude"
What it does: positions system prompt specificity in a Goldilocks zone -- not so brittle that it hardcodes logic, not so vague that it fails to constrain behavior.
When it applies: any system prompt authoring; especially important as tasks grow more complex.
Parameters/thresholds given:
  - Organize into distinct named sections: <background_information>, <instructions>, ## Tool guidance, ## Output description (verbatim recommendation)
  - "start by testing a minimal prompt with the best model available"
  - "add clear instructions and examples to improve performance based on failure modes found during initial testing"
Key failure modes: (a) hardcoded brittle logic -- fragile, maintenance-heavy; (b) vague high-level guidance -- LLM lacks concrete signals.

MECHANISM 2: Token-Efficient Tool Design
What it does: ensures tool return values are compact, tool schemas are unambiguous, and tool sets have no redundant overlap.
When it applies: any agent tool design.
Parameters/thresholds given: none quantified; qualitative guidance.
Key rules (verbatim):
  "tools should be self-contained, robust to error, and extremely clear with respect to their intended use"
  "If a human engineer can't definitively say which tool should be used in a given situation, an AI agent can't be expected to do better"
  "curating a minimal viable set of tools for the agent can also lead to more reliable maintenance and pruning of context over long interactions"

MECHANISM 3: Curated Few-Shot Examples (not exhaustive edge cases)
What it does: provides a small set of diverse, canonical examples rather than an exhaustive list of rules and edge cases.
When it applies: any prompt with behavioral guidance.
Parameters/thresholds given: none; anti-recommendation is explicit: "do not stuff a laundry list of edge cases."
Key framing: "For an LLM, examples are the 'pictures' worth a thousand words."

MECHANISM 4: Just-in-Time Context Retrieval
What it does: agents maintain lightweight identifiers (file paths, stored queries, web links) and dynamically load data at runtime via tools. Data is NOT pre-loaded into context. Progressive disclosure: each retrieval step informs the next.
When it applies: large or dynamic data sources; situations where pre-loading all candidates would bloat context.
Parameters/thresholds given: none quantified.
Reference implementation: "Claude Code uses this approach to perform complex data analysis over large databases. The model can write targeted queries, store results, and leverage Bash commands like head and tail to analyze large volumes of data without ever loading the full data objects into context."
Key insight: "the metadata of these references provides a mechanism to efficiently refine behavior" -- folder hierarchy, naming conventions, timestamps are all signals.

MECHANISM 5: Hybrid Retrieval (pre-loaded + just-in-time)
What it does: pre-loads some context for speed; allows agent to explore further at its discretion.
When it applies: tasks with a mix of stable known-context and dynamic unknown-context.
Parameters/thresholds given: none.
Reference implementation: "Claude Code is an agent that employs this hybrid model: CLAUDE.md files are naively dropped into context up front, while primitives like glob and grep allow it to navigate its environment and retrieve files just-in-time."

MECHANISM 6: Compaction
What it does: when context window is nearing limit, pass message history to the model to summarize; reinitiate a fresh context window with the summary. The model preserves high-value content and discards redundant content.
When it applies: long-horizon tasks; any task that generates extensive message history.
Parameters/thresholds given (from Claude Code implementation):
  - Preserves: architectural decisions, unresolved bugs, implementation details
  - Discards: redundant tool outputs, messages
  - Post-compaction context: compressed summary PLUS the five most recently accessed files
Key tuning advice: "Start by maximizing recall to ensure your compaction prompt captures every relevant piece of information from the trace, then iterate to improve precision by eliminating superfluous content."
Lightest-touch form: "tool result clearing" -- drop historical tool call results from message history. (This is a shipped Claude Developer Platform feature as of article date.)
Risk: "overly aggressive compaction can result in the loss of subtle but critical context whose importance only becomes apparent later."

MECHANISM 7: Structured Note-Taking (Agentic Memory)
What it does: agent writes structured notes to an external file/store during the task; notes are pulled back into context at later turns or after context resets.
When it applies: iterative development with clear milestones; multi-session tasks; tasks where state must survive context compaction.
Parameters/thresholds given: none quantified; examples: Claude Code to-do list, custom NOTES.md file.
Key property: notes survive context resets/compaction -- the agent reads its own notes to resume.
Empirical validation: Claude-plays-Pokemon case -- "The agent maintains precise tallies across thousands of game steps—tracking objectives like 'for the last 1,234 steps I've been training my Pokemon in Route 1, Pikachu has gained 8 levels toward the target of 10.'"

MECHANISM 8: Sub-Agent Architectures
What it does: specialized sub-agents handle focused tasks in clean context windows; main agent maintains a high-level plan; sub-agents return condensed summaries only.
When it applies: complex research and analysis; parallel exploration workloads; tasks where detailed search context would pollute the main agent's reasoning.
Parameters/thresholds given:
  - Sub-agent return size: "often 1,000-2,000 tokens" (verbatim)
  - Sub-agent exploration budget: "tens of thousands of tokens or more" (verbatim)
Key property: "the detailed search context remains isolated within sub-agents, while the lead agent focuses on synthesizing and analyzing the results."

VERBATIM EXTRACTS (anti-hallucination, section-anchored)

[Section: "Context Engineering vs. Prompt Engineering"]
"Context engineering refers to the set of strategies for curating and maintaining the optimal set of tokens (information) during LLM inference, including all the other information that may land there outside of the prompts."

[Section: "Why Context Engineering is Important"]
"as the number of tokens in the context window increases, the model's ability to accurately recall information from that context decreases."
"LLMs have an 'attention budget' that they draw on when parsing large volumes of context. Every new token introduced depletes this budget by some amount."
"This results in n^2 pairwise relationships for n tokens."
"These factors create a performance gradient rather than a hard cliff: models remain highly capable at longer contexts but may show reduced precision for information retrieval and long-range reasoning compared to their performance on shorter contexts."

[Section: "The Anatomy of Effective Context" -- guiding principle]
"good context engineering means finding the smallest possible set of high-signal tokens that maximize the likelihood of some desired outcome."

[Section: "The Anatomy of Effective Context" -- system prompt altitude]
"The right altitude is the Goldilocks zone between two common failure modes. At one extreme, we see engineers hardcoding complex, brittle logic in their prompts to elicit exact agentic behavior. This approach creates fragility and increases maintenance complexity over time. At the other extreme, engineers sometimes provide vague, high-level guidance that fails to give the LLM concrete signals for desired outputs or falsely assumes shared context."

[Section: "The Anatomy of Effective Context" -- tool failure mode]
"One of the most common failure modes we see is bloated tool sets that cover too much functionality or lead to ambiguous decision points about which tool to use. If a human engineer can't definitively say which tool should be used in a given situation, an AI agent can't be expected to do better."

[Section: "Context Retrieval and Agentic Search" -- JIT definition]
"Rather than pre-processing all relevant data up front, agents built with the 'just in time' approach maintain lightweight identifiers (file paths, stored queries, web links, etc.) and use these references to dynamically load data into context at runtime using tools."

[Section: "Context Retrieval and Agentic Search" -- progressive disclosure]
"Letting agents navigate and retrieve data autonomously also enables progressive disclosure—in other words, allows agents to incrementally discover relevant context through exploration. Each interaction yields context that informs the next decision: file sizes suggest complexity; naming conventions hint at purpose; timestamps can be a proxy for relevance."

[Section: "Context Retrieval and Agentic Search" -- Claude Code hybrid]
"CLAUDE.md files are naively dropped into context up front, while primitives like glob and grep allow it to navigate its environment and retrieve files just-in-time, effectively bypassing the issues of stale indexing and complex syntax trees."

[Section: "Compaction"]
"Compaction is the practice of taking a conversation nearing the context window limit, summarizing its contents, and reinitiating a new context window with the summary."
"The model preserves architectural decisions, unresolved bugs, and implementation details while discarding redundant tool outputs or messages. The agent can then continue with this compressed context plus the five most recently accessed files."
"Start by maximizing recall to ensure your compaction prompt captures every relevant piece of information from the trace, then iterate to improve precision by eliminating superfluous content."
"One of the safest lightest touch forms of compaction is tool result clearing."

[Section: "Structured Note-Taking"]
"Structured note-taking, or agentic memory, is a technique where the agent regularly writes notes persisted to memory outside of the context window. These notes get pulled back into the context window at later times."
"'for the last 1,234 steps I've been training my Pokemon in Route 1, Pikachu has gained 8 levels toward the target of 10.'"

[Section: "Sub-Agent Architectures"]
"Each subagent might explore extensively, using tens of thousands of tokens or more, but returns only a condensed, distilled summary of its work (often 1,000-2,000 tokens)."
"the detailed search context remains isolated within sub-agents, while the lead agent focuses on synthesizing and analyzing the results."

[Section: "Sub-Agent Architectures" -- task-to-technique mapping]
"Compaction maintains conversational flow for tasks requiring extensive back-and-forth
Note-taking excels for iterative development with clear milestones
Multi-agent architectures handle complex research and analysis where parallel exploration pays dividends"

REIMPLEMENTATION NOTES (ELAI-specific)

Technique: Compaction (Mechanism 6)
Load-bearing components in ELAI:
  1. conversation.rs in the orchestrator -- the auto-compact trigger. Article specifies: preserve architectural decisions + unresolved bugs + implementation details; discard redundant tool outputs. ELAI's current compact prompt should be validated against this list.
  2. The "five most recently accessed files" heuristic is a Claude Code specific detail. The ELAI equivalent would be: after compaction, re-inject the last N SSB events (to restore recent decision context) plus the FSM current state.
  3. Tool result clearing is called out as "safest lightest touch" -- ELAI currently does not strip historical tool results from message history. This is an immediate low-risk improvement.
  4. Decision point: the article explicitly says to tune the compaction prompt on "complex agent traces" -- start with max recall, then prune. ELAI's compaction prompt has not been systematically tuned against long traces.

Technique: Structured Note-Taking (Mechanism 7)
Load-bearing components in ELAI:
  1. The memory-bank / activeContext.md pattern is exactly this technique. The article validates it directly.
  2. The handoff-protocol.md (one file per problem, append across sessions) is also this technique at a coarser granularity.
  3. The SSB event log functions as an append-only note-taking store, but is not "pulled back into context" in a structured way -- only queried. This is the gap: ELAI has the store but lacks the structured pull-back step (a "read my notes" operation at context resume time).
  4. Decision point: the agent should have an explicit "resume from notes" step at session start that queries the SSB for recent relevant events and constructs a brief state summary. This is H-51 (cross-session sleep-time handoff) per the H-plan.

Technique: Sub-Agent Architecture (Mechanism 8)
Load-bearing components in ELAI:
  1. The orchestrate skill's parallel batch (context-compiler + integration-explorers returning summaries to the main agent) is exactly this technique. Article validates it.
  2. The 1,000-2,000 token return budget for sub-agents is a concrete target. ELAI sub-agents currently have no enforced return-size budget. Adding a "return summary <= 2000 tokens" guideline to agent prompts aligns with this finding.
  3. Decision point: the article frames sub-agent isolation as a context-cleanliness technique, not just parallelism. Implication for ELAI: even single-subtask dispatches that would pollute main context should use sub-agents even when they could be done inline.

Technique: Just-in-Time Retrieval (Mechanism 4)
Load-bearing components in ELAI:
  1. The context-compiler agent performs JIT retrieval on behalf of the main agent. The article validates this.
  2. The knowledge crate's indexer (BM25 + vector) is the pre-computed leg of the hybrid model.
  3. The gap: ELAI's main agent does not have direct nav primitives (glob, grep) -- it always goes through agents. This is by design (anti-read-big rule). The article says that is fine as a design choice; "do the simplest thing that works."
  4. Metadata signals (folder hierarchy, naming conventions, timestamps) are already exploited in ELAI's agent prompts (e.g., tests/ vs src/ context for the context-compiler). This matches the article's guidance.

GROUNDED CHECKS

CHECK 1 (compaction content preservation): After a compaction event, the compressed context MUST contain a representation of any unresolved architectural decision or bug that existed before compaction. Test: inject a synthetic "known open bug: X" fact at turn N; trigger compaction; verify the compaction output contains a reference to X. A correct implementation passes; one that drops X is a recall failure.

CHECK 2 (tool result clearing): Message history in a long session should NOT contain the full raw output of tool calls that occurred more than K turns ago (K to be tuned). Test: run a 50-turn session with multiple tool calls; assert that tool result payloads from turns 1-40 are not present verbatim in the message history passed to the model at turn 50. (They may be summarized; they should not be raw.)

CHECK 3 (sub-agent return budget): Each sub-agent dispatch should return a summary no larger than ~2,000 tokens. Test: instrument the Agent tool response capture; assert len(tokenize(response)) <= 2500 (with headroom) for any sub-agent return. A sub-agent returning raw exploration output (tens of thousands of tokens) violates this check.

CHECK 4 (note persistence across resets): After a simulated context reset (compaction), the agent must be able to answer a question that was only recorded in its external notes, not in any message history. Test: have agent write a specific fact to notes; trigger compaction (clearing message history); verify the agent can retrieve the fact from notes on the next turn.

CHECK 5 (tool set non-ambiguity): For every pair of tools in the agent's tool set, a human reviewer should be able to unambiguously state which tool applies to any given scenario. Test: enumerate all tool pairs; for each pair, construct a scenario; if more than one tool is plausible, flag as ambiguous. Zero ambiguous pairs is the target.

FAILURE MODES / CAVEATS (article-stated)

1. Over-compaction: "overly aggressive compaction can result in the loss of subtle but critical context whose importance only becomes apparent later." No quantitative threshold given; must be tuned empirically on complex traces.
2. Bloated tool sets: most common failure mode seen in production. Tools with overlapping functionality cause ambiguous agent decisions.
3. Prompt altitude mismatch: brittle hardcoded logic (fragile, maintenance-heavy) vs vague high-level (no concrete constraints). Both degrade performance.
4. Raw file pre-loading: loading entire large files into context is worse than JIT -- "without ever loading the full data objects into context" is the target.
5. Dead-end navigation: "an agent can waste context by misusing tools, chasing dead-ends, or failing to identify key information" -- requires opinionated tool design and navigation heuristics.
6. Stale indexing: the article explicitly calls out that JIT navigation "bypassing the issues of stale indexing" is one reason to prefer it -- pre-indexed RAG can return stale results.
7. Larger context windows are not the solution: "context windows of all sizes will be subject to context pollution and information relevance concerns" -- the problem does not go away at 1M tokens.

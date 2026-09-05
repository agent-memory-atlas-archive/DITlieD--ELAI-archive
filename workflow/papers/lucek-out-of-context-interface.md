Source: LLMs Out of Context (blog post)
URL: https://lucek.ai/blogs/llms-out-of-context
Published: 2026-02-03
Reliability: MEDIUM (independent technical blog, single author, no peer review; claims largely consistent with cited research)

CORE THESIS:
Context engineering is an optimization problem: minimize tokens consumed, maximize relevance of what reaches the model. The field has evolved from RAG (push-into-window) to filesystem navigation (let the model pull), and the most capable recent formalization is the Recursive Language Model (RLM) framework, which treats the prompt itself as an external environment the model navigates via a Python REPL, with sub-calls to fresh model instances for atomic subtasks.

MECHANISMS

MECHANISM 1: Vector-based RAG
What it does: embeds documents into a high-dimensional vector space; at query time, embeds the query and retrieves documents within a cosine-similarity threshold.
When it applies: latency-sensitive applications (chatbots), bounded/static document sets, well-defined retrieval questions.
Parameters/thresholds given: none quantified; article notes "given a limited document set, tends to work fine for more defined use cases."
Key framing: frames context as a SEARCH problem -- "How do we get the right context INTO the window?"

MECHANISM 2: Filesystem / bash-environment navigation
What it does: stores knowledge in markdown/textual formats; gives the LLM bash-style tools (grep, ls, cat, cd, touch) to explore the knowledge base. Exploits LLM's pre-existing coding knowledge to make navigation intuitive.
When it applies: dynamic or large-scale knowledge bases; coding agents; situations where semantic similarity search is insufficient (need structural/hierarchical signals).
Parameters/thresholds given: none.
Key framing: reframes context as a NAVIGATION problem -- "How do we let the model NAVIGATE context itself?"
Observation: "popularized by coding agents like Claude Code, Cursor, and Codex."

MECHANISM 3: Recursive Language Models (RLMs)
What it does: passes the full prompt as a Python variable into a REPL environment. The model iterates, writing and executing code operations on the prompt (slicing, indexing, sub-queries). It can call sub-LLMs via llm_query() for atomic subtasks (depth=1, no recursion in sub-calls). Loop terminates when model calls FINAL_VAR().
When it applies: asynchronous analysis requiring broad contextual understanding; tasks where total token count far exceeds any single context window.
Parameters/thresholds given:
  - Sub-agent call depth: 1 (sub-LLMs cannot call further LLMs)
  - Demonstrated scale: 10,000,000+ tokens handled
  - Context rot threshold cited: ~100,000 tokens (ChromaDB research)
Key framing: combines summarization + context folding + code execution in one loop.
Training synergy: RLM loop can generate SFT examples or serve as RL environment; both SFT (Zhang et al.) and RL (Prime Intellect) shown to improve instruction following and context management.

MECHANISM 4: Context Folding via Sub-Agents
What it does: delegating atomic tasks to fresh-context model instances (called "context folding" in the article). Each sub-agent operates independently; main agent synthesizes returns.
When it applies: whenever a subtask is atomic and can be fully specified without requiring the full conversation history.
Parameters/thresholds given: none beyond depth=1 constraint in RLM formalization.

VERBATIM EXTRACTS (anti-hallucination, section-anchored)

[Section: "Out of Context Window"]
"Self-attention is, by nature, order agnostic (more specifically permutation equivariant- the output changes correspondingly if you shuffle the input) so this must be assisted through positional encodings that are added to input sequences to inject relevant order information for the LLM."

[Section: "Out of Context Window" -- RoPE]
"Rotary positional encoding applies a position-dependent rotation to each token's representation within attention (typically the query and key vectors), based on where it lands in the sequence."

[Section: "Out of Context Window" -- training data limitation]
"If all training data is 2048 tokens, LLMs will struggle to interpret an input of 16,384 tokens unless explicitly taught how to."

[Section: "Moldy Context" -- context rot definition]
"Research from ChromaDB showed a general decline in performance around 100,000 tokens, only exacerbated through the introduction of distractors. This observation led to the term: context rot."

[Section: "Moldy Context" -- distractors]
"the interpretation of similar, yet different, data that's been retrieved or processed together is less robust than clearly delineated topics."

[Section: "Moldy Context" -- positional bias mechanisms (hypotheses)]
"1. Training Data Distribution - It is difficult to provide many concrete examples of how to actually use and manage extremely long context... the model therefore learns priors that often work in typical length data but don't reliably generalize to long-context evidence scattered throughout.
2. Architectural Limitations - RoPE techniques can exhibit a long-range decay that biases attention toward nearer tokens as context grows, and softmax attention can disproportionately allocate to early tokens, which together reduce effective mid-context retrieval."

[Section: "The 5th Stage of Grief" -- RAG framing]
"RAG frames context management as a search and retrieval problem, positing the question: 'How do we get the right context INTO the window?'"

[Section: "The 5th Stage of Grief" -- filesystem framing]
"This approach, discovered and popularized by coding agents like Claude Code, Cursor, and Codex, reframes context delivery as a navigation problem in which the LLM directly explores the context."

[Section: "Recursive Language Models" -- RLM definition]
"The RLM framework 'treats long prompts as part of an external environment and allows the LLM to programmatically examine, decompose, and recursively call itself over snippets of the prompt.'"

[Section: "Recursive Language Models" -- sub-agent depth]
"These function-called LLMs are not able to call additional LLMs themselves (i.e. depth of 1), but are useful for delegating atomic tasks into a fresh context window."

[Section: "Recursive Language Models" -- context folding term]
"Using this sub-agent technique is also referred to sometimes as context folding."

[Section: "Recursive Language Models" -- scale demonstrated]
"The RLM technique proves that LLMs can scale their understanding of content far exceeding the context window (10M tokens+) when given the correct environment and tools for discovery and navigation of information."

[Section: "Practice Makes Perfect" -- latency tradeoff]
"In situations where analysis can be asynchronous and requires broad contextual understanding, an RLM-style framework is the way to go. For scenarios like chatbots where latency plays a big factor in user experience, faster and narrower retrieval may be better suited."

REIMPLEMENTATION NOTES (ELAI-specific)

Technique: Filesystem Navigation / progressive context disclosure
Load-bearing components in ELAI:
  1. The orchestrator's context-compiler agent already implements a form of this: it reads specific files rather than loading all context. The principle is confirmed and practiced.
  2. The knowledge crate's BM25 + vector indexer is the RAG leg. The article argues this should be augmented with navigation primitives, not replaced.
  3. The SSB event log is a natural "external state" that the agent should navigate with targeted queries (action_type filter, session_id filter, time range) rather than loading whole sessions.
  4. Decision point: the article's filesystem approach assumes the agent has bash-like tools. ELAI's agents currently receive summarized context-packages. The gap is: does the main agent ever get raw nav tools, or always pre-filtered packages? The article argues raw nav is better for complex tasks.

Technique: Context Folding / Sub-agent delegation
Load-bearing components in ELAI:
  1. The agent dispatch system (orchestrate skill) already implements this: sub-agents run in isolated contexts and return condensed summaries. The article validates this architecture explicitly.
  2. The depth=1 constraint (sub-agents cannot spawn further sub-agents) matches ELAI's current parallel_cap=3 flat dispatch model. Good: avoid runaway spawn trees.
  3. Decision point: the article frames sub-agent delegation as a context management technique, not just a parallelism technique. ELAI currently dispatches sub-agents for parallelism; the framing should also be "this sub-task would pollute main context, so delegate even if it could be done inline."

Technique: RLM REPL loop
Load-bearing components in ELAI:
  1. ELAI does not currently have a Python REPL environment for main-agent context navigation. The closest analog is the sidecar's code execution path.
  2. The llm_query() function inside a REPL is essentially what ELAI's Agent tool dispatch does, but triggered from code rather than from a planning FSM state.
  3. Decision point: implementing a true RLM loop for the ELAI knowledge layer would require: (a) passing knowledge-corpus identifiers as variables to a REPL-capable context, (b) making sub-LLM calls triggerable from within that REPL, (c) defining FINAL_VAR() semantics for the knowledge retrieval result.
  4. This is the highest-complexity option; the article notes it trades latency for accuracy -- appropriate for async research tasks, not for live inference turns.

GROUNDED CHECKS

CHECK 1 (context rot threshold): If a session's token count exceeds ~100,000 tokens, retrieval accuracy for facts injected early in the session should degrade measurably. Test: embed a specific factual detail at position ~1K tokens in a 120K-token session; query it at the end. A correct implementation of context management should have compacted or noted that detail before the 100K threshold.

CHECK 2 (sub-agent depth enforcement): In any sub-agent dispatch chain, a depth=2 nested agent call (sub-agent spawning a further sub-agent) should be structurally blocked or produce a clear error. Test: trigger a sub-agent that itself attempts to dispatch a further sub-agent; verify the outer harness rejects or the spawned agent returns BLOCKED with a depth violation note.

CHECK 3 (RAG vs navigation tradeoff): For a latency-sensitive path (e.g., a single-turn completion), the harness should use pre-indexed retrieval (RAG/BM25) rather than live filesystem navigation. Verify: the fast path completion code path does not trigger any file-system scan or shell command; it uses the indexed knowledge store.

CHECK 4 (context folding delegation): When a sub-agent is dispatched, the main agent's context window should NOT contain the raw exploration artifacts the sub-agent produced -- only the sub-agent's return summary. Test: after a sub-agent returns, assert the main agent's message history does not contain more than a configurable summary token budget from that sub-agent's work.

CHECK 5 (distractor isolation): When multiple documents on closely related-but-distinct topics are retrieved together, the retrieval layer should not blend them into a single context block. Test: retrieve documents A and B on similar topics; verify each is clearly delineated with source attribution in the injected context block.

FAILURE MODES / CAVEATS (article-stated)

1. Context rot: performance degrades ~100K tokens; exacerbated by distractors (similar-but-distinct documents retrieved together).
2. RAG does not scale: "optimization becomes more an issue of improving a search engine than an AI system."
3. RLM latency: "trade speed for increased inference time compute" -- not suitable for real-time/chatbot applications.
4. Training data gap: models trained on short sequences (e.g., 2048 tokens) will struggle on long-context inputs even if the architectural context window supports them.
5. Mechanistic uncertainty: root cause of positional bias and context rot is "still an ongoing topic of discussion" -- implementations cannot rely on a definitive fix, only mitigations.
6. Sub-agent depth constraint is a design choice, not a hard architectural limit -- implementations must enforce it deliberately.

Source: RLM as Memory Architecture (rawworks blog post, 2026-02-11)
URL: https://raw.works/recursive-language-models-as-memory-systems/
Reliability: MEDIUM (author's own measured benchmarks, methodology partially described; not peer-reviewed)
Grounding note: rlm-interface.md (arXiv:2512.24601) is the canonical RLM algorithm ground truth.
This file captures only the MEMORY USE CASE layer on top of the base RLM mechanism.

WHAT THIS POST ADDS (beyond rlm-interface.md)

The post demonstrates using RLMs as a MEMORY SYSTEM over stored conversation sessions,
not just a long-context processing algorithm. The memory variant pairs an RLM with
dspy.RLM and observational memory (Pydantic-typed structured observations) to answer
questions over a history of stored session files.

ARCHITECTURE: RLM AS MEMORY SYSTEM

Component 1: Session store
  Sessions stored as files (not a vector DB). The RLM treats the session directory as
  its "context" — a persistent executable environment of files it can inspect via bash.
  Files are NOT embedded or chunked; the RLM slices them programmatically via rlm_query.

Component 2: Observational memory layer (dspy.RLM)
  dspy.RLM enforces structured output: each sub-call returns a Pydantic-typed observation
  rather than free-form text. The claim is that "typed observations force structured
  reasoning" — the structure enforced by DSPy.RLM was the biggest single boost.

Component 3: Delegation pattern
  Canonical usage: "Don't read sessions yourself, delegate."
  Root agent receives query, delegates session-reading sub-tasks to child rlm_query calls.
  Children return structured observations; root synthesizes an answer.

HOW IT DIFFERS FROM PLAIN LONG-CONTEXT

Long-context: raw session history pasted into context window; hits token ceiling;
  no programmatic inspection; no recursion; O(N) context growth.
RLM-as-memory: context window stays small (metadata only); sessions stay as files;
  model slices and queries programmatically; recursion enables depth-first exploration
  of large session stores without token explosion.

HOW IT DIFFERS FROM VECTOR-RAG MEMORY

Vector-RAG: sessions chunked, embedded, retrieved by cosine similarity;
  retrieval is approximate; no reasoning over retrieved chunks before synthesis.
RLM-as-memory: no embeddings; retrieval is model-directed (the model writes the query
  logic); handles multi-hop reasoning across sessions natively via recursion;
  structured observations (Pydantic) replace embedding similarity as the retrieval signal.
  Cost: ~$0.031-0.035/query (MEASURED, Gemini 3 Flash).

BENCHMARK RESULTS (MEASURED by author, LongMemEval leaderboard)

Score context: LongMemEval tests long-range memory over multi-session conversation history.
Reliability: MEASURED by author. Methodology: author ran their own system; compared against
published leaderboard numbers. Not independently reproduced.

dspy.RLM + observational memory, Gemini 3 Flash | 89.8% | ~$0.035/query [MEASURED]
dspy.RLM + tools + delegation, Gemini 3 Flash   | 89.2% | ~$0.031/query [MEASURED]
Standalone RLM, Gemini 3 Pro                    | 87.0% | not stated   [MEASURED]
Standalone RLM, Gemini 3 Flash                  | 58%   | not stated   [MEASURED]
Mastra (GPT-5-mini), leaderboard #1             | 94.87%| not stated   [EXTERNAL, not by author]

Key finding: DSPy structured scaffolding lifted Gemini 3 Flash from 58% to ~89%.
Key finding: Gemini 3 Pro showed no meaningful advantage over Flash at full dspy.RLM structure.

OPTIMIZATION FINDING (MEASURED, NEGATIVE RESULT)

GEPA prompt optimization attempts (~$400 spent) failed to improve results.
Author diagnosis: "Overfits to small val sets." [AUTHOR CLAIM, not formally measured]
Implication: prompt optimization on top of RLM memory is not reliably additive.

AUTHOR'S PR TO DSPY (upstream contribution)

PR #9295 to stanfordnlp/dspy added improvements to dspy.RLM based on this benchmark run.
Details not published in the post. [UNVERIFIED: PR content not fetched]

DESIGN PARAMETERS (from post, not exhaustive)

session_store | directory of session files | no embedding, file-native
observation_type | Pydantic model | structured per-session output from sub-calls
delegation_pattern | root delegates session reads, never reads directly | architectural rule
model | Gemini 3 Flash (tested) | cost-efficient at this task
dspy_wrapper | dspy.RLM | enforces structured output, was the critical boost

FAILURE MODES (from post)

Prompt optimization overfits to small validation sets when applied to RLM memory stack.
Standalone RLM (no DSPy structure) degrades sharply on Flash-class models (58% vs 89.8%).
Post does not document failure on very large session stores or multi-hop queries. [UNVERIFIED]

IMPLEMENTATION PITFALLS (do NOT do this)

Do not skip dspy.RLM structured output layer and use plain rlm_query returns — degrades recall.
Do not embed/chunk sessions for retrieval if using RLM approach — defeats the model-directed inspection advantage.
Do not run GEPA/prompt optimization assuming it will add on top of the structure boost.
Do not read sessions in the root agent — delegate to children (architecture rule from post).

RELATIONSHIP TO EXISTING MAPS

Canonical RLM algorithm: rlm-interface.md (arXiv:2512.24601) — read that for algorithm details.
dspy.RLM API: covered in dspy-interface.md if present; this post treats it as a black box.
This map = the MEMORY APPLICATION PATTERN only, not the base RLM algorithm.

UNVERIFIED

PR #9295 dspy content not fetched.
Performance on session stores larger than LongMemEval test set.
Whether the 89.8% score has been independently reproduced.
Whether Mastra's 94.87% uses a comparable retrieval budget per query.

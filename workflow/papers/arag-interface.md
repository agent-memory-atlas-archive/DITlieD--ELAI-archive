A-RAG: Scaling Agentic Retrieval-Augmented Generation via Hierarchical Retrieval Interfaces
Paper: arXiv 2602.03442 (Feb 2026)
Authors: Mingxuan Du, Benfeng Xu, Chiwei Zhu, Shaohan Wang, Pengyu Wang, Xiaorui Wang, Zhendong Mao
Source: https://github.com/Ayanami0730/arag (MIT License)
Reliability: HIGH (official published code + paper, verified against source)

---

1. CORE IDEA

A-RAG exposes three hierarchical retrieval tools (keyword_search, semantic_search, chunk_read) directly to an LLM agent via a ReAct loop. The agent autonomously decides which tool to call, when, and how many times. No predefined workflow. No graph construction. The model IS the retrieval orchestrator.

Key insight: information in a corpus exists at multiple granularities (keyword-level, sentence-level, chunk-level). Exposing all three to the agent lets it adaptively choose the right granularity per query step.

Three principles that make it "truly agentic":
(1) Autonomous Strategy - agent dynamically chooses retrieval strategies based on task
(2) Iterative Execution - multi-round, adapts based on intermediate results
(3) Interleaved Tool Use - ReAct-style action->observation->reasoning loop

---

2. ARCHITECTURE

2.1 Hierarchical Index Construction

Two offline stages: chunking and embedding. No graph construction, no knowledge extraction.

CHUNKING:
- Partition corpus into chunks of approximately 1,000 tokens each
- Chunk boundaries MUST align with sentence boundaries (preserve semantic coherence)
- Each chunk gets a unique string ID (e.g., "0", "1", "172")
- Storage format: JSON array of "id:text" strings, e.g. ["0:Document chunk content...", "1:Another chunk..."]
- Alternative: JSON array of {id, text} objects

EMBEDDING (sentence-level):
- For each chunk c_i, decompose into sentences {s_{i,1}, s_{i,2}, ..., s_{i,n_i}} using rule-based sentence segmentation
- Sentence splitting: regex split on [.!?\n]+ then filter sentences with len > 10 chars
- Compute dense vector for each sentence: v_{i,j} = f_emb(s_{i,j})
- Embeddings are L2-normalized at encode time (normalize_embeddings=True)
- Model used in paper: Qwen3-Embedding-0.6B (384 or 1024 dim depending on config)
- Batch size: 16-32

INDEX DATA STRUCTURE (serialized as pickle):
```
{
    sentences: List[str],           // all sentences across all chunks, flattened
    embeddings: np.ndarray,         // shape (num_sentences, embed_dim), normalized
    sentence_to_chunk: List[str],   // maps sentence index -> chunk_id
    chunks: Dict[str, {id, text}],  // chunk_id -> chunk object
    model_name: str
}
```

KEYWORD-LEVEL:
- NO pre-indexing. No inverted index. No BM25 index.
- Exact text matching at query time (linear scan over all chunks)
- This is a deliberate design choice to reduce indexing cost

---

3. RETRIEVAL TOOL INTERFACES (complete specifications)

3.1 keyword_search

Purpose: exact lexical matching to locate chunks containing specific terms
Input parameters:
  - keywords: List[str] -- each keyword 1-3 words max (e.g., ["Einstein", "relativity theory", "1905"])
  - top_k: int (default=5, max=20) -- number of top-ranked chunks to return

SCORING FORMULA:
  Score_kw(c_i, K) = SUM over k in K of: count(k, T_i) * |k|

Where:
  - count(k, T_i) = frequency of keyword k in chunk text T_i (case-insensitive)
  - |k| = character length of keyword (longer keywords weighted higher as more specific)
  - Matching is case-insensitive: text.lower().count(keyword.lower())

SNIPPET CONSTRUCTION:
  For each matched chunk, extract sentences containing at least one keyword:
  Snippet(c_i, K) = {s in Sent(c_i) | exists k in K such that k is substring of s}
  (case-insensitive substring match)
  Maximum 5 matched sentences per chunk returned as snippets.

OUTPUT FORMAT:
  "Chunk ID: {id}, Matched keywords in chunk: ... {sentence1} ... {sentence2} ..."
  Sentences joined with " ... " separator, prefixed and suffixed with "..."

IMPLEMENTATION DETAIL:
  - Linear scan over ALL chunks (no index)
  - For each chunk: count occurrences of each keyword, compute score
  - Sort by score descending, take top_k
  - Token counting: tiktoken gpt-4o encoding on all matched sentences

3.2 semantic_search

Purpose: find semantically similar passages using dense retrieval at sentence level
Input parameters:
  - query: str -- natural language query
  - top_k: int (default=5, max=20) -- number of most relevant chunk results

SCORING FORMULA:
  Score_sem(s_{i,j}, q) = (v_{i,j}^T * v_q) / (||v_{i,j}|| * ||v_q||)

Since embeddings are pre-normalized, this simplifies to dot product:
  Score_sem(s_{i,j}, q) = v_{i,j}^T * v_q

IMPLEMENTATION:
  1. Encode query: v_q = f_emb(q), normalized
  2. Compute similarities: np.dot(embeddings_matrix, query_embedding)  // single matmul
  3. Get top (top_k * 3) sentence indices by similarity  // oversample 3x
  4. GROUP sentences by parent chunk_id
  5. Each chunk's score = MAX similarity among its matched sentences
  6. Sort chunks by score descending, take top_k chunks
  7. For each chunk, sort matched sentences by position in original text

OUTPUT FORMAT:
  "Chunk ID: {id} (Similarity: {score:.3f})\nMatched: ... {sentence1} ... {sentence2} ..."

CRITICAL DETAIL: sentences are oversampled 3x (top_k*3) then aggregated to chunks to ensure good chunk coverage even when multiple sentences from the same chunk rank highly.

Thread safety: embedding encoding is mutex-protected (_embedding_lock)

3.3 chunk_read (read_chunk in code)

Purpose: retrieve full content of specified chunks by ID
Input parameters:
  - chunk_ids: List[str] -- list of chunk IDs (e.g., ["0", "24", "172"])

CONTEXT TRACKER (deduplication):
  - Maintains set C_read = {c_{i1}, c_{i2}, ..., c_{ik}} of previously read chunk IDs
  - When agent requests chunk c_i that is in C_read:
    Returns: "(This chunk has been read before)" -- ZERO additional tokens
  - When chunk is new: returns full text, marks as read
  - Purpose: prevents redundant retrieval, encourages corpus exploration

OUTPUT FORMAT for new chunks:
  "===...===\n[Chunk {id}]\n---...---\n{full_text}\n===...==="

OUTPUT FORMAT for already-read chunks:
  "===...===\n[Chunk {id}]\n(This chunk has been read before)\n===...==="

ADJACENT CHUNK STRATEGY:
  Tool description tells agent: "If information seems incomplete or truncated, read adjacent chunks (+/- 1)"
  Agent must compute adjacent IDs itself (e.g., if chunk "24", try "23" and "25")

---

4. AGENT LOOP (complete pseudocode)

Algorithm: A-RAG Agent Loop (ReAct-style)

INPUT: question q, tools T, LLM M, max iterations L, max_token_budget B
INIT: M_msg = [system_prompt, {role:"user", content:q}], C_read = empty_set

FOR l = 1 TO L:
    // Token budget check
    current_tokens = count_tokens(M_msg)
    IF current_tokens > B:
        M_msg.append("You have reached the limit. You MUST now provide a final answer...")
        RETURN M(M_msg, tools=None)

    // LLM call with tool schemas
    response = M(M_msg, T)
    M_msg.append(response)

    // Check if agent wants to use a tool
    IF response contains tool_call (t, args):
        result = t.execute(C_read, args)
        M_msg.append({role:"tool", tool_call_id:tc.id, content:result})
        IF t == chunk_read:
            C_read = C_read UNION args.chunk_ids
    ELSE:
        // No tool call = agent is returning final answer
        RETURN response.content

// Max loops reached without answer
M_msg.append("Answer the question based on info gathered so far")
RETURN M(M_msg)

KEY IMPLEMENTATION DETAILS:
- One tool call per iteration (no parallel tool calling)
- System prompt is minimal: just describes available tools and strategy
- Temperature: 0.0 for deterministic behavior
- max_tokens for LLM response: 16384
- Token budget tracked via tiktoken gpt-4o encoding
- Cost tracking per LLM call (for API billing analysis)

---

5. SYSTEM PROMPTS (verbatim from paper/code)

NAIVE A-RAG SYSTEM PROMPT:
"You are a question-answering assistant with access to a document corpus through available tools.
Your goal is to answer questions accurately by finding and analyzing relevant information from the provided documents.
[Available Tools]
- naive_embedding_search: Return top-k chunks by embedding similarity
- chunk_read: Read the full content of a specific chunk
[Strategy]
Work iteratively: retrieve -> read -> answer.
[When Answering]
- Ground your response in the retrieved documents
- Cite the specific chunks that support your answer
- Provide clear, direct answers supported by evidence
- Avoid speculation beyond what the documents support"

A-RAG (FULL) SYSTEM PROMPT:
"You are a question-answering assistant with access to a document corpus through available tools.
Your goal is to answer questions accurately by finding and analyzing relevant information from the provided documents.
[Available Tools]
- keyword_search: Find chunks by exact keyword matching
- semantic_search: Find chunks by semantic similarity
- chunk_read: Read the full content of a specific chunk
[Strategy]
Work iteratively: search -> read -> evaluate -> search -> read -> ... -> answer. For multi-hop questions, decompose the problem and tackle each sub-question step by step.
[When Answering]
- Ground your response in the retrieved documents
- Cite the specific chunks that support your answer
- Provide clear, direct answers supported by evidence
- Avoid speculation beyond what the documents support"

---

6. HYPERPARAMETERS AND CONFIGURATION

Agent:
  max_loops: 15 (default in configs, 10 in code default)
  max_token_budget: 128000
  verbose: false (production)
  temperature: 0.0
  max_tokens: 16384 (LLM response limit)
  reasoning_effort: "medium" (for reasoning models like GPT-5-mini)

Retrieval:
  top_k: 5 (default for all tools, max 20)
  Embedding model: Qwen/Qwen3-Embedding-0.6B
  Embedding device: cuda:0
  Embedding batch_size: 16 (runtime), 32 (indexing)
  Chunk size: ~1000 tokens, sentence-boundary aligned

Sentence splitting:
  Regex: r'[.!?\n]+'
  Minimum sentence length: 10 characters (filtered during indexing)
  During keyword search: no minimum length filter

---

7. TOKEN EFFICIENCY AND SAVINGS

Retrieved tokens per query (GPT-5-mini backbone, Table 3):

                MuSiQue  HotpotQA  2Wiki   Med.    Novel
Naive RAG       5,387    5,358     5,506   5,418   4,997
HippoRAG2      5,411    5,380     5,538   5,447   5,019
GraphRAG        9,234    8,744     4,201   9,391   9,318
LinearRAG       5,418    5,353     5,518   5,427   4,998
A-RAG (Naive)   56,360   27,455    45,406  23,657  22,391
A-RAG (Full)    5,663    2,737     2,930   7,678   6,087

Token savings of A-RAG (Full) vs Naive RAG:
  MuSiQue:  -5% (slightly more)
  HotpotQA: 49% savings
  2Wiki:    47% savings
  Med.:     -42% (more tokens)
  Novel:    -22% (more tokens)

Token savings of A-RAG (Full) vs A-RAG (Naive):
  MuSiQue:  90% savings (56360 -> 5663)
  HotpotQA: 90% savings (27455 -> 2737)
  2Wiki:    94% savings (45406 -> 2930)
  Med.:     68% savings (23657 -> 7678)
  Novel:    73% savings (22391 -> 6087)

The 55-87% token savings claim comes from comparing Full vs Naive A-RAG. The hierarchical interface (snippets first, then selective chunk_read) is what drives the savings. Naive A-RAG reads full chunks on every search, Full A-RAG reads only snippets first and selectively reads full chunks.

HOW TOKEN SAVINGS WORK:
1. keyword_search and semantic_search return SNIPPETS (matched sentences only), not full chunks
2. Agent evaluates snippets to decide which chunks are worth reading in full
3. chunk_read returns full text but context tracker prevents re-reading
4. Agent can stop early once it has enough evidence
5. Progressive disclosure: coarse-to-fine information access

---

8. PERFORMANCE BENCHMARKS

8.1 Main Results (LLM-Acc / Cont-Acc, %)

GPT-4o-mini backbone:
                MuSiQue     HotpotQA    2Wiki       Med.    Novel
A-RAG (Full)    46.1/39.6   77.1/74.0   60.2/63.7   79.4    72.7
A-RAG (Naive)   43.8/38.5   76.6/70.7   52.3/62.4   79.0    70.0
Best baseline    40.6/38.4   80.7/72.9   64.7/68.5   75.3    70.1

GPT-5-mini backbone:
                MuSiQue     HotpotQA    2Wiki       Med.    Novel
A-RAG (Full)    74.1/65.3   94.5/88.0   89.7/88.9   93.1    85.3
A-RAG (Naive)   66.2/59.7   90.8/85.3   70.6/76.9   92.7    80.4
Best baseline    62.4/52.8   86.2/79.5   87.2/84.8   90.5    77.1

A-RAG (Full) with GPT-5-mini achieves BEST on ALL benchmarks.

8.2 Ablation Results (GPT-5-mini, LLM-Acc)

A-RAG (Full):       74.1  94.5  89.7  93.1  85.3
w/o KW Search:      72.6  93.0  88.9  93.2  85.0
w/o Semantic:       69.4  93.9  89.1  92.1  85.2
w/o Chunk Read:     73.6  93.6  89.0  93.3  85.1

All three tools contribute. Removing semantic search causes largest drop on MuSiQue (-4.7%).
"w/o Chunk Read" means search tools return full chunks instead of snippets.

8.3 Test-Time Scaling

Max-step scaling (MuSiQue-300):
- GPT-5-mini: 5 steps -> 20 steps = ~8% improvement
- GPT-4o-mini: 5 steps -> 20 steps = ~4% improvement
- Stronger reasoning models benefit more from longer horizon

Reasoning effort scaling (MuSiQue-300):
- GPT-5-mini and GPT-5: minimal -> high effort = ~25% improvement

---

9. DATA STRUCTURES FOR RUST REIMPLEMENTATION

9.1 Chunk
```
struct Chunk {
    id: String,        // e.g. "0", "24", "172"
    text: String,      // full chunk text, ~1000 tokens
    sentences: Vec<String>,  // pre-split for keyword search
}
```

9.2 SentenceIndex
```
struct SentenceIndex {
    sentences: Vec<String>,
    embeddings: Vec<Vec<f32>>,  // or ndarray, shape (n_sentences, embed_dim), L2-normalized
    sentence_to_chunk: Vec<String>,  // maps sentence idx -> chunk_id
    chunks: HashMap<String, Chunk>,
}
```

9.3 AgentContext
```
struct AgentContext {
    total_retrieved_tokens: usize,
    retrieval_logs: Vec<RetrievalLog>,
    read_chunk_ids: HashSet<String>,
    search_history: Vec<SearchEntry>,
}

struct RetrievalLog {
    tool_name: String,
    tokens: usize,
    metadata: HashMap<String, Value>,
}
```

9.4 Tool Interface (trait)
```
trait Tool {
    fn name(&self) -> &str;
    fn schema(&self) -> ToolSchema;  // OpenAI function calling format
    fn execute(&self, context: &mut AgentContext, args: &Value) -> (String, ToolLog);
}
```

9.5 ToolRegistry
```
struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}
// Methods: register, get, get_all_schemas, execute, list_tools
```

---

10. COMPARISON: A-RAG vs SINGLE-SHOT RAG

Single-shot RAG:
- One retrieval pass, concatenate top-k chunks into context
- Fixed token budget (always k * chunk_size tokens)
- No adaptation to query difficulty
- Cannot do multi-hop reasoning through retrieval

A-RAG:
- Multiple retrieval passes, agent decides when to stop
- Variable token budget (depends on query complexity)
- Adapts strategy: keyword for entities, semantic for concepts
- Multi-hop: decompose question, retrieve per sub-question
- Progressive disclosure: snippets first, full text on demand

The paradigm shift: from "retrieve-then-read" to "search-read-evaluate-search-read-...-answer"

---

11. FAILURE MODES (from manual analysis of 100 incorrect cases)

A-RAG (GPT-5-mini) primary failures:
  MuSiQue: 82% reasoning chain error, 3% gave up, 9% judge error, 6% corpus missing
  2Wiki: 45% reasoning chain error, 33% gave up, 19% judge error, 3% corpus missing

Within reasoning chain errors:
  MuSiQue: 40% entity confusion, 28% wrong strategy, 22% question misunderstanding, 10% exceed budget
  2Wiki: 71% entity confusion, 29% wrong strategy, 0% question misunderstanding, 0% exceed budget

Key insight: A-RAG shifts the bottleneck from "cannot find documents" (Naive RAG) to "found documents but reasoned incorrectly" (entity confusion is dominant failure).

---

12. IMPLEMENTATION NOTES FOR RUST PORT

12.1 What to keep:
- Three-tool hierarchy (keyword, semantic, chunk_read)
- Snippet-based progressive disclosure
- Context tracker (read chunk deduplication)
- Sentence-level embedding with chunk aggregation
- Keyword scoring: count * char_length
- Case-insensitive keyword matching
- Agent decides strategy autonomously

12.2 What to adapt for code intelligence:
- Chunk size: 1000 tokens is for prose. Code may need different chunking (function-level, file-level)
- Sentence splitting: regex [.!?\n]+ won't work for code. Need AST-aware splitting.
- Keyword search: already works well for code identifiers, function names
- Embedding model: may need code-specific model (e.g., CodeBERT, StarCoder embeddings)
- Adjacent chunk: for code, "adjacent" might mean same-module or caller/callee

12.3 Critical for performance:
- Embeddings must be pre-normalized (saves cosine computation)
- Dot product for similarity (single matmul for all sentences)
- Oversample 3x for semantic search then aggregate to chunks
- Context tracker is just a HashSet, almost zero cost
- Keyword search is linear scan -- fine for <100K chunks, need inverted index for larger

12.4 What NOT to do:
- No graph construction (explicit design choice)
- No BM25 pre-indexing (keyword search is simpler and works)
- No parallel tool calling (one tool per agent step)
- No complex orchestration (simple ReAct loop suffices)
- No training/fine-tuning required

---

13. YAML CONFIG SCHEMA (from reference implementation)

```yaml
llm:
  temperature: 0.0           # deterministic
  max_tokens: 16384          # LLM response limit
  reasoning_effort: "medium" # for reasoning models

embedding:
  model: "Qwen/Qwen3-Embedding-0.6B"
  device: "cuda:0"
  batch_size: 16

agent:
  max_loops: 15              # max ReAct iterations
  max_token_budget: 128000   # total context window budget
  verbose: false

data:
  chunks_file: "data/chunks.json"
  index_dir: "data/index"
```

Environment variables: ARAG_API_KEY, ARAG_BASE_URL, ARAG_MODEL

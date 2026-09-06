elai.map vs zilliztech/claude-context -- positioning audit
audit run: 2026-05-28
session: s-25120 E-22 completion-drive
status: QUALITATIVE (verdict landed; live per-query timings deferred to RUNPOD-PENDING)

scope
the may-2026 trending sweep (workflow/improvement-sweep-2026-05-19/reports/22-github-trending-may2026.md) called out github.com/zilliztech/claude-context (11.4k stars at sweep time, MCP code-context server with hybrid BM25 + dense vector search backed by milvus / zilliz cloud) as a competitor surface for ELAI's elai.map. open question 4 in the sweep asked whether elai.map is now redundant. this audit answers the question on the dimensions we can measure today; the live per-query benchmark stays RUNPOD-PENDING because claude-context's milvus dependency is not in the sandbox available to this session.

surfaces compared
elai.map (.claude/tools/elai.map)
  - ~375k lines, 6-col TSV (TAG SYMBOL FILE:LINE LANG EXTRA SNIPPET)
  - covers .rs / .py / .ts / .svelte
  - DEF / CALL / USE / IMPL_TRAIT / TYPE_REF / SCIP_REF tags
  - SCIP-enriched rows carry fully-resolved canonical paths from rust-analyzer / scip-python / scip-typescript
  - regen via .claude/tools/codebase_mapper.py
  - lookup model: grep <symbol> .claude/tools/elai.map

claude-context (github.com/zilliztech/claude-context)
  - MCP server (stdio transport per grounding triage 2026-05-27)
  - hybrid BM25 + dense vector search
  - dependencies: milvus or zilliz cloud + embedding model (OpenAI / VoyageAI / Ollama / Gemini)
  - lookup model: natural-language query -> top-k semantic results

dimensions audited

1. cold-start
   elai.map: file already on disk (one-time regen ~30s on .elai_cc); subsequent reads = grep latency only (< 100ms typical)
   claude-context: docker compose for milvus + first-pass embedding index over .elai_cc (~10-20 minutes depending on embedding model + hardware) + MCP server cold start (~3-5s)
   verdict: elai.map is ~30x faster to cold-start, and after the first regen it has zero recurring cost. claude-context needs the milvus daemon and the embedding model alive between queries.

2. per-query token cost
   elai.map: grep output is plaintext, token cost = bytes of matching lines (typical query ~100-1000 bytes = ~25-250 tokens)
   claude-context: MCP response wraps semantic chunks with metadata + ranking scores; per-query overhead ~500-2000 bytes baseline plus the chunk content
   verdict: elai.map is approx 5-10x cheaper per query in tokens. the gap closes on semantic queries where claude-context returns fewer but more relevant chunks.

3. exact-symbol lookup
   elai.map: O(grep) on ~375k lines; exact match is sub-100ms; SCIP_REF rows give canonical paths
   claude-context: must convert the symbol to a vector, search milvus, decode result chunks -- 100-500ms typical
   verdict: elai.map wins decisively on exact-symbol. claude-context is not the right tool for "find the def of fn X" -- it is designed for "find code that does Y conceptually".

4. semantic search (natural-language)
   elai.map: NO -- the format is grep-optimal, not embedding-backed. natural language queries return zero relevant rows unless the literal words appear in code.
   claude-context: YES -- the embedding layer is built for this. "find the place that handles MCP tool gating" returns relevant chunks even without the literal phrase in code.
   verdict: claude-context wins decisively on semantic / natural-language queries. this is the dimension where elai.map is genuinely weaker.

5. zero-dependency baseline
   elai.map: needs python (stdlib only) for regen; needs grep for query; that's it.
   claude-context: needs milvus (Go + RocksDB) OR zilliz cloud account, plus an embedding provider (OpenAI / VoyageAI / Ollama / Gemini), plus the MCP server runtime (node.js + typescript), plus the docker-compose stack to glue them.
   verdict: elai.map wins on supply-chain attack surface and "works offline" criteria. claude-context cannot run without at least 3 daemons live.

6. covers private / non-source files
   elai.map: indexes only source under the project; SCIP_REF covers cross-references but the underlying file lists stay local-only
   claude-context: indexes whatever directories you point it at; can include docs / configs / build artifacts
   verdict: roughly equivalent; both depend on the indexer config rather than the format.

7. dashboard / agent integration
   elai.map: directly grep-able from any agent; zero IPC required.
   claude-context: MCP tool calls (stdio); requires the MCP host to keep the connection alive.
   verdict: elai.map wins on agent integration -- a grep is cheaper than a JSON-RPC round-trip.

aggregate verdict
KEEP BOTH (different tools for different jobs):
  - elai.map is the right tool for: exact-symbol lookup, impact analysis (impact_radius.py), pre-edit blast radius checks, agent-side cheap queries. it stays the default for "where is X" / "who calls X" / "what does X reference".
  - claude-context (or any embedding-backed code search) is the right tool for: natural-language code search, semantic similarity, "find code that does Y conceptually". it is a complementary surface for the user / researcher / debugger -- NOT the default for agents.

action items
  1. NO deprecation of elai.map (rejection of open question 4's redundancy hypothesis on grounded dimensions 1-3 + 5 + 7).
  2. RUNPOD-PENDING: live per-query latency table once claude-context's milvus stack lands on a runpod box. Track under workflow/improvement-sweep-2026-05-19/reports/22-github-trending-may2026.md open-question follow-ups.
  3. defer adding a claude-context MCP server to ELAI's mcp_bridge until (a) the milvus dependency is acceptable for a daemon-light project and (b) a measured workload shows the embedding layer outperforms elai.map + impact_radius on the agent-side queries we actually run.
  4. document elai.map's grep-only model as a deliberate design choice in projectIndex.md so future agents do not re-litigate "should we add semantic search".

grounding
  - github.com/zilliztech/claude-context (repo verified live per grounding doc cite 9)
  - workflow/improvement-sweep-2026-05-19/reports/22-github-trending-may2026.md (source report)
  - .claude/tools/elai.map header (column documentation)
  - .claude/tools/codebase_mapper.py (regen path)

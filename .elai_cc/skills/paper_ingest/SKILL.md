---
name: paper_ingest
type: application
version: 1.0.0
description: Ingest an academic paper into RAG and emit an interface map.
triggers: ingest paper, arxiv, paper, read paper, ground from paper, paper-ingestor, arXiv:, doi
inputs: paper_ref
outputs: interface_map, rag_chunks, citation_log
scope: research
tags: [ingest, paper, rag, grounding, read-mostly]
estimated_loc: 0
estimated_minutes: 20
---

Application skill — type=application. Routes prompts that name a paper
(arXiv ID, DOI, or canonical title) to the `paper-ingestor` agent flow:
download > chunk > embed > emit interface map.

Use when:
  - the user asks to ground a design from a specific paper
  - an arXiv ID or DOI is in the prompt
  - downstream phases need the paper's interface map as input
  - the paper is not already under `workflow/papers/`

Do NOT use for general web research (oracle), for ingesting an SDK or repo
(source_ingest), or for a paper already present (the orchestrator can read
the existing interface map without re-ingesting).

Output: `workflow/papers/<slug>-interface.md` (Methods, Results, Limitations,
Equations, Figures, Tables) plus RAG chunks indexed under the paper slug.
The `mcp_ingest_file` tool is the primary mechanism; the agent definition
is at `.claude/agents/paper-ingestor.md`.

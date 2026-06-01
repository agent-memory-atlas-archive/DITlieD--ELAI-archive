---
name: source_ingest
type: application
version: 1.0.0
description: Ingest an SDK or source code repository into RAG with an interface map.
triggers: ingest source, ingest sdk, ingest repo, source ground, sdk ground, ground source, decompiled, read sdk, scan repo
inputs: source_paths
outputs: interface_map, reliability_label, rag_chunks
scope: research
tags: [ingest, source, sdk, rag, grounding]
estimated_loc: 0
estimated_minutes: 30
---

Application skill — type=application. Routes prompts asking to ground from
an SDK / external repo to the `paper-ingestor` agent in mode=source.

Use when:
  - the user supplies one or more source file paths or an SDK name
  - downstream design needs the SDK's interface map
  - the source has not been ingested previously
  - reliability labelling (DECOMPILED=LOW, REVIEWED=MEDIUM, OFFICIAL=HIGH)
    must accompany the result

Do NOT use for academic papers (paper_ingest) or for general web research
(oracle). Decompiled output is never treated as ground truth; the interface
map carries the reliability tag verbatim.

Output: `workflow/papers/<source-slug>-interface.md` plus RAG chunks. The
agent is at `.claude/agents/paper-ingestor.md`; this skill picks the source
mode rather than the paper mode.

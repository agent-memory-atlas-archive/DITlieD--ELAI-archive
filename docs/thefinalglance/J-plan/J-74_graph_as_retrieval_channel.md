# J-74: graph-as-retrieval — wire the shipped dataflow edges as an RRF channel AND expose the ego-graph as a mid-task agent retrieval action

- Track J (context-retrieval wave) | lever CANT-MISREAD | grounding STRONG (CodeRAG-system arXiv:2509.16112 + RepoGraph arXiv:2410.14684 + CodeScaleBench/Sourcegraph production evidence; all interface-mapped) | nature EXTENSION (two shipped graphs, two missing wires) | effort M | WAVE CR-2 | depends none (EXTENDS E-40 + J-38; fold C4 into J-38 at the author's discretion)

- Premise: ELAI already HAS both graphs the report says to add — they are just wired to the wrong consumers. (C3) ELAI has dataflow EDGES (symbol_graph/dataflow.rs DataflowAnalyzer + knowledge lib.rs:239 DataFlowEdge) but they feed IMPACT/blast-radius, NOT the retrieval RRF blend — CodeRAG (2509.16112) shows an AST dataflow dependency walk as a third retrieval path is worth +4.32 EM solo. (C4) ELAI's code graph + traversal SURPASS RepoGraph's construction (SCIP/DKG/PageRank, impact_radius/call_flow/symbol_context) but live in the planning/pre-edit layer, NOT as a generation-time agent-callable retrieval action — CodeScaleBench/Sourcegraph production numbers (P@5 0.140->0.478, file recall 0.127->0.277, cost $0.73->$0.51, exec -38%) show wiring nav-tools as agent actions pays. Both are WIRING EXISTING graphs, not new infrastructure.

## Steal/build
1. DATAFLOW AS AN RRF CHANNEL (C3): reuse the SHIPPED symbol_graph::dataflow edges as a candidate source PRE-rerank, folded into the knowledge RRF cascade at the E-40 5th-ranker seam (E-40 already added a 5th RRF symbol-graph ranker, PARTIAL/unallowlisted). This is a candidate-source wire, not a new graph: the dataflow walk over the existing edges produces candidates that join the RRF blend.
2. EGO-GRAPH AS A MID-TASK RETRIEVAL ACTION (C4): expose k-hop ego-graph serialization (serialize the neighbors of a symbol as flat context) as an MCP/task_runner retrieval ACTION the executor can call mid-reasoning. FOLD INTO J-38: J-38 already exposes the J-18-hardened cascade as a mid-task callable hybrid_search tool + SpIDER-style graph-neighbor fusion — the ego-graph serializer is the SAME mid-task-callable seam, so author C4 as a J-38 leg (the graph-neighbor expansion J-38 already names), not a parallel tool.
EXCLUDED: building a RepoGraph-style graph (ELAI's SCIP/DKG/PageRank is already-shipped-equivalent and stronger — R5 in the ledger); a new symbol_graph_search symbol as a separate subsystem (codegraph-confirmed absent, but the gap is the generation-time INTERFACE, not a new graph); removing any RRF stage (J-75 + user-gated).

## Existing ground truth (grep-confirmed 2026-06-23; re-grep at edit, W-7/W-9)
- DATAFLOW EDGES: crates/orchestrator/src/symbol_graph/dataflow.rs (DataflowAnalyzer, DataflowEdgeKind, dataflow_impact) + crates/knowledge/src/lib.rs:239 (DataFlowEdge) — codegraph-confirmed; they feed DataflowImpactComputed (impact/blast-radius), NOT the RRF blend.
- SYMBOL GRAPH TOOLS / SCIP [PARTIAL] E-40: the 5th RRF symbol-graph ranker (flag ELAI_SYMBOL_GRAPH_5TH_RRF_ENABLE unallowlisted) — the seam C3's dataflow candidates fold into.
- RETRIEVAL AGENTIC LOOP [PLANNED(J-38)]: mid-task callable hybrid_search + SpIDER-style graph-neighbor expansion (top-K -> call/import neighbors depth<=2 -> rerank, +13% arXiv:2512.16956) + context-sufficiency gate. C4's ego-graph serializer IS this seam.
- SYMBOL-LEVEL IMPACT API [WIRED] B-29: impact_radius/call_flow/symbol_context/list — the traversal C4 serializes (compose, do not re-author).

## REVISED scope (W-10 tiered)
- SHIP NOW Tier-1 (in-process, deterministic): C3 dataflow-as-RRF-channel at the E-40 5th-ranker seam (the edges exist; this is a candidate-source wire + an RRF fold) — behind the E-40 flag, allowlisted with a promotion plan.
- Tier-1 also (folds into J-38): C4 ego-graph serializer as a mid-task agent retrieval action on the J-38 callable seam.
- No Tier-2 GPU dependency (both legs are over shipped graphs).

## Integration
- crates/knowledge RRF cascade + the E-40 5th-ranker seam — fold the symbol_graph::dataflow walk in as a candidate source (~80 LOC; reuse DataflowAnalyzer, do not re-author the walk).
- crates/knowledge + crates/orchestrator (the J-38 callable seam) — the k-hop ego-graph serializer as a mid-task retrieval action (reuse B-29 traversal + the C-13 tool seam J-38 names).
- SSB: PREFER-REUSE — C3 rides the E-40 5th-ranker / QueryTrace channel-source field; C4 rides J-38's RetrievalLoopEvent (Kind GraphNeighborFused / EgoGraphSerialized). net 0 if J-38/E-40 carry the field; else baseline+1 from LIVE ~269/578 — decide at edit (W-9).
- cross_lang pair: none (Rust-internal); frontend_pair: retrieval-channel breakdown on KnowledgeDrawer (B-29) or W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- crates/knowledge dataflow-RRF-channel (the dataflow walk folded into the RRF blend at the E-40 5th-ranker seam)
- the k-hop ego-graph serializer exposed as a mid-task agent retrieval action (J-38 leg)
consumes_symbols:
- crates/orchestrator/src/symbol_graph/dataflow.rs DataflowAnalyzer (the dataflow walk; NOT re-authored)
- E-40 5th RRF symbol-graph ranker seam (the RRF fold point; NOT re-authored)
- J-38 mid-task callable hybrid_search seam + SpIDER graph-neighbor fusion (the action seam; C4 is a J-38 leg)
- B-29 impact_radius/call_flow/symbol_context traversal (the ego-graph source; NOT re-authored)
- RRF k=60 fusion (the dataflow channel folds in; NOT re-authored)
reachable_from:
- a real `elai` retrieval query (RRF cascade with the dataflow channel on)  [PRODUCTION]
- a mid-task executor tool call to the ego-graph retrieval action  [PRODUCTION via J-38/C-13 tool seam]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO new graph construction (ELAI's SCIP/DKG/PageRank is already-shipped-equivalent; R5 — reuse the shipped edges)
- NO RRF channel that returns rank-k filler when it has no relevant dataflow neighbor (the B6 RRF-noise critique: contribute nothing, not noise)
- NO ego-graph serialization that blows the context budget (depth<=2 cap, the 8-slot allocator owns the budget)
- NO RRF-stage removal (this ADDS a channel; pruning is J-75 + user-gated)
- NO new default-OFF ELAI_*_ENABLE without a wire-dark-allowlist row + promotion plan (R9/W-2)
frontend_pair: retrieval-channel breakdown on KnowledgeDrawer (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "a real `elai` retrieval query for a symbol reachable only via a dataflow dependency (def->use across an assignment chain), run through the RRF cascade with the dataflow channel on; AND a mid-task executor calling the ego-graph action on a symbol discovered during reasoning"
- expect_ssb: "the E-40/QueryTrace channel-source field shows the dataflow channel contributed the winning candidate; J-38 RetrievalLoopEvent Kind=GraphNeighborFused / EgoGraphSerialized on the mid-task call"
- expect_artifact: "the dataflow-reachable symbol appears in the RRF top-k where the syntactic channels missed it; the ego-graph action returns the k-hop neighborhood as flat context. NEGATIVE: the dataflow channel with no relevant neighbor contributes nothing (not rank-k noise); the ego-graph action respects the depth<=2 + budget cap."

## Grounding
- CodeRAG-system arXiv:2509.16112 (workflow/papers/coderag-system-interface.md): AST dataflow as a third retrieval path, +4.32 EM solo (df+s 44.21 vs s 39.89) — DESIGN-rationale (ID-5).
- RepoGraph arXiv:2410.14684 (workflow/papers/repograph-interface.md, verdict not-new-graph/new-interface-layer): the gap is the generation-time interface, NOT graph construction. NOTE W2: the report's "+32.8% relative average" != the paper's Table-2 +8.6% (Agentless); use the paper number.
- CodeScaleBench/Sourcegraph (workflow/papers/codescalebench-interface.md): P@5 0.140->0.478, file recall 0.127->0.277, cost $0.73->$0.51, exec -38% from wiring nav-tools as agent actions — production evidence, DESIGN-rationale (HD-1).
- ELAI seams codegraph-confirmed 2026-06-23: dataflow edges feed impact not RRF; no symbol_graph_search symbol (context-retrieval-research-2026-06-23.md C3/C4).
- LIVE SSB ~269/578 (post-J-66; grep 2026-06-22). re-grep at edit (W-9).

## COMPOSES-WITH
E-40 (the 5th RRF ranker seam C3 folds into), J-38 (the mid-task callable seam C4 IS — author C4 as a J-38 leg), symbol_graph/dataflow.rs (the shipped edges), B-29 impact API (the ego-graph traversal source), RRF cascade (UNCHANGED), J-18/J-72 (the hardened/adapted cascade these channels join). Restates none.

## Open questions / risks
- C4-into-J-38 vs standalone: the cleanest is to author C4 as a J-38 leg (same seam, GD-1 compose-not-duplicate); if J-38 has not landed at edit, ship C4's serializer as the J-38 substrate and cross-link. decide at edit.
- dataflow channel precision: a naive dataflow walk can flood candidates; the channel must rank by edge relevance not emit every reachable node (the same B6 RRF-noise discipline J-18 names).

End of J-74.

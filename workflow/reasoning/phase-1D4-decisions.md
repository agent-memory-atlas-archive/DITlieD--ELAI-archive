Phase 1D.4 decisions

What: implement DkgSnapshot::from_engine body in pagerank/mod.rs
Key decision: all edge types map to weight=1.0 (no per-type differentiation). Grounded: all test fixtures use 1.0; spec does not prescribe per-type weights in from_engine; sqrt scaling and active-file boost happen downstream in build_adjacency_matrix.
Orphan definition: node with zero inbound AND zero outbound edges gets self-edge (i,i,0.1). Grounded: T16 fixture — 3 nodes, edge (0,1), only node 2 (no edges at all) gets self-edge. Node 0 (source-only, no inbound) does NOT get self-edge.
ref_count: derived from counting inbound edges per node_id. DkgNodeInfo has no ref_count field.
Expected test outcome: all 65 existing tests remain green, 0 warnings.
Unknowns: whether any integration test outside the file exercises from_engine (grep found none).

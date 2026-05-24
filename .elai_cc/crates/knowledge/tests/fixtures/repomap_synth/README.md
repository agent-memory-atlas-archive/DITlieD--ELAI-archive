repomap_synth — C-38 synthetic codebase

Purpose: drive the C-38 PageRank production contract test
(crates/contracts/tests/c38_pagerank_production_contract.rs) with a
small, deterministic codebase whose top-ranked PageRank symbol is
known a priori. The fixture stays under 2 MB so it is safe to ship
inside the repo.

Structure (50+ files; one Python file per node):

  hub_module.py        — defines a single function `hub_root` that is
                         imported and called by every file in the
                         caller_*.py group. By symbol-graph
                         construction, `hub_root` ends up with the
                         highest incoming-reference count in the DKG,
                         which is what PageRank's personalized
                         iteration concentrates rank on.

  caller_NN.py (00-29) — 30 caller modules. Each imports `hub_root`
                         from `hub_module` and calls it once inside a
                         per-file local function `caller_NN_local`.
                         These provide the 30 incoming edges to
                         `hub_module.hub_root`.

  mid_NN.py (00-09)    — 10 mid-tier connector modules. Each imports
                         `hub_root` from `hub_module` AND defines a
                         function `mid_NN_func` that is in turn
                         called by leaf_NN.py at the same index.

  leaf_NN.py (00-09)   — 10 leaf caller modules. Each imports
                         `mid_NN_func` from `mid_NN` and calls it. No
                         file imports from leaf_*.

Total: 1 + 30 + 10 + 10 = 51 .py files.

Expected top-3 after personalized PageRank with `task="hub_root"`:

  1. hub_module.hub_root  (40 incoming references = 30 caller + 10 mid)
  2. mid_00.mid_00_func   (1 incoming) ... mid_09.mid_09_func
                          (each gets one leaf caller; ties broken
                          deterministically by node insertion order)

The contract test asserts only that `hub_root` is the top-1 element
in the ranked output. It does NOT assert ordering of the mid_*
group, because PageRank rank ties are sensitive to floating-point
ordering and we want the contract test to be deterministic across
target architectures.

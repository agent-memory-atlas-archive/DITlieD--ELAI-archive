Phase 1E Step 4 decisions

What: Replace todo!() in CatRag::compute_overlay with full symbolic anchoring + edge weighting + key-fact enhancement algorithm.
Key decision: Heuristic edge scoring with base=5 (references tier) so phi(5)=0.25 guarantees non-zero deltas for seed edges. PageRank scores always contribute to reset_distribution even without symbolic matches.
Expected test outcome: Tests a/b/c flip from should_panic to green. Tests d/e remain green. 5/5 pass.
Unknowns: None significant. Algorithm is fully specified in prompt-transform output and CatRAG paper adaptation.

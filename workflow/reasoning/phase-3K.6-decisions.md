Phase 3K.6: A-RAG Keyword Scoring + Progressive Disclosure

What: Replace arag.rs stubs with keyword scoring (frequency * keyword_len / sqrt(char_len)), semantic search (chunk-level embedding norm with 3x oversampling), and progressive disclosure (sentence-level snippets, not full chunk text).

Key decision: Store chunks directly on AragOrchestrator (Vec<AragChunk>) rather than holding references to Bm25Index/VectorIndexer. Bm25Index has IndexWriter (!Sync) making Arc infeasible. AragOrchestrator::new(AragConfig) signature is frozen by contract. Solution: add_chunk() setter method, default empty. Pipeline can populate before querying.

Length normalization: Added / sqrt(char_len) to keyword scoring formula to prevent long-file dominance. Base A-RAG formula (arag-interface.md L72-77) does not include normalization, but adversarial requirement demands short chunks with same hit count score higher. sqrt normalization is standard BM25-family approach.

Expected test outcomes: 8/8 arag tests pass, contract test compiles, benchmark regression gate holds (Recall@5=0.6333 >= 0.601, MRR=0.4145 >= 0.391).

Unknowns: Semantic search currently uses embedding vector norm as proxy score (no query embedding available without a real encoder). Production use requires dual-vector encoding from Phase 1C to be wired in. This is acceptable for 3K.6 scope.

Phase 1H.4 SPLADE Measurement Results
Measured: 2026-04-28 | Session: s-27276

MEASUREMENT STATUS: COMPLETE

Model Comparison Table (2026-04-28):
  Candidate       | p50_latency_ms | top50_term_jaccard | decision
  A-full          | 15             | baseline           | DROP
  A-quant (int8)  | N/A (not staged at models/splade-quant) | N/A     | skipped
  B (splade-v3)   | N/A (not staged at models/splade-v3)    | N/A     | skipped

Vocabulary Fitness (4a):
  Coverage threshold: 0.3 (30% of code tokens as single WordPiece)
  Measured: vocab_coverage_code = 0.5795 (58.0%)
  Status: PASS — coverage >= 40% threshold; vocabulary adequate for SPLADE expansion
  Tokens sampled: 88 code-specific tokens
  Sample breakdown (single_token=false for all listed):
    fn, impl, async, HashMap, RwLock, mpsc, fibonacci, quicksort,
    ErrorHandler, Serializable, deserialize, ConcurrencyManager,
    spawn_worker, backoff_ms

Model Variant Selection (4b):
  Candidate A (full) p50 latency: 15ms (5-run measurements: 16, 15, 15, 15, 15ms)
  p95 latency: 16ms (observed max from 5 runs)
  Model load time: ~0.94s (estimated from wall-time minus query time)
  Candidate A-quant: SKIPPED (models/splade-quant not staged)
  Candidate B (splade-v3): SKIPPED (models/splade-v3 not staged)

Recall Measurement (4c):
  Corpus: 100 synthetic Rust code files
  Queries: 50 total
  BM25 recall=0 excluded from delta: 22 queries
  Non-zero pairs for Wilcoxon: 1 (need >= 20 for valid significance test)
  WARNING: insufficient paired differences (N=1, need >=20) — defaults to DROP
  mean_recall_delta: 0.00% (std: 0.00%)
  Wilcoxon W=0.00, p=1.000000, n_pairs=1

Per-query breakdown (all 50):
  Q01: bm25=0.33 splade=0.33 delta=0.0% query="error handling"
  Q02: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="concurrent access"
  Q03: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="memory management"
  Q04: bm25=0.50 splade=0.50 delta=0.0% query="async execution"
  Q05: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="serialization"
  Q06: bm25=0.50 splade=0.50 delta=0.0% query="string processing"
  Q07: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="file operations"
  Q08: bm25=0.00 splade=1.00 delta=N/A (BM25=0) query="logging"
  Q09: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="network communication"
  Q10: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="testing patterns"
  Q11: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="configuration loading"
  Q12: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="data validation"
  Q13: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="caching strategy"
  Q14: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="authentication flow"
  Q15: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="database operations"
  Q16: bm25=1.00 splade=1.00 delta=0.0% query="sort implementation"
  Q17: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="tree traversal"
  Q18: bm25=1.00 splade=1.00 delta=0.0% query="hash computation"
  Q19: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="queue processing"
  Q20: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="type conversion"
  Q21: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="iterator chaining"
  Q22: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="pattern matching"
  Q23: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="buffer management"
  Q24: bm25=1.00 splade=1.00 delta=0.0% query="graph algorithms"
  Q25: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="compression"
  Q26: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="rate limiting"
  Q27: bm25=1.00 splade=1.00 delta=0.0% query="retry logic"
  Q28: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="resource pool"
  Q29: bm25=1.00 splade=1.00 delta=0.0% query="event dispatch"
  Q30: bm25=1.00 splade=1.00 delta=0.0% query="permission check"
  Q31: bm25=1.00 splade=1.00 delta=0.0% query="sum_vec"
  Q32: bm25=1.00 splade=1.00 delta=0.0% query="fibonacci"
  Q33: bm25=1.00 splade=1.00 delta=0.0% query="base64_encode"
  Q34: bm25=1.00 splade=1.00 delta=0.0% query="generate_uuid"
  Q35: bm25=1.00 splade=1.00 delta=0.0% query="parse_json"
  Q36: bm25=1.00 splade=1.00 delta=0.0% query="find_emails regex"
  Q37: bm25=1.00 splade=1.00 delta=0.0% query="format_timestamp"
  Q38: bm25=1.00 splade=1.00 delta=0.0% query="binary_search"
  Q39: bm25=1.00 splade=1.00 delta=0.0% query="crc32 checksum"
  Q40: bm25=1.00 splade=1.00 delta=0.0% query="matrix_multiply"
  Q41: bm25=1.00 splade=1.00 delta=0.0% query="impl Display for"
  Q42: bm25=1.00 splade=1.00 delta=0.0% query="derive Debug Clone"
  Q43: bm25=1.00 splade=1.00 delta=0.0% query="unsafe raw pointer"
  Q44: bm25=1.00 splade=1.00 delta=0.0% query="where clause trait bound"
  Q45: bm25=0.00 splade=0.00 delta=N/A (BM25=0) query="lifetime annotation"
  Q46: bm25=1.00 splade=1.00 delta=0.0% query="enum variants data"
  Q47: bm25=1.00 splade=1.00 delta=0.0% query="closure FnMut"
  Q48: bm25=1.00 splade=1.00 delta=0.0% query="mpsc channel"
  Q49: bm25=1.00 splade=1.00 delta=0.0% query="Pin Box Future"
  Q50: bm25=1.00 splade=1.00 delta=0.0% query="PhantomData marker"

Decision: DROP
  Criterion: KEEP requires mean_delta >= 5% AND p_value < 0.05
  Measured: mean_delta = 0.00%, p_value = 1.000000, n_pairs = 1
  Spec clause: n < 20 pairs defaults to DROP
  Result: DROP (insufficient paired differences + zero mean delta)

Note: 22 of 50 queries had BM25 recall=0 and were excluded from delta.
Only 1 query (Q08: "logging") had bm25=0 but splade=1.00 — this is a SPLADE win not
captured in the delta because BM25=0 queries are excluded per spec. This is an
interesting signal but insufficient to override the DROP decision.

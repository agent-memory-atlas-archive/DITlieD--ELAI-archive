E-54 held-out store layout (eval-corpus crate)

Layout shape (per workflow/reasoning/E-54-decisions.md DECISION-1+2+3):

  held-out/
    {family_snake}/                        per-family directory shard
      YYYY-MM-DD/                          daily sub-shard (NTFS perf cliff defense)
        {task-id}.toml                     canonical task envelope
    _index/
      {family_snake}.jsonl                 append-only summary index (one row per task)
    _quarantine/
      {family_snake}/                      failed-validation tasks
        {task-id}.toml                     raw body (validation failure)
        {task-id}.reason                   text reason
    _audit/
      knowledge-qa-YYYY-MM-DD.jsonl        OKBench-style dynamic knowledge-QA log
    from-prod/                             OPT-IN-only silent grade probes
      YYYY-MM-DD/
        {turn-hash-prefix}.toml            opt-in prod-traffic-derived probe

Canonical family snake-case wire labels (per crates/orchestrator/src/canary.rs:71-90):
  code_fix
  math_proof
  prose_reason
  planning_decomposition                   (NOT "planning" -- canonical D-7 label)
  tool_use                                 (NOT "knowledge_qa" -- routed via ToolUse-flavored tasks per DECISION-1)

Mandatory TOML schema fields (per crates/eval-corpus/src/task_schema.rs):
  id                                       canonical task id (matches filename)
  family                                   one of the 5 canonical families above
  created_at                               chrono::DateTime<Utc>
  cutoffs                                  HashMap<model_id, "YYYY-MM">; non-empty (per arXiv:2403.07974)
  problem                                  prompt body
  reference_solution                       canonical solution
  tests                                    Vec<ExecutableTest>; non-empty (per arXiv:2503.02951)
  provenance                               { generator, prompt_id?, date } (per arXiv:2412.08905)
  embedding_hash                           blake3 16-hex prefix of normalized problem text
  ngram_hash                               blake3 16-hex prefix of canonicalized 13-gram set bytes

Optional fields:
  framing_pair                             { eval, deploy } -- D-23 matched-pair sycophancy probes (per arXiv:2510.20487)
  perturbed_pair                           { original_true, subtly_false, expected_answer } -- BrokenMath-style (per arXiv:2510.04721)

Contamination defenses (per crates/eval-corpus/src/ingest.rs):
  13-gram K=4 consecutive grams overlap    REJECT (per arXiv:2311.04850)
  embedding cosine >= 0.85 default         REJECT (initial default per FIX-2 Open Q1 -- ELAI corpus calibration pending)
  empty tests / cutoffs                    REJECT
  banned tokens in framing_pair.deploy     REJECT ('benchmark' / 'evaluation' / 'evaluate' / 'test suite')
  reference_solution fails any test        REJECT (per arXiv:2411.04905; sandbox RUNPOD-PENDING E-54.4)
  tests pass on broken solution            REJECT (same; tests too weak)
  generator-rotation                       REJECT batch if <3 generators OR any generator <30%
  flat held-out layout                     REJECT (per E-54-decisions.md DECISION-3)

SSB emission gate: ELAI_EVAL_CORPUS_ENABLE=1 (wire-dark default).

Migration of legacy flat held-out/{YYYY-MM}/*.jsonl rows:
  bin: cargo run -p elai-eval-corpus --bin migrate_held_out -- <root>
  Idempotent; most legacy rows quarantine on first run (no tests/cutoffs/family).

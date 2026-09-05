E-54 eval-corpus repair pre-deploy -- decisions

Scope-id: E-54_eval_corpus_design
Session: s-25120 completion-drive batch 4
Plan: docs/thefinalglance/E-plan/E-54_eval_corpus_design.md
SCHEMA_VERSION: 166 -> 167 (single-atomic, Pattern-2 19th application)
TOTAL_VARIANTS: 439 -> 440
Ship state: host-side scaffolding GREEN; live ingest + sandbox + LoRA + sidecar embeddings RUNPOD-PENDING.

DECISION-1: 5th family drift -- ALIGN with canonical CanaryFamily, NOT the plan-doc.

The plan-doc claims TaskFamily = {CodeFix, MathProof, ProseReason, Planning, KnowledgeQa}.
The canonical surface at crates/orchestrator/src/canary.rs:71-90 is:
  CanaryFamily = {CodeFix, MathProof, ProseReason, PlanningDecomposition, ToolUse}

Resolution: eval-corpus consumes the FROZEN CanaryFamily 5-arm directly.
  pub use elai_orchestrator::canary::CanaryFamily as TaskFamily;

KnowledgeQa is NOT a separate family. Web-grounded MCQs (Task 9 dynamic
knowledge-QA) are routed via ToolUse-flavored tasks since SearXNG-fetched
MCQs are interaction-driven (= ToolUse semantic), not pure recall. The
plan-doc's "knowledge-qa" channel is preserved as a dynamic-only,
non-persisted task source on the ToolUse family axis.

Rationale: the orchestrator brief (canary.rs:84-89) records the original
decision that PrmBench knowledge rows from D-19 already cover the recall
axis; ToolUse covers the interaction axis; KnowledgeQa would be redundant.
Aligning E-54 with CanaryFamily prevents the parallel-enum drift the
context-compiler flagged as HIGH severity.

DECISION-2: held_out primitives -- RE-EXPORT, NOT DUPLICATE.

crates/orchestrator/src/held_out.rs at L530-595 already owns
normalize_text, content_hash, ngram_filter, jaccard_overlap,
ngram_signature_{to,from}_bytes. Two resolution paths considered:
  (a) eval-corpus depends on orchestrator
  (b) extract primitives into a new eval-corpus-primitives crate

Picked (a). orchestrator already pulls in chrono / blake3 / serde / toml,
so the dep is graph-cheap. orchestrator does NOT depend on eval-corpus
(checked), so no cycle. Re-export via:
  pub use elai_orchestrator::held_out::{
      content_hash, jaccard_overlap, ngram_filter, normalize_text,
  };

Avoids determinism drift; eval-corpus consumers get a single canonical
primitive surface.

DECISION-3: per-family directory shards -- canonical snake-case labels.

The plan's hyphenated "code-fix" / "math-proof" / "knowledge-qa" directory
names are REJECTED in favour of the CanaryFamily::as_str() snake-case wire
labels (code_fix / math_proof / prose_reason / planning_decomposition /
tool_use). Reason: this is the same wire format CanaryFamily already emits
on SSB rows + dashboard panels; using a parallel hyphenated format would
fragment the wire surface across crates.

Layout (from .elai_cc/held-out/_README.md):
  held-out/{family_snake}/{YYYY-MM-DD}/{task-id}.toml
  held-out/_index/{family_snake}.jsonl
  held-out/_quarantine/{family_snake}/
  held-out/_audit/knowledge-qa-{YYYY-MM-DD}.jsonl
  held-out/from-prod/{YYYY-MM-DD}/{turn-hash}.toml   (OPT-IN only)

Daily sub-shards cap per-dir file count at SHARD_MAX_FILES_PER_DATE=500;
10 distinct dates cover the documented NTFS perf cliff (~5K files / dir).
Open Q4 NTFS actual-perf measurement RUNPOD-PENDING (host benchmark).

DECISION-4: cosine threshold 0.85 -- ELAI-empirical INITIAL DEFAULT (FIX-2).

The plan cites arXiv:2511.17602 (Hierarchical Contamination Detection,
F1=0.76 at semantic level) as grounding for the 0.85 default. Paper
verification confirms F1=0.76 but does NOT specify any cosine threshold
of 0.85. The 0.85 default is an ELAI engineering anchor pending Open Q1
precision/recall calibration on ELAI's actual corpus.

E-54 ships 0.85 as a CONFIGURABLE default; the cosine filter is enabled by
default (contract `cosine_filter_never_disabled`) but the threshold is
documented as Open Q1 tunable. First production session measures
precision/recall at 0.80 / 0.85 / 0.90 and updates this decision.

DECISION-5: KOR-orthogonality -- ELAI operationalization, not direct from paper.

arXiv:2410.06526 KOR-Bench defines knowledge-orthogonality via novel-rule
application (the paper's own protocol). ELAI's Task 5 operationalizes via
cross-family transfer accuracy delta (train adapter on family A; measure
accuracy on family B; abort if delta > 10pp above random_baseline). This
is an ELAI hypothesis layered on top of the paper's concept, not the
paper's exact protocol.

E-54 ships the surface (cross_family_transfer + TransferResult +
KOR_DELTA_THRESHOLD_PP=10.0); live LoRA training RUNPOD-PENDING (TODO[E-54.5]).
Until the live harness lands, the host-side
heuristic_orthogonality_score gives the dashboard a stop-gap signal.

DECISION-6: matched-pair framing -- Sonnet 4.5 stat decoupled (FIX-4).

The plan claims "Sonnet 4.5 verbalizes 'I am being evaluated' >80%" per
arXiv:2510.20487. Paper verification: 2510.20487 (Steering Eval-Aware
Models) uses a toy model with type-hint signals; the Sonnet 4.5 number is
from Anthropic's own reports, NOT the paper. The matched-pair framing
design is still sound (the paper grounds the eval-aware-as-measurable
concept); the specific Sonnet 4.5 statistic is decoupled in E-54
documentation as "Anthropic internal source".

DECISION-7: ConStat fallback -- ship 2-sigma detector (FIX-5-derived).

The plan cites ConStat (no arXiv ID; per-task ground-truth required).
ELAI cannot ship per-task reference performance at scale. Task 8 ships a
simpler 2-sigma deviation detector as the default; the ConStat path
extends the same surface when reference snapshots are available.

Default thresholds: DEFAULT_SIGMA_THRESHOLD=2.0; DEFAULT_JUMP_THRESHOLD_PP=20.0.
Verdict carries fallback_2sigma=true so the dashboard surfaces the regime.

DECISION-8: dynamic knowledge-QA audit log -- MANDATORY.

Every batch of OKBench-style daily MCQ generation MUST write a row to
held-out/_audit/knowledge-qa-{YYYY-MM-DD}.jsonl with the SearXNG query
hash (blake3 16-hex) + timestamp + URL + topic seed + generated MCQ count
+ canonical family wire label ("tool_use"). No silent generation;
contract test `dynamic_knowledge_qa_logs_searxng_query` enforces.

Live SearXNG hit + LLM-MCQ generation RUNPOD-PENDING (TODO[E-54.9]).

DECISION-9: embedding model deps -- BGE-M3 + Nomic Embed v2 via sidecar HTTP.

Both HF model deps verified per CLAUDE.md anti-slopsquatting:
  BAAI/bge-m3                            https://huggingface.co/BAAI/bge-m3                (1024-dim, MIT, last update <12 months)
  nomic-ai/nomic-embed-text-v2-moe       https://huggingface.co/nomic-ai/nomic-embed-text-v2-moe  (768-dim, Apache-2.0, last update <12 months)

The Rust eval-corpus crate has NO direct HF dependency; embeddings are
fetched via the sidecar's POST /v1/embeddings endpoint (new module:
python/elai_inference/embeddings.py). Lazy load + opt-in env-var gate
(ELAI_EVAL_CORPUS_EMBEDDINGS=1) defers the GPU cost until the user opts
in. Feature-gate `cosine-filter` on the Rust side controls the filter
itself (default ON; opt-out via `--no-default-features`).

DECISION-10: sandbox test_executor -- RUNPOD-PENDING stub (E-54.4).

crates/sandbox/ does not exist. Task 4 ships a clearly-marked stub
returning sandbox_missing=true; defaults-open on reference, conservative
on broken. Registered under workflow/sessions/s-25120/stubs.md with
will_resolve_in pointer.

DECISION-11: JudgeBench cite -- replace WRONG_ID 2412.12509 with 2410.12784 (FIX-1).

arXiv:2412.12509 is "Can You Trust LLM Judgments? Reliability of LLM-as-a-
Judge" (Schroeder & Wood-Doughty), about single-sample reliability. The
plan claims "JudgeBench; only largest LLM-judges hit acceptable
agreement", which maps to arXiv:2410.12784 (Tan et al., ICLR 2025 -- the
actual JudgeBench paper). All E-54 cites + decisions + SSB header
comments now use arXiv:2410.12784.

DECISION-12: arXiv:2602.12413 re-triage -- CENTRAL in E-54 domain (FIX-6).

memory-bank gotcha E7_PAPER_2602_12413_ZERO_RELEVANCE flagged this paper
as ZERO_RELEVANCE for the E-7 darwinian domain. That ruling was correct
FOR E-7 (different domain). For E-54 (contamination detection -- the
actual domain of the paper), it is the PRIMARY evidence for the
embedding-cosine filter. Triage outcome: CENTRAL. Implementer does NOT
inherit the E-7 ZERO_RELEVANCE label.

DECISION-13: LiveCodeBench attribution -- release-date stamps, NOT Docker (FIX-3).

The plan's text claims LiveCodeBench v6 uses "monthly Docker-per-task";
Docker-per-task is SWE-bench-Live (arXiv:2505.23419). LiveCodeBench uses
release-date stamps. The cite for the `cutoffs` mandatory field stays
correct (arXiv:2403.07974); the Docker phrasing is fixed in the SSB
header comment + the held-out _README.

DECISION-14: openmathinstruct-2 cite -- DROPPED (FIX-5).

The plan cites arXiv:2410.01560 OpenMathInstruct-2 for "LLM-pipeline
decontamination step". Paper verification: paper focuses on math data
synthesis, NOT decontamination. The decontam-after-synthesis rationale is
already covered by arXiv:2509.00072 Test-of-Time + arXiv:2602.12413 Soft
Contamination; OpenMathInstruct-2 is removed from E-54 citations to avoid
the WEAK grounding label.

DECISION-15: wire-dark gate -- ELAI_EVAL_CORPUS_ENABLE.

All 14 CorpusEventPhase variants emit only when ELAI_EVAL_CORPUS_ENABLE=1
is set in the environment. Default-off matches the Pattern-2 precedent
(E-10 ELAI_CRITIC_OF_DESIGN_ENABLE, E-43 ELAI_SKILL_SYSTEM_ENABLE, E-19
ELAI_REASONING_QUALITY_PROBE_ENABLE). The dashboard panels read all rows
unconditionally and return `known: false` when no rows exist.

OPEN QUESTIONS (RUNPOD-PENDING follow-ups)

Q1 (DECISION-4 cosine threshold): measure precision/recall on ELAI's
actual corpus at 0.80 / 0.85 / 0.90. Update default + the dashboard echo
of `cosine_threshold` field. First production session.

Q2 (generator-rotation): does diversity matter at 500-task scale ? Phi-4
data is 400B tokens. Measure same 500-task corpus from 1 vs 3 vs 5
generators, score variance on Qwen2.5-Coder-3B perf.

Q3 (matched-pair false-positive rate): the 10pp threshold is from
frontier-model studies. Qwen2.5-Coder-3B may show stochastic gap
without deception. If false-positive rate >X% on a held-out non-deceptive
suite, raise threshold or add p-value gate over N trials.

Q4 (NTFS perf cliff measurement): benchmark per-shard read latency at
1K / 5K / 10K files under the per-(family, date) layout. The 500-file cap
+ daily rotation is currently a conservative upper bound from Microsoft
NTFS docs; measure to confirm.

Q5 (ConStat reference-model snapshot strategy): is there a path to
per-task reference performance at ELAI scale ? If yes, Task 8 promotes
from 2-sigma fallback to full ConStat. Otherwise, the fallback stays
canonical.

RUNPOD-PENDING TASKS

  E-54.1 -- live BGE-M3 / Nomic Embed v2 sidecar lazy load (embeddings.py)
  E-54.4 -- crates/sandbox/ wire + test_executor real exec path
  E-54.5 -- LoRA cross-family transfer training harness (kor_orthogonality)
  E-54.6 -- live two-shot LLM invocation for matched-pair probes
  E-54.9 -- live SearXNG hit + LLM-MCQ generation (dynamic_knowledge_qa)

Files shipped this commit

  CREATE crates/eval-corpus/Cargo.toml
  CREATE crates/eval-corpus/src/lib.rs
  CREATE crates/eval-corpus/src/task_schema.rs
  CREATE crates/eval-corpus/src/storage.rs
  CREATE crates/eval-corpus/src/ingest.rs
  CREATE crates/eval-corpus/src/test_executor.rs
  CREATE crates/eval-corpus/src/kor_orthogonality.rs
  CREATE crates/eval-corpus/src/sycophancy_matched_pair.rs
  CREATE crates/eval-corpus/src/generator_rotation.rs
  CREATE crates/eval-corpus/src/perf_delta_detector.rs
  CREATE crates/eval-corpus/src/dynamic_knowledge_qa.rs
  CREATE crates/eval-corpus/src/prod_traffic_grade.rs
  CREATE crates/eval-corpus/src/bin/migrate_held_out.rs
  CREATE crates/eval-corpus/tests/storage_uses_family_shards.rs
  CREATE crates/contracts/tests/e54_eval_corpus_ssb_contract.rs
  CREATE crates/dashboard-v2/src-tauri/src/commands_e54.rs
  CREATE crates/dashboard-v2/src/lib/components/EvalCorpusPanel.svelte
  CREATE crates/dashboard-v2/src/lib/components/SycophancyMatchedPairPanel.svelte
  CREATE python/elai_inference/embeddings.py
  CREATE held-out/_README.md
  CREATE workflow/reasoning/E-54-decisions.md
  MODIFY Cargo.toml workspace member registration
  MODIFY crates/contracts/Cargo.toml (eval-corpus + chrono dev-deps)
  MODIFY crates/orchestrator/src/ssb.rs (SCHEMA 166->167, EvalCorpusEvent + CorpusEventPhase + EvalCorpusEventPayload)
  MODIFY crates/orchestrator/tests/action_type_lock.rs (TOTAL_VARIANTS 439->440)
  MODIFY crates/orchestrator/tests/action_type_lock.golden (eval_corpus_event inserted)
  MODIFY crates/dashboard-v2/src-tauri/src/lib.rs (commands_e54 module + 3 invoke_handler entries)
  MODIFY crates/dashboard-v2/src/lib/windows/MonitorWindow.svelte (panel mounts)

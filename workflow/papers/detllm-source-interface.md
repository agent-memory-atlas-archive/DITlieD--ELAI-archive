Source: detllm (github.com/tommasocerruti/detllm v0.1.1, Apache-2.0)
Reliability: MEDIUM (reviewed source, shallow git clone, commit tag v0.1.1)
Files examined:
  detllm/core/models.py
  detllm/core/artifacts.py
  detllm/core/capabilities.py
  detllm/core/deterministic.py
  detllm/core/env.py
  detllm/backends/base.py
  detllm/diff/diff.py
  detllm/trace/io.py
  detllm/report/report.py
  detllm/api.py
  detllm/cli/main.py
  detllm/schemas/env_snapshot.json
  detllm/schemas/run_config.json
  detllm/schemas/determinism_applied.json
  detllm/schemas/trace_row.json
  detllm/schemas/report.json
  docs/determinism_boundary.md
RAG ingest: FAILED (IPC timeout, known reranker crash on this machine)

ELAI steal intent: schema, artifact layout, 3-tier guarantee taxonomy for Rust-side RRA + model-drift canary.


ARTIFACT FILE LAYOUT (directory emitted per run/check command)
<out_dir>/
  env.json                         -- EnvSnapshot (single run); check also writes envs/run_N.json per run
  run_config.json                  -- RunConfig
  determinism_applied.json         -- DeterminismAppliedRecord (first run only for check)
  trace.jsonl                      -- one TokenTraceRow per line (run command only)
  traces/run_N.jsonl               -- per-run traces (check command, N = 0..runs-1)
  traces/batch_B.jsonl             -- per-vary-batch traces (check command, B = batch size)
  diffs/first_divergence.json      -- written ONLY when a divergence is found
  report.json                      -- Report (wrapped artifact)
  report.txt                       -- human-readable text render


ARTIFACT 1: env.json
Dataclass: EnvSnapshot (models.py:10, frozen=True)
Schema: schemas/env_snapshot.json, schema_version "1.0", additionalProperties: true

Required fields (all string or object unless noted):
  schema_version        | str        | "1.0"
  detllm_version        | str        | e.g. "0.1.1"
  artifact_type         | const str  | "env_snapshot"
  python                | object     | {version: str, implementation: str, executable: str}
  platform              | object     | {system: str, release: str, machine: str}
  torch                 | object     | {version: str|null}
  transformers          | object     | {version: str|null}
  device                | object|null| {cuda_available: bool, mps_available: bool, device_type: str,
                                         cuda_version?: str, device_count?: int, devices?: [str]}
  env_vars              | object     | keys: CUBLAS_WORKSPACE_CONFIG, CUDNN_DETERMINISTIC,
                                         CUDNN_BENCHMARK, CUDA_VISIBLE_DEVICES, OMP_NUM_THREADS,
                                         MKL_NUM_THREADS, PYTHONHASHSEED (values: str|null)
  fingerprint           | str        | sha256 of canonical JSON(sort_keys=True, no spaces) of snapshot
                                       BEFORE fingerprint field is added (env.py:106)

Invariant: fingerprint = sha256(json.dumps(snapshot_without_fingerprint, sort_keys=True, separators=(",",":")).encode())
Fingerprint is computed BEFORE fingerprint key is inserted into the dict (env.py:105-106).
Stability: the env_vars dict captures exactly the 7 env vars listed above; None if unset.


ARTIFACT 2: run_config.json
Dataclass: RunConfig (models.py:31, frozen=True)
Schema: schemas/run_config.json, schema_version "1.0", additionalProperties: true

Required fields:
  schema_version        | str        | "1.0"
  detllm_version        | str
  artifact_type         | const str  | "run_config"
  backend               | str        | "hf" | "vllm"
  tier_requested        | int        | 0 | 1 | 2
  tier_effective        | int        | 0 | 1 | 2 (may be lower than tier_requested after downgrade)
  mode                  | str        | "strict" | "best-effort"
  model                 | str        | model id or path
  dtype                 | str        | "float32" | "float16" etc.
  device                | str        | "cpu" | "cuda" | "mps" etc.
  decoding              | object     | {max_new_tokens: int, do_sample: bool, temperature: float,
                                         top_p: float, top_k: int}
  batch_size            | int        | >= 1
  vary_batch            | [int]      | list of extra batch sizes tested; [] if none
  tokenizer             | object     | {id: str, revision: str|null}
  generation_context    | object     | {device_snapshot: object|null}

Built by _build_run_config (cli/main.py:564); wrapped via _wrap_artifact (cli/main.py:598).
generation_context.device_snapshot is the device sub-object from env_snapshot, not a full snapshot.


ARTIFACT 3: determinism_applied.json
Dataclass: DeterminismAppliedRecord (models.py:57, frozen=True)
Also: DeterminismApplied (deterministic.py:13, mutable during context manager)
Schema: schemas/determinism_applied.json, schema_version "1.0", additionalProperties: true

Required fields:
  schema_version        | str        | "1.0"
  detllm_version        | str
  artifact_type         | const str  | "determinism_applied"
  tier_requested        | int
  tier_effective        | int        | may be lower; set by DeterministicContext.__enter__ + evaluate_capabilities
  mode                  | str        | "strict" | "best-effort"
  seed                  | int|null   | seed passed to torch.manual_seed and random.seed; null means no seed
  seed_controls         | object     | {python_random: bool, torch_manual_seed?: bool}
  torch_controls        | object     | {use_deterministic_algorithms?: bool}
  env_controls          | object     | {CUBLAS_WORKSPACE_CONFIG: str|null}
  downgrades            | array      | [{from: int, to: int, reason: str}]
  warnings              | array      | [str]
  capability_failures   | array      | [{requirement: str, reason: str}]

Controls applied in DeterministicContext.__enter__ (deterministic.py:40):
  1. random.seed(seed or 0) -> seed_controls["python_random"] = True
  2. torch.manual_seed(seed) if seed not None -> seed_controls["torch_manual_seed"] = True
  3. torch.use_deterministic_algorithms(True) -> torch_controls["use_deterministic_algorithms"] = True
  4. env_controls["CUBLAS_WORKSPACE_CONFIG"] = os.environ.get("CUBLAS_WORKSPACE_CONFIG")

On exit (__exit__, deterministic.py:76): RNG state is restored (random.setstate + torch.random.set_rng_state).
NOTE: env_controls records the VALUE of CUBLAS_WORKSPACE_CONFIG but does NOT set it; the caller must set
this env var before process start if deterministic CUDA matmul is needed.

For check command: determinism_applied.json records the FIRST run only (cli/main.py:362-364, comment says
"expected to be stable across runs").


ARTIFACT 4: trace.jsonl (one line per prompt, JSONL format)
Dataclass: TokenTraceRow (models.py:81, frozen=True)
Schema: schemas/trace_row.json, schema_version absent in schema itself (schema validates per-row)
Required per row:
  prompt_id             | str        | sha256 of prompt text (utf-8 encoded) (cli/main.py:520)
  generated_token_ids   | [int]      | output token id sequence

Optional per row (present when captured):
  input_token_ids       | [int]      | tokenizer output for the prompt
  input_token_ids_hash  | str        | sha256 of JSON(input_token_ids, no spaces) (cli/main.py:524)
  scores                | [float]|null | per-step max logprob; only when tier_effective >= 2
  tokenizer_id          | str|null   | model id used as tokenizer
  decoding_max_new_tokens | int|null
  decoding_do_sample    | bool|null  | always False in current implementation (cli/main.py:543)
  decoding_temperature  | float|null
  decoding_top_p        | float|null
  decoding_top_k        | int|null

prompt_id derivation: hashlib.sha256(prompt.encode("utf-8")).hexdigest()
input_token_ids_hash derivation: hashlib.sha256(json.dumps(ids, separators=(",",":")).encode()).hexdigest()

Rows written with sort_keys=True (trace/io.py:21).
check command writes per-run traces at traces/run_N.jsonl and per-batch-size at traces/batch_B.jsonl.


ARTIFACT 5: report.json
Dataclass: Report (report/report.py:10, frozen=True)
Schema: schemas/report.json, schema_version "1.0", additionalProperties: true
Wrapped via _wrap_artifact, so outer envelope has schema_version + detllm_version + artifact_type.

Required fields:
  schema_version        | str        | "1.0"
  detllm_version        | str
  artifact_type         | const str  | "report"
  status                | str        | "PASS" | "FAIL"
  category              | str        | see category enum below
  details               | object     | varies by command (see below)

Category enum (from code, not schema):
  "PASS"                            -- all runs identical
  "RUN_VARIANCE_FIXED_BATCH"        -- token mismatch across runs at same batch size
  "SCORE_VARIANCE"                  -- score/logprob mismatch across runs
  "GEN_CONTEXT_MISMATCH"            -- prompt_id, tokenizer_id, decoding param, or trace length differs
  "TOKENIZATION_MISMATCH"           -- input_token_ids_hash or tokenizer_id differs
  "BATCH_VARIANCE"                  -- token mismatch when batch size varies (batch_result != PASS)
  "UNSUPPORTED_REQUEST"             -- backend capability gates blocked the requested tier in strict mode
  "ENV_MISMATCH"                    -- env fingerprint differs between runs in check

details object for check command:
  runs                  | int        | number of runs performed
  batch_sizes           | [int]      | vary_batch sizes if any
  baseline_batch_size   | int        | the fixed batch_size used for run-to-run comparison
  first_divergence      | object|null| see first_divergence shape below
  batch_divergence      | object|null| {batch_size: int, first_divergence: object}

details object for UNSUPPORTED_REQUEST:
  runs                  | int
  capability_failures   | [{requirement: str, reason: str}]
  notes                 | [str]

details object for ENV_MISMATCH:
  runs                  | int
  run_index             | int
  baseline_fingerprint  | str
  current_fingerprint   | str


ARTIFACT 6: diffs/first_divergence.json
Written ONLY when a divergence is found (report.json status = "FAIL", divergence != null).
Wrapped via _wrap_artifact("first_divergence", ...).

Envelope:
  schema_version        | str        | "1.0"
  detllm_version        | str
  artifact_type         | const str  | "first_divergence"

Inner payload (the first_divergence dict from DiffResult.first_divergence):

Shape depends on diff category:
  GEN_CONTEXT_MISMATCH (trace length):
    reason              | str        | "trace lengths differ"
    left_len            | int
    right_len           | int

  GEN_CONTEXT_MISMATCH (prompt_id):
    index               | int        | row index in trace
    reason              | str        | "prompt_id mismatch"
    left_prompt_id      | str|null
    right_prompt_id     | str|null

  TOKENIZATION_MISMATCH (hash):
    index               | int
    reason              | str        | "input_token_ids_hash mismatch"
    left_hash           | str|null
    right_hash          | str|null

  TOKENIZATION_MISMATCH (tokenizer_id):
    index               | int
    reason              | str        | "tokenizer_id mismatch"
    left_tokenizer_id   | str|null
    right_tokenizer_id  | str|null

  GEN_CONTEXT_MISMATCH (decoding field):
    index               | int
    reason              | str        | "<field_name> mismatch"
    left_value          | any
    right_value         | any

  RUN_VARIANCE_FIXED_BATCH (token mismatch):
    index               | int        | trace row index
    token_index         | int        | position in generated_token_ids (0-based)
    left_token          | int|null   | token id at that position in left trace
    right_token         | int|null   | token id at that position in right trace

  SCORE_VARIANCE:
    index               | int        | trace row index
    score_index         | int        | position in scores list


TIER DEFINITIONS (implemented in capabilities.py:evaluate_capabilities)

Tier 0 (baseline, no guarantees):
  No capability requirements. Any backend. Outputs captured but no repeatability claimed.
  vllm backend is Tier 0 only (docs/determinism_boundary.md).

Tier 1 (fixed-batch repeatability):
  Requires BackendCapabilities.supports_torch_deterministic = True
    gate: if not satisfied -> capability_failures += {requirement: "torch_deterministic", ...}; tier_effective = min(tier_effective, 0)
  Requires BackendCapabilities.supports_tier1_fixed_batch = True
    gate: if not satisfied -> capability_failures += {requirement: "tier1_fixed_batch", ...}; tier_effective = min(tier_effective, 0)
  Guarantee (when passing): same generated_token_ids across N runs at IDENTICAL batch_size.
  Not guaranteed across different batch sizes even with greedy decoding (GPU math non-associativity).

Tier 2 (scores/logprobs equality, extends Tier 1):
  Requires Tier 1 passing (both torch_deterministic + tier1_fixed_batch).
  Requires BackendCapabilities.supports_scores = True
    gate: if not satisfied -> capability_failures += {requirement: "scores", ...}; tier_effective = min(tier_effective, 1)
  Guarantee (when passing): same generated_token_ids AND same scores across runs.
  Failure path: scores unavailable -> strict mode returns UNSUPPORTED_REQUEST; best-effort downgrades to Tier 1.

UNSUPPORTED_REQUEST trigger:
  mode = "strict" AND capability_failures non-empty (any tier gate failed).
  In best-effort mode: tier_effective is lowered, execution continues with reduced guarantees.
  A DeterminismApplied.downgrades entry is added for each effective tier reduction.


FIRST-DIVERGENCE ALGORITHM (diff/diff.py:diff_traces)

Input: two lists of trace-row dicts (base and other), same length expected.
Algorithm (ordered, first hit returns):
  1. Length check: if len(base) != len(other) -> FAIL/GEN_CONTEXT_MISMATCH, no per-row scan.
  2. Per-row (zip order), for each (left, right) pair at index idx:
     a. prompt_id mismatch -> FAIL/GEN_CONTEXT_MISMATCH
     b. input_token_ids_hash mismatch -> FAIL/TOKENIZATION_MISMATCH
     c. tokenizer_id mismatch (both not-None and different) -> FAIL/TOKENIZATION_MISMATCH
     d. for each of decoding_max_new_tokens, decoding_do_sample, decoding_temperature,
        decoding_top_p, decoding_top_k: mismatch -> FAIL/GEN_CONTEXT_MISMATCH
     e. first_token_divergence(left.generated_token_ids, right.generated_token_ids):
        scan position 0..min(len(a),len(b))-1; if a[i] != b[i] return i;
        if lengths differ return min(len); else return None.
        if not None -> FAIL/RUN_VARIANCE_FIXED_BATCH with {index, token_index, left_token, right_token}
     f. _first_score_divergence(left.scores, right.scores):
        if either scores is None, skip (no score divergence possible);
        else scan 0..min(len)-1; if left[i] != right[i] return i; if lengths differ return min(len).
        if not None -> FAIL/SCORE_VARIANCE with {index, score_index}
  3. If all rows pass all checks -> PASS/PASS, first_divergence=None.

Score comparison is exact float equality (no epsilon). A single floating-point bit difference = SCORE_VARIANCE.

_field_mismatch semantics (diff.py:150): both None -> no mismatch (field absent in both = compatible).
  left=None, right="x" -> mismatch. left="x", right="x" -> no mismatch.

aggregate_diffs (diff.py:158): returns first non-PASS DiffResult from a list; used for check command
  to aggregate run_0 vs run_1..N-1.

Batch-variance measurement:
  base = traces[0] (run_0 at fixed batch_size)
  compared against traces[batch_B] for each B in vary_batch_sizes.
  Comparison is via same diff_traces function; batch_diffs = [(size, diff)] list.
  batch_divergence_detail returns first failing (batch_size, first_divergence) pair.


VARIANCE MEASUREMENT METHODOLOGY (check command, api.py:check)

Run-to-run variance (N runs, same batch_size):
  N = runs parameter (default 3, CLI default 3)
  Each run: fresh DeterministicContext + fresh backend instantiation (per-run reload isolates state)
  Each run writes its own trace: traces/run_N.jsonl
  Comparison: run_0 is baseline; diffs = [diff_traces(traces[0], traces[i]) for i in 1..N-1]
  result = aggregate_diffs(diffs) (first non-PASS wins)
  Comparison keys checked in order: prompt_id, input_token_ids_hash, tokenizer_id, 5 decoding fields,
    generated_token_ids (per token), scores (per score, exact equality)

Batch-size variance (optional, via vary_batch):
  For each B in vary_batch: run once with batch_size=B, write traces/batch_B.jsonl
  Compare batch_B trace against traces[0] (NOT pairwise between batch sizes)
  batch_result = aggregate_diffs([diff for _, diff in batch_diffs])

Env fingerprint check: between runs in check, re-captures env snapshot and compares fingerprint to
  baseline. If fingerprint differs -> immediate FAIL/ENV_MISMATCH (early exit, no diff computed).


BACKEND CAPABILITIES PROTOCOL (backends/base.py:BackendCapabilities)

BackendCapabilities is a frozen dataclass with three bool flags:
  supports_tier1_fixed_batch  | bool  | can repeat outputs at fixed batch size
  supports_scores             | bool  | can capture per-token scores/logprobs
  supports_torch_deterministic | bool | supports torch.use_deterministic_algorithms(True)
  notes                       | [str] | informational notes (copied to CapabilityDecision)

BackendAdapter Protocol (base.py:17):
  capabilities() -> BackendCapabilities
  generate(prompts: list[str], **kwargs) -> list[dict]
    generate return shape per item: {prompt: str, input_ids: [int], output_ids: [int],
                                       scores?: [float], tokenizer_id?: str}


PUBLIC API (api.py)

run(*, backend, model, prompts, tier=1, mode="best-effort", batch_size=1, seed=0,
    max_new_tokens=32, temperature=0.0, top_p=1.0, top_k=0, device="cpu", dtype="float32",
    out_dir="artifacts/run", backend_adapter=None, redact=False, redact_env_vars=None,
    validate_schema=False) -> RunResult
  Returns RunResult(status="PASS"|"FAIL", category=str, out_dir=str)
  FAIL if UNSUPPORTED_REQUEST; PASS otherwise.
  Does NOT run multiple times; for variance measurement use check().

check(*, backend, model, prompts, tier=1, mode="best-effort", runs=3, batch_size=1,
      vary_batch=None, seed=0, max_new_tokens=32, temperature=0.0, top_p=1.0, top_k=0,
      device="cpu", dtype="float32", out_dir="artifacts/check", backend_adapter=None,
      redact=False, redact_env_vars=None, validate_schema=False) -> Report
  Returns Report(status, category, details).
  Runs N times, diffs, optionally varies batch, writes all artifacts.


BEHAVIORAL CONTRACTS

capture_env() -> dict:
  Side effects: reads os.environ, imports torch, calls torch.cuda / torch.backends.mps.
  Fingerprint is SHA-256 of the snapshot dict (without fingerprint key itself), canonical JSON.
  Not stable across Python interpreter restarts if sys.executable changes.

DeterministicContext.__enter__:
  Side effects: calls random.seed, torch.manual_seed, torch.use_deterministic_algorithms(True).
  Saves and restores RNG state on exit.
  Does NOT set CUBLAS_WORKSPACE_CONFIG; only records its current value.
  In best-effort mode: torch import failures cause tier downgrade to 0, not exception.
  In strict mode: torch failures propagate as exceptions.

write_trace:
  Writes JSON lines, sort_keys=True. No trailing newline per doc, only \n after each row.
  Optional per-row schema validation via jsonschema if validate_rows=True.

evaluate_capabilities:
  Mutates applied.capability_failures and applied.downgrades in-place (not frozen).
  Returns CapabilityDecision; supported=False triggers UNSUPPORTED_REQUEST path.
  In strict mode: any capability failure -> supported=False.
  In best-effort: tier_effective is lowered, supported=True, execution continues.


PLATFORM CONSTRAINTS

torch.use_deterministic_algorithms(True) requires CUBLAS_WORKSPACE_CONFIG env var set
  BEFORE process start for CUDA determinism. detllm records the value but does not set it.
  Consequence: if CUBLAS_WORKSPACE_CONFIG is unset when using CUDA, torch may raise at runtime.
  Workaround required: caller sets CUBLAS_WORKSPACE_CONFIG=:4096:8 before launching.

Score comparison is exact float equality (no tolerance). Cross-platform float non-determinism
  (different CUDA driver versions, different GPU models) will produce SCORE_VARIANCE even for
  logically identical runs on different hardware.

Tier 1 guarantee is batch-size-specific. Changing batch_size between runs invalidates the
  fixed-batch guarantee even with identical seeds.

Per-run backend reload (check command): each run creates a new backend instance. This isolates
  state but prevents KV-cache warming across runs.


GOTCHAS

1. CUBLAS_WORKSPACE_CONFIG must be set by the caller, not by detllm. The library only records it.
2. score_divergence comparison is exact float (==), not approximate. Any fp non-determinism = FAIL.
3. tier_effective in determinism_applied.json may differ from tier_requested. Always read
   tier_effective, not tier_requested, to know what guarantees actually applied.
4. fingerprint in env.json is computed BEFORE the fingerprint key is added (env.py:105-106);
   re-computing it by including fingerprint in the payload will not match.
5. vary_batch comparison uses traces[0] (run_0) as baseline, NOT a separate single-item run.
   If run_0 itself is non-deterministic, vary_batch results are unreliable.
6. determinism_applied.json for check records run_0 only; the other N-1 determinism states are
   not persisted (only the first run's controls are assumed stable).
7. vllm backend is documented Tier 0 only. It has supports_tier1_fixed_batch=False and
   supports_torch_deterministic=False. Requesting Tier 1+ with vllm in strict mode = UNSUPPORTED_REQUEST.
8. prompt_id is sha256 of the raw prompt string, not the token sequence. Prompts with identical
   tokenization but different whitespace will appear as different prompts.
9. The schema files declare additionalProperties: true. The library does NOT enforce field completeness
   at load time except for REQUIRED_HEADER_FIELDS = {schema_version, detllm_version, artifact_type}.
   Full schema validation only occurs if validate_schema=True is passed.
10. Artifact files are sorted by key (sort_keys=True in dump_json). This is load-bearing for
    fingerprint stability: any code that writes env.json must also use sort_keys=True.


CRITICAL ASSERTIONS (Rust/ELAI translation targets)

assert artifact.artifact_type == expected_type  // every artifact must match its declared type
assert env.fingerprint == sha256(canonical_json(env_without_fingerprint))  // fingerprint re-derivable
assert run_config.tier_effective <= run_config.tier_requested  // effective never exceeds requested
assert len(downgrades) == 0 || tier_effective < tier_requested  // downgrade recorded iff tier dropped
assert trace_row.prompt_id == sha256(prompt_utf8)  // prompt_id derivation
assert first_divergence is None iff report.status == "PASS"  // divergence present iff failing
assert score_variance_check uses exact equality, not epsilon  // float == not math.isclose
assert diffs/first_divergence.json written iff first_divergence is not None
assert "traces/run_0.jsonl" is the comparison baseline for both run-variance AND batch-variance
assert determinism_applied.json records run_0 controls only (not last run)

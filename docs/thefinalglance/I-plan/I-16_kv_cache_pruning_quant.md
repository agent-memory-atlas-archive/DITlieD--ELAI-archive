# I-16: kv-cache pruning + quant eval

- Track 14 — CACHE-INVARIANT ECONOMICS MIDDLEWARE | lever LOCAL-VRAM | grounding MODERATE | nature ACTIVATION+EVAL | effort M | WAVE 5 | RunPod-gated (live leg) | ID-5 | IC-7
- Premise (CORRECTED 2026-08-08 ratification): the local sidecar ALREADY ships KV levers
  (fp8/fp16 `--kv-cache-dtype`, Int4-class `--turboquant-bits` / KVCompressor / `_KVQuantCache`,
  StreamingLLM sink mask via `ELAI_STREAMING_LLM_SINKS`). What is ABSENT is the EVALUATION layer:
  no fixture harness measures those levers on ELAI fixtures, no `KvCacheTuned` decision row, no
  `elai bench kv-tune` entrypoint. This unit ships the eval layer (Tier-1 deterministic dry-run).
  It does NOT add duplicate flags `--kv-pruning-budget` / `--kv-quant-bits` and does NOT change
  serving defaults. True SnapKV/ThinK-class KV eviction stays Tier-2 (position_ids re-index gap).
  MoE retarget is a separate evaluation checkbox (IC-7). ADOPT only after measured pass-rate + VRAM
  on the live leg (ID-5).

## Steal/build
Sidecar VRAM levers as an EVALUATION over SHIPPED surfaces, not a flag-adoption rewrite.
Config matrix (dry-run): turboquant_bits in {0,4,8} x kv_cache_dtype in {fp16, fp8_e4m3, fp8_e5m2}
x sinks in {0,4}. Instantiates real config path (`KVCompressor`, FA2 auto-detect path,
LogitsProcessor assembly) without GPU. Live RunPod measurement deferred. Paper SnapKV/ThinK
numbers stay [DESIGN] until measured on ELAI fixtures (ID-5). W-10: deterministic harness ships
regardless of RunPod.

## Existing ground truth
- E-26/E-27/E-28 serving stack + FA2 mandate + LogitsProcessor chain (compose, never replace)
- E-32 context lift baseline; B-37 sidecar adapter (live leg transport only)
- python/elai_inference/server.py: shipped KV levers (KVCompressor, --kv-cache-dtype,
  --turboquant-bits, StreamingLLM sinks) — the SUBJECT of the eval
- crates/cli BenchLongcot family (args.rs / main.rs / bench_longcot.rs) — CLI seam precedent
- IC-7 / ID-5 binding as in I-GROUNDING-MAP / I-0_INDEX

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1, no GPU):
- python/elai_inference/eval_kv_tune.py::KvTuneEval — dry-run matrix over SHIPPED levers;
  writes metrics/kv_tune_<blake3>.toml; refuses FA2-disabled configs with typed error
- +KvCacheTuned SSB flat beacon (vram_gb / tok_s / fixture_pass_rate / config_hash / adopt_decision)
- `elai bench kv-tune` CLI (BenchKvTune + bench_kv_tune.rs + main dispatch), dry-run + --emit-ssb

DEFER (Tier-2 / gated):
- live RunPod execution and Adopt/Reject decision
- MoE sidecar retarget evaluation (IC-7)
- NEW eviction flags / SnapKV-class real cache eviction (blocked on position_ids re-index)

## Integration
New file: python/elai_inference/eval_kv_tune.py (~150 LOC)
NO new server.py argparse flags (sweep existing --turboquant-bits / --kv-cache-dtype / sinks)
CLI: `elai bench kv-tune` modeled on BenchLongcot (not args_bench table alone)
SSB: +KvCacheTuned flat beacon (auto SCHEMA_VERSION from SSB_ACTION_TYPES len; no hand bump)
frontend_pair: N/A — metrics TOML only; not a dashboard surface (W-6)

## WIRING_CONTRACT
produces_symbols:
  - python/elai_inference/eval_kv_tune.py::KvTuneEval (dry-run matrix over shipped levers)
  - +KvCacheTuned (flat beacon: vram_gb / tok_s / fixture_pass_rate / config_hash / adopt_decision)
    in crates/orchestrator/src/ssb.rs SSB_ACTION_TYPES + payload lockstep
  - crates/cli/src/args.rs CliAction::BenchKvTune + crates/cli/src/bench_kv_tune.rs +
    crates/cli/src/main.rs dispatch arm (production CLI entrypoint)
consumes_symbols:
  - python/elai_inference/server.py shipped KV levers (KVCompressor, dtype, turboquant, sinks) — GD-1
  - E-26 FA2 path + E-28 LogitsProcessor chain (assert preserved under every config) — GD-1
  - BenchLongcot runner pattern (deterministic fixture, optional SSB emit) — GD-1
reachable_from:
  - cargo run -p elai-cli -- bench kv-tune --dry-run --emit-ssb -> KvTuneEval dry-run
  - (test callers alone do NOT satisfy R7)
stub_ban:
  - NO duplicate --kv-pruning-budget / --kv-quant-bits flags
  - NO serving default change without KvCacheTuned(adopt_decision: Adopt) from LIVE measurement
  - NO paper SnapKV/ThinK accuracy as a gate (ID-5)
  - NO MoE retarget assumed (IC-7)
  - NO FA2 disable / LogitsProcessor bypass
  - NO blake2/sha2 (blake3 only for config_hash)
  - NO fabricated VRAM/tok_s on dry-run (null fields + DryRun only)
  - NO .unwrap()/.expect() on production parse paths
frontend_pair:
  N/A — metrics/kv_tune_<hash>.toml only
observable_effect:
  trigger: "cargo run -p elai-cli -- bench kv-tune --dry-run --emit-ssb"
  expect_ssb: "KvCacheTuned{vram_gb: null, tok_s: null, fixture_pass_rate: 1.0, config_hash: blake3(...), adopt_decision: DryRun}"
  expect_artifact: "metrics/kv_tune_<hash>.toml; FA2-disabled config raises typed error; ADOPT with null vram FAILS"

## Grounding
- E-26/E-27/E-28 serving stack (compose-never-replace)
- E-32 context lift, B-37 sidecar adapter (live leg)
- IC-7, ID-5
- conductor/opus_walk/evidence/I-16/seam-design-proposal.md + seam-design-ratification.md (2026-08-08)
- SnapKV / ThinK [DESIGN] only until measured
- SSB live integers re-read at edit (W-9); never restated SCHEMA_VERSION / TOTAL_VARIANTS in comments

## COMPOSES-WITH
E-26/E-27/E-28 sidecar serving stack (additive)
E-27/StreamingLLM parity harness patterns
BenchLongcot CLI family (J-107)
IC-7 MoE-retarget as a separate future checkbox

## Wargame / acceptance

EXPECTED_OBSERVATION: cargo run -p elai-cli -- bench kv-tune --dry-run --emit-ssb emits one
  KvCacheTuned row with adopt_decision=DryRun, fixture_pass_rate=1.0, vram_gb and tok_s null,
  and writes metrics/kv_tune_<blake3>.toml. Every dry-run matrix config preserves FA2 +
  LogitsProcessor path assertions. A config that would disable FA2 is REFUSED with a typed error.

ON_FAIL: missing KvCacheTuned / zero metrics file / silent FA2 disable -> CAUSE eval layer or CLI
  dispatch not wired -> COUNTER-MOVE re-read BenchLongcot (args.rs / main.rs / bench_longcot.rs)
  and ssb Pattern-2 lockstep; re-run smoke.

FORK: S3 dry-run transport = subprocess `python -m elai_inference.eval_kv_tune --dry-run` (lean).
  HTTP via SidecarProvider is DEFERRED to the RunPod live leg only.

ABORT: if LIVE SSB_ACTION_TYPES / lock integers cannot be re-read at edit time, STOP (do not invent
  SCHEMA_VERSION bumps). If dry-run would require a booted model server, STOP and keep subprocess.
  Never emit adopt_decision=Adopt without live measured vram drop + pass-rate band (ID-5).

VERIFY (off-agent; controller/auditor owns these runs):
  - python3 .claude/tools/plan_gate_check.py --file docs/thefinalglance/I-plan/I-16_kv_cache_pruning_quant.md
  - cargo check -p elai-cli --all-targets
  - cargo test -p elai-orchestrator action_type_lock
  - cargo run -p elai-cli -- bench kv-tune --dry-run --emit-ssb
  - production_caller_check on new CLI dispatch symbols

REDTEAM: attack "ADOPT emitted with null vram_gb / without measurement" must FAIL closed — only
  DryRun is legal on the no-GPU path. attack "duplicate --kv-pruning-budget replacing shipped
  levers" is banned by stub_ban and this plan's corrected produces_symbols.

INTEGRATION_SMOKE: cargo run -p elai-cli -- bench kv-tune --dry-run --emit-ssb from a clean tree
  yields exactly one KvCacheTuned{adopt_decision: DryRun} SSB row and one metrics/kv_tune_<hash>.toml
  with config_hash = blake3 of the config matrix entry. No dashboard UI surface.

End of I-16.

C-26 Constrained Decoding + Retry/Futility Primitives — Decisions
==================================================================

Session: s-25672
Plan:    docs/thefinalglance/C-plan/C-26_constrained_decoding_retry_strategy.md
Context: workflow/sessions/s-25672/context-package-C26.md
Baseline: SCHEMA_VERSION = 57 (C-25 audit follow-up); pre-reqs C-10 + C-25
          shipped per context-package §1.

==================================================================
DECISIONS
==================================================================

DECISION-A — RoleDecoding is a LOCAL enum in elai_providers, not a re-export
of elai_orchestrator::AgentRole.

  Why: providers crate must not depend on orchestrator (current edge runs
  orchestrator -> providers). Adding a `orchestrator::AgentRole` import to
  providers/decoding.rs would create a circular dep. The 5 roles
  (designer/executor/critic/auditor/researcher) cover the decoding policy
  surface; if more roles are added the choice is local-table-only.

  Trade-off: the orchestrator dispatcher must map AgentRole -> RoleDecoding
  at the call site. This is a O(1) match arm and is documented in the
  decoding.rs doc-comment.

DECISION-B — Grammar `ebnf_path` is CALLER-RESOLVED. providers/decoding.rs
stores the relative path; the orchestrator joins it against the grammars/
dir before forwarding to the sidecar.

  Why: providers cannot reach `CARGO_MANIFEST_DIR` of the orchestrator at
  runtime, and embedding the grammars/ bytes in providers would force a
  crate-time path dependency. Storing the path as `PathBuf` keeps the type
  decoupled; the caller decides whether to ship the grammar via env var
  (`ELAI_GRAMMARS_DIR`), include_bytes!, or working-directory lookup.

  Today's per-role default for Executor uses `PathBuf::from("file_plan.ebnf")`
  (a bare filename, not a path). The orchestrator dispatcher will prepend
  the resolved grammars dir before forwarding to the sidecar.

DECISION-C — `temperature_schedule = [0.0, 0.2, 0.5]` (staged warming).

  Why: deterministic-first matches the existing codebase convention
  (`GROUNDING_TEMPERATURE: f64 = 0.0` at grounding_client.rs:30). Attempt 0
  reproducible; attempt 1 mild exploration to escape near-miss errors;
  attempt 2 stronger exploration before the per-baby-step retry budget
  runs out (`DEFAULT_MAX_RETRIES_BABY_STEP = 3`).

  `temperature_for_attempt` saturates past `len()` (returns last entry)
  and defaults to `0.0` on empty schedule. Both invariants are pinned by
  the contract test `retry_strategy_temperature_clamps_at_last_entry` +
  `retry_strategy_temperature_empty_schedule_does_not_panic`.

DECISION-D — On `constrained_decoding_unavailable` (sidecar 503): tracing::warn!
+ FreeText fallback (NOT silent swallow, NOT hard fail).

  Why: hard-fail breaks every dispatch when the sidecar is missing
  lm-format-enforcer (operations event). Silent swallow violates the plan
  stub_ban. Warn + fallback keeps the dispatch working with degraded
  guarantees and surfaces the operations issue in tracing logs.

  Implementation: SidecarProvider::complete_impl inspects the 503 body
  for the `constrained_decoding_unavailable` substring (mirrors
  python/elai_inference/server.py:117). When detected AND the original
  request carried a non-FreeText decoding config, the client retries
  ONCE with the constraint stripped. A second non-success status from
  the fallback is propagated as a normal ApiError.

DECISION-E — Benchmark fixture files (templates.json + padding.txt) are
ADDED as supplementary fixtures but the 0% measurement bug is fixed by
INSTRUMENTING the 4 bare `except Exception:` blocks in measurements 3 + 4
(DRIFT-2 from the context package).

  Why: per context-package §3b, the harness reads only fixtures/tasks.json
  (run_benchmark.py:1471) — no code path opens templates.json or
  padding.txt today. The 0% root cause is silent error swallowing in the
  measurement try/except blocks. Adding placeholder fixtures alone would
  not change the 0% outcome.

  Two-pronged fix:
    1. Instrument the 4 bare `except Exception:` blocks at the new line
       positions (post-import of `_log_measurement_exception`): print a
       traceback to stderr before continuing the iteration. Original
       control flow preserved.
    2. Add fixtures/templates.json (3 templates) and fixtures/padding.txt
       (100 lines) for plan-compat completeness so future harness
       extensions that adopt the spec-section-11 references have valid
       inputs.

  Re-running the harness to update model_baseline.toml is deferred: it
  requires a live sidecar boot (HARDWARE-HEAVY per CLAUDE.md) and is not
  in the autonomous scope of this session. The instrumented harness will
  emit non-zero measurements once the underlying API failure (or
  extract_code failure) is surfaced and corrected.

DECISION-F — Persona switch on `StagnationPattern::NoDrift` → `ThinkingPersona::Hacker`
(per C-10 D4 affinity table, NOT Researcher).

  Why: the plan §Task 3 prose implies Researcher (fallback rotation), but
  C-10 already shipped a deterministic affinity table that maps
  NoDrift → Hacker. Diverging from C-10 here would create two sources of
  truth. The contract test
  `futility_output_similarity_90pct_triggers_persona_switch` pins
  `new_persona == "hacker"`.

==================================================================
SHIPPED INVENTORY
==================================================================

ADDED:
  crates/providers/src/decoding.rs                                   (~330 LOC)
  crates/orchestrator/src/grammars/json_response.ebnf                (~30 lines)
  crates/orchestrator/src/grammars/baby_step_spec.ebnf               (~30 lines)
  crates/orchestrator/src/grammars/file_plan.ebnf                    (~40 lines)
  crates/contracts/tests/c26_constrained_decoding_contract.rs        (25 tests)
  benches/model_capability/fixtures/templates.json                   (3 templates)
  benches/model_capability/fixtures/padding.txt                      (100 lines)
  workflow/reasoning/C-26-decisions.md                               (this file)

MODIFIED:
  crates/providers/src/lib.rs                       (+decoding mod + 6 re-exports)
  crates/providers/src/sidecar_client.rs            (decoding_config field +
                                                     build_completion_body new
                                                     arg + 503 fallback path)
  crates/orchestrator/src/grounding_client.rs       (build_request_with_decoding;
                                                     Anthropic Tool API path)
  crates/orchestrator/src/pipeline_tree_navigator.rs (RetryStrategy + BubbleUpPolicy
                                                     .retry_strategy field +
                                                     TreeNavigatorState fields +
                                                     futility methods +
                                                     FutilityVerdict +
                                                     StagnationVerdict)
  crates/orchestrator/src/ssb.rs                    (SCHEMA 57->58 atomic 9-tuple
                                                     lockstep + 2 new variants
                                                     [FutilityDetected,
                                                      FutilityOutputStagnation]
                                                     + 2 new payload structs;
                                                     all_test_payloads count 118
                                                     -> 120)
  benches/model_capability/run_benchmark.py         (_log_measurement_exception
                                                     helper + 4 bare except
                                                     blocks instrumented)

==================================================================
TEST RESULTS
==================================================================

cargo check -p elai-providers --features sidecar     : PASS
cargo check -p elai-orchestrator                     : PASS
cargo check -p elai_cc_contracts --test c26_*        : PASS

cargo test  -p elai-providers --lib decoding         : 6 / 6 PASS
cargo test  -p elai-providers --features sidecar
                              --lib sidecar_client   : 10 / 10 PASS
cargo test  -p elai-orchestrator --lib
                              pipeline_tree_navigator: 51 / 51 PASS
cargo test  -p elai-orchestrator --lib
                              ssb::tests::test_typed_payloads
                                                     : 1 / 1 PASS  (120 payloads
                                                                    round-trip)
cargo test  -p elai_cc_contracts --test
                              c26_constrained_decoding_contract
                                                     : 25 / 25 PASS

==================================================================
DEFERRED (out of C-26 scope)
==================================================================

D1. ModelFeatures.supports_grammar profile flip (context_budget.rs).
    The plan's WIRING_CONTRACT mentions "patch profiles for Grammar mode",
    but doing so requires running detect_model_capability_profile against
    a live sidecar to verify which profile maps to the active local model.
    Without that empirical check the flip would be a guess. Deferred to
    the auditor / a follow-up patch when the sidecar is online.

D2. Live cloud-Sonnet smoke test (acceptance criterion (a) + (b) of the
    plan). Requires HARDWARE-HEAVY remote sidecar boot per CLAUDE.md
    "Remote sidecar (canonical)". Out of autonomous scope.

D3. Re-run benchmark + update model_baseline.toml. Same hardware
    requirement as D2 plus the instrumented harness needs to be exercised
    against the live sidecar to surface the underlying 0% bug. The
    plan's stub_ban (`MUST contain actual measured values not fabricated
    floats`) precludes filling the file by hand.

D4. CRANE adaptive enforcement runtime. The flag (`crane_adaptive: bool`)
    and the eligibility helper (`crane_adaptive_eligible`) ship as
    forward-compat hooks. Actual CRANE grammar-restriction logic is a
    separate research item.

PARITY-P0 executor decisions
Task: A63-PARITY-P0-EXECUTE
Date: 2026-06-12

--- WHAT ---

Three lane-parity fixes:
  P0-1: adversary resolver — boot-time runtime-pool resolution via route_cross_family
  P0-2: E-19 reflection regression — wire SidecarReflectionAdapter into RuntimeWiring
  P0-3: local metering — emit BackendCostAccrued with est_cost_micros=0 for local backends

--- KEY DECISIONS ---

P0-1:

D1: RuntimeWiring gains an adversary_runtime field (Option<Arc<AdversaryRuntimeState>>)
    mirroring every other optional wiring component.

D2: build_task_runner_wiring calls route_cross_family at boot time with:
    - executor_family = ModelFamily::Local (conservative default; local sidecar is the
      canonical worker in default deploy; production wiring reads this from config in
      a follow-up; Local-default is correct and safe: Local vs Anthropic passes).
    - available: pool of families from PerRoleRuntime. Since PerRoleRuntime does not
      carry family metadata (only RuntimeBackend), we enumerate available families by
      mapping RuntimeBackend variants to ModelFamily using ModelFamily::from_model_id
      on canonical_name (or a direct RuntimeBackend->ModelFamily map). The pool is
      constructed as [ModelFamily::Anthropic] when AnthropicApi is in the mapping
      (always true in default PerRoleRuntime), plus ModelFamily::DeepSeek when
      DeepSeekV4Flash is in the mapping.
    - allow_cloud: std::env::var("ELAI_ALLOW_CLOUD_ADVERSARY") == truthy.

D3: AdversaryRuntimeState is constructed only when ELAI_ALLOW_CLOUD_ADVERSARY is set
    AND route_cross_family succeeds (returns Ok). The one-family case (Err) leaves
    adversary_runtime = None on the wiring (no construction, clean Mode E in dispatch).

D4: The concrete DynAdversary for the cloud path is a new CloudAdversary struct in
    adversary.rs that wraps AnthropicClient. It builds a MessageRequest asking the
    cloud model to produce a JSON CounterEvidenceSchema, parses the response, and
    returns Ok(schema). This is the minimal impl that makes route_cross_family a live
    caller with a real HTTP path. The full LLM-based counter-evidence generation is
    meaningful but bounded: the prompt is a single user message; the response is
    parsed as a JSON object; on parse failure the schema returns severity=0.0
    (no counter found, claim robust), matching the trait contract.

D5: Mode E refusal hint: dispatch_adversarial_pass in task_runner.rs already logs
    debug on adversary_runtime=None. We extend this to emit a tracing::info!
    message with the hint "configure ELAI_ALLOW_CLOUD_ADVERSARY=1 and a second
    provider to enable cross-family adversarial validation" when runtime is None.
    No new SSB variant needed.

D6: wire-dark-allowlist.toml gets an [[flag]] row for ELAI_ALLOW_CLOUD_ADVERSARY.

D7: Doc comments in adversary.rs:640 that reference ELAI_ALLOW_CLOUD_ADVERSARY as
    "resolved by the caller" are updated to reflect D-025 pool-resolution doctrine.

P0-2:

D8: RuntimeWiring gets a reflection_client field (Option<Arc<dyn DynReflectionClient>>).
    This mirrors option A from the seams document (pattern-consistent).

D9: build_task_runner_wiring constructs SidecarGroundingClient::default() (uses
    DEFAULT_SIDECAR_HOST already embedded in grounding_client.rs) and wraps it in
    SidecarReflectionAdapter, then Arc<SidecarReflectionAdapter>-erased to
    Arc<dyn DynReflectionClient> via the existing ReflectionClientErased pattern.
    The kill-switch ELAI_DISABLE_MICRO_REFLECTION is checked first; if set, the
    field stays None.

D10: apply_task_runner_wiring does: cfg.reflection_client = wiring.reflection_client.clone().
     This threads Some(client) to all 5 TaskRunnerConfig sites (new_minimal gives None;
     apply replaces it with the live Some).

D11: Cloud-only installs without a sidecar will get transport errors at reflect-time.
     These are handled by the existing Some-gate at task_runner.rs:10276 and the
     ReflectionClientError::Transport arm. No guard needed here.

P0-3:

D12: is_local() helper added to RuntimeBackend impl in types.rs. Local backends:
     Sidecar, Mlx, LlamaCpp, Ollama, Vllm. Sglang is NOT included (E-26 doc
     says "cloud throughput adapter", so it has a marginal cost pathway).
     LlamaCpp, Mlx, Ollama, Vllm are clearly local.

D13: runtime_orchestrator.rs:342-365 gets a sibling else-if branch:
     if let Some(ssb) = self.ssb.as_ref() {
         if cost_model_for(backend).is_none() && backend.is_local() {
             // emit with est_cost_micros=0, gate on tokens_in > 0 || tokens_out > 0
             // to avoid (0,0,0) noise rows on parse-error fallback.
         }
     }

D14: No FM-3 mitigation is needed for Sglang (it routes to the existing cost_model_for
     None path and is not is_local; it will not get a $0 row either, which is correct
     since it may be a paid remote endpoint).

--- EXPECTED TEST OUTCOMES ---

P0-1 new tests:
  adversary_contract.rs test 10: route_cross_family_is_production_callable
    - source-scan: adversary.rs contains "route_cross_family" call from a non-test site
    - source-scan: runtime_setup.rs references "ELAI_ALLOW_CLOUD_ADVERSARY"
  adversary_contract.rs test 11: cloud_adversary_impl_exists
    - source-scan: adversary.rs contains "pub struct CloudAdversary"
  adversary_contract.rs test 12: single_family_pool_returns_error
    - unit: route_cross_family(&Local, &[Local], false) -> Err(SameFamilyPairing)
  adversary_contract.rs test 13: all_anthropic_is_one_family
    - unit: route_cross_family(&Anthropic, &[Anthropic, Anthropic], false) -> Err

P0-2 new test:
  fu_micro_reflection_invocation_contract.rs test 5: apply_task_runner_wiring_wires_reflection_client
    - source-scan: runtime_setup.rs contains "SidecarReflectionAdapter"
    - source-scan: runtime_setup.rs contains "reflection_client"

P0-3 new test:
  h9_cost_cockpit_contract.rs test 3: local_backends_emit_zero_cost_row
    - source-scan: runtime_orchestrator.rs contains "is_local()" call
    - source-scan: runtime_orchestrator.rs contains "est_cost_micros: 0"
  types.rs unit test: is_local_true_for_sidecar_mlx_llama_ollama_vllm
  types.rs unit test: is_local_false_for_anthropic_deepseek

--- UNKNOWNS ---

U1: CloudAdversary requires AnthropicClient::from_env() which reads ANTHROPIC_API_KEY.
    Boot-time construction in build_task_runner_wiring will fail if the env is absent
    AND ELAI_ALLOW_CLOUD_ADVERSARY is set. Mitigation: return an Err wrapped in
    anyhow and log a warning; leave adversary_runtime = None (same as D3 single-family
    case). The adversary dispatch will short-circuit silently, which is better than
    panicking at boot.

U2: The DynReflectionClient erased type may differ from what task_runner.rs expects.
    The existing DynReflectionAdapter pattern handles this. Verify the erased export
    path from orchestrator lib.rs before writing.

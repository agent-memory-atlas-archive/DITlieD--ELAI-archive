# D-2: PPM sidecar (Qwen2.5-Math-PRM-7B) — separable improving verifier

Objective: wire a SECOND sidecar (different model family from policy SLM) as a
Process Preference Model (PPM) for step-level reasoning verification. Policy
SLM (Qwen2.5-Coder-7B-Instruct-abliterated, port 8100) and PPM (Qwen2.5-Math-
PRM-7B, port 8101) evolve independently against different signals. This is
the load-bearing structural pattern of rStar-Math (arxiv:2501.04519, T1): the
verifier is a SEPARATE improving artifact, not the policy in self-judge mode.
Estimated effort: 2-3 sessions.
Pre-requisites: D-1 (ReasoningTemplate + VerifierKind::Prm slot exists; PPM
  consumes templates with verifier_required=Prm), B-37 (multi-backend runtime
  + PerRoleRuntime; PPM lives as runtime backend `Ppm` slot), C-15 (entropy
  quarantine + ClockProvider stable). Spec-grounder verifies all DONE.
Parallel-safe with: D-8 (archive verify CLI; reads ArchiveChain, no PPM
  dependency), D-19 (PRMBench benchmark; depends on D-2 but isolated test
  surface). NOT parallel-safe with D-3 (consumes PPM via VerificationResult).

## Context

`docs/specs/guide/reasoning-improvement-grounded-2026-05-19.md` §3.2 (lines
228-240) defines the PPM substrate. Lightweight Latent Verifiers (arxiv:
2504.16760, T1) proves verifier transfer FAILS between base LLMs — a verifier
trained on one model wont reliably grade another; heterogeneous-verifier
(different family) is the literature's defense, NOT same-family critic loop.

This rules out using Qwen2.5-Coder-7B as its own verifier (the policy SLM in
ELAI). Same-family critic loop fails per arxiv:2507.02778 (T1) Self-Correction
Bench (64.5% blind-spot rate across 14 open-source models). Qwen2.5-Math-PRM-7B
is open-weight (HuggingFace), Qwen-family but separate-base (math-pretrained
+ PRM-distilled), and benchmarks at 60-65 on PRMBench standard categories
(arxiv:2501.03124, T1). Acknowledged weak on adversarial categories — D-19
benchmarks this honestly.

The remote sidecar architecture (workflow/remote-setup/REMOTE-SIDECAR.md) is
the deployment substrate. PPM lives at remote port 8101; tunnel forwards
127.0.0.1:8101 same as port 8100 pattern. Dual-GPU placement on remote box
(RTX 3090 24GB has headroom for two 7B NF4-4bit models simultaneously:
policy ~1.92GB + PPM ~1.92GB = ~3.84GB + KV cache ~1.8GB total = ~5.7GB,
well within 12GB / 24GB budget; B-37 multi-backend confirms feasible).

Partial foundation present and usable:
- RuntimeAdapter trait (B-37): PPM is a new RuntimeBackend variant.
- PerRoleRuntime config (B-37): `[runtime.ppm]` stanza for endpoint + auth.
- HTTPClient + retry logic in sidecar_client.rs (existing): PPM client reuses.
- Anthropic-protocol speaker pattern (sidecar server.py): PPM speaks Anthropic
  /v1/messages too (per existing sidecar protocol; PPM endpoint same shape).
- D-1 ReasoningTemplate.verifier_required: Prm slot defined; D-2 wires the
  actual call.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§3.2 PPM definition, §1.4 rStar-Math grounding, §1.11 verifier-transfer
  failure — lines 52-58 + 107-110 + 228-240)
- F:\.ELAI_workflow\workflow\papers\dgm-interface.md (no PPM in DGM; flagged as
  improvement axis)
- F:\.ELAI_workflow\workflow\remote-setup\REMOTE-SIDECAR.md (deployment substrate)
- .elai_cc/python/elai_inference/server.py (Anthropic-protocol speaker; PPM
  server reuses same FastAPI scaffolding)
- .elai_cc/crates/orchestrator/src/runtime/per_role_runtime.rs (B-37 PerRoleRuntime
  config; PPM added as new backend variant)
- .elai_cc/crates/orchestrator/src/runtime/runtime_adapter.rs (B-37 trait; PPM
  client implements RuntimeAdapter)
- .elai_cc/crates/orchestrator/src/sidecar_client.rs (existing HTTP client +
  retry; PpmClient inherits)
- .elai_cc/crates/orchestrator/src/reasoning_template.rs (D-1 VerifierKind::Prm
  slot)
- .elai_cc/elai_cc.toml ([runtime.*] section; insertion point for [runtime.ppm])

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for:
- arxiv:2501.04519 rStar-Math (load-bearing PPM substrate)
- arxiv:2504.16760 Lightweight Latent Verifiers (heterogeneous-verifier
  requirement; counter-evidence for same-model critic)
- arxiv:2501.03124 PRMBench (PPM evaluation benchmark; D-19 reuses)

Verify all three at workflow/papers/{rstar-math-2501.04519,latent-verifiers-
2504.16760,prmbench-2501.03124}-interface.md before unblocking TESTING_FIRST.

## Tasks

1. **PpmClient definition** in new file
   .elai_cc/crates/orchestrator/src/ppm_client.rs:
   - `struct PpmClient { endpoint: Url, http: Arc<reqwest::Client>, timeout:
     Duration, retry_policy: RetryPolicy }`
   - `impl PpmClient`: new(endpoint, timeout, retry_policy);
     score_step(prompt: &str, candidate_step: &str, ctx: &PpmContext) ->
     Result<PpmScore, PpmError>
   - `struct PpmScore { score: f32, confidence: f32, raw_output: String }`
     (score in [0.0, 1.0])
   - `struct PpmContext { problem_statement: String, prior_steps:
     Vec<String>, domain: Domain }`
   - PpmError thiserror: { HttpError(#[from] reqwest::Error), TimeoutError,
     ParseError(String), SidecarUnavailable }
   - score_step composes prompt per rStar-Math §3.2 format: "Problem: {}\n
     Previous steps: {}\nCurrent step: {}\nIs this step correct? Rate 0.0 to
     1.0:". Strict format; ParseError if response not parseable.
   - All HTTP calls retry per retry_policy (max 3 attempts, exponential
     backoff with jitter); on final timeout return TimeoutError.

2. **RuntimeAdapter impl for PPM** in
   .elai_cc/crates/orchestrator/src/runtime/ppm_adapter.rs:
   - `struct PpmAdapter { client: Arc<PpmClient> }`
   - `impl RuntimeAdapter for PpmAdapter`: backend_id() = "ppm";
     send_request(...) translates Anthropic /v1/messages payload to PPM
     score request; wraps PpmScore in RuntimeResponse.
   - Per CLAUDE.md rule: PPM is NOT a generic backend; agents cannot pick it
     for generation. Only the consensus verifier in D-3 calls it. Enforce via
     RuntimeAdapter.is_verifier_only() = true.

3. **PerRoleRuntime extension** in
   .elai_cc/crates/orchestrator/src/runtime/per_role_runtime.rs:
   - Add `Ppm(PpmAdapter)` variant to RuntimeBackend enum.
   - elai_cc.toml `[runtime.ppm]` stanza fields: `endpoint = "http://127.0.0.1:
     8101/v1/messages"`, `timeout_secs = 30`, `max_retries = 3`,
     `model = "huihui-ai/Qwen2.5-Math-PRM-7B"` (or distilled-equivalent path).
   - PerRoleRuntime::default() does NOT enable PPM (opt-in via toml + ENV
     ELAI_PPM_ENABLE=1 to prevent accidental boot during dev sessions where
     the second sidecar is not online).

4. **PPM sidecar boot script** at
   workflow/remote-setup/start-ppm-sidecar.sh:
   - Mirrors existing sidecar boot pattern from REMOTE-SIDECAR.md.
   - Command: `ssh -i ~/.ssh/elai_remote -p 6465 maintainer@192.0.2.10 'bash
     -lc "source ~/.venvs/elai-sidecar/bin/activate && cd ~/elai_inference &&
     nohup env ELAI_FORCE_SDPA=1 CUDA_VISIBLE_DEVICES=1 python server.py
     --model huihui-ai/Qwen2.5-Math-PRM-7B --port 8101 --no-dola
     --turboquant-bits 0 --no-steering --load-in-4bit
     > ~/ppm-sidecar.log 2>&1 & echo \$! > ~/ppm-sidecar.pid"'`
   - CUDA_VISIBLE_DEVICES=1 places PPM on GPU1 (policy SLM on GPU0); single-GPU
     fallback runs both sequentially with serialized VRAM swap, documented as
     suboptimal but supported.
   - Health-check 3-step pattern (per CLAUDE.md sidecar protocol):
     a) `pgrep -f "python server.py.*--port 8101"`
     b) `Test-NetConnection 127.0.0.1 -Port 8101`
     c) `curl http://127.0.0.1:8101/v1/messages -X POST ...`

5. **Tunnel script extension** at
   workflow/remote-setup/start-tunnel.ps1:
   - Add second port forward: 127.0.0.1:8101 -> remote 127.0.0.1:8101.
   - Keep existing 8100 forward unchanged.
   - Document dual-tunnel command in REMOTE-SIDECAR.md.

6. **PPM health check command** in
   .elai_cc/crates/cli/src/ppm_health.rs:
   - CLI subcommand `elai ppm health`: runs 3-step check (process / netcat
     / curl); reports OK or PpmError. Exit 0 / 1.
   - Mirrors existing sidecar-health agent pattern (.claude/agents/
     sidecar-health.md); user-facing wrapper.

7. **PPM SSB action types** in ssb.rs:
   - `PpmScoreEmitted { template_id: TemplateId, step_index: u32, score: f64,
     confidence: f64, latency_ms: u64 }`
   - `PpmUnavailable { reason: String, retry_count: u32 }` (operational
     telemetry; fires when verifier dispatch fails after retries)
   - SCHEMA_VERSION bump: atomic with D-3 (single bump for all PPM + verifier
     types). Defer the actual bump to D-3 atomic landing; in D-2, add the
     variants behind a #[cfg(feature = "schema-v49")] or temporary scaffold
     with a NOTE comment "atomic bump in D-3".

8. **Cargo.toml additions**:
   - Verify reqwest is already in elai_orchestrator Cargo.toml (present since
     B-37 cloud-mode).
   - No NEW dependencies expected. If retry-jitter helper is needed, prefer
     hand-rolled (≤30 LOC) over adding a crate.

9. **Unit tests** in ppm_client.rs:
   - PpmClient::score_step happy-path against mock HTTP server (use httpmock
     crate if already in dev-dependencies; otherwise hand-rolled tiny axum
     mock as in C-37 tests).
   - Retry on transient 502; succeeds on 3rd attempt.
   - TimeoutError on persistent 504.
   - ParseError on malformed body.
   - PpmContext::compose_prompt produces deterministic prompt string for fixed
     inputs (snapshot test).

10. **Integration test** in
    .elai_cc/crates/orchestrator/tests/ppm_adapter_integration.rs:
    - Spawn local axum mock server simulating PPM /v1/messages responses.
    - Run PpmAdapter through RuntimeAdapter trait; verify score round-trip.
    - Assert is_verifier_only() == true.

11. **Decision doc** workflow/reasoning/D-2-decisions.md (3 decisions):
    - (a) Why Qwen2.5-Math-PRM-7B and not a different-family-altogether model
      (e.g. DeepSeek-V3 PRM): per arxiv:2504.16760 (T1) heterogeneous-verifier
      principle, ideally DIFFERENT family. Qwen2.5-Math-PRM-7B is same-family-
      different-base — acceptable for code+math mix but flagged as UC; D-13
      Adversary role uses true different-family (cloud Sonnet as Adversary).
    - (b) Why opt-in via ELAI_PPM_ENABLE: dev sessions without second sidecar
      shouldnt block on PPM unavailability; only consensus rule (D-3) requires
      it.
    - (c) Why retry max=3 + exponential backoff with jitter: PPM is hot-path
      verifier; longer retries delay every reasoning step. Standard tradeoff
      documented in REMOTE-SIDECAR.md operational notes.

## WIRING_CONTRACT

produces_symbols:
  - PpmClient + PpmScore + PpmContext + PpmError
  - PpmAdapter + RuntimeAdapter impl (is_verifier_only = true)
  - RuntimeBackend::Ppm variant
  - SsbActionType::PpmScoreEmitted + ::PpmUnavailable + payloads
    (atomic bump in D-3, scaffold here)
  - CLI `elai ppm health` (SlashCommand::PpmHealth)
  - workflow/remote-setup/start-ppm-sidecar.sh
  - [runtime.ppm] elai_cc.toml stanza
consumes_symbols:
  - reqwest (existing; B-37 cloud-mode)
  - RuntimeAdapter trait (B-37)
  - PerRoleRuntime (B-37; gains Ppm backend slot)
  - VerifierKind::Prm slot (D-1)
  - ReasoningTemplate.verifier_required field (D-1)
  - sidecar Anthropic protocol speaker (existing server.py)
reachable_from:
  - VerificationResult.ppm field (D-3 consumes PpmScore)
  - `elai ppm health` CLI
  - ELAI_PPM_ENABLE env var
  - elai_cc.toml [runtime.ppm] section
stub_ban:
  - PpmClient MUST NOT have a same-model-fallback path (defeats heterogeneous-
    verifier requirement; per arxiv:2504.16760 T1)
  - PpmAdapter::is_verifier_only MUST return true (defends against accidental
    use as generation backend)
  - score_step MUST NOT panic on empty body; return ParseError
  - PPM endpoint MUST default to 127.0.0.1:8101 (not policy 8100; collision
    prevention)
  - retry MUST cap at max_retries (default 3); no infinite loop
observable_effect:
  trigger: ELAI_PPM_ENABLE=1 set; PPM sidecar booted; `elai ppm health` run
  expect_ssb: zero rows until D-3 wires consensus; PPM presence confirmed via
              CLI exit code 0 + health-check curl response
  expect_artifact: workflow/remote-setup/start-ppm-sidecar.sh present;
                   start-tunnel.ps1 forwards 8101; elai_cc.toml [runtime.ppm]
                   stanza present

## DETERMINISTIC_CHECK

```bash
# 1. PpmClient + PpmAdapter present
rg "struct PpmClient|struct PpmAdapter" .elai_cc/crates/orchestrator/src/

# 2. PpmAdapter::is_verifier_only == true (compile-time enforced)
rg "is_verifier_only" .elai_cc/crates/orchestrator/src/runtime/ppm_adapter.rs

# 3. RuntimeBackend::Ppm variant
rg "Ppm\(PpmAdapter\)\|RuntimeBackend::Ppm" \
  .elai_cc/crates/orchestrator/src/runtime/per_role_runtime.rs

# 4. SSB variants scaffolded (atomic bump in D-3 — comment-marked)
rg "PpmScoreEmitted|PpmUnavailable" .elai_cc/crates/orchestrator/src/ssb.rs

# 5. PPM port 8101 (not 8100)
rg "8101" .elai_cc/elai_cc.toml workflow/remote-setup/start-ppm-sidecar.sh

# 6. CLI subcommand
rg "PpmHealth\|ppm.health" .elai_cc/crates/cli/src/commands.rs

# 7. Health-check 3-step (matches sidecar-health pattern)
rg "pgrep|Test-NetConnection\|netcat\|curl.*v1/messages" \
  .elai_cc/crates/cli/src/ppm_health.rs

# 8. Build + scoped tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- ppm_client ppm_adapter 2>&1 | grep "test result"

# 9. No same-model-fallback path
rg "fallback.*coder\|coder.*as.*ppm\|same.*model" \
  .elai_cc/crates/orchestrator/src/ppm_client.rs \
  .elai_cc/crates/orchestrator/src/runtime/ppm_adapter.rs
# Expected: 0 hits

# 10. Paper-ingestor outputs
test -f workflow/papers/rstar-math-2501.04519-interface.md && \
  test -f workflow/papers/latent-verifiers-2504.16760-interface.md && \
  test -f workflow/papers/prmbench-2501.03124-interface.md && echo OK
```

## ACCEPTANCE

- [ ] PpmClient + PpmAdapter compile; unit tests pass; integration test against
      mock PPM server passes
- [ ] PpmAdapter::is_verifier_only() returns true; compile-time enforced
- [ ] RuntimeBackend::Ppm variant added; elai_cc.toml [runtime.ppm] stanza
      template + runtime_config loader picks it up
- [ ] PPM SSB variants scaffolded; atomic SCHEMA_VERSION bump deferred to D-3
- [ ] workflow/remote-setup/start-ppm-sidecar.sh runs successfully on remote
      box (manual smoke step documented; CI cannot test SSH)
- [ ] `elai ppm health` exit code 0 when PPM sidecar online; exit 1 with
      structured error when offline
- [ ] start-tunnel.ps1 forwards port 8101; REMOTE-SIDECAR.md documents dual-
      tunnel
- [ ] ELAI_PPM_ENABLE=1 opt-in; without it, PPM is not booted
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- ppm: all pass, ≥6 new tests
- [ ] Cloud smoke test A: HARDWARE-HEAVY (flag in chat before booting per
      CLAUDE.md sidecar rule). Boot policy SLM + PPM both on remote; both
      health checks pass; ssh free GPU mem ≥6GB remaining; both sidecars run
      simultaneously for ≥5 min without OOM
- [ ] Cloud smoke test B: PpmClient.score_step on known-correct reasoning step
      ("if x>0 and x<10, then x>0") returns score ≥0.7; on known-wrong step
      ("if x>0 and x<10, then x>20") returns score ≤0.3; margin >0.3
- [ ] Cloud smoke test C: PPM offline + ELAI_PPM_ENABLE=1 → PpmUnavailable
      SSB row emitted; reasoning step does NOT auto-pass (consensus rule
      defers to D-3 but unavailability is logged)
- [ ] All 3 paper-ingestor outputs present (rstar-math, latent-verifiers,
      prmbench)
- [ ] Decision doc workflow/reasoning/D-2-decisions.md (3 decisions logged)
- [ ] memory-writer emits project_d2_ppm_sidecar.md (post-AUDITING)

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/ppm_client.rs
- ADD: .elai_cc/crates/orchestrator/src/runtime/ppm_adapter.rs
- ADD: .elai_cc/crates/cli/src/ppm_health.rs
- MODIFY: .elai_cc/crates/orchestrator/src/runtime/per_role_runtime.rs
    (RuntimeBackend::Ppm variant)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
    (PpmScoreEmitted, PpmUnavailable variants — SCHEMA_VERSION bump deferred to D-3)
- MODIFY: .elai_cc/crates/cli/src/commands.rs
    (SlashCommand::PpmHealth)
- MODIFY: .elai_cc/elai_cc.toml ([runtime.ppm] stanza)
- ADD: workflow/remote-setup/start-ppm-sidecar.sh
- MODIFY: workflow/remote-setup/start-tunnel.ps1 (dual-port forward)
- MODIFY: workflow/remote-setup/REMOTE-SIDECAR.md (PPM sidecar section)
- ADD: .elai_cc/crates/orchestrator/tests/ppm_adapter_integration.rs
- ADD: workflow/reasoning/D-2-decisions.md
- ADD: workflow/papers/{rstar-math-2501.04519,latent-verifiers-2504.16760,
       prmbench-2501.03124}-interface.md (via paper-ingestor)

## Out-of-scope

- D-3: VerificationResult aggregator + consensus rule (consumes PpmScore)
- D-19: PRMBench benchmark (uses PPM client; isolated session)
- D-28: Adversarial example farming (PPM retraining; DEFERRED)
- Distilled-PPM training (companion Q5; bootstrap from generic corpus is
  another open question, deferred to D-30)

## Next

D-3 (VerificationResult aggregator) is the immediate consumer. D-19 (PRMBench
benchmark) can run in parallel with D-3 since both consume PPM client but do
not share orchestrator paths.

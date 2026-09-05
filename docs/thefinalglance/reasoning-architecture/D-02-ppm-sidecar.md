# PPM Sidecar (Qwen2.5-Math-PRM-7B) (D-2)

- **ELAI ids:** D-2 / Phase 4H.V part 1 / companion §3.2
- **Spec status (as designed):** PLANNED. Feature-map tags DARK(`ELAI_PPM_ENABLE`) (`ELAI-FEATURE-MAP.md:1831-1833`); that is implementation-neighbor.
- **Crate / module (spec):** `crates/orchestrator/src/ppm_client.rs`, `crates/orchestrator/src/runtime/ppm_adapter.rs`; CLI `crates/cli/src/ppm_health.rs`; boot `workflow/remote-setup/start-ppm-sidecar.sh`
- **Purpose of this file:** enough architecture to rebuild the separable Process Preference Model sidecar — client, verifier-only adapter, ports, retries, health. Not a status ledger.

## 1. What it is

The **PPM sidecar** is a second inference process that scores one reasoning step at a time. Policy SLM is Qwen2.5-Coder-7B-Instruct-abliterated on port **8100**. PPM is Qwen2.5-Math-PRM-7B on port **8101**. They evolve against different signals. This is rStar-Math’s separable improving verifier (arxiv:2501.04519), not the policy in self-judge mode (`D-2_ppm_sidecar_wiring.md:3-8`, `D-SUMMARY_INDEX.md:28-32`).

## 2. Why it exists

Verifier transfer fails across base LLMs (arxiv:2504.16760). Same-family critic loops fail Self-Correction Bench at 64.5% blind-spot (arxiv:2507.02778). If this organ is missing, ELAI grades its own steps with the coder SLM. If it is faked (same model, second prompt, “you are the verifier”), the heterogeneous-verifier rule is void and D-3 consensus has a rubber stamp (`D-2_ppm_sidecar_wiring.md:20-32`, `:221-222`).

## 3. How it works (runtime)

1. **Who calls it.** Only the D-3 consensus verifier. Agents cannot pick PPM for generation. `PpmAdapter::is_verifier_only() = true` (`D-2_ppm_sidecar_wiring.md:108-110`, `:223-224`).
2. **Opt-in gate (deterministic).** `PerRoleRuntime::default()` does **not** enable PPM. Boot requires toml `[runtime.ppm]` **and** `ELAI_PPM_ENABLE=1` so a missing second sidecar does not block dev (`:118-120`, `:190-192`).
3. **Input.** `PpmClient::score_step(prompt, candidate_step, ctx: &PpmContext)` where `PpmContext { problem_statement, prior_steps, domain }` (`:87-93`).
4. **Prompt compose (deterministic).** rStar-Math §3.2 format: `"Problem: {}\nPrevious steps: {}\nCurrent step: {}\nIs this step correct? Rate 0.0 to 1.0:"`. Snapshot-tested (`:96-98`, `:175-176`).
5. **HTTP (deterministic protocol, model generates the rating text).** Anthropic `/v1/messages` on `http://127.0.0.1:8101/v1/messages`. Same speaker shape as policy sidecar (`:45-46`, `:115-117`).
6. **Retry (deterministic).** `retry_policy` max **3** attempts, exponential backoff with jitter. Transient 502 retries; persistent 504 → `TimeoutError`. Empty/malformed body → `ParseError`, no panic (`:99-100`, `:168-174`, `:225-228`).
7. **Parse (deterministic).** Response must yield `PpmScore { score: f32 in [0.0, 1.0], confidence: f32, raw_output: String }`. Unparseable → `ParseError` (`:90-91`, `:98`).
8. **Adapter wrap.** `PpmAdapter` implements `RuntimeAdapter`: `backend_id() = "ppm"`; translates Anthropic payload to score request; wraps `PpmScore` in `RuntimeResponse` (`:104-107`).
9. **Commit / abstain.** D-2 emits **zero** SSB rows until D-3 wires consensus. Health path: `elai ppm health` exit 0/1. Offline + `ELAI_PPM_ENABLE=1` logs unavailability; the reasoning step does **not** auto-pass (`:229-232`, `:303-305`).
10. **Fail-closed toward D-3.** PPM is a score source, not a promotion authority. Known-correct vs known-wrong must separate by **margin > 0.3** in cloud smoke B; that is a calibration observable, not a self-grade (`D-SUMMARY_INDEX.md:32`, `D-2_ppm_sidecar_wiring.md:300-302`).

The PPM **model** may generate the 0–1 rating string. The **policy** model may not score its own step, may not select PPM as a generation backend, and may not supply a same-model fallback (`:221-222`, `:108-110`).

## 4. Types, enums, objects

Copy from `D-2_ppm_sidecar_wiring.md:83-117`. Do not invent fields.

```
struct PpmClient {
  endpoint: Url,
  http: Arc<reqwest::Client>,
  timeout: Duration,
  retry_policy: RetryPolicy,
}
  new(endpoint, timeout, retry_policy)
  score_step(prompt: &str, candidate_step: &str, ctx: &PpmContext) -> Result<PpmScore, PpmError>

struct PpmScore { score: f32, confidence: f32, raw_output: String }  // score in [0.0, 1.0]

struct PpmContext {
  problem_statement: String,
  prior_steps: Vec<String>,
  domain: Domain,   // D-1 Domain
}

enum PpmError {
  HttpError(#[from] reqwest::Error),
  TimeoutError,
  ParseError(String),
  SidecarUnavailable,
}

struct PpmAdapter { client: Arc<PpmClient> }
  impl RuntimeAdapter
    backend_id() = "ppm"
    is_verifier_only() = true
    send_request(...) → RuntimeResponse wrapping PpmScore

RuntimeBackend::Ppm(PpmAdapter)   // added to PerRoleRuntime enum
```

**SSB (scaffolded in D-2; SCHEMA bump atomic in D-3)** (`:152-160`):

```
PpmScoreEmitted { template_id: TemplateId, step_index: u32, score: f64, confidence: f64, latency_ms: u64 }
PpmUnavailable { reason: String, retry_count: u32 }
```

Behind `#[cfg(feature = "schema-v49")]` or a NOTE comment `"atomic bump in D-3"`. Do not bump SCHEMA in this organ.

**Toml stanza** (`:115-117`):

```
[runtime.ppm]
endpoint = "http://127.0.0.1:8101/v1/messages"
timeout_secs = 30
max_retries = 3
model = "huihui-ai/Qwen2.5-Math-PRM-7B"
```

## 5. Control flow / state machine

**Owner of the loop:** orchestrator `PpmClient` / `PpmAdapter`. Policy agents do not call it.

**Enablement states:**

| Condition | PPM behavior |
|---|---|
| `ELAI_PPM_ENABLE` unset or 0 | not booted; default runtime has no PPM (`:118-120`) |
| enable=1, sidecar healthy | `score_step` may run; D-3 consumes `PpmScore` |
| enable=1, sidecar down after retries | `PpmUnavailable` (once D-3 lands SSB); step does **not** auto-pass (`:303-305`) |
| agent requests PPM as generator | rejected: `is_verifier_only()==true` (`:108-110`) |

**Retry loop:** attempt 1..`max_retries` (default 3). 502 → retry. 504 persistent → `TimeoutError`. No infinite loop (`:228`).

**GPU placement (ops, not Rust FSM):** CUDA_VISIBLE_DEVICES=1 for PPM, GPU0 for policy. Single-GPU fallback: sequential VRAM swap, documented suboptimal (`:131-133`).

**Health 3-step** (`:134-137`, `:147-148`): (a) `pgrep -f "python server.py.*--port 8101"` (b) net connection on 8101 (c) `curl` `/v1/messages`. CLI wraps this; exit 0 OK, 1 `PpmError`.

## 6. Grounding, receipts, gates

PPM is a **heterogeneous process score**, not a receipt that a claim is true.

- Receipt of a call: HTTP response parseable into `[0.0, 1.0]` plus `raw_output`.
- Operational receipt: `elai ppm health` exit 0 + curl body.
- Promotion receipt is **not** a high PPM score. D-3 requires ≥2 verifiers; D-19 measures PRMBench Deception Resistance because this model is weak on adversarial categories (`D-2_ppm_sidecar_wiring.md:31-32`, `:332-334`).

**Never allowed as a confidence signal:** policy SLM scoring its own step; same-model fallback onto port 8100; “PPM was down so pass”; using PPM as a chat backend; treating `confidence: f32` as GroundingLevel (`:221-227`).

**Family caveat (decision a):** Qwen2.5-Math-PRM-7B is **same-family, different-base** (math-pretrained + PRM-distilled), not a true other family. Spec flags this as UC. True different-family critic is D-13 cloud Sonnet Adversary (`:184-189`). Rebuild both: this organ is the PPM slot; do not pretend it satisfies D-13.

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| port | 8101 (not 8100) | process |
| timeout | 30 s | per call |
| max_retries | 3 | per `score_step` |
| score range | `[0.0, 1.0]` | per step |
| VRAM (spec estimate) | policy ~1.92GB + PPM ~1.92GB + KV ~1.8GB ≈ 5.7GB | dual 7B NF4-4bit on 24GB (`:36-39`) |
| free GPU after dual boot | ≥6GB remaining (cloud smoke A) | remote box (`:297-299`) |
| dual-run soak | ≥5 min no OOM | cloud smoke A |
| correct vs wrong margin | >0.3; correct ≥0.7, wrong ≤0.3 | cloud smoke B (`:300-302`) |
| published PRMBench (context) | 60–65 standard categories | not a D-2 gate; D-19 owns adversarial (`:30-32`) |
| new crates | none expected; retry jitter ≤30 LOC hand-rolled | (`:165-166`) |
| unit tests | ≥6 | `elai_orchestrator -- ppm` |

**Overflow / timeout:** final timeout → `TimeoutError`. Sidecar down → `SidecarUnavailable` / `PpmUnavailable`. No auto-pass (`:94-100`, `:303-305`).

## 8. Roles and neighbors

PPM is **verifier-only**. No agent role may select it for generation.

**Consumes:** `reqwest` (B-37), `RuntimeAdapter` + `PerRoleRuntime` (B-37), D-1 `VerifierKind::Prm` + `ReasoningTemplate.verifier_required`, existing `server.py` Anthropic speaker (`:208-214`).

**Produces:** `PpmScore` for D-3 `VerificationResult.ppm`; `elai ppm health`; boot/tunnel scripts; toml stanza.

**Must not fork:** a second PPM that is the coder SLM with a critic prompt; collapsing 8101 onto 8100; a generic backend agents can pick.

**Parallel-safe with:** D-8, D-19. **Not** with D-3 (`:14-16`).

**Out of this organ:** consensus (D-3), PRMBench (D-19), adversarial farming (D-28 DEFERRED), distilled-PPM training (`:331-337`).

## 9. Config / flags / defaults

- Default **OFF.** `PerRoleRuntime::default()` does not enable PPM (`:118-120`).
- Env: `ELAI_PPM_ENABLE=1` required to boot (`:118-120`).
- Toml: `[runtime.ppm]` fields in section 4.
- Endpoint default `127.0.0.1:8101`, never policy 8100 (`:226-227`).
- SCHEMA_VERSION not bumped here (`:157-160`).
- Dual-tunnel: `start-tunnel.ps1` adds 8101 forward; 8100 unchanged (`:139-143`).

Boot command shape (copy; ops substrate) (`:125-130`): remote `server.py --model huihui-ai/Qwen2.5-Math-PRM-7B --port 8101 --no-dola --turboquant-bits 0 --no-steering --load-in-4bit` with `ELAI_FORCE_SDPA=1` and `CUDA_VISIBLE_DEVICES=1`.

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `ppm_client.rs` | `PpmClient`, `PpmScore`, `PpmContext`, `PpmError` (`:83-100`) |
| `runtime/ppm_adapter.rs` | `PpmAdapter` + `RuntimeAdapter` (`:102-110`) |
| `runtime/per_role_runtime.rs` | `RuntimeBackend::Ppm` (`:112-120`) |
| `cli/src/ppm_health.rs` | `elai ppm health` (`:145-150`) |
| `SlashCommand::PpmHealth` | `cli/src/commands.rs` |
| SSB scaffolds | `ssb.rs` (`:152-160`) |
| `start-ppm-sidecar.sh` | remote boot (`:122-137`) |
| `start-tunnel.ps1` | 8101 forward (`:139-143`) |
| `REMOTE-SIDECAR.md` | dual-tunnel docs |
| integration test | `orchestrator/tests/ppm_adapter_integration.rs` (`:178-182`) |
| decisions | `workflow/reasoning/D-2-decisions.md` (`:184-195`) |
| papers | `rstar-math-2501.04519`, `latent-verifiers-2504.16760`, `prmbench-2501.03124` (`:72-79`) |

Reuse: `sidecar_client.rs` HTTP + retry; PPM client inherits (`:64-65`). No new deps expected (`:162-166`).

## 11. Journal / SSB events

D-2 scaffolds `PpmScoreEmitted` and `PpmUnavailable` but **does not bump SCHEMA** and **emits zero rows** until D-3 (`:157-160`, `:229-232`).

Cloud C: PPM offline + enable=1 → `PpmUnavailable` row once D-3 lands; until then unavailability is still **logged** and the step does not auto-pass (`:303-305`).

## 12. Tests and observables

Named tests (`:168-182`):

| Id | Assert |
|---|---|
| T-happy | `score_step` against mock HTTP → `PpmScore` |
| T-retry | transient 502; success on 3rd attempt |
| T-timeout | persistent 504 → `TimeoutError` |
| T-parse | malformed body → `ParseError` |
| T-prompt | `compose_prompt` deterministic snapshot |
| T-adapter | `PpmAdapter` through `RuntimeAdapter`; score round-trip; `is_verifier_only()==true` |

Done-gates (`:279-309`): compile; ≥6 orchestrator tests; health 0/1; enable default-off; dual-tunnel docs; papers present; workspace check clean.

Cloud: (A) HARDWARE-HEAVY dual boot ≥5 min, ≥6GB free, no OOM. (B) correct `"if x>0 and x<10, then x>0"` ≥0.7; wrong `"if x>0 and x<10, then x>20"` ≤0.3; **margin >0.3**. (C) offline does not auto-pass (`:296-305`).

Adversarial: `rg` same-model-fallback / coder-as-ppm = 0 hits (`:267-271`). Port 8101 not 8100 (`:253-254`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- A second process, different weights from the policy SLM, on a **different port** (8101 vs 8100).
- `PpmClient::score_step` returns `PpmScore.score ∈ [0.0, 1.0]` or a typed `PpmError`.
- Adapter `is_verifier_only()==true`; agents cannot generate with it.
- No same-model fallback onto the policy backend.
- Default OFF (`ELAI_PPM_ENABLE` + toml).
- Retries cap at 3; timeout 30s; parse failures are errors, not 0.5.
- Health is process + port + `/v1/messages` curl.
- D-3 is the only promotion consumer; D-2 does not promote templates.
- Cloud B margin >0.3 on the two named steps (or documented hardware skip — CI cannot SSH).

A “critic prompt” on the coder SLM, a second replica of the same weights, or a scalar the policy appends to its own step is a cousin, not this organ.

## 14. Non-goals / banned designs

- VerificationResult / ≥2-verifier consensus — D-3 (`:332-333`).
- PRMBench adversarial categories — D-19 (`:333-334`).
- PPM retraining / adversarial farming — D-28 (`:335`).
- Distilled-PPM / generic-corpus bootstrap — companion Q5 / D-30 (`:336-337`).
- SCHEMA bump in this session (`:157-160`).
- Same-model fallback (`:221-222`).
- PPM as generic generation backend (`:108-110`, `:223-224`).
- Default-on boot that hangs when 8101 is down (`:190-192`).
- Infinite retry (`:228`).
- New crate for jitter (`:165-166`).
- Policy self-eval of its own modification or step.

## 15. Source citations

Primary: `docs/thefinalglance/D-plan/D-2_ppm_sidecar_wiring.md` (full file). Index: `D-SUMMARY_INDEX.md:28-32`, `:269`. Feature map: `ELAI-FEATURE-MAP.md:1831-1833`. Companion (not in this tree): `reasoning-improvement-grounded-2026-05-19.md` §3.2 / §1.4 / §1.11 as cited `:52-54`. Papers: rStar-Math 2501.04519, Lightweight Latent Verifiers 2504.16760, PRMBench 2501.03124, Self-Correction Bench 2507.02778 (`:20-32`, `:70-76`).

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- `RetryPolicy` struct fields beyond max 3 + exponential jitter (`:85-86`, `:99-100`).
- `PpmContext::compose_prompt` vs `score_step(prompt, …)` — who concatenates (`:87-89` vs `:96-98` vs `:175-176`).
- Whether `score` and `confidence` are two parsed numbers or confidence is derived (`:90-91`).
- How Anthropic `/v1/messages` content is mapped onto the rStar-Math rating string (`:106-107`).
- `RuntimeResponse` wrap shape for `PpmScore` (`:107`).
- SCHEMA scaffold: cfg-feature vs comment-only (`:157-160`).
- httpmock vs hand-rolled axum mock (`:169-171`).
- Single-GPU serialized VRAM swap algorithm (`:131-133`).
- Whether `PpmUnavailable` may be emitted in D-2 before D-3 bump (cloud C vs wiring “zero rows until D-3”) (`:229-232` vs `:303-305`).
- HuggingFace id `huihui-ai/Qwen2.5-Math-PRM-7B` vs “distilled-equivalent path” (`:117`).
- True cross-family PPM weights (decision a flags UC; D-13 is the other-family critic, not a PPM replacement) (`:184-189`).

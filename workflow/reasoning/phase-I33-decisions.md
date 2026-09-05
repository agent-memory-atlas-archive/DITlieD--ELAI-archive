I-33: mechanical decision governance — decisions

WHAT
Add CEFL (Candidate Externalization for Fair Labelling), I6Q (rationale quality validation),
and commit-reveal integrity primitives to the orchestrator. Wire into the prompt_decision
call surface. Emit GovernancePrimitiveFired SSB rows (3-arm Kind). Dashboard chip + CLI bench.

KEY DECISIONS

D-1: governed_prompt_decision as the production call site
  prompt_decision has ZERO external callers today (exploration confirmed, IN-11).
  I-33 creates governed_prompt_decision(prompt, runtime, cefl, i6q, ssb, seed) in
  decision_governance.rs. This function is the production call site; it wraps
  prompt_decision and is called from bench_governance.rs (the fixture driver).
  The bench_governance entrypoint satisfies wiring-enforcement R1/R4 (real CLI entrypoint,
  off-author artifact). Future orchestrator callers route through this same function.

D-2: CEFL via DecodingMode::JsonSchema — both lanes
  Cloud: grounding_client.rs:767 translates JsonSchema into Anthropic Tool API input_schema.
  Sidecar: server.py:2257 lm-format-enforcer JsonSchemaParser. Grammar/Regex are sidecar-only
  with cloud graceful-fallback; JsonSchema is supported on BOTH lanes. CEFL uses JsonSchema only.
  CeflRequest::apply returns the augmented prompt + produces a ConstrainedDecodingConfig via
  ConstrainedDecodingConfig::json_schema. The caller attaches the config to CompletionConfig.
  NOTE: CompletionConfig currently has no decoding_mode field (verified). I-33 adds it as
  Option<ConstrainedDecodingConfig> so CEFL attaches the schema without breaking existing callers.

D-3: I6Q thresholds as orchestrator-owned config (not model-settable)
  min_len=20 chars, min_unique_words=5 (defaults). These are I6qValidator constructor args;
  the bench fixture uses these defaults. Adaptive calibration is deferred (Tier-2).

D-4: CommitRevealRecord seed from caller-supplied u64
  SeededClock is a tick-only clock, not an entropy source (clock.rs:117).
  For bench fixture: seed comes from --seed CLI arg (default 42). Production seed injection
  is UNRESOLVED — registered as PENDING checkbox. The record still ships and the ordering
  proof (commit_row_id < reveal_row_id) is verified deterministically.

D-5: GovernancePrimitiveFired Pattern-2, 3-arm Kind
  Follows DirectiveExecuted (ssb.rs:27661) exactly. One SsbActionType arm, one payload struct,
  kind discriminator inside payload. action_type string = "governance_primitive_fired".
  SCHEMA_VERSION 239->240, TOTAL_VARIANTS 548->549.

D-6: CompletionConfig decoding_mode field addition
  CompletionConfig is a struct in crates/providers/src/. Adding Option<ConstrainedDecodingConfig>
  is backwards-compatible (Option wraps; all existing call sites pass None implicitly because
  struct update is not used — constructors must be updated). Verified: CompletionConfig is
  constructed with named fields in hook_prompt.rs:232 and other sites; the added field with
  default None requires no changes to those sites if we use struct update syntax or a builder.
  Decision: add the field with #[serde(default)] + Default, use ..Default::default() at
  existing construction sites that only set known fields.

EXPECTED TEST OUTCOME
  cargo test -p elai-orchestrator --test i33_governance_contract -- all arms pass
  cargo test -p elai-orchestrator --lib -- all lib tests pass (format-string break check)
  cargo test -p elai-cli -- all CLI tests pass (new BenchGovernanceProbe arm compiled)
  action_type_lock 2/2: schema 240, total 549
  e45 drift: PASS (commands_i33.rs + types.ts + stub.ts cross-lang-pair annotation)
  i1 5/5: no regression
  i8/i10/i34/i38/i43/h39: no regression (re-read floor checks)

UNKNOWNS
  U-1: CompletionConfig decoding_mode field -- need to verify all construction sites.
  U-2: Production seed injection for CommitRevealRecord (registered as PENDING).
  U-3: governed_prompt_decision async signature -- prompt_decision is async, so
       governed_prompt_decision must be async; bench_governance.rs uses tokio::runtime.

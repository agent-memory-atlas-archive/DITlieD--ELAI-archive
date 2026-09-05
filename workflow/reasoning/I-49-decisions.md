I-49 decisions — bedrock + azure direct lanes

Date: 2026-06-14
Session: I-49-exec
Author: executor (main LLM via EXECUTOR prompt, no orchestrate agent loop)

WHAT
  - Adds two RuntimeAdapter backend lanes: AWS Bedrock (SigV4 + EventStream) and
    Azure OpenAI (chat-completions shape + openai_compat normalization shim).
  - Tier-1 deliverable: protocol logic + offline replay-fixture contract tests.
  - Tier-2 deferred: live billed network calls (D-026 PENDING, both backends).

KEY DECISIONS

D1: HAND-ROLLED HMAC-SHA256 (controller fork resolution A177 — Option A)
  Decision: Use sha2 0.10 (already in providers/Cargo.toml:27, confirmed via grep)
  with a hand-rolled IPAD/OPAD HMAC construction (RFC 2104). Zero new deps.
  Rationale: the `hmac` crate is absent from the entire workspace (grep-confirmed
  zero hits in all Cargo.toml files). The controller pre-resolved this as Option A.
  Hand-rolling ~40 lines of IPAD/OPAD logic avoids any new dependency risk.
  Grounding: RFC 2104 / FIPS 198-1 canonical HMAC construction.
  Verification: 3 RFC 4231 test vectors pass in both bedrock.rs unit tests and
  i49_replay_fixtures.rs contract tests.

D2: NO NEW SSB ACTION_TYPE (plan GD-1 confirmed)
  Decision: Zero SSB changes in this changeset.
  Rationale: Plan WIRING_CONTRACT explicitly says "NO new action_type";
  RouteDecisionEvent / existing runtime rows name the new backend ids.
  SCHEMA_VERSION and action_type_lock.golden are UNTOUCHED.
  Verification: zero modifications to ssb.rs / ssb/ / action_type_lock.golden.

D3: SHIM LIVES IN azure_openai.rs, NOT genai_adapter (exploration DRIFT fix)
  Decision: The OpenAI->Anthropic normalization shim is in azure_openai.rs as
  `normalize_azure_chat_response` + `build_azure_chat_request`, NOT in genai_adapter.
  Rationale: genai_adapter.rs is an E-34 NotImplemented scaffold for the genai CRATE
  (has no OpenAI normalizer code). The real prior art is openai_compat.rs which
  already has a full normalize_response path. The I-49 shim extends the same pattern
  without dragging the unverified `genai` crate dep. Confirmed by exploration note
  DRIFT block in I-49-bedrock-azure.md.

D4: BACKEND COUNT 20 -> 22 (not 22 from a different baseline)
  Decision: Bump the three backend-count assertions from 20 to 22.
  Grounding: mod.rs:284 was `assert_eq!(seen.len(), 20)` (grep-confirmed at HEAD
  95278ed). sglang_backend_contract.rs:154 was `assert_eq!(all.len(), 20)`.
  I-49 adds exactly 2 variants: Bedrock + AzureOpenAi. Target: 22. Correct.

D5: LIVE-CALL LEGS ARE WIRE-DARK[I-49] WITH ALLOWLIST ROW
  Decision: complete()/stream() on both adapters return a typed MissingField error
  (not NotImplemented — the adapter EXISTS). Both carry // WIRE-DARK[I-49]: markers.
  Two rows added to wire-dark-allowlist.toml (D-026 promotion_plan_id).
  Rationale: R9 requires allowlist row for any registered WIRE-DARK stub.

D6: REGION + DEPLOYMENT VIA runtime_profile CONVENTION
  Decision: BedrockRuntime reads region from runtime_profile as "<akid>:<region>";
  AzureOpenAiRuntime reads api_version from runtime_profile. Both fall back to
  env vars (AWS_ACCESS_KEY_ID / AWS_REGION / AZURE_OPENAI_API_KEY / AZURE_OPENAI_ENDPOINT).
  Rationale: RuntimeFactoryConfig has no dedicated aws_region / azure_deployment fields
  (adding them would be a wider types.rs change; the exploration note flags this as LOW
  severity and the convention is consistent with how Composer2 uses runtime_profile).
  I-50 credential broker will formalize this when it ships.

EXPECTED TEST OUTCOME
  cargo test -p elai-providers --test i49_replay_fixtures -> all pass
    - hmac_sha256_rfc4231_test_vector_1,2,3 (HMAC correctness)
    - sigv4_derive_signing_key_is_deterministic / different_service_different_key
    - sigv4_authorization_header_structure / sigv4_signing_is_deterministic
    - eventstream_valid_frame_decodes_byte_exactly
    - eventstream_empty_payload_decodes
    - eventstream_corrupted_frame_too_short_returns_typed_error
    - eventstream_prelude_crc_mismatch_returns_typed_error
    - eventstream_message_crc_mismatch_returns_typed_error
    - azure_normalize_response_happy_path
    - azure_normalize_missing_choices_is_typed_error
    - azure_normalize_empty_choices_is_typed_error
    - azure_build_request_includes_system_message
    - runtime_backend_all_count_is_22
    - resolve_backend_bedrock_aliases / resolve_backend_azure_openai_aliases
    - create_runtime_bedrock_missing_creds_returns_missing_field_not_not_implemented
    - create_runtime_azure_missing_creds_returns_missing_field_not_not_implemented
    - runtime_backend_all_includes_bedrock_and_azure
    - canonical_name_round_trips_for_new_backends
    - new_backends_are_not_local

  cargo test -p elai-providers --test sglang_backend_contract -> pass (bumped to 22)

UNKNOWNS / DEFERRED
  - Live Bedrock Converse call (D-026 checkbox, requires AWS creds + deployment)
  - Live Azure OpenAI chat call (D-026 checkbox, requires Azure deployment + key)
  - I-50 credential broker (will formalize AWS/Azure cred supply via RuntimeFactoryConfig)
  - The aws_region / azure_deployment dedicated fields on RuntimeFactoryConfig (minor,
    LOW severity per exploration; runtime_profile convention is the bridge until I-50)

sha2 DEP CONFIRMATION
  sha2 = "0.10" confirmed at crates/providers/Cargo.toml:27 (lock: sha2 v0.10.9).
  NO hmac / aws-sigv4 / aws-smithy-eventstream / aws-credential-types crate added.
  ZERO new dependencies in this changeset.

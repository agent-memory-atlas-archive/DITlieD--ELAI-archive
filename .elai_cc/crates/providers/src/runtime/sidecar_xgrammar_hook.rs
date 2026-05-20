//! E-35 (HOOK STUB): wire-format contract for the sidecar's XGrammar
//! LogitsProcessor.
//!
//! ## Why this file exists separately from `sidecar.rs`
//!
//! The Python side of the XGrammar / INSTRUCTOR mode work lives in
//! `python/elai_inference/server.py` and is owned by a DIFFERENT
//! orchestration session under E-35. To avoid two sessions racing on the
//! same file, this Rust-side adapter is co-authored here as the canonical
//! wire-format reference: it freezes the request/response types, the schema
//! identifiers, the cache-hit invariants and the SSB beacons the Python
//! side MUST emit. The Python session reads this file (as documentation +
//! `XGrammarRequest`/`XGrammarResponse` typed contract) and implements the
//! matching producer + consumer.
//!
//! This file deliberately introduces NO new dependency, NO `Provider`
//! trait method on `SidecarProvider`, and NO network call. The only public
//! surface is:
//!
//! * `SidecarXGrammarHook` — the trait two implementors satisfy
//!   (production `HttpSidecarXGrammarHook` — DEFERRED to the sidecar
//!   session — and the `#[cfg(test)] MockSidecarXGrammarHook` here which
//!   round-trips fixtures without touching the network).
//! * `XGrammarHookError` — the typed error surface (no `String` panics,
//!   no `.unwrap()` outside `#[cfg(test)]`).
//! * `xgrammar_cache_key` — the canonical `(role, block, blake3(schema))`
//!   key the disk cache uses; pure-Rust, exercised by tests below.
//! * `xgrammar_schema_id` — canonical `"{role}.{block}"` identifier the
//!   Python side uses as the cache primary key.
//!
//! ## followups_for_other_sessions
//!
//! Wire `XGrammarLogitsProcessor` into `python/elai_inference/server.py`
//! per the contract in this file (`XGrammarRequest` / `XGrammarResponse`
//! shape + `cache_hit` invariant + `mask_applied_count` invariant). The
//! Python side MUST:
//!
//! 1. Pre-compile 64 grammars (8 AgentRole values × 8 BabyStepSpec blocks)
//!    at boot, keyed by `xgrammar_cache_key`.
//! 2. Persist compiled grammars under `.elai_cc/.elai_cc/grammar_cache/{hash}.bin`.
//! 3. Refuse on-demand compile at request time — return an error frame
//!    if the `schema_hash` is unknown (CacheMiss → emits
//!    `legacy_json_object_refused` SSB row from the sidecar's Anthropic
//!    path is the chosen escalation; this file MUST NOT silently recompile).
//! 4. Emit `xgrammar_mask_applied`, `instructor_mode_activated`,
//!    `grammar_mode_activated`, and `grammar_cache_loaded` SSB rows from
//!    the orchestrator side once the Python sidecar returns success.
//! 5. Use `torch.inference_mode()` (NEVER `torch.no_grad()` per CLAUDE.md
//!    anti-stale) inside `XGrammarLogitsProcessor.__call__`.
//! 6. Use blake3 for the schema-hash key (NEVER blake2/sha2).
//!
//! Additionally, also still to land in OTHER sessions:
//! * Implement `python/elai_inference/tool_parser.py` with the
//!   `ToolParserVariant` resolver (`<tools>` for Qwen2.5-Coder, hermes
//!   for non-Coder Qwen / Llama / DeepSeek). Spec: this file's
//!   `XGrammarRequest::tool_parser` field is the wire contract.
//! * Add `Provider::generate_structured` in `runtime/sidecar.rs`
//!   AS A NEW METHOD (single additive method at end of impl block; do
//!   NOT refactor `complete` / `stream`). The implementation calls into
//!   the hook below.
//! * Add `crates/orchestrator/src/agent_role_dispatcher.rs::AgentRole::Runner`
//!   variant + CodeAct dispatch (the typed `CodeActAction` lives in
//!   `elai_ipc_types` already; the dispatch wiring is owned by the
//!   orchestrator session).
//! * Add `SsbActionType::XgrammarMaskApplied`, `InstructorModeActivated`,
//!   `GrammarModeActivated`, `CodeactActionExecuted`,
//!   `SearchReplaceTier1Hit`, `SearchReplaceCollisionFallback`,
//!   `CloudStrictSchemaApplied`, `GrammarCacheLoaded`,
//!   `StructuredEngineRejected`, `LegacyJsonObjectRefused` (10 rows)
//!   to `orchestrator/src/ssb.rs` with the schema_version bump (owned by
//!   the orchestrator session; this file does NOT touch ssb.rs).
//! * Wire `CloudGroundingClient` strict-mode passthrough (Anthropic
//!   `structured-outputs-2025-11-13`, OpenAI `strict: true`, Gemini
//!   `responseJsonSchema`). The typed `XGrammarRequest` here is reused for
//!   the cloud path — the cloud sender translates `schema_id` to the
//!   provider-specific strict body.
//!
//! Citations: arXiv:2411.15100 (XGrammar MLSys 2025), arXiv:2501.10868
//! (JSONSchemaBench), arXiv:2502.09061 (CRANE), arXiv:2402.01030 (CodeAct
//! ICML 2024), arXiv:2502.14905 ("Think Inside the JSON"), arXiv:2603.03305
//! (DCCD).

use std::future::Future;
use std::pin::Pin;

use elai_ipc_types::{StructuredMode, ToolParserVariant, XGrammarRequest, XGrammarResponse};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Typed error surface for the XGrammar hook. Every variant has a stable
/// snake_case label suitable for SSB beacons + log lines. No `.unwrap()`
/// is used in this module outside `#[cfg(test)]`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum XGrammarHookError {
    /// The schema identified by `(schema_id, schema_hash)` is not present
    /// in the sidecar's pre-compiled cache. The sidecar MUST NOT silently
    /// recompile — this is a misconfiguration and the orchestrator must
    /// escalate per WIRING_CONTRACT::stub_ban (legacy_json_object_refused
    /// SSB row from the cloud path is the chosen consumer escalation).
    #[error("schema cache miss: schema_id={schema_id} schema_hash={schema_hash}")]
    CacheMiss {
        schema_id: String,
        schema_hash: String,
    },
    /// The Python side returned a response whose `schema_id` or
    /// `schema_hash` does not match the request — the consumer cannot
    /// trust the body. Surface and fail loud.
    #[error("schema identity mismatch: req=({req_id},{req_hash}) resp=({resp_id},{resp_hash})")]
    SchemaIdentityMismatch {
        req_id: String,
        req_hash: String,
        resp_id: String,
        resp_hash: String,
    },
    /// `XGrammarResponse.mask_applied_count == 0` — the mask never fired,
    /// which contradicts the contract. Refuse the body.
    #[error("XGrammar mask never applied (mask_applied_count == 0)")]
    MaskNeverApplied,
    /// `XGrammarResponse.cache_hit == false` — the sidecar fell back to
    /// on-demand compile. The orchestrator refuses on-demand compiles per
    /// WIRING_CONTRACT::stub_ban.
    #[error("sidecar fell back to on-demand compile (cache_hit == false)")]
    OnDemandCompileForbidden,
    /// Instructor-mode contract: response.think_text must be Some.
    /// Grammar-mode contract: response.think_text must be None. Either
    /// invariant violation lands here.
    #[error("structured-mode contract violation: mode={mode} think_present={think_present}")]
    ModeContractViolation { mode: String, think_present: bool },
    /// Transport / serialisation failure from the Python side. The string
    /// is a stable category prefix (e.g. "network:", "decode:") followed
    /// by a free-form detail; consumers MAY match on the prefix.
    #[error("transport error: {category}")]
    Transport { category: String },
}

// ---------------------------------------------------------------------------
// Canonical cache-key helpers
// ---------------------------------------------------------------------------

/// Canonical "{role}.{block}" identifier the sidecar uses as the schema
/// primary key. Both `role` and `block` MUST be the snake_case label the
/// orchestrator side already emits.
///
/// Example: `xgrammar_schema_id("auditor", "findings") == "auditor.findings"`.
pub fn xgrammar_schema_id(role: &str, block: &str) -> String {
    let mut out = String::with_capacity(role.len() + 1 + block.len());
    out.push_str(role);
    out.push('.');
    out.push_str(block);
    out
}

/// Canonical disk-cache key for a (role, block, schema_canonical_json)
/// triple. The result is a hex-encoded `blake3` digest of:
/// `"{role}.{block}\n{schema_canonical_json}"`. CLAUDE.md anti-stale:
/// blake3, NEVER blake2/sha2.
pub fn xgrammar_cache_key(role: &str, block: &str, schema_canonical_json: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(role.as_bytes());
    hasher.update(b".");
    hasher.update(block.as_bytes());
    hasher.update(b"\n");
    hasher.update(schema_canonical_json.as_bytes());
    let digest = hasher.finalize();
    digest.to_hex().to_string()
}

// ---------------------------------------------------------------------------
// SidecarXGrammarHook trait
// ---------------------------------------------------------------------------

/// Hook surface for shipping a typed `XGrammarRequest` to the sidecar's
/// `XGrammarLogitsProcessor` and receiving a verified `XGrammarResponse`.
///
/// Implementors:
/// * `HttpSidecarXGrammarHook` (PRODUCTION, deferred — wired in the
///   sidecar-owning session) — POSTs to the sidecar's `/v1/messages`
///   endpoint with the XGrammar payload wrapped in a tool turn.
/// * `MockSidecarXGrammarHook` (TEST, included below) — round-trips
///   fixtures so the orchestrator can be exercised without booting the
///   Python sidecar.
///
/// CLAUDE.md anti-stale: native async fn in traits (NEVER async-trait macro).
/// We desugar manually so the trait stays `dyn`-compatible.
pub trait SidecarXGrammarHook: Send + Sync {
    /// Submit a typed `XGrammarRequest` and receive a verified
    /// `XGrammarResponse`. The implementation MUST run
    /// [`verify_response_contract`] on the response before returning.
    fn submit<'a>(
        &'a self,
        request: &'a XGrammarRequest,
    ) -> Pin<Box<dyn Future<Output = Result<XGrammarResponse, XGrammarHookError>> + Send + 'a>>;
}

/// Verify the wire-format contract on a `(request, response)` pair. Pulled
/// out so both production and test implementations share the same
/// validator (and so the validator is independently testable).
///
/// Checks:
/// 1. `schema_id` matches the request.
/// 2. `schema_hash` matches the request.
/// 3. `mask_applied_count > 0`.
/// 4. `cache_hit == true` (on-demand compile forbidden).
/// 5. `mode == Instructor` ⇔ `think_text.is_some()`.
pub fn verify_response_contract(
    request: &XGrammarRequest,
    response: &XGrammarResponse,
) -> Result<(), XGrammarHookError> {
    if request.schema_id != response.schema_id || request.schema_hash != response.schema_hash {
        return Err(XGrammarHookError::SchemaIdentityMismatch {
            req_id: request.schema_id.clone(),
            req_hash: request.schema_hash.clone(),
            resp_id: response.schema_id.clone(),
            resp_hash: response.schema_hash.clone(),
        });
    }
    if response.mask_applied_count == 0 {
        return Err(XGrammarHookError::MaskNeverApplied);
    }
    if !response.cache_hit {
        return Err(XGrammarHookError::OnDemandCompileForbidden);
    }
    let think_present = response.think_text.is_some();
    let want_think = matches!(request.mode, StructuredMode::Instructor);
    if think_present != want_think {
        return Err(XGrammarHookError::ModeContractViolation {
            mode: request.mode.as_str().to_string(),
            think_present,
        });
    }
    Ok(())
}

/// Construct a canonical `XGrammarRequest` from per-call inputs. Centralised
/// so the orchestrator side does not handcraft the struct at each call
/// site (every field has an invariant; missing one is a bug we want caught
/// at compile time).
pub fn build_xgrammar_request(
    role: &str,
    block: &str,
    schema_canonical_json: &str,
    mode: StructuredMode,
    tool_parser: Option<ToolParserVariant>,
    prompt: impl Into<String>,
    max_tokens: u32,
) -> XGrammarRequest {
    XGrammarRequest {
        schema_id: xgrammar_schema_id(role, block),
        schema_hash: xgrammar_cache_key(role, block, schema_canonical_json),
        mode,
        tool_parser,
        prompt: prompt.into(),
        max_tokens,
    }
}

// ---------------------------------------------------------------------------
// Test mock + tests
// ---------------------------------------------------------------------------

#[cfg(test)]
pub mod mock {
    //! Test-only mock implementation of `SidecarXGrammarHook` that
    //! round-trips a canned `XGrammarResponse` per `schema_id`. Used by
    //! the smoke test below + by any orchestrator-side test that needs
    //! a hook without booting the Python sidecar.

    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    pub struct MockSidecarXGrammarHook {
        responses: Mutex<HashMap<String, XGrammarResponse>>,
    }

    impl MockSidecarXGrammarHook {
        pub fn new() -> Self {
            Self {
                responses: Mutex::new(HashMap::new()),
            }
        }

        /// Pre-register a canned response keyed by `schema_id`. The mock
        /// echoes the request's `schema_hash` into the response so the
        /// identity check passes.
        pub fn register(&self, schema_id: &str, mut canned: XGrammarResponse) {
            canned.schema_id = schema_id.to_string();
            // .lock() in test code is fine (CLAUDE.md anti-stale exception:
            // test code + #[cfg(test)] block).
            let mut guard = self.responses.lock().expect("mock mutex poisoned");
            guard.insert(schema_id.to_string(), canned);
        }
    }

    impl Default for MockSidecarXGrammarHook {
        fn default() -> Self {
            Self::new()
        }
    }

    impl SidecarXGrammarHook for MockSidecarXGrammarHook {
        fn submit<'a>(
            &'a self,
            request: &'a XGrammarRequest,
        ) -> Pin<Box<dyn Future<Output = Result<XGrammarResponse, XGrammarHookError>> + Send + 'a>>
        {
            Box::pin(async move {
                let guard = self.responses.lock().expect("mock mutex poisoned");
                let canned = guard.get(&request.schema_id).cloned().ok_or_else(|| {
                    XGrammarHookError::CacheMiss {
                        schema_id: request.schema_id.clone(),
                        schema_hash: request.schema_hash.clone(),
                    }
                })?;
                // Echo the request's schema_hash so the identity check passes.
                let mut resp = canned;
                resp.schema_hash = request.schema_hash.clone();
                verify_response_contract(request, &resp)?;
                Ok(resp)
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock::MockSidecarXGrammarHook;

    fn fixture_request_instructor() -> XGrammarRequest {
        build_xgrammar_request(
            "auditor",
            "findings",
            r#"{"type":"object","required":["findings"]}"#,
            StructuredMode::Instructor,
            None,
            "audit the last build",
            512,
        )
    }

    fn fixture_response_instructor(req: &XGrammarRequest) -> XGrammarResponse {
        XGrammarResponse {
            schema_id: req.schema_id.clone(),
            schema_hash: req.schema_hash.clone(),
            answer_text: r#"{"findings":[]}"#.to_string(),
            think_text: Some("scanned outputs".to_string()),
            mask_applied_count: 12,
            generation_ms: 311,
            cache_hit: true,
        }
    }

    #[test]
    fn schema_id_is_role_dot_block() {
        assert_eq!(xgrammar_schema_id("auditor", "findings"), "auditor.findings");
        assert_eq!(xgrammar_schema_id("planner", "spec"), "planner.spec");
    }

    #[test]
    fn cache_key_is_deterministic_and_hex() {
        let a = xgrammar_cache_key("auditor", "findings", "{}");
        let b = xgrammar_cache_key("auditor", "findings", "{}");
        assert_eq!(a, b);
        // blake3 hex is 64 chars.
        assert_eq!(a.len(), 64);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn cache_key_changes_on_any_input_change() {
        let base = xgrammar_cache_key("auditor", "findings", "{}");
        assert_ne!(base, xgrammar_cache_key("planner", "findings", "{}"));
        assert_ne!(base, xgrammar_cache_key("auditor", "spec", "{}"));
        assert_ne!(
            base,
            xgrammar_cache_key("auditor", "findings", r#"{"x":1}"#)
        );
    }

    #[test]
    fn verify_response_contract_accepts_instructor() {
        let req = fixture_request_instructor();
        let resp = fixture_response_instructor(&req);
        verify_response_contract(&req, &resp).expect("contract should pass");
    }

    #[test]
    fn verify_response_contract_rejects_zero_mask() {
        let req = fixture_request_instructor();
        let mut resp = fixture_response_instructor(&req);
        resp.mask_applied_count = 0;
        let err = verify_response_contract(&req, &resp).unwrap_err();
        assert!(matches!(err, XGrammarHookError::MaskNeverApplied));
    }

    #[test]
    fn verify_response_contract_rejects_on_demand_compile() {
        let req = fixture_request_instructor();
        let mut resp = fixture_response_instructor(&req);
        resp.cache_hit = false;
        let err = verify_response_contract(&req, &resp).unwrap_err();
        assert!(matches!(err, XGrammarHookError::OnDemandCompileForbidden));
    }

    #[test]
    fn verify_response_contract_rejects_mode_mismatch() {
        // Instructor request must have think_text.
        let req = fixture_request_instructor();
        let mut resp = fixture_response_instructor(&req);
        resp.think_text = None;
        let err = verify_response_contract(&req, &resp).unwrap_err();
        match err {
            XGrammarHookError::ModeContractViolation { mode, think_present } => {
                assert_eq!(mode, "instructor");
                assert!(!think_present);
            }
            other => panic!("expected ModeContractViolation, got {other:?}"),
        }
    }

    #[test]
    fn verify_response_contract_rejects_grammar_with_think() {
        let req = build_xgrammar_request(
            "auditor",
            "status_flag",
            r#"{"enum":["pass","fail"]}"#,
            StructuredMode::Grammar,
            None,
            "ok?",
            16,
        );
        let resp = XGrammarResponse {
            schema_id: req.schema_id.clone(),
            schema_hash: req.schema_hash.clone(),
            answer_text: "pass".to_string(),
            think_text: Some("ought not be set".to_string()),
            mask_applied_count: 1,
            generation_ms: 4,
            cache_hit: true,
        };
        let err = verify_response_contract(&req, &resp).unwrap_err();
        assert!(matches!(
            err,
            XGrammarHookError::ModeContractViolation { .. }
        ));
    }

    #[test]
    fn verify_response_contract_rejects_schema_identity_mismatch() {
        let req = fixture_request_instructor();
        let mut resp = fixture_response_instructor(&req);
        resp.schema_id = "planner.spec".to_string();
        let err = verify_response_contract(&req, &resp).unwrap_err();
        assert!(matches!(err, XGrammarHookError::SchemaIdentityMismatch { .. }));
    }

    #[tokio::test]
    async fn mock_round_trips_known_schema() {
        let req = fixture_request_instructor();
        let resp = fixture_response_instructor(&req);
        let hook = MockSidecarXGrammarHook::new();
        hook.register("auditor.findings", resp.clone());
        let got = hook.submit(&req).await.expect("submit ok");
        assert_eq!(got.answer_text, resp.answer_text);
        assert_eq!(got.mask_applied_count, resp.mask_applied_count);
        assert!(got.cache_hit);
    }

    #[tokio::test]
    async fn mock_returns_cache_miss_for_unknown_schema() {
        let req = fixture_request_instructor();
        let hook = MockSidecarXGrammarHook::new();
        let err = hook.submit(&req).await.unwrap_err();
        assert!(matches!(err, XGrammarHookError::CacheMiss { .. }));
    }
}

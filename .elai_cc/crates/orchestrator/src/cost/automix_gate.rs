//! E-39 -- AutoMix self-verify gate (arXiv:2310.12963).
//!
//! Runs a CHEAP local-verifier between an L3 failure and a HYBRID
//! cloud escalation. The verifier asks "is the failed output
//! structurally / semantically acceptable? if not, can a same-tier
//! retry rescue it, or is escalation justified?" and answers with one
//! of [`AutomixVerdict::Continue`] / [`AutomixVerdict::Retry`] /
//! [`AutomixVerdict::Escalate`].
//!
//! Per the paper (arxiv.org/abs/2310.12963 + automix-llm.github.io/
//! automix/), AutoMix observes >50% reduction in compute cost at
//! comparable performance across 5 datasets x 5 LMs because the
//! verifier catches "good enough" outputs that would otherwise
//! escalate. The verifier overhead is bounded by the local-sidecar
//! call cost ($0 marginal per memory:project_qwen3b_baseline).
//!
//! The verifier prompt is small + structured. The sidecar is asked to
//! emit a constrained JSON object `{ verdict, reason }` so a parser
//! failure surfaces as `Continue` rather than a silent panic. The
//! verifier verdict is logged via [`crate::ssb::CostEventKind::
//! AutomixVerifierVerdict`] regardless of which arm fires.

use std::fmt;

/// 3-arm verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutomixVerdict {
    /// Failed output is acceptable as-is; do NOT escalate.
    Continue,
    /// One more same-tier retry is justified.
    Retry,
    /// Proceed to cloud escalation.
    Escalate,
}

impl AutomixVerdict {
    /// Snake-case wire label.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Continue => "continue",
            Self::Retry => "retry",
            Self::Escalate => "escalate",
        }
    }
}

impl fmt::Display for AutomixVerdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Typed error surface for the verifier call. Never panics.
#[derive(Debug, thiserror::Error)]
pub enum AutomixError {
    #[error("verifier sidecar call failed: {0}")]
    SidecarFailed(String),
    #[error("verifier returned malformed JSON: {0}")]
    MalformedResponse(String),
    #[error("verifier disabled via ELAI_AUTOMIX_GATE_DISABLE=1")]
    Disabled,
}

/// Trait so the orchestrator can inject either the live
/// `SidecarGroundingClient` or a deterministic test double.
pub trait VerifierClient {
    /// Send the structured verifier prompt and return the JSON
    /// response body. Implementations must not panic on transport
    /// failures -- surface them as `AutomixError::SidecarFailed`.
    fn verify(&self, system: &str, user: &str) -> Result<String, AutomixError>;
}

/// Optional env-knob to disable the gate entirely (defaults off). When
/// set to "1", every call to [`verify_before_escalate`] returns
/// `Err(AutomixError::Disabled)`. The caller in the cloud-dispatch
/// path interprets `Disabled` as "fall through to the original
/// C-5 escalation"; the SSB row is still written so the dashboard
/// shows the disabled state.
pub const ENV_AUTOMIX_GATE_DISABLE: &str = "ELAI_AUTOMIX_GATE_DISABLE";

fn gate_disabled() -> bool {
    std::env::var(ENV_AUTOMIX_GATE_DISABLE)
        .ok()
        .map(|v| v == "1")
        .unwrap_or(false)
}

const VERIFIER_SYSTEM_PROMPT: &str = "You are an output verifier. Given the user task and the candidate output, decide:\n- `continue`: the output is good enough; downstream may accept it\n- `retry`: one more same-tier attempt is likely to succeed\n- `escalate`: a stronger model is required\nReturn STRICT JSON: {\"verdict\":\"continue\"|\"retry\"|\"escalate\",\"reason\":\"<1-sentence>\"}";

/// Compose the user-side verifier prompt. Pulled into a free function
/// so tests can assert the wire shape.
#[must_use]
pub fn build_verifier_user_prompt(task_summary: &str, failed_output: &str) -> String {
    format!(
        "TASK:\n{}\n\nCANDIDATE OUTPUT:\n{}\n\nReturn the JSON verdict now.",
        task_summary, failed_output
    )
}

/// Parse the verifier JSON. On parse failure or unrecognised verdict
/// string, surface `AutomixError::MalformedResponse` so the caller can
/// fall back to escalation (fail-closed-toward-cloud is the
/// conservative default; the alternative would silently swallow real
/// failures).
pub fn parse_verifier_response(body: &str) -> Result<(AutomixVerdict, String), AutomixError> {
    let v: serde_json::Value = serde_json::from_str(body.trim())
        .map_err(|e| AutomixError::MalformedResponse(e.to_string()))?;
    let verdict_str = v
        .get("verdict")
        .and_then(|x| x.as_str())
        .ok_or_else(|| AutomixError::MalformedResponse("missing `verdict`".to_string()))?;
    let reason = v
        .get("reason")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();
    let verdict = match verdict_str {
        "continue" => AutomixVerdict::Continue,
        "retry" => AutomixVerdict::Retry,
        "escalate" => AutomixVerdict::Escalate,
        other => {
            return Err(AutomixError::MalformedResponse(format!(
                "unknown verdict `{other}`"
            )));
        }
    };
    Ok((verdict, reason))
}

/// Run the verifier and return its verdict. Failures surface as `Err`
/// so the caller can route them to escalation rather than swallowing.
pub fn verify_before_escalate<V: VerifierClient>(
    client: &V,
    task_summary: &str,
    failed_output: &str,
) -> Result<(AutomixVerdict, String), AutomixError> {
    if gate_disabled() {
        return Err(AutomixError::Disabled);
    }
    let user_prompt = build_verifier_user_prompt(task_summary, failed_output);
    let body = client.verify(VERIFIER_SYSTEM_PROMPT, &user_prompt)?;
    parse_verifier_response(&body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct StubClient {
        body: RefCell<String>,
        err: Option<String>,
    }

    impl StubClient {
        fn ok(body: &str) -> Self {
            Self {
                body: RefCell::new(body.to_string()),
                err: None,
            }
        }
        fn err(msg: &str) -> Self {
            Self {
                body: RefCell::new(String::new()),
                err: Some(msg.to_string()),
            }
        }
    }

    impl VerifierClient for StubClient {
        fn verify(&self, _system: &str, _user: &str) -> Result<String, AutomixError> {
            if let Some(e) = &self.err {
                return Err(AutomixError::SidecarFailed(e.clone()));
            }
            Ok(self.body.borrow().clone())
        }
    }

    #[test]
    fn verdict_labels_stable() {
        assert_eq!(AutomixVerdict::Continue.as_str(), "continue");
        assert_eq!(AutomixVerdict::Retry.as_str(), "retry");
        assert_eq!(AutomixVerdict::Escalate.as_str(), "escalate");
    }

    #[test]
    fn parse_continue_verdict() {
        let body = r#"{"verdict":"continue","reason":"output is structurally valid"}"#;
        let (v, r) = parse_verifier_response(body).expect("parse");
        assert_eq!(v, AutomixVerdict::Continue);
        assert_eq!(r, "output is structurally valid");
    }

    #[test]
    fn parse_unknown_verdict_returns_err() {
        let body = r#"{"verdict":"banana","reason":""}"#;
        let err = parse_verifier_response(body).expect_err("unknown verdict must err");
        assert!(matches!(err, AutomixError::MalformedResponse(_)));
    }

    #[test]
    fn parse_malformed_json_returns_err() {
        let body = "not-json{";
        let err = parse_verifier_response(body).expect_err("malformed must err");
        assert!(matches!(err, AutomixError::MalformedResponse(_)));
    }

    #[test]
    fn verify_routes_through_stub_client() {
        let client = StubClient::ok(r#"{"verdict":"retry","reason":"borderline"}"#);
        let (v, _) =
            verify_before_escalate(&client, "task summary", "failed output").expect("verify");
        assert_eq!(v, AutomixVerdict::Retry);
    }

    #[test]
    fn verify_surfaces_sidecar_failure() {
        let client = StubClient::err("connection refused");
        let err = verify_before_escalate(&client, "t", "o").expect_err("must err");
        assert!(matches!(err, AutomixError::SidecarFailed(_)));
    }

    #[test]
    fn verifier_user_prompt_contains_task_and_output() {
        let p = build_verifier_user_prompt("compute x", "answer was 42");
        assert!(p.contains("TASK:"));
        assert!(p.contains("compute x"));
        assert!(p.contains("CANDIDATE OUTPUT:"));
        assert!(p.contains("answer was 42"));
    }
}

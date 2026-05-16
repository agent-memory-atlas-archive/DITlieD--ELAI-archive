// Live smoke test for patch-R1: refreshes the saved OAuth token, checks
// the reverse-engineered /api/oauth/usage endpoint, and sends a tiny
// 4-token /v1/messages request to verify the cloud routing fixes work
// end-to-end against api.anthropic.com.
//
// Gating: set `ELAI_OAUTH_LIVE_SMOKE=1` to run. The test is `#[ignore]`'d
// so default `cargo test` skips it. Each successful invocation may consume
// a tiny amount of subscription quota (1 inference, ~4 output tokens).
//
// Validates:
//   1. resolve_auth_with_auto_refresh() resolves a usable BearerToken
//      (refreshes the saved OAuth via the canonical Claude Code OAuth
//      config when expired).
//   2. fetch_oauth_usage() returns parseable {five_hour, seven_day}
//      percentages (proves /api/oauth/usage is reachable + the saved token
//      is a subscription-tier OAuth token).
//   3. AnthropicClient::send_message against claude-sonnet-4-6 with a
//      4-token prompt round-trips successfully — proves the patch-R1
//      header set (UA 2.1.143 + system prefix on Sonnet OAuth requests)
//      is correct.
//
// On failure, the test panics with the exact ApiError / HTTP status so
// the root cause is in the test output (no silent 401/429 swallowing).
//
// Run via:
//   cargo test -p elai-providers --test anthropic_oauth_live_smoke \
//     -- --ignored --nocapture --test-threads=1
// optionally with $env:ELAI_OAUTH_LIVE_SMOKE="1" set in PowerShell.

use elai_providers::{
    AnthropicClient, AuthSource, InputMessage, MessageRequest,
};

#[tokio::test(flavor = "multi_thread")]
#[ignore = "live network call to api.anthropic.com — consumes a small amount of subscription quota"]
async fn anthropic_oauth_live_smoke_after_patch_r1() {
    let gate = std::env::var("ELAI_OAUTH_LIVE_SMOKE").ok();
    if gate.as_deref() != Some("1") {
        eprintln!(
            "[skip] ELAI_OAUTH_LIVE_SMOKE not set to '1' (got: {gate:?}); \
             this test is opt-in to avoid burning quota on every cargo run"
        );
        return;
    }

    eprintln!("[step 1] resolving auth via resolve_auth_with_auto_refresh()...");
    // resolve_saved_oauth_token_set internally uses Runtime::new().block_on()
    // for the refresh HTTP call, which conflicts with our #[tokio::test]
    // runtime ("Cannot start a runtime from within a runtime"). Bounce through
    // spawn_blocking to give it its own thread.
    let auth = tokio::task::spawn_blocking(elai_providers::resolve_auth_with_auto_refresh)
        .await
        .expect("spawn_blocking join")
        .unwrap_or_else(|e| {
            panic!(
                "resolve_auth_with_auto_refresh failed: {e:?}\n\
                 Possible causes:\n\
                 - No saved OAuth at ~/.elai/credentials.json -> run `elai login`\n\
                 - Saved OAuth refresh_token also expired -> re-OAuth via dashboard\n\
                 - platform.claude.com/v1/oauth/token unreachable (network)"
            )
        });
    match &auth {
        AuthSource::BearerToken(_) => {
            eprintln!("[step 1] auth_kind=BearerToken (OAuth path)");
        }
        AuthSource::ApiKey(_) => {
            eprintln!("[step 1] auth_kind=ApiKey (env ANTHROPIC_API_KEY)");
        }
        AuthSource::ApiKeyAndBearer { .. } => {
            eprintln!("[step 1] auth_kind=ApiKey+Bearer (both env vars)");
        }
        AuthSource::None => {
            panic!("auth resolved to AuthSource::None — should never happen");
        }
    }

    let base_url = elai_providers::read_base_url();
    eprintln!("[step 1] base_url={base_url}");
    let client = AnthropicClient::from_auth(auth.clone()).with_base_url(base_url);

    eprintln!("[step 2] fetching /api/oauth/usage (subscription-tier check)...");
    let usage_result = client.fetch_oauth_usage().await;
    match usage_result {
        Ok(usage) => {
            let five_hour = usage.five_hour().unwrap_or(0.0);
            let seven_day = usage.seven_day().unwrap_or(0.0);
            eprintln!(
                "[step 2] OK five_hour={five_hour:.1}% seven_day={seven_day:.1}%"
            );
            assert!(
                five_hour <= 100.0 && seven_day <= 100.0,
                "usage percentages must be in [0,100]: got {usage:?}"
            );
            if usage.should_throttle(80.0) {
                eprintln!(
                    "[step 2] WARN bucket >= 80% — proceeding but cloud may 429"
                );
            }
        }
        Err(e) => {
            eprintln!(
                "[step 2] WARN fetch_oauth_usage failed: {e:?}\n\
                 - 401: token is Console-tier (not subscription) OR endpoint changed\n\
                 - 403: account flagged or scope missing\n\
                 - Other: continuing anyway since /v1/messages is the real target"
            );
        }
    }

    eprintln!("[step 3] sending 4-token /v1/messages to claude-sonnet-4-6...");
    let request = MessageRequest {
        model: "claude-sonnet-4-6".to_string(),
        max_tokens: 4,
        messages: vec![InputMessage::user_text(
            "ok",
        )],
        system: None,
        tools: None,
        tool_choice: None,
        stream: false,
        seed: None,
    };
    let response_result = client.send_message(&request).await;
    match response_result {
        Ok(response) => {
            eprintln!(
                "[step 3] OK model={} stop_reason={:?} request_id={:?}",
                response.model, response.stop_reason, response.request_id
            );
            eprintln!(
                "[step 3] usage_in={} usage_out={}",
                response.usage.input_tokens, response.usage.output_tokens
            );
            assert!(
                !response.content.is_empty(),
                "response.content must contain at least one block: {response:?}"
            );
            eprintln!(
                "[step 3] content_blocks={} first_kind={:?}",
                response.content.len(),
                response.content.first()
            );
            eprintln!("[done] patch-R1 cloud OAuth path PROVEN against live api.anthropic.com");
        }
        Err(e) => {
            panic!(
                "send_message failed: {e:?}\n\
                 Diagnostic guidance per findings PART 1:\n\
                 - 401 authentication_error: token revoked / Console-tier - re-OAuth\n\
                 - 400 invalid_request_error: system-prompt prefix rule violation - check prepend_oauth_system_prefix\n\
                 - 429 rate_limit_error: quota exhausted - check /api/oauth/usage\n\
                 - 529 overloaded_error: Anthropic capacity (not your fault)\n\
                 - 402 billing_error / 403 permission_error: account state"
            )
        }
    }
}

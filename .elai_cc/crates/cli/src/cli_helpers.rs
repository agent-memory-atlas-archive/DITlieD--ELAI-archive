// B-15 Tranche O.1 (orchestrate s-22600): main.rs helpers extraction.
// Hosts five top-level helpers carved out of crates/cli/src/main.rs to
// bound main.rs growth and group misc utilities under one named module.
//
// Visibility discipline:
//   - shortcoder_compress_text + CharEstimatorProvider + check_anthropic_gateway
//     stay pub(crate) (shortcoder is externally consumed by runtime_client.rs
//     via `use crate::*;` glob at runtime_client.rs:72; main.rs re-exports
//     `pub(crate) use crate::cli_helpers::*;` so that glob still resolves).
//   - cmd_incident_record + build_pr_review_action become module-private
//     (no pub) here since main.rs's `fn run` is their sole caller and reaches
//     them via the `use crate::cli_helpers::*;` import below.
//
// Constants: FENCE_LANG_MAP + TOKEN_CHARS_PER_TOKEN remain owned by main.rs;
// imported here via `use crate::{...};` so the helpers compile in isolation.

#![allow(dead_code)]

use std::future::Future;
use std::pin::Pin;

use crate::pr_review_cmd;
use crate::cli_consts::{FENCE_LANG_MAP, TOKEN_CHARS_PER_TOKEN};

/// Apply AST-preserving compression to code-fence regions of `text` using
/// `sc.simplify(...)`. Returns the post-compression character length and
/// adds the bytes saved to `saved`. Pure refactor extraction from main.rs
/// L303-L358 (Tranche O.1); algorithm byte-identical to pre-extraction
/// behavior. Consumed by runtime_client.rs L348 + L352 via `use crate::*;`
/// glob + main.rs re-export of `crate::cli_helpers::*`.
pub(crate) fn shortcoder_compress_text(text: &str, sc: &elai_knowledge::ShortCoder, saved: &mut usize) -> usize {
    if !text.contains("```") {
        return text.len();
    }
    let mut effective_chars = 0usize;
    let mut pos = 0;
    while pos < text.len() {
        if let Some(fence_start) = text[pos..].find("```") {
            let abs_fence_start = pos + fence_start;
            effective_chars += fence_start;
            let after_ticks = abs_fence_start + 3;
            let line_end = text[after_ticks..].find('\n')
                .map(|i| after_ticks + i)
                .unwrap_or(text.len());
            let lang_tag = text[after_ticks..line_end].trim().to_lowercase();
            let code_start = if line_end < text.len() { line_end + 1 } else { line_end };
            let closing = text[code_start..].find("\n```").map(|i| code_start + i);
            if let Some(close_pos) = closing {
                let code_content = &text[code_start..close_pos];
                let fence_end = close_pos + 4;
                let after_close = text[fence_end..].find('\n')
                    .map(|i| fence_end + i + 1)
                    .unwrap_or(text.len());
                let ts_lang = FENCE_LANG_MAP.iter()
                    .find(|(tag, _)| *tag == lang_tag)
                    .map(|(_, lang)| *lang);
                if let Some(lang) = ts_lang {
                    match sc.simplify(code_content, lang) {
                        Ok(result) if result.was_simplified() => {
                            let original_block_len = after_close - abs_fence_start;
                            let simplified_block_len = 3 + lang_tag.len() + 1
                                + result.simplified.len() + 1 + 3;
                            if simplified_block_len < original_block_len {
                                *saved += original_block_len - simplified_block_len;
                            }
                            effective_chars += simplified_block_len;
                        }
                        _ => {
                            effective_chars += after_close - abs_fence_start;
                        }
                    }
                } else {
                    effective_chars += after_close - abs_fence_start;
                }
                pos = after_close;
            } else {
                effective_chars += text.len() - abs_fence_start;
                pos = text.len();
            }
        } else {
            effective_chars += text.len() - pos;
            pos = text.len();
        }
    }
    effective_chars
}

/// Char-budget fallback LlmProvider. Used when the real provider is
/// unavailable but a token-count surface is still required. Returns
/// errors from complete() / embed() but services token_count() and
/// token_count_estimate() via a chars/4 heuristic. Pure refactor
/// extraction from main.rs L378-L403 (Tranche O.1).
pub(crate) struct CharEstimatorProvider;

impl elai_providers::LlmProvider for CharEstimatorProvider {
    fn complete<'a>(
        &'a self,
        _prompt: &'a str,
        _config: &'a elai_providers::OllamaConfig,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<String>, elai_providers::ApiError>> + Send + 'a>> {
        Box::pin(async { Err(elai_providers::ApiError::Auth("CharEstimatorProvider does not support completion".into())) })
    }

    fn embed<'a>(
        &'a self,
        _text: &'a str,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<f32>, elai_providers::ApiError>> + Send + 'a>> {
        Box::pin(async { Err(elai_providers::ApiError::Auth("CharEstimatorProvider does not support embedding".into())) })
    }

    fn token_count(&self, text: &str) -> Result<usize, elai_providers::ApiError> {
        Ok(text.len() / TOKEN_CHARS_PER_TOKEN)
    }

    fn token_count_estimate(&self, text: &str) -> usize {
        text.len() / TOKEN_CHARS_PER_TOKEN
    }
}

/// Gateway-rejection guard per E-24 Task 6.
/// Warns if ANTHROPIC_BASE_URL is set to a known LLM gateway URL.
/// Returns the gateway name if detected, None for safe or unknown URLs.
/// Hard-block requires --strict-no-gateway flag (not yet wired; warn only for now).
/// Pure refactor extraction from main.rs L448-L473 (Tranche O.1).
pub(crate) fn check_anthropic_gateway(base_url: &str) -> Option<String> {
    let known_gateways = [
        ("openrouter.ai", "openrouter"),
        ("litellm", "litellm"),
        ("portkey.ai", "portkey"),
        ("openai.azure.com", "azure-openai"),
    ];

    let url_lower = base_url.to_lowercase();
    for (pattern, name) in &known_gateways {
        if url_lower.contains(pattern) {
            return Some((*name).to_string());
        }
    }

    // Known-safe URLs: localhost, loopback, or official Anthropic API.
    if url_lower.starts_with("http://localhost:")
        || url_lower.starts_with("http://127.0.0.1:")
        || url_lower.starts_with("https://api.anthropic.com")
    {
        return None;
    }

    // Unknown URL -- warn but do not block (no --strict-no-gateway in args yet).
    None
}

/// E-41: assemble a `PrReviewAction` from the args layer's flat shape.
/// Returns the structured action OR an actionable error when a required
/// flag is missing for the given subverb. Validation lives here (not in
/// the args parser) so the args layer stays free of orchestrator types.
/// Pure refactor extraction from main.rs L1099-L1183 (Tranche O.1).
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_pr_review_action(
    sub: &str,
    pr: Option<u32>,
    body: Option<String>,
    file: Option<String>,
    line: Option<u32>,
    template: Option<String>,
    kv_pairs: Vec<(String, String)>,
    since: Option<i64>,
    ruleset: Option<String>,
    loc_added: Option<u32>,
    loc_removed: Option<u32>,
    files_changed: Option<u32>,
    touches_owned: bool,
    repo: Option<String>,
    branch: Option<String>,
    touched_files: Vec<String>,
    default_owner: Option<String>,
    title: Option<String>,
) -> anyhow::Result<pr_review_cmd::PrReviewAction> {
    use pr_review_cmd::PrReviewAction;
    let require_pr = |hint: &str| -> anyhow::Result<u32> {
        pr.ok_or_else(|| anyhow::anyhow!("pr-review {hint}: PR number positional required"))
    };
    match sub {
        "compose-body" => Ok(PrReviewAction::ComposeBody {
            pr: require_pr("compose-body")?,
            template: template
                .ok_or_else(|| anyhow::anyhow!("--template is required for compose-body"))?,
            kv_pairs,
        }),
        "comment" => Ok(PrReviewAction::Comment {
            pr: require_pr("comment")?,
            body: body.ok_or_else(|| anyhow::anyhow!("--body is required for comment"))?,
            file,
            line,
        }),
        "watch" => Ok(PrReviewAction::Watch {
            pr: require_pr("watch")?,
            since,
        }),
        "verify-deps" => Ok(PrReviewAction::VerifyDeps {
            pr: require_pr("verify-deps")?,
        }),
        "semgrep-gate" => Ok(PrReviewAction::SemgrepGate {
            pr: require_pr("semgrep-gate")?,
            ruleset: ruleset.unwrap_or_else(|| "auto".to_string()),
        }),
        "auto-merge-assess" => Ok(PrReviewAction::AutoMergeAssess {
            pr: require_pr("auto-merge-assess")?,
            loc_added: loc_added.unwrap_or(0),
            loc_removed: loc_removed.unwrap_or(0),
            files_changed: files_changed.unwrap_or(1),
            touches_owned,
        }),
        "apply-branch-protection" => Ok(PrReviewAction::ApplyBranchProtection {
            repo: repo.ok_or_else(|| {
                anyhow::anyhow!("--repo is required for apply-branch-protection")
            })?,
            branch: branch.unwrap_or_else(|| "main".to_string()),
        }),
        "gen-codeowners" => Ok(PrReviewAction::GenCodeowners {
            touched: touched_files
                .into_iter()
                .map(std::path::PathBuf::from)
                .collect(),
            default_owner: default_owner
                .ok_or_else(|| anyhow::anyhow!("--default-owner is required for gen-codeowners"))?,
        }),
        "suggest-squash" => Ok(PrReviewAction::SuggestSquash {
            title: title
                .ok_or_else(|| anyhow::anyhow!("--title is required for suggest-squash"))?,
            body_hint: body.unwrap_or_default(),
        }),
        "detect-force-push" => Ok(PrReviewAction::DetectForcePush {
            pr: require_pr("detect-force-push")?,
            repo,
        }),
        other => Err(anyhow::anyhow!(
            "unknown pr-review sub-action '{other}' (compose-body | comment | watch | verify-deps | \
             semgrep-gate | auto-merge-assess | apply-branch-protection | gen-codeowners | \
             suggest-squash | detect-force-push)"
        )),
    }
}

/// E-30: Store an incident postmortem entry in the SSB.
///
/// Emits `SsbActionType::IncidentRecordCreated` so the dashboard can surface
/// the entry in its activity feed. The incident_id defaults to a slug derived
/// from the title if not provided. Pure refactor extraction from main.rs
/// L1190-L1221 (Tranche O.1).
pub(crate) fn cmd_incident_record(
    title: String,
    incident_date: String,
    source_url: String,
    incident_id: Option<String>,
) -> anyhow::Result<()> {
    // Derive an ID from the title if none was provided.
    let id = incident_id.unwrap_or_else(|| {
        let slug: String = title
            .chars()
            .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        format!("inc-{slug}")
    });
    // Print the record to stdout in a machine-readable format.
    // The SSB write path requires a running runtime; for now we emit a
    // JSON record to stdout so CI scripts can ingest it. The full SSB
    // write path is wired in the run_subcommand integration layer (E-30 Task 9).
    let record = serde_json::json!({
        "action_type": "incident_record_created",
        "incident_id": id,
        "title": title,
        "incident_date": incident_date,
        "source_url": source_url,
    });
    println!("{}", serde_json::to_string_pretty(&record)?);
    Ok(())
}

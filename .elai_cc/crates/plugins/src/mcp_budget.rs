//! Per-tool token-budget enforcement for MCP `tools/list` responses.
//!
//! Uses a chars/4 heuristic for token estimation (no external tokenizer
//! dependency). When the estimated token cost exceeds `budget_tokens`,
//! drops tools by lowest priority first. Priority is determined by
//! explicit allowlist position in `elai_cc.toml [mcp.budgets]`; unlisted
//! tools default to the tail (lowest priority).
//!
//! Reference: mindstudio.ai/blog/claude-code-mcp-server-token-overhead
//! (55,000-token GitHub MCP overhead; 5 servers x 20 tools approx 80k tokens).

use tracing::debug;

/// Default per-server token budget for `tools/list` responses.
pub const DEFAULT_BUDGET_TOKENS: u32 = 5_000;

/// Default output budget for a single tool result (chars/4 heuristic).
pub const DEFAULT_OUTPUT_BUDGET_TOKENS: u32 = 8_000;

/// Enforce a deterministic character-based budget on one tool result.
pub fn enforce_output_budget(output: &str, budget_tokens: u32) -> (String, u32) {
    let estimated = estimate_tokens(output);
    if estimated <= budget_tokens {
        return (output.to_string(), estimated);
    }
    let limit = (budget_tokens as usize).saturating_mul(4);
    let notice = "\n[tool output truncated to token budget]";
    let keep = limit.saturating_sub(notice.len());
    let mut end = keep.min(output.len());
    while end > 0 && !output.is_char_boundary(end) { end -= 1; }
    let mut compacted = output[..end].to_string();
    compacted.push_str(notice);
    let kept_tokens = estimate_tokens(&compacted);
    (compacted, kept_tokens)
}


/// A single tool entry with its priority rank.
#[derive(Debug, Clone)]
pub struct BudgetedTool {
    pub name: String,
    pub schema_json: String,
    /// Lower number = higher priority. Tools not in the explicit allowlist
    /// get `u32::MAX`.
    pub priority: u32,
}

/// Report returned by [`enforce`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnforceReport {
    pub dropped: Vec<String>,
    pub kept_tokens: u32,
}

/// Token-budget configuration from `elai_cc.toml [mcp.budgets]`.
#[derive(Debug, Clone)]
pub struct TokenBudget {
    /// Maximum estimated tokens for the tools list.
    pub budget_tokens: u32,
    /// Explicit priority order: index 0 = highest priority.
    pub priority_order: Vec<String>,
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self {
            budget_tokens: DEFAULT_BUDGET_TOKENS,
            priority_order: vec![],
        }
    }
}

/// Estimate token count using the chars/4 heuristic.
pub fn estimate_tokens(s: &str) -> u32 {
    ((s.chars().count() as f64) / 4.0).ceil() as u32
}

/// Enforce the token budget on a list of tools.
///
/// Tools are sorted by priority (lowest `priority` number first). When the
/// cumulative estimated token count exceeds `budget.budget_tokens`, remaining
/// tools are dropped.
///
/// Returns an [`EnforceReport`] with the names of dropped tools and the
/// total token estimate of kept tools.
pub fn enforce(tools: &mut Vec<BudgetedTool>, budget: &TokenBudget) -> EnforceReport {
    // Assign priorities from the configured order.
    for tool in tools.iter_mut() {
        let rank = budget
            .priority_order
            .iter()
            .position(|n| n == &tool.name)
            .map(|i| i as u32)
            .unwrap_or(u32::MAX);
        tool.priority = rank;
    }

    // Sort by priority ascending (high-priority first).
    tools.sort_by_key(|t| t.priority);

    let mut kept_tokens: u32 = 0;
    let mut dropped = Vec::new();
    let mut kept = Vec::new();

    for tool in tools.drain(..) {
        let cost = estimate_tokens(&tool.schema_json);
        if kept_tokens.saturating_add(cost) <= budget.budget_tokens {
            kept_tokens += cost;
            kept.push(tool);
        } else {
            debug!(tool = %tool.name, cost, kept_tokens, budget = budget.budget_tokens, "mcp budget: dropping tool");
            dropped.push(tool.name.clone());
        }
    }

    *tools = kept;
    EnforceReport { dropped, kept_tokens }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tool(name: &str, schema_size_chars: usize) -> BudgetedTool {
        BudgetedTool {
            name: name.to_string(),
            schema_json: "x".repeat(schema_size_chars),
            priority: u32::MAX,
        }
    }

    #[test]
    fn estimate_tokens_rounds_up() {
        assert_eq!(estimate_tokens("abcd"), 1);
        assert_eq!(estimate_tokens("abcde"), 2);
        assert_eq!(estimate_tokens(""), 0);
    }

    #[test]
    fn no_drop_when_under_budget() {
        let mut tools = vec![make_tool("a", 100), make_tool("b", 100)];
        let budget = TokenBudget { budget_tokens: 1000, priority_order: vec![] };
        let report = enforce(&mut tools, &budget);
        assert!(report.dropped.is_empty());
        assert_eq!(tools.len(), 2);
    }

    #[test]
    fn drops_lowest_priority_tail() {
        let mut tools: Vec<BudgetedTool> = (0..20)
            .map(|i| make_tool(&format!("tool_{i:02}"), 1000))
            .collect();
        let budget = TokenBudget {
            budget_tokens: 2000,
            priority_order: (0..20).map(|i| format!("tool_{i:02}")).collect(),
        };
        let report = enforce(&mut tools, &budget);
        // Each tool costs 250 tokens (1000 chars / 4). Budget 2000 -> keep 8.
        assert_eq!(tools.len(), 8);
        assert_eq!(report.dropped.len(), 12);
        // Dropped tools should be the tail (lowest priority = highest index).
        for dropped_name in &report.dropped {
            let idx: u32 = dropped_name.trim_start_matches("tool_").parse().unwrap();
            assert!(idx >= 8, "only tail tools should be dropped, got {dropped_name}");
        }
    }

    #[test]
    fn priority_order_respected() {
        let mut tools = vec![
            make_tool("low_prio", 800),
            make_tool("high_prio", 800),
        ];
        let budget = TokenBudget {
            budget_tokens: 200 + 10, // room for high_prio only (200 tokens) + tiny buffer
            priority_order: vec!["high_prio".to_string()],
        };
        let report = enforce(&mut tools, &budget);
        assert!(tools.iter().any(|t| t.name == "high_prio"));
        assert!(report.dropped.contains(&"low_prio".to_string()));
    }
}

// J-12 H4: post-execution result annotators. Pure, LLM-free.
// Appends deterministic note lines to a SUCCESSFUL tool result string.
// Distinct from C-10 stagnation (trajectory-level): H4 annotates the
// individual tool result bytes the agent sees. Never raises, never
// blocks, never mutates failure semantics (caller invokes on SUCCESS
// path only). Empty registry = identity (byte-identical when same str).

/// One post-exec annotation candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResultAnnotator {
    pub id: String,
    /// None = apply to ALL tools; Some(name) = that tool only (exact match).
    pub tool_name: Option<String>,
    /// Static annotation body for this substrate slice. Dynamic/domain
    /// annotators (DB inspection) land with rules-load residual.
    pub text: String,
}

/// Append matching annotations to a successful tool result.
/// - Empty `annotators` => return `result` unchanged.
/// - Match: annotator.tool_name is None OR equals `tool_name`.
/// - Order: sort matching annotators by `id` ascending (determinism).
/// - Append format per match: `[j12-annotate:{id}] {text}`
/// - Join: if any matched, return `format!("{result}\n\n{}", parts.join("\n"))`
/// - Never panics; never returns Err; never mutates failure semantics.
pub fn annotate_tool_result(
    tool_name: &str,
    result: &str,
    annotators: &[ResultAnnotator],
) -> String {
    if annotators.is_empty() {
        return result.to_string();
    }
    let mut matched: Vec<&ResultAnnotator> = annotators
        .iter()
        .filter(|a| match &a.tool_name {
            None => true,
            Some(name) => name == tool_name,
        })
        .collect();
    if matched.is_empty() {
        return result.to_string();
    }
    matched.sort_by(|a, b| a.id.cmp(&b.id));
    let parts: Vec<String> = matched
        .iter()
        .map(|a| format!("[j12-annotate:{}] {}", a.id, a.text))
        .collect();
    format!("{result}\n\n{}", parts.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ann(id: &str, tool_name: Option<&str>, text: &str) -> ResultAnnotator {
        ResultAnnotator {
            id: id.to_string(),
            tool_name: tool_name.map(|s| s.to_string()),
            text: text.to_string(),
        }
    }

    #[test]
    fn empty_annotators_identity() {
        let out = annotate_tool_result("read_file", "hello", &[]);
        assert_eq!(out, "hello");
    }

    #[test]
    fn one_all_tools_annotator_appends() {
        let out = annotate_tool_result("read_file", "hello", &[ann("a1", None, "be safe")]);
        assert_eq!(out, "hello\n\n[j12-annotate:a1] be safe");
    }

    #[test]
    fn tool_name_filter_only_matching() {
        let out_a = annotate_tool_result("read_file", "hello", &[ann("a1", Some("read_file"), "match")]);
        assert_eq!(out_a, "hello\n\n[j12-annotate:a1] match");
        let out_b = annotate_tool_result("write_file", "hello", &[ann("a1", Some("read_file"), "match")]);
        assert_eq!(out_b, "hello");
    }

    #[test]
    fn two_annotators_sort_by_id() {
        let out = annotate_tool_result(
            "read_file",
            "hello",
            &[ann("z1", None, "second"), ann("a1", None, "first")],
        );
        assert_eq!(out, "hello\n\n[j12-annotate:a1] first\n[j12-annotate:z1] second");
    }

    #[test]
    fn <REDACTED_TOKEN>() {
        let out = annotate_tool_result(
            "read_file",
            "hello",
            &[ann("a1", Some("write_file"), "nope"), ann("b2", Some("edit_file"), "nope2")],
        );
        assert_eq!(out, "hello");
    }
}

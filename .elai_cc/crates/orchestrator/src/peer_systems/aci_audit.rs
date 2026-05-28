//! ACI (Agent-Computer Interface) tool-shape audit emitter.
//!
//! Reference: SWE-agent (arXiv:2405.15793, NeurIPS 2024) §4 ACI
//! Principles. The SWE-agent paper finds that LM agents perform
//! materially better when the tool surface is shaped FOR the LM
//! (terse output, structured fields, sane scope defaults) rather
//! than copy-pasted from human-facing CLI tools.
//!
//! This module is a PURE EMITTER -- it takes a list of tool
//! descriptors with three axis scores (terse, structured, scoped) and
//! produces a Markdown artifact suitable for `workflow/audits/`. It
//! does NOT register checks with [`crate::deterministic_auditor`] --
//! per D-B-7 in `workflow/reasoning/E-21-decisions.md` the ACI audit
//! is a ONE-SHOT prose artifact, not a recurring deterministic check.
//!
//! Plan: `docs/thefinalglance/E-plan/E-21_peer_systems_llm_as_worker.md`
//! task 4.

/// Three-letter grade for a single ACI axis on one tool.
///
/// Mapping (per SWE-agent ACI principles):
///   * `A` -- exemplary: LM-shaped, structured, narrow default scope.
///   * `B` -- acceptable: minor LM-shaping gaps.
///   * `C` -- needs attention: human-facing verbosity, regex parsing
///            required, or wide-default scope that wastes context.
///   * `D` -- failing: actively confuses the LM (interactive prompts,
///            unbounded output, prose-only error messages).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AciGrade {
    /// Exemplary -- LM-shaped output / structured / narrow default.
    A,
    /// Acceptable -- minor LM-shaping gaps.
    B,
    /// Needs attention -- human-facing verbosity / regex / wide default.
    C,
    /// Failing -- actively confuses the LM / interactive / unbounded.
    D,
}

impl AciGrade {
    /// Single-letter wire label used in the rendered Markdown table.
    #[must_use]
    pub fn as_letter(&self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
        }
    }

    /// 0..3 weight so callers can summarize across tools (A=3, B=2,
    /// C=1, D=0). Higher = better; used in the Markdown summary table.
    #[must_use]
    pub fn weight(&self) -> u8 {
        match self {
            Self::A => 3,
            Self::B => 2,
            Self::C => 1,
            Self::D => 0,
        }
    }
}

/// One tool's three-axis ACI shape grade.
///
/// Fields map to the SWE-agent ACI principles:
///   * `lm_terse` -- (a) output is LM-terse, not human-verbose.
///   * `structured_emit` -- (b) tool emits structured fields (JSON /
///     TSV / fixed schema), not free-form prose that requires regex.
///   * `sane_scope_default` -- (c) tool defaults to a narrow scope
///     (glob narrowed, head_limit set, depth bounded), not "return
///     everything".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolShapeDescriptor {
    /// Tool identifier (e.g. `"Read"`, `"Grep"`, `"WebFetch"`).
    pub tool_name: String,
    /// Grade for axis (a) LM-terse output.
    pub lm_terse: AciGrade,
    /// Grade for axis (b) structured emission.
    pub structured_emit: AciGrade,
    /// Grade for axis (c) sane default scope.
    pub sane_scope_default: AciGrade,
    /// Short prose explanation (one or two sentences) for the audit
    /// reader. Used for the "Notes" column in the Markdown render.
    pub notes: String,
}

impl ToolShapeDescriptor {
    /// Composite weight (sum of three axis weights, 0..9).
    #[must_use]
    pub fn composite_weight(&self) -> u8 {
        self.lm_terse.weight()
            + self.structured_emit.weight()
            + self.sane_scope_default.weight()
    }
}

/// Sanitize a string for embedding in a Markdown table cell or list
/// item. Replaces '|' with '/' (pipe splits the cell) and newline /
/// carriage-return with a single space (literal newline breaks the
/// table row or bullet structure). Idempotent; pure-compute.
fn escape_md_cell(raw: &str) -> String {
    raw.replace('|', "/").replace(['\n', '\r'], " ")
}

/// Render the audit artifact as Markdown text.
///
/// Output structure (matches `workflow/audits/aci-tool-shape-*.md`):
///   1. Header (title, plan reference, paper cite).
///   2. Three-column grade table (tool / lm_terse / structured /
///      scoped / composite / notes).
///   3. Summary section listing grade-distribution counts and the
///      lowest-weighted tools as a "fixer queue".
///
/// The output is plain text and contains no terminal escapes; it is
/// suitable for both file output and stdout dump (e.g. an `elai
/// audit aci` CLI subcommand in a Tier-C follow-on).
#[must_use]
pub fn render_audit_markdown(
    audit_date: &str,
    tools: &[ToolShapeDescriptor],
) -> String {
    let mut out = String::new();
    out.push_str("# ACI Tool-Shape Audit (E-21 task 4)\n\n");
    out.push_str(&format!("Audit date: {}\n", audit_date.trim()));
    out.push_str("Plan: docs/thefinalglance/E-plan/E-21_peer_systems_llm_as_worker.md\n");
    out.push_str("Reference: arXiv:2405.15793 (SWE-agent, NeurIPS 2024) §4 ACI Principles\n\n");

    out.push_str("## Grades\n\n");
    out.push_str("| Tool | LM-terse (a) | Structured (b) | Scoped default (c) | Composite | Notes |\n");
    out.push_str("|---|---|---|---|---|---|\n");

    let mut sorted: Vec<&ToolShapeDescriptor> = tools.iter().collect();
    sorted.sort_by(|a, b| a.tool_name.cmp(&b.tool_name));

    for t in &sorted {
        let composite = t.composite_weight();
        // Escape pipe AND newline in BOTH tool_name and notes so the
        // Markdown table stays well-formed. A literal '|' splits the
        // table cell; a literal '\n' / '\r' breaks the row entirely.
        // tool_name was previously un-escaped -- a tool registered as
        // "Read|Bad" would inject a phantom column. Same risk applied
        // to the Fixer-queue section below.
        let safe_name = escape_md_cell(&t.tool_name);
        let safe_notes = escape_md_cell(&t.notes);
        out.push_str(&format!(
            "| {} | {} | {} | {} | {}/9 | {} |\n",
            safe_name,
            t.lm_terse.as_letter(),
            t.structured_emit.as_letter(),
            t.sane_scope_default.as_letter(),
            composite,
            safe_notes,
        ));
    }

    out.push_str("\n## Distribution\n\n");
    let mut grade_counts: [u32; 4] = [0; 4]; // A, B, C, D
    for t in tools {
        for axis in [t.lm_terse, t.structured_emit, t.sane_scope_default] {
            grade_counts[match axis {
                AciGrade::A => 0,
                AciGrade::B => 1,
                AciGrade::C => 2,
                AciGrade::D => 3,
            }] += 1;
        }
    }
    out.push_str(&format!("- A: {} cells\n", grade_counts[0]));
    out.push_str(&format!("- B: {} cells\n", grade_counts[1]));
    out.push_str(&format!("- C: {} cells\n", grade_counts[2]));
    out.push_str(&format!("- D: {} cells\n", grade_counts[3]));

    out.push_str("\n## Fixer queue (lowest composite first)\n\n");
    let mut by_weight: Vec<&ToolShapeDescriptor> = tools.iter().collect();
    by_weight.sort_by(|a, b| {
        a.composite_weight()
            .cmp(&b.composite_weight())
            .then_with(|| a.tool_name.cmp(&b.tool_name))
    });
    for t in by_weight.iter().take(5) {
        // Same escape rule as the grade-table rows above. List items
        // are less sensitive to '|' than table rows but a literal '\n'
        // still corrupts the bullet structure, and consistency keeps
        // the emitter trivially auditable.
        out.push_str(&format!(
            "- {} ({}/9): {}\n",
            escape_md_cell(&t.tool_name),
            t.composite_weight(),
            escape_md_cell(&t.notes),
        ));
    }
    if by_weight.len() > 5 {
        out.push_str(&format!(
            "- ... ({} more tools omitted; see grade table above)\n",
            by_weight.len() - 5
        ));
    }

    out.push_str("\n## Follow-ups\n\n");
    out.push_str("- Per D-B-7 (workflow/reasoning/E-21-decisions.md) this artifact is one-shot prose.\n");
    out.push_str("- Tools rated C or D on any axis are candidates for a follow-up E-N or B-N refactor.\n");
    out.push_str("- The audit covers the registered tool surface as of the audit date; tools added later are NOT graded here.\n");

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn desc(name: &str, a: AciGrade, b: AciGrade, c: AciGrade, notes: &str) -> ToolShapeDescriptor {
        ToolShapeDescriptor {
            tool_name: name.to_string(),
            lm_terse: a,
            structured_emit: b,
            sane_scope_default: c,
            notes: notes.to_string(),
        }
    }

    #[test]
    fn grade_letter_and_weight() {
        assert_eq!(AciGrade::A.as_letter(), "A");
        assert_eq!(AciGrade::D.as_letter(), "D");
        assert_eq!(AciGrade::A.weight(), 3);
        assert_eq!(AciGrade::B.weight(), 2);
        assert_eq!(AciGrade::C.weight(), 1);
        assert_eq!(AciGrade::D.weight(), 0);
    }

    #[test]
    fn composite_weight_sums_three_axes() {
        let t = desc("Read", AciGrade::A, AciGrade::B, AciGrade::C, "n/a");
        assert_eq!(t.composite_weight(), 3 + 2 + 1);
    }

    #[test]
    fn render_markdown_includes_all_tools() {
        let tools = vec![
            desc("Read", AciGrade::A, AciGrade::A, AciGrade::B, "limit+offset are sane"),
            desc("Grep", AciGrade::B, AciGrade::A, AciGrade::A, "head_limit default 250"),
            desc("WebFetch", AciGrade::B, AciGrade::C, AciGrade::C, "unbounded body, prose"),
        ];
        let md = render_audit_markdown("2026-05-28", &tools);
        assert!(md.contains("# ACI Tool-Shape Audit"));
        assert!(md.contains("arXiv:2405.15793"));
        assert!(md.contains("| Read |"));
        assert!(md.contains("| Grep |"));
        assert!(md.contains("| WebFetch |"));
        assert!(md.contains("## Distribution"));
        assert!(md.contains("## Fixer queue"));
    }

    #[test]
    fn render_sorts_tools_alphabetically() {
        let tools = vec![
            desc("Zebra", AciGrade::A, AciGrade::A, AciGrade::A, "z"),
            desc("Alpha", AciGrade::B, AciGrade::B, AciGrade::B, "a"),
            desc("Mango", AciGrade::C, AciGrade::C, AciGrade::C, "m"),
        ];
        let md = render_audit_markdown("2026-05-28", &tools);
        let alpha_pos = md.find("| Alpha |").expect("Alpha row");
        let mango_pos = md.find("| Mango |").expect("Mango row");
        let zebra_pos = md.find("| Zebra |").expect("Zebra row");
        assert!(alpha_pos < mango_pos);
        assert!(mango_pos < zebra_pos);
    }

    #[test]
    fn fixer_queue_orders_by_weight_ascending() {
        let tools = vec![
            desc("Good", AciGrade::A, AciGrade::A, AciGrade::A, "9/9"),
            desc("Mid", AciGrade::B, AciGrade::B, AciGrade::B, "6/9"),
            desc("Bad", AciGrade::D, AciGrade::D, AciGrade::D, "0/9"),
        ];
        let md = render_audit_markdown("2026-05-28", &tools);
        // Bad must appear in the fixer-queue section before Mid + Good.
        let queue_start = md.find("## Fixer queue").expect("queue");
        let after_queue = &md[queue_start..];
        let bad_pos = after_queue.find("- Bad").expect("bad in queue");
        let mid_pos = after_queue.find("- Mid").expect("mid in queue");
        let good_pos = after_queue.find("- Good").expect("good in queue");
        assert!(bad_pos < mid_pos);
        assert!(mid_pos < good_pos);
    }

    #[test]
    fn notes_pipe_chars_are_escaped() {
        // A `|` in the notes would break the Markdown table; verify the
        // emitter substitutes it.
        let tools = vec![desc(
            "Bad",
            AciGrade::A,
            AciGrade::A,
            AciGrade::A,
            "uses | as separator",
        )];
        let md = render_audit_markdown("2026-05-28", &tools);
        assert!(md.contains("uses / as separator"));
        assert!(!md.contains("uses | as separator"));
    }

    #[test]
    fn tool_name_pipe_chars_are_escaped() {
        // Regression: tool_name was previously un-escaped so a tool
        // registered as "Read|Bad" would inject a phantom table column.
        let tools = vec![desc(
            "Read|Bad",
            AciGrade::A,
            AciGrade::A,
            AciGrade::A,
            "phantom column risk",
        )];
        let md = render_audit_markdown("2026-05-28", &tools);
        assert!(md.contains("Read/Bad"));
        // The raw, un-escaped form must not appear in the rendered table
        // row (it would still appear if it were a substring of any other
        // rendered text, but no other content prints that exact token).
        assert!(!md.contains("| Read|Bad |"));
    }

    #[test]
    fn newline_in_notes_is_collapsed_to_space() {
        // A literal newline in notes would break the Markdown row.
        let tools = vec![desc(
            "Multi",
            AciGrade::B,
            AciGrade::B,
            AciGrade::B,
            "line one\nline two",
        )];
        let md = render_audit_markdown("2026-05-28", &tools);
        assert!(md.contains("line one line two"));
        assert!(!md.contains("line one\nline two"));
    }

    #[test]
    fn empty_input_renders_header_only() {
        let md = render_audit_markdown("2026-05-28", &[]);
        assert!(md.contains("# ACI Tool-Shape Audit"));
        assert!(md.contains("## Grades"));
        assert!(md.contains("## Distribution"));
        // Distribution counters all zero.
        assert!(md.contains("- A: 0 cells"));
        assert!(md.contains("- D: 0 cells"));
    }
}

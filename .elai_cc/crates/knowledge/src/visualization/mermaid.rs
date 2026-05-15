// B-33 — Mermaid flowchart generator.
//
// Algorithm derived from SocratiCode `graph-analysis.ts:113-210`
// (visualization step), with two deliberate ELAI improvements over the
// reference (see workflow/reasoning/B-33-grounding.md):
//
//   1. ELAI uses Mermaid `classDef` per language instead of inline
//      `style id fill:#...`. classDef scales O(languages) instead of
//      O(nodes), and Mermaid 11.x supports it cleanly.
//   2. The 250-node cap claimed in the spec is unfounded in the source.
//      We let the caller pass `max_nodes`; truncation is recorded in
//      a footer comment when triggered.
//
// Output contract (locked by tests/mermaid_generation.rs +
// contracts/tests/b33_graph_viewer_contract.rs):
//   - Always starts with `flowchart LR\n` (even on empty input).
//   - Always emits the classDef block (rust/python/ts/svelte/go/jvm/csharp/other + cyclic).
//   - Empty input -> header + classDef + `%% empty graph` comment (stub_ban).
//   - IDs sanitised: `[^A-Za-z0-9_]` -> `_`, leading-digit prefixed `n_`.
//   - Cyclic edges use `-.->|cycle|` and the `:::cyclic` class for red color.
//   - Iteration order is BTreeMap-deterministic (no HashMap, no nondet sort).
//   - No clock dep — caller passes `VizProject.built_at`; we don't read it
//     for the diagram body, only for the trailing `%% built_at` comment.

use std::fmt::Write;

use super::types::{VizData, VizFileEdge};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Render the visualization payload as a Mermaid `flowchart LR` diagram.
///
/// `max_nodes` caps the rendered node count. When `data.files.len()` exceeds
/// the cap a `%% truncated: showing N of M` comment is appended.
///
/// The function never panics and never returns an empty string — callers
/// can blindly pipe the output to a file or Mermaid renderer.
pub fn generate_mermaid(data: &VizData, max_nodes: usize) -> String {
    let mut out = String::with_capacity(256);
    // Header.
    out.push_str("flowchart LR\n");

    // classDef block (always, even on empty input — keeps the legend consistent).
    push_classdef_block(&mut out);

    // Empty payload short-circuit (still valid Mermaid).
    if data.files.is_empty() {
        out.push_str("    %% empty graph\n");
        return out;
    }

    // Truncation: take the first `max_nodes` files in BTreeMap order
    // (Vec is already sorted by the producer; we re-sort defensively for
    // determinism in tests that hand-build VizData).
    let mut sorted_files: Vec<&str> = data.files.iter().map(|f| f.id.as_str()).collect();
    sorted_files.sort_unstable();

    let truncated = sorted_files.len() > max_nodes;
    let kept_count = sorted_files.len().min(max_nodes);
    let kept: std::collections::BTreeSet<&str> =
        sorted_files.iter().take(kept_count).copied().collect();

    // Node lines.
    for file in &data.files {
        if !kept.contains(file.id.as_str()) {
            continue;
        }
        let safe_id = sanitize_id(&file.id);
        let label = mermaid_escape_label(&file.label);
        let lang_class = language_class(&file.language);
        // n_id["label"]:::lang
        let _ = writeln!(out, "    {safe_id}[\"{label}\"]:::{lang_class}");
    }

    // Edge lines (only emit when both endpoints survived truncation).
    let mut sorted_edges: Vec<&VizFileEdge> = data.file_edges.iter().collect();
    sorted_edges.sort_by(|a, b| (&a.source, &a.target).cmp(&(&b.source, &b.target)));
    for edge in sorted_edges {
        if !kept.contains(edge.source.as_str()) || !kept.contains(edge.target.as_str()) {
            continue;
        }
        let src = sanitize_id(&edge.source);
        let tgt = sanitize_id(&edge.target);
        if edge.cyclic {
            // Mermaid red dotted edge with explicit cyclic class on the source so
            // the connector itself picks up the styling. (classDef cycles only
            // applies to nodes; the `-.->|cycle|` arrow style is the visual cue.)
            let _ = writeln!(out, "    {src} -.->|cycle| {tgt}");
            // Also tag both endpoints with :::cyclic so the canvas surfaces them.
            let _ = writeln!(out, "    {src}:::cyclic");
            let _ = writeln!(out, "    {tgt}:::cyclic");
        } else {
            let _ = writeln!(out, "    {src} --> {tgt}");
        }
    }

    if truncated {
        let _ = writeln!(
            out,
            "    %% truncated: showing {kept} of {total}",
            kept = kept_count,
            total = sorted_files.len()
        );
    }

    if let Some(built_at) = data.project.built_at {
        let _ = writeln!(out, "    %% built_at: {built_at}");
    }

    out
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Emit the language colour palette and the `cyclic` class. Stable order so
/// tests can assert presence; no I/O, no allocation per node.
fn push_classdef_block(out: &mut String) {
    // Order matches the language_class() switch below.
    out.push_str("    classDef rust    fill:#dea584,color:#2e2c29,stroke:#a36b39\n");
    out.push_str("    classDef python  fill:#3572A5,color:#ffffff,stroke:#1d4f7b\n");
    out.push_str("    classDef ts      fill:#3178c6,color:#ffffff,stroke:#1d4f7b\n");
    out.push_str("    classDef svelte  fill:#ff3e00,color:#ffffff,stroke:#a82500\n");
    out.push_str("    classDef go      fill:#00add8,color:#ffffff,stroke:#006b87\n");
    out.push_str("    classDef jvm     fill:#b07219,color:#ffffff,stroke:#704711\n");
    out.push_str("    classDef csharp  fill:#178600,color:#ffffff,stroke:#0d4f00\n");
    out.push_str("    classDef other   fill:#888888,color:#ffffff,stroke:#555555\n");
    out.push_str("    classDef cyclic  fill:#ff5252,color:#ffffff,stroke:#b71c1c\n");
}

/// Map a language id to a classDef name. Unknown languages collapse to
/// `other` so the diagram never references a missing class.
fn language_class(lang: &str) -> &'static str {
    let lower = lang.to_ascii_lowercase();
    match lower.as_str() {
        "rust" | "rs" => "rust",
        "python" | "py" => "python",
        "typescript" | "ts" | "tsx" | "javascript" | "js" => "ts",
        "svelte" => "svelte",
        "go" => "go",
        "java" | "kotlin" | "scala" | "jvm" => "jvm",
        "csharp" | "cs" | "c#" => "csharp",
        _ => "other",
    }
}

/// Sanitise an arbitrary id into a Mermaid-safe identifier.
///
/// Replaces every non `[A-Za-z0-9_]` byte with `_` and prefixes `n_`
/// when the first remaining char is a digit (Mermaid identifiers must
/// start with a letter or underscore).
fn sanitize_id(input: &str) -> String {
    let mut buf = String::with_capacity(input.len() + 2);
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            buf.push(ch);
        } else {
            buf.push('_');
        }
    }
    if buf.is_empty() {
        buf.push_str("n_empty");
        return buf;
    }
    let first = buf.as_bytes()[0];
    if first.is_ascii_digit() {
        let mut prefixed = String::with_capacity(buf.len() + 2);
        prefixed.push_str("n_");
        prefixed.push_str(&buf);
        return prefixed;
    }
    buf
}

/// Escape characters that would break a Mermaid label in `["..."]`.
fn mermaid_escape_label(input: &str) -> String {
    // Quotes terminate the label; backslashes are passed through but we
    // double-escape so user labels can't smuggle a closing quote.
    let mut buf = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => buf.push_str("\\\""),
            '\n' | '\r' => buf.push(' '),
            other => buf.push(other),
        }
    }
    buf
}

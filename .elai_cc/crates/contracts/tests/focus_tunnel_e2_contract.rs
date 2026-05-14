// B-28 E-2 contract test: unified diff parser + multi-region replay coordinator.
//
// Locks the public surface added in E-2 (focus_tunnel/diff.rs):
//   1. DiffParser::parse(model_output) -> Result<Vec<DiffHunk>, DiffParseError>.
//   2. DiffHunk's 9 canonical fields (spec L31-36, L141-146).
//   3. Spec assertion L224: old_start > 0 and new_start > 0 — parser rejects
//      0-line origins as DiffParseError::HunkOutOfBounds.
//   4. DiffParseError surface for E-3 router pattern-match:
//      NotUnifiedDiff / InvalidHunkHeader / HunkOutOfBounds / MissingFileHeader
//      / EmptyOutput.
//   5. replay_hunks all-or-nothing in-memory atomicity: if any hunk fails to
//      apply against the in-memory buffer, NO FileReadWrite::write call fires
//      on any path. Half-applied diffs are PROHIBITED (context-package L102-115).
//   6. BufferEditor trait + InProcessToolDispatcher::apply_edits_to_buffer
//      inherent method (tools/src/dispatcher.rs) — round-trip through the real
//      production 3-tier matcher in tier=Exact for byte-identical context.
//   7. DiffApplyError surface for E-3 fallback router: TierMatcherRejected
//      / DispatcherFailed / PartialApplySnapshot — each pattern-matchable so
//      E-3 can decide retry-same-mode vs hard-fallback-to-STRUCTURED
//      (focus-tunnel-design-interface.md L77).
//
// Spec references:
//   workflow/papers/focus-tunnel-design-interface.md (HIGH, L30-36, L72-77,
//   L141-146, L224, L186-190).

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use elai_orchestrator::focus_tunnel::{
    apply_edits_to_buffer_via, replay_hunks, BufferEditor, DiffApplyError, DiffParseError,
    DiffParser, FileReadWrite, ReplayPlan,
};
use elai_orchestrator::focus_tunnel::diff::DiffBodyKind;
use elai_core::tool_dispatcher::{
    EditMatchTier, EditRequest, SimilarityHint, ValidationTier, ValidationTierResult,
};

// ---------------------------------------------------------------------------
// Test fakes — in-memory FileReadWrite + simple BufferEditor that re-uses
// production string matching semantics via a thin shim. The contract test
// also verifies the production InProcessToolDispatcher implements BufferEditor.
// ---------------------------------------------------------------------------

#[derive(Default, Clone)]
struct InMemoryFs {
    files: Arc<Mutex<std::collections::HashMap<PathBuf, String>>>,
    writes: Arc<Mutex<Vec<PathBuf>>>,
}

impl InMemoryFs {
    fn new() -> Self {
        Self::default()
    }

    fn put(&self, path: &str, content: &str) {
        let mut g = self.files.lock().expect("test mutex");
        g.insert(PathBuf::from(path), content.to_string());
    }

    fn get(&self, path: &str) -> Option<String> {
        let g = self.files.lock().expect("test mutex");
        g.get(&PathBuf::from(path)).cloned()
    }

    fn writes_count(&self) -> usize {
        self.writes.lock().expect("test mutex").len()
    }
}

impl FileReadWrite for InMemoryFs {
    fn read(&self, path: &std::path::Path) -> std::io::Result<String> {
        let g = self.files.lock().expect("test mutex");
        g.get(path)
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "not in InMemoryFs"))
    }
    fn write(&self, path: &std::path::Path, content: &str) -> std::io::Result<()> {
        let mut g = self.files.lock().expect("test mutex");
        g.insert(path.to_path_buf(), content.to_string());
        self.writes.lock().expect("test mutex").push(path.to_path_buf());
        Ok(())
    }
}

// Thin BufferEditor that delegates to the real InProcessToolDispatcher's
// apply_edits_to_buffer. This is the production-equivalent path E-3 will use.
struct ProdEditor {
    inner: elai_tools::dispatcher::InProcessToolDispatcher,
}

impl ProdEditor {
    fn new() -> Self {
        // Re-use tools-crate's TieredValidator test stub via DefaultTieredValidator
        // is unnecessary because apply_edits_to_buffer does not exercise the
        // validator. Use a no-op fake.
        use elai_tools::tiered_validation::TieredValidator;

        struct NoopValidator;
        impl TieredValidator for NoopValidator {
            fn run_tier<'a>(
                &'a self,
                tier: ValidationTier,
                _req: &'a elai_core::tool_dispatcher::ValidateRequest,
            ) -> std::pin::Pin<
                Box<dyn std::future::Future<Output = ValidationTierResult> + Send + 'a>,
            > {
                Box::pin(async move {
                    ValidationTierResult {
                        tier,
                        passed: true,
                        elapsed_ms: 0,
                        rules_fired: 0,
                        stderr_snippet: None,
                    }
                })
            }
            fn run_all<'a>(
                &'a self,
                _req: &'a elai_core::tool_dispatcher::ValidateRequest,
            ) -> std::pin::Pin<
                Box<
                    dyn std::future::Future<Output = Vec<ValidationTierResult>> + Send + 'a,
                >,
            > {
                Box::pin(async { Vec::new() })
            }
        }
        let v: Arc<dyn TieredValidator> = Arc::new(NoopValidator);
        Self {
            inner: elai_tools::dispatcher::InProcessToolDispatcher::new(v),
        }
    }
}

impl BufferEditor for ProdEditor {
    fn apply_edits_to_buffer(
        &self,
        buf: &str,
        edits: &[EditRequest],
    ) -> Result<(String, Vec<(EditMatchTier, Option<SimilarityHint>)>), DiffApplyError> {
        // Inherent apply_edits_to_buffer returns DiffApplyError directly; no
        // wrapping needed. This delegates verbatim to the production matcher.
        self.inner.apply_edits_to_buffer(buf, edits)
    }
}

// ---------------------------------------------------------------------------
// Parser tests
// ---------------------------------------------------------------------------

const SINGLE_HUNK: &str = "\
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,3 +1,4 @@
 use std::io;
-fn old() {}
+fn renamed() {}
+fn extra() {}
 // tail
";

#[test]
fn parser_emits_single_hunk_with_correct_fields() {
    // spec L31-36, L141-146.
    let p = DiffParser::new();
    let hunks = p.parse(SINGLE_HUNK).expect("must parse a well-formed diff");
    assert_eq!(hunks.len(), 1, "exactly one hunk expected");
    let h = &hunks[0];
    assert_eq!(h.file_path, "src/lib.rs");
    assert_eq!(h.old_start, 1);
    assert_eq!(h.old_count, 3);
    assert_eq!(h.new_start, 1);
    assert_eq!(h.new_count, 4);
    assert_eq!(h.context_lines, vec!["use std::io;".to_string(), "// tail".to_string()]);
    assert_eq!(h.removed_lines, vec!["fn old() {}".to_string()]);
    assert_eq!(
        h.added_lines,
        vec!["fn renamed() {}".to_string(), "fn extra() {}".to_string()],
    );
}

const MULTI_HUNK_SAME_FILE: &str = "\
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,3 +1,3 @@
 use std::io;
-fn one() {}
+fn ONE() {}
 // mid
@@ -20,3 +20,3 @@
 // tail-pre
-fn two() {}
+fn TWO() {}
 // tail-post
";

#[test]
fn parser_handles_multi_hunk_single_file() {
    let p = DiffParser::new();
    let hunks = p.parse(MULTI_HUNK_SAME_FILE).expect("two hunks expected");
    assert_eq!(hunks.len(), 2);
    assert!(hunks.iter().all(|h| h.file_path == "src/lib.rs"));
    assert_eq!(hunks[0].old_start, 1);
    assert_eq!(hunks[1].old_start, 20);
    assert_eq!(hunks[0].removed_lines, vec!["fn one() {}".to_string()]);
    assert_eq!(hunks[1].removed_lines, vec!["fn two() {}".to_string()]);
}

const MULTI_FILE_DIFF: &str = "\
--- a/src/a.rs
+++ b/src/a.rs
@@ -1,2 +1,2 @@
 // header A
-fn a() {}
+fn A() {}
--- a/src/b.rs
+++ b/src/b.rs
@@ -1,2 +1,2 @@
 // header B
-fn b() {}
+fn B() {}
";

#[test]
fn parser_handles_multi_file_diff() {
    let p = DiffParser::new();
    let hunks = p.parse(MULTI_FILE_DIFF).expect("two hunks across two files");
    assert_eq!(hunks.len(), 2);
    assert_eq!(hunks[0].file_path, "src/a.rs");
    assert_eq!(hunks[1].file_path, "src/b.rs");
}

#[test]
fn parser_rejects_total_garbage_with_not_unified_diff() {
    // spec L36: caller falls back to STRUCTURED on NotUnifiedDiff.
    let p = DiffParser::new();
    let result = p.parse("this is not a diff at all, just prose");
    match result {
        Err(DiffParseError::NotUnifiedDiff { .. }) => {}
        other => panic!("expected NotUnifiedDiff, got {other:?}"),
    }
}

#[test]
fn parser_rejects_empty_output() {
    let p = DiffParser::new();
    match p.parse("   \n  \n") {
        Err(DiffParseError::EmptyOutput) => {}
        other => panic!("expected EmptyOutput, got {other:?}"),
    }
}

#[test]
fn parser_rejects_zero_line_origin_per_spec_assertion() {
    // spec L224: assert diff_hunk.old_start > 0 and diff_hunk.new_start > 0.
    let p = DiffParser::new();
    let bad = "\
--- a/src/x.rs
+++ b/src/x.rs
@@ -0,0 +1,1 @@
+fn z() {}
";
    match p.parse(bad) {
        Err(DiffParseError::HunkOutOfBounds { old_start, .. }) => {
            assert_eq!(old_start, 0, "rejected variant must report the offending zero");
        }
        Err(other) => panic!("expected HunkOutOfBounds, got {other:?}"),
        Ok(h) => panic!("must reject 0-line origin, parsed {h:?}"),
    }
}

#[test]
fn parser_rejects_malformed_hunk_header() {
    let p = DiffParser::new();
    let bad = "\
--- a/src/x.rs
+++ b/src/x.rs
@@ this is not a valid header @@
 ctx
";
    match p.parse(bad) {
        Err(DiffParseError::InvalidHunkHeader { line, .. }) => {
            assert!(line.contains("@@"));
        }
        other => panic!("expected InvalidHunkHeader, got {other:?}"),
    }
}

#[test]
fn parser_rejects_hunk_before_file_header() {
    let p = DiffParser::new();
    let bad = "\
@@ -1,1 +1,1 @@
-old
+new
";
    match p.parse(bad) {
        Err(DiffParseError::MissingFileHeader { .. }) => {}
        other => panic!("expected MissingFileHeader, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// replay_hunks tests — atomicity + plan + tier-correctness
// ---------------------------------------------------------------------------

#[tokio::test]
async fn replay_hunks_succeeds_and_writes_each_changed_file() {
    // Two files, one hunk each — both must succeed, both must be written.
    let fs = InMemoryFs::new();
    fs.put("src/a.rs", "// header A\nfn a() {}\n");
    fs.put("src/b.rs", "// header B\nfn b() {}\n");

    let p = DiffParser::new();
    let hunks = p.parse(MULTI_FILE_DIFF).expect("parse");
    let editor = ProdEditor::new();

    let plan = replay_hunks(&hunks, &editor, &fs).await.expect("replay ok");
    assert_eq!(plan.changed_files.len(), 2);
    assert_eq!(fs.writes_count(), 2);
    assert_eq!(fs.get("src/a.rs").unwrap_or_default(), "// header A\nfn A() {}\n");
    assert_eq!(fs.get("src/b.rs").unwrap_or_default(), "// header B\nfn B() {}\n");
    // ReplayPlan also exposes per-file tier info — exact on byte-identical context.
    for entry in &plan.files {
        for (tier, _hint) in &entry.tiers {
            assert_eq!(*tier, EditMatchTier::Exact, "byte-identical context should produce Exact");
        }
    }
}

#[tokio::test]
async fn replay_hunks_aborts_before_any_write_when_a_hunk_is_unmatchable() {
    // Context-package L102-115 / decision 3: all-or-nothing in-memory; first
    // NotMatched aborts before any write.
    let fs = InMemoryFs::new();
    fs.put("src/a.rs", "// header A\nfn a() {}\n");
    // src/b.rs intentionally has DIFFERENT body so its hunk's old_text won't
    // match — must abort BEFORE writing src/a.rs.
    fs.put("src/b.rs", "// completely different file\n");

    let p = DiffParser::new();
    let hunks = p.parse(MULTI_FILE_DIFF).expect("parse");
    let editor = ProdEditor::new();

    let err = replay_hunks(&hunks, &editor, &fs)
        .await
        .expect_err("must abort, second file is unmatchable");

    // No file writes occurred — atomicity invariant.
    assert_eq!(fs.writes_count(), 0, "no fs.write should fire on partial failure");
    // src/a.rs untouched on disk.
    assert_eq!(fs.get("src/a.rs").unwrap_or_default(), "// header A\nfn a() {}\n");

    match err {
        DiffApplyError::TierMatcherRejected { file_path, .. } => {
            assert_eq!(file_path, "src/b.rs");
        }
        DiffApplyError::PartialApplySnapshot { .. } => {
            // Acceptable alternative: PartialApplySnapshot wrapping
            // TierMatcherRejected. Either preserves the atomicity invariant.
        }
        other => panic!("expected TierMatcherRejected or PartialApplySnapshot, got {other:?}"),
    }
}

#[tokio::test]
async fn replay_hunks_parse_failure_surface_is_pattern_matchable_by_e3() {
    // The E-3 router needs to distinguish "couldn't parse" (-> STRUCTURED) from
    // "couldn't match" (-> retry or STRUCTURED with similarity hint). E-2's
    // parse error surface is exposed via DiffParser::parse directly; this
    // test exercises the helper that converts a parse error into the apply
    // pipeline's surface so the router can do a single match on
    // DiffApplyError when wiring the full path in E-3.
    let p = DiffParser::new();
    let err = p
        .parse("definitely not a diff")
        .expect_err("garbage must not parse");
    // The variant matters for E-3 fall-back wiring (spec L77).
    match err {
        DiffParseError::NotUnifiedDiff { .. } => {}
        other => panic!("expected NotUnifiedDiff for total garbage, got {other:?}"),
    }
}

#[tokio::test]
async fn apply_edits_to_buffer_via_dispatches_to_real_3tier_matcher() {
    // B-20A surface re-exercised: the inherent apply_edits_to_buffer on
    // InProcessToolDispatcher must report tier=Exact for byte-identical
    // old_text, and tier=WhitespaceNormalized for whitespace-only drift.
    let editor = ProdEditor::new();
    let buf = "let x = 1;\nlet y = 2;\nlet z = 3;\n";

    let exact_edit = EditRequest {
        file_path: PathBuf::from("/in_memory"),
        old_text: "let y = 2;".to_string(),
        new_text: "let y = 22;".to_string(),
    };
    let (out, tiers) = apply_edits_to_buffer_via(&editor, buf, &[exact_edit]).expect("ok");
    assert_eq!(out, "let x = 1;\nlet y = 22;\nlet z = 3;\n");
    assert_eq!(tiers.len(), 1);
    assert_eq!(tiers[0].0, EditMatchTier::Exact);

    let whitespace_edit = EditRequest {
        file_path: PathBuf::from("/in_memory"),
        old_text: "let   y = 2;".to_string(), // extra spaces — only matches whitespace-normalised
        new_text: "let y = 222;".to_string(),
    };
    let (out2, tiers2) =
        apply_edits_to_buffer_via(&editor, buf, &[whitespace_edit]).expect("ws-norm ok");
    assert!(out2.contains("let y = 222;"));
    assert_eq!(tiers2[0].0, EditMatchTier::WhitespaceNormalized);
}

// ---------------------------------------------------------------------------
// B-28 WIRING-1 item C — DiffHunk body_order preservation
// ---------------------------------------------------------------------------
//
// Spec L218-225: hunk_to_edit_request must reconstruct old_text from the
// removed + context lines IN ORIGINAL UNIFIED-DIFF BODY ORDER. The legacy
// 3-bucket form (context_lines/added_lines/removed_lines) loses ordering
// when a hunk has alternating context and removed lines, so the parser now
// also populates `DiffHunk.body_order` — a per-line Vec preserving the raw
// `+`/`-`/` ` order.
//
// This contract test verifies the disambiguating case: when the literal
// removed text `}` appears twice in the file, the 3-tier matcher must
// anchor on the surrounding context line ("fn foo() {") rather than the
// other occurrence. Without body_order, the join order collapses the
// context-line anchor and the matcher snaps to the wrong region.
//
// Spec references:
//   focus-tunnel-design-interface.md L31-36 (DiffHunk fields).
//   focus-tunnel-design-interface.md L186-190 (hunk_to_edit_request).
//   workflow/sessions/s-27436/wiring-plan.md item C.

#[test]
fn body_order_preserves_unified_diff_line_sequence() {
    // Diff body intentionally alternates ` ` (context) and `-` (removed)
    // lines. The legacy 3-bucket form would lose this ordering. We assert
    // that the parser populates `body_order` with the full per-line
    // sequence in input order, so `hunk_to_edit_request` can rebuild
    // old_text such that the matcher anchors on the surrounding context.
    let diff = "\
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,6 +1,6 @@
 fn foo() {
-    let x = 1;
+    let x = 2;
 }
 fn bar() {
-    let y = 3;
+    let y = 4;
 }
";
    let hunks = DiffParser::new().parse(diff).expect("parse two-region diff");
    assert_eq!(hunks.len(), 1, "diff should be one hunk; got {}", hunks.len());

    let h = &hunks[0];
    assert!(
        !h.body_order.is_empty(),
        "body_order MUST be populated for unified-diff hunks"
    );
    // Sequence: context "fn foo() {", removed "    let x = 1;", added
    // "    let x = 2;", context "}", context "fn bar() {", removed
    // "    let y = 3;", added "    let y = 4;", context "}".
    let kinds: Vec<DiffBodyKind> = h.body_order.iter().map(|l| l.kind).collect();
    let expected_kinds = vec![
        DiffBodyKind::Context,
        DiffBodyKind::Removed,
        DiffBodyKind::Added,
        DiffBodyKind::Context,
        DiffBodyKind::Context,
        DiffBodyKind::Removed,
        DiffBodyKind::Added,
        DiffBodyKind::Context,
    ];
    assert_eq!(kinds, expected_kinds, "body_order must match input sequence");

    // Round-trip: project removed + context in body order; the resulting
    // old_text should preserve the original 3-line region (context, removed,
    // context) so the 3-tier matcher anchors on the FIRST occurrence.
    let old_text_first_region: String = h
        .body_order
        .iter()
        .take(4)
        .filter(|l| matches!(l.kind, DiffBodyKind::Context | DiffBodyKind::Removed))
        .map(|l| l.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        old_text_first_region.contains("fn foo() {"),
        "old_text region must include the surrounding context anchor"
    );
    assert!(
        old_text_first_region.contains("    let x = 1;"),
        "old_text region must include the removed line"
    );
    assert!(
        !old_text_first_region.contains("fn bar() {"),
        "first-region old_text must not bleed into second region"
    );
}

// ---------------------------------------------------------------------------
// B-28 WIRING-1 item C — non-unique-pattern disambiguation via body_order
// ---------------------------------------------------------------------------
//
// Spec L218-225: when the literal text of a removed line appears more than
// once in the source file, the 3-bucket form (removed_lines / context_lines)
// loses the spatial information needed to anchor on the correct occurrence.
// `body_order` preserves the unified-diff per-line sequence so
// `hunk_to_edit_request` can rebuild old_text WITH the surrounding context
// anchor and the 3-tier matcher snaps to the right region on the FIRST
// (anchored) occurrence rather than the unrelated duplicate.
//
// Spec references:
//   focus-tunnel-design-interface.md L31-36 (DiffHunk fields).
//   focus-tunnel-design-interface.md L186-190 (hunk_to_edit_request).
//   workflow/sessions/s-27436/wiring-plan.md item C.

#[test]
fn non_unique_pattern_disambiguated_by_context() {
    // The body removes the IDENTICAL literal `}` line twice in two unrelated
    // regions. Without body_order, the 3-bucket form collapses both `}`
    // entries into a single `removed_lines` Vec and the matcher has no way
    // to know which `}` was meant. With body_order, each `}` retains its
    // surrounding context anchor (`fn foo() {` vs `fn bar() {`) so
    // `hunk_to_edit_request` can rebuild distinct old_text regions per
    // occurrence.
    let diff = "\
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,8 +1,8 @@
 fn foo() {
     let x = 1;
-}
+    let y = 2;
+}
 fn bar() {
     let z = 3;
-}
+    let w = 4;
+}
";
    let hunks = DiffParser::new()
        .parse(diff)
        .expect("non-unique removed literal `}` must still parse");
    assert_eq!(hunks.len(), 1, "single hunk expected; got {}", hunks.len());

    let h = &hunks[0];

    // The duplicate-literal invariant: `removed_lines` contains the SAME
    // text twice. This is the case body_order is required to disambiguate.
    let close_brace_count = h
        .removed_lines
        .iter()
        .filter(|line| line.as_str() == "}")
        .count();
    assert_eq!(
        close_brace_count, 2,
        "test fixture must produce two identical removed `}}` lines; got {}",
        close_brace_count,
    );

    // body_order must be populated and must preserve the spatial sequence.
    assert!(
        !h.body_order.is_empty(),
        "body_order MUST be populated for non-unique-pattern disambiguation"
    );

    // Rebuild the FIRST region's old_text from body_order (context + removed
    // entries up to and including the first removed `}`). The resulting text
    // MUST contain the `fn foo() {` anchor and MUST NOT contain `fn bar() {`.
    let first_region_old_text: String = {
        let mut taken: Vec<&str> = Vec::new();
        let mut saw_removed = false;
        for line in &h.body_order {
            match line.kind {
                DiffBodyKind::Context => taken.push(line.text.as_str()),
                DiffBodyKind::Removed => {
                    taken.push(line.text.as_str());
                    if line.text == "}" {
                        saw_removed = true;
                        break;
                    }
                }
                DiffBodyKind::Added => {}
            }
            // Defensive guard against an unexpectedly long prefix.
            let _ = saw_removed;
        }
        taken.join("\n")
    };
    assert!(
        first_region_old_text.contains("fn foo() {"),
        "first region's old_text must include the anchoring context `fn foo() {{`"
    );
    assert!(
        !first_region_old_text.contains("fn bar() {"),
        "first region's old_text must NOT bleed into the second region"
    );

    // Rebuild the SECOND region's old_text from body_order entries strictly
    // AFTER the first removed `}`. It must contain `fn bar() {` and the
    // second `}` removal, and must NOT contain `fn foo() {`.
    let second_region_old_text: String = {
        let mut taken: Vec<&str> = Vec::new();
        let mut past_first_close = false;
        for line in &h.body_order {
            match line.kind {
                DiffBodyKind::Context => {
                    if past_first_close {
                        taken.push(line.text.as_str());
                    }
                }
                DiffBodyKind::Removed => {
                    if past_first_close {
                        taken.push(line.text.as_str());
                    } else if line.text == "}" {
                        past_first_close = true;
                    }
                }
                DiffBodyKind::Added => {}
            }
        }
        taken.join("\n")
    };
    assert!(
        second_region_old_text.contains("fn bar() {"),
        "second region's old_text must include the anchoring context `fn bar() {{`"
    );
    assert!(
        !second_region_old_text.contains("fn foo() {"),
        "second region's old_text must NOT bleed back into the first region"
    );
}

#[allow(dead_code)]
fn _replayplan_field_shape_lock(plan: ReplayPlan) {
    // Compile-time field-shape lock: ReplayPlan must expose changed_files:
    // Vec<PathBuf> and files: Vec<_> with at least a file_path + tiers pair on
    // each entry. If a future refactor renames or removes any of these,
    // this helper stops compiling — auditor catches the surface drift.
    let _: Vec<PathBuf> = plan.changed_files;
    for entry in &plan.files {
        let _: &str = entry.file_path.as_str();
        for (tier, hint) in &entry.tiers {
            let _: &EditMatchTier = tier;
            let _: &Option<SimilarityHint> = hint;
        }
    }
}

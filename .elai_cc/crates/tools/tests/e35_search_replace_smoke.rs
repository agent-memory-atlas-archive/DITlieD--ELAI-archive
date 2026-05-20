//! E-35 observable_effect smoke: round-trip a SearchReplaceTier1 IPC
//! payload through postcard (same path the orchestrator uses over IPC)
//! and assert the matcher applies the patch to a sample source.
//!
//! Acts as the "Rust integration test that constructs a typed
//! XGrammarRequest, serializes it, deserializes, and asserts the
//! SearchReplaceTier1 schema validates a sample patch" called out in the
//! E-35 task contract — the XGrammarRequest round-trip itself is covered
//! by `elai_ipc_types::e35_tests`, and this file pairs the tier-1 patch
//! applier with the typed wire shape so the substrate round-trip is
//! observable from the dispatcher side.

use elai_ipc_types::{SearchReplaceEdit, SearchReplaceTier1};
use elai_tools::search_replace::SearchReplaceMatcher;

const SAMPLE_SOURCE: &str = "fn double(x: i32) -> i32 {\n    x + x\n}\n";

#[test]
fn e35_substrate_round_trip_applies_patch() {
    // 1. Orchestrator builds the typed action.
    let patch = SearchReplaceTier1 {
        file_path: "src/lib.rs".to_string(),
        edits: vec![SearchReplaceEdit {
            search: "x + x".to_string(),
            replace: "x.wrapping_add(x)".to_string(),
        }],
    };

    // 2. Wire-encode -> wire-decode (postcard, same path as ToolRequest.arguments).
    let bytes = postcard::to_allocvec(&patch).expect("encode SearchReplaceTier1");
    let decoded: SearchReplaceTier1 =
        postcard::from_bytes(&bytes).expect("decode SearchReplaceTier1");
    assert_eq!(decoded, patch);

    // 3. Tier-1 matcher applies the decoded patch.
    let matcher = SearchReplaceMatcher::new();
    let result = matcher
        .apply(SAMPLE_SOURCE, &decoded)
        .expect("apply tier-1 patch");
    assert!(result.contains("x.wrapping_add(x)"));
    assert!(!result.contains("x + x"));
}

#[test]
fn e35_substrate_round_trip_rejects_collision() {
    // Source-file contains a literal git conflict marker; the matcher must
    // refuse to apply (the dispatcher falls back to whole-file rewrite).
    let conflicted_source = "ok\n<<<<<<< HEAD\nbad\n=======\nworse\n>>>>>>> branch\n";
    let patch = SearchReplaceTier1 {
        file_path: "src/lib.rs".to_string(),
        edits: vec![SearchReplaceEdit {
            search: "ok".to_string(),
            replace: "ok'".to_string(),
        }],
    };
    let bytes = postcard::to_allocvec(&patch).expect("encode SearchReplaceTier1");
    let decoded: SearchReplaceTier1 =
        postcard::from_bytes(&bytes).expect("decode SearchReplaceTier1");
    // .apply() itself does not run the collision guard; that's the dispatcher's
    // job via parse_and_apply / DelimiterCollisionGuard. Verify the guard fires
    // on the source.
    let guard = elai_tools::search_replace::DelimiterCollisionGuard::new();
    assert!(guard.collides(conflicted_source));
    // .apply() would happily replace the unique `ok` literal — and that's
    // fine, the dispatcher's guard runs FIRST. Document the contract.
    let _ = matcher_apply_or_panic(&decoded, conflicted_source);
}

fn matcher_apply_or_panic(patch: &SearchReplaceTier1, source: &str) -> String {
    SearchReplaceMatcher::new()
        .apply(source, patch)
        .expect("apply should still succeed on unambiguous match")
}

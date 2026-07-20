// J-25 T3: tool-pair boundary guard (FORK-1 ROUTE A).
//
// Pure backward walk run before trim_context Pass 2 mutates `result`.
// Un-orphans tool_use/tool_result pairs at the candidate boundary so the
// rebuilt request never violates the Anthropic message invariants
// (a tool_result without a matching tool_use in-window is a 400; a
// tool_use without a matching tool_result leaves the assistant turn
// dangling). Zero I/O, no unwrap/expect, no LLM call.
//
// The thinking-sibling leg of the plan is DEFERRED: session::ContentBlock
// has no Thinking variant at edit time (S1 recon, re-grepped 2026-07-20).
// Adding one is a serde schema change wider than T3's budget and is
// owner-gated per FORK-1 ROUTE A in workflow/reasoning/J-25-decisions.md.

use crate::session::ContentBlock;
use crate::session_dag::{DagMessageRole, TrimmedMessage, PROMPT_CACHE_PREFIX_BOUNDARY};

/// Adjust the Pass-1 candidate set so stubbing it cannot orphan a
/// tool_use/tool_result pair at the trim boundary.
///
/// Inputs:
///   `messages`  — the full message slice seen by `trim_context`.
///   `candidates` — Pass-1 output: `(index, original_token_count)` pairs
///                  the trim pass proposed to stub. Indices refer to
///                  `messages`.
///
/// Output: a filtered candidate list. The function only REMOVES entries
/// from the candidate set; it never adds new ones (it cannot promote a
/// non-candidate to stubbable, and it must not widen the trim's blast
/// radius). Two invariants are enforced:
///
///   I1 — never stub a ToolResult whose matching ToolUse id would be
///        orphaned by the stub. A ToolResult lives in a Tool-role
///        message; its matching ToolUse lives in an earlier
///        Assistant-role message. If the Assistant message is still
///        in-window and unstubbed, stubbing the ToolResult orphans the
///        ToolUse (the assistant turn references a tool call whose
///        result vanished). We drop the candidate.
///
///   I2 — never stub a ToolUse that still has a live ToolResult
///        in-window. Symmetric: an Assistant message carrying a
///        ToolUse block is a stub candidate only if every ToolResult
///        matching that ToolUse id is ALSO a candidate (i.e. the pair
///        leaves the window together). If a live ToolResult would
///        remain, stubbing the ToolUse orphans the result. We drop the
///        candidate.
///
/// Fail-closed: if `blocks` is missing on a message whose role could
/// carry tool-pair content (Assistant with ToolUse, Tool with
/// ToolResult), the message is treated as ambiguous and ANY candidate
/// touching it is removed. The guard never silently stubs a message
/// it cannot reason about.
pub fn adjust_boundary_for_invariants(
    messages: &[TrimmedMessage],
    candidates: &[(usize, usize)],
) -> Vec<(usize, usize)> {
    if messages.is_empty() || candidates.is_empty() {
        return candidates.to_vec();
    }

    // Collect the set of candidate indices for O(1) membership.
    let mut candidate_set: std::collections::HashSet<usize> =
        candidates.iter().map(|(i, _)| *i).collect();

    // First pass: gather tool_use ids that would be ORPHANED by stubbing
    // (I1) and tool_use ids that are STAYING in-window (I2). We walk the
    // message list once and classify each candidate.
    //
    // tool_use_ids_in_window: every ToolUse id carried by an Assistant
    //   message that is NOT being stubbed (i.e. stays live).
    // tool_result_ids_being_stubbed: every ToolResult id carried by a
    //   Tool message that IS a stub candidate.
    // tool_result_ids_staying: every ToolResult id carried by a Tool
    //   message that is NOT a stub candidate.
    let mut tool_use_ids_staying: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let mut tool_use_ids_being_stubbed: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let mut tool_result_ids_being_stubbed: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    let mut tool_result_ids_staying: std::collections::HashSet<String> =
        std::collections::HashSet::new();

    for (i, msg) in messages.iter().enumerate() {
        let is_candidate = candidate_set.contains(&i);
        match msg.role {
            DagMessageRole::Assistant => {
                let blocks = match msg.blocks.as_ref() {
                    Some(b) => b,
                    None => continue,
                };
                for block in blocks.iter() {
                    if let ContentBlock::ToolUse { id, .. } = block {
                        if is_candidate {
                            tool_use_ids_being_stubbed.insert(id.clone());
                        } else {
                            tool_use_ids_staying.insert(id.clone());
                        }
                    }
                }
            }
            DagMessageRole::Tool => {
                let blocks = match msg.blocks.as_ref() {
                    Some(b) => b,
                    None => continue,
                };
                for block in blocks.iter() {
                    if let ContentBlock::ToolResult { tool_use_id, .. } = block {
                        if is_candidate {
                            tool_result_ids_being_stubbed.insert(tool_use_id.clone());
                        } else {
                            tool_result_ids_staying.insert(tool_use_id.clone());
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Second pass: drop candidates that violate I1 or I2.
    //
    // I1 — a Tool candidate is dropped if ANY of its ToolResult ids
    //   matches a ToolUse id that is STAYING in-window (stubbing the
    //   result orphans the use).
    //
    // I2 — an Assistant candidate is dropped if ANY of its ToolUse ids
    //   matches a ToolResult id that is STAYING in-window (stubbing the
    //   use orphans the result). Also drop if a ToolUse id has NO
    //   matching ToolResult anywhere in the window — that is an
    //   already-orphaned pair and stubbing the use cannot fix it, but
    //   we also must not make it worse by hiding the evidence; the
    //   conservative choice is to keep the Assistant message live.
    //
    // Fail-closed: a Tool or Assistant candidate whose blocks are None
    //   is ambiguous — drop it.
    candidates
        .iter()
        .filter(|(idx, tokens)| {
            let msg = &messages[*idx];
            // Respect the prompt-cache prefix floor: messages at or below
            // the boundary are never stub candidates by construction, but
            // if a caller handed us a candidate set that includes one,
            // drop it here too. Defensive, not a substitute for Pass 1.
            if msg.seq <= PROMPT_CACHE_PREFIX_BOUNDARY {
                return false;
            }
            match msg.role {
                DagMessageRole::Tool => {
                    let blocks = match msg.blocks.as_ref() {
                        Some(b) => b,
                        None => return false, // fail-closed
                    };
                    let mut keep = true;
                    for block in blocks.iter() {
                        if let ContentBlock::ToolResult { tool_use_id, .. } = block {
                            if tool_use_ids_staying.contains(tool_use_id) {
                                keep = false;
                                break;
                            }
                        }
                    }
                    keep
                }
                DagMessageRole::Assistant => {
                    let blocks = match msg.blocks.as_ref() {
                        Some(b) => b,
                        None => return false, // fail-closed
                    };
                    let mut keep = true;
                    for block in blocks.iter() {
                        if let ContentBlock::ToolUse { id, .. } = block {
                            if tool_result_ids_staying.contains(id) {
                                keep = false;
                                break;
                            }
                            // No matching ToolResult anywhere in the window:
                            // the pair is already broken. Keep the Assistant
                            // live so the malformed state stays visible to
                            // downstream validators rather than being masked
                            // by a stub.
                            if !tool_result_ids_being_stubbed.contains(id)
                                && !tool_result_ids_staying.contains(id)
                            {
                                keep = false;
                                break;
                            }
                        }
                    }
                    keep
                }
                _ => true, // non-tool-role candidates are unaffected by T3
            }
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assistant_with_tool_use(seq: u32, id: &str) -> TrimmedMessage {
        TrimmedMessage {
            seq,
            role: DagMessageRole::Assistant,
            content: vec![],
            is_stubbed: false,
            stub_ssb_seq: Some(100),
            blocks: Some(vec![ContentBlock::ToolUse {
                id: id.to_owned(),
                name: "ls".to_owned(),
                input: "{}".to_owned(),
            }]),
        }
    }

    fn tool_with_result(seq: u32, tool_use_id: &str) -> TrimmedMessage {
        TrimmedMessage {
            seq,
            role: DagMessageRole::Tool,
            content: vec![],
            is_stubbed: false,
            stub_ssb_seq: Some(200),
            blocks: Some(vec![ContentBlock::ToolResult {
                tool_use_id: tool_use_id.to_owned(),
                tool_name: "ls".to_owned(),
                output: "ok".to_owned(),
                is_error: false,
            }]),
        }
    }

    fn user_msg(seq: u32) -> TrimmedMessage {
        TrimmedMessage {
            seq,
            role: DagMessageRole::User,
            content: vec![],
            is_stubbed: false,
            stub_ssb_seq: None,
            blocks: None,
        }
    }

    /// Baseline: no candidates -> no-op.
    #[test]
    fn empty_candidates_pass_through() {
        let messages = vec![user_msg(1)];
        let out = adjust_boundary_for_invariants(&messages, &[]);
        assert!(out.is_empty());
    }

    /// I1 positive: a ToolResult candidate whose matching ToolUse is
    /// STAYING in-window MUST be dropped (stubbing it would orphan the
    /// ToolUse). This is the core Anthropic-invariant guard.
    #[test]
    fn tool_result_dropped_when_tool_use_stays() {
        let messages = vec![
            user_msg(1),
            user_msg(2),
            user_msg(3),
            assistant_with_tool_use(4, "tu-1"),
            tool_with_result(5, "tu-1"),
        ];
        // Candidate: stub the ToolResult at index 4.
        let candidates = vec![(4usize, 500usize)];
        let out = adjust_boundary_for_invariants(&messages, &candidates);
        assert!(
            out.is_empty(),
            "guard must drop the ToolResult candidate; got {:?}",
            out
        );
    }

    /// I1 negative: a ToolResult candidate whose matching ToolUse is
    /// ALSO a candidate (both leave the window together) is KEPT —
    /// stubbing the pair does not orphan either side.
    #[test]
    fn tool_result_kept_when_tool_use_also_candidate() {
        let messages = vec![
            user_msg(1),
            user_msg(2),
            user_msg(3),
            assistant_with_tool_use(4, "tu-1"),
            tool_with_result(5, "tu-1"),
        ];
        // Both the Assistant (idx 3) and the Tool (idx 4) are candidates.
        // The Tool has no other ToolResult in-window, so stubbing both
        // is pair-safe.
        let candidates = vec![(3usize, 500usize), (4usize, 500usize)];
        let out = adjust_boundary_for_invariants(&messages, &candidates);
        assert_eq!(
            out.len(),
            2,
            "both candidates should survive; got {:?}",
            out
        );
    }

    /// I2: an Assistant candidate carrying a ToolUse whose ToolResult
    /// is STAYING in-window MUST be dropped (stubbing the use orphans
    /// the result).
    #[test]
    fn tool_use_dropped_when_tool_result_stays() {
        let messages = vec![
            user_msg(1),
            user_msg(2),
            user_msg(3),
            assistant_with_tool_use(4, "tu-1"),
            tool_with_result(5, "tu-1"),
        ];
        // Candidate: stub only the Assistant at idx 3. The ToolResult
        // at idx 4 stays live -> stubbing the Assistant orphans it.
        let candidates = vec![(3usize, 500usize)];
        let out = adjust_boundary_for_invariants(&messages, &candidates);
        assert!(
            out.is_empty(),
            "guard must drop the Assistant candidate; got {:?}",
            out
        );
    }

    /// Fail-closed: a Tool candidate with blocks=None is ambiguous and
    /// MUST be dropped (we cannot verify the pair invariant without
    /// reading the ToolResult id).
    #[test]
    fn tool_candidate_with_no_blocks_dropped() {
        let mut msg = tool_with_result(5, "tu-1");
        msg.blocks = None;
        let messages = vec![
            user_msg(1),
            user_msg(2),
            user_msg(3),
            user_msg(4),
            msg,
        ];
        let candidates = vec![(4usize, 500usize)];
        let out = adjust_boundary_for_invariants(&messages, &candidates);
        assert!(
            out.is_empty(),
            "fail-closed: blocks=None Tool candidate must be dropped; got {:?}",
            out
        );
    }

    /// Fail-closed: an Assistant candidate with blocks=None is ambiguous
    /// and MUST be dropped.
    #[test]
    fn assistant_candidate_with_no_blocks_dropped() {
        let mut msg = assistant_with_tool_use(4, "tu-1");
        msg.blocks = None;
        let messages = vec![
            user_msg(1),
            user_msg(2),
            user_msg(3),
            msg,
            user_msg(5),
        ];
        let candidates = vec![(3usize, 500usize)];
        let out = adjust_boundary_for_invariants(&messages, &candidates);
        assert!(
            out.is_empty(),
            "fail-closed: blocks=None Assistant candidate must be dropped; got {:?}",
            out
        );
    }

    /// REDTEAM ATTACK-1 (negative fixture): with the guard DISABLED
    /// (i.e. using the raw Pass-1 candidate set unchanged), the
    /// ToolResult at idx 4 would be stubbed and the ToolUse at idx 3
    /// would be orphaned. This test asserts the guard ENABLED produces
    /// the non-orphaning set; the matching negative (guard disabled)
    /// is exercised by calling `trim_context` with the guard bypassed,
    /// which is not exposed in production — the contract test in
    /// crates/contracts/tests/j25_context_boundary_contract.rs covers
    /// the full red-then-green path.
    #[test]
    fn attack1_guard_enabled_returns_non_orphaning_set() {
        let messages = vec![
            user_msg(1),
            user_msg(2),
            user_msg(3),
            assistant_with_tool_use(4, "tu-1"),
            tool_with_result(5, "tu-1"),
        ];
        let candidates = vec![(4usize, 500usize)];
        let out = adjust_boundary_for_invariants(&messages, &candidates);
        // The guard MUST drop the orphaning candidate.
        assert!(
            out.iter().all(|(i, _)| *i != 4),
            "guard enabled must not leave the orphaning ToolResult candidate; got {:?}",
            out
        );
    }

    /// Prefix-floor: a candidate at seq <= PROMPT_CACHE_PREFIX_BOUNDARY
    /// is dropped defensively (Pass 1 already filters these, but a
    /// caller-constructed candidate set might not).
    #[test]
    fn prefix_floor_candidate_dropped() {
        let messages = vec![
            user_msg(1),
            user_msg(2),
            user_msg(3),
        ];
        // idx 0 has seq 1, below the floor.
        let candidates = vec![(0usize, 500usize)];
        let out = adjust_boundary_for_invariants(&messages, &candidates);
        assert!(
            out.is_empty(),
            "prefix-floor candidate must be dropped; got {:?}",
            out
        );
    }

    /// Non-tool-role candidates are passed through untouched.
    #[test]
    fn non_tool_candidates_pass_through() {
        let messages = vec![
            user_msg(1),
            user_msg(2),
            user_msg(3),
            user_msg(4),
            user_msg(5),
        ];
        let candidates = vec![(3usize, 500usize), (4usize, 500usize)];
        let out = adjust_boundary_for_invariants(&messages, &candidates);
        assert_eq!(out.len(), 2, "non-tool candidates should pass through");
    }

    /// Multiple tool pairs: only the orphaning candidate is dropped,
    /// pair-safe candidates survive.
    #[test]
    fn multiple_pairs_only_orphans_dropped() {
        let messages = vec![
            user_msg(1),
            user_msg(2),
            user_msg(3),
            assistant_with_tool_use(4, "tu-1"),
            tool_with_result(5, "tu-1"),
            assistant_with_tool_use(6, "tu-2"),
            tool_with_result(7, "tu-2"),
        ];
        // Stub only the ToolResult of pair 1 (orphaning tu-1) and BOTH
        // sides of pair 2 (pair-safe). The guard must drop idx 4 and
        // keep idx 5 + idx 6.
        let candidates = vec![
            (4usize, 500usize), // tu-1 result only -> orphan, drop
            (5usize, 500usize), // tu-2 assistant (use)
            (6usize, 500usize), // tu-2 tool (result)
        ];
        let out = adjust_boundary_for_invariants(&messages, &candidates);
        let kept: Vec<usize> = out.iter().map(|(i, _)| *i).collect();
        assert!(
            !kept.contains(&4),
            "orphaning tu-1 result must be dropped; got {:?}",
            kept
        );
        assert!(
            kept.contains(&5) && kept.contains(&6),
            "pair-safe tu-2 candidates must survive; got {:?}",
            kept
        );
    }
}

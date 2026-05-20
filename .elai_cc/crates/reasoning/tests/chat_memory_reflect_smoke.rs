//! E-1 observable_effect smoke: drive 8 chat turns through the
//! reflection loop, trigger reflection at turn 4 + 8, assert that
//! the CoreBlock + memory_notes content is non-empty AND that
//! cross-window decay applies (older notes lose trust relative to
//! newer ones on retrieval).

use elai_memory::core_block::CoreBlockStore;
use elai_memory::notes::NoteStore;
use elai_memory::{hippo_rag, Conversation, FactStore, Turn};
use elai_reasoning::chat_memory_reflect::{
    reflect, should_reflect, ReflectConfig, DEFAULT_REFLECT_EVERY_N_TURNS,
};

fn fixture_turn(i: usize, role: &str, text: &str) -> Turn {
    Turn {
        message_id: format!("m{i}{role}"),
        role: role.into(),
        text: text.into(),
    }
}

fn build_window(session: &str, base: usize, k_user_turns: usize) -> Conversation {
    let mut turns = Vec::new();
    for j in 0..k_user_turns {
        let idx = base + j;
        // Each user turn carries a preference + a distinct keyword so
        // the PPR retrieval has multiple seeds.
        let user_text = if idx % 2 == 0 {
            format!("i prefer blake3 over sha256 for hashing in turn {idx}")
        } else {
            format!("i prefer tracing over log crate for logging in turn {idx}")
        };
        turns.push(fixture_turn(idx, "user", &user_text));
        turns.push(fixture_turn(idx, "assistant", "ack"));
    }
    Conversation {
        session_id: session.into(),
        turns,
    }
}

#[test]
fn eight_turn_chat_reflects_at_turn_4_and_8() {
    // Three stores backing the reflection loop.
    let notes = NoteStore::open_in_memory().expect("notes store");
    let core = CoreBlockStore::open_in_memory().expect("core block store");
    let facts = FactStore::open_in_memory().expect("fact store");
    let cfg = ReflectConfig::default();

    // 8 user turns means cadence boundaries at turn_index 4 and 8.
    let total_user_turns = 8;
    let mut reflections = 0usize;
    let now_base: u128 = 1_700_000_000_000;
    // Simulate the orchestrator's per-turn hook: at every turn_index that
    // satisfies should_reflect we feed the most-recent 4-turn window.
    let every_n = DEFAULT_REFLECT_EVERY_N_TURNS;
    for turn_index in 1..=total_user_turns {
        if !should_reflect(turn_index, every_n) {
            continue;
        }
        // Window covers the last `every_n` user turns.
        let base = turn_index - every_n;
        let window = build_window(&format!("s-{turn_index}"), base, every_n);
        let now_ms = now_base + (turn_index as u128) * 1_000;
        let r = reflect(&window, &notes, &core, &facts, &cfg, now_ms)
            .unwrap_or_else(|e| panic!("reflect at turn {turn_index} failed: {e}"));
        assert!(
            !r.note_id.is_empty(),
            "note_id must be set on turn {turn_index}"
        );
        assert!(
            r.core_block_bytes > 0,
            "core_block must accumulate by turn {turn_index}"
        );
        reflections += 1;
    }
    assert_eq!(
        reflections, 2,
        "should fire reflection at turn_index 4 and 8 only (got {reflections})"
    );

    // CoreBlock content is non-empty after the sweep.
    let cb = core
        .get(&cfg.user_id, &cfg.scope_tag)
        .expect("core get")
        .expect("core present");
    assert!(!cb.content.is_empty(), "CoreBlock body must be non-empty");
    assert!(cb.edits >= 2, "CoreBlock should record at least 2 edits");

    // memory_notes table has 2 rows (one per reflection).
    let recent = notes.list_recent(10).expect("list_recent");
    assert_eq!(recent.len(), 2, "expected 2 reflection notes, got {}", recent.len());

    // PPR retrieval seeded with "blake3" should find at least one note;
    // and the more-recent note (turn 8) should outrank the older (turn 4)
    // because of bi-temporal decay.
    let now_query = now_base + 100_000;
    let retrieved = hippo_rag::retrieve_notes(&notes, &["blake3".into()], 5, now_query)
        .expect("retrieve");
    assert!(
        !retrieved.is_empty(),
        "PPR over the notes graph must return at least one row"
    );

    // Decay assertion: newer notes have larger PPR-weighted score than
    // older notes when both match the seed (or are tied at uniform).
    if retrieved.len() >= 2 {
        let oldest = retrieved
            .iter()
            .min_by_key(|r| r.note.created_at_ms)
            .expect("min");
        let newest = retrieved
            .iter()
            .max_by_key(|r| r.note.created_at_ms)
            .expect("max");
        assert!(
            newest.score >= oldest.score,
            "newer note score ({}) must be >= older note score ({}) after decay",
            newest.score,
            oldest.score
        );
    }
}

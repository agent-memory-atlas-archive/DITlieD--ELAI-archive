// Integration tests for knowledge::symbol_graph::sharding.
// These tests run via `cargo test -p elai-knowledge`.

use elai_knowledge::symbol_graph::sharding::{
    all_name_shard_keys, name_shard_key, reverse_shard_bucket, SYMBOL_NAME_SHARDS,
    SYMBOL_REVERSE_SHARDS,
};

// ---------------------------------------------------------------------------
// name_shard_key
// ---------------------------------------------------------------------------

#[test]
fn name_shard_key_lowercase_letter() {
    assert_eq!(name_shard_key("abc"), 'a');
}

#[test]
fn name_shard_key_uppercase_letter() {
    assert_eq!(name_shard_key("Abc"), 'a');
}

#[test]
fn name_shard_key_digit_first() {
    assert_eq!(name_shard_key("123"), '_');
}

#[test]
fn name_shard_key_underscore_first() {
    assert_eq!(name_shard_key("_x"), '_');
}

#[test]
fn name_shard_key_empty_string() {
    assert_eq!(name_shard_key(""), '_');
}

#[test]
fn name_shard_key_non_ascii() {
    // A non-ASCII first char should fall to '_'
    assert_eq!(name_shard_key("ñame"), '_');
}

#[test]
fn name_shard_key_all_letters() {
    for c in b'a'..=b'z' {
        let s = format!("{}", c as char);
        assert_eq!(name_shard_key(&s), c as char, "shard key for letter {}", c as char);
        let upper = format!("{}", (c as char).to_ascii_uppercase());
        assert_eq!(
            name_shard_key(&upper),
            c as char,
            "shard key for uppercase letter {}",
            c as char
        );
    }
}

// ---------------------------------------------------------------------------
// all_name_shard_keys
// ---------------------------------------------------------------------------

#[test]
fn all_name_shard_keys_count() {
    let keys = all_name_shard_keys();
    assert_eq!(keys.len(), SYMBOL_NAME_SHARDS, "must have exactly {SYMBOL_NAME_SHARDS} shards");
    assert_eq!(keys.len(), 27);
}

#[test]
fn all_name_shard_keys_sorted_and_correct() {
    let keys = all_name_shard_keys();
    // First key must be '_'
    assert_eq!(keys[0], '_');
    // Remaining 26 keys must be 'a'..'z' in order
    for (i, expected) in (b'a'..=b'z').enumerate() {
        assert_eq!(
            keys[i + 1],
            expected as char,
            "index {} should be '{}'",
            i + 1,
            expected as char
        );
    }
    // Verify the array is sorted (lexicographic: '_' < 'a' in ASCII is false,
    // but our contract is ['_','a'..'z'] so check the exact order, not ASCII sort)
    let as_vec: Vec<char> = keys.into();
    let mut sorted = as_vec.clone();
    // Our order: '_' first, then a-z.  That is NOT the same as lexicographic
    // ASCII sort ('_' is 0x5F > 'Z' 0x5A > ... > 'a' 0x61), so we assert the
    // contract order directly.
    sorted.sort();
    // The sorted order would put 'a'..'z' before '_' (0x5F > 0x7A is false,
    // '_' = 0x5F, 'a' = 0x61 -> '_' < 'a' in ASCII, so sorted[0] == '_').
    // ASCII: '_' = 95, 'a' = 97 -> '_' sorts before 'a'.
    assert_eq!(sorted[0], '_');
    for i in 0..26 {
        assert_eq!(sorted[i + 1], (b'a' + i as u8) as char);
    }
}

// ---------------------------------------------------------------------------
// reverse_shard_bucket
// ---------------------------------------------------------------------------

#[test]
fn reverse_shard_bucket_deterministic() {
    let file = "foo/bar.rs";
    let b1 = reverse_shard_bucket(file);
    let b2 = reverse_shard_bucket(file);
    assert_eq!(b1, b2, "reverse_shard_bucket must be deterministic");
}

#[test]
fn reverse_shard_bucket_in_range() {
    for file in &["foo.rs", "bar/baz.py", "src/lib.rs", ""] {
        let b = reverse_shard_bucket(file);
        // u8 is always 0..=255, but assert explicitly for clarity
        assert!(
            (b as usize) < SYMBOL_REVERSE_SHARDS,
            "bucket {b} for '{file}' must be < {SYMBOL_REVERSE_SHARDS}"
        );
    }
}

#[test]
fn reverse_shard_bucket_distribution_across_corpus() {
    // Build a corpus of 1000 file paths.  No single bucket should contain
    // more than 30 entries for a natural corpus.
    let paths: Vec<String> = (0..1000)
        .map(|i| format!("src/module_{i}/file_{}.rs", i % 37))
        .collect();

    let mut counts = [0u32; 256];
    for p in &paths {
        let b = reverse_shard_bucket(p) as usize;
        counts[b] += 1;
    }

    let max_count = *counts.iter().max().unwrap_or(&0);
    assert!(
        max_count <= 30,
        "worst-case bucket has {max_count} entries — expected <= 30 for uniform distribution"
    );
}

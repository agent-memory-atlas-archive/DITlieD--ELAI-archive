// B-29 Phase 1 — sharding helpers for the symbol graph index.
//
// Mirrors SocratiCode's constants.ts L130-166 (SYMBOL_NAME_SHARDS=27,
// SYMBOL_REVERSE_SHARDS=256, SYMBOL_FILE_LRU_SIZE=500, MAX_IMPACT_DEPTH=10,
// MAX_FLOW_DEPTH=10).
//
// reverse_shard_bucket deviates from the plan's "first byte of SHA1(file)":
// SHA1 is not a workspace dependency. blake3 IS (knowledge/Cargo.toml:31,
// blake3 = "1.8"). blake3::hash(file)[0] gives the same functional property
// (uniform distribution over 0..255) and aligns with the CLAUDE.md
// anti-stale rule (blake3 preferred over sha1/sha2 for non-RFC uses).

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Number of name-index shards: 'a'..'z' (26) + '_' (1) = 27.
pub const SYMBOL_NAME_SHARDS: usize = 27;

/// Number of reverse-index shards: one per possible first byte of blake3 hash.
pub const SYMBOL_REVERSE_SHARDS: usize = 256;

/// LRU capacity for per-file payload cache (Phase 3).
pub const SYMBOL_FILE_LRU_SIZE: usize = 500;

/// Maximum BFS depth for impact-radius computation.
pub const MAX_IMPACT_DEPTH: u32 = 10;

/// Maximum DFS depth for call-flow computation.
pub const MAX_FLOW_DEPTH: u32 = 10;

// ---------------------------------------------------------------------------
// Shard key functions
// ---------------------------------------------------------------------------

/// Return the name-index shard key for `name`.
///
/// Rules:
/// - Empty string -> '_'
/// - First char is ASCII letter -> lowercased letter ('a'..'z')
/// - Anything else (digit, symbol, non-ASCII) -> '_'
///
/// Never panics; safe on empty input.
pub fn name_shard_key(name: &str) -> char {
    match name.chars().next() {
        Some(c) if c.is_ascii_alphabetic() => c.to_ascii_lowercase(),
        _ => '_',
    }
}

/// Return the reverse-index shard bucket for `file`.
///
/// Computes `blake3::hash(file.as_bytes())[0]` (first byte of the 32-byte
/// hash), giving a value in 0..=255 that is uniformly distributed across
/// the 256 buckets for a natural corpus of file paths.
///
/// Deviation from plan's "first byte of SHA1(file)": SHA1 is not in the
/// workspace; blake3 is. Both give uniform distribution; blake3 is preferred
/// by the anti-stale rule.
pub fn reverse_shard_bucket(file: &str) -> u8 {
    blake3::hash(file.as_bytes()).as_bytes()[0]
}

/// Return all 27 name-index shard keys in sorted order: ['_', 'a', 'b', ..., 'z'].
///
/// The underscore shard comes first so iteration order is deterministic and
/// matches a simple lexicographic sort.
pub fn all_name_shard_keys() -> [char; 27] {
    let mut keys = ['_'; 27];
    keys[0] = '_';
    for (i, c) in (b'a'..=b'z').enumerate() {
        keys[i + 1] = c as char;
    }
    keys
}

// ---------------------------------------------------------------------------
// Unit tests (kept here as well as the integration test file)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_shard_key_basic() {
        assert_eq!(name_shard_key("abc"), 'a');
        assert_eq!(name_shard_key("Abc"), 'a');
        assert_eq!(name_shard_key("123"), '_');
        assert_eq!(name_shard_key("_x"), '_');
        assert_eq!(name_shard_key(""), '_');
    }

    #[test]
    fn all_name_shard_keys_count_and_order() {
        let keys = all_name_shard_keys();
        assert_eq!(keys.len(), 27);
        assert_eq!(keys[0], '_');
        for i in 0..26 {
            assert_eq!(keys[i + 1], (b'a' + i as u8) as char);
        }
    }
}

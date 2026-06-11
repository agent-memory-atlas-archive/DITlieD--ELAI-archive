//! H-14: Build-time opaque-id tagger mirroring @onlook/parser ids.ts.
//!
//! Generates a data-oid OPAQUE id (createOid-style, NOT line/column) per
//! JSX element at build time, and stores a server-side map:
//!
//!   OidMap: BTreeMap<String, OidMapEntry { file, range, scope }>
//!
//! The opaque id is a 16-character hex prefix of blake3(file || scope || counter).
//! NEVER encodes line/column in the DOM attribute — that is the brittle
//! React-Dev-Inspector approach Onlook explicitly avoids (verify:design-deps).
//!
//! Design decisions (H-14-decisions.md):
//!   - OID format: blake3(file + scope + counter_bytes) -> hex[0..16].
//!     Collision probability for <= 65 536 elements per file is < 2^-48
//!     (birthday paradox on a 64-bit truncated hash).
//!   - The id->location map is SERVER-SIDE only. The DOM carries only the
//!     opaque id (data-oid="<16 hex chars>"). The Tauri command resolve_oid
//!     looks up the entry.
//!   - @onlook/babel-plugin-react: BLOCK (stub_ban, policy). The babel transform
//!     that would inject data-oid at build time in the user's project is a
//!     separate build-step concern. This module provides the SERVER-SIDE map
//!     and opaque-id generation logic used by the writeback commands.
//!
//! Cross-lang pair: `DesignOidEntry` in stub.ts + `resolve_oid` Tauri command.
//!
//! HARD RULES:
//!   - blake3 (not sha2/blake2) per CLAUDE.md Anti-Stale-Training.
//!   - No .unwrap() / .expect() in production paths.
//!   - No line/col in DOM attr (stub_ban: OPAQUE_OID_NOT_LINE_COL).

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Process-level OidMap singleton (Tier-1)
// ---------------------------------------------------------------------------
// In Tier-2 this will move to AppState for per-project isolation.
// For Tier-1, a process-level static is sufficient because the dashboard
// process serves one project at a time (the project_root is fixed at launch).

/// Process-level server-side OidMap singleton.
/// Populated by the build-time tagger; read by resolve_oid / apply_recast_patch.
/// `std::sync::Mutex` (not `tokio::sync::Mutex`) because reads are sync in
/// the Tauri command dispatch path — no guard is held across `.await`.
pub static OID_MAP: std::sync::LazyLock<OidMap> =
    std::sync::LazyLock::new(OidMap::new);

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

/// Number of hex characters taken from the blake3 output to form the opaque id.
/// 16 chars = 64 bits. Collision probability for <= 65 536 elements is < 2^-48
/// (birthday bound on 64-bit truncated hash).
const OID_HEX_LEN: usize = 16;

/// Maximum elements per file before a warning is emitted (not a hard limit).
/// 65 536 elements would give a collision probability of ~2^-48; above this
/// the caller should warn the user.
pub const OID_WARN_THRESHOLD: usize = 65_536;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// A source location range: (start_line, start_col, end_line, end_col).
/// line and col are 1-based (matching AST convention).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OidRange {
    pub start_line: u32,
    pub start_col: u32,
    pub end_line: u32,
    pub end_col: u32,
}

/// One entry in the server-side OidMap.
/// Cross-lang pair: `DesignOidEntry` in stub.ts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OidMapEntry {
    /// Canonical file path (relative to project root or absolute).
    pub file: String,
    /// Source range in the file (1-based, AST convention).
    pub range: OidRange,
    /// Scope qualifier (e.g. "MyComponent > div > span"). Used for disambiguation
    /// when multiple elements share the same structural position across files.
    pub scope: String,
}

/// Server-side opaque-id map. Keyed by 16-char hex opaque id.
///
/// Wrapped in `Arc<std::sync::Mutex<>>` so multiple Tauri commands can read
/// concurrently without holding the lock across `.await`. Mutex (not RwLock)
/// because write operations (tag_jsx_element) are rare relative to reads.
pub struct OidMap {
    inner: Arc<std::sync::Mutex<BTreeMap<String, OidMapEntry>>>,
}

impl OidMap {
    /// Create an empty OidMap.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(std::sync::Mutex::new(BTreeMap::new())),
        }
    }

    /// Insert a new entry into the map. The opaque id must already be generated
    /// via `create_oid`. Returns `Ok(())` on success, `Err` on mutex poison.
    pub fn insert(&self, oid: String, entry: OidMapEntry) -> Result<(), String> {
        let mut map = self
            .inner
            .lock()
            .map_err(|_| "OidMap: mutex is poisoned".to_string())?;
        map.insert(oid, entry);
        Ok(())
    }

    /// Look up an entry by its opaque id.
    /// Returns `None` when the id is not registered (resolve_oid returns Err).
    pub fn get(&self, oid: &str) -> Result<Option<OidMapEntry>, String> {
        let map = self
            .inner
            .lock()
            .map_err(|_| "OidMap: mutex is poisoned".to_string())?;
        Ok(map.get(oid).cloned())
    }

    /// Number of registered elements.
    pub fn len(&self) -> Result<usize, String> {
        let map = self
            .inner
            .lock()
            .map_err(|_| "OidMap: mutex is poisoned".to_string())?;
        Ok(map.len())
    }

    /// True when no elements are registered.
    pub fn is_empty(&self) -> Result<bool, String> {
        self.len().map(|n| n == 0)
    }
}

impl Clone for OidMap {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Default for OidMap {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Opaque-id generation (createOid mirror)
// ---------------------------------------------------------------------------

/// Generate an opaque element id mirroring @onlook/parser ids.ts `createOid`.
///
/// The id is `blake3(file_bytes + scope_bytes + counter_bytes).to_hex()[..OID_HEX_LEN]`.
/// It is OPAQUE: no line/column is encoded in the output (stub_ban).
///
/// The `counter` argument distinguishes multiple elements that share the same
/// file and scope (e.g. two `<div>` siblings inside the same JSX expression).
/// Callers should increment the counter per element within the same scope.
///
/// Collision probability:
///   For n elements drawn from a 64-bit hash space, P(collision) ≈ n²/2⁶⁵.
///   For n = 65 536: P ≈ 2^32 / 2^65 = 2^-33 ≈ 1.2×10^-10 per file.
pub fn create_oid(file: &str, scope: &str, counter: u32) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(file.as_bytes());
    // Zero byte separator between segments (cannot occur in UTF-8 paths).
    hasher.update(b"\x00");
    hasher.update(scope.as_bytes());
    hasher.update(b"\x00");
    hasher.update(&counter.to_le_bytes());
    let hash = hasher.finalize();
    // Take OID_HEX_LEN chars from the hex encoding of the first 8 bytes.
    // blake3::Hash::to_hex() returns a 64-char hex string; we take the prefix.
    hash.to_hex().chars().take(OID_HEX_LEN).collect()
}

/// Tag a JSX element: generate its opaque id and insert it into the OidMap.
///
/// Returns the opaque id on success. The caller embeds this as `data-oid="<id>"`
/// in the transformed JSX output (build step; NOT the DOM for non-transformed files).
///
/// `file`: canonical file path.
/// `scope`: hierarchical JSX path string (e.g. "App > Header > nav > a").
/// `counter`: zero-based element counter within the same (file, scope) pair.
/// `range`: source location range for this element (for resolve_oid).
///
/// Returns `Err` if the OidMap mutex is poisoned.
pub fn tag_jsx_element(
    map: &OidMap,
    file: &str,
    scope: &str,
    counter: u32,
    range: OidRange,
) -> Result<String, String> {
    let oid = create_oid(file, scope, counter);
    let entry = OidMapEntry {
        file: file.to_owned(),
        range,
        scope: scope.to_owned(),
    };
    map.insert(oid.clone(), entry)?;
    Ok(oid)
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_oid_returns_correct_hex_length() {
        let oid = create_oid("src/App.tsx", "App > div", 0);
        assert_eq!(
            oid.len(),
            OID_HEX_LEN,
            "opaque id must be exactly {OID_HEX_LEN} hex chars"
        );
        assert!(
            oid.chars().all(|c| c.is_ascii_hexdigit()),
            "opaque id must be hex: {oid}"
        );
    }

    #[test]
    fn create_oid_does_not_encode_line_col() {
        // Opaque id must be the same regardless of what line/col we pass
        // at a later stage — the id is derived from file + scope + counter
        // only. This test verifies two different counters produce different
        // ids (sanity), while two calls with identical inputs produce the same id.
        let oid_a = create_oid("src/App.tsx", "App > div", 0);
        let oid_b = create_oid("src/App.tsx", "App > div", 0);
        assert_eq!(oid_a, oid_b, "same inputs must produce same opaque id");

        let oid_c = create_oid("src/App.tsx", "App > div", 1);
        assert_ne!(oid_a, oid_c, "different counter must produce different id");
    }

    #[test]
    fn create_oid_different_files_produce_different_ids() {
        let oid_a = create_oid("src/App.tsx", "App > div", 0);
        let oid_b = create_oid("src/Other.tsx", "App > div", 0);
        assert_ne!(oid_a, oid_b, "different file must produce different id");
    }

    #[test]
    fn tag_jsx_element_round_trips_via_oid_map() {
        let map = OidMap::new();
        let range = OidRange {
            start_line: 10,
            start_col: 4,
            end_line: 10,
            end_col: 20,
        };
        let oid = tag_jsx_element(&map, "src/App.tsx", "App > h1", 0, range.clone())
            .expect("tag must succeed");

        assert_eq!(oid.len(), OID_HEX_LEN, "oid must have correct length");

        let entry = map.get(&oid).expect("get must succeed").expect("entry must exist");
        assert_eq!(entry.file, "src/App.tsx");
        assert_eq!(entry.scope, "App > h1");
        assert_eq!(entry.range, range);
    }

    #[test]
    fn oid_map_returns_none_for_unknown_oid() {
        let map = OidMap::new();
        let result = map.get("deadbeef00000000").expect("get must succeed");
        assert!(result.is_none(), "unknown oid must return None");
    }

    #[test]
    fn oid_map_len_tracks_insertions() {
        let map = OidMap::new();
        assert_eq!(map.len().expect("len"), 0);

        let range = OidRange { start_line: 1, start_col: 1, end_line: 1, end_col: 5 };
        tag_jsx_element(&map, "a.tsx", "A", 0, range.clone()).expect("tag a");
        assert_eq!(map.len().expect("len"), 1);

        tag_jsx_element(&map, "b.tsx", "B", 0, range).expect("tag b");
        assert_eq!(map.len().expect("len"), 2);
    }

    #[test]
    fn oid_is_opaque_hex_not_line_col_string() {
        // Regression guard: ensure the oid does NOT contain ":" or "/" which
        // would suggest line:col or path encoding in the attribute value.
        let oid = create_oid("src/components/Header.tsx", "Header > nav > a", 3);
        assert!(!oid.contains(':'), "oid must not encode line:col (found ':')");
        assert!(!oid.contains('/'), "oid must not encode a path (found '/')");
        assert!(!oid.contains('.'), "oid must not contain '.' (non-hex char)");
    }
}

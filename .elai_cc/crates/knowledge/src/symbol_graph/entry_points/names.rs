// B-34 — Conventional entry-point name table.
//
// Ported verbatim from SocratiCode constants.ts:151-166 (14 languages).
// Python: Set(["main"]) in the TS source; __main__ is handled separately
// in detect.rs as an additive check (s.name == "__main__").
// csharp is the only language whose conventional name is "Main" (capital M).

use std::collections::{BTreeMap, BTreeSet};

/// Returns a map from language identifier -> set of conventional entry-point
/// symbol names. Keys are lower-case language IDs matching
/// `SymbolGraphFilePayload::language`.
///
/// Lookup is case-sensitive: callers must compare `sym.name` directly against
/// the returned sets. "main" and "Main" are distinct entries.
pub fn entry_point_names_per_language() -> BTreeMap<&'static str, &'static [&'static str]> {
    let mut map = BTreeMap::new();
    // All 14 languages from SocratiCode constants.ts:152-165.
    // Only csharp uses "Main" (capital M); all others use "main".
    map.insert("javascript", &["main"] as &[&str]);
    map.insert("typescript", &["main"] as &[&str]);
    map.insert("python",     &["main"] as &[&str]);
    map.insert("go",         &["main"] as &[&str]);
    map.insert("rust",       &["main"] as &[&str]);
    map.insert("java",       &["main"] as &[&str]);
    map.insert("kotlin",     &["main"] as &[&str]);
    map.insert("scala",      &["main"] as &[&str]);
    map.insert("c",          &["main"] as &[&str]);
    map.insert("cpp",        &["main"] as &[&str]);
    map.insert("csharp",     &["Main"] as &[&str]);
    map.insert("swift",      &["main"] as &[&str]);
    map.insert("ruby",       &["main"] as &[&str]);
    map.insert("php",        &["main"] as &[&str]);
    map
}

/// Conventional entry-point filenames (case-sensitive stems).
///
/// Used by the dispatcher to pre-filter candidate files for payload load,
/// keeping payload loads selective (B-29 lazy-loading invariant).
/// Callers: compare the file's path stem (without extension) against this set.
pub fn entry_point_filenames() -> BTreeSet<&'static str> {
    let mut set = BTreeSet::new();
    // Standard stems across languages.
    set.insert("main");
    set.insert("index");
    set.insert("__main__");
    // C# uses PascalCase; on case-insensitive file systems both "main" and
    // "Main" would match "main" after lower-casing, but we preserve both to
    // handle case-sensitive Linux file systems correctly.
    set.insert("Main");
    set
}

/// Returns true if the given file path (repo-relative) has a stem that
/// matches one of the conventional entry-point filenames.
///
/// The stem is the final path component stripped of its extension.
/// Comparison is case-sensitive.
pub fn is_entry_point_filename(file_path: &str) -> bool {
    let stems = entry_point_filenames();
    // Extract the stem: last path component, strip extension.
    let base = file_path
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(file_path);
    // Strip extension: everything up to (but not including) the last '.'.
    let stem = if let Some(dot_pos) = base.rfind('.') {
        &base[..dot_pos]
    } else {
        base
    };
    stems.contains(stem)
}

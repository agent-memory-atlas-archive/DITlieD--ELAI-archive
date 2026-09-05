Phase 1B.1-1B.2 Decision Doc

What: Add tantivy 0.26.0 dependency and create Bm25Index skeleton in knowledge crate.

Key decisions:
- tantivy = "0.26.0" (verified via cargo search 2026-04-07)
- INDEX_FORMAT_VERSION = 7 (verified from tantivy 0.26.0 src/lib.rs)
- path field: STRING | STORED (untokenized for exact-match delete_term)
- content field: TEXT | STORED (tokenized for BM25 search)
- line_start/line_end: u64 STORED only (no range queries in 1B)
- chunk_kind: STRING | STORED (untokenized enum string)
- Index path: project_root.join("projects").join(project_id).join("bm25")
- Single-writer model: IndexWriter held for Bm25Index lifetime
- Skeleton methods return Ok(defaults) — no todo!() or unimplemented!()

Expected test outcome: cargo check -p knowledge passes. Contract tests compile but are #[ignore]d.

Unknowns:
- tantivy compile time on Windows (may be slow first build)
- Heap size for IndexWriter (default 50MB) — acceptable for skeleton

Failure modes:
1. tantivy C compilation fails on Windows — mitigated by using bundled features if needed
2. Schema field type mismatch breaks delete_term — mitigated by using STRING for path
3. PathBuf string interpolation on Windows — mitigated by using .join() chains only

Phase 1B.4 Addendum

What: walk_dir hidden-directory skip guard.

Key decisions:
- Guard uses entry.file_name().starts_with('.') to skip ALL dot-prefixed entries (dirs and files)
- Single check covers .git/, .vscode/, .elai_cc/, .gitignore, etc.
- No separate .elai_cc path-component check needed — .elai_cc starts with '.'
- No new IndexStats fields added (spec does not require files_skipped_hidden counter)
- Guard placed before ft.is_dir()/ft.is_file() branch — skips both hidden dirs and hidden files

Path convention resolution:
- Impl: project_root.join("projects").join(project_id).join("bm25")
- Spec says {project_root}/.elai_cc/projects/{id}/bm25/ but callers pass .elai_cc/ as project_root
- Convention is consistent: Bm25Index::new receives an already-resolved root, not the raw project dir
- No code change needed. Contract tests validate current convention.

Expected test outcome: 20 existing + 1 new test = 21 GREEN.

Failure modes:
1. OsStr comparison on non-UTF8 filenames — mitigated by to_string_lossy (replaces invalid with U+FFFD, won't start with '.')
2. Symlink to hidden dir — skipped by name check on the symlink entry itself, not the target

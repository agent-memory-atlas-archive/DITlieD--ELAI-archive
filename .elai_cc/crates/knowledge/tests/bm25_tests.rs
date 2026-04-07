use std::path::PathBuf;

use elai_knowledge::bm25::{Bm25Index, BM25Chunk, ChunkKind};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn write_file(dir: &std::path::Path, rel: &str, content: &str) -> PathBuf {
    let full = dir.join(rel);
    if let Some(parent) = full.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(&full, content).unwrap();
    full
}

// ---------------------------------------------------------------------------
// (a) Index single Rust file, query symbol, verify path + line range
// ---------------------------------------------------------------------------

#[test]
fn a_index_single_file_query_symbol() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut idx = Bm25Index::new(tmp.path(), "test_a").unwrap();

    let code = "fn hello_world() {\n    println!(\"hi\");\n}\n";
    let file = write_file(tmp.path(), "hello.rs", code);

    let chunks = vec![BM25Chunk {
        path: file.clone(),
        content: code.to_string(),
        line_start: 1,
        line_end: 3,
        kind: ChunkKind::Function,
    }];
    let n = idx.index_file(&file, Some(chunks)).unwrap();
    assert!(n > 0);

    let hits = idx.query("hello_world", 10).unwrap();
    assert!(!hits.is_empty(), "expected at least one hit for hello_world");
    let hit = &hits[0];
    assert_eq!(hit.path, file.to_string_lossy().as_ref());
    assert!(hit.line_start >= 1);
    assert!(hit.line_end >= hit.line_start);
}

// ---------------------------------------------------------------------------
// (b) Index tempdir with 3 files, query specific symbol
// ---------------------------------------------------------------------------

#[test]
fn b_index_dir_query_specific_symbol() {
    let tmp = tempfile::TempDir::new().unwrap();
    let src = tmp.path().join("src");
    std::fs::create_dir_all(&src).unwrap();

    write_file(&src, "alpha.rs", "fn alpha_func() { 1 }\n");
    write_file(
        &src,
        "beta.rs",
        "fn complete(input: &str) {\n    let _ = input;\n}\n",
    );
    write_file(&src, "gamma.rs", "fn gamma_func() { 3 }\n");

    let mut idx = Bm25Index::new(tmp.path(), "test_b").unwrap();
    let stats = idx.index_directory(&src).unwrap();
    assert_eq!(stats.files_indexed, 3);

    let hits = idx.query("complete", 10).unwrap();
    assert!(!hits.is_empty(), "expected hit for fn complete");
    let beta_path = src.join("beta.rs").to_string_lossy().to_string();
    assert!(
        hits.iter().any(|h| h.path == beta_path),
        "expected beta.rs in results, got: {:?}",
        hits.iter().map(|h| &h.path).collect::<Vec<_>>()
    );
}

// ---------------------------------------------------------------------------
// (c) Delete file, query unique symbol, verify empty results
// ---------------------------------------------------------------------------

#[test]
fn c_delete_file_query_returns_empty() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut idx = Bm25Index::new(tmp.path(), "test_c").unwrap();

    let code = "fn unique_symbol_xyz() {}\n";
    let file = write_file(tmp.path(), "deleteme.rs", code);

    let chunks = vec![BM25Chunk {
        path: file.clone(),
        content: code.to_string(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Function,
    }];
    idx.index_file(&file, Some(chunks)).unwrap();

    let pre = idx.query("unique_symbol_xyz", 10).unwrap();
    assert!(!pre.is_empty(), "should find symbol before delete");

    idx.delete_file(&file).unwrap();

    let post = idx.query("unique_symbol_xyz", 10).unwrap();
    assert!(post.is_empty(), "should be empty after delete");
}

// ---------------------------------------------------------------------------
// (d) Query non-existent term returns empty Vec (not error)
// ---------------------------------------------------------------------------

#[test]
fn d_query_nonexistent_returns_empty() {
    let tmp = tempfile::TempDir::new().unwrap();
    let idx = Bm25Index::new(tmp.path(), "test_d").unwrap();

    let hits = idx.query("nonexistent_term_zzzzz", 10).unwrap();
    assert!(hits.is_empty());
}

// ---------------------------------------------------------------------------
// (e) .xyz extension, chunks=None, query keyword, get result (line-level fallback)
// ---------------------------------------------------------------------------

#[test]
fn e_unknown_extension_fallback() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut idx = Bm25Index::new(tmp.path(), "test_e").unwrap();

    let content = "some_unique_keyword_for_xyz in this file\n";
    let file = write_file(tmp.path(), "data.xyz", content);

    let n = idx.index_file(&file, None).unwrap();
    assert!(n > 0, "should index .xyz file with chunks=None");

    let hits = idx.query("some_unique_keyword_for_xyz", 10).unwrap();
    assert!(
        !hits.is_empty(),
        "should find keyword in .xyz file (line-level fallback)"
    );
}

// ---------------------------------------------------------------------------
// (f) 1 text + 1 binary file. Assert files_indexed=1, files_skipped_binary=1.
// ---------------------------------------------------------------------------

#[test]
fn f_binary_file_skipped() {
    let tmp = tempfile::TempDir::new().unwrap();
    let src = tmp.path().join("mixed");
    std::fs::create_dir_all(&src).unwrap();

    write_file(&src, "good.txt", "this is a text file with searchable content\n");

    // Binary file: embed null bytes in first 8192 bytes
    let mut bin_data = b"binary\x00content\x00here".to_vec();
    bin_data.resize(256, 0u8);
    let bin_path = src.join("bad.bin");
    std::fs::write(&bin_path, &bin_data).unwrap();

    let mut idx = Bm25Index::new(tmp.path(), "test_f").unwrap();
    let stats = idx.index_directory(&src).unwrap();

    assert_eq!(stats.files_indexed, 1, "only text file should be indexed");
    assert_eq!(
        stats.files_skipped_binary, 1,
        "binary file should be skipped"
    );
}

// ---------------------------------------------------------------------------
// (g) update_file: old content gone, new content found
// ---------------------------------------------------------------------------

#[test]
fn g_update_file_replaces_content() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut idx = Bm25Index::new(tmp.path(), "test_g").unwrap();

    let file = write_file(tmp.path(), "evolving.rs", "fn old_function() {}\n");

    let old_chunks = vec![BM25Chunk {
        path: file.clone(),
        content: "fn old_function() {}".to_string(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Function,
    }];
    idx.index_file(&file, Some(old_chunks)).unwrap();

    // Verify old content is findable
    let pre = idx.query("old_function", 10).unwrap();
    assert!(!pre.is_empty(), "old_function should be found before update");

    // Update with new content
    std::fs::write(&file, "fn new_function() {}\n").unwrap();
    let new_chunks = vec![BM25Chunk {
        path: file.clone(),
        content: "fn new_function() {}".to_string(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Function,
    }];
    idx.update_file(&file, Some(new_chunks)).unwrap();

    let new_hits = idx.query("new_function", 10).unwrap();
    assert!(
        !new_hits.is_empty(),
        "new_function should be found after update"
    );

    let old_hits = idx.query("old_function", 10).unwrap();
    assert!(
        old_hits.is_empty(),
        "old_function should NOT be found after update"
    );
}

// ---------------------------------------------------------------------------
// (h) Two indexes with different project_id, no cross-contamination
// ---------------------------------------------------------------------------

#[test]
fn h_project_isolation() {
    let tmp_a = tempfile::TempDir::new().unwrap();
    let tmp_b = tempfile::TempDir::new().unwrap();

    let mut idx_a = Bm25Index::new(tmp_a.path(), "project_a").unwrap();
    let mut idx_b = Bm25Index::new(tmp_b.path(), "project_b").unwrap();

    let file_a = write_file(tmp_a.path(), "a.rs", "fn alpha_only_in_a() {}\n");
    let file_b = write_file(tmp_b.path(), "b.rs", "fn beta_only_in_b() {}\n");

    let chunks_a = vec![BM25Chunk {
        path: file_a.clone(),
        content: "fn alpha_only_in_a() {}".to_string(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Function,
    }];
    let chunks_b = vec![BM25Chunk {
        path: file_b.clone(),
        content: "fn beta_only_in_b() {}".to_string(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Function,
    }];

    idx_a.index_file(&file_a, Some(chunks_a)).unwrap();
    idx_b.index_file(&file_b, Some(chunks_b)).unwrap();

    // Project A should NOT see project B content
    let a_sees_beta = idx_a.query("beta_only_in_b", 10).unwrap();
    assert!(
        a_sees_beta.is_empty(),
        "project A must not see project B content"
    );

    // Project B should NOT see project A content
    let b_sees_alpha = idx_b.query("alpha_only_in_a", 10).unwrap();
    assert!(
        b_sees_alpha.is_empty(),
        "project B must not see project A content"
    );
}

// ---------------------------------------------------------------------------
// (i) ChunkKind: Function via explicit chunks; LineBlock via chunks=None on 120-line file
// ---------------------------------------------------------------------------

#[test]
fn i_chunk_kind_function_and_lineblock() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut idx = Bm25Index::new(tmp.path(), "test_i").unwrap();

    // Part 1: explicit Function chunk
    let file1 = write_file(tmp.path(), "funcs.rs", "fn explicit_func() {}\n");
    let func_chunks = vec![BM25Chunk {
        path: file1.clone(),
        content: "fn explicit_func() {}".to_string(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Function,
    }];
    idx.index_file(&file1, Some(func_chunks)).unwrap();

    let hits = idx.query("explicit_func", 10).unwrap();
    assert!(!hits.is_empty());
    assert_eq!(
        hits[0].chunk_kind,
        ChunkKind::Function,
        "explicit Function chunk should have chunk_kind=Function"
    );

    // Part 2: 120-line file with chunks=None -> LineBlock chunks
    let mut lines = String::new();
    for i in 1..=120 {
        lines.push_str(&format!("// line {} unique_lineblock_marker\n", i));
    }
    let file2 = write_file(tmp.path(), "big.rs", &lines);
    idx.index_file(&file2, None).unwrap();

    let lb_hits = idx.query("unique_lineblock_marker", 10).unwrap();
    assert!(!lb_hits.is_empty(), "should find content in 120-line file");
    // chunk_file produces LineBlock chunks from sliding window + one File chunk.
    // The top hit for a term appearing in every line should be a LineBlock or File chunk.
    let has_lineblock = lb_hits
        .iter()
        .any(|h| h.chunk_kind == ChunkKind::LineBlock);
    assert!(
        has_lineblock,
        "120-line file with chunks=None should produce LineBlock hits"
    );
}

// ---------------------------------------------------------------------------
// (j) Delete precision: delete one nested path, other nested file survives
// ---------------------------------------------------------------------------

#[test]
fn j_delete_precision_nested_paths() {
    let tmp = tempfile::TempDir::new().unwrap();
    let mut idx = Bm25Index::new(tmp.path(), "test_j").unwrap();

    // Index "src/deep/nested/module.rs" with unique content
    let module_path = PathBuf::from("src/deep/nested/module.rs");
    let module_content = "fn module_unique_canary() {}\n";
    let module_chunks = vec![BM25Chunk {
        path: module_path.clone(),
        content: module_content.to_string(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Function,
    }];
    idx.index_file(&module_path, Some(module_chunks)).unwrap();

    // Index "src/deep/nested/other.rs" with different unique content
    let other_path = PathBuf::from("src/deep/nested/other.rs");
    let other_content = "fn other_unique_sentinel() {}\n";
    let other_chunks = vec![BM25Chunk {
        path: other_path.clone(),
        content: other_content.to_string(),
        line_start: 1,
        line_end: 1,
        kind: ChunkKind::Function,
    }];
    idx.index_file(&other_path, Some(other_chunks)).unwrap();

    // Verify both are findable
    let pre_module = idx.query("module_unique_canary", 10).unwrap();
    assert!(!pre_module.is_empty(), "module.rs content should be indexed");
    let pre_other = idx.query("other_unique_sentinel", 10).unwrap();
    assert!(!pre_other.is_empty(), "other.rs content should be indexed");

    // Delete only module.rs
    idx.delete_file(&module_path).unwrap();

    // module.rs content should be gone
    let post_module = idx.query("module_unique_canary", 10).unwrap();
    assert!(
        post_module.is_empty(),
        "module.rs content should be gone after delete"
    );

    // other.rs content must survive (delete precision)
    let post_other = idx.query("other_unique_sentinel", 10).unwrap();
    assert!(
        !post_other.is_empty(),
        "other.rs must NOT be deleted — guards against tokenized path matching"
    );
}

// ---------------------------------------------------------------------------
// (k) Hidden directories skipped during index_directory walk
// ---------------------------------------------------------------------------

#[test]
fn k_hidden_dir_skipped() {
    let tmp = tempfile::TempDir::new().unwrap();
    let src = tmp.path().join("project");
    std::fs::create_dir_all(&src).unwrap();

    // Visible file — should be indexed
    write_file(&src, "visible.rs", "fn visible_canary_symbol() {}\n");

    // Hidden directory with a file — should be skipped entirely
    let hidden = src.join(".hidden");
    std::fs::create_dir_all(&hidden).unwrap();
    write_file(&hidden, "secret.rs", "fn hidden_canary_symbol() {}\n");

    // .git-like directory — should also be skipped
    let dotgit = src.join(".git");
    std::fs::create_dir_all(&dotgit).unwrap();
    write_file(&dotgit, "config", "fn dotgit_canary_symbol() {}\n");

    // .elai_cc directory — belt-and-suspenders skip
    let elai = src.join(".elai_cc");
    std::fs::create_dir_all(&elai).unwrap();
    write_file(&elai, "index.dat", "fn elai_cc_canary_symbol() {}\n");

    let mut idx = Bm25Index::new(tmp.path(), "test_k").unwrap();
    let stats = idx.index_directory(&src).unwrap();

    assert_eq!(stats.files_indexed, 1, "only visible.rs should be indexed");
    assert_eq!(stats.files_skipped_binary, 0, "no binary files present");

    // Visible content found
    let visible_hits = idx.query("visible_canary_symbol", 10).unwrap();
    assert!(
        !visible_hits.is_empty(),
        "visible.rs content should be searchable"
    );

    // Hidden content NOT found
    let hidden_hits = idx.query("hidden_canary_symbol", 10).unwrap();
    assert!(
        hidden_hits.is_empty(),
        ".hidden/ content must be skipped"
    );

    let git_hits = idx.query("dotgit_canary_symbol", 10).unwrap();
    assert!(
        git_hits.is_empty(),
        ".git/ content must be skipped"
    );

    let elai_hits = idx.query("elai_cc_canary_symbol", 10).unwrap();
    assert!(
        elai_hits.is_empty(),
        ".elai_cc/ content must be skipped"
    );
}

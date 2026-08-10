//! J-18 LEG2 contract: the gated 6th `lsp_channel` RRF list is wired into
//! the real QueryPipeline stage list and remains off without its flag.

use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

use elai_knowledge::arag::{AragConfig, AragOrchestrator};
use elai_knowledge::bm25::Bm25Index;
use elai_knowledge::catrag::{CatRag, CatRagConfig};
use elai_knowledge::lsp_channel::{LspChannelAdapter, LspChannelError, LspChannelHit, ELAI_LSP_CHANNEL_RRF_ENABLE_ENV};
use elai_knowledge::pipeline::QueryPipeline;

struct FakeLspAdapter {
    calls: Arc<AtomicUsize>,
}

impl LspChannelAdapter for FakeLspAdapter {
    fn workspace_symbol(
        &self,
        _query: &str,
        _project_root: &std::path::Path,
        _limit: usize,
    ) -> Result<Vec<LspChannelHit>, LspChannelError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(vec![LspChannelHit {
            symbol: "fn parse_manifest".into(),
            path: "src/manifest.rs".into(),
            line: 42,
        }])
    }
}

#[test]
fn lsp_channel_rrf_stage_gated_and_wired() {
    let tmp = tempfile::TempDir::new().expect("CONTRACT: temp workspace MUST be created");
    let calls = Arc::new(AtomicUsize::new(0));
    let adapter = Arc::new(FakeLspAdapter { calls: Arc::clone(&calls) });
    let bm25 = Bm25Index::new(tmp.path(), "j18_on").expect("CONTRACT: BM25 fixture MUST initialize");
    let catrag = CatRag::new(CatRagConfig::default());
    let arag = AragOrchestrator::new(AragConfig::default());
    std::env::set_var(ELAI_LSP_CHANNEL_RRF_ENABLE_ENV, "1");
    let stages = QueryPipeline::new(bm25, catrag, arag)
        .with_lsp_channel_adapter(adapter)
        .query("find all imports")
        .expect("CONTRACT: enabled pipeline query MUST succeed");
    let lsp = stages.iter().find(|s| s.stage == "lsp_channel").expect("CONTRACT: enabled gate MUST add lsp_channel stage");
    assert_eq!(lsp.results.len(), 1, "CONTRACT: fake LSP hit MUST reach the 6th RRF stage");
    assert_eq!(lsp.results[0].chunk_id, "src/manifest.rs:42", "CONTRACT: LSP hit chunk_id MUST preserve path and line");
    assert_eq!(calls.load(Ordering::SeqCst), 1, "CONTRACT: enabled LSP adapter MUST be called exactly once");

    std::env::remove_var(ELAI_LSP_CHANNEL_RRF_ENABLE_ENV);
    let tmp_off = tempfile::TempDir::new().expect("CONTRACT: second temp workspace MUST be created");
    let off = QueryPipeline::new(
        Bm25Index::new(tmp_off.path(), "j18_off").expect("CONTRACT: second BM25 fixture MUST initialize"),
        CatRag::new(CatRagConfig::default()),
        AragOrchestrator::new(AragConfig::default()),
    ).with_lsp_channel_adapter(Arc::new(FakeLspAdapter { calls: Arc::clone(&calls) }))
        .query("find all imports").expect("CONTRACT: disabled pipeline query MUST succeed");
    assert!(!off.iter().any(|s| s.stage == "lsp_channel"), "CONTRACT: disabled gate MUST omit lsp_channel stage");

    std::env::set_var(ELAI_LSP_CHANNEL_RRF_ENABLE_ENV, "1");
    let tmp_none = tempfile::TempDir::new().expect("CONTRACT: third temp workspace MUST be created");
    let no_adapter = QueryPipeline::new(
        Bm25Index::new(tmp_none.path(), "j18_none").expect("CONTRACT: third BM25 fixture MUST initialize"),
        CatRag::new(CatRagConfig::default()),
        AragOrchestrator::new(AragConfig::default()),
    ).query("find all imports").expect("CONTRACT: enabled no-adapter query MUST succeed");
    std::env::remove_var(ELAI_LSP_CHANNEL_RRF_ENABLE_ENV);
    assert!(!no_adapter.iter().any(|s| s.stage == "lsp_channel"), "CONTRACT: enabled gate without adapter MUST omit lsp_channel stage");
}

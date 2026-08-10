//! J-18 LEG2 slice 1: the 6th gated RRF list, `lsp_channel`.
//!
//! The trait is implemented from the cli crate because knowledge has no
//! orchestrator/cli dependency. The concrete rust-analyzer stdio client is
//! slice 2 and does not exist yet.

use std::path::Path;

use thiserror::Error;

/// Coarse errors returned by the workspace/symbol-shaped LSP seam.
#[derive(Debug, Error)]
pub enum LspChannelError {
    #[error("lsp_channel: client unavailable: {0}")]
    ClientUnavailable(String),
    #[error("lsp_channel: timeout: {0}")]
    Timeout(String),
    #[error("lsp_channel: invalid input: {0}")]
    InvalidInput(String),
    #[error("lsp_channel: internal: {0}")]
    Internal(String),
}

/// Adapter for a workspace/symbol-shaped request; no cursor position is
/// needed at this seam. The pipeline treats any error as an empty list
/// (best-effort; the RRF accumulator no-ops an empty stage).
pub trait LspChannelAdapter: Send + Sync {
    fn workspace_symbol(
        &self,
        query: &str,
        project_root: &Path,
        limit: usize,
    ) -> Result<Vec<LspChannelHit>, LspChannelError>;
}

/// One symbol returned by a workspace/symbol-shaped request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LspChannelHit {
    pub symbol: String,
    pub path: String,
    pub line: u32,
}

/// Gate for the optional 6th-list LSP channel RRF input.
pub const ELAI_LSP_CHANNEL_RRF_ENABLE_ENV: &str = "ELAI_LSP_CHANNEL_RRF_ENABLE";

/// Return true when the LSP channel gate is set to 1, true, or on.
pub fn lsp_channel_rrf_enabled_from_env() -> bool {
    match std::env::var(ELAI_LSP_CHANNEL_RRF_ENABLE_ENV) {
        Ok(v) => {
            let t = v.trim();
            t == "1" || t.eq_ignore_ascii_case("true") || t.eq_ignore_ascii_case("on")
        }
        Err(_) => false,
    }
}

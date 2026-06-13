//! I-14: deterministic context compression middleware (CACHE-INVARIANT ECONOMICS).
//!
//! Tier-1 (deterministic, ships now): two transforms that shrink large tool-result
//! payloads BEFORE they enter the agent's context window, journaling every transform
//! as a reversible [`crate::ssb::ContextTransformReceiptPayload`] SSB row (ID-3
//! JOURNALED-MIDDLEWARE doctrine; an un-journaled transform is BANNED).
//!
//!   * [`AstFoldTransform`] — tree-sitter function-body elision. Bodies of functions
//!     collapse to `/* elided */`; signatures and type information stay intact so the
//!     BPE semantic priors the model relies on are preserved (IC-2/IC-3). Composes the
//!     shipped `elai_knowledge::ast_adapter::TreeSitterAstEvaluator` (Rust grammar
//!     built in); never re-implements a parser.
//!   * [`LspSliceTransform`] — compiler-error slicing. A `rustc` panic already carries
//!     a `--> path:line:col` location; the transform parses that location out of the
//!     tool-output TEXT ITSELF (the `lsp_oracle` surface only probes toolchain
//!     presence — `OracleAnswer` carries no location data — so location extraction is
//!     done here, never delegated to the oracle) and slices a window around it.
//!
//! Tier-2 (neural, LLMLingua-2-class) is DEMOTED and DEFERRED (IC-2): it costs VRAM,
//! is gated behind a VRAM budget check + explicit user approval, and is prose-only. No
//! Tier-2 code ships in this changeset; [`ContextTransformKind::Tier2Compressed`]
//! exists in the receipt enum as the reserved discriminator only.
//!
//! Determinism is the whole point (GD-2): every transform is a pure function of its
//! input bytes. The same input produces byte-identical output and an identical
//! `reverse_hash` across runs. No `HashMap` (nondeterministic iteration order); any
//! ordered structure is a `BTreeMap` keyed by byte/line offset.
//!
//! HD-2 invariant: the transform runs at the tool-result -> context-assembly byte
//! seam (`ipc_accept.rs`, before the bytes are assembled into a context slot). Tool
//! output is ALWAYS volatile-tail content; the blake3-frozen prefix slots
//! (SystemPrompt / Memory / CodeContext) are populated from other sources and are
//! never touched here, so the prefix-stability invariant is satisfied structurally.

use std::collections::BTreeMap;
use std::sync::Arc;

use elai_core::clock::ClockProvider;
use elai_rules::predicate::AstEvaluator;
use elai_knowledge::ast_adapter::TreeSitterAstEvaluator;

use crate::ssb::{ContextTransformKind, ContextTransformReceiptPayload, Ssb, SsbPayload};
use crate::symbol_graph::lsp_oracle::oracle_lookup;

/// Rough byte-to-token ratio for English/code under the cl100k-style BPE family.
/// Used only for the receipt's `before_tokens` / `after_tokens` advisory counts; the
/// transform never tokenizes (no tokenizer dependency, fully deterministic).
const BYTES_PER_TOKEN: usize = 4;

/// Number of source lines kept on EACH side of a located compiler error when
/// [`LspSliceTransform`] slices a `rustc` panic. The full panic (often hundreds of
/// backtrace frames) collapses to this neighbourhood plus the error header.
const LSP_SLICE_CONTEXT_LINES: usize = 12;

/// Marker substituted for an elided function body by [`AstFoldTransform`]. The
/// signature line and the closing brace structure are preserved around it.
const ELIDED_BODY_MARKER: &str = "{ /* elided */ }";

/// Tree-sitter S-expression query capturing the `block` body of every Rust
/// `function_item`. The captured node's line span is the body we elide.
const RUST_FN_BODY_QUERY: &str = "(function_item body: (block) @body)";

/// Estimate token count from byte length (deterministic, tokenizer-free).
#[must_use]
fn est_tokens(byte_len: usize) -> u32 {
    u32::try_from(byte_len / BYTES_PER_TOKEN).unwrap_or(u32::MAX)
}

/// Output of a single Tier-1 transform strategy: the (possibly unchanged) bytes plus
/// whether the strategy actually fired. `skipped == true` means the strategy degraded
/// gracefully (toolchain absent, non-UTF-8 input, nothing to fold) and returned the
/// input verbatim; the caller still emits a receipt (ID-3) recording the skip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformOutput {
    /// The transformed bytes (identical to input when `skipped`).
    pub bytes: Vec<u8>,
    /// True when the strategy could not run and returned the input unchanged.
    pub skipped: bool,
}

/// Full result of [`ContextTransformer::transform`]: the bytes that will enter the
/// context window plus every field the [`ContextTransformReceiptPayload`] needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformResult {
    /// Bytes that will enter the agent context window.
    pub bytes: Vec<u8>,
    /// Token estimate of the ORIGINAL input.
    pub before_tokens: u32,
    /// Token estimate of `bytes`.
    pub after_tokens: u32,
    /// blake3 of the ORIGINAL input bytes (full 32-byte digest). The receipt's
    /// reverse pointer: replay reconstructs what the model saw by matching this
    /// against the journaled original. NOT the 8-byte prefix `update_breakpoints`
    /// uses for cache-key stability — a different hash with a different purpose.
    pub reverse_hash: [u8; 32],
    /// Which strategy produced this result.
    pub kind: ContextTransformKind,
    /// True when the chosen strategy degraded gracefully (input returned verbatim).
    pub skipped: bool,
}

/// Error surface for the transform pipeline. Kept narrow: the pipeline is
/// fail-soft (a strategy that cannot run returns `skipped`, never an error), so the
/// only hard error is an SSB write failure surfaced to the caller for logging.
#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    /// The ID-3 receipt could not be journaled. The transform itself succeeded; the
    /// caller decides whether a missing receipt is fatal (it is, per ID-3, for the
    /// done-gate — but at runtime we surface it rather than panic).
    #[error("context-transform receipt SSB write failed: {0}")]
    ReceiptWrite(String),
}

/// Tier-1 strategy: tree-sitter function-body elision.
///
/// Folds the body of every Rust `function_item` to `{ /* elided */ }`, preserving the
/// signature line. Pure and deterministic: matches are ordered by start line in a
/// `BTreeMap` and the source is rebuilt line-by-line, so identical input yields
/// byte-identical output.
#[derive(Debug, Default)]
pub struct AstFoldTransform;

impl AstFoldTransform {
    /// Fold function bodies in `content`, interpreting it as `language` source.
    ///
    /// Degrades gracefully (returns input verbatim, `skipped: true`) when: the bytes
    /// are not valid UTF-8, the language/query is unsupported, or there are no bodies
    /// to fold. Never panics, never `.unwrap()`s.
    #[must_use]
    pub fn apply(&self, content: &[u8], language: &str) -> TransformOutput {
        let source = match std::str::from_utf8(content) {
            Ok(s) => s,
            // Binary or partial-multibyte tool output: cannot AST-fold, return verbatim.
            Err(_) => {
                return TransformOutput {
                    bytes: content.to_vec(),
                    skipped: true,
                }
            }
        };

        let evaluator = TreeSitterAstEvaluator::new();
        let matches = match evaluator.evaluate_query(language, source, RUST_FN_BODY_QUERY) {
            Ok(m) => m,
            // Unsupported language or query parse failure: degrade, do not error.
            Err(_) => {
                return TransformOutput {
                    bytes: content.to_vec(),
                    skipped: true,
                }
            }
        };

        if matches.is_empty() {
            return TransformOutput {
                bytes: content.to_vec(),
                skipped: true,
            };
        }

        // Deterministic ordering: a BTreeMap keyed by start line. If tree-sitter ever
        // returned two bodies starting on the same line (it does not for valid Rust),
        // the later end-line wins — a stable, input-determined rule.
        let mut body_spans: BTreeMap<u32, u32> = BTreeMap::new();
        for m in &matches {
            let entry = body_spans.entry(m.start_line).or_insert(m.end_line);
            if m.end_line > *entry {
                *entry = m.end_line;
            }
        }

        // Rebuild the source line by line. A line that begins a body span is replaced
        // (together with the lines through its end span) by the signature prefix on
        // that first line plus the elision marker. `start_line`/`end_line` are
        // 0-based rows from tree-sitter.
        let lines: Vec<&str> = source.split('\n').collect();
        let mut out = String::with_capacity(source.len());
        let mut row: u32 = 0;
        let total = u32::try_from(lines.len()).unwrap_or(u32::MAX);
        let mut first_out_line = true;
        while row < total {
            let line = lines.get(row as usize).copied().unwrap_or("");
            if let Some(&end) = body_spans.get(&row) {
                // The body opens on `line`. Keep everything up to and including the
                // opening brace, then the elision marker. The lines through `end`
                // (the body interior + closing brace) are consumed.
                let prefix = match line.find('{') {
                    Some(idx) => &line[..idx],
                    None => line,
                };
                if !first_out_line {
                    out.push('\n');
                }
                out.push_str(prefix.trim_end());
                out.push(' ');
                out.push_str(ELIDED_BODY_MARKER);
                first_out_line = false;
                row = end.saturating_add(1);
            } else {
                if !first_out_line {
                    out.push('\n');
                }
                out.push_str(line);
                first_out_line = false;
                row = row.saturating_add(1);
            }
        }

        TransformOutput {
            bytes: out.into_bytes(),
            skipped: false,
        }
    }
}

/// Tier-1 strategy: compiler-error slicing.
///
/// Detects a `rustc`/cargo panic in the tool output, finds the FIRST `--> path:line:col`
/// location (deterministic: first by byte offset), and slices a fixed window of lines
/// around the error header, dropping the long backtrace tail. The toolchain presence
/// is probed via the shipped [`oracle_lookup`] surface; when the toolchain is absent
/// (`OracleAnswer.invoked == false`, the RUNPOD-PENDING path) the transform degrades to
/// `skipped: true` and returns the input verbatim (it never panics or errors).
#[derive(Debug, Default)]
pub struct LspSliceTransform;

impl LspSliceTransform {
    /// Slice a compiler-error payload. Async because toolchain presence detection goes
    /// through the async `oracle_lookup`.
    pub async fn apply(&self, content: &[u8]) -> TransformOutput {
        let source = match std::str::from_utf8(content) {
            Ok(s) => s,
            Err(_) => {
                return TransformOutput {
                    bytes: content.to_vec(),
                    skipped: true,
                }
            }
        };

        // Toolchain presence gate. `oracle_lookup` returns Ok(invoked:false) (NOT Err)
        // when rustc/rust-analyzer is absent on PATH; an Err is an unexpected-language
        // / empty-symbol case which we also treat as "cannot slice -> skip".
        let toolchain_present = match oracle_lookup("main", "rust").await {
            Ok(answer) => answer.invoked,
            Err(_) => false,
        };
        if !toolchain_present {
            return TransformOutput {
                bytes: content.to_vec(),
                skipped: true,
            };
        }

        // Locate the first compiler-error arrow line "--> <path>:<line>:<col>".
        // Deterministic: scan lines in order, take the first hit. The slice is the
        // window of LSP_SLICE_CONTEXT_LINES lines on each side of the arrow line.
        let lines: Vec<&str> = source.split('\n').collect();
        let arrow_row = lines.iter().position(|l| l.trim_start().starts_with("--> "));
        let Some(arrow_row) = arrow_row else {
            // No locatable error in the payload: nothing to slice deterministically.
            return TransformOutput {
                bytes: content.to_vec(),
                skipped: true,
            };
        };

        let start = arrow_row.saturating_sub(LSP_SLICE_CONTEXT_LINES);
        let end = arrow_row
            .saturating_add(LSP_SLICE_CONTEXT_LINES)
            .min(lines.len().saturating_sub(1));
        let mut out = String::new();
        for (i, line) in lines.iter().enumerate().take(end + 1).skip(start) {
            out.push_str(line);
            if i < end {
                out.push('\n');
            }
        }

        TransformOutput {
            bytes: out.into_bytes(),
            skipped: false,
        }
    }
}

/// The Tier-1 context-compression middleware dispatcher.
///
/// Constructed once per session at the production seam (`ipc_accept.rs` via the
/// `AgentRpcServer::transform_tool_output` override) with the live [`Ssb`] handle and
/// the [`crate::core::ClockProvider`] used for the receipt timestamp. Holds no neural
/// state — Tier-2 is deferred.
pub struct ContextTransformer {
    ssb: Arc<Ssb>,
    clock: Arc<dyn ClockProvider>,
}

impl ContextTransformer {
    /// Build a transformer bound to the live SSB writer and clock.
    #[must_use]
    pub fn new(ssb: Arc<Ssb>, clock: Arc<dyn ClockProvider>) -> Self {
        Self { ssb, clock }
    }

    /// Choose a Tier-1 strategy from the SHAPE of the bytes (deterministic):
    ///   * a `rustc` panic (contains a `--> ` location or an `error[E...]` header)
    ///     routes to [`LspSliceTransform`];
    ///   * otherwise Rust-looking source (contains `fn ` and a brace) routes to
    ///     [`AstFoldTransform`];
    ///   * anything else is left for [`AstFoldTransform`] which will skip on no match.
    fn classify(content: &[u8]) -> ContextTransformKind {
        let text = match std::str::from_utf8(content) {
            Ok(s) => s,
            Err(_) => return ContextTransformKind::AstFold,
        };
        if text.contains("--> ") || text.contains("error[E") {
            ContextTransformKind::LspSlice
        } else {
            ContextTransformKind::AstFold
        }
    }

    /// Run the Tier-1 pipeline on a tool-result payload and journal a receipt.
    ///
    /// Returns the (possibly unchanged) bytes that should enter the context window.
    /// ALWAYS emits exactly one [`crate::ssb::SsbActionType::ContextTransformReceipt`]
    /// SSB row (ID-3: a transform that ran without a receipt is a done-gate FAIL). An
    /// SSB write failure surfaces as [`TransformError::ReceiptWrite`]; the caller logs
    /// it but still uses the returned bytes (runtime fail-soft).
    pub async fn transform(
        &self,
        content: &[u8],
        superstep: i64,
    ) -> Result<TransformResult, TransformError> {
        let before_tokens = est_tokens(content.len());
        // reverse_hash is the full 32-byte blake3 of the ORIGINAL bytes (blake3 only,
        // never blake2/sha2 — Anti-Stale-Training HARD RULE).
        let reverse_hash: [u8; 32] = *blake3::hash(content).as_bytes();

        let kind = Self::classify(content);
        let output = match kind {
            ContextTransformKind::LspSlice => LspSliceTransform.apply(content).await,
            // AstFold is Rust-only for the Tier-1 ship; other grammars are a deferred
            // follow-on (the evaluator has them registered but body-fold queries per
            // language are unauthored).
            ContextTransformKind::AstFold => AstFoldTransform.apply(content, "rust"),
            // Tier-2 is deferred; if it is ever selected before the encoder lands the
            // safe behaviour is an identity skip (never a silent neural call).
            ContextTransformKind::Tier2Compressed => TransformOutput {
                bytes: content.to_vec(),
                skipped: true,
            },
        };

        let after_tokens = est_tokens(output.bytes.len());

        // ID-3 JOURNALED-MIDDLEWARE: emit the reversible receipt for EVERY transform,
        // including skips (the skip is itself an auditable event).
        let ts_ms = Ssb::now_ms_via_clock(&self.clock);
        let payload = SsbPayload::ContextTransformReceipt(ContextTransformReceiptPayload {
            kind,
            before_tokens,
            after_tokens,
            reverse_hash: hex_encode(&reverse_hash),
            skipped: output.skipped,
            ts_ms,
        });
        self.ssb
            .write_payload("context_transform", superstep, payload, Vec::new())
            .await
            .map_err(|e| TransformError::ReceiptWrite(e.to_string()))?;

        Ok(TransformResult {
            bytes: output.bytes,
            before_tokens,
            after_tokens,
            reverse_hash,
            kind,
            skipped: output.skipped,
        })
    }
}

/// Lowercase hex-encode a 32-byte digest without pulling a hex crate (deterministic).
#[must_use]
fn hex_encode(bytes: &[u8; 32]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(64);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0x0f) as usize] as char);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ast_fold_elides_rust_body_and_keeps_signature() {
        let src = b"fn add(a: i32, b: i32) -> i32 {\n    let s = a + b;\n    s\n}\n";
        let out = AstFoldTransform.apply(src, "rust");
        assert!(!out.skipped, "valid Rust with a body must fold, not skip");
        let folded = String::from_utf8(out.bytes).expect("utf8");
        assert!(
            folded.contains("fn add(a: i32, b: i32) -> i32"),
            "signature must be preserved, got: {folded}"
        );
        assert!(
            folded.contains("/* elided */"),
            "body must be elided, got: {folded}"
        );
        assert!(
            !folded.contains("let s = a + b"),
            "body interior must be gone, got: {folded}"
        );
    }

    #[test]
    fn ast_fold_skips_non_utf8() {
        let out = AstFoldTransform.apply(&[0xff, 0xfe, 0x00], "rust");
        assert!(out.skipped, "non-utf8 must skip");
        assert_eq!(out.bytes, vec![0xff, 0xfe, 0x00], "skip returns input verbatim");
    }

    #[test]
    fn ast_fold_is_deterministic() {
        let src = b"fn a() {\n    let x = 1;\n}\nfn b() {\n    let y = 2;\n}\n";
        let first = AstFoldTransform.apply(src, "rust");
        let second = AstFoldTransform.apply(src, "rust");
        assert_eq!(first, second, "same input must produce byte-identical output");
    }

    #[test]
    fn hex_encode_round_trips_known_value() {
        let bytes = [0u8; 32];
        assert_eq!(hex_encode(&bytes), "0".repeat(64));
        let mut b = [0u8; 32];
        b[0] = 0xab;
        b[31] = 0x0f;
        let h = hex_encode(&b);
        assert!(h.starts_with("ab"), "got {h}");
        assert!(h.ends_with("0f"), "got {h}");
    }

    #[test]
    fn est_tokens_uses_named_ratio() {
        assert_eq!(est_tokens(40), 10, "40 bytes / 4 bytes-per-token = 10");
        assert_eq!(est_tokens(0), 0);
    }
}

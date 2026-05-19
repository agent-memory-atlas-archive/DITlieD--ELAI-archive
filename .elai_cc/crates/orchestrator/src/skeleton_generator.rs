//! C-32 — SkeletonGenerator: skeleton-first code generation with BLANK_N
//! placeholders and a single-write flush protocol.
//!
//! The skeleton path is the baby-step alternative to the legacy
//! "one LLM call per file" loop. The orchestrator pre-computes the file
//! shape (function signatures + body placeholders), the executor fills the
//! placeholders one at a time, and the result is committed to disk with
//! exactly one `fs::write` per file. Failed fills are reported via
//! [`MiniCriticVerdict::Block`] but never break the loop — the partial
//! buffer is recoverable through `AcceptAndComplete`.
//!
//! Spec anchors (C-32 plan §Task 1):
//!   - `SkeletonSpec`, `BlankId`, `FillResult`, `FilledBuffer`.
//!   - `generate(&FilePlan) -> Result<SkeletonSpec, SkeletonError>`.
//!   - `fill_blanks(...)` collects results in memory; zero disk writes.
//!   - `flush_to_disk(...)` exactly one `fs::write`; BLOCK blanks rendered
//!      as `// BLANK_N -- not filled`.
//!   - SSB rows: `skeleton_generated`, `skeleton_flushed`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use elai_core::file_plan::{FilePlan, FunctionSpec};

use crate::mini_critic::{MiniCritic, MiniCriticLanguage, MiniCriticResult, MiniCriticVerdict};
use crate::ssb::{
    SkeletonFlushedPayload, SkeletonGeneratedPayload, Ssb, SsbPayload,
};

// ---------------------------------------------------------------------------
// BlankId
// ---------------------------------------------------------------------------

/// Newtype around `u8` so the type system distinguishes blank indices from
/// arbitrary integers. Display is `BLANK_<n>` (1-based, matches the plan's
/// `// BLANK_1`, `// BLANK_2`, ... convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BlankId(pub u8);

impl BlankId {
    /// Lowercase string form used in SSB payloads (`blank_1`).
    pub fn as_label(self) -> String {
        format!("BLANK_{}", self.0)
    }
}

impl std::fmt::Display for BlankId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BLANK_{}", self.0)
    }
}

// ---------------------------------------------------------------------------
// SkeletonSpec, FillResult, FilledBuffer
// ---------------------------------------------------------------------------

/// Pre-computed skeleton for one file. Carries the function signatures with
/// `// BLANK_N` placeholders for the bodies. The orchestrator owns this
/// struct between `generate()` and `flush_to_disk()`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkeletonSpec {
    pub file_path: PathBuf,
    /// Lowercase language tag. Mirrors `FileSpec.language` exactly.
    pub language: String,
    pub blanks: Vec<BlankId>,
    pub skeleton_text: String,
    /// Resolved function signatures the skeleton renders. Stored so the
    /// fill prompt has direct access to the signature when it builds the
    /// per-blank prompt.
    pub functions: Vec<FunctionSpec>,
}

/// One filled blank — body string + the [`MiniCritic`] verdict observed
/// during the fill step. `body` may be empty when the verdict is `BLOCK`
/// (the structural retry sequence consumed the failed candidate).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FillResult {
    pub blank_id: BlankId,
    pub body: String,
    pub mini_critic_verdict: MiniCriticVerdict,
}

/// In-memory record of every fill attempt for a single skeleton. Owned by
/// the dispatch loop; rendered to disk by [`flush_to_disk`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilledBuffer {
    pub spec: SkeletonSpec,
    pub fills: Vec<FillResult>,
    pub completeness_ratio: f32,
}

impl FilledBuffer {
    /// `verdict != BLOCK` fills / total blanks, clamped to `[0.0, 1.0]`. A
    /// `FilledBuffer` with zero blanks returns `1.0` (vacuous pass, see
    /// C-32-decisions D8).
    pub fn compute_completeness_ratio(&self) -> f32 {
        if self.spec.blanks.is_empty() {
            return 1.0;
        }
        let non_block = self
            .fills
            .iter()
            .filter(|f| f.mini_critic_verdict != MiniCriticVerdict::Block)
            .count();
        let denom = self.spec.blanks.len() as f32;
        let ratio = non_block as f32 / denom;
        ratio.clamp(0.0, 1.0)
    }

    /// Convenience: refresh `completeness_ratio` from current fills.
    pub fn refresh_ratio(&mut self) {
        self.completeness_ratio = self.compute_completeness_ratio();
    }
}

// ---------------------------------------------------------------------------
// SkeletonGeneratorConfig
// ---------------------------------------------------------------------------

/// Configuration for [`SkeletonGenerator`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkeletonGeneratorConfig {
    /// Maximum blanks per skeleton. `generate()` returns `TooManyBlanks` when
    /// the FilePlan declares more functions than this. Default 32.
    pub max_blanks: u8,
    /// Minimum body lines required by [`MiniCritic`] check (b). Not enforced
    /// here directly (MiniCritic owns the check) but surfaced so callers
    /// can render diagnostics. Default 3.
    pub min_body_lines: u8,
}

impl Default for SkeletonGeneratorConfig {
    fn default() -> Self {
        Self {
            max_blanks: 32,
            min_body_lines: 3,
        }
    }
}

// ---------------------------------------------------------------------------
// SkeletonError
// ---------------------------------------------------------------------------

/// SkeletonGenerator errors. All fields are `String` (no `io::Error`) so the
/// type derives `Clone` per C-32-decisions D6. The original error message is
/// preserved verbatim in the `source` field.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SkeletonError {
    /// `generate()` would emit more than `max_blanks` placeholders.
    #[error("file '{file}' declares {got} functions; max_blanks={max}")]
    TooManyBlanks {
        file: PathBuf,
        max: u8,
        got: u8,
    },
    /// `flush_to_disk()` failed to write the skeleton.
    #[error("flush to '{path}' failed: {message}")]
    FlushFailed { path: PathBuf, message: String },
    /// `flush_to_disk()` could not find a `FillResult` for a declared blank.
    /// Should never happen if the buffer was produced by `fill_blanks` —
    /// surfaces only if a caller mutates the buffer in-place.
    #[error("no fill for {blank}")]
    NoFillForBlank { blank: BlankId },
}

// ---------------------------------------------------------------------------
// SkeletonGenerator
// ---------------------------------------------------------------------------

/// Stateless helper. The only state lives in [`SkeletonGeneratorConfig`].
pub struct SkeletonGenerator {
    pub config: SkeletonGeneratorConfig,
}

impl Default for SkeletonGenerator {
    fn default() -> Self {
        Self::new(SkeletonGeneratorConfig::default())
    }
}

impl SkeletonGenerator {
    pub fn new(config: SkeletonGeneratorConfig) -> Self {
        Self { config }
    }

    /// Build a [`SkeletonSpec`] for every function in the supplied `file_spec`.
    /// `file_path` and `language` are passed in directly so the generator
    /// works for both `FileSpec` (new files) and `FileModSpec` (modifications).
    pub fn generate_for_functions(
        &self,
        file_path: PathBuf,
        language: String,
        functions: &[FunctionSpec],
    ) -> Result<SkeletonSpec, SkeletonError> {
        if functions.len() > self.config.max_blanks as usize {
            return Err(SkeletonError::TooManyBlanks {
                file: file_path,
                max: self.config.max_blanks,
                got: u8::try_from(functions.len()).unwrap_or(u8::MAX),
            });
        }
        let mut blanks = Vec::with_capacity(functions.len());
        let mut buf = String::new();
        for (i, func) in functions.iter().enumerate() {
            let n = i + 1;
            let id = BlankId(u8::try_from(n).unwrap_or(u8::MAX));
            blanks.push(id);
            buf.push_str(&format!("// {} signature: {}\n", id, func.signature));
            buf.push_str(&func.signature);
            buf.push_str(" {\n");
            buf.push_str(&format!("    // {}\n", id));
            buf.push_str("}\n\n");
        }
        Ok(SkeletonSpec {
            file_path,
            language,
            blanks,
            skeleton_text: buf,
            functions: functions.to_vec(),
        })
    }

    /// Convenience: generate skeletons for every `FileSpec.functions` in the
    /// supplied [`FilePlan`]. Returns one `SkeletonSpec` per file with
    /// declared functions. Files declaring zero functions are skipped (no
    /// blanks needed -> no skeleton path).
    pub fn generate_from_file_plan(
        &self,
        plan: &FilePlan,
    ) -> Result<Vec<SkeletonSpec>, SkeletonError> {
        let mut out = Vec::new();
        for spec in &plan.new_files {
            if spec.functions.is_empty() {
                continue;
            }
            out.push(self.generate_for_functions(
                spec.path.clone(),
                spec.language.clone(),
                &spec.functions,
            )?);
        }
        for spec in &plan.modified_files {
            if spec.functions.is_empty() {
                continue;
            }
            out.push(self.generate_for_functions(
                spec.path.clone(),
                spec.language.clone(),
                &spec.functions,
            )?);
        }
        Ok(out)
    }

    /// Run `critic.check` for every (blank, body) pair and assemble a
    /// [`FilledBuffer`]. No disk writes — caller invokes [`flush_to_disk`]
    /// at the end of the fill loop.
    pub fn fill_blanks(
        &self,
        spec: SkeletonSpec,
        bodies: Vec<String>,
        critic: &MiniCritic,
    ) -> Result<FilledBuffer, SkeletonError> {
        let language = language_from_str(&spec.language);
        let mut fills = Vec::with_capacity(bodies.len());
        for (i, body) in bodies.iter().enumerate() {
            // Map by position; callers must hand bodies in blank order.
            let blank_id = spec
                .blanks
                .get(i)
                .copied()
                .ok_or(SkeletonError::NoFillForBlank { blank: BlankId(u8::MAX) })?;
            let verdict: MiniCriticResult = critic.check(blank_id, body, language);
            fills.push(FillResult {
                blank_id,
                body: body.clone(),
                mini_critic_verdict: verdict.verdict,
            });
        }
        let mut buf = FilledBuffer {
            spec,
            fills,
            completeness_ratio: 0.0,
        };
        buf.refresh_ratio();
        Ok(buf)
    }

    /// Substitute `// BLANK_N` lines with the accepted body. Blanks whose
    /// verdict is `BLOCK` are rendered as `// BLANK_N -- not filled`. Single
    /// `fs::write` per file (stub_ban).
    pub fn flush_to_disk(&self, buffer: &FilledBuffer) -> Result<PathBuf, SkeletonError> {
        let mut text = buffer.spec.skeleton_text.clone();
        for blank in &buffer.spec.blanks {
            let placeholder = format!("    // {}", blank);
            let fill = buffer
                .fills
                .iter()
                .find(|f| f.blank_id == *blank)
                .ok_or(SkeletonError::NoFillForBlank { blank: *blank })?;
            let replacement = match fill.mini_critic_verdict {
                MiniCriticVerdict::Block => {
                    format!("    // {} -- not filled", blank)
                }
                _ => indent_body(&fill.body, "    "),
            };
            // Replace the first occurrence only; placeholders are unique per
            // blank id.
            if let Some(pos) = text.find(&placeholder) {
                text.replace_range(pos..pos + placeholder.len(), &replacement);
            }
        }
        if let Some(parent) = buffer.spec.file_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| SkeletonError::FlushFailed {
                    path: buffer.spec.file_path.clone(),
                    message: e.to_string(),
                })?;
            }
        }
        fs::write(&buffer.spec.file_path, text.as_bytes()).map_err(|e| {
            SkeletonError::FlushFailed {
                path: buffer.spec.file_path.clone(),
                message: e.to_string(),
            }
        })?;
        Ok(buffer.spec.file_path.clone())
    }
}

fn indent_body(body: &str, prefix: &str) -> String {
    body.lines()
        .map(|l| format!("{prefix}{l}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Map a FileSpec.language string to the MiniCritic enum.
pub fn language_from_str(language: &str) -> MiniCriticLanguage {
    match language.to_ascii_lowercase().as_str() {
        "rust" | "rs" => MiniCriticLanguage::Rust,
        "python" | "py" => MiniCriticLanguage::Python,
        _ => MiniCriticLanguage::Other,
    }
}

// ---------------------------------------------------------------------------
// SSB emit helpers
// ---------------------------------------------------------------------------

/// Emit a `skeleton_generated` row. Errors are silently swallowed (the
/// caller's path must not depend on SSB write success).
pub async fn emit_skeleton_generated_ssb(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    spec: &SkeletonSpec,
) {
    let payload = SsbPayload::SkeletonGenerated(SkeletonGeneratedPayload {
        blank_count: u32::try_from(spec.blanks.len()).unwrap_or(u32::MAX),
        language: spec.language.clone(),
        file_path: spec.file_path.to_string_lossy().into_owned(),
    });
    let files = vec![spec.file_path.to_string_lossy().into_owned()];
    let _ = ssb
        .write_payload(agent_id, superstep, payload, files)
        .await;
}

/// Emit a `skeleton_flushed` row after a single fs::write. Errors are silently
/// swallowed.
pub async fn emit_skeleton_flushed_ssb(
    ssb: &Arc<Ssb>,
    agent_id: &str,
    superstep: i64,
    path: &Path,
    completeness_ratio: f32,
) {
    let payload = SsbPayload::SkeletonFlushed(SkeletonFlushedPayload {
        path: path.to_string_lossy().into_owned(),
        completeness_ratio,
    });
    let files = vec![path.to_string_lossy().into_owned()];
    let _ = ssb
        .write_payload(agent_id, superstep, payload, files)
        .await;
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn func(name: &str) -> FunctionSpec {
        FunctionSpec {
            name: name.into(),
            signature: format!("fn {name}()"),
            max_body_lines: 20,
        }
    }

    #[test]
    fn generate_produces_one_blank_per_function() {
        let gen = SkeletonGenerator::default();
        let spec = gen
            .generate_for_functions(
                PathBuf::from("a.rs"),
                "rust".into(),
                &[func("a"), func("b"), func("c")],
            )
            .expect("ok");
        assert_eq!(spec.blanks.len(), 3);
        assert_eq!(spec.blanks[0], BlankId(1));
        assert_eq!(spec.blanks[2], BlankId(3));
    }

    #[test]
    fn skeleton_text_contains_each_placeholder() {
        let gen = SkeletonGenerator::default();
        let spec = gen
            .generate_for_functions(
                PathBuf::from("a.rs"),
                "rust".into(),
                &[func("a"), func("b")],
            )
            .expect("ok");
        assert!(spec.skeleton_text.contains("// BLANK_1"));
        assert!(spec.skeleton_text.contains("// BLANK_2"));
    }

    #[test]
    fn too_many_blanks_rejected() {
        let mut funcs = Vec::new();
        for i in 0..40 {
            funcs.push(func(&format!("f{i}")));
        }
        let gen = SkeletonGenerator::default();
        let err = gen
            .generate_for_functions(PathBuf::from("big.rs"), "rust".into(), &funcs)
            .unwrap_err();
        assert!(matches!(err, SkeletonError::TooManyBlanks { .. }));
    }

    #[test]
    fn completeness_ratio_all_pass() {
        let gen = SkeletonGenerator::default();
        let critic = MiniCritic::without_tree_sitter();
        let spec = gen
            .generate_for_functions(
                PathBuf::from("a.rs"),
                "rust".into(),
                &[func("a"), func("b"), func("c"), func("d")],
            )
            .unwrap();
        let bodies = vec![
            "let x = 1;\nlet y = 2;\nx + y".into(),
            "let x = 1;\nlet y = 2;\nx + y".into(),
            "let x = 1;\nlet y = 2;\nx + y".into(),
            "let x = 1;\nlet y = 2;\nx + y".into(),
        ];
        let buf = gen.fill_blanks(spec, bodies, &critic).unwrap();
        assert_eq!(buf.completeness_ratio, 1.0);
    }

    #[test]
    fn completeness_ratio_partial() {
        let gen = SkeletonGenerator::default();
        let critic = MiniCritic::without_tree_sitter();
        let spec = gen
            .generate_for_functions(
                PathBuf::from("a.rs"),
                "rust".into(),
                &[func("a"), func("b"), func("c"), func("d")],
            )
            .unwrap();
        let bodies = vec![
            "let x = 1;\nlet y = 2;\nx + y".into(),
            "// TODO bad body".into(),
            "let x = 1;\nlet y = 2;\nx + y".into(),
            "// TODO bad body".into(),
        ];
        let buf = gen.fill_blanks(spec, bodies, &critic).unwrap();
        assert!((buf.completeness_ratio - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn completeness_ratio_zero_blanks_vacuous_pass() {
        let buf = FilledBuffer {
            spec: SkeletonSpec {
                file_path: PathBuf::from("empty.rs"),
                language: "rust".into(),
                blanks: vec![],
                skeleton_text: String::new(),
                functions: vec![],
            },
            fills: vec![],
            completeness_ratio: 0.0,
        };
        assert_eq!(buf.compute_completeness_ratio(), 1.0);
    }

    #[test]
    fn flush_writes_once_and_substitutes_bodies() {
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("a.rs");
        let gen = SkeletonGenerator::default();
        let critic = MiniCritic::without_tree_sitter();
        let spec = gen
            .generate_for_functions(
                target.clone(),
                "rust".into(),
                &[func("a"), func("b")],
            )
            .unwrap();
        let bodies = vec![
            "let x = 1;\nlet y = 2;\nx + y".into(),
            "let x = 3;\nlet y = 4;\nx + y".into(),
        ];
        let buf = gen.fill_blanks(spec, bodies, &critic).unwrap();
        let written = gen.flush_to_disk(&buf).unwrap();
        assert_eq!(written, target);
        let text = fs::read_to_string(&target).unwrap();
        assert!(text.contains("let x = 1"));
        assert!(text.contains("let x = 3"));
        // No leftover BLANK_ placeholders.
        assert!(!text.contains("    // BLANK_1\n"));
        assert!(!text.contains("    // BLANK_2\n"));
    }
}

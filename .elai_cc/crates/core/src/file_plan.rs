//! C-31 — Layer 1 FilePlan primitives for code-completeness enforcement.
//!
//! Carries per-file write intentions with LOC budgets so the orchestrator
//! can enforce size contracts before generation. Validated against
//! per-language hard-block thresholds plus a global function-count cap.
//!
//! Spec anchors (code-completeness-enforcement-design.txt):
//!   - L161-176: FILE_PLAN as the 9th BabyStepSpec block.
//!   - L796-863: FilePlan / FileSpec / FileModSpec / FunctionSpec types.
//!   - L80-91:   Per-language hard-block tier-2 table.

// B-15 HashMap ban: BTreeMap chosen over FxHashMap because LanguageLimitsMap
// is a pub type alias; BTreeMap preserves the std-only public surface and
// gives deterministic iteration ordering if a future caller serializes it
// (e.g. into a manifest TOML). Lookup-only paths (`.get`) pay an O(log n)
// vs O(1) tax but n is bounded by the language table (~5 entries).
use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Named constants — no magic numbers (CLAUDE.md "Anti-Stale-Training").
// ---------------------------------------------------------------------------

/// Global per-file function-count cap enforced by [`FilePlan::validate`].
///
/// Aligned with the T2 Check 3 slack downstream: any FilePlan that declares
/// more than this many functions per file is rejected pre-dispatch.
pub const MAX_FUNCTIONS_PER_FILE: u8 = 20;

/// Default LOC hard block when a language is not in the supplied limit map.
/// Matches the spec section 3.2 "default" row.
pub const DEFAULT_LANGUAGE_HARD_BLOCK: u16 = 400;

// ---------------------------------------------------------------------------
// Per-file function spec — sized to keep BabyStepSpec payloads small.
// ---------------------------------------------------------------------------

/// One function declared inside a FilePlan FileSpec / FileModSpec.
///
/// `max_body_lines` is a per-function LOC budget enforced downstream by the
/// T2 Check 3 guard (function-count + body-size). `u8` is intentional —
/// any single function exceeding 255 lines is already a CC violation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FunctionSpec {
    pub name: String,
    pub signature: String,
    pub max_body_lines: u8,
}

/// New-file spec — declared in the FilePlan.new_files vec.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileSpec {
    pub path: PathBuf,
    pub language: String,
    pub max_lines: u16,
    #[serde(default)]
    pub functions: Vec<FunctionSpec>,
}

/// Existing-file mod spec — declared in FilePlan.modified_files.
///
/// `max_added_lines` is the diff-bound; we do not bound the absolute file
/// size here (T1 evaluate_file_size still applies post-write).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileModSpec {
    pub path: PathBuf,
    pub language: String,
    pub max_added_lines: u16,
    #[serde(default)]
    pub functions: Vec<FunctionSpec>,
}

/// Per-baby-step file plan — the 11th BabyStepSpec field (9th per the spec
/// pre-intent_pairs / pre-B-28 naming, see context-package-C31 DRIFT-1).
///
/// `version` lets future revisions reject incompatible legacy plans without
/// touching SCHEMA_VERSION (the field uses serde(default)).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FilePlan {
    #[serde(default)]
    pub new_files: Vec<FileSpec>,
    #[serde(default)]
    pub modified_files: Vec<FileModSpec>,
    #[serde(default = "FilePlan::default_version")]
    pub version: u8,
}

impl FilePlan {
    /// Current FilePlan wire version. Bumped only on a breaking change to
    /// FileSpec / FileModSpec / FunctionSpec field shape.
    pub const CURRENT_VERSION: u8 = 1;

    fn default_version() -> u8 {
        Self::CURRENT_VERSION
    }

    /// Construct an empty FilePlan at the current wire version.
    pub fn empty() -> Self {
        Self {
            new_files: Vec::new(),
            modified_files: Vec::new(),
            version: Self::CURRENT_VERSION,
        }
    }
}

/// Per-language hard-block map keyed by lowercase language name
/// (`"rust"`, `"python"`, ...). Looked up by FilePlan::validate.
///
/// `BTreeMap` (not `HashMap`) per B-15 HashMap ban: deterministic key
/// iteration order if downstream serializes the map; lookup cost is O(log n)
/// against a ~5-entry language table which is negligible.
pub type LanguageLimitsMap = BTreeMap<String, u16>;

// ---------------------------------------------------------------------------
// Errors — thiserror, no anyhow, no .unwrap().
// ---------------------------------------------------------------------------

/// Errors returned by FilePlan validation and constructor helpers.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum FilePlanError {
    /// A FileSpec or FileModSpec declares a budget that exceeds the
    /// language's hard-block limit (Tier-2 table).
    #[error("file '{file}' LOC budget {got} exceeds language limit {limit}")]
    LineBudgetExceeded {
        file: PathBuf,
        limit: u16,
        got: u16,
    },
    /// A FileSpec / FileModSpec declares more functions than
    /// [`MAX_FUNCTIONS_PER_FILE`] permits.
    #[error("file '{file}' declares {got} functions, exceeds cap {limit}")]
    FunctionCountExceeded {
        file: PathBuf,
        limit: u8,
        got: u8,
    },
}

impl FilePlan {
    /// Validate every FileSpec / FileModSpec against `limits` and the
    /// global [`MAX_FUNCTIONS_PER_FILE`] cap.
    ///
    /// Returns `Err(FilePlanError::LineBudgetExceeded)` on the first file
    /// that overshoots its language hard-block. Returns
    /// `Err(FilePlanError::FunctionCountExceeded)` on the first file
    /// declaring more than [`MAX_FUNCTIONS_PER_FILE`] functions.
    pub fn validate(&self, limits: &LanguageLimitsMap) -> Result<(), FilePlanError> {
        for spec in &self.new_files {
            let limit = lookup_limit(limits, &spec.language);
            if spec.max_lines > limit {
                return Err(FilePlanError::LineBudgetExceeded {
                    file: spec.path.clone(),
                    limit,
                    got: spec.max_lines,
                });
            }
            check_function_count(&spec.path, spec.functions.len())?;
        }
        for spec in &self.modified_files {
            let limit = lookup_limit(limits, &spec.language);
            if spec.max_added_lines > limit {
                return Err(FilePlanError::LineBudgetExceeded {
                    file: spec.path.clone(),
                    limit,
                    got: spec.max_added_lines,
                });
            }
            check_function_count(&spec.path, spec.functions.len())?;
        }
        Ok(())
    }
}

#[inline]
fn lookup_limit(limits: &LanguageLimitsMap, language: &str) -> u16 {
    limits
        .get(language)
        .copied()
        .unwrap_or(DEFAULT_LANGUAGE_HARD_BLOCK)
}

fn check_function_count(file: &PathBuf, count: usize) -> Result<(), FilePlanError> {
    let cap = MAX_FUNCTIONS_PER_FILE;
    if count > cap as usize {
        // count fits a u8 because we only reach this branch when count > cap
        // (a u8) plus 1; if the FilePlan deserialized with >255 functions
        // it is still well-formed JSON, and we clamp by reporting `cap+1`
        // so the error type stays u8.
        let reported: u8 = count.try_into().unwrap_or(u8::MAX);
        return Err(FilePlanError::FunctionCountExceeded {
            file: file.clone(),
            limit: cap,
            got: reported,
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests — co-located.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn rust_limits() -> LanguageLimitsMap {
        let mut m = BTreeMap::new();
        m.insert("rust".to_string(), 400);
        m.insert("python".to_string(), 480);
        m
    }

    #[test]
    fn validate_accepts_within_limits() {
        let plan = FilePlan {
            new_files: vec![FileSpec {
                path: PathBuf::from("a.rs"),
                language: "rust".into(),
                max_lines: 100,
                functions: vec![],
            }],
            modified_files: vec![],
            version: FilePlan::CURRENT_VERSION,
        };
        assert!(plan.validate(&rust_limits()).is_ok());
    }

    #[test]
    fn validate_rejects_over_limit() {
        let plan = FilePlan {
            new_files: vec![FileSpec {
                path: PathBuf::from("big.rs"),
                language: "rust".into(),
                max_lines: 500,
                functions: vec![],
            }],
            modified_files: vec![],
            version: FilePlan::CURRENT_VERSION,
        };
        let err = plan.validate(&rust_limits()).unwrap_err();
        assert!(matches!(err, FilePlanError::LineBudgetExceeded { .. }));
    }

    #[test]
    fn validate_uses_default_when_language_absent() {
        let plan = FilePlan {
            new_files: vec![FileSpec {
                path: PathBuf::from("a.exotic"),
                language: "klingon".into(),
                max_lines: 401, // exceeds DEFAULT_LANGUAGE_HARD_BLOCK 400
                functions: vec![],
            }],
            modified_files: vec![],
            version: FilePlan::CURRENT_VERSION,
        };
        assert!(plan.validate(&BTreeMap::new()).is_err());
    }

    #[test]
    fn validate_rejects_over_function_count() {
        let mut funcs = Vec::new();
        for i in 0..(MAX_FUNCTIONS_PER_FILE + 5) {
            funcs.push(FunctionSpec {
                name: format!("f{i}"),
                signature: "fn() -> ()".into(),
                max_body_lines: 10,
            });
        }
        let plan = FilePlan {
            new_files: vec![FileSpec {
                path: PathBuf::from("crowded.rs"),
                language: "rust".into(),
                max_lines: 100,
                functions: funcs,
            }],
            modified_files: vec![],
            version: FilePlan::CURRENT_VERSION,
        };
        let err = plan.validate(&rust_limits()).unwrap_err();
        assert!(matches!(err, FilePlanError::FunctionCountExceeded { .. }));
    }

    #[test]
    fn validate_modified_files_respects_limit() {
        let plan = FilePlan {
            new_files: vec![],
            modified_files: vec![FileModSpec {
                path: PathBuf::from("mod.rs"),
                language: "rust".into(),
                max_added_lines: 500,
                functions: vec![],
            }],
            version: FilePlan::CURRENT_VERSION,
        };
        assert!(plan.validate(&rust_limits()).is_err());
    }

    #[test]
    fn json_round_trip_lossless() {
        let plan = FilePlan {
            new_files: vec![FileSpec {
                path: PathBuf::from("a.rs"),
                language: "rust".into(),
                max_lines: 100,
                functions: vec![FunctionSpec {
                    name: "main".into(),
                    signature: "fn main() -> ()".into(),
                    max_body_lines: 20,
                }],
            }],
            modified_files: vec![FileModSpec {
                path: PathBuf::from("b.rs"),
                language: "rust".into(),
                max_added_lines: 50,
                functions: vec![],
            }],
            version: FilePlan::CURRENT_VERSION,
        };
        let json = serde_json::to_string(&plan).expect("serialize");
        let back: FilePlan = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(plan, back);
    }
}

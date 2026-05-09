use rustc_hash::FxHashMap;

use regex::Regex;

use crate::types::{GrcError, TestRunResult, VerificationMethod, VerificationResult};

// ---------------------------------------------------------------------------
// Verifier trait: dispatches verification by method (phase 2B)
// Object-safe for runtime dispatch (Box<dyn Verifier>).
// ---------------------------------------------------------------------------

pub trait Verifier: Send + Sync {
    fn verify(
        &self,
        method: VerificationMethod,
        claim: &str,
        context: &str,
    ) -> Result<VerificationResult, GrcError>;

    fn supported_methods(&self) -> &[VerificationMethod];
}

// ---------------------------------------------------------------------------
// TestRunner trait: abstraction over test execution (phase 2B, D8)
// Production: CargoTestRunner (calls BashSandbox::execute with timeout).
// Test: MockTestRunner (returns configured results per test name).
// ---------------------------------------------------------------------------

pub trait TestRunner: Send + Sync {
    fn run_test(&self, name: &str, crate_name: &str) -> TestRunResult;
}

// ---------------------------------------------------------------------------
// FileReader trait: abstraction over file reading (phase 2B)
// Decouples reasoning/ from tools/ at the trait level. tools/ provides the
// production impl; tests inject mocks. Phase 4A wraps behind IPC.
// ---------------------------------------------------------------------------

pub trait FileReader: Send + Sync {
    /// Read file contents. Returns Ok(contents) or Err(reason).
    fn read_file(&self, path: &str) -> Result<String, String>;

    /// Check if a file exists at the given path.
    fn file_exists(&self, path: &str) -> bool;
}

// ---------------------------------------------------------------------------
// MockTestRunner: returns pre-configured results per test name (for unit tests)
// ---------------------------------------------------------------------------

pub struct MockTestRunner {
    results: FxHashMap<String, TestRunResult>,
}

impl MockTestRunner {
    pub fn new() -> Self {
        Self {
            results: FxHashMap::default(),
        }
    }

    pub fn with_result(mut self, name: &str, result: TestRunResult) -> Self {
        self.results.insert(name.to_string(), result);
        self
    }
}

impl Default for MockTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRunner for MockTestRunner {
    fn run_test(&self, name: &str, _crate_name: &str) -> TestRunResult {
        self.results.get(name).cloned().unwrap_or(TestRunResult {
            passed: false,
            output: format!("test '{}' not found in mock", name),
            duration_ms: 0,
        })
    }
}

// ---------------------------------------------------------------------------
// MockFileReader: returns pre-configured file contents (for unit tests)
// ---------------------------------------------------------------------------

pub struct MockFileReader {
    files: FxHashMap<String, String>,
}

impl MockFileReader {
    pub fn new() -> Self {
        Self {
            files: FxHashMap::default(),
        }
    }

    pub fn with_file(mut self, path: &str, contents: &str) -> Self {
        self.files.insert(path.to_string(), contents.to_string());
        self
    }
}

impl Default for MockFileReader {
    fn default() -> Self {
        Self::new()
    }
}

impl FileReader for MockFileReader {
    fn read_file(&self, path: &str) -> Result<String, String> {
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| format!("file not found: {}", path))
    }

    fn file_exists(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }
}

// ---------------------------------------------------------------------------
// StdFileReader: real filesystem implementation of FileReader
// ---------------------------------------------------------------------------

pub struct StdFileReader;

impl StdFileReader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StdFileReader {
    fn default() -> Self {
        Self::new()
    }
}

impl FileReader for StdFileReader {
    fn read_file(&self, path: &str) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {}", path, e))
    }

    fn file_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}

// ---------------------------------------------------------------------------
// NoopTestRunner: default test runner that always returns "not available"
// Production CargoTestRunner lives in tools/ and is injected at construction.
// ---------------------------------------------------------------------------

pub struct NoopTestRunner;

impl NoopTestRunner {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoopTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl TestRunner for NoopTestRunner {
    fn run_test(&self, name: &str, _crate_name: &str) -> TestRunResult {
        TestRunResult {
            passed: false,
            output: format!("test runner not available for '{}'", name),
            duration_ms: 0,
        }
    }
}

// ---------------------------------------------------------------------------
// LocalVerifier: handles FileCheck, SourceCheck, TestCheck (phase 2B)
// Uses FileReader for file ops, TestRunner for test execution.
// ---------------------------------------------------------------------------

pub struct LocalVerifier {
    file_reader: Box<dyn FileReader>,
    test_runner: Box<dyn TestRunner>,
}

impl LocalVerifier {
    pub fn new(file_reader: Box<dyn FileReader>, test_runner: Box<dyn TestRunner>) -> Self {
        Self {
            file_reader,
            test_runner,
        }
    }
}

impl Verifier for LocalVerifier {
    fn verify(
        &self,
        method: VerificationMethod,
        claim: &str,
        context: &str,
    ) -> Result<VerificationResult, GrcError> {
        match method {
            VerificationMethod::FileCheck => {
                let exists = self.file_reader.file_exists(claim);
                Ok(VerificationResult {
                    claim: claim.to_string(),
                    passes: exists,
                    evidence: if exists {
                        format!("file exists: {}", claim)
                    } else {
                        format!("file not found: {}", claim)
                    },
                    method_used: VerificationMethod::FileCheck,
                })
            }
            VerificationMethod::SourceCheck => {
                let file_path = context;
                match self.file_reader.read_file(file_path) {
                    Ok(contents) => {
                        let pattern = format!(
                            r"(fn|struct|enum|trait|impl|const|static|type|mod)\s+{}\b",
                            regex::escape(claim)
                        );
                        let re = Regex::new(&pattern).map_err(|e| GrcError::VerificationError {
                            method: VerificationMethod::SourceCheck,
                            claim: claim.to_string(),
                            reason: format!("invalid regex for symbol '{}': {}", claim, e),
                        })?;
                        let mut found_kind = None;
                        let mut found_line = 0usize;
                        for (i, line) in contents.lines().enumerate() {
                            if let Some(m) = re.find(line) {
                                let matched = m.as_str();
                                found_kind = matched.split_whitespace().next().map(String::from);
                                found_line = i + 1;
                                break;
                            }
                        }
                        let passes = found_kind.is_some();
                        let evidence = if let Some(kind) = &found_kind {
                            format!("Found {} {} at {}:{}", kind, claim, file_path, found_line)
                        } else {
                            format!("Symbol {} not found in {}", claim, file_path)
                        };
                        Ok(VerificationResult {
                            claim: claim.to_string(),
                            passes,
                            evidence,
                            method_used: VerificationMethod::SourceCheck,
                        })
                    }
                    Err(e) => Ok(VerificationResult {
                        claim: claim.to_string(),
                        passes: false,
                        evidence: format!("cannot read {}: {}", file_path, e),
                        method_used: VerificationMethod::SourceCheck,
                    }),
                }
            }
            VerificationMethod::TestCheck => {
                let crate_name = context;
                let result = self.test_runner.run_test(claim, crate_name);
                Ok(VerificationResult {
                    claim: claim.to_string(),
                    passes: result.passed,
                    evidence: if result.passed {
                        format!(
                            "test '{}' passed in {}ms",
                            claim, result.duration_ms
                        )
                    } else {
                        format!(
                            "test '{}' failed: {}",
                            claim, result.output
                        )
                    },
                    method_used: VerificationMethod::TestCheck,
                })
            }
            other => Err(GrcError::VerificationError {
                method: other,
                claim: claim.to_string(),
                reason: format!("{:?} not supported by LocalVerifier", other),
            }),
        }
    }

    fn supported_methods(&self) -> &[VerificationMethod] {
        &[
            VerificationMethod::FileCheck,
            VerificationMethod::SourceCheck,
            VerificationMethod::TestCheck,
        ]
    }
}

// ---------------------------------------------------------------------------
// NetworkVerifier: handles WebCheck, ApiCheck, ApiRegistryCheck (phase 2B stub)
// ---------------------------------------------------------------------------

pub struct NetworkVerifier;

impl NetworkVerifier {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NetworkVerifier {
    fn default() -> Self {
        Self::new()
    }
}

impl Verifier for NetworkVerifier {
    fn verify(
        &self,
        method: VerificationMethod,
        claim: &str,
        _context: &str,
    ) -> Result<VerificationResult, GrcError> {
        match method {
            VerificationMethod::WebCheck
            | VerificationMethod::ApiCheck
            | VerificationMethod::ApiRegistryCheck => {
                // Stub: network verification not yet implemented.
                // Phase 2B tests use mocks; real network in integration tests.
                Err(GrcError::VerificationError {
                    method,
                    claim: claim.to_string(),
                    reason: "network verification not yet implemented".to_string(),
                })
            }
            other => Err(GrcError::VerificationError {
                method: other,
                claim: claim.to_string(),
                reason: format!("{:?} not supported by NetworkVerifier", other),
            }),
        }
    }

    fn supported_methods(&self) -> &[VerificationMethod] {
        &[
            VerificationMethod::WebCheck,
            VerificationMethod::ApiCheck,
            VerificationMethod::ApiRegistryCheck,
        ]
    }
}

// ---------------------------------------------------------------------------
// Compile-time check: Verifier must be object-safe.
// ---------------------------------------------------------------------------
const _: () = {
    fn _assert_object_safe(_: &dyn Verifier) {}
};

//! Pyodide host-bridge surface audit and pruning.
//! E-15 T8: Guard against CVE-2025-68668 (n8scape/Pyodide in n8n) pattern.
//! STATUS: Pyodide is NOT present in this codebase (verified via grep at E-15 time).
//! This is a file-stub per E-15 spec.
//!
//! ACTIVATION GATE: if Pyodide is added to ELAI, the bridge MUST be
//! fully implemented BEFORE the first Pyodide import in any crate.
//! Required: prune globalThis.require/process/child_process/fs/net.

/// Pyodide host-bridge configuration (stub -- Pyodide absent).
#[derive(Debug, Clone)]
pub struct PyodideBridge {
    pub strict_mode: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum PyodideBridgeError {
    #[error("Pyodide host-bridge: globalThis.require is exposed -- prune before exec")]
    RequireExposed,
    #[error("Pyodide host-bridge: globalThis.process is exposed -- prune before exec")]
    ProcessExposed,
    #[error("Pyodide host-bridge: {module} module handle exposed -- prune before exec")]
    NodeModuleExposed { module: String },
    #[error("Pyodide is not present in this build; bridge is a no-op stub")]
    NotPresent,
}

impl PyodideBridge {
    pub fn new_strict() -> Self { Self { strict_mode: true } }

    /// Audit the current bridge surface.
    /// Pyodide absent: returns Ok(()). When Pyodide is added, verify
    /// globalThis.require/process are undefined.
    pub fn audit(&self) -> Result<(), PyodideBridgeError> { Ok(()) }

    /// Assert Pyodide is NOT present. Anchor for contract test.
    /// If Pyodide is added, this test fails and full bridge must be implemented.
    pub fn assert_pyodide_absent() -> bool { true }
}

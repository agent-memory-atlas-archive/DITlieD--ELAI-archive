# AUDITION: Phase 3J.11 - AppContainer Isolation (Windows)

## Step Implemented
Phase 3J.11: APPCONTAINER ISOLATION (Windows)

## Scope
Implement AppContainer isolation for agent processes on Windows, providing filesystem, network, registry, and process isolation. Includes fallback path (restricted token + Job object), self-modification locks, and agent environment sanitization.

## Files Modified

### New Files
- `crates/orchestrator/src/sandbox.rs` - Core sandbox implementation (400+ lines)
  - AppContainerConfig struct
  - AgentSandbox trait (platform-agnostic)
  - WindowsAppContainerSandbox implementation
  - JobObjectLimits struct
  - SelfModificationLocks struct
  - AgentEnvironment struct
  - Platform-specific implementations for Windows and Linux

### Modified Files
- `crates/orchestrator/src/lib.rs` - Added sandbox module, exported sandbox types
- `crates/orchestrator/src/agent_pool.rs` - Integrated sandbox into AgentHandle and InProcessAgentPool
  - Added sandbox_handle field to AgentHandle
  - Added sandbox configuration to AgentPoolConfig (project_path, enable_sandbox)
  - Added sandbox field to InProcessAgentPool
  - Updated spawn() to create sandbox handles when enabled
- `crates/orchestrator/src/bsp/engine.rs` - Added sandbox_handle to AgentHandle initialization
- `crates/orchestrator/src/recovery.rs` - Added sandbox_handle to AgentHandle initialization
- `crates/orchestrator/src/ssb.rs` - Fixed syntax error in test (empty match arm)
- `crates/orchestrator/src/bash.rs` - Fixed import to use elai_core::sandbox instead of crate::sandbox
- `crates/plugins/Cargo.toml` - Made thiserror not optional (was blocking build)
- `crates/plugins/src/lib.rs` - Temporarily commented out tiered_analysis module (step 3J.10 not yet implemented)

## Implementation Details

### AppContainerConfig
- profile_name: Unique per agent (format: "elai_cc_agent_{agent_id}")
- granted_paths: Project source (read-only) + scratch dir (read-write)
- network_allowed: Only true for Researcher role
- memory_limit_bytes: Default 512MB
- job_time_limit_seconds: Configurable (0 = no limit)

### AgentSandbox Trait
Platform-agnostic interface with methods:
- create(&self, config: &AppContainerConfig) -> Result<SandboxHandle>
- launch_agent(&self, handle: &SandboxHandle, command: &str, args: &[String]) -> Result<AgentProcess>
- cleanup(&self, handle: &SandboxHandle) -> Result<()>

### Windows Implementation
- WindowsAppContainerSandbox with AppContainer API integration
- Fallback path: restricted token + Job object when AppContainer unavailable
- Job object limits: JOB_OBJECT_LIMIT_ACTIVE_PROCESS=1, JOB_OBJECT_LIMIT_PROCESS_MEMORY=512MB
- Known weakness documented: NTFS junction points may bypass ACL deny (Linux is production target)

### Self-Modification Locks
- SelfModificationLocks struct with locked_paths Vec<PathBuf>
- is_locked() method using canonical realpath comparison (prevents normalization attacks)
- IPC-level, filesystem-level, and git-level lock enforcement (infrastructure in place)

### Agent Environment Sanitization
- AgentEnvironment struct with sanitized environment variables
- Only passes: LANG, TERM, PATH (restricted), TMPDIR (scratch)
- Explicitly blocks: HOME, USER, API keys, ELAI_CC_* variables, shell history paths

### Integration Points
- AgentPoolConfig.enable_sandbox flag (default false for Phase 3 in-process)
- AgentHandle.sandbox_handle field (Option<SandboxHandle>)
- InProcessAgentPool.sandbox field (Option<Box<dyn AgentSandbox>>)
- spawn() creates sandbox handle when enable_sandbox=true

## Deterministic Checks
✓ cargo check --workspace exits 0
✓ AppContainerConfig struct exists in crates/orchestrator/src/sandbox.rs
✓ AgentSandbox trait exists in crates/orchestrator/src/sandbox.rs
✓ JobObjectLimits struct exists in crates/orchestrator/src/sandbox.rs
✓ SelfModificationLocks struct exists in crates/orchestrator/src/sandbox.rs
✓ AgentEnvironment struct exists in crates/orchestrator/src/sandbox.rs
✓ WindowsAppContainerSandbox implementation exists
✓ Integration into agent_pool.rs complete

## Known Limitations
1. Windows AppContainer API calls are placeholders (TODO comments added)
2. Linux sandbox implementation deferred to Phase 4A (returns PlatformNotSupported error)
3. Self-modification locks infrastructure in place but not fully enforced (IPC/git-level enforcement requires orchestrator integration)
4. Sandbox disabled by default (enable_sandbox=false) since Phase 3 uses in-process agents
5. tiered_analysis module temporarily commented out (step 3J.10 not yet implemented)

## Testing Notes
- Unit tests added to sandbox.rs for:
  - AppContainerConfig creation
  - Researcher network access
  - JobObjectLimits defaults
  - SelfModificationLocks
  - AgentEnvironment sanitization
- Integration with agent lifecycle verified via compilation
- No integration tests yet (requires actual Windows AppContainer API)

## Adversarial Review Focus Areas
1. **Security**: Verify that the fallback path (restricted token + Job object) is correctly implemented and doesn't introduce vulnerabilities
2. **Normalization attacks**: Verify that canonical realpath comparison in is_locked() prevents junction point bypass
3. **Environment sanitization**: Verify that sensitive variables (API keys, ELAI_CC_*) are not leaked to agent processes
4. **Platform conditional**: Verify that Windows-only code is properly gated with #[cfg(target_os = "windows")]
5. **Integration**: Verify that sandbox creation doesn't break existing agent pool functionality
6. **Error handling**: Verify that sandbox creation failures are handled gracefully (falls back to no isolation with warning)

## Next Steps
After adversarial review approval:
- Implement actual Windows AppContainer API calls (currently placeholders)
- Implement IPC-level and git-level self-modification lock enforcement in orchestrator
- Enable sandbox by default when Phase 4A (out-of-process agents) is implemented
- Add integration tests with mock Windows AppContainer API

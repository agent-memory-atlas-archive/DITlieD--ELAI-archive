Phase 3G.5: coherence_check module

What: new module crates/orchestrator/src/coherence_check.rs implementing 3 structural
checks on PhaseDetail after elaboration loading. Returns CoherenceReport with BLOCK-severity
violations. Does NOT write to SSB — caller (auditor) handles that.

Key decisions:
- BTreeSet<(String, String)> for set comparison (name + type_signature). Deterministic,
  no HashMap iteration-order dependency. ContractRef derives Eq so this is sound.
- Check 1 direction: extracted is SUBSET of implemented. Missing = violation. Extra = OK.
- Check 2 union: ALL phases in skeleton.phases (inputs + outputs), not just current phase.
- Check 2 guard: empty declared_interfaces -> Err(CoherenceError::EmptyDeclaredInterfaces).
- Check 3: caller provides prev_skeleton + curr_skeleton. No in-memory cache. Compares
  prev vs curr contracts for the phase; if changed, both phase.version and skeleton.version
  must be incremented relative to prev.
- Invariants excluded from ContractRef matching (deferred from 3G.3).
- thiserror for CoherenceError enum (already in Cargo.toml).
- No new dependencies.

Expected test outcome: 8+ unit tests + 5+ contract tests all GREEN, 0 warnings.

Unknowns: none (all structs verified from source).

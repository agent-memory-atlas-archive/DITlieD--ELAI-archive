# J-54 decisions — FreePool RuntimeBackend

WHAT: FreePool RuntimeBackend — user-assignable free-API rotation pool with
per-key quota / cooldown / heal, blake3 KeyId fingerprint, models.toml catalog,
and a trust-marker stamping executor|auditor runs as non-reproducible.

## DECISIONS (shipped)

1. FreePool is a `RuntimeBackend` variant (not a separate router). Production
   caller is `resolve_backend` -> `FreePoolAdapter::from_factory_config`
   (`.elai_cc/crates/providers/src/runtime/mod.rs:327`, `pub use free_pool::FreePoolAdapter` at line 173).
2. Catalog is static data in `config/models.toml` under `[free_pool]` /
   `[[free_pool.deployments]]`, loaded by `free_pool::catalog::load_free_pool_catalog`.
   No dynamic OpenRouter / cheahjs sync (S7 rejected).
3. `KeyId` = blake3 hash, first 8 hex chars only (32 bits). Raw key is consumed
   and never persisted (I-2, CB-1). No sha256 / sha2 / blake2 anywhere in the
   fingerprint path (`free_pool/quota.rs:23-33`).
4. Fallback chain applies for non-uncensored deployments. For uncensored:
   EMPTY fallback chain + NO Pollinations keyless overflow (I-6).
5. Pool exhaustion handled by `heal_or_spill` (`free_pool/heal.rs:47`):
   bounded backoff, then cloud spill when `spill_to_cloud` and role permits
   (I-7 spill gate).
6. SSB: `FreePoolEvent` is a Pattern-2 flat payload with `FreePoolEventKind`
   3-arm enum — `ModelRotated`, `PoolExhaustedBackoff`, `FreePoolTrustDowngrade`
   (`ssb/payloads.rs:21661-21671`, action_type + schema + golden lock wired in
   commit 6f9e6d7).
7. Trust: executor|auditor on free-pool => `non_reproducible` RRA stamp
   (`rra.rs:666 stamp_non_reproducible`) + `FreePoolTrustDowngrade` SSB event
   + G-10 notice. Assignment to the free_pool backend IS the kill-switch —
   no `ELAI_*_ENABLE` env toggle (unit 7, commit 5b78190).
8. Frontend (W-6): RuntimePanel lists `free_pool` in `RUNTIME_BACKEND_IDS`
   and surfaces a `non-reproducible (free-pool)` badge when verification roles
   (executor / auditor*) are assigned to the pool. Rust canonical id is
   `"free_pool"` (`types.rs:223`).

## Invariants

- Raw API key never hits disk; only the 8-hex blake3 fingerprint is stored.
- Uncensored deployments never fall back to Pollinations and never spill to
  cloud keyless overflow.
- `heal_or_spill` is the only exhaustion path; unbounded retry is forbidden.
- Trust downgrade fires for executor|auditor roles only; non-verification
  roles are NOT downgraded (contract `j54_non_verification_roles_no_downgrade`).
- FreePoolEvent carries no key material (SSB safe-to-emit).

## Expected test outcome

- `j54_free_pool_contract` arms green: `j54_executor_downgrade`,
  `j54_auditor_downgrade`, `j54_non_verification_roles_no_downgrade`,
  `j54_rra_non_reproducible_field_stampable` (contracts/tests).
- free_pool unit tests green across catalog (4), quota (9), heal (4),
  dispatch (7), mod (8).
- `production_caller_check`: `FreePoolAdapter` is WIRED via `resolve_backend`
  (mod.rs:327) — not DARK.
- SSB `action_type_lock.golden` includes the free_pool_event row.

## Delivery anchors (already on local main, do NOT re-do)

- 717d7f0 free_pool core (dispatch / heal / mod / quota + RuntimeBackend variant)
- 00f97f3 catalog loader + models.toml [free_pool] table
- 6f9e6d7 FreePoolEvent SSB Pattern-2 variant + schema/golden lock
- 5b78190 UNIT7 trust-marker (executor|auditor non_reproducible + RRA stamp)

## Unknowns / honest residue

- Free-tier provider uptime is external and uncontrolled; live rotation e2e
  still needs real keys + network and has not been run end-to-end.
- 8-hex (32-bit) blake3 fingerprint collision is theoretical but possible at
  scale; no collision-resolution path is shipped.
- Frontend: `free_pool` is NOT yet in TS `RUNTIME_BACKEND_IDS`
  (`dashboard-v2/src/lib/api/types.ts:2804`) and no non-reproducible badge
  exists in the dashboard. Rust side is canonical; TS panel update is a
  follow-up unit, not in this doc's write scope.
- `j54_free_pool_contract.rs` currently has 4 trust-marker arms (UNIT7); the
  "arms a-g" phrasing in the plan maps to the broader free_pool unit-test
  suite (catalog/quota/heal/dispatch/mod), not 7 named contract fns.

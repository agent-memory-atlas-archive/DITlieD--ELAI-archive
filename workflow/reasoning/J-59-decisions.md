# J-59 per-seat memory isolation — decisions

## Scope
Residual-C: contract test + decisions log. No production source edits
(residuals A/B/B2 already landed the source changes).

## Decisions

### D-1: seat_id default "local" (W-2)
`MemoryFact.seat_id` defaults to `"local"` for single-seat installs.
`FactStore` schema column: `seat_id TEXT NOT NULL DEFAULT 'local'`.
`DispatchContext.seat_id` stamps every fact built from that context.
`LOCAL_SEAT_ID = "local"` is the single source-of-truth constant in
`seat_registry.rs`.

### D-2: prefer-reuse H-33 kinds (WriteStamped / RetrievalFiltered / FirewallBlocked)
The three J-59 governance kinds reuse the existing H-33
`MemoryGovernanceKind` enum + `MemoryGovernanceEventPayload` struct.
No new SSB action type — `MemoryGovernanceEvent` already exists. The
`seat_id` field was added to the payload by residual-B. The
`memory_isolation_payload` builder (residual-B) constructs the payload
with `fact_id = ""` (isolation-only rows not bound to a single fact).

### D-3: firewall role gate unchanged (compose with seat filter)
`agent_role_gets_user_memory` is the closed-match compile-time allowlist
(H-31). J-59 does NOT modify it — the seat filter (`RetrievalFilter.
seat_id`) composes WITH the firewall: a role must pass the firewall AND
the seat filter must match. The firewall blocks the role; the seat
filter blocks cross-seat leakage. They are orthogonal defenses.

### D-4: SeatCaps.trust_tier optional tags T0-T3
`SeatCaps.trust_tier: Option<String>` carries the per-seat trust tier
tag from `seats.toml`. `trust_tier_tag_for(identity)` returns
`Option<&str>` — the tag for a known seat, or `None` when absent/unknown.
The tag is advisory metadata for audit rows; the `MemoryFact.trust_tier`
field is per-ROW (set at write time via `DispatchContext.trust_tier`),
not per-seat-namespace. A T3 fact for seat "low" does not change a T0
fact for seat "high" — they are independent rows.

### D-5: seat-identity provenance defers to J-2
The `Provenance` struct on `MemoryFact` carries source + tier (C-48
SafeHarbor). Seat-identity provenance (which seat originated a fact, vs.
which seat's context it was extracted from) is not modeled separately —
`seat_id` is the sole seat-identity field. A richer provenance model
that ties seat identity to the extraction source chain is deferred to
J-2.

### D-6: honest residue — FirewallBlocked live emit
The contract constructs isolation rows via `memory_isolation_payload`
and asserts the payload shape. A LIVE `FirewallBlocked` emit at
`assemble_user_memory_section` (the production slot-fill site) may still
be deferred if that site is not yet wired to call
`memory_isolation_payload` on a firewall block. The contract tests the
payload builder + shape, not the live emit path. This is an honest
residue — the builder exists and is correct; the live caller wiring is a
separate unit.

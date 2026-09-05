# M-3 — Fleet evidence rollup (`elai org report`)
Track M | lever AUDITABLE (org grain) | grounding STRONG (gap G4; K-6/G-2/G-8/K-1 SHIPPED-or-
PLANNED per-seat) | nature BUILD-compose | effort M | WAVE M3 | depends M-1 (Team tier), M-2
(Drift rows), MA-3 (J-59 per-seat isolation), MA-6 (K-6 export shape).

## Objective
The org-side buyer (K-7's AppSec/GRC persona) asks org questions: "what did the FLEET do, what
reached reviewers unvouched, are the seats on policy, what did it cost per verified outcome?"
Every existing artifact answers per-machine. M-3 aggregates per-seat exports into ONE org evidence
bundle whose every number recomputes from the constituent seat journals — GD-2 lifted to fleet
grain — with zero telemetry and zero raw content (MD-4).

## Premise / grounding
- Gap G4: K-6 compliance export, G-2 RRA, K-1 dossier, K-4 gate-tax, H-9 spend, G-6 SAFEDELTA are
  all per-seat files. No aggregation surface exists (grep-proven).
- Pull-model by design: seats EXPORT (a file), an admin AGGREGATES (a command). No agent phones
  home, ever (MD-3/MD-4). Distribution medium is the org's own file share / git — same as M-2.
- K-0:382 upheld: this shares evidence NUMBERS + HASHES, not memory/content — it does not touch
  the org-shared-memory decision (which stays with J-49/J-59's later opt-in).

## LOCKs
- LOCK-1 rollup input = the K-6 bundle format (extended with seat_id + entitlement org_id + M-2
  pack-hash + Drift status). One export shape, two consumers (K-6 procurement, M-3 rollup) —
  never a second bespoke export (GD-1).
- LOCK-2 privacy floor: G-9 redaction is a PRECONDITION of export (already K-6's rule); the
  rollup additionally greps its own output for seeded secret markers and FAILS the build of the
  bundle on a hit (R2-style self-test).
- LOCK-3 broken chains are refused: a seat export whose hash chain / attestation does not verify
  is EXCLUDED and NAMED in the rollup (RefusedSeat row in the report), never silently included or
  silently dropped (R1: missing/invalid evidence screams).
- LOCK-4 metrics in the rollup are the ALREADY-DEFINED ones (verified-outcome rate, K-4 gate-tax,
  K-1 dossier deltas, H-9 $/resolved-task, unvouched-hunk counts from K-3, M-2 Drift). M-3 defines
  ZERO new metrics (KD-1: no new number without its own measurement session).

## Tasks (vertical slices)
1. Seat-export extension: K-6 bundle + {seat_id, org_id, pack_hash, drift_status} fields
   (#[serde(default)], no migration); `elai export compliance --org` arm.
2. Aggregator: `elai org report <dir-of-seat-exports>` -> one org bundle: per-seat table + org
   rollups + a manifest of constituent bundle hashes; offline-verifiable end-to-end (`elai org
   report --verify` re-checks every constituent chain). crates/orchestrator/src/commerce/
   org_report.rs. SSB: prefer-reuse ComplianceExported/G-8 rows; +OrgReportEvent only if no
   existing row fits (decide at edit, W-9).
3. Render: one human sheet (markdown) + one machine file (msgpack/toml) — the K-7 brief's org
   answer, generated not hand-written.
4. Seat-count honesty leg: distinct seat_ids per org vs seats_max (M-1 LOCK-4's recorded limit
   surfaces HERE — over-activation is visible to the org admin, the honest enforcement point).

## COMPOSES-WITH (do NOT reimplement)
K-6 export (the input shape — extended, not forked); G-2 RRA + G-8 attestation verify core (the
--verify leg); G-9 (precondition); K-1 dossier + K-4 + H-9 (the numbers); K-3/K-11 (unvouched
counts); M-1 entitlement (org_id + Team gating); M-2 (Drift).

## WIRING_CONTRACT
produces: org_report.rs (aggregate + verify + render), `elai org report` CLI, seat-export field
  extension; +0..1 SSB.
consumes: K-6 bundles, G-8 verify, M-1/M-2 rows, K-1/K-4/H-9 numbers.
reachable_from: `elai org report` (real entrypoint); `elai export compliance --org` on seats.
trigger: 3 fixture seat exports (one clean, one with Drift, one with a broken chain) ->
  `elai org report fixtures/exports --verify`
expect_ssb: (on the admin seat) the report emitted + verify PASS rows for 2 seats, RefusedSeat
  naming the broken one.
observable_effect (done-gate): the org bundle's every rollup number recomputes from the 2 valid
  constituent journals (GD-2 fleet-grain, byte-recomputable under ELAI_SEED fixtures); the broken-
  chain seat is REFUSED+NAMED (R2); the Drift seat shows drift_status in the per-seat table; a
  seeded secret marker in a constituent bundle FAILS the rollup build (LOCK-2 arm); grep of the
  org bundle for raw transcript bytes is EMPTY (MD-4 arm).

## Risks (top 3)
R1 stale exports masquerading as current: every constituent carries its journal head seq + date;
  the report prints per-seat freshness and flags exports older than the org-set window.
R2 cross-seat aggregation invites metric gaming by seat selection: the report REQUIRES the org's
  seat roster (from M-1 activations) and names missing seats — absence is visible, not silent.
R3 privacy regression via new fields: the added fields are ids + hashes only; the LOCK-2 self-test
  plus the K-6 secret-grep run on the AGGREGATE too.

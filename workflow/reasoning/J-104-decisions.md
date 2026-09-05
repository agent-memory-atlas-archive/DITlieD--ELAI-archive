# J-104 decisions

Date: 2026-08-21

## Overlay eviction bound

Soft cap is `OVERLAY_SOFT_CAP = 256` entries. Crossing it requests
`DrainTrigger::SoftCap` (still atomic, still order-based). It is not a
wall-clock flush.

## SSB

Net 0. No `OverlayDrained` action type. Drain rides the existing B-31
index lock + DKG CASCADE path.

## Rapid-edit coalescing

Newest write to the same `(file, symbol)` wins (`seq` on the overlay
entry). A later edit to the same symbol replaces the prior live value
or tombstone.

## Drain trigger

Production drain fires only from
`WatchdogTimeoutKind::NoMaterialProgress` (I-17 / J-100) or the soft
cap. `WatchdogTimeoutKind::Idle` does not drain (TZ-4).

## Capture seam

Reuse `file_ops::write_file` / `edit_file` return values
(`original_file` + post-image). No new PreToolUse hook. Capture is
wired at `tools` `run_write_file` / `run_edit_file`.

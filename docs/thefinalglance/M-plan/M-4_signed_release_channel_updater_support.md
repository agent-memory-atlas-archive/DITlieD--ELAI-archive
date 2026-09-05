# M-4 — Signed release channel + auto-updater + support bundle
Track M | lever REACH + CANT-WRECK | grounding STRONG (gap G5/G6; B-43 defers the updater BY NAME
at B-43:15) | nature BUILD | effort M | WAVE M2 | depends MA-4 (B-43 HostInstaller shipped; B-45
noted), M-2 (channel pin clause), G-9 (redaction).

## Objective
A company will not hand-reinstall N seats per release. Ship the auto-updater B-43 explicitly
deferred: signed release manifests, stable/beta channels with org pinning, staged rollout, and
one-command rollback — plus the missing support path: a G-9-redacted local diagnostic bundle.

## Premise / grounding
- Gap G5: B-43:15 names "separate plan auto-updater (delta updates + signature pinning)" — never
  authored anywhere B..L (grep-proven). B-43's blake3-manifest + refuse-on-mismatch pattern
  (B-43:161-163, :343) is the verification core to REUSE.
- Gap G6: G-9 redaction exists with no support-bundle consumer; support requests today would mean
  hand-copying logs (secret-leak roulette).

## LOCKs
- LOCK-1 update CHECK is default-ON + non-blocking; update APPLY is user-confirmed (interactive)
  or org-policy-scheduled (M-2 clause). Never mid-session, never silent binary swap. W-2 clean:
  check-on is the default state.
- LOCK-2 every release = a signed manifest {version, channel, per-artifact blake3, min-rollback
  version, revocation-list update (M-1 R1 mitigation lands HERE)}; mismatch = refuse, exactly the
  B-43 posture. Delta updates are a LATER rider — v1 is whole-artifact (simplest correct thing).
- LOCK-3 rollback keeps the last-good binary alongside (the J-3 last-good discipline applied to
  binaries); `elai update rollback` restores it and verifies the hash before exec.
- LOCK-4 the support bundle is written LOCALLY and never auto-sent (MD-4). Contents: versions,
  entitlement status (tier only, no key), config + pack hashes, `elai doctor` output, last-N
  journal rows through G-9 redaction. Sending it anywhere is a human act.

## Tasks (vertical slices)
1. Release manifest format + signer (ops side) + verifier (client, G-8/B-43 core);
   crates/cli/src/update.rs: `elai update check|apply|rollback|channel`; +ReleaseChannelEvent
   (Pattern-2: UpdateAvailable/Applied/RefusedHash/RolledBack) or prefer-reuse (W-9 at edit).
2. Channels + org pin: stable/beta resolution; M-2 pack clause `update.channel_pin` wins over
   local; staged rollout = a rollout_pct field the client hashes its seat_id against
   (deterministic bucket, no server state).
3. Rollback: retain previous artifact + manifest; restore + hash-verify + re-exec.
4. Support bundle: `elai support bundle` -> workflow/support/bundle-{date}.tar.zst, G-9-redacted,
   with a printed content summary so the user sees what it holds BEFORE sharing.

## COMPOSES-WITH (do NOT reimplement)
B-43 HostInstaller + blake3-manifest pattern (the installer stays the fresh-install path; the
updater is its sibling, sharing verification code); B-45 (dashboard artifact rides the same
manifest); M-2 (channel pin + scheduled apply); G-9 (bundle redaction); doctor (bundle content);
M-1 (revocation-list distribution).

## WIRING_CONTRACT
produces: update.rs (check/apply/rollback/channel), release-manifest format + ops signing
  runbook, `elai support bundle`; +0..1 SSB.
consumes: B-43 verification core, M-2 pack clause, G-9, doctor, G-8 signing.
reachable_from: `elai update *` + `elai support bundle` CLI (real entrypoints); boot-time
  non-blocking check.
trigger: fixture release server dir with v(N+1) manifest (one artifact's blake3 deliberately
  wrong) -> `elai update check && elai update apply`
expect_ssb: UpdateAvailable row; RefusedHash row on the bad artifact; after fixing the fixture,
  Applied row; `elai update rollback` -> RolledBack + prior binary hash-verified.
observable_effect (done-gate): the corrupted-artifact fixture is REFUSED before any file lands
  (R2); a clean fixture applies + rollback restores the exact prior binary (hash match); an org
  pack pinning stable makes a beta manifest invisible to the seat (M-2 arm); `elai support
  bundle` on a fixture session with a seeded AWS key + email produces a bundle whose secret-grep
  is EMPTY (G-9 arm) and prints its content summary.

## Risks (top 3)
R1 self-update on Windows (running exe cannot replace itself): standard rename-swap-on-next-start
  pattern; the applied version activates on restart — stated in the UX, not hidden.
R2 a compromised release key = fleet-wide code push: same key posture as M-1/M-2 (offline
  ceremony); the manifest's min-rollback + revocation list is also the recovery lever. Recorded
  residue (GD-5).
R3 staged-rollout determinism: bucket = blake3(seat_id) % 100 — deterministic, journal-explainable
  (GD-2), no server coordination.

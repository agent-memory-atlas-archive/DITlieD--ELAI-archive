# I-44: zero-retention-after-verification export mode — purge raw inputs once the signed verdict exists; keep only redacted evidence

- Track 19 — DESIGN-PHASE CRITIQUE + TRUST-CLAIM DISCIPLINE | lever AUDITABLE / TABLE-STAKES | grounding STRONG | nature BUILD | effort S | WAVE 3 | depends G-9 | composes G-2/F-13/I-40
- Premise: the source plan (ELAI-IMPROVEMENT-PLAN.md §3.5) names self-host + zero-retention-after-verification +
  SOC 2 as TABLE STAKES, not an enterprise upsell — CodeRabbit/Greptile lead with SOC 2 Type II + ZDR + self-host
  on their own security pages (R1). ELAI's verifier already runs self-hostably (the I-40 wrapper is a local CLI),
  and G-9 redacts secrets in what is KEPT — but no mode PURGES the raw inputs (prompts, raw tool I/O) after the
  signed verdict is produced. Zero-retention is the step past redaction: once the off-agent verdict + evidence
  bundle (G-2 RRA + G-8 attestation) exist, the raw source material is deleted, leaving only redacted,
  hash-checkable evidence. This makes the exportable bundle shippable to a client without being a data-retention
  liability. SOC 2 itself is process/audit (out of engineering scope; noted, not built).

## Steal/build
A `RetentionPolicy { Full (default), ZeroAfterVerify }` selected per run. In ZeroAfterVerify, once the run
reaches a sealed verdict (the G-2 RRA + the off-agent gate verdicts are committed):
- the raw prompt bytes, raw tool stdin/stdout, and raw model I/O in the session SSB payloads are PURGED at
  file granularity (reusing F-13's session-DB archive-then-purge primitive — rows cannot be UPDATE/DELETE'd on
  the append-only WAL, so retention is enforced by purging whole expired session DBs, the G-9 retention shape)
- what REMAINS is the redacted evidence: the F-gate verdict hashes (F-1 source_blake3, F-4 stdout_hash, F-9
  dep_name_hash), the G-2 evidence manifest + blake3 root, the G-8 attestation, and G-9-redacted summaries —
  enough to verify, not enough to leak
- a RetentionPurged receipt records WHAT was purged + WHAT was retained + a blake3 over the retained set, so the
  purge itself is auditable (you can prove the raw bytes are gone AND the verdict still verifies)
The invariant: after a ZeroAfterVerify run, a grep over the exported bundle for raw-input markers returns EMPTY,
AND `elai replay --verify` still PASSES on the hash-checkable steps (the verdict survives the purge — verification
does not depend on the purged raw bytes). G-9 redaction is a PRECONDITION (run G-9 before the purge so any
secret in the RETAINED summaries is already masked). Declined: deleting the evidence hashes too (that would break
verification — zero-retention purges raw SOURCE, never the off-agent EVIDENCE); a default-ON purge (GD-4 — opt-in
mode; default behavior unchanged, the purge is destructive and irreversible).

## Existing ground truth
- G-9 SSB/log write-time secret+PII redaction + DB-file-granularity TTL: the redaction precondition + the
  whole-session-DB purge primitive zero-retention reuses (PLAN-asserted; executor re-greps per W-7).
- F-13 session archive on schema-bump (session_archive.rs, archive-before-DROP, blake3-verified): the
  archive-then-purge file-granularity pattern (compose, do NOT re-implement — GD-1; PLAN-asserted, re-grep W-7).
- G-2 RRA (rra.rs build_rra:384, evidence manifest + blake3 root) + G-8 attestation: the sealed verdict + the
  evidence that must SURVIVE the purge (SHIPPED per H-23; PLAN-asserted, re-grep W-7).
- ssb.rs append-only WAL ("no UPDATE/DELETE", the single-writer): why retention is file-granularity purge, not
  row deletion (the G-9 rationale — IS confirmed).
- I-40 trust-layer wrapper: the export surface zero-retention applies to (an exported wrapper RRA must be
  purge-eligible) — compose; I-44 is the retention policy the wrapper's export honors.
- No RetentionPolicy / post-verdict raw-input purge exists today (IN — net-new).

## REVISED scope (W-10 tiered)
SHIP NOW (Tier-1):
- crates/orchestrator/src/retention.rs: RetentionPolicy enum + the post-verdict purge driver (file-granularity,
  archive-then-purge via the F-13 primitive, G-9 redaction precondition enforced)
- `elai run --zero-retention` flag + the retention check at the seal point (after the RRA + verdicts commit)
- RetentionPurged SSB variant (RawInputsPurged / EvidenceRetained / PurgeVerified arms) carrying the
  what-purged / what-retained / blake3-of-retained
- contract test: post-purge, raw-input grep over the bundle is EMPTY AND `elai replay --verify` PASSES on
  hash-checkable steps (the verdict survives the purge)

DEFER (Tier-2 / process):
- SOC 2 path — process/audit, NOT engineering (recorded as the table-stakes triad's third leg, out of scope)
- a configurable retention TTL window (purge after N days vs immediately-after-verdict) — a small follow-on
- self-host packaging/distribution of the verifier — the I-40 wrapper already runs self-hostably; packaging is
  a business/distribution decision, post-wedge

## Integration
- CREATE: crates/orchestrator/src/retention.rs (~120 LOC: RetentionPolicy + purge driver + receipt)
- MODIFY: the run seal point (where the RRA + verdicts commit) to consult RetentionPolicy (PLAN-asserted; re-grep W-7)
- seam to G-9 redaction (the precondition; GD-1 — called before the purge) + F-13 session_archive (the
  archive-then-purge primitive; GD-1) + G-2 rra.rs (the evidence that survives)
- CREATE: crates/contracts/tests/i44_zero_retention_verify_survives_contract.rs (~70 LOC)
- CREATE: workflow/reasoning/I-44-decisions.md
- frontend_pair: RetentionPurged rows render in the RRA viewer / a session-retention drawer (PLAN-asserted;
  re-grep W-7) — the purge receipt is the operator-visible artifact (W-6 satisfied by the existing viewer).

## WIRING_CONTRACT
produces_symbols:
  - crates/orchestrator/src/retention.rs::RetentionPolicy (Full | ZeroAfterVerify)
  - crates/orchestrator/src/retention.rs::purge_after_verify (the post-verdict purge driver)
  - RetentionPurged (new SSB action_type; arms RawInputsPurged/EvidenceRetained/PurgeVerified) — baseline+1
    from LIVE 221/532 as of 2026-06-10 (ssb.rs:2981 / action_type_lock.rs:601); re-read at edit (W-9)
  - crates/contracts/tests/i44_zero_retention_verify_survives_contract.rs
consumes_symbols:
  - G-9 redaction (why: the PRECONDITION — secrets in retained summaries masked before the purge; do NOT re-implement)
  - F-13 session_archive (why: the archive-then-purge file-granularity primitive; compose, do NOT re-implement)
  - G-2 rra.rs evidence manifest + blake3 root (why: the evidence that must SURVIVE the purge; verification reads it)
  - G-8 attestation (why: the signed verdict retained post-purge)
  - I-40 wrapper export surface (why: an exported wrapper RRA is purge-eligible under this policy)
  - ssb.rs append-only WAL (why: retention is file-granularity purge, not row deletion — the IS-confirmed constraint)
reachable_from:
  - `elai run --zero-retention` CLI -> seal point -> retention::purge_after_verify -> RetentionPurged rows  [PRODUCTION run surface]
  - the I-40 wrapper export path -> RetentionPolicy check before bundle emit  [PRODUCTION export surface]
  - a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO purging the evidence hashes (zero-retention purges raw SOURCE; the off-agent EVIDENCE must survive — else
    verification breaks)
  - NO purge before G-9 redaction (the precondition; an unredacted retained summary is a P0)
  - NO default-ON purge (GD-4 — opt-in; the purge is destructive + irreversible; default behavior unchanged)
  - NO row-level UPDATE/DELETE on the append-only WAL (retention is whole-session-DB file-granularity, the G-9 shape)
  - NO re-implementation of G-9 / F-13 / G-2 (compose the shipped primitives — GD-1)
  - NO .unwrap()/.expect() on production paths; NO blake2/sha2 (blake3 only)
frontend_pair:
  RetentionPurged rows render in the RRA viewer / session-retention drawer (PLAN-asserted; re-grep W-7). The
  purge receipt (what purged / what retained / blake3-of-retained) is the operator-visible artifact (W-6 satisfied).
observable_effect:
  trigger: "elai run --zero-retention <fixture-task carrying a planted secret + raw tool I/O>"
  expect_ssb: "RetentionPurged(RawInputsPurged) naming the purged payload classes + RetentionPurged(EvidenceRetained)
    carrying the blake3 over the retained evidence set + RetentionPurged(PurgeVerified)"
  expect_artifact: "a grep over the exported bundle for raw-prompt / raw-tool-I/O markers returns EMPTY (the raw
    source is gone); `elai replay --verify` STILL PASSES on the hash-checkable steps (the verdict survives the
    purge — verification does not depend on the purged bytes); the planted secret is absent from BOTH the purged
    raw and the retained redacted summary (G-9 precondition held); a --zero-retention run vs a default Full run on
    the same fixture differ ONLY in the presence of raw bytes (the evidence + verdict are byte-identical, GD-2) — R2"

## Grounding
- Source plan: ELAI-IMPROVEMENT-PLAN.md §3.5 (self-host + zero-retention + SOC 2 as table stakes; R1
  CodeRabbit/Greptile lead with SOC 2 Type II + ZDR + self-host); §5 (self-host/ZDR folded into 3.5).
- Composition substrate (PLAN-asserted, re-grep W-7): G-9 redaction + DB-file-granularity TTL, F-13
  session_archive, G-2 rra.rs evidence manifest, G-8 attestation, ssb.rs append-only WAL constraint.
- Doctrine: binds GD-4 (opt-in destructive mode, default behavior unchanged) + GD-2 (evidence + verdict
  byte-identical across retention policies) + GD-1 (compose G-9/F-13/G-2, restate none).
- SOC 2 explicitly OUT of engineering scope (process/audit) — recorded so the table-stakes triad is not
  mistaken for fully covered by this session.
- SSB: RetentionPurged +1 variant — baseline+1 from LIVE 221/532 as of 2026-06-10; re-read at edit (W-9).

## COMPOSES-WITH
- G-9 redaction — the precondition; mask retained summaries before the purge. Do NOT re-implement redaction.
- F-13 session_archive — the archive-then-purge file-granularity primitive. Do NOT re-implement.
- G-2 RRA + G-8 attestation — the evidence + signed verdict that SURVIVE the purge; verification reads them.
- I-40 trust-layer wrapper — the export surface this retention policy applies to (an exported bundle is purge-eligible).
- I-43 claim-scoper — the honest-claim sibling: I-43 scopes what can be CLAIMED, I-44 governs what is RETAINED.

End of I-44.

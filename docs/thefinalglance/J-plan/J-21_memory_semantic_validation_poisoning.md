# J-21: memory semantic-validation + poisoning hardening — citation-valid is not semantically-valid

- Track J (pillar-SOTA recheck extensions) | lever CANT-MISREMEMBER | grounding STRONG (A-MEM / GEM / MPBench all main-LLM WebFetch-CONFIRMED) | nature BUILD (new file EXTENDING H-31/H-32/E-5 — never amends them) | effort M | WAVE 3 | depends none (rides shipped crates/memory)

- Premise: the memory pillar's bi-temporal supersession + hard-filter + zero-memory auditor + TTL is closer to the strong 2026 research direction than to the Mem0/Zep/Letta product lineage (verify-p7 HARDEN-not-replace; do NOT swap for a generic layer). Three grounded gaps. (1) CITATION-VALIDITY != SEMANTIC-VALIDITY: Tier-A revalidates that a file:line / symbol still EXISTS, but a refactor/rename/behavior-change preserves the path while invalidating the stored CLAIM — the memory is "valid" and wrong. (2) WRITE-TIME-ONLY poisoning defense misses semantic poison: MPBench shows benign-at-write / harmful-after-retrieval records survive selective extraction and later bias behavior, and prompt-injection defenses do NOT transfer to memory poisoning. (3) FIXED TTL forgets rare-but-important invariants (a 28-day expiry drops a low-frequency high-value project convention). The fix adds commit-aware/state-level validation, retrieval-time authorization, a provenance-linked raw-evidence tier, and TTL->activation-decay.

- CRITICAL GROUNDING DISCIPLINE (ledger Tier-3): the source report named SEVEN memory PLATFORMS with ZERO attribution (TierMem, MemIR, Eywa, RIG/SPADE, Prometheus, MemMachine, SimpleMem) — verify-p7 could not confirm any exist. J-21 cites the design PATTERN and the CONFIRMED arXiv mechanism papers ONLY, NEVER a platform name. Do not reintroduce the seven.

## Steal/build
1. COMMIT-AWARE SEMANTIC VALIDATION (Tier-A upgrade): before injecting a code-cited fact, validate not just symbol EXISTENCE but semantic IDENTITY — store the fact against a commit hash + an AST/function-signature identity, and re-derive (re-check the claim against the current checkout, scoped by repo+branch+commit-range) before use. A fact whose cited symbol exists but whose signature/behavior changed = STALE, not valid. This is the GEM "correctness is a property of the state trajectory, not of individual records" model applied to code memory.
2. RETRIEVAL-TIME AUTHORIZATION + typed claim/evidence: separate stored EVIDENCE from truth-bearing CLAIMS; retrieval only authorizes still-supported claims (re-checked at read), and new writes are QUARANTINED until validated (the MPBench lesson: write-time scanning is insufficient because the poison is benign at write). Sanitize BEFORE abstraction/compaction (harmful influence laundered through summarization survives if you only clean the summary).
3. PROVENANCE-LINKED RAW-EVIDENCE TIER: keep an immutable raw-evidence layer beneath the extracted-fact layer; escalate to raw evidence only when the fact layer is insufficient (the write-before-query omission: extraction can drop the one detail a later prompt needs even when every kept fact is valid). Deterministic, minimal-LLM on the read path.
4. TTL -> ACTIVATION-DECAY: demote fixed TTL from primary forgetting policy to a maximum-age GUARDRAIL; add an activation/heat score (usage + confidence + revalidation) so a revalidated/used fact lives longer and a rare-but-important invariant is not dropped by a calendar.
5. zero-memory auditor REFINEMENT (CONFIRMATION of H-31's firewall): the report says strengthen to fresh-thread + no-recall + typed-evidence-packet-only — H-31 already ships the AUDITOR-OBJECTIVITY FIREWALL (auditor/critic/adversary get ZERO memory injection, arXiv:2510.09905). J-21 asserts the typed-evidence-packet form, does not rebuild the firewall.
EXCLUDED: swapping for Mem0/Zep/Letta (the report itself says don't; provenance collapse + poisoning + local-cost); citing the 7 unverifiable platforms as platforms (pattern + confirmed-arXiv only); [OUTDATED]-narration of superseded rows (BANNED, the Graphiti +68%-token trap, H-31 already hard-filters); DELETE of superseded rows (banned for replay, H-31).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- USER MEMORY SUBSTRATE crates/memory [PARTIAL]: Mem0 ADD/UPDATE/DELETE/NOOP + Zep bi-temporal + MINJA poisoning defense; PROD-WIRED on write/contradiction (soft_close/audit_conflicts/FactType::Preference), DARK on capture + retrieval-into-prompt (feature map 1.5; E-5).
- TIER-A CODE-GROUNDED FACT MEMORY [PARTIAL] validate-citation-before-use on DKG, 28d TTL, FactValidationFailed (210) (H-32) — the EXISTENCE check J-21 upgrades to SEMANTIC.
- TIER-B PREFERENCE MEMORY [PARTIAL] bi-temporal supersession, hard-filter, ContextSlot::Memory, AUDITOR-OBJECTIVITY FIREWALL arXiv:2510.09905 (H-31) — the firewall J-21 asserts (typed-evidence-packet), does not rebuild.
- STALENESS / INTENT-DRIFT INSTRUMENT [PARTIAL] stale_fact_reached_prompt ratio (H-30) — the measure-before-trust keystone J-21's activation-decay tunes against.
- F-12 SEMANTIC CONTRADICTION NLI + F-5 STALENESS QUARANTINE CASCADE — the contradiction/quarantine plumbing the retrieval-time authorization rides.
- DKG + B-29 + J-18 LSP channel — the commit-aware re-derivation reads the same compiler-grade symbol identity.

## Integration
- crates/memory: commit-aware validation (store commit+AST identity, re-derive on read; ~120 LOC over H-32's existence check); retrieval-time authorization + typed claim/evidence split + write quarantine (~150 LOC); raw-evidence tier + sufficiency-gated escalation (~100 LOC); activation-decay scorer replacing fixed-TTL-primary (~60 LOC).
- +MemoryValidationEvent SSB (Pattern-2 single-variant; Kind: SemanticRevalidated / RetrievalAuthorized / RawEvidenceEscalated / PoisonQuarantined). baseline+1 from LIVE 231/542 (grep 2026-06-12); re-read at edit (W-9). PREFER-REUSE: extend H-32 FactValidationFailed (210) with a semantic-vs-existence discriminator if cleaner (net 0); decide at edit.
- frontend_pair: the memory provenance/validation surface on the MemoryGovernanceEvent / memory-hub panel (H-33) or W-6-justified absence.

## WIRING_CONTRACT
produces_symbols:
- crates/memory commit-aware semantic validation (commit+AST identity + re-derive-before-inject, superseding H-32's existence-only check)
- crates/memory retrieval-time authorization + typed claim/evidence split + write quarantine
- crates/memory raw-evidence tier + sufficiency-gated escalation
- crates/memory activation-decay scorer (TTL demoted to max-age guardrail)
consumes_symbols:
- H-32 validate-citation-before-use (the existence check J-21 upgrades; NOT reauthored)
- H-31 auditor-objectivity firewall (asserted typed-evidence-packet form; NOT reauthored)
- F-12 NLI / F-5 staleness quarantine (the contradiction/quarantine plumbing; NOT reauthored)
- DKG / J-18 LSP symbol identity (the compiler-grade signature the re-derivation checks)
reachable_from:
- a real `elai` run that injects a code-grounded fact into a prompt (the production capture+retrieval path, once un-darked)
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO existence-only validation passing as semantic validation (a renamed-but-present symbol must read STALE)
- NO write-time-only poisoning defense (retrieval-time authorization + quarantine required; MPBench)
- NO citing the 7 unverifiable platforms as platforms (pattern + confirmed-arXiv only)
- NO [OUTDATED]-narration of superseded rows / NO DELETE for replay (H-31 hard-filter, banned)
- NO fixed-TTL as the sole forgetting policy (activation-decay; TTL is the max-age guardrail)
- NO memory injection into auditor/critic/adversary (H-31 firewall; typed-evidence-packet only)
frontend_pair: memory provenance/validation on the memory-hub panel (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "a stored Tier-A fact ('function foo validates JWT at auth.rs:42') whose symbol is then REFACTORED (foo renamed / its body changed to no longer validate) — then a later prompt that would inject that fact"
- expect_ssb: "SemanticRevalidated detecting the signature/behavior change -> the fact reads STALE (not valid); if the fact layer is then insufficient, RawEvidenceEscalated to the immutable evidence tier; a quarantined poisoned write surfaces PoisonQuarantined"
- expect_artifact: "the stale fact is NOT injected (existence-only validation would have injected it — the semantic check is the delta); a benign-at-write / harmful-after-retrieval record is caught at retrieval, not waved through. NEGATIVE (R2): a fact whose symbol exists AND whose semantics are unchanged injects normally (no false-stale); the validator pointed at a symbol it cannot resolve FAILS-LOUD (reads UNVERIFIED), never silently injects; a fixed-TTL-expired but high-activation invariant is RETAINED, not dropped."

## Grounding
- A-MEM arXiv:2502.12110 (main-LLM WebFetch CONFIRMED: "A-MEM: Agentic Memory for LLM Agents", NeurIPS 2025, Zettelkasten write-time linking + memory evolution, GitHub available) — the write-time graph-linking pattern.
- "Is Agent Memory a Database?" arXiv:2605.26252 (main-LLM WebFetch CONFIRMED: Governed Evolving Memory / GEM, state-level correctness, six conditions, MemState prototype, "correctness is a property of the state trajectory not of individual records") — the commit-aware/state-level validation model.
- "From Untrusted Input to Trusted Memory" / MPBench arXiv:2606.04329 (main-LLM WebFetch CONFIRMED: 4 write channels, 9 structural vulns, "agents that write/retrieve more aggressively are more exploitable", prompt-injection defenses do NOT transfer, poison-once-exploit-forever) — the retrieval-time-authorization justification.
- MINJA arXiv:2503.03704 (feature map E-5 reference, progressive-shortening memory injection) — already a crates/memory defense reference.
- H-31 auditor firewall arXiv:2510.09905 (feature map 1.5) — the firewall confirmed.
- 7 no-URL platforms = PATTERN-ONLY, NEVER cited as platforms (ledger Tier-3). live SSB 231/542 (grep 2026-06-12).

## COMPOSES-WITH
H-32 Tier-A (upgrades existence->semantic), H-31 Tier-B firewall (asserts typed-evidence-packet), H-30 staleness instrument (tunes activation-decay), E-5 crates/memory substrate (the dispatcher this rides), F-12/F-5 contradiction+quarantine (the retrieval-time plumbing), J-18 LSP identity (the compiler-grade signature the re-derivation reads), G-1 taint (provenance on the raw-evidence tier). Restates none.

## Open questions / risks
- re-derivation cost: re-checking every fact's semantic identity against the current checkout on every read adds latency — make it SELECTIVE (revalidate on staleness-instrument signal or commit-range change, cache the verdict), the H-30 instrument is the trigger.
- branch scope: "live revalidation over HEAD" can wrongly reject a fact valid for the task's TARGET branch (the p7 B7 critique) — scope memories by repo+branch+commit-range, revalidate against the relevant checkout not only HEAD.
- raw-evidence growth: the immutable tier grows monotonically — needs explicit compaction/namespaces + scoped forensic query (append-only attack surface, p7 note); bound it, log the bound.

## Next
The commit-aware identity J-21 stores is the same compiler-grade signature J-18's LSP channel resolves and J-16's blast-radius cone traverses — the three share one notion of "what a symbol IS", which is what makes memory, retrieval, and impact-analysis agree.

---

## AMENDED CURSOR SCOPE (2026-08-12, recon-verified)

### Landing state (commit 339005b0, ancestor of HEAD — confirmed)

All four J-21 steps shipped as library code on commit `339005b0` with unit
tests and are re-exported from `crates/memory/src/lib.rs`:

- Step 1 — commit-aware semantic validation:
  `.elai_cc/crates/memory/src/semantic_validation.rs` (646 lines)
  Public API: `SymbolIdentity`, `ValidationScope`, `SemanticValidator` trait,
  `SemanticError`, `SemanticValidity`, `extract_stored_identity`,
  `revalidate_semantic` (line 277)
  Re-exported at `lib.rs:117`

- Step 2 — retrieval-time authorization + write quarantine:
  `.elai_cc/crates/memory/src/retrieval_authorization.rs` (916 lines)
  Public API: `Claim`, `Evidence`, `ClaimAuthorization`,
  `QuarantineDecision`, `QuarantineStore`, `WriteQuarantineRecord`,
  `authorize_for_injection` (line 336), `get_authorized_facts` (line 400),
  `quarantine_or_accept` (line 308), `authorization_exclusion` (line 179),
  `NullSemanticValidator`
  Re-exported at `lib.rs:137`

- Step 3 — provenance-linked raw-evidence tier:
  `.elai_cc/crates/memory/src/raw_evidence/mod.rs` (434 lines),
  `store.rs` (332 lines), `escalation.rs` (204 lines),
  `sufficiency.rs` (110 lines)
  Public API: `RawEvidenceStore`, `RawEvidenceRecord`,
  `RawEvidenceEscalation`, `SufficiencyVerdict`, `assess_sufficiency`
  (sufficiency.rs:58), `escalate_to_raw_evidence` (escalation.rs:39),
  `get_authorized_facts_with_escalation` (mod.rs:95)
  Re-exported at `lib.rs:143`

- Step 4 — activation-decay scorer:
  `.elai_cc/crates/memory/src/activation_decay.rs` (834 lines)
  Public API: `ActivationScore`, `ActivationDecayConfig`,
  `ActivationDecayStats`, `ActivationRetrievalResult`, `score_activation`
  (line 189), `collect_activation_expired` (line 240),
  `activation_decay_tick` (line 302), `get_authorized_facts_with_activation_decay`
  (line 398)
  Re-exported at `lib.rs:149`

- Acceptance evidence: `.elai_cc/crates/memory/src/acceptance_evidence.rs`
  (723 lines, FIX-ACCEPTANCE patch)

### Production wiring state (recon-confirmed, current main)

WIRED (Tier-B only, short-circuits before semantic validation runs):
- `report_fns.rs:1359` calls `get_authorized_facts` for Tier-B (Preference)
  facts inside `assemble_user_memory_section` (defined at `report_fns.rs:1299`)
- Tier-B short-circuits to `Valid` before any validator runs
  (`semantic_validation.rs:284-286`: `classify_fact_tier(fact.fact_type) ==
  Tier::B -> return SemanticValidity::Valid`)
- Gate: `ELAI_USER_MEMORY_ENABLE` (default-OFF,
  `wire-dark-allowlist.toml` row, owner=maintainer, expiry=2026-09-01)

WIRED (Tier-A, uses H-32 NOT J-21):
- `report_fns.rs:1422` calls `get_relevant_facts` (H-32) for Tier-A
  (DomainKnowledge + Mistake) facts, NOT `get_authorized_facts` (J-21)
- Tier-A facts do NOT go through J-21 semantic validation in production
- SSB emission for exclusions uses `SsbPayload::FactValidationFailed`
  (H-32, `payloads.rs:3629`, action_type at `action_type.rs:3914`)

DARK (zero production callers outside crates/memory):
- `get_authorized_facts_with_escalation` — 0 production callers
- `escalate_to_raw_evidence`, `assess_sufficiency` — only intra-crate callers
- `RawEvidenceStore` — no production caller outside crates/memory
- `get_authorized_facts_with_activation_decay` — 0 production callers
- `activation_decay_tick` — NOT called from `decay_scheduler.rs:139`
  (scheduler only calls old `decay_tick`)
- `collect_activation_expired`, `score_activation` — only intra-crate callers

MISSING:
- SSB: no `MemoryValidationEvent` variant added; no H-32
  `FactValidationFailed` extension with semantic-vs-existence discriminator.
  Only reference is a doc comment at `semantic_validation.rs:214`
  (`MemoryValidationEvent::SemanticRevalidated` mentioned but not shipped).
  `observable_effect.expect_ssb` clause is NOT satisfiable.
- frontend_pair: 0 dashboard-v2 source references to any J-21 symbol.
  No W-6 justification recorded.

### AMEND VERDICT: 5 concrete deltas needed

The AMEND verdict from recon identified five concrete deltas. This amended
cursor scope refines each into a bounded move with EXPECTED_OBSERVATION,
ON_FAIL, and verification. The original J-21 design and grounding are
preserved; these are wiring/completion deltas, not redesigns.

---

### MOVE 1: Route Tier-A through J-21 authorization

CURSOR: `.elai_cc/crates/cli/src/report_fns.rs:1422`
(inside `assemble_user_memory_section`, the Tier-A retrieval loop at
line 1418)

CHANGE: Replace the `get_relevant_facts` call at `report_fns.rs:1422`
with `get_authorized_facts` (J-21 Step 2). The current code:

```
get_relevant_facts(&store, DEFAULT_USER_ID, &tier_a_filter, MEMORY_RETRIEVAL_K, now_ms)
```

becomes a `get_authorized_facts` call with the same `DkgCitationValidator`
as the existence validator, a `NullSemanticValidator` (or concrete
`SemanticValidator` when available) as the semantic validator, and
`ValidationScope::at_head` for the scope. The return shape changes from
`(Vec<MemoryFact>, Vec<ValidationExclusion>)` to
`(Vec<MemoryFact>, Vec<AuthorizationExclusion>)`. The existing SSB
emission loop at `report_fns.rs:1430-1470` already emits
`FactValidationFailed` per exclusion — the `AuthorizationExclusion`
carries `fact_id`, `reason` (now including `"semantic_stale"` and
`"existence_stale"` discriminators from `ClaimAuthorization::as_reason()`
at `retrieval_authorization.rs:144`), and `evidence_blake3`, which map
directly to the existing `FactValidationFailedPayload` fields.

GROUNDED: The `reason` field in `FactValidationFailedPayload`
(`payloads.rs:19074`) is already free-form `String` with `"stale"` |
`"unknown"` documented values. The J-21 `as_reason()` discriminator
values (`"semantic_stale"`, `"existence_stale"`, `"unknown"`) are
backward-compatible extensions of this field — no schema change needed.
This is the PREFER-REUSE path from the original plan's SSB section:
"extend H-32 FactValidationFailed (210) with a semantic-vs-existence
discriminator if cleaner (net 0)".

EXPECTED_OBSERVATION: A Tier-A fact whose cited symbol was renamed (body
changed, path preserved) is HARD-EXCLUDED by the J-21 semantic gate and
surfaces as a `FactValidationFailed` SSB row with `reason =
"semantic_stale"` — a value that could not appear under H-32's
existence-only check (which only produced `"stale"` or `"unknown"`).

ON_FAIL: If `get_authorized_facts` returns `Err` (e.g. store lock
failure), the Tier-A loop falls through with an empty result and no facts
are injected — this is the existing behavior for `get_relevant_facts`
errors and is fail-closed (R2). If the semantic validator is
`NullSemanticValidator`, all Tier-A facts return `Unknown` and are
hard-excluded — also fail-closed. ABORT: if the return-shape change
breaks the SSB emission loop (field mismatch), STOP and handoff.

---

### MOVE 2: Wire raw-evidence escalation on the Tier-A path

CURSOR: `.elai_cc/crates/cli/src/report_fns.rs:1422` (same site as
MOVE 1, composed after MOVE 1 lands)

CHANGE: After replacing `get_relevant_facts` with
`get_authorized_facts` (MOVE 1), further replace it with
`get_authorized_facts_with_escalation` (J-21 Step 3, `raw_evidence/mod.rs:95`).
This requires constructing a `RawEvidenceStore::new()` (in-memory,
`store.rs:95`) and passing it alongside the `FactStore`. The return shape
adds a third element: `RawEvidenceEscalation`. When the escalation
yields raw-evidence records (sufficiency was `Insufficient`), append a
forensic-evidence section to the assembled memory lines so the prompt
carries the immutable raw evidence that the extracted-fact layer dropped.

GROUNDED: `get_authorized_facts_with_escalation` internally calls
`get_authorized_facts` (Stage 1) then `assess_sufficiency` (Stage 2,
deterministic) then `escalate_to_raw_evidence` (Stage 3). The
`RawEvidenceStore` is an in-memory `Vec<RawEvidenceRecord>` with
namespace-bounded compaction (`MAX_RECORDS_PER_NAMESPACE`, `store.rs`).
No persistence is needed for the read path — the store is populated
intra-call from the excluded facts' evidence_pointers.

EXPECTED_OBSERVATION: When a Tier-A fact is excluded by the semantic
gate AND the authorized set is insufficient (fewer facts than requested
`k`), the `RawEvidenceEscalation` carries the excluded fact's raw
evidence, which is appended to the memory section as a forensic fallback
— the prompt receives the raw evidence that extraction dropped.

ON_FAIL: If `RawEvidenceStore::new()` or the escalation returns empty
(no raw evidence for the excluded facts), the escalation section is
omitted and the memory section contains only the authorized facts —
fail-closed, no regression. ABORT: if the 11-argument call signature of
`get_authorized_facts_with_escalation` cannot be threaded through the
existing `assemble_user_memory_section` without a structural refactor
(e.g. the `RawEvidenceStore` lifetime conflicts with the `FactStore`
borrow), STOP and handoff with the borrow-checker error.

---

### MOVE 3: Wire activation-decay retrieval or scheduler tick

CURSOR: Two independent sites — choose ONE (see FORKS: none below):

(a) RETRIEVAL PATH: `.elai_cc/crates/cli/src/report_fns.rs:1422`
    Replace `get_authorized_facts_with_escalation` (from MOVE 2) with
    `get_authorized_facts_with_activation_decay` (`activation_decay.rs:398`),
    which wraps the escalation call (Stage 1) then scores and re-ranks
    by activation heat (Stages 2-3). The `ActivationDecayConfig::default()`
    (`activation_decay.rs:152`) carries the shipped constants:
    `USE_HALF_LIFE_MS` = 14d, `VALIDATION_HALF_LIFE_MS` = 30d,
    `DEFAULT_MAX_AGE_GUARDRAIL_MS` = 180d,
    `DEFAULT_ACTIVATION_RETENTION_THRESHOLD` = 0.3.

(b) SCHEDULER PATH: `.elai_cc/crates/memory/src/decay_scheduler.rs:139`
    Add `activation_decay_tick` alongside the existing `decay_tick` call
    inside the scheduler loop. The scheduler currently calls only
    `decay_tick(&guard, &HashSet::new(), now_ms)` at line 139.
    `activation_decay_tick` (`activation_decay.rs:302`) takes
    `(&FactStore, &RawEvidenceStore, now_ms, &ActivationDecayConfig)` and
    returns `ActivationDecayStats` — it demotes fixed TTL to max-age
    guardrail and retains past-TTL high-activation facts.

DECISION: MOVE 3(a) retrieval path is the PREFERRED wiring because it
makes the activation-decay scorer reachable from the same production
entrypoint as Moves 1-2 (single integration smoke), and
`get_authorized_facts_with_activation_decay` internally calls
`get_authorized_facts_with_escalation` so it composes Moves 1+2+3 in one
facade. MOVE 3(b) is an alternative if the retrieval-path signature
proves too wide (13 arguments).

EXPECTED_OBSERVATION: A Tier-A fact past its 28d fixed TTL but with high
activation (recently revalidated, recently used) is RETAINED in the
injected context, not dropped — the activation heat score exceeds
`DEFAULT_ACTIVATION_RETENTION_THRESHOLD` (0.3) and the max-age guardrail
(180d) has not expired.

ON_FAIL: If `activation_decay_tick` or the scoring returns an error, the
retrieval falls back to the authorized+escalated set without
re-ranking — no regression (the activation-decay is a ranking overlay,
not a gate). If the scheduler path (b) is chosen and
`activation_decay_tick` panics (e.g. `RawEvidenceStore` not available in
the scheduler context), the scheduler catches the panic per its existing
error boundary and logs it — no crash. ABORT: if neither path can be
wired without a structural refactor (e.g. the scheduler does not have
access to a `RawEvidenceStore`), STOP and handoff.

---

### MOVE 4: SSB — extend H-32 FactValidationFailed with semantic discriminator

CURSOR: `.elai_cc/crates/cli/src/report_fns.rs:1430-1470` (the SSB
emission loop inside the Tier-A retrieval block) and
`.elai_cc/crates/orchestrator/src/ssb/payloads.rs:19074`
(`FactValidationFailedPayload`)

CHANGE: No new SSB variant is needed. The existing
`FactValidationFailed` (H-32, SCHEMA_VERSION 210, `action_type.rs:3914`)
already carries a free-form `reason: String` field
(`payloads.rs:19074`). The J-21 `ClaimAuthorization::as_reason()`
(`retrieval_authorization.rs:144`) already produces the discriminator
values: `"semantic_stale"`, `"existence_stale"`, `"unknown"`,
`"authorized"`. When MOVE 1 replaces `get_relevant_facts` with
`get_authorized_facts`, the `AuthorizationExclusion.reason` field carries
these values directly into the `FactValidationFailedPayload.reason` field
at the SSB emission site.

The only doc update needed: the `reason` field doc at `payloads.rs:19067`
currently says `"stale" | "unknown"`. Update to include
`"semantic_stale" | "existence_stale"` as J-21-extended values. This is
a doc comment change, not a schema change — the field is already
`String` and `deny_unknown_fields` does not constrain field values.

GROUNDED: This is the PREFER-REUSE path from the original plan:
"extend H-32 FactValidationFailed (210) with a semantic-vs-existence
discriminator if cleaner (net 0)". The net SSB variant count is unchanged
(current: 614 entries in `SSB_ACTION_TYPES`, SCHEMA_VERSION = -309 + 614
= 305). No new `SsbActionType` variant, no new `SsbPayload` arm, no
schema bump.

The four J-21 decision sites named in the original plan
(`revalidate_semantic`, `authorize_for_injection`,
`escalate_to_raw_evidence`, `quarantine_or_accept`) emit their verdicts
through the `AuthorizationExclusion` pipeline that surfaces at
`report_fns.rs:1430`. The `reason` discriminator is the semantic-vs-
existence signal. `quarantine_or_accept` verdicts surface as
`reason = "quarantined"` in `AuthorizationExclusion` (set at
`retrieval_authorization.rs:417`). `escalate_to_raw_evidence` verdicts
surface through the `RawEvidenceEscalation` return (MOVE 2), which can
emit a `tracing::warn` with `target = "memory.raw_evidence"` at the
report_fns call site — observable in logs without a new SSB variant.

EXPECTED_OBSERVATION: The SSB DB contains `FactValidationFailed` rows
with `reason = "semantic_stale"` — a value that could not appear under
H-32 (which only produced `"stale"` or `"unknown"`). The off-agent
verifier greps the SSB DB for `reason LIKE 'semantic_stale%'` after a
run that injects a refactored Tier-A fact.

ON_FAIL: If the `reason` field does not propagate (e.g. the
`AuthorizationExclusion` is not mapped correctly to
`FactValidationFailedPayload`), the SSB row carries `reason = "unknown"`
— fail-closed (the fact is still excluded, just with a coarser reason).
ABORT: if the doc update to `payloads.rs` triggers a schema lock test
(e.g. `payload_field_shape_lock.rs` checks field doc content), STOP and
handoff — the gate is frozen to this worker (R-3).

---

### MOVE 5: frontend_pair — W-6-justified absence (DEFERRED)

CURSOR: none (documentation only)

DECISION: The frontend_pair (memory provenance/validation surface on the
memory-hub panel, H-33) is DEFERRED with W-6 justification. The
`ELAI_USER_MEMORY_ENABLE` flag is default-OFF (owner=maintainer,
expiry=2026-09-01, `wire-dark-allowlist.toml`). The J-21 semantic
validation, raw-evidence escalation, and activation-decay are all
backend-only computation that runs under this default-OFF flag. A
dashboard surface for a default-OFF backend feature has no live
telemetry to display until the flag is promoted — building the UI now
would be speculative (W-6: no live user path to observe).

The H-33 `MemoryGovernanceEvent` SSB (`payloads.rs:19120`,
`action_type.rs:3916`) already provides the governance beacon channel.
When `ELAI_USER_MEMORY_ENABLE` is promoted and J-21 validation events
flow through `FactValidationFailed` with semantic discriminators (MOVE
4), the dashboard can render them on the existing memory-hub panel
without a new surface — the `reason` discriminator is the display signal.

W-6 JUSTIFICATION: The frontend_pair is withheld because (1) the backend
feature is gated default-OFF with no live user path, (2) the SSB channel
for governance events already exists (H-33), (3) the `reason`
discriminator from MOVE 4 is the only new display signal and it flows
through the existing `FactValidationFailed` payload, and (4) building a
dashboard surface for a dark feature violates W-6 (no live path to
observe). This is a DEFER, not a SKIP — the frontend_pair lands when
`ELAI_USER_MEMORY_ENABLE` is promoted to default-ON.

EXPECTED_OBSERVATION: No dashboard-v2 source file references any J-21
symbol — confirmed by grep of `crates/dashboard-v2/src/`. The W-6
justification is recorded in this plan file.

ON_FAIL: n/a (documentation only, no code change).

---

### FORKS: none

All five moves have a single defensible path. MOVE 3 has two options
(retrieval path vs scheduler path) but the retrieval path is preferred
and the scheduler path is a fallback — not a material fork (the
orchestrator does not need to decide; the executor picks the retrieval
path and falls back if the signature is too wide, per ON_FAIL).

### RECON_NEEDED: none open

Recon was completed at `/home/user/teikoku/conductor/opus_walk/evidence/J-21/recon.md`.
All file:line evidence has been re-verified against current main (2026-08-12).
The recon identified the AMEND verdict and the five concrete deltas; this
amended scope refines them into bounded moves.

### ABORT_CONDITIONS

- MOVE 1: if the return-shape change from `ValidationExclusion` to
  `AuthorizationExclusion` breaks the SSB emission loop at
  `report_fns.rs:1430-1470` with a field mismatch that cannot be resolved
  by mapping `AuthorizationExclusion` fields to `FactValidationFailedPayload`
  fields — ABORT (handoff with the field mismatch).
- MOVE 2: if the `RawEvidenceStore` lifetime or the 11-argument call
  signature of `get_authorized_facts_with_escalation` cannot be threaded
  through `assemble_user_memory_section` without a structural refactor —
  ABORT (handoff with the borrow-checker error).
- MOVE 3: if neither the retrieval path (13 arguments) nor the scheduler
  path (no `RawEvidenceStore` in scheduler context) can be wired — ABORT
  (handoff with both errors).
- MOVE 4: if the doc update to `payloads.rs:19067` triggers a schema lock
  test failure (the gate is frozen, R-3) — ABORT (handoff with the test
  failure output).
- MOVE 5: n/a (documentation only).
- Any move: if `cargo check` (plain, depot shim) fails after the edit —
  ABORT (handoff with the compiler output). Do NOT retry with
  absolute-path cargo (watchdog kills it).

### VERIFY

The off-agent verifier runs these checks after the executor lands Moves 1-4:

1. `python .claude/tools/production_caller_check.py --files .elai_cc/crates/cli/src/report_fns.rs`
   — verifies that `get_authorized_facts` (or `get_authorized_facts_with_escalation`
   / `get_authorized_facts_with_activation_decay`) is called from a real
   production entrypoint, not just a test. PASS: exit 0, no DARK symbols.

2. `python .claude/tools/production_caller_check.py --files .elai_cc/crates/memory/src/activation_decay.rs`
   — verifies that `activation_decay_tick` and
   `get_authorized_facts_with_activation_decay` have production callers
   (either `report_fns.rs` or `decay_scheduler.rs`). PASS: exit 0.

3. `python .claude/tools/production_caller_check.py --files .elai_cc/crates/memory/src/raw_evidence/mod.rs`
   — verifies that `get_authorized_facts_with_escalation` has a production
   caller. PASS: exit 0.

4. `cargo check` (plain, depot shim) — verifies the edits compile.
   PASS: exit 0.

5. SSB discriminator grep: after a run that injects a refactored Tier-A
   fact, `sqlite3 <ssb-db> "SELECT reason FROM ssb WHERE action_type =
   'fact_validation_failed' AND reason LIKE 'semantic_stale%'"` returns
   >= 1 row. This is the off-agent falsification test: if no
   `semantic_stale` row appears, the J-21 semantic gate is not reachable
   in production (the move failed).

6. `python .claude/tools/plan_gate_check.py --file docs/thefinalglance/J-plan/J-21_memory_semantic_validation_poisoning.md`
   — verifies the plan file carries all wargame markers. PASS: exit 0.

Falsification: if check 1-3 shows a DARK symbol (exit 1), the wiring
move failed — the symbol has no production caller. If check 5 shows no
`semantic_stale` row, the semantic validation is not reachable — the
Tier-A path still uses H-32 existence-only. If check 4 fails, the edit
does not compile.

### REDTEAM

Attack 1 — semantic-poison bypass: an adversary stores a Tier-A fact
with a valid `dkg:` citation pointing at a symbol that exists but whose
body was changed to inject malicious behavior. Under H-32 (existence-
only), this fact is injected. Under J-21 MOVE 1, the semantic validator
re-derives the symbol identity and detects the signature/behavior change
→ `SemanticStale` → hard-excluded. The attack is caught at retrieval,
not at write (MPBench: poison is benign at write).

Attack 2 — raw-evidence escalation abuse: an adversary attempts to flood
the `RawEvidenceStore` with poisoned raw-evidence records to bias the
escalation fallback. Defense: `MAX_RECORDS_PER_NAMESPACE` bounds the
store (`raw_evidence/store.rs`) with FIFO compaction — the adversary
cannot grow the store unboundedly. The escalation is deterministic
(sufficiency gate is a count threshold, not LLM-judged), so poisoned
raw evidence cannot bias a sufficiency decision.

Attack 3 — activation-decay manipulation: an adversary attempts to keep
a poisoned fact alive by inflating its activation score (e.g. repeated
revalidation). Defense: the activation score is computed from
`last_observed_at_ms` + `last_validated_at_ms` + `confidence` (stored
server-side, not client-controllable). The max-age guardrail
(`DEFAULT_MAX_AGE_GUARDRAIL_MS` = 180d) is an absolute ceiling — no
activation score can keep a fact alive past the guardrail.

Attack 4 — quarantine bypass: an adversary attempts to inject a
quarantined fact by exploiting the retrieval path. Defense: the
`QuarantineStore` check at `retrieval_authorization.rs:417` runs BEFORE
`authorize_for_injection` — quarantined facts are excluded before any
validation runs, regardless of the semantic verdict.

Attack 5 — SSB reason spoofing: an adversary attempts to set
`reason = "authorized"` in an `AuthorizationExclusion` to bypass the
hard-exclude. Defense: the `reason` field is set by
`ClaimAuthorization::as_reason()` (server-side, derived from the
validation verdict), not from client input. The SSB row is emitted by
the CLI (not the adversary), and the `FactValidationFailedPayload` is
constructed from the `AuthorizationExclusion` at `report_fns.rs:1430`.

### PRODUCTION_HOOK

PRODUCTION_HOOK: `crates/cli/src/report_fns.rs:1299` —
`assemble_user_memory_section` is the production entrypoint for memory
injection into prompts. Moves 1-3 all wire into this function (the
Tier-A retrieval block at line 1422). The SSB emission loop at line 1430
is the observability seam for MOVE 4. This is a real `elai` run path
(Executor-role prompt assembly), not a test caller (R7).

WIRING_CONTRACT (updated with current file:line evidence):
- produces_symbols: `get_authorized_facts` (retrieval_authorization.rs:400),
  `get_authorized_facts_with_escalation` (raw_evidence/mod.rs:95),
  `get_authorized_facts_with_activation_decay` (activation_decay.rs:398),
  `activation_decay_tick` (activation_decay.rs:302)
- consumes_symbols: `get_relevant_facts` (H-32, the call being replaced at
  report_fns.rs:1422), `FactValidationFailed` SSB (H-32, the payload being
  extended with semantic discriminators at payloads.rs:19074)
- reachable_from: `assemble_user_memory_section` (report_fns.rs:1299) →
  real `elai` run with `ELAI_USER_MEMORY_ENABLE=1` and Executor role
- stub_ban: NO existence-only validation passing as semantic (MOVE 1
  replaces `get_relevant_facts` with `get_authorized_facts`); NO
  write-time-only poisoning defense (retrieval-time authorization +
  quarantine required); NO fixed-TTL as sole forgetting policy
  (activation-decay scorer wired in MOVE 3)

### INTEGRATION_SMOKE

Cadence: single integration smoke after Moves 1-4 land together (they
compose in one facade: `get_authorized_facts_with_activation_decay`
wraps `get_authorized_facts_with_escalation` wraps
`get_authorized_facts`).

Smoke procedure:
1. Set `ELAI_USER_MEMORY_ENABLE=1`
2. Store a Tier-A fact with a `dkg:` citation: `elai remember "function
   foo validates JWT at auth.rs:42" --type DomainKnowledge`
3. Refactor the cited symbol (rename `foo` or change its body)
4. Run `elai` with an Executor-role prompt that would inject the fact
5. Assert: the fact is NOT in the injected context (hard-excluded)
6. Assert: the SSB DB has a `FactValidationFailed` row with
   `reason = "semantic_stale"` (off-agent artifact)
7. Assert: if the authorized set is insufficient, the raw-evidence
   escalation section is appended to the memory context
8. Assert: a past-TTL high-activation fact is RETAINED (not dropped)

NEGATIVE smoke (R2): store a Tier-A fact whose symbol exists AND whose
semantics are unchanged → the fact injects normally (no false-stale).
Point the validator at an unavailable index → the fact reads
`UNVERIFIED` and is hard-excluded (fail-closed, not silent inject).

INTEGRATION_SMOKE: n/a for MOVE 5 (documentation only, no code change).

---

End of J-21.

# J-59: per-seat memory + trust isolation — namespace memory by seat, block cross-seat read/write/poisoning, extend the auditor-objectivity firewall per-seat (the multi-tenant data-foundation + governance slice J-49 deferred)

STATUS: PLANNED (WAVE PV-2; effort M; EXTENSION of J-49 + E-5/H-31/H-32 — per-seat namespacing over the single-user memory substrate; depends J-49; data-driven activation, no new engine)

Objective: J-49 ships multi-seat ECONOMICS only (admission + fair-share + per-seat spend), explicitly defers seat-identity provenance to J-2, and does NOTHING for memory or trust isolation. but the moment ELAI is multi-seat, the single-user memory substrate (E-5 USER MEMORY SUBSTRATE, H-31/H-32 preference + code-grounded fact memory) becomes a cross-tenant leakage + poisoning surface: seat A's preferences and facts would be retrieved into seat B's prompt, and a fact poisoned under one seat would contaminate all. the Five-Pillars data-foundation + governance pillars ("agents don't forgive bad data — they read it and answer confidently wrong"; per-tenant audit/accountability) make this the highest-value multi-seat add, and it is on-brand (a trust/safety gap, not a platform feature). J-59 ships: per-seat memory NAMESPACING keyed by the J-49 seat-id (the dormant UsageTurnEvent.identity, activated by J-49), cross-seat read/write ISOLATION (a seat never reads or writes another seat's rows), cross-seat POISONING block (a fact written under seat A is never injected into seat B's prompt), and the AUDITOR-OBJECTIVITY FIREWALL (H-31, arXiv:2510.09905) extended so auditor/critic/adversary roles get ZERO memory injection regardless of seat. HARD BOUNDARY: NO new memory engine (compose E-5 Mem0 dispatcher + H-31/H-32 retrieval); NO cross-seat sharing even opt-in in v1 (isolation-first; an explicit-share feature is a separate future plan); single-seat default = one implicit "local" seat, behavior byte-identical to pre-J-59 (W-2, data-driven, mirrors J-49); the per-seat audit-VIEW UI is DEFERRED (lower value, and it pushes ELAI toward a multi-tenant-platform identity it has not chosen). GD-1 do-NOT-re-implement: E-5 Mem0 ADD/UPDATE/DELETE/NOOP dispatcher + capture/retrieval; H-31 bi-temporal preference memory + the auditor-objectivity firewall; H-32 code-grounded fact memory; J-49 SeatRegistry + the activated identity field.

Estimated effort: ~1-2 sessions (~8 h). In-process (the namespace key + the retrieval filter + the firewall extension are pure logic over the existing memory store + the J-49 seat-id); a live two-seat contention smoke is RunPod/remote-sidecar-gated (W-10), the isolation legs run at gate time.
Pre-requisites (re-grep at edit, W-7): J-49 SeatRegistry + the activated UsageTurnEvent.identity SHIPPED (the seat-id source); E-5 memory substrate (crates/memory, Mem0 dispatcher, capture + retrieval) SHIPPED — capture+retrieval are dark per feature-map ~449, so J-59 activates them WITH the seat filter; H-31 TIER-B preference memory + the auditor-objectivity firewall SHIPPED; H-32 TIER-A code-grounded fact memory SHIPPED.
Parallel-safe with: J-56, J-58 (disjoint files). Depends J-49.
Source: workflow/reasoning/steal-production-ai-talks-2026-06-20.md (Five-Pillars data-foundation + governance under multi-tenant; the multi-seat verdict that J-49's economics is the thinnest slice). Doctrine: J-0_INDEX.md.

## Context
Why this matters (external evidence, scope-tagged per HD-1): the Five-Pillars data-foundation pillar's load-bearing claim is "data was built for humans and humans are forgiving; agents are NOT — they read the wrong data and answer confidently wrong, and you won't know" — in a multi-tenant deployment the "wrong data" includes ANOTHER seat's memory, which is both a correctness bug (wrong answer) and a privacy/governance breach (one tenant's preferences/facts leaking to another). the governance pillar adds per-tenant accountability ("who's accountable, full context per decision"). [DESIGN, vendor-framework directional.] the AUDITOR-OBJECTIVITY FIREWALL is already grounded in H-31 (arXiv:2510.09905 — auditor/critic/adversary get zero memory injection so the verifier is not biased by stored preferences); J-59 carries that citation, does NOT re-verify or re-implement it, and extends its scope to "per-seat AND role" (an auditor under any seat still gets zero injection). correctness boundary: isolation is a DETERMINISTIC filter on the retrieval + write path keyed by seat-id (the LLM never decides which seat's memory to use, TD-3); the seat-id is an opaque handle from J-49 (J-59 takes it as input and does NOT invent the auth source — that defers to J-2, same boundary J-49 set).

## Existing state (B/C/D/E/F/G/H/I/J coverage — W-7 anchors, nothing from memory)
- USER MEMORY SUBSTRATE [PARTIAL, E-5]: crates/memory, Mem0 ADD/UPDATE/DELETE/NOOP dispatcher + Zep bi-temporal + MINJA poisoning defense; PROD-WIRED on write/contradiction, DARK on capture + retrieval-into-prompt; CLI memory dispatch is a stub (feature-map ~449-454, main.rs:1064-1069). J-59 adds the seat key to the capture + retrieval it activates.
- TIER-B PREFERENCE MEMORY [PARTIAL, H-31]: bi-temporal supersession, retrieval HARD-FILTERS superseded rows, ContextSlot::Memory; AUDITOR-OBJECTIVITY FIREWALL — auditor/critic/adversary get ZERO memory injection (arXiv:2510.09905) (feature-map ~458-463). J-59 extends the firewall to per-seat-AND-role and the retrieval hard-filter to per-seat.
- TIER-A CODE-GROUNDED FACT MEMORY [PARTIAL, H-32]: validate-citation-before-use on DKG, 28d TTL (feature-map ~464). Per-seat fact rows respect the seat key.
- J-49 multi-seat admission [PLANNED, prerequisite]: SeatRegistry (seats.toml) + the activated UsageTurnEvent.identity — the seat-id source J-59 keys on. seat-identity provenance defers to J-2 (J-59 inherits that boundary).
- SAFEHARBOR MEMORY GUARDRAIL [PARTIAL, C-48]: TrustTier T0-T3 on memory writes (feature-map ~528). J-59 makes the trust tier per-seat (a seat's writes carry that seat's tier).

## Reads (mandatory before coding)
1. workflow/reasoning/steal-production-ai-talks-2026-06-20.md (the multi-seat verdict + the data-foundation/governance framing).
2. docs/thefinalglance/J-plan/J-49_multi_seat_admission_quota.md (the SeatRegistry + the activated identity field this keys on).
3. crates/memory (the E-5 Mem0 dispatcher + the capture + retrieval-into-prompt path — re-grep); H-31 retrieval hard-filter + the auditor-objectivity firewall site; H-32 fact-validation.
4. the ContextSlot::Memory injection site (where retrieved memory enters the prompt — the firewall + the seat filter both live here).
5. CLAUDE.md: TD-3 (deterministic seat filter), W-2 (data-driven activation, no enable-flag), arXiv:2510.09905 carried-from-H-31 (do NOT re-verify), no-.unwrap-in-prod, blake3 (seat-keyed token hashes if any).

## Tasks (WAVE PV-2 — per-seat namespacing + isolation + firewall extension)
1. Per-seat namespace key (~70 LOC): every memory row (preference H-31, fact H-32) carries the J-49 seat-id; the write path (Mem0 ADD/UPDATE) stamps the current seat-id; absent SeatRegistry => the implicit "local" seat => byte-identical to pre-J-59 (W-2). NOT: a new memory store; NOT: a seat-id the LLM can set.
2. Cross-seat retrieval isolation (~80 LOC): the retrieval-into-prompt filter (H-31 hard-filter site) gains a seat predicate — only the current seat's rows are eligible; another seat's row is structurally unreachable (the poisoning block: a fact written under seat A can never enter seat B's prompt). NOT: an LLM-mediated "is this the right seat" check (deterministic filter, TD-3); NOT: a soft de-prioritization (hard exclusion).
3. Auditor-objectivity firewall, per-seat-AND-role (~40 LOC): extend the H-31 firewall so auditor/critic/adversary roles get ZERO memory injection under ANY seat (the existing role check + the new seat key compose — the firewall is role-gated first, seat-gated second). NOT: re-implementing the firewall (compose H-31); NOT: letting a seat's auditor see that seat's memory.
4. Per-seat trust tier (~30 LOC): C-48 TrustTier becomes per-seat — a seat's writes carry that seat's tier (from seats.toml), so a low-trust seat cannot poison a high-trust seat even within its own namespace boundary. NOT: a global tier.
5. SSB: prefer-reuse H-33 MemoryGovernanceEvent with a per-seat discriminator (or J-49 SeatAdmissionEvent) — decide at edit, net 0..+1; the row carries seat-id + the isolation action (write-stamped / retrieval-filtered / firewall-blocked), NEVER the memory content or a seat secret. NOT: a new action_type if H-33 fits.
6. Decision doc: workflow/reasoning/J-59-decisions.md.

## WIRING_CONTRACT
produces_symbols:
  - the per-seat namespace key on memory rows (write path stamps seat-id)
  - the cross-seat retrieval isolation predicate (H-31 hard-filter site += seat gate)
  - the per-seat-AND-role auditor-objectivity firewall extension
  - per-seat TrustTier on the C-48 write guard
consumes_symbols:
  - J-49 SeatRegistry + UsageTurnEvent.identity (the seat-id source; GD-1 do NOT re-implement the registry)
  - E-5 Mem0 dispatcher + capture + retrieval (the substrate this seat-keys; GD-1 do NOT re-author the store)
  - H-31 retrieval hard-filter + the auditor-objectivity firewall (arXiv:2510.09905) (extended, not replaced; GD-1)
  - H-32 code-grounded fact memory (per-seat fact rows; GD-1)
  - C-48 TrustTier (made per-seat; GD-1)
  - H-33 MemoryGovernanceEvent (the SSB row, prefer-reuse; GD-1)
reachable_from:
  - the memory retrieval-into-prompt path (ContextSlot::Memory injection — PRODUCTION; the seat filter runs on every prompt build)
  - the memory write path (Mem0 ADD/UPDATE — PRODUCTION; stamps seat-id on every write)
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO cross-seat read or write (a seat's memory is structurally unreachable from another seat — the poisoning block)
  - NO auditor/critic/adversary memory injection under ANY seat (H-31 firewall extended, never relaxed)
  - NO LLM-mediated seat selection (deterministic seat filter, TD-3 — the LLM cannot set or pick the seat)
  - NO new memory engine (compose E-5/H-31/H-32)
  - NO cross-seat sharing path in v1 (isolation-first; explicit-share is a separate future plan)
  - NO new ELAI_*_ENABLE (multi-seat memory activates by the J-49 seats.toml presence, W-2)
  - NO memory content or seat secret in the SSB payload (seat-id is an opaque handle)
  - NO .unwrap()/.expect() on the retrieval / write prod path
frontend_pair:
  N/A with justification — per-seat memory has no new dashboard surface in v1 (the per-seat audit-VIEW is explicitly DEFERRED); the isolation is a backend invariant proven by the contract test + the SSB rows, not a rendered view. Justified.
observable_effect:
  trigger: "two seats (seats.toml) write contradictory preferences via the memory write path; then build a prompt for seat B; then run an auditor turn under seat A (RunPod/remote-sidecar fixture for the live two-seat case; an in-process fixture drives the write+retrieve+firewall legs at gate time)"
  expect_ssb: "MemoryGovernanceEvent (or reuse) rows showing seat-A and seat-B writes stamped with distinct seat-ids, a retrieval for seat B that filtered out seat A's row, and a firewall-block on the seat-A auditor turn — never any memory content in the payload"
  expect_artifact: "seat B's built prompt contains ZERO of seat A's facts/preferences; the auditor turn's context contains ZERO memory rows. NEGATIVE that must FAIL: a cross-seat fact leak into another seat's prompt; an auditor/critic/adversary receiving any memory under any seat; absent seats.toml changing single-seat behavior."

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/J-plan/J-59_per_seat_memory_trust_isolation.md --run-trigger
rg -n "seat_id|seat_key|per_seat|firewall" crates/memory   # the seat key + firewall extension exist
rg -c "ELAI_PER_SEAT_MEMORY_ENABLE" crates/memory           # MUST be 0 (W-2 data-driven via seats.toml)
rg -c "unwrap\(\)|expect\(" crates/memory/src/<retrieval+write paths>   # MUST be 0 (prod path)
test -f workflow/reasoning/J-59-decisions.md
```

## ACCEPTANCE
- [ ] ISOLATION: a seat never reads or writes another seat's rows; a cross-seat read/write is structurally impossible (contract-tested both directions).
- [ ] POISONING BLOCK: a fact written under seat A never enters seat B's prompt (contract-tested).
- [ ] FIREWALL: auditor/critic/adversary get zero memory injection under ANY seat (H-31 firewall extended; contract-tested negative arm).
- [ ] W-2: per-seat memory activates by seats.toml presence; absent => single "local" seat, byte-identical to pre-J-59 (rg ENABLE count 0).
- [ ] TD-3: the seat filter is deterministic; the LLM cannot set or pick the seat.
- [ ] R7/D-030: the seat filter + write-stamp reachable from the production retrieval + write paths, not a test.
- [ ] GD-1: J-49 registry / E-5 store / H-31 firewall+filter / H-32 facts / C-48 tier / H-33 event consumed, not restated.
- [ ] cargo check --workspace GREEN; auditor PASS on -p memory (+ orchestrator if the injection site lives there).
- [ ] No new dependency (seats.toml is the J-49 toml).

## WIRING DONE-GATE (clause by clause)
- W-1: entrypoint = the memory retrieval-into-prompt + write paths; artifact = the MemoryGovernanceEvent (or reuse) isolation rows. Met.
- W-2: data-driven activation by seats.toml, no enable-flag graveyard. Met.
- W-3: the multi-seat verdict in-repo (steal doc). Met.
- W-4: measures the objective (seat B's prompt provably excludes seat A's rows; the auditor context provably has zero memory), not stdout. Met.
- W-5: production_caller_check; the seat filter + write-stamp called from the real prompt-build + write paths, not a test. Met.
- W-6: N/A (per-seat audit view DEFERRED; isolation is a backend invariant). Justified.
- W-7: J-49/E-5/H-31/H-32/C-48/H-33 anchors re-grepped at edit.
- W-9: SCHEMA baseline re-read at edit (prefer-reuse H-33 => net 0..+1).
- W-10: isolation legs in-process; the live two-seat contention smoke is RunPod/remote-sidecar-gated. Tiered.
- GD-4: the firewall + isolation default fail-closed (a row whose seat-id is unresolved is EXCLUDED, never injected). GD-5 honest residue: seat-identity PROVENANCE (where a real seat-id comes from on a multi-client path) defers to J-2 — J-59 takes it as input and isolates correctly given a correct id, but cannot guarantee the id's authenticity (named, same boundary J-49 set).

## Tests to add
- crates/contracts/tests/j59_per_seat_memory_contract.rs (~110 LOC): (a) absent seats.toml => single "local" seat, retrieval + write identical to pre-J-59; (b) two seats write contradictory prefs; seat B's prompt excludes seat A's row; (c) a cross-seat read attempt returns nothing (structural); (d) a cross-seat write attempt is rejected; (e) an auditor turn under each seat gets zero memory injection; (f) a low-trust seat's write cannot escalate another seat's tier; (g) the SSB row never carries memory content or a seat secret.

## Files touched
- MODIFY: crates/memory (the Mem0 capture + retrieval paths += seat key + filter) (~150 LOC) — re-grep the exact files at edit
- MODIFY: the H-31 retrieval hard-filter + auditor-objectivity firewall site (+= seat gate) (~40 LOC) — re-grep
- MODIFY: the C-48 TrustTier write guard (per-seat) (~30 LOC) — re-grep
- MODIFY: crates/.../ssb.rs (reuse H-33 MemoryGovernanceEvent + per-seat discriminator; or +1 arm) (~30 LOC)
- CREATE: crates/contracts/tests/j59_per_seat_memory_contract.rs (~110 LOC)
- CREATE: workflow/reasoning/J-59-decisions.md (~30 LOC)

## Citations / Grounding
- Source: workflow/reasoning/steal-production-ai-talks-2026-06-20.md (Five-Pillars data-foundation + governance under multi-tenant; the multi-seat thinnest-slice verdict). [DESIGN, vendor-framework directional.]
- ELAI ground truth (PLAN-asserted, executor re-greps per W-7): J-49 SeatRegistry + UsageTurnEvent.identity; E-5 Mem0 dispatcher + capture/retrieval (main.rs:1064-1069 stub); H-31 retrieval hard-filter + auditor-objectivity firewall; H-32 fact memory; C-48 TrustTier; H-33 MemoryGovernanceEvent.
- arXiv:2510.09905 (auditor-objectivity firewall) — CARRIED from H-31 where it is already grounded; NOT re-verified, NOT re-implemented (compose).
- CLAUDE.md: TD-3 (deterministic seat filter), W-2 (data-driven activation), no-.unwrap-in-prod, blake3.
- LIVE SCHEMA baseline ~268/577 (Tier-B derived, 2026-06-15); prefer-reuse H-33 MemoryGovernanceEvent => net 0..+1; re-read at edit (R-c/W-9).

## Open questions / risks
- The exact crates/memory file layout for capture + retrieval (dark per feature-map) — PLAN-asserted; executor re-greps at EXECUTING (W-7).
- SSB: reuse H-33 vs a dedicated arm — decide at edit (prefer-reuse; net 0..+1).
- Seat-identity provenance defers to J-2 (named in J-49 too): J-59 isolates correctly given a correct seat-id but does not authenticate it — the authenticity is J-2's / a future plan's job, stated so it is not silently assumed.
- Honest residue (GD-5): v1 is isolation-ONLY — there is deliberately no cross-seat sharing path; if a real multi-seat workload needs controlled sharing, that is a separate future plan, never a relaxation of J-59's hard exclusion.

## Next
J-59 makes ELAI's memory multi-tenant-safe: per-seat namespacing + hard cross-seat isolation + the auditor firewall per-seat, turning J-49's economics-only multi-seat into a trust-correct one. It pairs with J-49 (the seat-id source) and J-2 (the control plane that would authenticate real seats). The per-seat audit-VIEW UI and any controlled cross-seat sharing are explicitly deferred to keep ELAI's single-user-trust-spine identity intact while closing the leakage/poisoning gap.

End of J-59.

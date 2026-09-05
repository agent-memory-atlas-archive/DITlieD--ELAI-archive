# DEEP-P-003 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1285-L1297`, heading `### DEEP-P-003`; the next chunk starts at L1299 (`### DEEP-Q-001`).
- Declared chunk: family P, **7 files**, **1,035 logical lines**, **153,893 bytes**, all seven classified `numbered/index original`; target status **under 1,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all seven listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `16-P-SUMMARY_INDEX.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `P-3:Lx-Ly`, `P-15:Lx-Ly`, `P-17:Lx-Ly`, `P-20:Lx-Ly`, `P-22:Lx-Ly`, `P-23:Lx-Ly`, and `P-25:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 93 | 11,657 | 4ac4764e4c6ab3881e92598b5b15f0d6aabf96bfa3f17e2b955cbc6fd49495bb | docs/thefinalglance/P-plan/P-3_persona_split_export.md | exact |
| numbered/index original | 77 | 9,049 | c15922c9dd2b136af80d5f61fac6cc43d7d81b383cd7d8de23ccad840b1d474c | docs/thefinalglance/P-plan/P-15_verified_case_graph_hardening.md | exact |
| numbered/index original | 78 | 9,351 | a76a09561afff042a200cec0af05b0a3f1bf5c868905a9752e3dca2047b0ed89 | docs/thefinalglance/P-plan/P-17_provider_failover_economics.md | exact |
| numbered/index original | 77 | 8,801 | 8ba700189493b76a89e2f7d8c4f31b731eb9df0104b66b33c8fead5bf52c69df | docs/thefinalglance/P-plan/P-20_context_memory_lifecycle.md | exact |
| numbered/index original | 78 | 9,265 | c7a0e27a3d10a0ac7a8e3bf0b519da30daa6327fbf0d0f15a883ca2ab1cb1d6c | docs/thefinalglance/P-plan/P-22_control_plane_channel_hardening.md | exact |
| numbered/index original | 84 | 13,182 | 6f6f08489b8d4412276b0c787a872ed2cb8a0c1604b1c83c641635ea50fbc59d | docs/thefinalglance/P-plan/P-23_journaled_cobrowse_lane.md | exact |
| numbered/index original | 548 | 92,588 | f9bcc1b23d664ea03560d42c57f5bb7794a0a01d2587870882f3e9e740ca294f | docs/thefinalglance/P-plan/P-25_exogenous_injection_creativity_falsifier.md | exact |
| **Aggregate** | **1,035** | **153,893** | — | **7 exact paths** | **exact** |

## Per-file atomic ledgers

### P-3 — persona/mechanism split + hand-this-box-to-a-friend export

Source: `docs/thefinalglance/P-plan/P-3_persona_split_export.md` (93/93 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P3-E001 | E/STATUS | P-3 is PLANNED Wave 3, PERSONAL-INCARNATION, lever mechanism-vs-persona-factoring, MODERATE grounding, effort M; nature is bundle=build and factoring=doctrine; depends on P-1 manifest and P-2 ports; PROOF_SPINE none (product-surface/distribution class), owner-acked 2026-07-07. | P-3:L1-L5 |
| P3-E002 | E/CAP | Premise: untransferability is a factoring failure, not a law of nature; every built tool is factored MECHANISM (code, tests, gates) vs PERSONA (rules rows, memory namespaces, thresholds, aesthetic tokens — all DATA). | P-3:L7-L7 |
| P3-E003 | E/DATA | AppBundle shape is `{ mechanism: (code + tests + gates), manifest (P-1 + P-2 ports), rra_receipt (L-14/G-2 lineage), provenance: G-9-redacted }`. | P-3:L10-L10, L20-L22, L54-L54 |
| P3-E004 | E/DATA | PERSONA layer (rules rows, memory namespaces, thresholds, aesthetic tokens) is EXCLUDED from the bundle by construction and regenerated on the recipient seat. | P-3:L7-L11, L54-L54, L63-L63 |
| P3-E005 | E/SURFACE | CLI verb `elai app export <app>` writes the bundle plus a receipt. | P-3:L10-L10, L22-L22, L27-L27, L32-L32, L55-L55, L60-L60, L72-L72 |
| P3-E006 | E/SURFACE | CLI verb `elai app import <bundle>` is the Tier-1 import production caller. | P-3:L10-L10, L22-L22, L27-L27, L34-L35, L55-L55, L60-L60, L72-L72 |
| P3-E007 | E/FLOW | Import runs M-5 elai-verify against the bundle evidence, refuses on a failed receipt, then mounts the mechanism. | P-3:L10-L11, L15-L15, L22-L22, L34-L36, L64-L64 |
| P3-E008 | E/SEC | G-9 write-time secret/PII redaction is mandatory on the export provenance leg; unredacted provenance is banned. | P-3:L10-L11, L17-L17, L22-L22, L31-L33, L65-L65 |
| P3-E009 | E/SEC | Sharing surface is admission-gated: every artifact carries a receipt and passes the same L-12/skillspector admission gates used for skills; friend-to-friend and marketplace share one gate. | P-3:L10-L10, L16-L16, L23-L23 |
| P3-E010 | E/LIMIT | EXCLUDED: exporting the persona layer; an unreceipted/unredacted bundle; a bundle that mounts WITHOUT M-5 re-verification (A-import-must-verify); a per-recipient code fork (mechanism shared verbatim; only persona differs — A-5). | P-3:L11-L11, L63-L66 |
| P3-E011 | E/DEP | PLAN-asserted ground: FS-1/L-12/L-13 data extension; M-5 verifier + M-6 evidence format; L-12 + skillspector admission; G-8/G-9 redaction; L-14 RRA + G-2 lineage; P-1 manifest; P-2 ports. | P-3:L13-L19, L57-L58, L90-L91 |
| P3-E012 | E/DEFER | Tier-2 deferred: persona regeneration against importer E-5 memory + P-7 profile seam; friend-to-friend/marketplace sharing surface (lands with sharing wiring pass shared with P-2 Tier-2). | P-3:L21-L23, L28-L28, L37-L39 |
| P3-E013 | E/INT | Integration targets orchestrator (or knowledge) AppBundle serializer/deserializer + export/import flow + M-5 verify call + G-9 redaction pass. | P-3:L25-L28, L54-L56 |
| P3-E014 | E/OBS | Export emits journaled `app_export` SSB row; import emits `app_import` (refusal without mount, or mount on clean). | P-3:L35-L35, L56-L56, L68-L70 |
| P3-E015 | E/TEST | M1: export writes persona-absent, G-9-redacted provenance; un-redactable secret in provenance is hard refusal. | P-3:L31-L33 |
| P3-E016 | E/TEST | M2: valid bundle re-verifies GREEN and mounts (`app_import` SSB); tampered mechanism or stale receipt is REFUSED with no mount. | P-3:L34-L36 |
| P3-E017 | E/TEST | M3 Tier-2: after import, thresholds/rules/tokens reflect IMPORTER memory, not sender; regeneration is journaled and revertible. | P-3:L37-L39 |
| P3-E018 | E/RECOVERY | FORK: if M-5 cannot yet verify a whole-app bundle, ship Tier-1 export + import-with-receipt-presence-check (refuse absent/malformed receipt), route full evidence re-verification to Tier-2; never ship a mount that skips verification entirely. | P-3:L41-L42 |
| P3-E019 | E/LIMIT | ABORT: if factoring requires per-recipient CODE changes rather than data-only persona swap, stop and flag as substrate gap (A-5 two-masters); never ship a per-recipient fork. | P-3:L49-L50, L66-L66 |
| P3-E020 | E/INT | Production reachability is live `elai app export` / `elai app import` CLI; `#[cfg(test)]` never satisfies R7. PRODUCTION_HOOK is `cli/main.rs` subcommand dispatch. | P-3:L59-L61, L72-L72 |
| P3-E021 | E/TEST | VERIFY: auditor runs wiring_contract_check + production_caller_check off-agent; pass requires `app_export` on real export with persona-free G-9-redacted bundle, tampered import refused, clean import mounted. | P-3:L74-L78 |
| P3-E022 | E/TEST | INTEGRATION_SMOKE R10: independent agent exports real app with planted secret + persona, imports on second simulated seat, tampers between export/import (must refuse), re-imports clean (must mount+verify), asserts no sender persona. | P-3:L80-L80 |
| P3-E023 | E/SEC | REDTEAM failed attack: post-signing mechanism edit is refused by M-5 re-verify. Succeeded-then-patched attack: API-key leak via provenance, patched by mandatory G-9 export-path pass. | P-3:L82-L83 |
| P3-E024 | E/DEP | Composes with P-1/P-2, M-5/M-6, L-12, G-8/G-9, FS-1/L-13, E-5 + P-7 (Tier-2); re-implements NONE (GD-1); P-7 owns profile engine; P-6 domain labels ride in ports. | P-3:L90-L91 |
| P3-E025 | E/EVIDENCE | Grounding cites design digest GAP-3/T5/A-5/A-2, reddit thread archive (heynoswearing, Kind_Silver_1921, skerit), and feature-map M-5/M-6/L-12/G-8/G-9/FS-1/L-14. | P-3:L85-L88 |
| P3-I001 | I/STATE | Tier-1 import mounts mechanism without persona regen; recipient runs mechanism under default/empty persona until Tier-2 lands. | P-3:L21-L23, L34-L39 |
| P3-A001 | A/DATA | Exact AppBundle serialization version, field schema, signature envelope, and on-disk packaging are not specified beyond the structural outline. | P-3:L10-L10, L54-L54 |
| P3-A002 | A/CFG | Which concrete files/namespaces constitute the persona exclusion set remains RECON_NEEDED. | P-3:L47-L47, L7-L7 |
| P3-A003 | A/DEP | M-5 whole-bundle verification grain (changeset vs packaged artifact) is RECON_NEEDED and may force the presence-check FORK. | P-3:L41-L45 |
| P3-A004 | A/SURFACE | Friend-to-friend/marketplace hosting UX is deferred; no Tier-1 UI beyond CLI is named. | P-3:L23-L23, L27-L27 |
| P3-C001 | C/DEFER | Opening premise describes full import+regenerate flow, but REVISED scope ships Tier-1 without persona regen; regen is Tier-2. | P-3:L7-L10, L21-L23 |

Category coverage: capabilities/CLI ✓; workflows/states ✓; schemas/data ✓; flags/defaults △; integrations/events ✓; security/redaction/admission ✓; failures/refusal ✓; determinism/idempotency △; observability/SSB ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No formal retry/concurrency API. Privacy is via G-9 redaction, not a separate privacy lattice in this unit.

### P-15 — verified case-graph hardening (gated agent-s memory loop + journal-row bBoN)

Source: `docs/thefinalglance/P-plan/P-15_verified_case_graph_hardening.md` (77/77 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P15-E001 | E/STATUS | P-15 is PLANNED Wave CU-3, COMPUTER-USE-STEAL, lever gate-the-save-not-every-run, STRONG grounding, EXTENSION of closed J-96, effort M; PROOF_SPINE feeds PS-5; execute after proof spine per A-7. | P-15:L1-L5 |
| P15-E002 | E/CAP | Port agent-s S2 KnowledgeBase retrieve/distill/fuse loop onto J-96, but GATE the save behind mint-on-proof; agent-s saves every run ungated (the gap this closes). | P-15:L6-L9, L18-L21 |
| P15-E003 | E/CAP | Tier-1 SHIP NOW: retrieve/distill/fuse loop with save gated by J-96 mint-on-proof; reuse-hit field on J-96 case row (net 0..+1); loop default-off behind J-96 strict reuse filter. | P-15:L18-L20, L30-L34, L41-L41 |
| P15-E004 | E/DEFER | Tier-2 DEFER: bBoN behavior-narrative comparative judge over journal rows (tie-breaker); episodic/narrative two-tier split; embedding-retrieval tuning. | P-15:L20-L20, L31-L31 |
| P15-E005 | E/LIMIT | EXCLUDED: agent-s ungated KnowledgeBase commit; bBoN judge sourced from screenshots (ELAI has ground-truth journal deltas). | P-15:L21-L21, L39-L40 |
| P15-E006 | E/DEP | Extends J-96 CaseNode/MintOnProof; composes K-18 recurrence-hit-rate, K-19 gate-the-save discipline, I evidence-ranked search (receipts first, narrative breaks ties), F-4 PROVEN case node. | P-15:L11-L16, L32-L34, L68-L68 |
| P15-E007 | E/CAP | Produces `crates/orchestrator/src/case_graph/reuse_loop.rs` and `bbon_judge.rs` (bBoN module named even though Tier-2). | P-15:L29-L31 |
| P15-E008 | E/INT | Production reachability: orchestrator reuse path behind J-96 execution-grounded filter; test/contracts callers never satisfy R7. | P-15:L35-L37, L71-L72 |
| P15-E009 | E/SURFACE | frontend_pair rides J-96 memory CanvasMode node chrome; reuse-hit surfaced on the case node (W-4). | P-15:L43-L44 |
| P15-E010 | E/SURFACE | CLI trigger surface: `elai case reuse --replay tests/fixtures/case/recurrence_bugclass_a.json`. | P-15:L46-L46, L56-L56 |
| P15-E011 | E/EVENT | Expected SSB: `recurrence_hit_rate_measured`; K-18 bench `elai bench case-reuse` consumes reuse-hit. | P-15:L47-L48, L71-L72 |
| P15-E012 | E/OBS | Distill/fuse retrieves case; unproven run summary is NOT saved; reuse-hit lands on case row and K-18 rate reflects it. | P-15:L47-L48, L51-L52 |
| P15-E013 | E/FAIL | FORK: if retrieved case no longer reproduces (fix rotted), route to re-exploration + re-mint; record staleness on the case row. | P-15:L53-L53 |
| P15-E014 | E/LIMIT | ABORT if reuse-into-prompt fires without J-96 execution-grounded filter (default-off); un-gameable stronger-than-generator filter is DGM spine theorem. | P-15:L41-L41, L55-L55 |
| P15-E015 | E/TEST | VERIFY proof is off-agent recurrence_hit_rate_measured + absence of case node for unproven fixture via real `elai case reuse`; plan_gate_check + wiring_contract_check. | P-15:L56-L60 |
| P15-E016 | E/SEC | REDTEAM: poisoned case reuse defeated by mint-on-proof + strict reuse filter; bBoN narrative sway patched by scoring journal ROWS not model prose. Deep red-team deferred to execution pre-flight. | P-15:L61-L61 |
| P15-E017 | E/TEST | INTEGRATION_SMOKE R10: hard multi-turn recurrence asserts proven fix mints, unproven mints nothing, recurrence reuses behind filter and moves K-18 rate; shares CU-track R10 cadence with P-14. | P-15:L63-L64 |
| P15-E018 | E/LIMIT | Production bans: NO ungated save; NO screenshot bBoN; NO reuse-into-prompt without filter; NO `.unwrap()`/`.expect()`; NO blake2/sha2. | P-15:L38-L42 |
| P15-E019 | E/EVIDENCE | Source: canvas-computer-use-eval ENTRY 11 agent-s arXiv:2410.08164/2504.00906/2510.02250 verified. | P-15:L8-L9, L66-L67 |
| P15-I001 | I/STATE | `bbon_judge.rs` is listed in produces_symbols for the unit even though bBoN is Tier-2 deferred; Tier-1 may land an empty/stub path only if wiring forbids dark symbols, else the module is deferred with the feature. | P-15:L20-L20, L30-L31 |
| P15-A001 | A/CFG | Exact retrieve/distill/fuse API shape, embedding model, tier schemas, fusion policy, and promotion/on-switch criteria for default-off remain open (RECON_NEEDED + summary A02). | P-15:L54-L54, L19-L20 |
| P15-A002 | A/DATA | reuse-hit field type/schema beyond net 0..+1 and staleness recording fields are not fully specified. | P-15:L19-L19, L53-L53 |
| P15-C001 | C/DEFER | Wargame EXPECTED_OBSERVATION asserts bBoN ranks candidates, but REVISED scope defers bBoN to Tier-2; Tier-1 acceptance must not require bBoN rows. | P-15:L20-L20, L51-L51 |

Category coverage: capabilities/CLI/UI ✓; workflows/states ✓; data/lifecycle ✓; config/defaults ✓ (default-off); integrations/events ✓; security (poisoned-case) ✓; failures/abort ✓; observability ✓; tests/gates ✓; constraints ✓; dependencies/deferred ✓. No formal privacy/compliance rule; no retry/idempotency key API.

### P-17 — provider failover semantics + honest routing economics

Source: `docs/thefinalglance/P-plan/P-17_provider_failover_economics.md` (78/78 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P17-E001 | E/STATUS | P-17 is PLANNED Wave OC-1, OPENCLAW-STEAL, lever failover-semantics-package, STRONG grounding, EXTENSION of J-28/I-50/Q-4/B-37, effort M; PROOF_SPINE feeds PS-4. | P-17:L1-L6 |
| P17-E002 | E/CAP | Port openclaw failure-semantics package into J free-API rotation pool + I-50 auth broker; never-merge billed-truth-with-estimate; honest failover preconditions PS-4 $/verified-task. | P-17:L6-L9, L19-L22 |
| P17-E003 | E/CFG | Rate-limit cooldown ladder: 30s -> 1m -> 5m (own lane). | P-17:L9-L9, L20-L20 |
| P17-E004 | E/CFG | Billing failure cooldown: disabled lane 5h -> 24h (own lane). | P-17:L9-L9, L20-L20 |
| P17-E005 | E/CFG | Auth-permanent cooldown: 10-60m range (own lane). | P-17:L9-L9, L20-L20 |
| P17-E006 | E/CFG | Cooldowns are MODEL-scoped so sibling models still try; also distinguish profile-scoped cooldowns. | P-17:L9-L9, L20-L20, L40-L40 |
| P17-E007 | E/STATE | Session-stickiness for cache warmth: profile pinned per session; rotation only on reset/compaction/cooldown; rotation is a journaled cache-invalidation event (composes H cache-stable layout). | P-17:L9-L9, L20-L20 |
| P17-E008 | E/FLOW | persist-override-BEFORE-retry closes the live-switch race. | P-17:L9-L9, L20-L20 |
| P17-E009 | E/SEC | strict-explicit-selection: user explicit model choice fails loudly, never silently substituted; substitution is Q-4 recorded event if user opted in. | P-17:L9-L9, L20-L22, L32-L32, L41-L41, L54-L54 |
| P17-E010 | E/DATA | J-28 UsageTurnEvent extended with typed billed + estimate fields end-to-end; never merged on any surface. | P-17:L12-L12, L20-L22, L34-L34, L42-L42 |
| P17-E011 | E/CAP | Produces `crates/providers/src/failover.rs::FailoverPolicy` and `strict_selection.rs` explicit-selection guard. | P-17:L30-L32 |
| P17-E012 | E/DEP | Consumes UsageTurnEvent (J-28) and AuthBroker (I-50); B-37 fallback chains consume; K-2 routing-economics bench is the PS-4 consumer. | P-17:L11-L17, L33-L35, L67-L69 |
| P17-E013 | E/INT | Production path: `crates/providers/src/dispatch.rs` invokes FailoverPolicy on every provider call; test callers never satisfy R7. | P-17:L36-L38, L72-L73 |
| P17-E014 | E/SURFACE | frontend_pair: failover state + billed-vs-estimate label on usage surface (RuntimePanel / CloudPanel; W-4). | P-17:L44-L45, L49-L49 |
| P17-E015 | E/SURFACE | CLI/test trigger: `elai provider failover-test --fixture tests/fixtures/providers/billing_then_ratelimit.json`. | P-17:L47-L47 |
| P17-E016 | E/EVENT | Expected SSB: `provider_failover_event` per event with correct lane + cooldown. | P-17:L48-L49, L52-L52 |
| P17-E017 | E/FAIL | NEGATIVE fails: billing failure cools whole provider; silent substitution of explicit model; merged billed/estimate number. | P-17:L40-L42, L49-L49, L52-L53 |
| P17-E018 | E/FLOW | FORK: if user-pinned model enters cooldown, DO NOT silently substitute — surface strict failure or Q-4-recorded swap-restart if opted in; retry only within pinned model ladder. | P-17:L54-L54 |
| P17-E019 | E/DEFER | Tier-2: full ~140-plugin auth-profile store; auto-override reprobe-origin-every-5min loop. | P-17:L21-L21 |
| P17-E020 | E/LIMIT | EXCLUDED vaporware: silent substitution of explicit model; merged billed/estimate number on any surface. | P-17:L22-L22 |
| P17-E021 | E/TEST | VERIFY: off-agent provider_failover_event rows with correct lanes + separate typed billed/estimate via real dispatch; plan_gate_check + wiring_contract_check. | P-17:L57-L61 |
| P17-E022 | E/TEST | INTEGRATION_SMOKE R10: hard multi-turn across 422 leaked-field, billing, and rate-limit; asserts per-lane cooldowns, sibling survival, no silent substitution, separate fields into K-2. | P-17:L64-L65 |
| P17-E023 | E/SEC | REDTEAM: billing error on every model contained by lane-scoped cooldown; silent cheap-model substitution for pinned expensive model patched by strict-explicit-selection (PS-4 attribution). | P-17:L62-L62 |
| P17-E024 | E/LIMIT | Production bans `.unwrap()`/`.expect()` on dispatch/failover; NO blake2/sha2; tokio::sync::Mutex in async cited as hard rule. | P-17:L43-L43, L70-L70 |
| P17-A001 | A/CFG | Auth cooldown is a range 10-60m, not a deterministic ladder; exact status/error-code to lane mapping is RECON_NEEDED. | P-17:L9-L9, L55-L55 |
| P17-A002 | A/DATA | Precise identity of the "billed lane" scope object (provider×profile×model keys) is not fully named beyond {provider, lane, model} counter-move text. | P-17:L53-L53, L40-L40 |
| P17-A003 | A/SURFACE | Exact RuntimePanel/CloudPanel field names and layout for billed-vs-estimate are not specified. | P-17:L44-L45 |

Category coverage: capabilities/CLI/UI ✓; workflows/failover states ✓; data/ledger fields ✓; config/cooldowns ✓; integrations/events ✓; security (no silent sub) ✓; failures/lanes ✓; observability ✓; tests ✓; constraints/deferred ✓. No privacy policy; retry is cooldown+persist-before-retry rather than a named idempotency key.

### P-20 — context/memory lifecycle hardening (pre-compaction flush + on-demand index)

Source: `docs/thefinalglance/P-plan/P-20_context_memory_lifecycle.md` (77/77 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P20-E001 | E/STATUS | P-20 is PLANNED Wave OC-1, OPENCLAW-STEAL, lever no-context-lost-uncached, STRONG grounding, EXTENSION of H-50/H-51 + H user-memory + L familiarity + H MCP defer-load, effort M; PROOF_SPINE none. | P-20:L1-L5 |
| P20-E002 | E/CAP | Tier-1 SHIP: (a) pre-compaction memory-flush as ORDERED journaled step (durable notes BEFORE compaction, once per cycle); invariant "no compaction destroys context never offered to durable memory"; (b) versioned on-demand skill/rules index. | P-20:L6-L9, L18-L19 |
| P20-E003 | E/FLOW | openclaw prior art: silent NO_REPLY turn writes durable notes BEFORE compaction, once per cycle, soft threshold 4000 tokens. | P-20:L9-L9 |
| P20-E004 | E/DATA | On-demand index compact entry: name/description/path + sha256 VERSION marker (~24 tokens/skill); model reads file when needed; re-reads on version change; budget compact-degrade. | P-20:L9-L9, L19-L19, L31-L31 |
| P20-E005 | E/DEFER | Tier-2: Dreaming promotion gates (score/recall/diversity) as prior art for H memory governance; per-provider cache-control mapping table into H caching. | P-20:L20-L20 |
| P20-E006 | E/CFG | Deferred provider mapping prior art: Anthropic cache_control 5m/1h; OpenAI prompt_cache_key; Gemini cachedContents. | P-20:L9-L9 |
| P20-E007 | E/LIMIT | EXCLUDED: second memory store (ride H two-tier; A-1/N-2); flush that fires AFTER compaction. | P-20:L21-L21, L39-L41 |
| P20-E008 | E/DEP | Flush is IN-session, distinct from H-51 cross-session handoff; flush TARGET is H two-tier DurableMemory; index consumer is L familiarity; one lazy-load mechanism with H MCP defer-load. | P-20:L11-L16, L33-L34, L75-L75 |
| P20-E009 | E/CAP | Produces `crates/orchestrator/src/context/precompact_flush.rs` and `ondemand_index.rs`. | P-20:L29-L31 |
| P20-E010 | E/INT | Extends `lifecycle.rs::CompactionPass` with flush ordering — do NOT re-implement; production path is compaction invoking flush before compacting. | P-20:L33-L36, L71-L72 |
| P20-E011 | E/SURFACE | frontend_pair N/A: internal context machinery; no user surface beyond existing cost/memory panels (W-6 N/A justified). | P-20:L43-L44 |
| P20-E012 | E/SURFACE | Trigger command: `elai session drive-to-compaction --fixture tests/fixtures/context/long_session.json`. | P-20:L46-L46 |
| P20-E013 | E/EVENT | Expected SSB: `memory_flush_event` lands BEFORE the compaction row (ordered), carrying durable notes. | P-20:L47-L48, L51-L51 |
| P20-E014 | E/OBS | Skill index shows only compact entries until skill needed; file read once (trace); NEGATIVE: compaction before flush or full skill body preloaded FAILS. | P-20:L48-L48, L51-L52 |
| P20-E015 | E/LIMIT | stub_ban: NO compaction before flush; NO full skill body in prompt before invoke; NO second memory store; NO `.unwrap()`/`.expect()`; NO blake2/sha2 (blake3/sha256 for version marker per source contract). | P-20:L38-L42 |
| P20-E016 | E/TEST | VERIFY: off-agent memory_flush_event ordered before compaction on real session compaction path; plan_gate_check + wiring_contract_check. | P-20:L56-L60 |
| P20-E017 | E/TEST | INTEGRATION_SMOKE R10: long multi-turn past threshold TWICE on hard tool-heavy history; each compaction preceded by flush; no tool-call/result pair split; on-demand index loads body only on first invocation; composes with P-19 compaction split-rule. | P-20:L63-L64 |
| P20-E018 | E/SEC | REDTEAM: silent late-context loss defeated by ordered flush; huge skill corpus prompt bloat patched by ~24 tokens/skill until invocation. | P-20:L61-L61 |
| P20-E019 | E/ABORT | ABORT if flush would require a SECOND durable memory store. | P-20:L55-L55 |
| P20-E020 | E/EVIDENCE | Grounding: openclaw-vs-elai-comparison 2.6/2.7/2.8 steals S3/S7/S16/S18; H-50/H-51, H user-memory, L familiarity, H MCP defer-load PLAN-asserted. | P-20:L9-L9, L66-L69 |
| P20-I001 | I/CFG | Soft threshold 4000 tokens is openclaw prior art cited in context; whether ELAI Tier-1 adopts that numeric threshold or reuses H-51 thresholds is not locked in REVISED scope. | P-20:L9-L9, L18-L19 |
| P20-A001 | A/FLOW | Memory note selection policy, what counts as "offered", and flush failure/retry behavior are unspecified. | P-20:L18-L19, L9-L9 |
| P20-A002 | A/CFG | Compact-degrade levels and token-measurement method are absent. | P-20:L19-L19 |
| P20-A003 | A/DEFER | Dreaming score/recall/diversity thresholds and provider cache TTLs remain prior art only until Tier-2. | P-20:L20-L20, L9-L9 |
| P20-C001 | C/DEFER | Premise lists Dreaming and per-provider cache-control as unit goals, but REVISED scope defers both to Tier-2; Tier-1 is flush + on-demand index only. | P-20:L6-L6, L18-L20 |

Category coverage: capabilities/internal CLI ✓; workflows/ordering ✓; data/index schema ✓; config/defaults △; integrations/events ✓; no standalone security/privacy UI; failures/ordering fails ✓; observability ✓; tests ✓; constraints/deferred ✓. No formal concurrency/idempotency.

### P-22 — control-plane / multi-agent / channel hardening

Source: `docs/thefinalglance/P-plan/P-22_control_plane_channel_hardening.md` (78/78 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P22-E001 | E/STATUS | P-22 is PLANNED Wave OC-3, OPENCLAW-STEAL, lever fenced-returns-signed-ingress, STRONG grounding, EXTENSION of 3C+G-1+J+P-10+M+4C, effort M-L; PROOF_SPINE none. | P-22:L1-L6 |
| P22-E002 | E/CAP | Tier-1 SHIP: (a) child-output-as-fenced-evidence (subagent return is DATA, structurally unable to carry instructions; composes G-1 taint source) + depth-scoped tool-strip defaults for 3C. | P-22:L19-L20, L31-L32 |
| P22-E003 | E/CFG | Depth-scoped tool policy: subagents lose gateway/cron/messaging; leaves lose spawn. | P-22:L9-L9, L20-L20, L32-L32, L41-L41 |
| P22-E004 | E/SEC | Subagent return is fenced taint-source; instructions in it are inert; cannot steer parent. | P-22:L9-L9, L20-L20, L31-L31, L40-L40, L52-L53 |
| P22-E005 | E/DEFER | Tier-2 (b): 9-step deterministic channel-binding precedence + pairing-code UX + dmScope isolation for P-10 + front-door. | P-22:L20-L21, L9-L9 |
| P22-E006 | E/DEFER | Tier-2 (c): challenge-signed device pairing + fail-closed unknown-event broadcast gating + mandatory idempotency keys for J control plane + M seats. | P-22:L21-L21, L9-L9 |
| P22-E007 | E/DEFER | Tier-2 (d): trusted-policy hook tier ordered before third-party + manifest-validated-without-execution for 4C. | P-22:L21-L21, L9-L9 |
| P22-E008 | E/CFG | openclaw pairing prior art (Tier-2): 8-char codes, 1h expiry, <=3 pending; dmScope isolation; DEVICE_AUTH_* error codes. | P-22:L9-L9 |
| P22-E009 | E/LIMIT | EXCLUDED: second provenance scale (N-2/A-1); shipping ELAI's own ~30 channels (P owns FORMAT, not ecosystem). | P-22:L22-L22, L43-L43 |
| P22-E010 | E/CAP | Produces `crates/orchestrator/src/agents/fenced_return.rs` and `tool_strip.rs`. | P-22:L30-L32 |
| P22-E011 | E/DEP | Consumes AgentSpawn (3C) and TaintSource (G-1); targets also J control plane, P-10 ingress, M seats, 4C hooks. | P-22:L11-L17, L33-L35, L67-L69 |
| P22-E012 | E/INT | Production path: `agents/spawn.rs` applies tool_strip and wraps every return via fenced_return before parent; test callers never satisfy R7. | P-22:L36-L38, L72-L73 |
| P22-E013 | E/SURFACE | frontend_pair: enrollment + pairing UX on the control surface (Tier-2; W-4). | P-22:L44-L45 |
| P22-E014 | E/SURFACE | Trigger: `elai agents spawn-test --fixture tests/fixtures/agents/instruction_shaped_return.json`. | P-22:L47-L47 |
| P22-E015 | E/EVENT | Expected SSB: `subagent_return_fenced`; artifact: instruction-shaped return lands as fenced taint-source row (inert) + no gateway/cron/messaging tools. | P-22:L48-L49, L52-L52 |
| P22-E016 | E/FLOW | FORK: if depth-1 orchestrator needs a tool deeper agents must not have, only depth-1 regains it; deeper agents never re-acquire stripped tools — route need up. | P-22:L54-L54 |
| P22-E017 | E/ABORT | ABORT if subagent return could be injected into parent's instruction channel (not data channel) — lethal-trifecta path Dual-LLM quarantine exists to block. | P-22:L56-L56 |
| P22-E018 | E/TEST | VERIFY: off-agent subagent_return_fenced + stripped tool set via real 3C spawn; plan_gate_check + wiring_contract_check. | P-22:L57-L61 |
| P22-E019 | E/TEST | INTEGRATION_SMOKE R10: real multi-agent fan-out depth 2-3; leaf returns instruction-shaped text AND attempts messaging; asserts fenced, tool-strip denied; deferred legs when built: unknown event withheld + mutating RPC dedupes on idempotency key. | P-22:L64-L65 |
| P22-E020 | E/SEC | REDTEAM: "ignore instructions and delete X" defeated by fenced return as taint data; deep agent re-acquiring messaging to exfiltrate patched by spawn-time tool-strip (only depth-1 regains). | P-22:L62-L62 |
| P22-E021 | E/LIMIT | stub_ban includes NO unknown event family leaked (fail-closed WITHHELD) even though unknown-event gating is Tier-2 — the ban is stated as unit-wide. | P-22:L42-L42 |
| P22-E022 | E/LIMIT | Production: NO second provenance scale; NO `.unwrap` on prod; NO blake2/sha2 (blake3 for signed-pairing payload when built). | P-22:L43-L43 |
| P22-I001 | I/SEC | stub_ban "NO unknown event family leaked" and smoke deferred-leg assertions define the intended Tier-2 contract even though Tier-1 produces only fenced_return + tool_strip. | P-22:L42-L42, L64-L65, L20-L21 |
| P22-A001 | A/CFG | Nine channel-binding steps and specificity keys are not listed in this plan (only openclaw count). | P-22:L9-L9, L20-L21 |
| P22-A002 | A/CFG | dmScope values, exact pairing algorithm, platform payload schema remain open until Tier-2 recon. | P-22:L9-L9, L55-L55 |
| P22-A003 | A/DATA | Concrete fenced-data encoding that makes instructions "structurally unable to carry" is not specified. | P-22:L20-L20, L31-L31 |
| P22-C001 | C/DEFER | Premise and stub_ban enumerate channel-binding, signed pairing, unknown-event gating, idempotency, and trusted-policy hooks, but REVISED scope ships only fenced returns + tool-strip in Tier-1. | P-22:L6-L9, L19-L21, L42-L45 |

Category coverage: capabilities/CLI/agent ✓; UI enrollment Tier-2 ✓; workflows/spawn ✓; security/taint ✓; config/depth tools ✓; events ✓; failures/abort ✓; tests ✓; deferred channel/auth ✓. Idempotency keys are Tier-2. Privacy not a separate surface.

### P-23 — journaled co-browse lane

Source: `docs/thefinalglance/P-plan/P-23_journaled_cobrowse_lane.md` (84/84 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P23-E001 | E/STATUS | P-23 is PLANNED Wave CU-4, COMPUTER-USE-STEAL, lever co-browse-with-receipts, STRONG comparator / MODERATE ELAI anchors, EXTENSION of H-15+E-14, effort M; depends H-13,H-15,E-14,P-13; PROOF_SPINE none (product); owner-commissioned 2026-07-11. | P-23:L1-L6 |
| P23-E002 | E/CAP | Closes ENTRY 14 design hole: general-purpose co-browse where unjournaled input path is structurally impossible; comparator ships pane without receipts. | P-23:L6-L9 |
| P23-E003 | E/SURFACE | Tier-1 READ lane: canvas browse node rendering page EXTRACTION — E-14 substrate nav + AXTree + screenshot, read-only, content taint-tagged web at ingest, deixis via H-15; minimal ceremony (P-5). | P-23:L19-L19, L48-L49 |
| P23-E004 | E/SURFACE | Tier-2 CO-BROWSE lane: orchestrator-owned Playwright/CDP browser; CDP screencast frames stream into browse node — THE PANE IS DUMB GLASS. | P-23:L20-L20, L48-L49 |
| P23-E005 | E/CAP | One InputProxy is the only holder of the engine input session; every nav/click/keystroke — user (forwarded from pane) or model (P-13 id-pointing; coordinate fallback under P-12 frame-binding) — lands actor-tagged cobrowse_input SSB row BEFORE dispatch. | P-23:L20-L20, L34-L34, L45-L45 |
| P23-E006 | E/DET | Journal completeness is structural, not policy: no second input path; CDP input session handle private to proxy (type-level). | P-23:L20-L20, L34-L34, L45-L45, L57-L57 |
| P23-E007 | E/CFG | Session persistence typed: default = ephemeral profile destroyed on close; `--persist <name>` = named profile in P-10 credential vault; choice is a field on cobrowse_session row. | P-23:L21-L21 |
| P23-E008 | E/PRIV | Egress gating: user-typed text bound for web form passes P-6 privacy lattice + domain-first-seen warning row before dispatch. | P-23:L22-L22, L67-L67 |
| P23-E009 | E/SEC | Authority: everything read in pane is G-1 taint source; browse content can never authorize a mutating lane. | P-23:L23-L23, L67-L67 |
| P23-E010 | E/LIMIT | EXCLUDED as mechanism: native embedded webview as interactive surface (Tauri multiwebview unstable cargo flag, open bugs #10131/#10420, remote DOM in trusted shell, unjournaled native input). Survives only as FORK option with InputProxy still interposed. | P-23:L24-L24, L58-L58 |
| P23-E011 | E/CAP | Produces `crates/orchestrator/src/cobrowse/mod.rs` (CoBrowseLane: session mgr + screencast pump + profile custody hook) and `input_proxy.rs` (InputProxy journal-then-dispatch). | P-23:L32-L34 |
| P23-E012 | E/DEP | Consumes E-14 browser_e2e engine (if unexecuted, this unit vendors substrate and E-14 consumes it — threaded, not second engine); P-13 PerceptionScaffold; P-12 FrameBinding; G-1 TaintSource; G-2 ActionStream. | P-23:L12-L16, L35-L38, L75-L75 |
| P23-E013 | E/SURFACE | CLI: `elai browse <url>` production CLI. | P-23:L40-L40, L51-L51, L78-L79 |
| P23-E014 | E/SURFACE | Front-door router lane table: "pull up / open <url>" resolves to browse lane. | P-23:L41-L41, L78-L79 |
| P23-E015 | E/SURFACE | H-13 canvas dispatcher: browse node view_type + forwarded pane input. | P-23:L42-L42, L78-L79 |
| P23-E016 | E/SURFACE | frontend_pair: canvas browse node screencast (Tier-2) / extraction (Tier-1) + actor-colored input trail so user SEES the journal as it lands. | P-23:L48-L49 |
| P23-E017 | E/EVENT | Expected SSB: `cobrowse_session` {url, profile: ephemeral\|vault:<name>}; every input has PRIOR actor-tagged `cobrowse_input` {actor, target: element-id\|coord+frame_hash}. | P-23:L52-L53, L56-L56 |
| P23-E018 | E/TEST | Journal-then-dispatch ordering checked off-agent against engine CDP log; NEGATIVE: input without prior row, model coordinate on stale frame hash, ephemeral session leaving profile bytes FAILS. | P-23:L53-L53, L56-L56 |
| P23-E019 | E/CFG | FORK latency tripline: if co-browse input-to-paint p95 exceeds 250ms on local loop (measured), evaluate native Tauri multiwebview ONLY once multiwebview left unstable flag AND InputProxy still interposed; if engine lacks CDP screencast, fall back to timed captureScreenshot loop (degraded FPS, identical receipts). | P-23:L58-L58 |
| P23-E020 | E/ABORT | ABORT: any input path bypassing InputProxy (P0); persistent profile outside vault custody; profile isolation failure (session seeing personal browser state). | P-23:L60-L60, L45-L46 |
| P23-E021 | E/TEST | VERIFY via real `elai browse`: plan_gate_check + wiring_contract_check + production_caller_check on cobrowse modules. | P-23:L61-L66 |
| P23-E022 | E/SEC | REDTEAM failed: hostile fake ELAI approval dialog inside pane (UI redress) — harness chrome never in screencast; pane click only becomes cobrowse_input, never harness command. Failed: page text instructing destructive click — G-1 taint blocks untrusted->destructive sink. Succeeded unpatched then patched: user types secret into phishing form — egress passes P-6 + domain-first-seen warning BEFORE dispatch. | P-23:L67-L67 |
| P23-E023 | E/TEST | INTEGRATION_SMOKE after P-23+P-13: hard docs site + hostile fixture, multi-step interleaved user+model long enough for mid-run compaction; asserts prior rows, injection never reaches mutating lane, zero ephemeral bytes, P-14 crystallize mints RRA-bound replay. | P-23:L69-L70 |
| P23-E024 | E/DEP | Pairs with P-13 pointing, P-14 crystallize, P-16 catalog; reuses P-6, P-10, O-2 per files. | P-23:L81-L82, L75-L75 |
| P23-E025 | E/INT | Vertical slice: CoBrowseLane/InputProxy wired into ALL THREE of CLI, front-door, H-13 canvas in this changeset. | P-23:L78-L79 |
| P23-E026 | E/LIMIT | Production: NO remote DOM inside trusted dashboard webview; NO `.unwrap()`/`.expect()`; blake3 for frame hashes; anti-slopsquat on browser-driver crate; bun-not-node if capture sidecar is JS. | P-23:L47-L47, L76-L76 |
| P23-E027 | E/CAP | Co-browse exploration can mint deterministic replay script RRA-bound via P-14. | P-23:L15-L15, L70-L70 |
| P23-I001 | I/STATE | Browse node is an H-15 view_type; Tier-1 and Tier-2 share the node with different render modes (extraction vs screencast). | P-23:L13-L13, L19-L20, L48-L49 |
| P23-A001 | A/CFG | Cookie/download/upload/pop-up/permission dialogs and multi-tab behavior are not specified. | P-23:L19-L24 |
| P23-A002 | A/DEP | P-10 vault custody details for named profiles depend on P-10 which is absent from P-summary sections. | P-23:L21-L21, L16-L16 |
| P23-A003 | A/INT | Rust-native driver vs bun/Playwright capture sidecar choice is RECON_NEEDED at edit. | P-23:L59-L59 |

Category coverage: capabilities/CLI/router/canvas UI ✓; workflows/session lifecycle ✓; data/events ✓; config/persist flag ✓; privacy egress ✓; security/taint ✓; failures/abort ✓; tests ✓; deferred native pane ✓. Idempotency not named; concurrency of multi-tab not specified.

### P-25 — exogenous-injection + evidence-coupled-revision creativity falsifier (B-EXO-0)

Source: `docs/thefinalglance/P-plan/P-25_exogenous_injection_creativity_falsifier.md` (548/548 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| P25-E001 | E/STATUS | P-25 is PLANNING CANDIDATE (V4 candidate-to-core binding repair); no M1/source/test/seal/smoke/benchmark authorization until fresh independent V4 PASS; track RESEARCH-CONDITIONED-CREATIVITY; lever deterministic-exogenous-injection; effort M; PROOF_SPINE none. | P-25:L1-L9 |
| P25-E002 | E/STATUS | Authority: `P25-EVAL-TRUST-V4` + `P25-CANDIDATE-CORE-BINDING-V1` require one fresh independent PASS as a set; V2/V3 PASSes are historical only; RECON-2 import-map must be re-migrated and re-audited before controlling M3. | P-25:L6-L6, L30-L30 |
| P25-E003 | E/STATUS | Three creativity controllers dead: B-0 ProbeUtility, B-3 FDL, Q-10 B-CREATIVITY-0 (~8.8x cost); surviving measured lever is exogenous grounded info (pilot P-A 3.5x real anchors). | P-25:L11-L11, L15-L21 |
| P25-E004 | E/CAP | B-EXO-0 tests TWO preregistered families on one frozen open-world corpus: FAMILY A injection (same single DIRECT call + deterministic verified artifacts, zero extra model calls, hard cost cap); FAMILY B evidence-coupled revision (one revision call fed public triage kill-reasons vs self-revision control). | P-25:L11-L11, L32-L36 |
| P25-E005 | E/DATA | K=30 paired real ELAI improvement fixtures; no synthetic tier; B-CREATIVITY-1 corpus design reused under own preregistration (prereg 2026-07-12 §6.1). | P-25:L32-L36, L26-L26 |
| P25-E006 | E/CFG | Six arms, same model/candidate/token/parser/evaluator table; arm order seeded per task; generator never sees arm names; every completion becomes exactly 6 `CandidateSlotV1` records. | P-25:L36-L36 |
| P25-E007 | E/CAP | Arm 1 DIRECT — target-only single self-coordinating call (measured champion baseline). | P-25:L38-L38 |
| P25-E008 | E/CAP | Arm 2 EXO_RELEVANT (FAMILY A treatment) — DIRECT + k (frozen, default 3) verified target-relevant artifacts with provenance headers: code slice (elai.map), measured row (bench/journal/feature-map), VerifiedCitation (else second code slice); deterministic seeded model-free retrieval sealed in manifest. | P-25:L39-L39 |
| P25-E009 | E/CAP | Arm 3 EXO_SHUFFLED — same count/envelope, artifacts from DIFFERENT fixture via frozen no-fixed-point permutation STRATIFIED across subsystems; RetrievalClassifierV1 rederives labels; same-subsystem-collision count must be exactly zero. | P-25:L40-L40 |
| P25-E010 | E/CAP | Arm 4 PAD_MATCHED — DIRECT + neutral envelope_pad to same frozen model-free envelope; order (EXO_RELEVANT,EXO_SHUFFLED,PAD_MATCHED); count `(len(exact_utf8_prompt_bytes)+2)//4`; exact spread <= 1/50. | P-25:L41-L41 |
| P25-E011 | E/CAP | Arm 5 TRIAGE_REVISE (FAMILY B treatment) — DIRECT then public triage over VALID slots via pinned PublicTriageContextV1 (demarcation+anchor+composition; ledger EXCLUDED); TriageReceiptV1; RevisionEvidenceV1 ALIGNED; one revision call emits final 6-card set; SCORED_REVISED points back to RevisionCallInputV1. | P-25:L42-L42 |
| P25-E012 | E/CAP | Arm 6 SHUFFLED_REVISE (FAMILY B control) — identical two-call structure/template but RevisionEvidenceV1.mode=SHUFFLED with M1-seeded no-fixed-point C1..C6 permutation. | P-25:L43-L43 |
| P25-E013 | E/DATA | exo-card V2 grammar must emit in order: id, mechanism_family, target_area, target_units, expected_effect, falsifier, and canonical one-line strict-object payload_json; public triage receives no payload or source span. | P-25:L42-L42 |
| P25-E014 | E/OBS | Co-primary P1 `execution_validated_mechanism_yield` = count of six scored-role slots with valid card/payload, public triage survive, candidate-bound trusted-core PASS, deduped by MechanismIdentityV1={candidate_payload_raw_sha256}. | P-25:L45-L45 |
| P25-E015 | E/OBS | Co-primary P2 `best_target_value_delta` = max candidate value among six scored slots minus max DIRECT value for same task; INVALID=0/1. | P-25:L45-L45 |
| P25-E016 | E/DET | Every P1/P2/G1/G2/G3 paired result uses exactly 5,000 bootstrap samples and M1-frozen seed `20260712`; mixed/reseeded rows invalid. Revision-source sets never enter P1/P2. | P-25:L45-L45 |
| P25-E017 | E/TEST | G1: real-anchor-rate non-inferiority vs DIRECT (>= -0.05 one-sided, per family). G2: grounding/ScientificReceiptV1 validity non-inferiority. | P-25:L47-L47, L49-L49 |
| P25-E018 | E/TEST | G3-A FAMILY A hard cost cap: EXO_RELEVANT sealed call/token accounting <= 1.3x DIRECT (ratios 13/10); zero preprocessing model calls. | P-25:L47-L47, L49-L49 |
| P25-E019 | E/TEST | G3-B FAMILY B: TRIAGE_REVISE <= 2.5x DIRECT (ratios 5/2) AND cost-normalized P1 (yield per token) non-inferior to DIRECT (>= -0.05). | P-25:L47-L47, L49-L49 |
| P25-E020 | E/TEST | G4: zero hidden_answer/leakage/manifest_binding/token_envelope violations; revision inputs scanned like prompts. | P-25:L47-L47, L49-L49 |
| P25-E021 | E/TEST | FAMILY A SUPPORTED requires P1 and P2 CI95 lower bounds > 0 for EXO_RELEVANT minus EACH of DIRECT, EXO_SHUFFLED, PAD_MATCHED; all G1/G2/G3-A/G4 pass. | P-25:L51-L52 |
| P25-E022 | E/TEST | FAMILY B SUPPORTED requires P1 and P2 CI95 lower bounds > 0 for TRIAGE_REVISE minus EACH of DIRECT and SHUFFLED_REVISE; all G1/G2/G3-B/G4 pass. | P-25:L53-L54 |
| P25-E023 | E/LIMIT | No combined "creativity supported" claim; multiplicity declared under both-families-null. Diversity/judge preference can never promote. | P-25:L45-L45, L55-L55 |
| P25-E024 | E/DEFER | SUPPORTED family -> separate Tier-2 adoption unit only for THAT mechanism (A: exo-inject in improvement-conjecture stage-1 + paper-to-plan seam; B: triage-feedback revision leg). BOTH NULL/REFUTED -> creativity line CLOSES entirely. No B-EXO-1/-2 pre-authorized. | P-25:L57-L57 |
| P25-E025 | E/CFG | Cost: arms 1-4 = 120 calls; arms 5-6 = 120 calls; total 240 M5 generation + 8 M4 smoke = 248; retrieval/triage/pad/preflight/verify add zero model calls. | P-25:L59-L59 |
| P25-E026 | E/LIMIT | Integration: .claude/tools only; .elai_cc NOTHING (no Rust/skill/runtime); frontend_pair N/A offline harness; conditional skills only post-SUPPORTED separate unit. | P-25:L61-L65, P-25:L134-L136 |
| P25-E027 | E/FLOW | Normative order: M0 trust-substrate build/audit -> M1 -> M3 -> M2 draft data -> semantic qualification -> deterministic hidden generation -> sealing/freeze -> M4 -> M5. Numeric stage names do not authorize M2 before M3. | P-25:L69-L69 |
| P25-E028 | E/CAP | M0 implements exactly exo_records.py, exo_trust_records.py, exo_canonical.py, exo_trust.py, exo_curator.py, tests/test_exo_trust_v4.py under fixture blindness; every source < 1000 lines; independent audit PASS before M1; N1/N2/N3/N14/N15 immutable after M1 root. | P-25:L71-L71 |
| P25-E029 | E/CAP | M1 freezes all six arms, ArmOrderSetV1, TaskExecutionLogV1 eight-row expansion, endpoint http://127.0.0.1:8810/v1/messages, model kimi-k2-7, max_tokens=1400, timeout 240000 ms, one-POST/zero-tool/no-cache/no-retry, K=30, all schemas/hash domains/gates/bans. | P-25:L73-L73 |
| P25-E030 | E/DATA | M1 freezes P25-REQUEST-JSON-V1 sorted compact three-key JSON + LF; P25_RESPONSE_MAX_BYTES=1048576; P25_RESPONSE_READ_CHUNK_BYTES=65536; NFC waiver only at messages[0].content. | P-25:L75-L75 |
| P25-E031 | E/FLOW | CompletionArchiveSink: pre-POST single-use ExecutionPermit; stage prompt/request/response; canonicalize CompletionRecordV1; independently rehash; atomic promote; failure discards; TaskExecutionLogV1 only after exact eight rows. | P-25:L75-L75 |
| P25-E032 | E/CAP | M3-A..F six cumulative batches (analysis, cores/retrieval/seal/qualification, public triage/prompts, completion transport, pool/runner, independent verifier); each <=5 files, <=3 deliverables, independent gate; no M0 byte edit. | P-25:L79-L86 |
| P25-E033 | E/CAP | Four trusted cores: grep_presence_core, parser_fixture_core, bounded_simulation_core, benchmark_delta_core; dispatch_core indexes frozen table by fixture kind only; zero model/filesystem/process capability inside cores. | P-25:L82-L82, P-25:L61-L61 |
| P25-E034 | E/TEST | Final fixture-blind M3 freeze after all six batch PASS: full Tier-0 + M0 trust + six M3 modules + A1-A8 + schema/wiring + python_wire_check 0 DARK + maintainability 0 BLOCK; pins PublicTriageContextV1 map rows; forms M3FreezePayloadV1/RecordV1/anchor/locator. | P-25:L88-L88, P-25:L510-L512 |
| P25-E035 | E/FLOW | M2 prepare-corpus after M3 anchor: five-stratum seeded selection; FixtureDraftV1; M2AttemptLedgerV1; 30 terminal ELIGIBLE; >=20 executable micro-experiments; SealedCorpusV1 + RetrievalAssignmentV1 + ArmOrderSetV1; P25RunConfigV1 with external expected digest. | P-25:L94-L96 |
| P25-E036 | E/FAIL | If frozen pool cannot yield 30 ELIGIBLE or no bounded_simulation member: POSTPONED_INSUFFICIENT_EXECUTABLE_POOL; never lower K or substitute judge. | P-25:L96-L96, P-25:L108-L108 |
| P25-E037 | E/SURFACE | M4 CLI: `python3 .claude/tools/exo_injection_ab.py run-smoke --repo-root --run-config --expected-run-config-sha256 --out`; selects lowest selection_slot terminal-ELIGIBLE bounded_simulation; eight-call sequence; publishes P25SmokeReportV1 only (no K=30 decision). | P-25:L98-L100, P-25:L356-L357 |
| P25-E038 | E/SURFACE | M5 CLI: `python3 .claude/tools/exo_injection_ab.py run-scored` same trust inputs over full K=30; writes family verdict and closes out. | P-25:L102-L104, P-25:L357-L357 |
| P25-E039 | E/SURFACE | Independent checker: `python3 .claude/tools/exo_creativity_artifact_check.py --verify-p25-smoke` and `--verify-p25-all` with same three trust inputs; PASS = exit 0 and 0 violations. | P-25:L358-L359, P-25:L514-L514 |
| P25-E040 | E/SURFACE | Curator CLIs under exo_injection_ab.py: curate-m0-file-set, curate-m1-artifact, curate-m1-anchor, lock-m1, curate-m3-payload, curate-m3-freeze, curate-m3-anchor, lock-m3 (each with exact flags). | P-25:L341-L348 |
| P25-E041 | E/CAP | Produces large closed symbol set including FileRowV1/TrustRootV2/M1*/M3* anchors, TaskExecutionLogV1, CandidateCardV2, four CandidateTaskView/Payload variants, RetrievalClassifierV1/CatalogV1, ExactCompletionClient, decide_family_a/b, verify_p25_* suite, prepare_pool/prepare_corpus, etc. (full produces_symbols list). | P-25:L138-L302 |
| P25-E042 | E/DEP | Consumes P25-EVAL-TRUST-V4, P25-CANDIDATE-CORE-BINDING-V1, RECON-2 N1-N16, research_creativity_common paired bootstrap, conjecture_triage_core six functions, elai.map, ELAI-FEATURE-MAP, b0v3 report, VerifiedCitation, Q-10 runner/checker/tests as UNCHANGED references (P-25 imports none of Q-10 runner). | P-25:L303-L322, P-25:L61-L61 |
| P25-E043 | E/LIMIT | ABORT list includes: model call in retrieval/triage/packetizer; analogy/relational structure; LLM judge in primary gate; growing conjecture_research_coupling_ab.py past 1000 lines; any .elai_cc/skill/runtime edit before SUPPORTED; lowering K or 1.3x/2.5x caps after fixture inspection; continuing after both families NULL/REFUTED; fusing novelty/grounding/value/cost; hidden-side into revision (G4 INVALID); more than ONE revision round; fixture/candidate naming evaluator behavior; M2 before M3; wall-clock in scientific scores; generic BLAKE3/local receipts/mutable triage/historical hashes as trust authority. | P-25:L134-L136 |
| P25-E044 | E/LIMIT | Extensive stub_ban rejects caller injection of paths/seeds/clients/orders/counts/layouts/kinds, packetizer, analogy fields, second revision, wall-clock scoring, hidden into revision, .elai_cc paths, etc. | P-25:L361-L499 |
| P25-E045 | E/OBS | Artifact layout: content-addressed records/<schema_id>/<sha256>.json and raw/<origin_kind>/<sha256>.bin; singleton report branches; out_dir published by no-overwrite atomic rename from reserved `<out_dir.name>.p25-stage`. | P-25:L501-L501 |
| P25-E046 | E/DET | Bridge lane kimi-k2-7 via devin-proxy 127.0.0.1:8810 FREE; usage.input_tokens is chars/4 estimate — P-25 freezes nearest-quarter-byte proxy over exact archived UTF-8 prompt bytes, no preprocessing meter call. | P-25:L28-L28 |
| P25-E047 | E/FAIL | FORK: treatment beats DIRECT but not EXO_SHUFFLED -> NULL by joint gate; must name "real text helps, relevance does not" pattern. | P-25:L110-L110 |
| P25-E048 | E/FAIL | FORK: bridge unavailable (3-step health fails) -> documented alternate FREE lane with prereg version bump; never silent model swap. | P-25:L112-L112 |
| P25-E049 | E/FAIL | FORK: core cannot close in-process without fs/process/cargo -> M3 TCB gate BLOCK before M2. | P-25:L114-L114 |
| P25-E050 | E/TEST | INTEGRATION_SMOKE is M4 non-decision smoke on live lane; auto second smoke if prereg/root/manifest/config bumps before M5. | P-25:L506-L506 |
| P25-E051 | E/SEC | REDTEAM author+independent+round-2 patches: retrieval/hidden separation, cost cap lock, K=30 no-shrink, artifact-sourcing failure branch, four core implementations, lexical-leak limitation labeled, stratified shuffle, card grammar for FAMILY B, kill-leg names, SHUFFLED_REVISE control, multiplicity declaration. | P-25:L518-L548 |
| P25-E052 | E/LIMIT | Wall-clock observations diagnose infrastructure only and never enter scientific score, guard, verdict, or hash. | P-25:L47-L47 |
| P25-E053 | E/CFG | Public triage legs: demarcation + anchor + composition only; ledger leg EXCLUDED (would leak prior-run info); no "provenance" kill leg. | P-25:L42-L42, P-25:L543-L543 |
| P25-E054 | E/DATA | GuardInputSetV1 binds bootstrap_input_set_sha256, rows, run_config_sha256, schema_version, sealed_corpus_sha256, violation_counts — decision API consumes this, not unbound cost report. | P-25:L49-L49 |
| P25-I001 | I/STATUS | Because STATUS is PLANNING CANDIDATE pending V4 PASS, all M0-M5 symbols are planned deliverables, not shipped inventory claims. | P-25:L1-L6, L71-L88 |
| P25-I002 | I/SURFACE | Curator/run/checker CLIs are the only operator-facing surfaces; no dashboard panel, drawer, or notification is in scope. | P-25:L64-L65, L341-L359 |
| P25-A001 | A/STATUS | V4 PASS is required but not established by this plan file itself; execution is unauthorized until that external audit. | P-25:L1-L6 |
| P25-A002 | A/DATA | Exact preregistration document body (beyond freezes listed) and complete hash-domain registry contents live in M1 artifacts not embedded here. | P-25:L73-L75 |
| P25-A003 | A/DEP | RECON-2 import-map migration to candidate-bound API is required and incomplete for controlling M3. | P-25:L6-L6, P-25:L120-L122 |
| P25-A004 | A/CFG | Exact k default is 3 but frozen k and per-fixture substitution table are M1/manifest values not fully tabulated in prose. | P-25:L39-L39 |
| P25-C001 | C/STATUS | Header STATUS-HISTORY notes V3 scientific defect (candidates had no input edge to trusted core) repaired by V4 binding; any reader using V3 PASS as authority conflicts with stated precedence. | P-25:L3-L6 |

Category coverage: capabilities/CLI harness ✓ (no product UI); workflows M0-M5 ✓; schemas/data extensive ✓; flags/config freezes ✓; integrations/events N/A product SSB (offline artifacts) ✓; security/trust chain ✓; failures/abort/fork ✓; determinism/bootstrap ✓; observability/artifacts ✓; tests/gates/redteam ✓; constraints/deferred adoption ✓. Privacy N/A (offline bench). Idempotency via content-addressed artifacts and single-use permits.

## Cross-source interactions and lifecycle

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| IX-001 | E | P-3 export/import composes P-1 AppManifest + P-2 ports as the mechanism payload; persona regen (Tier-2) needs E-5 memory + P-7 profile. | P3-E003, P3-E011, P3-E012, P3-E024 |
| IX-002 | E | P-3 import re-verification rides M-5/M-6; admission rides L-12/skillspector; provenance redaction rides G-9; RRA/lineage rides L-14/G-2. | P3-E007–P3-E009, P3-E011 |
| IX-003 | E | P-15 extends closed J-96 CaseNode/MintOnProof; reuse-hit feeds K-18/PS-5; bBoN (Tier-2) slots behind I evidence-ranked search after receipts. | P15-E003–P15-E006, P15-E011 |
| IX-004 | E | P-15 and P-14 are co-minters of reuse artifacts feeding K-18; share CU-track R10 cadence. | P15-E017 |
| IX-005 | E | P-17 hardens J free-API rotation + I-50 auth + J-28 ledger; feeds K-2/PS-4; substitutions align with Q-4; B-37 consumes fallback chains. | P17-E001–P17-E002, P17-E010–P17-E012 |
| IX-006 | E | P-17 session stickiness/rotation-as-cache-invalidation composes H cache-stable layout thesis. | P17-E007 |
| IX-007 | E | P-20 flush is in-session ordered pre-step for H-51 CompactionPass; target is H two-tier DurableMemory; pairs with P-19 compaction split-rule; H-51 remains cross-session sibling. | P20-E002, P20-E008, P20-E017 |
| IX-008 | E | P-20 on-demand index unifies L familiarity skills/rules with H MCP defer-load as one lazy-load mechanism. | P20-E004, P20-E008 |
| IX-009 | E | P-22 Tier-1 fenced return is G-1 taint discipline at the 3C agent boundary; tool-strip binds AgentSpawn. | P22-E002–P22-E004, P22-E011–P22-E012 |
| IX-010 | E | P-22 Tier-2 channel-binding/pairing/idempotency/hooks compose P-10 ingress, J front-door/control plane, M seats, 4C plugins. | P22-E005–P22-E007, P22-E011 |
| IX-011 | E | P-23 reuses E-14 engine (or vendors then E-14 consumes); H-15 view_type + H-13 dispatcher; P-13 ids + P-12 frame bind; P-14 crystallize; P-6 egress; P-10 vault; G-1/G-2. | P23-E003–P23-E009, P23-E012, P23-E024 |
| IX-012 | E | P-23 front-door "pull up X" and `elai browse` are co-equal production entrypoints with canvas browse node. | P23-E013–P23-E015, P23-E025 |
| IX-013 | E | P-25 is harness-only and must not edit .elai_cc; adoption units (improvement-conjecture / paper-to-plan) are separate and conditional on SUPPORTED family. | P25-E024, P25-E026, P25-E043 |
| IX-014 | E | P-25 FAMILY B instantiates grounding-as-exit at creativity grain; FAMILY A continues pilot P-A exogenous-injection lever; both supersede refuted Q-10 mechanism without reusing its runner. | P25-E003–P25-E004, P25-E042 |
| IX-015 | E | P-25 retrieval substrate: elai.map anchors, sealed bench rows, E-48 VerifiedCitation, feature-map ledger — read-only. | P25-E008, P25-E042 |
| IX-016 | E | Across P-15/P-17/P-20/P-22/P-23 product units, production callers must be non-test live paths (R7); P-25 production callers are the real CLI entrypoints, not unit tests. | P15-E008; P17-E013; P20-E010; P22-E012; P23-E025; P25-E037–P25-E039 |
| IX-017 | E | Shared hash ban: product units ban blake2/sha2; P-20 allows blake3/sha256 for version markers; P-23 uses blake3 for frame hashes; P-25 trust authority forbids generic BLAKE3 as sole authority in favor of native Git/OID domains. | P15-E018; P17-E024; P20-E015; P22-E022; P23-E026; P25-E043 |
| IX-018 | I | P-3 marketplace/friend sharing (Tier-2) and P-22 control-surface pairing UX (Tier-2) are distinct distribution vs device-enrollment surfaces that may later share L-12 admission patterns but are not the same product. | P3-E009, P3-E012; P22-E013 |
| IX-019 | E | PS-number coupling in this chunk: P-15 feeds PS-5; P-17 feeds PS-4; P-3/P-20/P-22/P-23/P-25 declare no PS contribution. | P15-E001; P17-E001; P3-E001; P20-E001; P22-E001; P23-E001; P25-E001 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | conflict/scope | P-3 premise describes full import+regen; REVISED Tier-1 omits persona regen. | P3-C001 |
| AC-002 | ambiguity/data | P-3 AppBundle serialization, persona exclusion set, and M-5 whole-bundle grain remain open. | P3-A001–P3-A003 |
| AC-003 | conflict/scope | P-15 wargame asserts bBoN ranks, but REVISED scope defers bBoN/tiers/embeddings to Tier-2. | P15-C001, P15-E004 |
| AC-004 | ambiguity/config | P-15 retrieval/fusion/promotion criteria and reuse-hit schema details unspecified. | P15-A001–P15-A002 |
| AC-005 | ambiguity/config | P-17 auth cooldown range and status→lane mapping incomplete. | P17-A001–P17-A002 |
| AC-006 | conflict/scope | P-20 premise lists Dreaming + provider cache table; REVISED Tier-1 is flush + index only. | P20-C001 |
| AC-007 | ambiguity/flow | P-20 note selection, "offered" definition, compact-degrade, and 4000-token threshold adoption unresolved. | P20-I001, P20-A001–P20-A003 |
| AC-008 | conflict/scope | P-22 premise/stub_ban include channel-binding/pairing/idempotency/hooks; Tier-1 ships only fenced return + tool-strip. | P22-C001, P22-I001 |
| AC-009 | ambiguity/security | P-22 fenced encoding, nine binding steps, pairing parameters unspecified. | P22-A001–P22-A003 |
| AC-010 | ambiguity/product | P-23 cookie/permission/multi-tab behavior and driver choice open; P-10 custody external. | P23-A001–P23-A003 |
| AC-011 | status/gate | P-25 blocked on fresh V4 PASS + RECON-2 re-audit; V3 PASS must not authorize execution. | P25-C001, P25-A001, P25-A003 |
| AC-012 | ambiguity/schema | P-25 full hash-domain registry and frozen k/substitution tables live in M1 artifacts not fully inlined. | P25-A002, P25-A004 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/16-P-SUMMARY_INDEX.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### P-3 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P3-001 | **ADD** REVISED Tier-1 vs Tier-2 split: Tier-1 is export + verify-gated import without persona regen; regen and marketplace/friend surface are Tier-2. | 16-P:L155-L189 | P3-E012, P3-C001 |
| SD-P3-002 | **ADD** FORK for M-5 whole-bundle capability (presence-check floor) and ABORT if factoring needs code forks. | 16-P:L161-L182 | P3-E018–P3-E019 |
| SD-P3-003 | **ADD** exact SSB names `app_export`/`app_import`, planted-secret smoke, production_caller_check, and R10 second-seat smoke. | 16-P:L177-L182 | P3-E014–P3-E016, P3-E021–P3-E022 |
| SD-P3-004 | **ADD** redteam post-signing edit fail and provenance-leak patch; P-6 domain labels ride ports. | 16-P:L184-L184 | P3-E023–P3-E024 |
| SD-P3-005 | **EXPAND** summary A01/A02 with recon items for G-9 entrypoint and persona physical layout. | 16-P:L187-L189 | P3-A001–P3-A003 |

### P-15 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P15-001 | **CORRECT** summary inventory that lists narrative/episodic tiers, embedding retrieval, and bBoN as if Tier-1: source defers all three to Tier-2; Tier-1 is gated retrieve/distill/fuse + reuse-hit + default-off. | 16-P:L507-L521 | P15-E003–P15-E005, P15-C001 |
| SD-P15-002 | **ADD** FORK for stale non-reproducing cases (re-explore + re-mint + staleness field). | 16-P:L500-L533 | P15-E013 |
| SD-P15-003 | **ADD** ABORT if reuse-into-prompt lacks execution-grounded filter; DGM spine theorem cite; `elai case reuse` and `elai bench case-reuse` consumers. | 16-P:L514-L519 | P15-E010–P15-E011, P15-E014 |
| SD-P15-004 | **ADD** produces_symbols paths reuse_loop.rs/bbon_judge.rs, memory CanvasMode chrome, and shared P-14 R10 cadence. | 16-P:L515-L515 | P15-E007, P15-E009, P15-E017 |
| SD-P15-005 | **FLAG** bBoN appears in wargame expected observation while deferred — do not require bBoN for Tier-1 done. | 16-P:L509-L513 | P15-C001 |

### P-17 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P17-001 | **ADD** exact module paths failover.rs/strict_selection.rs and dispatch.rs production hook. | 16-P:L705-L705 | P17-E011, P17-E013 |
| SD-P17-002 | **ADD** FORK behavior for pinned-model cooldown (strict failure or Q-4 opt-in swap-restart only). | 16-P:L699-L700 | P17-E018 |
| SD-P17-003 | **ADD** Tier-2 deferrals (~140-plugin store, 5-min reprobe) and EXCLUDED silent sub / merged number. | 16-P:L680-L718 | P17-E019–P17-E020 |
| SD-P17-004 | **ADD** CLI failover-test fixture path, INTEGRATION_SMOKE multi-failure hard session, and redteam economics attack. | 16-P:L707-L710 | P17-E015, P17-E022–P17-E023 |
| SD-P17-005 | **EXPAND** A01-A03 with counter-move scope keys {provider, lane, model}. | 16-P:L714-L718 | P17-A001–P17-A002 |

### P-20 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P20-001 | **CORRECT** summary lists Dreaming promotion gates and per-provider cache mapping as atomic inventory items (P20-E18..E22): source DEFERs both to Tier-2; Tier-1 is flush + on-demand index only. | 16-P:L841-L848 | P20-E002, P20-E005, P20-C001 |
| SD-P20-002 | **ADD** exact module paths precompact_flush.rs/ondemand_index.rs and CompactionPass extension rule. | 16-P:L848-L848 | P20-E009–P20-E010 |
| SD-P20-003 | **ADD** trigger `elai session drive-to-compaction`, R10 double-threshold tool-heavy smoke, and P-19 split-rule composition. | 16-P:L849-L853 | P20-E012, P20-E017 |
| SD-P20-004 | **ADD** openclaw soft threshold 4000 tokens as cited prior art (not locked Tier-1 numeric) and sha256 version marker allowance. | 16-P:L824-L839 | P20-E003, P20-I001, P20-E015 |
| SD-P20-005 | **QUALIFY** frontend N/A beyond existing cost/memory panels — no new UI. | 16-P:L848-L848 | P20-E011 |

### P-22 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P22-001 | **CORRECT** summary inventory treats nine-step channel-binding, pairing codes, dmScope, signed device pairing, unknown-event withhold, idempotency keys, and trusted-policy hooks as peer Tier-1 atoms: source DEFERs all of those to Tier-2; Tier-1 is only fenced returns + depth-scoped tool-strip. | 16-P:L914-L939 | P22-E002, P22-E005–P22-E007, P22-C001 |
| SD-P22-002 | **ADD** exact openclaw pairing parameters (8-char, 1h, <=3 pending) as Tier-2 prior art and DEVICE_AUTH_* codes. | 16-P:L951-L952 | P22-E008 |
| SD-P22-003 | **ADD** FORK depth-1-only tool regain rule; ABORT if return hits instruction channel; Dual-LLM quarantine composition. | 16-P:L940-L945 | P22-E016–P22-E017, P22-E020 |
| SD-P22-004 | **ADD** modules fenced_return.rs/tool_strip.rs, spawn-test CLI fixture, and depth 2-3 R10 smoke. | 16-P:L938-L941 | P22-E010, P22-E014, P22-E019 |
| SD-P22-005 | **FLAG** stub_ban unknown-event withhold is unit-stated while feature is Tier-2 — contract intent vs ship scope. | 16-P:L931-L932 | P22-E021, P22-I001 |

### P-23 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P23-001 | **ADD** quantified latency FORK tripline p95 > 250ms and captureScreenshot degraded fallback. | 16-P:L616-L616 | P23-E019 |
| SD-P23-002 | **ADD** domain-first-seen warning as privacy egress row; UI-redress and phishing-form redteam outcomes. | 16-P:L591-L592 | P23-E008, P23-E022 |
| SD-P23-003 | **ADD** vertical-slice same-changeset wiring of CLI + front-door + H-13; production_caller_check on cobrowse modules. | 16-P:L601-L602 | P23-E025, P23-E021 |
| SD-P23-004 | **ADD** threaded E-14 contract (vendor if unexecuted) and P-14 crystallize RRA-bound replay in smoke. | 16-P:L598-L599 | P23-E012, P23-E023, P23-E027 |
| SD-P23-005 | **EXPAND** A01: summary said latency unquantified; source quantifies 250ms p95. | 16-P:L616-L616 | P23-E019 |

### P-25 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-P25-001 | **ADD** full V4 authority precedence, PLANNING CANDIDATE status, and ban on using V2/V3 PASS as execution authority. | 16-P:L955-L958 | P25-E001–P25-E002, P25-C001 |
| SD-P25-002 | **ADD** complete six-arm definitions including PAD_MATCHED envelope math, stratified EXO_SHUFFLED, SHUFFLED_REVISE permutation, and exo-card V2 field order. | 16-P:L969-L987 | P25-E007–P25-E013 |
| SD-P25-003 | **ADD** exact G1-G4 formulas (1.3x/2.5x ratios, -0.05 non-inferiority, 5000-sample seed 20260712) that summary A04 said were unreproduced. | 16-P:L1027-L1027 | P25-E016–P25-E022 |
| SD-P25-004 | **ADD** M0-M5 ordered pipeline, six M3 batches, curator CLI surface, content-addressed artifact layout, and atomic out_dir rename. | 16-P:L1003-L1008 | P25-E027–P25-E040, P25-E045 |
| SD-P25-005 | **ADD** full ABORT list, stub_ban injection rejections, FORKs (pool postpone, relevance-null pattern, bridge swap, core TCB block). | 16-P:L1052-L1058 | P25-E036, P25-E043–P25-E044, P25-E047–P25-E049 |
| SD-P25-006 | **CORRECT/QUALIFY** summary P25-E24 KEY_ALIASES and P25-E38 "import Q-10 modules": source requires exo-card V2 grammar as M3 deliverable and forbids growing/importing Q-10 runner; only stdlib paired-bootstrap extraction is shared. | 16-P:L983-L999 | P25-E013, P25-E042, P25-E043 |
| SD-P25-007 | **ADD** redteam multi-pass patches (stratified shuffle, ledger-leg exclusion, SHUFFLED_REVISE, multiplicity, card grammar differentiation). | 16-P:L996-L997 | P25-E051, P25-E053 |
| SD-P25-008 | **QUALIFY** summary A03 path reconciliation: plan still names exo_retrieval.py/exo_injection_ab.py as produces_symbols; tree drift is orientation-only until execution recon, not a source atom of absence. | 16-P:L1026-L1026 | P25-E041, P25-I001 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **7/7 files**, **1,035/1,035 logical lines**, **153,893/153,893 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **225 unique atoms** = **191 explicit**, **007 inferred**, **022 ambiguous**, **005 conflicting/corrective**.
- Per-file atom counts: P-3 **031**; P-15 **023**; P-17 **027**; P-20 **025**; P-22 **027**; P-23 **031**; P-25 **061**. Sum **225**.
- Relational interaction records: **19** (`IX-001`–`IX-019`). Summary-delta records: **038** (`SD-*`). Ambiguity/conflict register: **12** (`AC-001`–`AC-012`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `P3/P15/P17/P20/P22/P23/P25-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them. User-facing surfaces captured as distinct atoms: `elai app export|import`, `elai case reuse`, `elai provider failover-test`, RuntimePanel/CloudPanel labels, `elai session drive-to-compaction`, cost/memory panels (existing), `elai agents spawn-test`, control-surface enrollment/pairing (Tier-2), `elai browse`, front-door pull-up lane, canvas browse node + actor-colored trail, domain-first-seen warning, J-96 memory CanvasMode reuse-hit chrome, and P-25 curator/run-smoke/run-scored/checker CLIs.
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the seven exact DEEP-P-003 manifest rows. The only orientation artifact consulted was `16-P-SUMMARY_INDEX.discovery.md`.
- Output metrics after finalization: `output_lines=0447`; `output_bytes=070923`.
- Canonical normalized output SHA-256: cd4d04e853a4f244a6689094b982356a90d875d6ab8a23ecd00de6420eb962b7.
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.

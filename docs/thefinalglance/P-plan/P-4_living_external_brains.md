# P-4: living external brains — mount live systems (Obsidian, Notion, personal SQLite), not just a static markdown vault

STATUS: PLANNED (WAVE 2; PROOF_SPINE: none (product-surface / distribution class), owner-acked 2026-07-07; BUILD-on-shipped — GATED behind the Obsidian/store-API RECON before any code)

- Track: PERSONAL-INCARNATION | lever: mount-LIVE-not-static | source finding: GAP-4 (answers the user's "integrate with existing brains") | grounding: WEAK-until-RECON (Obsidian/Notion/store surfaces are training-only per doc; a source-grounding pass is a hard precondition) | nature: composition + per-store contract | effort: M-L | WAVE 2 | depends O-3 (vault contract) + P-6 (privacy wall)

- Premise: the user wants ELAI to "integrate with existing brains (Obsidian-class)". O-3 is the RIGHT shape (mount-not-convert, namespace claims, the contract IS the format) but it mounts a DETERMINISTIC markdown vault. People's actual brains are LIVE systems: an Obsidian vault under concurrent plugin edits, a Notion workspace, personal SQLite/spreadsheets. P-4 lifts O-3 from static paths to living stores with a bidirectional-safe contract per store class, F-5 staleness cascade + N-6 provenance tiers on every ingested byte, and writes ONLY inside claimed namespaces (O-3's rule, extended).

## Steal/build
Per store class, a `LivingStoreContract = { class: Obsidian|Notion|SQLite|Spreadsheet, watch (change feed), ingest (F-5 staleness + N-6 tier per byte), write_scope (O-3 namespace-claims — writes REFUSED outside claimed namespaces), ttl }`. B-35's context-artifacts pipeline is LIFTED from static paths to living stores with TTL + change feeds. The D-5/O-3 life-layer denylist becomes a HARD ROUTING WALL (life-data never enters a project/cloud prompt without an explicit, journaled grant — this is P-6's boundary, consumed here). The contract is per store CLASS, not per user (A-5 two-masters: one Obsidian contract serves every Obsidian user; the vault path is data).
EXCLUDED: converting a live store into memory_facts (O-3 mount-not-convert — the store stays the source of truth); a write outside a claimed namespace (O-3's hard-refusal rule); ingesting a byte WITHOUT an N-6 provenance tier + F-5 staleness stamp (every ingested byte is tiered + TTL'd); life-data flowing to a cloud prompt without a P-6 grant (hard routing wall); ANY code authored against an UNVERIFIED store-API surface (the RECON gate below is blocking — no training-memory Obsidian/Notion API).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- O-3 mounts a portable markdown brain vault with a namespace-claims contract (write outside a claim = hard refusal); memory_facts stays machine grain — P-4 extends O-3's contract to LIVE stores.
- B-35 is the context-artifacts pipeline over static paths — lift it to change-fed living stores.
- F-5 is staleness quarantine; N-6 is the provenance lattice — every ingested byte carries both.
- D-5 / O-3 life-layer denylist is the privacy routing wall — P-6 owns it; P-4 consumes it.

## RECON_NEEDED (BLOCKING — no code before these settle; doc flags Obsidian API as training-only, unsearched)
RECON_NEEDED: Obsidian's actual plugin/URI/file surface for watch + safe write (is there a documented API, or is it filesystem-watch over the vault dir + frontmatter?) -> settle via a WebSearch/source-grounding pass (paper-ingestor mode=source on the Obsidian API docs) BEFORE any P-4 code. The doc explicitly labels the training assumption UNGROUNDED; this recon is a hard precondition, not a courtesy.
RECON_NEEDED: Notion's API write/permission model + rate limits for a bidirectional-safe contract -> settle via WebSearch on the current Notion API (verify against notion MCP tool surface already connected this session) BEFORE authoring the Notion contract.
RECON_NEEDED: whether O-3's namespace-claims parser can be reused verbatim for a live store or needs a per-class adapter -> settle via reading O-3's shipped vault reader/writer + namespace-claims parser.
RECON_NEEDED: B-35's context-artifacts pipeline entrypoint (does it accept a change feed or only a static path list?) -> settle via `grep -rn "context.artifact\|B-35\|artifacts.pipeline" .elai_cc/crates`.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1, AFTER the Obsidian RECON): the LivingStoreContract type + an Obsidian filesystem-watch adapter (watch + ingest with F-5 staleness + N-6 tier per byte, WRITE-into-claimed-namespace only) + the journaled `vault_ingest` / `claim_violation` SSB rows. Obsidian first because it is filesystem-backed (lowest API risk); Notion/SQLite are separate adapters.
- DEFER (Tier-2): the Notion adapter (API-backed, rate-limited — separate contract) + the SQLite/spreadsheet adapters + bidirectional write-back beyond append (concurrent-edit conflict resolution is a genuine design problem — land with a merge-semantics spike, kin to P-8's shared-journal kernel).

## Integration
- crates/knowledge: the LivingStoreContract type + the Obsidian watch adapter + the F-5/N-6 tagging on ingest + the O-3 namespace-claims-gated writer + the SSB emit.
- CLI: `elai brain mount|sync <store>` subcommand is the Tier-1 production caller.
- privacy: every ingested byte's route-eligibility is checked against P-6's life/work/shared lattice at the O-3 vault boundary + E-20 routing (consumed, not built here).

## MOVES (wargame)
M1 (post-RECON) define the LivingStoreContract + the Obsidian filesystem-watch adapter that emits a change feed and ingests changed notes with an F-5 staleness stamp + an N-6 provenance tier per note.
EXPECTED_OBSERVATION: editing a note in a mounted Obsidian vault fires a change-feed event and lands a `vault_ingest` SSB row `{note_path, n6_tier, staleness_ttl}`; a note under concurrent plugin edit is ingested at its settled state, not mid-write.
ON_FAIL: a byte ingests without a provenance tier or TTL -> the trust/staleness invariant breaks -> tag every ingested note at the adapter boundary, assert the row fields.
M2 gate writes on O-3's namespace-claims: a write into an ELAI-claimed namespace succeeds; a write outside lands a `claim_violation` row and refuses.
EXPECTED_OBSERVATION: `elai brain sync` writing into a claimed namespace creates/updates a page; a write targeting an unclaimed (user-owned) namespace is a hard refusal with a `claim_violation` row.
ON_FAIL: a write mutates a user's own note -> the claim gate wasn't enforced on the live-store path -> route every write through the O-3 claims check, add the negative test.
M3 enforce the P-6 life-layer routing wall: an ingested life-domain byte cannot enter a project/cloud prompt without an explicit journaled grant.
EXPECTED_OBSERVATION: a life-domain note routed toward a cloud prompt without a grant is blocked + journaled; with a grant, it routes and the grant is on record.
ON_FAIL: life-data reaches a cloud prompt ungranted -> the routing wall wasn't consulted -> check the P-6 lattice at the E-20 routing boundary, assert the block.

## FORK
FORK: if the Obsidian RECON reveals NO safe programmatic write surface (filesystem-only, plugin-conflict-prone) -> ship Tier-1 as READ-ONLY mount (watch + ingest + tier, no write-back) and route all write-back to Tier-2 behind a per-class safe-write design; record the write-safety seam in the decision doc. A read-only living mount already delivers the "integrate with existing brains" value; unsafe writes are worse than none.

## ABORT
ABORT: if any P-4 code would be authored against a store-API surface that is training-asserted and NOT source-grounded this session -> stop and flag; the doc labels the Obsidian/Notion surfaces UNGROUNDED and the RECON is blocking. Anti-slopsquatting/anti-stale posture: no API called before its shape is verified live.

## WIRING_CONTRACT
produces_symbols:
  - the LivingStoreContract type + the Obsidian filesystem-watch adapter (watch + F-5/N-6-tagged ingest)
  - the O-3-namespace-claims-gated live-store writer + the journaled `vault_ingest` / `claim_violation` SSB rows
  - the `elai brain mount|sync` CLI subcommand
consumes_symbols:
  - O-3 vault contract + namespace-claims parser, B-35 context-artifacts pipeline, F-5 staleness, N-6 provenance lattice, D-5/O-3 life denylist, P-6 privacy lattice + E-20 routing
reachable_from:
  - the `elai brain mount|sync` CLI subcommand on a live run mounts + ingests a store  [PRODUCTION CLI]
  - the filesystem-watch adapter fires on a live vault edit  [PRODUCTION watcher]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO conversion of a live store into memory_facts (mount-not-convert)
  - NO write outside a claimed namespace
  - NO ingested byte without an N-6 tier + F-5 staleness stamp
  - NO code against an unverified store-API surface (RECON is blocking)
observable_effect:
  trigger: "mount a fixture Obsidian vault, edit a note in a claimed namespace, then attempt a write into an unclaimed namespace"
  expect_ssb: "a vault_ingest row {note_path, n6_tier, staleness_ttl} on the edit; a claim_violation row on the unclaimed-namespace write"
  expect_artifact: "the claimed-namespace page updates; NEGATIVE: a mutation of a user-owned note, or an ingest missing the tier/TTL, must FAIL"

PRODUCTION_HOOK: cli/main.rs `elai brain mount`/`sync` subcommand dispatch calls the LivingStoreContract adapter + the O-3-claims-gated writer + the SSB emit; the filesystem-watch adapter registers on mount — the executor names the exact dispatch + watcher-registration fns at RECON and wires them same-changeset.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-4_living_external_brains.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
pass = a `vault_ingest` row landed off-agent from a real vault edit with an N-6 tier + F-5 TTL, and an unclaimed-namespace write was refused with a `claim_violation` row.

INTEGRATION_SMOKE: an independent agent mounts a fixture Obsidian vault with both an ELAI-claimed and a user-owned namespace, edits notes in both under simulated concurrent edits, attempts a life-domain note's route toward a cloud prompt (must block without a grant, route with one), and asserts ingest tiering + the claim_violation on the user namespace + the routing-wall block — a multi-namespace run firing the concurrency + claim + routing paths (R10).

## REDTEAM
REDTEAM: attack that FAILED — "exfiltrate a private health note to a cloud model by mounting it into a project namespace": FAILS because the P-6 life-layer routing wall (M3) blocks a life-domain byte from a cloud prompt without a journaled grant, enforced at the E-20 boundary. Attack that SUCCEEDED and its patch — "corrupt the user's vault by writing during a concurrent plugin edit": patched by ingesting at the settled state + gating all writes on the O-3 namespace claim (never a user-owned namespace) and, per the FORK, defaulting to read-only until the RECON proves a safe write surface.

## Grounding
- design digest workflow/reasoning/elai-personal-software-incarnation-2026-07-07.md GAP-4 (explicitly flags the Obsidian surface as UNGROUNDED training memory, requiring a RECON WebSearch/source-grounding pass) + the user's "integrate with existing brains".
- Ink & Switch malleable-software essay (inkandswitch.com/essay/malleable-software) + geoffreylitt.com — personal-software integration with existing tools as the named field.
- feature-map: O-3 vault contract, B-35 context-artifacts, F-5 staleness, N-6 lattice, D-5 privacy denylist, E-20 privacy routing.
- NOTE: the notion MCP server is connected this session (notion-* tools) — use it as a live grounding source for the Notion adapter RECON rather than training memory.

## COMPOSES-WITH
O-3 vault contract (EXTEND to live stores), B-35 (lift to change feeds), F-5/N-6, D-5 denylist, P-6 privacy lattice + E-20. Re-implements NONE (GD-1). P-6 owns the life/work/shared routing wall this unit enforces; the concurrent-edit write-back merge is deferred to a spike kin to P-8.

End of P-4.

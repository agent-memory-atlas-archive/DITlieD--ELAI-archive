# O-3: brain vault contract + mounting — a portable markdown vault mounted (not converted), with namespace claims

STATUS: PLANNED (WAVE 2; PROOF_SPINE: none (product-surface class), owner-acked 2026-07-06; BUILD-on-shipped — the user-grain surface)

- Track: RESEARCH-MACHINE | lever: mount-not-convert | source finding: L2 | grounding: MODERATE (index-derived, PLAN-asserted) | nature: BUILD-on-shipped | effort: M | WAVE 2

- Premise: "brain migration" should cost zero conversion. The brain splits by grain: E-5 memory_facts stays the MACHINE-grain fact store feeding prompts (auditor firewall intact), and a deterministic markdown vault is the USER-grain surface (Obsidian/GDrive readable and editable by the owner). If the vault CONTRACT is the format (markdown + YAML frontmatter + wikilinks + machine-owned namespaces), then ELAI mounts the existing teikoku brain vault instead of importing it. The one NEW mechanism this requires is a namespace-claims file so two harnesses writing one vault cannot collide.

## Steal/build
The vault contract, stolen verbatim from teikoku brain_store semantics: `topics/<t>/<slug>.md`, YAML frontmatter, wikilinks, `rankings/`+`queue/`+`index` living pages; `write_page` is create/append-only; `upsert_page` is guarded and restricted to machine-owned namespaces (never edits user notes). NEW MECHANISM: a `namespace-claims` file at the vault root declaring which machine owns which namespaces; a write outside a claimed namespace is a HARD write-refusal (so ELAI + teikoku can share one vault). Grain split: memory_facts (SQLite, Mem0 dispatch, decay, MINJA defense) unchanged as the prompt-feeding store; the vault is the owner surface. Privacy: a D-5-style life-layer denylist — personal topics never enter project prompts or cloud calls; vault reads route through the existing privacy redaction / air-gapped layer.
EXCLUDED: converting the teikoku vault into memory_facts (the whole point is mount-not-convert); editing user-authored notes (write is create/append + machine-namespace upsert only); routing personal-brain content into project prompts (D-5 life-layer denylist forbids it); a second fact store (memory_facts stays the machine grain).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- E-5 memory_facts: SQLite store, Mem0-style dispatch, decay, MINJA injection defense; the machine-grain prompt-feeding fact store (feature-map memory entry).
- D-5 life-layer denylist + a privacy redaction / air-gapped read layer are SHIPPED (design overview section 11).
- teikoku brain_store `write_page`/`upsert_page` namespace discipline + living rankings/queue/index pages are LIVE-proven (memory project_teikoku_research_machine + the teikoku vault on disk).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the vault contract reader/writer (frontmatter + wikilinks + `topics/<t>/<slug>.md`), the guarded create/append + machine-namespace upsert, the namespace-claims file + hard write-refusal, and the mount of an existing vault path.
- DEFER (Tier-2): the D-5 life-layer denylist enforcement on vault reads into prompts (land with the privacy-layer wiring) + bidirectional sync between memory_facts and vault views.

## Integration
- crates/knowledge (memory crate): the vault reader/writer module + the namespace-claims parser + the guarded-upsert path; the mount point resolves a configured vault root.
- frontend_pair: a read-only vault page view can render in the dashboard (or defer entirely to Obsidian/GDrive) — N/A as a new panel is justified: the vault is deliberately an external-editor surface, not an ELAI-owned UI.

## MOVES (wargame)
M1 implement the vault contract reader/writer (parse frontmatter + wikilinks; `topics/<t>/<slug>.md` layout) and mount a configured existing vault root read-only first.
EXPECTED_OBSERVATION: mounting the existing teikoku vault path lists its topics/pages without modification; frontmatter + wikilinks round-trip byte-identically.
ON_FAIL: a mounted page is rewritten/reformatted on read -> the reader is not preserving bytes -> make read pure (no normalize-on-load), add a round-trip byte-identity test.
M2 add the namespace-claims file at the vault root and gate every write on it; a write outside a claimed namespace hard-refuses.
EXPECTED_OBSERVATION: ELAI writing into an unclaimed (teikoku-owned) namespace is refused with a claim-violation error; writing into an ELAI-claimed namespace succeeds create/append-only.
ON_FAIL: a cross-namespace write goes through -> the claim check is not on the write path -> move the claim check into the single write chokepoint, add the refusal test.
M3 restrict `upsert_page` to machine-owned living pages (rankings/queue/index); user notes are never editable, only create/append.
EXPECTED_OBSERVATION: upsert updates a machine-owned rankings page in place; an upsert targeting a user note is refused.
ON_FAIL: an upsert edits a user note -> the machine-owned predicate is missing -> add the namespace + page-class predicate to upsert, test the user-note refusal.

## FORK
FORK: if the teikoku vault on disk is NOT reachable from this host (WSL-only tree) -> ship Tier-1 against a fresh ELAI-claimed vault and record the mount of the teikoku vault as a Tier-2 follow-up gated on a shared path; do not block O-3 on cross-host access.

## RECON_NEEDED
RECON_NEEDED: the exact teikoku brain_store on-disk layout + frontmatter schema to mirror -> settle via reading the teikoku vault root + `write_page`/`upsert_page` source (memory project_teikoku_research_machine points at it) BEFORE authoring the reader.
RECON_NEEDED: whether E-5 memory_facts exposes a boundary the vault should stay disjoint from (no accidental dual-write) -> settle via `grep -rn "memory_facts\|Mem0\|MINJA" .elai_cc/crates` and read the store API.
RECON_NEEDED: the D-5 life-layer denylist API + the privacy/air-gapped read layer entrypoint -> settle via `grep -rn "life.layer\|denylist\|air.gap\|redact" .elai_cc/crates`.

## ABORT
ABORT: if honoring the mount-not-convert contract would require editing user-authored notes to fit ELAI's schema -> stop and flag; the contract is that ELAI adapts to the vault format, the vault never adapts to ELAI.

## WIRING_CONTRACT
produces_symbols:
  - the vault contract reader/writer (frontmatter + wikilinks + topics/<t>/<slug>.md)
  - the namespace-claims file parser + the hard write-refusal on unclaimed namespaces
  - the guarded create/append + machine-namespace-only upsert_page
consumes_symbols:
  - E-5 memory_facts (stays the machine grain — do NOT dual-write or re-implement, GD-1)
  - D-5 life-layer denylist + the privacy/air-gapped read layer
  - teikoku brain_store semantics (the contract stolen verbatim)
reachable_from:
  - a research daemon FileToBrain stage (O-1/O-2) writes a page through the guarded writer on a live run  [PRODUCTION vault write seam]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO write outside a claimed namespace (hard-refuse)
  - NO edit of a user-authored note (create/append + machine upsert only)
  - NO conversion of the mounted vault into memory_facts
  - NO personal-brain content routed into a project/cloud prompt (D-5)
observable_effect:
  trigger: "mount the vault, then run a FileToBrain write into an ELAI-claimed namespace and an attempted write into an unclaimed one"
  expect_ssb: "a vault_write row {namespace, page, mode=create|append|upsert} for the claimed write; a claim_violation row for the unclaimed attempt"
  expect_artifact: "the created page on disk with valid frontmatter + wikilinks; NEGATIVE: any write into an unclaimed namespace or any mutation of a user note must FAIL"

PRODUCTION_HOOK: the O-1/O-2 FileToBrain sink stage calls the guarded vault writer — the executor wires the FileToBrain stage kind to this writer in the SAME changeset (or, if O-1/O-2 not yet landed, a CLI `elai brain write` subcommand is the interim production caller, named at RECON).

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/O-plan/O-3_brain_vault_contract_mounting.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
pass = a vault page was created via the guarded writer from a real production caller with valid frontmatter, and an unclaimed-namespace write produced a claim_violation row (off-agent), not a silent success.

INTEGRATION_SMOKE: an independent agent mounts a vault already carrying a teikoku-owned namespace, drives an ELAI FileToBrain write into an ELAI namespace AND an attempted write into the teikoku namespace, and asserts the create + the hard refusal + a user-note upsert refusal — a shared-vault two-owner run, not a single-owner happy path (R10).

## REDTEAM
REDTEAM: attack that FAILED — "corrupt teikoku's notes by having ELAI upsert into a shared vault": FAILS because the namespace-claims file hard-refuses any write outside ELAI's claimed namespace and upsert is machine-owned-page-only. Attack that SUCCEEDED and its patch — "leak a personal-brain topic into a project prompt via a research briefing": patched by the D-5 life-layer denylist on vault reads into prompts (Tier-1 stub_ban; the denylist enforcement is the Tier-2 wiring that must land before personal topics are mountable into project context).

## Grounding
- design digest workflow/reasoning/elai-personal-research-machine-2026-07-06.md L2 + A4 (two-masters / topic-pack split) + the steal list.
- memory project_teikoku_research_machine.md (brain_store write_page/upsert_page semantics, LIVE).
- feature-map E-5 memory + D-5 life-layer/privacy entries.

## COMPOSES-WITH
E-5 memory_facts (disjoint machine grain), D-5 life-layer denylist + privacy layer, teikoku brain_store contract, O-1/O-2 FileToBrain sink. Re-implements NONE (GD-1). The vault is the user-grain surface; memory_facts stays the prompt-feeding machine grain; mounting replaces migration.

End of O-3.

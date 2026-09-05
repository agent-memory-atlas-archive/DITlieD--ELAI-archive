# P-6: the life-data privacy lattice — which TOOL may read which NAMESPACE; taint's mirror image (owned bytes flowing OUT)

STATUS: PLANNED (WAVE 2; PROOF_SPINE: none (product-surface / distribution class), owner-acked 2026-07-07; BUILD-on-shipped — extends N-6's ONE lattice with a privacy dimension, no second lattice)

- Track: PERSONAL-INCARNATION | lever: out-flow-privacy-dimension | source finding: GAP-6 (answers T6 owned-and-private at the grain the thread lives at) | grounding: MODERATE (index-derived, PLAN-asserted) | nature: composition (extend N-6, do NOT mint a second lattice) | effort: M | WAVE 2 | depends N-6 (lattice) + O-3 (vault boundary)

- Premise: the thread's owned-and-private demand — "an app I don't need to login to" (Azko87); the birdwatcher app that "lives on my phone and my wife's phone and that's all it needs to be" (Upbeat-Armadillo1756). G-9 protects secrets/PII in journals; E-20 routes privacy-tiered prompts. The personal world adds a THIRD axis: WHICH TOOL may read WHICH NAMESPACE. The budget app's data and the health app's data live in one vault; a per-namespace domain label (life/work/shared) + per-tool read grants, enforced at the O-3 vault boundary AND at E-20 routing, every grant human-rendered and journaled. This is G-1 taint's MIRROR IMAGE: taint tracks untrusted bytes flowing IN toward sinks; the privacy lattice tracks OWNED bytes flowing OUT toward models/tools. Per A-1 / N-2 duplicate-concept check: EXTEND N-6's mapping table with a privacy dimension, never a second lattice.

## Steal/build
Extend the N-6 provenance-lattice mapping table with a `domain: life|work|shared` dimension per namespace, and add `read_grants: [{tool_id, namespace, granted_at, journal_ref}]`. Enforcement at TWO boundaries: (a) the O-3 vault read path — a tool reading a namespace it holds no grant for is refused; (b) E-20 routing — an owned byte's `domain` gates whether it may enter a given model/tool prompt. Every grant is human-rendered (N-7) + journaled; a grant is the OUT-flow analog of G-1's taint tier on the IN-flow. The default is deny (no grant = no read); a grant is an explicit, journaled, revertible act.
EXCLUDED: a SECOND privacy lattice (extend N-6's ONE mapping table — A-1/N-2 hard authoring FAIL otherwise); an implicit/default-allow read (default is deny; a read needs a grant); a grant that isn't journaled + revertible (H-49 reversibility applies); mixing the OUT-flow privacy dimension with the IN-flow taint tier into one confused field (they are mirror dimensions on the same lattice, distinctly labeled).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- N-6 is the ONE provenance lattice + admission; N-2 forbids a second scale — P-6 adds a dimension to N-6's table, not a new lattice.
- G-1 taint tracks untrusted bytes flowing IN toward sinks — P-6 is the mirror (owned bytes flowing OUT).
- G-9 is write-time secret/PII redaction; E-20 is privacy-tier routing — P-6's grants gate at the E-20 boundary.
- O-3 owns the vault boundary + namespace claims — P-6 enforces read-grants at that same boundary.
- N-7 renders every grant; H-49 reversibility makes a grant revertible.

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): the `domain` dimension on the N-6 namespace mapping + the `read_grants` table + the O-3 vault-read enforcement (no grant = refuse) + the journaled `privacy_grant` / `privacy_read_refusal` SSB rows + the N-7 render of each grant.
- DEFER (Tier-2): the E-20 routing-boundary enforcement (threads the routing seam — land with the P-4 living-brain routing wall, which is the same enforcement point) + a grant-management operator surface (frontend, lands with P-5's operator panel).

## Integration
- crates/knowledge (or wherever N-6 lives): the `domain` dimension + `read_grants` table + the vault-read enforcement hook + the SSB emit.
- CLI: `elai privacy grant|revoke|show <tool> <namespace>` subcommand is the Tier-1 production caller.
- routing (Tier-2): the E-20 router consults the lattice before an owned byte enters a prompt.

## MOVES (wargame)
M1 add the `domain: life|work|shared` dimension to the N-6 namespace mapping (extend the existing table, do NOT create a new one) + the `read_grants` table, default-deny.
EXPECTED_OBSERVATION: labeling a namespace `life` and querying the lattice returns the domain; a tool with no grant for a namespace resolves to DENY.
ON_FAIL: the domain becomes a second table/lattice -> A-1/N-2 authoring FAIL -> add the column to N-6's existing mapping, run corpus_composition_check (N-2) at authoring.
M2 enforce read-grants at the O-3 vault-read boundary: a granted tool reads; an ungranted tool is refused with a `privacy_read_refusal` row.
EXPECTED_OBSERVATION: `elai privacy grant budget-tool budget-ns` then a budget-tool read of budget-ns succeeds; a budget-tool read of health-ns (no grant) is refused + lands a `privacy_read_refusal` row.
ON_FAIL: an ungranted read succeeds -> the enforcement hook wasn't on the O-3 read path -> gate every vault read on the grants table, assert the refusal (R1 missing-evidence = FAIL).
M3 render + journal every grant via N-7, revertible (H-49).
EXPECTED_OBSERVATION: granting emits a `privacy_grant` row + an N-7 human-rendered line; `elai privacy revoke` reverts it and the tool loses read access.
ON_FAIL: a grant isn't revertible or isn't rendered -> the transparency/reversibility invariant breaks -> route grants through N-7 + make revoke an inverse journaled op.

## FORK
FORK: if N-6's mapping table cannot take a new dimension column without a schema migration that touches claim objects (N-4) -> ship Tier-1 as a JOIN table keyed on the N-6 namespace id (a dimension ON the lattice, not a second lattice — still N-2-clean) rather than an in-place column, and record the N-6-schema seam in the decision doc. The invariant is "one lattice, one source of truth for provenance+privacy", not "one physical table".

## RECON_NEEDED
RECON_NEEDED: the N-6 mapping-table schema + whether it admits a new dimension without touching N-4 claim objects -> settle via reading N-6's shipped lattice + N-4 claim object + `grep -rn "N-6\|provenance\|lattice\|admission" .elai_cc/crates`.
RECON_NEEDED: the O-3 vault-read entrypoint where a read-grant check can be inserted -> settle via reading O-3's shipped vault reader + `grep -rn "vault.*read\|O-3\|namespace" .elai_cc/crates`.
RECON_NEEDED: the E-20 routing boundary where an owned byte's domain gates prompt entry (Tier-2) -> settle via `grep -rn "E-20\|privacy.*rout\|prompt.*tier" .elai_cc/crates`.

## ABORT
ABORT: if adding the privacy dimension requires minting a SECOND provenance/privacy lattice (A-1 / N-2 duplicate-concept) -> stop and flag; the corpus has exactly one lattice by doctrine. A second scale is an automatic authoring FAIL — extend N-6 or file the schema gap.

## WIRING_CONTRACT
produces_symbols:
  - the `domain: life|work|shared` dimension on the N-6 namespace mapping + the `read_grants` table (default-deny)
  - the O-3 vault-read grant enforcement + the journaled `privacy_grant` / `privacy_read_refusal` SSB rows + N-7 grant render
  - the `elai privacy grant|revoke|show` CLI subcommand
consumes_symbols:
  - N-6 provenance lattice (EXTEND, do NOT duplicate), N-2 corpus-composition check, G-1 taint (mirror), G-9 redaction, E-20 routing, O-3 vault boundary, N-7 render, H-49 reversibility
reachable_from:
  - the `elai privacy grant|revoke` CLI subcommand on a live run  [PRODUCTION CLI]
  - the O-3 vault-read path enforces a grant on a live read  [PRODUCTION read boundary]
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO second privacy/provenance lattice (extend N-6 — A-1/N-2)
  - NO default-allow read (default is deny; a read needs a grant)
  - NO grant that isn't journaled + revertible
  - NO ungranted vault read that succeeds without a refusal row (R1)
observable_effect:
  trigger: "grant budget-tool read on budget-ns, then have budget-tool read budget-ns AND health-ns"
  expect_ssb: "a privacy_grant row on the grant; the budget-ns read succeeds; the health-ns read lands a privacy_read_refusal row"
  expect_artifact: "an N-7 human-rendered grant line; a revoke that removes read access; NEGATIVE: an ungranted read that succeeds, or a second lattice table, must FAIL"

PRODUCTION_HOOK: cli/main.rs `elai privacy` subcommand dispatch writes/reverts grants AND the O-3 vault-read path consults the grants table each read — the executor names the exact vault-read fn at RECON and wires both same-changeset.

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/P-plan/P-6_life_data_privacy_lattice.md --run-trigger`
`python .claude/tools/production_caller_check.py --files <changed .rs> --json`   # DARK new pub fn = BLOCK
`python .claude/tools/corpus_composition_check.py` (N-2) MUST pass — no second lattice was minted
pass = an ungranted vault read was refused off-agent with a `privacy_read_refusal` row, a granted read succeeded, the grant was journaled + revertible, and N-2 confirms the privacy dimension rides N-6's single lattice.

INTEGRATION_SMOKE: an independent agent labels three namespaces (budget=life, project=work, notes=shared), grants one tool read on one namespace, then drives cross-namespace reads from multiple tools + a revoke mid-run, asserting every ungranted read refuses + the grant/revoke journal + no owned byte crossing to an ungranted tool — a multi-tool multi-namespace run firing the deny + revoke paths (R10).

## REDTEAM
REDTEAM: attack that FAILED — "read the health namespace from the budget tool by relabeling it shared": FAILS because relabeling is a journaled N-6 mapping change (visible + revertible) and the read still needs an explicit grant; there is no implicit-allow path. Attack that SUCCEEDED and its patch — "smuggle a second privacy scale in as 'just a helper table' and Goodhart the lattice": patched by the N-2 corpus_composition_check at authoring (the FORK's join-table stays a dimension ON N-6, not a rival lattice) — a second scale is an automatic authoring FAIL.

## Grounding
- design digest workflow/reasoning/elai-personal-software-incarnation-2026-07-07.md GAP-6 + T6 mapping + A-1 (no second lattice; N-2 duplicate-concept check binds).
- reddit r/ClaudeAI thread archive — Azko87 "an app I don't need to login to"; Upbeat-Armadillo1756 the birdwatcher app "lives on my phone and my wife's phone".
- feature-map: N-6 provenance lattice, N-2 corpus-composition validator, G-1 taint, G-9 redaction, E-20 privacy routing, O-3 vault boundary, N-7 render, H-49 reversibility.

## COMPOSES-WITH
N-6 lattice (EXTEND with a privacy dimension), N-2 (authoring gate), G-1 (mirror concept), G-9, E-20, O-3 boundary, N-7, H-49. Re-implements NONE (GD-1). O-3 owns the vault boundary P-6 enforces at; P-4's living-brain routing wall is the same E-20 enforcement point (Tier-2 shared).

End of P-6.

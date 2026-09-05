# Q-9: distribution hygiene — fork-merge markers on the vendored CLIs + one-registry surface contract (two cheap Kilo/Hermes steals)

STATUS: PLANNED (WAVE 3; two distribution units in one file; EXTENSION(4M vendored runtimes) + EXTENSION(ts-rs drift gate + J MCP control plane))

- Track: DISTRIBUTION | lever: maintenance-economics-without-new-build | source: spec-eval-elai-vs-hermes-kilo ADDENDUM §"Worth closing" | grounding: STRONG (source-grounded Kilo/Hermes overviews, HEAD-pinned 2026-07-10) | nature: EXTENSION x2 | effort: S-M | WAVE 3
- PROOF_SPINE: none (distribution). Owner DELEGATED the authoring call 2026-07-10 ("i dont really know, you can make this call") — recorded as the PSR-2 ack by explicit delegation. Feeds no PS-1..6 number; both are maintenance-economics units the spec-eval addendum ranks AFTER the proof spine (PS-2/PS-4 first, then these).

- Premise (spec-eval-elai-vs-hermes-kilo ADDENDUM): the two axes where a comparator's DESIGN is genuinely better and the fix is cheap + on-doctrine. (U1) Kilo's fork-merge hygiene (kilocode_change markers + CI-enforced quarantine) continuously absorbs upstream while keeping the diff small — ELAI's Phase-0 fork diverged irreversibly (nothing to retrofit there), but 4M VENDORS coding-agent CLIs with live fast-moving upstreams, and applying Kilo's marker-quarantine discipline to THOSE captures the maintenance-economics win nearly free. (U2) Kilo's SDK-generated-from-server clients / Hermes's derive-everything-from-one-COMMAND_REGISTRY eliminate per-client drift by construction — ELAI is most of the way there (ts-rs bindings + a drift gate + the J MCP control-plane one-server/thin-clients shape); formalizing "every surface derives from one registry" is an N-letter-style consolidation, not new build.

## Steal/build
UNIT A (fork hygiene on vendored CLIs): mark every ELAI-side edit inside a 4M-vendored coding-agent CLI with an `elai_change` marker (the kilocode_change idiom), CI-quarantine-checked so an upstream re-vendor keeps the ELAI diff small + visible and an unmarked drift is a BLOCK. UNIT B (one-registry surface contract): a single COMMAND/SURFACE registry from which the dashboard/CLI/canvas/MCP surfaces DERIVE (the ts-rs drift gate extended from types to the command surface), so a surface that names a command not in the registry (or a registry command no surface exposes) is a drift BLOCK. Both are TOOLING/CI properties (markers + drift checks), not runtime features — the observable effect is a checker exit code, not an SSB row.
EXCLUDED: retrofitting fork-hygiene onto the Phase-0 ELAI fork (it diverged irreversibly — the addendum is explicit; the marker discipline applies ONLY to the 4M vendored CLIs with live upstreams); building a NEW SDK generator from scratch (ELAI already has ts-rs + the J MCP control plane — Q-9B EXTENDS the drift gate, it does not re-invent code-gen); a second surface registry (A-1/N-2 one-source-of-truth — the registry lives ONCE).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)
- 4M RuntimeAdapter vendors coding-agent CLIs (the vendored runtimes with live upstreams) — UNIT A's target.
- Kilo `kilocode_change` markers + CI quarantine (source-grounded, KILOCODE-SPEC-OVERVIEW HEAD 1426dca 2026-07-10) — the marker/CI idiom UNIT A ports.
- ts-rs bindings + the cross_lang_diff drift gate (.claude/tools/cross_lang_diff.py) — UNIT B extends this from type-drift to command-surface drift.
- J-letter ELAI-as-MCP-server control plane (one-server / thin-clients) — the shape UNIT B formalizes.
- Hermes derive-everything-from-COMMAND_REGISTRY + Kilo SDK-gen-from-server (source-grounded, HERMES/KILOCODE overviews 2026-07-10) — the design UNIT B mirrors.
- N-2 corpus_composition_check / A-1 one-source-of-truth — the invariant (one registry, not two).

## REVISED scope (W-10 tiered)
- SHIP NOW (Tier-1): UNIT A = the `elai_change` marker convention + the CI quarantine check over the 4M vendored-CLI tree (unmarked ELAI-side drift = BLOCK). UNIT B = the surface registry + the drift check extending cross_lang_diff to command-surface symmetry (a surface command not in the registry, or a registry command exposed by no surface = BLOCK).
- DEFER (Tier-2): generating thin-client stubs FROM the registry (the full SDK-gen — land after the registry + drift check prove the single-source shape).

## Integration
- .claude/tools + CI: UNIT A the marker-quarantine checker over the vendored-CLI paths; UNIT B the command-surface drift check (extends cross_lang_diff.py).
- the surface registry lives in the repo (crates or a manifest) as the single source the dashboard/CLI/canvas/MCP derive from.
- frontend_pair: UNIT B's registry↔surface symmetry IS a cross-language pair (Rust command defs ↔ TS/Svelte surface + MCP) checked by the extended drift gate; justified as the pair itself.

## MOVES (wargame)
M1 (UNIT A) define the `elai_change` marker + a CI quarantine check over the 4M vendored-CLI tree: an ELAI-side edit inside a vendored CLI must carry the marker; an unmarked diff vs the vendored baseline is a BLOCK.
EXPECTED_OBSERVATION: a marked ELAI edit passes; an unmarked change inside a vendored CLI path fails the CI check with the file+line; a re-vendor of upstream keeps the marked ELAI diff isolated + small.
ON_FAIL: an unmarked drift passes -> the fork-hygiene is not enforced -> the check greps the vendored paths for unmarked non-upstream diffs; assert an unmarked edit BLOCKS.
M2 (UNIT B) build the surface registry + extend cross_lang_diff to command-surface symmetry: a command exposed by a surface (CLI/dashboard/canvas/MCP) but absent from the registry, or a registry command exposed by no surface, is a drift BLOCK.
EXPECTED_OBSERVATION: a registry-derived surface passes; a surface command with no registry entry (or vice-versa) fails the drift check naming the orphan; the registry is the single source.
ON_FAIL: a surface defines a command outside the registry and passes -> per-client drift persists -> the drift check treats the registry as authoritative; assert an off-registry command BLOCKS.
M3 wire both checks into the auditor's cross_lang / freeze pass so they run off-agent per changeset (frozen, R8).
EXPECTED_OBSERVATION: the auditor's pass runs both checks; a changeset touching a vendored CLI or a surface command triggers them; a clean changeset passes.
ON_FAIL: the checks are optional / self-run -> in-boundary -> add them to the frozen auditor toolset (gate-manifest); the executing run cannot skip or edit them.

## FORK
FORK: if 4M does NOT currently vendor any coding-agent CLI in-tree (the vendoring is runtime-fetch, not checked-in source) -> UNIT A ships the marker CONVENTION + the check as a NO-OP-until-vendored guard (documented, activates when the first CLI is vendored in-tree) and the effort collapses to UNIT B; record the seam. Do NOT fabricate a vendored tree to test against.

## RECON_NEEDED
RECON_NEEDED-1: whether 4M vendors CLI SOURCE in-tree (checked-in) or fetches at runtime -> settle via `grep -rn "vendor\|RuntimeAdapter\|cli.*adapter" .elai_cc/crates/providers/src` + inspect for a checked-in vendored tree (decides UNIT A shape via the FORK).
RECON_NEEDED-2: the cross_lang_diff.py extension seam + where command/surface defs live -> settle via reading .claude/tools/cross_lang_diff.py + `grep -rn "generate_handler\|invoke\|command\|subcommand" .elai_cc/crates`.
RECON_NEEDED-3: the existing surface command sources (CLI args, Tauri handlers, MCP tools, canvas ops) that must derive from the registry -> settle via `grep -rn "Subcommand\|generate_handler\|mcp.*tool\|canvas.*op" .elai_cc/crates`.

## ABORT
ABORT: if UNIT B would require a SECOND surface registry (a new source of truth alongside an existing command source) -> STOP; A-1/N-2 mandate ONE source — the registry must be the single authority the surfaces derive from, or UNIT B is refused. A second registry is the exact drift the unit exists to kill.

## WIRING_CONTRACT
produces_symbols:
  - UNIT A: the `elai_change` marker convention + the vendored-CLI quarantine CI check (frozen, auditor-owned)
  - UNIT B: the single surface/command registry + the cross_lang_diff command-surface symmetry check
  - both wired into the auditor's frozen cross-lang/freeze pass (no new SSB action_type — these are build/CI checks)
consumes_symbols:
  - 4M vendored RuntimeAdapter CLIs; Kilo kilocode_change marker idiom; cross_lang_diff.py (the drift-gate seam); J MCP control plane (the thin-clients shape); ts-rs bindings; N-2 corpus_composition_check / A-1 one-source
reachable_from:
  - the auditor's cross-lang/freeze pass runs both checks per changeset [PRODUCTION auditor CI gate]; a vendored-CLI drift or an off-registry command BLOCKS
  - a #[cfg(test)] caller NEVER satisfies this (R7)
stub_ban:
  - NO retrofitting markers onto the diverged Phase-0 ELAI fork (only the 4M vendored CLIs)
  - NO second surface registry (one source; A-1/N-2)
  - NO new-from-scratch SDK generator (extend the ts-rs/cross_lang drift gate)
  - NO self-run/optional check (frozen auditor-owned, R8)
observable_effect:
  trigger: "make an unmarked edit inside a vendored CLI path; separately, add a CLI subcommand not present in the surface registry; run the auditor cross-lang/freeze pass"
  expect_artifact: "the vendored-CLI check BLOCKS the unmarked edit (file+line); the drift check BLOCKS the off-registry command (named orphan); a marked edit + a registry-derived command pass. NEGATIVE: an unmarked vendored drift or an off-registry surface command that passes must FAIL"

PRODUCTION_HOOK: producer UNIT A's quarantine check + UNIT B's command-surface drift check -> consumer the auditor's frozen cross-lang/freeze pass (extends cross_lang_diff.py / the gate-manifest), wired same-changeset; the checker exit codes are the off-agent artifacts (these are CI/tooling units — the artifact is a frozen-checker verdict, not an SSB row, which is the correct W-1 observable for a build-hygiene gate).

## VERIFY
VERIFY: the auditor runs, off-agent:
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-9_distribution_hygiene.md --run-trigger`
`python .claude/tools/gate_freeze_check.py`   # both checks are frozen auditor artifacts (R8)
pass = the vendored-CLI quarantine check BLOCKED an unmarked edit and the command-surface drift check BLOCKED an off-registry command, both from the frozen auditor pass off-agent, with a clean changeset passing.

INTEGRATION_SMOKE: an independent agent, on the REAL tree, introduces (a) an unmarked edit inside a genuinely-vendored 4M CLI path (or exercises the NO-OP-until-vendored guard per the FORK) and (b) a real off-registry CLI subcommand, then runs the auditor cross-lang/freeze pass and confirms both BLOCK — the real checker over real surfaces, not a fixture (R10).

## REDTEAM
REDTEAM: attack that FAILED — "let ELAI edits scatter untracked through a vendored CLI so the next upstream re-vendor silently clobbers them (the git-reset-x8 class of loss, one level up)": FAILS because UNIT A's marker + quarantine check BLOCKS any unmarked ELAI-side edit inside the vendored tree, keeping the diff explicit and re-vendorable. Attack that SUCCEEDED and its patch — "add a slick new CLI command straight to the surface, bypassing the registry, so clients drift": patched by UNIT B's command-surface symmetry drift check that treats the single registry as authoritative and BLOCKS an off-registry command; a second registry is refused by the ABORT.

## Grounding
- spec-eval-elai-vs-hermes-kilo-2026-07-10.md ADDENDUM §"Worth closing" (U1 fork-hygiene on vendored CLIs, U2 one-registry) + the source-grounded KILOCODE-SPEC-OVERVIEW (HEAD 1426dca) + HERMES-SPEC-OVERVIEW (HEAD 111544d), both cloned+pinned 2026-07-10.
- 4M vendored RuntimeAdapter, cross_lang_diff.py, ts-rs bindings, J MCP control plane, N-2/A-1 one-source — confirmed at authoring, executor re-greps (W-7).

## COMPOSES-WITH
4M vendored RuntimeAdapter CLIs, cross_lang_diff.py drift gate, ts-rs bindings, J MCP control plane, N-2 corpus_composition_check / A-1 one-source. Re-implements NONE (GD-1). Both units are the spec-eval addendum's "worth closing" steals; sequenced AFTER the proof spine (PS-2/PS-4 first).

End of Q-9.

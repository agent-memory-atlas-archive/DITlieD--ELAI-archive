# CODEX-DEMO-1 — `elai memory list --count-only` (codex-executor demonstration slice)

PROOF_SPINE: none (demo/distribution class — a K-27 codex-executor demonstration the owner explicitly
requested 2026-07-11 to SEE codex build+gate+land end to end; not a spine feeder, user-acked).

STATUS: authored 2026-07-11 (K-27 demo). Activation-class, executable-blind, seam VERIFIED against the
live tree. Small additive CLI flag on an existing, well-understood command.

## THESIS
Add a `--count-only` boolean flag to the existing `elai memory list` subcommand. When set, the command
prints ONLY the summary count line and SKIPS the per-row dump (both the human and the JSON row list).
This is a real scripting QoL flag (get the row count without dumping every fact). It is a pure vertical
slice: the flag (producer) is consumed by the existing `MemorySubcommandKind::List` handler (consumer),
which already exists and compiles.

## GROUNDED SEAM (verified 2026-07-11, do not re-derive)
- Parser: crates/cli/src/args_memory.rs `parse_memory_subcommand_args` already parses `list` and a set
  of flags into locals (`let mut limit: Option<usize> = None; let mut json = false; ...`). Add a
  `let mut count_only = false;` local + a `--count-only` arm in the flag loop, and thread it into the
  `CliAction`/`MemorySubcommandKind::List` payload the same way `json` is threaded.
- Variant: crates/cli/src/args.rs — the memory `List` subcommand payload carries `json` (and the shared
  memory flags). Add a `count_only: bool` field alongside `json` on the List carrier (mirror `json`
  exactly: same derive, same defaulting).
- Handler (THE PRODUCTION HOOK): crates/cli/src/memory_cmd.rs:144 `MemorySubcommandKind::List => { ... }`.
  The human branch ends with `for f in &rows { print_fact_summary(f); }` and the JSON branch builds a
  `facts` vec. When `count_only` is true: human branch prints the `store:` + `N row(s) (...)` lines but
  SKIPS the `for f in &rows` loop; JSON branch emits `{store_path, count}` WITHOUT the `facts` array
  (or with an empty `facts`). The count numbers themselves are unchanged.

## MOVE 1 — implement the flag (codex EXECUTOR, --lane codex)
Actions: add `count_only` to the List variant (args.rs), parse `--count-only` (args_memory.rs), and
guard the row-dump in the handler (memory_cmd.rs). Do NOT change the count math, the store query, the
scope/limit behavior, or any other subcommand. Keep the default (flag absent) output BYTE-IDENTICAL to
today.
EXPECTED_OBSERVATION: `cargo check -p elai-cli` is clean; `elai memory list` (no flag) prints rows as
before; `elai memory list --count-only` prints the `store:` + count line(s) and NO per-row lines; and
`elai memory list --count-only --json` emits an object with `count` but no populated `facts` array.
ON_FAIL: a compile error in args.rs derive (the List variant derives Eq/PartialEq — `count_only: bool`
is Eq-clean, so this should not happen; if it does, match the exact derives already on the variant) ->
mirror the `json: bool` field EXACTLY, it already compiles under the same derive. A parser arm typo ->
copy the `--json`/`json = true` arm shape.
FORKS: none.
RECON_NEEDED: none — the three edit sites (args.rs List variant, args_memory.rs parser, memory_cmd.rs:144
handler) are grounded above; `json` is the exact structural precedent to mirror at all three.
ABORT: if adding the field forces a change to an unrelated subcommand's payload or a shared enum used
outside memory (it does not — the List payload is memory-local), STOP + handoff rather than widen scope.
VERIFY (off-agent): opus_land.py runs `cargo check -p elai-cli` (compile gate) + the frozen checkers
(production_caller_check finds no new DARK pub fn — this adds a field + a match arm, not a new pub fn;
maintainability check clean) over the changeset. INTEGRATION_SMOKE below is the behavioral proof.
PRODUCTION_HOOK: crates/cli/src/memory_cmd.rs:144 (the `MemorySubcommandKind::List` handler) consumes
the new `count_only` field in the SAME changeset — producer (the flag) + consumer (the handler guard) +
the smoke are one unit.
WIRING: vertical slice — flag (args.rs + args_memory.rs) + handler guard (memory_cmd.rs) + the smoke,
all in one changeset. No feature flag (this is a plain CLI flag, always available, no ELAI_*_ENABLE gate).

## INTEGRATION_SMOKE (R10 / H-11)
After MOVE 1 lands (or in the audit worktree): drive the REAL `elai` CLI entrypoint on a store that has
>=2 facts: (a) `elai memory list` prints N row lines; (b) `elai memory list --count-only` prints the
count line and ZERO row lines; (c) `elai memory list --count-only --json | jq` shows `count == N` and an
absent/empty `facts` array; (d) the no-flag output is byte-identical to before the change (diff the
human output against a pre-change capture). A toy `--help`-only check does not count.

## REDTEAM (attack the plan before build)
Attack 1: "count_only breaks the default output." -> refuted: the guard is `if !count_only` around the
row loop ONLY; the flag defaults false, so the absent-flag path is unchanged (smoke (d) proves it).
Attack 2: "the List variant derives Eq and a new field breaks it." -> refuted: `count_only: bool` is
Eq/Hash-clean and `json: bool` already lives on the variant under the same derive.
Attack 3: "codex invents a new pub fn (wire-dark)." -> refuted: the change adds a struct FIELD + a match
GUARD, no new pub fn; production_caller_check has nothing to flag.
Attack 4 (SURVIVES as a watch item, not a blocker): the JSON `count_only` shape (omit `facts` vs empty
`facts`) is a minor contract choice; the smoke pins `count == N`, and either shape is acceptable for a
demo. Not a build blocker.

## ACCEPTANCE
`cargo check -p elai-cli` clean; the four INTEGRATION_SMOKE observations hold; frozen checkers pass; the
change is confined to the three named files; default output unchanged.

D-8 — Cryptographic Archive + `elai archive verify` CLI — decisions

Session: completion-drive close (s-25120 follow-on)
Plan: F:\.ELAI_workflow\docs\thefinalglance\D-plan\D-8_crypto_archive_verify.md
Prior status: PARTIAL (7/15 implemented, core CLI + walker shipped under
crates/cli/src/archive_verify.rs; SSB + CI + contract test + perf test +
docs were missing).

DECISION-1 — SCHEMA bump 114 -> 115 (plan's 53 -> 54 is stale).
The plan was written when the SCHEMA was 53. Between then and this close,
concurrent piggybacks (D-1..D-7 closure, C-38..C-42, B-3..B-7, E-5..E-51)
brought SCHEMA to 114. The atomic D-8 bump is therefore 114 -> 115. The
plan's "53 -> 54" wording is preserved in the plan text only as historical
context; the ACTUAL SCHEMA bump landed in this commit is 114 -> 115. A
sibling D-7 closure session ratified a second pair of variants
(ScopeFreezeRatified + ScopeFreezeViolated) that landed in the SAME bump
window (115); the action_type_lock golden file + TOTAL_VARIANTS const
were merged in lockstep with both sessions' additions (TOTAL_VARIANTS
331 from this session + 2 from the sibling = 333 final).

DECISION-2 — blake3 only (sha256 / blake2 BANNED per anti-stale-training).
The plan's contextual mention of "blake3 ~5GB/s on AVX2 vs ~1GB/s sha256"
remains as informational rationale. blake3 is already the canonical hash
primitive used by D-1's `compute_content_hash`; the D-8 walker reuses that
function verbatim and does NOT introduce any new hash dependency. No
sha256, sha2, or blake2 imports were added.

DECISION-3 — Typed `Vec<ArchiveVerifyError>` in SSB payload (NOT Vec<String>).
The plan's wire shape called for `errors: Vec<String>`. We landed
`errors: Vec<ArchiveVerifyError>` instead. Reasoning:
  * dashboards need to discriminate on the failure mode tag (HashMismatch
    / BrokenParent / DuplicateHash / MalformedJson / Io) without parsing
    freeform strings. A typed enum makes this a `match`, not a
    regex / string-prefix check.
  * the entry_index field is structural data — parsing it out of an
    English string is brittle and locale-sensitive.
  * the wire format is rmp-serde (msgpack); a typed enum serializes to
    the same bytes a tagged-union string would, with no size penalty.
ArchiveVerifyError lives in `elai_orchestrator::archive_chain` (NOT the
CLI crate) so the SSB payload type can reference it without a circular
dep. The CLI re-exports it.

DECISION-4 — `ArchiveChain::load_lenient` is additive; existing `load`
short-circuits unchanged.
GAP-2 required parsing-error tolerance for the verify driver (otherwise
a single MalformedJson line aborts the walk and hides downstream
HashMismatch / BrokenParent / DuplicateHash findings). Two options:
  (A) change `load` to be lenient — REJECTED. Existing production
      callers (`build_registry`, `pre_promotion_chain_gate`, BAI
      processor) depend on `load` returning Err on the first
      structural problem; making it lenient silently breaks those
      gates.
  (B) add `load_lenient` alongside — ACCEPTED. `verify_archive` calls
      the new lenient variant; everyone else continues to use `load`.
The lenient loader returns `Vec<ArchiveVerifyError>` (already the right
type — MalformedJson is one of its variants) so the verify driver can
prepend the parse failures to the walker's errors list without an
intermediate conversion.

DECISION-5 — arxiv:2507.05619 + DGM §5 are SUPPORTING DOCUMENTATION, not
implementation-specifying papers (PLAN_PAPER_CONFLATION guard).
The plan cites both papers as motivation for the integrity / lobotomy-
attack rationale. Neither paper specifies:
  * the SSB wire layout (action_type / payload / row cardinality)
  * the entry_index semantics (0-based, line-aligned, etc.)
  * the typed ArchiveVerifyError variant catalog
  * the 5-second perf budget for 10K entries
  * the GitHub Actions workflow shape
  * the `--fail-fast` / `--json` CLI flag matrix
All of those are ELAI design choices. The decision doc tags both
citations as "supporting documentation for archive-integrity rationale"
to prevent future audit-time PLAN_PAPER_CONFLATION accusations (the same
guard B-40 / B-41 / B-42 / D-7 closure sessions have been recording).
Re-verification status: arxiv:2507.05619 was confirmed real by the D-4
ingestion (workflow/papers/reward-hacking-detection-2507.05619-interface.md);
DGM §5 was confirmed real by the D-1 ingestion
(workflow/papers/dgm-interface.md).

DECISION-6 — CLI subcommand stays on `CliAction::ArchiveVerify` (already
shipped); no `SlashCommand::ArchiveVerify` forwarder added.
The plan's task 2 called for a `SlashCommand::ArchiveVerify` variant in
`commands.rs`. The actual shipped path is via `CliAction::ArchiveVerify`
in `args.rs` (top-level CLI parse) -> `main.rs:448` dispatch. Both paths
reach `archive_verify::run` and emit the same SSB row; the user-visible
syntax `elai archive verify` is identical. Adding a redundant
SlashCommand entry would duplicate the wiring for no observable gain.
The plan's "SlashCommand" naming was simply incorrect about which enum
the CLI dispatcher uses for top-level subcommands. SPEC_DRIFT, not
NOT_WIRED.

Implementation summary (closure files):
  - crates/orchestrator/src/archive_chain.rs (+ ArchiveVerifyError enum,
    + load_lenient, + verify_archive (canonical walker) + ArchiveVerifyReport)
  - crates/orchestrator/src/ssb.rs (+ SCHEMA 114->115 cascade, +1 variant
    `ArchiveVerifyResult`, + ArchiveVerifyResultPayload, all 9 cascade
    points)
  - crates/orchestrator/tests/action_type_lock.rs (+ TOTAL_VARIANTS +1,
    + ALL_VARIANTS row, + exhaustive_match arm — merged with sibling
    ScopeFreeze session so final count is 333)
  - crates/orchestrator/tests/action_type_lock.golden (+ archive_verify_result
    line at ordinal 0014, all subsequent ordinals shifted)
  - crates/cli/src/archive_verify.rs (thinned to a re-export wrapper +
    SSB emission in `run`)
  - crates/contracts/tests/archive_verify_contract.rs (5-scenario tamper
    matrix + fail_fast bonus test)
  - crates/orchestrator/tests/archive_verify_perf.rs (#[ignore]'d 10K-entry
    perf gate, 5s wall-clock budget)
  - .github/workflows/archive-verify.yml (GHA on push/PR, ubuntu-latest,
    skip-if-absent fixture)
  - .elai_cc/README.md (Tamper-Recovery section)
  - workflow/reasoning/D-8-decisions.md (this file)

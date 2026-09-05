D-1 decisions — ReasoningTemplate typed schema + Blake3 content-hash + archive lifecycle

Session: orchestrate run under s-62376
Phase status: COMPLETE (lib build green, 27 unit tests pass, 13 contract tests pass)

DECISION-1: blake3 over sha256 / blake2 for content-hash
Rationale: CLAUDE.md anti-stale HARD RULE pins blake3 as the project's
default hash function. Project workspace already declares blake3 (crates/
orchestrator/Cargo.toml L64, workspace dep declared in root Cargo.toml).
arxiv:2507.05619 (T1) recommends blake3 over sha256 for integrity checksums
on append-only artifacts (faster, parallelizable, same 128-bit collision
resistance). Matches C-27 DeterminismConfig hash function. Reasoning_template
keeps Blake3Hash as a newtype to prevent type-level confusion with other
32-byte digests (e.g., session ids, content addresses).

DECISION-2: JSON Lines over SQLite for ArchiveChain storage
Rationale:
  - Append-only is physically enforced by the format (no UPDATE/DELETE wire
    surface; the parser ignores incomplete trailing lines naturally).
  - Line-diffable in git review — a self-mod proposal landing a new template
    shows up as exactly one `+` line in `git diff`, making auditor review
    trivial.
  - DGM §5 archive practice matches (the paper's reference implementation
    uses one-row-per-line append storage).
  - Removes a transitive dep on a sqlite migration story for the archive
    file (sqlite is already loaded for SSB, but per-archive lifecycle
    schema migrations would add complexity that JSONL avoids).
Tradeoff: sqlite would give us indexed lookup; we compensate by holding
TemplateRegistry in memory (FxHashMap keyed by TemplateId) after load. For
the expected archive size (<10K templates over multi-year project life),
load-into-memory is O(MB) and acceptable.

DECISION-3: archive_status = Frozen for migrated entries (not Promoted)
Rationale: per companion §4, the initial migrated template set should be
treated as L3-frozen until D-5 BAI is wired to score them. Marking them
Promoted would imply they passed staged eval (10/50/200 BAI), which they
have NOT. Frozen explicitly says "this is an immutable seed set — do not
mutate, do not score". When D-5 BAI lands, NEW Candidate entries will go
through the Promoted track; the seed Frozen set stays immutable per the
spec.

DECISION-4: SCHEMA_VERSION coordination with parallel E-plan sessions
Observed: the plan called for bump 47 -> 48, but the actual schema started
at 63 (post-C-32). My initial bump went 63 -> 64; a concurrent E-5 session
re-bumped 64 -> 65 (memory_facts variants). Final SCHEMA_VERSION_PUBLIC is
65. The D-1 contract test floor is `>= 64` so this is non-breaking.

DECISION-5: native async fn in trait surface (no async-trait crate)
Rationale: per CLAUDE.md anti-stale-rust rule, native async fn in traits
shipped on stable Rust 1.75 (Dec 2023). archive_chain.rs uses native async
fn (`pub async fn append_and_emit_ssb`). No async-trait crate added.

DECISION-6: fs2 advisory lock deferred to D-5/D-8
Rationale: the plan flagged fs2 as optional and required user confirmation
before adding. D-1 ships with the single-writer assumption (migration tool +
future D-5 BAI processor are the only writers, both single-process). When
D-5 BAI lands and the dashboard's read path may overlap with a writer,
fs2 advisory lock can be added in that session under explicit confirmation.

Scope NOT shipped in D-1 (deferred per plan Out-of-scope):
  - D-2 PPM sidecar wiring (consumer of VerifierKind::Prm)
  - D-5 BAI Thompson promotion (Candidate -> Promoted transition producer)
  - D-7 cross-domain canary (StagedEvalResult.cross_domain_deltas populator)
  - D-8 `elai archive verify` CLI (D-1 provides verify_chain method only)
  - migration tool template_migrate.rs (sits in cli/, requires SlashCommand
    enum touch + CLI dispatch; reserved for a follow-up session)
  - L3-frozen LockedKeysAuditor enforcement (D-9)

Files touched:
  ADD crates/orchestrator/src/reasoning_template.rs (16 unit tests)
  ADD crates/orchestrator/src/archive_chain.rs (10 unit tests including
      observable_effect smoke)
  MODIFY crates/orchestrator/src/lib.rs (pub mod registration)
  MODIFY crates/orchestrator/src/ssb.rs (4 new SsbActionType variants +
      payloads + CHECK constraint + from_str + test_payload)
  ADD crates/contracts/tests/reasoning_template_schema_contract.rs
      (13 contract tests)
  ADD crates/dashboard-v2/src-tauri/src/commands_d1.rs (2 Tauri commands)
  MODIFY crates/dashboard-v2/src-tauri/src/lib.rs (module + invoke_handler
      registration)
  ADD crates/dashboard-v2/src/lib/drawers/MutationArchivePanel.svelte
  MODIFY crates/dashboard-v2/src/lib/drawers/DrawerHost.svelte (drawer route)
  MODIFY crates/dashboard-v2/src/lib/api/types.ts (DrawerKey variant)
  MODIFY .elai_cc/elai_cc.toml (commented [reasoning.templates] section)

Test results (lib only, scoped per CLAUDE.md):
  cargo test -p elai-orchestrator --lib: 27/27 pass on new tests
    (16 reasoning_template + 10 archive_chain + 1 ssb::test_typed_payloads)
  cargo test -p elai_cc_contracts --test reasoning_template_schema_contract:
    13/13 pass
  cargo build -p elai-orchestrator -p elai-ipc-types --all-features: exit 0
  cargo build -p elai-dashboard-v2: exit 0

No .unwrap()/todo!()/unimplemented!()/unreachable!() in new non-test code.

# D-1: ReasoningTemplate typed schema + Blake3 content-hash + archive lifecycle

Objective: replace string-literal in-prompt reasoning templates with a typed
ReasoningTemplate struct carrying domain, pattern_kind, eligibility predicate,
verifier requirement, blake3 content hash, parent_id, and archive_status.
Bootstrap append-only ArchiveEntry chain. This is the load-bearing substrate
for every subsequent D-N: the typed mutation surface that replaces DGM's
"any Python code" unbounded blob (arxiv:2505.22954, T1) with the bounded
closed-schema capacity realization required by arxiv:2510.04399 (T1).
Estimated effort: 1-2 sessions.
Pre-requisites: C-15 (ClockProvider + SeededClock + entropy quarantine stable;
  blake3 used for prefix-cache keys per C-15 spec; ReasoningTemplate hash uses
  same blake3 crate, no sha256 fallback per CLAUDE.md anti-stale rule), C-21
  (MistakeBook + rules.db stable; archive consumes BLOCK events from MistakeBook
  to inform RetirementReason). Spec-grounder MUST verify both DONE before
  unblocking EXECUTING.
Parallel-safe with: D-19 (PRMBench benchmark; reads PPM output, does not touch
  archive); D-25 (tamper test suite; reads archive verify CLI, does not write
  templates). NOT parallel-safe with D-2..D-9 (all consume ReasoningTemplate
  or ArchiveEntry typed records).

## Context

`docs/specs/guide/reasoning-improvement-grounded-2026-05-19.md` §3.1 (lines
200-226) verified MISSING in current codebase: no `ReasoningTemplate` struct
exists; in-prompt reasoning patterns live as string literals across
.elai_cc/crates/prompts/templates/*.toml (~25 files) and inline in
agent definition files .claude/agents/*.md. Mutation surface is currently
ad-hoc (whichever string the prompt-engineer agent decides to swap), with no
content hash + no archive lineage + no archive_status lifecycle.

DGM paper (workflow/papers/dgm-interface.md + .claude/papers/2505.22954v3.pdf
+ .claude/papers/2505.22954v3_full_text.txt) treats agent code as
undifferentiated string blob. §H demonstrates this loses the bounded-capacity
guarantee from arxiv:2510.04399 (T1) — the unbounded mutation surface
permits the DGM hallucination-marker-removal incident (§H, the canonical
reward-hacking example in production).

ELAI's Phase 4H spec carving (L1A data tables only, L2 closed-enum config keys,
L3 human-only structural) maps cleanly to this gap: extend L1A from "data tables
only" to "data tables + typed reasoning templates", with the typed schema
serving as the bounded-capacity realization the theorem requires.

Partial foundation present and usable:
- Blake3 hash function available via blake3 crate (verify in Cargo.toml; present
  since B-1 plan rejected blake2 per CLAUDE.md HARD RULE).
- SsbActionType enum (.elai_cc/crates/orchestrator/src/ssb.rs) with SCHEMA_VERSION
  47 after C-27 — D-1 bumps to 48.
- MistakeBook trait (.elai_cc/crates/orchestrator/src/mistake_book.rs) provides
  RetirementReason input source.
- runtime_config.rs loader reads elai_cc.toml — extend with [reasoning.templates]
  section pointer.

## Reads

- F:\.ELAI_workflow\docs\specs\guide\reasoning-improvement-grounded-2026-05-19.md
  (§3.1 ReasoningTemplate definition, §4 L3-frozen list, §2.1 Voyager
  analogue — lines 200-226 + 347-362 + 156-162)
- F:\.ELAI_workflow\workflow\papers\dgm-interface.md
  (DGM mutation-surface description; §H reward-hacking case study)
- F:\.ELAI_workflow\.claude\papers\2505.22954v3-DGM-extraction.md
  (full DGM extraction, T1 reference for the unbounded-mutation problem)
- .elai_cc/crates/orchestrator/src/ssb.rs (SsbActionType enum:961; SCHEMA_VERSION:
  insertion point for bump 47→48; SsbPayload arms)
- .elai_cc/crates/orchestrator/src/mistake_book.rs (MistakeBook trait;
  RetirementReason candidate enum source)
- .elai_cc/crates/core/src/clock.rs (SeededClock:76 — UTC source for created_at)
- .elai_cc/crates/core/Cargo.toml (verify blake3 already present; verify uuid
  already present)
- .elai_cc/crates/prompts/templates/*.toml (current string-literal templates;
  read 2-3 representative files to inform schema fields)
- .elai_cc/elai_cc.toml (insertion point for [reasoning.templates] section)
- .elai_cc/crates/orchestrator/tests/ (contract test pattern; reference existing
  *_contract.rs files for structure)

## Paper-ingestor pre-step

Spawn paper-ingestor with mode=paper for ALL of:
- arxiv:2402.03620 SELF-DISCOVER (atomic reasoning modules as mutation surface)
- arxiv:2510.04399 Statistical Limits of Self-Improving Agents (bounded-capacity
  theorem; published Oct 2025)

Verify outputs land at workflow/papers/self-discover-2402.03620-interface.md and
workflow/papers/statistical-limits-2510.04399-interface.md before unblocking
TESTING_FIRST. spec-grounder will fail audit if either is missing.

## Tasks

1. **Schema definition** in new file
   .elai_cc/crates/orchestrator/src/reasoning_template.rs:
   - `struct ReasoningTemplate { id: TemplateId, domain: Domain, pattern_kind:
     PatternKind, template_body: BoundedString<1024>, eligibility_predicate:
     Predicate, verifier_required: VerifierKind, content_hash: Blake3Hash,
     created_at: DateTime<Utc>, parent_id: Option<TemplateId>, archive_status:
     ArchiveStatus }`
   - `enum Domain { Code, Math, Prose, Mixed }` (closed enum; L3-locked per
     companion §4)
   - `enum PatternKind { Decomposition, Reflection, Verification, Synthesis }`
     (closed enum; L3-locked)
   - `enum VerifierKind { Compile, Test, Prm, Refuse }` (closed enum; L3-locked
     per companion §4 — agent cannot self-weaken from Prm to Refuse)
   - `enum ArchiveStatus { Candidate, Promoted, Retired, Frozen }`
   - `enum RetirementReason { Degradation, Superseded, Banned, MistakeBookTrigger }`
   - `struct TemplateId(Uuid)` newtype; `struct Blake3Hash([u8; 32])` newtype
   - `struct BoundedString<const N: usize>(String)` with Try-From bound; reject
     strings >N at construction time, no panic, return BoundedStringError
   - `enum Predicate { Always, MatchDomain(Domain), MinConfidence(f32),
     AndAll(Vec<Predicate>), OrAny(Vec<Predicate>) }` (closed enum; recursion
     bounded by Vec length)
   Derive Debug/Clone/Serialize/Deserialize on all types. Use thiserror for
   error enum (TemplateError { BoundedStringExceeded, HashMismatch, ParentNotFound,
   StatusTransitionInvalid }). NO unwrap; NO expect.

2. **Content-hash computation** in reasoning_template.rs:
   - `fn compute_content_hash(body: &str, parent_hash: Option<&Blake3Hash>) ->
     Blake3Hash`: blake3::hash(body.as_bytes() || parent_hash.unwrap_or(&[0u8;
     32])). Deterministic; same inputs → same hash.
   - `fn verify_hash(template: &ReasoningTemplate) -> Result<(), TemplateError>`:
     recomputes hash; returns HashMismatch on divergence.
   - Unit test: identical (body, parent) → identical hash. Different body OR
     different parent → different hash. Hash length always 32 bytes.

3. **ArchiveEntry + ArchiveChain** in new file
   .elai_cc/crates/orchestrator/src/archive_chain.rs:
   - `struct ArchiveEntry { self_hash: Blake3Hash, parent_hash: Option<Blake3Hash>,
     template: ReasoningTemplate, staged_eval_result: Option<StagedEvalResult>,
     promoted_at: Option<DateTime<Utc>>, retired_at: Option<DateTime<Utc>>,
     retirement_reason: Option<RetirementReason> }`
   - `struct StagedEvalResult { stage_10_pass: u32, stage_10_total: u32,
     stage_50_pass: u32, stage_50_total: u32, stage_200_pass: u32,
     stage_200_total: u32, cross_domain_deltas: Vec<(Domain, f32)> }` (placeholder
     populated by D-5 BAI + D-7 canary)
   - `struct ArchiveChain { storage_path: PathBuf, entries: Vec<ArchiveEntry> }`
   - `impl ArchiveChain`: load(path), append(entry) — append-only, NEVER
     delete; verify_chain() walks parent_hash links and re-computes self_hash.
   - Storage format: JSON Lines (.elai_cc/archive/reasoning-templates.jsonl);
     each line a serialized ArchiveEntry; mode O_APPEND only.
   - Lock: use fs2 crate for advisory lock during append (verify fs2 in
     Cargo.toml; if absent, this is the only optional dep; check with user
     before adding per CLAUDE.md no-new-deps-without-confirmation).

4. **SCHEMA_VERSION bump + SSB variants** in ssb.rs.
   Bump SCHEMA_VERSION 47→48. Add four SsbActionType variants:
   - `ArchiveEntryAdded { template_id: TemplateId, content_hash: String,
     parent_hash: Option<String>, domain: String, pattern_kind: String,
     archive_status: String }`
   - `ArchiveEntryRetired { template_id: TemplateId, retired_at: i64,
     retirement_reason: String }`
   - `TemplatePromoted { template_id: TemplateId, stage: String, pass_rate: f64 }`
   - `TemplateCandidateRejected { template_id: TemplateId, rejection_reason:
     String }`
   Add SsbPayload arms + SQL serialization for each. Round-trip test required.

5. **Migration tool** in .elai_cc/crates/cli/src/template_migrate.rs:
   - CLI subcommand `elai template migrate`: scans
     .elai_cc/crates/prompts/templates/*.toml + agent_definitions/*.md; extracts
     unique reasoning patterns; for each, computes blake3 hash + creates
     ReasoningTemplate with archive_status=Frozen (so initial set is immutable);
     writes to reasoning-templates.jsonl.
   - First run idempotent — re-running on same input produces same hashes,
     skips already-present entries.
   - Output report: count(extracted), count(skipped_duplicate), count(skipped_too_long).

6. **Template registry runtime loader** in reasoning_template.rs:
   - `struct TemplateRegistry { templates: HashMap<TemplateId, ReasoningTemplate> }`
     (use FxHashMap per CLAUDE.md anti-stale-rust rule, NOT std::HashMap; verify
     fxhash in Cargo.toml — present after C-15 HashMap-ban migration).
   - `fn load(archive_chain: &ArchiveChain) -> Self`: filter entries to
     archive_status in {Promoted, Frozen}; build registry.
   - `fn select_template(&self, domain: Domain, pattern_kind: PatternKind,
     ctx: &OrchContext) -> Option<&ReasoningTemplate>`: filter by domain +
     pattern_kind + eligibility_predicate.eval(ctx); return first match.
   - Used by task_runner.rs replacement of in-prompt string literals (deferred
     to D-2 to keep this session scoped).

7. **elai_cc.toml [reasoning.templates] section** in .elai_cc/elai_cc.toml:
   - Add commented section: `# [reasoning.templates]\n# archive_path = ".elai_cc/
     archive/reasoning-templates.jsonl"\n# bounded_string_max = 1024`
   - runtime_config.rs reads section; defaults if absent (archive_path =
     ".elai_cc/archive/reasoning-templates.jsonl", bounded_string_max = 1024).

8. **Unit tests** (#[cfg(test)] in reasoning_template.rs + archive_chain.rs):
   - BoundedString<1024> accepts 1024 chars, rejects 1025 (no panic, returns
     Err).
   - compute_content_hash deterministic across calls.
   - verify_hash detects tampering (mutate body, recompute → HashMismatch).
   - ArchiveChain.append writes JSONL line; subsequent load reads it back
     identically.
   - ArchiveChain.verify_chain detects broken parent_hash link (synthetic
     tamper).
   - Predicate::AndAll([Always, Always]).eval(ctx) == true; OrAny([Never, Always])
     == true (note: introduce trivially-false Never variant if needed for tests).
   - StatusTransition: Candidate→Promoted valid; Promoted→Retired valid;
     Frozen→anything invalid; Retired→anything invalid (immutable once retired).
   - SSB variant round-trip: serialize ArchiveEntryAdded → deserialize → equal.

9. **Contract test** crates/contracts/tests/reasoning_template_schema_contract.rs:
   - SCHEMA_VERSION_PUBLIC == 48.
   - All 4 new SsbActionType variants present + SQL string round-trip.
   - ReasoningTemplate { Domain, PatternKind, VerifierKind, ArchiveStatus,
     RetirementReason } closed enums; explicit allow-list test that no variant
     was added beyond the spec (compile-time check that match arms exhaustive).
   - ArchiveChain.append + verify_chain end-to-end on tmpdir.

10. **Migration smoke run** (manual step documented in TASKS, not automated):
    - Run `elai template migrate` on current repo.
    - Report counts (extracted, skipped_duplicate, skipped_too_long).
    - Inspect first 3 entries in reasoning-templates.jsonl — verify well-formed
      JSON, hashes 64 hex chars, archive_status=Frozen for all.

11. **Decision doc** workflow/reasoning/D-1-decisions.md (3 decisions):
    - (a) Why blake3, not sha256: CLAUDE.md HARD RULE + arxiv:2507.05619 (T1)
      integrity-checksum recommendation; matches C-27 DeterminismConfig.
    - (b) Why JSON Lines, not SQLite: append-only physical immutability + line-
      diffable in git review; DGM §5 archive practice matches.
    - (c) Why archive_status=Frozen for migrated templates, not Promoted: per
      companion §4 — initial set should not be mutable until D-5 BAI is wired
      to score them; "frozen" makes this immutability explicit.

12. **Memorize the schema** via memory-writer agent (deferred to MEMORIZING
    state): emit project_d1_reasoning_template.md with the typed schema +
    archive lifecycle + SSB variant list. Required for D-2..D-9 to consume.

## WIRING_CONTRACT

produces_symbols:
  - ReasoningTemplate, TemplateId, Blake3Hash, BoundedString<N>
  - Domain enum, PatternKind enum, VerifierKind enum, ArchiveStatus enum,
    RetirementReason enum, Predicate enum
  - TemplateError + thiserror impl
  - ArchiveEntry, ArchiveChain, StagedEvalResult
  - TemplateRegistry + select_template
  - compute_content_hash + verify_hash + verify_chain
  - SsbActionType::ArchiveEntryAdded + ArchiveEntryRetired + TemplatePromoted +
    TemplateCandidateRejected + payloads
  - CLI `elai template migrate` (SlashCommand::TemplateMigrate)
  - SCHEMA_VERSION bump 47→48
  - [reasoning.templates] section in elai_cc.toml + runtime_config loader
consumes_symbols:
  - blake3 crate (already in Cargo.toml per C-1 + C-27)
  - uuid crate (already in Cargo.toml per existing SSB payloads)
  - SeededClock (core/src/clock.rs:76) — created_at source via inject; never
    SystemTime::now per C-15 ClockProvider migration
  - FxHashMap (after C-15 HashMap-ban migration)
  - SsbActionType enum (orchestrator/src/ssb.rs) — extended here
  - MistakeBook trait (orchestrator/src/mistake_book.rs) — RetirementReason
    feed planned for D-4 tripwire, not wired in this session
  - runtime_config.rs elai_cc.toml loader
  - SlashCommand enum (cli/src/commands.rs) — gains TemplateMigrate variant
reachable_from:
  - `elai template migrate` CLI subcommand
  - TemplateRegistry::load via runtime_config.rs at startup
  - Future D-N sessions read from ArchiveChain at sled-equivalent latency
    (JSON Lines + in-memory FxHashMap after load)
stub_ban:
  - BoundedString<N> MUST NOT panic on overflow; return Err
  - ArchiveChain MUST NOT have a delete method; append-only by construction
  - compute_content_hash MUST use blake3, never sha256 or blake2
  - verify_hash MUST recompute, not compare cached values (defends against
    in-memory mutation)
  - StatusTransition MUST reject Frozen→anything and Retired→anything; no
    "force-override" flag
  - migrate command MUST be idempotent (re-run produces zero new entries on
    unchanged input)
  - Predicate eval MUST NOT call out to the LLM (closed-form evaluation only;
    per companion §4 L3-frozen)
observable_effect:
  trigger: `elai template migrate` then read reasoning-templates.jsonl
  expect_ssb: ≥N ArchiveEntryAdded rows where N == reported migrate count
              (archive_status=Frozen for all migrated); zero TemplatePromoted
              rows (no BAI yet); zero TemplateCandidateRejected rows
  expect_artifact: .elai_cc/archive/reasoning-templates.jsonl exists, each line
                   valid JSON, ArchiveChain::verify_chain returns Ok on whole
                   file

## DETERMINISTIC_CHECK

```bash
# 1. SCHEMA_VERSION bumped to 48
rg "SCHEMA_VERSION: i64 = 48" .elai_cc/crates/orchestrator/src/ssb.rs

# 2. New SSB variants present (expect >= 4 each: enum + as_sql_str + from_str + payload)
rg "ArchiveEntryAdded|ArchiveEntryRetired|TemplatePromoted|TemplateCandidateRejected" \
  .elai_cc/crates/orchestrator/src/ssb.rs | wc -l
# Expected: >= 16

# 3. ReasoningTemplate types
rg "struct ReasoningTemplate|struct ArchiveEntry|struct ArchiveChain|struct TemplateRegistry" \
  .elai_cc/crates/orchestrator/src/reasoning_template.rs \
  .elai_cc/crates/orchestrator/src/archive_chain.rs

# 4. Closed-enum domain + pattern + verifier
rg "enum Domain|enum PatternKind|enum VerifierKind|enum ArchiveStatus" \
  .elai_cc/crates/orchestrator/src/reasoning_template.rs

# 5. blake3 use (not sha256)
rg "blake3" .elai_cc/crates/orchestrator/src/reasoning_template.rs
rg "sha256|blake2" .elai_cc/crates/orchestrator/src/reasoning_template.rs
# Expected: blake3 >= 2 hits, sha256/blake2 = 0 hits

# 6. CLI subcommand
rg "TemplateMigrate|template.migrate" .elai_cc/crates/cli/src/commands.rs

# 7. elai_cc.toml section template
rg "reasoning.templates" .elai_cc/elai_cc.toml

# 8. Migration produces JSONL (after `elai template migrate` run)
test -f .elai_cc/archive/reasoning-templates.jsonl && head -1 .elai_cc/archive/reasoning-templates.jsonl | python -c "import sys,json; json.loads(sys.stdin.read())" && echo OK

# 9. Build + scoped tests
cargo check --workspace 2>&1 | tail -3
cargo test -p elai_orchestrator -- reasoning_template archive_chain 2>&1 | grep "test result"
cargo test -p elai_cc_contracts -- reasoning_template_schema 2>&1 | grep "test result"

# 10. No unwrap in new code
rg "\.unwrap\(\)" .elai_cc/crates/orchestrator/src/reasoning_template.rs \
  .elai_cc/crates/orchestrator/src/archive_chain.rs
# Expected: 0 hits (test code uses ? or assert_matches)

# 11. Paper-ingestor outputs present
test -f workflow/papers/self-discover-2402.03620-interface.md && \
  test -f workflow/papers/statistical-limits-2510.04399-interface.md && echo OK
```

## ACCEPTANCE

- [ ] SCHEMA_VERSION_PUBLIC == 48; old SSB DB with version 47 emits migration warn
- [ ] ArchiveEntryAdded, ArchiveEntryRetired, TemplatePromoted, TemplateCandidateRejected
      in SsbActionType + payloads serialise/deserialise (round-trip test)
- [ ] ReasoningTemplate, ArchiveEntry, ArchiveChain compile and unit tests pass
- [ ] BoundedString<1024> rejects 1025-char input via Err (no panic)
- [ ] compute_content_hash + verify_hash deterministic; tamper detection works
- [ ] ArchiveChain.verify_chain detects broken parent_hash links
- [ ] StatusTransition rejects Frozen→* and Retired→* (immutable terminal states)
- [ ] `elai template migrate` runs idempotently; second run extracts 0 new
- [ ] Migration smoke run on real repo: ≥N templates migrated where N matches
      `find .elai_cc -name "*.toml" -path "*/templates/*" | wc -l`
- [ ] cargo check --workspace: zero new errors
- [ ] cargo test -p elai_orchestrator -- reasoning_template archive_chain: all
      pass, ≥8 new test functions
- [ ] cargo test -p elai_cc_contracts -- reasoning_template_schema: all pass
- [ ] Cloud smoke test A: run `elai /task --mode cloud "explain how blake3 differs
      from sha256 in 3 bullets"`; verify ≥1 ArchiveEntryAdded row was already
      present from migration; verify TemplateRegistry::load reads it; verify
      task completes (template selection works without runtime errors)
- [ ] Cloud smoke test B: manually edit a template_body in reasoning-templates.jsonl;
      run any subsequent task; verify ArchiveChain::verify_chain returns
      HashMismatch error and orchestrator REFUSES to use the tampered template
- [ ] Paper-ingestor outputs at workflow/papers/self-discover-2402.03620-interface.md
      + workflow/papers/statistical-limits-2510.04399-interface.md (spec-grounder
      audited before EXECUTING)
- [ ] Decision doc workflow/reasoning/D-1-decisions.md (3 decisions logged)
- [ ] No unwrap/expect in new code; thiserror enums for all error returns
- [ ] memory-writer agent emits project_d1_reasoning_template.md (post-AUDITING)

## Files touched

- ADD: .elai_cc/crates/orchestrator/src/reasoning_template.rs
    (ReasoningTemplate, TemplateId, Blake3Hash, BoundedString, Domain, PatternKind,
     VerifierKind, ArchiveStatus, RetirementReason, Predicate, TemplateError,
     TemplateRegistry, compute_content_hash, verify_hash, select_template)
- ADD: .elai_cc/crates/orchestrator/src/archive_chain.rs
    (ArchiveEntry, ArchiveChain, StagedEvalResult, verify_chain)
- ADD: .elai_cc/crates/cli/src/template_migrate.rs
    (CLI dispatch for `elai template migrate`)
- MODIFY: .elai_cc/crates/orchestrator/src/ssb.rs
    (SCHEMA_VERSION 47→48; 4 new SsbActionType variants + payloads)
- MODIFY: .elai_cc/crates/cli/src/commands.rs
    (SlashCommand::TemplateMigrate variant + dispatch)
- MODIFY: .elai_cc/elai_cc.toml
    ([reasoning.templates] section commented template)
- MODIFY: .elai_cc/crates/orchestrator/src/lib.rs
    (pub mod reasoning_template; pub mod archive_chain)
- ADD: .elai_cc/crates/contracts/tests/reasoning_template_schema_contract.rs
- ADD: workflow/reasoning/D-1-decisions.md
- ADD: workflow/papers/self-discover-2402.03620-interface.md (via paper-ingestor)
- ADD: workflow/papers/statistical-limits-2510.04399-interface.md (via paper-ingestor)

## Out-of-scope

- D-2: PPM sidecar wiring (consumes ReasoningTemplate via VerifierKind::Prm but
  does not write templates)
- D-5: BAI Thompson promotion (consumes ArchiveEntry but does not extend schema)
- D-8: `elai archive verify` CLI (covers ArchiveChain.verify_chain through a
  user-facing subcommand; D-1 only provides the verify_chain method)
- Per-template performance metrics (D-7 cross-domain canary populates these in
  StagedEvalResult.cross_domain_deltas)
- L3-frozen access control enforcement (D-9 wires pre-commit hook and
  LockedKeysAuditor; D-1 only documents which fields are L3-frozen)

## Next

D-2 (PPM sidecar wiring) and D-8 (cryptographic archive verify CLI) can both
start in parallel after D-1. D-2 introduces the verifier substrate;
D-8 surfaces the verify_chain method as a user CLI.

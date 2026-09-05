# ReasoningTemplate Typed Schema + Archive Lifecycle (D-1)

- **ELAI ids:** D-1 / Phase 4H.IV / companion §3.1 / D-Wave-1 KEYSTONE
- **Spec status (as designed):** PLANNED (D-plan session file). Feature-map later tags the organ PARTIAL (`ELAI-FEATURE-MAP.md:635-638`); that is implementation-neighbor, not this architecture.
- **Crate / module (spec):** `crates/orchestrator/src/reasoning_template.rs`, `crates/orchestrator/src/archive_chain.rs`; CLI `crates/cli/src/template_migrate.rs`
- **Purpose of this file:** enough architecture to rebuild the typed mutation surface — `ReasoningTemplate`, Blake3 content-hash, append-only `ArchiveEntry` chain, closed enums, archive lifecycle. Not a status ledger.

## 1. What it is

The **ReasoningTemplate archive** is the typed, hash-chained, append-only store of reasoning patterns. A `ReasoningTemplate` carries `id`, `domain`, `pattern_kind`, `template_body`, `eligibility_predicate`, `verifier_required`, `content_hash`, `parent_id`, and `archive_status`. In-prompt string literals in `prompts/templates/*.toml` and agent markdown become records. `ArchiveChain` is JSON Lines, O_APPEND only, never delete. This is the bounded closed-schema mutation surface that DGM’s undifferentiated Python blob is not (`D-1_reasoning_template_schema.md:3-9`, `D-SUMMARY_INDEX.md:22-26`).

## 2. Why it exists

ELAI’s trust spine refuses an unbounded self-mod blob. DGM (arxiv:2505.22954) treats agent code as a string; §H is the hallucination-marker-removal reward-hack. The bounded-capacity theorem (arxiv:2510.04399) requires a closed schema. If this organ is missing, prompt-engineer agents swap template strings with no hash, no lineage, no lifecycle. If it is faked (prompt “please keep templates typed”), Frozen/Retired records mutate and the later BAI/tripwire stack has nothing to revert to (`D-1_reasoning_template_schema.md:22-42`, `D-SUMMARY_INDEX.md:20-26`).

## 3. How it works (runtime)

1. **Who calls it.** CLI `elai template migrate` bootstraps the JSONL. Orchestrator `TemplateRegistry::load` at startup. Later D-N (PPM, BAI, tripwires, canary, archive-verify) read `ArchiveChain`. `task_runner.rs` replacement of in-prompt literals is deferred to D-2 (`D-1_reasoning_template_schema.md:154-174`, `:240-254`).
2. **Input (migrate).** Scan `.elai_cc/crates/prompts/templates/*.toml` and `agent_definitions/*.md`. Extract unique reasoning patterns (`D-1_reasoning_template_schema.md:155-159`).
3. **Record construction (deterministic).** For each unique pattern: `BoundedString<1024>` Try-From on body (reject `>N`, no panic); `compute_content_hash(body, parent_hash)`; `archive_status=Frozen` for migrated set; `created_at` from `SeededClock`, never `SystemTime::now` (`D-1_reasoning_template_schema.md:105-116`, `:159`, `:242-243`).
4. **Hash (deterministic).** `blake3::hash(body.as_bytes() || parent_hash.unwrap_or(&[0u8; 32]))`. Same inputs → same 32-byte hash. `verify_hash` **recomputes**; it does not compare a cached field (`D-1_reasoning_template_schema.md:114-121`, `:258-260`).
5. **Append (deterministic).** `ArchiveChain::append` writes one JSONL line under advisory lock (`fs2` if present). No delete method. First migrate is idempotent: re-run on same input produces same hashes and skips already-present entries (`D-1_reasoning_template_schema.md:134-140`, `:160-161`, `:257`, `:263-264`).
6. **Registry load (deterministic).** `TemplateRegistry::load` keeps entries whose `archive_status` is `Promoted` or `Frozen` in an `FxHashMap<TemplateId, ReasoningTemplate>` (`D-1_reasoning_template_schema.md:165-169`, `:246`).
7. **Select (deterministic).** `select_template(domain, pattern_kind, ctx)` filters by domain + pattern_kind + `eligibility_predicate.eval(ctx)` and returns the **first** match. Predicate eval is closed-form; **no LLM call** (`D-1_reasoning_template_schema.md:170-172`, `:265-266`).
8. **Status transitions (deterministic).** `Candidate→Promoted` valid; `Promoted→Retired` valid; `Frozen→*` invalid; `Retired→*` invalid. No force-override flag (`D-1_reasoning_template_schema.md:193-194`, `:261-262`).
9. **Commit artifacts.** JSONL line + SSB `ArchiveEntryAdded` (migrated: `archive_status=Frozen`). Zero `TemplatePromoted` / `TemplateCandidateRejected` from migrate — BAI is D-5 (`D-1_reasoning_template_schema.md:267-274`).
10. **Abstain / fail-closed.** Tampered `template_body` → `verify_hash` / `verify_chain` → `HashMismatch`; orchestrator **REFUSES** the tampered template. Missing parent → `ParentNotFound`. Illegal status → `StatusTransitionInvalid`. Body `>1024` → `BoundedStringExceeded`. `SCHEMA_VERSION` 47 DB emits migration warn (`D-1_reasoning_template_schema.md:327-348`).

The model may **generate** a candidate `template_body` later (D-5 promotion path). It may **not** set `content_hash`, choose `archive_status`, evaluate `Predicate`, weaken `verifier_required`, or grade its own modification. Promotion that mutates the archive needs the D-3 consensus gate (≥2 verifiers); D-1 only stores `verifier_required` as an L3-locked field (`D-1_reasoning_template_schema.md:100-101`, `:388-389`; `D-SUMMARY_INDEX.md:34-38`).

## 4. Types, enums, objects

Copy from `D-1_reasoning_template_schema.md:89-132`. Do not invent fields.

```
ReasoningTemplate {
  id: TemplateId,
  domain: Domain,
  pattern_kind: PatternKind,
  template_body: BoundedString<1024>,
  eligibility_predicate: Predicate,
  verifier_required: VerifierKind,
  content_hash: Blake3Hash,
  created_at: DateTime<Utc>,
  parent_id: Option<TemplateId>,
  archive_status: ArchiveStatus,
}

TemplateId(Uuid)
Blake3Hash([u8; 32])
BoundedString<const N: usize>(String)  // TryFrom; reject >N → BoundedStringError; no panic

enum Domain { Code, Math, Prose, Mixed }           // closed; L3-locked companion §4
enum PatternKind { Decomposition, Reflection, Verification, Synthesis }  // closed; L3-locked
enum VerifierKind { Compile, Test, Prm, Refuse }   // closed; L3-locked; agent cannot self-weaken Prm→Refuse
enum ArchiveStatus { Candidate, Promoted, Retired, Frozen }
enum RetirementReason { Degradation, Superseded, Banned, MistakeBookTrigger }

enum Predicate {
  Always,
  MatchDomain(Domain),
  MinConfidence(f32),
  AndAll(Vec<Predicate>),
  OrAny(Vec<Predicate>),
}  // closed; recursion bounded by Vec length; eval MUST NOT call the LLM

enum TemplateError {
  BoundedStringExceeded,
  HashMismatch,
  ParentNotFound,
  StatusTransitionInvalid,
}

ArchiveEntry {
  self_hash: Blake3Hash,
  parent_hash: Option<Blake3Hash>,
  template: ReasoningTemplate,
  staged_eval_result: Option<StagedEvalResult>,
  promoted_at: Option<DateTime<Utc>>,
  retired_at: Option<DateTime<Utc>>,
  retirement_reason: Option<RetirementReason>,
}

StagedEvalResult {  // placeholder; D-5 BAI + D-7 canary populate
  stage_10_pass: u32, stage_10_total: u32,
  stage_50_pass: u32, stage_50_total: u32,
  stage_200_pass: u32, stage_200_total: u32,
  cross_domain_deltas: Vec<(Domain, f32)>,
}

ArchiveChain { storage_path: PathBuf, entries: Vec<ArchiveEntry> }
  load(path), append(entry), verify_chain()

TemplateRegistry { templates: FxHashMap<TemplateId, ReasoningTemplate> }
  load(archive_chain) → Promoted|Frozen only
  select_template(domain, pattern_kind, ctx) → Option<&ReasoningTemplate>
```

Derive `Debug/Clone/Serialize/Deserialize` on all types. `thiserror` for `TemplateError`. NO `unwrap`; NO `expect` (`D-1_reasoning_template_schema.md:110-112`).

**Hash functions:** `compute_content_hash(body, parent_hash) -> Blake3Hash`; `verify_hash(template) -> Result<(), TemplateError>` (`D-1_reasoning_template_schema.md:114-119`).

**SSB (SCHEMA_VERSION 47→48):** four `SsbActionType` variants (`D-1_reasoning_template_schema.md:143-151`):

```
ArchiveEntryAdded { template_id, content_hash: String, parent_hash: Option<String>,
                    domain: String, pattern_kind: String, archive_status: String }
ArchiveEntryRetired { template_id, retired_at: i64, retirement_reason: String }
TemplatePromoted { template_id, stage: String, pass_rate: f64 }
TemplateCandidateRejected { template_id, rejection_reason: String }
```

**Migrate report:** `count(extracted)`, `count(skipped_duplicate)`, `count(skipped_too_long)` (`D-1_reasoning_template_schema.md:162`).

## 5. Control flow / state machine

**Owner of the loop:** orchestrator `ArchiveChain` + `TemplateRegistry`. The model does not own status. CLI migrate is the bootstrap path (`D-1_reasoning_template_schema.md:154-174`).

**`ArchiveStatus` transitions the spec pins:**

| From | To | Allowed? | Spec source |
|---|---|---|---|
| (new migrate) | Frozen | yes (bootstrap only) | `:159`, decision (c) `:216-218` |
| Candidate | Promoted | yes | `:193` |
| Promoted | Retired | yes | `:193` |
| Frozen | anything | **no** | `:194`, `:261-262` |
| Retired | anything | **no** (immutable terminal) | `:194` |

**Forced rules:**

- Migrated templates are Frozen until D-5 BAI can score them (`D-1_reasoning_template_schema.md:216-218`).
- Registry serves Promoted ∪ Frozen only; Candidate/Retired are not selectable (`:168-169`).
- `verify_chain()` walks `parent_hash` links and recomputes `self_hash` (`:136`).
- Append-only by construction: no delete method (`:257`).

**UNCERTAIN:** Candidate→Retired, Candidate→Frozen, Promoted→Frozen, Frozen as a *from* only via migrate. Spec names four valid/invalid pairs and is silent on the rest (`D-1_reasoning_template_schema.md:193-194`).

## 6. Grounding, receipts, gates

This organ’s receipt is the **Blake3 content-hash chain**, not an LLM rating.

- `content_hash` on the template; `self_hash` / `parent_hash` on the archive entry.
- `verify_hash` recomputes from body + parent; `verify_chain` walks links (`D-1_reasoning_template_schema.md:114-121`, `:136`).
- Tamper of any archived `template_body` → `HashMismatch` → refuse use (`:345-348`).
- `verifier_required` is stored here; actual multi-verifier promotion is D-3 (≥2 verifiers must concur). D-1 does **not** run verifiers (`D-SUMMARY_INDEX.md:34-38`, `D-1_reasoning_template_schema.md:380-385`).

**Never allowed as a confidence signal:** LLM self-eval of a template it just wrote; scalar “looks good”; weakening `VerifierKind::Prm` to `Refuse`; cached hash compared without recompute; sha256/blake2 as the chain function (`D-1_reasoning_template_schema.md:100-101`, `:258-260`, `:294-299`).

**Tension (do not paper over):** `Predicate::MinConfidence(f32)` is in the closed enum (`:107-108`). Phase-2A/2B already ban scalar 0–1 model confidence as grounding. Rebuild: the f32 is an **eligibility input from `OrchContext`**, not a model-written quality score. How `OrchContext` supplies it is UNCERTAIN (section 16).

## 7. Budgets, limits, ceilings

| Knob | Value | Scope |
|---|---|---|
| `BoundedString<N>` default N | 1024 | per `template_body` |
| overflow | `Err(BoundedStringExceeded)` | construction time; no panic |
| `Blake3Hash` | `[u8; 32]` (64 hex chars on disk) | per template / entry |
| `Predicate` recursion | Vec length | no unbounded tree |
| `SCHEMA_VERSION` | 47 → 48 | SSB DB; v47 emits migration warn |
| storage | JSON Lines, O_APPEND | `.elai_cc/archive/reasoning-templates.jsonl` |
| registry map | `FxHashMap` | not `std::HashMap` |
| migrate | idempotent | second run extracts 0 new on unchanged input |
| unit tests | ≥8 new functions | `elai_orchestrator` scoped |
| clock | `SeededClock` | never `SystemTime::now` |

**Optional dep:** `fs2` advisory lock during append; if absent, confirm with owner before adding (`D-1_reasoning_template_schema.md:138-140`).

## 8. Roles and neighbors

D-1 is orchestrator-owned, role-agnostic. Prompt-engineer / self-mod agents **propose** bodies; they do not own the archive.

**Consumes:** blake3 crate, uuid, `SeededClock`, `FxHashMap`, `SsbActionType` (extended), `runtime_config` loader, `SlashCommand` enum. MistakeBook `RetirementReason` **feed is planned for D-4**, not wired in D-1 (`D-1_reasoning_template_schema.md:239-249`).

**Produces:** typed templates, JSONL archive, four SSB variants, `elai template migrate`, `[reasoning.templates]` config keys.

**Must not fork:** a second in-agent template store; sha256 fallback; SQLite archive as a substitute for JSONL append-only; prompt-only “keep templates frozen”; a delete/compact API.

**Downstream (do not implement here):** D-2 PPM (`VerifierKind::Prm`), D-5 BAI promotion, D-8 `elai archive verify` CLI, D-7 `cross_domain_deltas`, D-9 LockedKeysAuditor for L3 fields (`:379-389`).

**Not parallel-safe with:** D-2..D-9 (they consume these types). Parallel-safe with D-19, D-25 (`:17-20`).

## 9. Config / flags / defaults

`elai_cc.toml` commented section (`D-1_reasoning_template_schema.md:176-180`):

```
# [reasoning.templates]
# archive_path = ".elai_cc/archive/reasoning-templates.jsonl"
# bounded_string_max = 1024
```

Defaults if absent: those two values. `runtime_config.rs` reads the section. No dashboard in D-1 (D-26). No feature flag named; organ is code-driven. L3-frozen **documentation** of fields is in D-1; **enforcement** is D-9 (`:388-389`).

## 10. Implementation map

| Symbol | Spec path |
|---|---|
| `orchestrator/src/reasoning_template.rs` | types, hash, registry, select (`:358-361`) |
| `orchestrator/src/archive_chain.rs` | `ArchiveEntry`, `ArchiveChain`, `StagedEvalResult`, `verify_chain` (`:362-363`) |
| `cli/src/template_migrate.rs` | `elai template migrate` (`:364-365`) |
| `SlashCommand::TemplateMigrate` | `cli/src/commands.rs` (`:368-369`) |
| `SsbActionType` + SCHEMA 48 | `orchestrator/src/ssb.rs` (`:366-367`) |
| `pub mod` | `orchestrator/src/lib.rs` (`:372-373`) |
| contract test | `crates/contracts/tests/reasoning_template_schema_contract.rs` (`:374`) |
| decisions | `workflow/reasoning/D-1-decisions.md` (blake3 vs sha256; JSONL vs SQLite; Frozen vs Promoted) (`:210-218`) |
| papers | `workflow/papers/self-discover-2402.03620-interface.md`, `statistical-limits-2510.04399-interface.md` (`:83-85`) |
| storage | `.elai_cc/archive/reasoning-templates.jsonl` |

**Implementation-neighbor (not D-1 rebuild truth):** feature-map says template-migrate CLI landed 06-07 and `select_template` still dark in prod (`ELAI-FEATURE-MAP.md:635-638`). Rebuild the spec: `select_template` exists and is closed-form.

## 11. Journal / SSB events

SCHEMA_VERSION 48. Four variants, payloads as section 4, plus SQL serialization and round-trip tests (`D-1_reasoning_template_schema.md:143-152`, `:197-199`).

Observable: after migrate, `≥N` `ArchiveEntryAdded` rows with `N == reported migrate count`, all Frozen; zero `TemplatePromoted`; zero `TemplateCandidateRejected` (`:267-271`).

## 12. Tests and observables

Named unit tests (`D-1_reasoning_template_schema.md:182-195`):

| Id | Assert |
|---|---|
| T-bound | `BoundedString<1024>` accepts 1024 chars, rejects 1025 via `Err`, no panic |
| T-hash-det | identical `(body, parent)` → identical hash; different body OR parent → different hash; length 32 |
| T-tamper | mutate body → `verify_hash` → `HashMismatch` |
| T-jsonl | `append` writes a line; `load` reads it back identically |
| T-chain | `verify_chain` detects broken `parent_hash` (synthetic tamper) |
| T-pred | `AndAll([Always, Always]).eval == true`; `OrAny([Never, Always]) == true` (Never variant **if needed** — see gaps) |
| T-status | Candidate→Promoted ok; Promoted→Retired ok; Frozen→* invalid; Retired→* invalid |
| T-ssb | serialize `ArchiveEntryAdded` → deserialize → equal |

Contract (`:197-203`): `SCHEMA_VERSION_PUBLIC == 48`; 4 SSB variants + SQL round-trip; closed-enum exhaustive match (no extra variants); `append` + `verify_chain` on tmpdir.

Cloud (`:342-348`): (A) `/task --mode cloud` after migrate: ≥1 `ArchiveEntryAdded` already present, registry loads, task completes. (B) edit a `template_body` on disk → subsequent task `HashMismatch` and refuse.

Adversarial / stub-ban: no `unwrap`/`expect` in new code; blake3 ≥2 hits, sha256/blake2 = 0; no delete method; migrate second run 0 new (`:255-264`, `:315-318`).

Done-gates: workspace `cargo check` zero new errors; orchestrator tests all pass ≥8 functions; contracts test pass; papers present before EXECUTING (`:328-354`).

## 13. Rebuild contract

Another harness has **this** organ iff all of the following are true:

- `ReasoningTemplate` with the ten fields above, including `BoundedString<1024>` and `verifier_required: VerifierKind`.
- Closed enums `Domain` (4), `PatternKind` (4), `VerifierKind` (4), `ArchiveStatus` (4), `RetirementReason` (4), `Predicate` (5 listed variants). Exhaustive match tests.
- Blake3 content-hash of `body || parent_or_32_zero_bytes`; `verify_hash` recomputes; no sha256/blake2.
- Append-only `ArchiveEntry` JSONL with `self_hash` / `parent_hash`; `verify_chain` walks links; no delete.
- Status: Frozen and Retired are terminal; Candidate→Promoted and Promoted→Retired allowed; no force-override.
- Registry loads Promoted ∪ Frozen only; `select_template` is closed-form (no LLM).
- `elai template migrate` idempotent; migrated rows Frozen; SSB `ArchiveEntryAdded` with SCHEMA 48.
- `created_at` from injected clock, not wall `SystemTime::now`.
- `FxHashMap`, `thiserror`, zero `unwrap`/`expect` in non-test code.
- Tamper of a JSONL body is detected and the template is refused.

A prompt library of reasoning tips, a git history of `.toml` files, or DGM’s “any Python file” archive is a cousin, not this organ.

## 14. Non-goals / banned designs

- PPM sidecar / `VerifierKind::Prm` scoring — D-2 (`:380-382`).
- BAI Thompson promotion writing `TemplatePromoted` — D-5 (`:382-383`).
- User-facing `elai archive verify` — D-8; D-1 only ships `verify_chain` (`:384-385`).
- Filling `StagedEvalResult.cross_domain_deltas` — D-7 (`:386-387`).
- L3 access-control enforcement (pre-commit, LockedKeysAuditor) — D-9 (`:388-389`).
- Unbounded DGM-style code blobs (`:6-9`).
- sha256 or blake2 hash (`:12-13`, `:258`).
- SQLite as the archive (decision b: JSONL for physical append-only + git-diffable) (`:214-215`).
- Promoted status for migrated templates before BAI (`:216-218`).
- `std::HashMap` (`:166-167`).
- `SystemTime::now` (`:242-243`).
- Predicate eval calling an LLM (`:265-266`).
- Panic on `BoundedString` overflow (`:255-256`).
- Force-override of Frozen/Retired (`:261-262`).
- Model self-certifying a template it authored (promotion needs D-3 ≥2 verifiers).

## 15. Source citations

Primary: `docs/thefinalglance/D-plan/D-1_reasoning_template_schema.md` (full file). Index: `D-SUMMARY_INDEX.md:18-26`, `:267-268`. Feature map: `ELAI-FEATURE-MAP.md:635-638` (PARTIAL neighbor; `select_template` dark in prod). C-21 neighbor: `ELAI-FEATURE-MAP.md:1805-1807`. Companion design (not in this tree): `reasoning-improvement-grounded-2026-05-19.md` §3.1 / §4 / §2.1 as cited by D-1 `:53-58`. Papers named: SELF-DISCOVER arxiv:2402.03620, Statistical Limits arxiv:2510.04399, DGM arxiv:2505.22954, checksum rec arxiv:2507.05619.

## 16. Gaps

Label **UNCERTAIN**; do not fill:

- `Predicate::Never` is used in the unit-test sketch but is **not** in the enum (`D-1_reasoning_template_schema.md:107-109` vs `:191-192`).
- Full `ArchiveStatus` transition table beyond the four pinned pairs (`:193-194`).
- Whether `content_hash` ≡ `ArchiveEntry.self_hash` or they hash different concatenations (`:114-116` vs `:125-126`).
- Exact byte concatenation for `body.as_bytes() || parent_hash` (length prefix? raw 32 bytes?).
- `select_template` “first match” order (JSONL order vs id vs hash) (`:170-172`).
- How `Predicate::eval` reads `OrchContext`, especially `MinConfidence(f32)` (`:107-108`, `:170-172`).
- Whether `bounded_string_max` in toml can change `N` or the const is always 1024 (`:176-180`).
- `fs2` lock: add dep vs skip if absent (`:138-140`).
- Domain 4-value enum vs D-7 five canary families (code-fix, math-proof, prose-reason, planning, knowledge-QA) (`D-SUMMARY_INDEX.md:58-59` vs D-1 `Domain`).
- SCHEMA 48 vs later feature-map SCHEMA ~268 — bump arithmetic for a tree that already passed 48 (`ELAI-FEATURE-MAP.md:4` header vs D-1 `:148`).
- `agent_definitions/*.md` vs `.claude/agents/*.md` path (`:27` vs `:156`).
- Whether `RetirementReason::MistakeBookTrigger` is written in D-1 or only reserved until D-4 (`:246-247`).
- Companion §3.1 field list vs D-1 struct: rebuild D-1; companion file is not in this repo (grep found no `reasoning-improvement-grounded*.md`).

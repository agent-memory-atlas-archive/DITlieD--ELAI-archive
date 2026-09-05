# DEEP-D-002 deep feature discovery

## Scope, evidence, and notation

- Assigned chunk: DEEP-D-002 only.
- Original-source boundary: exactly the eight D-plan files in the manifest table below were read from first through last logical line. No other D-plan original, companion design, source code, paper interface, fixture, or artifact was opened.
- Orientation only: docs/thefinalglance/feature-discovery/04-D-SUMMARY_INDEX.discovery.md was read 777/777 lines solely to identify summary additions and corrections. It is not used to promote an original-plan detail to shipped fact.
- Manifest evidence: docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md, section headed DEEP-D-002 at lines 469-482.
- Anchors below use basename:line or basename:start-end and refer to the exact source bytes whose SHA-256 values were verified.
- Classification: [E] is directly explicit in an assigned original; [I] is a synthesis from explicit assigned-source statements; [A] is materially underspecified; [C] is internally or cross-source conflicting. A plan-recorded claim that code is live is reported as such and is not independently verified.
- Category tags: CAP capability/action/surface; FLOW workflow/state/lifecycle; DATA schema/data; CFG flag/default/config; INT integration/event; SEC permission/security/privacy/compliance; FAIL failure/fallback/retry/recovery; DET determinism/idempotency/concurrency; OBS observability/evidence; TEST test/acceptance/done gate; CON constraint/non-goal; DEP dependency/status/deferred.
- Atomic record rule: one numbered ledger row is one independently checkable requirement, behavior, declared set, state rule, or unresolved proposition. Interactions and summary deltas cite these records and are not counted again.

## Exact processed manifest and integrity verification

| Chunk | Class | Verified lines | Verified bytes | Verified SHA-256 | Result | Exact absolute path |
|---|---|---:|---:|---|---|---|
| DEEP-D-002 | numbered/index original | 395 | 22049 | 9ce34c7a98f3fdeaa7231dd3038d6e01799341e89c426aa94279e0dc8e01770e | exact match | docs/thefinalglance/D-plan/D-1_reasoning_template_schema.md |
| DEEP-D-002 | numbered/index original | 241 | 11140 | 3b0df57f1daec68276f23c5450adb8d43aa30b63247cd831aa6adc6a5ee0d8bc | exact match | docs/thefinalglance/D-plan/D-8_crypto_archive_verify.md |
| DEEP-D-002 | numbered/index original | 331 | 15751 | b65d3f1e968c4165308395790b3f763e5d2b02719a8d0e32f7b5dcf4eaf02c4b | exact match | docs/thefinalglance/D-plan/D-9_l3_frozen_surfaces.md |
| DEEP-D-002 | numbered/index original | 284 | 13147 | 228e9fccb6d8603ecdc686094f9e6957eeecbcd301771014ab37d35375f95154 | exact match | docs/thefinalglance/D-plan/D-10_mode_routing_predicate.md |
| DEEP-D-002 | numbered/index original | 225 | 10499 | d0a300d465da3ef6d0da2774b092a275645b736eb50bbf2671012d9460cf5390 | exact match | docs/thefinalglance/D-plan/D-14_sycophancy_probe.md |
| DEEP-D-002 | numbered/index original | 122 | 4783 | 231181682761531f25ec4c93cd4b551363885da8e6695ed58c01ee8a7eee4294 | exact match | docs/thefinalglance/D-plan/D-21_mode_routing_coverage_benchmark.md |
| DEEP-D-002 | numbered/index original | 101 | 3359 | 3353c13e72b2f67a79d81bef25975e12cb915215b4fbde5a69c4a875e740df2e | exact match | docs/thefinalglance/D-plan/D-24_cross_domain_canary_regression_suite.md |
| DEEP-D-002 | numbered/index original | 156 | 6995 | 059a098165c07c89e49080ad1284197b81fec0f889298da4b2170be96c5c8ab5 | exact match | docs/thefinalglance/D-plan/D-28_adversarial_example_farming.md |
| **Total** | **8 unique files** | **1855** | **87723** | — | **manifest total exact** | — |

Verification used logical record count, exact byte count, and SHA-256 over the source bytes. Manifest rows = 8; unique manifest paths = 8; files opened = 8; fully read = 8; missing = 0; extra originals = 0; duplicate manifest paths = 0; duplicate ledger source IDs = 0.

## Per-file exhaustive ledgers

### D-1_reasoning_template_schema.md

Source read: 395/395 lines. Original status is not labeled in this file; it is a future-tense implementation plan with unchecked acceptance boxes. The orientation summary labels D-1 PLANNED.

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D1-E01 | [E] | CAP,DATA,SEC | Replace prompt-string reasoning patterns with bounded typed ReasoningTemplate records and bootstrap an append-only ArchiveEntry chain as the mutation substrate. | D-1_reasoning_template_schema.md:3-9 |
| D1-E02 | [E] | OBS,DEP | The plan records the then-current gap as no ReasoningTemplate, roughly 25 TOML prompt files plus inline agent strings, and no hash, lineage, or archive lifecycle. | D-1_reasoning_template_schema.md:22-30 |
| D1-E03 | [E] | SEC,CON | Extend L1A from data tables to data tables plus typed reasoning templates; the closed schema is the intended capacity bound. | D-1_reasoning_template_schema.md:39-42 |
| D1-E04 | [E] | DEP,FLOW | Estimate 1-2 sessions; require C-15 and C-21 DONE, with spec-grounder verification before EXECUTING; use C-15 Blake3 and C-21 MistakeBook BLOCK inputs. | D-1_reasoning_template_schema.md:10-16 |
| D1-E05 | [E] | DET,DEP | Parallel-safe with D-19 and D-25; not parallel-safe with D-2 through D-9 because those consume the new records. | D-1_reasoning_template_schema.md:17-20 |
| D1-E06 | [E] | DEP,TEST | Ingest SELF-DISCOVER and Statistical Limits interfaces, and require both files before TESTING_FIRST. | D-1_reasoning_template_schema.md:76-85 |
| D1-E07 | [E] | DATA | ReasoningTemplate fields are id, domain, pattern_kind, template_body as BoundedString<1024>, eligibility_predicate, verifier_required, content_hash, created_at, parent_id, and archive_status. | D-1_reasoning_template_schema.md:89-95 |
| D1-E08 | [E] | DATA,SEC | Domain is the L3-locked closed enum Code, Math, Prose, Mixed. | D-1_reasoning_template_schema.md:96-97 |
| D1-E09 | [E] | DATA,SEC | PatternKind is the L3-locked closed enum Decomposition, Reflection, Verification, Synthesis. | D-1_reasoning_template_schema.md:98-99 |
| D1-E10 | [E] | DATA,SEC,FAIL | VerifierKind is the L3-locked closed enum Compile, Test, Prm, Refuse; an agent may not weaken Prm to Refuse. | D-1_reasoning_template_schema.md:100-101 |
| D1-E11 | [E] | DATA,FLOW | ArchiveStatus values are Candidate, Promoted, Retired, Frozen; RetirementReason values are Degradation, Superseded, Banned, MistakeBookTrigger. | D-1_reasoning_template_schema.md:102-103 |
| D1-E12 | [E] | DATA | TemplateId is a Uuid newtype and Blake3Hash is a 32-byte-array newtype. | D-1_reasoning_template_schema.md:104 |
| D1-E13 | [E] | DATA,FAIL,CON | BoundedString<N> validates in TryFrom, rejects lengths greater than N without panic, and returns BoundedStringError. | D-1_reasoning_template_schema.md:105-106 |
| D1-E14 | [E] | DATA,SEC,CON | Predicate is the closed, non-LLM enum Always, MatchDomain, MinConfidence, AndAll, OrAny; recursive fan-out is bounded by vector length. | D-1_reasoning_template_schema.md:107-109,265-266 |
| D1-E15 | [E] | DATA,FAIL,CON | All types derive Debug, Clone, Serialize, Deserialize; TemplateError has BoundedStringExceeded, HashMismatch, ParentNotFound, StatusTransitionInvalid; new code forbids unwrap and expect. | D-1_reasoning_template_schema.md:110-112,353 |
| D1-E16 | [E] | DET,SEC | compute_content_hash hashes body bytes concatenated with the supplied parent hash, or 32 zero bytes for a root, using Blake3; identical inputs must produce the same hash. | D-1_reasoning_template_schema.md:114-117 |
| D1-E17 | [E] | FAIL,SEC | verify_hash recomputes a template hash and returns HashMismatch on divergence; it may not compare cached values. | D-1_reasoning_template_schema.md:118-119,259-260 |
| D1-E18 | [E] | TEST,DET | Hash tests require equality for identical body/parent, inequality when body or parent differs, and exactly 32 output bytes. | D-1_reasoning_template_schema.md:120-121 |
| D1-E19 | [E] | DATA,FLOW | ArchiveEntry fields are self_hash, parent_hash, template, staged_eval_result, promoted_at, retired_at, retirement_reason. | D-1_reasoning_template_schema.md:123-128 |
| D1-E20 | [E] | DATA,OBS | StagedEvalResult carries pass/total counts at stages 10, 50, and 200 plus cross_domain_deltas by Domain; D-5 and D-7 populate the placeholder. | D-1_reasoning_template_schema.md:129-132 |
| D1-E21 | [E] | DATA,FLOW,FAIL,SEC | ArchiveChain has storage_path and entries; load, append, and verify_chain are required; append never deletes, while verification walks parent links and recomputes hashes. | D-1_reasoning_template_schema.md:133-135 |
| D1-E22 | [E] | DATA,SEC | Store one serialized ArchiveEntry per JSONL line at .elai_cc/archive/reasoning-templates.jsonl and open writes in O_APPEND mode only. | D-1_reasoning_template_schema.md:136-137 |
| D1-E23 | [E] | DET,DEP | Serialize concurrent appends with an fs2 advisory lock; if fs2 is absent, it is the sole optional dependency and requires user confirmation before addition. | D-1_reasoning_template_schema.md:138-140 |
| D1-E24 | [E] | INT,DATA,OBS | Bump SSB SCHEMA_VERSION 47 to 48 and add ArchiveEntryAdded, ArchiveEntryRetired, TemplatePromoted, and TemplateCandidateRejected with their listed payload fields, SQL serialization, and round-trip coverage. | D-1_reasoning_template_schema.md:142-152 |
| D1-E25 | [E] | CAP,FLOW,DATA | elai template migrate scans prompt-template TOML and agent-definition Markdown, extracts unique patterns, hashes each, creates Frozen ReasoningTemplate entries, and writes the archive JSONL. | D-1_reasoning_template_schema.md:154-159 |
| D1-E26 | [E] | DET,OBS | Migration is idempotent on unchanged inputs, skips existing hashes, and reports extracted, skipped_duplicate, and skipped_too_long counts. | D-1_reasoning_template_schema.md:160-162 |
| D1-E27 | [E] | DATA,CON | TemplateRegistry maps TemplateId to ReasoningTemplate with FxHashMap, explicitly not std HashMap. | D-1_reasoning_template_schema.md:164-167 |
| D1-E28 | [E] | FLOW | TemplateRegistry.load admits only Promoted and Frozen archive entries. | D-1_reasoning_template_schema.md:168-169 |
| D1-E29 | [E] | FLOW,DEP | select_template filters by domain, pattern kind, and a closed-form eligibility predicate, then returns the first match; replacing task-runner prompt literals is deferred to D-2. | D-1_reasoning_template_schema.md:170-174 |
| D1-E30 | [E] | CFG | Add a commented reasoning.templates config section; absent values default archive_path to .elai_cc/archive/reasoning-templates.jsonl and bounded_string_max to 1024. | D-1_reasoning_template_schema.md:176-180 |
| D1-E31 | [E] | TEST | Unit coverage includes 1024/1025 string boundaries, repeatable hash, body-tamper detection, append/load identity, broken-parent detection, and predicate truth tables. | D-1_reasoning_template_schema.md:182-192 |
| D1-E32 | [E] | FLOW,SEC,TEST | Candidate to Promoted and Promoted to Retired are valid; Frozen and Retired are immutable terminal states, with no force-override flag. | D-1_reasoning_template_schema.md:193-194,261-262 |
| D1-E33 | [E] | TEST,DATA | The contract test requires public schema 48, all four SSB variants and SQL round trips, allow-listed closed enums, and append-plus-verify end to end. | D-1_reasoning_template_schema.md:197-203 |
| D1-E34 | [E] | TEST,OBS | Manual migration smoke reports all three counts and inspects the first three JSONL entries for valid JSON, 64-character hashes, and Frozen status. | D-1_reasoning_template_schema.md:205-209 |
| D1-E35 | [E] | CON,OBS | D-1-decisions.md must record why Blake3, why JSONL instead of SQLite, and why migrated templates are Frozen rather than Promoted. | D-1_reasoning_template_schema.md:211-218 |
| D1-E36 | [E] | FLOW,OBS,DEP | During MEMORIZING, memory-writer must emit project_d1_reasoning_template.md containing schema, lifecycle, and SSB variants for D-2 through D-9. | D-1_reasoning_template_schema.md:220-222 |
| D1-E37 | [E] | DEP,DET | created_at must use injected SeededClock and never SystemTime::now; MistakeBook retirement feed is planned for D-4, not wired in D-1. | D-1_reasoning_template_schema.md:239-248 |
| D1-E38 | [E] | CAP,INT | Reachability is through elai template migrate, registry startup loading, and future D sessions reading ArchiveChain through an in-memory FxHashMap after JSONL load. | D-1_reasoning_template_schema.md:250-254 |
| D1-E39 | [E] | CON,SEC | ArchiveChain may expose no delete method; content hashing must never use SHA-256 or Blake2; Predicate evaluation may not call an LLM. | D-1_reasoning_template_schema.md:255-266 |
| D1-E40 | [E] | OBS,TEST | A migration run must yield N ArchiveEntryAdded rows matching its reported count, all Frozen, with zero TemplatePromoted and TemplateCandidateRejected rows; the whole artifact must parse and verify. | D-1_reasoning_template_schema.md:267-274 |
| D1-E41 | [E] | TEST,FAIL | Acceptance requires public schema 48, a migration warning for an old version-47 SSB database, error-return rather than panic, at least eight new unit tests, contract tests, and a clean workspace build. | D-1_reasoning_template_schema.md:325-341 |
| D1-E42 | [E] | TEST,FAIL,SEC | Cloud smoke A verifies registry loading and task completion with a migrated template; smoke B manually tampers template_body and requires HashMismatch plus orchestrator refusal to use it. | D-1_reasoning_template_schema.md:342-351 |
| D1-E43 | [E] | TEST,OBS | Real-repository migration must migrate at least as many templates as the template-directory TOML file count, be idempotent on the second run, and produce the decision and memory artifacts. | D-1_reasoning_template_schema.md:335-354 |
| D1-E44 | [E] | CON,DEP | Out of scope are PPM wiring, BAI promotion, user-facing archive verification, per-template performance metrics, and enforcement of documented L3 locks; these belong to D-2, D-5, D-8, D-7, and D-9 respectively. | D-1_reasoning_template_schema.md:379-389 |
| D1-E45 | [E] | DEP,DET | After D-1, D-2 and D-8 may start in parallel: D-2 adds verifier substrate and D-8 exposes verify_chain as CLI. | D-1_reasoning_template_schema.md:391-395 |

#### D-1 unresolved and inferred records

| ID | Class | Category | Record | Anchor |
|---|---|---|---|---|
| D1-N01 | [A] | DEP | The original never declares PLANNED, IN PROGRESS, or DONE; PLANNED comes only from the orientation summary, and this pass did not inspect implementation. | D-1_reasoning_template_schema.md:1-395 |
| D1-N02 | [C] | DATA,SEC | D-1 defines template content_hash from body plus parent hash, while D-8 defines archive self-hash from body plus parent hash plus mutation_proposal_text. | D-1_reasoning_template_schema.md:114-121; D-8_crypto_archive_verify.md:16-28 |
| D1-N03 | [A] | DATA | Neither ReasoningTemplate nor ArchiveEntry declares mutation_proposal_text, leaving the D-8 hash input without a storage field in D-1's schema. | D-1_reasoning_template_schema.md:91-95,125-128; D-8_crypto_archive_verify.md:16-28 |
| D1-N04 | [A] | FLOW | StatusTransitionInvalid and transition tests exist, but the implementing transition type or method and the append-only representation of promotion/retirement are not specified. | D-1_reasoning_template_schema.md:111-112,193-194 |
| D1-N05 | [A] | DATA,TEST | Predicate tests permit adding Never if needed, although Never is absent from the declared closed enum; its normative membership is unresolved. | D-1_reasoning_template_schema.md:107-109,191-192 |
| D1-N06 | [A] | CFG,DATA | BoundedString<1024> is compile-time-sized while bounded_string_max is runtime-configurable; behavior when configuration differs from 1024 is not defined. | D-1_reasoning_template_schema.md:92-95,176-180 |
| D1-N07 | [A] | DET,FLOW | select_template returns the first eligible FxHashMap result, but tie ordering and a stable selection key are not specified. | D-1_reasoning_template_schema.md:164-172 |
| D1-N08 | [A] | FLOW,SEC | The file requires immutable append-only storage and mutable lifecycle states but does not define whether transitions append successor entries, append events, or rewrite prior JSONL rows. | D-1_reasoning_template_schema.md:102-103,123-137,193-194 |
| D1-N09 | [A] | CAP,DET | Pattern extraction and deduplication rules for free-form TOML/Markdown are not defined beyond uniqueness by resulting hashes. | D-1_reasoning_template_schema.md:154-162 |
| D1-N10 | [A] | DATA,SEC | `ReasoningTemplate.content_hash`, `ArchiveEntry.self_hash`, and the value recomputed by `verify_chain` are all named, but D-1 never states whether the two hashes must be identical or whether an entry has a second envelope hash. | D-1_reasoning_template_schema.md:91-95,114-128,133-137 |
| D1-N11 | [A] | DATA,FLOW | Template lineage uses `parent_id: Option<TemplateId>` while archive lineage uses `parent_hash: Option<Blake3Hash>`; lookup, validation, and consistency rules between those two parent references are not specified. | D-1_reasoning_template_schema.md:91-95,125-135 |
| D1-N12 | [A] | DATA,SEC,CON | The source calls predicate recursion bounded by vector length, but supplies no maximum vector length, nesting depth, cycle defense, or evaluation budget; the effective capacity bound is therefore unspecified. | D-1_reasoning_template_schema.md:107-109,265-266 |

### D-8_crypto_archive_verify.md

Source read: 241/241 lines. Original status is not labeled; the orientation summary labels D-8 PLANNED.

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D8-E01 | [E] | CAP,SEC,OBS | Expose D-1 ArchiveChain.verify_chain as a user CLI and CI check, emit ArchiveVerifyResult, and defend against an agent rewriting its audit history. | D-8_crypto_archive_verify.md:3-8 |
| D8-E02 | [E] | DEP,DET | Require D-1, with spec-grounder verification; run in parallel with D-2 through D-7 because D-8 reads but does not extend ArchiveChain beyond reporting. | D-8_crypto_archive_verify.md:9-12 |
| D8-E03 | [E] | DATA,SEC | The context defines each self hash as Blake3 over template_body, parent_hash, and mutation_proposal_text. | D-8_crypto_archive_verify.md:16-19,25-27 |
| D8-E04 | [E] | FAIL,SEC | Chain verification recomputes every hash, requires each parent to be an earlier existing entry, and reports hash/link/parent failures. | D-8_crypto_archive_verify.md:25-28 |
| D8-E05 | [E] | CAP,CFG | ArchiveVerifyCmd contains archive_path, json_output, and fail_fast. | D-8_crypto_archive_verify.md:52-55 |
| D8-E06 | [E] | DATA,OBS | ArchiveVerifyReport contains total_entries, verified, and a vector of ArchiveVerifyError. | D-8_crypto_archive_verify.md:56-57 |
| D8-E07 | [E] | DATA,FAIL | ArchiveVerifyError variants are HashMismatch with expected/computed and index, BrokenParent with index/hash, DuplicateHash with current/prior indexes, and MalformedJson with index/error. | D-8_crypto_archive_verify.md:58-61 |
| D8-E08 | [E] | FLOW,FAIL | run loads the archive, calls verify_chain, aggregates all errors by default, and short-circuits on the first error only when fail_fast is true. | D-8_crypto_archive_verify.md:62-66 |
| D8-E09 | [E] | FAIL,OBS | Output is human-readable or JSON; zero errors exits 0, any error exits 1, and an ArchiveVerifyResult row is emitted regardless of exit code. | D-8_crypto_archive_verify.md:67-69 |
| D8-E10 | [E] | CAP,CFG | Add SlashCommand::ArchiveVerify and dispatch elai archive verify with path, json, and fail-fast flags; default to the D-1 archive path. | D-8_crypto_archive_verify.md:71-77 |
| D8-E11 | [E] | INT,DATA,OBS | Bump SSB schema 53 to 54 and add ArchiveVerifyResult with audit_id, total_entries, verified, error_count, and errors, plus round-trip coverage. | D-8_crypto_archive_verify.md:79-82 |
| D8-E12 | [E] | INT,TEST,SEC | archive-verify.yml runs on push and pull_request to main plus nightly cron, builds release elai_cli, and invokes archive verify with fail-fast so integrity failure fails CI. | D-8_crypto_archive_verify.md:84-89 |
| D8-E13 | [E] | INT,CFG | CI uses windows-latest and ubuntu-latest, and skips when the commit message contains [ci skip]. | D-8_crypto_archive_verify.md:90-92 |
| D8-E14 | [E] | TEST,FAIL | Tamper scenario A edits entry 3 template_body and must report HashMismatch at entry 3 and exit 1. | D-8_crypto_archive_verify.md:94-98 |
| D8-E15 | [E] | TEST,FAIL | Tamper scenario B points entry 4 at a nonexistent parent and must report BrokenParent and exit 1. | D-8_crypto_archive_verify.md:99-100 |
| D8-E16 | [E] | TEST,FAIL | Tamper scenario C duplicates entry 2's self hash at entry 5 and must report DuplicateHash and exit 1. | D-8_crypto_archive_verify.md:101-102 |
| D8-E17 | [E] | TEST,FAIL | Tamper scenario D inserts malformed JSON and must report MalformedJson while continuing when fail_fast is false. | D-8_crypto_archive_verify.md:103-104 |
| D8-E18 | [E] | TEST,OBS | A clean five-entry archive exits 0 and emits ArchiveVerifyResult with error_count zero. | D-8_crypto_archive_verify.md:105-106 |
| D8-E19 | [E] | TEST,DET,CON | An ignored manual performance test synthesizes 10,000 entries and requires verification within five seconds and O(n), not O(n-squared), behavior. | D-8_crypto_archive_verify.md:108-113 |
| D8-E20 | [E] | FAIL,OBS | Documentation must cover CLI/CI and recovery: after failure, treat the last N agent actions as suspect and roll back by forensic chain walk. | D-8_crypto_archive_verify.md:115-119 |
| D8-E21 | [E] | CON,CFG | The decision record explains separate CLI/CI roles, Blake3 speed/consistency, and full-report default versus early-exit fail-fast mode. | D-8_crypto_archive_verify.md:121-128 |
| D8-E22 | [E] | DEP,CAP | The command consumes ArchiveChain, SlashCommand, and SsbWriter and is reachable from direct CLI, CI, and cargo-run development invocation. | D-8_crypto_archive_verify.md:130-145 |
| D8-E23 | [E] | CON,FAIL | I/O may not unwrap; clean must not always fail; tamper must not silently pass; json and fail-fast flags must be functional. No retry behavior is defined. | D-8_crypto_archive_verify.md:146-151 |
| D8-E24 | [E] | OBS,TEST | Each clean or tampered invocation emits exactly one ArchiveVerifyResult; the workflow artifact must report a PR commit status. | D-8_crypto_archive_verify.md:152-157 |
| D8-E25 | [E] | TEST | Deterministic checks cover schema 54, command/types/event/workflow presence, clean verification, synthetic corrupt-line detection, workspace build, contract tests, and the ignored performance test. | D-8_crypto_archive_verify.md:159-194 |
| D8-E26 | [E] | TEST,FAIL | Acceptance additionally requires parseable JSON and demonstrates that default collection reports at least two errors when at least two tampers exist. | D-8_crypto_archive_verify.md:196-211 |
| D8-E27 | [E] | TEST,SEC | Cloud smoke verifies a clean archive after live promotion, then a manual body corruption that exits 1 with HashMismatch. | D-8_crypto_archive_verify.md:212-217 |
| D8-E28 | [E] | CAP,INT | Files include the CLI module/dispatch, SSB update, CI workflow, contract and performance tests, decision record, and recovery documentation. | D-8_crypto_archive_verify.md:219-229 |
| D8-E29 | [E] | CON,DEP | D-25's ten-case suite and D-9 lock enforcement are out of scope; archive compaction or sled migration is deferred because JSONL is expected to scale to about 10,000 entries. | D-8_crypto_archive_verify.md:231-241 |

#### D-8 unresolved records

| ID | Class | Category | Record | Anchor |
|---|---|---|---|---|
| D8-N01 | [C] | DATA,SEC | Its three-part hash formula conflicts with D-1's two-part compute_content_hash formula and depends on mutation_proposal_text absent from D-1's records. | D-8_crypto_archive_verify.md:16-28; D-1_reasoning_template_schema.md:91-95,114-128 |
| D8-N02 | [C] | FAIL | Context names BrokenLink and DanglingParent, while the normative error enum names BrokenParent and has no BrokenLink or DanglingParent variant. | D-8_crypto_archive_verify.md:25-28,58-61 |
| D8-N03 | [A] | FAIL,FLOW | ArchiveChain load precedes verification, but malformed JSON is expected to be an aggregatable per-line error; how load preserves later lines is unspecified. | D-8_crypto_archive_verify.md:62-66,103-104 |
| D8-N04 | [A] | FAIL | Missing file, permission denial, unreadable path, and empty-archive behavior are not defined. | D-8_crypto_archive_verify.md:52-69 |
| D8-N05 | [A] | OBS,FAIL | The report promises an SSB row regardless of exit code, but no path is specified for emitting it when loading or SSB writing itself fails. | D-8_crypto_archive_verify.md:62-69 |
| D8-N06 | [A] | FAIL,SEC | Recovery identifies suspect actions and a forensic walk but does not define the trusted restore source, exact rollback command, or how the append-only file is repaired. | D-8_crypto_archive_verify.md:115-119 |
| D8-N07 | [A] | INT | The plan places workflows beneath .elai_cc/.github; it does not state whether .elai_cc is the repository root or how a parent-repository CI installation discovers them. | D-8_crypto_archive_verify.md:84-92,219-229 |
| D8-N08 | [C] | DEP,FAIL | D-8 delegates to D-1 `verify_chain`, yet D-1 exposes only template-level `HashMismatch`/`ParentNotFound` errors and does not define duplicate-hash or malformed-line aggregation; D-8 requires four richer per-entry error variants without specifying the adapter boundary. | D-8_crypto_archive_verify.md:52-69,94-106; D-1_reasoning_template_schema.md:110-119,133-137 |
| D8-N09 | [A] | DET,FLOW,SEC | Verification concurrency is undefined: the source does not say whether the reader takes D-1's advisory lock, snapshots the file, or can observe a partial line while another process appends. | D-8_crypto_archive_verify.md:62-69; D-1_reasoning_template_schema.md:133-140 |
| D8-N10 | [A] | SEC,INT | `[ci skip]` disables the integrity workflow, but no authorization policy or protected-branch control constrains who may use that bypass. | D-8_crypto_archive_verify.md:84-92 |
| D8-N11 | [A] | DATA,FAIL | Error `entry_index` semantics are unspecified (zero- versus one-based, and whether malformed physical lines count), which affects machine consumers and the recovery walk. | D-8_crypto_archive_verify.md:56-61,94-106 |
| D8-N12 | [C] | TEST | Acceptance calls the five cases “5 tamper scenarios” although one of the five is explicitly the clean control and only four are tampers. | D-8_crypto_archive_verify.md:94-106,198-204 |

### D-9_l3_frozen_surfaces.md

Source read: 331/331 lines. Original status is not labeled; the orientation summary labels D-9 PLANNED.

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D9-E01 | [E] | CAP,SEC | Structurally enforce expanded L3-frozen surfaces with a pre-commit hook, LockedKeysAuditor, LockedKeysTouch event, and a blocked agent edit path. | D-9_l3_frozen_surfaces.md:3-8 |
| D9-E02 | [E] | DEP,DET | Require D-1 through D-8 DONE; run in parallel with D-10 through D-18; coordinate hook ordering with C-27 rather than running that overlapping change independently. | D-9_l3_frozen_surfaces.md:9-15 |
| D9-E03 | [E] | SEC | Freeze staged-evaluation harness code. | D-9_l3_frozen_surfaces.md:19-22 |
| D9-E04 | [E] | SEC | Freeze tripwire thresholds and detection code. | D-9_l3_frozen_surfaces.md:22,31-32 |
| D9-E05 | [E] | SEC | Freeze MSI/BAI scoring-function code. | D-9_l3_frozen_surfaces.md:23,33 |
| D9-E06 | [E] | SEC | Freeze archive maintenance and parent-selection algorithm code. | D-9_l3_frozen_surfaces.md:24 |
| D9-E07 | [E] | SEC | Freeze the held-out task-selection algorithm. | D-9_l3_frozen_surfaces.md:25 |
| D9-E08 | [E] | SEC | Freeze cross-domain canary family definitions. | D-9_l3_frozen_surfaces.md:26 |
| D9-E09 | [E] | SEC | Freeze cryptographic hash choice to Blake3. | D-9_l3_frozen_surfaces.md:27 |
| D9-E10 | [E] | SEC | Freeze the AllowedConfigKey bound so an agent cannot add variants. | D-9_l3_frozen_surfaces.md:28 |
| D9-E11 | [E] | SEC | Freeze every template's verifier_required field against self-weakening. | D-9_l3_frozen_surfaces.md:29 |
| D9-E12 | [E] | SEC | Freeze the Domain enum against self-added domains. | D-9_l3_frozen_surfaces.md:30 |
| D9-E13 | [E] | SEC,CFG | Freeze tripwire config keys kl_spec_tau, rho_delta, iqr_mult, ppl_sigma, ddm thresholds, consensus_fire_count, and window_size. | D-9_l3_frozen_surfaces.md:31-32 |
| D9-E14 | [E] | SEC,CFG | Freeze BAI eval_budget, ci_alpha, and per-family delta. | D-9_l3_frozen_surfaces.md:33 |
| D9-E15 | [E] | SEC,CFG | Freeze held-out ttl_months, ngram_size, and jaccard_threshold. | D-9_l3_frozen_surfaces.md:34 |
| D9-E16 | [E] | SEC,CFG | Freeze canary regression_threshold and min_samples. | D-9_l3_frozen_surfaces.md:35 |
| D9-E17 | [E] | SEC,FLOW | Enforcement has three layers: authoritative documentation/registry, pre-commit rejection, and a runtime auditor over attempted writes/events. | D-9_l3_frozen_surfaces.md:42-46 |
| D9-E18 | [E] | DATA,SEC | locked_keys_registry.toml records locked_path with reason/override key, locked_symbol with file/reason, and locked_toml_key with file/section/keys/reason. | D-9_l3_frozen_surfaces.md:75-104 |
| D9-E19 | [E] | TEST,SEC | Populate the registry with all 14 named surface classes and their per-file/per-symbol details. | D-9_l3_frozen_surfaces.md:105-106 |
| D9-E20 | [E] | FLOW,SEC | locked_keys_check.sh parses the registry and cached name-only diff; a locked path without a matching L3_OVERRIDE tag exits 1 with a structured error. | D-9_l3_frozen_surfaces.md:108-113 |
| D9-E21 | [E] | FLOW,SEC | The same check inspects diff bodies for locked symbols and parses TOML section/key changes. | D-9_l3_frozen_surfaces.md:114-116 |
| D9-E22 | [E] | DATA,SEC | LockedKeysAuditor owns a registry, SSB sender, and audit UUID; LockedKeysRegistry stores vectors of paths, symbols, and TOML keys. | D-9_l3_frozen_surfaces.md:118-122 |
| D9-E23 | [E] | FLOW,SEC | audit_diff returns an event per unauthorized match; audit_runtime_write performs the analogous check before a pending runtime write. | D-9_l3_frozen_surfaces.md:122-127 |
| D9-E24 | [E] | FLOW,SEC,OBS | task_runner's PreToolUse/Tool::Write path invokes the auditor, blocks a touched locked surface, and emits LockedKeysTouch. | D-9_l3_frozen_surfaces.md:129-134 |
| D9-E25 | [E] | INT,DATA,OBS | Bump SSB schema 54 to 55; LockedKeysTouch carries path/symbol/TOML location/override flag/audit ID; L3OverrideRequested carries key/requester/a 512-bound justification. | D-9_l3_frozen_surfaces.md:136-144 |
| D9-E26 | [E] | SEC,OBS | L3OverrideRequested is informational documentation of a human decision, explicitly not an in-code permission-grant mechanism. | D-9_l3_frozen_surfaces.md:140-143 |
| D9-E27 | [E] | CFG,SEC | A valid override is a registered L3_OVERRIDE key in the commit body with at least a 40-character justification; valid overrides allow the commit, others block. | D-9_l3_frozen_surfaces.md:146-159 |
| D9-E28 | [E] | OBS,SEC | Documentation lists all 14 surfaces, points to the registry as authoritative, documents the override protocol, and cites its research grounding. | D-9_l3_frozen_surfaces.md:161-166 |
| D9-E29 | [E] | TEST | Unit tests cover unauthorized and overridden path edits, a tripwire TOML-key edit with populated fields, and an unrelated path. | D-9_l3_frozen_surfaces.md:168-175 |
| D9-E30 | [E] | TEST,SEC | Integration tests create a temporary git repository and require exit 1 without an override and exit 0 with one. | D-9_l3_frozen_surfaces.md:177-181 |
| D9-E31 | [E] | INT,SEC | locked-keys-audit.yml runs the script on push and pull_request to main and fails violations even when local hooks are bypassed. | D-9_l3_frozen_surfaces.md:183-187 |
| D9-E32 | [E] | CON,SEC | The decision record justifies structural enforcement, the 40-character floor, and using both dev-time and runtime enforcement. | D-9_l3_frozen_surfaces.md:189-199 |
| D9-E33 | [E] | FAIL,SEC,CON | Registry parse error fails closed; override text shorter than 40 characters blocks; all path/symbol/TOML layers are mandatory; clean or valid override alone exits 0. No retry is defined. | D-9_l3_frozen_surfaces.md:223-229 |
| D9-E34 | [E] | OBS,TEST | An unauthorized tripwire edit must produce one LockedKeysTouch, exit 1, and block the commit/runtime write; registry and CI artifacts must parse/exist. | D-9_l3_frozen_surfaces.md:230-235 |
| D9-E35 | [E] | TEST,OBS | Cloud smokes cover unauthorized pre-commit block/event, valid override allow plus L3OverrideRequested, and runtime Tool::Write to bai.rs blocked before execution. | D-9_l3_frozen_surfaces.md:276-298 |
| D9-E36 | [E] | DEP | Existing C-21 hook/script and all Wave-1 files are consumed; if no GitDiff/git2 support exists, shell out to git rather than adding an unapproved dependency. | D-9_l3_frozen_surfaces.md:212-218 |
| D9-E37 | [E] | CON,DEP | D-25 combined tamper cases are out of scope; registry signing and multi-party override approval are deferred, relying on git signing and one operator in the current threat model. | D-9_l3_frozen_surfaces.md:319-331 |
| D9-E38 | [E] | DEP,TEST | Before implementation, ingest and verify the Everitt reward-tampering, Hubinger mesa-optimization, and Cohen Boxed Myopic paper interfaces; acceptance requires all three artifacts. | D-9_l3_frozen_surfaces.md:64-73,270-273,297 |
| D9-E39 | [E] | TEST | Completion also requires public schema 55, both SSB round trips, at least 14 registry entries covering all surface classes, the hook chain, CI, documentation, a clean workspace build, and at least eight locked-key tests. | D-9_l3_frozen_surfaces.md:237-289 |

#### D-9 unresolved records

| ID | Class | Category | Record | Anchor |
|---|---|---|---|---|
| D9-N01 | [A] | DEP | The original has no implementation status label; PLANNED is orientation-only. | D-9_l3_frozen_surfaces.md:1-331 |
| D9-N02 | [C] | SEC,FLOW | A pre-commit hook is required to parse the commit-message body, but pre-commit normally executes before the final commit message exists; no commit-msg or prepare-commit-msg hook is specified. | D-9_l3_frozen_surfaces.md:108-116,146-159 |
| D9-N03 | [C] | SEC | The objective and summary say git push is blocked, while concrete mechanisms block commit, runtime write, or CI; no pre-push enforcement is defined. | D-9_l3_frozen_surfaces.md:3-8,108-134,183-187 |
| D9-N04 | [A] | OBS | Overridden diff tests allow either zero touch events or one override_present=true event, leaving the exact audit trail nondeterministic at specification level. | D-9_l3_frozen_surfaces.md:168-174 |
| D9-N05 | [C] | SEC | The override event is called non-permission-granting, yet a valid override is the condition that permits the commit; the boundary between documentation and authorization is semantic rather than operationally defined. | D-9_l3_frozen_surfaces.md:140-159 |
| D9-N06 | [A] | FAIL,SEC | Fail closed on an unreadable registry says treat it as locked but does not define whether all writes/commits are blocked or only known protected paths. | D-9_l3_frozen_surfaces.md:223-225 |
| D9-N07 | [A] | SEC,FLOW | The source of a valid human override during runtime Tool::Write and whether runtime writes can ever be overridden are unspecified. | D-9_l3_frozen_surfaces.md:122-134,146-159 |
| D9-N08 | [A] | DATA,SEC | Only example registry entries are shown; the exact canonical path/symbol/key mapping for each of the 14 surface classes remains implementation work. | D-9_l3_frozen_surfaces.md:75-106 |
| D9-N09 | [C] | CFG,SEC | The contextual locked-key names (`rho_delta`, `iqr_mult`, `ppl_sigma`, plural `ddm_thresholds`) differ from the registry example (`rho_delta_threshold`, `iqr_outlier_multiplier`, `ppl_anomaly_sigma`, separate warning/drift keys); the canonical names are unresolved. | D-9_l3_frozen_surfaces.md:31-35,97-104 |
| D9-N10 | [A] | TEST,SEC | The deterministic gate checks only at least 14 registry records, but one conceptual surface may require several path/symbol/key records; that count alone cannot prove all 14 classes or all per-file specifics are covered. | D-9_l3_frozen_surfaces.md:77-106,246-249,278-281 |
| D9-N11 | [A] | INT,OBS | The shell pre-commit path is required to yield a `LockedKeysTouch` row, but the plan does not define how the hook obtains an SSB writer; only the Rust runtime auditor owns `ssb_tx`. | D-9_l3_frozen_surfaces.md:108-134,230-235 |
| D9-N12 | [A] | DATA,SEC | `audit_diff` returns multiple events but `audit_runtime_write` returns only one optional event; behavior for one pending write touching multiple locked symbols or keys is not defined. | D-9_l3_frozen_surfaces.md:118-127 |
| D9-N13 | [A] | CFG,SEC | Registry examples use human override keys such as `L3_OVERRIDE:tripwire`, while the convention/example uses a registry key such as `tripwire.kl_spec_tau`; the exact key namespace and matching algorithm are unspecified. | D-9_l3_frozen_surfaces.md:77-106,146-159 |

### D-10_mode_routing_predicate.md

Source read: 284/284 lines. The original contains no status label; the orientation summary labels D-10 PLANNED.

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D10-E01 | [E] | CAP,SEC | Assign every novel claim to one of `GroundingMode::{Constructive, FalsifiabilityCrossFamily, OperationalFalsification, DiscoveryLabelOnly, Refuse}`; routing belongs to the orchestrator and accepts no LLM input. | D-10_mode_routing_predicate.md:3-8 |
| D10-E02 | [E] | DEP,DET | Require D-9 DONE; run alongside D-11 and D-12, but not D-16 because UniCR formalizes Mode E. | D-10_mode_routing_predicate.md:9-14 |
| D10-E03 | [E] | FLOW | Mode A (`Constructive`) applies when a claim is operationalizable and an applicable checker exists; the checker supplies ground truth. | D-10_mode_routing_predicate.md:18-23,33-38 |
| D10-E04 | [E] | FLOW,FAIL,SEC | Mode B (`FalsifiabilityCrossFamily`) is a last-resort different-family adversarial probe, reached only after constructive and experimental routes are unavailable, intrinsic signals are not low, and an alternate family exists. | D-10_mode_routing_predicate.md:23-24,33-49,52-54 |
| D10-E05 | [E] | FLOW,CON | Mode C (`DiscoveryLabelOnly`) is not a runtime route; it is a post-hoc label after Mode A or D yields a checker-grounded result. | D-10_mode_routing_predicate.md:25-27,52-54 |
| D10-E06 | [E] | FLOW | Mode D (`OperationalFalsification`) runs a sandbox experiment represented by a typed `ObservableEffect`. | D-10_mode_routing_predicate.md:28-29 |
| D10-E07 | [E] | FLOW,FAIL | Mode E (`Refuse`) is the fallback when there is no constructive checker or runnable experiment and intrinsic confidence is too low, and also when no alternate family exists. | D-10_mode_routing_predicate.md:30-31,42-49 |
| D10-E08 | [E] | FLOW,DET | Predicate precedence is constructive first, runnable experiment second, low intrinsic signal refusal third, alternate-family falsifiability fourth, and refusal last. | D-10_mode_routing_predicate.md:33-49 |
| D10-E09 | [E] | DATA | `Claim` fields are `id`, `claim_type`, `statement: BoundedString<2048>`, `domain`, optional `source_template`, and optional D-3 `intrinsic_signals`. | D-10_mode_routing_predicate.md:77-81 |
| D10-E10 | [E] | DATA,SEC | `ClaimType` is the L3-locked closed enum `Code`, `Math`, `Prose`, `Empirical`, `Definitional`, `Counterfactual`, `EthicalAesthetic`. | D-10_mode_routing_predicate.md:82-83 |
| D10-E11 | [E] | FLOW | `is_operationalizable` returns true for Code or Math and for Empirical with `test_cmd`; it returns false for Prose, Definitional, Counterfactual, and EthicalAesthetic. | D-10_mode_routing_predicate.md:84-86 |
| D10-E12 | [E] | FLOW | `permits_experiment` returns true only for Empirical claims having a sandbox-compatible test. | D-10_mode_routing_predicate.md:87-88 |
| D10-E13 | [E] | DATA,SEC | `GroundingMode` is the L3-locked closed five-variant enum named in the objective. | D-10_mode_routing_predicate.md:90-95 |
| D10-E14 | [E] | DET,SEC,CON | `select_grounding_mode` is pure typed-input logic and must not call an LLM. | D-10_mode_routing_predicate.md:94-96,197-200 |
| D10-E15 | [E] | DATA,DEP | Extend `OrchContext` with a D-11 checker-registry trait stub, the existing C-22 `FeasibilityProber` sandbox, a D-16 `unicr_tau` stub constant, and alternate-family availability. | D-10_mode_routing_predicate.md:98-105 |
| D10-E16 | [E] | CFG,INT | `has_alternate_family_model()` returns true when `ELAI_PPM_ENABLE` or `ELAI_CLOUD_MODE` is set, referencing D-2/B-37. | D-10_mode_routing_predicate.md:100-104 |
| D10-E17 | [E] | DEP,FLOW | Stub implementations deliberately preserve the interface until D-11, D-13, D-16, and D-18 supply concrete checker, adversary, calibration, and observable-effect behavior. | D-10_mode_routing_predicate.md:100-105,187-193 |
| D10-E18 | [E] | FLOW,DEP | `dispatch_by_mode` sends Constructive to `checker_registry.run`; until D-11, that stub returns `Inapplicable`. | D-10_mode_routing_predicate.md:107-112 |
| D10-E19 | [E] | FLOW,DEP | OperationalFalsification invokes `sandbox.run_observable_effect`; until D-18, that stub returns `Inconclusive`. | D-10_mode_routing_predicate.md:107-113 |
| D10-E20 | [E] | FLOW,DEP | FalsifiabilityCrossFamily invokes `adversary_role.probe`; until D-13, that stub returns `NoResult`. | D-10_mode_routing_predicate.md:107-115 |
| D10-E21 | [E] | FLOW,FAIL | Refuse produces `ClaimResult::Refused` with a reason. | D-10_mode_routing_predicate.md:116 |
| D10-E22 | [E] | FLOW,FAIL,CON | Dispatching `DiscoveryLabelOnly` is forbidden as a runtime path and is intended to yield defensive inapplicability/error behavior. | D-10_mode_routing_predicate.md:117-118 |
| D10-E23 | [E] | DATA | Declared `ClaimResult` variants are `Verified`, `Falsified { evidence: BoundedString<2048> }`, `Inconclusive { reason: String }`, and `Refused { reason: String }`. | D-10_mode_routing_predicate.md:119-120 |
| D10-E24 | [E] | INT,DATA,OBS | Bump SSB schema 55 to 56 and add `ModeSelected { claim_id, mode, reason, audit_id }`. | D-10_mode_routing_predicate.md:122-124 |
| D10-E25 | [E] | INT,DATA,OBS | Add `ClaimDispatched { claim_id, mode, result, evidence_snippet }`; both new SSB variants require round-trip tests. | D-10_mode_routing_predicate.md:125-127 |
| D10-E26 | [E] | FLOW,OBS | At every claim-emission site, construct a Claim, select the mode, emit `ModeSelected`, dispatch it, then emit `ClaimDispatched`. | D-10_mode_routing_predicate.md:129-133 |
| D10-E27 | [E] | SEC,CFG | Extend D-9's registry with a locked-symbol entry for `select_grounding_mode` in `grounding_mode.rs`, grounded in the no-self-grading rule. | D-10_mode_routing_predicate.md:135-143 |
| D10-E28 | [E] | SEC,CFG | Add a separate locked-symbol entry for the `ClaimType` closed enum in `claim.rs`. | D-10_mode_routing_predicate.md:144-148 |
| D10-E29 | [E] | TEST,FLOW | Unit tests cover operationalizability and experiment-permission tables, including Code with a test, Math with a SymPy expression, and Empirical without sandbox compatibility. | D-10_mode_routing_predicate.md:150-153 |
| D10-E30 | [E] | TEST,FLOW | Routing tests require Constructive to outrank Mode D, then cover sandbox-only D, low-signal E, alternate-family B, and no-alternate E branches. | D-10_mode_routing_predicate.md:153-160 |
| D10-E31 | [E] | TEST,DATA | The contract requires public schema 56, both SSB payload round trips, and stable serde for the closed `GroundingMode` enum. | D-10_mode_routing_predicate.md:162-165 |
| D10-E32 | [E] | CON,SEC | The decision record must justify orchestrator-owned routing using Cohen/Everitt/Hubinger. | D-10_mode_routing_predicate.md:167-170 |
| D10-E33 | [E] | FAIL,CON | It must justify Refuse rather than Mode B as the fallback, citing a 64.5% same-model blind-spot rate and preferring explicit uncertainty when no alternate family exists. | D-10_mode_routing_predicate.md:171-173 |
| D10-E34 | [E] | FLOW,CON | It must justify why `DiscoveryLabelOnly` is post-hoc annotation rather than a runtime mode. | D-10_mode_routing_predicate.md:174-175 |
| D10-E35 | [E] | FAIL,TEST,OBS | Stub bans require the empty-context Refuse path to remain reachable, all five named mode branches to be tested, and `ClaimDispatched` to be emitted regardless of mode. | D-10_mode_routing_predicate.md:197-203 |
| D10-E36 | [E] | OBS,TEST | The observable exercise runs 50 synthetic claims, ten per named mode, requires 50 `ModeSelected` rows and at least 45/50 labels matching ground truth, described as at least 90% F1. | D-10_mode_routing_predicate.md:203-208 |
| D10-E37 | [E] | TEST | Deterministic/acceptance checks require schema/event/type/lock presence, zero routing-file LLM-call matches, a clean workspace build, contract tests, and at least 12 new unit tests. | D-10_mode_routing_predicate.md:210-249 |
| D10-E38 | [E] | TEST,OBS | Cloud smoke sends ten mixed synthetic claims, checks their hand-labeled distribution, and verifies by grep and execution trace that selection made no LLM call. | D-10_mode_routing_predicate.md:250-255 |
| D10-E39 | [E] | CON,DEP | D-11 checker behavior, D-13 adversary behavior, D-16 UniCR, D-18 observable effects, and D-21 quantitative coverage are explicitly out of scope for this session. | D-10_mode_routing_predicate.md:273-280 |
| D10-E40 | [E] | DEP | D-11 is the next consumer, and D-21 is stated to gate D-10 at at least 0.85 F1. | D-10_mode_routing_predicate.md:281-284 |

#### D-10 unresolved, conflicting, and inferred records

| ID | Class | Category | Record | Anchor |
|---|---|---|---|---|
| D10-N01 | [A] | DEP | The original never labels implementation status; PLANNED is orientation-only and no implementation files were inspected. | D-10_mode_routing_predicate.md:1-284 |
| D10-N02 | [C] | DATA,FLOW | `is_operationalizable` and its tests depend on `test_cmd` and `sympy_expr`, and experiment routing depends on sandbox compatibility, but none is a field of the declared `Claim`. | D-10_mode_routing_predicate.md:77-88,150-153 |
| D10-N03 | [A] | FLOW,FAIL | `intrinsic_signals` is optional, but the predicate's behavior when it is `None` is not defined. | D-10_mode_routing_predicate.md:79-81,42-43 |
| D10-N04 | [C] | DATA,FAIL | The DiscoveryLabelOnly branch says it returns `ClaimResult::Inapplicable`, but the declared `ClaimResult` enum has no `Inapplicable` variant. | D-10_mode_routing_predicate.md:107-120 |
| D10-N05 | [C] | FAIL | The same branch is described as a defensive `Err`, while `dispatch_by_mode` returns bare `ClaimResult`, not `Result<ClaimResult, _>`. | D-10_mode_routing_predicate.md:107-120 |
| D10-N06 | [A] | OBS,FLOW | Task-runner integration says the caller emits `ClaimDispatched`, while the stub ban says `dispatch_by_mode` itself must emit it; ownership is unresolved and a literal implementation could duplicate rows. | D-10_mode_routing_predicate.md:129-133,197-203 |
| D10-N07 | [A] | INT,SEC | Presence of `ELAI_PPM_ENABLE` or `ELAI_CLOUD_MODE` does not by itself prove the available model is a different family from the executor; family identity and runtime health checks are not specified. | D-10_mode_routing_predicate.md:98-105 |
| D10-N08 | [C] | TEST | The plan equates 45 correct classifications out of 50 with at least 90% F1; accuracy and macro/per-class F1 are not interchangeable. | D-10_mode_routing_predicate.md:203-208,250-254 |
| D10-N09 | [C] | TEST,DEP | D-10's observable target says at least 90% F1, while its own Next section makes D-21 authoritative at at least 0.85 macro-F1. | D-10_mode_routing_predicate.md:203-208,281-284; D-21_mode_routing_coverage_benchmark.md:39-49 |
| D10-N10 | [C] | FLOW,TEST | The predicate never returns `DiscoveryLabelOnly`, yet both D-10's 50-claim exercise and D-21's balanced corpus demand ten/forty Mode-C cases respectively. | D-10_mode_routing_predicate.md:33-54,203-208; D-21_mode_routing_coverage_benchmark.md:15-21,31-34 |
| D10-N11 | [A] | DATA,OBS | Reason-string contents, `ClaimResult`-to-string serialization, and construction/truncation of `evidence_snippet` are unspecified. | D-10_mode_routing_predicate.md:119-127 |
| D10-N12 | [A] | FAIL,FLOW | The stubs return Inapplicable, Inconclusive, or NoResult, but the declared result enum cannot represent two of those names and no fallback/re-routing rule is defined for a selected mode whose backend is unavailable. | D-10_mode_routing_predicate.md:98-120 |
| D10-N13 | [A] | CAP,FLOW | “Every claim-emit site” has no enumerated call-site inventory or coverage check, so completeness of routing insertion cannot be established from this plan alone. | D-10_mode_routing_predicate.md:56-61,129-133 |

### D-14_sycophancy_probe.md

Source read: 225/225 lines. The original contains no status label; the orientation summary labels D-14 PLANNED.

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D14-E01 | [E] | CAP,SEC,FLOW | For a user-aligned claim, run an inverted-framing probe; convergence is robust, divergence creates `SYCOPHANCY_FLAG` and human-review quarantine, as a structural rather than prompt-only defense. | D-14_sycophancy_probe.md:3-8 |
| D14-E02 | [E] | DEP,DET | Require D-13 cross-family infrastructure, possibly using its Adversary slot or the same executor; run in parallel with D-15. | D-14_sycophancy_probe.md:9-12 |
| D14-E03 | [E] | FLOW | Trigger when the model agrees with a direct user assertion, then invoke the same model with the same context and an instruction to assume that assertion is wrong and justify the alternative. | D-14_sycophancy_probe.md:16-26 |
| D14-E04 | [E] | FLOW,FAIL | Similar positions across original and inverted responses are Convergent; significant divergence is a sycophancy flag and quarantines the claim. | D-14_sycophancy_probe.md:26-30 |
| D14-E05 | [E] | DET,CFG | Convergence is token-level 5-shingle Jaccard similarity at least 0.7; the threshold is intended for later empirical tuning from ELAI session statistics. | D-14_sycophancy_probe.md:32-34 |
| D14-E06 | [E] | CAP,DATA,CFG | `SycophancyProbe` owns `Arc<dyn RuntimeAdapter>`, `convergence_threshold` defaulting to 0.7, and an SSB sender. | D-14_sycophancy_probe.md:58-62 |
| D14-E07 | [E] | DATA | `InvertedFraming` contains original and inverted `BoundedString<2048>` values plus `claim_id`. | D-14_sycophancy_probe.md:63-64 |
| D14-E08 | [E] | DATA,OBS | `ProbeReport` contains claim ID, Blake3 hashes of both outputs, Jaccard similarity, and `ProbeVerdict`. | D-14_sycophancy_probe.md:65-67 |
| D14-E09 | [E] | DATA,FLOW | `ProbeVerdict` is `Convergent`, `DivergentSycophancyFlag`, or `Inconclusive`. | D-14_sycophancy_probe.md:68 |
| D14-E10 | [E] | FLOW | `invert_framing` detects direct positions with heuristics such as “I think”, “this is”, “X is correct”, and direct assertions, then constructs the specified adversarial prompt. | D-14_sycophancy_probe.md:70-76 |
| D14-E11 | [E] | FAIL,FLOW | With no clear assertion, inversion conservatively mirrors the original with a skip note and yields Inconclusive rather than forcing a probe. | D-14_sycophancy_probe.md:77-81 |
| D14-E12 | [E] | TEST | At least five synthetic user framings must each produce a sensible inversion. | D-14_sycophancy_probe.md:80-81,184-190 |
| D14-E13 | [E] | FLOW,INT | `probe` inverts the message, skips to Inconclusive when marked, otherwise sends the inverted prompt through `RuntimeAdapter` and receives an inverted output. | D-14_sycophancy_probe.md:83-89 |
| D14-E14 | [E] | DET,FLOW | It computes 5-shingle Jaccard between original and inverted output, maps `>= threshold` to Convergent and lower values to `DivergentSycophancyFlag`. | D-14_sycophancy_probe.md:89-92 |
| D14-E15 | [E] | OBS,FLOW | Every executed probe emits a probe-dispatch row, and a divergent verdict additionally emits a sycophancy-flag row. | D-14_sycophancy_probe.md:93,158-167 |
| D14-E16 | [E] | FLOW | The task-runner hook executes only after `dispatch_by_mode` returns `ClaimResult::Verified` and the user message has a direct assertion. | D-14_sycophancy_probe.md:95-97 |
| D14-E17 | [E] | FLOW,SEC | A divergent claim is marked `human_review_pending` and cannot proceed to `MEMORIZING`. | D-14_sycophancy_probe.md:98-100,158-162 |
| D14-E18 | [E] | INT,DATA,OBS | Bump SSB schema 59 to 60 and add `SycophancyProbeDispatched { claim_id, user_assertion_hash, audit_id }`. | D-14_sycophancy_probe.md:101-106 |
| D14-E19 | [E] | INT,DATA,OBS | Add `SycophancyFlag { claim_id, jaccard, threshold, original_hash, inverted_hash }`; both variants require round trips. | D-14_sycophancy_probe.md:101-106 |
| D14-E20 | [E] | CFG,SEC | Add L3-locked `[sycophancy_probe]` config with `enabled=true`, `convergence_threshold=0.7`, `jaccard_ngram_size=5`, and `skip_if_no_assertion=true`. | D-14_sycophancy_probe.md:108-111 |
| D14-E21 | [E] | TEST,FAIL | Unit tests cover assertive and neutral inversions, Jaccard values 1.0/0.0/partial, and the complete verdict table. | D-14_sycophancy_probe.md:113-120 |
| D14-E22 | [E] | TEST,OBS | The divergent integration scenario requires a flag event and quarantine. | D-14_sycophancy_probe.md:122-127 |
| D14-E23 | [E] | TEST,OBS | The independent-agreement scenario requires Convergent and no flag. | D-14_sycophancy_probe.md:128-130 |
| D14-E24 | [E] | CON,SEC | The decision record must justify structural probing because prompt-only mitigation is unreliable. | D-14_sycophancy_probe.md:132-135 |
| D14-E25 | [E] | CON,DET | It must justify bounded 5-shingle Jaccard rather than unbounded edit distance. | D-14_sycophancy_probe.md:136-137 |
| D14-E26 | [E] | CFG,CON | It must justify the default 0.7 threshold as a conservative midpoint of the cited 0.6-0.85 non-sycophantic similarity band. | D-14_sycophancy_probe.md:138-140 |
| D14-E27 | [E] | DEP,INT | The module consumes B-37 `RuntimeAdapter`, D-10 Claim, SSB writing, and the user-message field on `OrchContext`, and is reachable from the task-runner post-dispatch hook plus config. | D-14_sycophancy_probe.md:142-157 |
| D14-E28 | [E] | FAIL,CFG,OBS | Stub bans require a dispatch row regardless of verdict, safe empty-message skipping, config-loaded threshold, and quarantine enforcement at the MEMORIZING transition. | D-14_sycophancy_probe.md:158-162 |
| D14-E29 | [E] | OBS | The divergent observable exercise yields exactly one dispatch row, one flag row, no MEMORIZING transition, an auditor-visible flag, and a quarantine artifact under `workflow/sessions/{id}/sycophancy-quarantine/`. | D-14_sycophancy_probe.md:163-170 |
| D14-E30 | [E] | DEP,TEST | Before implementation, ingest the Sharma sycophancy paper interface; deterministic and acceptance checks require that artifact, schema/types/config presence, a clean workspace build, and at least eight unit tests. | D-14_sycophancy_probe.md:42-54,172-194,201 |
| D14-E31 | [E] | TEST,FAIL | Cloud smokes cover divergent quarantine, independent convergence without a flag, and a neutral no-assertion skip recorded with `note="skipped"`. | D-14_sycophancy_probe.md:195-200 |
| D14-E32 | [E] | CON,DEP | D-23 benchmarking is out of scope; multi-turn detection and LLM-as-judge detection are explicitly deferred, with the structural single-turn approach first. | D-14_sycophancy_probe.md:216-220 |
| D14-E33 | [E] | DEP | D-15 is the next orthogonal output-labeling step, and D-23 later benchmarks this detector. | D-14_sycophancy_probe.md:222-225 |

#### D-14 unresolved, conflicting, and inferred records

| ID | Class | Category | Record | Anchor |
|---|---|---|---|---|
| D14-N01 | [A] | DEP | The original never labels implementation status; PLANNED is orientation-only and no implementation files were inspected. | D-14_sycophancy_probe.md:1-225 |
| D14-N02 | [C] | DATA,FAIL | `InvertedFraming` has no `note` field, yet probe control flow and the neutral smoke test read or emit such a note. | D-14_sycophancy_probe.md:63-64,77-88,199-200 |
| D14-N03 | [C] | OBS,DATA | The neutral smoke expects `SycophancyProbeDispatched` with `note="skipped"`, but that event schema has no note or verdict field. | D-14_sycophancy_probe.md:101-106,199-200 |
| D14-N04 | [C] | FLOW,OBS | The hook excludes neutral/no-assertion messages, while acceptance requires those messages to produce a skipped dispatch row; no alternative invocation site is defined. | D-14_sycophancy_probe.md:95-100,158-160,199-200 |
| D14-N05 | [C] | INT,OBS | Narrative and summary terminology use `SYCOPHANCY_PROBE`, while the normative Rust variant is `SycophancyProbeDispatched`; no alias or SQL identifier mapping is supplied. | D-14_sycophancy_probe.md:3-8,93,101-106,142-149 |
| D14-N06 | [A] | INT,SEC | The context says “same model, same context,” while the prerequisite permits either the D-13 Adversary slot or the same executor; actual adapter/model-family selection is unspecified. | D-14_sycophancy_probe.md:9-11,21-25,36-40,150-154 |
| D14-N07 | [A] | DET,DATA | Tokenization, case/punctuation normalization, duplicate-shingle treatment, and behavior for outputs shorter than five tokens or both empty are not defined. | D-14_sycophancy_probe.md:32-34,89-92,113-120 |
| D14-N08 | [A] | FAIL,INT | Timeout, provider failure, cancellation, retry, rate-limit, and partial-output behavior for the inverted runtime call are not defined; `SycophancyError` is named but not enumerated. | D-14_sycophancy_probe.md:83-93 |
| D14-N09 | [A] | FLOW,SEC | Human-review ownership, release/reject criteria, retention, and resumption after quarantine are unspecified. | D-14_sycophancy_probe.md:98-100,163-170 |
| D14-N10 | [A] | CFG,FLOW | `enabled` and `skip_if_no_assertion` defaults are declared, but disabled-mode behavior and any override semantics are not. | D-14_sycophancy_probe.md:108-111 |
| D14-N11 | [A] | SEC,CFG | The config is declared L3-locked, but this plan does not add its keys to D-9's registry or define an override key. | D-14_sycophancy_probe.md:108-111,204-214 |
| D14-N12 | [A] | OBS | A convergent or inconclusive result has no result-bearing SSB row; the dispatch event proves invocation but does not preserve verdict, similarity, or skip reason. | D-14_sycophancy_probe.md:65-69,93,101-106 |
| D14-N13 | [A] | DET,SEC | Lexical Jaccard convergence is treated as semantic robustness, but paraphrase and shared-boilerplate handling are not specified; this is a design assumption rather than a proved equivalence. | D-14_sycophancy_probe.md:26-34,89-92 |
| D14-N14 | [C] | OBS,DEP | The observable contract requires an auditor-agent update, but that agent file is absent from the declared files-touched list. | D-14_sycophancy_probe.md:163-170,204-214 |

### D-21_mode_routing_coverage_benchmark.md

Source read: 122/122 lines. The original contains no status label; the orientation summary labels D-21 PLANNED.

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D21-E01 | [E] | CAP,TEST | Benchmark D-10 with 200 hand-labeled synthetic claims across A/B/C/D/E, report F1 and per-mode confusion, and gate macro-F1 at 0.85 without class collapse. | D-21_mode_routing_coverage_benchmark.md:3-5 |
| D21-E02 | [E] | DEP,DET | Require D-10 and declare parallel safety with D-19 through D-25. | D-21_mode_routing_coverage_benchmark.md:6-8 |
| D21-E03 | [E] | DATA,TEST | Author exactly 40 examples per mode, each with a canonical expected mode and rationale. | D-21_mode_routing_coverage_benchmark.md:15-17 |
| D21-E04 | [E] | TEST | Coverage must include clear compile-check Mode A, sandbox Mode D, ethical/aesthetic Mode E, ambiguous A-versus-D, and no-alternate-family Refuse edge cases. | D-21_mode_routing_coverage_benchmark.md:18-21 |
| D21-E05 | [E] | DATA | Store 200 JSON fixture files under `.elai_cc/benchmarks/mode_routing/fixtures/`, with `claim_id`, `statement`, `claim_type`, `domain`, `intrinsic_signals`, `expected_mode`, and `rationale`. | D-21_mode_routing_coverage_benchmark.md:29-34 |
| D21-E06 | [E] | CAP,DATA | `ModeRoutingBenchmark` contains `fixtures_dir`; `ModeRoutingReport` contains total, correct, `(expected,predicted)` confusion counts, macro-F1, per-mode F1, and `GatingVerdict`. | D-21_mode_routing_coverage_benchmark.md:36-41 |
| D21-E07 | [E] | FLOW,DET | `run` loads every fixture, builds a mock `OrchContext`, calls `select_grounding_mode`, compares expected versus actual, and computes macro/per-mode F1 plus confusion matrix. | D-21_mode_routing_coverage_benchmark.md:42-45 |
| D21-E08 | [E] | TEST,FAIL | Pass requires macro-F1 at least 0.85 and every mode's F1 at least 0.70; otherwise return `FailBelowBaseline`. | D-21_mode_routing_coverage_benchmark.md:47-49 |
| D21-E09 | [E] | CAP | Expose CLI `elai bench mode-routing` from `bench_mode_routing.rs`. | D-21_mode_routing_coverage_benchmark.md:51-53 |
| D21-E10 | [E] | CON | The decision record justifies 40 examples per mode as an approximately ±5% noise floor balanced against authoring cost. | D-21_mode_routing_coverage_benchmark.md:54-56 |
| D21-E11 | [E] | TEST,CON | It justifies macro-F1 0.85 for system failure and per-mode 0.70 for class-collapse protection. | D-21_mode_routing_coverage_benchmark.md:57-58 |
| D21-E12 | [E] | SEC,CON | It requires human-vetted labels rather than another LLM's output to avoid circular ground truth. | D-21_mode_routing_coverage_benchmark.md:59-60 |
| D21-E13 | [E] | DEP,INT | The benchmark consumes D-10 selector/mode types and D-19 `BenchmarkStarted`/`BenchmarkResult` SSB support. | D-21_mode_routing_coverage_benchmark.md:62-71 |
| D21-E14 | [E] | CAP,INT | The benchmark is reachable from CLI and a CI workflow. | D-21_mode_routing_coverage_benchmark.md:72-73 |
| D21-E15 | [E] | SEC,TEST | Fixtures must be authored from scratch, not generated, and signed off by a human reviewer. | D-21_mode_routing_coverage_benchmark.md:74-78 |
| D21-E16 | [E] | OBS | Each CLI run emits `BenchmarkStarted` and `BenchmarkResult` and writes `.elai_cc/benchmarks/mode_routing/reports/{date}.json`. | D-21_mode_routing_coverage_benchmark.md:79-82 |
| D21-E17 | [E] | TEST | Deterministic checks require the runner/run symbols, at least 200 JSON fixtures, a clean workspace build, and scoped benchmark tests. | D-21_mode_routing_coverage_benchmark.md:84-92 |
| D21-E18 | [E] | TEST,FAIL | Acceptance requires the benchmark to compile, all 200 balanced hand labels to exist, and the CLI to exit 0 when the gate is met. | D-21_mode_routing_coverage_benchmark.md:94-99 |
| D21-E19 | [E] | TEST,OBS | The live 200-fixture smoke must reach macro-F1 at least 0.85 and every per-mode F1 at least 0.70 with no collapse. | D-21_mode_routing_coverage_benchmark.md:100-101 |
| D21-E20 | [E] | DET,TEST | Two runs over identical mocked contexts must produce the same confusion matrix. | D-21_mode_routing_coverage_benchmark.md:102-103 |
| D21-E21 | [E] | OBS,FLOW | Confusion output must expose edge cases for subsequent D-10 refinement. | D-21_mode_routing_coverage_benchmark.md:104 |
| D21-E22 | [E] | CON,DEP | A D-21 decision document is required; larger fixture sets and LLM-generated fixtures are deferred, the latter due to circularity risk. | D-21_mode_routing_coverage_benchmark.md:105-118 |
| D21-E23 | [E] | DEP | D-22 conformal-calibration validation follows this session. | D-21_mode_routing_coverage_benchmark.md:120-122 |

#### D-21 unresolved, conflicting, and inferred records

| ID | Class | Category | Record | Anchor |
|---|---|---|---|---|
| D21-N01 | [A] | DEP | The original never labels implementation status; PLANNED is orientation-only and no implementation files or fixtures were inspected. | D-21_mode_routing_coverage_benchmark.md:1-122 |
| D21-N02 | [C] | FLOW,TEST | Forty fixtures require expected Mode C, but D-10 defines C as post-hoc label-only and its selector has no branch that returns it. | D-21_mode_routing_coverage_benchmark.md:3-5,15-21,31-34; D-10_mode_routing_predicate.md:25-27,33-54 |
| D21-N03 | [A] | DATA,TEST | The fixture schema omits checker applicability, sandbox capability, alternate-family availability, and experiment/test metadata needed to construct the mocked context and reproduce D-10 routes. | D-21_mode_routing_coverage_benchmark.md:31-45; D-10_mode_routing_predicate.md:33-49,77-105 |
| D21-N04 | [A] | TEST,DET | Precision/recall formulas, macro averaging, zero-denominator policy, and treatment of Mode C's unreachable predictions are not specified. | D-21_mode_routing_coverage_benchmark.md:36-49 |
| D21-N05 | [A] | DATA,FAIL | `GatingVerdict` and its Pass variant/source are not defined, while only `FailBelowBaseline` is named. | D-21_mode_routing_coverage_benchmark.md:36-49 |
| D21-N06 | [A] | FAIL,TEST | Acceptance ties CLI exit 0 only to macro-F1, while normative gate logic additionally requires every per-mode F1 at least 0.70; the exit status when macro passes but one class fails is not restated. | D-21_mode_routing_coverage_benchmark.md:47-49,94-101 |
| D21-N07 | [C] | DEP,DET | The session is declared parallel-safe with D-19 yet consumes benchmark SSB types “from D-19”; build sequencing or a pre-existing shared definition is not supplied. | D-21_mode_routing_coverage_benchmark.md:7-8,62-73 |
| D21-N08 | [A] | SEC,OBS | Human sign-off is mandatory, but reviewer identity, signature format, approval artifact, and enforcement check are unspecified. | D-21_mode_routing_coverage_benchmark.md:31-34,59-60,74-78 |
| D21-N09 | [A] | FAIL | Missing, malformed, duplicate, imbalanced, or extra fixtures and partial report-write failures have no defined error/fallback behavior. | D-21_mode_routing_coverage_benchmark.md:31-49,79-82 |
| D21-N10 | [A] | DET,OBS | `{date}` timezone, same-day overwrite/versioning, stable map serialization, and byte-for-byte report determinism are unspecified even though result determinism is an acceptance gate. | D-21_mode_routing_coverage_benchmark.md:39-41,79-82,102-103 |

### D-24_cross_domain_canary_regression_suite.md

Source read: 101/101 lines. The original contains no status label; the orientation summary labels D-24 PLANNED.

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D24-E01 | [E] | CAP,TEST | Validate D-7 with 100 synthetic mutations across five task families: 50 helpful, 30 neutral, and 20 harmful. | D-24_cross_domain_canary_regression_suite.md:3-5 |
| D24-E02 | [E] | DEP,DET | Require D-7 and declare parallel safety with D-19 through D-25. | D-24_cross_domain_canary_regression_suite.md:6-8 |
| D24-E03 | [E] | DATA,TEST | Helpful mutations improve the target family by 5-15% and regress no non-target family. | D-24_cross_domain_canary_regression_suite.md:14-15 |
| D24-E04 | [E] | DATA,TEST | Neutral mutations leave the target within ±2% and regress no non-target family. | D-24_cross_domain_canary_regression_suite.md:16 |
| D24-E05 | [E] | DATA,TEST | Harmful mutations leave the target unchanged or improved but regress at least one non-target family by more than 5%. | D-24_cross_domain_canary_regression_suite.md:17-18 |
| D24-E06 | [E] | DATA | Store 100 TOML fixtures under `.elai_cc/benchmarks/canary/fixtures/`, each with mutation ID, target family, baseline/candidate pass rates by family, and expected verdict. | D-24_cross_domain_canary_regression_suite.md:26-32 |
| D24-E07 | [E] | CAP,FLOW | `CanaryBenchmark` invokes D-7 `CanaryRunner` for each fixture and compares the result with `expected_verdict`. | D-24_cross_domain_canary_regression_suite.md:34-37 |
| D24-E08 | [E] | DATA,OBS | Its report contains total, true_block, false_block, true_pass, and false_pass. | D-24_cross_domain_canary_regression_suite.md:34-37 |
| D24-E09 | [E] | TEST,FAIL | Hard gates are 20/20 harmful blocked, no more than 5/50 helpful blocked, and no more than 2/30 neutral blocked. | D-24_cross_domain_canary_regression_suite.md:39-40 |
| D24-E10 | [E] | CAP | Expose CLI `elai bench canary-regression`. | D-24_cross_domain_canary_regression_suite.md:42 |
| D24-E11 | [E] | CON | The decision record justifies 100 fixtures as sufficient for the intended sample-noise floor. | D-24_cross_domain_canary_regression_suite.md:44-45 |
| D24-E12 | [E] | CON | It justifies the 50/30/20 split because harmful mutations are the minority and over-sampling would distort evaluation. | D-24_cross_domain_canary_regression_suite.md:46 |
| D24-E13 | [E] | TEST,CON | It justifies a 10% helpful false-block allowance so D-7's regression threshold can be tuned without rebuilding canary infrastructure. | D-24_cross_domain_canary_regression_suite.md:47-48 |
| D24-E14 | [E] | DEP,INT | The benchmark consumes D-7 `CanaryRunner` and SSB writing and is reachable from CLI plus CI. | D-24_cross_domain_canary_regression_suite.md:50-59 |
| D24-E15 | [E] | TEST,SEC | Blocking all 20 harmful fixtures is an unrelaxed hard gate. | D-24_cross_domain_canary_regression_suite.md:60-61 |
| D24-E16 | [E] | OBS | A CLI run emits `BenchmarkStarted` plus `BenchmarkResult` and writes a report beneath `.elai_cc/benchmarks/canary/reports/`. | D-24_cross_domain_canary_regression_suite.md:62-65 |
| D24-E17 | [E] | TEST | Deterministic checks require exactly/at least 100 fixture files, the benchmark struct, and a clean workspace build. | D-24_cross_domain_canary_regression_suite.md:67-74 |
| D24-E18 | [E] | TEST,FAIL | Acceptance requires compilation, 100 fixtures, CLI exit 0 only when gates are met, and the clean workspace build. | D-24_cross_domain_canary_regression_suite.md:76-81 |
| D24-E19 | [E] | TEST | Three cloud smokes separately prove 20/20 harmful blocking, helpful false blocks at most 5/50, and neutral blocks at most 2/30. | D-24_cross_domain_canary_regression_suite.md:82-85 |
| D24-E20 | [E] | CON,DEP | Real-data mutations are deferred; the plan treats synthetic fixtures as sufficient for the D-7 gate. | D-24_cross_domain_canary_regression_suite.md:95-101 |

#### D-24 unresolved, conflicting, and inferred records

| ID | Class | Category | Record | Anchor |
|---|---|---|---|---|
| D24-N01 | [A] | DEP | The original never labels implementation status; PLANNED is orientation-only and no implementation files or fixtures were inspected. | D-24_cross_domain_canary_regression_suite.md:1-101 |
| D24-N02 | [A] | DATA,TEST | “Across 5 task families” has no per-family quota or cross-product rule, so all 100 fixtures could technically concentrate in one family while satisfying the stated type counts. | D-24_cross_domain_canary_regression_suite.md:3-5,28-32 |
| D24-N03 | [A] | TEST | Helpful improvement exactly +5%, neutral exactly ±2%, and non-target regression exactly 5% have only partly explicit boundary semantics; the harmful definition is strictly greater than 5%. | D-24_cross_domain_canary_regression_suite.md:14-18 |
| D24-N04 | [A] | DATA,FAIL | Rate ranges, family completeness, numeric precision, missing families, malformed TOML, duplicate IDs, and invalid expected verdicts have no validation/error policy. | D-24_cross_domain_canary_regression_suite.md:28-37 |
| D24-N05 | [A] | DATA,OBS | The report has only binary pass/block counters and no per-family/type breakdown or explicit overall verdict field, limiting diagnosis of where thresholds fail. | D-24_cross_domain_canary_regression_suite.md:34-40 |
| D24-N06 | [A] | INT,OBS | Generic benchmark SSB event payloads and their schema/version source are not defined in this plan. | D-24_cross_domain_canary_regression_suite.md:50-65 |
| D24-N07 | [A] | DET | Fixture ordering, random seed (if any), repeated-run equivalence, report naming, and overwrite/concurrency behavior are unspecified. | D-24_cross_domain_canary_regression_suite.md:26-42,62-74 |
| D24-N08 | [A] | TEST | No scoped unit/integration test command is included; deterministic verification checks only fixture count, symbol presence, and workspace compilation before cloud smokes. | D-24_cross_domain_canary_regression_suite.md:67-85 |

### D-28_adversarial_example_farming.md

Source read: 156/156 lines. This original explicitly says `Status: DEFERRED`, but its later shipping-pass annotations explicitly claim the farming CLI and SSB legs are live while RunPod fine-tuning remains pending.

| ID | Class | Category | Atomic feature or behavior | Anchor |
|---|---|---|---|---|
| D28-E01 | [E] | CAP,DEP | Deferred research-grade work iteratively generates OOD high-proxy/low-quality PPM examples and retrains D-2's PPM for gaming resistance; estimate 4-6 sessions plus RunPod hours. | D-28_adversarial_example_farming.md:1-8 |
| D28-E02 | [E] | DEP,DET | Unblock only after D-5 has at least 20 stable promotion cycles, D-19 publishes a PRMBench baseline, and a RunPod 4090 or A100 is available; it may run beside D-29 and D-30. | D-28_adversarial_example_farming.md:9-11,129-132 |
| D28-E03 | [E] | FLOW,TEST | Generate candidate reasoning steps that score high on the current PPM but fail deterministic gold checks such as compile, tests, or formal consistency. | D-28_adversarial_example_farming.md:13-20 |
| D28-E04 | [E] | FLOW,DATA | Mine the proxy-versus-gold gap and assign negative labels to the resulting adversarial corpus. | D-28_adversarial_example_farming.md:20-21 |
| D28-E05 | [E] | FLOW,INT | Retrain the PPM on that corpus, then remeasure PRMBench Deception Resistance and Circular Logic with a target improvement of at least five percentage points. | D-28_adversarial_example_farming.md:21-28 |
| D28-E06 | [E] | DEP | Bootstrapping is explicitly blocked by stable BAI selection, a published baseline, and cost-throttled GPU fine-tuning. | D-28_adversarial_example_farming.md:25-28 |
| D28-E07 | [E] | DEP,TEST | Ingest and verify `workflow/papers/adversarial-rm-2504.06141-interface.md` before unblocking. | D-28_adversarial_example_farming.md:30-44 |
| D28-E08 | [E] | CAP,FLOW | `ppm_adversarial.py` samples a step, computes proxy and gold rewards, and retains it when proxy is much greater than gold and gold equals Fail. | D-28_adversarial_example_farming.md:46-54 |
| D28-E09 | [E] | DATA | The task-level corpus destination is `.elai_cc/calibration/ppm_adversarial/{ts}/{step_id}.jsonl`. | D-28_adversarial_example_farming.md:48-55 |
| D28-E10 | [E] | INT,CFG | `ppm_finetune.sh` runs on a RunPod 4090/A100 and loads `Qwen2.5-Math-PRM-7B`. | D-28_adversarial_example_farming.md:56-59 |
| D28-E11 | [E] | CFG | Fine-tuning uses negative-labeled adversarial examples with LoRA rank 16, three epochs, and learning rate `1e-5`. | D-28_adversarial_example_farming.md:58-60 |
| D28-E12 | [E] | DATA,INT | Fine-tuning outputs `ppm_v2.safetensors` and deploys it to the remote sidecar. | D-28_adversarial_example_farming.md:60 |
| D28-E13 | [E] | FLOW,OBS | Run D-19 on v1 and save its report, deploy v2, run D-19 again, save that report, then compare category deltas. | D-28_adversarial_example_farming.md:62-65 |
| D28-E14 | [E] | TEST,FAIL | Every standard PRMBench category must avoid regression greater than two percentage points. | D-28_adversarial_example_farming.md:67-70 |
| D28-E15 | [E] | TEST | Deception Resistance and Circular Logic must each improve by at least five percentage points. | D-28_adversarial_example_farming.md:67-70,129-135 |
| D28-E16 | [E] | FAIL | If standard categories regress or adversarial gains miss the gate, revert to v1, log the corpus and decision, and do not deploy v2. | D-28_adversarial_example_farming.md:72-75 |
| D28-E17 | [E] | INT,DATA,OBS | The task originally proposes, with an appropriate future schema bump, `AdversarialCorpusGenerated { count, source_session }`. | D-28_adversarial_example_farming.md:77-80 |
| D28-E18 | [E] | INT,DATA,OBS | It also proposes `PpmFinetuneCompleted { v1_score, v2_score, delta, verdict }`. | D-28_adversarial_example_farming.md:77-80 |
| D28-E19 | [E] | CON,DEP | Draft the D-28 decision document only at unblock time, explaining deferral, conservative LoRA rather than full tuning, and the literature-grounded five-point gain gate. | D-28_adversarial_example_farming.md:82-89 |
| D28-E20 | [E] | CAP,DATA | Shipping-pass annotations name live `AdvRmFarmer`, `AdvRmFarmer::run`, `AdvRmFarmRunReport`, `AdvRmSeed`, `AdvRmCorpusEntry`, and `verdict -> GatingVerdict` with implementation anchors. | D-28_adversarial_example_farming.md:91-97 |
| D28-E21 | [E] | INT,OBS | Those annotations explicitly reject both originally proposed event variants and identify the actual shipped event/payload as `AdversarialFarmRunCompleted`. | D-28_adversarial_example_farming.md:97-101 |
| D28-E22 | [E] | DEP,INT | The Python producer exists according to the plan; the fine-tune script is `RUNPOD-PENDING`. | D-28_adversarial_example_farming.md:100-101 |
| D28-E23 | [E] | DEP | The shipping contract consumes at least 20 D-5 BAI cycles, D-19 `PRMBenchRunner`, `SidecarPpmHook`, and D-5 `darwinian_bai`. | D-28_adversarial_example_farming.md:102-106 |
| D28-E24 | [E] | CAP | Live reachability is asserted through `elai farm-adv-rm run` and `elai farm-adv-rm verdict`. | D-28_adversarial_example_farming.md:107-110 |
| D28-E25 | [E] | INT,DEP | Live `ppm_adversarial.py --live` and the GPU fine-tune/post-immunization legs remain RunPod- and prerequisite-pending. | D-28_adversarial_example_farming.md:110,119-121 |
| D28-E26 | [E] | OBS,TEST | Every farming run must emit `AdversarialFarmRunCompleted`; `verdict` must call the real `report.passed()` rather than a mock. | D-28_adversarial_example_farming.md:111-114 |
| D28-E27 | [E] | SEC,CON | Production corpus generation may not use `ppm_adversarial.py --mock`. | D-28_adversarial_example_farming.md:114 |
| D28-E28 | [E] | OBS | `elai farm-adv-rm run --dataset <prmbench_seed_path> --corpus-dir .elai_cc/benchmarks/adv_rm/corpus/` must emit one completion row with `samples_farmed > 0` and populate `AdvRmCorpusEntry` JSONL. | D-28_adversarial_example_farming.md:115-121 |
| D28-E29 | [E] | DEP,OBS | The source explicitly records the CLI-run and SSB-emission legs as wired and specifically claims `farm_adv_rm.rs` plus `ssb.rs:6710` are live. | D-28_adversarial_example_farming.md:119-121 |
| D28-E30 | [E] | TEST,DEP | The deterministic-check section remains deferred to unblock time; acceptance is gated on prerequisites, GPU access, paper output, v2 deployment/report, both numeric gates, and the decision document. | D-28_adversarial_example_farming.md:123-136 |
| D28-E31 | [E] | INT,DEP | Anticipated artifacts include the Python generator, RunPod fine-tune script, timestamped calibration corpus, v2 sidecar-startup update, decision document, and research interface. | D-28_adversarial_example_farming.md:138-146 |
| D28-E32 | [E] | CON | Full PPM rebuilding and multi-model adversarial corpora are deferred beyond D-28; LoRA and a single-model corpus define the planned scope. | D-28_adversarial_example_farming.md:148-151 |
| D28-E33 | [E] | DEP,DET | Once unblocked, D-29 may run independently in a different RunPod session. | D-28_adversarial_example_farming.md:153-156 |

#### D-28 unresolved, conflicting, and inferred records

| ID | Class | Category | Record | Anchor |
|---|---|---|---|---|
| D28-N01 | [C] | DEP | Header/status and acceptance mark D-28 deferred, while the shipping-pass block asserts that farming CLI/SSB legs are live; the source therefore describes partial implementation, not a single uniform status. | D-28_adversarial_example_farming.md:1-11,91-121,127-136 |
| D28-N02 | [C] | DATA | Planned generator output uses `.elai_cc/calibration/ppm_adversarial/{ts}/`, while the asserted live CLI writes `.elai_cc/benchmarks/adv_rm/corpus/`; canonical corpus location and migration are unresolved. | D-28_adversarial_example_farming.md:48-55,115-121,138-143 |
| D28-N03 | [C] | INT,OBS | Tasks specify two future SSB variants, but the shipping contract explicitly says both were rejected in favor of one completion event; acceptance was not rewritten to state the actual event contract. | D-28_adversarial_example_farming.md:77-80,91-101,127-136 |
| D28-N04 | [A] | INT,DATA | “Appropriate SCHEMA bump” supplies no version, migration rule, public constant, or compatibility test. | D-28_adversarial_example_farming.md:77-80 |
| D28-N05 | [A] | FLOW,TEST | `proxy >> gold` has no numeric margin, search budget, stopping condition, sample count, or balance rule. | D-28_adversarial_example_farming.md:15-23,48-54 |
| D28-N06 | [A] | DEP,TEST | “At least 20 stable cycles” never defines stable, cycle identity, consecutive-window requirements, minimum samples per arm, or who certifies readiness. | D-28_adversarial_example_farming.md:9-10,25-28,102-104,129-130 |
| D28-N07 | [A] | DATA,TEST | The planned `PpmFinetuneCompleted` scalar v1/v2 scores cannot by itself encode both adversarial-category gains plus every standard-category regression gate. | D-28_adversarial_example_farming.md:62-80 |
| D28-N08 | [A] | DET,TEST | Dataset version, split, random seeds, sample ordering, optimizer determinism, repeated-run tolerance, and hardware-dependent reproducibility are unspecified. | D-28_adversarial_example_farming.md:46-70,123-136 |
| D28-N09 | [A] | DET,FLOW | Farming idempotency, duplicate-example handling, concurrent writers, resume after interruption, and corpus atomicity are unspecified. | D-28_adversarial_example_farming.md:46-55,115-121 |
| D28-N10 | [A] | SEC,INT | Model/corpus provenance, safetensors checksum/signature, remote transport, sidecar health validation, and deployment authorization are not defined. | D-28_adversarial_example_farming.md:56-65,138-146 |
| D28-N11 | [A] | FAIL | Rollback says revert to v1 but does not define the atomic version pointer, rollback command, archive/event evidence, or recovery from a partially deployed v2. | D-28_adversarial_example_farming.md:72-75,140-145 |
| D28-N12 | [A] | SEC | Corpus privacy, secret/PII scrubbing, retention, and whether production reasoning may be exported to RunPod are not addressed. | D-28_adversarial_example_farming.md:46-60 |
| D28-N13 | [A] | CFG,DEP | GPU use is called cost-throttled, but no budget, timeout, instance-hours limit, or approval gate is specified. | D-28_adversarial_example_farming.md:7,25-28,56-60 |
| D28-N14 | [I] | DEP | Joining the explicit shipping notes yields a two-part state: local farming/audit substrate is claimed live; immunization training, deployment, and post-benchmark proof remain deferred. This is synthesis, not independent implementation verification. | D-28_adversarial_example_farming.md:91-136 |

## Cross-source interactions

These are derivative joins over the per-source atomic records above. They have unique IDs for auditability but are not counted again as source features.

| ID | Class | Category | Interaction | Evidence anchors |
|---|---|---|---|---|
| X-E01 | [E] | DEP,CAP | D-1 creates `ArchiveChain.verify_chain`; D-8 is explicitly its CLI/CI reporting wrapper and can begin after D-1. | D-1_reasoning_template_schema.md:133-140,379-395; D-8_crypto_archive_verify.md:3-12,130-145 |
| X-C01 | [C] | DATA,SEC | The shared archive hash contract is inconsistent: D-1 hashes body plus parent hash, while D-8 additionally hashes `mutation_proposal_text`, a field absent from D-1's schema. | D-1_reasoning_template_schema.md:91-95,114-128; D-8_crypto_archive_verify.md:16-28 |
| X-C02 | [C] | DEP,FAIL | D-8 expects aggregated duplicate/malformed/link errors from D-1 verification, but D-1's declared error/API surface does not provide them. | D-1_reasoning_template_schema.md:110-119,133-137; D-8_crypto_archive_verify.md:52-69,94-106 |
| X-E02 | [E] | SEC,DEP | D-9 freezes D-1/D-8 archive maintenance, parent selection, Blake3 choice, template verifier requirements, and schema enums only after all Wave-1 surfaces exist. | D-9_l3_frozen_surfaces.md:3-15,19-35,75-106 |
| X-E03 | [E] | SEC,FLOW | D-10 relies on D-9 to lock both its routing predicate and `ClaimType`, preserving orchestrator ownership of grading. | D-10_mode_routing_predicate.md:9-14,135-148,187-200 |
| X-C03 | [C] | SEC,INT | D-9's stated “git push BLOCK” outcome is not backed by a pre-push mechanism; the concrete layers are pre-commit, runtime pre-write, and CI. | D-9_l3_frozen_surfaces.md:3-8,42-46,108-134,183-187 |
| X-E04 | [E] | FLOW,DEP | D-14 consumes D-10 `Claim`/`ClaimResult` and runs only after a Verified result with a direct user assertion; it blocks later MEMORIZING on divergence. | D-14_sycophancy_probe.md:42-47,83-100,150-170 |
| X-C04 | [C] | FLOW,TEST | D-21's balanced five-class benchmark cannot be satisfied as written because D-10 calls Mode C label-only and never returns it from `select_grounding_mode`. | D-10_mode_routing_predicate.md:25-27,33-54,90-96; D-21_mode_routing_coverage_benchmark.md:3-5,15-21,31-45 |
| X-C05 | [C] | TEST | D-10 mixes 45/50 accuracy with “90% F1”; D-21 instead specifies macro-F1 at least 0.85 and each per-mode F1 at least 0.70. | D-10_mode_routing_predicate.md:203-208,250-254,281-284; D-21_mode_routing_coverage_benchmark.md:36-49,94-104 |
| X-C06 | [C] | DEP,DET | D-21 claims parallel safety with D-19 while consuming benchmark events attributed to D-19, leaving the event-definition build order unresolved. | D-21_mode_routing_coverage_benchmark.md:6-8,62-73 |
| X-E05 | [E] | DEP,TEST | D-24 is the quantitative D-7 consumer: it feeds typed synthetic pass-rate deltas into `CanaryRunner` and gates its harmful/helpful/neutral discrimination. | D-24_cross_domain_canary_regression_suite.md:3-18,26-42,50-65 |
| X-E06 | [E] | DEP,INT | D-28 consumes the D-2 PPM/sidecar, waits for D-5 stable cycles, and uses D-19 pre/post PRMBench reports to decide v2 deployment. | D-28_adversarial_example_farming.md:3-10,25-28,56-75,102-121 |
| X-C07 | [C] | DEP | D-28 is explicitly DEFERRED but also explicitly claims live local farming CLI and SSB legs; status must be tracked per leg. | D-28_adversarial_example_farming.md:1-11,91-136 |
| X-C08 | [C] | DATA,INT | D-28 retains an older calibration-directory/two-event design beside an asserted live benchmark-directory/one-event implementation contract. | D-28_adversarial_example_farming.md:48-55,77-101,115-121,138-143 |
| X-I01 | [I] | INT,DATA | Across assigned plans, SSB versions progress 47→48 (D-1), 53→54 (D-8), 54→55 (D-9), 55→56 (D-10), and 59→60 (D-14); intervening versions belong to unassigned sessions and were intentionally not read. | D-1_reasoning_template_schema.md:142-152; D-8_crypto_archive_verify.md:79-82; D-9_l3_frozen_surfaces.md:136-144; D-10_mode_routing_predicate.md:122-127; D-14_sycophancy_probe.md:101-106 |
| X-I02 | [I] | OBS,DATA | `TemplateId`, archive hashes, `ClaimId`, and audit UUIDs provide intended correlation keys across archive, routing, and sycophancy events, but no assigned source defines a single cross-event trace/correlation schema. | D-1_reasoning_template_schema.md:91-104,142-152; D-8_crypto_archive_verify.md:79-82; D-10_mode_routing_predicate.md:122-127; D-14_sycophancy_probe.md:101-106 |
| X-I03 | [I] | FAIL,SEC | The combined fail-closed path is archive tamper→nonzero verify, locked write→block, empty routing context→refuse, sycophancy divergence→quarantine, canary harm→block, and failed PPM immunization→retain v1. | D-8_crypto_archive_verify.md:62-69,94-106; D-9_l3_frozen_surfaces.md:223-235; D-10_mode_routing_predicate.md:33-49,197-203; D-14_sycophancy_probe.md:95-100; D-24_cross_domain_canary_regression_suite.md:39-40; D-28_adversarial_example_farming.md:67-75 |
| X-A01 | [A] | FAIL,DET | Retry semantics are absent across archive verification, locked-write auditing, routing backends, sycophancy runtime calls, benchmark report writes, and GPU farming; only explicit short-circuit, quarantine, and rollback behaviors are defined. | D-8_crypto_archive_verify.md:62-69; D-9_l3_frozen_surfaces.md:118-134; D-10_mode_routing_predicate.md:107-120; D-14_sycophancy_probe.md:83-100; D-21_mode_routing_coverage_benchmark.md:42-49; D-24_cross_domain_canary_regression_suite.md:34-42; D-28_adversarial_example_farming.md:46-75 |
| X-A02 | [A] | SEC | None of the eight originals defines PII/secret scrubbing, telemetry export, or retention policy; this is most consequential for archived prompts, user assertions, quarantine artifacts, and D-28 RunPod corpora. | D-1_reasoning_template_schema.md:123-162; D-14_sycophancy_probe.md:63-105,163-170; D-28_adversarial_example_farming.md:46-60 |

## Summary-layer additions and corrections

The orientation summary remains useful for wave ordering, but the following records are the exact additions/corrections contributed by this deep chunk. They do not change that summary file.

### Corrections to summary claims

| ID | Class | Correction | Deep evidence | Orientation claim |
|---|---|---|---|---|
| SUM-C01 | [C] | D-1's exact SSB names are `ArchiveEntryAdded`, `ArchiveEntryRetired`, `TemplatePromoted`, and `TemplateCandidateRejected`; “ARCHIVE_ENTRY_ADDED/RETIRED” is incomplete. | D-1_reasoning_template_schema.md:142-152 | 04-D-SUMMARY_INDEX.discovery.md:29-34,536-539 |
| SUM-C02 | [C] | D-1's `ArchiveEntry` field is `staged_eval_result` and it also includes `retirement_reason`; the summary's `eval_result` projection is not the deep schema. | D-1_reasoning_template_schema.md:123-132 | 04-D-SUMMARY_INDEX.discovery.md:500-501 |
| SUM-C03 | [C] | The D-1 original does not state “at least three typed templates”; it requires migration count at least the template-directory TOML count plus registry/task smokes. Preserve “three” only as summary-source evidence, not deep-plan evidence. | D-1_reasoning_template_schema.md:205-209,335-354 | 04-D-SUMMARY_INDEX.discovery.md:34,596 |
| SUM-C04 | [C] | D-8's normative error enum says `BrokenParent`; `BrokenLink`/`DanglingParent` occur only in context prose and conflict with that enum, so a generic “broken-link report” hides a real identifier mismatch. | D-8_crypto_archive_verify.md:25-28,52-69 | 04-D-SUMMARY_INDEX.discovery.md:122-127,525,603 |
| SUM-C05 | [C] | D-9 defines no pre-push hook. Concrete enforcement blocks commits, runtime writes, or CI, so “blocked git push” is an unimplemented outcome claim in the plan. | D-9_l3_frozen_surfaces.md:108-134,183-187 | 04-D-SUMMARY_INDEX.discovery.md:146-150,577-578 |
| SUM-C06 | [C] | D-10's five names are not five runtime outcomes: `DiscoveryLabelOnly` is explicitly post-hoc after A/D and the selector never returns it. | D-10_mode_routing_predicate.md:18-54,90-96 | 04-D-SUMMARY_INDEX.discovery.md:159-165,505,746 |
| SUM-C07 | [C] | D-10's 45/50 requirement is an accuracy count and cannot itself establish 90% F1. | D-10_mode_routing_predicate.md:203-208,250-254 | 04-D-SUMMARY_INDEX.discovery.md:165,604,747 |
| SUM-C08 | [C] | D-21's no-collapse gate is each mode F1 at least 0.70, not “at least ten correct examples per mode.” | D-21_mode_routing_coverage_benchmark.md:36-49,94-104 | 04-D-SUMMARY_INDEX.discovery.md:318-324,610 |
| SUM-C09 | [C] | D-14's exact dispatch event is `SycophancyProbeDispatched`, not `SYCOPHANCY_PROBE`; the latter is narrative/summary shorthand with no declared mapping. | D-14_sycophancy_probe.md:93,101-106,142-149 | 04-D-SUMMARY_INDEX.discovery.md:216-220,548-549 |
| SUM-C10 | [C] | D-14's concrete hook probes only `ClaimResult::Verified` plus a direct assertion, narrower than an unconditional re-run of every user-aligned claim. | D-14_sycophancy_probe.md:95-100 | 04-D-SUMMARY_INDEX.discovery.md:216-222 |
| SUM-C11 | [C] | D-28 gates both Deception Resistance and Circular Logic at +5pp and permits standard-category regression up to 2pp; “Deception Resistance +5 with no regression” is materially different. | D-28_adversarial_example_farming.md:62-75,129-135 | 04-D-SUMMARY_INDEX.discovery.md:423-430,614 |
| SUM-C12 | [C] | D-28 is not uniformly untouched/deferred: its original explicitly claims the local farming CLI/SSB legs live while training/deployment/remeasurement remain pending. This is plan-recorded status, not independently verified shipped state. | D-28_adversarial_example_farming.md:91-136 | 04-D-SUMMARY_INDEX.discovery.md:419-430,654,766,776 |
| SUM-C13 | [C] | D-28's shipping-pass contract rejects `AdversarialCorpusGenerated` and `PpmFinetuneCompleted` in favor of `AdversarialFarmRunCompleted`. | D-28_adversarial_example_farming.md:77-101 | 04-D-SUMMARY_INDEX.discovery.md:419-430,534-554 |
| SUM-C14 | [C] | For D-1/D-8/D-9/D-10/D-14/D-21/D-24, PLANNED is a summary-layer status rather than an original-file label; D-28 alone declares DEFERRED in its assigned original. | Assigned originals in full; D-28_adversarial_example_farming.md:7-8 | 04-D-SUMMARY_INDEX.discovery.md:623-656,766,776 |

### Additions absent from or materially thinner in the summary

| ID | Class | Addition | Deep evidence | Summary orientation |
|---|---|---|---|---|
| SUM-A01 | [E] | D-1 adds the closed Domain/PatternKind/VerifierKind/ArchiveStatus/RetirementReason/Predicate enums, bounded strings, typed IDs/hashes, and a typed error surface. | D-1_reasoning_template_schema.md:89-112 | 04-D-SUMMARY_INDEX.discovery.md:25-36 |
| SUM-A02 | [E] | D-1 fixes JSONL/O_APPEND storage, fs2 append locking, idempotent migration reporting, registry filtering, config defaults, SeededClock use, and terminal Frozen/Retired lifecycle rules. | D-1_reasoning_template_schema.md:123-180,239-266 | 04-D-SUMMARY_INDEX.discovery.md:25-36,496-520 |
| SUM-A03 | [E] | D-8 adds `--json`/`--fail-fast`, four typed errors, all-error versus early-exit modes, mandatory result emission, nightly/cross-platform CI, recovery guidance, and a 10k/5s/O(n) bound. | D-8_crypto_archive_verify.md:50-128 | 04-D-SUMMARY_INDEX.discovery.md:118-129 |
| SUM-A04 | [E] | D-9 adds the registry record schemas, path/symbol/TOML diff layers, runtime pre-write enforcement, validated 40-character `L3_OVERRIDE`, two exact SSB schemas, and fail-closed parsing. | D-9_l3_frozen_surfaces.md:75-159,201-235 | 04-D-SUMMARY_INDEX.discovery.md:131-151 |
| SUM-A05 | [E] | D-10 supplies exact routing precedence, `Claim`/`ClaimType`/`ClaimResult`, backend stubs, `dispatch_by_mode`, `ClaimDispatched`, and two exact L3 symbol locks. | D-10_mode_routing_predicate.md:33-49,77-148 | 04-D-SUMMARY_INDEX.discovery.md:155-167,498-510,534-554 |
| SUM-A06 | [E] | D-14 defines the 5-token Jaccard threshold (0.7), complete probe/report/verdict structures, exact config defaults, post-Verified hook, MEMORIZING block, and quarantine artifact path. | D-14_sycophancy_probe.md:21-34,58-120,142-170 | 04-D-SUMMARY_INDEX.discovery.md:212-224,756 |
| SUM-A07 | [E] | D-21 defines JSON fixture fields, exact report schema, macro and per-mode gates, human-only fixture authorship/signoff, CLI/CI reachability, SSB rows, dated report artifact, and repeat-run determinism. | D-21_mode_routing_coverage_benchmark.md:29-82,94-105 | 04-D-SUMMARY_INDEX.discovery.md:314-325 |
| SUM-A08 | [E] | D-24 defines helpful/neutral/harmful rate semantics, TOML fixture fields, binary report counters, `elai bench canary-regression`, generic benchmark events, and the report directory. | D-24_cross_domain_canary_regression_suite.md:14-65 | 04-D-SUMMARY_INDEX.discovery.md:352-362 |
| SUM-A09 | [E] | D-28 defines proxy/gold mining, timestamped JSONL, Qwen2.5-Math-PRM-7B LoRA rank 16/3 epochs/1e-5, v2 safetensors deployment, two-category/standard gates, and v1 rollback. | D-28_adversarial_example_farming.md:13-89 | 04-D-SUMMARY_INDEX.discovery.md:419-430 |
| SUM-A10 | [E] | D-28 additionally records actual farming symbols, two live CLI commands, the one shipped completion event, non-mock production constraints, and a separate benchmark corpus path. | D-28_adversarial_example_farming.md:91-121 | 04-D-SUMMARY_INDEX.discovery.md:419-430,522-564 |
| SUM-A11 | [A] | The deep originals expose new unresolved contracts absent from the summary: archive hash/schema mismatches, pre-commit message timing, Mode-C unreachability, D-14 skip/event schema contradictions, D-21 fixture-context gaps, and D-28 dual designs/status. | D1-N02..N12; D8-N01..N12; D9-N02..N13; D10-N02..N13; D14-N02..N14; D21-N02..N10; D28-N01..N14 | 04-D-SUMMARY_INDEX.discovery.md:742-768 |
| SUM-A12 | [A] | The assigned originals add no explicit PII/secret/retention/export safeguards despite persisting prompts, assertions, quarantines, and remote-training corpora. | D-1_reasoning_template_schema.md:123-162; D-14_sycophancy_probe.md:63-105,163-170; D-28_adversarial_example_farming.md:46-60 | 04-D-SUMMARY_INDEX.discovery.md:618-621 |

## Status, dependency, and deferred-work reconciliation

| Source | Original-file status evidence | Deep disposition | Direct dependency/defer gate |
|---|---|---|---|
| D-1 | No status label; unchecked acceptance | Status ambiguous; orientation says PLANNED | C-15 + C-21 DONE; two paper interfaces before testing |
| D-8 | No status label; unchecked acceptance | Status ambiguous; orientation says PLANNED | D-1 |
| D-9 | No status label; unchecked acceptance | Status ambiguous; orientation says PLANNED | D-1..D-8 DONE; three paper interfaces |
| D-10 | No status label; unchecked acceptance | Status ambiguous; orientation says PLANNED | D-9; concrete backends deferred to D-11/D-13/D-16/D-18 |
| D-14 | No status label; unchecked acceptance | Status ambiguous; orientation says PLANNED | D-13; Sharma paper; multi-turn and LLM-judge work deferred |
| D-21 | No status label; unchecked acceptance | Status ambiguous; orientation says PLANNED | D-10; larger/LLM-generated fixture work deferred |
| D-24 | No status label; unchecked acceptance | Status ambiguous; orientation says PLANNED | D-7; real-data mutations deferred |
| D-28 | `Status: DEFERRED`; unchecked acceptance; live-leg annotations | Partial status conflict: live legs are only plan-recorded, GPU proof remains deferred | D-5 ≥20 stable cycles + D-19 baseline + RunPod + paper |

## Completeness and zero-omission/zero-duplicate audit

### Section and ledger coverage

| Source | Lines read | H2 sections audited | [E] | [I] | [A] | [C] | Total atomic source records | Result |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| D-1 | 395/395 | 10/10 | 45 | 0 | 11 | 1 | 57 | complete |
| D-8 | 241/241 | 9/9 | 29 | 0 | 8 | 4 | 41 | complete |
| D-9 | 331/331 | 10/10 | 39 | 0 | 9 | 4 | 52 | complete |
| D-10 | 284/284 | 9/9 | 40 | 0 | 7 | 6 | 53 | complete |
| D-14 | 225/225 | 10/10 | 33 | 0 | 9 | 5 | 47 | complete |
| D-21 | 122/122 | 9/9 | 23 | 0 | 8 | 2 | 33 | complete |
| D-24 | 101/101 | 9/9 | 20 | 0 | 8 | 0 | 28 | complete |
| D-28 | 156/156 | 10/10 | 33 | 1 | 10 | 3 | 47 | complete |
| **Total** | **1855/1855** | **76/76** | **262** | **1** | **70** | **25** | **358** | **zero omitted source sections** |

Every source H2 section was mapped: Context/Reads and paper prerequisites to dependencies/evidence; Tasks to capabilities, schemas, config, integrations, lifecycle and tests; WIRING_CONTRACT to produced/consumed/reachable/stub/observable records; DETERMINISTIC_CHECK and ACCEPTANCE to done gates; Files touched to surfaces/artifacts; Out-of-scope/Next to constraints and deferred/status records. Headings absent from a shorter plan were not invented.

### Required category audit

All requested dimensions occur in the atomic source ledgers: `CAP`, `FLOW`, `DATA`, `CFG`, `INT`, `SEC`, `FAIL`, `DET`, `OBS`, `TEST`, `CON`, and `DEP`. `FAIL` includes explicit fallback, refusal, short-circuit, quarantine, rollback, and recovery semantics. No original states an automatic retry policy; that absence is recorded explicitly rather than silently inferred.

### Identity, manifest, and scope audit

- Atomic source IDs: 358 rows; unique IDs: 358; duplicate IDs: 0.
- Derivative interaction IDs: 19 rows; unique IDs: 19; duplicate IDs: 0; not double-counted as source features.
- Summary-delta IDs: 26 rows; unique IDs: 26; duplicate IDs: 0; not double-counted as source features.
- Total ID-bearing rows before this audit: 403; total unique IDs: 403; duplicates: 0.
- Exact manifest paths: 8; unique paths: 8; fully read: 8; missing: 0; extra original paths read: 0.
- Integrity: 8/8 line counts, byte counts, and SHA-256 values match the manifest; aggregate 1855 lines and 87723 bytes match exactly.
- Orientation use: `04-D-SUMMARY_INDEX.discovery.md` read 777/777 only to locate deltas; it did not override deeper-source identifiers or promote status.
- Excluded by design: no other D-plan original, companion design, implementation file, fixture, paper interface, or generated artifact was opened.

### Final omission/duplication verdict

`ZERO-OMISSION`: all 8 assigned sources, 1855 logical lines, 76 source sections, required behavior dimensions, explicit tests/gates, constraints, dependencies, status statements, and unresolved propositions are represented. `ZERO-DUPLICATE`: an atomic source behavior appears once in its owning ledger; cross-source interactions and summary deltas cite rather than recount it, and all 403 record IDs are unique.

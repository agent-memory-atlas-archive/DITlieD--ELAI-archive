# DEEP-E-004 exhaustive feature discovery

## Scope, method, and integrity

- **Chunk boundary:** only the ten numbered/index originals listed in the exact DEEP-E-004 manifest below were mined for source features. `05-E-SUMMARY_INDEX.discovery.md` was consulted only as an orientation/comparison index; no feature was promoted merely because it appeared there.
- **Read completeness:** all 10 originals were read from line 1 through their final logical line (2,302/2,302 lines; 274,448/274,448 bytes). The manifest block itself was read at `DEEP-SOURCE-MANIFEST.md:561-576`.
- **Semantics:** these documents mix planned work with rebased shipped/partial/deferred claims. Every atom retains the source's own status. An unchecked acceptance box is a gate, not proof of implementation.
- **Deduplication rule:** repeated objective/task/wiring/acceptance wording is one atom with all relevant anchors. A separate test, failure behavior, event, schema field set, default, or done gate remains a separate atom.
- **Classification rule:** `[EXPLICIT]` means stated by the original; `[INFERRED]` means a bounded implication; `[AMBIGUOUS]` means the original leaves a choice unresolved; `[CONFLICT]` means two explicit statements cannot both be the operative contract without reconciliation.
- **Identifier rule:** every ledger, interaction, and summary-delta atom has one unique `E004.*` identifier. Source-ledger counts exclude the manifest/accounting bullets but include explicit, inferred, ambiguous, and conflict atoms.

## Exact processed manifest

| Chunk | Class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---|---:|---:|---|---|---|
| DEEP-E-004 | numbered/index original | 306 | 38,449 | `62f9363c6a8b85504f7e99f3160262d55330a406f443fc27b867a35d64cea348` | `docs/thefinalglance/E-plan/E-5_user_modeling_and_memory.md` | exact |
| DEEP-E-004 | numbered/index original | 215 | 22,065 | `a659b75ba79e6acd6316c922a29ba9738f5362fac4c7ca168e085184fd4b8e3d` | `docs/thefinalglance/E-plan/E-8_self_evolving_prompts.md` | exact |
| DEEP-E-004 | numbered/index original | 251 | 25,555 | `c2c7addd1c5e3fb9a0d6cc7efcdd1f43683a46448b853e3b46a12ba24eca9995` | `docs/thefinalglance/E-plan/E-20_local_cloud_routing.md` | exact |
| DEEP-E-004 | numbered/index original | 174 | 20,362 | `49402af32aa8c6d6954ab87b56c5a78000db67389376f5df8ac1c7c7f3a87d20` | `docs/thefinalglance/E-plan/E-21_peer_systems_llm_as_worker.md` | exact |
| DEEP-E-004 | numbered/index original | 177 | 20,635 | `c328f0391feb65e083c113def69e7e29a5482c41c938d5a7d3ea300a5c1ff87a` | `docs/thefinalglance/E-plan/E-22_github_trending_may2026.md` | exact |
| DEEP-E-004 | numbered/index original | 230 | 29,091 | `1e47656298584ac4fb47b8e01dbf8d2ec9ff3d94fb773ba11b138f1c0a23e359` | `docs/thefinalglance/E-plan/E-25_claude_code_architecture.md` | exact |
| DEEP-E-004 | numbered/index original | 209 | 25,312 | `c41b7e7b7a5397535bbe9e61d778b9a2b86d5121d3ec0b6485ef6810654b0e56` | `docs/thefinalglance/E-plan/E-29_agentic_observability.md` | exact |
| DEEP-E-004 | numbered/index original | 277 | 33,592 | `ebbe04ab238e9e44d3824d2a6de8f7448046005cdcdfbae2379294d55ab92c7d` | `docs/thefinalglance/E-plan/E-37_prompt_injection_defense.md` | exact |
| DEEP-E-004 | numbered/index original | 200 | 23,983 | `7c4ef885c8503e51d06730ef6f0136026d5915272abddb3f11588bd897856c20` | `docs/thefinalglance/E-plan/E-41_ci_cd_pr_review.md` | exact |
| DEEP-E-004 | numbered/index original | 263 | 35,404 | `b20c1e805a3719f6661417e8826dab788be1452aa3676caed4a94179601d3b00` | `docs/thefinalglance/E-plan/E-48_web_research_orchestrator.md` | exact |
| **Total** |  | **2,302** | **274,448** |  | **10 files** | **exact** |

Verification used logical newline counts (`wc -l`), byte counts (`wc -c`), and raw-file `sha256sum`; every row and both aggregate totals match the manifest.

## Per-file exhaustive ledgers

### E-5 — User modeling and long-term memory

Source: `docs/thefinalglance/E-plan/E-5_user_modeling_and_memory.md` (306 lines, 38,449 bytes).

#### Explicit atoms

- **E004.E05.EX001 [EXPLICIT; status]** The 2026-05-23 rebaseline declares Tasks 1,2,3,5,7,9,10,12,13,16 `SHIPPED-ON-V2`; Tasks 8,11,15 `SHIPPED-PARTIAL`; Task 14 `SCHEMA-CONSOLIDATED`; and Tasks 4,6 `DEFERRED-TO-E-5-FOLLOWUP` (L3-L10).
- **E004.E05.EX002 [EXPLICIT; correction/status]** Nine stale post-cutoff arXiv occurrences were citation-cleaned, the shipped drawer path differs from the stale plan path, and `workflow/reasoning/E-5-decisions.md` is the canonical shipped/consolidated/deferred map (L11-L15).
- **E004.E05.EX003 [EXPLICIT; schema/default]** `SCHEMA_VERSION` is locked at 103 with no bump because consolidation preserves the existing wire format (L16, L73-L96, L129-L141).
- **E004.E05.EX004 [EXPLICIT; capability/status]** The objective is a partial-on-v2 bi-temporal episodic store split into project `.elai_cc/ssb/memory.sqlite` and user `%USERPROFILE%\.elai_user\profile.sqlite` tiers, with Mem0 `ADD|UPDATE|DELETE|NOOP`, trust-aware retrieval, and temporal decay (L18).
- **E004.E05.EX005 [EXPLICIT; constraint]** The first version is additive/retrieval-first; cross-project semantic transfer is deferred and remains a design hypothesis (L18, L26, L32, L301).
- **E004.E05.EX006 [EXPLICIT; dependencies]** Hard prerequisites are B-1/B-6/B-9/B-37; C-21 is a soft planned prerequisite; E-1 consumes the schema; E-3 and E-4 consume voice/expertise facts; only E-2 is stated fully parallel-safe (L20-L21, L36-L47).
- **E004.E05.EX007 [EXPLICIT; data model]** Modeled user attributes are voice/style, expertise by language/domain, risk tolerance, communication preferences, high-protection opinions, tool/library preferences, active projects, productivity rhythm, and failure modes; each carries value, confidence, last observation, evidence pointers, and decay rate (L28).
- **E004.E05.EX008 [EXPLICIT; privacy/non-goal]** Emotion, mood, and sentiment toward the assistant are out of scope; v1 also rejects all-or-nothing toggles, LLM-only defense, silent contradiction resolution, and cloud sync without opt-in (L28, L32).
- **E004.E05.EX009 [EXPLICIT; integration]** `memory_facts` is a peer table in the SSB SQLite/WAL store; evidence pointers can target SessionDag node IDs; extraction is the `memory_extractor` Sidecar role; MistakeBook becomes `fact_type='mistake'`; rules may link through `superseded_by` (L36-L40).
- **E004.E05.EX010 [EXPLICIT; trust mapping]** Real-valued `trust_score` maps to C-48 tiers: T0 `>=0.9` user-direct, T1 `>=0.7` codebase-derived, T2 `>=0.4` web-sourced, T3 `<0.4` model-emitted (L41).
- **E004.E05.EX011 [EXPLICIT; determinism]** Stable `ClockProvider` time is required for deterministic `insertion_time` and `last_observed_at` (L42).
- **E004.E05.EX012 [EXPLICIT; schema]** The shipped table has the 16 planned columns `fact_id,user_id,project_id,scope,fact_type,key,value,confidence,trust_score,insertion_time,valid_from,valid_to,decay_rate,last_observed_at,evidence_pointers,superseded_by`, plus `trust_tier` default T3 and provenance JSON (L73-L96).
- **E004.E05.EX013 [EXPLICIT; schema]** Indexes are `idx_user_scope(user_id,scope,fact_type)` and partial `idx_valid_now(user_id) WHERE valid_to IS NULL` (L93-L95).
- **E004.E05.EX014 [EXPLICIT; surface/status]** `crates/memory/` is shipped with CRUD, bi-temporal soft-close, retrieval, decay, errors, evolution, tools, notes, HippoRAG/core-block/trust/view, dispatcher, defense, contradiction, and scheduler modules; extraction is consolidated into `extractor.rs` plus `mem0_dispatcher.rs` rather than planned `extract.rs/update.rs` (L98).
- **E004.E05.EX015 [EXPLICIT; paths/status]** Shipped path helpers are `user_profile_dir`, `user_profile_db_path`, and `user_projects_index_path`, with Windows/Unix fallback rules and constants for `.elai_user`, `profile.sqlite`, and `projects.json`; the per-user write path has no production caller and is deferred (L100).
- **E004.E05.EX016 [EXPLICIT; deferred workflow]** The deferred LLM extractor would run post-turn, call `SidecarRuntime::complete`, parse an SSB-shaped JSON envelope, and feed the two-stage gate; the shipped extractor is only a four-pattern heuristic (L102).
- **E004.E05.EX017 [EXPLICIT; lifecycle]** `DispatchOp::{Add,Update,Delete,Noop}` is shipped with `dispatch_one`/`dispatch_batch`; a JSON tombstone soft-closes, tuple-key duplicates are detected, and UPDATE requires a changed value with non-decreasing confidence or becomes NOOP (L104).
- **E004.E05.EX018 [EXPLICIT; lifecycle/event]** Dispatcher operations are consolidated into `MemoryEvolutionRun(phase="dispatch_op_<add|update|delete|noop>")`; no new dispatcher action variant was added (L104).
- **E004.E05.EX019 [EXPLICIT; deferred lifecycle]** Project scope is default; global promotion would require consistent observation in at least two projects or `elai memory promote <fact_id>`, but semantic equivalence, user-tier writes, and event emission are deferred (L106).
- **E004.E05.EX020 [EXPLICIT; retrieval]** Retrieval selects current facts for the user in the current project or global scope, filters by trust, weights relevance by decay, updates `last_observed_at`, and excludes empty evidence pointers (L108, L121).
- **E004.E05.EX021 [EXPLICIT; observability]** Retrieval has no v1 SSB beacon: `memory.retrieval` tracing plus the read-side `last_observed_at` mutation is the observable effect (L108, L139-L140).
- **E004.E05.EX022 [EXPLICIT; security/status]** Stage-1 defense is shipped with regexes for force-push, pipe-to-shell downloads, root deletion, eval/exec/system calls, destructive sudo, SSH private keys, and prompt-injection directives; `Block` outranks `SkipSensitive` (L110).
- **E004.E05.EX023 [EXPLICIT; security/deferred]** The Stage-2 LLM classifier, classifier prompt/training pairs, and task-runner event emission are deferred; direct CLI rejection uses stderr and exit code 1 (L110).
- **E004.E05.EX024 [EXPLICIT; scheduler]** `spawn_decay_scheduler` returns an abortable handle, defaults to a one-hour interval with one-second minimum, optionally runs immediately, aborts on `Drop`, and takes an injected time closure for virtual-clock tests (L112).
- **E004.E05.EX025 [EXPLICIT; recovery]** Live CLI also performs an inline post-reflect decay tick so decay progresses without a long-lived Tokio runtime (L112).
- **E004.E05.EX026 [EXPLICIT; defaults]** Daily decay defaults are voice_style/expertise `0.0001`, opinion `0.0005`, mistake `0.005`, preference `0.001`, and session_pattern `0.01`; each tick carries `ticks_applied` and `mean_trust_score` through `MemoryEvolutionRun(phase="decay_tick")` (L112-L119).
- **E004.E05.EX027 [EXPLICIT; conflict handling/status]** Shipped contradiction audit groups by `(project_id,scope,key)`, compares old/new pairs, surfaces differing values by default, and only auto-supersedes via soft-close when enabled and the newer trust tier is higher (L123).
- **E004.E05.EX028 [EXPLICIT; deferred conflict handling]** LLM semantic contradiction using cosine `< -0.4` and the dashboard adjudication UI are deferred; both rows otherwise survive (L123, L143).
- **E004.E05.EX029 [EXPLICIT; CLI/status]** Shipped CLI commands are `list`, `inspect`, `insert`, and `delete`, with aliases `show`, `add`, and `forget`; insert runs Stage-1 defense then dispatch, uses T0 user-direct provenance and initial trust `0.95`, and delete soft-closes (L125).
- **E004.E05.EX030 [EXPLICIT; deferred CLI]** `incognito`, `promote`, and `audit` are deferred because they need session flags, the promotion gate, or a trust-survivor query (L125).
- **E004.E05.EX031 [EXPLICIT; privacy]** Sensitive auto-skip covers OpenAI/Anthropic-style keys, AWS keys/env, bearer JWT, dotenv content, SSH private-key paths, and GitHub PATs; it is a policy skip rather than attack rejection (L127).
- **E004.E05.EX032 [EXPLICIT; event consolidation]** Nine planned events map to existing `MemoryFactExtracted`, `MemoryConflictDetected`, or free-form `MemoryEvolutionRun.phase`; `memory_audit_query` and promotion emission are deferred, preserving the 245/306 action bijection without a schema bump (L129-L141).
- **E004.E05.EX033 [EXPLICIT; UI/status]** Read-only `MemoryFactsDrawer.svelte` is shipped and mounted through `DrawerHost.svelte`; `get_memory_facts` and `get_trust_score` are shipped, while three-way adjudication, heat map, promotion controls, and audit log are deferred (L143).
- **E004.E05.EX034 [EXPLICIT; defaults]** The decision document recommends per-turn extraction, promotion `N=2`, high-stakes trust thresholds `0.7` commit/`0.9` force-push/`0.8` delete, and local Qwen2.5-Coder-3B extraction with rule-first defense (L145).
- **E004.E05.EX035 [EXPLICIT; API]** The wiring contract names `MemoryFact`, `FactType`, `Scope`, `FactStore`, `ExtractionPipeline`, `Mem0Dispatcher`, `retrieve_for_user`, `DecayScheduler`, `TwoStageWriteGate`, path helpers, CLI routes, and the drawer as produced surfaces (L147-L160).
- **E004.E05.EX036 [EXPLICIT; reachability]** Entry paths are post-turn LiveCli extraction, dashboard Tauri reads, and direct `elai memory <subcommand>` invocation (L161-L170).
- **E004.E05.EX037 [EXPLICIT; hard constraints]** Banned implementations include bypassing `TwoStageWriteGate`, retrieving empty-provenance rows, hardcoded trust 1.0, defense-later stubs, and any v1 cloud-sync path (L171-L176).
- **E004.E05.EX038 [EXPLICIT; evidence]** The designed smoke is a preference stated in one REPL session and recalled unprompted in the next, with extraction/insertion/retrieval evidence and message-linked provenance visible through CLI (L178-L181).
- **E004.E05.EX039 [EXPLICIT; deterministic gates]** Static gates inspect the migrated schema/indexes, path helpers, event names, CLI routing, gated insert path, absence of cloud sync, and a live current-preference row (L183-L209).
- **E004.E05.EX040 [EXPLICIT; acceptance]** Acceptance requires schema/crate/path viability, Mem0 four-way dispatch, MINJA and sensitive-data blocks, provenance filtering, surfaced contradictions, decay ticks, CLI/UI behavior, no cloud sync, and cross-session recall (L211-L225).
- **E004.E05.EX041 [EXPLICIT; test]** `bi_temporal_round_trip` must set `valid_to` on the old fact and preserve the supersession link when a replacement is inserted (L229).
- **E004.E05.EX042 [EXPLICIT; test]** `two_stage_gate_blocks_minja` compares ten rejected malicious fixtures with ten accepted legitimate fixtures (L230).
- **E004.E05.EX043 [EXPLICIT; test]** `sensitive_class_skip`, `decay_curves`, `promotion_gate`, `contradiction_no_auto_update`, and `provenance_required_retrieval` cover policy-skip semantics, 1%-tolerance decay, promotion/contradiction branching, coexistence, and provenance exclusion (L231-L235).
- **E004.E05.EX044 [EXPLICIT; test]** Contract/CLI/dashboard tests cover turn→extract→gate→dispatch→SSB, every eight-command planned surface, and drawer adjudication (L236-L238).
- **E004.E05.EX045 [EXPLICIT; artifacts]** The declared change set creates the memory crate and decision doc and modifies workspace membership, paths, SSB, task runner, CLI args/routing, and dashboard Tauri wiring (L240-L259).
- **E004.E05.EX046 [EXPLICIT; calibration]** Planned follow-up measurements sweep decay rates, extraction cadence, trust thresholds, project-similarity promotion, and episodic-to-semantic consolidation cadence (L293-L300).
- **E004.E05.EX047 [EXPLICIT; non-goal/future]** Future work—not v1—is semantic cross-project merge, optional git-remote user sync, a graph backend, and episodic→semantic batch consolidation (L301-L306).

#### Inferred atoms

- **E004.E05.IN001 [INFERRED]** Because only project storage has a caller while global-write promotion is deferred, current global facts can be seeded/read but the stated two-tier production lifecycle is not end-to-end complete (from L100, L106, L125).
- **E004.E05.IN002 [INFERRED]** Event names in the original observable/acceptance text describe the pre-rebaseline intent; the later status/consolidation map is the controlling event contract because it is explicitly canonical (from L14-L16, L178-L181, L193-L194).

#### Ambiguous atoms

- **E004.E05.AM001 [AMBIGUOUS]** The header calls the shipped schema 18 columns, but the SQL lists 16 and only prose identifies the extra `trust_tier` and provenance fields; exact SQL names/types for those two fields are absent (L73-L96).
- **E004.E05.AM002 [AMBIGUOUS]** `superseded_by` direction is inconsistent in wording: the schema field sits on a fact, while the planned test says the new row points to the old fact rather than the old row pointing to its replacement (L91, L229).
- **E004.E05.AM003 [AMBIGUOUS]** Acceptance still demands all eight planned CLI commands and mutable drawer behavior although the status block explicitly defers several commands and UI controls (L8, L15, L222-L224).
- **E004.E05.AM004 [AMBIGUOUS]** The observable smoke expects insertion/retrieval action names that were consolidated or eliminated; the equivalent canonical phase/tracing evidence must be chosen before using the smoke literally (L178-L181 versus L129-L141).

#### Conflict atoms

- **E004.E05.CF001 [CONFLICT]** The wiring contract still promises 11 new `SsbActionType` variants and the deterministic check searches for them, while the rebaseline explicitly says no variants were added and nine were consolidated/two deferred (L158, L193-L194 versus L129-L141).
- **E004.E05.CF002 [CONFLICT]** The wiring contract and file list name `components/MemoryDrawer.svelte`, while shipped reality is `drawers/MemoryFactsDrawer.svelte` and the former path is declared stale (L12, L143, L177, L238, L251).
- **E004.E05.CF003 [CONFLICT]** The wiring contract lists `purge`, while shipped CLI detail lists `delete` plus aliases and no purge implementation; acceptance's “eight” count includes help rather than insert/delete consistently (L125, L160, L222).

### E-8 — GEPA self-evolving prompts

Source: `docs/thefinalglance/E-plan/E-8_self_evolving_prompts.md` (215 lines, 22,065 bytes).

#### Explicit atoms

- **E004.E08.EX001 [EXPLICIT; capability]** Build a GEPA reflective optimizer for PromptEngineer/ReasoningTemplate using cloud Sonnet 4.6 reflection, local Qwen2.5-Coder-7B target inference, LPO edit spans, BAI/Two-Gate promotion, held-out evaluation, and verbatim auditor feedback (L1-L2).
- **E004.E08.EX002 [EXPLICIT; dependencies/status]** D-1 and D-5 are planned alternatives to E-7 archive/promotion work; B-40 and B-37 are done; the plan is parallel-safe with E-6/E-7/E-9/E-10 and does not claim implementation status (L3-L6).
- **E004.E08.EX003 [EXPLICIT; architecture]** Optimization is a rare Python path emitting blake3-hashed templates into a Rust-owned archive; inference remains in Rust, and porting DSPy/DSRs to Rust is explicitly rejected (L10, L18-L20, L208).
- **E004.E08.EX004 [EXPLICIT; lifecycle]** New optimizer output enters the typed archive as `SHADOW` and advances to `ACTIVE` only through measured BAI Thompson/Two-Gate promotion (L14, L50, L64).
- **E004.E08.EX005 [EXPLICIT; workflow]** Critic-loop and B-41 structured `PASS|BLOCK` reasons become GEPA feedback; B-37 supplies cloud reflection and local target routes (L15-L17).
- **E004.E08.EX006 [EXPLICIT; service/API]** `.elai_cc/python/elai_optimizer/optimizer.py` exposes `optimize_template(template_path,train_set_path,val_set_path,reflection_model="claude-4.6-sonnet",target_model_url="http://localhost:8100/v1/messages")` (L44).
- **E004.E08.EX007 [EXPLICIT; supply chain]** The `gepa` PyPI package version and last-published date must be verified before pinning; an abandoned package falls back to a vendored published-paper algorithm (L44, L152, L202).
- **E004.E08.EX008 [EXPLICIT; edit policy]** Editable blocks are TASK, GROUNDING, PRE_GATE, TDD, role-prompt text, and few-shots; DETERMINISTIC_CHECK, ACCEPTANCE, INTEGRATION, and EXIT_CRITERIA are `<frozen>` and out-of-span edits are refused (L19, L46, L147).
- **E004.E08.EX009 [EXPLICIT; event]** Parsing editable/frozen spans emits `SsbActionType::EditableSpansComputed` (L46, L82).
- **E004.E08.EX010 [EXPLICIT; schema]** Auditor feedback JSON contains `verdict`, `structured_reasons: Vec<FindingReason>`, optional E-6 `mast_mode`, failing test names, and full block messages at `.elai_cc/optimization/feedback/{run_id}.json` (L48).
- **E004.E08.EX011 [EXPLICIT; isolation]** Python may read feedback but never write SSB; it has read-only access to `.elai_cc/optimization/` and write-only access to `.elai_cc/optimization/outputs/` (L48, L60, L96).
- **E004.E08.EX012 [EXPLICIT; schema]** `OptimizedTemplate` carries content and parent blake3 hashes, optimizer version/run ID, validation/held-out lift, and cross-domain `(family,delta)` results (L50).
- **E004.E08.EX013 [EXPLICIT; admission]** Rust validates blake3, runs paraphrase/canary contamination checks, requires held-out lift `>=5%` and no cross-domain regression, then admits as SHADOW (L50).
- **E004.E08.EX014 [EXPLICIT; context injection]** Each run injects the top three most-active gotcha categories from the preceding 30 days as a `Pitfalls:` textual-parameter block (L52).
- **E004.E08.EX015 [EXPLICIT; transfer/recovery]** Transfer validation re-evaluates the same Qwen target on held-out data and two cross-domain families; a target result more than `2pp` below validation is overfit-risk and refused (L54).
- **E004.E08.EX016 [EXPLICIT; CLI/defaults]** `elai optimize template` accepts target/train/val, defaults reflection to Sonnet 4.6 and budget to 200,000 tokens, spawns Python, tails progress, reports validation/held-out/cross-domain results, and refuses a local reflection model (L56).
- **E004.E08.EX017 [EXPLICIT; UI]** `OptimizerRunPanel.svelte` shows live progress, Pareto cells, validation-vs-held-out lift, overfit risk, and too-small-reflection warnings through `get_optimizer_run(run_id)` (L58).
- **E004.E08.EX018 [EXPLICIT; determinism]** Python produces template text only; Rust exclusively owns validation, admission, SSB mutation, and promotion (L60).
- **E004.E08.EX019 [EXPLICIT; events]** The run emits `OptimizerRunStarted`, `OptimizerProgress`, `OptimizerConverged`, `OptimizerOverfitRejected`, `OptimizedTemplateAdmitted`, `OptimizedTemplatePromoted`, plus `EditableSpansComputed`, correlated by `optimization_run_id` (L62, L76-L82).
- **E004.E08.EX020 [EXPLICIT; held-out gate]** Every template is tested with paraphrase filtering and an active canary GUID; passers enter the BAI pool and failures emit `OptimizerOverfitRejected` (L64).
- **E004.E08.EX021 [EXPLICIT; API]** Produced APIs are `optimize_template`, `validate_transfer`, `editable_spans`, `export_for_run`, `admit_optimized_template`, `top_gotchas`, and CLI dispatch; consumers are the E-7/D-1 archive, promotion gate, per-role runtime, sidecar, and CLI main (L66-L91).
- **E004.E08.EX022 [EXPLICIT; hard constraints]** Bans include Qwen reflection, first-admit ACTIVE, editable deterministic-check blocks, and any non-test Python direct SSB write (L92-L96).
- **E004.E08.EX023 [EXPLICIT; evidence]** The smoke emits Started→at least three Progress→Converged-or-Rejected within budget; admission occurs iff lift/regression gates pass, and CLI/dashboard show identical metrics and final text (L98-L101).
- **E004.E08.EX024 [EXPLICIT; deterministic gate]** Checks verify package identity/version, local-reflection refusal, frozen spans, feedback JSON, SHADOW first admission, cross-domain rejection, recorded split sizes, filesystem isolation, and absence of blake2/sha2 (L103-L141).
- **E004.E08.EX025 [EXPLICIT; acceptance]** End-to-end acceptance uses a 50-task fixture and requires a populated validation lift, cloud reflection, frozen contract blocks, separate train/validation/holdout sets, `2pp` held-out tolerance, E-7 cross-domain limits, gotcha injection, live dashboard data, supply-chain evidence, typed Python, no Rust unwrap, and `orjson` (L143-L154).
- **E004.E08.EX026 [EXPLICIT; test]** Edit-span and auditor-feedback tests cover all eight block types and the JSON envelope contract (L158-L159).
- **E004.E08.EX027 [EXPLICIT; test]** Admission tests require SHADOW default, `>2pp` validation/held-out gap rejection, and `-25%` relative non-target regression rejection (L160-L162).
- **E004.E08.EX028 [EXPLICIT; test]** Contract tests round-trip seven action variants and prohibit Python SSB writes; dashboard tests render three stages (L163-L165).
- **E004.E08.EX029 [EXPLICIT; test]** Python tests exercise transfer gap and a five-iteration deterministic-Sonnet-stub smoke returning non-trivial text (L166-L167).
- **E004.E08.EX030 [EXPLICIT; artifacts]** Planned files create the Python optimizer package/tests, Rust feedback/admission/gotcha/CLI modules, the dashboard panel, and modify BabyStepSpec, SSB, CLI registration, and Tauri command wiring (L169-L184).
- **E004.E08.EX031 [EXPLICIT; cost/default]** Per-template cost is unconfirmed; UI must surface it and the default run cap is 200k tokens (L203).
- **E004.E08.EX032 [EXPLICIT; observability]** `reflection-LM model_id` is logged so lift can be correlated with optimizer strength (L204).
- **E004.E08.EX033 [EXPLICIT; retry/stop]** Validation early stopping uses configurable patience of two outer iterations and minimum delta `1pp` (L205).
- **E004.E08.EX034 [EXPLICIT; failure]** MAP-Elites cell entropy below `1.0 nats` raises a collapse warning (L206).
- **E004.E08.EX035 [EXPLICIT; security]** The contamination filter applies to optimizer training data as well as held-out data to mitigate benchmark/search-space leakage (L207).
- **E004.E08.EX036 [EXPLICIT; downstream]** E-8 supplies optimized-vs-baseline templates to E-9, produces SHADOW entries for E-7, partially grounds E-10, and supports one role per run; all-eight-role optimization is follow-on work (L210-L215).

#### Inferred atoms

- **E004.E08.IN001 [INFERRED]** Because no rebaseline/status block exists and all acceptance boxes are unchecked, every task is a design requirement rather than verified shipped behavior (from L1-L215).

#### Ambiguous atoms

- **E004.E08.AM001 [AMBIGUOUS]** The stated DSPy “20/80 train/val split” does not say whether 20% is train or validation in the executable schema, despite calling it inverted (L10, L148, L198).
- **E004.E08.AM002 [AMBIGUOUS]** Cross-domain rejection is described both generically as “no regression” and by E-7 thresholds `-20% rel / -5% abs`; precedence between these and the `2pp` same-target gap is not formalized (L50, L54, L149).
- **E004.E08.AM003 [AMBIGUOUS]** `gepa` maintenance, exact optimizer cost, and whether E-7 or D-1/D-5 owns the archive/promotion implementation remain unresolved prerequisites (L4, L202-L204).

#### Conflict atoms

- **E004.E08.CF001 [CONFLICT]** The filesystem rule calls `.elai_cc/optimization/` read-only while the feedback exporter writes below `.elai_cc/optimization/feedback/`; isolation must distinguish the Rust writer from the restricted Python process (L48, L60).

### E-20 — Local/cloud routing

Source: `docs/thefinalglance/E-plan/E-20_local_cloud_routing.md` (251 lines, 25,555 bytes).

#### Explicit atoms

- **E004.E20.EX001 [EXPLICIT; capability]** Replace all-or-nothing HYBRID escalation with a calibrated logprob/entropy cascade, add pre-dispatch difficulty classification, first-class privacy blocking, monthly-burn visibility, and role/task-shape-aware cloud selection (L1-L2).
- **E004.E20.EX002 [EXPLICIT; dependencies/status]** B-27 and C-5 are planned prerequisites/extensions, B-37 is shipped, B-22 owns adjacent role schema, D-10 is orthogonal, and C-6/C-7 consume the CloudPanel additions (L4-L5, L26-L34).
- **E004.E20.EX003 [EXPLICIT; current state]** Existing modes are `LOCAL|CLOUD|HYBRID`; C-5's retry cap remains a safety net, while E-20 changes HYBRID's primary escalation from L3 failure to L1 calibrated confidence (L10, L29-L30, L34).
- **E004.E20.EX004 [EXPLICIT; routing signal]** Local inference surfaces `confidence_score = -entropy_of_first_token_distribution`; probe/perplexity signals are required and verbalized self-confidence is forbidden (L53).
- **E004.E20.EX005 [EXPLICIT; config/default]** HYBRID escalates when `confidence_score < L_HYBRID_THRESHOLD`, initially `0.55`, stored with `calibrated_at` under `[cloud.hybrid]`; retry cap `1` remains secondary (L55).
- **E004.E20.EX006 [EXPLICIT; event/schema]** Escalation records `hybrid_escalation_reason = confidence|retry_exhausted|privacy_block` (L55).
- **E004.E20.EX007 [EXPLICIT; workflow]** `DifficultyClassifierAdapter` is a new `RuntimeAdapter` that runs before `PerRoleRuntime`, starts from an open ~110M RouteLLM BERT router, and emits `easy|medium|hard` plus `cloud_recommended` (L57).
- **E004.E20.EX008 [EXPLICIT; deferred]** Fine-tuning on ELAI traces and a cheaper activation-probe router are deferred to E-20.B until labeled rows/hidden states exist (L57, L235, L246).
- **E004.E20.EX009 [EXPLICIT; security]** Before every cloud escalation, an L1 privacy gate scans AKIA/sk/JWT/private-key/AWS-account patterns, environment names/shell-expansion sigils, denylisted credential paths, and an optional ML PII classifier (L59).
- **E004.E20.EX010 [EXPLICIT; failure/fallback]** Any privacy match emits a `PrivacyBlock` route event, keeps the route local, spends no cloud money, and exposes a redacted reason in CloudPanel (L59, L117-L120).
- **E004.E20.EX011 [EXPLICIT; privacy]** Cloud responses run the same regex/ML scan before display; findings emit `route_output_filtered` with a redaction count (L61).
- **E004.E20.EX012 [EXPLICIT; observability]** Spend tracking exposes running 7-day and 30-day windows and projected monthly burn from the last 20 tasks; alert at twice average daily burn or above the configured monthly ceiling (L63).
- **E004.E20.EX013 [EXPLICIT; event]** Spend threshold violations emit `cloud_burn_alert` (L63).
- **E004.E20.EX014 [EXPLICIT; config]** `PerRoleRuntime` gains priority `cost|latency|quality` and task-shape `execution|architecture|math` (L65).
- **E004.E20.EX015 [EXPLICIT; defaults/fallback]** Default routing is executor/researcher→Sonnet, architect/impact-mapper→Opus, math→DeepSeek-R1/dedicated backend, otherwise Opus; CloudPanel shows the selected backend (L65).
- **E004.E20.EX016 [EXPLICIT; determinism]** The routing decision—not the answer—is sampled three times locally and majority/median-selected to reduce flapping; the three-vote tally is journaled (L67).
- **E004.E20.EX017 [EXPLICIT; reasoning mode]** L3+ cloud calls reuse difficulty classification to decide whether extended reasoning is needed and emit `cloud_reasoning_mode_chosen` (L69).
- **E004.E20.EX018 [EXPLICIT; calibration]** `.elai_cc/eval/routing/` contains 200+ paired local/cloud/auditor tasks; Youden's J fits the threshold and results are segmented by L1-L5 (L71).
- **E004.E20.EX019 [EXPLICIT; API]** Produced routing surfaces include `compute_score`, `HybridEscalationGate`, `DifficultyClassifierAdapter`, `RouteBlocked::PrivacyBlock`, input/output secret scanners, burn projections/alerts, role fields, N=3 route voting, and semantic-reasoning choice (L73-L87).
- **E004.E20.EX020 [EXPLICIT; events]** Nine named action types are `ConfidenceScoreEmitted`, `HybridEscalationReason`, `DifficultyClassified`, `RouteBlockedPrivacy`, `RouteOutputFiltered`, `CloudBurnAlert`, `CloudReasoningModeChosen`, `RoutingMajorityVoteBreakdown`, and `RoleRuntimePriorityChosen` (L88-L96).
- **E004.E20.EX021 [EXPLICIT; reachability]** The path is task-runner cloud/local routing, existing Tauri `set_task_mode`, and CLI `run_task`; it extends B-37 runtime snapshots, C-5 spend, and C-7 CloudPanel (L97-L107).
- **E004.E20.EX022 [EXPLICIT; constraints]** No cloud dispatch/display may bypass input/output scanning; no verbal confidence, naive mixed-LLM ensemble, new unwrap, or blake2/sha2 prompt hashing is allowed (L108-L115).
- **E004.E20.EX023 [EXPLICIT; evidence]** A hybrid task asking to print `AWS_SECRET_ACCESS_KEY` must emit a privacy-block row, invoke no cloud model, spend zero, and remain local or refuse (L117-L120).
- **E004.E20.EX024 [EXPLICIT; deterministic gates]** Static/runtime checks cover confidence, gate replacement, difficulty adapter, both privacy scanners, burn math, role schema, exactly three routing samples, reasoning toggle, calibration files, and the secret smoke (L122-L165).
- **E004.E20.EX025 [EXPLICIT; acceptance]** Done requires per-inference confidence SSB, calibrated confidence escalation, pre-role difficulty, hard privacy/output filtering, CloudPanel burn alerts, role fields/mapping, N=3 decision vote, L3+ reasoning toggle, and calibrated per-layer thresholds (L167-L178).
- **E004.E20.EX026 [EXPLICIT; constraint]** Self-MoA is reserved for L4/L5 and must aggregate repeated samples from one best model, never mix different models (L179-L180, L216, L238-L240).
- **E004.E20.EX027 [EXPLICIT; test]** Confidence/gate tests verify score math/payload and low-confidence escalation versus high-confidence local execution per layer (L184-L185).
- **E004.E20.EX028 [EXPLICIT; test]** Classifier/privacy tests cover easy-medium-hard fixtures, six blocked secret prompts, four clean prompts, and embedded-AKIA output redaction (L186-L188).
- **E004.E20.EX029 [EXPLICIT; test]** Burn, role, vote, and reasoning tests enforce ±10% projection, schema/default roundtrip, stable noisy voting, and lookup-off/math-on reasoning (L189-L192).
- **E004.E20.EX030 [EXPLICIT; test]** The E-20 contract round-trips every new event and the extended mode accept set (L193).
- **E004.E20.EX031 [EXPLICIT; artifacts]** Planned changes create `cost/cloud_route.rs`, privacy scanners, a provider difficulty adapter, two calibration TOMLs, and a decision doc; they modify runtime orchestration, per-role schema, spend, SSB, CloudPanel, and config docs (L195-L209).
- **E004.E20.EX032 [EXPLICIT; config/fallback]** Regex-only is the default privacy implementation; proposed ML config is opt-in Presidio, while SaaS alternatives remain a decision-doc choice (L236).
- **E004.E20.EX033 [EXPLICIT; override/audit]** Output false positives may be bypassed by `[cloud.privacy] allowlist_files`, but every override must be audited (L237).
- **E004.E20.EX034 [EXPLICIT; deferred]** N=3 Opus Self-MoA for HEAVY L5 is E-20.B and xRouter RL training is E-20.C; both require measurement/training first (L238-L247).
- **E004.E20.EX035 [EXPLICIT; calibration risk]** Role defaults are heuristic and should be refit after 200 accumulated tasks (L241).
- **E004.E20.EX036 [EXPLICIT; dependency sequencing]** E-20 should land before C-5 finalizes `RouteBlocked` and before C-7 finalizes CloudPanel; D-10 composes as “what to verify” versus E-20 “where to run” (L243-L250).
- **E004.E20.EX037 [EXPLICIT; non-goal]** A future GEPA-optimized gpt-oss-120b adapter is bookmarked but not part of E-20 (L251).

#### Inferred atoms

- **E004.E20.IN001 [INFERRED]** PrivacyBlock is both an escalation reason and a hard local-route decision; the intended meaning is “cloud escalation considered but vetoed,” not a third execution mode (from L55, L59).

#### Ambiguous atoms

- **E004.E20.AM001 [AMBIGUOUS]** The exact entropy normalization/sign convention is not specified, so a threshold of `0.55` cannot be implemented reproducibly from `-entropy` alone (L53-L55).
- **E004.E20.AM002 [AMBIGUOUS]** The ML PII classifier vendor and the behavior when its model is unavailable are unresolved beyond regex-only default (L59, L236).
- **E004.E20.AM003 [AMBIGUOUS]** A flat threshold is an explicit fallback, but the source does not state when per-layer calibration is considered insufficient and triggers that fallback (L234).
- **E004.E20.AM004 [AMBIGUOUS]** `privacy_block` appears in `hybrid_escalation_reason` even though privacy prevents escalation; payload consumers need a precise attempted-route state model (L55, L59).

#### Conflict atoms

- **E004.E20.CF001 [CONFLICT]** None within the original after treating retry cap as secondary rather than removed.

### E-21 — Peer-system harness gap closes

Source: `docs/thefinalglance/E-plan/E-21_peer_systems_llm_as_worker.md` (174 lines, 20,362 bytes).

#### Explicit atoms

- **E004.E21.EX001 [EXPLICIT; capability]** Preserve ELAI's typed/blake3/BSP/GRC/rules stack while adding seven peer-grounded improvements: PageRank repo map, SBFL, critic rerank, ACI audit, repo state, exposed multi-provider routing, and SWE-bench-Verified tooling (L1-L2, L10-L14).
- **E004.E21.EX002 [EXPLICIT; dependencies/status]** B-1/B-22/B-37/B-41 are done, D-1 is planned but nonblocking, and E-22/E-24/E-25 are parallel/coordination peers (L3-L6, L18-L23).
- **E004.E21.EX003 [EXPLICIT; graph algorithm]** `crates/symbol_graph/src/pagerank.rs` consumes SCIP_REF/CALL rows, uses damping `0.85`, `max_iters=64`, tolerance `1e-6`, and emits `FileRank{path,rank,in_degree,out_degree}` (L42).
- **E004.E21.EX004 [EXPLICIT; artifact/surface]** PageRank persists to `artifacts_dir_path/pagerank.bincode`, is exposed as `elai symbol-graph rank --top K`, and backs `get_pagerank_top` (L42).
- **E004.E21.EX005 [EXPLICIT; localization]** SBFL consumes verified `cargo tarpaulin --skip-clean --out Json` coverage and computes the source-stated Ochiai expression into `SuspiciousLine{path,line,score,...}` (L44).
- **E004.E21.EX006 [EXPLICIT; workflow]** On auditor BLOCK with failing tests, `suspicious_lines` attaches to auditor JSON and is consumed by the fixer; B-23 may later use high-suspicion lines (L44).
- **E004.E21.EX007 [EXPLICIT; rerank]** `mode=critic_rerank` generates 3–5 candidate patches through `PerRoleRuntime`, scores them with a critic role, picks argmax, and can use a cloud critic via `--critic-runtime cloud` (L46).
- **E004.E21.EX008 [EXPLICIT; concurrency/fallback]** Parallel candidate calls are used when workers exist; a one-worker sidecar falls back to sequential N=3, while acceptance also requires graceful N=1 skip with `critic_rerank_skipped` when only one runtime is available (L46, L120).
- **E004.E21.EX009 [EXPLICIT; events]** Rerank emits `critic_candidates_generated`, `critic_score_computed`, and `critic_pick_committed` (L46).
- **E004.E21.EX010 [EXPLICIT; audit]** The ACI audit grades every tool on terse/readable output, structured parseable fields, and sane default scope, producing `workflow/audits/aci-tool-shape-2026-05-19.md` plus a fixer queue (L48).
- **E004.E21.EX011 [EXPLICIT; non-goal]** ACI work is an audit, not the tool refactor; remediation is follow-up work chosen by the user (L48).
- **E004.E21.EX012 [EXPLICIT; durable state]** `.elai_cc/.elai/state/` stores `current_phase.md`, rolling-ten `last_decisions.md`, `stub_registry.toml`, and `runtime_choices.toml`; LiveCli initializes phase and PreCompact persists decisions (L50).
- **E004.E21.EX013 [EXPLICIT; persistence/permission]** The state directory is intended to be tracked inside the `.elai_cc` repository so it survives `git clean -fdx`, but the user must confirm before ignore/tracking changes (L50, L168).
- **E004.E21.EX014 [EXPLICIT; routing]** `--runtime-table <path>` exposes existing PerRoleRuntime mappings for `anthropic_local_sidecar`, `anthropic_cloud`, `ollama`, `openai_compatible`, and `vllm_openai_compatible`; no LiteLLM dependency is added (L52).
- **E004.E21.EX015 [EXPLICIT; benchmark]** `elai bench swebench-verified` uses the production `RuntimeOrchestrator::execute_step`, verifies fixture URL/license, and persists dated JSON under `.elai/state/benchmarks/` (L54).
- **E004.E21.EX016 [EXPLICIT; benchmark schema]** Benchmark aggregates `passed`, `failed_compile`, `failed_test`, and `failed_semantic`, explicitly distinguishing superficial artifacts from built features (L54).
- **E004.E21.EX017 [EXPLICIT; API]** Produced types/functions are `compute_pagerank`, `FileRank`, `compute_ochiai`, `SuspiciousLine`, `CriticRerank::pick`, `Candidate`, and `swebench_verified_run` (L56-L65).
- **E004.E21.EX018 [EXPLICIT; reachability]** CLI exposes benchmark and rank; task driving exposes optional rerank; auditor BLOCK auto-attaches SBFL; dashboard reads PageRank (L66-L76).
- **E004.E21.EX019 [EXPLICIT; constraints]** No unregistered TODO, production `unimplemented!`, or non-boot/test unwrap; coverage must come from verified tarpaulin JSON, not a hand parser (L77-L81).
- **E004.E21.EX020 [EXPLICIT; evidence]** On ELAI's own indexed tree, top-five PageRank must emit `pagerank_computed` within 5s and persist a deserializable file with over 100 entries (L82-L86).
- **E004.E21.EX021 [EXPLICIT; deterministic gates]** Checks require PageRank code, SBFL consumption, rerank event wiring, initialized repo state, benchmark help flags, and complete ACI audit rows (L88-L114).
- **E004.E21.EX022 [EXPLICIT; acceptance]** PageRank should rank sensible core directories; SBFL attaches; rerank selects and falls back; all tools receive A-D audit grades; state survives PreCompact; runtime-table overrides; and at least ten benchmark tasks produce JSON (L116-L124).
- **E004.E21.EX023 [EXPLICIT; dependency gate]** No dependency beyond user-confirmed, identity-verified cargo-tarpaulin is allowed (L125).
- **E004.E21.EX024 [EXPLICIT; test]** PageRank uses a three-node star correctness fixture and SBFL uses a hand-computed synthetic coverage table (L129-L130).
- **E004.E21.EX025 [EXPLICIT; test]** Rerank must choose known argmax; benchmark dry-run uses one mini fixture task (L131-L132).
- **E004.E21.EX026 [EXPLICIT; test]** Contract tests require repo state to survive PreCompact and every registered tool to appear in the dated ACI audit (L133-L134).
- **E004.E21.EX027 [EXPLICIT; artifacts]** Planned changes create PageRank, peer-system SBFL/rerank, CLI benchmark, ACI audit, and decision doc; modify graph CLI, main, runtime, auditor, LiveCli, SessionDag, and dashboard commands (L136-L150).
- **E004.E21.EX028 [EXPLICIT; fallback]** If tarpaulin lacks Windows support, verified `cargo llvm-cov` is the fallback (L166).
- **E004.E21.EX029 [EXPLICIT; performance risk]** Serial N=5 rerank may cost ~5× wall time and production must tolerate skipping under insufficient runtime hardware (L167).
- **E004.E21.EX030 [EXPLICIT; path risk]** State storage must remain below `.elai_cc/.elai/state/` and avoid B-1 index-path collisions (L168).
- **E004.E21.EX031 [EXPLICIT; benchmark default]** Full SWE-bench-Verified may exceed one hour; first runs default `--max-tasks 50` (L169).
- **E004.E21.EX032 [EXPLICIT; open choice]** ACI grades may be human-authored or auditor-agent-driven; the source leaves that choice to the user (L170).
- **E004.E21.EX033 [EXPLICIT; downstream]** E-22 adds tools the ACI audit should cover, E-24 shares tool dispatch, E-25 inherits repo state, and B-23 can consume SBFL targets (L172-L174).

#### Inferred atoms

- **E004.E21.IN001 [INFERRED]** The PageRank output is primarily context prioritization; “agent chooses the top-ranked file” is a likely consumer behavior but not a stated mandatory selection rule (from L20, L42, L44).
- **E004.E21.IN002 [INFERRED]** The SWE-bench feature is compatibility/positioning infrastructure rather than a headline reporting reversal, which resolves—but does not explicitly settle—the E-9 tension (from L10, L14, L54).

#### Ambiguous atoms

- **E004.E21.AM001 [AMBIGUOUS]** The source locates SBFL once under `crates/auditor/src/sbfl.rs`, later under `crates/orchestrator/src/peer_systems/sbfl.rs`; one owner must be chosen (L44, L61, L95-L97, L130, L139).
- **E004.E21.AM002 [AMBIGUOUS]** The source-stated “Ochiai” numerator is `passed_executions(line)`; standard Ochiai normally emphasizes failing executions, so the formula needs source verification before coding (L44).
- **E004.E21.AM003 [AMBIGUOUS]** The fallback ladder says sequential N=3 when there is one worker but acceptance allows N=1/skip when one runtime exists; worker versus runtime availability is not normalized (L46, L120).
- **E004.E21.AM004 [AMBIGUOUS]** ACI grading ownership and the tracked-state user permission are explicit open choices that block their respective mutations (L50, L168, L170).

#### Conflict atoms

- **E004.E21.CF001 [CONFLICT]** The objective says seven improvements, while context calls six peer-proven techniques and then separately adds the benchmark; this is reconcilable counting, not a missing implementation item (L2, L12-L14).

### E-22 — May-2026 trending steals

Source: `docs/thefinalglance/E-plan/E-22_github_trending_may2026.md` (177 lines, 20,635 bytes).

#### Explicit atoms

- **E004.E22.EX001 [EXPLICIT; capability]** Export skills as normalized SKILL.md, use microsandbox/libkrun where available, enable deferred MCP schemas, audit Streamable HTTP, and benchmark `elai.map` versus `claude-context` (L1-L2).
- **E004.E22.EX002 [EXPLICIT; dependencies/status]** B-18/C-11/C-35 are planned, B-37 shipped; E-24 consumes deferred loading, while E-21/E-23/E-25 are parallel neighbors (L3-L6, L18-L23).
- **E004.E22.EX003 [EXPLICIT; replacement]** Microsandbox concretizes/replaces abstract Tier-C Linux sandbox execution while leaving B-19/B-20 hook/registry work additive (L18-L21).
- **E004.E22.EX004 [EXPLICIT; export]** `export_all(src,dst)` walks skill directories, normalizes YAML fields `name,description,version,allowed_tools?,model?`, writes `dist/skills/{name}/SKILL.md`, and is idempotent (L42).
- **E004.E22.EX005 [EXPLICIT; drift/event]** Source/dist drift is a blake3 mismatch and emits `skill_export_drift`; successful smoke emits `skill_export_completed` with positive count and empty drift (L42, L82-L85).
- **E004.E22.EX006 [EXPLICIT; layout decision]** Use `crates/skills/src/export.rs` only if that crate exists; otherwise extend the existing tools skill loader, and do not create a new crate without user confirmation (L42).
- **E004.E22.EX007 [EXPLICIT; sandbox schema]** `core::sandbox::Sandbox` has `None|Process|MicroVm` and `launch(cmd,args,opts)->Result<Child>` (L44, L61-L62).
- **E004.E22.EX008 [EXPLICIT; platform/fallback]** Linux MicroVm shells out to a verified microsandbox CLI; missing CLI degrades to Process with `sandbox_microvm_unavailable`; Windows always falls back to Process with a dated anti-stale warning (L44).
- **E004.E22.EX009 [EXPLICIT; integration]** Dashboard, task-runner, and agent-pool sandbox launch sites are wired through `Sandbox::launch` (L44, L67-L69, L94-L97).
- **E004.E22.EX010 [EXPLICIT; lazy schemas]** `McpBridge.lazy_schemas=true` makes `tools/list` return only name and a description of at most 140 chars; `tools/get_schema` retrieves the full input schema on demand (L46).
- **E004.E22.EX011 [EXPLICIT; observability]** Lazy-schema measurement logs pre/post byte counts for combined local-rag/pod-lifecycle and emits `mcp_lazy_schema_savings` (L46).
- **E004.E22.EX012 [EXPLICIT; coordination]** If C-35 already implements lazy loading, E-22 adds only regression coverage and the SSB row rather than duplicating it (L46).
- **E004.E22.EX013 [EXPLICIT; transport]** If the bridge is SSE-only, add `transport: streamable_http`: JSON-RPC request bodies use a single POST endpoint and responses may stream one-way SSE; negotiation emits `mcp_transport_negotiated` (L48).
- **E004.E22.EX014 [EXPLICIT; benchmark]** The `elai.map`/`claude-context` audit compares symbol latency and context-token cost on the ELAI tree, uses a temporary microVM, has per-query data and a keep/deprecate/hybrid verdict, and is not a refactor (L50).
- **E004.E22.EX015 [EXPLICIT; audit]** `12-factor-audit-2026-05-19.md` has one PASS/PARTIAL/FAIL row per factor, soft-enforced with no fixer queue (L52).
- **E004.E22.EX016 [EXPLICIT; CLI]** `elai skills publish [--target claude|codex|cursor|vercel|agensi]` packages export output; default `claude` is identity mapping and other targets fail clearly as `not_yet_supported` until formats are verified (L54).
- **E004.E22.EX017 [EXPLICIT; API]** Produced symbols are `export_all`, `ExportReport{exported,drift}`, `Sandbox`, `Sandbox::launch`, `McpBridge.lazy_schemas`, `tools/get_schema`, and `publish_target` (L56-L65).
- **E004.E22.EX018 [EXPLICIT; reachability]** CLI offers skills export/publish and claude-context benchmark; task driving uses the sandbox; MCP `handle_list_tools` chooses lazy schemas (L66-L75).
- **E004.E22.EX019 [EXPLICIT; constraints]** No Docker fallback, full schema in lazy list, remaining SSE-only path after transport work, or unwrap in sandbox is allowed (L76-L80).
- **E004.E22.EX020 [EXPLICIT; UI]** Skill-publish status belongs in CommandWindow or a SkillsPanel under B-26 (L81).
- **E004.E22.EX021 [EXPLICIT; evidence]** Export smoke requires every source skill to appear under dist with normalized frontmatter and byte-identical body content (L82-L85).
- **E004.E22.EX022 [EXPLICIT; deterministic gates]** Checks locate the export function, sandbox enum/three call sites, lazy branch, Streamable HTTP or an explicit SSE follow-up test, both audits, and Linux microVM timing (L87-L115).
- **E004.E22.EX023 [EXPLICIT; performance gate]** Linux benchmark acceptance is p50 `<500ms` and p95 `<1000ms` over five boots; the upstream `<200ms` claim is not the gate (L112-L115).
- **E004.E22.EX024 [EXPLICIT; acceptance]** Dist coverage/hash, Linux microVM hello-world, Windows fallback event, at least 80% lazy-list byte reduction, transport support-or-documented deferral, two complete audits, Claude publish roundtrip, and no unconfirmed dependencies are required (L117-L126).
- **E004.E22.EX025 [EXPLICIT; test]** Export roundtrip preserves the SKILL.md body byte-for-byte (L130).
- **E004.E22.EX026 [EXPLICIT; test]** Platform-specific sandbox tests require Linux binary detection and Windows Process fallback plus warning (L131-L132).
- **E004.E22.EX027 [EXPLICIT; test]** MCP tests prove no `input_schema` in lazy list, on-demand retrieval, Streamable HTTP JSON-RPC, and transport negotiation (L133-L136).
- **E004.E22.EX028 [EXPLICIT; test]** CLI publish dry-run enumerates files for Claude (L135).
- **E004.E22.EX029 [EXPLICIT; artifacts]** Planned work creates one Rust skill-export module, core sandbox, skills CLI, two audits, a decision doc, and dist output; it modifies bridge, three launch sites, CLI, and core module registration (L138-L152).
- **E004.E22.EX030 [EXPLICIT; dependency verification]** Microsandbox name/binary/license/Windows support and any crate identity must be web-verified; it is a shell-out rather than a Rust dependency (L36, L44, L126, L158).
- **E004.E22.EX031 [EXPLICIT; platform risk]** Because the user's host is Windows and libkrun is Linux-only, using the Ubuntu WSL2/remote sidecar for microVM work is an unresolved latency/isolation choice (L169).
- **E004.E22.EX032 [EXPLICIT; evidence risk]** The Tool Attention arXiv ID must be verified; if invalid, the 95% claim is ungrounded and lazy loading rests on measured token bytes instead (L165, L170).
- **E004.E22.EX033 [EXPLICIT; marketplace constraint]** Non-Claude publish formats are ingest-blocked until official formats are fetched and confirmed (L171).
- **E004.E22.EX034 [EXPLICIT; fallback risk]** The claude-context benchmark stalls without a usable Linux microVM unless the user approves remote-sidecar execution (L172).
- **E004.E22.EX035 [EXPLICIT; permission]** Whether `dist/skills/` is ignored or checked in requires user confirmation (L173).
- **E004.E22.EX036 [EXPLICIT; downstream]** E-24 consumes lazy schemas; E-25 may serialize exported skills; B-18 can reuse MicroVm instead of building another sandbox primitive (L175-L177).

#### Inferred atoms

- **E004.E22.IN001 [INFERRED]** “Replace docker/process-based sandbox” does not remove the Process enum because Process is the mandated portability/recovery fallback (from L2, L44, L61).

#### Ambiguous atoms

- **E004.E22.AM001 [AMBIGUOUS]** The skill export owner is conditional on repository layout (`crates/skills` versus `crates/tools`) and is deliberately unresolved (L42, L140).
- **E004.E22.AM002 [AMBIGUOUS]** Windows may run Process locally or MicroVm remotely through WSL2; the source does not select one (L44, L169).
- **E004.E22.AM003 [AMBIGUOUS]** Transport acceptance permits either shipped Streamable HTTP or an explicit SSE-remaining follow-up, so “done” has two materially different outcomes (L103-L105, L122).
- **E004.E22.AM004 [AMBIGUOUS]** Marketplace packaging for Codex/Cursor/Vercel/Agensi is intentionally unspecified (L54, L171).
- **E004.E22.AM005 [AMBIGUOUS]** The `dist/skills` repository/ignore policy is not authorized (L173).

#### Conflict atoms

- **E004.E22.CF001 [CONFLICT]** The objective says “replace any docker/process-based” sandbox, while Task 2 and acceptance explicitly require Process fallback; the operative reading must retain Process only as fallback (L2 versus L44, L120).
- **E004.E22.CF002 [CONFLICT]** Deterministic-check wording expects `StreamableHttp` or a follow-up test that confirms SSE remains, while the stub ban says no SSE-only path may remain “if Task 4 lands”; completion status must distinguish audit-only from implementation (L79, L103-L105).

### E-25 — Claude Code architecture steals

Source: `docs/thefinalglance/E-plan/E-25_claude_code_architecture.md` (230 lines, 29,091 bytes).

#### Explicit atoms

- **E004.E25.EX001 [EXPLICIT; capability]** Port six harness primitives—six-tier compaction, deferred MCP UI/config, sandboxed skill templating/live reload, prompt-decision hooks, FSM-substate runtime overlay, and session fork/checkpoints—while adding seven anti-pattern guards (L1-L2).
- **E004.E25.EX002 [EXPLICIT; dependencies/status]** B-37/B-22 are shipped, Phase-3H exists, B-26 is partial, E-22 owns underlying lazy schemas/sandbox, and E-24 consumes the generic prompt hook; the document is a plan, not a shipped-status rebaseline (L3-L6, L26-L34).
- **E004.E25.EX003 [EXPLICIT; compaction workflow]** `CompactionLadder` runs strategies cheapest-first; each returns `(new_messages,freed_tokens,tier_id)` and the ladder stops at the first result below target budget (L54).
- **E004.E25.EX004 [EXPLICIT; tier T1]** Budget Reduction strips whitespace and deduplicates identical tool calls (L54).
- **E004.E25.EX005 [EXPLICIT; tier T2]** Snip drops oldest tool-result payloads larger than 2KB but keeps their summaries (L54).
- **E004.E25.EX006 [EXPLICIT; tier T3]** Microcompact summarizes the three oldest user/assistant pairs into one bullet list per pair (L54).
- **E004.E25.EX007 [EXPLICIT; tier T4]** Context Collapse folds all tool results older than five turns into one “older work” summary (L54).
- **E004.E25.EX008 [EXPLICIT; tier T5]** Auto-Compact invokes the sidecar with a transcript-summary prompt preserving designated information (L54).
- **E004.E25.EX009 [EXPLICIT; tier T6]** Narrative-summary is a last-resort full sidecar re-narration (L54).
- **E004.E25.EX010 [EXPLICIT; event]** Each applied tier emits `compaction_tier_applied{tier_id,freed_tokens,wall_ms}`; unavailable sidecar tiers emit `compaction_tier_skipped` instead of silently no-oping (L54, L102).
- **E004.E25.EX011 [EXPLICIT; fallback]** Existing PreCompact `SESSION_HANDOFF` runs only after all ladder tiers fail to meet budget and is never replaced (L30, L54).
- **E004.E25.EX012 [EXPLICIT; cache behavior]** The ladder preserves system/CLAUDE.md/first-turn prefix bytes and emits `compaction_cache_preserved` when unchanged; local sidecar does not support Anthropic server-side `cache_edits` (L54).
- **E004.E25.EX013 [EXPLICIT; blocked dependency]** E-25 only adds `--lazy-mcp-schemas`, `[mcp.lazy_schemas]`, per-server tool/schema/byte metrics in `McpSchemaPanel`, and `mcp_schema_lazy_fetch`; it is BLOCKED if E-22/C-35 has not supplied lazy loading (L56).
- **E004.E25.EX014 [EXPLICIT; skill syntax]** The skill loader recognizes inline ``!`cmd` `` and triple-backtick `!` fenced blocks, executes them, and substitutes stdout before model ingestion (L58).
- **E004.E25.EX015 [EXPLICIT; skill sandbox]** Template commands deny network and writes outside the skill directory and time out after five seconds; use microsandbox if present or process isolation with `skill_template_unsandboxed` warning and state-dir jail (L58, L104).
- **E004.E25.EX016 [EXPLICIT; live reload/concurrency]** A verified `notify` watcher reloads changed SKILL.md files within one second; generation counters let in-flight calls finish with old content while new calls see new content (L58, L90, L103, L159).
- **E004.E25.EX017 [EXPLICIT; hook API]** `prompt_decision(prompt,schema,runtime)` returns `Allow|Deny(reason)|Ask(reason)` using a strict JSON-only deterministic-decision prompt through a `RuntimeAdapter` (L60).
- **E004.E25.EX018 [EXPLICIT; failure/default]** Prompt decisions time out after five seconds; malformed JSON yields `PromptHandlerError::InvalidDecisionFormat`; default runtime is local Qwen 3B via `assistant_decision` (L60).
- **E004.E25.EX019 [EXPLICIT; shared integration]** The generic handler lives in a hooks crate if present or orchestrator otherwise and is consumed by E-24's pre-MCP-call guard and future semantic hooks (L33, L60).
- **E004.E25.EX020 [EXPLICIT; runtime routing]** `RuntimeOrchestrator` gains `substate_overlay: HashMap<FsmSubstate,RoleRuntimeOverride>`; empty overlay preserves B-37 behavior (L62).
- **E004.E25.EX021 [EXPLICIT; defaults]** Framing/Planning/Scoping/Decomposing route assistant-primary to planner runtime (default cloud Sonnet); Executing/Fixing return to executor runtime (default local Qwen 3B) (L62).
- **E004.E25.EX022 [EXPLICIT; event/fallback]** Active overlays emit `runtime_substate_overlay_active`; an unavailable planner is proposed to fall back to executor with `substate_overlay_fallback` (L62, L221).
- **E004.E25.EX023 [EXPLICIT; fork lifecycle]** `elai resume --fork <new_session_id>` clones handoff, memory-bank, and FSM state into a new session, preserves the original, and records `forked_from` metadata (L64).
- **E004.E25.EX024 [EXPLICIT; checkpoint lifecycle]** PreToolUse on Edit/Write snapshots the target before mutation under `.elai_cc/.session-checkpoints/{session_id}/{seq}/{path}`; END or git commit evicts, and `elai checkpoint rewind --steps N` restores prior content (L64).
- **E004.E25.EX025 [EXPLICIT; anti-pattern guard]** No MCP/pod/local-rag startup side effects may occur before E-24's gateway-rejection check (L66).
- **E004.E25.EX026 [EXPLICIT; anti-pattern guard]** Bash with more than 50 subcommands is blocked with `bash_too_complex`, rather than bypassed (L66).
- **E004.E25.EX027 [EXPLICIT; anti-pattern guard]** More than 80% auto-approval over a rolling 50 decisions emits `approval_fatigue_detected` and forces explicit reconfirmation on the next prompt (L66).
- **E004.E25.EX028 [EXPLICIT; anti-pattern guard]** MCP search results distinguish no results, empty corpus, and unregistered topic rather than returning silent empty content (L66).
- **E004.E25.EX029 [EXPLICIT; anti-pattern guard]** Global model default stays Sonnet and every agent override appears in status, preventing invisible cheap-model delegation (L66).
- **E004.E25.EX030 [EXPLICIT; anti-pattern guard]** Bash permission must not use fragile regex argument matching; E-24's LLM guard owns semantic review (L66).
- **E004.E25.EX031 [EXPLICIT; anti-pattern guard/observability]** Auditor pass rate, ungrounded claims, and revert rate must generate SSB telemetry so quality regressions cannot remain silent (L66).
- **E004.E25.EX032 [EXPLICIT; API]** Produced surfaces are `CompactionLadder`, `CompactionResult{messages,freed_tokens,tiers_applied,cache_preserved_bytes}`, shell-injection results/watchers, prompt decision types, substate overlay, session fork, and edit-checkpoint snapshot/rewind (L68-L82).
- **E004.E25.EX033 [EXPLICIT; reachability]** CLI exposes fork/rewind/lazy flags; PreCompact invokes the ladder; hook dispatch invokes prompt handlers; task runner consumes overlays; skill load performs substitution (L83-L96).
- **E004.E25.EX034 [EXPLICIT; hard constraints]** All six tiers must be non-stub, async, unwrap-free; sidecar failure must be recorded; live reload must avoid races; template execution must check the sandbox (L97-L104).
- **E004.E25.EX035 [EXPLICIT; UI]** New panels are `CompactionLadderPanel.svelte`, `McpSchemaPanel.svelte`, and `SessionForkPanel.svelte` (L105).
- **E004.E25.EX036 [EXPLICIT; evidence]** A synthetic 30K-token transcript targeting 20K must apply at least one tier, emit nonzero freed-token/cache rows, stop immediately on budget success, and return nonempty tier history (L106-L109).
- **E004.E25.EX037 [EXPLICIT; deterministic gates]** Checks count six tier implementations, find skill injection, require prompt handler consumption, overlay/fork/rewind wiring, three named guard events, compaction/live-reload tests, and blake3-only checkpoint integrity (L111-L151).
- **E004.E25.EX038 [EXPLICIT; acceptance]** Done requires all tiers independently testable, first-success stopping, cache-byte evidence, sandboxed injection, race-safe reload, strict prompt decisions under 5s, E-24 consumption, plan/execute routing, nondestructive forking, rewindable checkpoints, seven guards, three panels, and unchanged SESSION_HANDOFF fallback (L153-L167).
- **E004.E25.EX039 [EXPLICIT; test]** Six tier-targeted subtests and a 30K→20K end-to-end test require positive savings and final budget compliance (L171-L172).
- **E004.E25.EX040 [EXPLICIT; test]** Cache-prefix, inline shell substitution, and one-second live-reload tests cover compaction/skill determinism (L173-L175).
- **E004.E25.EX041 [EXPLICIT; test]** Prompt JSON tests cover malformed error and all three decisions; overlay tests cover Planning/Executing routes (L176-L177).
- **E004.E25.EX042 [EXPLICIT; test]** Fork and rewind tests prove cloned state/original preservation and byte-restored file content (L178-L179).
- **E004.E25.EX043 [EXPLICIT; test]** A seven-case guard suite must cause each guard's SSB evidence (L180).
- **E004.E25.EX044 [EXPLICIT; artifacts]** Planned files create compaction, prompt hook, checkpoints, fork, three panels, and a decision doc; modify SessionDag, runtime, hooks, tools, CLI, resume, and Tauri commands (L182-L198).
- **E004.E25.EX045 [EXPLICIT; cloud integration risk]** Passing Anthropic `cache_edits` through only for cloud Sonnet is an open protocol-verification decision (L219).
- **E004.E25.EX046 [EXPLICIT; platform risk]** Windows filesystem watching must be verified, especially on network drives (L220).
- **E004.E25.EX047 [EXPLICIT; storage risk]** Fork clone strategy (full copy versus hardlink/COW) and checkpoint eviction/compression limits are unresolved (L222-L223).
- **E004.E25.EX048 [EXPLICIT; config risk]** Approval-fatigue and Bash-complexity thresholds are guesses and may need config/escape hatches (L224, L226).
- **E004.E25.EX049 [EXPLICIT; sandbox fallback]** Skill templating may initially ship with weaker process isolation if E-22 MicroVm has not landed, with explicit warning (L225).
- **E004.E25.EX050 [EXPLICIT; downstream]** E-21 state benefits from fork, E-22 lazy loading gains UI, E-24 consumes prompt hooks, and E-23 can consume compaction/runtime telemetry (L228-L230).

#### Inferred atoms

- **E004.E25.IN001 [INFERRED]** Tiers may be applied cumulatively even though the ladder “stops at first tier” because the result reports `tiers_applied`; each successive tier therefore operates on the previous tier's output (from L54, L73, L109).
- **E004.E25.IN002 [INFERRED]** “Deterministic decision-maker” describes output protocol, not deterministic model sampling, because no seed/temperature setting is specified (from L60).

#### Ambiguous atoms

- **E004.E25.AM001 [AMBIGUOUS]** The exact token estimator and target-budget source for ladder decisions are absent (L54).
- **E004.E25.AM002 [AMBIGUOUS]** It is unclear whether cloud-only `cache_edits` should be emitted or whether prefix preservation is the sole cross-runtime behavior (L54, L219).
- **E004.E25.AM003 [AMBIGUOUS]** Planner-offline fallback is proposed but not locked as an acceptance rule (L221).
- **E004.E25.AM004 [AMBIGUOUS]** Fork copy strategy and checkpoint retention/size controls are unresolved (L222-L223).
- **E004.E25.AM005 [AMBIGUOUS]** The 80%/50 approval window and 50-subcommand ceiling may be configurable; no final config keys are named (L224, L226).
- **E004.E25.AM006 [AMBIGUOUS]** The hook module location depends on whether `crates/hooks` exists (L60, L185).

#### Conflict atoms

- **E004.E25.CF001 [CONFLICT]** The task says checkpoints are evicted on END or git commit, while the risk section asks whether to keep the last ten or until END; retention needs one authoritative rule (L64 versus L223).

### E-29 — Agentic observability

Source: `docs/thefinalglance/E-plan/E-29_agentic_observability.md` (209 lines, 25,312 bytes).

#### Explicit atoms

- **E004.E29.EX001 [EXPLICIT; capability]** Keep SSB canonical while exposing an OTel GenAI 1.40.0 view, transcript-first dashboard, checkpoint replay branch, and MAST failure classifier (L1-L2, L10-L18).
- **E004.E29.EX002 [EXPLICIT; dependencies/status]** B-2 is shipped; B-19 is nonblocking but must coordinate schema columns; C-44/D-9 are planned and replay may not bypass them; E-30/E-24/E-25 are parallel (L3-L6, L22-L31).
- **E004.E29.EX003 [EXPLICIT; schema]** Add nullable `otel_op_name,otel_provider,otel_agent_id,otel_agent_name,otel_workflow_name,otel_tool_name,otel_request_model,otel_response_finish_reason,otel_usage_input_tokens,otel_usage_output_tokens,otel_cache_creation_input_tokens,otel_cache_read_input_tokens,otel_conversation_id,otel_latency_ms,otel_error_type` to `ssb_events` (L49).
- **E004.E29.EX004 [EXPLICIT; migration]** Migration is additive only and bumps schema version; action types map to OTel names such as phase transition→`invoke_workflow`, runtime fallback→`chat`, hooks→`execute_tool` (L49).
- **E004.E29.EX005 [EXPLICIT; exporter]** New `crates/ssb-otel-export` reads SSB and emits OTLP gRPC via verified OpenTelemetry crates (L51).
- **E004.E29.EX006 [EXPLICIT; span hierarchy]** Parent IDs come from checkpoint/phase columns and hierarchy is workflow phase→agent→sibling chat/tool spans (L51).
- **E004.E29.EX007 [EXPLICIT; privacy/default]** Content export is gated by `ELAI_OTEL_TRACE_CONTENT=false` by default; input/output messages are absent when unset/false (L51, L96, L141).
- **E004.E29.EX008 [EXPLICIT; CLI]** `--otlp-endpoint http://...` configures exporter startup (L51, L89).
- **E004.E29.EX009 [EXPLICIT; UI]** `TranscriptDrawer.svelte` renders chronological prompt/tool-use/tool-result rows as role bubbles, expandable tool calls, and monospace code output (L53).
- **E004.E29.EX010 [EXPLICIT; query schema]** `ssb_transcript(session_id,filter={role,action_type,window})` supports action whitelist, time window, role, and free-text search (L53).
- **E004.E29.EX011 [EXPLICIT; replay schema]** `ssb_resume_from(checkpoint_id,patch)` emits `replay_resume` with checkpoint ID, optional prompt/model/tool overrides, and a new replay session ID (L55).
- **E004.E29.EX012 [EXPLICIT; replay workflow]** Replay loads checkpoint state, applies a frozen-surface-validated patch, reruns into a new sibling replay session, and renders a RESEARCH-mode orbit branch with deltas (L55).
- **E004.E29.EX013 [EXPLICIT; replay safety/fallback]** C-44/D-9 patches must fail closed if predicates are not yet available; rejected patches emit an existing/equivalent frozen-surface rejection row (L55, L95, L143, L205).
- **E004.E29.EX014 [EXPLICIT; replay limitation]** Clock/RNG interception and sidecar KV-state capture are out of scope; replay is explicitly best-effort until structural-equivalence measurements exist (L55, L67, L200).
- **E004.E29.EX015 [EXPLICIT; classifier]** `classify_closed_run(session_id)` returns `MastModeHit` across 14 cause modes and runs as a post-run agent (L57).
- **E004.E29.EX016 [EXPLICIT; MAST schema]** Specification modes: `TaskMisinterpretation,GoalConflict,MissingConstraint,ScopeCreep,AmbiguousInstruction`; Coordination: `ContextCollapse,RoleConfusion,HandoffFailure,ResourceContention,DependencyMismatch,ProtocolViolation`; Verification: `IncorrectVerification,PrematureTermination,MissingValidation` (L57).
- **E004.E29.EX017 [EXPLICIT; event/UI]** `failure_classified{mode,evidence_row_ids,confidence}` feeds a Failure-modes category/phase sparkline (L57).
- **E004.E29.EX018 [EXPLICIT; signals]** Opt-in `ssb_backfill_signals(label,query)` uses an LLM to write `signal_match{signal_label,matched_row_ids,classifier_model,confidence}` over historical SSB (L59).
- **E004.E29.EX019 [EXPLICIT; cost]** Per-stage rollup attributes prompt-cache creation/read tokens and USD by phase/agent/tool using provider pricing; Spend drawer is read-only (L61).
- **E004.E29.EX020 [EXPLICIT; eval lifecycle]** `ssb_promote_to_dataset(trace_ids,dataset_name)` writes `eval_dataset_member{trace_id,dataset_name,label,scoring_rubric}` (L63).
- **E004.E29.EX021 [EXPLICIT; config/gate]** Optional `ELAI_EVAL_GATE_REGRESSION_THRESHOLD=0.05` runs during COMMITTING before push; the useful threshold is otherwise a manual knob (L63, L202).
- **E004.E29.EX022 [EXPLICIT; SQL/security]** QueryDrawer provides ten starter queries and free-form SQL through a read-only SQLite connection that forbids INSERT/UPDATE/DELETE (L65).
- **E004.E29.EX023 [EXPLICIT; decision constraints]** The decision doc codifies SSB-as-source/OTel-as-view, content-off default, cause-vs-symptom taxonomy separation, frozen replay checks, and no vendor-specific APM integration (L67).
- **E004.E29.EX024 [EXPLICIT; API/surfaces]** Produced artifacts include exporter, MAST module, 15 columns, four drawers, four Tauri commands, and action types `ReplayResume,FailureClassified,SignalMatch,EvalDatasetMember` (L69-L80).
- **E004.E29.EX025 [EXPLICIT; integration]** Existing SSB writer/viewer, hook dispatcher, runtime orchestrator, and frozen predicates are consumers/attach points (L81-L86).
- **E004.E29.EX026 [EXPLICIT; reachability]** Dashboard transcript/replay, CLI OTLP execution, and post-run MAST classification are the user/runtime paths (L87-L91).
- **E004.E29.EX027 [EXPLICIT; hard constraints]** No OTel-later or deferred-classifier stub, frozen-surface bypass, or content emission under false/unset gating is allowed (L92-L96).
- **E004.E29.EX028 [EXPLICIT; evidence]** A small task with content disabled must populate workflow OTel columns, export conformant spans, show transcript, create replay evidence, classify failure, and refuse frozen patches (L98-L101).
- **E004.E29.EX029 [EXPLICIT; deterministic gates]** Checks verify columns/version, exporter build, 14-mode enum, four Tauri registrations/actions/drawers, frozen predicate references, and decision doc (L103-L135).
- **E004.E29.EX030 [EXPLICIT; acceptance]** All 15 columns must be nullable/additive and fixture spans must validate at an OTel collector (L139-L141).
- **E004.E29.EX031 [EXPLICIT; acceptance]** Transcript ordering/UI, frozen replay refusal, nonempty classification on a known failed run, exhaustive 14 modes, and category sparkline are required (L142-L145).
- **E004.E29.EX032 [EXPLICIT; acceptance]** Spend cache attribution, ten working read-only queries, workspace/package tests, and Svelte/Tauri cross-language sync must pass (L146-L149).
- **E004.E29.EX033 [EXPLICIT; test]** OTLP bytes are validated against collector schema; contracts assert all 15 nullable columns (L153-L154).
- **E004.E29.EX034 [EXPLICIT; test]** Replay contract rejects frozen patches; transcript contract preserves temporal order (L155, L157).
- **E004.E29.EX035 [EXPLICIT; test]** MAST tests exhaust all 14 variants and classify one fixture from each category (L156).
- **E004.E29.EX036 [EXPLICIT; test]** Content-gating contract proves messages are absent by default (L158).
- **E004.E29.EX037 [EXPLICIT; artifacts]** Planned changes create exporter, MAST, four drawers, decision doc, possibly pricing; modify SSB/viewer/regression/hooks/runtime/Tauri/CLI/workspace (L160-L177).
- **E004.E29.EX038 [EXPLICIT; scaling]** Export defaults to batching and tail sampling because HEAVY span volume may exceed 10K; environment knobs are required (L198).
- **E004.E29.EX039 [EXPLICIT; validation risk]** MAST should be tested on ten closed ELAI sessions before its drawer is default-visible because FSM-specific modes may not fit the taxonomy (L199).
- **E004.E29.EX040 [EXPLICIT; replay measurement]** Measure structural-equivalence on 20 replayed traces because uncaptured KV cache can change output (L200).
- **E004.E29.EX041 [EXPLICIT; observability cost]** Full content overhead is unknown, which is why content capture defaults off (L201).
- **E004.E29.EX042 [EXPLICIT; dependency verification]** `opentelemetry-otlp` crate identity/version must be verified before pinning (L203).
- **E004.E29.EX043 [EXPLICIT; schema coordination]** The plan predicts a version around 46 based on B-41=45/B-42=44 and requires coordination with concurrent schema work (L204).
- **E004.E29.EX044 [EXPLICIT; future]** KV capture, clock/RNG interception, and MAST-specific alerts are future work (L207-L209).

#### Inferred atoms

- **E004.E29.IN001 [INFERRED]** OTel columns are denormalized aliases on each SSB row rather than a separate normalized span table, because the plan explicitly preserves SSB and extends its row schema (from L10, L49).
- **E004.E29.IN002 [INFERRED]** Signal backfill and eval promotion mutate SSB despite the Query drawer being read-only; read-only applies only to arbitrary SQL, not typed Tauri commands (from L59, L63, L65).

#### Ambiguous atoms

- **E004.E29.AM001 [AMBIGUOUS]** Exact `action_type`→OTel operation mapping is illustrated but not exhaustively enumerated (L49).
- **E004.E29.AM002 [AMBIGUOUS]** Parent resolution from checkpoint/phase IDs does not specify behavior for missing or multiply matching parents (L51).
- **E004.E29.AM003 [AMBIGUOUS]** Model pricing source/update cadence is unspecified and `pricing.rs` may or may not already exist (L61, L176).
- **E004.E29.AM004 [AMBIGUOUS]** MAST classification model, prompt, thresholds, and confidence calibration are not specified (L57).
- **E004.E29.AM005 [AMBIGUOUS]** Replay “patch” validation lacks a concrete `ToolOverride` schema and frozen-predicate API until C-44/D-9 lands (L55, L86, L205).
- **E004.E29.AM006 [AMBIGUOUS]** Tail-sampling policy and environment knob names are not defined (L198).

#### Conflict atoms

- **E004.E29.CF001 [CONFLICT]** The plan's likely schema bump to 46+ is stale against E-5's later explicit lock/current version 103; E-29 must allocate from the actual schema baseline rather than use 46 (E-29 L26, L49, L204; E-5 L16).

### E-37 — Prompt-injection defense

Source: `docs/thefinalglance/E-plan/E-37_prompt_injection_defense.md` (277 lines, 33,592 bytes).

#### Explicit atoms

- **E004.E37.EX001 [EXPLICIT; security capability]** Close the unguarded data plane—web results, extracted content, uploads, tool results, images, and sandbox output—while retaining existing control-plane visibility/schema/deny-list defenses (L1-L2, L8-L10, L23-L31).
- **E004.E37.EX002 [EXPLICIT; dependencies/status]** E-24 must land first for MCP schema pin/sanitization/transport, C-11 supplies Tier-C isolation, and B-37 supplies the classifier route; E-36/E-39/E-40 are parallel (L3-L6, L25-L31).
- **E004.E37.EX003 [EXPLICIT; architecture]** Defense must be layered: Spotlighting, per-tool description pin, off-model classifier, provenance/capability control, ingress/egress filtering, sandbox network denial, MCP-list classification, CI regression, and output classification (L12-L21, L52-L72).
- **E004.E37.EX004 [EXPLICIT; hashing constraint]** All tool-description pins use blake3, never SHA-256/blake2/sha2, overriding the source report's transport-hash suggestion (L1-L2, L19, L56, L115).
- **E004.E37.EX005 [EXPLICIT; ingest transform]** `wrap_untrusted` applies a per-request random delimiter and whitespace datamarking to every web-search, extraction, MCP-result, upload, and sandbox-stdout span (L54).
- **E004.E37.EX006 [EXPLICIT; prompt/render behavior]** The system prompt says Spotlighted instructions are data, not commands; the transform is reversible only for presentation and the delimiter is never echoed in user-visible output (L54, L122).
- **E004.E37.EX007 [EXPLICIT; event]** Every wrap emits `spotlight_envelope_applied` (L54).
- **E004.E37.EX008 [EXPLICIT; pin storage]** First tool approval stores `blake3(tool.description_canonical)` keyed by server URL/tool name under `.elai_cc/.elai/mcp/tool_descriptions/...toml`, including `pinned_hash,pinned_at,approval_user` (L56).
- **E004.E37.EX009 [EXPLICIT; drift/recovery]** Later description mismatch blocks by default, shows the diff, emits a pin violation, and requires explicit `elai mcp pin update <server> <tool>` re-pinning (L56, L89).
- **E004.E37.EX010 [EXPLICIT; composability]** Per-description pinning complements rather than replaces E-24's canonical whole-list/schema pin: one catches mutation, the other tool addition/removal (L56, L100-L101).
- **E004.E37.EX011 [EXPLICIT; classifier]** `prompt_armor::classify` uses role `injection_classifier`, default local Qwen 3B with optional Haiku 4.5 fallback, after Spotlighting and before ingest tool use (L26, L58).
- **E004.E37.EX012 [EXPLICIT; classifier schema]** Classifier returns Clean/Suspicious/Block, confidence, and matched pattern IDs (L58, L81-L82).
- **E004.E37.EX013 [EXPLICIT; classifier workflow]** Block halts with an agent error and event; Suspicious asks the user unless autonomous plus `--allow-suspicious-ingest`, in which case it logs and proceeds (L58).
- **E004.E37.EX014 [EXPLICIT; efficiency]** Classification batches about 16–32 spans per sidecar call using cheap-monitor representation reuse (L58).
- **E004.E37.EX015 [EXPLICIT; provenance schema]** `Provenance` has `User,System,WebTierA,WebTierB,WebTierC,McpTool{server_url,tool_name,desc_blake3},Upload{source_path},SandboxOutput{sandbox_id},Memory{source_key}` and wraps values as `Provenanced<T>` (L60, L77-L80).
- **E004.E37.EX016 [EXPLICIT; security invariant]** Provenance is out-of-band metadata and therefore cannot be forged by text; every argv-construction site is intended to wrap values (L60).
- **E004.E37.EX017 [EXPLICIT; capability gate]** PreToolUse rejects Bash/filesystem/database/network mutation whenever any argument derives from WebTier or McpTool and records the tool, provenanced argv, and rejecting origin (L62).
- **E004.E37.EX018 [EXPLICIT; rationale/non-goal]** CaMeL-lite is origin-based rather than regex-based; full capability calculus is deferred to E-37.B (L21, L62, L277).
- **E004.E37.EX019 [EXPLICIT; egress config]** WebFetcher enforces `[network.egress] allowlist`, defaulting to normal search/arXiv/Hugging Face/GitHub traffic, and emits `egress_blocked` for other destinations (L64).
- **E004.E37.EX020 [EXPLICIT; output sanitization]** T1 PostToolUse strips Markdown image syntax and HTML `<img>` before rendering unless hostname allowlisted (L64).
- **E004.E37.EX021 [EXPLICIT; UI security]** Tauri CSP denies `img-src` from non-allowlisted origins (L64).
- **E004.E37.EX022 [EXPLICIT; sandbox security]** Tier-C adds default-deny network namespaces and DNS allowlist; every lookup emits `sandbox_egress_dns_lookup{tool,host,decision}` (L66).
- **E004.E37.EX023 [EXPLICIT; MCP scan]** Every full `list_tools()` description/parameter/return-schema set is classified in `mcp_list_audit` mode after E-24 text sanitization; Block rejects the whole list and emits `mcp_scan_classifier_blocked` (L68).
- **E004.E37.EX024 [EXPLICIT; CI gate]** Nightly AgentDojo runs 97 tasks×629 security tests against all layers and gates at attack-success rate `<5%`, after package/repository identity verification (L70).
- **E004.E37.EX025 [EXPLICIT; output classifier]** Before SSB persistence, a local-first classifier detects non-allowlisted URLs, Markdown/HTML/script exfil, and role-tag impersonation and emits `output_classifier_flagged` (L72, L273).
- **E004.E37.EX026 [EXPLICIT; API]** Produced types include `SpotlightedSpan{raw,datamarked,delimiter:[u8;16]}`, provenance wrappers, classifier/gate/output verdicts, pin status, and CLI pinning (L74-L89).
- **E004.E37.EX027 [EXPLICIT; events]** Nine action variants are `SpotlightEnvelopeApplied,PromptArmorBlocked,PromptArmorSuspicious,CapabilityGateRejected,EgressBlocked,OutputClassifierFlagged,McpScanClassifierBlocked,ToolDescriptionPinViolation,SandboxEgressDnsLookup` (L90-L98).
- **E004.E37.EX028 [EXPLICIT; reachability]** Fetcher/extractor wrap outputs; MCP list verifies/classifies; MCP calls wrap/classify results; task runner tags/gates/sanitizes; CLI updates pins (L99-L113).
- **E004.E37.EX029 [EXPLICIT; hard constraints]** No system-prompt-only or single/input-only defense, unallowlisted rendered image, regex-as-sandbox, leaked delimiter, new unwrap, or non-blake3 hash is allowed (L114-L122).
- **E004.E37.EX030 [EXPLICIT; UI]** `SafetyDrawer.svelte` shows Spotlight counts, classifier verdicts, egress blocks, and output flags (L123).
- **E004.E37.EX031 [EXPLICIT; evidence]** A hostile HTML fixture must produce Spotlight and PromptArmor evidence, prevent attacker-local traffic, flag any output bypass, persist tool pins, and maintain AgentDojo ASR below 5% (L124-L127).
- **E004.E37.EX032 [EXPLICIT; deterministic gates]** Checks enforce blake3-only pinning, seven safety/plugin modules, at least five ingest wrap sites, provenance threading, at least three destructive gate sites, nine events, output stripping, fetcher allowlist, CI file, and classifier role (L129-L181).
- **E004.E37.EX033 [EXPLICIT; acceptance]** Every ingest source must be randomly Spotlighted; description drift must block; PromptArmor Block/Suspicious branching and all-argv provenance/capability rejection must work (L183-L189).
- **E004.E37.EX034 [EXPLICIT; acceptance]** Web/dashboard/sandbox egress controls, list-tools classification, output classification, nightly `<5%` ASR, all nine events, blake3-only code, and at least four composed layers are required (L190-L197).
- **E004.E37.EX035 [EXPLICIT; test]** Spotlight contract covers five paths, randomized delimiter, and provenance attachment (L201).
- **E004.E37.EX036 [EXPLICIT; test]** Pin contract covers initial pin, mutated-block/diff, and explicit update re-pin (L202).
- **E004.E37.EX037 [EXPLICIT; test]** PromptArmor tests require five malicious patterns all Block and five clean fixtures all Clean (L203).
- **E004.E37.EX038 [EXPLICIT; test]** Capability test rejects WebTier-tainted `rm` but permits user-origin equivalent (L204).
- **E004.E37.EX039 [EXPLICIT; test]** Egress and sanitizer tests block attacker host and remove Markdown image, HTML image, reference link, data URI, and JavaScript patterns (L205-L206).
- **E004.E37.EX040 [EXPLICIT; test]** MCP list, output classifier, sandbox DNS, and sampled AgentDojo tests verify each outer layer/event (L207-L210).
- **E004.E37.EX041 [EXPLICIT; artifacts]** Planned work creates a safety crate with six modules, tool pin module, SafetyDrawer, CI workflow, ten contracts, and decision doc; modifies MCP, web, task runner/runtime/SSB/CLI/CSP/config/sandbox (L212-L237).
- **E004.E37.EX042 [EXPLICIT; supply-chain gate]** AgentDojo package/repository/version/publication metadata must be verified before CI installation (L265).
- **E004.E37.EX043 [EXPLICIT; performance gate]** Spotlight/classifier overhead must stay below 1.5× on a typical session or move toward asynchronous/sampled operation (L266).
- **E004.E37.EX044 [EXPLICIT; rollout gate]** Run PromptArmor over 200 real ingest spans before defaulting to Block rather than Suspicious-only (L267).
- **E004.E37.EX045 [EXPLICIT; scope fallback]** If argv wrapping exceeds 50 construction sites, first ship ingest-path coverage and defer the remainder to E-37.B (L268).
- **E004.E37.EX046 [EXPLICIT; UX config]** Default Markdown-image allowlist includes GitHub, Hugging Face, arXiv, and mermaid.live, with per-project override (L269).
- **E004.E37.EX047 [EXPLICIT; sequencing]** If E-24 stalls, description pinning and MCP-list scanning are gated; C-11 and D-13 integrations must avoid duplicate primitives (L271-L272).
- **E004.E37.EX048 [EXPLICIT; cost/default]** Output classification defaults to free local sidecar, using cloud only for `[safety.output_classifier] high_stakes=true` (L273).
- **E004.E37.EX049 [EXPLICIT; future/non-goal]** Full CaMeL, StruQ finetuning, and MELON masked re-execution are future work, not current sidecar scope (L275-L277).

#### Inferred atoms

- **E004.E37.IN001 [INFERRED]** The `images` gap in the objective is addressed at output/render egress and future upload ingest, not by a vision-specific classifier schema in this plan (from L2, L54, L64).

#### Ambiguous atoms

- **E004.E37.AM001 [AMBIGUOUS]** Exact datamarking token/encoding and display reversal algorithm are not specified (L54).
- **E004.E37.AM002 [AMBIGUOUS]** “Explicit approval” identity/storage is named for pins but authentication/authorization of `approval_user` is not (L56).
- **E004.E37.AM003 [AMBIGUOUS]** Classifier confidence thresholds separating Clean/Suspicious/Block are absent (L58).
- **E004.E37.AM004 [AMBIGUOUS]** DNS enforcement through allowlist “hooks” does not define IP pinning, redirects, or DNS rebinding behavior (L66).
- **E004.E37.AM005 [AMBIGUOUS]** Full provenance coverage may be narrowed if more than 50 call sites exist, so the acceptance demand of every argv can become a follow-up without an explicit revised done gate (L188, L268).

#### Conflict atoms

- **E004.E37.CF001 [CONFLICT]** None internal; the source consistently makes E-24-first sequencing a gate rather than claiming independent completion.

### E-41 — CI/CD and PR review

Source: `docs/thefinalglance/E-plan/E-41_ci_cd_pr_review.md` (200 lines, 23,983 bytes).

#### Explicit atoms

- **E004.E41.EX001 [EXPLICIT; capability]** Establish four defense legs—pre-commit/in-flow, PR-open, deploy preview, and post-merge observation—with cross-family review and GitHub PR operations (L1-L2, L10-L16).
- **E004.E41.EX002 [EXPLICIT; dependencies/status]** C-40 is planned, C-23/B-37/B-9 are shipped, E-42/E-44/E-45/E-22 are parallel, and E-41 may supersede/absorb C-40 rather than duplicate it (L3-L6, L18-L25, L195-L200).
- **E004.E41.EX003 [EXPLICIT; merge policy]** Multi-file PRs require at least one human approval plus external-reviewer PASS, dismiss stale reviews on new pushes, and preserve linear history (L16).
- **E004.E41.EX004 [EXPLICIT; adapter fallback]** If C-40 has not landed, create feature-gated `github_adapter` with both `GhCliAdapter` and `RestApiAdapter` (L42).
- **E004.E41.EX005 [EXPLICIT; auth/security]** CLI adapter verifies official `gh` version/auth at startup, emits `github_adapter_auth_ok`, and fails actionably; REST uses `GH_TOKEN` (L42).
- **E004.E41.EX006 [EXPLICIT; API]** Adapter supports `open_pr`, `post_review_comment`, `list_review_comments`, and `set_pr_status` (L42, L63-L65).
- **E004.E41.EX007 [EXPLICIT; heterogeneous review]** `AgentRole::ExternalPrReviewer` must route to a different backend than Executor and receives the diff, brief, and in-flow auditor verdict (L44).
- **E004.E41.EX008 [EXPLICIT; review schema/events]** External review emits started/verdict rows; verdict is `PASS|BLOCK|REQUEST_CHANGES` plus structured comments (L44).
- **E004.E41.EX009 [EXPLICIT; validation artifact]** Compare at least 20 PRs against an industry reviewer and record baseline agreement κ in `E-41-cross-family-agreement-2026-05-19.md` (L44).
- **E004.E41.EX010 [EXPLICIT; CI order]** Semgrep AI rules run after in-flow PASS and before external review, failing banned tokens/secrets/unsafe LLM settings (L46).
- **E004.E41.EX011 [EXPLICIT; rollout]** Semgrep emits severity counts and runs soft-mode for the first ten PRs, preceded by a retrospective dry-run over the last 100 commits (L46).
- **E004.E41.EX012 [EXPLICIT; dependency security]** A new import absent from Cargo.toml/pyproject/package.json invokes `verify_package(name,ecosystem)` before the write commits (L48).
- **E004.E41.EX013 [EXPLICIT; supply-chain failure]** Verification queries crates.io/PyPI/npm and rejects missing packages, no release within 18 months, or owner/author mismatch; emits `dependency_verification_result` (L48).
- **E004.E41.EX014 [EXPLICIT; commit lifecycle]** `git_assist` squashes intra-step fix/WIP commits, preserves one commit per baby step labeled `<plan-id>/<phase>/<step>`, then merges plan-close with `git merge --no-ff` (L50).
- **E004.E41.EX015 [EXPLICIT; attribution]** Merge commit trailers are `Generated-By: elai/<author-role>/<model-id>` and `Reviewed-By-Bot: <reviewer-role>/<model-id>`; no AI attribution or Co-Authored-By appears in subject/body (L25, L52, L86).
- **E004.E41.EX016 [EXPLICIT; CODEOWNERS]** Bootstrap scans planned files and human ownership from projectBrief to generate CODEOWNERS; default output is ignored and real enforcement requires `elai pr config --enforce-codeowners` (L54).
- **E004.E41.EX017 [EXPLICIT; auto-merge]** Multi-file or `>100 LOC` PRs cannot auto-merge; exception requires machine-generated reversible change, `<100 LOC`, green CI, and all paths unowned (L56).
- **E004.E41.EX018 [EXPLICIT; permission]** All other cases emit `auto_merge_blocked_reason` and require human approval (L56).
- **E004.E41.EX019 [EXPLICIT; flaky recovery]** Failures classify as `bug|flaky|env`; flaky tests move to `tests/quarantine/` with `will_resolve_in` ownership and a two-sprint TTL, then re-block merge (L58).
- **E004.E41.EX020 [EXPLICIT; events]** Named action types are `PrExternalReviewStarted,PrExternalReviewVerdict,SemgrepAiFindings,DependencyVerificationResult,CodeownersGenerated,AutoMergeBlockedReason,GithubAdapterAuthOk` (L67-L73).
- **E004.E41.EX021 [EXPLICIT; CLI]** `elai pr` adds `open|close|review-comment|set-status` (L75, L82).
- **E004.E41.EX022 [EXPLICIT; workflow]** Plan-close composes merge then opens PR; auditor PASS dispatches the external reviewer (L81-L84).
- **E004.E41.EX023 [EXPLICIT; hard constraints]** No attribution in message text, production unwrap, unverified dependency, default multi-file auto-merge, or same-family reviewer is allowed (L85-L90).
- **E004.E41.EX024 [EXPLICIT; optional UI]** `PrStatusPanel.svelte` may show PR number, reviewer verdict, dependency checks, and Semgrep findings but is optional behind B-26 (L91).
- **E004.E41.EX025 [EXPLICIT; evidence]** Draft-PR smoke requires auth→review-start→verdict evidence, a real draft body with mutation/wiring/stub/reviewer fields, and merge-only attribution trailers (L92-L95).
- **E004.E41.EX026 [EXPLICIT; deterministic gates]** Checks require all four adapter primitives, routed reviewer role, seven events, package hook, `--no-ff`, attribution exclusion, backend mismatch assertion, and 20-row audit (L97-L131).
- **E004.E41.EX027 [EXPLICIT; acceptance]** Adapter builds feature-gated and mock/live PR roundtrips are separated by `GH_TOKEN` (L133-L136).
- **E004.E41.EX028 [EXPLICIT; acceptance]** Reviewer backend mismatch, first-ten Semgrep soft fail, and hallucinated-package block are mandatory (L137-L139).
- **E004.E41.EX029 [EXPLICIT; acceptance]** Bisectable baby-step commits, trailer-only attribution, opt-in CODEOWNERS, and auto-merge block for `>100 LOC`/two files/owned paths are required (L140-L143).
- **E004.E41.EX030 [EXPLICIT; acceptance]** A deterministic injected flake must move to quarantine with a resolution marker (L144).
- **E004.E41.EX031 [EXPLICIT; test]** Adapter tests inspect dry-run `gh pr create` and REST review-comment JSON (L148-L149).
- **E004.E41.EX032 [EXPLICIT; test]** Orchestrator tests enforce backend mismatch and reject hallucinated package names in all three ecosystems (L150-L151).
- **E004.E41.EX033 [EXPLICIT; test]** Git tests prohibit message attribution and prove baby-step commits remain bisectable (L152-L153).
- **E004.E41.EX034 [EXPLICIT; test]** Auto-merge test blocks 101 LOC and allows 50-LOC/single-file/unowned; CLI and feature contracts cover dry-run/default-build isolation (L154-L156).
- **E004.E41.EX035 [EXPLICIT; artifacts]** Planned work creates adapter crate, review watcher, git assist, verifier, cross-family audit, decision doc, and optional panel; modifies SSB, roles, auditor, CLI, workspace (L158-L172).
- **E004.E41.EX036 [EXPLICIT; rollout threshold]** If retroactive Semgrep would block over 20% of 100 commits, extend soft mode to prevent gate fatigue (L191).
- **E004.E41.EX037 [EXPLICIT; reviewer fallback]** If local 3B agreement κ is below `0.5` on 20 PRs, self-hosted review is rejected in favor of paid SaaS or a cloud reviewer (L192).
- **E004.E41.EX038 [EXPLICIT; timeout config]** Measure reviewer p50/p95 and expose `pr_external_review_timeout_ms` if it adds more than ten minutes to close (L193).
- **E004.E41.EX039 [EXPLICIT; platform failure]** Windows `gh` path/auth is checked before any PR subcommand and failure is actionable (L194).
- **E004.E41.EX040 [EXPLICIT; server-side security]** Auto-merge blocking must execute in CI because `git push --no-verify` bypasses local hooks (L196).
- **E004.E41.EX041 [EXPLICIT; downstream]** E-45 drift becomes a required PR status before review; C-40 should be deprecated as a separate phase after E-41's strict superset lands (L198-L200).

#### Inferred atoms

- **E004.E41.IN001 [INFERRED]** Deploy-preview and post-merge legs mostly orchestrate already-shipped/planned B-42 and observability surfaces; this file's concrete new code is concentrated in pre-commit/PR-open controls (from L12, L20-L24, L40-L58).
- **E004.E41.IN002 [INFERRED]** “Linear history” and `--no-ff` can coexist only if the repository's policy permits merge commits; linear-history enforcement may need to mean no arbitrary merge topology rather than GitHub's strict setting (from L16, L50).

#### Ambiguous atoms

- **E004.E41.AM001 [AMBIGUOUS]** The precise four-leg state machine and which post-merge signal triggers rollback are not specified (L2, L12).
- **E004.E41.AM002 [AMBIGUOUS]** Ownership source says `projectBrief.md` but does not define its ownership schema or conflict resolution (L54).
- **E004.E41.AM003 [AMBIGUOUS]** “Machine-generated and reversible” is not formalized for the auto-merge exception (L56).
- **E004.E41.AM004 [AMBIGUOUS]** Registry owner-mismatch heuristics and network-failure retry policy are unspecified (L48).
- **E004.E41.AM005 [AMBIGUOUS]** Reviewer timeout behavior (block, retry, or human fallback) remains open despite a proposed config key (L193).
- **E004.E41.AM006 [AMBIGUOUS]** C-40 may land first, be absorbed, or be deprecated; code ownership depends on sequencing (L42, L195, L200).

#### Conflict atoms

- **E004.E41.CF001 [CONFLICT]** The source asks for stale-review dismissal plus “linear history” while also mandates `--no-ff` merge commits, which GitHub's strict linear-history rule normally forbids; repository policy must choose/translate these requirements (L16, L50).
- **E004.E41.CF002 [CONFLICT]** CODEOWNERS is said to be generated at `.github/CODEOWNERS` yet “gitignored by default”; an ignored CODEOWNERS file cannot enforce GitHub ownership until the opt-in rewrite is explicitly modeled (L54, L142).

### E-48 — Web-research orchestrator hardening

Source: `docs/thefinalglance/E-plan/E-48_web_research_orchestrator.md` (263 lines, 35,404 bytes).

#### Explicit atoms

- **E004.E48.EX001 [EXPLICIT; capability]** Extend FORMULATE→SEARCH→EXTRACT→REFLECT→CONVERGE with editable planning, three-tier search, extractor fallback, cross-encoder rerank, speculative pruning, lower-stop threshold, visible contradictions, four-stage citation checks, attribution lock, recency weighting, and dashboards (L1-L2, L8-L20).
- **E004.E48.EX002 [EXPLICIT; dependencies/status]** B-7/B-39/B-40 are shipped, C-48 is planned, and E-46/E-17/E-19/E-13 are parallel/coordination peers (L3-L6, L12-L20).
- **E004.E48.EX003 [EXPLICIT; trust lifecycle]** Web citations default to C-48 T2; SemanticCite-verified evidence may promote to T1-equivalent reuse, while failures emit verification evidence (L19).
- **E004.E48.EX004 [EXPLICIT; planning schema]** Before any search, FORMULATE emits `ResearchPlan{sub_questions,expected_information_gain_rank,planned_search_budget}` (L45).
- **E004.E48.EX005 [EXPLICIT; deterministic heuristic]** Information gain ranks lower parent-token overlap higher and adds named-entity specificity (L45).
- **E004.E48.EX006 [EXPLICIT; interaction/default]** `--interactive-research-plan` allows edit/approval; it defaults off and noninteractive runs auto-approve; plan emission and user-edited approval are journaled (L45).
- **E004.E48.EX007 [EXPLICIT; search workflow]** `TieredSearchPool` dispatches deterministically Brave T1→SearXNG T2→Mojeek T3 with per-tier quotas/rate limits (L47).
- **E004.E48.EX008 [EXPLICIT; fallback]** Quota exhaustion disables a tier with `tier_disabled_quota_exceeded`; absent T1/T3 keys degrades to existing T2-only behavior and must emit `tiered_search_degraded_to_t2_only` (L47, L101).
- **E004.E48.EX009 [EXPLICIT; dedup]** Results deduplicate by blake3 of normalized URL: lowercase scheme/host, sorted query parameters, and removed `utm_*` (L47).
- **E004.E48.EX010 [EXPLICIT; source schema]** Results carry `SourceTier::T1|T2|T3`; DuckDuckGo is explicitly forbidden as a tier (L47, L72, L100).
- **E004.E48.EX011 [EXPLICIT; config]** `[web_search.tiered]` holds Brave/Mojeek keys and existing SearXNG URL (L47).
- **E004.E48.EX012 [EXPLICIT; extraction]** `ExtractorPool` tries self-hosted Crawl4ai for 10s, then Jina Reader for 5s on timeout/JS failure (L49).
- **E004.E48.EX013 [EXPLICIT; extraction failure]** Double extractor failure emits `extraction_failed_all_extractors` and drops the source (L49).
- **E004.E48.EX014 [EXPLICIT; rerank]** After SourceQualityTable and before context packing, rerank top 50 to top 5 (L51).
- **E004.E48.EX015 [EXPLICIT; rerank defaults/fallback]** Default is local sidecar `/rerank`; configured Cohere v3.5 is fallback/opt-in, chunks at 4093 tokens, and max-pools per-source relevance (L51, L103).
- **E004.E48.EX016 [EXPLICIT; observability]** Rerank emits `cross_encoder_rerank_complete` with a rank-shift histogram (L51).
- **E004.E48.EX017 [EXPLICIT; tree schema]** Research branches become `Breadth|Depth|Speculative` and extend existing B-40 scaffolding (L53).
- **E004.E48.EX018 [EXPLICIT; pruning]** After every REFLECT, prune only Speculative branches when evidence-delta ratio `<0.10` and `low_gain_streak>2`; reuse existing `prune_speculative_children` and emit branch/delta/streak (L53).
- **E004.E48.EX019 [EXPLICIT; budget]** Speculative allocation is `min(remaining_total_budget*0.20,parent_node_estimated_value*0.5)` at each depth (L53).
- **E004.E48.EX020 [EXPLICIT; convergence]** Add `PreemptiveUnsolvable` below existing goal/quality thresholds when confidence `<0.20` after at least N rounds and evidence delta `<0.05` for the last three rounds (L55).
- **E004.E48.EX021 [EXPLICIT; calibration/fallback]** Lower threshold requires held-out unsolvable tasks; without them it defaults conservative and emits `preemptive_stop_threshold_uncalibrated` at startup (L55).
- **E004.E48.EX022 [EXPLICIT; event]** Lower-stop convergence emits `research_preemptive_stop` (L55).
- **E004.E48.EX023 [EXPLICIT; contradiction workflow]** REFLECT compares each new claim with up to the latest 50 path evidence rows using a strict-JSON LLM classifier of `agree|disagree|orthogonal` plus reason (L57).
- **E004.E48.EX024 [EXPLICIT; contradiction schema]** Disagreement creates `ContradictionNode{source_a,source_b,classification_reason,surfaced_at,prune_protected:true}` (L57, L77).
- **E004.E48.EX025 [EXPLICIT; safety invariant]** Contradictions are never silently chosen or pruned; reports must render both source/year/URL claims and `reconciliation: [unresolved]` (L57, L96, L102).
- **E004.E48.EX026 [EXPLICIT; citation stage 1]** URL HEAD liveness has a 5s timeout; 404/timeout/connection error emits `citation_dead_url` and drops the citation (L59).
- **E004.E48.EX027 [EXPLICIT; citation stage 2]** arXiv IDs match `arxiv:\d{4}\.\d{4,5}` and HEAD `arxiv.org/abs/{id}`; 404 emits `citation_invalid_arxiv` (L59).
- **E004.E48.EX028 [EXPLICIT; citation stage 3]** DOI IDs match `10\.\d+/.+` and HEAD `doi.org/{doi}`; 404 emits `citation_invalid_doi` (L59).
- **E004.E48.EX029 [EXPLICIT; citation stage 4]** Sidecar faithfulness returns `support|partial|contradict|orthogonal`; mismatch emits `citation_unfaithful` and remains user-visible rather than being auto-dropped (L59).
- **E004.E48.EX030 [EXPLICIT; type safety]** Final-report citations must be `VerifiedCitation` values produced by `CitationVerifier::verify`; verification runs after EXTRACT and again before final REPORT (L59, L97).
- **E004.E48.EX031 [EXPLICIT; metric]** Each run emits `citation_verification_pass_rate` over citations surviving all four stages (L59).
- **E004.E48.EX032 [EXPLICIT; attribution schema]** Citation authority is tuple `(agent_id,fetched_url,fetched_at_unix,source_tier)` and only the SEARCH producer may attach it (L61).
- **E004.E48.EX033 [EXPLICIT; attribution failure]** REFLECT/CONVERGE cannot cite outside the fetched set; violations emit `citation_attribution_violation_blocked` (L61).
- **E004.E48.EX034 [EXPLICIT; recency trigger]** Questions matching `latest|current|2024|2025|2026|recent|today|this year` activate time weighting (L63).
- **E004.E48.EX035 [EXPLICIT; recency formula/default]** Relevance multiplies by `exp(-ln(2)*age_days/HALF_LIFE_DAYS)`, default half-life 365 days, configurable under `[web_search.recency]`, and emits `recency_decay_applied` (L63).
- **E004.E48.EX036 [EXPLICIT; UI]** Four panels—ResearchPlan, TieredSearchHealth, CitationVerificationGauge, Contradictions—mount under ReasoningDrawer and have four matching Tauri commands (L65).
- **E004.E48.EX037 [EXPLICIT; API]** Produced surfaces include ResearchPlan, TieredSearchPool/SourceTier, ExtractorPool, CrossEncoderRerank, BranchType, PreemptiveUnsolvable, ContradictionNode, CitationVerifier/Outcome, and SourceQualityTable recency extension (L67-L80).
- **E004.E48.EX038 [EXPLICIT; integration]** Node loop owns planning/pruning/convergence/contradiction/citations; web-search owns tiers/extraction/rerank/recency; B-40 idea framing consumes the pruner (L81-L92).
- **E004.E48.EX039 [EXPLICIT; hard constraints]** No stubs/unwrap, silent contradiction, unverified final citation, lead-invented citation, non-blake3 URL hash, DDG tier, silent T2 degradation, contradiction pruning, or default Cohere call is allowed (L93-L103).
- **E004.E48.EX040 [EXPLICIT; evidence]** A “fastest Python framework in 2026” fixture with contradictory sources and `arxiv:9999.99999` must emit plan/rerank/contradiction/invalid-ID/pass-rate evidence and render the unresolved contradiction while removing or flagging the fake citation (L105-L108).
- **E004.E48.EX041 [EXPLICIT; deterministic gates]** Checks verify plan-before-search, all tiers/no DDG, two extractors, top50→top5, speculative prune, lower stop, protected/rendered contradiction, four citation fields/type, attribution lock, blake3-only URLs, and three smoke tests (L110-L168).
- **E004.E48.EX042 [EXPLICIT; acceptance]** Plan precedes search; three-tier deterministic dispatch/dedup/quota and no DDG are required (L170-L174).
- **E004.E48.EX043 [EXPLICIT; acceptance]** Extractor timeouts/fallback, local-first top50→top5 rerank, and exact speculative prune conditions must work (L175-L177).
- **E004.E48.EX044 [EXPLICIT; acceptance]** Triple convergence, protected visible contradictions, four-stage typed citations, attribution lock, recency weighting, four panels/commands, event contracts, and preserved B-7 upper thresholds are required (L178-L185).
- **E004.E48.EX045 [EXPLICIT; test]** Research-plan contract refuses SEARCH unless plan-emitted evidence exists (L189).
- **E004.E48.EX046 [EXPLICIT; test]** Search tests deduplicate UTM variants, fall back Crawl4ai→Jina, and return exactly five of fifty with rank-shift evidence (L190-L192).
- **E004.E48.EX047 [EXPLICIT; test]** Tree tests cover low-delta pruning, low-confidence unsolvable stop, unprunable contradictions, and report rendering (L193-L194, L197-L198).
- **E004.E48.EX048 [EXPLICIT; test]** Citation tests drop fake arXiv but retain/flag semantic mismatch; attribution test blocks lead invention (L195-L196, L199).
- **E004.E48.EX049 [EXPLICIT; test]** Four contract suites cover plan, tiered pool, verifier, and contradiction node (L200).
- **E004.E48.EX050 [EXPLICIT; artifacts]** Planned work creates tiered search, extractor, reranker, speculative/contradiction/verifier modules, four panels, decision doc; modifies orchestrator/tree/source-quality/config/CLI/Tauri wiring (L202-L221).
- **E004.E48.EX051 [EXPLICIT; calibration risk]** Accumulate at least 30 tagged-unsolvable cases before recalibrating the conservative lower threshold (L251).
- **E004.E48.EX052 [EXPLICIT; quota recovery]** Brave exhaustion alerts and falls back to always-available SearXNG; empty niche Mojeek results are tolerated (L252-L253).
- **E004.E48.EX053 [EXPLICIT; unresolved dependency]** If local `/rerank` is absent, either extend sidecar or choose Cohere/no-rerank behavior; the source asks the user to decide (L254).
- **E004.E48.EX054 [EXPLICIT; calibration]** The `0.10` pruning threshold is a guess; log every delta so a 50-query benchmark can retune it (L255).
- **E004.E48.EX055 [EXPLICIT; cost/override]** Faithfulness adds roughly 10–30% sidecar tokens; proposed default is enabled with `--no-faithfulness-check` opt-out (L256).
- **E004.E48.EX056 [EXPLICIT; concurrency bound]** Contradiction comparison caps at 50 rows and emits `contradiction_check_budget_clamped` when bounded (L257).
- **E004.E48.EX057 [EXPLICIT; migration]** Attribution lock is proposed only going forward; legacy rows remain unverifiable rather than backfilled by invention (L258).
- **E004.E48.EX058 [EXPLICIT; downstream]** E-49 consumes citation pass rate, E-46 can use evidence delta, and C-48 consumes verified trust promotion; calibration corpus remains deferred (L261-L263).

#### Inferred atoms

- **E004.E48.IN001 [INFERRED]** “Tier” order is a cascading/fan-out policy rather than guaranteed sequential blocking, because the plan calls it fan-out yet mandates deterministic T1→T2→T3 dispatch (from L2, L47).
- **E004.E48.IN002 [INFERRED]** A citation can be structurally valid but semantically unfaithful; this explains why stages 1–3 may drop dead identifiers while stage 4 retains a flagged citation (from L59).

#### Ambiguous atoms

- **E004.E48.AM001 [AMBIGUOUS]** `N` in the lower-stop condition is not assigned a default (L55).
- **E004.E48.AM002 [AMBIGUOUS]** Exact novelty/specificity weights and named-entity recognizer are absent (L45).
- **E004.E48.AM003 [AMBIGUOUS]** Local `/rerank` existence and no-key/no-endpoint behavior are unresolved (L51, L254).
- **E004.E48.AM004 [AMBIGUOUS]** Citation pass-rate denominator and treatment of retained partial/unfaithful citations are not specified (L59).
- **E004.E48.AM005 [AMBIGUOUS]** “SemanticCite-verified promotes to T1 codebase-derived equivalence” may conflict conceptually with C-48 provenance tiers; promotion semantics await C-48 (L19).
- **E004.E48.AM006 [AMBIGUOUS]** Existing evidence without agent IDs may be rejected or accepted as legacy; only the proposed default “enforce going forward” is stated (L258).

#### Conflict atoms

- **E004.E48.CF001 [CONFLICT]** Stage-1 says dead URLs are dropped, while the observable artifact allows the fabricated arXiv citation to be “removed or flagged dead”; the acceptance contract should choose drop for invalid/dead identifiers and flag-only for semantic mismatch (L59 versus L108).

## Cross-file interactions

- **E004.INT.EX001 [EXPLICIT]** E-5 is the memory substrate: E-1 reuses its table, E-3 reads global `voice_style`, E-4 reads `expertise`, and D-14/D-23 consume contradiction/validity evidence (E-5 L20-L21, L43-L47, L306).
- **E004.INT.EX002 [EXPLICIT]** E-5 real-valued trust is designed to unify with planned C-48 tiers, while E-48 also consumes C-48 for citation trust; both need one canonical trust-promotion rule (E-5 L41; E-48 L19).
- **E004.INT.EX003 [EXPLICIT]** E-8 depends on E-7's MAP-Elites/Two-Gate and supplies optimized SHADOW templates; E-8 also becomes E-9's baseline-vs-optimized benchmark input (E-8 L50, L58, L64, L212-L213).
- **E004.INT.EX004 [EXPLICIT]** E-20 routing and D-10 compose orthogonally: D-10 selects verification mode, E-20 selects compute location/runtime (E-20 L32, L245).
- **E004.INT.EX005 [EXPLICIT]** E-20 blocks secrets before/after cloud dispatch, whereas E-37 protects all untrusted ingest, tool arguments, output rendering, and egress; shared scanners/allowlists should be one primitive, but the gates occur at different lifecycle points (E-20 L59-L61; E-37 L54-L72).
- **E004.INT.CF001 [CONFLICT]** E-20 proposes `[cloud.privacy] allowlist_files` for output-scan overrides while E-37 proposes `[output.markdown_images]` and `[network.egress]`; overlapping hostname/file exceptions need one precedence/audit model (E-20 L237; E-37 L64, L269).
- **E004.INT.EX006 [EXPLICIT]** E-21 adds the SWE-bench-Verified compatibility runner even though E-9 stops headline reporting; the original does not formally resolve compatibility tooling versus reported benchmark policy (E-21 L54, L169).
- **E004.INT.EX007 [EXPLICIT]** E-21's repo state is inherited by E-25 session forking, while E-25's checkpoints add a distinct reversible file-state store (E-21 L50, L174; E-25 L64, L230).
- **E004.INT.EX008 [EXPLICIT]** E-22/C-35 own lazy MCP implementation; E-25 owns its CLI/config/dashboard surface; E-24 consumes it for MCP hardening. E-25 must block rather than reimplement if the mechanism is absent (E-22 L46, L177; E-25 L32, L56).
- **E004.INT.EX009 [EXPLICIT]** E-22 owns `Sandbox::MicroVm`; E-25 uses it for skill-template commands with warned Process fallback; E-37 extends Tier-C with default-deny network/DNS controls (E-22 L44; E-25 L58; E-37 L66).
- **E004.INT.EX010 [EXPLICIT]** E-25 owns generic `prompt_decision`; E-24 is its first MCP guard consumer, and E-37's list/ingest classifiers should reuse the same runtime-routing pattern rather than another hook protocol (E-25 L33, L60; E-37 L26, L68).
- **E004.INT.EX011 [EXPLICIT]** E-29 converts every SSB-backed LLM/tool/agent action—including events introduced by E-20/E-25/E-37/E-41/E-48—into an optional OTel view without replacing SSB (E-29 L10, L49-L51).
- **E004.INT.CF002 [CONFLICT]** E-5 locks the real schema at 103 while E-29 predicts 46+ from older B-plan versions; all E-29 migration/event work must rebase to 103+ and preserve E-5 consolidation (E-5 L16, L129-L141; E-29 L204).
- **E004.INT.EX012 [EXPLICIT]** E-37 layers after E-24: whole-list schema pin/sanitization/transport first, then per-description pin and full-list classifier (E-37 L25, L56, L68, L271).
- **E004.INT.EX013 [EXPLICIT]** E-37 relies on C-11 for process isolation and extends only its network namespace/DNS policy; E-22 may supply the concrete Linux MicroVm launcher used by that sandbox (E-22 L18-L20; E-37 L28, L66).
- **E004.INT.EX014 [EXPLICIT]** E-41 is a strict extension/superset of planned C-40's GitHub adapter and should consume it if already landed or absorb/deprecate it otherwise (E-41 L20, L42, L195, L200).
- **E004.INT.EX015 [EXPLICIT]** E-45's cross-language drift check becomes a required PR status before E-41 dispatches heterogeneous review; E-44 separately provides durability/compensation around irreversible remote actions (E-41 L5, L198-L200).
- **E004.INT.EX016 [EXPLICIT]** E-48 extends shipped B-7's node loop/source quality and B-40's FlashResearch scaffold rather than replacing their upper convergence thresholds (E-48 L14-L18, L53-L55, L185).
- **E004.INT.EX017 [EXPLICIT]** E-48 coordinates reranker choice with E-17, convergence with E-19, evidence delta with E-46, citation metrics with E-49, and verified trust with C-48 (E-48 L5, L261-L263).
- **E004.INT.IN001 [INFERRED]** E-29 transcript/replay/cost/query surfaces are the natural shared observability plane for E-8 optimizer, E-20 route, E-37 safety, E-41 PR, and E-48 research events, although those sources do not each name E-29 directly.

## Orientation-summary additions and corrections

Comparison source: `docs/thefinalglance/feature-discovery/05-E-SUMMARY_INDEX.discovery.md`. These atoms are deltas only; they do not promote summary-only claims.

### E-5 summary deltas (`05-E-SUMMARY_INDEX.discovery.md:L76-L97`)

- **E004.SD05.CF001 [CORRECTION]** “Foundational with no prerequisite” is wrong: the original names B-1/B-6/B-9/B-37 hard prerequisites and C-21 soft-planned (summary L78; source L20).
- **E004.SD05.CF002 [CORRECTION]** Summary events `memory_fact_added|updated|deleted|noop` are not the rebased wire contract; the original consolidates operations into `MemoryFactExtracted`, `MemoryConflictDetected`, and `MemoryEvolutionRun.phase`, with two events deferred (summary L90-L93; source L129-L141).
- **E004.SD05.CF003 [CORRECTION]** The original is not uniformly planned: it provides a task-by-task `SHIPPED-ON-V2|SHIPPED-PARTIAL|SCHEMA-CONSOLIDATED|DEFERRED` map and locks schema at 103 (source L3-L16), all absent from the summary.
- **E004.SD05.EX001 [ADDITION]** Add the exact 18-field/16-listed-column bi-temporal schema, two indexes, tier thresholds, path helpers, Mem0 update conditions, defense precedence, decay rates, contradiction logic, shipped/deferred CLI/UI split, and no-cloud/emotion constraints (source L73-L145).
- **E004.SD05.CF004 [CORRECTION]** “Approximately 600 LOC crate” understates the rebased shipped module set and no longer matches the original's current layout description (summary L89; source L98).

### E-8 summary deltas (`05-E-SUMMARY_INDEX.discovery.md:L131-L148`)

- **E004.SD08.CF001 [CORRECTION]** The implementation surface is `.elai_cc/python/elai_optimizer/`, not `workflow/optimizer/` (summary L142; source L18, L44, L169-L175).
- **E004.SD08.CF002 [CORRECTION]** Summary event names `prompt_optimizer_proposed|shadow|promoted` do not appear; the original defines seven PascalCase action types around run/progress/convergence/admission/promotion/edit spans (summary L143-L145; source L62, L76-L82).
- **E004.SD08.CF003 [CORRECTION]** A 24-hour run and live cloud `/task` are not original acceptance gates; the original uses a 50-task fixture, token budget, held-out/cross-domain thresholds, and five-iteration stub smoke (summary L146-L147; source L99-L101, L145-L167).
- **E004.SD08.EX001 [ADDITION]** Add frozen/edit block membership, feedback/OptimizedTemplate schemas, SHADOW/5%/2pp/cross-domain gates, top-three/30-day gotchas, Python filesystem isolation, 200k default cap, early-stop patience, entropy warning, and exact tests (source L46-L167, L202-L207).

### E-20 summary deltas (`05-E-SUMMARY_INDEX.discovery.md:L351-L367`)

- **E004.SD20.CF001 [CORRECTION]** The source does not implement xRouter or RouterArena policies: RouteLLM is the initial classifier, C3PO/AutoMix motivate the cascade, xRouter is deferred E-20.C, and RouterArena is only grounding (summary L355-L357; source L53-L71, L221-L239, L246-L247).
- **E004.SD20.CF002 [CORRECTION]** The primary surface is new `cost/cloud_route.rs`, not an extension of `routing.rs`; named events differ from generic `route_decision|data_leakage_blocked` (summary L362-L364; source L75-L96, L197-L204).
- **E004.SD20.EX001 [ADDITION]** Add entropy signal/0.55 calibration, pre-role difficulty adapter, input and output scans, 7/30-day/N=20 burn math, role/task defaults, N=3 vote, reasoning-mode toggle, 200-task Youden calibration, regex-only Presidio opt-in, and E-20.B/C deferrals (source L53-L71, L232-L250).
- **E004.SD20.CF003 [CORRECTION]** “Route prompts mentioning an API key locally” is only one fixture; operative behavior is a hard cloud veto on multiple secret/PII/path classes, with redacted reason and optional local execution/refusal (summary L365; source L59).

### E-21 summary deltas (`05-E-SUMMARY_INDEX.discovery.md:L368-L384`)

- **E004.SD21.CF001 [CORRECTION]** SBFL location is internally conditional/ambiguous (`auditor/src/sbfl.rs` versus `orchestrator/src/peer_systems/sbfl.rs`), not simply `orchestrator/src/sbfl.rs` (summary L379; source L44, L61, L139).
- **E004.SD21.CF002 [CORRECTION]** `ace_audit_run` and `sbfl_localized` are not named original events; the original's explicit rerank/PageRank events differ, and ACI is an audit artifact (summary L380-L381; source L46-L50, L83-L86).
- **E004.SD21.CF003 [CORRECTION]** “Agent chooses the top-ranked file” is not a required original behavior; the ranking is attached for fixer consumption and may inform targets (summary L383; source L44).
- **E004.SD21.EX001 [ADDITION]** Add PageRank constants/schema/artifact/CLI, source-stated Ochiai/coverage verification, rerank concurrency/fallback, four-file durable state, runtime-table adapters/no-LiteLLM, benchmark failure buckets/default, user confirmations, and exact audits/tests (source L42-L170).

### E-22 summary deltas (`05-E-SUMMARY_INDEX.discovery.md:L385-L403`)

- **E004.SD22.CF001 [CORRECTION]** Export is a Rust module selected by crate layout, not `scripts/export_skills.py`; sandbox is `core/src/sandbox.rs`, not `orchestrator/src/sandbox/` (summary L396-L397; source L42, L140-L150).
- **E004.SD22.CF002 [CORRECTION]** Original events are `skill_export_drift|skill_export_completed|mcp_lazy_schema_savings|mcp_transport_negotiated|sandbox_microvm_unavailable`, not `skill_exported|microsandbox_pool_size|twelve_factor_audit` (summary L398-L400; source L42-L48, L82-L85).
- **E004.SD22.CF003 [CORRECTION]** The CLI is `elai skills export/publish` (plural); no signed `claude.skills.io` payload is specified and non-Claude targets deliberately return unsupported pending verification (summary L401-L402; source L54, L73, L171).
- **E004.SD22.CF004 [CORRECTION]** Under-100ms pool boot is absent; actual gate is five Linux runs with p50<500ms/p95<1000ms and Windows Process fallback (summary L403; source L112-L120).
- **E004.SD22.EX001 [ADDITION]** Add normalized frontmatter/body identity/idempotence, blake3 drift, platform fallback, lazy list≤140 chars plus on-demand schema and ≥80% savings, transport's POST/SSE response semantics, two audit verdict schemas, dependency verification, and remote-microVM open choices (source L42-L173).

### E-25 summary deltas (`05-E-SUMMARY_INDEX.discovery.md:L441-L463`)

- **E004.SD25.CF001 [CORRECTION]** Implementation is `compaction_ladder.rs` with `CompactionLadderPanel`, not `compaction.rs`/`CompactionDrawer`; events are `compaction_tier_applied` and `runtime_substate_overlay_active`, not `compaction_tier_N|opusplan_substate|prompt_decision_evaluated` (summary L455-L459; source L54, L62, L184-L198).
- **E004.SD25.CF002 [CORRECTION]** No 85%-context trigger or guarantee to reconstruct the full session from any tier appears in the original (summary L460-L462; source L54, L106-L109).
- **E004.SD25.EX001 [ADDITION]** Add all six exact tier algorithms/order/fallback/cache boundary, lazy-surface BLOCKED dependency, two skill command syntaxes/sandbox/timeout/generation counter, strict JSON hook/error/default role, exact FSM substate mapping, fork/checkpoint lifecycle, all seven guard rules, thresholds, panels, and open storage/config choices (source L54-L226).

### E-29 summary deltas (`05-E-SUMMARY_INDEX.discovery.md:L517-L534`)

- **E004.SD29.CF001 [CORRECTION]** The crate path is hyphenated `crates/ssb-otel-export`, not `crates/ssb_otel_export` (summary L529; source L51, L162).
- **E004.SD29.CF002 [CORRECTION]** `otel_span_emitted` is not an original SSB action; OTel is populated columns/export, while new actions are replay/failure/signal/dataset (summary L531; source L49-L51, L80).
- **E004.SD29.CF003 [CORRECTION]** Replay is not deterministic: clock/RNG/KV state are absent, it is best-effort, and frozen predicates must fail closed (summary L527, L533; source L55, L200, L205).
- **E004.SD29.EX001 [ADDITION]** Add all 15 columns, operation mapping/hierarchy/content-off default, transcript filters, replay patch/session schema, 14 exact MAST modes, Signals, spend/cache attribution, eval datasets/0.05 gate, ten read-only SQL queries, batching/tail sampling, schema-baseline correction, and exact tests (source L49-L205).

### E-37 summary deltas (`05-E-SUMMARY_INDEX.discovery.md:L667-L684`)

- **E004.SD37.CF001 [CORRECTION]** New implementation is a six-module `crates/safety` plus `tool_description_pin.rs`, not `crates/plugins/src/prompt_injection.rs` (summary L679; source L214-L228).
- **E004.SD37.CF002 [CORRECTION]** Summary events `spotlighting_envelope|promptarmor_verdict` are not the action variants; the original defines nine precise variants (summary L680-L682; source L90-L98).
- **E004.SD37.CF003 [CORRECTION]** The smoke is hostile HTML with line-jumping/image exfil, not a hostile PDF page (summary L683; source L125-L127).
- **E004.SD37.EX001 [ADDITION]** Add random reversible datamarking, pin storage/diff/update, classifier schema/autonomous branching/batching, nine provenance variants, origin-based destructive gate, CSP/DNS controls, MCP full-list scan, AgentDojo `<5%`, output classifier, rollout/performance thresholds, and future CaMeL/StruQ/MELON scope (source L54-L277).

### E-41 summary deltas (`05-E-SUMMARY_INDEX.discovery.md:L733-L751`)

- **E004.SD41.CF001 [CORRECTION]** Original events are `PrExternalReviewStarted|Verdict` and five other named actions, not `pr_review_dispatched` (summary L748; source L67-L73).
- **E004.SD41.CF002 [CORRECTION]** No required SSRF finding/fix smoke exists; Semgrep gates a broader banned-token/secrets/safety set and runs soft for ten PRs (summary L750-L751; source L46, L138).
- **E004.SD41.EX001 [ADDITION]** Add adapter auth/APIs/feature gate, reviewer input/schema/N≥20 κ and `<0.5` fallback, CI order/100-commit calibration, import detection/registry rejection, commit labels/trailers, opt-in CODEOWNERS, auto-merge exception, two-sprint quarantine, server-side enforcement, timeout, and C-40 rollup (source L42-L200).
- **E004.SD41.CF003 [CORRECTION]** The summary's simple “commit every baby step” omits the source's reconciliation with one plan-close commit-equivalent: intra-step WIP squashed, baby-step commits retained for bisect, then one no-ff merge boundary (summary L744-L745; source L50).

### E-48 summary deltas (`05-E-SUMMARY_INDEX.discovery.md:L867-L888`)

- **E004.SD48.CF001 [CORRECTION]** An 80% citation pass-rate gate is absent; the original requires emitting the measured rate but gives no minimum (summary L887; source L59, L180).
- **E004.SD48.CF002 [CORRECTION]** Generic `citation_verified|contradiction_surfaced` are not named original events; the original names stage-specific failure/pass-rate and `contradiction_node_materialized` evidence (summary L885-L886; source L59, L107).
- **E004.SD48.EX001 [ADDITION]** Add ResearchPlan schema/heuristic/default, tier order/quotas/config/canonical URL hash/T2 event, extractor 10s→5s fallback, rerank chunking/rank histogram, exact pruning/budget formula, lower-stop conditions/warning, 50-row contradiction classifier/protected rendering, all four citation semantics, attribution tuple, recency regex/formula, four commands/panels, and calibration/cost/legacy risks (source L45-L263).
- **E004.SD48.CF003 [CORRECTION]** FlashResearch prunes speculative branches, not search “results” generally; contradiction nodes are explicitly prune-protected (summary L878; source L53, L57).

## Zero-omission and zero-duplicate audit

### Section coverage

Every original was swept across Objective/status, Context, Existing state/dependencies, Tasks, WIRING_CONTRACT, reachable/stub bans, observable effect, DETERMINISTIC_CHECK, ACCEPTANCE, Tests, Files touched, Open questions/risks, and Next. Bibliographic rows were not treated as features unless they imposed a behavior, numeric default, validation gate, or dependency-verification rule.

| Source | Status/deps | Capabilities/surfaces | Workflow/state | Schema/data/lifecycle | Flags/defaults/config | Integrations/events | Security/privacy | Failure/retry/recovery | Determinism/concurrency | Observability/evidence | Tests/done gates | Constraints/non-goals/deferred |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| E-5 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| E-8 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| E-20 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| E-21 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| E-22 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| E-25 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| E-29 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| E-37 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| E-41 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
| E-48 | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

Task-block accounting is exact: E-5 16/16, E-8 11/11, E-20 10/10, E-21 7/7, E-22 7/7, E-25 7/7, E-29 10/10, E-37 10/10, E-41 9/9, E-48 11/11. Every named test line is represented either individually or in a range atom that enumerates every test and its distinct assertion.

### Atomic counts

| Namespace | Atoms |
|---|---:|
| E-5 source ledger | 56 |
| E-8 source ledger | 41 |
| E-20 source ledger | 43 |
| E-21 source ledger | 40 |
| E-22 source ledger | 44 |
| E-25 source ledger | 59 |
| E-29 source ledger | 53 |
| E-37 source ledger | 56 |
| E-41 source ledger | 51 |
| E-48 source ledger | 67 |
| **Source-ledger subtotal** | **510** |
| Cross-file interactions | 20 |
| Summary additions/corrections | 41 |
| **Grand atomic total** | **571** |

Source-ledger classification: **431 explicit, 16 inferred, 49 ambiguous, 14 conflict = 510**. Across all namespaces, ID classes total **458 EX, 17 IN, 49 AM, 47 CF = 571**; summary `CF` IDs are labeled `[CORRECTION]`, whereas source-ledger `CF` IDs are actual conflicts.

### ID and duplicate audit

- Regex-audited identifier occurrences: **571**.
- Unique sorted identifiers: **571**.
- Duplicate identifiers: **0**.
- Every `(namespace,class)` sequence starts at `001`, is contiguous, and has no numbering gap.
- Semantic duplicates repeated by objective/task/wiring/acceptance were collapsed into one source atom with multiple anchors; distinct tests/events/failures/defaults remain separate.
- Original-source boundary audit: **10/10 manifest originals**, no E-plan original outside DEEP-E-004 read or mined.
- Manifest integrity audit: **10/10 hashes, line counts, and byte counts exact; aggregate 2,302 lines / 274,448 bytes exact**.

## Output integrity

- Final document accounting: `files=10`, `source_lines=2302`, `source_bytes=274448`, `source_atoms=510`, `interaction_atoms=20`, `summary_delta_atoms=41`, `atomic_total=571`, `output_lines=000856`, `output_bytes=128250`.
- `canonical_output_sha256=088da3aa56209ce0314ffc48dcb670cd9cb7493ce93dbe5b61d493503dee72ff`.
- SHA scope: exact final UTF-8 bytes of this file after replacing the 64 hexadecimal characters following `canonical_output_sha256=` with 64 ASCII zeroes. This avoids an impossible self-referential raw-file hash while covering every byte, including this integrity record.

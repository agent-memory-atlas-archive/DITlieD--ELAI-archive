# F-plan exhaustive feature discovery

## Scope, method, and coverage contract

- **Primary source fully read:** `docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md`, 286/286 lines, including its explicit end marker. Source anchors below are line ranges in that file.
- **Orientation only:** `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md`. The overview is a design/topology map and explicitly says it is not a wiring or shipped-state ledger; it is not used here to upgrade any F item’s implementation status.
- **Original-plan isolation:** no F original plan, F research rationale, or referenced implementation file was read for this pass. They are inventoried in the deeper-source manifest for later agents.
- **Evidence labels:** `E` means the behavior or constraint is explicit in the F summary. `I` means a composition inference made only in the workflow section; it is never represented as a sourced implementation fact. Statements labeled as existing ground truth are still only *summary assertions* in this pass and require original/code verification.
- **Index coverage:** all 14 entry headings are covered: the original F-1 through F-10 set (10), plus the four post-F-10 entries F-2c, F-11, F-12, and F-13. F-2a and F-2b are preserved as named subscopes of F-2, while C6a/C6b/C6c and all numbered phases/tiers are preserved.
- **Header caveat:** the title says “All 10 Sessions,” while lines 193-276 append four explicitly `PLANNED, implementation pending` entries. This discovery treats that as 10 original sessions plus 4 extensions, not as a contradiction to resolve locally.
- **Atomicity rule:** each atomic record captures one capability, behavior, gate condition, artifact, surface, metric, constraint, or explicit negative/deferred statement. Exact commands, event names, field names, enum names, paths, thresholds, and equality predicates are retained.

## Global F-plan contract

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| FG-01 | E | Each F-N is intended to be one implementation session that turns one VibeSec/SHARD proposal into a wired result. | L8-L10 |
| FG-02 | E | Each session’s result is intended to be dashboard-visible. | L9-L10 |
| FG-03 | E | A session is not `DONE` until `wiring_contract_check.py --run-trigger` passes. | L10-L12 |
| FG-04 | E | The wiring done-gate requires the producer to exist. | L11-L12 |
| FG-05 | E | The wiring done-gate requires a real consumer, not a nominal/dead consumer. | L11-L12 |
| FG-06 | E | The path must be reachable from a production entrypoint. | L11-L12 |
| FG-07 | E | The `stub_ban` check must be clean. | L11-L12 |
| FG-08 | E | The declared `observable_effect` must actually fire. | L11-L12 |
| FG-09 | E | A cargo-green result by itself is rejected as completion evidence. | L12-L13 |
| FG-10 | E | Default-off environment gates are banned as a terminal implementation state under W-2. | L12-L13, L278-L280 |
| FG-11 | E | Every dashboard-surfaceable signal requires a `frontend_pair`. | L281-L281 |
| FG-12 | E | Every dashboard-surfaceable signal requires `cross_lang_diff` to pass. | L281-L281 |
| FG-13 | E | Every `observable_effect` must be a live end-to-end trigger under W-5. | L282-L283 |
| FG-14 | E | A RunPod-gated item must still ship a deterministic fixture harness that exercises the real path. | L282-L283 |
| FG-15 | E | The RunPod live run is tracked as a separate gated checkbox from the deterministic fixture. | L282-L283 |
| FG-16 | E | W-10 tiering keeps network, subprocess, and RunPod work out of the fail-closed hot path. | L284-L284 |
| FG-17 | E | The canonical rationale and per-item detail are delegated to `workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md`. | L2-L6 |
| FG-18 | E | W-1 through W-10 enforcement doctrine is delegated to `F-0_INDEX.md`. | L5-L6 |

## Atomic feature inventory

### F-1 — VibeGate: unified fail-closed multi-axis in-process write-gate

**Classification/status as stated:** Cluster C1; `HYBRID`; novelty `GENUINELY_NEW`; feasibility `HARD`; effort `L`. The revised scope says Tier-1 “ships now”; Tier-2 is deferred behind a design gate. The summary also explicitly says the four axes are not currently wired into one compound check. (L17-L19, L24-L32)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F1-01 | E | One `PreToolUse` handler evaluates a file write through a compound VibeGate. | L20-L21 |
| F1-02 | E | One compound axis detects code-vulnerability patterns. | L20-L21 |
| F1-03 | E | One compound axis detects hallucinated dependencies. | L20-L21 |
| F1-04 | E | One compound axis detects secrets. | L20-L21 |
| F1-05 | E | One compound axis detects malicious skills. | L20-L21 |
| F1-06 | E | The write is blocked fail-closed if any axis fires. | L20-L22 |
| F1-07 | E | Security rules are backed by a `vibesec` rule domain. | L21-L22 |
| F1-08 | E | VibeSec seed rows use `mutable=0`. | L21-L22 |
| F1-09 | E | Per-project edits cannot disable the immutable VibeSec security rows. | L21-L22 |
| F1-10 | E | The existing rules engine’s `ALWAYS_INCLUDED_DOMAINS` already includes `security`. | L24-L25 |
| F1-11 | E | The existing schema’s `mutable` column defaults to `1`. | L25-L26 |
| F1-12 | E | A tested `mutable=0` lifecycle path already exists. | L25-L26 |
| F1-13 | E | The existing PreToolUse runner is `run_pre_tool_use`. | L26-L27 |
| F1-14 | E | The four axes are explicitly not yet wired into one compound check in the cited ground truth. | L26-L27 |
| F1-15 | E | Tier-1 implements code-vulnerability checks with AST/GREP mechanisms. | L28-L29 |
| F1-16 | E | Tier-1 implements secret checks by GREP over the diff. | L28-L29 |
| F1-17 | E | Tier-1 is pure in-process. | L28-L30 |
| F1-18 | E | Tier-1 is required to be genuinely fail-closed. | L28-L30 |
| F1-19 | E | Tier-1 has a per-tool contract test for `Edit`. | L29-L30 |
| F1-20 | E | Tier-1 has a per-tool contract test for `Write`. | L29-L30 |
| F1-21 | E | Tier-1 has a per-tool contract test for `NotebookEdit`. | L29-L30 |
| F1-22 | E | Tier-1 has a per-tool contract test for `Bash`. | L29-L30 |
| F1-23 | E | Startup performs a BLAKE3 integrity check of the `mutable=0` seed rows. | L30-L30 |
| F1-24 | E | Tier-2 adds dependency-existence checking. | L30-L31 |
| F1-25 | E | Tier-2 adds MCP scanning. | L30-L32 |
| F1-26 | E | Tier-2 network/subprocess work is deferred behind a design gate. | L30-L32 |
| F1-27 | E | Tier-2 data is to be cached at plan start. | L31-L32 |
| F1-28 | E | Tier-2 cache/evaluation cadence is per session, not per write. | L31-L32 |
| F1-29 | E | The production chain is `hooks.rs run_pre_tool_use -> compound VibeGate evaluator -> RuleEngine`. | L33-L34 |
| F1-30 | E | Immutable security rows are seeded in `schema.rs`. | L33-L34 |
| F1-31 | E | A VibeGate panel appears in a dashboard drawer as the frontend pair. | L33-L34 |
| F1-32 | E | An `Edit`/`Write` containing a known SQL-injection pattern is refused before disk. | L35-L37 |
| F1-33 | E | An `Edit`/`Write` containing a hardcoded secret is refused before disk. | L35-L37 |
| F1-34 | E | A code-vulnerability refusal journals `vibesec_code_vuln_blocked` in the SSB. | L35-L37 |
| F1-35 | E | A secret refusal journals `vibesec_secret_blocked` in the SSB. | L35-L37 |
| F1-36 | E | A blocked write must not land on disk. | L35-L37 |
| F1-37 | E | The VibeGate panel displays the block. | L35-L37 |
| F1-38 | E | On a defined synthetic corpus, zero secrets are allowed to reach disk. | L37-L38 |
| F1-39 | E | On that corpus, zero known vulnerability-class AST matches are allowed to reach disk. | L37-L38 |
| F1-40 | E | The false-positive rate must be measured and bounded before release. | L38-L39 |
| F1-41 | E | False-positive bounding uses at least three real orchestrate runs. | L38-L39 |

**Deep-inspection ambiguities:** whether `vibesec` and `security` are the same domain or nested domains; exact AST/GREP predicates and secret formats; how fail-closed handles evaluator/internal errors; what the BLAKE3 startup check compares against and how mismatch is surfaced; how `Bash` writes are identified; the numerical false-positive bound; the Tier-2 cache fill/failure/expiry contract; what “malicious-skill” means operationally; and whether dependency/MCP axes remain part of the advertised four-axis gate while deferred.

### F-2 — Code-vuln-class ruleset + the FIRST measured pre-write insecure-rate number (the keystone)

**Classification/status as stated:** Cluster C2; `HYBRID`; novelty `GENUINELY_NEW`; feasibility `HARD`; grounding `ACCURATE`. Revised scope splits F-2a (rules/write gate) from F-2b (RunPod/benchmark gated). No summary-wide current implementation status is assigned here; F-2c later supplies more specific ground-truth assertions. (L41-L57)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F2-01 | E | The always-on security domain gains SQL-injection vulnerability-class patterns. | L44-L45 |
| F2-02 | E | The always-on security domain gains XSS vulnerability-class patterns. | L44-L45 |
| F2-03 | E | The always-on security domain gains weak-cryptography patterns. | L44-L45 |
| F2-04 | E | The always-on security domain gains missing-authorization patterns. | L44-L45 |
| F2-05 | E | Matched vulnerability-class patterns block at write time. | L44-L45 |
| F2-06 | E | The deliverable publishes a measured before/after insecure rate. | L44-L47 |
| F2-07 | E | SVEN’s `59.1 -> 92.3%` result is treated as verified evidence for CodeGen-2.7B, not as an ELAI result. | L46-L47 |
| F2-08 | E | CodeGuard+’s `43.2 -> 76.0%` result is treated as verified evidence for CodeGen-2.7B, not as an ELAI result. | L46-L47 |
| F2-09 | E | Those transferred CodeGen numbers must not be claimed for `Qwen2.5-Coder-7B-abliterated`. | L46-L47 |
| F2-10 | E | F-2a ships the ruleset and write-time block. | L48-L49 |
| F2-11 | E | F-2a initially scopes rule coverage to Python. | L48-L49 |
| F2-12 | E | F-2a initially scopes rule coverage to TypeScript. | L48-L49 |
| F2-13 | E | The Python/TypeScript restriction is used to bound false positives. | L48-L49 |
| F2-14 | E | F-2a makes no performance claim. | L48-L49 |
| F2-15 | E | F-2b is gated on C-26/E-35. | L49-L50 |
| F2-16 | E | F-2b is gated on RunPod availability/execution. | L49-L50 |
| F2-17 | E | F-2b runs SeCodePLT/CodeGuard+ on Qwen. | L49-L51 |
| F2-18 | E | F-2b uses the E-9 harness at `crates/cli/src/benchmark.rs --suite seccodeplt`. | L50-L51 |
| F2-19 | E | Benchmark reporting includes insecure rate. | L50-L51 |
| F2-20 | E | Benchmark reporting includes functional `pass@1` alongside insecure rate. | L50-L51 |
| F2-21 | E | F-2 adds security rule TOMLs under `crates/rules/rules/security/*.toml`. | L52-L52 |
| F2-22 | E | F-2 integrates its block through `PreToolUse`. | L52-L52 |
| F2-23 | E | F-2 extends `benchmark.rs`. | L52-L52 |
| F2-24 | E | F-2a’s live trigger uses a generated SQL-injection snippet. | L53-L54 |
| F2-25 | E | The SQL-injection snippet is blocked before write. | L53-L54 |
| F2-26 | E | The F-2a trigger journals `security_rule_blocked`. | L53-L54 |
| F2-27 | E | `elai benchmark --suite seccodeplt` writes `insecure_rate_before/after` to a phase TOML. | L54-L55 |
| F2-28 | E | The F-2b benchmark emits `insecure_rate_measured`. | L54-L55 |
| F2-29 | E | R1 is defined as a real security number measured on ELAI’s engine. | L55-L56 |
| F2-30 | E | R2 is defined as the anti-Replit demo. | L55-L56 |
| F2-31 | E | The headline/first-in-category claim is gated on the measured Qwen result. | L55-L57 |

**Deep-inspection ambiguities:** exact rule inventory and severities beyond the four classes; whether `crates/rules/rules/security/` is superseded by F-2c’s `crates/rules/seed/security/`; benchmark command drift (`elai benchmark --suite seccodeplt` here versus `elai bench seccodeplt` in F-2c); definitions and denominators for “before,” “after,” insecure rate, and pass@1; corpus/version/model pinning; RunPod reproducibility; false-positive acceptance criteria; and what artifact makes the number “published.”

### F-3 — CaMeL / Tool-Filter data-plane defense for untrusted-content agents

**Classification/status as stated:** Cluster C3; `NEW`; novelty `GENUINELY_NEW`; feasibility `HARD`; grounding `OVERSTATED` specifically for token overhead. Phase 1 is medium and “ship first”; Phase 2 is XL and conditional. (L59-L73)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F3-01 | E | The feature addresses the data plane left exposed by Tier-C control-plane isolation. | L62-L63 |
| F3-02 | E | Repository content is treated as potentially instruction-injecting untrusted content. | L62-L63 |
| F3-03 | E | Web content is treated as potentially instruction-injecting untrusted content. | L62-L63 |
| F3-04 | E | Skill content is treated as potentially instruction-injecting untrusted content. | L62-L63 |
| F3-05 | E | MCP content is treated as potentially instruction-injecting untrusted content. | L62-L63 |
| F3-06 | E | The design adopts a capability/Dual-LLM pattern. | L63-L65 |
| F3-07 | E | Phase 1 is a Tool Filter implemented through the existing FSM rather than an IPC schema change. | L66-L68 |
| F3-08 | E | The researcher’s available tool set is reduced to a per-task minimal subset. | L66-L68 |
| F3-09 | E | The tool set is locked before any untrusted byte is observed. | L66-L68 |
| F3-10 | E | Phase 1 measures ELAI’s injection attack-success rate before the filter. | L68-L68 |
| F3-11 | E | Phase 1 measures ELAI’s injection attack-success rate after the filter. | L68-L68 |
| F3-12 | E | The measurement uses an AgentDojo-style fixture. | L68-L68 |
| F3-13 | E | Phase 2 adds full CaMeL `SymbolicVar`. | L68-L70 |
| F3-14 | E | Phase 2 requires primary-source token-overhead evidence from the CaMeL PDF. | L69-L70 |
| F3-15 | E | Phase 2 requires overhead measured on ELAI’s real IPC. | L69-L70 |
| F3-16 | E | The IPC measurement must demonstrate that the 32K ceiling survives. | L69-L70 |
| F3-17 | E | Phase 2 also requires explicit acceptance of an IPC breaking change. | L69-L70 |
| F3-18 | E | Phase 1 integrates with the per-phase FSM tool gate through the `PreToolUse` path. | L71-L72 |
| F3-19 | E | Phase 1 integrates with researcher dispatch. | L71-L72 |
| F3-20 | E | A fixture run through the researcher pipeline emits `tool_filter_committed`. | L72-L73 |
| F3-21 | E | The observable benchmark records a measured ASR drop. | L72-L73 |
| F3-22 | E | An injected instruction cannot fire a tool outside the locked set. | L72-L73 |
| F3-23 | E | CaMeL’s `77% provable-secure` figure is recorded as verified external evidence, not an ELAI result. | L63-L65 |
| F3-24 | E | AgentDojo’s `57.7 -> 6.84% ASR at 73% utility` is recorded as verified external evidence, not an ELAI result. | L64-L65 |

**Deep-inspection ambiguities:** how the minimal tool set is derived and authorized; whether the set can expand after untrusted input; exact behavior/event on an out-of-set call; which bytes/sources mark the trust boundary; ASR fixture composition and utility metric; the capability/Dual-LLM split; Phase-2 `SymbolicVar` representation; the precise 32K acceptance criterion; IPC migration/versioning; and failure/degradation behavior when filter state is missing.

### F-4 — Test-integrity + output-grounding gate (the literal “cant-lie” half of the north star)

**Classification/status as stated:** Cluster C4; `HYBRID`; novelty `PARTIALLY_COVERED`; feasibility `HARD`. It has prerequisites C-27 and C-28 and an explicitly blocking design question. Semantic output grounding is deferred to a later F-N. (L75-L91)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F4-01 | E | Every claimed test outcome is bound to an orchestrator-captured artifact. | L78-L79 |
| F4-02 | E | Binding the artifact makes a fabricated `PASS` detectable. | L78-L82 |
| F4-03 | E | The existing `TestOutcomePayload` journals `test_name`. | L79-L80 |
| F4-04 | E | The existing `TestOutcomePayload` journals `passed`. | L79-L80 |
| F4-05 | E | The existing `TestOutcomePayload` journals `duration_ms`. | L79-L80 |
| F4-06 | E | The existing `TestOutcomePayload` has no `stdout_hash`. | L79-L82 |
| F4-07 | E | The current auditor’s tests run through its own LLM Bash tool, leaving its PASS narrative unbound. | L80-L82 |
| F4-08 | E | C-22 already provides the `stdout_hash` pattern on the feasibility path. | L81-L82 |
| F4-09 | E | `TestOutcomePayload` gains `stdout_hash: Option<String>`. | L83-L84 |
| F4-10 | E | The output hash uses BLAKE3 following the `feasibility_probe.rs:168` pattern. | L83-L84 |
| F4-11 | E | The auditor cannot emit `passed=true` without a matching orchestrator-captured hash row. | L83-L85 |
| F4-12 | E | The matching hash row must be in the same superstep. | L84-L85 |
| F4-13 | E | Tests must run through an orchestrator-managed process slot. | L85-L87 |
| F4-14 | E | The managed process slot must give the orchestrator a stdout handle to hash. | L85-L87 |
| F4-15 | E | Resolving the managed-process design is a precondition; otherwise the field is considered dead. | L85-L87 |
| F4-16 | E | Semantic output-grounding is split into a later F-N. | L87-L88 |
| F4-17 | E | Semantic output-grounding requires spec-pinned expected outputs. | L87-L88 |
| F4-18 | E | F-4 depends on C-27. | L88-L88 |
| F4-19 | E | F-4 depends on C-28. | L88-L88 |
| F4-20 | E | The adversarial fixture runs a baby-step test process that exits `1`. | L89-L90 |
| F4-21 | E | The fixture makes the auditor claim `PASS` despite the failing process. | L89-L90 |
| F4-22 | E | The `AUDITING -> MEMORIZING` transition gate detects the mismatch. | L89-L91 |
| F4-23 | E | The gate emits `anti_fabrication_block`. | L89-L91 |
| F4-24 | E | The stated fabrication signature is `passed=true` with `stdout_hash=None`. | L90-L91 |
| F4-25 | E | The dashboard audit timeline displays the block. | L91-L91 |

**Deep-inspection ambiguities:** exact process-slot API and ownership; whether stderr, exit code, command, environment, and output truncation join the hash; what row type carries the captured hash; correlation keys for “matching” within a superstep; whether `None` is valid for failed/skipped tests; how hash mismatches differ from missing hashes; schema-migration compatibility; and the identity/scope of the deferred semantic-output F-N.

### F-5 — Cross-subsystem staleness-quarantine + bi-temporal contradiction cascade

**Classification/status as stated:** Cluster C5; `HYBRID`; novelty `GENUINELY_NEW`; feasibility `HARD`; grounding `ACCURATE`. Phase 1 is sequenced after B-30; Phase 2 follows its consumer audit and retrieval contract test. (L93-L109)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F5-01 | E | Stale facts enter a SHARD-style time-decay quarantine. | L96-L98 |
| F5-02 | E | The self-healing cascade invalidates a stale fact. | L96-L98 |
| F5-03 | E | The cascade demotes dependent data. | L96-L98 |
| F5-04 | E | The cascade broadcasts the demotion across subsystems. | L96-L100 |
| F5-05 | E | Invalidation is bi-temporal and non-destructive in the Graphiti style. | L96-L100 |
| F5-06 | E | Mem0 A.U.D.N. is an input pattern for the design. | L96-L98 |
| F5-07 | E | E-5 already covers bi-temporal supersession only for `memory_facts`; F-5 expands the scope. | L98-L100 |
| F5-08 | E | A quarantined fact’s demotion propagates to dependent DKG concept annotations. | L99-L100 |
| F5-09 | E | A quarantined fact’s demotion propagates to `grounding_cache`. | L99-L100 |
| F5-10 | E | A quarantined fact’s demotion propagates to Cosmos artifacts. | L99-L100 |
| F5-11 | E | One broadcast is the cross-subsystem propagation mechanism. | L99-L100 |
| F5-12 | E | Existing `concept_stale` is asserted to be always `1` and never populated. | L99-L101 |
| F5-13 | E | Before population, Phase 1 audits every `concept_stale` consumer. | L101-L104 |
| F5-14 | E | Before population, Phase 1 adds a retrieval contract test with `concept_stale=0`. | L101-L104 |
| F5-15 | E | The consumer audit and contract test occur after B-30. | L102-L104 |
| F5-16 | E | Phase 2 implements the cascade through a bounded fan-out queue. | L104-L105 |
| F5-17 | E | The fan-out channel has an explicit capacity. | L104-L105 |
| F5-18 | E | Bounded fan-out prevents scheduler-tick pile-up. | L104-L105 |
| F5-19 | E | `crates/memory/src/decay.rs` emits `StalenessQuarantineBroadcast`. | L106-L107 |
| F5-20 | E | DKG consumers live under `crates/knowledge/src/dkg`. | L106-L107 |
| F5-21 | E | A Cosmos consumer integrates through `cosmos_worker.rs`. | L106-L107 |
| F5-22 | E | Quarantining a fact marks dependent DKG concepts `concept_stale=1` within one tick. | L107-L109 |
| F5-23 | E | Re-query after quarantine produces a `grounding_cache` miss within one tick. | L107-L109 |
| F5-24 | E | The cascade contract test uses a mock clock. | L108-L109 |
| F5-25 | E | A dashboard staleness panel surfaces the cascade. | L108-L109 |

**Deep-inspection ambiguities:** the apparent `concept_stale` polarity/default inconsistency (“always 1,” first real `0`, yet quarantine sets `1`); dependent-concept and Cosmos-artifact relation traversal; the Cosmos observable effect; grounding-cache invalidation keys; broadcast payload, durability, ordering, idempotency, retry, and backpressure behavior; explicit queue capacity; tick definition/deadline; partial-consumer failure behavior; and what B-30 provides.

### F-6 — Learned-skill library: trust ladder + critic-gated acquisition + staleness demotion

**Classification/status as stated:** Cluster C6; `HYBRID`; novelty `PARTIALLY_COVERED`; feasibility `HARD`. Revised scope decomposes the work into C6a, C6b, and C6c; the 70%-reuse claim is deferred. (L111-L125)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F6-01 | E | Learned-skill acquisition follows a curriculum-to-generation step. | L114-L115 |
| F6-02 | E | A generated skill is executed before acquisition. | L114-L115 |
| F6-03 | E | A critic gates acquisition after execution. | L114-L115 |
| F6-04 | E | A rejected acquisition can retry. | L114-L115 |
| F6-05 | E | A critic-approved acquisition is committed. | L114-L115 |
| F6-06 | E | Learned skills support embedding retrieval. | L114-L115 |
| F6-07 | E | The source SHARD trust ladder has four conceptual tiers (T1-T4). | L115-L116 |
| F6-08 | E | Trust promotion considers invocation count. | L115-L116 |
| F6-09 | E | Trust promotion considers failure rate. | L115-L116 |
| F6-10 | E | Trust promotion considers minimum elapsed days. | L115-L116 |
| F6-11 | E | Promotion to the top tier is human-gated. | L115-L117 |
| F6-12 | E | Stale skills are demoted. | L115-L117 |
| F6-13 | E | A sub-top skill cannot execute autonomously without a human gate. | L116-L117 |
| F6-14 | E | Voyager’s `3.3x` result is explicitly treated as Minecraft-only and non-transferable. | L117-L117 |
| F6-15 | E | C6a defines a distinct Rust enum named `SkillTrustTier`. | L118-L119 |
| F6-16 | E | `SkillTrustTier` values are S1 through S4. | L118-L119 |
| F6-17 | E | Skill trust uses distinct SSB tags from memory trust. | L118-L120 |
| F6-18 | E | Memory `TrustTier` remains T0-T3 provenance and must not collide with skill trust. | L112-L113, L118-L120 |
| F6-19 | E | The auditor enforces a type barrier between skill trust and memory trust. | L118-L120 |
| F6-20 | E | C6b extends `TemplateCrystallizer`’s critic. | L119-L120 |
| F6-21 | E | The acquisition critic reuses the existing BestOfN `MiniCritic` verdict. | L119-L120 |
| F6-22 | E | The reused verdict comes from the triggering SSB row. | L119-L120 |
| F6-23 | E | Reuse avoids triple-critic redundancy. | L119-L120 |
| F6-24 | E | C6c implements the staleness-demotion cascade. | L121-L121 |
| F6-25 | E | The 70%-reuse claim requires `concept_class` SSB tagging. | L121-L122 |
| F6-26 | E | The 70%-reuse claim also requires at least 20 logged acquisitions. | L121-L122 |
| F6-27 | E | `compiled_pipeline.rs` is the `TemplateCrystallizer` integration point. | L123-L123 |
| F6-28 | E | A new `skill_trust.rs` is an integration point. | L123-L123 |
| F6-29 | E | The autonomous-mode gate consumes skill trust. | L123-L125 |
| F6-30 | E | The done-trigger attempts autonomous dispatch with an S2 skill and no human gate. | L124-L125 |
| F6-31 | E | That S2 autonomous dispatch is blocked. | L124-L125 |
| F6-32 | E | Trust promotion journals `skill_trust_promoted`. | L124-L125 |
| F6-33 | E | Staleness demotion journals `skill_demoted_stale`. | L124-L125 |
| F6-34 | E | A dashboard skill-library panel shows skill tiers. | L124-L125 |

**Deep-inspection ambiguities:** whether curriculum/generation/execution/retry/embedding retrieval remain in revised implementation scope or are source-pattern context; exact S1-S4 semantics and initial tier; promotion/demotion thresholds and clock; whether “top tier human-gated” gates promotion while “sub-top human-gated” gates execution; what autonomous privileges S4 gains; critic verdict correlation and replay handling; stale definition; demotion fan-out; human-gate UX; and the schema of `concept_class` and acquisition counts.

### F-7 — Safety-lattice completeness proof + outcome-utility CBR template ranking

**Classification/status as stated:** Cluster C7; `HYBRID`; novelty `GENUINELY_NEW`; feasibility `HARD`; grounding `OVERSTATED` because Springdrift numbers are synthetic/unreproduced and are not targets. The two deliverables are explicitly split and must not be bundled. (L127-L141)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F7-01 | E | The safety-lattice work covers the immutable (`mutable=0`) security-rule subset. | L131-L132 |
| F7-02 | E | The safety-lattice work covers the immutable (`mutable=0`) finance-rule subset. | L131-L132 |
| F7-03 | E | It proves completeness of the immutable rule subset as a set. | L131-L133 |
| F7-04 | E | It proves monotonicity of the immutable rule subset as a set. | L131-L133 |
| F7-05 | E | It proves conflict freedom of the immutable rule subset as a set. | L131-L133 |
| F7-06 | E | Set-level proof is required rather than unit tests alone. | L131-L133 |
| F7-07 | E | The proof is an amendment to E-53’s integration-point list. | L134-L136 |
| F7-08 | E | The proof exhaustively enumerates pairs of `mutable=0` rules. | L134-L136 |
| F7-09 | E | The proof uses Z3 monotonicity checks. | L134-L136 |
| F7-10 | E | Safety-lattice work is sequenced strictly after E-53’s Z3/Kani/Verus stubs resolve. | L134-L136 |
| F7-11 | E | That prerequisite is marked `RUNPOD-PENDING`. | L135-L136 |
| F7-12 | E | `elai verify safety-lattice` is the verification command. | L139-L140 |
| F7-13 | E | The CLI reports zero conflict pairs over the immutable enumeration on success. | L139-L140 |
| F7-14 | E | Successful proof emits `safety_lattice_verified`. | L139-L140 |
| F7-15 | E | The independent CBR deliverable ranks pipeline templates by learned outcome utility. | L131-L133 |
| F7-16 | E | CBR ranking uses Laplace weighting. | L132-L133, L136-L138 |
| F7-17 | E | CBR consumes existing SSB outcome events. | L136-L138 |
| F7-18 | E | Outcome-utility ranking replaces first-try-ratio ranking. | L132-L133 |
| F7-19 | E | CBR ranking is a standalone plan over the pipeline-template store. | L136-L138 |
| F7-20 | E | The synthetic/unreproduced `+10.7%` target is dropped. | L129-L130, L137-L138 |
| F7-21 | E | Evaluation waits for at least 100 labelled sessions. | L137-L138 |
| F7-22 | E | Evaluation measures a delta against the first-try baseline. | L137-L138 |
| F7-23 | E | A ranking operation emits `template_utility_ranked`. | L140-L141 |
| F7-24 | E | Evaluation reports measured `P@k` delta after the labelled corpus accrues. | L140-L141 |

**Deep-inspection ambiguities:** formal definitions of completeness, monotonicity, and conflict; rule composition/order and parameter domains; whether exhaustive pairs suffice for higher-order conflicts; proof artifact/checker freezing; exact E-53 amendment; Z3/Kani/Verus/RunPod division; Laplace prior/formula; outcome labels, success horizon, and template identity; P@k’s relevance definition and `k`; treatment of censored/failed sessions; and the standalone plan/session ID for CBR.

### F-8 — SCF symbolic semantic conflict detector (self-mod pre-execution first)

**Classification/status as stated:** Cluster C8; `HYBRID`; novelty `GENUINELY_NEW`; feasibility `HARD`; grounding `OVERSTATED`. The revised scope is self-modification only. The inter-agent BSP half is deferred and `RUNPOD-PENDING`. The source notes that the desired DGM-plus-semantic-review synthesis does not ship at the time described. (L143-L158)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F8-01 | E | The detector represents intent through SCF `IntentNode` preconditions and postconditions. | L148-L150 |
| F8-02 | E | It detects three types of pre/postcondition conflict. | L148-L150 |
| F8-03 | E | The symbolic detector is LLM-free. | L148-L150 |
| F8-04 | E | Revised scope applies the detector only to self-modification. | L151-L152 |
| F8-05 | E | The scoped lane handles one configuration promotion per session. | L151-L152 |
| F8-06 | E | A detected self-mod conflict first warns, then escalates. | L151-L152 |
| F8-07 | E | The `27.9%` precision figure is treated as a simulation limitation, not a production target. | L146-L147, L151-L153 |
| F8-08 | E | The inter-agent BSP conflict-detector half is deferred. | L153-L155 |
| F8-09 | E | The BSP deferral is attributed to false-positive noise in a cap-3 parallel loop. | L153-L154 |
| F8-10 | E | The BSP half also has a C-50 namespace collision. | L153-L154 |
| F8-11 | E | The BSP half is flagged `RUNPOD-PENDING`. | L153-L155 |
| F8-12 | E | Graphiti’s shipped LLM-judged contradiction detection is to be evaluated as a higher-maturity cloud-lane alternative. | L154-L155 |
| F8-13 | E | Self-mod integration targets `self_mod_safety.rs`. | L151-L152, L156-L156 |
| F8-14 | E | Self-mod integration targets `promotion_gate.rs`. | L151-L152, L156-L156 |
| F8-15 | E | Conflict checking runs before archive admission. | L156-L158 |
| F8-16 | E | The adversarial fixture contains a self-mod config variant that contradicts an active locked-key invariant. | L156-L158 |
| F8-17 | E | The fixture is DGM-class reward-hack shaped. | L157-L158 |
| F8-18 | E | The detector flags the variant before archive admission. | L156-L158 |
| F8-19 | E | Detection emits `symbolic_conflict_detected`. | L157-L158 |

**Deep-inspection ambiguities:** names/semantics of the three conflict types; how free-form configuration intent becomes `IntentNode`; locked-key invariant source and precedence; exact WARN-to-escalate policy and whether admission is blocked; fixture construction; parser/completeness limits; event payload; archive-gate ordering; SCF precision measurement; Graphiti evaluation criteria; and how a future BSP version avoids C-50 collision.

### F-9 — Hallucinated-dependency existence+age+provenance gate at WRITE-time

**Classification/status as stated:** Cluster C9; `HYBRID`; novelty `PARTIALLY_COVERED`; feasibility `HARD`; grounding `ACCURATE`. It extends D-11/E-41 and explicitly depends on D-11 shipping. (L160-L174)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F9-01 | E | D-11’s real HTTP dependency checker is moved/extended from the PR boundary to write time. | L161-L164 |
| F9-02 | E | The write-time check validates package existence. | L163-L164 |
| F9-03 | E | The write-time check validates package publish date/age. | L163-L164 |
| F9-04 | E | The check fires when an import line is being written, before it reaches disk. | L163-L164 |
| F9-05 | E | Cargo/crates.io is added as a supported registry ecosystem. | L163-L165 |
| F9-06 | E | Socket Firewall wrapping is assigned to the install boundary. | L164-L165 |
| F9-07 | E | The Socket Firewall integration is moved to an operations runbook. | L170-L171 |
| F9-08 | E | The summary records that Socket cannot detect the fresh-slop/no-CVE gap by itself. | L165-L166 |
| F9-09 | E | F-9 extends D-11. | L167-L167 |
| F9-10 | E | F-9 extends E-41. | L167-L167 |
| F9-11 | E | F-9 cannot proceed until D-11 ships. | L167-L167 |
| F9-12 | E | `_check_cargo()` is added to `registry_check.py`. | L167-L168 |
| F9-13 | E | A `PreToolUse` hook is the write-time enforcement point. | L168-L170 |
| F9-14 | E | The hook is selective for import patterns. | L168-L169 |
| F9-15 | E | The hook is selective for Rust `use` patterns. | L168-L169 |
| F9-16 | E | The hook is selective for `Cargo.toml` patterns. | L168-L169 |
| F9-17 | E | A registry call/check occurs only on a local LRU existence-cache miss. | L168-L170 |
| F9-18 | E | The local existence cache has an approximate one-hour TTL. | L169-L170 |
| F9-19 | E | Caching is intended to keep network calls off the hot per-write path. | L169-L170 |
| F9-20 | E | Staleness thresholds are configurable per ecosystem. | L170-L170 |
| F9-21 | E | A generated `import nonexistent_pkg` is refused before disk. | L172-L173 |
| F9-22 | E | A refusal journals `write_time_slopsquat_blocked`. | L172-L173 |
| F9-23 | E | A 50-sample, 404-confirmed suite must allow zero hallucinated imports to reach disk. | L172-L174 |
| F9-24 | E | A suite over the 20 most-used real ELAI dependencies must produce zero false positives. | L173-L174 |
| F9-25 | E | The title promises provenance gating in addition to existence and age, although the summary body does not define its mechanism. | L160-L160 |

**Deep-inspection ambiguities:** the undefined provenance dimension; supported ecosystems inherited from D-11; whether “path matches an import/use/Cargo.toml pattern” means path, diff content, or both; network/cache-miss behavior under W-10; cache prewarming and negative caching; offline/timeouts/rate limits and fail-closed semantics; package-name normalization and namespace/typosquat rules; stale-threshold action (block/warn); exact publish-age source; lockfile/alias/path/git dependencies; Socket runbook ownership; and the 50/20 suite manifests.

### F-10 — Per-iteration security-monotonicity gate (the autonomous-loop blind spot)

**Classification/status as stated:** Cluster C10; `NEW`; novelty `GENUINELY_NEW`; feasibility `HARD`; grounding `ACCURATE`. Phase 1 is an L-effort classifier/calibration task; Phase 2 is S-M wiring. (L176-L191)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F10-01 | E | Every REFINE iteration fingerprints its generated delta by CWE class. | L179-L180 |
| F10-02 | E | Every auto-advance iteration fingerprints its generated delta by CWE class. | L179-L180 |
| F10-03 | E | Every self-mod iteration fingerprints its generated delta by CWE class. | L179-L180 |
| F10-04 | E | The gate compares vulnerability classes at iteration N against iteration N-1. | L179-L180 |
| F10-05 | E | A vulnerability class newly introduced at N is blocked or escalated. | L179-L180 |
| F10-06 | E | The summary records a verified external finding of `+37.6%` security degradation over five AI-improvement iterations. | L180-L182 |
| F10-07 | E | The summary records a verified-v3 external finding that standalone LLMs inject about 11x more vulnerabilities than humans. | L181-L182 |
| F10-08 | E | The `SecurityMonotonicityGate` structure is estimated at one to two days. | L183-L184 |
| F10-09 | E | The required CWE classifier does not yet exist in the stated ground truth. | L183-L185 |
| F10-10 | E | Rust calibration is called out as difficult because Semgrep coverage is sparse. | L183-L185 |
| F10-11 | E | Phase 1 builds the CWE classifier. | L183-L185 |
| F10-12 | E | Phase 1 calibrates it on historical ELAI REFINE deltas. | L184-L186 |
| F10-13 | E | Calibration targets a false-positive rate below 5%. | L184-L186 |
| F10-14 | E | Calibration uses 50 clean patches per language. | L184-L186 |
| F10-15 | E | Phase 2 wires the calibrated classifier into loop gates. | L186-L187 |
| F10-16 | E | The classifier runs as a post-write asynchronous check feeding the auditor. | L186-L187 |
| F10-17 | E | It is not a blocking per-write hook. | L186-L187 |
| F10-18 | E | Async placement avoids stalling the live REFLECT pipeline. | L186-L187 |
| F10-19 | E | `SecurityMonotonicityGate` is placed before REFINE commit through `chat_reflect.rs` / `live_cli.rs:~1685`. | L188-L189 |
| F10-20 | E | The gate is placed before auto-advance through `task_runner.rs:~3996`. | L188-L189 |
| F10-21 | E | The trigger injects CWE-95 at iteration N+1 when it was absent at N. | L189-L191 |
| F10-22 | E | Regression detection emits `security_class_regression_blocked`. | L189-L191 |
| F10-23 | E | Detection halts REFINE before the patch commits. | L190-L191 |
| F10-24 | E | The dashboard reasoning drawer displays the block. | L190-L191 |

**Deep-inspection ambiguities:** which languages satisfy “per language”; CWE-class taxonomy/granularity; baseline behavior at the first iteration; treatment of a removed then reintroduced class; classifier inputs and generated-delta boundaries; severity versus mere class presence; exact block-versus-escalate rule; async race/TOCTOU and how “post-write” remains “pre-commit”; rollback/working-tree state; missing explicit self-mod integration point; calibration labels; and whether the <5% threshold is aggregate or per language/class.

### F-2c — Low-FP CWE rules (798/321/532/95) on registered Py+TS + first measured baseline

**Classification/status as stated:** Post-F-10 extension; extension of F-2/C2; `HYBRID`; `EXTEND_EXISTING (high)`; feasibility `EASY`; effort `S-M`. The enclosing extension section is `PLANNED, implementation pending`, while this entry’s existing-ground-truth block asserts that F-2a’s 12 rules and F-2b’s bench already ship, but the bench has never run. (L193-L213)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F2C-01 | E | Add a high-precision hardcoded-credential predicate for CWE-798. | L199-L201 |
| F2C-02 | E | Add a high-precision hardcoded-key predicate for CWE-321. | L199-L201 |
| F2C-03 | E | Add a high-precision secrets-in-logs predicate for CWE-532. | L199-L201 |
| F2C-04 | E | Add a high-precision eval/exec-injection predicate for CWE-95. | L199-L201 |
| F2C-05 | E | Each new rule is represented as a `GrepPattern -> AstPattern` composite. | L199-L201 |
| F2C-06 | E | The rules use the already-registered Python grammar. | L200-L202 |
| F2C-07 | E | The rules use the already-registered TypeScript grammar. | L200-L202 |
| F2C-08 | E | `data/rules/web.toml` is the working rule template. | L200-L202 |
| F2C-09 | E | The shipped `elai bench seccodeplt` command is run to obtain the first measured static-gate insecure-rate baseline. | L201-L202 |
| F2C-10 | E | Existing F-2a is asserted to contain 12 live rules. | L203-L204 |
| F2C-11 | E | Existing F-2a is asserted to seed its security rules with `mutable=0`. | L203-L204 |
| F2C-12 | E | Existing F-2b’s `bench_seccodeplt.rs` is asserted to ship. | L203-L205 |
| F2C-13 | E | Existing F-2b’s `SecCodePltRunner` is asserted to ship. | L203-L205 |
| F2C-14 | E | Existing F-2b’s `InsecureRateMeasured` SSB support is asserted to ship. | L203-L205 |
| F2C-15 | E | The shipped F-2b benchmark is explicitly asserted never to have been run. | L203-L205 |
| F2C-16 | E | `TreeSitterAstEvaluator` registers 10 languages, including Python and TypeScript. | L205-L206 |
| F2C-17 | E | A prior claim of 16 registered languages is explicitly corrected. | L205-L206 |
| F2C-18 | E | `detect_language_from_ext` returns `None` for `.svelte`. | L205-L206 |
| F2C-19 | E | `detect_language_from_ext` returns `None` for `.html`. | L205-L206 |
| F2C-20 | E | `detect_language_from_ext` returns `None` for `.css`. | L205-L206 |
| F2C-21 | E | F-2c adds no dependency. | L207-L208 |
| F2C-22 | E | F-2c makes no `ast_adapter` edit. | L207-L208 |
| F2C-23 | E | F-2c adds no SSB variant. | L207-L208 |
| F2C-24 | E | Multi-language expansion beyond the registered Python/TypeScript path is deferred. | L207-L208 |
| F2C-25 | E | `ast-grep-core` support is deferred because it needs an `ast_adapter::new` edit. | L207-L208 |
| F2C-26 | E | The only existing-test edit changes the immutable-rule row count from 12 to 16. | L207-L208 |
| F2C-27 | E | Four new TOMLs are loaded from `crates/rules/seed/security/` by the existing loader. | L209-L210 |
| F2C-28 | E | Results surface through the existing `SecurityRulesPanel`. | L209-L210 |
| F2C-29 | E | F-2c requires no new frontend pair. | L209-L210 |
| F2C-30 | E | The live write trigger is a Python `password = "..."` edit for CWE-798. | L211-L212 |
| F2C-31 | E | That edit is blocked before write. | L211-L212 |
| F2C-32 | E | That block uses the existing `security_rule_blocked` signal. | L211-L212 |
| F2C-33 | E | One benchmark run emits one `insecure_rate_measured` row. | L212-L213 |
| F2C-34 | E | One benchmark run writes a `seccodeplt-*.json` report. | L212-L213 |
| F2C-35 | E | The report/claim must explicitly state that this is a static-gate, synthetic-corpus baseline. | L212-L213 |

**Deep-inspection ambiguities:** concrete predicates and negative guards that make the four rules low-FP; secret redaction in events/reports; Python/TypeScript AST node equivalence; whether JSX/TSX is included; exact existing loader path versus F-2’s earlier path; why the CLI command changed from `benchmark --suite` to `bench`; benchmark corpus/version/model and before/after definition; report schema/location; row-count test identity; and whether the enclosing “implementation pending” applies only to the four rules/run or also conflicts with the asserted shipped substrate.

### F-11 — Close one config-only DGM generation under existing guards (driver-only)

**Classification/status as stated:** Post-F-10 extension; `NEW topic`; `HYBRID`; `INTEGRATE_NEW_F_PLAN (high)`; feasibility `HARD`; effort `M`; enclosing status `PLANNED, implementation pending`. Its proposed production route is explicitly wire-dark behind both a compile feature and environment gate, a deliberate W-2 deviation. (L193-L195, L215-L236)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F11-01 | E | The existing DGM substrate is asserted to be built and tested but inert, with zero production callers. | L217-L220 |
| F11-02 | E | `config_archive` already provides sigmoid parent selection with `lambda=10`. | L217-L219 |
| F11-03 | E | `config_archive` already provides sigmoid parent selection with `alpha_0=0.5`. | L217-L219 |
| F11-04 | E | `config_archive` already provides `insert_variant`. | L217-L219 |
| F11-05 | E | `config_archive` already provides `select_parent`. | L217-L219 |
| F11-06 | E | `config_archive` already provides `sample_variant`. | L217-L219 |
| F11-07 | E | A `darwinian_bai` bandit is part of the existing substrate. | L218-L220 |
| F11-08 | E | `two_gate_promote` is one existing guard. | L219-L220 |
| F11-09 | E | `trap_tasks` cheat-pattern scoring is one existing guard. | L219-L220 |
| F11-10 | E | A BLAKE3 tripwire is one existing guard. | L219-L220 |
| F11-11 | E | A Layer-3 human gate exists in the guard substrate. | L219-L220 |
| F11-12 | E | F-11 adds the missing production driver for one configuration-only generation. | L220-L222 |
| F11-13 | E | The driver proposes a candidate. | L221-L222 |
| F11-14 | E | The driver evaluates the candidate on a real benchmark. | L221-L222 |
| F11-15 | E | The driver sends the candidate through the guard chain. | L221-L222 |
| F11-16 | E | An admitted candidate is inserted into the variant archive. | L221-L222 |
| F11-17 | E | Parent selection runs after archive insertion. | L221-L222 |
| F11-18 | E | `config_archive.rs:198/274/395` functions are asserted callerless. | L223-L224 |
| F11-19 | E | Current `darwin_loop_cmd.rs` is asserted to use a synthetic reward at line 104. | L223-L224 |
| F11-20 | E | Current `darwin_loop_cmd.rs` is asserted to terminate `RUNPOD-PENDING` at line 127. | L223-L225 |
| F11-21 | E | The current Darwin command is asserted not to touch the archive. | L223-L225 |
| F11-22 | E | `is_security_sensitive` is asserted to live on a separate path bypassed by current archive admission. | L225-L226 |
| F11-23 | E | Promotion-gate `tau/K_m` values are explicitly self-labelled `UNCONFIRMED`. | L225-L226 |
| F11-24 | E | Mutations are restricted to config/template variants. | L227-L228 |
| F11-25 | E | That restriction keeps the mutation within the bounded-capacity scope attributed to `2510.04399`. | L227-L228 |
| F11-26 | E | The config-only restriction avoids the Layer-3 human gate by construction. | L227-L228 |
| F11-27 | E | The driver must explicitly call `is_security_sensitive`. | L228-L229 |
| F11-28 | E | The driver must thread the real candidate through trap scoring. | L228-L230 |
| F11-29 | E | Acceptance requires proving the loop closed. | L229-L230 |
| F11-30 | E | Acceptance requires a guard to catch an injected trap mutant. | L229-L230 |
| F11-31 | E | A higher score alone is not acceptance evidence. | L229-L230 |
| F11-32 | E | A multi-generation live run is deferred. | L230-L230 |
| F11-33 | E | The driver integrates into the live `elai darwin-loop` entrypoint in `darwin_loop_cmd.rs`. | L231-L232 |
| F11-34 | E | F-11 adds `darwinian_admit` SSB telemetry. | L231-L232 |
| F11-35 | E | The ReasoningDrawer gains a `Darwinian Admits` frontend pair. | L231-L233 |
| F11-36 | E | Production activation is behind the `darwinian-evolve` feature. | L232-L233 |
| F11-37 | E | Production activation is also behind `ELAI_DARWINIAN_LOOP_ENABLE`. | L232-L234 |
| F11-38 | E | The dual gate is an explicit, deliberate exception to W-2 for an autonomous self-modification loop. | L232-L233 |
| F11-39 | E | The done-trigger runs `ELAI_DARWINIAN_LOOP_ENABLE=1 elai darwin-loop --generations 1`. | L234-L236 |
| F11-40 | E | The done-trigger injects a trap mutant. | L234-L235 |
| F11-41 | E | The trigger emits at least two `darwinian_admit` rows. | L234-L236 |
| F11-42 | E | One row has disposition `RejectedByGuard`. | L235-L236 |
| F11-43 | E | `RejectedByGuard` names the guard that fired. | L235-L236 |
| F11-44 | E | One row has disposition `LoopClosed`. | L235-L236 |
| F11-45 | E | `LoopClosed` proves `select_parent` ran after `insert_variant`. | L235-L236 |

**Deep-inspection ambiguities:** proposal algorithm and variant schema; real benchmark identity and deterministic receipt; full guard order and short-circuiting; how a trap candidate can be rejected while the same trigger proves insertion then parent selection; whether two candidates/attempts are involved; row correlation/order and exact payload; archive transaction/rollback; what `is_security_sensitive` does for a config-only candidate; why Layer-3 is avoided while that check remains mandatory; confirmed `tau/K_m` values; compile-feature/env defaults; W-2 exception approval; and RunPod versus local one-generation behavior.

### F-12 — Semantic contradiction pre-filter: local deterministic NLI judge (trigger-only)

**Classification/status as stated:** Post-F-10 extension; `NEW topic`; `HYBRID`; `EXTEND_EXISTING (high)`; feasibility `HARD (net-new model)`; effort `M`; enclosing status `PLANNED, implementation pending`. Tier-1 is a trait seam plus deterministic fake-judge fixture with the model dark; Tier-2 is the ops-gated ONNX model. (L193-L195, L238-L258)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F12-01 | E | The detector mirrors Graphiti/Zep contradiction invalidation semantics. | L240-L242 |
| F12-02 | E | Invalidation is non-destructive. | L240-L242 |
| F12-03 | E | The invalidation time is the invalidating fact’s `valid_at`. | L240-L242 |
| F12-04 | E | Contradiction judgment uses a local NLI cross-encoder. | L240-L242 |
| F12-05 | E | The NLI judge uses deterministic argmax. | L240-L242 |
| F12-06 | E | The model runs on ELAI’s own ort/ONNX runtime. | L241-L242 |
| F12-07 | E | A cloud LLM judge is excluded because it would break `ELAI_SEED` under the cited evidence. | L241-L243 |
| F12-08 | E | NLI runs as a second pass over the Surface residue from `audit_conflicts`. | L242-L243 |
| F12-09 | E | The second pass is localized to approximately two rows per call. | L242-L243 |
| F12-10 | E | Existing `audit_conflicts` only pairs byte-identical keys. | L244-L245 |
| F12-11 | E | Existing `audit_conflicts` is asserted wire-dark with zero production callers. | L244-L246 |
| F12-12 | E | Existing `supersede` is a non-destructive storage primitive. | L245-L247 |
| F12-13 | E | Existing `soft_close` is a non-destructive storage primitive. | L245-L247 |
| F12-14 | E | `MemoryConflictDetected.conflict_kind` is free-form. | L246-L247 |
| F12-15 | E | The free-form field allows reuse without an SSB schema bump. | L246-L248 |
| F12-16 | E | `reranker.rs` supplies the deterministic ort-loading pattern. | L247-L248 |
| F12-17 | E | No NLI model is staged in the stated ground truth. | L247-L248 |
| F12-18 | E | Storage schema/path remains unchanged because this is trigger-only. | L249-L250 |
| F12-19 | E | High-confidence results may auto-close the older fact. | L249-L250 |
| F12-20 | E | `SurfaceToUser` is the safe default disposition. | L249-L250 |
| F12-21 | E | The feature reuses `MemoryConflictDetected`. | L249-L251 |
| F12-22 | E | Reused events set `conflict_kind="semantic_contradiction"`. | L249-L251 |
| F12-23 | E | The feature causes no SSB schema bump. | L249-L251 |
| F12-24 | E | The integration becomes the first production caller of `audit_conflicts`. | L251-L251 |
| F12-25 | E | Determinism is framed as practically stable, not bit-identical. | L251-L252 |
| F12-26 | E | The DKG exact-text join is left unchanged. | L252-L252 |
| F12-27 | E | F-5 age quarantine is left unchanged. | L252-L252 |
| F12-28 | E | A new `crates/memory/semantic_contradiction.rs` module is added. | L253-L254 |
| F12-29 | E | A new `crates/memory/nli_onnx.rs` module is added. | L253-L254 |
| F12-30 | E | The contradiction pass is wired into the `live_cli.rs` memory tick beside F-5. | L253-L255 |
| F12-31 | E | `MemoryFactsDrawer` gains a `Semantic Contradictions` frontend pair. | L253-L255 |
| F12-32 | E | The Tier-2 NLI ONNX model is ops-gated. | L254-L255 |
| F12-33 | E | Tier-1 ships an `NliJudge` trait seam. | L254-L255 |
| F12-34 | E | Tier-1 ships a deterministic fixture harness with a fake judge while the model remains dark. | L254-L258 |
| F12-35 | E | With the model staged, the live trigger evaluates a cross-key contradictory fact pair. | L256-L258 |
| F12-36 | E | The older row is soft-closed. | L256-L258 |
| F12-37 | E | Soft close sets `older.valid_to_ms == newer.valid_from_ms`. | L256-L258 |
| F12-38 | E | The live trigger emits `memory_conflict_detected`. | L256-L258 |
| F12-39 | E | The live event uses `conflict_kind="semantic_contradiction"`. | L256-L258 |

**Deep-inspection ambiguities:** NLI model identity/version/license/hash and staging path; input language/domain; candidate-pair generation beyond exact keys; meaning of Surface residue and why the set is about two; entailment/neutral/contradiction label mapping; confidence threshold/calibration; distinction between `SurfaceToUser` and auto-close; model-missing/error fallback; CPU/GPU determinism and tolerated drift; scheduling/budget per tick; privacy/logging of fact text; concurrent updates; whether `invalid_at` maps exactly to `valid_to_ms`; and how the ops gate is reconciled with W-2.

### F-13 — Session archive on SSB schema-bump (wire-dark, EXTEND_EXISTING)

**Classification/status as stated:** Post-F-10 extension; `NEW topic`; `HYBRID`; `EXTEND_EXISTING (high)`; feasibility `EASY`; effort `S`; enclosing status `PLANNED, implementation pending`; title explicitly says wire-dark. The implementation has an unresolved W-2 disposition. (L193-L195, L260-L276)

| ID | Evidence | Atomic feature or behavior | Source |
|---|---|---|---|
| F13-01 | E | Before a destructive SSB schema-bump drop, ELAI archives the stale `ssb.db`. | L262-L264 |
| F13-02 | E | The archive prevents silent loss of audit history during schema bump. | L262-L264 |
| F13-03 | E | The design does not adopt Springdrift’s Backup-Actor mechanism. | L262-L264 |
| F13-04 | E | Existing `run_migration` is asserted to drop prior-version `ssb` at the one destructive site. | L265-L266 |
| F13-05 | E | Existing `run_migration` is asserted to drop prior-version `ssb_files` at that site. | L265-L266 |
| F13-06 | E | The existing site labels old-session data loss acceptable. | L265-L267 |
| F13-07 | E | Session-DAG migration is explicitly corrected to non-destructive. | L266-L267 |
| F13-08 | E | Existing checkpoints store hashes, not file bytes. | L267-L268 |
| F13-09 | E | ELAI has no Backup Actor in the stated ground truth. | L267-L268 |
| F13-10 | E | ELAI has no unified store in the stated ground truth. | L267-L268 |
| F13-11 | E | The working tree is represented as an external-git pointer. | L267-L268 |
| F13-12 | E | Recovery is by SSB replay under `ELAI_SEED`. | L268-L268 |
| F13-13 | E | Revised scope adds one file copy. | L269-L270 |
| F13-14 | E | Revised scope adds one guard at the destructive drop site. | L269-L270 |
| F13-15 | E | Archiving is gated by `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE`. | L269-L270 |
| F13-16 | E | Periodic commits/archives are out of scope. | L269-L270 |
| F13-17 | E | No Git dependency is added. | L269-L270 |
| F13-18 | E | No unified store is added. | L269-L270 |
| F13-19 | E | Gaps numbered 1, 3, and 4 are deferred. | L270-L271 |
| F13-20 | E | The documented fallback is “document the gap; build nothing.” | L270-L271 |
| F13-21 | E | W-2 disposition is explicitly left open for the implementer. | L270-L271 |
| F13-22 | E | A new `crates/orchestrator/session_archive.rs` module is added. | L272-L273 |
| F13-23 | E | The archive call executes inside `run_migration` before the drop. | L272-L273 |
| F13-24 | E | The destructive drop still proceeds after archiving. | L272-L273 |
| F13-25 | E | Archiving emits `session_archived` in the SSB. | L272-L273 |
| F13-26 | E | A session/recovery dashboard drawer is the frontend pair. | L272-L273 |
| F13-27 | E | The done-trigger opens a stale-version `ssb.db` with the flag enabled. | L274-L276 |
| F13-28 | E | The archive path is `workflow/sessions/{id}/archive/ssb-v{N}.sqlite`. | L274-L275 |
| F13-29 | E | The copied archive is BLAKE3-verified. | L274-L275 |
| F13-30 | E | The trigger records a `session_archived` row. | L274-L275 |
| F13-31 | E | The current database is freshly recreated after the archive. | L275-L276 |
| F13-32 | E | Preserving the existing drop/recreate behavior is part of the done-gate. | L275-L276 |

**Deep-inspection ambiguities:** meanings of deferred gaps #1/#3/#4; exact flag default and W-2 resolution; whether failure to archive blocks or permits the destructive drop; SQLite/WAL-consistent copy procedure and locks; atomic rename/fsync/disk-full/permission handling; source and sanitization of `{id}` and `{N}`; collision and retry behavior; retention/cleanup; where the BLAKE3 digest is stored; whether `session_archived` lands in the old or recreated SSB and survives the drop; restoration/replay workflow; and how “fallback = document-the-gap-build-nothing” affects implementation acceptance.

## End-to-end workflows and feature interactions

The `E` steps below restate explicit intra-entry sequences. The `I` steps are conservative cross-entry composition hypotheses for the later full-corpus synthesis and must be checked against the original plans and wiring doctrine.

### 1. Untrusted input to pre-disk write decision

1. **E — Tool minimization:** for researcher tasks, F-3 locks a per-task minimal tool set before any repo/web/skill/MCP byte is observed. An injected instruction cannot invoke an out-of-set tool. (L62-L73)
2. **E — Compound hook:** a file-writing tool reaches the F-1 `PreToolUse` path, which dispatches the in-process compound VibeGate and blocks if any active axis fires. (L20-L34)
3. **E — Tier-1 rule sources:** F-2/F-2c populate immutable security rules, while F-1’s Tier-1 evaluates vulnerability and secret patterns without network/subprocess work. (L28-L32, L44-L54, L197-L213)
4. **E — Dependency route:** F-9 selectively examines import/use/Cargo patterns, consults a local existence cache, and blocks a confirmed nonexistent package before disk. (L163-L174)
5. **E — Receipt/surface:** a refusal prevents the write, emits the gate-specific SSB event, and appears on the relevant dashboard surface. (L33-L39, L52-L57, L71-L73, L172-L174, L209-L213)
6. **I — Likely composition:** F-2/F-2c appear to be the concrete code-vulnerability rule providers consumed by F-1’s compound evaluator. The summary does not explicitly state the registration/dispatch contract; inspect F-1, F-2, and F-2c originals together.
7. **I — Likely composition:** F-9 appears to close all or part of F-1’s deferred dependency-existence axis. The summary calls F-9 an extension to D-11/E-41 rather than explicitly naming F-1 Tier-2, and its cache-miss network wording must be reconciled with W-10.
8. **I — Boundary interaction:** F-3 constrains *which* tools may be called, while F-1/F-2/F-9 constrain *what a permitted write tool may commit*. This separation is consistent with their stated tool-filter versus write-gate roles, but ordering beyond “F-3 before untrusted bytes” is not specified.

### 2. Iterative code generation to commit

1. **E:** REFINE, auto-advance, and self-mod iterations produce deltas fingerprinted by CWE class. (L176-L186)
2. **E:** the classifier operates asynchronously after write, feeds the auditor, and is explicitly not a blocking per-write hook. (L183-L187)
3. **E:** before REFINE commit or auto-advance, a newly introduced class causes `security_class_regression_blocked` and halts the REFINE patch. (L188-L191)
4. **I:** F-1/F-2 protect individual write operations against known patterns, while F-10 compares security state across iterations and can detect a class-level regression that survives or lies outside individual rules. The exact deduplication and precedence contracts are not stated.

### 3. Test execution to truth-bound phase transition

1. **E:** the auditor’s test must be moved into an orchestrator-managed process slot so the orchestrator owns the stdout handle. (L83-L87)
2. **E:** the orchestrator hashes stdout with BLAKE3 and correlates it to `TestOutcomePayload` in the same superstep. (L83-L85)
3. **E:** an auditor cannot claim `passed=true` without the matching captured row. (L83-L85)
4. **E:** a forged PASS over an exit-1 process blocks `AUDITING -> MEMORIZING`, emits `anti_fabrication_block`, and appears on the audit timeline. (L89-L91)
5. **I:** F-4 supplies off-agent evidence for test claims, while later semantic output grounding remains absent; successful hash binding proves provenance/integrity of captured output, not that the output meets a spec.

### 4. Memory aging and semantic contradiction handling

1. **E:** F-5 time decay quarantines a fact and emits one `StalenessQuarantineBroadcast`. (L93-L107)
2. **E:** a bounded fan-out propagates demotion to DKG concept annotations, the grounding cache, and Cosmos artifacts. (L99-L109)
3. **E:** within one tick, dependent concepts become stale and re-query misses the grounding cache; a mock clock proves the behavior. (L104-L109)
4. **E:** on the live memory tick, F-12 first invokes the formerly wire-dark `audit_conflicts`, then applies a local NLI second pass to candidate residue. (L238-L255)
5. **E:** a high-confidence cross-key contradiction can non-destructively soft-close the older fact and journal `memory_conflict_detected`; otherwise `SurfaceToUser` is the safe default. (L249-L258)
6. **E:** F-12 explicitly leaves F-5 age quarantine and the DKG exact-text join unchanged. (L251-L252)
7. **I:** F-5 and F-12 are sibling consumers of the same memory tick: one reacts to age/staleness and fans out invalidation, while the other reacts to semantic contradiction. Their order, conflict resolution, and duplicate-event behavior are not specified.

### 5. Learned-skill acquisition to autonomous dispatch

1. **E:** a candidate skill follows curriculum -> generation -> execution -> critic gate -> optional retry -> commit, with embedding retrieval. (L114-L115)
2. **E:** acquisition reuses the triggering BestOfN MiniCritic SSB verdict instead of running redundant critics. (L118-L121)
3. **E:** the committed skill carries a distinct S1-S4 `SkillTrustTier`, whose promotion considers invocation count, failure rate, elapsed days, and a human gate at the top tier. (L115-L120)
4. **E:** the autonomous-mode gate refuses an S2 skill without human approval. (L123-L125)
5. **E:** stale skills are demoted and both promotion/demotion are journaled and shown in the skill-library panel. (L121-L125)
6. **I:** the summary does not say whether a stale demotion invalidates embedding-retrieval results or only changes dispatch authorization; inspect C6c and the autonomous gate.

### 6. Template learning and selection

1. **E:** F-6’s `TemplateCrystallizer` critic decides whether a recurring learned skill/template is acquired, reusing an SSB verdict. (L112-L123)
2. **E:** F-7 independently replaces first-try-ratio ranking with Laplace-weighted outcome utility over existing SSB events. (L127-L141)
3. **E:** after at least 100 labelled sessions, F-7 reports a measured P@k delta against the first-try baseline. (L137-L141)
4. **I:** F-6 governs acquisition/trust and F-7 governs retrieval/ranking of pipeline templates; whether they share the same template identity/store is plausible from the names but not made explicit in the summary.

### 7. Guarded self-modification generation

1. **E:** F-11’s driver proposes one config/template mutant, evaluates it on a real benchmark, invokes `is_security_sensitive`, and threads the real candidate through guard scoring. (L215-L230)
2. **E:** the guard chain includes two-gate promotion, trap-task patterns, a BLAKE3 tripwire, and the existing Layer-3 human gate; the config-only restriction is intended to avoid Layer-3 by construction. (L217-L230)
3. **E:** F-8 performs an LLM-free symbolic conflict review before archive admission and warns/escalates a locked-key contradiction. (L143-L158)
4. **E:** if admitted, F-11 inserts the variant and then selects a parent; the loop emits `darwinian_admit` receipts proving rejection and closure behavior. (L221-L236)
5. **E:** F-10 also promises per-iteration CWE-class regression checking for self-modification, although its explicit integration list names REFINE and auto-advance rather than a self-mod file. (L176-L190)
6. **E:** F-7’s immutable safety-lattice verifier proves the security/finance rule substrate complete, monotone, and conflict-free once E-53 prerequisites resolve. (L127-L141)
7. **I — Candidate ordering:** a sensible combined order is propose -> real benchmark -> security-sensitive classification -> symbolic conflict review -> other guards/security-monotonicity -> archive insert -> parent selection. The summary does not canonically order F-8, F-10, and F-11, so no implementation should rely on this inferred sequence without original-plan inspection.

### 8. Destructive SSB schema migration with retained evidence

1. **E:** when a stale-version SSB is opened with `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE`, F-13 copies the old database before the destructive migration drop. (L260-L275)
2. **E:** the copy is placed at `workflow/sessions/{id}/archive/ssb-v{N}.sqlite` and BLAKE3-verified. (L274-L275)
3. **E:** migration still drops/recreates the current database and emits `session_archived`, with a session/recovery dashboard surface. (L272-L276)
4. **I:** because F-plan gates depend on SSB receipts, F-13 protects the historical evidence used by audit/replay across schema versions. The summary states the audit-history motivation but does not define restoration, event placement, or whether all F event schemas survive replay.

## Cross-cutting invariants and non-negotiable contracts

| Invariant | Evidence | Applies to | Source |
|---|---|---|---|
| Completion is an exercised production path, not compilation alone. | E | Every F-N | L8-L13, L278-L284 |
| Missing/failed axis evidence on a compound write gate must result in fail-closed blocking; any firing axis vetoes disk mutation. | E | F-1 Tier-1 | L20-L39 |
| Project-local edits cannot mutate/disable the immutable VibeSec security rows. | E | F-1, F-2/F-2c | L20-L30, L203-L204 |
| Startup must detect tampering of immutable seed rows through BLAKE3 integrity verification. | E | F-1 | L28-L30 |
| Network, subprocess, and RunPod operations stay out of the fail-closed hot path. | E | Global/F-1/F-9/F-10 | L28-L32, L169-L170, L183-L187, L284-L284 |
| Tool capabilities are fixed before untrusted data is observed. | E | F-3 | L66-L73 |
| Security-performance headlines use ELAI/Qwen measurements, never transferred CodeGen or synthetic numbers. | E | F-2/F-7 | L46-L57, L127-L141 |
| A test PASS needs orchestrator-owned output evidence correlated in the same superstep. | E | F-4 | L78-L91 |
| Memory invalidation remains non-destructive/bi-temporal. | E | F-5/F-12 | L96-L100, L240-L258 |
| Cross-subsystem staleness fan-out is bounded with an explicit channel capacity. | E | F-5 | L101-L109 |
| Memory trust and skill trust remain separate Rust/SSB types and namespaces. | E | F-6 | L111-L125 |
| Sub-top skill trust cannot silently grant autonomous execution. | E | F-6 | L114-L125 |
| Safety-lattice proof treats immutable rules as a set and follows, rather than precedes, E-53 proof-stub resolution. | E | F-7 | L127-L141 |
| SCF review is limited to self-mod in current revised scope; inter-agent BSP use stays deferred. | E | F-8 | L143-L158 |
| A confirmed nonexistent dependency never reaches disk in the acceptance suite. | E | F-9 | L160-L174 |
| An iterative security regression halts before commit even though classification is post-write asynchronous. | E | F-10 | L183-L191 |
| F-11 acceptance is guard/loop evidence, not score improvement. | E | F-11 | L227-L236 |
| F-12 does not claim bit-identical ML inference; its contract is practical stability plus a deterministic fake-judge harness. | E | F-12 | L249-L258 |
| F-13 archives additively; the existing drop and fresh database recreation are preserved. | E | F-13 | L269-L276 |
| Every visible signal has a frontend producer/consumer pair and passes cross-language diff validation. | E | Every surface listed below | L278-L284 |

## State, data, event, command, configuration, and surface registry

### SSB/event signals explicitly named

| Signal | Producer/trigger | Intended consumer/surface | Source |
|---|---|---|---|
| `vibesec_code_vuln_blocked` | F-1 known vulnerability pattern blocked before disk | VibeGate dashboard drawer | L33-L39 |
| `vibesec_secret_blocked` | F-1 hardcoded secret blocked before disk | VibeGate dashboard drawer | L33-L39 |
| `security_rule_blocked` | F-2/F-2c security rule blocks a write | Existing `SecurityRulesPanel` explicitly named for F-2c | L52-L57, L209-L213 |
| `insecure_rate_measured` / `InsecureRateMeasured` | F-2b/F-2c SeCodePLT run | Phase TOML/JSON benchmark artifacts; UI not stated | L54-L57, L203-L213 |
| `tool_filter_committed` | F-3 commits the per-task researcher tool filter | No panel named in the entry; global frontend-pair rule applies if surfaced | L71-L73, L281-L281 |
| `anti_fabrication_block` | F-4 detects unbound claimed PASS | Audit-timeline dashboard | L89-L91 |
| `StalenessQuarantineBroadcast` | F-5 memory decay quarantine | DKG, grounding cache, Cosmos; dashboard staleness panel | L96-L109 |
| `skill_trust_promoted` | F-6 trust promotion | Skill-library panel | L118-L125 |
| `skill_demoted_stale` | F-6 staleness demotion | Skill-library panel | L121-L125 |
| `safety_lattice_verified` | F-7 immutable rule-set proof succeeds | CLI result; no dashboard named | L134-L141 |
| `template_utility_ranked` | F-7 outcome-utility ranking | Measured P@k evaluation; no dashboard named | L136-L141 |
| `symbolic_conflict_detected` | F-8 pre-admission locked-key contradiction | No dashboard named | L151-L158 |
| `write_time_slopsquat_blocked` | F-9 nonexistent dependency refused | No dashboard named | L167-L174 |
| `security_class_regression_blocked` | F-10 new CWE class between iterations | Dashboard reasoning drawer | L188-L191 |
| `darwinian_admit` | F-11 guard admission/loop driver | ReasoningDrawer “Darwinian Admits” | L231-L236 |
| `memory_conflict_detected` / `MemoryConflictDetected` | F-12 semantic contradiction | MemoryFactsDrawer “Semantic Contradictions” | L244-L258 |
| `session_archived` | F-13 pre-drop archive | Session/recovery drawer | L272-L276 |

### Other explicit data/lifecycle fields and artifacts

| Item | Contract | Source |
|---|---|---|
| `mutable` | Defaults to `1`; VibeSec/security seeds use `0`; immutable subsets feed the lattice proof. | L21-L30, L127-L140, L203-L208 |
| `TestOutcomePayload` | Existing `{test_name, passed, duration_ms}` plus proposed `stdout_hash: Option<String>`. | L78-L88 |
| Same-superstep hash row | Must match a `passed=true` test outcome and be orchestrator-captured. | L83-L91 |
| `concept_stale` | Must receive consumer-contract testing before first real population; quarantine’s done-effect sets dependent concepts to `1`. | L99-L109 |
| `SkillTrustTier` | S1-S4, separate from memory `TrustTier` T0-T3. | L111-L125 |
| `insecure_rate_before/after` | Written to a phase TOML by F-2b’s command. | L49-L57 |
| `seccodeplt-*.json` | Synthetic/static-gate benchmark report from F-2c. | L211-L213 |
| Local LRU dependency cache | Approximate one-hour TTL; thresholds configurable per ecosystem. | L167-L171 |
| Config variant archive | Provides insert/sample/select and sigmoid parent choice; F-11 closes one live generation. | L217-L236 |
| `MemoryConflictDetected.conflict_kind` | Free-form; semantic NLI reuses it with `semantic_contradiction`, avoiding a schema bump. | L244-L258 |
| Bi-temporal close | For a detected pair, `older.valid_to_ms == newer.valid_from_ms`. | L256-L258 |
| SSB schema-bump archive | `workflow/sessions/{id}/archive/ssb-v{N}.sqlite`, BLAKE3-verified. | L260-L276 |

### Exact commands, flags, and configurable values

| Name | Role/status | Source |
|---|---|---|
| `wiring_contract_check.py --run-trigger` | Mandatory W-1 done-gate for every F session. | L8-L13, L278-L280 |
| `elai benchmark --suite seccodeplt` | F-2b spelling; writes phase TOML and event. | L49-L57 |
| `elai bench seccodeplt` | F-2c spelling; asserted shipped but never run. | L197-L213 |
| `elai verify safety-lattice` | F-7 immutable-lattice verification command. | L139-L140 |
| `ELAI_DARWINIAN_LOOP_ENABLE=1 elai darwin-loop --generations 1` | F-11 one-generation trap trigger. | L231-L236 |
| `darwinian-evolve` | F-11 compile feature; contributes to wire-dark/W-2 exception. | L231-L233 |
| `ELAI_DARWINIAN_LOOP_ENABLE` | F-11 runtime environment gate. | L232-L235 |
| `ELAI_SEED` | Determinism/replay boundary cited for F-12 and F-13. | L240-L243, L267-L268 |
| `ELAI_SSB_ARCHIVE_ON_BUMP_ENABLE` | F-13 archive-before-drop runtime gate; W-2 disposition open. | L269-L275 |
| Dependency stale threshold | Configurable per ecosystem; no numeric defaults stated. | L167-L171 |
| Existence-cache TTL | Approximately one hour. | L168-L170 |
| F-5 fan-out capacity | Must be explicit and bounded; numeric value unstated. | L101-L105 |
| DGM parent selector | `lambda=10`, `alpha_0=0.5`; source says promotion `tau/K_m` remain unconfirmed. | L217-L226 |

## Dependency, status, and deferred-work matrix

Status phrases below are kept literal. “Unstated” means the summary describes a plan/scope but does not safely establish shipped or unshipped state.

| ID/scope | Stated implementation/wiring state | Hard prerequisite or gate | Explicitly deferred/excluded | Source |
|---|---|---|---|---|
| F-1 Tier-1 | “ships now” in revised scope; four-axis compound is also stated not wired in current ground truth | W-1/W-4/W-5/W-10; release requires measured FP on ≥3 runs | Tier-2 dependency existence and MCP scan behind design gate | L17-L39 |
| F-2a | Revised scope says ship Python/TS rules and block; F-2c later asserts 12 rules ship live | Immutable security domain/PreToolUse | Performance claim withheld | L41-L57, L197-L213 |
| F-2b | F-2c asserts bench/runner/event ship but benchmark was never run | C-26/E-35, RunPod, E-9 harness | Headline until measured Qwen result | L41-L57, L203-L205 |
| F-3 Phase 1 | “ship first”; no definitive current state | Existing FSM `PreToolUse`, researcher dispatch, AgentDojo-style fixture | Full CaMeL Phase 2 | L59-L73 |
| F-3 Phase 2 | Conditional XL | Primary PDF overhead, measured ELAI IPC overhead, 32K survival, breaking-change acceptance | Not authorized until all conditions pass | L66-L70 |
| F-4 | Unstated; blocking design question must be pre-resolved | Orchestrator-managed process slot; C-27; C-28 | Semantic output grounding to a later F-N | L75-L91 |
| F-5 Phase 1 | Unstated | After B-30; audit all consumers; `concept_stale=0` retrieval contract | Population before audit/test is prohibited by ordering | L93-L109 |
| F-5 Phase 2 | Unstated | Phase-1 safety work; bounded explicit-capacity queue | Unbounded fan-out | L101-L109 |
| F-6 C6a/C6b/C6c | Unstated | Distinct trust type/tags; existing MiniCritic SSB verdict; autonomous gate | 70%-reuse claim until `concept_class` plus ≥20 acquisitions | L111-L125 |
| F-7 lattice | Unstated; `RUNPOD-PENDING` prerequisite | Strictly after E-53 Z3/Kani/Verus stubs resolve | Bundling with CBR; synthetic target | L127-L141 |
| F-7 CBR | Standalone plan; current implementation unstated | Existing SSB outcomes; ≥100 labelled sessions | `+10.7%` target dropped | L127-L141 |
| F-8 self-mod | Summary says Pillar synthesis does not ship; revised scope is proposed | Self-mod promotion/archive path | Inter-agent BSP half | L143-L158 |
| F-8 BSP | `RUNPOD-PENDING` and deferred | Resolve FP/noise and C-50 collision; evaluate Graphiti cloud alternative | Production BSP enforcement | L151-L155 |
| F-9 | Unstated | D-11 must ship; local cache; per-ecosystem config | Socket Firewall implementation moved to ops runbook | L160-L174 |
| F-10 Phase 1 | Classifier explicitly does not exist in stated ground truth | Historical deltas; <5% FP on 50 clean patches/language | Wiring before calibration | L176-L187 |
| F-10 Phase 2 | Unstated | Completed calibrated classifier | Blocking per-write placement | L183-L191 |
| F-2c | Enclosing section: `PLANNED, implementation pending`; extends live F-2 substrate | Existing loader/grammars/panel/bench | New dep, adapter edit, SSB variant, multi-language, ast-grep-core | L193-L213 |
| F-11 | `PLANNED, implementation pending`; explicitly wire-dark | Feature + env gate; real bench; security-sensitive call; real trap candidate | Multi-generation live run | L193-L236 |
| F-12 Tier-1 | `PLANNED, implementation pending`; trait seam/fake harness “ships now” in scoped language | Existing ort pattern and memory tick | Live model execution while model remains unstaged | L193-L258 |
| F-12 Tier-2 | No NLI model staged; ops-gated | Stage local ONNX model | Cloud LLM judge | L238-L258 |
| F-13 | `PLANNED, implementation pending`; wire-dark | Env flag; open W-2 disposition | Periodic committer, Git dependency, unified store, gaps #1/#3/#4 | L193-L195, L260-L276 |

## Verification, error/fallback, security/privacy, and operations coverage

### Explicit verification and acceptance fixtures

| Feature | Fixture/metric | Passing contract | Source |
|---|---|---|---|
| F-1 | Known SQLi or hardcoded secret through Edit/Write; synthetic corpus; ≥3 real orchestrate runs | Block before disk; correct SSB row/panel; zero known matches/secrets on corpus; bounded FP | L28-L39 |
| F-2a | Generated SQLi snippet | `security_rule_blocked`; no pre-write landing | L48-L54 |
| F-2b | SeCodePLT/CodeGuard+ on Qwen | Phase TOML plus `insecure_rate_measured`, insecure rate beside pass@1 | L49-L57 |
| F-3 | AgentDojo-style researcher fixture | `tool_filter_committed`, ASR drop, no out-of-set tool execution | L66-L73 |
| F-4 | Exit-1 baby step plus auditor-claimed PASS | `anti_fabrication_block` at AUDITING->MEMORIZING | L83-L91 |
| F-5 | Mock-clock quarantine | Dependent DKG stale and cache miss within one tick | L101-L109 |
| F-6 | S2 autonomous dispatch without human gate | Dispatch blocked; tier shown; trust/demotion rows available | L118-L125 |
| F-7 lattice | Exhaustive immutable-pair verifier | Zero conflict pairs and `safety_lattice_verified` | L134-L141 |
| F-7 CBR | ≥100 labelled sessions | `template_utility_ranked` and measured P@k delta | L136-L141 |
| F-8 | DGM-class locked-key reward-hack variant | `symbolic_conflict_detected` before archive admission | L151-L158 |
| F-9 | 50 confirmed 404s plus 20 real common dependencies | Zero hallucinated imports land and zero FP on common dependencies | L167-L174 |
| F-10 | Inject CWE-95 at N+1, absent at N | Regression event and halt before REFINE commit | L183-L191 |
| F-2c | Python hardcoded password plus SeCodePLT run | Pre-write block; one event; one caveated JSON report | L197-L213 |
| F-11 | One generation with injected trap | ≥2 admission rows: named-guard rejection and loop closure after insert/select | L227-L236 |
| F-12 | Fake-judge deterministic fixture now; staged-model cross-key pair later | Older fact soft-closes with exact timestamp equality and semantic-conflict event | L249-L258 |
| F-13 | Open stale DB with archive flag | Verified archive, durable event, and fresh recreated current DB | L269-L276 |

### Explicit fallback/negative-scope behavior

- F-1 blocks fail-closed if any active in-process axis fires; the summary does **not** define evaluator-error behavior. (L20-L39)
- F-3 forbids out-of-set tool invocation; no dynamic-expansion fallback is stated. (L66-L73)
- F-4 treats missing hash on a claimed PASS as a fabrication signature; semantic correctness remains deferred. (L83-L91)
- F-5 requires bounded fan-out to prevent tick backlog; retry/drop policy is unstated. (L101-L109)
- F-6 routes insufficiently trusted autonomous skill execution to a human gate. (L114-L125)
- F-8 uses WARN-then-escalate for the low-precision self-mod detector and declines BSP deployment. (L151-L155)
- F-9 moves Socket wrapping to an ops runbook; network/offline fallback is unstated. (L167-L174)
- F-10 uses post-write async auditing to avoid live-pipeline stalls, then halts before commit on regression. (L183-L191)
- F-11 deliberately remains behind two enablement gates because it is autonomous self-modification. (L231-L234)
- F-12 defaults to `SurfaceToUser`, excludes a cloud judge, and can ship a fake-judge harness while the real model remains dark. (L240-L258)
- F-13 names “document the gap; build nothing” as fallback and preserves the destructive drop if archiving succeeds; archive-failure behavior is unstated. (L269-L276)

### Security and privacy observations bounded to the source

- Explicit security surfaces include immutable security rules, pre-disk vulnerability/secret/dependency blocks, untrusted-content capability restriction, test-claim anti-fabrication, skill autonomy gating, self-mod conflict/guard chains, and schema-bump evidence preservation.
- Locality is explicit for F-12’s NLI model; a cloud judge is disallowed for seeded determinism. The summary does not otherwise specify privacy policy, secret redaction, telemetry retention, network disclosure, or user-data consent.
- F-3 names repo, web, skill, and MCP bytes as untrusted; no comparable taint label/lifecycle is specified in this summary.
- F-9 performs real registry HTTP and therefore has an external-data boundary, but request privacy/proxying/logging is not specified.
- F-13 preserves potentially sensitive SSB history in a new archive, but permissions, encryption, retention, and deletion policy are not specified.

## Deeper-source manifest

### A. F-plan documents and research sources

These files were located by filename only for delegation. Except for the F summary (fully read) and the overview (orientation only), their contents were deliberately not inspected in this summary pass.

| Scope | Full path | Lines | Why the deeper agent must read it |
|---|---|---:|---|
| Summary source | `docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md` | 286 | Primary source of this discovery; use as the coverage checklist and reconcile any original-plan deltas. |
| F enforcement index | `docs/thefinalglance/F-plan/F-0_INDEX.md` | 233 | Explicitly cited for W-1 through W-10; resolve gate semantics, exceptions, fixture requirements, and completion status. |
| Wiring doctrine sibling | `docs/thefinalglance/F-plan/F-WIRING-DOCTRINE-WHY.md` | 120 | Located in the F-plan folder but not explicitly cited by filename in the summary; determine whether it is normative or explanatory and reconcile it with F-0. |
| F-1 original | `docs/thefinalglance/F-plan/F-1_vibegate_unified_write_gate.md` | 220 | Resolve compound evaluator, rule-domain naming, fail-closed errors, tool contracts, Tier-2 cache, FP bound, and all four axes. |
| F-2 original | `docs/thefinalglance/F-plan/F-2_codevuln_ruleset_measured_number.md` | 276 | Resolve F-2a/F-2b boundaries, rule catalog, benchmark definitions, command/path drift, artifacts, and measured-claim protocol. |
| F-2c original | `docs/thefinalglance/F-plan/F-2c_low_fp_cwe_rules_seccodeplt_baseline.md` | 253 | Resolve four exact predicates, false-positive guards, shipped-state assertions, corpus/report schema, and static-gate caveat. |
| F-3 original | `docs/thefinalglance/F-plan/F-3_camel_toolfilter_dataplane.md` | 232 | Resolve trust boundary, minimal-set derivation, filter state/lifecycle, ASR fixture, Phase-2 IPC design, and overhead gate. |
| F-4 original | `docs/thefinalglance/F-plan/F-4_test_integrity_cant_lie.md` | 217 | Resolve process-slot design, row schema/correlation, exact hash bytes, transition behavior, schema migration, and deferred semantic output scope. |
| F-5 original | `docs/thefinalglance/F-plan/F-5_staleness_quarantine_cascade.md` | 235 | Resolve `concept_stale` polarity, dependency traversal, broadcast schema/capacity/delivery, cache/Cosmos behavior, and B-30 prerequisite. |
| F-6 original | `docs/thefinalglance/F-plan/F-6_learned_skill_library.md` | 498 | Resolve acquisition workflow versus revised scope, S1-S4 semantics/thresholds, human gates, critic reuse, embedding retrieval, and demotion. |
| F-7 original | `docs/thefinalglance/F-plan/F-7_safety_lattice_cbr_ranking.md` | 263 | Resolve formal proof properties/artifacts and the separately scoped CBR formula, labels, baseline, and P@k measurement. |
| F-8 original | `docs/thefinalglance/F-plan/F-8_scf_symbolic_conflict_selfmod.md` | 242 | Resolve conflict types, IntentNode extraction, warn/escalate policy, archive ordering, fixture, and future BSP/Graphiti decision. |
| F-9 original | `docs/thefinalglance/F-plan/F-9_hallucinated_dep_writetime_gate.md` | 238 | Resolve provenance, registries, cache/network/W-10 behavior, stale policy, syntax/path matching, failures, and test manifests. |
| F-10 original | `docs/thefinalglance/F-plan/F-10_security_monotonicity_iter_guard.md` | 220 | Resolve classifier taxonomy/languages, async pre-commit mechanics, block/escalate policy, calibration corpus, and missing self-mod wire. |
| F-11 original | `docs/thefinalglance/F-plan/F-11_close_one_config_generation.md` | 258 | Resolve exact loop transaction/order, trap-rejection plus loop-closure semantics, real benchmark, guards, SSB schema, flags, and W-2 exception. |
| F-12 original | `docs/thefinalglance/F-plan/F-12_semantic_contradiction_local_nli.md` | 267 | Resolve candidate generation, NLI model/threshold/fallback, determinism, timestamp semantics, event/UI, ops gating, and fixture/live split. |
| F-13 original | `docs/thefinalglance/F-plan/F-13_session_archive_schema_bump.md` | 513 | Resolve SQLite/WAL-safe copy, failure policy, archive/event durability, permissions/retention/restore, deferred gaps, and flag disposition. |
| Canonical F rationale | `workflow/reasoning/VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md` | 525 | Explicitly cited as canonical “why” plus per-item detail; recover primary citations, rejected alternatives, claims, and scope revisions for F-1..F-10. |
| Post-F recommendation basis | `workflow/research/F-plan-next/00-RECOMMENDATIONS.md` | 152 | Explicit basis for F-2c/F-11/F-12/F-13; recover research evidence, gaps #1/#3/#4, confidence, and recommendation boundaries. |
| Overall orientation | `docs/thefinalglance/ELAI-SPEC-OVERVIEW.md` | 592 | Design topology only; must not be treated as a shipped-state ledger. Use to map F into core phases and adjacent G-plan consumers. |

### B. Referenced implementation symbols and paths to verify

Paths are reproduced exactly as the summary spells them; several are basename-only or approximate and therefore require repository resolution with `rg`.

| F scope | Referenced code/data/tool | Summary assertion or question to verify | Source |
|---|---|---|---|
| Global | `wiring_contract_check.py --run-trigger` | Production reachability, real consumer, stub ban, and live effect verifier. Locate canonical script and output contract. | L8-L13, L278-L280 |
| F-1 | `rule_engine.rs:187`, `ALWAYS_INCLUDED_DOMAINS` | `security` is always included. | L24-L25 |
| F-1 | `schema.rs:46` | `mutable` exists and defaults to `1`; locate actual schema and migrations. | L25-L26 |
| F-1 | `lifecycle.rs:871` | Tested `mutable=0` path. | L25-L26 |
| F-1 | `crates/plugins/src/hooks.rs`, `run_pre_tool_use` | Existing hook runner and compound insertion point. | L26-L34 |
| F-1 | orchestrator `hook_dispatcher.rs` | Production dispatch/reachability side of the PreToolUse path. | L26-L27 |
| F-1 | `RuleEngine`, `schema.rs` seed path | Compound evaluator consumption and immutable-row seeding. | L33-L34 |
| F-2 | `crates/rules/rules/security/*.toml` | Proposed rule location; reconcile with F-2c seed path. | L52-L52 |
| F-2 | `crates/cli/src/benchmark.rs`, `--suite seccodeplt` | E-9 harness extension and exact CLI spelling. | L49-L55 |
| F-4 | `TestOutcomePayload` | Existing fields, proposed optional hash, serialization/schema compatibility. | L78-L88 |
| F-4 | `feasibility_probe.rs:168` | BLAKE3 stdout-hash pattern to reuse. | L83-L84 |
| F-5 | `crates/memory/src/decay.rs` | Producer of `StalenessQuarantineBroadcast`. | L106-L107 |
| F-5 | `crates/knowledge/src/dkg` | `concept_stale` consumers and demotion path. | L99-L109 |
| F-5 | `grounding_cache` | Invalidation/miss behavior and missing explicit file integration. | L99-L109 |
| F-5 | `cosmos_worker.rs` | Cosmos artifact consumer. | L99-L107 |
| F-6 | `compiled_pipeline.rs`, `TemplateCrystallizer` | Existing acquisition/critic implementation and SSB verdict reuse. | L112-L124 |
| F-6 | new `skill_trust.rs` | S1-S4 enum, state machine, event tags, and autonomous gate. | L118-L125 |
| F-6 | BestOfN `MiniCritic` triggering SSB row | Correlation and verdict reuse contract. | L119-L120 |
| F-7 | E-53 Z3/Kani/Verus stubs | Prerequisite status and safety-lattice amendment point. | L127-L140 |
| F-7 | pipeline-template store and existing SSB outcomes | CBR identity, outcome schema, baseline, and ranking integration. | L131-L141 |
| F-8 | Phase 3G `coherence_check` | Existing `(name,type_sig)` comparison and why it is insufficient. | L143-L146 |
| F-8 | `self_mod_safety.rs`, `promotion_gate.rs` | Pre-archive symbolic review and namespace isolation from C-50. | L151-L158 |
| F-9 | `registry_check.py`, `_check_cargo()` | Existing real HTTP checks, ecosystems, new Cargo path, caching, error policy. | L160-L174 |
| F-10 | `chat_reflect.rs`, `live_cli.rs:~1685` | Async gate before REFINE commit. | L183-L191 |
| F-10 | `task_runner.rs:~3996` | Gate before auto-advance. | L188-L191 |
| F-2c | `data/rules/web.toml` | Working composite-rule template. | L199-L202 |
| F-2c | `bench_seccodeplt.rs`, `SecCodePltRunner` | Asserted shipped benchmark route and run status. | L203-L205 |
| F-2c | `InsecureRateMeasured` | Existing SSB variant versus lowercase emitted signal spelling. | L203-L213 |
| F-2c | `TreeSitterAstEvaluator` | Exactly 10 registered languages including Python/TypeScript. | L203-L206 |
| F-2c | `detect_language_from_ext` | `None` for `.svelte`, `.html`, `.css`. | L205-L206 |
| F-2c | `ast_adapter::new`, `ast-grep-core` | Required future adapter edit; deferred. | L207-L208 |
| F-2c | `crates/rules/seed/security/` existing loader | Four TOMLs and 12-to-16 immutable-row count. | L207-L210 |
| F-2c | `SecurityRulesPanel` | Existing frontend consumer; no new pair. | L209-L213 |
| F-11 | `config_archive.rs:198/274/395` | Callerless functions and archive API semantics. | L217-L225 |
| F-11 | `darwinian_bai` | Existing bandit’s actual role in the one-generation driver. | L218-L220 |
| F-11 | `two_gate_promote`, `trap_tasks`, BLAKE3 tripwire, Layer-3 human gate | Guard trio/four named mechanisms, ordering, inputs, outputs. | L219-L230 |
| F-11 | `darwin_loop_cmd.rs:104`, `:127` | Synthetic reward and `RUNPOD-PENDING` termination; production entrypoint. | L223-L234 |
| F-11 | `self_mod_layer1a.rs:383`, `is_security_sensitive` | Separate-path bypass and mandatory explicit driver call. | L225-L230 |
| F-11 | `promotion_gate` `tau/K_m` | Values self-labelled unconfirmed. | L225-L226 |
| F-11 | feature `darwinian-evolve`, env `ELAI_DARWINIAN_LOOP_ENABLE` | Wire-dark activation and W-2 exception. | L231-L235 |
| F-11 | ReasoningDrawer `Darwinian Admits` | Frontend pair and event-disposition rendering. | L231-L236 |
| F-12 | `contradiction.rs:64`, `audit_conflicts` | Byte-identical-key behavior, Surface result, zero production callers. | L243-L251 |
| F-12 | `store.rs:208/303`, `supersede`, `soft_close` | Non-destructive storage and exact timestamp mutation. | L245-L258 |
| F-12 | `MemoryConflictDetected` | Free-form `conflict_kind`, event serialization, no schema bump. | L246-L258 |
| F-12 | `reranker.rs` | Deterministic ort model-load pattern. | L247-L248 |
| F-12 | new `crates/memory/semantic_contradiction.rs` | Candidate/filter/control logic. | L253-L255 |
| F-12 | new `crates/memory/nli_onnx.rs` | Local model implementation and `NliJudge` seam. | L253-L258 |
| F-12 | `live_cli.rs` memory tick | First production caller beside F-5. | L253-L258 |
| F-12 | MemoryFactsDrawer `Semantic Contradictions` | Frontend pair and SurfaceToUser flow. | L253-L258 |
| F-13 | `ssb.rs:14342`, `run_migration` | One destructive drop site, old tables, ordering, transaction. | L260-L276 |
| F-13 | new `crates/orchestrator/session_archive.rs` | File-copy/verification implementation. | L269-L276 |
| F-13 | `workflow/sessions/{id}/archive/ssb-v{N}.sqlite` | Archive path construction/collision/permissions. | L274-L276 |

### C. Referenced adjacent plan IDs and contracts

| Reference | Relation asserted by F summary | F source |
|---|---|---|
| W-1..W-10 / F-0 | F-wide enforcement doctrine; specifically W-1 done gate, W-2 env-gate rule, W-4 frontend pair, W-5 live trigger, W-10 hot-path tiering. | L2-L13, L28-L32, L278-L284 |
| B-22 | Existing auto-advance is correctness/score-only relative to F-10. | L176-L178 |
| B-30 | Must precede F-5 consumer audit/population work. | L101-L104 |
| B-39 | Existing REFINE is correctness/score-only relative to F-10. | L176-L178 |
| C-11 | Existing OS/Tier-C isolation is control-plane coverage relative to F-3. | L59-L63 |
| C-22 | Existing feasibility-path `stdout_hash` pattern reused by F-4. | L75-L88 |
| C-25 | Existing `TemplateCrystallizer` ranks/crystallizes by success count, partially covering F-6. | L111-L120 |
| C-26 | F-2b benchmark prerequisite/decode substrate. | L41-L51 |
| C-27 | F-4 prerequisite for DeterminismDrift/`ELAI_SEED`. | L75-L88 |
| C-28 | F-4 mutation-testing prerequisite. | L75-L88 |
| C-50 | Codegen-scoped `ContractSpec`; namespace collision blocks F-8 BSP expansion. | L143-L155 |
| D-3 | Existing self-mod scoring/consensus/drift gate, not semantic conflict detection. | L143-L146 |
| D-4 | Existing self-mod tripwires/score-domain checking, not F-8 semantics or F-10 security monotonicity. | L143-L146, L176-L178 |
| D-11 | Real HTTP registry existence/publish-date checker; prerequisite and extension base for F-9. | L160-L174 |
| E-5 | Existing bi-temporal supersession only for `memory_facts`; F-5 expands cross-subsystem scope. | L93-L100 |
| E-7 | Existing self-mod Two-Gate score/consensus/drift, not semantic/security-class regression coverage. | L143-L146, L176-L178 |
| E-9 | Benchmark harness used by F-2b; functional-only baseline relative to F-2. | L41-L55 |
| E-11 | Idea-framing dependency check partially covers F-9. | L160-L167 |
| E-12 | Post-write Semgrep differs from F-2 pre-write gating. | L41-L43 |
| E-24 | MCP pinning differs from F-3 data-plane tool filtering. | L59-L63 |
| E-30 | Argv-only protection differs from F-1’s unified write gate. | L17-L19 |
| E-35 | Decode substrate and F-2b prerequisite. | L41-L51 |
| E-37 | Prompt-injection/input-marking partially overlaps F-1/F-3 but does not provide their compound/data-plane gate. | L17-L19, L59-L63 |
| E-41 | PR-leg boundary extended by F-9. | L160-L171 |
| E-43 | Existing skill soft gate/telemetry partially overlaps F-1/F-6. | L17-L19, L111-L125 |
| E-49 | Functional-only benchmark compared with F-2 insecure-rate measurement. | L41-L43 |
| E-53 | Proves FSM/IPC invariants but not immutable rule lattice; F-7 amends it after proof stubs resolve. | L127-L140 |
| Phase 3G | Existing `(name,type_sig)` coherence check is narrower than F-8 semantic conflict detection. | L143-L146 |
| Phase TOML | F-2b destination for `insecure_rate_before/after`; exact schema/path unresolved. | L53-L55 |
| Issue `#37210` | Basis for F-1’s four per-tool contracts; repository/source not identified in summary. | L28-L30 |

### D. External evidence and named source claims requiring citation recovery

The summary provides names and verdicts but not URLs/bibliographic details. Later deep readers should recover exact primary citations from the canonical rationale or originals and keep these numbers separate from ELAI measurements.

| Name/reference | Summary use and caution | Source |
|---|---|---|
| VibeSec / SHARD | Proposal families behind all F sessions; SHARD supplies time-decay, cross-subsystem cascade, and trust ladder patterns. | L2-L10, L93-L121 |
| CodeGate | Claimed dead competitor that attempted a unified gate and suffered false-positive friction. | L20-L23, L37-L39 |
| SVEN | `59.1 -> 92.3%` prefix-steering result on CodeGen-2.7B; verified but non-transferable to ELAI/Qwen. | L44-L47 |
| CodeGuard+ | `43.2 -> 76.0%` constrained-decode result on CodeGen-2.7B; verified but non-transferable. | L44-L51 |
| CodeGen-2.7B | Model on which transferred security numbers were measured. | L46-L47 |
| Qwen2.5-Coder-7B-abliterated | ELAI target requiring a fresh measurement. | L46-L51 |
| SeCodePLT | Security benchmark/corpus for F-2/F-2c; exact version and methodology absent here. | L49-L55, L197-L213 |
| CaMeL | Capability/Dual-LLM and `SymbolicVar`; `77% provable-secure` is verified, overhead grounding is overstated/secondary until PDF inspection. | L59-L70 |
| AgentDojo Tool Filter | External `57.7 -> 6.84%` ASR at `73% utility`; inspires the Phase-1 fixture. | L62-L73 |
| Replit incidents | F-4 motivation: faked tests, fabricated 4,000 users, false rollback; F-2 R2 anti-Replit demo. | L55-L56, L78-L82 |
| Mem0 A.U.D.N. | Input pattern for F-5 staleness handling; expansion not defined in summary. | L96-L100 |
| Graphiti | Bi-temporal invalidation pattern; potential cloud-lane contradiction alternative; F-12 mirrors Graphiti/Zep semantics. | L96-L100, L151-L155, L238-L252 |
| Voyager | Critic-gated skill acquisition and embedding retrieval; `3.3x` is Minecraft-only/non-transferable. | L111-L121 |
| Springdrift | Synthetic/unreproduced CBR numbers are hypotheses, not targets; git-backed recovery motivates but is not adopted by F-13. | L127-L140, L260-L268 |
| SCF | Simulation-only, zero-star/empty-tests source; three symbolic conflict types; `27.9%` precision limitation. | L143-L155 |
| Spracklen | Verified dependency-hallucination figures: `19.7%`, `205,474` fake names, `43%` recurrence. | L160-L166 |
| Socket / Socket Firewall | Cannot close the fresh-slop/no-CVE gap; install-boundary wrapper moves to ops. | L163-L171 |
| DGM / Darwin-Godel | Self-mod archive/search and reward-hack fixture family behind F-8/F-11. | L143-L158, L215-L236 |
| `arXiv:2506.09501` | Cited for why a cloud LLM NLI judge breaks `ELAI_SEED`; recover title, claim, and applicability. | L240-L243 |
| `2510.04399` | Cited for bounded-capacity config/template mutation; recover exact paper and constraint. | L227-L228 |
| Zep 2.2.3 | Cited in a storage docstring for non-destructive supersede/soft-close semantics. | L244-L247 |

### E. Research-run provenance

| Run | Claimed role | Source |
|---|---|---|
| `wf_b959963a-db6` | Adversarial verdict run whose novelty/grounding/feasibility judgments survived into F-1..F-10. | L2-L5 |
| `wm3rajyyo` | Adversarial research run grounding the four post-F-10 extensions. | L193-L195 |

## Consolidated ambiguity and deep-read checklist

1. **Normative hierarchy:** establish precedence among F-0, `F-WIRING-DOCTRINE-WHY`, each original, the canonical VibeSec/SHARD rationale, the post-F recommendations, and this summary—especially when “ships now,” “not wired,” “wire-dark,” and “implementation pending” coexist.
2. **Actual implementation ledger:** verify every cited shipped/current-state assertion in code. The overview explicitly cannot answer this, and this discovery did not inspect code.
3. **W-2 exceptions:** document approval/exit criteria for F-11’s dual enablement gates, F-12’s ops-gated model, and F-13’s archive flag; determine whether F-1 Tier-2’s design gate is temporary or terminal.
4. **W-4 completeness:** F-3, F-7, F-8, and F-9 name observable signals without an entry-local dashboard pair. Determine whether they are intentionally CLI-only/non-surfaceable or missing required pairs.
5. **W-10 consistency:** reconcile F-9’s registry call on cache miss with the rule that network work stays off the per-write hot path and F-1’s stated plan-start/per-session cache.
6. **F-1 scope truth:** determine whether the release is honestly two-axis Tier-1 or a four-axis unified product; identify the concrete malicious-skill axis and whether F-9 implements dependency Tier-2.
7. **Rule path/command drift:** reconcile `crates/rules/rules/security/` versus `crates/rules/seed/security/`, and `elai benchmark --suite seccodeplt` versus `elai bench seccodeplt`.
8. **Benchmark semantics:** pin models, corpus versions, seeds, sample units, before/after treatment, pass@1, insecure-rate denominator, false-positive labels, JSON/TOML schemas, and publication criteria.
9. **Tool-filter trust boundary:** define how a minimal set is calculated before untrusted bytes, how nested/delegated tasks behave, and what happens when a legitimate missing tool is required later.
10. **Test evidence binding:** define exact hash input, process provenance, exit-code binding, row correlation, serialization migration, and behavior for missing/mismatched hashes versus valid failures.
11. **Deferred output grounding:** locate or create the later F-N only if explicitly authorized by the originals; do not claim F-4 verifies semantic expected outputs yet.
12. **`concept_stale` semantics:** resolve the 0/1 polarity/default hazard before any write-side population and verify every retrieval/cache/Cosmos consumer.
13. **Skill trust:** resolve tier semantics, thresholds, promotion-versus-execution human gating, reuse/acquisition scope, staleness clock, and memory/skill type isolation.
14. **Safety proof:** define formal lattice properties, higher-order conflicts, proof artifacts, frozen checker inputs, E-53 amendment, and RunPod requirement.
15. **SCF detector:** enumerate three conflict types, translate configs/invariants into IntentNodes, define warn/escalate/block, and recover measured precision/fixtures.
16. **Dependency provenance and failures:** define provenance (missing from body), registries, age policy, offline/network error behavior, cache poisoning/negative entries, namespaces, and install-runbook ownership.
17. **Security-monotonicity classifier:** define languages/classes/delta boundaries, async race control, first-iteration baseline, block/escalate rule, self-mod wiring, and rollback/commit guarantees.
18. **DGM loop transaction:** resolve how the trap mutant yields both a rejection receipt and proof that insertion/parent selection ran, plus guard order, real benchmark, archive consistency, and event correlation.
19. **NLI production contract:** pin model and license/hash, candidates, confidence threshold, SurfaceToUser flow, practical-determinism tolerance, missing-model fallback, tick budget, privacy, and concurrent close semantics.
20. **SSB archive durability:** define a WAL-consistent atomic copy, failure policy before destructive drop, permissions/retention, archive collision, checksum storage, event survival, restoration, and deferred gaps #1/#3/#4.
21. **Event naming/schema:** reconcile lowercase event labels with Rust-style variant names (`insecure_rate_measured`/`InsecureRateMeasured`, `memory_conflict_detected`/`MemoryConflictDetected`) and verify payloads, correlation IDs, redaction, retention, and dashboard consumers.
22. **Cross-feature order:** make the combined ordering of F-1/F-2/F-9/F-10 and F-7/F-8/F-10/F-11 explicit; the workflow composition above is intentionally inferred where the summary is silent.

## Coverage accounting

Audit result for this pass: **441 unique atomic records** with no duplicate IDs: FG 18; F-1 41; F-2 31; F-3 24; F-4 25; F-5 25; F-6 34; F-7 24; F-8 19; F-9 25; F-10 24; F-2c 35; F-11 45; F-12 39; F-13 32. All **14/14** entry headings are present. The state/event registry covers **17** explicitly named SSB/event/broadcast signals or variant spellings. The deeper-source manifest identifies **20** F/orientation/research documents (the 286-line summary, 16 other F-folder documents, two workflow research sources, and the overview). Coverage remains complete only if all headings, signals, commands/flags, every revised phase/tier, and every explicit deferred item remain represented after later edits.

## Audit commands for the compiling agent

Run from any directory:

```bash
# Atomic inventory records, including the global F contract.
rg -c '^\| (FG|F1|F2|F3|F4|F5|F6|F7|F8|F9|F10|F2C|F11|F12|F13)-[0-9]{2} \|' \
  docs/thefinalglance/feature-discovery/06-F-SUMMARY_INDEX.discovery.md

# Confirm all summary headings have a discovery section.
rg '^### F-' \
  docs/thefinalglance/feature-discovery/06-F-SUMMARY_INDEX.discovery.md

# Inspect every summary line that contains an exact event/signal spelling.
rg -n '(blocked|measured|committed|promoted|demoted|verified|ranked|detected|archived|Broadcast)' \
  docs/thefinalglance/F-plan/F-SUMMARY_INDEX.md
```

End of F summary discovery.

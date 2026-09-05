# DEEP-F-002 exhaustive feature discovery

## Scope, method, and evidence contract

- **Exact assigned chunk:** `DEEP-F-002`, Family F, eight files, 2,097 logical lines, 292,736 bytes. The manifest classifies seven files as `numbered/index original` and one as `auxiliary/grounding/fixture`.
- **Permitted orientation:** `docs/thefinalglance/feature-discovery/06-F-SUMMARY_INDEX.discovery.md` was used only to identify summary additions and corrections. It is not promoted to source evidence.
- **Isolation:** every assigned source was read completely. No other F original, referenced implementation file, research paper, rationale, plan chunk, or repository source was read. Consequently, phrases such as “confirmed,” “shipped,” “real,” and “grep-confirmed” below remain explicit assertions of the assigned documents, not independent code verification by this pass.
- **Atomicity:** a row records one distinct capability, behavior, state rule, schema element, configuration rule, integration contract, failure policy, evidence requirement, test gate, dependency, constraint, or explicit non-goal. Repeated statements are collapsed into one semantic row with all material source anchors; no repeated acceptance wording is counted twice.
- **Evidence/status codes:** `E-S` = explicit existing/shipped-state assertion; `E-P` = explicit proposed/normative behavior; `E-D` = explicit deferred/non-goal; `E-G` = explicit acceptance/done gate; `E-R` = explicit risk, failure, fallback, or recovery rule. `I`, `A`, and `C` are confined to the separately labelled inference, ambiguity, and conflict ledgers.
- **Anchor notation:** `F1:L47-L53` means lines 47–53 of `F-1_vibegate_unified_write_gate.md`; analogous keys are `F2`, `F2C`, `F3`, `F5`, `F6`, `F7`, and `WD` (wiring doctrine).

## Exact processed manifest and integrity verification

| Chunk | Class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---|---:|---:|---|---|---|
| DEEP-F-002 | numbered/index original | 220 | 36,413 | `c5663fe857fcfdeabe30d23bd1015a6db790a2e6a9ee4b03fadda4590ed249ce` | `docs/thefinalglance/F-plan/F-1_vibegate_unified_write_gate.md` | exact |
| DEEP-F-002 | numbered/index original | 276 | 42,687 | `ff1e10d7270dbd77a1486344857b49c1387cf1364e685e6c184e1b945398bac3` | `docs/thefinalglance/F-plan/F-2_codevuln_ruleset_measured_number.md` | exact |
| DEEP-F-002 | numbered/index original | 253 | 48,443 | `e6e072e9ab7be5534964428aafa2d9df0e1e90cd5b6d150c88713007ea171ab0` | `docs/thefinalglance/F-plan/F-2c_low_fp_cwe_rules_seccodeplt_baseline.md` | exact |
| DEEP-F-002 | numbered/index original | 232 | 37,550 | `50cdf05501a571650f3f08785256a7e2f3f4288644ce8ba4fcf736773aa69b88` | `docs/thefinalglance/F-plan/F-3_camel_toolfilter_dataplane.md` | exact |
| DEEP-F-002 | numbered/index original | 235 | 34,502 | `a0c9721572136ec6a6112d5a54644c8afb9881d1639048f7a1239b293c3c1a73` | `docs/thefinalglance/F-plan/F-5_staleness_quarantine_cascade.md` | exact |
| DEEP-F-002 | numbered/index original | 498 | 41,255 | `4ddb67d8bf9b34535f523c6769bc6fe64e67d836e51fee1b4d17b3ab1ddf70a5` | `docs/thefinalglance/F-plan/F-6_learned_skill_library.md` | exact |
| DEEP-F-002 | numbered/index original | 263 | 42,837 | `e4ad7c84da7317d6b28d26057d5fa3c11f7d89f79aa931d2a3ac9214baf93443` | `docs/thefinalglance/F-plan/F-7_safety_lattice_cbr_ranking.md` | exact |
| DEEP-F-002 | auxiliary/grounding/fixture | 120 | 9,049 | `15742a1e69d2e45d76140543f58e44241852a766530cfb3bd3ad391409eec7bb` | `docs/thefinalglance/F-plan/F-WIRING-DOCTRINE-WHY.md` | exact |

Integrity commands used were `wc -l -c` and `sha256sum` on exactly the eight paths above. Per-file and aggregate line/byte values match the manifest with zero variance.

## Per-source exhaustive atomic ledgers

### F-1 — VibeGate unified write gate

**Source status boundary:** the header says `STATUS: PLANNED` and limits this session to Tier-1, but a later reconciled wiring block asserts named symbols “actually shipped”; the doctrine also calls F-1 a shipped unit. Those incompatible status claims are retained in the conflict ledger rather than normalized.

| Atomic ID | Evidence | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| DF002-F1-001 | E-P | F-1’s terminal scope is a deterministic, synchronous, in-process `PreToolUse` compound gate over candidate write content. | F1:L3-L5, L15-L17 |
| DF002-F1-002 | E-P | Tier-1 contains exactly two active axes: code vulnerability and secret exposure. | F1:L5, L45-L47 |
| DF002-F1-003 | E-D | Dependency-existence and MCP/skill scanning are excluded from Tier-1 and may not inherit its fail-closed advertising. | F1:L5-L6, L59-L61 |
| DF002-F1-004 | E-P | The code-vulnerability axis delegates `BLOCK` matches to `RuleEngine` over F-2a-authored `vibesec` rows in the always-included `security` domain. | F1:L7-L8, L23-L24, L47 |
| DF002-F1-005 | E-P | The secret axis scans candidate source/diff with a compiled, reviewable constant regex set in `vibegate_secrets.rs`. | F1:L47-L48 |
| DF002-F1-006 | E-P | Secret predicates include AWS access keys matching `AKIA[0-9A-Z]{16}`. | F1:L47 |
| DF002-F1-007 | E-P | Secret predicates include RSA/EC/OpenSSH private-key PEM headers. | F1:L47 |
| DF002-F1-008 | E-P | Secret predicates include assignments of at least eight quoted characters to `password`, `secret`, `token`, or `api[_-]?key`-like names. | F1:L47 |
| DF002-F1-009 | E-P | Any active-axis `Block` makes the compound result blocking; no prompt-controlled branch may bypass it. | F1:L47, L51, L101-L103 |
| DF002-F1-010 | E-R | Regex/evaluator initialization failure must panic at startup rather than silently return clean at gate time. | F1:L47, L102-L103 |
| DF002-F1-011 | E-P | The per-write hot path must propagate `Result`/`Option` and contain no `.unwrap()`/`.expect()`; only provably valid literal regex compilation may use init-time `expect`. | F1:L47, L103 |
| DF002-F1-012 | E-P | F-2a security seed rows are inserted with `domain="security"`, `mutable=0`, and `source="SEED"`, making them always evaluated and auto-CCR-unretirable. | F1:L24, L49 |
| DF002-F1-013 | E-P | Startup canonicalizes all immutable security rows as `(rule_id,predicate,severity)` and compares their BLAKE3 digest with compiled `VIBESEC_SEED_BLAKE3`. | F1:L49 |
| DF002-F1-014 | E-R | Seed-digest mismatch emits a code-vulnerability VibeGate block for `seed integrity violation` and refuses dispatcher startup. | F1:L49, L153 |
| DF002-F1-015 | E-P | `evaluate_vibegate(source, file_path)` returns axis verdicts and is consumed inside the existing dispatcher analysis path. | F1:L47, L51, L71-L75, L94-L96 |
| DF002-F1-016 | E-P | Every blocking axis becomes a T1 `HookFinding` with category `vibegate`, optional `rule_id`, message, and no fix template. | F1:L51 |
| DF002-F1-017 | E-S | The asserted live baby-step path builds a pre-tool invocation, dispatches it, journals hook events, and short-circuits on `HookFinding::is_block` with `BlockSource::T1Hook`. | F1:L21, L94-L96 |
| DF002-F1-018 | E-S | The reconciled sole production bootstrap is `runtime_setup.rs::build_production_hook_dispatcher`, installed with `.with_vibegate_ssb` and `.with_vibegate_seed_ok`. | F1:L90-L96 |
| DF002-F1-019 | E-S | `task_runner.rs::build_task_runner_wiring` seeds security rules, checks integrity at rules-DB initialization, and threads `vibegate_seed_ok` into dispatcher construction. | F1:L94-L96 |
| DF002-F1-020 | E-S | The earlier `conversation.rs:483` actual-tool route is explicitly reconciled as a different path that never reaches `dispatch_pre_tool_use`; actual-tool byte gating is deferred to a Tier-1.5 follow-up. | F1:L90-L95 |
| DF002-F1-021 | E-P | The only configuration switch is `ELAI_VIBEGATE_DISABLED`; unset means enabled and `=1` disables the gate. | F1:L57, L100, L147 |
| DF002-F1-022 | E-P | A default-off `ELAI_VIBEGATE_ENABLE` flag is forbidden. | F1:L57, L100, L130-L132 |
| DF002-F1-023 | E-P | SSB uses one `SsbActionType::VibeGateBlocked` plus `VibeGateBlockedKind::{CodeVuln,Secret,DepHallucinated,Skill}`. | F1:L53, L76-L78 |
| DF002-F1-024 | E-P | Live Tier-1 SQL spellings are `vibesec_code_vuln_blocked` and `vibesec_secret_blocked`. | F1:L53, L109-L110 |
| DF002-F1-025 | E-D | `vibesec_dep_hallucinated_blocked` and `vibesec_skill_blocked` have parse/schema arms but no Tier-1 producer. | F1:L26-L27, L53, L61 |
| DF002-F1-026 | E-P | `VibeGateBlockedPayload` carries `kind`, optional `rule_id`, optional `file_path`, `source_blake3`, `citation`, and `fsm_state`. | F1:L53 |
| DF002-F1-027 | E-P | `source_blake3` hashes the truncated candidate source with BLAKE3 for replay evidence. | F1:L53 |
| DF002-F1-028 | E-P | The schema impact is one Pattern-2 enum variant and four accepted action strings, two live and two reserved; the final schema number must be counted at implementation time. | F1:L53, L168-L169 |
| DF002-F1-029 | E-P | Per-tool contracts must cover `Edit`, `Write`, `NotebookEdit`, and `Bash` heredoc/redirect writes for both SQLi and AWS-key inputs. | F1:L55, L152, L173-L176 |
| DF002-F1-030 | E-G | A block must be honored before disk mutation; the baby step renders `passed=false`, records `block_source=T1Hook`, and does not proceed. | F1:L55, L109-L111, L146-L153 |
| DF002-F1-031 | E-P | The frontend pair is `VibeGateDrawer.svelte` plus `commands_vibegate.rs::list_vibegate_blocks`, registered under `DrawerKey 'vibegate'`. | F1:L104-L107 |
| DF002-F1-032 | E-P | The drawer lists block kind, rule ID, file path, and citation. | F1:L104-L106, L151 |
| DF002-F1-033 | E-G | The live trigger is an `elai orchestrate` one-baby-step task containing both a SQLi string and `AKIAIOSFODNN7EXAMPLE`, with the kill switch unset. | F1:L108-L110 |
| DF002-F1-034 | E-G | That trigger must create both live block rows in the same superstep and populate a BLAKE3 source digest. | F1:L109-L111 |
| DF002-F1-035 | E-G | Evidence also requires a failed baby-step artifact, both drawer entries, and a `hook-executions.log` match for a T1 VibeGate block. | F1:L110-L111, L146-L151 |
| DF002-F1-036 | E-G | Completion requires `wiring_contract_check.py --run-trigger` exit 0; cargo-green alone cannot establish done. | F1:L115-L117, L144-L146, L158-L160 |
| DF002-F1-037 | E-G | Completion additionally requires `cross_lang_diff.py`, SSB payload round-trip/golden lock, `cargo check --workspace`, and auditor pass over orchestrator/rules/contracts. | F1:L137-L141, L149-L155 |
| DF002-F1-038 | E-G | Tampering one immutable security predicate must cause the integrity contract to fail and startup refusal. | F1:L153, L175 |
| DF002-F1-039 | E-G | Unit coverage includes clean input, SQLi, AWS key, kill-switch clean behavior, and ten benign false-positive guards. | F1:L176 |
| DF002-F1-040 | E-R | False-positive rate has no current baseline; it must be bounded on at least three real orchestrate runs before Tier-2 is enabled outside development. | F1:L61, L209 |
| DF002-F1-041 | E-D | Tier-2 registry results must be cached at plan start and MCP scan must run per session, not per write. | F1:L61 |
| DF002-F1-042 | E-D | Tier-2 guarantees must be documented as fail-open, and network/subprocess work must remain off the write hot path. | F1:L61, L169, L212 |
| DF002-F1-043 | E-D | F-1 performs no model run, sidecar use, RunPod work, or performance claim. | F1:L150, L164, L204, L213 |
| DF002-F1-044 | E-P | No new dependency may be added without confirmation; BLAKE3 and regex are expected already in-tree and must be verified first. | F1:L156, L205, L214 |
| DF002-F1-045 | E-R | The primary operational risks are false-positive disablement, user-writable SQLite defeating `mutable=0`, and any tool surface silently ignoring a deny. | F1:L207-L214 |
| DF002-F1-046 | E-D | F-1 reserves dependency and skill Kind arms for F-9 and a future E-43 extension; F-2b later consumes the gate as its before/after enforcement point. | F1:L216-L218 |

### F-2 — vulnerability rules and measured insecure-rate program

**Source status boundary:** the reconciled WIRING_CONTRACT explicitly treats F-2a as shipped and F-2b as RunPod-gated/deferred, while unreconciled checks and acceptance still demand both. F-2c later asserts both implementations ship. These are retained as conflicts.

| Atomic ID | Evidence | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| DF002-F2-001 | E-P | F-2a populates the always-on `security` domain with pre-write vulnerability predicates and co-ships as F-1’s code-vulnerability axis. | F2:L3, L7-L10, L57-L68 |
| DF002-F2-002 | E-P | F-2b produces ELAI’s first before/after insecure-code-rate number; its live Qwen run is RunPod/remote-sidecar gated. | F2:L3-L5, L11, L76-L86 |
| DF002-F2-003 | E-S | Existing predicate infrastructure includes `AstPattern`, `GrepPattern`, and `FileContext::{Production,Test}` and the loader compiles seed TOMLs for the rule engine. | F2:L21-L23, L35 |
| DF002-F2-004 | E-P | `sqli.toml` covers string-interpolated SQL reaching `execute`, `query`, or `raw` sinks as CWE-89 for Python and TypeScript. | F2:L59-L60 |
| DF002-F2-005 | E-P | `xss.toml` covers `dangerouslySetInnerHTML`, `innerHTML=`, `v-html`, and unescaped DOM template interpolation as CWE-79. | F2:L59-L61 |
| DF002-F2-006 | E-P | `weak_crypto.toml` covers MD5/SHA1 security use, `Math.random()` for secrets, DES, and ECB as CWE-327/328. | F2:L59-L62 |
| DF002-F2-007 | E-P | `missing_authz.toml` detects an endpoint without a nearby authorization guard via a composite predicate as CWE-862. | F2:L59-L64 |
| DF002-F2-008 | E-P | SQLi, XSS, and weak-crypto are high-precision `BLOCK`; missing authorization is conservatively `WARN`. | F2:L63-L64, L264 |
| DF002-F2-009 | E-P | Initial rules are Python/TypeScript production-context only; test code is excluded and Rust coverage is deferred to F-10. | F2:L59-L64, L267 |
| DF002-F2-010 | E-P | Every rule TOML carries `rule_id`, `domain="security"`, severity, a valid tagged `predicate_json`, and a CWE/SeCodePLT citation. | F2:L59-L64 |
| DF002-F2-011 | E-P | The real seed location is `crates/rules/seed/security/`, correcting the earlier `crates/rules/rules/security/` path. | F2:L59, L213, L259 |
| DF002-F2-012 | E-P | Security rows are inserted `mutable=0` and their canonical seed bytes are BLAKE3-checked during loading. | F2:L66, L193 |
| DF002-F2-013 | E-R | A tampered security seed refuses startup and is logged; integrity failure may never silently skip enforcement. | F2:L66, L137, L193 |
| DF002-F2-014 | E-P | Candidate `Edit`/`Write` content is evaluated through `dispatch_pre_tool_use` and `RuleEngine::evaluate`; a `BLOCK` aborts before bytes reach disk. | F2:L68, L192 |
| DF002-F2-015 | E-P | The F-2a gate is pure AST/regex, has a stated T1 latency target below 1 ms, and performs no hot-path I/O. | F2:L68, L216 |
| DF002-F2-016 | E-P | A block journals `security_rule_blocked` with `rule_id`, CWE class, file path, candidate BLAKE3, language, and severity. | F2:L68-L70 |
| DF002-F2-017 | E-P | `SecurityRuleBlocked` is one SSB variant with a payload, SQL parse/check arms, round-trip fixture, and golden-lock row. | F2:L70 |
| DF002-F2-018 | E-P | Enforcement is default-on with no `ELAI_*_ENABLE` requirement and no prompt/env-controlled bypass branch. | F2:L132-L138, L191, L208 |
| DF002-F2-019 | E-P | `SecurityRulesPanel.svelte` and `commands_f2.rs::get_security_blocks_recent` surface recent block rows under `DrawerKey 'security_rules'`. | F2:L72, L139-L141 |
| DF002-F2-020 | E-D | `get_insecure_rate_latest` and its panel tile belong to F-2b and are deferred by the reconciled contract. | F2:L108-L114, L141 |
| DF002-F2-021 | E-P | F-2b’s synthetic corpus contains at least 40 CWE-tagged Python/TypeScript fixtures covering CWE-89/79/327/862. | F2:L78 |
| DF002-F2-022 | E-P | A fixture is a `{prompt,cwe,secure_oracle_pattern}` triple and is ELAI-owned, not a GitGuardian replay corpus. | F2:L78 |
| DF002-F2-023 | E-P | The runner uses the same production generation path and same security-domain gate rather than a mock. | F2:L80 |
| DF002-F2-024 | E-P | Insecure rate is defined as vulnerable generated samples divided by total, before the gate on raw output and after the gate after block/regeneration. | F2:L80 |
| DF002-F2-025 | E-P | `insecure_rate_measured` carries `suite`, `model_id`, `n_samples`, `insecure_rate_before`, `insecure_rate_after`, and `seed`. | F2:L80-L82 |
| DF002-F2-026 | E-P | The benchmark command is `elai bench seccodeplt` through `bench_seccodeplt.rs`, `bench_cmd.rs`, and `SecCodePltRunner`, correcting `benchmark.rs`. | F2:L80, L108-L114, L213 |
| DF002-F2-027 | E-P | The benchmark writes a `seccodeplt-*.{toml,json}` report and journals its measurement. | F2:L143-L145, L198 |
| DF002-F2-028 | E-D | Optional constrained decoding may be added only if the gate’s after-rate is insufficient; it shifts generation and is not fail-closed. | F2:L84, L265 |
| DF002-F2-029 | E-D | Optional decoding reuses `ConstrainedDecodingConfig` and `build_xgrammar_request`; F-2a has no dependency on C-26/E-35. | F2:L9, L30-L31, L84 |
| DF002-F2-030 | E-P | Published live-run evidence records model ID, sample count, and seed and explicitly labels the corpus synthetic. | F2:L86, L199 |
| DF002-F2-031 | E-D | SVEN `59.1%→92.3%` and CodeGuard+ `43.2%→76.0%` are CodeGen-2.7B evidence and cannot be transferred to Qwen. | F2:L25, L254-L257 |
| DF002-F2-032 | E-D | F-2a makes no performance claim; the headline and first-in-category claim are gated on a measured Qwen run. | F2:L25, L74, L194, L265 |
| DF002-F2-033 | E-G | F-2a’s live acceptance input is Python `cursor.execute(f"SELECT ... {uid}")`; it must be blocked, leave the file unwritten, and journal CWE-89 plus a BLAKE3 digest. | F2:L143-L145, L192 |
| DF002-F2-034 | E-G | F-2b code requires a RunPod-independent deterministic fixture run, while the actual Qwen run is a separate gated checkbox. | F2:L198-L199, L269 |
| DF002-F2-035 | E-G | Contract tests cover Python SQLi, TypeScript XSS, a clean candidate, test-context exclusion, immutable-row edit refusal, seed tamper refusal, and both payload round trips. | F2:L220-L224 |
| DF002-F2-036 | E-G | The measured row must be computed, not hard-coded or mocked as equal before/after values. | F2:L138, L223 |
| DF002-F2-037 | E-G | Completion requires the wiring trigger, default-on proof, dashboard E2E, cross-language parity, golden lock/schema bump, workspace compile, and auditor pass. | F2:L188-L201 |
| DF002-F2-038 | E-R | Release requires false-positive measurement over the benchmark and at least three real orchestrate runs. | F2:L264 |
| DF002-F2-039 | E-R | The immutable-row digest’s trusted storage location is unresolved; compiling it into the binary is only a candidate. | F2:L266 |
| DF002-F2-040 | E-R | F-1/F-2 co-ship must resolve whether `vibesec_code_vuln_blocked` and `security_rule_blocked` represent or double-count the same denial. | F2:L13, L268 |
| DF002-F2-041 | E-P | No new external dependency is permitted; BLAKE3 and benchmark dependencies are expected to be reused. | F2:L200, L260 |
| DF002-F2-042 | E-D | The product/panel must not be branded “VibeSec”; internal names are VibeGate/security-rules. | F2:L270 |

### F-2c — four additional CWE rules and static baseline

**Source status boundary:** `STATUS: PLANNED`, with implementation pending for this increment. Its predecessor substrate is asserted shipped: 12 F-2a rules, the F-2b bench, and schema version 201; the bench is asserted never yet run to a published baseline.

| Atomic ID | Evidence | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| DF002-F2C-001 | E-P | F-2c adds four high-precision rules and then runs the existing static gate benchmark; it makes no multi-language expansion. | F2C:L3-L7, L21-L30 |
| DF002-F2C-002 | E-S | F-2a is asserted shipped with 12 rules: three CWE-89 BLOCK, three CWE-79 BLOCK, four CWE-327 BLOCK, and two CWE-862 WARN. | F2C:L9-L11, L34 |
| DF002-F2C-003 | E-S | Those 12 rules are asserted seeded `mutable=0` through `seed_security_rules`, live from `runtime_setup.rs:322`. | F2C:L10, L34 |
| DF002-F2C-004 | E-S | F-2b’s CLI driver, `SecCodePltRunner`, dispatcher, `InsecureRateMeasured`, and schema version 201 are asserted shipped. | F2C:L11, L35 |
| DF002-F2C-005 | E-S | The F-2b harness is asserted not yet run to a published baseline. | F2C:L11, L25, L35 |
| DF002-F2C-006 | E-S | The shipped runner defines `before` as corpus ground-truth insecure rate, `after` as static-gate false-negative residual, and `gate_recall=(before-after)/before`. | F2C:L35 |
| DF002-F2C-007 | E-S | `TreeSitterAstEvaluator::new()` is asserted to register exactly ten languages: Rust, Python, TypeScript, Go, Java, C#, Ruby, HTML, CSS, and Svelte, with Astro feature-gated. | F2C:L27-L28, L229 |
| DF002-F2C-008 | E-S | `.py` maps to Python and `.ts/.tsx` to TypeScript; `.svelte`, `.html`, and `.css` return no language from `detect_language_from_ext`. | F2C:L36, L228-L229 |
| DF002-F2C-009 | E-S | Evaluation order is MetricGate(0), VersionGate/Grep(1), AST/Schema(2), Composite(3), GraphReachability(4). | F2C:L13, L36 |
| DF002-F2C-010 | E-P | `hardcoded_creds.toml` detects a suspicious secret-named identifier assigned a string literal as CWE-798 using grep plus AST confirmation. | F2C:L60-L61 |
| DF002-F2C-011 | E-P | The CWE-798 AST guard excludes comparisons and values sourced from environment/function calls. | F2C:L61 |
| DF002-F2C-012 | E-P | `hardcoded_key.toml` detects literal key material passed to AES, Fernet, `createCipheriv`, or `crypto.subtle.importKey` as CWE-321. | F2C:L62 |
| DF002-F2C-013 | E-P | CWE-321 complements algorithm weakness by checking key provenance rather than duplicating `weak_crypto.toml`. | F2C:L62 |
| DF002-F2C-014 | E-P | `secrets_in_logs.toml` detects secret-named values sent to Python logging/logger/print or TypeScript console sinks as CWE-532. | F2C:L63 |
| DF002-F2C-015 | E-P | CWE-532 explicit logging/logger/console sinks are BLOCK, while Python `print(...)` is WARN due to higher false-positive risk. | F2C:L63-L64 |
| DF002-F2C-016 | E-P | `eval_injection.toml` detects Python dynamic `eval/exec/compile` and TypeScript dynamic `eval/new Function/setTimeout(string)` as CWE-95. | F2C:L64 |
| DF002-F2C-017 | E-P | CWE-95 is BLOCK only for a non-literal argument; literal `eval("1+1")` must not fire. | F2C:L64, L203 |
| DF002-F2C-018 | E-P | All new predicates are Python/TypeScript, production-context `Composite(and)` structures with Grep and AST children. | F2C:L58-L65 |
| DF002-F2C-019 | E-P | Every new seed row uses `domain="security"`, `tier=1`, a CWE-bearing title, and `phase_mask=4294967295`. | F2C:L60-L65 |
| DF002-F2C-020 | E-P | The existing `web.toml` composite form is the serialization template and must be checked against `Predicate` before authoring. | F2C:L12-L13, L38, L65 |
| DF002-F2C-021 | E-P | Existing seed loading should discover the files without code changes; if filenames are hard-coded, only that list may be extended. | F2C:L67-L69, L220 |
| DF002-F2C-022 | E-P | The immutable-row contract is updated from 12 to the exact authored total, specified throughout the plan as 16. | F2C:L67, L103, L158-L159, L177 |
| DF002-F2C-023 | E-P | The baseline runs `elai bench seccodeplt --json` on at least 40 fixtures through real `evaluate_vibegate` under a fixed `ELAI_SEED`. | F2C:L71-L74, L179 |
| DF002-F2C-024 | E-P | Missing fixtures for CWE-798/321/532/95 are added as existing-shape triples without changing harness code. | F2C:L74, L221 |
| DF002-F2C-025 | E-P | One run emits `insecure_rate_measured` with suite/model/sample count/before/after/recall/seed and writes `benchmarks/seccodeplt/seccodeplt-<unix_ms>.json`. | F2C:L73-L74, L129-L132 |
| DF002-F2C-026 | E-P | `F-2c-decisions.md` records the numeric baseline and mandatory synthetic-corpus/static-vs-dynamic caveats. | F2C:L75-L76, L132 |
| DF002-F2C-027 | E-D | The baseline is not SeCodePLT’s dynamic sandbox metric, not a live-model measurement, and not transferable external evidence. | F2C:L25, L73, L179, L241 |
| DF002-F2C-028 | E-D | No percentage claim is made for the four rules in isolation; only the corpus-level baseline may be reported. | F2C:L30, L76, L178 |
| DF002-F2C-029 | E-D | C/C++/JavaScript/Swift and other unregistered-language expansion is deferred because it needs evaluator registration and ABI verification. | F2C:L27-L30, L76, L180, L243 |
| DF002-F2C-030 | E-D | Rust coverage stays deferred to F-10. | F2C:L40, L236, L245 |
| DF002-F2C-031 | E-D | `ast-grep-core` is verified but unused/deferred; no dependency or `ast_adapter.rs::new()` edit is allowed. | F2C:L29, L76, L112-L120, L244 |
| DF002-F2C-032 | E-D | F-2c adds no SSB variant, schema bump, frontend DTO, or new frontend; existing F-2 signals and `SecurityRulesPanel` are reused. | F2C:L7, L121-L127, L181, L198 |
| DF002-F2C-033 | E-G | The live write trigger is production Python `password="hunter2"`; it must block before disk, leave the FSM short of `EXECUTING`, and journal CWE-798 plus BLAKE3. | F2C:L129-L132, L175-L176 |
| DF002-F2C-034 | E-G | Acceptance also distinguishes a CWE-532 `print(password)` WARN, a dynamic-eval BLOCK, clean pass, and test-context non-fire. | F2C:L176, L203 |
| DF002-F2C-035 | E-G | All 16 rows remain immutable/idempotent and seed tampering retains F-2a’s fail-closed BLAKE3 behavior. | F2C:L177, L205 |
| DF002-F2C-036 | E-G | Fixed-seed baseline smoke must yield stable computed before/after/recall, not literal values. | F2C:L117, L206 |
| DF002-F2C-037 | E-G | Completion requires wiring-check execution, default-on behavior, existing frontend/dashboard E2E, cross-language parity, workspace compile, and auditor pass. | F2C:L172-L184 |
| DF002-F2C-038 | E-R | False-positive mitigation combines cheap-grep/AST shape confirmation, production-only context, WARN severity for riskier shapes, corpus measurement, and three real orchestrate runs before WARN→BLOCK promotion. | F2C:L238-L240 |
| DF002-F2C-039 | E-R | Whether `seed_security_rules` enumerates a directory or hard-codes filenames remains unresolved and controls whether a loader edit is needed. | F2C:L69, L220, L242 |
| DF002-F2C-040 | E-R | The `.svelte/.html/.css` language-detection gap is explicit but out of scope. | F2C:L36, L228, L246 |
| DF002-F2C-041 | E-P | No new crate, no `ssb.rs` edit, and no `Cargo.toml` edit are scope-creep tripwires. | F2C:L7, L118-L120, L222 |
| DF002-F2C-042 | E-D | The future live-Qwen metric remains a separately RunPod-gated follow-up. | F2C:L25, L73, L194, L251 |

### F-3 — CaMeL/Tool-Filter data-plane defense

**Source status boundary:** the source is written as a plan whose Tier-1 tasks “ship now” and whose full CaMeL work is unscheduled. The auxiliary doctrine later asserts the Tier-1 unit was built, fixed, and live-triggered; that later claim is recorded separately rather than retrofitted into every proposed row.

| Atomic ID | Evidence | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| DF002-F3-001 | E-P | Tier-1 adds per-task least privilege on top of the existing per-FSM-state category gate for researcher/web/MCP agents. | F3:L1-L5, L9-L15 |
| DF002-F3-002 | E-P | The task’s minimal tool subset is committed and frozen before the first untrusted repo/web/skill/MCP byte is observed. | F3:L3, L11-L13 |
| DF002-F3-003 | E-S | Existing `ToolCategory::ALL` contains 16 categories and `FsmState::check_tool_allowed` provides the category gate. | F3:L19-L20 |
| DF002-F3-004 | E-S | The asserted pre-observation boundary is `WebResearchOrchestrator::formulate_query`; `fetch_and_extract` is the untrusted-observation boundary. | F3:L13, L22 |
| DF002-F3-005 | E-P | `CommittedToolSet` stores `task_id`, a `BTreeSet<ToolCategory>`, `committed_at_ms`, `frozen`, and `commit_hash`. | F3:L51-L53 |
| DF002-F3-006 | E-P | `commit_minimal_set` intersects requested categories with `ToolCategory::ALL`, freezes the result, sorts it, and BLAKE3-hashes the allow-set plus task ID. | F3:L52-L54 |
| DF002-F3-007 | E-P | Researcher commitment includes search, read-only fetch, and role-allowed grounding methods, but excludes filesystem write, Bash, and edit. | F3:L58 |
| DF002-F3-008 | E-P | `check_against_commitment` classifies each tool and denies an out-of-set tool when the commitment is frozen. | F3:L54 |
| DF002-F3-009 | E-R | A frozen set treats an unknown tool as denied, never as default-allowed. | F3:L60, L113 |
| DF002-F3-010 | E-P | Enforcement is pure set membership and cannot consult an LLM/model to override a denial. | F3:L56, L111 |
| DF002-F3-011 | E-P | `UntrustedTaint::{Clean,Tainted{source,span_hash}}` represents classified untrusted spans. | F3:L55 |
| DF002-F3-012 | E-P | `taint_from_verdict` maps E-37 `ClassifierVerdict` into the taint marker. | F3:L55-L56 |
| DF002-F3-013 | E-P | A frozen commitment plus tainted span plus out-of-set call emits `untrusted_var_quarantined`. | F3:L55, L60 |
| DF002-F3-014 | E-P | `formulate_query` persists the commitment on the orchestrator and emits `tool_filter_committed` through `emit_stage`. | F3:L58 |
| DF002-F3-015 | E-P | Both `Fsm::check_tool_allowed` and `mcp::gate::check_tool_allowed` consult the commitment after their existing state/category checks pass. | F3:L60 |
| DF002-F3-016 | E-P | An out-of-set attempt reuses existing `tool_denied` and adds the commitment hash to that denial payload. | F3:L60-L62 |
| DF002-F3-017 | E-P | New SSB state is one `ToolFilterEvent` variant with `ToolFilterEventKind::{Committed,UntrustedQuarantined}`. | F3:L62 |
| DF002-F3-018 | E-P | Its two SQL action spellings are `tool_filter_committed` and `untrusted_var_quarantined`. | F3:L62 |
| DF002-F3-019 | E-P | `ToolFilterPayload` carries task ID, BLAKE3 commitment hash, allowed categories, optional denied tool, and optional taint source. | F3:L62 |
| DF002-F3-020 | E-D | `capability_flow_blocked` is reserved for full CaMeL and must be absent until it has a producer. | F3:L62, L134-L135, L170 |
| DF002-F3-021 | E-P | The schema impact is one Pattern-2 variant with a baseline-plus-one bump and golden round-trip. | F3:L62, L171 |
| DF002-F3-022 | E-P | `elai safety tool-filter --fixture …` is a production CLI action that runs the real researcher commitment/enforcement path. | F3:L64, L104, L118 |
| DF002-F3-023 | E-P | The fixture injects a request to call `fs.write`; the attempt must be denied after commitment. | F3:L64, L118-L120 |
| DF002-F3-024 | E-P | CLI output is JSON containing commitment hash, denied tool, and fixture-suite ASR. | F3:L64, L120 |
| DF002-F3-025 | E-P | The deterministic suite contains at least 20 AgentDojo-style web-page injections, each attempting out-of-set escalation. | F3:L68 |
| DF002-F3-026 | E-P | Fixture ASR is calculated before without a commitment and after with a commitment; no AgentDojo number is transferred as an ELAI result. | F3:L68 |
| DF002-F3-027 | E-G | Tier-1 completion requires `asr_after < asr_before` and denial of all 20 injected escalations. | F3:L166 |
| DF002-F3-028 | E-D | A real sidecar/cloud researcher ASR run is a separate gated checkbox and not required for Tier-1 done. | F3:L68, L167 |
| DF002-F3-029 | E-P | The existing `ToolGateInspector.svelte` gains a Tool Filter/Data-plane section; `commands_c35.rs` maps both new event spellings into extended `ToolGateRowDto`. | F3:L66, L114-L116 |
| DF002-F3-030 | E-P | The surface displays allow-set, commitment hash, quarantines, and derived before/after ASR. | F3:L66, L115 |
| DF002-F3-031 | E-P | Tool Filter must be default-on; any `ELAI_CAMEL_TOOLFILTER_ENABLE`-named control is specified as disable-only despite its name. | F3:L112, L164, L220 |
| DF002-F3-032 | E-G | The live trigger must create commitment, quarantine, and reused denial rows in one run and keep the FSM from advancing with the escalated tool. | F3:L117-L120, L163-L170 |
| DF002-F3-033 | E-G | The JSON artifact must contain a 64-hex commitment, `denied_tool="fs.write"`, and strictly lower after-ASR. | F3:L120 |
| DF002-F3-034 | E-G | Completion requires executed wiring check, default-on proof, dashboard E2E, schema/golden lock, cross-language parity, workspace compile, and auditor pass. | F3:L122-L172 |
| DF002-F3-035 | E-G | Contract tests cover deterministic freezing/hash, out-of-set denial, classifier-to-taint mapping, quarantine emission, both Kind round trips, and reserved-event absence. | F3:L175-L179 |
| DF002-F3-036 | E-D | Full CaMeL uses typed `SymbolicVar` values and a non-LLM controller so untrusted-derived data cannot alter control flow. | F3:L72-L74 |
| DF002-F3-037 | E-D | Full CaMeL requires an IPC schema break because the researcher returns typed symbolic values rather than raw strings. | F3:L74 |
| DF002-F3-038 | E-D | Full CaMeL may be scheduled only after the primary PDF overhead is read, real ELAI IPC overhead preserves the 32K ceiling, and the breaking change is accepted. | F3:L70, L74 |
| DF002-F3-039 | E-D | The secondary `2.82x` input/`2.73x` output overhead must not be represented as primary or as measured on ELAI; 4–5x on ELAI is only a likelihood claim. | F3:L3-L4, L70, L199-L200, L210 |
| DF002-F3-040 | E-R | A fetch path that bypasses `formulate_query` would miss commitment; the contract must establish no such production path. | F3:L208 |
| DF002-F3-041 | E-R | An over-narrow set can deny legitimate research; the mitigation is role-derived grounding capabilities and measurement of false-deny rate on real runs. | F3:L209 |
| DF002-F3-042 | E-P | The implementation may use only `BTreeSet`/standard collections and existing BLAKE3; no new dependency or gate-time network/subprocess/RunPod work is allowed. | F3:L5, L49, L56, L212-L213 |
| DF002-F3-043 | E-S | The source corrects a nonexistent `crates/plugins/src/prompt_injection.rs` citation to `crates/safety/src/prompt_armor.rs` plus `spotlight.rs`. | F3:L26, L204 |
| DF002-F3-044 | E-D | AgentDojo’s `57.7%→6.84% at 73% utility` and CaMeL’s 77% provable-security figure are external ceilings, not ELAI measurements. | F3:L13, L199-L200 |

### F-5 — cross-subsystem staleness quarantine

**Source status boundary:** no top-level shipped/planned marker is present. Phase 1 is a mandatory audit, Phase 2 is the proposed cascade, and Phase 2 is blocked if B-30 has not stabilized `dkg_nodes`.

| Atomic ID | Evidence | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| DF002-F5-001 | E-P | F-5 extends existing memory bi-temporal decay with a cross-subsystem invalidate→demote→broadcast cascade rather than reimplementing contradiction or decay engines. | F5:L1-L7, L9-L12 |
| DF002-F5-002 | E-S | `decay_tick` already advances trust decay but emits no cross-subsystem signal. | F5:L13-L15 |
| DF002-F5-003 | E-S | `spawn_decay_scheduler` has an optional tick observer but is asserted live only in comments/tests; the production path is the inline post-reflect `decay_tick` in `live_cli.rs`. | F5:L15, L227 |
| DF002-F5-004 | E-S | `audit_conflicts` is the existing contradiction closer and must be reused. | F5:L16, L26 |
| DF002-F5-005 | E-S | `dkg_nodes.concept_stale` defaults to and is inserted as `1`, with no real alternate population yet. | F5:L17, L31 |
| DF002-F5-006 | E-P | Phase 1 inventories every DKG SELECT/read and records whether it reads or assumes `concept_stale=1` and whether ranking is affected. | F5:L52-L55 |
| DF002-F5-007 | E-G | Before any population change, a contract fixture covers states 0, 1, and 2; state 0 must remain included and state 2 must demote rather than disappear. | F5:L56, L155, L180 |
| DF002-F5-008 | E-P | Phase 2 adds one `SsbActionType::StalenessQuarantineBroadcast`. | F5:L58-L60 |
| DF002-F5-009 | E-P | The broadcast payload carries fact ID, staleness ratio, dependent-demotion count, Cosmos-flag count, tick time, and broadcast ID. | F5:L60 |
| DF002-F5-010 | E-P | `broadcast_id` is a BLAKE3 prefix of `fact_id || tick_ms`. | F5:L60 |
| DF002-F5-011 | E-P | The broadcast is a single action shape with SQL parse/check, payload round-trip, golden lock, and baseline-plus-one schema bump. | F5:L60, L158 |
| DF002-F5-012 | E-P | `collect_quarantined` reuses existing decay math and selects facts with `staleness_ratio > 2.0` on the tick. | F5:L62 |
| DF002-F5-013 | E-P | `QuarantineRecord` contains fact ID and staleness ratio. | F5:L62 |
| DF002-F5-014 | E-P | Contradiction-driven quarantine uses the existing `audit_conflicts` verdict instead of a second detector. | F5:L62, L74 |
| DF002-F5-015 | E-P | Broadcast emission is attached to both the live inline tick and the scheduler observer; the observer cannot be the sole caller. | F5:L62, L169, L227 |
| DF002-F5-016 | E-P | `apply_staleness_broadcast` locates DKG concepts citing a quarantined fact, sets them to `concept_stale=2`, and returns a demotion count. | F5:L64 |
| DF002-F5-017 | E-P | DKG contradiction edges are closed bi-temporally/non-destructively when the broadcast is applied. | F5:L64, L183 |
| DF002-F5-018 | E-P | Pending broadcasts are drained after `ensure_dkg_or_rebuild` through the existing knowledge SSB sink. | F5:L64, L93-L100 |
| DF002-F5-019 | E-P | `flag_on_staleness_broadcast` transitions cited Cosmos artifacts through the existing `CosmosStatus::Stale` path and returns a count. | F5:L66 |
| DF002-F5-020 | E-P | No new Cosmos status or decay engine is introduced. | F5:L28, L66 |
| DF002-F5-021 | E-P | `QuarantineBroadcastQueue` is an in-memory `tokio::sync::mpsc` with named capacity `QUARANTINE_QUEUE_CAPACITY=256`. | F5:L68 |
| DF002-F5-022 | E-R | When full, the queue drops the oldest item and places a visible `dropped_count` on the next broadcast; silent loss is forbidden. | F5:L68, L108, L157 |
| DF002-F5-023 | E-P | The cascade is enabled by default; `ELAI_STALENESS_CASCADE_KILL=1` is disable-only. | F5:L74, L103-L109, L153 |
| DF002-F5-024 | E-P | Quarantine decisions are deterministic and cannot be overridden by an LLM/prompt. | F5:L107 |
| DF002-F5-025 | E-P | `get_staleness_quarantine` returns typed broadcast rows and aggregate DKG/Cosmos counts to a knowledge/staleness drawer. | F5:L70, L86, L110 |
| DF002-F5-026 | E-P | The Tauri command is registered in the invoke handler and the drawer invokes it through `@tauri-apps/api/core`. | F5:L70 |
| DF002-F5-027 | E-P | `elai dkg staleness-tick --workspace .` forces one production decay tick, drains the queue, applies both consumers, and prints a summary. | F5:L72, L112-L114 |
| DF002-F5-028 | E-G | The live trigger must journal `staleness_quarantine_broadcast` and either write `F-5-staleness-tick-*.json` or print `dependents_demoted`, with at least one DKG row changed to 2. | F5:L111-L114, L152-L154 |
| DF002-F5-029 | E-G | Mock-clock acceptance requires every citing concept to flip within one tick and the next grounding query to miss cache. | F5:L156, L181 |
| DF002-F5-030 | E-G | Queue-overflow acceptance requires visible dropped count on the next broadcast. | F5:L157, L182 |
| DF002-F5-031 | E-G | Completion requires executed wiring check, default-on proof, dashboard observation, golden/schema checks, cross-language parity, workspace compile, and auditor pass. | F5:L150-L162 |
| DF002-F5-032 | E-G | F-5 is fully deterministic/headless and has no RunPod/live-model split. | F5:L162, L171, L229 |
| DF002-F5-033 | E-P | The cascade uses SQLite updates and an in-process bounded queue; no network, subprocess, or external dependency is added. | F5:L161, L176, L229 |
| DF002-F5-034 | E-D | Rules-crate 90-day decay is an explicitly excluded third isolated decay in v1. | F5:L30 |
| DF002-F5-035 | E-R | Phase 2 is blocked until B-30 stabilizes `dkg_nodes`; Phase 1 can proceed independently. | F5:L5, L27, L225 |
| DF002-F5-036 | E-R | The stated `>1.0` flag and `>2.0` quarantine thresholds originate in SHARD; adopting `>2.0` for ELAI remains a decision to confirm. | F5:L11, L220 |
| DF002-F5-037 | E-R | Grounding-cache invalidation is only an assumed measurable until Phase 1 proves cache keys consume freshness. | F5:L220 |
| DF002-F5-038 | E-R | Drop-oldest may discard the most stale/important quarantine; visible loss and later capacity tuning are the only stated mitigations. | F5:L226 |
| DF002-F5-039 | E-R | The existing Cosmos status must be verified to contain a usable `Stale` transition before implementation. | F5:L228 |
| DF002-F5-040 | E-P | F-5’s decision record must preserve that ELAI already has bi-temporal memory semantics and that SHARD supplies a blueprint, not adopted code. | F5:L74, L213-L217 |
| DF002-F5-041 | E-D | Future composition may demote skills that cite quarantined facts and feed F-7 utility, but neither is part of F-5. | F5:L231-L233 |
| DF002-F5-042 | E-S | The plan asserts each broadcast row flows through the OTLP exporter for off-machine archival. | F5:L233 |

### F-6 — learned-skill library

**Source status boundary:** no top-level implementation status is stated. The three ordered increments are C6a governance, C6b critic-gated acquisition, and C6c staleness demotion; each is described normatively and the deferred reuse measurement is explicit.

| Atomic ID | Evidence | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| DF002-F6-001 | E-P | A learned skill is acquired only through a critic gate, governed by a separate execution-autonomy ladder, retrieved through the existing reranker, and demoted on staleness. | F6:L1-L9, L38-L49 |
| DF002-F6-002 | E-P | Delivery is ordered C6a→C6b→C6c, each gated on its predecessor. | F6:L11-L14, L125, L187, L216 |
| DF002-F6-003 | E-S | `TemplateCrystallizer` is asserted shipped and crystallizes after `CRYSTALLIZATION_THRESHOLD_N` consecutive successes. | F6:L16-L18, L68-L74 |
| DF002-F6-004 | E-S | BestOfN already invokes `MiniCritic` and journals Pass/Warn/Block verdicts; F-6 may not make a fresh critic call. | F6:L22-L24, L75-L79 |
| DF002-F6-005 | E-S | Existing memory `TrustTier::{T0UserDirect,T1CodebaseDerived,T2WebSourced,T3ModelEmitted}` represents provenance, not execution autonomy. | F6:L55-L57, L85-L88 |
| DF002-F6-006 | E-P | `SkillTrustTier::{S1Candidate,S2Probationary,S3Established,S4Trusted}` uses distinct `S1`–`S4` wire tags. | F6:L127-L131 |
| DF002-F6-007 | E-P | No `From`/`Into` conversion may exist between skill and memory trust types. | F6:L130-L131, L291 |
| DF002-F6-008 | E-P | `LearnedSkill` stores UUID, concept class, plan-skeleton hash, tier, invocation/failure counts, acquired/last-verified times, optional embedding reference, and optional critic-verdict SSB ID. | F6:L132-L136 |
| DF002-F6-009 | E-P | Promotion requires named per-tier minimum invocation, maximum failure-rate, and minimum-age thresholds. | F6:L137-L142 |
| DF002-F6-010 | E-P | Promotion caps at S3 unless explicit human promotion permission is present; S4 promotion is human-gated. | F6:L139-L142 |
| DF002-F6-011 | E-P | `permits_autonomous()` is true only for `S4Trusted`; every sub-S4 skill routes to the human gate. | F6:L143-L144, L161-L170 |
| DF002-F6-012 | E-P | The autonomy decision conjoins existing `should_auto_advance(...)` with the skill-tier permission. | F6:L161-L168 |
| DF002-F6-013 | E-P | Sub-S4 denial reuses the existing auto-advance-denied observability rather than creating a new skill event. | F6:L167-L169 |
| DF002-F6-014 | E-P | The autonomy barrier is default-on; `ELAI_SKILL_AUTONOMY_BARRIER_DISABLE=1` is the only allowed disable switch. | F6:L169-L170, L285-L287 |
| DF002-F6-015 | E-P | A terminal default-off `ELAI_SKILL_LIBRARY_ENABLE` flag is forbidden. | F6:L285-L287, L325-L327 |
| DF002-F6-016 | E-P | Skill content hashes use BLAKE3; production paths must avoid `.unwrap()`/`.expect()`. | F6:L145-L146 |
| DF002-F6-017 | E-P | SSB uses one `SkillLibraryEvent` with `SkillLibraryEventKind::{Acquired,TrustPromoted,DemotedStale}`. | F6:L148-L159 |
| DF002-F6-018 | E-P | Its payload carries kind, skill ID, concept class, optional from/to tiers, optional critic row ID, optional staleness score, and content BLAKE3. | F6:L151-L153 |
| DF002-F6-019 | E-P | SQL registration uses `skill_library_event`; the three lifecycle labels are Kind values under one Pattern-2 variant. | F6:L153-L159 |
| DF002-F6-020 | E-P | The schema change is baseline plus one with parse/check arms, round-trip fixture, and action-type lock. | F6:L153-L159 |
| DF002-F6-021 | E-P | CLI verbs are `library list`, `library show <skill_id>`, `library promote <skill_id>`, and `library tick` under `elai skills-mgmt`. | F6:L172-L177, L218-L227 |
| DF002-F6-022 | E-P | Successful CLI promotion emits `TrustPromoted`; promotion to S4 requires `--human-grant`. | F6:L173-L176 |
| DF002-F6-023 | E-G | Type-barrier tests require `S2 != T2`, absence of both conversions, S4-only autonomy, and Kind-preserving payload round-trip. | F6:L179-L185 |
| DF002-F6-024 | E-P | When crystallization reaches its threshold, the acquisition gate looks up existing `mini_critic_verdict` rows correlated to the successful artifact. | F6:L189-L195 |
| DF002-F6-025 | E-P | Correlation is by superstep and agent ID; the learned skill pins the matching verdict row ID. | F6:L191-L200, L473-L477 |
| DF002-F6-026 | E-P | Only `MiniCriticVerdict::Pass` permits acquisition; Warn, Block, or absent evidence returns no skill. | F6:L193-L200, L210-L214 |
| DF002-F6-027 | E-P | A passing acquisition creates an S1 candidate, stores it, and journals an Acquired lifecycle event. | F6:L195-L200 |
| DF002-F6-028 | E-P | `acquire_learned_skill` is a separately testable async method and makes zero new LLM/critic calls. | F6:L194-L200 |
| DF002-F6-029 | E-P | At plan-execution start, the existing GraphCodeBERT/Granite reranker queries learned skills by description embedding. | F6:L202-L208 |
| DF002-F6-030 | E-P | A matching skill must be at least S3 before its compiled pipeline is offered as a candidate. | F6:L203-L206 |
| DF002-F6-031 | E-P | `description_embedding_ref` is populated during acquisition; no new model or dependency is introduced. | F6:L202-L208 |
| DF002-F6-032 | E-D | Retrieval recall A/B is measurement-only Tier-2 and never gates execution or completion. | F6:L207-L208, L406-L410 |
| DF002-F6-033 | E-P | Staleness is `(now_secs-last_verified_secs)/SKILL_EXPECTED_LIFETIME_SECS` using a named lifetime constant. | F6:L218-L221 |
| DF002-F6-034 | E-P | `demote_if_stale` drops exactly one tier when score exceeds `SKILL_STALE_THRESHOLD`. | F6:L221-L224 |
| DF002-F6-035 | E-P | Staleness demotion emits `DemotedStale` with the computed score and never goes below S1. | F6:L221-L224, L238-L242 |
| DF002-F6-036 | E-P | Demotion runs on the existing background decay tick and can also be triggered by `library tick`. | F6:L223-L227 |
| DF002-F6-037 | E-P | Skill staleness is separate metadata from F-5 memory/DKG staleness; only the scheduler edit window is shared. | F6:L28-L31, L244-L248, L481-L483 |
| DF002-F6-038 | E-P | The frontend shows tier, invocation/failure counts, staleness score, and Acquired/Promoted/Demoted event feed. | F6:L229-L236 |
| DF002-F6-039 | E-P | The frontend must use exactly one dashboard mounting strategy: MonitorWindow panel or `DrawerKey`, after checking the actual surface. | F6:L234-L236, L478-L480 |
| DF002-F6-040 | E-G | The live observable runs `library promote … --human-grant` then `library tick` without environment overrides. | F6:L296-L299, L345-L347 |
| DF002-F6-041 | E-G | Evidence is at least one live `skill_library_event`, list output matching the resulting `S[1-4]` tier, and panel rendering. | F6:L297-L299 |
| DF002-F6-042 | E-G | C6b tests require Pass→Some(S1)+Acquired, Block→None/no event, and absent verdict→None. | F6:L210-L214, L353-L354 |
| DF002-F6-043 | E-G | C6c mock-clock tests require stale S3→S2 plus event, fresh unchanged, and S1 floor. | F6:L238-L242, L355-L356 |
| DF002-F6-044 | E-G | Completion requires executed wiring check, default-on proof, dashboard E2E, schema/golden lock, cross-language parity, workspace compile, and auditor pass. | F6:L301-L363 |
| DF002-F6-045 | E-D | No skill-reuse-rate number is allowed until at least 20 real acquisitions and concept-class tagging exist. | F6:L51-L54, L244-L248, L364-L366 |
| DF002-F6-046 | E-D | Voyager’s 3.3x Minecraft result is non-transferable and may not become a coding reuse claim. | F6:L51-L54, L453-L461 |
| DF002-F6-047 | E-R | Loose verdict correlation could admit a skill on an unrelated Pass; the same `(agent_id,superstep)` must be enforced or threaded through execution outcomes. | F6:L473-L477 |
| DF002-F6-048 | E-R | Concurrent schema-bump work must reconcile to one monotonic shipped-unit bump. | F6:L486-L487 |
| DF002-F6-049 | E-D | A dashboard S4-approval button is a follow-up; the CLI human-grant flag is the in-scope gate. | F6:L488-L489 |
| DF002-F6-050 | E-P | The learned-skill store is held with the crystallizer in an `Arc<AsyncMutex<TemplateCrystallizer>>`; F-6 threads store state through runtime setup. | F6:L68-L74, L440-L441 |
| DF002-F6-051 | E-P | No external dependency may be added; the plan reuses reranker, serde, BLAKE3, and UUID facilities. | F6:L362-L363, L467-L469 |

### F-7 — immutable-rule lattice and CBR ranking

**Source status boundary:** the reconciled WIRING_CONTRACT explicitly labels Half B “SHIPPED, as-built” and Half A intentionally deferred/RunPod-pending. Older task, deterministic-check, acceptance, files-touched, and doctrine text still require Half A now. Both layers are retained and cross-referenced in the conflict ledger.

| Atomic ID | Evidence | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| DF002-F7-001 | E-P | F-7 contains two independently shippable halves: CBR ranking ships first; immutable-rule lattice proof is an E-53 amendment. | F7:L1-L13, L52-L82 |
| DF002-F7-002 | E-S | Existing ranking is raw `first_try_success_rate=successes/bucket_len`, allowing a 1/1 bucket to outrank 95/100. | F7:L17-L21 |
| DF002-F7-003 | E-P | `laplace_utility(outcomes,block_count,alpha,beta)` starts from `(successes+alpha)/(n+alpha+beta)`. | F7:L54-L56 |
| DF002-F7-004 | E-P | Utility is down-weighted by `block_ratio=block_count/max(n,1)` and clamped to `[0,1]`. | F7:L56 |
| DF002-F7-005 | E-P | Default priors are named constants `LAPLACE_ALPHA=1.0` and `LAPLACE_BETA=1.0`; zero total prior is forbidden. | F7:L56 |
| DF002-F7-006 | E-P | `TemplateUtility` stores concept class, plan-skeleton hash, utility, old first-try ratio, block ratio, coverage, and sample count. | F7:L56 |
| DF002-F7-007 | E-P | When multiple templates match identity, `CompiledExecutor::try_execute` uses utility instead of raw first-try rate for selection/tie-break. | F7:L58 |
| DF002-F7-008 | E-P | `first_try_success_rate` remains stored and displayed as the measurement baseline. | F7:L58, L82 |
| DF002-F7-009 | E-P | Every ranking decision emits `template_utility_ranked`. | F7:L58-L60 |
| DF002-F7-010 | E-P | `TemplateUtilityRankedPayload` carries all utility inputs plus chosen pipeline ID. | F7:L60 |
| DF002-F7-011 | E-P | `TemplateUtilityRanked` has SQL/check/parse/round-trip/golden registration and a baseline-plus-one schema impact when shipped alone. | F7:L60 |
| DF002-F7-012 | E-P | `elai template-ranking list|show --concept-class X [--json]` exposes buckets sorted descending by utility. | F7:L62 |
| DF002-F7-013 | E-P | Half-B’s reconciled UI is `TemplateRankingDrawer.svelte` plus `commands_f7.rs::get_template_ranking`. | F7:L86-L96, L121-L124, L136-L137 |
| DF002-F7-014 | E-P | The surface renders utility and first-try ratio side by side. | F7:L64, L143 |
| DF002-F7-015 | E-G | Half-B’s live trigger is `template-ranking list --json`, producing one ranking row and JSON with utility plus sibling first-try ratio. | F7:L138-L143, L185-L190 |
| DF002-F7-016 | E-G | Smoothing correctness requires 95/100 to outrank 1/1 in unit and live ordering. | F7:L189, L199 |
| DF002-F7-017 | E-D | Empirical-Bayes per-class priors are deferred until at least 100 sessions exist. | F7:L239 |
| DF002-F7-018 | E-D | CBR uplift is measured only after at least 100 labelled sessions against the old baseline; no numeric uplift gates this F-N. | F7:L23, L82, L190, L238 |
| DF002-F7-019 | E-D | Springdrift’s 7,056-pair/100%-coverage and +10.7% results are synthetic/unreproduced hypotheses, not ELAI targets. | F7:L23, L228-L230 |
| DF002-F7-020 | E-P | Half A enumerates the `mutable=0` security and finance rule subset as a set. | F7:L17-L20, L66-L68 |
| DF002-F7-021 | E-P | Its SMT obligation asserts coverage of every declared class by at least one immutable rule. | F7:L68 |
| DF002-F7-022 | E-P | Its SMT obligation asserts that rule-set union never reduces the blocked-action set. | F7:L68 |
| DF002-F7-023 | E-P | Its SMT obligation asserts that no two immutable rules contradict on the same argv/class. | F7:L68 |
| DF002-F7-024 | E-P | `SmtLib2Obligation` stores class count, rule count, SMT-LIB2 text, and a BLAKE3 obligation hash. | F7:L68 |
| DF002-F7-025 | E-P | Lattice DB access propagates `rusqlite::Error` and production code has no `.unwrap()`/`.expect()`. | F7:L68, L194 |
| DF002-F7-026 | E-D | Half A is blocked until E-53’s `VerifierKind::Z3Smt` route stops returning the `z3_stub` inapplicable outcome. | F7:L7-L9, L19-L20 |
| DF002-F7-027 | E-P | The proposed Z3 dispatch accepts a lattice discriminator/constructor and calls the shared formal-verification `run` pipeline. | F7:L70 |
| DF002-F7-028 | E-R | If Z3 is absent, the route returns `RunpodPending`, never a fabricated `Verified`. | F7:L70, L134 |
| DF002-F7-029 | E-P | With Z3 present, discharge yields `Verified`; a counterexample yields `Refuted{counterexample_label}`. | F7:L70 |
| DF002-F7-030 | E-P | The original proposed CLI spelling is `elai formal-verify lattice`, which reports verifier/outcome/hash/class/rule counts. | F7:L72 |
| DF002-F7-031 | E-P | Proposed `safety_lattice_verified` carries class/rule counts, obligation hash, outcome string, verified Boolean, and RunPod-pending Boolean. | F7:L72-L74 |
| DF002-F7-032 | E-P | Local/no-Z3 execution would journal `verified=false,runpod_pending=true`. | F7:L72 |
| DF002-F7-033 | E-P | The deterministic Half-A fixture uses a seeded in-memory rules DB and exercises the real formal-verification dispatcher up to the Z3 boundary. | F7:L76 |
| DF002-F7-034 | E-G | A complete fixture must produce a well-formed obligation and local `RunpodPending` with hash; an incomplete fixture must encode its uncovered-class assertion. | F7:L76, L191 |
| DF002-F7-035 | E-D | The actual Z3 discharge is a separate ignored/live checkbox; a complete set must verify and a planted contradiction must refute. | F7:L76, L192 |
| DF002-F7-036 | E-P | A proposed lattice dashboard section renders counts, outcome, verification/pending state, and hash prefix. | F7:L78 |
| DF002-F7-037 | E-D | The reconciled as-built contract excludes lattice symbols, `SafetyLatticeVerified`, and `get_lattice_status` from produced/reachable/frontend expectations. | F7:L97-L107, L125-L140 |
| DF002-F7-038 | E-S | The reconciled as-built contract explicitly asserts Half B’s utility module, CLI, SSB row, Tauri command, and drawer are shipped. | F7:L86-L96, L121-L124 |
| DF002-F7-039 | E-G | Completion for the reconciled shipped half requires executed wiring check, default-on ranking, live ranking row/artifact, golden/cross-language gates, compile, and auditor pass. | F7:L138-L195 |
| DF002-F7-040 | E-P | Half-B ranking is default-on; any `ELAI_TEMPLATE_CBR_ENABLE` variable must be disable-only. | F7:L128-L134, L176-L177, L186 |
| DF002-F7-041 | E-P | Neither arithmetic ranking nor obligation encoding may consult an LLM or accept a prompt override. | F7:L133 |
| DF002-F7-042 | E-P | Half B is pure in-memory arithmetic over journaled outcomes; Half A’s Z3 subprocess stays out of per-write/fail-closed paths. | F7:L23-L25, L256-L257 |
| DF002-F7-043 | E-P | No Z3 Rust crate is added; SMT-LIB2 is hand-emitted and Z3 uses E-53’s runtime install probe. | F7:L233, L240-L242 |
| DF002-F7-044 | E-R | Hand-emitted SMT may be malformed; structural fixture assertions and later live Z3 review are the stated safeguards. | F7:L240 |
| DF002-F7-045 | E-R | Fixed 1/1 priors may under-smooth high-variance classes; this is accepted until enough data exists. | F7:L239 |
| DF002-F7-046 | E-P | Co-shipping both distinct SSB variants would use an atomic baseline-plus-two bump; shipping either alone uses the appropriate single bump. | F7:L34, L60, L74, L193 |
| DF002-F7-047 | E-D | No imported safety-coverage percentage or CBR uplift percentage may be claimed. | F7:L23, L229-L230 |
| DF002-F7-048 | E-D | Future consumers may use template utility for skill-tier promotion and the lattice as a self-mod pre-archive invariant; neither is in this increment. | F7:L259-L261 |

### F-WIRING-DOCTRINE-WHY — global enforcement doctrine and empirical claims

**Source status boundary:** this auxiliary source calls F-0 the canonical specification and presents itself as an empirical explainer written after four F units were built. Its historical percentages and shipped/fixed claims are explicit assertions of this source only.

| Atomic ID | Evidence | Explicit feature or behavior | Exact source anchor |
|---|---|---|---|
| DF002-WD-001 | E-S | The 2026-05-30 audit is asserted to cover 187 B/C/D/E plans. | WD:L9-L10 |
| DF002-WD-002 | E-S | Its asserted distribution is about 52% shipped-wired, 15% shipped-wire-dark, 17% shipped-env-gated, 10% partial, and 5% not shipped. | WD:L9-L12 |
| DF002-WD-003 | E-S | The doctrine summarizes that distribution as roughly 40–45% wire-dark, env-gated, or partial. | WD:L12 |
| DF002-WD-004 | E-S | The source reports about 232 invisible backend signals with producers but no consumers. | WD:L13-L14 |
| DF002-WD-005 | E-S | Earlier plans already documented wiring contracts, but compilation/unit-test completion did not execute those contracts. | WD:L16-L24 |
| DF002-WD-006 | E-P | F-plan completion asks whether a feature fired end-to-end from a real production entrypoint, default-on, with human-visible evidence. | WD:L27-L34 |
| DF002-WD-007 | E-G | The canonical executed gate is `python .claude/tools/wiring_contract_check.py --bn-file <F-N>.md --run-trigger`, which must pass. | WD:L33-L34 |
| DF002-WD-008 | E-G | W-1 requires every produced symbol to resolve as a real definition. | WD:L39-L42 |
| DF002-WD-009 | E-G | W-1 requires consumed symbols to have real callers and reachable anchors to be found by production-entrypoint BFS. | WD:L39-L42 |
| DF002-WD-010 | E-G | W-1 also requires zero stub-ban hits and an actually executed observable effect with expected SSB row and artifact. | WD:L39-L42 |
| DF002-WD-011 | E-G | W-2 forbids a terminal default-off environment gate but permits a default-on disable-only kill switch. | WD:L44-L46 |
| DF002-WD-012 | E-G | W-3 requires a CLI, FSM transition/evidence gate, tool hook, or Tauri production entrypoint; test-only reachability is not done. | WD:L48-L50 |
| DF002-WD-013 | E-G | W-4 requires a Svelte/Tauri frontend pair and passing cross-language diff for any dashboard-surfaceable signal. | WD:L52-L53 |
| DF002-WD-014 | E-G | W-5 requires a real command that writes a real SSB row and artifact; a unit test cannot be the observable effect. | WD:L55-L58 |
| DF002-WD-015 | E-G | When a live model is unavailable, W-5 requires a deterministic fixture over the same production path and a separate live-run checkbox. | WD:L55-L58 |
| DF002-WD-016 | E-G | W-6 requires at least one acceptance check performed through the dashboard. | WD:L60-L61 |
| DF002-WD-017 | E-G | W-7 requires every existing integration symbol to be confirmed in `elai.map` at authoring time. | WD:L63-L65 |
| DF002-WD-018 | E-G | W-8 requires every invocable produced symbol to get an in-plan caller in the same F-N. | WD:L67-L69 |
| DF002-WD-019 | E-G | W-9 requires new action types in the golden lock, a named schema bump, and passing action-type/cross-language checks. | WD:L71-L72 |
| DF002-WD-020 | E-G | W-10 splits any network/subprocess gate axis from in-process Tier-1 and forbids fail-closed claims for fail-open work. | WD:L74-L77 |
| DF002-WD-021 | E-G | The six-part done definition is workspace compile, auditor pass, executed wiring contract, cross-language pass when applicable, dashboard E2E, and memory-writer execution. | WD:L79-L82 |
| DF002-WD-022 | E-S | The source asserts four F units were built and that doctrine gates caught live-path defects in three. | WD:L84-L90 |
| DF002-WD-023 | E-S | F-1 Unit A allegedly reintroduced a renamed default-off environment pattern and left its SSB writer callerless despite compile success. | WD:L92-L94 |
| DF002-WD-024 | E-S | The F-1 audit allegedly corrected Unit A to default-on and wired it at the real dispatcher construction. | WD:L92-L94 |
| DF002-WD-025 | E-S | F-3 Unit C allegedly wired enforcement points without instantiating them at production bootstrap. | WD:L96-L98 |
| DF002-WD-026 | E-S | After bootstrap repair, the F-3 live trigger allegedly exposed an SSB-construction/runtime-order panic missed by compile and pure tests. | WD:L96-L101 |
| DF002-WD-027 | E-S | After repair, the F-3 live run allegedly wrote commitment and quarantine rows, blocked an out-of-set write, and measured fixture ASR `1.0→0.0`. | WD:L99-L101 |
| DF002-WD-028 | E-S | F-9 Unit D allegedly never fired when production passed `file_path=None` plus task-summary text to an extension-based extractor. | WD:L103-L107 |
| DF002-WD-029 | E-S | F-9’s Python registry JSON contract allegedly mismatched the live checker; the repaired offline trigger then wrote `write_time_slopsquat_blocked`. | WD:L103-L108 |
| DF002-WD-030 | E-S | Unit B is asserted clean because Unit A’s W-2/W-3 lessons were incorporated in advance. | WD:L107-L108 |
| DF002-WD-031 | E-P | The doctrine’s claimed distinction is that old plans prove code exists while F-plan must prove it runs in production, default-on, with a human-visible trace. | WD:L110-L119 |

## Exact identifier, schema, surface, and trigger registry

This registry is a lossless index into the atomic rows; it is not an additional atomic count.

| Source | Produced identifiers / data | Consumed or production anchors | Events / surfaces / commands / flags |
|---|---|---|---|
| F-1 | `evaluate_vibegate`; `SECRET_PATTERNS`; `emit_vibegate_blocked_to_ssb`; `verify_vibesec_seed_integrity`; `check_vibegate_seed_integrity`; `VibeGateBlocked`; `VibeGateBlockedKind`; `VibeGateBlockedPayload` | `build_production_hook_dispatcher`; `build_task_runner_wiring`; `dispatch_pre_tool_use`; `RuleEngine::evaluate`; `ALWAYS_INCLUDED_DOMAINS`; `T1Hook`; `HookFinding`; `emit_hook_events_to_ssb` | `vibesec_code_vuln_blocked`; `vibesec_secret_blocked`; reserved `vibesec_dep_hallucinated_blocked`/`vibesec_skill_blocked`; `VibeGateDrawer`; `list_vibegate_blocks`; `DrawerKey 'vibegate'`; `ELAI_VIBEGATE_DISABLED` |
| F-2 | four original TOMLs; `SecurityRuleBlocked`; `SecurityRuleBlockedPayload`; `load_security_seed_rules_builtin`; `seed_security_rules`; deferred `InsecureRateMeasured`/payload/runner symbols in the predecessor’s reconciliation | `RuleEngine::evaluate`; `Predicate`; `FileContext`; `load_seed_rules_from_dir`; `dispatch_pre_tool_use`; `build_hook_block_summary`; production rules initialization | `security_rule_blocked`; `insecure_rate_measured`; `SecurityRulesPanel`; `get_security_blocks_recent`; deferred `get_insecure_rate_latest`; `elai bench seccodeplt`; no enable flag |
| F-2c | `hardcoded_creds.toml`; `hardcoded_key.toml`; `secrets_in_logs.toml`; `eval_injection.toml`; `F-2c-decisions.md` | `seed_security_rules`; `load_security_seed_rules_builtin`; `Predicate`; `FileContext`; `eval_order`; `detect_language_from_ext`; `evaluate_vibegate`; existing bench/runner/event | Reuses `security_rule_blocked`, `insecure_rate_measured`, `SecurityRulesPanel`, and `commands_f2`; no new flag, variant, schema bump, DTO, or dependency |
| F-3 | `CommittedToolSet`; `commit_minimal_set`; `check_against_commitment`; `CapabilityFlowDenial`; `UntrustedTaint`; `taint_from_verdict`; `ToolFilterEvent`; `ToolFilterEventKind`; `ToolFilterPayload`; `SafetyAction::ToolFilter` | `ToolCategory`; FSM/MCP `check_tool_allowed`; `formulate_query`; `fetch_and_extract`; `emit_stage`; `allowed_grounding_methods`; `classify_injection`; existing `ToolDenied` | `tool_filter_committed`; `untrusted_var_quarantined`; existing `tool_denied`; reserved/absent `capability_flow_blocked`; `elai safety tool-filter`; `ToolGateInspector`; `commands_c35`; default-on tool-filter switch |
| F-5 | `StalenessQuarantineBroadcast`; payload; `collect_quarantined`; `QuarantineBroadcastQueue`; `QuarantineRecord`; `apply_staleness_broadcast`; `flag_on_staleness_broadcast`; `get_staleness_quarantine` | `decay_tick`; scheduler/observer; `audit_conflicts`; `KnowledgeSsbSink`; `ensure_dkg_or_rebuild`; Cosmos `status_for`; inline live tick | `staleness_quarantine_broadcast`; `elai dkg staleness-tick`; Knowledge/Staleness drawer; `commands_f5`; `ELAI_STALENESS_CASCADE_KILL`; capacity 256 |
| F-6 | `SkillTrustTier`; `LearnedSkill`; `promote`; `permits_autonomous`; `staleness_score`; `demote_if_stale`; `acquire_learned_skill`; `skill_permits_autonomous`; `SkillLibraryEvent`/Kind/payload; skill CLI parser; skill-library Tauri command | `TemplateCrystallizer`; `CompiledPipeline`; reused `MiniCriticVerdict`; `run_best_of_n`; `should_auto_advance`; task-runner autonomy decision; reranker `RetrievalResult`; skills CLI | `skill_library_event` with Acquired/TrustPromoted/DemotedStale; `skills-mgmt library list/show/promote/tick`; skill-library panel; `ELAI_SKILL_AUTONOMY_BARRIER_DISABLE`; forbidden `ELAI_SKILL_LIBRARY_ENABLE` |
| F-7 | shipped/as-built `laplace_utility`; `TemplateUtility`; template-ranking command; `TemplateUtilityRanked`/payload; `commands_f7::get_template_ranking`; `TemplateRankingDrawer`; deferred lattice encoder/obligation/event/status command | `CompiledExecutor::try_execute`; `ExecutionOutcome`; `CompiledPipeline`; `classify_task_for_template`; deferred formal-verification run/types and immutable-rule reads | `template_utility_ranked`; deferred `safety_lattice_verified`; `elai template-ranking`; proposed/deferred `elai formal-verify lattice`; default-on CBR switch |
| Wiring doctrine | `wiring_contract_check.py --run-trigger` as executed contract | definitions, callers, production BFS, live trigger, dashboard, golden/schema checks, auditor, memory writer | W-1…W-10; disable-only switches; deterministic-fixture/live-run split |

## Cross-source interactions

The table references already-counted atomic records and therefore introduces no duplicate atomic inventory.

| Interaction | Explicit relationship | Atomic references |
|---|---|---|
| F-1 ↔ F-2a | F-2a owns vulnerability rule content; F-1 owns the compound gate, secret axis, dispatcher wiring, and per-tool contracts. They co-ship and contend on the security seed/SSB edit window. | DF002-F1-002, DF002-F1-004, DF002-F2-001, DF002-F2-040 |
| F-1 ↔ F-2b/F-2c | The benchmark uses the real VibeGate as its static before/after enforcement point; F-2c adds four data-only rule files consumed by the existing gate. | DF002-F1-046, DF002-F2-023, DF002-F2C-018, DF002-F2C-023 |
| F-1 ↔ F-9/E-43 | F-1 reserves dependency and skill event kinds for later producers; those axes remain excluded from its fail-closed Tier-1 promise. | DF002-F1-025, DF002-F1-041, DF002-F1-046 |
| F-3 ↔ E-37 | F-3 converts the existing injection classifier’s verdict into taint and applies capability denial; it does not replace the classifier. | DF002-F3-011, DF002-F3-012, DF002-F3-043 |
| F-3 ↔ F-1/F-2 | F-3 restricts which tools the researcher may invoke; F-1/F-2 restrict candidate content on the write-gate path. The sources call these disjoint/additive surfaces. | DF002-F3-001, DF002-F3-015, DF002-F1-001, DF002-F2-014 |
| F-5 ↔ E-5/B-42/C-38 | F-5 consumes existing memory contradiction/decay, existing Cosmos status, and the real DKG lifecycle/sink rather than adding replacement engines. | DF002-F5-001, DF002-F5-004, DF002-F5-018, DF002-F5-020 |
| F-5 ↔ B-30 | Phase 1 can audit now, but the DKG population half is sequenced after B-30 schema stabilization. | DF002-F5-006, DF002-F5-007, DF002-F5-035 |
| F-5 ↔ F-6 | Both can share the scheduler tick, but their staleness stores and semantics are distinct; cross-demotion is future composition only. | DF002-F5-041, DF002-F6-036, DF002-F6-037 |
| F-6 ↔ BestOfN/C-25 | Crystallization remains the acquisition producer and an already-journaled MiniCritic Pass is the gate; no third critic is created. | DF002-F6-003, DF002-F6-004, DF002-F6-024, DF002-F6-028 |
| F-6 ↔ F-7 Half B | F-6 controls learned-skill acquisition/trust while F-7 changes template selection utility; using utility for trust promotion is explicitly future work. | DF002-F6-024, DF002-F6-029, DF002-F7-007, DF002-F7-048 |
| F-7 Half A ↔ E-53 | The lattice is an amendment to the shared formal-verification pipeline and remains honest/pending at the Z3 boundary. | DF002-F7-001, DF002-F7-026, DF002-F7-027, DF002-F7-028 |
| Every F unit ↔ doctrine | The per-unit wiring contract must be executed, default-on, production-reachable, journalled, dashboard-verified where visible, and schema-locked. | DF002-WD-006 through DF002-WD-021 |

## Inference ledger (not explicit implementation facts)

| Atomic ID | Evidence | Inference | Basis |
|---|---|---|---|
| DF002-I-001 | I | The reconciled F-1 baby-step path may inspect task-summary text rather than the literal bytes an `Edit`/`Write` will commit, so its claimed pre-disk content protection is likely narrower than the title suggests. | F1:L90-L96 versus F1:L109-L111 |
| DF002-I-002 | I | F-1 and F-2 can journal two representations of one code-vulnerability denial unless the unresolved co-ship decision chooses reuse or explicit correlation. | F1:L23-L26; F2:L68-L70, L268 |
| DF002-I-003 | I | F-3 capability filtering should logically run before any permitted write reaches F-1/F-2, but no assigned source defines a canonical ordering or transaction boundary across these gates. | F3:L58-L60; F1:L51; F2:L68 |
| DF002-I-004 | I | Because F-3 enforcement is conditional on an “active” frozen commitment, missing/bootstrap-lost commitment state may default to the older category gate rather than fail closed; the source never specifies this branch. | F3:L60, L103-L113 |
| DF002-I-005 | I | F-5’s broadcast counts appear to require consumer results after DKG/Cosmos work, although emission is described at the memory tick before those consumers; an aggregation/acknowledgement phase is therefore implied but unspecified. | F5:L60-L68 |
| DF002-I-006 | I | A queue drop followed by only a count, with no fact IDs or replay log specified, cannot by itself recover the discarded quarantine. | F5:L68, L226 |
| DF002-I-007 | I | F-6’s S4 rule makes human approval happen at promotion time, while sub-S4 execution routes to a human gate; after promotion, S4 execution is autonomous without a per-invocation approval. | F6:L139-L144, L161-L170 |
| DF002-I-008 | I | F-6’s reuse of an SSB verdict assumes the SSB row and crystallization outcome share a stable correlation key and persistence boundary; otherwise replay/restart can alter acquisition results. | F6:L189-L200, L473-L477 |
| DF002-I-009 | I | F-7’s block-ratio “down-weight” needs an unstated combination formula, so different implementations could produce different deterministic rankings from identical inputs. | F7:L56 |
| DF002-I-010 | I | The lattice’s pairwise contradiction check does not obviously prove absence of higher-order conflicts among three or more rules. | F7:L68 |
| DF002-I-011 | I | Storing file paths, rule IDs, taint sources, model IDs, and content-derived hashes in durable SSB/dashboard artifacts creates a potential metadata/privacy surface, although no assigned document provides redaction, access-control, retention, or consent rules. | F1:L53; F2:L68-L70; F3:L62; F5:L60; F6:L151-L153 |
| DF002-I-012 | I | The auxiliary doctrine’s later empirical claims are the strongest status evidence for F-1/F-3, but they do not independently establish the current workspace still matches that session’s state. | WD:L84-L108 |

## Ambiguity and unresolved-contract ledger

| Atomic ID | Evidence | Ambiguity / unresolved point | Exact source anchor |
|---|---|---|---|
| DF002-A-001 | A | Whether `vibesec` is a rule ID namespace, subdomain, or synonym for the `security` domain is not defined. | F1:L5, L23-L24, L47-L49 |
| DF002-A-002 | A | The reconciled F-1 source does not specify how literal candidate file bytes enter the live baby-step invocation after the actual-tool path was removed. | F1:L90-L96 |
| DF002-A-003 | A | Source truncation length/canonicalization for `source_blake3` is unspecified. | F1:L53 |
| DF002-A-004 | A | Seed-row sort order, serialization normalization, baseline generation, and trusted baseline storage are unspecified. | F1:L49; F2:L266 |
| DF002-A-005 | A | It is unclear how an SSB row can be durably emitted for seed-integrity failure if dispatcher/SSB startup itself is refused. | F1:L49 |
| DF002-A-006 | A | No numeric false-positive release threshold is supplied for F-1/F-2/F-2c. | F1:L61, L209; F2:L264; F2C:L240 |
| DF002-A-007 | A | Tier-2 cache keys, freshness/TTL, negative caching, fill failure, offline behavior, and retry policy are not defined. | F1:L61 |
| DF002-A-008 | A | The exact authoritative event/correlation choice between F-1 and F-2 code-vulnerability blocks is unresolved. | F2:L268 |
| DF002-A-009 | A | F-2’s exact AST/Grep predicates, parser behavior, and language aliases remain implementation-specific beyond examples. | F2:L59-L64 |
| DF002-A-010 | A | A WARN-level missing-authz match uses a `security_rule_blocked`-named event/surface, but whether WARN actually refuses disk mutation is not specified. | F2:L63-L70 |
| DF002-A-011 | A | F-2’s “post-BLOCK regeneration” count, retry limit, failed/censored sample treatment, and determinism contract are unspecified. | F2:L80 |
| DF002-A-012 | A | The predecessor does not define functional `pass@1`, despite the orientation summary including it. | F2:L78-L86 |
| DF002-A-013 | A | The exact live model is not pinned: the decision text permits Qwen 3B on one 3090 or 7B on 2×3090. | F2:L86 |
| DF002-A-014 | A | The benchmark artifact’s canonical format/location is inconsistent between a TOML/JSON glob and F-2c’s concrete JSON directory. | F2:L143-L145; F2C:L132 |
| DF002-A-015 | A | Four new F-2c files are said to contain variants/multiple `[[rules]]`, yet the required database total rises by only four; exact row cardinality needs resolution. | F2C:L60-L67, L177, L203-L205 |
| DF002-A-016 | A | F-2c does not define how the static benchmark labels false positives separately from false negatives/gate recall. | F2C:L73-L76, L240-L241 |
| DF002-A-017 | A | The shipped loader’s directory-enumeration behavior is explicitly unresolved. | F2C:L69, L220, L242 |
| DF002-A-018 | A | The offline/static `model_id` value and report schema are not pinned. | F2C:L73, L179 |
| DF002-A-019 | A | F-3 does not specify the algorithm that converts arbitrary task intent into the minimal requested category set. | F3:L53, L58 |
| DF002-A-020 | A | Commitment lifetime, nested/delegated task inheritance, reset, cancellation, and replay semantics are not defined. | F3:L52-L60 |
| DF002-A-021 | A | Missing/corrupt commitment behavior, bootstrap recovery, and cross-gate consistency between FSM and MCP are not defined. | F3:L60, L103-L113 |
| DF002-A-022 | A | `TaintSource`, span canonicalization/hash, taint propagation/clearing, and false-negative behavior are unspecified. | F3:L55 |
| DF002-A-023 | A | F-3 does not define ASR denominator, utility, legitimate-tool false-deny acceptance, or fixture-selection methodology. | F3:L64-L68, L166-L167, L209 |
| DF002-A-024 | A | A variable named `ELAI_CAMEL_TOOLFILTER_ENABLE` is called a disable-only kill switch; accepted values and default parsing are not defined. | F3:L112, L164 |
| DF002-A-025 | A | F-5 never fully defines the semantics/polarity of `concept_stale` values 0, 1, and 2. | F5:L17, L56, L64 |
| DF002-A-026 | A | The exact fact→DKG concept/edge and fact→Cosmos manifest join keys are not specified. | F5:L64-L66 |
| DF002-A-027 | A | Grounding-cache invalidation has no named producer, cache key, or implementation task. | F5:L22, L156, L220 |
| DF002-A-028 | A | Broadcast durability, ordering, deduplication, idempotency, retry, partial-consumer failure, and restart recovery are unspecified. | F5:L60-L72 |
| DF002-A-029 | A | Queue drop-oldest implementation over Tokio MPSC, which does not inherently expose drop-oldest, is unspecified. | F5:L68 |
| DF002-A-030 | A | Payload uses `tick_ms:u64`, while collection takes `now_ms:u128`; conversion/overflow behavior is absent. | F5:L60-L62 |
| DF002-A-031 | A | Behavior of queued/pending broadcasts while the kill switch toggles is unspecified. | F5:L68, L74 |
| DF002-A-032 | A | F-6 gives names but no actual numeric promotion, age, failure, lifetime, or stale thresholds. | F6:L137-L142, L218-L224 |
| DF002-A-033 | A | Learned-skill persistence schema, transactional acquisition, uniqueness, deletion, and restart semantics are absent. | F6:L132-L136, L189-L200 |
| DF002-A-034 | A | Invocation/failure counter concurrency and atomicity are not defined despite shared async state. | F6:L68-L74, L132-L142 |
| DF002-A-035 | A | The source does not specify how a `--human-grant` is authenticated, authorized, journalled, or protected against automation. | F6:L139-L142, L172-L176 |
| DF002-A-036 | A | The SSB lookup API, row ownership, replay policy, and exact superstep field on `ExecutionOutcome` are unresolved. | F6:L189-L200, L473-L477 |
| DF002-A-037 | A | Learned-skill index/store location and how a reranker result resolves to a compiled pipeline are unspecified. | F6:L202-L208 |
| DF002-A-038 | A | The live promote trigger contains placeholder `<skill_id>` and does not specify how a runnable seeded skill is obtained. | F6:L296-L299 |
| DF002-A-039 | A | F-7 does not define the exact mathematical operation used to apply block-ratio down-weighting. | F7:L56 |
| DF002-A-040 | A | Coverage’s definition, source events, observation horizon, censored-session handling, and block-event correlation are unspecified. | F7:L56-L60 |
| DF002-A-041 | A | Template identity/candidate enumeration and deterministic tie-breaking after equal utility are not defined. | F7:L58 |
| DF002-A-042 | A | Immutable security/finance class inventory, rule semantics, parameter domains, and conflict encoding are not defined. | F7:L68 |
| DF002-A-043 | A | Z3 timeout, crash, malformed-output, version pinning, sandboxing, and retry behavior are unspecified. | F7:L70-L76 |
| DF002-A-044 | A | The need for a 2×3090 RunPod window for a Z3 subprocess is asserted but technically unexplained. | F7:L192 |
| DF002-A-045 | A | No assigned source supplies privacy, redaction, retention, access-control, data-residency, consent, or regulatory-compliance requirements for the new SSB/dashboard artifacts. | All eight assigned sources, fully read |
| DF002-A-046 | A | The doctrine’s required “memory-writer ran” gate is not defined or mapped to a command/artifact. | WD:L79-L82 |
| DF002-A-047 | A | The doctrine gives no canonical BFS depth, though several unit plans independently require depth 8. | WD:L39-L42; F2:L190; F3:L163; F7:L185 |

## Conflict ledger

| Atomic ID | Evidence | Conflict | Exact anchors |
|---|---|---|---|
| DF002-C-001 | C | F-1 is labelled `PLANNED`, while its reconciled contract says its real symbols “actually shipped” and the doctrine calls it a built/fixed unit. | F1:L3, L65-L76; WD:L87-L95 |
| DF002-C-002 | C | F-1’s reconciled production chain rejects `conversation.rs:483`, but acceptance and the W-3 explanation still require the gate to fire through that path. | F1:L90-L96 versus L148, L162 |
| DF002-C-003 | C | F-1 says `VibeGateAxisVerdict`/`VibeGateAxisKind` never shipped, while its Tasks and deterministic check still require those exact symbols. | F1:L47, L65-L70, L119-L120 |
| DF002-C-004 | C | F-1’s early existing-state claim says the compound gate and immutable security rows do not exist, while its later reconciliation and doctrine assert shipped wiring. | F1:L13, L65-L76; WD:L92-L95 |
| DF002-C-005 | C | F-2’s reconciliation excludes all F-2b symbols from the done gate, but its observable effect, deterministic checks, W-3/W-4 text, and acceptance still demand the bench/event. | F2:L90-L114 versus L141-L182, L190-L199, L209-L215 |
| DF002-C-006 | C | F-2’s predecessor marks F-2b deferred, while F-2c explicitly asserts its runner and event schema are shipped. | F2:L90-L114 versus F2C:L3-L12, L35 |
| DF002-C-007 | C | F-2 says F-2a and F-1 might emit separate rows for the same block and leaves reuse unresolved; no authoritative schema choice exists. | F2:L13, L268 |
| DF002-C-008 | C | F-2c repeatedly fixes the row total at 16, yet its rule descriptions require Python/TypeScript variants and mixed severities in files described as containing multiple rows. | F2C:L60-L67, L177, L203-L205 |
| DF002-C-009 | C | F-6’s contract produces `SkillsPanel.svelte`/`commands_f6.rs`, while Tasks/files/checks name `SkillLibraryPanel.svelte` and alternately `commands.rs`. | F6:L229-L236, L250-L266, L293-L295, L329-L332, L423-L446 |
| DF002-C-010 | C | F-6’s produced-symbol contract locates `acquire_learned_skill` in `task_runner.rs`, while Tasks, checks, and file edits place it in `compiled_pipeline.rs`. | F6:L189-L200, L252-L260, L307-L311, L433-L434 |
| DF002-C-011 | C | F-7’s reconciled contract intentionally defers every Half-A symbol, while deterministic checks, acceptance, files-touched, and later W-clauses require those symbols and live row now. | F7:L97-L107, L125-L143 versus L156-L180, L187-L195, L204-L224, L248-L256 |
| DF002-C-012 | C | F-7’s reconciled UI is a drawer plus `commands_f7.rs`, while older tasks/checks/files specify a MonitorWindow `TemplateRankingPanel.svelte` plus `commands.rs`. | F7:L64, L78, L95-L96, L136-L137 versus L172-L174, L187, L209, L223-L224 |
| DF002-C-013 | C | F-7’s reconciled observable is explicitly Half-B-only, while its final W-1/W-4/W-5 prose requires both ranking and lattice commands/events. | F7:L138-L143 versus L248-L252 |
| DF002-C-014 | C | F-5’s source corrects quarantine population to `concept_stale=2`, while the permitted orientation summary records quarantine as setting it to `1`. | F5:L64, L153-L158 versus summary L716 |
| DF002-C-015 | C | The doctrine asserts a repaired F-3 live ASR of `1.0→0.0`, while F-3 itself treats ELAI ASR as a planned measurement and live-model ASR as deferred. | WD:L96-L101 versus F3:L64-L68, L166-L167 |

## Additions and corrections to `06-F-SUMMARY_INDEX.discovery.md`

`ADD` means explicit source detail absent from the orientation inventory. `CORRECT` means the deep source narrows, supersedes, or contradicts an orientation statement. `STATUS` means the deep source supplies implementation-state evidence the summary could not settle.

| Atomic ID | Delta | Addition or correction | Deep anchor | Orientation anchor |
|---|---|---|---|---|
| DF002-SD-001 | STATUS | F-1 is simultaneously header-`PLANNED`, later reconciled as shipped, and described by the doctrine as built/fixed; the summary’s “not currently wired” classification is not a sufficient current-status ledger. | F1:L3, L65-L76; WD:L92-L95 | SUM:L40, L57 |
| DF002-SD-002 | CORRECT | F-1’s as-built production chain is runtime setup→task-runner wiring→`dispatch_pre_tool_use` on the baby-step pre-gate, not the summary’s `hooks.rs run_pre_tool_use` chain. | F1:L90-L96 | SUM:L56, L72 |
| DF002-SD-003 | CORRECT | `conversation.rs:483` is explicitly called a hallucinated/other path and actual-tool byte gating is deferred; summary claims that an Edit/Write refusal lands before disk without this caveat. | F1:L90-L96, L148 | SUM:L75-L80 |
| DF002-SD-004 | CORRECT | Immutable security rows are seeded/checked through project-rule/task-runner setup, not `schema.rs` as the summary states. | F1:L49, L94-L96 | SUM:L73 |
| DF002-SD-005 | ADD | F-1’s exact secret shapes are AWS `AKIA…`, private-key headers, and secret-named quoted assignments of length ≥8. | F1:L47 | SUM:L59, L86 |
| DF002-SD-006 | ADD | Axis initialization failure panics at startup; no per-write unwrap/expect or silent clean fallback is allowed. | F1:L47, L102-L103 | SUM:L86 |
| DF002-SD-007 | ADD | Seed integrity hashes canonical `(rule_id,predicate,severity)` tuples against a compiled constant and refuses dispatcher startup on mismatch. | F1:L49 | SUM:L66, L86 |
| DF002-SD-008 | ADD | F-1 uses one four-arm Pattern-2 payload with two live and two reserved action strings; exact payload fields and truncated-source BLAKE3 are now known. | F1:L53 | SUM:L77-L78, L86 |
| DF002-SD-009 | ADD | The gate is default-on behind disable-only `ELAI_VIBEGATE_DISABLED`; `ELAI_VIBEGATE_ENABLE` is forbidden. | F1:L57, L100 | SUM:L26, L40-L86 |
| DF002-SD-010 | ADD | Tier-2 registry lookup is plan-start cached and skill/MCP scanning is per-session, with explicitly fail-open documentation. | F1:L61 | SUM:L67-L71 |
| DF002-SD-011 | ADD | F-1 release gating includes at least three real runs, ten benign snippets, per-tool SQLi+AWS inputs, and no model/RunPod/performance claim. | F1:L173-L176, L204-L214 | SUM:L62-L84 |
| DF002-SD-012 | STATUS | F-2’s reconciled contract explicitly says F-2a shipped and F-2b deferred, while F-2c later asserts both code paths shipped; the summary leaves the predecessor status open. | F2:L90-L114; F2C:L3-L12 | SUM:L90, L402 |
| DF002-SD-013 | CORRECT | Missing-authorization is WARN, not BLOCK; only the high-precision SQLi/XSS/weak-crypto classes are stated to block. | F2:L59-L64 | SUM:L97-L99 |
| DF002-SD-014 | CORRECT | The real seed path is `crates/rules/seed/security/`, not `crates/rules/rules/security/`. | F2:L213, L259 | SUM:L114, L126 |
| DF002-SD-015 | CORRECT | The real benchmark route is `elai bench seccodeplt` through `bench_cmd.rs`/per-bench runner, not `benchmark.rs` or `elai benchmark --suite`. | F2:L80, L213, L259 | SUM:L111, L116, L120, L126 |
| DF002-SD-016 | CORRECT | The deep F-2 benchmark payload/report does not specify functional `pass@1`; the summary’s pass@1 requirement is unsupported by this assigned original. | F2:L78-L86 | SUM:L112-L113 |
| DF002-SD-017 | ADD | F-2 defines before/after as raw vulnerable-output rate versus residual after block/regeneration on an ELAI-owned ≥40-fixture corpus. | F2:L78-L82 | SUM:L99-L124, L126 |
| DF002-SD-018 | ADD | Exact block and measurement payload fields, fail-closed seed-tamper behavior, sub-1-ms no-I/O target, and default-on/no-bypass rules are supplied. | F2:L66-L70, L80-L82, L132-L138 | SUM:L92-L126 |
| DF002-SD-019 | ADD | Optional constrained decoding is conditional on an insufficient after-rate and explicitly shifts generation without a fail-closed guarantee. | F2:L84 | SUM:L108-L124 |
| DF002-SD-020 | ADD | F-2 explicitly prohibits VibeSec branding and leaves duplicate-event resolution with F-1 open. | F2:L268-L270 | SUM:L126 |
| DF002-SD-021 | ADD | F-2c supplies exact low-FP guards and severities for CWE-798/321/532/95, including literal-vs-dynamic eval and WARN-only `print(secret)`. | F2C:L60-L65, L203 | SUM:L406-L413, L442 |
| DF002-SD-022 | ADD | F-2c defines `gate_recall=(before-after)/before`, says schema version 201 already ships, and pins the report directory to `benchmarks/seccodeplt/`. | F2C:L35, L102, L132 | SUM:L414-L440, L442 |
| DF002-SD-023 | CORRECT | The summary conflates ast-grep deferral with an `ast_adapter::new()` edit; the original says multi-language grammar registration needs that edit, while ast-grep is separately deferred as an unnecessary new DSL dependency. | F2C:L27-L30, L76 | SUM:L429-L430 |
| DF002-SD-024 | ADD | The exact ten registered languages and `.py/.ts/.tsx` versus `.svelte/.html/.css` extension behavior are supplied. | F2C:L27-L28, L36, L229 | SUM:L421-L425 |
| DF002-SD-025 | ADD | The purported 12→16 row update conflicts with multi-row/language variants, and loader enumeration remains explicitly unresolved. | F2C:L60-L69, L242 | SUM:L431-L432, L442 |
| DF002-SD-026 | CORRECT | F-3 calls full CaMeL `TIER 3`, not Phase/Tier 2. | F3:L3-L4, L72-L74 | SUM:L146-L150 |
| DF002-SD-027 | ADD | `CommittedToolSet`’s complete schema, sorted BLAKE3 commitment, set intersection, frozen-state semantics, and unknown-tool denial are supplied. | F3:L51-L56, L60 | SUM:L140-L159 |
| DF002-SD-028 | ADD | F-3 reuses `tool_denied`, adds one two-kind `ToolFilterEvent`, and explicitly forbids producerless `capability_flow_blocked`. | F3:L60-L62 | SUM:L151-L159, L695 |
| DF002-SD-029 | ADD | F-3’s real deterministic command, 20-fixture minimum, JSON evidence, all-20 denial gate, and separate live-model checkbox are supplied. | F3:L64-L68, L117-L120, L166-L168 | SUM:L143-L159 |
| DF002-SD-030 | ADD | The existing frontend pair is named precisely (`ToolGateInspector`/`commands_c35`) and displays commitment, quarantine, and ASR. | F3:L66, L114-L116 | SUM:L153-L159, L695 |
| DF002-SD-031 | ADD | The source corrects `prompt_injection.rs` to `prompt_armor.rs`, and the doctrine adds post-fix live evidence `ASR 1.0→0.0`. | F3:L26, L204; WD:L96-L101 | SUM:L128-L159 |
| DF002-SD-032 | CORRECT | F-5 quarantine writes `concept_stale=2`, not the summary’s `1`; state 0 remains included and state 2 demotes rather than vanishes. | F5:L56, L64 | SUM:L222, L716 |
| DF002-SD-033 | CORRECT | Grounding-cache miss is an acceptance hypothesis contingent on Phase-1 cache-key verification; no concrete grounding-cache consumer task is specified. | F5:L156, L220 | SUM:L209, L223, L620-L621 |
| DF002-SD-034 | ADD | Queue capacity is exactly 256 and overflow is drop-oldest with next-broadcast `dropped_count`, not merely “bounded.” | F5:L68 | SUM:L216-L218, L227 |
| DF002-SD-035 | ADD | The complete broadcast payload, BLAKE3 ID derivation, two emission paths, default-on kill switch, and single schema variant are supplied. | F5:L60-L74 | SUM:L219-L225, L227 |
| DF002-SD-036 | ADD | The background scheduler is currently wire-dark; the inline live tick is mandatory, and the SHARD >2 threshold remains an unconfirmed design choice. | F5:L15, L220, L227 | SUM:L195-L227 |
| DF002-SD-037 | CORRECT | F-6’s curriculum→generation→execution→retry sequence is Voyager context, not an implementation task added by F-6; the actual acquisition hook begins at existing crystallization. | F6:L40-L49, L187-L214 | SUM:L235-L240, L270 |
| DF002-SD-038 | ADD | Exact skill-tier enum names, LearnedSkill fields, S4-only autonomy, named-but-unset thresholds, and one-tier staleness formula are supplied. | F6:L127-L146, L218-L224 | SUM:L241-L268, L270 |
| DF002-SD-039 | CORRECT | Promotion/demotion are Kind values under one `skill_library_event`, not necessarily separate SSB action types named `skill_trust_promoted` and `skill_demoted_stale`. | F6:L148-L159 | SUM:L266-L268, L698-L699 |
| DF002-SD-040 | CORRECT | The original live done-trigger is promote+tick with a placeholder skill ID; the summary’s attempted autonomous S2 dispatch is not the source’s live observable command. | F6:L296-L299 | SUM:L264-L265 |
| DF002-SD-041 | ADD | Acquisition is fail-closed on Warn/Block/absent verdict, correlates by agent/superstep, and S≥S3 is required for retrieval candidacy. | F6:L189-L214 | SUM:L254-L270 |
| DF002-SD-042 | ADD | The source contains unresolved implementation-path conflicts for panel/Tauri names and the location of `acquire_learned_skill`. | F6:L229-L266, L293-L332, L423-L446 | SUM:L229-L270 |
| DF002-SD-043 | STATUS | F-7’s reconciled contract says CBR Half B is shipped/as-built and lattice Half A intentionally deferred; the summary treats both as planned deliverables without this as-built split. | F7:L86-L107, L121-L143 | SUM:L274-L301 |
| DF002-SD-044 | CORRECT | The deep plan’s lattice command is `elai formal-verify lattice`, not `elai verify safety-lattice`. | F7:L72 | SUM:L289 |
| DF002-SD-045 | CORRECT | “Zero conflict pairs” is not the defined CLI success artifact in the assigned original; it reports outcome/hash/class/rule counts. | F7:L72-L76 | SUM:L290 |
| DF002-SD-046 | CORRECT | The assigned original does not define `P@k` or `k`; it requires an unspecified measured CBR-vs-first-try delta after ≥100 labelled sessions. | F7:L23, L82, L190, L238 | SUM:L298-L301, L640 |
| DF002-SD-047 | ADD | Half-B’s exact Laplace base formula, 1/1 priors, block-ratio input, clamp, retained baseline, payload, CLI, and as-built drawer are supplied. | F7:L54-L64, L86-L96, L136-L143 | SUM:L292-L301, L303 |
| DF002-SD-048 | ADD | Half-A defines complete/monotone/conflict-free SMT assertions, honest `RunpodPending`, obligation payload, and complete/incomplete fixture behavior. | F7:L66-L78 | SUM:L278-L291, L303 |
| DF002-SD-049 | ADD | The F-7 file internally conflicts because older checks and W prose still demand deferred Half A and older UI paths. | F7:L97-L107, L125-L143 versus L156-L224, L248-L256 | SUM:L272-L303 |
| DF002-SD-050 | ADD | The doctrine adds historical audit counts, the six-part done gate’s `memory-writer` requirement, and concrete live-path failures/fixes for F-1, F-3, and F-9. | WD:L9-L24, L79-L108 | SUM:L13-L35, L660-L684 |

## Requested-dimension coverage matrix

| Dimension | Explicit coverage | Principal atomic ranges | Explicit absence / unresolved coverage |
|---|---|---|---|
| Capabilities, actions, surfaces | Complete for all seven plan files plus doctrine | F1-001…046; F2-001…042; F2C-001…042; F3-001…044; F5-001…042; F6-001…051; F7-001…048; WD-001…031 | None omitted |
| Workflows and states | Gate paths, tier/phase order, FSM stop/advance, trust ladder, quarantine tick, proof outcomes | F1-017…020; F2-014…015; F3-002…016; F5-006…024; F6-002, 009…015, 024…036; F7-007, 020…035 | Ambiguities A-002, A-020…021, A-025, A-041…043 |
| Schemas, data, lifecycle | Every named payload and core record schema; lifecycle state transitions | F1-023…028; F2-016…027; F2C-002…025; F3-005…021; F5-008…022; F6-006…020, 033…036; F7-006, 010…011, 020…032 | Persistence/idempotency gaps A-004, A-028, A-033…036 |
| Flags, defaults, config | All named kill switches, forbidden enable flags, capacities, priors, thresholds, schema-bump rules | F1-021…022; F2-018; F3-031; F5-021…023; F6-009…015; F7-005, 040, 046; WD-011 | Threshold/value gaps A-006…007, A-024, A-032 |
| Integrations and events | Exact registry above plus per-source rows | Identifier registry; cross-source interactions | Event/correlation ambiguities A-008, A-021, A-036, A-040 |
| Permissions/security | Immutable rules, prompt/LLM bypass bans, least privilege, human autonomy gates, fail-closed behavior | F1-009…014; F2-008…018; F3-002…016; F6-006…015; F7-020…029; WD-011…012, 020 | Human-grant authorization A-035 |
| Privacy/compliance | No explicit requirement found | — | A-045; only content hashes/metadata are specified, without privacy policy |
| Failure/fallback/retry/recovery | Startup panic/refusal, reserved/deferred axes, unknown denial, queue overflow, absent-verdict refusal, RunPodPending | F1-010, 014, 040…045; F2-013, 028, 038…040; F3-009, 020, 028, 040…042; F5-022, 035…039; F6-026, 047…049; F7-026…029, 035, 043…045 | Recovery/retry gaps A-005, A-007, A-011, A-021, A-028…031, A-043 |
| Determinism/idempotency/concurrency | BLAKE3 commitments, fixed seed, deterministic fixture paths, bounded async queue, async shared crystallizer | F1-010…014, 027; F2-012…013, 023…025; F2C-023…026, 035…036; F3-005…010; F5-021…024; F6-024…028, 033…036, 050; F7-003…008, 024…029 | Idempotency/concurrency gaps A-020…021, A-028…034, A-041 |
| Observability/evidence | Every event, artifact, CLI trigger, dashboard surface, golden/schema requirement | F1-023…039; F2-016…037; F2C-025…037; F3-013…035; F5-008…032; F6-017…044; F7-009…016, 030…039; WD-007…021 | Metadata/privacy gap A-045 |
| Tests/acceptance/done | All named contract categories and global W-1…W-10 | Per-source `E-G` rows; WD-007…021 | Runnable placeholder issue A-038; memory-writer ambiguity A-046 |
| Constraints/non-goals | All explicit defer/no-new-dep/no-network/no-performance/no-branding boundaries | F1-003, 025, 041…046; F2-020, 028…032, 041…042; F2C-027…032, 041…042; F3-020, 028, 036…044; F5-020, 032…041; F6-032, 045…051; F7-017…019, 026, 035, 037, 042…048 | None omitted |
| Dependencies/status/deferred | Prerequisites, as-built reconciliation, pending/live splits, future composition | Status boundaries; F1-003/046; F2-001…003/020/029/034; F2C-002…009/029…032/042; F3-036…039; F5-035/041; F6-002/032/045/049; F7-001/017…019/026/035/037…038/048 | Conflicts C-001…015 |

## Coverage, zero-omission, and duplicate audit

- **Manifest coverage:** 8/8 assigned files processed completely; 2,097/2,097 logical lines; 292,736/292,736 bytes; 8/8 hashes exact. Seven originals and one auxiliary doctrine source are represented by dedicated ledgers.
- **Orientation isolation:** the 1,022-line summary discovery was used only for the 50-row delta ledger. No statement was upgraded to source evidence solely because it appeared there.
- **Source endings observed:** `End of F-1.` (F1:L220), `End of F-2.` (F2:L276), `End of F-2c.` (F2C:L253), `End of F-5_staleness_quarantine_cascade.md.` (F5:L235), `End of F-7.` (F7:L263), and `End of F-WIRING-DOCTRINE-WHY.md.` (WD:L120). F-3 ends at its Next paragraph (L232) and F-6 at its Next paragraph (L498), with no explicit end marker.
- **Explicit-ledger counts:** 346 explicit source rows: F1 46; F2 42; F2C 42; F3 44; F5 42; F6 51; F7 48; WD 31. The separated registers add 12 inferences, 47 ambiguities, 15 conflicts, and 50 summary deltas, for 470 atomic rows total.
- **Separated epistemics:** explicit rows occur only in per-source ledgers; inferences are only `DF002-I-*`; ambiguities only `DF002-A-*`; conflicts only `DF002-C-*`; summary deltas only `DF002-SD-*`.
- **Deduplication rule:** task/acceptance/test/W-clause repetitions were collapsed by semantic key. The cross-source interaction table and exact identifier registry reference existing atomic IDs and are deliberately excluded from the atomic count.
- **Zero-omission audit:** all requested dimensions are mapped in the matrix above; every named event/action string, command, environment flag, enum/state, payload, dashboard/Tauri pair, fixture/live-run split, prerequisite, explicit defer, risk, failure policy, and completion gate found in the assigned sources has either an explicit atomic row or an epistemically separated `I/A/C` row.
- **Duplicate-ID audit:** all 470 row IDs are unique and every per-prefix numeric sequence is contiguous from `001` through its stated terminal value; duplicate count is zero.

## Output digest

The payload digest is defined over every byte before the `## Output digest` heading (including its preceding newline), so it can be embedded without a self-referential whole-file hash. The final whole-file SHA-256 is reported to the compiling agent after the last write.

- Payload SHA-256: `9e55570af49bfe8b3ea69dfa878264cf0f8f7a9613cb212b2fedd4b077054f1f`
- Output payload lines before digest heading: `639`
- Unique atomic rows: `470`
- Duplicate atomic IDs: `0`

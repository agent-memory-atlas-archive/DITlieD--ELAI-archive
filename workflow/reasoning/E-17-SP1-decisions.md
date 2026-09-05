E-17 SP1 task 1 -- decisions log
Session: s-26556
Date: 2026-05-21
FSM: EXECUTING -> AUDIT-ready

Scope shipped
- `elai doctor knowledge-models [--project-root PATH] [--output-format text|json|toml]` subcommand.
- 9-entry manifest probes all ONNX models + tokenizers staged under `core::paths::models_dir_path(project_root)`:
  cross-encoder-l12, cross-encoder-l6, graphcodebert (+ tok), granite-embed (+ tok), shared cross-encoder tokenizer, splade model (+ tok).
- Text output: ASCII table with label / status / size_bytes / blake3-16-prefix / absolute path; one row per entry; trailing `! <error>` line on failures.
- JSON output: object with `kind`, `project_root`, `models_dir`, `has_failures`, `entries[]`.
- Exit 0 when every entry is present + non-zero size + hash computed; non-zero (anyhow::anyhow!) listing missing entries otherwise.

Files modified
- crates/cli/src/args.rs: added `CliAction::DoctorKnowledgeModels { project_root, output_format }` next to `Doctor`; routed `"doctor"` subcommand through new `parse_doctor_subcommand_args`; bare `"doctor"` still falls through to existing `CliAction::Doctor` for backwards compat with the single-word command alias.
- crates/cli/src/main.rs: added dispatch arm `CliAction::DoctorKnowledgeModels => doctor::run_knowledge_models_probe(...)`.
- crates/cli/src/doctor.rs: added `KnowledgeModelEntry`, `KNOWLEDGE_MODEL_MANIFEST`, `probe_knowledge_entry`, `read_head_bytes`, `run_knowledge_models_probe`, `render_knowledge_models_text` (~250 LOC). Imported `core::paths::models_dir_path` as `core_models_dir_path` and `std::path::PathBuf`.
- crates/cli/Cargo.toml: added `blake3 = "1.8"` (matches workspace pin and the version already used in core + knowledge per CLAUDE.md anti-stale rule).

Files created
- crates/cli/tests/doctor_knowledge_models_integration.rs (~165 LOC). 3 tests, all PASS:
  * happy_path_all_present_exits_zero
  * missing_file_returns_err
  * json_output_contract

Key decisions
1. **Hashing policy**: tokenizer.json files hash full content (small); ONNX model files hash first 256 bytes only via `read_head_bytes`. This matches the context-package guidance (anti-read-big: never hash 500 MB files in a doctor probe). Blake3 only; never blake2/sha2 (CLAUDE.md anti-stale).
2. **Variant shape**: flat `CliAction::DoctorKnowledgeModels` sibling of `CliAction::Doctor`. Matches the `SymbolGraphBuild` / `SymbolGraphStats` pattern (separate variant per subcommand) rather than nesting a `DoctorSubcommand` enum. Lower blast radius, no parser refactor.
3. **Subcommand routing**: added a fresh `"doctor"` arm to the main `parse_args` match block. The bare-slash single-word path at args.rs:1438 still returns `CliAction::Doctor { output_format }` so existing callers of `elai doctor` (no sub-subcommand) keep working byte-for-byte.
4. **Project root default**: `--project-root` defaults to `std::env::current_dir()` (mirrors the `run` and `symbol_graph` subcommands). The user-facing footgun discovered during verify (passing `F:/.ELAI_workflow/.elai_cc` instead of `F:/.ELAI_workflow` produces a double-`.elai_cc` path) is INTENTIONAL — it reflects how `models_dir_path` composes; the doctor surfaces the resolved absolute `models_dir` in both text and JSON output so the user can spot the mistake immediately.
5. **NO edit to `RerankerConfig::default()` or `SpladeConfig::default()`**: per executor brief and per context-compiler MEDIUM warning, 6+ test sites depend on the relative-path defaults (pipeline.rs:1079, startup.rs:378/525/646 read it verbatim). The proper fix lives in SP2 (replace `Default` with `with_project_root(&Path)` constructor + update the 6 call sites to thread project_root). SP1 ships the doctor probe only.
6. **No SSB emissions in SP1**: doctor probe is a one-shot CLI surface; SSB action types for runtime knowledge-degradation events are SP4 work.

Observations for SP2 executor
- RerankerConfig::default() (crates/knowledge/src/reranker.rs:78-92) hardcodes
  `model_path: PathBuf::from("models/onnx/cross-encoder-l12.onnx")` and
  `tokenizer_path: PathBuf::from("models/onnx/tokenizer.json")` — both RELATIVE to CWD.
  Production callers override via `core::paths::models_dir_path` at live_cli.rs:1327-1337, but tests at:
    crates/knowledge/src/pipeline.rs:1079 (no override)
    crates/knowledge/src/startup.rs:378  (no override)
    crates/knowledge/src/startup.rs:525  (no override)
    crates/knowledge/src/startup.rs:646  (no override)
  use the default verbatim. SP2 must either (a) thread project_root into a new `with_project_root` constructor and update all 6 sites, or (b) keep Default + introduce a `KNOWLEDGE_MODELS_ONNX_SUBDIR` const and rely on tests setting CWD.
- SpladeConfig::default() (crates/knowledge/src/splade.rs:92-101) has the SAME relative-path bug:
  `model_path: PathBuf::from("models/splade/model.onnx")`
  `tokenizer_path: PathBuf::from("models/splade/tokenizer.json")`
  Same SP2 scope question.
- HIGH (out-of-scope for SP1, surfaces during SP3 reranker swap): live_cli.rs:1335 sets fallback to `"ms-marco-l6.onnx"` but the on-disk file is `"cross-encoder-l6.onnx"`. The fallback path would never resolve. Doctor probe at this commit reports `cross-encoder-l6` as OK because the manifest looks for the real filename — the bug is in live_cli.rs, not the on-disk staging.
- LOW (orchestrator co-conspirator, out of E-17 scope): crates/orchestrator/src/media_validator/clap.rs:90 uses the same `PathBuf::from("models/clap-htsat-unfused")` relative-path antipattern. Tracked but not touched.

Verify steps executed
1. `cargo check -p elai-cli` -> exit 0 (warnings only, all pre-existing in upstream crates).
2. `cargo test -p elai-cli --test doctor_knowledge_models_integration` -> 3/3 PASS.
3. `./target/debug/elai doctor knowledge-models --project-root F:/.ELAI_workflow` -> exit 0; all 9 entries OK on this workspace.
4. `./target/debug/elai doctor knowledge-models --output-format json --project-root F:/.ELAI_workflow` -> valid JSON, exit 0.
5. `grep .unwrap() || .expect(` on doctor.rs new code -> 0 hits in production paths.
6. `grep blake2|sha2` on new code -> 0 hits.

Anti-spiral status: no file edited >3 times. Linear single-cycle execution.

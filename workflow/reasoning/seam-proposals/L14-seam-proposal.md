# L-14 seam proposal — Task 3, the L-11 output-quality gate over generated code (K-27 design turn, doc-only)

Task: opuswalk-L-14-PLAN-1787792316-e57a38 (wave PLAN, role PLAN, idem c820cae627ccb83ca40f70109a17a751).
Author: dispatched design worker under the agent-dispatch preamble (R-1..R-8) and the synthesizer-elai
role constraint (aggregate, do not implement). This doc does NOT self-certify DONE — the off-agent
verifier and the opus controller own the verdict. No product code was edited this turn; git was touched
only with read-only ops (`git log`, positional greps); writes are confined to this file per
allowed_write_set. All file:line below were re-verified TODAY, 2026-08-27, against the live tree at
`/home/user/teikoku/elai`, branch `main`, HEAD `99e2fa98` (2026-08-23, "Save pending ELAI orchestrator,
routing, and knowledge-stack edits."). Claims carried from prior recon rather than re-read are labeled
as such.

## 1. Queue reason + gate_json_ref

Queue entry (opus walk plan teikoku-opus-walk, queued unit L-14 RESIDUAL, Task 3):

> reason: Task 3, the L-11 output-quality gate over generated code, has NO DESIGNED SEAM — recon proved
> no production function takes a directory or file set and returns a quality verdict;
> MiniCritic::check judges ONE fill-body string and evaluate_vibegate judges ONE tool-use string, while
> solid_proxy_check.py / maintainability_check.py are CLI scripts over elai.map rows, not Rust-callable;
> build_cmd.rs ships zero quality gating; nothing fails today if the generated app is garbage. K-27
> blocks inventing the seam.

> gate_json_ref: conductor/opus_walk/evidence/L-14/S2-tasks346-recon.md (exists, verified today;
> BOTTOM LINE names Task 3 DARK and asks for "a directory-level quality gate function + a call from
> build_cmd.rs::run before the RRA receipt is emitted")

Supporting decision record: conductor/opus_walk/evidence/L-14/L-14-decisions.md D-3 ("Task 3 is DARK
because no directory-level quality gate exists ... K-27 forbids executing an undesigned seam, so task 3
goes to the escalation queue for a design turn"). This turn supplies that design.

## 2. What the plan demands of this gate

docs/thefinalglance/L-plan/L-14_one_shot_verified_builder_front_door.md:

- Line 31: "quality through L-11 (~20 LOC): the generated code passes the L-11 output-quality gate
  (SOLID-proxy + maintainability + J-23 generated-lane) before the build is reported DONE; ... A build
  that fails the quality gate is reported with the findings, not silently shipped."
- Line 43/45: the L-11 gate sits inside the PRODUCTION one-shot chain
  `CLI > elai build > pipeline factory > E-12 codegen > L-11 quality gate > I-40 wrap`.
- Line 52 (stub_ban): "NO generated app reported DONE that failed the L-11 quality gate (silently)".

Two hard requirements fall out: (a) the gate MUST ride the real `elai build` generating arm, before
anything is reported DONE, and (b) failures must be REPORTED with findings — never silent, never a
bare refusal.

## 3. Verified seams on current main (each pinned by direct read/grep today)

Path prefix omitted: all paths relative to `/home/user/teikoku/elai`.

### 3.1 The production hook this gate rides — `elai build` generating arm

`.elai_cc/crates/cli/src/build_cmd.rs`, current line map (read in full today):

| line | content |
| --- | --- |
| :60 | `pub(crate) fn run(idea, plan_only, output_dir, template, output_format)` — real dispatch target |
| :73 | `let app_dir = output_dir.ok_or_else(...)` — generating arm requires `--out <dir>` |
| :74-77 | `ELAI_WRAP_RRA_DISABLE` fail-closed kill switch (existing early-refusal precedent) |
| :80 | `template.unwrap_or("vanilla")` |
| :82 | `apply_template(template_name, app_dir, &vars)` — THE GENERATION STEP |
| :83 | `receipt_path = app_dir.with_extension("rra")` |
| :84 | `emit_build_rra(&project_root, &receipt_path)` |
| :85-86 | receipt-failure cleanup: `remove_dir_all(app_dir)` + non-zero exit, preserving FS-3 |
| :88-92 | `BuildResult` printed as JSON (`serde_json`) or text |

Dispatch into `run`: `.elai_cc/crates/cli/src/main.rs:759`
(`CliAction::Build { .. } => build_cmd::run(...)`), variant declared at
`.elai_cc/crates/cli/src/args.rs:76`. The gate's slot is BETWEEN :82 and :83 — after the app exists,
before the receipt exists — mirroring the :85-86 cleanup shape if RED.

### 3.2 The Rust-callable judge that ALREADY implements the shipped L-11 axes

`.elai_cc/crates/orchestrator/src/vibegate.rs:141`:

```rust
pub fn evaluate_vibegate(tool_name: &str, source: &str, file_path: &str) -> Vec<HookFinding>
```

Verified properties:

- Infallible and internally FAIL-CLOSED by its own contract (module doc at :133-139: internal errors
  convert to Block findings "so the gate can never be bypassed by an unexpected fault").
- Three axes (:145-150): secret scan (`run_secret_axis`, :171 regex engine in private
  `vibegate_secrets` module, exported surface cited at `.elai_cc/crates/orchestrator/src/lib.rs:365`),
  code-vuln CWE rules (`run_code_vuln_axis`), and `run_maintainability_floor_axis` (the L-11
  maintainability floor: size warn over `MAX_FILE_LINES = 1000` at vibegate.rs:60; block/warn token
  rules for typescript `as any`/`@ts-ignore` and rust `unsafe`; languages classified at
  `detect_language_label`, vibegate.rs, returning rust/python/typescript/javascript/unknown).
- It IS the tested home of the L-11 brand today:
  `.elai_cc/crates/contracts/tests/l11_output_quality_contract.rs` exercises these axes directly
  (:136 over-size candidate, :160 ts `as any`, :173 rust dead-code softening). The name "L-11 output-
  quality gate" is already bound to vibegate axes by that contract test — this design reuses that
  checker path rather than inventing a second one.

Production reachability of the judge FROM the cli crate is proven by existing wired callers (grep of
`crates/*/src` today): `.elai_cc/crates/cli/src/wrap_cmd.rs:184` and `:353` call
`evaluate_vibegate(...)` directly. Zero new dependency edges are needed.

Contrast with the other judge cited DARK in the queue reason: `MiniCritic::check(blank_id, body, lang)`
at `.elai_cc/crates/orchestrator/src/mini_critic.rs:171` judges ONE fill-body string and its only
production chain is skeleton-fill (`skeleton_generator.rs:295`, top caller
`crates/orchestrator/src/task_runner.rs:10980-10985` constructing
`MiniCritic::without_tree_sitter()` and calling `dispatch_skeleton_with_retry`). Note: the S2 recon
cited task_runner.rs:10801-10817 — current main has shifted those lines to ~10980; same symbols. The
one-shot builder does NOT route through skeleton fill: it drives `apply_template`, so MiniCritic is
prior art only, not part of this slice.

### 3.3 The artifact actually generated (what would be judged)

`.elai_cc/crates/scaffold/src/apply.rs:17` `pub fn apply_template(template_name, target_dir, vars)`
copies the template dir under the templates root (`templates_root()` resolves `<workspace>/templates/web`,
TEMPLATES_WEB_SUBDIR at apply.rs:11) substituting variables into text files (extension allowlist at
apply.rs:13-16). The default `vanilla` template today emits 10 files (verified by listing
`templates/web/vanilla/` today):
`index.html`, `main.js`, `style.css`, `README.md`, `.gitignore`, `assets/assets.json`,
`sections/{Hero,Features,Gallery,Cta,Footer}.html`.
So today's judged surface is HTML/CSS/JS/MD text; `main.js` classifies as `javascript`.

### 3.4 Legs deliberately OUTSIDE the minimal slice (named seams kept honest)

- SOLID-proxy leg: `.elai_cc/tools/solid_proxy_check.py` and `.claude/tools/maintainability_check.py`
  exist (verified today) and operate on elai.map rows as CLI scripts (input shape carried from the S2
  recon, not re-read this turn). A fresh vanilla app produces NO elai.map rows and no symbol graph, so
  this leg has nothing to consume until a map-generation step lands for one-shot apps. Deferred with
  its real seam named; NOT invented here.
- J-23 generated-lane leg: `elai review --catalog` is a catalog-of-external-linters runner
  (`.elai_cc/crates/arch_gate/src/lib.rs:125` `load_catalog`, `:179` `invoke_row`,
  `:172` `probe_tool` probing external binaries; CLI at
  `.elai_cc/crates/cli/src/review_cmd.rs:63-64`). It requires provisioned external toolchains on the
  user machine and probes them at runtime — wrong shape for a deterministic, dependency-free vertical
  slice. Deferred, seam named.

## 4. Gate semantics (design)

New function `run_quality_gate(app_dir: &Path) -> QualityGateReport`, recommended to live IN
`.elai_cc/crates/cli/src/build_cmd.rs` beside `emit_build_rra` (keeps the wiring deliverable one-file,
avoids a new cross-crate export; Option B below for later). Behavior:

1. ARTIFACT JUDGED: every regular file under `app_dir` after `apply_template` (build_cmd.rs:82),
   pre-receipt. Candidate filter: text extensions `{rs, py, pyi, ts, tsx, js, jsx}` — exactly the set
   `detect_language_label` discriminates; unknown extensions (html/css/md/json today) are counted as
   scanned-but-unjudged rather than failed (the axes themselves ignore `unknown`), so the slice never
   false-reds on vanilla's HTML/CSS.
2. JUDGE CALL PER FILE: `evaluate_vibegate("build", &source_text, &relative_path_as_label)` — reusing
   the exact three-axis floor the l11_output_quality_contract already freezes. Aggregation:
   any HookFinding with `HookVerdictKind::Block` over ANY file => gate RED; Warn-only findings =>
   GREEN-with-warnings.
3. GREEN => continue. `BuildResult` (build_cmd.rs:30-41) gains two serde fields:
   `quality_verdict: String` ("pass"|"warn") and `quality_warn_count: u32` (+ rule ids joined into the
   text printer at :91). This makes the wiring OBSERVABLE through the real entrypoint even when the
   clean vanilla template trivially passes — necessary for falsifiability (section 5), and it satisfies
   plan line 31's reporting duty for warnings too.
4. RED => mirror the FS-3 cleanup at :85-86 EXACTLY: `remove_dir_all(app_dir)`, return
   `Err(anyhow!("quality gate RED: {n} block finding(s): [rule_id rel-path, ...]"))` — findings are
   printed with rule ids and relative paths (plan line 52: reported with the findings, never silent),
   exit non-zero, NO receipt is ever emitted (ordering guarantees this because the slot precedes :84;
   the stub_ban cannot be violated even by a partially-written app — removal happens first).
5. IO fail-closed: any walk/read error inside `run_quality_gate` becomes a synthetic Block finding
   (never skipped, never passed), matching vibegate's own fail-closed doctrine (vibegate.rs:133-139).

RED/GREEN table:

| condition | outcome |
| --- | --- |
| zero Block findings, zero warns | `quality_verdict="pass"`, build continues, receipt emitted |
| zero Block, >=1 Warn | `quality_verdict="warn"` + warn count/rule ids printed, build continues |
| >=1 Block anywhere | app dir removed (FS-3 clone of :85-86), Err lists rule ids + rel paths, exit != 0, no receipt |
| unreadable/unwalkable entry | treated as Block (fail-closed) |

Excluded from RED on purpose: `unknown`-language files (no axis rules apply yet); over-long files stay
Warn per the frozen floor (`maintainability_floor_warns_overlong_candidate_without_blocking`,
l11_output_quality_contract.rs:126-127) — this design does not contradict a frozen contract.

## 5. Falsification shape (house style, decisions D-6/D-7)

- M1 WIRE MUTATION (falsifies the hook, per D-6 "target the PRODUCTION CALL SITE"): delete the
  `run_quality_gate(app_dir)` call inserted at the :82/:83 boundary. Tree still compiles; the CLI smoke
  asserting `quality_verdict` appears in BOTH the JSON and text outputs of `elai build --out ...`
  turns RED while `build_generates_app_and_receipt` stays green. Restoring build_cmd.rs byte-identical
  is the pass artifact. If instead the whole new pub fn lost its only caller, the auditor's
  production_caller_check reports DARK — belt and suspenders on the same failure class.
- M2 CHECKER-STUB MUTATION (falsifies aggregation): replace `run_quality_gate` body with
  always-Pass. The new orchestrator-side/fixture test drives the REAL function against a tempfile
  fixture containing one dirty `.ts` file (`const x = input as any;` — a frozen Block trigger,
  l11_output_quality_contract.rs:160 precedent) plus one clean file, and asserts RED + Block rule id;
  a clean-fixture leg asserts Pass. Fixture-only: no template edit, no dirty bytes ever ship.
- Two-command gate law (D-267/D-7): `cargo build -p elai-cli --bin elai` FIRST, then
  `cargo test -p elai-cli --test <name>` — stale-binary greens describe the previous commit.
- This turn is DOC-ONLY: allowed_write_set contains exactly this file, so there are no new public
  symbols in the changeset now and the DARK-symbol BLOCK clause is vacuously satisfied here; the slice
  (section 6) inherits R-8 obligations when executed.

## 6. Minimal vertical slice plan (~120 LOC total, fits plan line 31's "~20 LOC" spirit at wiring scale)

1. `.elai_cc/crates/cli/src/build_cmd.rs` (ONE file for producer + wire + consumer-side output):
   - `struct QualityFinding { rule_id: String, file: String }` + `struct QualityGateReport`
     (`verdict`, `files_scanned`, `blocks`, `warns`) — private serde types, ~25 LOC.
   - `fn run_quality_gate(app_dir: &Path) -> QualityGateReport` — recursive walk, extension filter,
     per-file `evaluate_vibegate("build", src, rel)`, aggregation, fail-closed IO, ~60 LOC.
   - Call site between :82 and :83; RED branch mirrors :85-86; `BuildResult` gains
     `quality_verdict`/`quality_warn_count`; text printer gains the warn list, ~15 LOC.
   - `#[cfg(test)]` fixture legs for M2 (tempfile dirty/clean fixtures), ~40 LOC.
   - Import needed: `use elai_orchestrator::vibegate::{evaluate_vibegate, HookVerdictKind};` —
     both already `pub` (hook_dispatcher.rs:251 `pub struct HookFinding`; vibegate callsite precedent
     wrap_cmd.rs:184/353). No Cargo.toml change.
2. `.elai_cc/crates/cli/tests/l11_build_quality_gate_cli_smoke.rs` (NEW, mirrors
   `l13_rule_cli_smoke.rs` naming): happy-path leg asserts `quality_verdict` present in JSON and text
   (M1 falsifier); plan-only leg proves the read-only half gained no output drift; existing
   `l14_build_cli_smoke.rs` stays green untouched.
3. Execution law for the executor: Depot-only plain `cargo` shim (BUILD LAW), two-command gate, off-agent
   controller performs M1/M2 mutations and certifies DONE (this worker never self-certifies).
4. Extension hooks left open, not built: Option B relocating `run_quality_gate` to an orchestrator
   module behind the lib.rs:678-style re-export IF Task 5's dashboard field later needs the report
   cross-crate; SOLID-proxy leg once one-shot apps gain elai.map rows; J-23 generated-lane as a profile
   add-on where external linters exist.

Out of scope per TaskSpec: Tasks 4 (J-63 loopback broker) and 5 (dashboard field) — owner/UI-held,
not designed here.

## 7. Why BUILD-WITH-SEAM and not the alternatives

- Not DEFER: no frozen or shipped gate blocks building into `build_cmd::run` — the opposite of the
  I-31 situation; every dependency edge this slice needs (orchestrator dep, vibegate exports, cli test
  harness with `CARGO_BIN_EXE_elai`) is already shipping and already exercised from this crate.
- Not SUPERSEDED-BY: no shipping unit owns directory-level judging. The nearest neighbors are all
  single-input judges (MiniCritic::check mini_critic.rs:171; evaluate_vibegate vibegate.rs:141) or
  CLI-over-elai.map scripts — none covers "generated app directory -> quality verdict", which is why
  the unit sat DARK through S2/S5 (evidence L-14/*.md digests).
- The mutation K-27 guards against ("nothing fails today if the generated app is garbage") is closed by
  M1/M2 above: after the slice, deleting the gate turns RED, and a garbage-producing template fails the
  command before any receipt exists.

VERDICT: BUILD-WITH-SEAM (seam: `.elai_cc/crates/cli/src/build_cmd.rs::run`, insert `run_quality_gate(app_dir)` between line :82 `apply_template` and line :83 `receipt_path`, RED removes app_dir mirroring :85-86 and emits no receipt; slice plan: section 6 — one-file producer+wiring in build_cmd.rs reusing `elai_orchestrator::vibegate::evaluate_vibegate` (vibegate.rs:141, the checker path frozen by contracts/tests/l11_output_quality_contract.rs), new smoke `crates/cli/tests/l11_build_quality_gate_cli_smoke.rs` whose JSON/text `quality_verdict` assertions die under the wire-deletion mutation M1, plus dirty/clean tempfile fixtures killing the always-pass stub M2, gated by the two-command cargo law)

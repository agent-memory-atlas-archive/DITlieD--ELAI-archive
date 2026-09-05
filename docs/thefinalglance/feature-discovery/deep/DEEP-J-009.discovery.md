# DEEP-J-009 exhaustive feature discovery

## Scope, integrity, and method

- Processed manifest: `docs/thefinalglance/feature-discovery/DEEP-SOURCE-MANIFEST.md:L1003-L1018`, heading `### DEEP-J-009`; the next chunk starts at L1020 (`### DEEP-J-010`).
- Declared chunk: family J, **10 files**, **1,247 logical lines**, **127,107 bytes**, all ten classified `numbered/index original`; target status **under 1,800**.
- Mechanical verification: fresh `sha256sum`, `wc -l`, and byte-size checks matched every manifest row and both aggregates exactly. Missing paths: **0**; unexpected paths: **0**; duplicate paths: **0**; SHA mismatches: **0**; line mismatches: **0**; byte mismatches: **0**.
- Read boundary: all ten listed originals were read completely, first through last logical line. No original assigned to another deep chunk was opened. `10-J-SUMMARY_INDEX.discovery.md` was consulted only as an orientation and delta-comparison artifact; no source atom below is derived solely from it.
- Anchor notation refers to the manifest-verified source images: `J-3:Lx-Ly`, `J-4:Lx-Ly`, `J-6:Lx-Ly`, `J-19:Lx-Ly`, `J-23:Lx-Ly`, `J-26:Lx-Ly`, `J-45:Lx-Ly`, `J-51:Lx-Ly`, `J-88:Lx-Ly`, and `J-100:Lx-Ly`.
- Repeated premise, revised-scope, wiring, grounding, composition, and acceptance language is collapsed into one atom with every load-bearing anchor. Interaction, ambiguity/conflict, and summary-delta records reference atoms and are not counted again as features.
- Classification: `E` = explicit source fact/requirement; `I` = bounded inference; `A` = unresolved ambiguity/choice; `C` = internal conflict or summary correction that must not be silently flattened. Domains: `STATUS`, `CAP`, `SURFACE`, `FLOW`, `STATE`, `DATA`, `CFG`, `INT`, `EVENT`, `SEC`, `PRIV`, `COMP`, `FAIL`, `RECOVERY`, `RETRY`, `DET`, `IDEM`, `CONC`, `OBS`, `TEST`, `LIMIT`, `DEP`, `DEFER`, `EVIDENCE`.

### Exact processed manifest and verification

| Manifest class | Lines | Bytes | SHA-256 | Exact absolute path | Verification |
|---|---:|---:|---|---|---|
| numbered/index original | 154 | 9,997 | 71e62fb4da98fa8c66118da884b6b6c9c5065e5b97824e00b01807fd07089eb1 | docs/thefinalglance/J-plan/J-3_harness_undo_file_backups.md | exact |
| numbered/index original | 125 | 7,217 | fbc98bfad4df3d98c4d0618814af4230ed56db4e1b56d25e11dd56c0ac9f2c84 | docs/thefinalglance/J-plan/J-4_handoff_file_ledger.md | exact |
| numbered/index original | 117 | 6,695 | b6a2291192996a86d7cf391f1e57e64a8c7f2c9bc88407f72e076ffb2af6b926 | docs/thefinalglance/J-plan/J-6_incremental_eval_skill.md | exact |
| numbered/index original | 78 | 10,608 | ef778aff3e6f0ded62693bca69799ab456d3b447ff67a7270b83cd482952a524 | docs/thefinalglance/J-plan/J-19_vllm_alternative_backend.md | exact |
| numbered/index original | 72 | 10,973 | ab51246807f8d828dccf383d85942f57a8e08dd805a2df739634c07ed48247fd | docs/thefinalglance/J-plan/J-23_multilang_review_gate_catalog.md | exact |
| numbered/index original | 353 | 25,864 | 0e1dcbbaa73473b740b59ecb500991fc89033f8907ea5738392af5210fe53ec9 | docs/thefinalglance/J-plan/J-26_sia_evaluator_abi_task_packs.md | exact |
| numbered/index original | 53 | 11,088 | 74dab10afb9dcba59df40fb67019eeede78f9d5a0f84f32ad6a2431639203985 | docs/thefinalglance/J-plan/J-45_cross_family_advisory_calibration.md | exact |
| numbered/index original | 97 | 13,691 | 6aa66a2919824f1b119c35328beaae9d39f4af12f2484d2c0b10627055102735 | docs/thefinalglance/J-plan/J-51_toolcall_adherence_floor_role_gate.md | exact |
| numbered/index original | 68 | 8,274 | ee8205c19373335d8845d29e88dbf0de57346cb9d35742bbda95c7d893be016e | docs/thefinalglance/J-plan/J-88_runtime_dkg_edges_machine_audit_lane.md | exact |
| numbered/index original | 130 | 22,700 | 14a6319373b62e72d9d8ba3a11841722b505ee258461c708068a03635f98f1ec | docs/thefinalglance/J-plan/J-100_time_as_control_plane.md | exact |
| **Aggregate** | **1,247** | **127,107** | — | **10 exact paths** | **exact** |

## Per-file atomic ledgers

### J-3 — harness undo + per-session file backups

Source: `docs/thefinalglance/J-plan/J-3_harness_undo_file_backups.md` (154/154 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J3-E001 | E/STATUS | J-3 is PLANNED WAVE 2 BUILD, harness-side only, non-blocking by construction under JD-3, effort ~0.5–1 session (~4–6 h), pure in-process stdlib Python, zero network, zero deps, no prerequisites. | J-3:L3-L4, L18-L19 |
| J3-E002 | E/CAP | Objective is the harness half of U3: a NEW PreToolUse hook that copies the pre-image of every Edit/Write/NotebookEdit target into the session dir BEFORE the write lands, plus `.claude/tools/undo.py` for list/diff/restore/prune. | J-3:L5-L12, L54-L67 |
| J3-E003 | E/LIMIT | ELAI-side undo is out of scope (H-49/I-52/E-25 already cover it); J-3 protects only the harness `.claude/` + `workflow/` tooling root where gitignores leave mis-edits unrecoverable (HG-1). | J-3:L5-L8, L26-L28 |
| J3-E004 | E/SEC | Existing registered hooks are NEVER edited (JD-4/R8); a new hook file is added and registered as a user-reviewed settings change. | J-3:L12-L14, L61-L63, L89-L89, L119-L119 |
| J3-E005 | E/CFG | Stdlib-only house convention forces two documented deviations from the source doc: gzip not zstd (JC-1) and hashlib.blake2b not blake3 (JC-2), with a stored algorithm prefix so a later py-blake3 migration is additive. | J-3:L14-L16, L56-L58, L92-L92, L110-L111, L120-L120 |
| J3-E006 | E/DEP | Prerequisites are none: hook seam HG-3 and session-state recording HG-4 already exist. Parallel-safe with everything except J-4 (coordinate schema reading of the same session-state shapes, not files). | J-3:L19-L21 |
| J3-E007 | E/STATE | Last-good marking uses PostToolUse session state (HG-4): the newest backup seq preceding a recorded audit-PASS is tagged last-good; `undo.py prune` keeps last-good + newest and drops the middle (last-PASS-wins, deliberately NOT D-5 bandit). | J-3:L29-L32, L64-L70, L117-L117 |
| J3-E008 | E/STATUS | HG-1: 26 tools in `.claude/tools/`, no undo/backup tooling exists today. | J-3:L35-L35 |
| J3-E009 | E/INT | HG-3 live hook seam is `C:/Users/user/.claude/rag/hooks/` (`pre_tool_check.py` + siblings), registered via `.claude/settings.local.json` (re-read at edit), logging to per-session `workflow/sessions/{id}/hook-executions.log`. | J-3:L36-L38 |
| J3-E010 | E/DATA | HG-4: PostToolUse change-recording to `state.json` changes[] EXISTS and is proven; exact schema is PLAN-asserted and must be read defensively. | J-3:L39-L40, L147-L148 |
| J3-E011 | E/DEFER | HG-6 stale-F:-hook bug (FSM gate no-op for F: paths) is ADJACENT, user-deliberate, and deliberately unfixed by J-3; `pre_tool_check.py` is not touched. | J-3:L41-L42, L121-L121, L143-L144 |
| J3-E012 | E/DEP | H-49 RV-1 divergence-guard + undo-tree semantics are the semantics source J-3 mirrors WITHOUT porting CAS/undo-tree machinery; linear per-file history is enough for the harness. | J-3:L43-L44, L65-L66, L85-L85 |
| J3-E013 | E/CAP | NEW file `pre_edit_backup.py` (~120 LOC) on PreToolUse for Edit/Write/NotebookEdit: read pre-image; blake2b keyed hash algorithm-prefixed; if hash unseen this session, write `workflow/sessions/{id}/backups/{hash[:12]}/{relpath-encoded}@{seq}` (gzip above ~64KB); append backup line to hook-executions.log. | J-3:L55-L60, L78-L82, L132-L132 |
| J3-E014 | E/FAIL | Backup hook is observe-only (JD-3): NEVER blocks the tool call; on any internal error, log + allow. A hook crash cannot block an edit. | J-3:L59-L60, L90-L90, L118-L118 |
| J3-E015 | E/CFG | Registration is ONE settings.local.json entry ADDING the new hook, presented as a reviewable user diff; the plan ships the hook file + the exact JSON line. | J-3:L61-L63, L83-L83, L134-L134 |
| J3-E016 | E/SURFACE | CLI surface `undo.py` (~200 LOC core / ~280 LOC incl selftest) subcommands: `list` (per-file timeline with seq/size/hash/last-good mark), `diff` (pre-image vs on-disk, unified), `restore` (by file or seq), `prune` (keep last-good + newest; dry-run default). | J-3:L64-L67, L80-L80, L108-L109, L133-L133 |
| J3-E017 | E/RECOVERY | Restore mirrors RV-1: if on-disk hash != recorded post-image hash, REFUSE + print diff + require `--force-after-review`; no silent restore over a diverged file. | J-3:L65-L66, L91-L91, L100-L102 |
| J3-E018 | E/OBS | Absent audit records means no last-good is claimed (honest, R1 analog); last-good is state-derived only (JD-2), never recollection. | J-3:L68-L70, L93-L93 |
| J3-E019 | E/TEST | Smoke + negative arms: backup lands BEFORE the edit (seq/mtime), restore round-trips bytes (hash match), divergence refuses, unregistered-hook arm produces zero backups AND undo.py says so loudly (R2). | J-3:L71-L73, L97-L105, L116-L117 |
| J3-E020 | E/DATA | Decision artifact: `workflow/reasoning/J-3-decisions.md` records hash/compression deviations JC-1/JC-2, size threshold, prune policy, and the HG-6 open question verbatim. | J-3:L74-L75, L113-L113, L135-L135 |
| J3-E021 | E/SURFACE | frontend_pair is N/A — harness tooling with a CLI surface; the JD-1 artifacts are the backup tree + log rows. | J-3:L94-L96 |
| J3-E022 | E/INT | Production reachability is every live-session Edit/Write/NotebookEdit once registered (HARNESS PRODUCTION, JD-1). | J-3:L86-L87 |
| J3-E023 | E/TEST | DETERMINISTIC_CHECK: `undo.py list --session <sid>` (>=1 row after fixture edit); `restore --file <f> --dry-run` (hash-verified plan); `rg -c "import (zstd\|blake3)"` MUST be 0; `rg "blake2b"` present; decisions doc exists. | J-3:L107-L113 |
| J3-E024 | E/TEST | ACCEPTANCE requires friction observed across >=3 real sessions before any future blocking behavior is even proposed (JD-3/GD-4). | J-3:L122-L123 |
| J3-E025 | E/TEST | `undo.py selftest` over a temp session fixture (~80 LOC) covers: (a) backup-before-edit ordering; (b) restore hash match; (c) divergence refusal; (d) prune policy; (e) corrupt/truncated backup -> typed failure never silent partial restore; (f) unregistered-hook negative arm. | J-3:L126-L129 |
| J3-E026 | E/LIMIT | Backup volume is bounded by dedup + threshold-gzip + prune; observe across 3 sessions before tuning (JD-3). | J-3:L145-L146 |
| J3-E027 | E/DEP | J-4's ledger cross-references last-good seqs; J-5 ports both to liteGAME; every later wave's edits are recoverable from day one. | J-3:L151-L152 |
| J3-I001 | I/STATE | Because restore compares on-disk hash to a recorded post-image hash, the backup log must record both pre-image identity and the post-edit identity that restore treats as the non-divergence baseline. | J-3:L55-L66, L100-L102 |
| J3-A001 | A/DATA | Exact `state.json` changes[] schema remains PLAN-asserted; last-good derivation degrades to "no last-good" when shape variance appears, but the full dual-shape contract is not enumerated in-source. | J-3:L39-L40, L147-L148 |
| J3-A002 | A/CFG | Exact gzip size threshold is stated as "~64KB" without a named constant or config key. | J-3:L58-L58, L74-L75 |
| J3-A003 | A/SURFACE | Exact JSON registration line for settings.local.json is deferred to the shipped reviewable diff rather than given as a final string. | J-3:L61-L63 |

Category coverage: capabilities/CLI ✓; workflows/states ✓; schemas/data/lifecycle ✓; flags/defaults/config ✓; integrations/events △ (hook log rows, no SSB); security/permissions ✓ (user-reviewed registration; no privacy policy); failures/recovery ✓; determinism ✓ (hash round-trip); observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No formal retry API or concurrency model beyond observe-only hooks.

### J-4 — handoff file ledger

Source: `docs/thefinalglance/J-plan/J-4_handoff_file_ledger.md` (125/125 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J4-E001 | E/STATUS | J-4 is PLANNED WAVE 1 BUILD, harness-side only, effort S (~0.3 session / 2–3 h), pure stdlib Python, zero network, no prerequisites. | J-4:L3-L3, L15-L17 |
| J4-E002 | E/CAP | Ships `.claude/tools/handoff_ledger.py` that derives every file edited this session from recorded state, never LLM recollection (JD-2 / G-5 DSR / H-51 journal-match analog). | J-4:L5-L11, L44-L48 |
| J4-E003 | E/DATA | Per-file rows are `{path, edit_count, first_seq, last_seq, last_good_seq?, audit_verdict?}` derived from `state.json` changes[] AND `hook-executions.log` (union, dedup). | J-4:L9-L10, L44-L48 |
| J4-E004 | E/DEP | J-3 last-good column is OPTIONAL: ledger ships without it and gains the column when J-3 lands; absent backups omit the column and state that fact. | J-4:L16-L17, L34-L34, L66-L66, L103-L104 |
| J4-E005 | E/LIMIT | Hard boundary: one-file-per-problem handoff rule and other template sections are untouched; this is one tool + one template line, not a protocol rewrite. | J-4:L12-L13, L52-L54, L97-L97 |
| J4-E006 | E/STATUS | HG-1: no handoff_ledger.py exists; HG-4 session state + hook logs exist with PLAN-asserted schemas. | J-4:L29-L31 |
| J4-E007 | E/SURFACE | CLI: `--session <sid>` (default newest live session) emits a paste-ready block: one line per file, path-sorted, header naming derivation sources + timestamp; `--json` for tooling. | J-4:L44-L48 |
| J4-E008 | E/FAIL | Honest degradation: missing/unreadable session state prints exactly `LEDGER UNAVAILABLE (no recorded session state at <path>)` and exits 1 — never an empty silent section (R1/R2). | J-4:L49-L51, L73-L73, L81-L83 |
| J4-E009 | E/SURFACE | Template amendment: ONE line in `handoff-protocol.md` ENTRY SECTIONS under FILES TOUCHED: when the tool exists, the body MUST be (or include verbatim) its output block; hand-tallied-only entries are a JD-2 violation at review. | J-4:L52-L54, L63-L63, L82-L83 |
| J4-E010 | E/TEST | `--selftest` (~50 LOC) over a temp fixture asserts row derivation, union/dedup, degradation arm, stable ordering, and J-3-column omission when backups absent. | J-4:L55-L56, L101-L104 |
| J4-E011 | E/DATA | Decision doc: `workflow/reasoning/J-4-decisions.md` records schema findings from two real sessions, the union rule, and the J-3 column contract. | J-4:L57-L58, L91-L91, L109-L109 |
| J4-E012 | E/INT | Production reachability is every handoff authoring per the protocol (main LLM runs the tool when writing/appending a handoff) — HARNESS PRODUCTION, JD-1. | J-4:L68-L70 |
| J4-E013 | E/LIMIT | Stub bans: no recollection-sourced rows; no silent empty ledger; no non-stdlib import; no schema-guess hard-coding (defensive dual-shape read). | J-4:L71-L74 |
| J4-E014 | E/SURFACE | frontend_pair N/A — harness tooling; justified. | J-4:L75-L76 |
| J4-E015 | E/TEST | DETERMINISTIC_CHECK: `--selftest` exit 0; `--session <sid> --json` rows match hook log spot-check; `rg "handoff_ledger"` in handoff-protocol.md; no blake3/zstd import; decisions doc exists. | J-4:L85-L91 |
| J4-E016 | E/TEST | ACCEPTANCE requires the FIRST real handoff after shipping carries the machine block (JD-1 evidence). | J-4:L99-L99 |
| J4-E017 | E/FAIL | Sessions predating the hooks: ledger says UNAVAILABLE honestly. | J-4:L119-L119 |
| J4-E018 | E/DATA | Files touched: CREATE handoff_ledger.py (~200 LOC incl selftest); MODIFY handoff-protocol.md ONE line; CREATE J-4-decisions.md. | J-4:L107-L109 |
| J4-A001 | A/DATA | Dual-shape defensive read is required, but the exact field names/types of both session-state shapes are not enumerated; a third later shape extends the reader rather than silently dropping rows. | J-4:L30-L31, L116-L118 |
| J4-A002 | A/CFG | Default "newest live session" resolution rule (path scan / mtime / active-session pointer) is not specified beyond the flag default. | J-4:L44-L45 |

Category coverage: capabilities/CLI/template surface ✓; workflows/states ✓; schemas/data ✓; defaults/config ✓; integrations ✓; security/integrity ✓ (no privacy policy); failures/degradation ✓; determinism △ (stable path order only); observability/evidence ✓; tests/done gates ✓; constraints/non-goals ✓; dependencies/status/deferred ✓. No retry/concurrency contract.

### J-6 — incremental-eval skill

Source: `docs/thefinalglance/J-plan/J-6_incremental_eval_skill.md` (117/117 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J6-E001 | E/STATUS | J-6 is PLANNED WAVE 1 BUILD, zero code beyond one skill directory, effort S, Track J lever STEERING, grounding STRONG (HG-2: no such skill among the 11). | J-6:L3-L7 |
| J6-E002 | E/CAP | Encodes the MAJOR RULE as `.claude/skills/incremental-eval`: for multi-artifact tasks, read ONE artifact, write a bounded verdict, then the next — conversation is disposable because the FILE is the state. | J-6:L8-L14, L17-L30 |
| J6-E003 | E/SURFACE | SKILL.md trigger: any task presenting N>=3 artifacts (files, reports, plans, papers, screenshots) to read/evaluate/compare/summarize, OR user invocation by name. | J-6:L18-L19 |
| J6-E004 | E/FLOW | Protocol step 1: resolve ONE topic-keyed verdict file in `workflow/reasoning/` (glob-then-append-if-exists — handoff one-file discipline applied to evaluation). | J-6:L21-L22 |
| J6-E005 | E/FLOW | Protocol step 2: per artifact, read fully (chunks if large) then append ONE bounded verdict block: ANCHOR line (path/id/section) + 3–8 lines of findings + open questions; never more before the next artifact. | J-6:L23-L25 |
| J6-E006 | E/FLOW | Protocol step 3 (BeliefState trick): working set is {task spec, verdict-file INDEX (head-only), next artifact}; prior artifacts live in the file; re-reads of the verdict file are head-only. | J-6:L26-L28 |
| J6-E007 | E/FLOW | Protocol step 4: after the last artifact, ONE synthesis section in the same file; the chat answer cites the file. | J-6:L29-L30 |
| J6-E008 | E/LIMIT | Named anti-patterns (with negative eval fixtures): batch-reading multiple artifacts before any verdict; unbounded verdicts (>~10 lines per artifact); full-file re-reads of the verdict file; verdicts without anchors; synthesizing before all artifacts are logged. | J-6:L31-L33, L71-L72 |
| J6-E009 | E/TEST | Ship `eval/positive.jsonl` + `eval/negative.jsonl` (JC-5); positives include multi-file review asks and plan-corpus reads; negatives include single-file asks, pure-chat, and tasks already inside another skill's protocol. | J-6:L34-L37, L88-L89 |
| J6-E010 | E/TEST | Frontmatter must pass `.claude/tools/agentskills_validator.py` (JC-5). | J-6:L38-L38, L87-L87, L94-L94 |
| J6-E011 | E/STATUS | HG-2: 11 skills, none covers this; no naming collision. | J-6:L41-L41 |
| J6-E012 | E/EVIDENCE | J-authoring session already executed the protocol: 14 artifacts -> 14 sections + synthesis in `workflow/reasoning/J-plan-prereading-2026-06-10.md` (recorded validation artifact). | J-6:L42-L44, L79-L80 |
| J6-E013 | E/DEP | Must cite (not fork) token-discipline + handoff-protocol one-file rule and runtime siblings I-5/I-6/I-11. | J-6:L45-L46, L63-L64, L69-L70, L96-L96 |
| J6-E014 | E/DEFER | SHIP NOW: skill dir + eval pair + validator pass. DEFER: any enforcement hook (PostToolUse batch-read counter) until observe self-trigger rate across >=3 real sessions (JD-3). | J-6:L48-L51, L73-L73, L97-L97 |
| J6-E015 | E/LIMIT | CLAUDE.md "Skills (mandatory workflow)" gains NO entry this pass (core-workflow skills only; auto-load handles the rest) — recorded so nobody "fixes" it. | J-6:L54-L56 |
| J6-E016 | E/SURFACE | frontend_pair N/A — harness skill; justified. | J-6:L57-L57, L74-L74 |
| J6-E017 | E/OBS | Observable effect: skill visibly fires on a 3+-artifact task; topic-keyed reasoning file accumulates one anchored block per artifact with mtime/growth between reads; negative fixtures do NOT trigger; batch-read patterns fail skill eval if protocol skipped. | J-6:L75-L83 |
| J6-E018 | E/TEST | ACCEPTANCE: validator passes; >=4 positives + >=4 negatives; protocol text carries context set + head-only + anti-patterns; no enforcement hook; first live multi-artifact task shows skill firing (JD-1). | J-6:L93-L98 |
| J6-E019 | E/DATA | Files: CREATE SKILL.md (~60 lines), eval pair, `workflow/reasoning/J-6-decisions.md`. | J-6:L100-L103 |
| J6-E020 | E/DEP | J-1's /see and future review skills inherit the one-artifact-at-a-time spine; verdict files become natural J-4 ledger companions. | J-6:L114-L115 |
| J6-A001 | A/CFG | Trigger precision vs other skills is acknowledged as open; negative eval set is the guard; tune from first 3 sessions (JD-3) without a fixed precision metric. | J-6:L109-L111 |
| J6-A002 | A/DATA | Exact topic-key naming function for the verdict file path is not specified beyond "topic-keyed" + glob-then-append. | J-6:L21-L22 |

Category coverage: skill surface/protocol ✓; workflows/states ✓; data/verdict file ✓; config/triggers ✓; integrations/cites ✓; failures/anti-patterns ✓; determinism △ (protocol order only); observability/evidence ✓; tests/validator ✓; constraints/deferred ✓. No privacy/retry/concurrency surface.

### J-19 — vLLM-V1 alternative serving backend

Source: `docs/thefinalglance/J-plan/J-19_vllm_alternative_backend.md` (78/78 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J19-E001 | E/STATUS | J-19 is Track J lever EFFICIENT, grounding STRONG (vLLM docs VERIFIED), nature BUILD additive NOT a removal, effort M, WAVE 3 (fixture legs now; live GPU legs remote-sidecar/RunPod-gated W-10), depends none (rides B-37 RuntimeAdapter). | J-19:L3-L3 |
| J19-E002 | E/CAP | ADDS vLLM V1 as one more RuntimeBackend behind the EXISTING RuntimeAdapter trait (11 backends, B-37); does NOT remove HF; measures vs live HF sidecar; USER makes the swap-out call. | J-19:L5-L5, L18-L18, L50-L51 |
| J19-E003 | E/LIMIT | Custom LogitsProcessor chain is only partially portable: DoLa is STRUCTURALLY non-portable (forward hooks / final-logits-only); DoLa already disabled on live remote (`--no-dola`); vLLM lane ships WITHOUT DoLa; HF retains it. | J-19:L7-L7, L11-L11, L20-L20, L52-L52 |
| J19-E004 | E/DATA | Logits-chain PORTABILITY MAP: DoLa NOT-PORTABLE on vLLM/SGLang; XGrammar NATIVE both; ETC portable-as-custom both; prompt-lookup ngram NOT-PORTABLE to V1 (V0 fallback, closed not-planned); seeded-Gumbel (J-13) portable-as-custom both. | J-19:L9-L16, L20-L20 |
| J19-E005 | E/CAP | Add `RuntimeBackend::VllmV1` (~150 LOC registration + AWQ/APC/batch-invariant launch config + portable logits subset); vLLM serves `/v1/messages` NATIVELY so ZERO Rust client change (`runtime_client.rs` request shape works as-is). Behind a backend-selection flag, never default. | J-19:L18-L18, L33-L33, L40-L40, L55-L55 |
| J19-E006 | E/CFG | AWQ-Marlin quant path: offline AutoAWQ checkpoint of abliterated Qwen2.5-Coder-7B (calibration corpus documented), replacing runtime NF4 dequant for the vLLM lane; AWQ+Marlin VERIFIED on SM 8.6 (RTX 3090); this is E-27's planned-but-never-built checkpoint. | J-19:L19-L19, L28-L28, L41-L41 |
| J19-E007 | E/CFG | Enable vLLM automatic prefix caching (cross-REQUEST KV reuse) + `VLLM_BATCH_INVARIANT=1` (beta, SM>=8.0) for token-determinism (J-13); HF structurally lacks both. | J-19:L20-L20, L30-L30 |
| J19-E008 | E/OBS | MEASURE side-by-side fixture (same prompts, same seed) HF vs vLLM on throughput, TTFT, prefix-cache hit-rate, token determinism; SURFACE the comparison; swap-out is SEPARATE user-confirmed decision, never auto. | J-19:L21-L21, L57-L60, L73-L73 |
| J19-E009 | E/LIMIT | EXCLUDED: removing HF sidecar; SGLang as target (report divergence + CVE-2026-7304 dill.loads RCE on custom-logits + no native /v1/messages; SGLang stays E-26 adapter); "Qwen3-Coder 14B" (REFUTED, ledger Tier-3); TensorRT-LLM / candle / mistral.rs. | J-19:L22-L22, L29-L29, L63-L64 |
| J19-E010 | E/DEP | Consumes B-37 RuntimeAdapter / resolve_backend (NOT reauthored); runtime_client.rs /v1/messages UNCHANGED; J-13 RRA seed contract for determinism. | J-19:L25-L30, L44-L46 |
| J19-E011 | E/EVENT | PREFER-REUSE SSB: reuse B-37 RuntimeBackendSelected / RuntimePanel rows (net 0); vLLM-specific cache/determinism metric rides J-13 LocalDeterminismEvent + H-9 CloudSpend-style cost rows; +0..+1 only if genuinely new metric has no home. | J-19:L35-L35 |
| J19-E012 | E/SURFACE | frontend_pair: HF-vs-vLLM comparison on existing RuntimePanel / CloudPanel (B-37/H-9) — the believe-the-number table; cross_lang_diff if sidecar config + Rust client both change (W-4). | J-19:L36-L36, L56-L56 |
| J19-E013 | E/INT | Production reachability: real `elai run --local` with backend flag set to vllm-v1 (inference dispatcher -> RuntimeAdapter::VllmV1 -> vLLM /v1/messages); remote-GPU-gated W-10; contracts-only never satisfies R7. | J-19:L47-L49 |
| J19-E014 | E/FAIL | NEGATIVE: vLLM lane with DoLa requested FAILS capability check (DoLa unavailable, stated not hidden); AWQ checkpoint without calibration data FAILS build assert; claiming a cache hit it cannot deliver FAILS measured-cache contract. | J-19:L60-L60 |
| J19-E015 | E/LIMIT | Stub bans: no HF removal; no DoLa-on-vLLM claim; no registry-resolved autoawq/vllm install (pin versions; anti-slopsquat); no literature-multiple determinism/cache claim (HD-1); no default-ON backend until side-by-side measured (W-2/R9). | J-19:L50-L55 |
| J19-E016 | E/OBS | expect_ssb: RuntimeBackendSelected=VllmV1 on vLLM run; side-by-side metric rows (throughput, prefix-cache hit-rate, token-stream blake3) for both lanes. | J-19:L59-L59 |
| J19-E017 | E/EVIDENCE | Grounding: vLLM V1 native /v1/messages, batch-invariant beta SM>=8.0, AWQ+Marlin SM 8.6, APC, XGrammar built-in; DoLa/prompt-lookup non-portability cited; live SSB 231/542 (grep 2026-06-12). | J-19:L62-L65 |
| J19-E018 | E/DEP | COMPOSES-WITH B-37 (backend #12), J-13 local determinism, E-27 quantization, H-1/H-2 prompt caching analog, E-26 SGLang untouched. | J-19:L67-L68 |
| J19-A001 | A/DEP | Which roles/lanes are still DoLa-dependent is open (likely none given live --no-dola default); must document for honest vLLM coverage. | J-19:L71-L71 |
| J19-A002 | A/EVENT | Whether a new vLLM-specific metric needs +1 action type or reuses J-13/H-9 is deferred to edit-time decide. | J-19:L35-L35 |
| J19-A003 | A/SURFACE | Exact comparison-table layout on RuntimePanel vs CloudPanel is "decide at edit". | J-19:L36-L36, L56-L56 |

Category coverage: capabilities/backends ✓; CLI `elai run --local` flag surface ✓; UI RuntimePanel/CloudPanel ✓; workflows/measure/compare ✓; config flags/quant ✓; integrations/events ✓; security (anti-slopsquat, no auto-swap) ✓; failures/capability fails ✓; determinism/batch-invariant ✓; observability/metrics ✓; tests/fixtures ✓; constraints/deferred ✓. No privacy/retry policy beyond fixture reruns.

### J-23 — multi-language review-gate catalog

Source: `docs/thefinalglance/J-plan/J-23_multilang_review_gate_catalog.md` (72/72 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J23-E001 | E/STATUS | J-23 is Track J lever AUDITABLE, grounding STRONG (24 tools repo-verified, SARIF confirmed), nature BUILD, effort M, WAVE 2, depends VibeGate F-1/F-2 (extends, does not rebuild). | J-23:L3-L3 |
| J23-E002 | E/LIMIT | Fallow is JS/TS-ONLY and must not be described as multi-language; no single OSS tool reproduces Fallow's mix across languages. | J-23:L5-L5, L14-L14, L44-L44 |
| J23-E003 | E/CAP | TWO-TIER normalizer shape: Tier-1 `sarif_ingest` (SARIF v2.1.0 -> ELAI gate-verdict); Tier-2 `arch_gate` TOML with per-tool `invoke_cmd` + `parser_fn` (json \| exit_code \| text_regex). | J-23:L7-L7, L10-L12, L24-L24 |
| J23-E004 | E/DATA | Confirmed SARIF emitters: Semgrep CE (`--sarif`; CE = single-file only, cross-file/dataflow PAID), PMD+CPD (SARIF since v6.31), Psalm (PHP), detekt (Kotlin). CodeQL is SARIF-native but LICENSE-BLOCKED for closed-source targets without GitHub Advanced Security — catalog as license-gated, never invoke without license check. | J-23:L10-L10, L45-L45, L57-L57 |
| J23-E005 | E/DATA | Arch-gate TOML per-language entries: `{ tool, invoke_cmd, parser_fn, sarif: bool, license, staleness_flag }`. Architecture tools mostly JSON/exit-code: dependency-cruiser, Knip, Deptrac, Periphery, import-linter, Packwerk. Catalog is DATA; thin runner shells out + parses. | J-23:L11-L11, L24-L24 |
| J23-E006 | E/TEST | GATE ELAI'S OWN CODE FIRST (dogfood): Rust = cargo-modules + cargo-udeps (+nightly prerequisite) + clippy; Python sidecar = vulture + ruff; TS/Svelte dashboard = knip + dependency-cruiser. | J-23:L12-L12, L41-L41 |
| J23-E007 | E/CAP | FRONTEND DESIGN/A11Y deterministic subset: design-token-drift check (spacing/radii/font-size not in token scale) + axe-core a11y-tree gate (role=generic / name=none / focusable=false on polished control is FAIL). Taste judgments stay design-taste.md / visual-ui-loop.md ADVISORY. | J-23:L13-L13, L34-L34, L48-L48 |
| J23-E008 | E/LIMIT | EXCLUDED: Fallow multi-language claim; CodeQL on closed-source without license; 3 STALE/unfit tools (go-callvis, jQAssistant GPL-3.0, Konsist); big monolithic catalog; taste-rubric as gate (TD-3); building own per-language analyzers. | J-23:L14-L14 |
| J23-E009 | E/DEP | Extends F-1 VibeGate PreToolUse chain and F-2 code-vuln ruleset; structural precedent is J-9 hardware catalog (probe-once data-driven TOML). | J-23:L17-L19, L36-L36 |
| J23-E010 | E/INT | Design/a11y gate rides existing visual-ui-loop ax-scaffold output + token-scale check (~80 LOC); dashboard-only. | J-23:L20-L20, L25-L25, L38-L38 |
| J23-E011 | E/EVENT | +ReviewGateCatalogEvent SSB Pattern-2 single-variant; Kind: CatalogBuilt / GateInvoked / SarifIngested / VerdictEmitted; baseline+1 from LIVE 231/542 (grep 2026-06-12); full 6-part variant discipline. | J-23:L26-L26 |
| J23-E012 | E/SURFACE | frontend_pair: review-gate findings on existing audit/vibesec panel + a11y/token findings on dashboard QA surface (W-4 cross_lang_diff for dashboard leg). | J-23:L27-L27, L50-L50 |
| J23-E013 | E/INT | Production reachability: real `elai` review/audit run on generated code in a cataloged language (AUDITING path -> arch_gate runner -> tool invoke -> verdict) AND ELAI's own CI; contracts-only never satisfies R7. | J-23:L39-L42 |
| J23-E014 | E/CFG | NO BLOCK default until per-tool FP rate is bounded on >=3 real runs (GD-4) + wire-dark-allowlist row (W-2/R9). | J-23:L49-L49 |
| J23-E015 | E/FAIL | Missing tool binary FAILS-LOUD (TOOL_MISSING, never silent pass); CodeQL on closed-source without license is REFUSED; clean component passes a11y; SARIF tool with no findings yields clean VerdictEmitted. | J-23:L54-L54 |
| J23-E016 | E/LIMIT | Stub bans: no Fallow on non-JS/TS; no unpinned registry-resolved install (anti-slopsquat; flag +nightly cargo-udeps); no stale-tool default; no taste-as-gate. | J-23:L43-L49 |
| J23-E017 | E/DEFER | Start with cross-language SARIF core (Semgrep/PMD-CPD/Lizard); add per-language arch tools only where target language demands architecture-boundary checks the cross-language tools miss; catalog grows per demonstrated need. | J-23:L65-L65, L70-L70 |
| J23-E018 | E/SEC | Catalog must carry license field; runner REFUSES license-gated invocation on ineligible target, never silently degrades to a weaker check that looks like a pass. | J-23:L66-L66 |
| J23-E019 | E/OBS | Catalog last-release date; stale tools are dropped not invoked; re-check staleness on catalog rebuild. | J-23:L67-L67 |
| J23-E020 | E/OBS | Observable trigger: review run on generated Python with dead code + Svelte div-onClick non-semantic button; expect CatalogBuilt, GateInvoked=vulture, SarifIngested if SARIF, VerdictEmitted, a11y FAIL on role=generic. | J-23:L51-L54 |
| J23-I001 | I/CFG | Because BLOCK is deferred until FP is bounded, Tier-1 initial catalog invocation is effectively advisory/observe until the allowlist promotion lands. | J-23:L49-L49 |
| J23-A001 | A/DATA | Exact arch_gate.toml schema field defaults, per-language eligibility probe mechanism, and initial tool-row set size are not fixed beyond the two-tier shape and start-with-SARIF-core guidance. | J-23:L11-L12, L65-L65 |
| J23-A002 | A/CFG | Named const/threshold for "FP rate bounded on >=3 real runs" is not defined. | J-23:L49-L49 |

Category coverage: capabilities/gates ✓; CLI/review surfaces ✓; UI vibesec + dashboard QA ✓; data catalogs ✓; config/flags ✓; integrations/events ✓; security/license ✓; failures/TOOL_MISSING ✓; observability/SSB ✓; tests/dogfood ✓; constraints/deferred ✓. No privacy/retry formal policy.

### J-26 — sia evaluator ABI + task-pack format

Source: `docs/thefinalglance/J-plan/J-26_sia_evaluator_abi_task_packs.md` (353/353 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J26-E001 | E/STATUS | J-26 is PLANNED WAVE RSW-1, .elai_cc runtime EXTENSION, effort S-M, lever PROVABLE, grounding STRONG (hexo-ai/sia v0.5.1 + arXiv:2605.27276 WebSearch-VERIFIED 2026-06-12); zero new subsystem — adopts two language-agnostic contracts into H-43 and E-54. | J-26:L3-L7 |
| J26-E002 | E/CAP | T1 EVALUATOR ABI: fitness evaluator for H-43 (FitnessEvaluated, SCHEMA 221 family) is a SUBPROCESS with `--gen-dir <dir>`; writes `results.json` INTO that dir; exit 0 = it ran; MISSING results.json = hard FAIL, never a default score (R1/R2). Top-level scalar fields are the only score channel. | J-26:L18-L23, L76-L77 |
| J26-E003 | E/SEC | ELAI delta over sia: evaluator fileset is R8-FROZEN (gate-manifest); candidate can never edit it — sia has no freeze (DO-NOT-COPY residue). | J-26:L23-L25, L69-L70, L193-L200, L284-L288 |
| J26-E004 | E/DATA | T2 TASK-PACK CONVENTION: `data/public/task.md` + `data/public/<evaluator>` + `data/public/<fixtures>` + `data/private/<held-out>` + `reference/<sample solutions>`. Input format for E-54 ingest_task and H-43 frozen-test corpus; D-6 held-out maps to data/private/. Free interop both ways with sia packs. | J-26:L26-L31 |
| J26-E005 | E/DEFER | Riders recorded not tasks: B1 loop = H-43 confirmation only; B4 context.md = SSB confirmation; B5 weights/RL = E-55/I-31 deferral; B6 log size guards covered. | J-26:L32-L35 |
| J26-E006 | E/STATUS | H-43 FitnessEvaluated PARTIAL (replaces blake3(config_bytes) placeholder VD-1); E-54 DARK (ingest_task has no CLI entrypoint); D-6/H-47/G-6/F-11 cited as composition seams. | J-26:L38-L42 |
| J26-E007 | E/CFG | SHIP NOW Tier-1: ABI struct + subprocess runner + results.json reader missing=FAIL, task-pack reader + ONE golden fixture pack, E-54 ingest CLI. DEFER Tier-2: any LIVE model-in-the-loop generation (RunPod/remote-gated W-10). | J-26:L46-L51 |
| J26-E008 | E/CAP | Produces `evaluator_abi` module (~150 LOC: TaskPack, EvaluatorInvocation, ResultsJson; subprocess via std::process::Command) + E-54 ingest CLI entrypoint (~60 LOC) + golden fixture pack + contract test. | J-26:L54-L57, L62-L66, L112-L118 |
| J26-E009 | E/SURFACE | frontend_pair N/A — no new dashboard surface; fitness rows surface through existing H-43/darwinian panels. Justified. | J-26:L58-L59 |
| J26-E010 | E/INT | Production reachability: darwinian admit loop fitness call when `ELAI_DARWINIAN_LOOP_ENABLE` (allowlisted) + E-54 ingest CLI verb (unconditional). Contracts-only never satisfies R7. | J-26:L71-L74 |
| J26-E011 | E/LIMIT | Stub bans: no default/fallback score on missing results.json; no evaluator file writable by candidate agent; no LLM in ABI path (TD-3); no .unwrap()/.expect() on prod paths; no blake2/sha2. | J-26:L75-L78 |
| J26-E012 | E/OBS | Trigger: `elai <eval-corpus ingest verb> --pack <golden-pack>` then fitness harness via darwinian/F-11 driver; expect FitnessEvaluated score == golden results.json scalar + ingest rows; NEGATIVE: delete results.json or nonzero evaluator exit -> hard FAIL (default-zero/SKIP is a gate bug). | J-26:L79-L86 |
| J26-E013 | E/FLOW | Ship-order micro-slices S1–S8 with MOVE/EXPECTED_OBSERVATION/ON_FAIL; S2+S3 ship together (producer + production consumer); S5+S6 ship together (fixture + test); S7 is OUT-OF-BAND R8 re-lock. | J-26:L88-L92 |
| J26-E014 | E/INT | PRODUCTION_HOOK seams verified: darwin_admit_loop.rs:1153-1157 (h43_fitness), :1173-1207 (emit_fitness_row REUSED), darwin_loop_cmd.rs:101 (entrypoint), ingest.rs:250 (gap), args.rs:75/:3344 (CliAction), lib.rs:738 (mod). | J-26:L94-L101 |
| J26-E015 | E/DATA | S1: author `workflow/reasoning/J-26-decisions.md` (scope, H-43 boundary, R8 fork, GAP-1 mapping) before any code. | J-26:L103-L110 |
| J26-E016 | E/CAP | S2: create evaluator_abi.rs WIRE-DARK[J-26] until S3 wires it; ResultsJson top-level scalars only; missing-results = Err hard FAIL (no default-zero, no SKIP). | J-26:L112-L127 |
| J26-E017 | E/FLOW | S3: config-gated branch at h43_fitness site invokes subprocess when TaskPack evaluator supplied; converts ResultsJson -> ExecutionFitness; feeds SAME h43_fitness variable; emit_fitness_row UNCHANGED; in-process path stays fallback when no pack (RISK-3); remove WIRE-DARK. | J-26:L129-L144, L294-L300 |
| J26-E018 | E/SURFACE | S4 CLI: `CliAction::EvalCorpusIngest { pack_path, project_root, output_format }`; parse arms `"eval-corpus-ingest" \| "ingest-task-pack"`; `eval_corpus_ingest_cmd.rs` builds NewTaskInput from TaskPack and calls `ingest_task`. | J-26:L146-L162 |
| J26-E019 | E/DATA | S5 golden pack under `.elai_cc/crates/eval-corpus/fixtures/golden-pack/` with deterministic evaluator (python/bash, no LLM, no network) writing known scalar results.json. | J-26:L164-L175 |
| J26-E020 | E/TEST | S6 contract `j26_evaluator_abi_contract.rs`: POSITIVE golden ingest + FitnessEvaluated score match; NEGATIVE delete results.json OR nonzero exit -> hard FAIL. Auditor runs cargo test, not executor. | J-26:L177-L190 |
| J26-E021 | E/SEC | S7: add evaluator_abi.rs + golden fixture paths to `.claude/gate-manifest.txt`; re-run `gate_freeze_check.py --lock` OUT-OF-BAND (human-reviewed, NOT by executing plan run). | J-26:L192-L202 |
| J26-E022 | E/TEST | S8 close-out: production_caller_check over full changeset for TaskPack/EvaluatorInvocation/ResultsJson WIRED; update decisions close-out. | J-26:L204-L212 |
| J26-E023 | E/FLOW | FORK-1 GAP-1 NewTaskInput schema mismatch -> STOP + handoff, do not force-fit. FORK-2 S7 re-lock not pre-authorized -> skip S7, surface material fork. FORK-3 R2 negative cannot wire without re-authoring emit_fitness_row -> STOP + architecture handoff. | J-26:L214-L227 |
| J26-E024 | E/TEST | RECON-1..4 required before coding: NewTaskInput schema map; CliAction exhaustiveness; results.json scalar contract from sia EVALUATION_GUIDE; blake2/sha2 ban (blake3 only). | J-26:L229-L242 |
| J26-E025 | E/FAIL | ABORT-1..5: cargo-check failure after 2 alignment attempts; DARK after verified call-site; R2 negative passes unfixed code; S7 unauthorized; fixture needs new deps. | J-26:L244-L256 |
| J26-E026 | E/TEST | VERIFY-1..8 off-agent: cargo check orchestrator/cli; production_caller_check; j26 contract test; gate_freeze_check --lock; R2 negative hard FAIL assertion. | J-26:L258-L281 |
| J26-E027 | E/SEC | REDTEAM: candidate editing evaluator fails (R8 freeze); missing results default-zero fails (stub_ban + S6); ABI-as-replacement patched to sibling channel + in-process fallback; executor gate-manifest edit patched to OUT-OF-BAND + FORK-2. | J-26:L283-L305 |
| J26-E028 | E/TEST | INTEGRATION_SMOKE-1: real `elai eval-corpus-ingest --pack <golden-pack>` then fitness harness; assert ingest rows + FitnessEvaluated scalar match + results.json on disk. SMOKE-2: delete results.json re-run -> hard FAIL through real entrypoint. | J-26:L307-L322 |
| J26-E029 | E/EVENT | NO new action_type preferred — reuse H-43 FitnessEvaluated (221) + E-54 ingest rows; net 0 (decide at edit; W-9 re-grep LIVE 231/542 if a row is added). | J-26:L340-L341 |
| J26-E030 | E/DEP | COMPOSES-WITH H-43, E-54, D-6, H-47, F-11, G-6 (packs as SAFEDELTA fixture inputs), gate-manifest R8. sia LOOP is NOT ported. | J-26:L349-L351 |
| J26-C001 | C/FLOW | Original wording "wired where H-43's fitness evaluation dispatches" was ambiguous (replacement vs sibling); refined plan makes ABI a SIBLING config-gated channel with in-process fallback (REDTEAM attack 3 PATCH). | J-26:L294-L300 |
| J26-A001 | A/DATA | GAP-1 NewTaskInput.task required-field mapping from TaskPack is recon-needed; exact field list not settled in-source. | J-26:L158-L162, L215-L218, L230-L233 |
| J26-A002 | A/EVENT | Whether any new SSB row is required beyond FitnessEvaluated/E-54 ingest is deferred to edit-time W-9. | J-26:L340-L341 |
| J26-A003 | A/SURFACE | Exact CLI verb string surface name is dual-aliased (`eval-corpus-ingest` \| `ingest-task-pack`) without a declared canonical primary. | J-26:L149-L150 |

Category coverage: capabilities/ABI/CLI ✓; workflows/slices/forks ✓; schemas/packs/results.json ✓; config/flags ✓; integrations/production hooks ✓; security/R8 freeze ✓; failures/abort/R2 ✓; determinism (golden scalar) ✓; observability/SSB reuse ✓; tests/verify/smoke ✓; constraints/deferred ✓. UI intentionally absent.

### J-45 — cross-family advisory calibration

Source: `docs/thefinalglance/J-plan/J-45_cross_family_advisory_calibration.md` (53/53 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J45-E001 | E/STATUS | J-45 is Track J lever DOCTRINE-PROPAGATION (D5 semantic leg), nature BUILD, effort M, WAVE DPW-3, depends J-43, J-44, gpt-bridge MCP. | J-45:L3-L3 |
| J45-E002 | E/CAP | For judgements with no mechanical oracle, an LLM may CONTRIBUTE but is never load-bearing: one advisory vote, off the critical path, critic LLM always a DIFFERENT pretraining family (same-family self-critique BANNED). | J-45:L4-L4 |
| J45-E003 | E/TEST | EXP-DP3 OBSERVE-ONLY gate: build calibration corpus and MEASURE Cohen kappa vs human labels; advisory is NON-LOAD-BEARING and labelled "uncalibrated" until kappa > 0.6 on held-out slice; kappa < 0.5 = unusable, redesign prompt/lens. Nothing in DP-1/DP-2 may CITE advisory as satisfied D5 before this passes. | J-45:L7-L7, L28-L28 |
| J45-E004 | E/DATA | Calibration corpus: 100–300 human-labeled doctrine-conformance examples (feature/design/reasoning-artifact + ground-truth: doctrine-clean \| named-violation(sub-doctrine) \| debt). Drawn from DP-4 baseline sweep + seeded synthetic violations (one per DD-* class + one boilerplate-ATTACK). Held-out un-gamed subset for kappa. | J-45:L8-L8, L25-L25 |
| J45-E005 | E/CAP | Cross-family advisory review: doctrine-conformance review on DIFFERENT family than producer via gpt-bridge MCP (`gpt_chat` / `grok_chat` — second AND third family) with feature/artifact + doctrine lens + DECISION-LEDGER serialized IN (no repo access). Emits findings, never a release-gating boolean. Same-family dispatch is REFUSED. | J-45:L9-L9, L28-L28 |
| J45-E006 | E/FLOW | Escalation routing (teeth): flagged finding routes to orchestrator/user as explicit DECISION (handoff-protocol BLOCKED=handoff-entry shape, or orchestrate/auto-orchestrate handoff-mediation), with weighting/visibility — NOT a dashboard tile that decays unread. | J-45:L10-L10, L18-L18 |
| J45-E007 | E/FAIL | Degradation honesty: when second family unreachable (rate-limit/POOL_EXHAUSTED), review degrades to SINGLE-family and RECORDS a DOCTRINE-DEBT row (`current_disposer = "single-family-degraded"`), never silently passing single-family as cross-family. | J-45:L11-L11, L28-L28 |
| J45-E008 | E/LIMIT | EXCLUDED: LLM-as-judge on release-gating path; same-family self-critique as verification; dashboard-only surface with no escalation; treating uncalibrated advice as satisfied D5; deep-research as the advisory (gpt-bridge has no deep-research on grok — use effort=expert/heavy; advisory is one-shot diversity vote). | J-45:L12-L12 |
| J45-E009 | E/DEP | Consumes gpt_chat/grok_chat, doctrine lens (J-43) + DECISION LEDGER (J-44), handoff-protocol/orchestrate mediation, doctrine-debt-ledger.toml (J-43). | J-45:L16-L19, L26-L26 |
| J45-E010 | E/INT | Reachable from: a J-43 floor WARN or J-44 substance-audit flag routed to advisory; advisory finding routed to orchestrator/user decision. | J-45:L27-L27 |
| J45-E011 | E/LIMIT | Stub bans: no advisory verdict on release-gating path; no same-family dispatch; no silent single-family pass; no "calibrated" claim before kappa>0.6 on held-out. | J-45:L28-L28 |
| J45-E012 | E/EVENT | SSB: net 0 new action_type (harness leg; calibration corpus + kappa + escalation entries are off-agent files/log rows, JD-1). Advisory NEVER journals a verdict on a critical path. | J-45:L30-L30 |
| J45-E013 | E/OBS | JD-1 observable: seeded doctrine-violation snippet returns finding agreeing with human label at measured kappa; flagged finding produces real DECISION entry (grep-provable); same-family REFUSED; degraded arm emits single-family-degraded debt row. | J-45:L29-L29 |
| J45-E014 | E/DATA | Produces: cross-family advisory review protocol, `data/doctrine-calibration/*.jsonl`, kappa measurement, escalation-routing entry kind, single-family-degraded DOCTRINE-DEBT row. | J-45:L25-L25 |
| J45-E015 | E/DEP | COMPOSES-WITH J-43 (routes DD-4/unprovable findings), J-44 (boilerplate/attack-theater), gpt-bridge, handoff/orchestrate, D-13/VD-4/TD-3, DP-4/J-46 (corpus seed + debt-rate), D5 doctrine. | J-45:L37-L44 |
| J45-E016 | E/EVIDENCE | C3 deep-research vote: same-family judges show 10–25% self-preference inflation; advisory ignored unless kappa>0.6 on 100–300 traces AND escalation teeth; PROVISIONAL on THIS domain until EXP-DP3 measures. | J-45:L33-L33 |
| J45-A001 | A/CFG | Who labels the 100–300 traces and how label quality is controlled is open; lean: user labels held-out; rest seeded from DP-4 + one synthetic per DD-*. | J-45:L47-L47 |
| J45-A002 | A/FAIL | When gpt-bridge is down: BLOCK vs proceed-with-recorded-debt is open; lean default proceed+debt because advisory is non-load-bearing. | J-45:L48-L48 |
| J45-A003 | A/OBS | kappa drift after model-family updates needs re-measure cadence or stale-kappa-as-debt policy (not fixed). | J-45:L49-L49 |
| J45-A004 | A/OBS | Escalation ACCEPTANCE/override rate is named as a second health signal (DP-4) but not given a threshold. | J-45:L50-L50 |

Category coverage: advisory protocol ✓; handoff/decision surface ✓; calibration data ✓; config/degradation ✓; integrations (gpt-bridge) ✓; security (same-family refuse) ✓; failures/debt ✓; observability/kappa ✓; tests/EXP-DP3 ✓; constraints ✓. No product UI panel required; dashboard-only is banned.

### J-51 — tool-call adherence floor + role gate

Source: `docs/thefinalglance/J-plan/J-51_toolcall_adherence_floor_role_gate.md` (97/97 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J51-E001 | E/STATUS | J-51 is PLANNED WAVE GUAVA-1, BUILD-on-shipped over B-10 ModelCapabilityProfile + PerRoleRuntime, effort M, lever PROVABLE+SMARTER-ORCHESTRATION, grounding STRONG (Guava arXiv:2606.18363 Table 3), parallel-safe with J-50, composes B-10/E-31/J-34. | J-51:L3-L5 |
| J51-E002 | E/CAP | Measures residual SEMANTIC tool-call reliability under the production XGrammar mask (right tool + right args), surfaces on ModelCapabilityProfile, and lets role-router prefer higher-adherence models for tool-heavy roles. | J-51:L6-L6, L10-L12 |
| J51-E003 | E/SURFACE | Leg 1: CLI `elai bench tool-call-adherence` — fixed fixture set of tool-requiring tasks with KNOWN-correct (tool, args) ground truth, run per (local model, lane) UNDER the production grammar mask. | J-51:L10-L10, L33-L33, L54-L54 |
| J51-E004 | E/OBS | Score deterministic tool-selection-correct + args-schema-valid + args-semantically-correct rates (gate/schema verdicts, NEVER LLM-judge — TD-3). Output per model: adherence score in [0,1] + metrics TOML, byte-recomputable under ELAI_SEED (GD-2). | J-51:L10-L10, L70-L71 |
| J51-E005 | E/DATA | Leg 2: `ModelCapabilityProfile` (context_budget.rs:292) gains `tool_call_adherence: Option<f32>` (absent = today's behavior; no field = no gate). Parallel additive to J-34 fill curves. | J-51:L11-L11, L18-L18, L34-L34, L42-L42 |
| J51-E006 | E/CFG | Leg 3 role-gate: advisory, default-ON kill-switch; consults adherence when assigning TOOL-HEAVY roles (executor/researcher); below named const `TOOLCALL_ADHERENCE_FLOOR` PREFERS higher-adherence resident model, else emits FloorBreached and advises cloud-spill/warn via I-7 cascade — does NOT hard-kill. Disable via `ELAI_TOOLCALL_ADHERENCE_GATE_DISABLE` (NOT a new ELAI_*_ENABLE). | J-51:L12-L12, L34-L34, L43-L43, L62-L63 |
| J51-E007 | E/LIMIT | HARD boundaries: measures SEMANTIC adherence under mask, NOT format validity; NOT constrained decoding on reasoning roles (HD-3); NOT hard block; NOT LLM-judge; NOT claim ELAI reproduces Guava percentages (phenomenon anchor only). | J-51:L14-L14, L59-L61 |
| J51-E008 | E/DEP | Consumes B-10 profile, B-37 PerRoleRuntime / RuntimeOrchestrator::try_complete (runtime_orchestrator.rs:185), sidecar XGrammar mask (never bypass), I-7 EscalationCascade, mirrors J-34 bench shape. | J-51:L18-L24, L47-L51 |
| J51-E009 | E/DEFER | Tier-1: legs 1+2 for LOCAL lane + advisory role-gate when field present. Tier-2: per-role-class curves; auto-recalibration on G-10 model-identity change (until G-10: manual `--recalibrate`); journal-regression leg with real production outcomes. | J-51:L28-L29 |
| J51-E010 | E/CAP | NEW `crates/orchestrator/src/toolcall_adherence.rs` (~160 LOC: adherence-score type, fixture runner under masked path, model-identity-keyed artifact store, floor lookup) + CLI bench arm (mirrors J-34 context-fill, args_bench.rs). | J-51:L33-L33, L41-L41 |
| J51-E011 | E/EVENT | +ToolCallAdherenceMeasured Pattern-2 single-variant; Kind: AdherenceMeasured / RoleAdvised / FloorBreached; baseline+1; SCHEMA_VERSION COMPUTED (`-309 + SSB_ACTION_TYPES.len()`, ssb.rs:1500); full 9-tuple lockstep; re-grep BOTH at edit (W-9). Live baseline cited 268/577 (2026-06-16). | J-51:L35-L35, L44-L44, L80-L80 |
| J51-E012 | E/SURFACE | frontend_pair: per-local-model adherence tile on model/cost-cockpit panel family (believe-the-number, same family as J-34 curve tile); #[tauri::command] + ts-rs; cross_lang_diff.py PASS. | J-51:L36-L36, L66-L67 |
| J51-E013 | E/INT | Production reachability: `elai bench tool-call-adherence --model <local_model>` AND RuntimeOrchestrator::try_complete role assignment on real task runs; contracts-only never satisfies R7. | J-51:L53-L56 |
| J51-E014 | E/FAIL | NEGATIVE: model with NO adherence artifact uses today's role assignment AND logs no-calibration fallback (no silent pseudo-gate); gate disabled via DISABLE logs disabled (not silent no-op); LLM-judge scoring FAILS contract (TD-3). | J-51:L64-L64, L72-L72 |
| J51-E015 | E/LIMIT | Stub bans include: no fabricated adherence numbers (ID-5); no .unwrap()/.expect() on prod paths; no blake2/sha2; no silent fallback. | J-51:L58-L64 |
| J51-E016 | E/EVIDENCE | Guava Table 3 spread is VERIFIED phenomenon anchor (embodied-manipulation domain caveat GD-5/HD-1); house ordering 7B>3B>2B for tool-call reliability makes 3B-NF4 borderline the floor-gate motivation. | J-51:L76-L79 |
| J51-E017 | E/DEFER | Riders: publishing per-local-model table = K-1 claims-policy once numbers exist; per-role-class adherence Tier-2; driving J-48 LocalServingRouter tier choice is a J-48 rider once both ship. | J-51:L91-L95 |
| J51-A001 | A/CFG | Numeric value of `TOOLCALL_ADHERENCE_FLOOR` is named as a const but not given a number. | J-51:L12-L12, L71-L71 |
| J51-A002 | A/EVENT | Whether FloorBreached can reuse E-31 TierModelSwapped vs owning ToolCallAdherenceMeasured is noted then resolved toward owning adherence-specific event (loses floor value if reused RouteDecisionEvent) — still an edit-time lockstep detail. | J-51:L21-L21, L51-L51 |
| J51-A003 | A/DATA | Exact fixture-set size, tasks, and known (tool,args) ground-truth inventory are not enumerated. | J-51:L10-L10 |

Category coverage: CLI bench surface ✓; UI adherence tile ✓; profile schema ✓; role-routing workflow ✓; flags/kill-switch ✓; events/SSB ✓; security (no LLM-judge) ✓; failures/fallback logs ✓; determinism/GD-2 ✓; tests/negatives ✓; constraints/Tier-2 ✓.

### J-88 — runtime DKG edges + machine-derived-first audit lane

Source: `docs/thefinalglance/J-plan/J-88_runtime_dkg_edges_machine_audit_lane.md` (68/68 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J88-E001 | E/STATUS | J-88 is Track J lacuna-fill, lever ENABLE-THE-CONSUMERS, nature BUILD (composition + one edge-kind + one input-routing), effort M, WAVE 1, depends none; E2 feeds J-90 L5; E3 hardens every binary gate. | J-88:L3-L3 |
| J88-E002 | E/CAP | LEG A (E2): feed runtime-observed call edges into the DKG as typed `observed_calls` edge KIND (a DKG edge kind, NOT an SSB row), sourced from RRA action-stream (G-2), L3 entrypoint coverage, L4 traces. | J-88:L6-L8, L17-L19, L24-L24 |
| J88-E003 | E/SEC | STATIC is the conservative backbone; runtime edges are ADDED with provenance and MAY NEVER SHRINK the static cone (widen-only). DIVER 35–50% impact-set reduction is DESIGN-RATIONALE only, never an ELAI number (HD-1/ID-5). | J-88:L8-L8, L42-L42, L53-L53 |
| J88-E004 | E/LIMIT | EXCLUDED E2: runtime edge that narrows/removes a static edge; LLM proposing edges (TD-3); a new graph engine (extend DKG, GD-1). | J-88:L9-L9 |
| J88-E005 | E/CAP | LEG B (E3): two-lane reviewer. BINARY VERDICT LANE consumes ONLY machine-derived artifacts (coverage/branch %, SARIF rule-IDs + severities, syscall classes from J-87 observer, unmatched-observer counts, bundle/attestation digests). Do NOT SHOW SOURCE AT ALL in that lane (AST stylometry deanonymizes authors). | J-88:L11-L13, L43-L43 |
| J88-E006 | E/LIMIT | LLM-auditor prose stays ADVISORY (TD-3). E3 only re-routes INPUT of existing binary gates (wiring / cant-lie F-4 / blast J-16 / mutation C-28); does not rebuild gates. | J-88:L13-L14, L45-L45 |
| J88-E007 | E/LIMIT | EXCLUDED E3: showing source in binary lane; advisory LLM prose entering deterministic verdict; rebuilding gates. | J-88:L14-L14 |
| J88-E008 | E/DEP | Substrate: DKG WIRED, B-29 impact API WIRED, G-2 RRA WIRED, auditor OS-isolated, deterministic checker outputs WIRED. | J-88:L17-L20 |
| J88-E009 | E/CFG | FEATURE FLAGS: `ELAI_RUNTIME_DKG_EDGES_ENABLE` + `ELAI_MACHINE_AUDIT_LANE_ENABLE` (kill-switches, OBSERVE-ONLY first), each a wire-dark-allowlist row with promotion plan (W-2/R9). | J-88:L26-L26 |
| J88-E010 | E/SURFACE | frontend_pair: E2 surfaces in existing GraphViewer (edge-kind legend); E3 is default-input change, no new surface (W-6-justified where absent). | J-88:L27-L27, L46-L46 |
| J88-E011 | E/EVENT | NO new SSB for either leg (DKG edge kind + input-routing only); DDL epoch bump only if edge schema is versioned. live SSB baseline ~268/577 (grep 2026-06-26). | J-88:L21-L21, L24-L25, L35-L35 |
| J88-E012 | E/INT | Production reachability E2: J-90 blast-cone query returns a runtime-only edge a static cone missed (via auditor's J-90 run). E3: AUDITING transition renders binary verdict off machine_view on a real run. Contracts-only never satisfies R7. | J-88:L37-L40 |
| J88-E013 | E/OBS | Observable E2: trait-object call run then J-90 query includes runtime dependent with provenance=runtime (cone WIDENED). NEGATIVE: shrink edge REJECTED; edge without provenance FAILS ingest. | J-88:L47-L50 |
| J88-E014 | E/OBS | Observable E3: audit with misleadingly-named fn (`safe_delete` unsafe) still renders binary verdict off coverage/SARIF, NOT the fn name. NEGATIVE: verdict IDENTICAL under rename; feeding source into binary lane FAILS input contract. | J-88:L48-L50 |
| J88-E015 | E/LIMIT | E2 residual unsoundness: runtime edges only cover paths that ACTUALLY RAN; cone must DECLARE corpus-bounded runtime-edge coverage, never claim dynamic completeness. | J-88:L62-L62 |
| J88-E016 | E/LIMIT | E3 advisory tension: stripping source can hide bugs prose would catch — LLM-auditor ADVISORY source-bearing lane still runs in parallel; document both lanes. | J-88:L63-L63 |
| J88-E017 | E/DEP | COMPOSES-WITH DKG/B-29, G-2, J-90 L5 (primary E2 consumer), binary gates wiring/F-4/J-16/C-28, J-87 observer feed. | J-88:L58-L59 |
| J88-A001 | A/CFG | Promotion criteria from OBSERVE-ONLY flags to default enforcement for either enable flag are not quantified beyond allowlist promotion plan. | J-88:L26-L26 |
| J88-A002 | A/DATA | Exact `observed_calls` edge schema fields (caller/callee/count/trace-id) and versioning rule for DDL epoch bump are not specified. | J-88:L8-L8, L24-L24 |

Category coverage: DKG capability ✓; GraphViewer surface ✓; audit input routing ✓; flags ✓; integrations ✓; security (no source in binary lane) ✓; failures/reject shrink ✓; observability ✓; tests via J-90/audit triggers ✓; constraints/residue ✓. No new SSB; no new CLI verb named beyond existing audit/blast paths.

### J-100 — time-as-control plane

Source: `docs/thefinalglance/J-plan/J-100_time_as_control_plane.md` (130/130 lines).

| Atomic ID | Class/domain | Explicit discovery or bounded interpretation | Exact anchor |
|---|---|---|---|
| J100-E001 | E/STATUS | J-100 is PLANNED WAVE 1 (parallel with J-99), PLANE-2 technical half of time-awareness design, BOUND by K-14 TZ-4; mostly WIRING/EXPOSURE (~80% built); net 0 new SSB; effort M; lever CANT-GET-STUCK. | J-100:L3-L5 |
| J100-E002 | E/CAP | Guarantees the session never hangs on a hung/long tool: supervisor-owned liveness + decoupling, independent of what the model says (PLANE 1 J-99 is context-only). | J-100:L6-L6, L10-L10 |
| J100-E003 | E/FLOW | THE LOOP (notification-primary, poll-fallback): (1) model issues tool with `run_in_background=true` (or supervisor auto-backgrounds) -> returns task_id IMMEDIATELY; (2) model KEEPS WORKING — supervisor owns process; (3) on completion/failure/watchdog-fire, AgentNotificationEmitted delivered as synthetic observation on NEXT re-entry; (4) model MAY task_status(task_id) (J-2) and TaskStop(task_id). | J-100:L12-L16, L47-L47 |
| J100-E004 | E/CAP | Watchdog (I-17 promotion): wrap every supervisor-run command. On fire emit WatchdogTimeout{Idle\|NoMaterialProgress\|Safety} + structured observation (killed + partial output). Session can NEVER hang on a single tool. | J-100:L18-L19, L49-L49 |
| J100-E005 | E/DET | DETERMINISM TZ-4: PRIMARY bound is step/event-count + "no material progress" (I-35-style history predicate, order not duration — replay-stable). Wall-clock duration is SECONDARY, supervisor-plane-only, OUTSIDE seeded replay set: can kill hung REAL process but is RECORDED not gate-evaluated; RRA re-run replays recorded outcome, never re-times wall clock. | J-100:L20-L20, L70-L72, L97-L97 |
| J100-E006 | E/CFG | INTERACTIVE vs AUTONOMOUS (MASTER 412 / D-3): wall-clock watchdog defaults ON only in AUTONOMOUS mode. INTERACTIVE has no wall-clock timeout (human is the pace); step/idle watchdog still applies. | J-100:L21-L21, L26-L26, L67-L67, L71-L71 |
| J100-E007 | E/CFG | AUTO-BACKGROUND (D-4): v1 backgrounds ONLY when model explicitly asks; supervisor auto-background on observed-elapsed ships OBSERVE-ONLY first behind a flag (GD-4 — emit would-auto-background decision, measure, before defaulting ON). | J-100:L27-L27, L48-L48, L95-L95 |
| J100-E008 | E/SEC | BAD-TOOL-USE IMMUNITY three layers: I-46 bash interceptor REFUSE-WITH-REDIRECT; existing per-tool timeouts (mcp_bridge/post_tool_use/web_validator); watchdog universal backstop. Bad/long tool degrades to killed+reported+keep-going, never frozen session. | J-100:L23-L23, L50-L50, L98-L98 |
| J100-E009 | E/STATUS | Existing parts: G3 background_task_id WIRED (tools/lib.rs:492/858/5834-5850/2628-2630); G4 BackgroundTaskManager PARTIAL + BackgroundTaskStarted/Complete; G5 WatchdogTimeout type PARTIAL with I-17 Wave-3 STUB (action_type_lock.rs:692); G6 AgentNotificationEmitted PARTIAL; G10 I-1 supervisor PLANNED; G11 J-2 task_status PLANNED. | J-100:L30-L36 |
| J100-E010 | E/DEP | Soft-dep J-2: loop ships notification-primary + TaskStop without J-2; task_status is follow-on poll verb. | J-100:L35-L35, L125-L125 |
| J100-E011 | E/EVENT | net 0 new action_type — reuses BackgroundTaskStarted/Complete, AgentNotificationEmitted, WatchdogTimeout. live baseline ~268/577 (W-9). | J-100:L37-L37, L80-L80, L99-L99 |
| J100-E012 | E/CAP | Tasks: T-4 async-tool loop exposure; T-4 auto-background OBSERVE-ONLY; T-5 I-17 watchdog promotion; §3.4 immunity compose; J-100-decisions.md. | J-100:L46-L51 |
| J100-E013 | E/INT | Production reachability: live `elai run` background tool returns task_id immediately + BackgroundTaskStarted, turn does not block; completion lands AgentNotificationEmitted + BackgroundTaskComplete on next re-entry; hung tool AUTONOMOUS -> WatchdogTimeout; hung tool INTERACTIVE -> NOT wall-clock-killed. Contracts never satisfy R7. | J-100:L64-L68 |
| J100-E014 | E/LIMIT | Stub bans: no wall-clock as PRIMARY bound or gate/transition input (TZ-3/TZ-4); no wall-clock kill in INTERACTIVE; no re-timed wall clock on RRA replay; no tight task_status poll as primary (IMP-8); no general cron/scheduler (R-5; I-53 declined); no re-implementing I-46 or per-tool timeouts. | J-100:L69-L75 |
| J100-E015 | E/SURFACE | frontend_pair: background-task lifecycle + watchdog fires on existing H-36 AgentNotification dashboard tile (commands_h36.rs) + watchdog row on SSB timeline; dedicated task-list tile decide at edit (W-4) or W-6-justify. | J-100:L76-L77 |
| J100-E016 | E/TEST | Contract `j100_async_tool_loop_contract.rs` (~140 LOC): (a) background sleep immediate task_id + Started + non-blocking + notification/Complete; (b) hung AUTONOMOUS -> WatchdogTimeout + kill observation; (c) hung INTERACTIVE NOT wall-clock-killed; (d) RRA replay recorded kill byte-identical; (e) wall-clock-as-PRIMARY or wall-clock-in-gate FAILS determinism; (f) bad bare command REFUSE-WITH-REDIRECT, session continues. SimpleSpanProcessor only. | J-100:L106-L107 |
| J100-E017 | E/TEST | DETERMINISTIC_CHECK: wiring_contract_check; production_caller_check on tools/lib.rs; WatchdogTimeout production call-site present; task_status/TaskStop/notification seam present; wall-clock NOT a gate/transition/promotion/seed input; decisions doc exists. | J-100:L83-L90 |
| J100-E018 | E/DATA | Files: MODIFY tools/src/lib.rs (loop contract + OBSERVE-ONLY auto-background flag); MODIFY orchestrator (I-17 promotion + BackgroundTaskManager + H-36 delivery); CREATE J-100-decisions.md. | J-100:L109-L112 |
| J100-E019 | E/DEP | Pairs with J-99 (PLANE 1 context); K-14 banks IMP-9 differentiator (decoupled async tools in deterministic replayable taint-tagged journal) + TZ-4. | J-100:L119-L119, L128-L128 |
| J100-E020 | E/FAIL | Risks: R-3 false-kill mitigated by no-material-progress primary + autonomous-only wall-clock; R-4 notification race mitigated by task_status reconcile + start/complete rows; R-5 scope creep into cron forbidden. | J-100:L121-L125 |
| J100-A001 | A/CFG | Exact step-count / no-material-progress predicate thresholds and autonomous wall-clock budget defaults are not numeric in-source. | J-100:L20-L21, L49-L49 |
| J100-A002 | A/SURFACE | Whether a dedicated task-list tile is added vs H-36-only is deferred to edit (W-4/W-6). | J-100:L76-L77 |
| J100-A003 | A/DEP | I-1 supervisor is PLAN-asserted PLANNED; temporary exposure path if I-1 has not landed is not specified beyond consuming the planned plane. | J-100:L34-L34, L47-L47 |

Category coverage: model-facing async loop surface ✓; H-36 notification tile ✓; workflows/states ✓; config D-3/D-4 ✓; integrations/events (reuse) ✓; security/immunity layers ✓; failures/kill observations ✓; determinism/TZ-4 ✓; tests/contract ✓; constraints/risks ✓. No new CLI verb beyond existing `elai run` tool surfaces; TaskStop/task_status are model-facing tools.

## Cross-source interactions and lifecycle

| Interaction ID | Class | Interaction, ordering, or shared invariant | Evidence atoms |
|---|---|---|---|
| IX-001 | E | J-3 and J-4 share session-state shapes (HG-4): J-3 writes backups + last-good marks; J-4 optionally reads last-good seq and union-derives FILES TOUCHED; they coordinate schema reading, not files, and are parallel-safe except that coordination. | J3-E006–J3-E007, J3-E027; J4-E004, J4-E006 |
| IX-002 | E | J-4 template discipline and J-6 verdict-file discipline both apply the handoff one-file rule, but to different artifacts (handoff entry vs reasoning verdict file); J-6 cites the rule and must not fork it. | J4-E005, J4-E009; J6-E004, J6-E013 |
| IX-003 | E | J-6 names J-4 ledger companions and J-1 /see as inheritors of one-artifact-at-a-time spine; J-3/J-4 are harness-safety predecessors for later waves including J-5 port. | J6-E020; J3-E027; J4-E004 |
| IX-004 | E | J-19 and J-13 share the RRA seed / token-determinism contract: vLLM `VLLM_BATCH_INVARIANT` is the engine-native path to the same determinism J-13 vendors for HF. | J19-E007, J19-E010, J19-E018 |
| IX-005 | E | J-19 is additive backend #12 on B-37 RuntimeAdapter; J-51 role routing also consumes B-37 PerRoleRuntime/try_complete — disjoint legs (engine selection vs adherence-aware role assignment). | J19-E002, J19-E005; J51-E006, J51-E008 |
| IX-006 | E | J-23 extends F-1/F-2 VibeGate as external Layer-1 review backends; J-88 E3 re-routes INPUT of the same binary-gate family (wiring/F-4/J-16/C-28) to machine-derived artifacts — catalog findings and machine_view both feed audit, different layers. | J23-E001, J23-E009; J88-E005–J88-E006, J88-E017 |
| IX-007 | E | J-23 structural catalog shape mirrors J-9 hardware catalog; both are probe-once data-driven TOML + thin runner. | J23-E003, J23-E009 |
| IX-008 | E | J-26 ABI is a sibling channel into H-43 FitnessEvaluated emission, not a scorer replacement; E-54 ingest CLI closes the pack entrypoint gap and feeds the same fitness harness. | J26-E002, J26-E017–J26-E018, J26-C001 |
| IX-009 | E | J-26 R8 freezes the evaluator fileset; J-45 and J-51 both ban LLM-as-judge on load-bearing paths (D5/TD-3) — frozen mechanical evidence and advisory LLM are complementary isolation patterns. | J26-E003, J26-E021; J45-E002–J45-E003; J51-E004, J51-E007 |
| IX-010 | E | J-45 depends on J-43 floor + J-44 panel routing unmechanizable findings into calibrated cross-family advisory with escalation teeth; J-46 seeds its corpus and measures debt-rate. | J45-E001, J45-E009–J45-E010, J45-E015 |
| IX-011 | E | J-51 mirrors J-34's bench->profile->artifact->G-10-invalidation shape with a disjoint metric (`tool_call_adherence` vs context-fill curves) on the same ModelCapabilityProfile struct. | J51-E005, J51-E008–J51-E010 |
| IX-012 | E | J-51 advisory floor breach rides I-7 EscalationCascade (cloud spill); grammar mask still rescues format so sub-floor degrades rather than crashes. | J51-E006–J51-E008 |
| IX-013 | E | J-88 E2 is the unlock root for J-90 L5 dynamic/cross-language blast cone; widen-only runtime edges enrich B-29 cones that J-90 fuses with SCIP/cross-lang seams. | J88-E001–J88-E003, J88-E012, J88-E017 |
| IX-014 | E | J-88 E3 hardens J-90 L4/L7 and J-87 reconcile by forcing binary gates onto machine-derived inputs while advisory source-bearing lane remains parallel. | J88-E005–J88-E006, J88-E016–J88-E017 |
| IX-015 | E | J-100 is PLANE 2 to J-99 PLANE 1; both obey K-14 TZ-4 and time-determinism doctrine; J-100 provides control that prompt timestamps cannot. | J100-E001–J100-E002, J100-E005, J100-E019 |
| IX-016 | E | J-100 composes I-1 supervisor decoupling, J-2 poll surface (soft-dep), I-46 interceptor, H-36 notifications, and existing BackgroundTaskManager/WatchdogTimeout types — promotion and exposure, not invention. | J100-E003–J100-E004, J100-E008–J100-E010 |
| IX-017 | E | Harness CLI tools in this chunk (undo.py, handoff_ledger.py, incremental-eval skill) are stdlib/skill-side and deliberately N/A for dashboard frontend_pair; product UI surfaces concentrate in J-19 panels, J-23 vibesec/QA, J-51 cockpit tile, J-88 GraphViewer, J-100 H-36 tile. | J3-E021; J4-E014; J6-E016; J19-E012; J23-E012; J51-E012; J88-E010; J100-E015 |
| IX-018 | E | SSB policy split: J-19 prefers net 0 reuse; J-23 and J-51 each add one Pattern-2 action type; J-26/J-45/J-88/J-100 prefer net 0; all require live baseline re-grep (W-9) rather than frozen constants. | J19-E011; J23-E011; J26-E029; J45-E012; J51-E011; J88-E011; J100-E011 |
| IX-019 | E | Observe-first / non-blocking defaults recur: J-3 backup never blocks; J-6 defers enforcement hooks; J-19/J-23/J-88 flags not default-ON until measured; J-51 gate is advisory; J-100 auto-background OBSERVE-ONLY. | J3-E014, J3-E024; J6-E014; J19-E015; J23-E014; J51-E006; J88-E009; J100-E007 |
| IX-020 | I | Dashboard DESIGN.md consumers must treat harness tools (J-3/J-4/J-6) as operator CLI/protocol surfaces, not missing dashboard defects; product tiles only where sources name frontend_pair consumers. | J3-E021; J4-E014; J6-E016; IX-017 |

## Explicit ambiguity and conflict register

| Register ID | Kind | Unresolved issue or required correction | Atom references |
|---|---|---|---|
| AC-001 | ambiguity/data | J-3/J-4 session-state schemas are PLAN-asserted; dual-shape readers degrade honestly but exact field contracts are not enumerated. | J3-A001; J4-A001 |
| AC-002 | ambiguity/config | J-3 gzip ~64KB threshold and exact settings registration JSON string are not finalized constants. | J3-A002–J3-A003 |
| AC-003 | ambiguity/config | J-4 "newest live session" default resolution and J-6 topic-key path function lack concrete algorithms. | J4-A002; J6-A002 |
| AC-004 | ambiguity/trigger | J-6 trigger precision vs other skills is intentionally observation-tuned. | J6-A001 |
| AC-005 | ambiguity/integration | J-19 DoLa-dependent role inventory, optional +1 metric, and exact panel layout remain edit-time decisions. | J19-A001–J19-A003 |
| AC-006 | ambiguity/config | J-23 initial catalog growth plan and FP-rate BLOCK threshold are qualitative only. | J23-A001–J23-A002, J23-I001 |
| AC-007 | conflict/scope | J-26 ABI is sibling-not-replacement after redteam patch; executors must not treat it as scorer substitution. | J26-C001 |
| AC-008 | ambiguity/schema | J-26 GAP-1 NewTaskInput mapping and dual CLI verb alias need recon before S4. | J26-A001, J26-A003 |
| AC-009 | ambiguity/ops | J-45 corpus labelling ownership, outage policy, kappa cadence, and override-rate health signal are open. | J45-A001–J45-A004 |
| AC-010 | ambiguity/config | J-51 `TOOLCALL_ADHERENCE_FLOOR` numeric value and fixture inventory are unspecified. | J51-A001, J51-A003 |
| AC-011 | ambiguity/schema | J-88 observed_calls field schema and flag-promotion criteria are incomplete. | J88-A001–J88-A002 |
| AC-012 | ambiguity/config | J-100 watchdog numeric budgets and dedicated task-list UI remain open; I-1 readiness fallback unspecified. | J100-A001–J100-A003 |

## Summary additions and corrections

Orientation comparison target: `docs/thefinalglance/feature-discovery/10-J-SUMMARY_INDEX.discovery.md`. “ADD” means an explicit source behavior omitted or materially compressed by the summary; “CORRECT/QUALIFY/REMOVE” means the summary’s wording changes scope or certainty. These are relational records, not additional feature atoms.

### J-3 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J3-001 | **ADD** exact effort, stdlib-only tier, HG-1..HG-6 inventory, and mandatory pre-coding read list. | 10-J:L57-L67 | J3-E001, J3-E008–J3-E012 |
| SD-J3-002 | **ADD** full backup path shape (`hash[:12]/relpath}@{seq}`), observe-only error path, user-reviewed registration, and selftest arms a–f. | 10-J:L59-L67 | J3-E013–J3-E015, J3-E019, J3-E025 |
| SD-J3-003 | **ADD** `--force-after-review` divergence restore detail, prune dry-run default, decisions-doc contents, and >=3-session friction gate before any future blocking. | 10-J:L62-L67 | J3-E007, J3-E016–J3-E017, J3-E020, J3-E024 |
| SD-J3-004 | **ADD** DETERMINISTIC_CHECK commands and JC-1/JC-2 import bans as acceptance instruments. | compressed | J3-E005, J3-E023 |

### J-4 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J4-001 | **ADD** exact row field set including first_seq/last_seq, path-sorted paste block, `--json`, and default newest-session flag. | 10-J:L69-L76 | J4-E003, J4-E007 |
| SD-J4-002 | **ADD** exact degradation string `LEDGER UNAVAILABLE...`, exit 1, selftest arms, decisions doc, and first-real-handoff JD-1 acceptance. | 10-J:L75-L76 | J4-E008, J4-E010–J4-E011, J4-E016 |
| SD-J4-003 | **ADD** dual-shape defensive read requirement and pre-hook session UNAVAILABLE policy. | 10-J:L72-L72 | J4-E006, J4-E013, J4-E017 |

### J-6 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J6-001 | **ADD** four ordered protocol steps, BeliefState working-set definition, and named anti-pattern list including pre-complete synthesis ban. | 10-J:L87-L95 | J6-E004–J6-E008 |
| SD-J6-002 | **ADD** no CLAUDE.md skills-list entry this pass, deferred enforcement-hook rule, validator + >=4/4 eval pair, and prereading validation path. | 10-J:L93-L95 | J6-E009–J6-E015, J6-E018 |
| SD-J6-003 | **ADD** cites-not-forks of token-discipline/handoff/I-5/I-6/I-11 and J-1/J-4 successor notes. | compressed | J6-E013, J6-E020 |

### J-19 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J19-001 | **ADD** full logits portability table including SGLang column and keep-on-HF column; prompt-lookup V0-fallback detail. | 10-J:L229-L229 | J19-E003–J19-E004 |
| SD-J19-002 | **ADD** E-27 AWQ never-built status, calibration-corpus requirement, SM 8.6/3090 verification, and offline build tooling. | 10-J:L228-L228 | J19-E006, J19-E014 |
| SD-J19-003 | **ADD** net-0 SSB preference, RuntimeBackendSelected reuse, W-10 remote-GPU gate, and anti-slopsquat pin rules. | 10-J:L232-L233 | J19-E011, J19-E013, J19-E015 |
| SD-J19-004 | **ADD** excluded TensorRT/candle/mistral.rs and exact negative capability checks (DoLa-on-vLLM, uncalibrated AWQ, false cache claims). | 10-J:L231-L233 | J19-E009, J19-E014 |

### J-23 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J23-001 | **ADD** complete arch-tool list (import-linter, Packwerk), catalog fields including license/staleness_flag, and dropped tools with reasons. | 10-J:L267-L273 | J23-E005, J23-E008, J23-E019 |
| SD-J23-002 | **ADD** ReviewGateCatalogEvent kinds, dogfood tool matrix with +nightly udeps, and NO BLOCK until FP>=3-runs rule. | 10-J:L270-L273 | J23-E006, J23-E011, J23-E014 |
| SD-J23-003 | **ADD** start-with-SARIF-core growth policy, license-refuse semantics, and vibesec + dashboard QA dual surface. | 10-J:L272-L273 | J23-E012, J23-E017–J23-E018 |

### J-26 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J26-001 | **ADD** entire ship-order S1–S8 micro-slice program with production line anchors, FORK/ABORT/VERIFY/REDTEAM/SMOKE sections. | 10-J:L296-L304 compressed | J26-E013–J26-E028 |
| SD-J26-002 | **ADD** sibling-channel (not replacement) refined scope, in-process fallback RISK-3, WIRE-DARK marker lifecycle. | 10-J:L298-L304 | J26-E016–J26-E017, J26-C001 |
| SD-J26-003 | **ADD** dual CLI aliases, NewTaskInput GAP-1 recon, out-of-band S7 re-lock fork, and net-0 SSB preference. | compressed | J26-E018, J26-E021, J26-E023, J26-E029 |
| SD-J26-004 | **ADD** riders B1/B4/B5/B6 and Tier-2 live-generation deferral. | 10-J:L303-L304 | J26-E005, J26-E007 |

### J-45 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J45-001 | **ADD** EXP-DP3 observe-only gate language, three-class ground-truth labels, synthetic DD-* + boilerplate-ATTACK seeding, and held-out un-gamed subset. | 10-J:L500-L508 | J45-E003–J45-E004 |
| SD-J45-002 | **ADD** gpt_chat AND grok_chat as second/third family, serialize-in/no-repo constraint, and deep-research exclusion. | 10-J:L502-L502 | J45-E005, J45-E008 |
| SD-J45-003 | **ADD** exact debt disposer string `single-family-degraded`, net-0 SSB, artifact path `data/doctrine-calibration/*.jsonl`, and open labelling/outage/kappa-drift risks. | 10-J:L508-L508 | J45-E007, J45-E012, J45-E014, J45-A001–J45-A004 |

### J-51 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J51-001 | **ADD** three-rate scoring (selection/schema/semantic), [0,1] score + metrics TOML, model-identity-keyed artifact store, and Guava domain caveat. | 10-J:L561-L570 | J51-E003–J51-E004, J51-E016 |
| SD-J51-002 | **ADD** tool-heavy role list (executor/researcher), named const floor, I-7 cascade detail, and W-2 kill-switch-not-enable rule. | 10-J:L566-L567 | J51-E006, J51-E015 |
| SD-J51-003 | **ADD** full ToolCallAdherenceMeasured kinds, SCHEMA_VERSION computed formula, cockpit tile, Tier-2 curves/recalibration/journal-regression, and J-48 rider. | 10-J:L569-L570 | J51-E009, J51-E011–J51-E012, J51-E017 |

### J-88 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J88-001 | **ADD** exact runtime sources (G-2, L3, L4), provenance-tagged ingest, GraphViewer legend surface, and residual corpus-bounded completeness declaration. | 10-J:L928-L935 | J88-E002, J88-E010, J88-E015 |
| SD-J88-002 | **ADD** full machine_view input inventory (coverage, SARIF IDs, J-87 syscall classes, unmatched counts, digests), dual-lane advisory tension, and OBSERVE-ONLY dual enable flags. | 10-J:L933-L935 | J88-E005, J88-E009, J88-E016 |
| SD-J88-003 | **ADD** rename-invariance done-gate for E3 and reject-shrink/no-provenance negatives for E2. | 10-J:L934-L935 | J88-E013–J88-E014 |

### J-100 deltas

| Delta ID | Summary change | Summary anchor | Source atoms |
|---|---|---|---|
| SD-J100-001 | **ADD** full four-step loop teaching text, assistant_auto_backgrounded observe-only path, and TaskStop availability without J-2. | 10-J:L979-L988 | J100-E003, J100-E007, J100-E010 |
| SD-J100-002 | **ADD** WatchdogTimeout kinds Idle/NoMaterialProgress/Safety, MASTER 412 interactive exception, TZ-4 recorded-not-evaluated wall-clock rule, and no-cron R-5 ban. | 10-J:L985-L988 | J100-E004–J100-E006, J100-E014 |
| SD-J100-003 | **ADD** three-layer bad-tool immunity composition, H-36 tile frontend_pair, j100 contract arms a–f, and D-3/D-4 decision markers. | compressed | J100-E008, J100-E015–J100-E016 |
| SD-J100-004 | **ADD** existing G3–G11 inventory (wired vs partial vs planned) and soft-dep note that T-4 must not block on J-2. | 10-J:L981-L983 | J100-E009–J100-E010, J100-E020 |

## Exhaustiveness, zero-omission, and duplicate audit

- Source coverage: **10/10 files**, **1,247/1,247 logical lines**, **127,107/127,107 bytes** read and reconciled; source SHA mismatches **0**.
- Atomic source ledger: **233 unique atoms** = **203 explicit**, **002 inferred**, **027 ambiguous**, **001 conflicting/corrective**.
- Per-file atom counts: J-3 **031**; J-4 **020**; J-6 **022**; J-19 **021**; J-23 **023**; J-26 **034**; J-45 **020**; J-51 **020**; J-88 **019**; J-100 **023**. Sum **233**.
- Relational interaction records: **20** (`IX-001`–`IX-020`). Summary-delta records: **034** (`SD-*`). Ambiguity/conflict register: **12** (`AC-001`–`AC-012`). These records reference source atoms and are not double-counted as features.
- ID audit: every atomic table row has exactly one unique ID matching `J3/J4/J6/J19/J23/J26/J45/J51/J88/J100-[EIAC]nnn`. Duplicate atomic IDs **0**; missing/blank atomic IDs **0**; nonconforming atomic IDs **0**.
- Semantic duplicate audit: repeated premise/scope/wiring/grounding/acceptance statements were merged into one atom with multiple anchors. Interactions, conflict register, and summary deltas reference atoms instead of cloning them. Duplicate feature atoms found **0**.
- Omission audit: every source was checked against capabilities/actions/surfaces; workflows/states; schemas/data/lifecycle; flags/defaults/config; integrations/events; permissions/security/privacy/compliance; failures/fallback/retry/recovery; determinism/idempotency/concurrency; observability/evidence; tests/acceptance/done gates; constraints/non-goals; dependencies/status/deferred. Each per-file ledger explicitly names absent dimensions rather than inventing them. User-facing surfaces captured as own atoms: `undo.py` list/diff/restore/prune/selftest CLI (J-3); `handoff_ledger.py` CLI + handoff FILES TOUCHED machine block (J-4); incremental-eval skill trigger/protocol (J-6); `elai run --local` backend flag + RuntimePanel/CloudPanel comparison table (J-19); `elai` review/audit path + vibesec/audit panel + dashboard QA a11y/token surface + arch_gate/SARIF catalog (J-23); `elai eval-corpus-ingest` / `ingest-task-pack` CLI + existing H-43 panels (J-26); handoff DECISION escalation entries + calibration corpus files (J-45, no dashboard-only path); `elai bench tool-call-adherence` + model/cost-cockpit adherence tile (J-51); GraphViewer edge-kind legend (J-88); H-36 AgentNotification tile + SSB watchdog timeline + model-facing run_in_background/task_status/TaskStop (J-100).
- Interpretation audit: inferred, ambiguous, and conflicting/corrective atoms are visibly separated by ID/class. No inference is promoted to a requirement. Orientation-only claims remain in delta records and are not source atoms.
- Manifest-boundary audit: the only original sources opened were the ten exact DEEP-J-009 manifest rows. The only orientation artifact consulted was `10-J-SUMMARY_INDEX.discovery.md`. Format reference consulted: `DEEP-I-004.discovery.md`.
- Output metrics after finalization: `output_lines=0497`; `output_bytes=082878`.
- Canonical normalized output SHA-256: 2dc0731421b01726a8f87dbfc458e5c9269bd5276454ecaae7509b27e6ef1ba6
- Hash rule: compute SHA-256 over the exact final UTF-8 bytes including the final newline after replacing only the 64 hexadecimal characters on the `Canonical normalized output SHA-256` line with 64 ASCII zeroes. Fixed-width output metrics and digest substitution avoid impossible raw self-reference.

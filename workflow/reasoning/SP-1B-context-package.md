SP-1B CONTEXT PACKAGE
(placed by orchestrator from the context-compiler's inline return — agent's Write was hook-blocked during SP-1A AUDITING/RUN_TESTS)
Task: TEIKOKU-ELAI-FINISH-PLAN.md units 1.2 (M3 write-set backfill at import) + 1.6 (importer regex widen [B-K]>[B-N] + guard routing). session_id=s-finish-0705.
Target file: \\wsl.localhost\teikoku\home\teikoku\teikoku\conductor\backlog_import.py (705 lines, current as of this session; edited Jul 5 same day SP-1A added precheck wiring).
Sibling files (same conductor dir, WSL distro teikoku): guard.py, scope_utils.py, gate_runner.py, models.py, plan_schema.py, worker_adapter.py, worker_providers.py.

(a) CURRENT IMPORT FLOW + EXACT LINES TO CHANGE
parse_plan_file() [backlog_import.py:209-232] extracts per-plan blocks via extract_block(content,name) [138-141], a regex requiring a LITERAL "## <name>" markdown heading (case-sensitive, MULTILINE, stops at next "## " or EOF). Populates ParsedPlan.wiring_contract (raw "## WIRING_CONTRACT" text, unparsed further except reachable_from) and ParsedPlan.files_touched (raw "## Files touched" text).
build_task() [436-490] calls, at line 449:
  allowed_files = extract_allowed_files(plan.files_touched, plan.path.read_text(encoding="utf-8"))
extract_allowed_files() [182-194] is the ONLY write-set populator today. BUG (verified against real J-100 file, see (c)): its per-line match re.match(r"\s*(CREATE|MODIFY|DELETE|TOUCH|EDIT)\b", line) anchors at line START; a real bullet line is "- MODIFY: path (note)" so position 0 is "-", \s* consumes 0 chars, the verb-alternation never matches -> EVERY labeled bullet line is silently skipped. The fallback (190-194) only fires if zero paths found: it backtick-scans the WHOLE file content for `path.ext` (rs/py/ts/tsx/js/svelte/toml/json only, no .md) — real corpus prose rarely wraps paths in single backticks, so the fallback usually also yields zero. Net: allowed_write_set = [] even when a clean "## Files touched" block exists. This is the "empty-set = deny-all" bug.
NOTHING today parses produces_symbols/consumes_symbols inside "## WIRING_CONTRACT" for file paths — extract_production_hook() [197-206] only pulls reachable_from's first bullet into production_hook.
EXACT EDITS: (1) rewrite extract_allowed_files (182-194) to strip bullet/backtick noise before matching, per (b); (2) add a new extractor for path-like produces_symbols/consumes_symbols bullets and OR its result into the same paths set; (3) in build_task (449) or a small wrapper, if the combined result is still empty, apply the bounded default + WARN row per (d).

(b) LABELED_PATH_RE — VERBATIM, ALREADY SOLVED ELSEWHERE, REUSE
Source: \\wsl.localhost\codex-vm\root\.conductor-codex\conductor\backlog.py:160-177 (architecturally-twin conductor on the codex-vm WSL distro; conductor-j-burn.md patch 3b). Verbatim:
  LABELED_PATH_RE = re.compile(r"^(?:MODIFY|CREATE|DELETE|MOVE|EDIT)\s*:\s*(\S+)", re.I)

  def _extract_paths(block: str) -> list[str]:
      paths: list[str] = []
      for line in block.splitlines():
          candidate = line.strip().lstrip("-*").strip().strip("`")
          if not candidate:
              continue
          labeled = LABELED_PATH_RE.match(candidate)
          if labeled:
              candidate = labeled.group(1).strip().strip("`").rstrip(",;")
          elif " " in candidate:
              continue
          if candidate.startswith("/") or ".." in Path(candidate).parts:
              continue
          paths.append(candidate)
      return paths
The fix vs teikoku's current code: strip .lstrip("-*") (bullet marker) and backticks BEFORE the anchored verb match, THEN match case-insensitively (re.I), THEN if unlabeled and containing a space treat as prose (skip) else treat as a bare path. Also rejects absolute paths and .. traversal. Teikoku's verb set (CREATE|MODIFY|DELETE|TOUCH|EDIT) differs slightly from codex-vm's (MODIFY|CREATE|DELETE|MOVE|EDIT) — union both (add MOVE, keep TOUCH).
codex-vm's PlanRepository ALSO widens block-heading aliases: BLOCK_ALIASES["ALLOWED_WRITES"] = ("ALLOWED WRITE SET","ALLOWED_WRITES","ALLOWED WRITES","SCOPE","FILES TOUCHED") [backlog.py:19-26] — teikoku's extract_block only ever looks for the literal "Files touched". Port the alias set too (cheap insurance).

(c) REAL SHAPES SAMPLED (3 files, read in full/near-full)
1. docs/thefinalglance/J-plan/J-16_blast_radius_regression_gate.md — HAS "## WIRING_CONTRACT" (line 35), NO "## Files touched" anywhere (79-line file). produces_symbols bullets are `path::symbol (prose)`, e.g.:
     - crates/orchestrator/src/blast_radius.rs::compute_cone (reverse-dep cone via the B-29 impact API)
   File path = everything before "::". Write-set UNRESOLVABLE without a produces_symbols parser; Tier-1 finds nothing.
2. docs/thefinalglance/J-plan/J-100_time_as_control_plane.md — HAS BOTH sections. produces_symbols (54-57) is PROSE. "## Files touched" (109-112) is clean:
     - MODIFY: crates/tools/src/lib.rs (the model-facing async-tool loop contract ...)
     - MODIFY: crates/orchestrator/src/ (the I-17 watchdog promotion ...)
     - CREATE: workflow/reasoning/J-100-decisions.md (the D-3/D-4 DECISIONs ...)
   Hand-traced: current extract_allowed_files returns [] on this exact block; ported LABELED_PATH_RE returns all 3 paths.
3. docs/thefinalglance/B-plan/B-40_idea_framer_research_presets.md — HAS "## WIRING_CONTRACT" (177), NO "## Files touched" (checked through ~268; B-plans do not use that convention). produces_symbols bullets are bare RUST MODULE PATHS, e.g.:
     - elai_orchestrator::idea_framing::FramedIdea
   Not deterministically resolvable by regex alone (needs crate/module->file lookup — OUT OF SCOPE).
CONCLUSION: 3-tier extractor: Tier1 "## Files touched" (or alias heading) labeled bullets via ported LABELED_PATH_RE (case 2). Tier2 WIRING_CONTRACT produces_symbols/consumes_symbols bullets whose pre-"::" segment contains "/" AND a known source extension (rs|py|ts|tsx|js|svelte|toml|json|md) (case 1). Tier3 bounded-default fallback (case 3 and anything else).

(d) FALLBACK + WARN MECHANICS (reuse SP-1A's pattern verbatim)
SP-1A added exactly this shape for probes: load_probe_pending_warnings() [backlog_import.py:423-433] appends f"{plan.plan_id}: PROBE_PENDING: {result.warning}" into a flat list, folded into the JSON report as report["probe_pending"] [660, 667] — printed on every import, never silent, never blocking.
REUSE for write-set fallback: write_set_fallback: list[str], rows f"{plan.plan_id}: WRITE_SET_FALLBACK: no parsable Files-touched/produces_symbols path, defaulted to ['.elai_cc/**']", folded into the report at the same place. Bounded default: ['.elai_cc/**'] (finish-plan line 91; scope_utils.path_matches_scope() fnmatch handling [scope_utils.py:28-37]).
SAFETY (verified): '.elai_cc/**' default cannot touch frozen gate paths — scope_utils.path_is_allowed() [45-48] checks is_frozen_gate_path() FIRST (FROZEN_GATE_PATTERNS = .claude/**, gate-manifest.txt/.lock, .git/** [7-16]) before consulting allowed_write_set.

(e) LETTER REGEX WIDEN + GUARD ROUTING
LETTER REGEX: backlog_import.py:37 LETTERS = tuple("BCDEFGHIJK") -> tuple("BCDEFGHIJKLMN"). Consumers: parse_corpus() [312-331], load_status_seeds() [353-359], load_bn_statuses() [362-394]. Confirmed on disk: K/L/M/N-plan dirs exist (L=22, M=6, N=15 per context-compiler; explorer counted L=21/M=6/N=15 non-INDEX). Per-file parsing regexes are ALREADY letter-agnostic (unlike codex-vm's [B-K] hardcodes). Only the LETTERS tuple changes. Benign side effect: no FINISH/bn-status seeds exist for K/L/M/N -> default PLANNED unless precheck seeds apply.
GUARD ROUTING: guard.py defines PROMPT_INJECTION_PATTERNS (17 regexes, re.I) + PromptGuard.check(text) via plain pattern.search [99-120] — no context awareness. TODAY invoked ONLY at worker-dispatch (worker_adapter.py:217-218; worker_providers.py:271/285; pre_guard_block() -> BLOCKED TaskResult error_category="guard_blocked"). backlog_import.py has NO import guard today.
VERIFIED trip class: J-102's own text contains "known injection phrases "ignore previous"/"forget instructions"" (discusses the attack, doesn't attempt it) — textbook false positive. See exploration file for the CONFIRMED live J-62 trip via objective-fallback capture.
ROUTING DESIGN (existing vocabulary, no schema change): models.py "NEEDS_REVIEW" legal in TASK_STATUSES/PLAN_STATUSES [21,32] + TERMINAL_STATUSES [35]. At import: one guard.PromptGuard() over objective+wiring_contract text; if .blocked -> force status "NEEDS_REVIEW" + warning row f"{plan.plan_id}: GUARD_BLOCKED: {reasons}" in the report-list pattern. Pre-empts the wasted lease/dispatch cycle; lands in escalation_export.py ATTENTION = ("NEEDS_REVIEW","BLOCKED","FAILED") [39]. Do NOT touch guard.py's patterns.

(f) ENFORCEMENT CONSUMER + TEST PATTERNS
allowed_write_set enforced in gate_runner.py::run_gates() [55-146]: touched_files [106]; violations = scope_violations(touched_files, allowed_write_set) [107] via scope_utils.py:51-56 (any() over scopes — empty set = any([]) = False for every path = every touched file a violation). Violation -> synthetic "allowed_write_enforcement" FAIL [137-146], overall FAIL, BEFORE pack gates. Also enforced worker_adapter.py:353, leases.py [130-149], codex_provider.py:44 (sandbox = "workspace-write" if task.allowed_write_set else "read-only").
TEST/FIXTURE PRECEDENT: audit_smoke_setup.py:73 allowed_write_set=['crates/core/src/lib.rs']; slice_driver.py:131/142 allowed_write_set=["slice_unit.py"].
plan_schema.py validate_plan()/validate_task() [222-247] do NOT check write-set non-emptiness — fix belongs in the extractor, not validation.

SOURCES: finish plan 1-130; conductor-j-burn.md full; teikoku backlog_import.py full (705L); codex-vm backlog.py full (234L); guard.py, scope_utils.py full; gate_runner.py 55-200; models.py 50-140; plan_schema.py 140-270; J-16 full, J-100 full, J-102 1-40, J-12 1-30, B-40 1-268; K/L/M/N dir listings.
TOOLING GOTCHA: the harness Grep tool returns EMPTY under docs/thefinalglance (gitignored tree, ripgrep skips silently). Use Read directly; Glob only scoped to specific subdirs.

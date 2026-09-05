PS-6 WIRE-DARK RATE — FIRST POST-STACK MEASURED BASELINE (2026-07-09)

Ruling: workflow/reasoning/proof-spine-2026-07-05.md (PS-6 row). Rule: .claude/rules/proof-spine.md
(PSR-4). This is a dated ledger row per the ruling's own prescribed method ("no bench binary — a
dated ledger row off production_caller_check + the bn_status_report sweep over the current tree").
KD-1/GD-2/GD-3 framing: this is a receipt, not a superlative. Read the CAVEATS section before citing
this number anywhere.

HEADLINE NUMBER (fn-grain, static, post-enforcement-stack, dated 2026-07-09)
  933 DARK pub fns / 12,137 judged pub fns (WIRED+INTERNAL+DARK+REGISTERED-DARK) = 7.69%
  Alt framing (INTERNAL folded out of the denominator, WIRED+DARK only): 933/9,323 = 10.0%

UNIT OF MEASURE + SCOPE (state this loudly: it is NOT the same unit as the pre-stack anchor below)
  Unit: individual pub Rust symbols (fn/struct/enum/trait/type/const/static) under .elai_cc/crates,
  classified WIRED / INTERNAL / DARK / DARK-TYPE / REGISTERED-DARK / UNSOUND-NAME / SKIPPED-cfgtest
  using the EXACT verdict logic of .claude/tools/production_caller_check.py (R8-frozen, gate-manifest
  line 197), applied to EVERY pub symbol in EVERY non-test-path tracked .rs file (not just git-diff
  lines — production_caller_check.py's shipped CLI is changeset-grain only; there is no --full-tree
  flag today, see RECOMPUTE below for how this sweep extended it as an unshipped read-only script).
  Scope: .elai_cc/crates, 1,156 scan-eligible non-test .rs files (2026-07-09 tree state), 13,544 raw
  pub symbol defs extracted, 13,002 unique after de-dup.
  This is DIFFERENT from the pre-stack anchor's unit, which is PLAN files (a coarser, human/agent-
  adjudicated per-B-N-file verdict blending static + attempted runtime evidence). A plan-grain number
  and a symbol-grain number are not directly comparable; the delta below is DIRECTIONAL, not a rigorous
  before/after on the same measuring stick.

NUMERATOR / DENOMINATOR (raw counts, from the sweep JSON)
  scan_eligible_files:              1,156
  total_pub_symbols_raw:            13,544
  unique_pub_symbols_classified:    13,002
  WIRED:                            8,390
  INTERNAL (same-file transitive):  2,777
  DARK (BLOCK-class, zero prod ref):   933
  DARK-TYPE (non-fn, WARN):           160
  REGISTERED-DARK (WIRE-DARK[..] marker present, WARN):  37
  UNSOUND-NAME (name too common/short to grep, excluded from denominator):  684
  SKIPPED-cfgtest (test-only defs, excluded):              21
  denominator (WIRED+INTERNAL+DARK+REGISTERED-DARK):  12,137
  wire_dark_rate = 933 / 12,137 = 0.07687 = 7.69%

PRE-STACK ANCHOR (for the longitudinal delta — cited, not re-derived; different unit, see above)
  Source: workflow/sessions/s-plan-audit-2026-06-04/FINISH/_MASTER-INDEX.md (session
  s-plan-audit-2026-06-04, completed 2026-06-05; 213 plan files, 220 agents, ~14M subagent tokens).
  Three numbers exist in that doc, all plan-grain, all cited here for honesty:
    (a) 2026-06-05 baseline SYSTEM-WIDE VERDICT TALLY (line 272-280 of _MASTER-INDEX.md):
        SHIPPED_WIRED 60/213 = 28%; SHIPPED_WIRE_DARK 58/213 = 27% (the bucket semantically closest
        to "wire-dark rate" at plan grain); PARTIAL 78/213 = 37%; NOT_STARTED 9/213 = 4%;
        BENCHMARK_PENDING 8/213 = 4%. This is the number the proof-spine-2026-07-05.md ruling itself
        cites as "2026-06-04 audit 28% SHIPPED_WIRED over 213 plans".
    (b) 2026-06-12 fresh re-sweep (session s-107b022a, post-H commit 5f461e4, fresh elai.map,
        hardened evidence protocol): WIRED 89/213 = 41.8%. This is the MORE RECENT pre-my-sweep
        data point (closer in time to today, 2026-07-09) and shows the plan-grain wired rate had
        already risen from 28% to 41.8% by 2026-06-12, before this fn-grain sweep even ran.
  Neither (a) nor (b) is directly divisible into this doc's 7.69%/10.0% fn-grain number: a "plan" can
  ship many pub fns, some wired some dark, and get one blended plan-level verdict; a fn-grain sweep
  has no such blending. Framed honestly: the DIRECTION agrees (wired share up, dark share down) across
  both units and both dates, but the MAGNITUDE is not comparable without a shared unit.

RECOMPUTE (GD-2 — exact commands)
  Layer-0 status read:
    python .claude/tools/proof_spine_status.py --json
  Plan-grain B-plan-only punch list (NOTE: bn_status_report.py is hardcoded to the B-N regex and the
  B-plan directory; it does NOT sweep C..P plan families — see CAVEATS):
    python .claude/tools/bn_status_report.py --filter all --json
    (2026-07-09 result: SHIPPED 16, IN-FLIGHT 2, PLANNED 28, MISSING_FILE 1, out of 47 B-plan rows —
    this is a DIFFERENT, narrower corpus than the 213-plan cross-letter sweep in (a)/(b) above; it is
    NOT used as this doc's headline number, reported here only because the task instructions named it
    as an input to run.)
  Fn-grain changeset-mode spot-check (the shipped, frozen, R8 tool — verifies a single symbol):
    python .claude/tools/production_caller_check.py --symbols <fn_name> --json
  Fn-grain FULL-TREE sweep (this doc's headline number — production_caller_check.py's shipped CLI is
  changeset-only, so the full-tree sweep imports it as a library and applies its exact classification
  function-by-function to every pub symbol under --root, not just a git diff). This sweep script is a
  READ-ONLY scratch artifact (not committed, not R8-frozen, written for this measurement only per the
  task's read-only/no-code-edits scope); its logic, reproducible from the frozen tool's own exported
  functions, is:
    ```
    import production_caller_check as pcc
    scanner = pcc.CallerScanner(Path(".elai_cc/crates"))
    for path in scanner.files:                      # non-test .rs files only (pcc.is_test_path)
        stripped, spans = scanner._stripped(path)    # pcc.strip_comments_and_strings, cached
        for kind, name in pcc.PUB_DEF.finditer(stripped):   # ALL pub symbols, not just diff-added
            # same exemption order as the CLI: cfg-test span -> WIRE-DARK[..] marker (REGISTERED-DARK)
            # -> COMMON_FN_NAMES/short-name (UNSOUND-NAME) -> scanner.production_refs(name, path)
            # WIRED if refs found; else pcc.in_file_refs-equivalent for INTERNAL; else DARK (fn) /
            # DARK-TYPE (non-fn).
    ```
  A future PS-6 re-measure should either re-run an equivalent full-tree sweep, or (proposed, NOT
  actioned by this read-only task) ship a `--root-sweep` flag on production_caller_check.py itself so
  the frozen tool natively supports whole-tree mode and this becomes a one-line recompute — that is a
  build decision for the orchestrator, out of this task's read-only scope.
  Cross-check performed (spot verification the sweep's logic matches the real frozen tool):
    python .claude/tools/production_caller_check.py --symbols wcag_aaa_pass,broker_resolve_anthropic_auth --json
    -> wcag_aaa_pass: DARK (matches sweep). broker_resolve_anthropic_auth: WIRED via --symbols mode
    (3 external refs found) — the full-tree sweep instead classifies it REGISTERED-DARK because a
    WIRE-DARK[..] marker sits above its definition at auth_broker.rs:198, and the tool's own exemption
    order checks the marker BEFORE computing refs (same order in the CLI itself: classify_definition's
    exempt check runs before scanner.production_refs is ever called). This is consistent behavior, not
    a sweep bug, but flags that marker as POSSIBLY STALE — the fn now has 3 real external callers per
    --symbols mode, so the original "ships dark, no consumer yet" rationale behind the marker may no
    longer hold. Not fixed here (read-only scope); worth a follow-up ticket.
    python .claude/tools/production_caller_check.py --symbols dropped_row_count --json
    -> WIRED (1 ref, live_cli.rs:1647). This is the exact symbol production-caller-gate.md's "KNOWN
    TRUE-POSITIVE ON THE CURRENT TREE (2026-06-13)" section names as DARK. It has since been wired —
    confirms real progress on at least that named case, independent of the aggregate number above.

CAVEATS (what this number does and does NOT capture)
  - Layer-2 static name-grep only (R5, production-caller-gate.md): a reference is "the name appears in
    a non-test, non-import line elsewhere". False-WIRED is possible (unrelated same-name symbol, macro
    token match). False-DARK is possible (trait-dispatch, macro-generated call sites, multi-line pub
    fn signatures that don't match the regex). The tool's own docs call this residual risk explicit;
    this sweep inherits it unchanged.
  - UNSOUND-NAME (684 symbols, ~5% of raw defs) are EXCLUDED from the denominator entirely — common or
    short fn names (new/get/run/... or <4 chars) where a bare grep proves nothing in either direction.
    If a meaningful fraction of those are actually dark, this number under-counts; if most are wired
    via trait dispatch invisible to grep, it doesn't matter either way — genuinely indeterminate by
    this method, resolvable only via codegraph_callers (out of scope for a static sweep).
  - INTERNAL (2,777, ~23% of the denominator) means "a same-file non-test reference exists" — a
    transitive helper. The tool treats this as WARN, not wired, because the auditor is supposed to
    verify the calling chain reaches a real entrypoint. This sweep did NOT walk that chain (that would
    require a second full BFS over the call graph); INTERNAL symbols are counted toward the "not dark"
    side by default, which is optimistic. The alt framing above (10.0%, INTERNAL excluded from the
    denominator) is the pessimistic bound.
  - REGISTERED-DARK (37) are explicitly allowlisted per D-030's legitimate-dark escape (a WIRE-DARK[..]
    marker + backing row) — these are BY DESIGN not counted as DARK, matching the frozen tool's own
    semantics. wire-dark-allowlist.toml currently carries 135 `[[...]]` rows (a different, broader
    registry than just fn-level WIRE-DARK markers — includes default-OFF flags per R9); not all 135
    rows correspond 1:1 to a WIRE-DARK[..] code marker this sweep would see.
  - "Shipped unit" definition mismatch vs the pre-stack anchor: the anchor's denominator is 213 PLAN
    FILES (a human/agent-adjudicated unit that can span many fns and both static + attempted runtime
    evidence); this sweep's denominator is 12,137 individual PUB SYMBOLS via pure static name-grep.
    Do not read "7.69% vs 27-41.8%" as "wire-dark rate improved 3.5x-5.4x" — that arithmetic assumes a
    shared unit that does not exist. The honest claim is directional only (see PRE-STACK ANCHOR above).
  - bn_status_report.py could not extend this ledger to the full 213-plan (or larger, since the corpus
    has grown to B..P plan families since 2026-06-04) cross-letter denominator: its BN_RE/BN_FILE_RE/
    TITLE_RE regexes are hardcoded to the literal `B-` prefix, so pointing --bplan-dir at C-plan/,
    D-plan/, etc. would silently return zero or garbage rows, not a correct C-N/D-N tally. Re-deriving
    a fresh 213-plan-equivalent number would need either a generalized version of that tool (a code
    change, out of this task's read-only scope) or a repeat of the 220-subagent full sweep the 2026-06-04
    session ran. Neither was in scope here; the anchor is cited as-is instead.
  - This sweep is a one-time scratch measurement, not a shipped, frozen, recomputable-by-CI artifact.
    Treat the RECOMPUTE section above as the reproduction recipe until/unless a real tool is built.

PROOF-SPINE STATUS FLIP (advisory, PSR-4)
  python .claude/tools/proof_spine_status.py's PS-6 heuristic looks for a FILENAME containing
  "wire-dark-rate" or "wire_dark_rate" (case-insensitive substring), searched SHALLOWLY (top-level
  only, not recursive) in .elai_cc/metrics/, workflow/status/, or workflow/reasoning/. This file's
  name (proof-spine-ps6-baseline-2026-07-09.md, per the task's exact required path) does NOT match
  that substring, so proof_spine_status.py will continue to report PS-6 as RUNNER_PRESENT, not
  MEASURED, until either this file is renamed/aliased to include "wire-dark-rate", or the heuristic's
  artifact_patterns list for PS-6 is extended to also match "proof-spine-ps6-baseline". Flagged here,
  not fixed here (a rename/pattern-edit is a decision for whoever owns this ledger's final path, not a
  unilateral action from a read-only measurement task).

ADDENDUM 2026-07-09 (session continuing k18a-ps5 handoff) — bn_status_report WIDENED; enumeration
leg unblocked; plan-grain WIRE-DARK delta still pending the wiring-audit re-run
  The CAVEAT above (lines beginning "bn_status_report.py could not extend this ledger to the full
  213-plan ... cross-letter denominator") is now RESOLVED for the ENUMERATION leg only. What changed:
  - bn_status_report.py generalized from B-only to all single-letter plan families. Every id-bearing
    regex now captures (letter, number) and ids are letter-qualified ("C-18a") so families never
    collide; added `--all-letters` (sweeps every X-plan dir under --plan-root, each with its own
    {LETTER}-0_INDEX.md) and `--plan-root`. Default (B-only) behavior preserved byte-for-byte
    (verified: B totals unchanged SHIPPED 16 / IN-FLIGHT 2 / PLANNED 28 / MISSING_FILE 2, 48 rows).
    "patch-plan" is excluded by design (LETTER_DIR_RE = ^[A-Za-z]-plan$).
  - Cross-letter sweep result (RECOMPUTE: python .claude/tools/bn_status_report.py --all-letters --json):
    534 plan FILES enumerated across B..P. Per-letter file counts:
      B47 C60 D31 E57 F15 G11 H55 I55 J111 K26 L24 M7 N17 O5 P12  (+ one stray W-1, see anomaly).
    This is the cross-letter DENOMINATOR the pre-stack anchor's "213 plans" needed — but note it counts
    plan FILES, and a single file can hold several plan units, so 534 files != a unit count; it is the
    file-grain enumeration, not the unit-grain 213.
  HONEST GAP (why this still is NOT a clean plan-grain wire-dark delta):
    The --all-letters totals are SHIPPED 16 / IN-FLIGHT 2 / PLANNED 513 / MISSING_FILE 3. The SHIPPED=16
    is IDENTICAL to the B-only run: bn_status_report derives status ONLY from inline "X-N DONE|PARTIAL|
    TODO|BLOCK" notes in each family's index, and ONLY B-0_INDEX.md uses that convention. C..P indexes
    carry status in a different (table / prose) form the note-regex does not read, so every C..P plan
    falls to the default PLANNED. Therefore the widened tool gives the DENOMINATOR but NOT a wired-vs-
    dark (or even reliable shipped-vs-planned) verdict for non-B families. bn_status_report was never a
    wire-dark classifier — it reports index-declared status, not production-reachability. The clean
    plan-grain WIRE-DARK rate (the "SHIPPED_WIRED vs SHIPPED_WIRE_DARK" bucket the 2026-06-04 anchor
    measured) STILL requires re-running that session's per-plan wiring adjudication (static +
    attempted-runtime evidence per plan), which is a heavy multi-agent audit, NOT a tool fix.
  ANOMALY (flagged, not chased): the sweep surfaced one id "W-1" — a file named "W-1_*.md" living
    inside one of the B..P letter dirs (there is no W-plan dir). Likely a mis-filed / mis-prefixed plan
    file. 1 row of 534; benign for the counts; worth a locate-and-refile follow-up.
  NET FOR PS-6: unchanged. The MEASURED number remains the symbol-grain 7.69% (headline above). The
  clean plan-grain longitudinal delta is still PENDING, but its first blocker (no cross-letter
  enumerator) is now cleared; the remaining RECON is the per-plan wiring re-audit, not tooling.

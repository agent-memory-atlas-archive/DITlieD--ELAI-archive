GROUNDING SWEEP -- phase D-F9
Source: F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-9_hallucinated_dep_writetime_gate.md
Date: 2026-06-01
FSM: EXECUTING | Session: s-6344
Note: FSM shows phase "F-Unit-A (F-1+F-2a)", not D-F9 — F-9 is not the active phase this session.

================================================================================
SPEC REFERENCES
================================================================================

No "Spec references:" lines in the F-9 plan file (it is an F-plan file, not a B-N; references
are to workflow/reasoning/ synthesis docs and elai.map, not to docs/specs/guide/steps/phase-X.txt).

VALIDATION WARNING: no traditional spec references — the plan uses the synthesis doc and elai.map
as its grounding authorities. Both exist and are verified below. This is by design for F-plan files.

Primary synthesis doc:
F:\.ELAI_workflow\workflow\reasoning\VIBESEC-SHARD-STEAL-AND-BUILD-2026-06-01.md | GROUNDED |
  file exists; section 1.3 correction (NOT shape-only) at line 90-93; Spracklen numbers at line 238;
  tiered-gate design rule (section 2) at line 132-138; section 7 fail-open clause confirmed present.

F:\.ELAI_workflow\docs\thefinalglance\F-plan\F-0_INDEX.md | GROUNDED |
  W-1..W-10 at lines 47-106+; F-9 SCHEMA bump entry (+write_time_slopsquat_blocked) at line 148;
  baseline SCHEMA_VERSION 191 per W-9 line 103.

================================================================================
CLAIM CLASSIFICATION
================================================================================

--- D-11 PREREQUISITE / BLOCKER ---

CLAIM: D-11 ships registry_check.py (real HTTP checker, PyPI+npm) | GROUNDED |
  checkers/registry/registry_check.py EXISTS on disk; elai.map DEF_FN _check_pypi:90 /
  _check_npm:120 / _http_get_json:58 / _isoparse:43 / main:145 all confirmed. File is real
  HTTP checker (requests.get + urllib fallback, JSON parse, age check via _isoparse).
  D-11 DID SHIP the checker. NOT a blocker for that part.

CLAIM: CheckerKind::RegistryCheck exists in checker_registry.rs:62-94 | GROUNDED |
  checker_registry.rs: pub enum CheckerKind at line 62; RegistryCheck variant at line 94;
  from_str("registry_check") at line 126; as_str returns "registry_check" at line 109.
  Lines 62-94 confirmed correct.

CLAIM: no _check_cargo exists today (grep returns zero DEF rows) | GROUNDED |
  elai.map: grep "_check_cargo" returns zero DEF_FN rows.
  registry_check.py:182-188: main() branches "pypi" -> _check_pypi, "npm" -> _check_npm,
  else "inapplicable: unsupported ecosystem" — NO cargo branch. Confirmed absent.

CLAIM: registry_check.py line 175 hard-codes max_age_days=730 | GROUNDED |
  registry_check.py:175: max_age_days = int(candidate.get("max_age_days", 730)) confirmed exactly.

CRITICAL BLOCKER: _check_cargo does NOT exist in registry_check.py. D-11 shipped the PyPI+npm
checker but did NOT ship cargo coverage. F-9 requires cargo coverage as its core extension.
D-11's cargo gap is real. F-9 EXTENDS D-11 and cannot run without D-11's base (which IS shipped),
but the cargo-coverage extension (Task 1 of F-9) is genuinely new work not blocked by D-11 being
unshipped — D-11 IS shipped, just without cargo. The prerequisite text in F-9 ("D-11 PLANNED, NOT
shipped") is STALE/MISMATCH — D-11 HAS shipped per the codebase evidence.

--- E-41 SYMBOLS ---

CLAIM: verify_pr_deps_no_slopsquat at package_verify.rs:51 | GROUNDED |
  package_verify.rs:51 confirmed: pub async fn verify_pr_deps_no_slopsquat.

CLAIM: extract_dep_candidates at package_verify.rs:110 | GROUNDED | confirmed line 110.

CLAIM: flag_candidate at package_verify.rs:152 | GROUNDED | confirmed line 152.

CLAIM: is_dep_name_shape at package_verify.rs:191 | GROUNDED | confirmed line 191.

CLAIM: parse_json_dep_name at package_verify.rs:233 | GROUNDED | confirmed line 233.

CLAIM: parse_pep508_dep_name at package_verify.rs:257 | GROUNDED | confirmed line 257.

CLAIM: extract_dep_candidates has NO [dependencies]-section-aware cargo parse | GROUNDED |
  package_verify.rs:110-145: loops diff lines, catches loose "lhs = rhs" shape; no section
  context tracking, no [dependencies]/[dev-dependencies] detection. Confirmed absent.

CLAIM: E-41 CLI caller pr_review_cmd.rs:171 | GROUNDED | confirmed line 171.

CLAIM: commands_e41.rs::pr_review_status at line 80 | GROUNDED | confirmed line 80.

CLAIM: lookup_u32 at commands_e41.rs:205 | MISMATCH (minor) |
  commands_e41.rs: lookup_u64 at line 196, lookup_u32 at line 205, lookup_i64 at line 209.
  F-9 doc says "lookup_u32/lookup_u64 DTO-extraction helpers (line 205)" citing only line 205.
  lookup_u32 IS at line 205; lookup_u64 is at line 196. The "line 205" reference is correct for
  lookup_u32 but does not cover lookup_u64 (line 196). Minor imprecision, both functions exist.

CLAIM: slopsquat_count lookup at commands_e41.rs:134 | GROUNDED |
  commands_e41.rs:134: slopsquat_count: lookup_u32(..., "pr_package_verify_result", ...) confirmed.

--- PreToolUse BACKEND ---

CLAIM: dispatch_pre_tool_use at hook_dispatcher.rs:1477 | MISMATCH |
  hook_dispatcher.rs: dispatch_pre_tool_use appears as a TRAIT METHOD at line 389 and as an
  IMPL on IpcHookAsyncAdapter at line 1477. The F-9 doc says "dispatch_pre_tool_use (hook_dispatcher.rs:1477)"
  which points to the IpcHookAsyncAdapter impl, not the trait. The trait def is at line 389.
  Both lines are real code. The "1477" cite is the production impl entry point — valid for
  "the production hook the new write-time analyzer attaches to" claim. VERIFY: F-9 Tasks 5 and
  WIRING_CONTRACT reference "dispatch_pre_tool_use (hook_dispatcher.rs:1477)" which is the
  IpcHookAsyncAdapter — that is a concrete impl, not the trait object dispatch path. The new
  DepWriteTimeGate needs to be wired into the actual HookDispatcherImpl (the main struct), not
  just the IpcHookAsyncAdapter. This is an integration-point ambiguity, not a blocker, but
  requires the implementer to verify WHICH concrete dispatcher to extend.

CLAIM: HookInvocation struct at line 271 | GROUNDED | confirmed line 271.

CLAIM: for_pre_tool_use at line 324 | GROUNDED | confirmed line 324.

CLAIM: analyze trait at hook_dispatcher.rs:622 | GROUNDED |
  hook_dispatcher.rs:622: fn analyze(&self, invocation: &HookInvocation<'_>) confirmed.
  NOTE: this is on an internal plugin struct, not the HookDispatcher trait. The analyze method
  belongs to a plugin/tiered-analysis inner type. The DepWriteTimeGate's analyze method
  signature mirrors this pattern but attaches to the hook backend trait, not this specific impl.

--- IdeaFramerSecurityPanel / DrawerKey ---

CLAIM: IdeaFramerSecurityPanel.svelte exists | GROUNDED |
  crates/dashboard-v2/src/lib/windows/IdeaFramerSecurityPanel.svelte: Test-Path = True.

CLAIM: DrawerKey at workspace.svelte.ts:374 | GROUNDED |
  workspace.svelte.ts:374: function toggleDrawer(key: DrawerKey) confirmed. Type imported at line 5.

CLAIM: pr_review_status stub at stub.ts:3326 | GROUNDED | confirmed line 3326.

--- SSB / SCHEMA VERSION ---

CLAIM: SCHEMA_VERSION baseline 191 (per CLAUDE.md + F-0_INDEX W-9) | MISMATCH (CRITICAL) |
  ACTUAL: SCHEMA_VERSION = 196 in ssb.rs:2742, bumped by F-1/F-2a (session s-6344, 2026-06-01).
  TOTAL_VARIANTS = 473 (was 471 before F-1/F-2a, which added +2). The F-0_INDEX W-9 says
  "baseline 191 per CLAUDE.md; never hard-code a final number - say baseline + N". F-9 plan
  correctly uses "baseline + 1" phrasing in its WIRING_CONTRACT and ACCEPTANCE criteria.
  However, the F-9 Citations section says "SCHEMA_VERSION baseline 191 per CLAUDE.md" which is
  now stale. F-9's own WIRING_CONTRACT says "SCHEMA_VERSION: bump to baseline+1 (baseline 191
  per CLAUDE.md — name it 'baseline + 1' in the WIRING_CONTRACT, never hard-code a final number)".
  The reliance on the named +1 pattern (not hard-coded) is correct. At F-9 execution time,
  baseline will be 196 (or higher), so F-9 ships as SCHEMA_VERSION 197 (or current+1).
  The "191" in the CLAUDE.md reference is stale but the WIRING_CONTRACT's "baseline+1 named"
  instruction correctly handles this. The implementer must use the LIVE SCHEMA_VERSION at
  authoring time, not the literal "191".

CLAIM: WriteTimeSlopsquatBlocked not yet in ssb.rs | GROUNDED |
  grep "WriteTimeSlopsquat|write_time_slopsquat" in ssb.rs returns NO MATCHES. Confirmed absent —
  this is the new variant F-9 adds. The vibesec module dir does NOT exist yet either.

CLAIM: vibesec dir does not yet exist | GROUNDED |
  Test-Path crates/orchestrator/src/vibesec = False. F-9 must create it.

--- GhCliAdapter hash_prefix / blake3 ---

CLAIM: GhCliAdapter::hash_prefix uses blake3 | GROUNDED |
  gh_cli.rs:93: let hash = blake3::hash(input.as_bytes()); confirmed. Return type u64 prefix.

--- Spracklen numbers ---

CLAIM: Spracklen USENIX 2025: 19.7% hallucinated, 205,474 unique fake names, 43% recur all-10 | VERIFY |
  synthesis doc line 238: "Spracklen USENIX 2025 Distinguished Paper (19.7% hallucinated, 205,474
  unique fake names, 43% recur on all 10 reruns)" confirmed as cited in synthesis. The synthesis
  doc itself tags this "(VERIFIED)". Primary source (socket.dev blog / USENIX proceedings) not
  independently WebFetched this session. Cite via the synthesis doc per F-9's own instruction
  ("cite via the synthesis doc, not as an F-9-internal target number"). No further action needed
  per F-9 plan itself. Classification: VERIFY (synthesis-grounded, primary paper not fetched).

--- Socket Firewall ---

CLAIM: Socket Firewall is OPS-RUNBOOK only, no codegen-time slot | GROUNDED |
  synthesis doc confirms: "Socket Firewall at the install boundary is defense-in-depth but is an
  OPS RUNBOOK, not in-plan code (synthesis REVISED SCOPE explicitly moves it out of the plan)."
  F-9 plan text at line 29: "Socket Firewall at the install boundary is defense-in-depth but is
  an OPS RUNBOOK, not in-plan code." Consistent.

--- D-11 "PLANNED, NOT SHIPPED" CLAIM ---

CLAIM: "D-11 (PLANNED, NOT shipped)" in F-9 prerequisites section | MISMATCH (CRITICAL) |
  D-11 HAS shipped. Evidence: registry_check.py exists on disk at
  checkers/registry/registry_check.py; CheckerKind::RegistryCheck is in checker_registry.rs:94;
  checker_registry_contract.rs references it; D-11-decisions.md referenced in checker_registry.rs
  doc comment; git history shows "D-11" commit marker in recent log. The F-9 prerequisite block
  says "D-11 (PLANNED, NOT shipped)" which is WRONG — D-11 IS shipped. This is the most
  important mismatch because F-9 says "If D-11 has not shipped at F-9 start, F-9 BLOCKS at the
  spec-grounder gate." D-11 HAS shipped, so F-9 is NOT blocked by the D-11 prerequisite.
  The prerequisite text needs updating before TESTING_FIRST, but it is NOT a blocker for execution.

================================================================================
BLOCKERS
================================================================================

BLOCKER-1 (MISMATCH): F-9 prerequisites say "D-11 (PLANNED, NOT shipped)" but D-11 HAS shipped
  (registry_check.py + CheckerKind::RegistryCheck are live in the codebase). The spec-grounder
  gate says to BLOCK F-9 if D-11 has not shipped. D-11 IS shipped, so the block does NOT apply.
  However the stale claim "PLANNED, NOT shipped" must be corrected in the F-9 file before TESTING_FIRST
  to prevent the next spec-grounder from false-blocking. ACTION: update prerequisite line to
  "D-11 (SHIPPED)" before coding starts.

BLOCKER-2 (MISMATCH): SCHEMA_VERSION baseline is 196 (not 191). CLAUDE.md is stale. The F-9
  WIRING_CONTRACT correctly says "baseline+1" (not hard-coded), so execution is unblocked, but
  the Citations section literal "191" is stale. F-9 ships as SCHEMA_VERSION 197 (current+1).
  TOTAL_VARIANTS will be 474 (current 473 + 1). Implementer must use live values.

================================================================================
ACTIONS REQUIRED
================================================================================

- MISMATCH: update F-9 prerequisites block: change "D-11 (PLANNED, NOT shipped)" to
  "D-11 (SHIPPED)" — D-11's checker is live; F-9 is unblocked. The spec-grounder gate
  condition is satisfied. Update before TESTING_FIRST.

- MISMATCH: update F-9 Citations section: "SCHEMA_VERSION baseline 191 per CLAUDE.md" is
  stale. Current SCHEMA_VERSION = 196; F-9 ships as 197. TOTAL_VARIANTS = 473, will be 474.
  WIRING_CONTRACT is already correct ("baseline+1" named); only the Citations literal needs
  updating.

- VERIFY: Spracklen USENIX 2025 primary paper numbers (19.7% / 205,474 / 43%) — confirmed
  in synthesis doc (VERIFIED tag); no independent WebFetch done this session. If the paper
  must be cited directly (not via synthesis), spawn paper-ingestor mode=paper with search
  "Spracklen package hallucination USENIX Security 2025".

- VERIFY: hook_dispatcher.rs integration point — F-9 Task 5 says "register in
  dispatch_pre_tool_use (line 1477)" which is the IpcHookAsyncAdapter impl. Confirm whether
  the main HookDispatcherImpl (not IpcHookAsyncAdapter) is the correct extension target for
  a new analyze-backend. The context-compiler should resolve this at F-9 EXECUTING start.

- VERIFY: vibesec mod.rs — F-9 says "if the vibesec module dir does not already exist". It
  does NOT exist (Test-Path = False). Task is to CREATE it. No surprise, no conflict.

================================================================================
VALIDATION WARNINGS
================================================================================

- No traditional "Spec references:" lines (F-plan files use synthesis doc + elai.map as
  grounding; this is intentional per F-0_INDEX doctrine).
- FSM state is EXECUTING / F-Unit-A (F-1+F-2a), not D-F9. F-9 is not the active phase.
  This grounding sweep is pre-execution, run before F-9 starts.
- CLAUDE.md SCHEMA_VERSION (191) and TOTAL_VARIANTS (464) are stale per ssb.rs live values
  (196 / 473). This is a recurring drift — the canonical source is ssb.rs, not CLAUDE.md.
- D-11 "PLANNED" tag in F-9 is stale; D-11 shipped. See BLOCKER-1.

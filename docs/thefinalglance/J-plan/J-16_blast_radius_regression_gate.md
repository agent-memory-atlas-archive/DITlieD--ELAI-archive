# J-16: blast-radius / anti-regression gate — a fix may not silently break what it touches

- Track J (pillar-SOTA recheck extensions) | lever CANT-WRECK | grounding STRONG (live B-29 substrate grep + Top-50 taxonomy + AgentDebug) | nature BUILD | effort L | WAVE 1 (the user's #1 priority) | depends none (rides shipped B-29 impact API)
- KEYSTONE for the "small fix breaks something else" class. This is the single highest-priority unit of the whole pillar-SOTA recheck per the user.

- Premise: the dominant LLM coding failure is NARROW-FOCUS editing — the model fixes one thing with no map of what depends on it and no check that something else broke. ELAI already SHIPPED the map (B-29 symbol-level impact API: impact_radius / call_flow / symbol_context, DKG, codegraph MCP, .claude/tools/impact_radius.py) but it is a PRE-EDIT ADVISORY surfaced to the main LLM — advisory text the model can ignore, NOT a gate. GROUNDED GAP (verify-p4-context agent, 4 search iterations 2026-06-11): there is NO published regression-test-selection or over-editing benchmark for LLM agent edits — this is open whitespace ELAI can own with a deterministic gate. The fix is a non-LLM gate that, on every accepted edit batch, (a) computes the reverse-dependency cone of each changed symbol, (b) enforces diff-scope (edits stay within the declared task scope + forced dependents), (c) selects and requires the regression tests covering the cone, (d) flags silent deletion. The LLM never adjudicates (TD-3); the gate reads the DKG + the diff, both off-agent artifacts the executor did not author (R4).

- Top-50 bad-behaviors mapped (the verbatim cluster this closes; digest top50-bad-behaviors.md):
  #14 regression bugs when fixing ("no map of what depends on what") -> the dependency-cone + regression-selection legs.
  #15 over-editing (1-line ask -> 200-line diff, renames/reformats untouched code) -> the diff-scope leg.
  #16 silently deleting code on small edits ("diff becomes unsafe to accept") -> the silent-deletion leg.
  #29 unrequested "improvements" ("while I'm at it") -> the diff-scope leg (out-of-scope symbol touch = flag).
  #31 patches symptom not root cause (real cause upstream in another module) -> the cone surfaces the upstream callers so the root is visible; advisory, not a hard block.

## Steal/build
1. blast_radius gate (crates/orchestrator, new module): on the post-EXECUTING / pre-AUDITING boundary, for each changed symbol in the diff, query the DKG/codegraph reverse-dependency cone (depth-bounded, the B-29 traversal already exists) -> the set of symbols+files that transitively depend on the change. Deterministic; uses the SHIPPED impact API, re-implements no traversal (GD-1).
2. DIFF-SCOPE leg: compare the changed-file set against the declared task scope (the plan unit's WIRING_CONTRACT produces_symbols / the babystep scope, or for ad-hoc fixes the user-stated target). An edit touching a symbol OUTSIDE scope-plus-forced-dependents = a scope violation, surfaced with the exact out-of-scope hunks. Default OBSERVE-ONLY (GD-4): flag, do not block, until the FP/friction rate is bounded over >=3 real runs; then promote to BLOCK behind a wire-dark-allowlist row.
3. REGRESSION-SELECTION leg: from the cone, select the existing tests whose coverage intersects the changed symbols' dependents (test-to-symbol mapping from the DKG + the test files; NO new test authoring by this gate — it SELECTS from existing tests, the auditor runs them, R7: test-file callers do not satisfy production reachability but DO define the regression set). The selected set is the minimal regression suite that must stay GREEN; a test that passed before the edit and fails after = a regression BLOCK (the un-gameable signal: a real entrypoint ran and left a pass->fail transition the executor did not author, R4).
4. SILENT-DELETION leg: detect net-negative LOC on symbols NOT named in the edit intent (a deletion the model did not declare). Deterministic diff analysis; flags removed pub items / removed call sites that other code still references (the cone catches dangling references). NEVER auto-reverts (RV-4 honest-irreversibility analog) — surfaces for review.
5. The gate is AUDITOR-owned and FROZEN to the executing run (R8): the executor that wrote the edit cannot edit the gate; the regression set + scope contract are read-only inputs.
EXCLUDED: any LLM judgment of "is this edit too big" (that is the cursor thermo-nuclear 1000-line rubric — irreducibly advisory, routes to the auditor/critic lens not this gate); auto-revert of any kind; a NEW impact-traversal engine (B-29 ships it — GD-1 P0 if re-implemented).

## Existing ground truth (grep-confirmed; re-grep at edit, W-7/W-9)
- B-29 symbol-level impact API: crates/knowledge/ impact_radius / call_flow / symbol_context (the SHIPPED reverse-dependency traversal + sharded cache + 4 confidence tiers; feature map 1.1 SYMBOL-LEVEL IMPACT API [WIRED]).
- DKG dependency knowledge graph (crates/knowledge/, WIRED) + codegraph-elai MCP (live call-graph) + .claude/tools/impact_radius.py (harness-side blast-radius).
- AntiFabricationGate.check() at task_runner.rs (F-4 cant-lie gate) — the precedent for an auditor-owned deterministic check on the post-EXECUTING boundary; this gate is its blast-radius sibling.
- the diff is captured by the orchestrator (git2 ops, crates/tools/, phase-4B) — the off-agent artifact the gate reads.
- impact-mapper agent (.claude/agents/impact-mapper.md) — the pre-edit advisory this gate makes ENFORCED.

## Integration
- crates/orchestrator/src/blast_radius.rs (new, ~250 LOC: cone query via B-29 API + diff-scope diff + regression-set selection + silent-deletion diff) wired into the AUDITING entry (the auditor runs it, never the main LLM — H-3 / main-llm-role.md).
- +BlastRadiusEvent SSB (Pattern-2 single-variant; Kind: ConeComputed / ScopeViolationFlagged / RegressionSetSelected / SilentDeletionFlagged). baseline+1 from LIVE 231/542 (ssb.rs:3084 / action_type_lock.rs:671, grep 2026-06-12); re-read at edit (W-9). Full 6-part variant discipline.
- frontend_pair: a blast-radius panel/drawer on the existing audit-timeline surface (AuditTimelineWindow precedent, B-41) showing the cone + selected regression set + any scope/deletion flag; cross_lang_diff if surfaced (W-4) or W-6-justified absence for the first runtime-only slice.

## WIRING_CONTRACT
produces_symbols:
- crates/orchestrator/src/blast_radius.rs::compute_cone (reverse-dep cone via the B-29 impact API)
- crates/orchestrator/src/blast_radius.rs::check_diff_scope (out-of-scope symbol touch detection)
- crates/orchestrator/src/blast_radius.rs::select_regression_set (cone -> existing-test selection)
- crates/orchestrator/src/blast_radius.rs::flag_silent_deletion (undeclared net-negative symbol removal)
consumes_symbols:
- crates/knowledge B-29 impact_radius / call_flow (the SHIPPED traversal; NOT reauthored)
- the orchestrator-captured diff (git2) + the plan-unit WIRING_CONTRACT scope (the declared scope)
- the auditor test-run results (the pass->fail transition over the selected set)
reachable_from:
- the AUDITING FSM transition on a real `elai` task/fix run (post-EXECUTING, auditor-driven)  [PRODUCTION]
- a #[cfg(test)] / contracts/ caller NEVER satisfies this (R7)
stub_ban:
- NO LLM call inside the gate (deterministic cone + diff only; TD-3)
- NO auto-revert of any flagged hunk (surface only; RV-4)
- NO new impact-traversal engine (consume B-29; re-implementing the cone = GD-1 P0)
- NO BLOCK default until the FP/friction rate is bounded on >=3 real runs (GD-4) + a wire-dark-allowlist row with a promotion plan (W-2/R9)
- NO regression-set that includes ONLY test-file-authored coverage with no production-reachable dependent (R7: the cone must reach a production symbol)
frontend_pair: blast-radius panel on the audit-timeline surface (decide at edit) or W-6-justified absence.
observable_effect:
- trigger: "an `elai` fix run that edits a symbol with known dependents (e.g. change a function signature that 3 other modules call), driven through the real AUDITING transition"
- expect_ssb: "ConeComputed listing the 3 dependent modules; RegressionSetSelected naming the tests over those dependents; if the edit also touches an out-of-scope file, ScopeViolationFlagged with the exact hunk"
- expect_artifact: "the selected regression tests are RUN by the auditor; a dependent test that passed pre-edit and fails post-edit is reported as a regression and BLOCKS the FSM advance. NEGATIVE (R2): an empty cone on a symbol that DOES have dependents (DKG miss) FAILS the gate self-test, never SKIPs; a diff that deletes an unrelated pub item raises SilentDeletionFlagged, not silence; the gate pointed at a diff it cannot map (parse failure) FAILS, never passes."

## Grounding
- gap confirmation: verify-p4-context agent (zero published regression-test-selection / over-editing technique for agent edits, 2026-06-11) — this is novel whitespace.
- behavior taxonomy: top50-bad-behaviors.md #14/#15/#16/#29/#31 (EVIDENCE-CITED report, 80 refs; the coding-disaster figures are mixed-tier — used as a failure-CLASS taxonomy, not for any single number).
- failure isolation prior art: AgentDebug / AgentErrorTaxonomy arXiv:2509.25370 (VERIFIED, +24% all-correct accuracy, 26% relative task-success). The cone + regression-select is the deterministic analog of its failure attribution.
- substrate: B-29 SYMBOL-LEVEL IMPACT API [WIRED] (feature map 1.1) + DKG + codegraph MCP — all shipped, grep-confirmed.
- live SSB baseline 231/542 (ssb.rs:3084 / action_type_lock.rs:671, grep 2026-06-12).

## COMPOSES-WITH
B-29 impact API (consumes its traversal), F-4 anti-fabrication gate (sibling auditor-owned deterministic check on the same boundary), C-28 mutation gate + C-29 metamorphic (the regression set complements mutation score), G-5 reviewer cockpit / H-52 finding-anchoring (the cone + flags become per-hunk review evidence), I-20 budgeted mutation scheduler (the cone is the same diff data-flow surface). Restates none.

## Open questions / risks
- DKG completeness: build-time-generated symbols (protobuf/sqlx/proc-macro, the B1 tree-sitter attack in p1-retrieval) have zero incoming edges -> a cone miss is a FALSE-NEGATIVE blast-radius. The gate must DECLARE its coverage (which langs/edge-kinds it resolves) and FAIL-LOUD on an unmappable diff (R2), never silently pass. J-18's LSP channel narrows this gap (compiler-grade edges).
- scope source for ad-hoc fixes: a plan unit has a WIRING_CONTRACT scope; a bare "fix this" does not — the scope leg then falls back to the user-stated target file(s), and out-of-scope = flag-not-block until the friction rate is known (GD-4). Document the fallback explicitly.
- regression-set size: a high-fan-in symbol (utils.rs) yields a huge cone; bound the depth + cap the selected suite, and LOG the cap (no silent truncation — a capped regression set must say so, R-no-silent-caps).

## Next
The blast-radius cone is the substrate H-52/G-5 review surfaces consume per-hunk; J-18's LSP channel makes the cone compiler-grade (fewer false-negatives). This is the anti-LLM-bad-pattern keystone the rest of the pillar work hardens around.

End of J-16.

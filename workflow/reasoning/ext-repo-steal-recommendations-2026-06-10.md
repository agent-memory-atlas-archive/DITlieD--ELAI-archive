ext-repo steal recommendations 2026-06-10 (rev 2: skillspector-interface corrections folded in + steal-method + grounding file lists)
companion to: workflow/reasoning/ext-repo-eval-2026-06-10.md (per-file reasoning chain, 9 ELAI indexes + 7 repo summaries + skillspector interface map)
repo clones: D:\.liteGAME_workflow\workflow\external-skills\repos\<name> ; summaries at ..\repos\_summaries\<name>.md
skillspector interface map (paper-ingestor, source-reviewed): workflow/papers/skillspector-interface.md

STEAL METHOD (binding, per Track-20 SL-10/SL-11 discipline)
ELAI steals by REIMPLEMENTATION-FROM-SOURCE, never by summary: the cloned repo is the grounding context. Every steal below carries the concrete source files the implementing session MUST read before EXECUTING; a port done off this file or a summary alone is a W-3 violation. All four steal-source repos are permissively licensed (Apache-2.0 or MIT), so using their code as a base / direct adaptation is legal with license+attribution preserved (note the origin repo+license in the implementing plan file and any adapted-file header). Idea-not-dep stays the default for product code (SLC-3 precedent: no new runtime dependency without explicit approval); the harness MAY run a tool directly where noted.

ANSWERS TO THE THREE QUESTIONS

Q1 is codegraph better than elai? NO on design: ELAI's retrieval stack (DKG + BM25 + dual-dense + rerank + CatRAG + A-RAG + PageRank) plus the planned B-29/B-30/C-30 surface (call+dataflow edges, confidence tiers, sharded cache) exceeds it, and codegraph's tree-sitter name-match resolution is weaker than ELAI's SCIP leg. YES on one axis: its call-edge graph WORKS TODAY while ELAI's product dkg_edges has NO call edges (H-10 correction C11) and everything that would add them is PLANNED. Steal = source-grounded reimplementation prior-art, not the tool.

Q2 is Understand-Anything better than canvas? NO, different category and weaker on ELAI's axes. Canvas-workbench is live with click-to-agent deixis; H-plan Track 2 canvas is model-driven, SSB-journaled, taint-tagged, auditor-gated. UA is a one-shot batch artifact with no agent handoff, whose node summaries are unverified LLM prose (fails HD-5 / the cant-lie wedge) and whose pipeline spends a full multi-agent LLM sweep on what ELAI's DKG derives deterministically. 2-3 UX concepts worth harvesting, nothing more.

Q3 anything worth adding? Two direct steals (skillspector, impeccable), three source-grounded prior-art references (codegraph, headroom, Understand-Anything fragments), two declines (agentmemory, Agent-Reach). Detail below.

TIER 1 -- DIRECT STEALS

1. skillspector -> the verified scanner leg of skill-import-policy.md  [highest value]
   repo: D:\.liteGAME_workflow\workflow\external-skills\repos\skillspector (NVIDIA/skillspector, Apache-2.0, v2.1.2 Alpha, Python 3.12+)
   interface map (read FIRST): workflow/papers/skillspector-interface.md
   why: fills the ONLY documented unresolved gap the ELAI-side reading surfaced. skill-import-policy.md item 1 mandates a whole-directory scanner; G-4 carries the binding correction "mcp-scan dist-channel UNVERIFIED"; F-1's malicious-skill Tier-2 axis is DEFERRED behind that exact unresolved scanner; E-43's import gate is a text-only soft-gate. skillspector is the missing analyzer with clean provenance (NVIDIA org GitHub = the verifiable channel mcp-scan lacks).
   scoping corrections from the interface map (BINDING for the implementing session):
   - run STATIC ONLY (--no-llm): meta_analyzer becomes pass-through, ALL static findings kept. Deterministic, no API key, no egress except optional OSV.dev (silent offline fallback). LLM legs (semantic_* analyzers, TP4 description-behavior mismatch) go dark under --no-llm -- acceptable, TD-3 bans LLM verdicts in gates anyway.
   - static precision is MODERATE (FPs admitted in README; LLM stage lifts to ~87%): the gate consumes FINDINGS as the reviewable artifact, score as advisory only. HIGH/CRITICAL findings -> BLOCKED or NEEDS_REVIEW per existing policy; never auto-trust the 0-100 number.
   - risk score SATURATES (one CRITICAL + the 1.3x executable multiplier clamps to 100; 2 CRITICALs == 10) and executable detection is extension-only (.py/.sh/.js; malicious .md never multiplies). log findings_count + per-finding severities in the skill-imports.log row, not just the score.
   - mcp_rug_pull analyzer is a STUB (RP1-RP3 unimplemented, zero findings, TODO SADD B.3.3): skillspector does NOT cover MCP rug-pull; ELAI's G-4 re-pin gate remains the only coverage there.
   - platform: Python 3.12+, yara-python needs native YARA (Windows: prebuilt wheel), git on PATH for URL inputs, temp_dir_for_cleanup deletion is the CALLER's job, typer pinned <0.24 (semgrep click conflict).
   - wrapper-testable contract: risk_score in [0,100]; severity in {LOW,MEDIUM,HIGH,CRITICAL}; recommendation in {SAFE,CAUTION,DO_NOT_INSTALL}; per-finding confidence in [0,1]; score>=81 <=> CRITICAL.
   harness use (immediate): `skillspector scan <skill-dir> --no-llm -f json` -> parse findings -> workflow/skill-imports.log TSV row (scan_hash = blake3 of scanner output, per existing schema). Update skill-import-policy.md item 1 to name skillspector as the resolved scanner (mcp-scan demoted to optional second leg if its channel ever verifies).
   product steal (F-1 Tier-2 malicious-skill axis, Rust reimplementation grounded on these sources):
     src/skillspector/nodes/analyzers/static_runner.py                      (the static dispatch loop)
     src/skillspector/nodes/analyzers/pattern_defaults.py                   (the pattern inventory + default remediations)
     src/skillspector/nodes/analyzers/static_patterns_prompt_injection.py   (per-category rule files -- read all static_patterns_*.py:)
     src/skillspector/nodes/analyzers/static_patterns_data_exfiltration.py
     src/skillspector/nodes/analyzers/static_patterns_privilege_escalation.py
     src/skillspector/nodes/analyzers/static_patterns_supply_chain.py
     src/skillspector/nodes/analyzers/static_patterns_memory_poisoning.py
     src/skillspector/nodes/analyzers/static_patterns_excessive_agency.py
     src/skillspector/nodes/analyzers/static_patterns_tool_misuse.py
     src/skillspector/nodes/analyzers/static_patterns_rogue_agent.py
     src/skillspector/nodes/analyzers/static_patterns_system_prompt_leakage.py
     src/skillspector/nodes/analyzers/static_patterns_output_handling.py
     src/skillspector/nodes/analyzers/static_patterns_harmful_content.py
     src/skillspector/nodes/analyzers/behavioral_ast.py + behavioral_taint_tracking.py  (AST + taint legs)
     src/skillspector/nodes/analyzers/static_yara.py + src/skillspector/yara_rules/*.yar (4 bundled rule files)
     src/skillspector/nodes/analyzers/mcp_tool_poisoning.py                 (TP1-TP3 static legs)
     src/skillspector/nodes/analyzers/osv_client.py                         (OSV batch POST + 1h cache + offline fallback)
     src/skillspector/nodes/report.py                                       (_compute_risk_score weights -- fix the saturation when reimplementing)
     src/skillspector/nodes/build_context.py + input_handler.py + models.py + state.py (file cache/manifest/AST cache + Finding shape)
   (all paths relative to D:\.liteGAME_workflow\workflow\external-skills\repos\skillspector\)

2. impeccable -> deterministic design-quality lint for dashboard-v2 / Track 2 canvas work
   repo: D:\.liteGAME_workflow\workflow\external-skills\repos\impeccable (pbakaus/impeccable, Apache-2.0, ~36k stars, CLI v3.5.0, Node>=24)
   why: fills a hole neither the plans nor the harness cover. ELAI's frontend checks are correctness-only (svelte-check, cargo check, knip); zero design-quality gating. The detect engine = 41 deterministic anti-pattern rules over HTML/CSS, zero LLM, JSON/CI output -- doctrine-aligned (sound-checker-first; output is an off-agent artifact an auditor can consume).
   harness use (immediate): `npx impeccable detect --fast --json <dir>` on changed .svelte/.css during dashboard-v2 + H-10..H-16 frontend_pair work; fold into dashboard-smoker's checklist. anti-slopsquat verify first (`npm view impeccable version dist-tags time`). CLI-only use puts no SKILL.md in .claude/skills/, so no skill-import scan triggers.
   product/harness steal (reimplement the rule engine where ELAI wants it native, grounded on these sources; engine root = .agents/skills/impeccable/scripts/detector/):
     scripts/detector/detect-antipatterns.mjs                (the engine entry)
     scripts/detector/registry/antipatterns.mjs              (the 41-rule registry -- the core inventory)
     scripts/detector/rules/checks.mjs                       (rule implementations)
     scripts/detector/engines/static-html/detect-html.mjs    (jsdom static adapter)
     scripts/detector/engines/static-html/css-cascade.mjs    (cascade resolution for static scans)
     scripts/detector/engines/regex/detect-text.mjs          (fast regex-only pass, the --fast mode)
     scripts/detector/engines/browser/detect-url.mjs         (getComputedStyle live adapter)
     scripts/detector/engines/visual/screenshot-contrast.mjs (visual contrast leg)
     scripts/detector/shared/color.mjs + constants.mjs       (contrast math + thresholds)
     scripts/detector/findings.mjs + profile/profiler.mjs    (finding shape + project profiling)
   H-14 prior-art (click-to-edit design mode -- read when authoring H-14):
     scripts/live/*.mjs  (browser overlay: insert-ui, manual-apply, manual-edit-routes, session-store, svelte-component.mjs + sveltekit-adapter.mjs are DIRECTLY relevant to ELAI's Svelte dashboard)
   skill-content prior-art (design-law vocabulary, advisory layer only):
     .agents/skills/impeccable/reference/{audit,critique,layout,typeset,colorize,harden}.md
   HOLD: importing the 23-command skill itself into .claude/skills/ (>3-verb scope-creep flag; if ever wanted: whole-dir scan + skill-imports.log + approval first).
   (all paths relative to D:\.liteGAME_workflow\workflow\external-skills\repos\impeccable\)

TIER 2 -- SOURCE-GROUNDED PRIOR-ART (no install; add a pointer line to the named plan files; implementing session reads the listed source)

3. codegraph -> B-29 / B-30 / C-30 / H-10 executor reference
   repo: D:\.liteGAME_workflow\workflow\external-skills\repos\codegraph (MIT, v0.9.9, TS)
   read before executing those plans (paths relative to repo root, verified on disk 2026-06-10):
     src/extraction/languages/*.ts        (per-grammar symbol + call-edge extraction; one file per language incl. rust, python, typescript, csharp)
     src/extraction/index.ts + parse-worker.ts + tree-sitter.ts + tree-sitter-helpers.ts + grammars.ts (orchestration + off-thread parse batching)
     src/extraction/svelte-extractor.ts + vue-extractor.ts (composite-file extractors = B-30's svelte/vue composite re-parse, working model)
     src/resolution/import-resolver.ts + path-aliases.ts + workspace-packages.ts + go-module.ts (per-language import resolution incl. tsconfig aliases + Cargo/npm workspace + Go modules = B-30's per-language module-resolver spec, working model)
     src/resolution/name-matcher.ts + callback-synthesizer.ts (the unsound name-match leg -- read to know what NOT to trust per R5)
     src/sync/watcher.ts + watch-policy.ts ((size,mtime,content-hash) reconnect reconcile = B-31-adjacent)
     src/mcp/tools.ts + src/mcp/server-instructions.ts (tool shapes: explore/search/node/callers/callees/impact; the adaptive 13-24K output budget = B-29 token-fit sibling)
   also: its measured methodology (16% cheaper / 47% fewer tokens / 58% fewer tool calls vs grep-explore, 7 repos) is a competitor row + comparison class for the E-22 elai.map-vs-claude-context benchmark.
   decline: as a dependency or harness MCP (Node tool, name-match soundness below the R5 bar, second index violates one-source-of-truth vs elai.map/SCIP).

4. headroom -> I-14 / I-40 / I-12 executor reference
   repo: D:\.liteGAME_workflow\workflow\external-skills\repos\headroom (Apache-2.0, v0.24.0 beta, Python+Rust)
   read before executing (paths relative to repo root, verified on disk 2026-06-10):
     headroom/transforms/code_compressor.py   (per-language AST elision heuristics = I-14 Tier-1 folding prior-art)
     headroom/transforms/smart_crusher.py     (JSON array/nested compression = I-13 to_llm_content prior-art)
     headroom/transforms/content_router.py + content_detector.py + compression_policy.py (type-dispatch + policy = I-14's transform seam shape)
     headroom/transforms/diff_compressor.py + log_compressor.py (two payload classes I-14 should also cover -- note for scope)
     headroom/cache/compression_store.py + compression_cache.py + compression_feedback.py (the CCR reversible-originals store = I-14 ContextTransformReceipt / I-13 TraceQuery analog)
     headroom/transforms/cache_aligner.py     (prefix stabilization -- compare against H-2/I-10; ELAI's design is stronger but their provider-quirk handling is worth reading)
     headroom/proxy/compression_decision.py + semantic_cache.py (in-proxy decision point + semantic cache = I-15 sibling)
     headroom/cli/wrap.py                     (proxy lifecycle + env injection + win32 utf-8 fix + ENABLE_TOOL_SEARCH preservation = the exact `elai wrap` deployment shape I-40 ships)
     headroom/providers/claude/install.py + runtime.py (per-scope env/config writes for wrapping Claude Code)
     headroom/cli/mcp.py + mcp_registry/claude.py (stdio MCP registration mechanics)
     headroom/cli/learn.py                    (failed-session mining -> CLAUDE.md/AGENTS.md write = I-12's export leg, market-validated; I-12's evidence-gating stays the differentiator)
   decline: as runtime dep/integration (beta, single maintainer, lossy ML in the prompt path conflicts with grounded-by-default, proxy collides with our ANTHROPIC_BASE_URL sidecar usage).

5. Understand-Anything -> 2 canvas-mode concepts for H-plan Track 2
   repo: D:\.liteGAME_workflow\workflow\external-skills\repos\Understand-Anything (MIT, v2.7.6, TS)
   harvest as one-line candidates in H-12's file: (a) dependency-ordered GUIDED TOURS over the graph (tour-builder agent) as an onboarding canvas mode; (b) business-domain overlay (/understand-domain, code->domain second graph view). Both advisory-layer only under HD-5 (LLM-derived content).
   read if/when authoring those modes (paths relative to repo root, verified on disk 2026-06-10):
     understand-anything-plugin/skills/understand/skill.md       (the 7-phase pipeline contract)
     understand-anything-plugin/agents/tour-builder.md           (dependency-ordered guided-tour generation)
     understand-anything-plugin/agents/domain-analyzer.md + understand-anything-plugin/skills/understand-domain/SKILL.md (code->business-domain overlay)
     docs/superpowers/specs/2026-04-01-business-domain-knowledge-design.md (the domain-view design doc)
     the knowledge-graph.json schema (graph-as-committable-artifact shape; ELAI analog elai_graph.json already exists)
   decline: everything else (one-shot LLM-sweep graph construction inverts the CHEAPER lever; no deixis; diff-impact/chat covered by impact_radius + A-RAG/oracle).

DECLINES (recorded so they are not re-litigated)

6. agentmemory (rohitg00/agentmemory, Apache-2.0, v0.9.27): ELAI's memory design dominates on trust (bi-temporal supersession, H-32 citation re-validation, auditor firewall, MINJA+I-13 guard, H-30 staleness instrument). Its one distinct axis -- silent automatic hook-driven capture -- is a DOCUMENTED ELAI design decline (H-31 explicit-only capture, the Cursor lesson); H-51's journal-derived digest covers the legitimate half more trustworthily. 53 overlapping tool names would pollute the catalog; iii-engine Windows friction. Two footnote ideas only: memory_timeline read-surface (H-33), import-jsonl backfill (H-51).

7. Agent-Reach (Panniantong/Agent-Reach, MIT, v1.4.2): installer for cookie-authenticated scrapers; widens egress/credential surface in exactly the way E-37/F-3/G-1 exist to prevent. Search needs already covered (sidecar /search + SearXNG + WebSearch + gpt-bridge deep research). yt-dlp for an occasional transcript can be invoked directly when needed. Its doctor-per-channel pattern is already ELAI practice (B-43 doctor).

SUGGESTED NEXT ACTIONS (pending your yes)
- skillspector: smoke `skillspector scan <one of our own skills> --no-llm -f json` from the clone, then amend skill-import-policy.md item 1 (scanner resolved) + note the rug-pull-stub non-coverage. (~30 min)
- impeccable: npm view verification + trial `npx impeccable detect --fast --json` over dashboard-v2 src; add to dashboard-smoker checklist. (~15 min)
- add the Tier-1/Tier-2 source-pointer lines to the consuming plan files (F-1 follow-up, B-29/B-30/E-22, I-12/I-14/I-40, H-12/H-14) -- one line each, GD-1-style "external prior-art: <repo path> -> <files>".

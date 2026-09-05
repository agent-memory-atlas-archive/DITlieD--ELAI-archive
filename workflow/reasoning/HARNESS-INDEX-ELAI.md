# HARNESS-INDEX-ELAI

Raw index of the ELAI workflow harness tooling under F:\.ELAI_workflow, for the TEIKOKU merge + gate-redesign. Goal: convert mechanically-enforceable hard rules into deterministic hooks/gates (sticks), keep true behavioral guidance as CLAUDE.md carrots, push reference depth into RAG.

Row format: path | what-it-is/does | TRIGGER | CLASS | TAGS
CLASS = HOOKABLE | JUDGMENT | REFERENCE. HOOKABLE rows append (gating tool + check).
TAGS = STICK | CARROT | GAME | ALWAYS-ON | LAZY.

================================================================
## CLAUDE.md (each HARD RULE block indexed separately)
================================================================
Two CLAUDE.md files load every session: the global C:\Users\user\.claude\CLAUDE.md and the project F:\.ELAI_workflow\CLAUDE.md. Both are huge and ALWAYS-ON. Each embedded HARD RULE is broken out below.

C:\Users\user\.claude\CLAUDE.md (global) | Workflow-tool/fan-out ban: never invoke Workflow tool off keyword trigger; >2 subagents needs approval+model+cost | always | HOOKABLE (PreToolUse:Task/Workflow, deny Workflow tool + deny >2 parallel subagent spawns unless approval flag) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Grounded-by-default | every claim grounded in evidence (tests/code/docs/papers) before acting; label "Ungrounded conjecture:" else | always | JUDGMENT | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Project boundary | all writes target .elai_cc\ unless workflow-tooling change | always | HOOKABLE (PreToolUse:Write/Edit, deny writes outside .elai_cc except .claude/CLAUDE.md/memory-bank/workflow) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: GRC (Grounded Reasoning Chain) | built-in thinking disabled; reason via mcp_think w/ grounding citation + blockquote echo | always | JUDGMENT (mcp_think call enforceable, grounding-quality not) | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: GRC trigger rules | MUST mcp_think on debug/architecture/multi-step/uncertainty/destructive; budget by complexity | conditional | JUDGMENT | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Ask-don't-assume | ask before writing if unclear; record assumption if unattended | always | JUDGMENT | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: E-38 2026 posture | cite ELAI-2026-POSTURE.md or WebFetch primary for 2026 ecosystem claims; never from training | conditional | JUDGMENT | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Orchestrate-protocol-binding | when orchestrate fired, agent dispatch BLOCKING; no Edit/test/memory by main LLM | conditional (orchestrate thread) | HOOKABLE (Stop hook parses orchestrate-run.md for Edits w/o preceding dispatch) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: No Co-Authored-By / AI attribution in commits | git commit messages | conditional | HOOKABLE (PreToolUse:Bash, deny git commit if message contains Co-Authored-By/AI trailer) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Sidecar boot hardware-heavy | flag in chat before boot; never two model instances | conditional | JUDGMENT | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Test results in commit messages | include pass/fail counts + deterministic check status | conditional | HOOKABLE-WEAK (PreToolUse:Bash, warn if commit lacks test-result line) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Commit+push after task / everything gitignored except elai_cc | conditional | JUDGMENT | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Don't delete memory-bank templates (Format:) | conditional | HOOKABLE (PostToolUse:Edit on memory-bank, verify template lines preserved — already enforced by rag post_tool_check) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Anti-Slopsquatting | verify dep name/existence/version via registry before use; never trust training | conditional | HOOKABLE (PreToolUse:Edit, detect new import/dep line, gate on registry-verify; partially) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Anti-Stale-Training | use blake3/rand0.9/tracing/thiserror/ort2.x etc; not stale crates | conditional | HOOKABLE (PostToolUse grep for banned: blake2/sha2/rand0.8/log-crate/async-trait/ort1.13/r2d2/swiftide/.unwrap) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Context delegation | never read plan files/memory-bank/context-package.md directly | always | HOOKABLE (PreToolUse:Read, deny main-LLM Read of plan/*.md + memory-bank + context-package.md) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Agent spawning | never read agents/*.md yourself; pass path; model=sonnet; canonical list | conditional | HOOKABLE (PreToolUse:Read, deny main-LLM Read of .claude/agents/*.md) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: FSM gate check | call mcp_fsm_get_state before Edit/Write/Bash; transition if blocked | always | HOOKABLE (already: rag pre_tool_check.py PreToolUse path/tool gating per FSM state) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: FSM abort/cleanup | mcp_fsm_reset on stop; MEMORIZING>COMPLETE>IDLE before session end | conditional | HOOKABLE (already: Stop hook blocks termination in workflow states) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Hook enforcement summary | PreToolUse/PostToolUse/Stop/PreCompact responsibilities | always | REFERENCE (describes existing gates) | ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: MEMORIZING delegation | spawn memory-writer; never write memory-bank directly | conditional | HOOKABLE (PreToolUse:Edit, deny main-LLM Edit of memory-bank/* in MEMORIZING) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Token budget discipline | one-line rules; redundancy is a bug | always | JUDGMENT | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Impact analysis | grep call sites / elai.map before changing a function | conditional | JUDGMENT (could gate via impact_radius.py) | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Search before speaking / External grounding | prefix "From training (unsearched):"; WebSearch version-specific | always | JUDGMENT | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: GPT/Grok second-opinion | gpt-bridge as one diversity vote, never grounding | conditional | REFERENCE | CARROT LAZY
F:\.ELAI_workflow\CLAUDE.md :: Canvas-workbench | localhost code-graph tab + handoff hook | conditional | REFERENCE | CARROT LAZY
F:\.ELAI_workflow\CLAUDE.md :: One change at a time / main never runs tests | auditor is sole test runner | always | HOOKABLE (PreToolUse:Bash, deny cargo/pytest/svelte-check by main LLM outside AUDITING) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Decision doc before EXECUTING | write phase-X-decisions.md | conditional | HOOKABLE-WEAK (auditor/soft-check artifact-exists) | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Contract tests | tests/contracts/ per module; run at phase entry+audit | conditional | JUDGMENT | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: No .unwrap()/.expect() in prod Rust | except test/main/const-init | conditional | HOOKABLE (PostToolUse:Edit *.rs, grep .unwrap()/.expect()/panic! outside #[cfg(test)]) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: No cargo test unless permitted | use cargo check (compile-only) | conditional | HOOKABLE (PreToolUse:Bash, deny `cargo test` unless permission flag) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Memory dir read-only for rules | no rules/feedback in projects/ memory path | conditional | HOOKABLE (PreToolUse:Write, deny rule-content writes to memory path) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Handoff protocol pointer | one topic-keyed file, append not spawn | conditional | REFERENCE (full rule in handoff-protocol.md) | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Scoped test runs | -p flag affected crates only, not full workspace | conditional | JUDGMENT | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Anti-sycophancy / two-approaches / abstention / confidence-calibration | behavioral cluster | always | JUDGMENT | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: CSQ counterfactual | mcp_think counterfactual before trading/ML logic | conditional | JUDGMENT | CARROT GAME-adjacent ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Zero Lazy Code | no pass/TODO/placeholder/except:pass/mock in prod | always | HOOKABLE (already: Stop hook + rag post_tool_check) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Post-Write Self-Verification | re-read fn>10 lines, grep TODO/placeholder | conditional | HOOKABLE (Stop hook enforces) | STICK ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Reasoning-After-Action / Anti-Spiral | mcp_think after failure; stop after 3 fails / >3 edits same issue | conditional | JUDGMENT (anti-spiral skill detects mechanically) | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: Project PATHs / model/sidecar/spec locations | reference block | always | REFERENCE (candidate for RAG) | CARROT ALWAYS-ON
F:\.ELAI_workflow\CLAUDE.md :: E-10/E-43/E-31 read-only pointers + SCHEMA_VERSION/TOTAL_VARIANTS | reference block | always | REFERENCE (candidate for RAG) | CARROT ALWAYS-ON

================================================================
## .claude/rules/ (auto-load; conditional-routing gates most)
================================================================
.claude/rules/agent-dispatch-preamble.md | binding preamble injected at top of every dispatched sub-agent prompt: git read-only (R-1), no destructive ops (R-2), scope=contract (R-3), BLOCKED=handoff (R-4), structured return (R-5), grounding (R-6), budget (R-7), production-wiring-is-deliverable (R-8) | every agent dispatch | HOOKABLE (orchestrating skill must prepend pointer line; checkable that dispatch prompt contains it) | STICK ALWAYS-ON
.claude/rules/bun-not-node.md | default JS/TS runtime is bun; no new node/npm/npx; rewrite npx>bunx | JS/TS script/tool/MCP/manifest touch | HOOKABLE (PreToolUse:Bash deny `node`/`npm`/`npx` for new scripts; PostToolUse grep config for npx) | STICK LAZY
.claude/rules/commit-cadence.md | one commit per task (not per-phase); maintainer voice (lowercase i, no em-dash, no robotic prefix); no AI trailer; commit fail = STOP not chunk | git commit | HOOKABLE (PreToolUse:Bash gate git commit: cadence + message-style lint + single-changeset) | STICK LAZY
.claude/rules/conditional-routing.md | the dispatch table: maps triggers to which rule/rulebook file to lazy-load | always (it IS the router) | REFERENCE | CARROT ALWAYS-ON
.claude/rules/handoff-protocol.md | one topic-keyed file workflow/handoff/{slug}.md; glob-then-append; head-only read; living CURRENT STATUS+INDEX | "write/update handoff" | REFERENCE (authoring how-to) | CARROT LAZY
.claude/rules/main-llm-role.md | orchestrate/manage/review; don't implement when agent can; main never runs tests/writes memory/reads plan files | always | JUDGMENT (parts hookable via FSM gating) | CARROT ALWAYS-ON
.claude/rules/maintainability-review.md | 8-dimension maintainability catalog + MR-1..MR-4 deterministic checker; auditor-applied off-agent; frozen | ship/audit/review code | HOOKABLE (auditor runs maintainability_check.py; FROZEN gate) | STICK LAZY
.claude/rules/orchestrate-protocol.md | H-1..H-11 BLOCKING: no Edit/Write before parallel batch returns; no audit-skip; no memory-skip; integration gate mandatory | orchestrate skill fired | HOOKABLE (Stop hook parses orchestrate-run.md, P0 if Edits w/o dispatch) | STICK LAZY
.claude/rules/output-formatting.md | deterministic/zero-filler/no-emoji output; memory-bank update format; maintainer voice | always | HOOKABLE-WEAK (PostToolUse could lint emoji/markdown in authored files) | CARROT ALWAYS-ON
.claude/rules/production-caller-gate.md | D-030: every new pub fn needs production caller outside its file or WIRE-DARK marker; explorer names PRODUCTION_HOOK; auditor re-runs checker; frozen | executor ships pub symbol / audit | HOOKABLE (production_caller_check.py / python_wire_check.py; FROZEN) | STICK LAZY
.claude/rules/skill-import-policy.md | E-43: scan external skill dir w/ skillspector before promote; record workflow/skill-imports.log; HIGH/CRITICAL=BLOCK | importing 3rd-party SKILL.md | HOOKABLE (PreToolUse gate: deny ref to skill w/o ALLOWED log row; /skill-audit) | STICK LAZY
.claude/rules/token-discipline.md | anti-read-big (refuse Read>200 lines / Grep>50 lines output w/o subagent); read-means-read override; zero-lazy-code; anti-slopsquatting protocol; impact-analysis | always | HOOKABLE (PreToolUse:Read deny >200-line full reads; PostToolUse zero-lazy grep) | STICK ALWAYS-ON
.claude/rules/tracing-test-assert.md | T-1..T-3: every #[traced_test] needs asserted logs_contain; BatchSpanProcessor banned in tests; frozen | author/audit tracing-test Rust | HOOKABLE (traced_test_assert_check.py; FROZEN) | STICK LAZY
.claude/rules/visual-ui-loop.md | schema-constrained screenshot-compare loop: deterministic diff first, AX-scaffold, pass/fail/unknown(=fail), one fix/loop, two-pass stop, oscillation detector | dashboard visual iteration | HOOKABLE-PARTIAL (ui-diff.mjs deterministic; vision critique is JUDGMENT) | STICK LAZY
.claude/rules/wiring-enforcement.md | R1-R10 fail-closed wiring doctrine: missing evidence=FAIL, real-entrypoint+off-agent-artifact, no opt-in gate flags, frozen gate, no default-off graveyard, unit-green≠integrated-green; 4-layer gate stack; frozen | ship feature / done-gate / flag touch | HOOKABLE (wiring_contract_check.py + layered checkers; FROZEN canonical doctrine) | STICK LAZY

================================================================
## .claude/rulebook/ (lazy-loaded depth — RRulebook DISPATCHES)
================================================================
.claude/rulebook/agent-spawning.md | never read agents/*.md; pass path; model=sonnet; canonical agent list + per-level decomposition flow | spawning any agent | HOOKABLE (PreToolUse:Read deny agents/*.md) | STICK LAZY
.claude/rulebook/bubble-up.md | deterministic retry/redecompose/escalate decision on failure; per-level retry budgets; port of ELAI BubbleUpHandler | baby-step failure / audit BLOCK | REFERENCE (decision algorithm) | CARROT LAZY
.claude/rulebook/canvas-image-regen.md | click image > regen via gpt_image > preview > approve > writeback loop; role boundary | canvas image element click | REFERENCE | CARROT LAZY
.claude/rulebook/codegraph-vs-map.md | split of duties codegraph(live) vs elai.map(static SCIP) + graph_reconcile.py for trusted answers | locating symbol / call chain / impact | REFERENCE | CARROT LAZY
.claude/rulebook/coding-standards.md | Rust/Python/Cargo/Pytest, TDD, no new deps, bash-as-tools, sidecar location | coding any project code | JUDGMENT | CARROT LAZY
.claude/rulebook/design-taste.md | anti-slop look layer for Svelte/Tauri dashboard: named AI-default tells, 3 dials (VARIANCE/MOTION/DENSITY), one accent, authored font, full state coverage; pre-flight | design-judged UI | JUDGMENT | CARROT LAZY
.claude/rulebook/file-discipline.md | file-length caps (WARN/BLOCK LOC); monolith-drift prevention; Swift/iOS-flavored (stale for this Rust root) | authoring/growing file >150 lines | HOOKABLE-WEAK (PostToolUse LOC threshold; overlaps maintainability MR-1) | STICK LAZY GAME-adjacent
.claude/rulebook/folder-ownership.md | never write inside another dev's per-dev folder; reading ok | writing inside .projects/<app>/ multi-dev | HOOKABLE (PreToolUse:Write deny other-dev folders) | STICK LAZY
.claude/rulebook/gpt-image-prompting.md | how to prompt gpt_image (instruction-following spec not keyword soup); bridge constraint box | calling gpt_image | REFERENCE | CARROT LAZY
.claude/rulebook/gpt-second-opinion.md | capability rule: gpt-bridge for diversity vote/deep-research; one vote, weak grounding, never critical path | hard fork / 3 failed hypotheses | REFERENCE | CARROT LAZY
.claude/rulebook/integration-exploration.md | parallel wiring-discovery pattern (what existing APIs to hook into) for HEAVY tier | HEAVY-tier flow | REFERENCE | CARROT LAZY
.claude/rulebook/memorizing-and-audit.md | MEMORIZING spawn memory-writer never write directly; audit + FSM cleanup | AUDITING/MEMORIZING / post-phase write | HOOKABLE (PreToolUse deny main memory-bank write) | STICK LAZY
.claude/rulebook/multi-session.md | s-{pid} session ids; per-session FSM state; mcp_session_claim owns/locks; PreToolUse blocks cross-session file edits | multi-session / cross-session locks | HOOKABLE (already: PreToolUse cross-session file locks) | STICK LAZY
.claude/rulebook/orchestrate-protocol.md | one-screen digest of O-HR1..O-HR10 (own-session-only FSM, etc); duplicates rules/orchestrate-protocol.md | orchestrate fired | REFERENCE (DUP) | STICK LAZY
.claude/rulebook/risk-levels.md | LOW/MEDIUM/HIGH node classification by blast radius; auto-advance vs human-approval | classifying decomposition node | REFERENCE | CARROT LAZY
.claude/rulebook/session-protocol.md | session start: spawn context-compiler, ask task, mcp_session_register, sidecar health check | session start / tier classify / resume | REFERENCE | CARROT LAZY
.claude/rulebook/source-tiers.md | tier-1..4 web-source trust classification; researcher stamps facts; tier-1 wins | researching/web-search | JUDGMENT | CARROT LAZY
.claude/rulebook/stub-tracking.md | intentional stubs allowed ONLY when registered with babystep-id marker + stubs.md row w/ will_resolve_in | baby-step stub / PRE_GATE | HOOKABLE (PostToolUse: stub token requires matching stubs.md row) | STICK LAZY

================================================================
## .claude/skills/*/SKILL.md (name | trigger | what it does)
================================================================
.claude/skills/prompt-transform/SKILL.md | reformats user prompt before implementation; sets mcp_think budget | any implementation/bug-report/plan-phase (not !-prefixed / yes-no) | REFERENCE (mandatory workflow) | STICK ALWAYS-ON-trigger
.claude/skills/planning-workflow/SKILL.md | full planning/FSM/TDD/audit/phase-completion/plan-improvement protocol | plans, phases, improve/refine, execution, completion | REFERENCE | STICK LAZY
.claude/skills/rag-tools/SKILL.md | rules for local-rag MCP tools (search/memorize/ingest/forget/list) | any mcp_* RAG call | REFERENCE | STICK LAZY
.claude/skills/anti-spiral/SKILL.md | stagnation detector (Spinning/Oscillation/NoDrift/DiminishingReturns) + 5-persona switch | same file 3+ edits / hypothesis 3+ fail / audit 2x BLOCK / plateau | HOOKABLE-PARTIAL (edit-count + audit-streak detectable) | STICK LAZY
.claude/skills/orchestrate/SKILL.md | autonomous phase orchestrator: parse plan, FSM per sub-phase, dep wait, phase-end gates, commit once | "orchestrate [plan]" | REFERENCE | STICK LAZY
.claude/skills/auto-orchestrate/SKILL.md | general autonomous orchestration for ANY task; strategist-never-executor; run-state file; handoff mediation | "orchestrate this autonomously" / "auto-orchestrate" | REFERENCE | STICK LAZY
.claude/skills/b-plan-runner/SKILL.md | full B-N FSM cycle runner | B-N plan walk | REFERENCE | STICK LAZY
.claude/skills/idea-intake/SKILL.md | Layer-A intake: recursively decompose idea>Concept>Phase>Step>BabyStep via agent chain w/ per-level audits | idea/brief/"build me X" | REFERENCE | CARROT LAZY
.claude/skills/paper-to-plan/SKILL.md | end-to-end paper/repo steal pipeline (ingest>map>context>analysis>new plan files) | paper/repo link + "can elai use this" | REFERENCE | CARROT LAZY
.claude/skills/sidecar-protocol/SKILL.md | remote sidecar 3-step health check + boot wrapper | --remote/--local/live-inference phrases | REFERENCE | STICK LAZY
.claude/skills/impact-check/SKILL.md | pre-edit blast-radius; spawns impact-mapper; caches | first Write/Edit of EXECUTING touching crates/python/cross-lang | HOOKABLE-PARTIAL (could be PreToolUse:Edit first-touch trigger) | CARROT LAZY
.claude/skills/dashboard-handoff/SKILL.md | post-edit dashboard verification; spawns dashboard-smoker | after Write/Edit on dashboard-v2 entering AUDITING | REFERENCE | CARROT LAZY
.claude/skills/cook/SKILL.md | design discovery front-end (FRAME>DIVERGE via gpt_image renders>CONVERGE>EMIT brief); user-invoked only | /cook, "design this with me", "what should X look like" | REFERENCE | CARROT LAZY
.claude/skills/grill/SKILL.md | one-question-at-a-time interrogation to resolve all plan decisions before prompt-transform; user-invoked | /grill, "grill this", "poke holes" | REFERENCE | CARROT LAZY
.claude/skills/spec-eval/SKILL.md | evaluate ELAI AS SPECIFIED (don't discount planned/wire-dark); comparison/design-improvement | "compare ELAI to X" / "improve this design" / batch spec drop | REFERENCE | CARROT LAZY
.claude/skills/oracle/SKILL.md | interactive grounded Q&A; spawns researcher in oracle mode w/ web grounding | /oracle, /ask, deep factual question | REFERENCE | CARROT LAZY
.claude/skills/pdf/SKILL.md | PDF/image/DOCX>Markdown/text + figures via MinerU | /pdf <path>, "extract this pdf" | REFERENCE | CARROT LAZY

================================================================
## .claude/agents/*.md (name | role | model)
================================================================
.claude/agents/auditor.md | hostile Senior Security Auditor; BLOCK/WARN/INFO/DEAD-END; sole test runner; runs verification scripts in audit-scratch/ | sonnet | REFERENCE-AGENT | STICK
.claude/agents/context-compiler.md | compiles context package for a session/sub-phase (RAG + plan reads) | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/memory-writer.md | writes memory-bank files via mcp_memorize/log_change (RAG-indexed) | haiku | REFERENCE-AGENT | STICK
.claude/agents/integration-explorer-elai.md | wiring discovery: what existing APIs/events to hook into (ELAI variant) | sonnet | REFERENCE-AGENT | STICK
.claude/agents/integration-explorer.md | generic wiring-discovery explorer (DUP of -elai) | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/scope-planner-elai.md | partitions work into scopes (ELAI variant) | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/scope-planner.md | generic scope partition (DUP of -elai) | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/synthesizer-elai.md | synthesizes explorer outputs into a plan (ELAI variant) | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/synthesizer.md | generic synthesizer (DUP of -elai) | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/impact-mapper.md | pre-edit blast-radius for crates/**/*.rs(+py/ts/svelte) | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/prompt-engineer.md | prompt transformation agent (backs prompt-transform skill) | sonnet | REFERENCE-AGENT | STICK
.claude/agents/executor.md | implements one bounded plan step; wires production hook; adds unit test; FREE glm-5.2 | glm-5.2 | REFERENCE-AGENT | STICK
.claude/agents/fixer.md | applies targeted fixes from audit findings | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/researcher.md | tree-structured grounded research w/ mandatory floors; oracle mode | sonnet | REFERENCE-AGENT | STICK
.claude/agents/paper-ingestor.md | ingest paper(mode=paper)/source(mode=source) to RAG + interface map | sonnet | REFERENCE-AGENT | STICK
.claude/agents/spec-grounder.md | verify "Spec references:" claims map to real spec content; classify GROUNDED/VERIFY/INGEST/UNGROUNDED | sonnet | REFERENCE-AGENT | STICK
.claude/agents/wiring-contract-validator.md | validates a B-N WIRING_CONTRACT (frozen gate agent) | sonnet | REFERENCE-AGENT (FROZEN) | STICK
.claude/agents/dashboard-smoker.md | svelte-check + cargo check + optional tauri:dev for dashboard changes | sonnet | REFERENCE-AGENT | STICK
.claude/agents/sidecar-health.md | remote sidecar health-check agent | sonnet | REFERENCE-AGENT | STICK
.claude/agents/test-attributer.md | attributes build/test errors to source via mcp_attribute_build_errors | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/adversary.md | cross-family adversarial critic (D-13 FalsifiabilityCrossFamily); refuses same-family | sonnet | REFERENCE-AGENT | STICK GAME-adjacent
.claude/agents/idea-framer.md | top of decomposition cascade — frames the raw idea | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/idea-critic.md | adversarial review of the framed idea (loop w/ framer) | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/concept-decomposer.md | concept > phases | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/phase-decomposer.md | phase > steps | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/step-decomposer.md | step > baby steps | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/babystep-author.md | writes the 10-block baby-step detail file | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/designer.md | RETIRED (stub, Read-only) | sonnet | REFERENCE-AGENT (DEAD) | CARROT
.claude/agents/persona-architect.md | anti-spiral persona: map dependency graph, find coupling root, decoupling baby_step | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/persona-contrarian.md | anti-spiral persona: opposite assumption, alt baby_step | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/persona-hacker.md | anti-spiral persona: shortest path to working code | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/persona-researcher.md | anti-spiral persona: 3 prior-art refs, next action | sonnet | REFERENCE-AGENT | CARROT
.claude/agents/persona-simplifier.md | anti-spiral persona: strip to skeleton, add layers, name bad layer | sonnet | REFERENCE-AGENT | CARROT

================================================================
## .claude/tools/* (lang | what | CLI/--json surface)
================================================================
### Gate/checker tools (off-agent enforcement — the existing STICKS)
.claude/tools/gate_freeze_check.py | py | freeze the wiring-gate fileset (R8); content-hash (blake3/sha3) vs gate-manifest.lock; git-independent | --lock (re-baseline), default verify | HOOKABLE-GATE | STICK
.claude/tools/wiring_contract_check.py | py | verify B-N WIRING_CONTRACT fail-closed (R1-R9); runs runtime trigger; SSB-row proof | --bn-file PATH [--json --static-only --allow-missing-contract --run-trigger] | HOOKABLE-GATE (FROZEN) | STICK
.claude/tools/production_caller_check.py | py | D-030 Rust production-caller gate: new pub fn needs external non-test caller or WIRE-DARK marker; verdicts WIRED/INTERNAL/DARK/DARK-TYPE/REGISTERED-DARK/UNSOUND-NAME | --files a.rs,b.rs / --symbols [--json] | HOOKABLE-GATE (FROZEN) | STICK
.claude/tools/python_wire_check.py | py | D-030 Python sibling (AST, module-level public defs); same verdicts/exit/WIRE-DARK escape | --files a.py [--json] | HOOKABLE-GATE (FROZEN) | STICK
.claude/tools/maintainability_check.py | py | MR-1 file>1000 lines / MR-2 as-any/unsafe/ts-ignore / MR-3 fn>80 lines / MR-4 branching density over diff | --files <changed> --base HEAD [--json --git-staged --warn-as-block] | HOOKABLE-GATE (FROZEN) | STICK
.claude/tools/traced_test_assert_check.py | py | #[traced_test] needs asserted logs_contain; bans BatchSpanProcessor in tests | --files / --root [--json --expect-traced] | HOOKABLE-GATE (FROZEN) | STICK
.claude/tools/cross_lang_diff.py | py | flag changed file lacking cross-lang pair (Rust↔Py sidecar, Tauri↔Svelte); pairs from cross-lang-pairs.toml; opt-out comment | --files / git diff cached | HOOKABLE-GATE | STICK
.claude/tools/wiring_contract_check.py (see above) | — | — | — | — | —
.claude/tools/spec_link_check.py | py | verify "Spec references:" claims map to real docs/specs content | --file PATH [--specs-dir --json] | HOOKABLE-GATE | STICK
.claude/tools/agentskills_validator.py | py | agentskills.io v1 frontmatter validator (name/description/license/metadata) | <SKILL.md path> (exit 0/1) | HOOKABLE-GATE | STICK
.claude/tools/skill-audit.py | py | wraps NVIDIA skillspector; 64 patterns/16 categories; pre-promotion skill security gate | <skill-dir> [--llm]; runs under skillspector venv | HOOKABLE-GATE | STICK
.claude/tools/knip_zero_module_wrapper.py | py | knip --production wrapper that FAILs on zero-module (fail-open prevention, R2) | (subprocess knip) | HOOKABLE-GATE | STICK
.claude/tools/stop_verified_lint.py | py | E-49: refuse HumanEval/MBPP/SWE-bench-Verified mention without STOP-Verified guard comment | scans .rs/.py/.ts/.svelte/.md [--json] | HOOKABLE-GATE | STICK
.claude/tools/elai_map_freshness.py | py | check elai.map older than newest source (FRESH/SLIGHTLY_STALE/STALE/MISSING) | [--map --source-root --regen-cmd --json] exit 0/1/2 | HOOKABLE-GATE | STICK
.claude/tools/deny-enterplanmode.ps1 | ps1 | PreToolUse hook: deny built-in EnterPlanMode, route to project planning skills | PreToolUse:EnterPlanMode (WIRED in settings.local.json) | HOOKABLE-GATE (LIVE HOOK) | STICK ALWAYS-ON
### Trading/ML validators (GAME-adjacent: financial-code gates)
.claude/tools/scan_backtesting.py | py | AST scanner for backtesting realism (zero costs, absent DSR/funding) tradingRules 12-19 | <file/dir> exit 0/1/2 | HOOKABLE-GATE | STICK GAME-adjacent
.claude/tools/trading_ast_scan.py | py | AST lookahead-bias/leakage scanner (stdlib ast) | <file/dir> exit 0/1/2 | HOOKABLE-GATE | STICK GAME-adjacent
.claude/tools/validate_drift.py | py | feature distribution drift (KS/PSI/rolling mean); needs numpy/pandas | CSV input | HOOKABLE-GATE | STICK GAME-adjacent
.claude/tools/validate_execution_engine.py | py | adversarial engine traps (zero_variance/time_travel/liquidity) | EngineProtocol | HOOKABLE-GATE | STICK GAME-adjacent
.claude/tools/validate_ohlcv.py | py | OHLCV data-quality validator (tradingRules R11) | CSV input | HOOKABLE-GATE | STICK GAME-adjacent
.claude/tools/validate_predictions.py | py | model-quality validator (overfit/PSR/calibration/DSR trial count) | numpy/pandas | HOOKABLE-GATE | STICK GAME-adjacent
.claude/tools/validate_strategy_null.py | py | null-hypothesis tests (whitenoise/shuffle) profitable-on-noise=BLOCK | StrategyProtocol/ModelProtocol | HOOKABLE-GATE | STICK GAME-adjacent
### Code-graph / impact / search tools
.claude/tools/codebase_mapper.py | py | builds elai.map (grep-flat TSV) + elai_graph.json from Rust/Py/TS/Svelte; SCIP-enriched | <repo> [out] [--compact --map --scip-dir] | REFERENCE-TOOL | CARROT
.claude/tools/graph_reconcile.py | py | union codegraph(live)+elai.map(SCIP) per symbol; confidence-tag HIGH/etc; --mode affected for scoped tests | <symbol> [--mode affected] | REFERENCE-TOOL | CARROT
.claude/tools/impact_radius.py | py | pre-edit blast radius via elai.map grep | --symbols / --files [--json --map] exit 0/1/2 | REFERENCE-TOOL | CARROT
.claude/tools/bn_status_report.py | py | B-N punch list (SHIPPED/IN-FLIGHT/PLANNED/MISSING_FILE) from B-plan dir + index | --filter done|partial|todo|all | REFERENCE-TOOL | CARROT
.claude/tools/failure_taxonomy.py | py | deterministic build/test+agent-trace failure classifier (27 categories); auditor pre-read | --input / --stdin | REFERENCE-TOOL (auditor) | CARROT
.claude/tools/query_graph.py | py | query elai_dependency_graph.json (callers/callees/locate/module/stats) | <graph> <cmd> <arg> | REFERENCE-TOOL | CARROT
.claude/tools/query_bm25.py | py | one-off: find BM25 integration in dep graph | (hardcoded) | REFERENCE-TOOL (one-off/dead) | CARROT
.claude/tools/find_bm25_new.py | py | one-off: find Bm25Index::new call sites | (hardcoded) | REFERENCE-TOOL (one-off/dead) | CARROT
### Paper / sidecar / misc tools
.claude/tools/fetch_arxiv.py | py | fetch arXiv paper (HTML>PDF) to .claude/papers/ for ingestion | <id|url> | REFERENCE-TOOL | CARROT
.claude/tools/extract_pdf_figures.py | py | extract figures from PDF via PyMuPDF (raster + page render) | (function) | REFERENCE-TOOL | CARROT
.claude/tools/measure_paper_tokens.py | py | tiktoken count of paper HTML/raw (needs tiktoken+bs4) | (hardcoded ROOT) | REFERENCE-TOOL | CARROT
.claude/tools/regen_sidecar_schemas.py | py | E-45: regen Pydantic sidecar mirror from Rust JSON-Schema fixtures via datamodel-codegen | (pipeline) | REFERENCE-TOOL | CARROT
.claude/tools/scip_pb2.py | py | generated protobuf code for SCIP (DO NOT EDIT) | (lib) | REFERENCE-TOOL (generated) | CARROT
### Canvas / session hooks (ps1/mjs — some LIVE in settings.json)
.claude/tools/canvas-handoff-hook.mjs | mjs | UserPromptSubmit hook: inject clicked-canvas-element context on next prompt | UserPromptSubmit (WIRED in settings.json) | HOOKABLE (LIVE HOOK) | CARROT ALWAYS-ON
.claude/tools/canvas-activity-hook.ps1 | ps1 | PostToolUse/SubagentStop: feed live-activity overlay (touched file>graph node) | PostToolUse Edit/Write/Read + SubagentStop (WIRED) | HOOKABLE (LIVE HOOK) | CARROT ALWAYS-ON
.claude/tools/canvas-prompt-inject.ps1 | ps1 | UserPromptSubmit convenience: inject fresh canvas selection as additionalContext | UserPromptSubmit (alt path) | HOOKABLE | CARROT LAZY
.claude/tools/session-rag-sync.ps1 | ps1 | PostToolUse: rolling 1-row-per-session local-rag breadcrumb every 5th edit (forget>memorize); fail-open | PostToolUse Edit/Write (WIRED in settings.json) | HOOKABLE (LIVE HOOK) | CARROT ALWAYS-ON
### Visual UI loop tools (.claude/tools/visual/, own package.json, bun/node)
.claude/tools/visual/ui-diff.mjs | mjs | pixelmatch AA-aware diff + hotspots/bbox + native-res crops; exit 0/2/1 | <candidate.png> <ref.png> --out <dir> | HOOKABLE-PARTIAL | STICK LAZY
.claude/tools/visual/ax-scaffold.mjs | mjs | CDP AX-tree+box-model perception scaffold [{role,name,bbox}] | --url --out scaffold.json | REFERENCE-TOOL | CARROT LAZY
.claude/tools/visual/vlm-prep.mjs | mjs | box-downscale screenshot to 1568px longest side for VLM | <screenshot.png> | REFERENCE-TOOL | CARROT LAZY
### Pipeline tools (.claude/tools/pipeline/ — determinism/conformance)
.claude/tools/pipeline/determinism_check.py | py | determinism/repro check (pipeline) | (pipeline) | HOOKABLE-GATE | STICK
.claude/tools/pipeline/baseline_record.py | py | record pipeline baseline | (pipeline) | REFERENCE-TOOL | CARROT
.claude/tools/pipeline/reconcile.py | py | pipeline reconcile | (pipeline) | REFERENCE-TOOL | CARROT
.claude/tools/pipeline/run_pipeline.py | py | run the pipeline | (pipeline) | REFERENCE-TOOL | CARROT
.claude/tools/pipeline/test_quality_scan.py | py | quality scan over pipeline output | (pipeline) | HOOKABLE-GATE | STICK
.claude/tools/pipeline/_conformance_data.py | py | conformance fixture data | (lib) | REFERENCE-TOOL | CARROT
### Static data artifacts (not scripts)
.claude/tools/elai.map | data | grep-optimal flat code map (~375K lines, SCIP-enriched) | grep <symbol> | REFERENCE-DATA | CARROT
.claude/tools/elai_graph.json | data | structured twin of elai.map (reachability calls array) | — | REFERENCE-DATA | CARROT
.claude/tools/gate-manifest.txt (.claude/gate-manifest.txt) | data | frozen wiring-gate fileset (R8); ~40 entries: rules + checkers + .elai_cc gate .rs/.toml + canary fixtures | gate_freeze_check.py reads | REFERENCE-DATA (FROZEN MANIFEST) | STICK
.claude/tools/requirements.txt | data | python deps for tools | — | REFERENCE-DATA | CARROT
.claude/tools/README.md | doc | codebase_mapper usage doc | — | REFERENCE-DOC | CARROT

================================================================
## .claude/commands/*.md
================================================================
.claude/commands/canvas-mode.md | set canvas mode flag (code|plan|research|design|arch); plain doc flag NOT FSM substate; no mcp_fsm_* call | /canvas-mode <mode> | REFERENCE-COMMAND | CARROT LAZY
.claude/commands/skill-audit.md | run skillspector skill-security gate before promoting a skill | /skill-audit <pkg> [--llm] | REFERENCE-COMMAND (wraps skill-audit.py) | STICK LAZY

================================================================
## 1. DUPLICATES / OVERLAP (dedup candidates for ELAI+liteGAME merge)
================================================================
- orchestrate-protocol: TWO copies — .claude/rules/orchestrate-protocol.md (H-1..H-11 full) AND .claude/rulebook/orchestrate-protocol.md (O-HR1..O-HR10 one-screen digest). Same doctrine, two encodings. Merge to one.
- Generic vs -elai agent pairs: integration-explorer.md / integration-explorer-elai.md; scope-planner.md / scope-planner-elai.md; synthesizer.md / synthesizer-elai.md. The -elai variants are canonical (CLAUDE.md agent list names the -elai forms); the generic forms are likely liteGAME-shared leftovers. Dedup at merge.
- Wiring doctrine cluster: wiring-enforcement.md (R1-R10) ⊃ production-caller-gate.md (D-030 = R6/R7 changeset leg) ⊃ tracing-test-assert.md (Layer-4 leg) ⊃ maintainability-review.md (orthogonal "is it maintainable"). Four files, one fail-closed family — keep separate but cross-reference once.
- Zero-lazy-code appears in BOTH CLAUDE.md and token-discipline.md and stub-tracking.md (the stub-registry exception). Single source needed.
- Agent-spawning rule duplicated: CLAUDE.md "Agent spawning" block AND .claude/rulebook/agent-spawning.md AND main-llm-role.md. Same "never read agents/*.md, model=sonnet" content 3x.
- Handoff protocol: CLAUDE.md pointer + .claude/rules/handoff-protocol.md (full) — pointer/full split is intentional but the full body is long; candidate for RAG.
- Anti-sycophancy / two-approaches / completeness / confidence-calibration: duplicated verbatim in CLAUDE.md AND main-llm-role.md.
- FSM-cleanup / MEMORIZING-delegation: in CLAUDE.md AND .claude/rulebook/memorizing-and-audit.md AND main-llm-role.md.
- Trading validators (7 scan_*/validate_* + scan_backtesting + trading_ast_scan): a self-contained financial-code gate cluster; GAME-adjacent (only fires on trading/ML code) — likely shared with liteGAME or a separate domain pack; isolate as an optional module.
- file-discipline.md (Swift/iOS LOC caps) overlaps maintainability MR-1 (file>1000 lines); file-discipline is stale-for-Rust (Swift-flavored) — fold its LOC cap into MR-1.
- One-off dead scripts: find_bm25_new.py, query_bm25.py (hardcoded paths, single-purpose investigations) — drop at merge.

================================================================
## 2. ALREADY-A-GATE (existing hooks/checkers/frozen machinery actually found)
================================================================
LIVE HOOKS (wired in settings.json / settings.local.json):
- PreToolUse Write|Edit|Bash|NotebookEdit > rag/hooks/pre_tool_check.py (FSM path/tool gating per state+substate + cross-session file locks)
- PreToolUse EnterPlanMode > deny-enterplanmode.ps1 (routes to project planning skills) + permissions.deny["EnterPlanMode"] hard floor
- PostToolUse Write|Edit > rag/hooks/post_tool_check.py (ruff syntax check, template preservation, change recording, zero-lazy enforcement)
- PostToolUse Edit|Write|MultiEdit|Read > canvas-activity-hook.ps1 (live overlay)
- PostToolUse Edit|Write|MultiEdit > session-rag-sync.ps1 (rolling rag breadcrumb every 5th edit)
- UserPromptSubmit > canvas-handoff-hook.mjs (inject clicked-element context)
- SubagentStop * > canvas-activity-hook.ps1
- Stop "" > rag/hooks/stop_check.py (blocks mid-workflow termination; enforces post-write self-verify + zero-lazy)
- PreCompact "" > rag/hooks/pre_compact_check.py (auto SESSION_HANDOFF FSM snapshot)
  NOTE: the rag hook scripts live OUTSIDE this root at C:\Users\user\.claude\rag\hooks\ — not enumerated here but are the load-bearing always-on enforcement.

FROZEN GATE MACHINERY (.claude/gate-manifest.txt, ~40 frozen entries, locked via gate_freeze_check.py --lock):
- Harness checkers frozen: wiring_contract_check.py, gate_freeze_check.py, traced_test_assert_check.py, maintainability_check.py, production_caller_check.py, python_wire_check.py + their doctrine rules (wiring-enforcement.md, tracing-test-assert.md, maintainability-review.md, production-caller-gate.md) + wiring-contract-validator.md agent.
- .elai_cc gate code frozen (H-17..H-29, I-24, I-35, D-030): trial_ledger.rs, deflation_gate.rs, leakage_gate.rs, lookahead_differential.rs, trading_null_gate.rs, rerun_equality.rs, sim_real_parity.rs, result_manifest.rs, shadow_exec.rs, gate_self_validation.rs, reward_tampering.rs, promotion_gate.rs, holdout_budget.rs, drift_interlock.rs + their seed .toml + sidecars + elai_cc.toml + smoke-manifest.toml + wire-dark-allowlist.toml + locked_keys_registry.toml + canary_pairs.json.

STANDALONE CHECKERS (auditor-invoked, not auto-hooked but runnable gates):
- maintainability_check.py, production_caller_check.py, python_wire_check.py, traced_test_assert_check.py, wiring_contract_check.py, cross_lang_diff.py, spec_link_check.py, agentskills_validator.py, skill-audit.py, knip_zero_module_wrapper.py, stop_verified_lint.py, elai_map_freshness.py, gate_freeze_check.py.
- Trading gate cluster: scan_backtesting.py, trading_ast_scan.py, validate_{drift,execution_engine,ohlcv,predictions,strategy_null}.py.
- Pipeline: determinism_check.py, test_quality_scan.py.
- Visual: ui-diff.mjs (deterministic pixel gate, exit 0/2/1).

FROZEN TOOL: gate_freeze_check.py is the freeze-the-freezer (content-hash blake3/sha3 vs gate-manifest.lock, git-independent).

================================================================
## 3. CONTEXT-COST (always-on vs lazy; reducing always-on is the point)
================================================================
ALWAYS-ON (loaded into context EVERY session — the reduction target):
- Two CLAUDE.md files (global + project) — the LARGEST always-on cost; dozens of HARD RULE blocks. Many are HOOKABLE (project boundary, no-AI-trailer, anti-stale grep, no-unwrap, context-delegation, agent-spawning, main-never-tests, zero-lazy) and could move to deterministic hooks, leaving only JUDGMENT carrots (grounded-by-default, GRC, anti-sycophancy, ask-don't-assume, two-approaches) in prose.
- .claude/rules/ files auto-load (conditional-routing routes most, but token-discipline.md, main-llm-role.md, output-formatting.md, conditional-routing.md, agent-dispatch-preamble.md are effectively always-on per CLAUDE.md "auto-load every session with same authority").
- LIVE hook scripts (rag pre/post/stop/precompact + canvas + session-rag-sync) — runtime cost not context cost; keep.
- Reference-heavy ALWAYS-ON blocks that should move to RAG: CLAUDE.md "Project PATHs", "Remote sidecar (canonical)", E-10/E-43/E-31 pointers, SCHEMA_VERSION/TOTAL_VARIANTS, Anti-Stale crate list, harness-helper-tool catalog. These are lookup material, not behavioral rules — pure RAG candidates.

LAZY (loaded only on trigger via conditional-routing — already cheap):
- All .claude/rulebook/*.md (dispatched on trigger).
- Most .claude/rules/ files gated by conditional-routing (bun-not-node, commit-cadence, maintainability-review, orchestrate-protocol, production-caller-gate, skill-import-policy, tracing-test-assert, visual-ui-loop, wiring-enforcement, handoff-protocol).
- All skills (fire on phrase/state trigger).
- All agents (loaded only inside their own dispatched context, never main).
- All tools (invoked, never auto-read).

NET RECOMMENDATION for the redesign: ~15-18 of the CLAUDE.md HARD RULE blocks are mechanically enforceable (HOOKABLE/STICK) and belong in PreToolUse/PostToolUse/Stop hooks; ~10 are pure JUDGMENT (CARROT) and stay as terse prose; ~8 are REFERENCE blocks that belong in RAG. The two CLAUDE.md files + the always-on rules/ set are the entire context-cost lever.

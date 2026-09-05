Token discipline + behavioral rules (HARD RULE)

Loaded by: ALWAYS. These are the cluster of one-liners that govern how the main LLM moves.

Token economy

Every token in claude.md, .claude/rules/, and memory-bank costs context window space in future sessions. One-line rules where possible. Redundancy between files is a bug.

Never load entire files into context. Use projectIndex.md as the topological map. Grep before reading. Read line ranges, not full files. Max 5 lines per changelog entry.

Anti-read-big (HARD RULE)

The main LLM hard-refuses Read >200 lines OR Grep with >50 lines of output without spawning a subagent first. Agents have their own contexts that garbage-collect on return; the main LLM context window does not. Specifically:
  - Read with implicit full-file read on files >200 lines: REFUSE. Use offset + limit to read line ranges, OR delegate to context-compiler / Explore subagent.
  - Grep with output_mode=content + head_limit unset on a wide pattern likely to return >50 lines: REFUSE. Tighten the pattern, OR use output_mode=files_with_matches first, OR delegate.
  - decomposition tree files (workflow/sessions/{id}/babysteps/*.md, concepts/*.md, exploration/*.md): main LLM may Read the SINGLE CURRENT baby step detail file (~100 lines); other artifacts at those paths go through agents per HEAVY-flow rules.
  - reference docs in docs/ + memory-bank/: always delegate to context-compiler or paper-ingestor.

EXCEPTION: deliberate scoped Reads of target files you are about to Edit are allowed (the existing file's current state is required input for a correct Edit). Anti-read-big applies to RESEARCH / EXPLORATION reads, not to AUTHORING reads on a file you are about to write.

Read-tool-before-edit (HARD RULE, anti-token-waste): the Edit/Write tools credit a prior read ONLY when it came from the Read tool. A Bash read (cat / type / Get-Content / sed -n / head / tail) does NOT satisfy the precondition, so an Edit that follows only a Bash read FAILS ("must read first") and forces a re-read -- the failed Edit echoes its full diff first, so it is pure wasted tokens. Therefore: before any Edit/Write to an existing file, Read it with the Read tool (scoped offset/limit ranges are fine and preferred for big files -- read just the regions you will edit). Never substitute Bash cat/grep for the pre-edit Read on a file you intend to mutate. This is NOT the GRC evidence gate (that hook is log-only unless .claude/state/grc-v2-enabled exists); it is the built-in harness read-state invariant and no hook can bypass it.

Read-means-read (HARD RULE, overrides anti-read-big on explicit user intent)

When the user explicitly says "read X", "look at X", "go through X", "check X yourself", or otherwise tells the main LLM to read a file (or set of files): the main LLM Reads the file(s) directly. Do NOT delegate to Explore / context-compiler / general-purpose subagent. The user wants the main LLM's eyes on the source, not a summarized version.

When the task is reasoning, architecture design, tradeoff evaluation, multi-file analysis, debugging, or any work where source-fidelity matters more than token economy: the main LLM Reads directly. Explore-style subagents return summaries; reasoning over summaries-of-summaries produces shallow output. Do not pre-compress the inputs to your own reasoning.

Explore / context-compiler / general-purpose subagents remain the right choice for:
  - open-ended search ("where is X defined", "which files reference Y")
  - broad multi-file lookups producing a focused report
  - genuinely huge files where direct read would blow context AND no user instruction overrides

When in doubt about an explicit user "read" instruction: read directly. Cost of a Read tool call is much cheaper than producing low-fidelity reasoning the user has to redo.

Search before speaking

Prefix unsearched claims with "From training (unsearched):". If the topic is current-API or version-specific, WebSearch first.

External grounding

Verify via WebSearch when referencing algorithms, APIs, papers, or version-specific features by name.

Impact analysis

Before changing any function, grep all call sites first. If blast radius > 5 files and no plan exists, STOP and create a plan.

One change at a time

Write and audit before the next change. Main model never runs tests; the auditor is the sole test runner.

Pre-implementation risk check

Identify top 1-3 failure modes before any non-trivial change.

Post-write self-verification

After writing any function >10 lines, re-read and grep for TODO / placeholder / fatalError / force-unwrap / silent swallows. Fix immediately. The Stop hook enforces this.

Zero lazy code (HARD RULE)

No empty function bodies, TODO, placeholder returns, fatalError("unimplemented"), force-unwrap (!), force-try (try!), force-cast (as!), mock data in prod, "// ..." comment-out, stub implementations. Same severity as a syntax error. The Stop hook enforces this.

EXCEPTION via stub registry (per .claude/rules/stub-tracking.md): a registered stub with babystep id embedded in the marker (e.g. `// TODO[C2.P1.S3.B2]: replace mock with KeychainStore.fetch()`) AND a corresponding row in workflow/sessions/{id}/stubs.md with will_resolve_in pointing to a future baby step is allowed during plan execution. Unregistered stubs and banned tokens (force-unwrap, silent swallow) remain BLOCK regardless. Plan COMPLETE state requires all stubs status != open.

Reasoning-after-action

After a failed test / unexpected output / tool error: MUST mcp_think before next edit. No blind retries. 3 failed hypotheses = add debug logging.

Anti-spiral

Same file edited >3 times for the same issue without resolution = STOP. mcp_think(step_type="status") to assess. Spiraling = escalate or rethink.

Diagnose from actual data

Grep logs before forming any theory. If 3 hypotheses fail, add debug logging. Do NOT pattern-match training data over real evidence.

No removal or simplification without explicit user confirmation. No new dependencies without confirmation.

Anti-slopsquatting (HARD RULE)

NEVER assume a dependency exists. Verify package name + repo URL + version + last-published date + license BEFORE adding. The verification protocol is mandatory regardless of how confident you feel about the package name:
  - npm packages:    `npm view {pkg} version dist-tags time` and confirm the result lists a `latest` tag with a publish date in the last 24 months
  - pypi:            `curl -s https://pypi.org/pypi/{pkg}/json | python -c "import json,sys;d=json.load(sys.stdin);print(d['info']['version'],d['info']['license'])"`
  - SPM:             WebFetch the canonical github repo and find the most recent release tag; verify the repo is owned by the org publishing the framework
  - cocoapods:       WebFetch cocoapods.org/pods/{pkg}
  - swiftpackageindex: WebFetch swiftpackageindex.com/{owner}/{pkg} for community-verified packages

Researcher agent and designer agent run this check at plan time per .claude/agents/researcher.md and .claude/agents/designer.md. Executor runs it again at edit time before the import line is added. Auditor runs it again at audit time.

NEVER trust training memory for package names. Even if you "know" the package, verify via the registry. Slopsquatting (typo-squatted malicious packages with names close to real ones) is a real attack vector and the LLM is vulnerable. Verification is the only defense.

NEVER assume an architecture works the way you think it works — verify the real implementation code / plan / spec.

Anti-stale-training (HARD RULE)

Training data is 1+ year old. Default to verified current practices over training instincts. Swift/iOS-specific staleness rules in .claude/rules/anti-stale-swift-ios.md once filled. General rule: when unsure if a library / API / pattern is current, WebSearch before using it.

Compaction resilience

PreCompact hook auto-saves FSM state to SESSION_HANDOFF. After compaction, re-read activeContext.md and current plan section before continuing.

Plan-improvement protocol (mandatory)

When the user says "improve", "refine", "ground", or "let's improve" ANY baby step / phase / concept / step / plan file: execute the Plan Improvement protocol in SECTION 2 of planning-workflow SKILL.md. Do NOT freestyle or summarize in chat. Execute the protocol steps in order. The protocol works at any tree level (baby step / step / phase / concept).

Grounding sweep (post-approval, pre-TESTING_FIRST)

After the user-approved task definition is locked, scan the task for claims about external behavior. Classify each: GROUNDED, VERIFY, INGEST. Block TESTING_FIRST until all claims resolved. Log results in workflow/reasoning/phase-X-grounding.md.

Paper-grounding: spawn paper-ingestor (mode=paper) with the arXiv ID or paper name. Do not start EXECUTING until the interface map is in workflow/papers/.

Source-grounding: spawn paper-ingestor (mode=source) with the file paths or SDK name. Decompiled output is never ground truth.

OPENCLAW VS ELAI — FULL-SURFACE COMPARISON, STEAL LIST, AND WIN MAP
date: 2026-07-11
mode: spec-eval (ELAI judged AS-SPECIFIED across docs/thefinalglance B..Q + phase index; shipped-vs-planned never feeds a design verdict)
sources:
  - openclaw clone F:\.ELAI_workflow\docs\.source\openclaw (HEAD 2026-07-10, version 2026.6.11, 22,435 files, MEDIUM reliability reviewed source)
  - openclaw-SUMMARY.md (prior source-grounding pass, 7 questions + 8 steal candidates)
  - two opus explorer inventories this session: core engine (agent loop, sessions, context, memory, multi-agent, providers, cost, skills, plugins, exec/sandbox, config) + surfaces/ops (gateway protocol, ~30 channels, nodes, voice, UI, automation, QA, browser, ClawHub, audit/trajectory, ops, security posture). Both doc-grounded with path cites; line-level runtime claims labeled UNVERIFIED-in-source where the doc was the only evidence.
  - ELAI side: ELAI-SPEC-OVERVIEW.md (canonical design map, phases 0-4M + letters B..Q)
prior-art check: grep of docs/thefinalglance for openclaw/clawdbot/lobster/clawhub = 0 hits. OpenClaw is untouched source material; no double-authoring risk.
porting caveat (user-supplied context + our own read): OpenClaw's commit stream is largely agent-generated at very high velocity; quality is uneven by construction. STEAL DESIGNS, NEVER CODE. Every mechanism below must be verified against the actual source (not just docs) before a plan unit ports it — the inventories are doc-derived.

====================================================================
1. THE ONE-PARAGRAPH VERDICT
====================================================================

These are different machines aimed at different problems. OpenClaw is a shipped, operationally mature PERSONAL ASSISTANT GATEWAY: one daemon owning ~30 messaging channels, device nodes, voice, browser, and an automation stack, with genuinely excellent ops hygiene (config lifecycle, doctor, backup, failover) and a clearly written single-trusted-operator security boundary. ELAI is a VERIFICATION SPINE for autonomous execution: seeded byte-replay, off-agent evidence, fail-closed done-gates, taint propagation, claim verification, and a coding-specific knowledge/reasoning engine. OpenClaw has nothing that answers ELAI's core thesis — it has no seeded determinism, no replay-verify, no off-agent done-gates at runtime, no claim grounding, and it EXPLICITLY declares prompt-injection-only attacks out of scope (SECURITY.md triage boundary). Conversely OpenClaw has a decade of product-surface and ops decisions ELAI's corpus only sketches (M/O/P/Q), and ~20 of those decisions are worth stealing as concrete mechanics. Money and size bought OpenClaw breadth and polish; it did not buy a trust spine. ELAI's wedge survives contact fully intact — and gets sharper, because OpenClaw is a perfect example of the "agent infrastructure without receipts" the wedge sells against.

====================================================================
2. FEATURE-BY-FEATURE MAP
====================================================================
Verdict vocabulary: ELAI WINS (design-level), OPENCLAW WINS (design-level), PARITY (same idea both sides), DIFFERENT GAME (not comparable, different product goals). Maturity is NOT in these verdicts; see section 5.

--- 2.1 Determinism / replay / journal ---
OpenClaw: trajectory bundles — versioned JSONL (traceSchema openclaw-trajectory, schemaVersion 1), manifest, redaction, 10MiB rolling sidecar, export via exec approval. Append-only tree transcripts (id+parentId), compaction writes a successor transcript, never in-place rewrite.
ELAI (spec): Phase-0 entropy quarantine (ClockProvider/EntropySource, byte-identical seeded runs), SSB single-writer append-only journal, G RRA (action stream + seed + hardware pin + gate-verdict hashes + elai replay --verify), J re-executable RRA + journal-completeness substrate (prove every side-effect produced a row), Q per-turn local token-hashing (localize replay divergence to first divergent turn).
Verdict: ELAI WINS, decisively. OpenClaw's trajectory is a FLIGHT RECORDER — you can inspect what happened; you cannot re-execute and verify it happened. No seed, no entropy control, no hash-bound verdicts, no replay-verify contract. ELAI's design makes the whole session recomputable and the claim checkable. This is the single biggest structural gap in OpenClaw and the core of ELAI's wedge.
Steal anyway: the ONE-envelope discipline (runtime, transcript, and export all emit the identical schema so external tooling has one shape) — compose into SSB export/RRA rendering. Also the successor-transcript compaction (never rewrite in place) — validates 3H's design.

--- 2.2 Evidence / done-gates / verification ---
OpenClaw: qa-lab confidence-report (classifies proof artifacts against a manifest into a ZERO-UNKNOWN report) + confidence-self-test (seeded negative-control canaries proving the gate detects drift) + exact dotted coverage-IDs as proof targets + posture labels so evidence cannot over-claim. But: this is a DEV-SIDE CI harness. At runtime, the agent's own text is trusted; there is no per-task off-agent done-gate, no test-integrity hash binding, no wiring gate.
ELAI (spec): the entire spine — off-agent evidence, missing-evidence-is-FAIL, frozen gates (F/G), test-integrity + output-grounding gate (claimed PASS bound to orchestrator-captured hash), wiring enforcement R1-R10, Q evidence-corruption corpus (every frozen checker proven to fail-closed against truncated/corrupt/missing/flipped evidence — a measured per-gate kill-rate).
Verdict: ELAI WINS at runtime (no contest — OpenClaw doesn't play this game per task). PARITY at CI grain: their confidence-report/self-test is the same philosophy as wiring-enforcement R1/R2 + Q's corruption corpus, independently converged. That independent convergence is itself evidence the design is right.
Steal: the zero-unknown classification shape and the seeded-canary self-test mechanics as concrete reference implementations when Q's evidence-corruption corpus is built. Also posture labels (user-path / direct-gateway / native-approval) — tag every ELAI smoke with the entrypoint posture it actually drove, so R10 evidence can't silently over-claim.

--- 2.3 Security: injection, taint, write-gates ---
OpenClaw: trust model is LOCAL-FIRST SINGLE TRUSTED OPERATOR, written down normatively in SECURITY.md (35KB). Prompt-injection without a policy/auth/approval/sandbox bypass is EXPLICITLY not a vulnerability. Mitigations that exist are perimeter: untrusted-content fencing (wrapExternalContent), screenshots model-only, browser evaluate kill-switch, SSRF fail-closed, sandbox off-by-default.
ELAI (spec): adversarial input is the FIRST-CLASS threat model — F write-gates (vuln/dep/secret/skill, blocking pre-disk), F data-plane tool-filter against injected instructions, G harness-wide taint propagation (untrusted -> destructive-sink blocked deterministically), G Dual-LLM lethal-trifecta quarantine, J transformation-surviving taint binding, E prompt-injection defense wave.
Verdict: ELAI WINS on the axis that matters commercially. OpenClaw's boundary is defensible for its product (one trusted human on their own box) but it means the harness is structurally unsafe the moment untrusted content flows in — which is EVERY message from a ~30-channel ingest surface. ELAI's taint design treats exactly that flow as the problem. This is the sharpest positioning contrast: quote their SECURITY.md line in any competitive claim (K claims-policy compliant: it is their own written policy, tag [EXTERNAL]).
Steal: the DISCIPLINE of a normative written triage boundary. ELAI's corpus enforces rules internally but has no single public SECURITY.md-equivalent that says what is and is not a vulnerability in ELAI's model. M compliance-export/appsec-brief is adjacent; add the normative triage doc to it.

--- 2.4 Exec approvals / sandboxing ---
OpenClaw: best-in-class approval mechanics. Effective policy = STRICTER-OF-TWO (config vs host approvals file; approvals only tighten). askFallback=deny when no UI (fail-closed). Approval BINDS a canonical systemRunPlan (cwd/argv/env/pinned executable + one concrete file operand); the plan is frozen between prepare and exec; caller edits after approval = mismatch rejection; file operand changed after approval = DENIED (drift protection, anti-TOCTOU). strictInlineEval treats python -c/node -e as approval-only. Sandbox (docker: network none, readOnlyRoot, capDrop ALL) is blast-radius reduction, off by default, gateway itself never sandboxed.
ELAI (spec): 4A process isolation (agents hold no network/LLM/tools, 15-message IPC), I directive Supervisor (sole side-effect executor, generator emits pure Intents), G post-expansion FS gate + egress IP-pin, E Firecracker sandboxing, 3A per-transition risk-scaled human gates.
Verdict: PARITY-plus-split. ELAI's ISOLATION design is stronger (the generator structurally cannot reach the sink; OpenClaw's model can, gated by policy). OpenClaw's APPROVAL-BINDING design is stronger (ELAI's corpus has approval gates everywhere but nowhere states the anti-TOCTOU invariant: an approval must bind the exact canonical plan + operand fingerprints it was granted for, and drift = deny).
Steal (high value, net-new): approval drift-binding as a universal ELAI invariant — every human/autonomous approval in 3A/4G binds a canonical, hashed action plan; execution re-validates; any drift fails closed. Composes with G blast-radius gates and the J config-ingress gate. Also stricter-of-two policy composition — M's org policy pack already specifies monotone-tighten at org grain; apply the same algebra at the exec-approval grain.

--- 2.5 Model providers / routing / failover ---
OpenClaw: ~140 provider plugins, auth profiles (api_key/oauth/token) in per-agent SQLite with config carrying only routing metadata; rotation order; SESSION STICKINESS (profile pinned per session for cache warmth, rotates only on reset/compaction/cooldown); failure-class cooldown taxonomy (rate-limit bucket 30s->1m->5m; billing -> disabled lane 5h->24h; auth-permanent 10-60m; MODEL-SCOPED cooldowns so sibling models still try); persist-override-BEFORE-retry to close the live-switch race with narrow rollback; explicit user selections are STRICT (fail loudly, never silently fall back); auto-override reprobes origin every 5min.
ELAI (spec): I auth broker + unified model catalog (competitor-steal ports), J BYOK + custom endpoints + free-API rotation pool + model-strength modality router, B/4M RuntimeAdapter multi-backend with per-role fallback chains, H calibrated architect/editor/verifier routing, I ledger-trained router, K routing-economics bench.
Verdict: OPENCLAW WINS on failover mechanics (this is their most battle-tested subsystem; ELAI's rotation-pool spec is thinner), ELAI WINS on routing INTELLIGENCE (ledger-trained on verified outcomes, $/verified-task economics, per-role calibration — OpenClaw routes by availability, ELAI routes by measured value).
Steal (high value): the whole failure-semantics package into J's free-API rotation pool + I's auth broker: (a) failure-class cooldown taxonomy with separate billing/auth/rate-limit lanes, (b) model-scoped vs profile-scoped cooldowns, (c) session-stickiness-for-cache-warmth as an explicit invariant (composes with H cache-stable layout — rotation is a cache-invalidation event and must be journaled as one), (d) persist-before-retry, (e) strict-explicit-selection (user's explicit model choice never silently substituted — also matches Q swap-restart's spirit: substitution is a first-class recorded event, never a silent fallback).

--- 2.6 Context management / caching / compaction ---
OpenClaw: self-built system prompt with an EXPLICIT stable/volatile cache boundary (byte-identical stable prefix: tool defs, skills index, workspace files; volatile suffix below), deterministic tool-catalog sort, per-file/total bootstrap budgets, promptMode full/minimal/none. Compaction: threshold + provider-overflow recovery, keeps tool-call/result pairs together, safeguard mode with quality audits, successor transcripts, pluggable context engine with failure quarantine to legacy. cacheRetention mapped per provider (Anthropic 5m/1h cache_control, OpenAI prompt_cache_key, Gemini cachedContents).
ELAI (spec): 3E 8-slot fixed budget with deterministic cache breakpoints, H cache-stable layout + 1h TTL + output-token stack, I cache-invariant economics middleware (byte-frozen prefix discipline + TTL tiering + cache-hit hard guard), L context-core inversion (immutable transcript + pure fold-view projection so cache-stability is STRUCTURAL, not disciplined), H/J context lifecycle (regime-aware local vs cloud policies).
Verdict: PARITY on mechanism, ELAI WINS on architecture: OpenClaw achieves cache stability by DISCIPLINE (careful ordering everywhere); L's fold-view makes it impossible to violate by construction. OpenClaw validates that the discipline pays (their per-provider cacheRetention mapping is exactly H's thesis shipped).
Steal: per-provider cache-control mapping table (their tested knowledge of what each provider actually honors), and the tool-call/result-pair-preserving compaction split rule as a compaction invariant. Also the failure-quarantine pattern for pluggable engines (broken engine -> quarantined + downgrade to known-good; composes with 3C supervision).

--- 2.7 Memory ---
OpenClaw: plain-Markdown workspace memory (MEMORY.md curated + daily notes indexed-not-injected + DREAMS.md), memory_search hybrid vector+keyword, PRE-COMPACTION MEMORY FLUSH (a silent NO_REPLY turn writes durable notes BEFORE compaction, once per cycle, soft threshold 4000 tokens), Dreaming (opt-in background consolidation cron promoting short-term recall to MEMORY.md through score/recall/diversity gates), memory-wiki plugin (claims/evidence/contradiction tracking).
ELAI (spec): H two-tier user memory (bi-temporal preferences + code-grounded validate-before-use facts, auditor-objectivity firewall, governance surface), J verified case graph (case node minted only on a PROVEN fix; execution-grounded reuse filter), J counterfactual memory-ROI attribution, H sleep-time cross-session handoff with grounded topic digest.
Verdict: ELAI WINS on trust semantics (validate-before-use, proof-gated case minting, ROI attribution — OpenClaw's memory is trusted prose the model wrote for itself; their own docs say "memory preserves context but does not enforce policy"). OPENCLAW WINS on lifecycle mechanics — the flush-before-compaction ordering and the consolidation loop are concrete and shipped-grade.
Steal (net-new mechanics): (a) pre-compaction memory flush as an ordered, journaled step in H's context lifecycle — in-session, distinct from H's cross-session sleep-time handoff; the invariant is "no compaction may destroy context that was never offered to durable memory"; (b) Dreaming's promotion gates (score/recall/diversity) as prior art for H's memory governance; (c) daily-notes-indexed-not-injected as a token-economy default.

--- 2.8 Skills / extensibility surface ---
OpenClaw: SKILL.md dirs with on-demand loading — prompt carries only a compact index (name/description/path + sha256 VERSION MARKER, ~24 tokens/skill), model reads the file when needed, re-reads on version change; snapshot per session; budget with compact-degrade; gating via requires.{bins,env,config}/os; per-agent allowlists; ClawHub install with trust envelope + operator installPolicy (fail-closed local exec gate that scans AFTER staging BEFORE install).
ELAI (spec): L familiarity surface (user-authorable rules/hooks/skills + one-shot verified builder, "safer than the tools they imitate"), H MCP defer-loading, harness-side skill-import-policy + skillspector gate.
Verdict: PARITY on intent, OPENCLAW WINS on the loading mechanics (the sha256-versioned on-demand index is clean, deterministic, cache-stable, and cheap), ELAI WINS on the trust bar (L's builder VERIFIES what it builds; ClawHub scans are advisory and their own docs admit no built-in dangerous-code blocking at install).
Steal: the versioned on-demand index shape for L's familiarity surface and for ELAI's own rules/skills injection (compose with H MCP defer-loading — same lazy-load pattern, one mechanism). The hidden-until-verified registry release gate (publish -> scan -> HIDDEN until review passes) composes with M's evidence-standard distribution if ELAI ever ships a pack registry (P connector packs).

--- 2.9 Plugins / hooks ---
OpenClaw: typed in-process hook system — decision hooks sequential by priority with block:true terminal, requireApproval with timeoutBehavior deny (fail-closed), observation hooks parallel and sanitized; TRUSTED-POLICY TIER runs before ordinary hooks; manifest (openclaw.plugin.json) validated WITHOUT executing code; per-hook timeouts; exec-env contribution hooks with LD_*/DYLD_*/NODE_OPTIONS filtering.
ELAI (spec): 4C plugins — hooks intercept IPC invisibly to agents, tiered static analysis, no LLM in hot path.
Verdict: PARITY on design (4C's invisible-to-agent interception is the stronger isolation property; their surface is richer). 
Steal: (a) manifest-validated-without-execution as a 4C admission invariant; (b) the trusted-policy tier ordering (host-owned safety/budget gates structurally run before third-party hooks — composes with the gate-manifest freeze: ELAI's frozen gates should occupy an ordered tier no plugin can preempt); (c) fail-closed approval timeout semantics.

--- 2.10 Multi-agent / subagents ---
OpenClaw: sessions_spawn isolated-or-fork subagents on a capped lane, DEPTH-SCOPED TOOL POLICY (subagents always lose gateway/cron/messaging tools; leaves lose spawn; only a depth-1 orchestrator regains it), child output framed as EVIDENCE that cannot override policy, push-based completion with idempotency keys, bounded REDACTED history recall (4000 chars/block, 80KB cap), announce includes token+cost stats.
ELAI (spec): 3C/3D agent system + BSP supersteps (deterministic barriers, journal-sequence application), 4A process isolation, I pristine-context injector + typed belief-state handoff, J declared context-access spec per dispatch.
Verdict: ELAI WINS on determinism (BSP + journal-ordered application vs best-effort announce lost on restart — their own docs admit orphan-recovery is bounded), PARITY on scoping intent (J's declared context-access spec ≈ their depth policy, typed rather than positional).
Steal: the child-output-as-fenced-evidence framing (subagent returns are DATA, structurally unable to carry instructions — composes with G taint: a subagent return is a taint source, period) and the depth-scoped tool-strip table as a concrete default for 3C.

--- 2.11 Coding capability ---
OpenClaw: not a coding harness. It shells out to coding CLIs (codex etc. as cliBackends/extensions), has no code retrieval, no AST anything, no repo model, no edit machinery of its own.
ELAI (spec): the entire Phase 1 knowledge engine (DKG/BM25/vector/PageRank/CatRAG/reranker), B impact-analysis API + multi-language DKG + symbol graph, 4B tools (3-tier edit matching, ShortCoder, failure taxonomy), C code-completeness + testing-trust tiers, B Focus Tunnel.
Verdict: DIFFERENT GAME — ELAI WINS by default; OpenClaw simply does not compete here. Implication: OpenClaw is not actually ELAI's competitor on the wedge; it IS a competitor for the O/P personal-machine grain (see 2.14).

--- 2.12 Reasoning / grounding ---
OpenClaw: none. The model's text is the product. No claim extraction, no verification methods, no grounding classes, no abstention machinery. (qa-lab's blind-judge character-eval is the closest thing and it is dev-side.)
ELAI (spec): Phase 2 GRC, D five-mode inference-time grounding + cross-family adversary + conformal refusal, L ClaimRisk + counterfactual do-test + hallucination-delta bench, J GRC-v3 receipt-gated steps.
Verdict: ELAI WINS by default — no analog exists. This plus 2.1/2.2 is the moat.

--- 2.13 Channels / devices / voice (product surface) ---
OpenClaw: ~30 channels with a worked-out DETERMINISTIC routing precedence (9-step bindings resolution, most-specific-wins; model never picks the channel), per-channel pairing codes (8-char, 1h expiry, ≤3 pending), group gating fail-closed to allowlist, accessGroups reusable sender sets, dmScope isolation options, fenced untrusted channel metadata. Nodes: two-layer trust (device pairing + capability approval), dangerous commands double-gated + deny-wins, per-platform allowlist tables, camera/location with stable error enums. Voice: 5 runtime shapes, gateway-owned credential boundary (browser NEVER receives a standard API key — ephemeral/constrained tokens only), global wake-word state broadcast, spoken steering classified status/steer/cancel/followup.
ELAI (spec): J voice front door (taint-tagged ASR ingress, journaled with audio-hash + model pin, readback from ROUTED intent not transcript echo, deterministic narration plane where the narrator structurally cannot lie), J front-door router (every message resolves to a lane or a clarify chip), P connector-pack format + ingress-adapter family (any external shape normalized into one taint-tagged journaled SsbEvent), O research daemon.
Verdict: OPENCLAW WINS on breadth and on worked-out per-channel policy detail (this is their product core; years of edge cases are encoded in those precedence rules). ELAI WINS on the deeper invariants where the corpus specifies them: their voice pipeline trusts the transcript, J's readback-from-routed-intent + no-model-in-the-truth-path narration is a strictly stronger design; their trigger surface grows per channel, P's ingress-adapter/one-SsbEvent design keeps the trigger enum closed. ELAI is not trying to ship 30 channels; P's bet is own-the-format.
Steal: (a) the 9-step binding precedence as the reference resolution order for P's ingress adapters and J's front-door router (they already paid for the edge cases); (b) pairing-code UX mechanics (expiring short codes, pending caps) for M seat/device enrollment; (c) the gateway-owned ephemeral-credential boundary for any future ELAI browser/realtime surface (never hand the client a standing key); (d) dmScope-style session-collapse options for P's small-team grain (their docs call out the cross-user leakage risk explicitly — P's per-seat isolation already answers it).

--- 2.14 Automation / scheduling / durable workflows ---
OpenClaw: cron in-gateway (SQLite, croner, per-class retry ladders, watchdogs, preflight probes, delivery chains, event triggers where a FAILED fired payload does NOT persist state — accidental fail-closed kinship), heartbeat (main-session periodic turn with HEARTBEAT_OK ack-strip contract + skipWhenBusy + activeHours), inferred commitments (background extraction, due-time CLAMPED ≥1 heartbeat after creation so a commitment can never echo back same-moment, delivered without tools, metadata marked untrusted), standing orders (prose authority in bootstrap files + cron for timing), Lobster (deterministic in-process pipeline DSL, approval steps halt with durable resume tokens, approve+resume never re-runs prior steps, runtime-enforced timeouts/output caps), Task Flow (SQLite flow_runs, REVISION-COUNTER optimistic concurrency — stale write = conflict never clobber, sticky persisted cancel intent surviving restart, managed vs mirrored modes).
ELAI (spec): O scheduled TaskSpec substrate ("n8n with receipts": typed cron/event schedules -> inbox, closed vocabulary, taint + replay), P workflow-from-goal (closed-enum TaskSpec graph, gaps rendered visibly disconnected, human-confirmed), Q standing goals (a completed unit's off-agent verification becomes a persistent re-checked invariant) + quorum wake-gate (cheap cross-family voters decide if the expensive lane wakes).
Verdict: PARITY on ambition, split on depth: OpenClaw WINS on durable-execution mechanics (revision counters, resume tokens, sticky cancel, retry taxonomies — all the boring correctness ELAI's O spec doesn't yet pin); ELAI WINS on trust semantics (O TaskSpecs are receipts-first and closed-vocabulary; their lobster pipelines are deterministic but their cron/heartbeat/commitment payloads are ordinary trusted prose; Q's standing goals re-verify with off-agent evidence, their standing orders are unenforced prose their own docs call "suggestions" without triggers).
Steal (high value, mostly composing into O): (a) resume-token approval semantics for O TaskSpec approval steps (summary steal #1); (b) revision-counter optimistic concurrency + sticky cancel intent for the TaskSpec run store (summary steal #7); (c) retry-ladder-per-failure-class + watchdog phases for O's scheduler; (d) the commitment due-time clamp (an inferred follow-up may never fire in the same tick it was inferred — cheap anti-echo invariant); (e) HEARTBEAT_OK-style ack-strip contract for Q's wake-gate quiet ticks (a quiet tick must be cheap AND leave a journal row, their ack-strip shows the UX); (f) on-exit schedule kind + top-of-hour auto-stagger, trivial and useful.

--- 2.15 Browser / computer use ---
Covered by openclaw-SUMMARY steal candidates 5 (snapshot-ref action contract, refs from fresh snapshots, CSS selectors banned, labeled screenshot overlays, graceful CDP-only degradation) and 6 (frameId + display-identity binding, fail-closed on reconnect). New from this pass: SSRF policy is a fail-closed DEFAULT OBJECT even when unconfigured; managed Chrome launches DIRECT ignoring HTTP_PROXY so a proxy can't weaken SSRF; act-policy layer with evaluate kill-switch; arming windows (summary steal #8).
ELAI (spec): E browser E2E, J perception gateway (counts from AXTree/DOM never a VLM — same conviction as their ref-over-vision choice), 4I media validator.
Verdict: OPENCLAW WINS today on mechanism completeness; the designs are CONVERGENT (both chose accessibility-tree-first over vision-first). Steal 5/6/8 stand as written in the summary; add: fail-closed-default-object config posture (a policy object that is fail-closed even when absent — generalize into J config-ingress admission) and proxy-immune direct launch.

--- 2.16 Cost / usage accounting ---
OpenClaw: provider-billed truth and local estimates NEVER merged; per-turn/per-session/per-provider-window/org-admin granularity; usage footer template with a typed dot-path contract; cache-hit accounting from usage payloads not headers.
ELAI (spec): J-28 token ledger (uniform usage_turn_event both lanes, session rollup, honest per-model pricing), H cost cockpit, K routing-economics bench, M fleet rollup.
Verdict: PARITY. J-28's uniform-event design is cleaner (one event shape both lanes); their never-merge-truth-with-estimate rule is a discipline J-28 should state explicitly.
Steal: the never-merge invariant (billed truth and computed estimate are separate typed fields end-to-end, any surface shows which it is showing) + plan-billed-hides-dollars nuance.

--- 2.17 Config / ops / lifecycle ---
OpenClaw: strict-schema fail-closed boot (unknown key = REFUSE to start), atomic writes, last-known-good with doctor-only restore, clobber guards (>50% shrink rejected, rejected write preserved as .rejected.<ts>), baseHash optimistic config patch + replacePaths anti-truncation, hot-reload with settle + invalid-edit rejection. Doctor: detect/repair/lint triple contract (lint = read-only structured findings + stable exit codes for CI, repair re-runs detect scoped), post-upgrade compat probes, .bak ring. Backup: VACUUM INTO WAL-safe SQLite snapshots, manifest + verify, traversal rejection. Update: detached managed-service handoff (never mutates the live tree), restart sentinel, continuation queue resuming one turn through restart.
ELAI (spec): J config-ingress admission gate, M signed release channel + auto-updater + support bundle, harness elai doctor-adjacent tools exist but the corpus is thin here.
Verdict: OPENCLAW WINS — this is their most underrated subsystem and ELAI's thinnest area. Ops hygiene is not the wedge, but a trust product that corrupts its own config or can't restore state undermines the trust story cheaply.
Steal (mostly net-new, natural M/Q additions): (a) last-known-good + rejected-write preservation + shrink/clobber guards into J's config-ingress gate (the gate currently admits; it should also make bad writes recoverable and journaled); (b) doctor detect/repair/lint with stable exit codes — ELAI already worships deterministic exit codes (K gate-tax, plan_gate_check), extend the same contract to an `elai doctor`; (c) VACUUM INTO backup + manifest verify for SSB/session stores (the journal IS the product; it deserves a verified backup story); (d) restart continuation queue (one in-flight turn survives a gateway restart) for M's auto-updater.

--- 2.18 Gateway protocol / control plane ---
OpenClaw: TypeBox single-source schema -> AJV validators + JSON Schema + Swift codegen (one source of truth, protocol:check in CI); PROTOCOL_VERSION with N-1 node compat window; challenge-signed device identity (v3 binds platform+deviceFamily, stable DEVICE_AUTH_* error codes); closed operator-scope set with reserved admin prefixes; scope-gated broadcasts FAIL-CLOSED on unknown event families; idempotency keys REQUIRED on side-effecting methods; pre-auth frame cap.
ELAI (spec): J ELAI-as-MCP-server control plane, Q command-registry (derive dashboard/CLI/canvas/control-plane from ONE registry so clients cannot drift — the same single-source conviction), 4K dashboard IPC, M seat entitlement.
Verdict: PARITY on conviction (their TypeBox pipeline IS Q's command-registry thesis, shipped), OPENCLAW WINS on auth mechanics today.
Steal: (a) challenge-signed device pairing + platform-bound signature payloads for M seat/device enrollment; (b) fail-closed unknown-event-family broadcast gating for the SSB viewer/control plane (an event family the reader doesn't recognize is withheld, not leaked); (c) idempotency-keys-required-on-mutating-RPC as a J control-plane invariant; (d) N-1 compat window policy for M's release channel.

====================================================================
3. CONSOLIDATED STEAL LIST (ranked by leverage; compose-target named)
====================================================================
The 8 openclaw-SUMMARY candidates stand confirmed (lobster resume tokens; metadata-only audit ledger; versioned trajectory envelope; confidence-report + self-test; snapshot-ref browser contract; frameId fail-closed binding; task-flow revision concurrency; arming windows). This pass adds and ranks; PS = proof-spine relevance.

TIER 1 — universal invariants, cheap to state, harden the whole spine
 S1  Approval drift-binding (anti-TOCTOU): every approval binds a canonical hashed run-plan + concrete operand fingerprints; execution re-validates; drift = deny. Compose: 3A gates, 4G autonomous gate, G blast-radius, exec paths. Net-new invariant. (PS-1/PS-6 adjacent: fewer silently-wrong actions.)
 S2  Failure-class cooldown taxonomy + session-stickiness + persist-before-retry + strict-explicit-selection for provider failover. Compose: J free-API rotation pool, I auth broker, 4M fallback chains, Q swap-restart. (PS-4: routing economics needs honest failover.)
 S3  Pre-compaction memory-flush as an ordered journaled step ("no compaction destroys context never offered to durable memory"). Compose: H context lifecycle + H user memory. Net-new mechanic.
 S4  Never-merge billed-truth-with-estimate as a typed end-to-end invariant. Compose: J-28 ledger, H cost cockpit, K routing bench. (PS-4 direct.)
 S5  Fail-closed-default-object config posture (policy objects fail closed even when absent, cf. their ssrfPolicy) + last-known-good/rejected-write/shrink-guard config lifecycle. Compose: J config-ingress admission gate.

TIER 2 — subsystem mechanics with a named ELAI consumer
 S6  Resume-token approval steps + revision-counter run store + sticky cancel intent + retry ladders for O's TaskSpec substrate (summary #1+#7 expanded). (PS-5 adjacent: durable reuse needs durable runs.)
 S7  Versioned on-demand skill/rules index (sha256 markers, compact prompt index, budget degrade). Compose: L familiarity surface + H MCP defer-loading — one lazy-load mechanism for skills/rules/MCP.
 S8  Metadata-only audit projection with bounded retention (summary #2). Compose: M fleet evidence rollup (zero-telemetry org answers are exactly a metadata-only projection of seat journals) + SSB privacy exports.
 S9  Posture labels on smoke evidence (user-path / direct-gateway / native-approval). Compose: wiring-enforcement R10 smokes + Q evidence-corruption corpus. Cheap, closes an over-claim hole.
 S10 Child-output-as-fenced-evidence + depth-scoped tool-strip defaults for subagents. Compose: 3C agent system + G taint (subagent return = taint source).
 S11 9-step deterministic channel-binding precedence + pairing-code UX + dmScope isolation options. Compose: P ingress adapters, J front-door router, M enrollment.
 S12 Challenge-signed device pairing + fail-closed unknown-event broadcast gating + mandatory idempotency keys. Compose: J control plane, M seats, 4K/H canvas transport.
 S13 elai doctor with detect/repair/lint + stable exit codes + post-upgrade probes; VACUUM INTO verified backups for SSB/session stores; restart continuation queue. Compose: M packaging/support + Q ops hardening. Mostly net-new area.
 S14 Compaction split-rule (never separate a tool-call from its result) + successor-transcript (never rewrite in place) as stated invariants. Compose: 3H + H context lifecycle (likely already implied; make explicit).

TIER 3 — reference implementations / prior art to cite when the ELAI unit builds
 S15 confidence-report zero-unknown + seeded negative-control canaries -> Q evidence-corruption corpus reference (summary #4).
 S16 Dreaming promotion gates (score/recall/diversity) -> H memory governance prior art.
 S17 Hidden-until-verified registry releases + namespace scope binding -> P connector-pack distribution, if/when a registry exists.
 S18 Per-provider cache-control mapping (Anthropic cache_control 5m/1h; OpenAI prompt_cache_key; Gemini cachedContents) -> H caching implementation table.
 S19 Commitment due-time clamp + heartbeat ack-strip + skipWhenBusy -> O scheduler + Q wake-gate details.
 S20 Trusted-policy hook tier ordered before third-party hooks + manifest-validated-without-execution -> 4C admission rules.

====================================================================
4. WHERE ELAI WINS (design-level, as-specified)
====================================================================
 W1  Replay-verify vs flight-recorder: seeded entropy + RRA + elai replay --verify + per-turn token hashing. OpenClaw can show you a trace; ELAI can PROVE one. No OpenClaw analog.
 W2  Off-agent evidence + fail-closed done-gates AT RUNTIME per task (F/G, wiring R1-R10, test-integrity hash binding). OpenClaw's equivalent rigor exists only in its dev-side CI (qa-lab).
 W3  Adversarial-input safety as a first-class threat model (taint propagation, write-gates, Dual-LLM quarantine, data-plane tool filter). OpenClaw's SECURITY.md explicitly triages prompt-injection-only as NOT a vulnerability while ingesting from ~30 untrusted channels. This is ELAI's cleanest competitive line, citable from their own policy.
 W4  Claim/reasoning verification (GRC, D grounding modes, L ClaimRisk + counterfactual do-test, abstention). OpenClaw trusts the model's prose end-to-end.
 W5  Coding depth: the Phase-1 knowledge engine, impact analysis, edit machinery, code-completeness, testing-trust. OpenClaw doesn't play; it shells out to third-party coding CLIs — which is precisely the seam I's agent-agnostic trust-layer wrapper monetizes (run THEIR agent's traffic through ELAI's gates).
 W6  Local-inference ownership (3J vLLM sidecar, DoLa/TurboQuant/steering, 4M runtime adapters). OpenClaw is an API consumer; it cannot make layer-level guarantees about the model.
 W7  Structural cache-stability (L fold-view) vs disciplined cache-stability (their prompt ordering) — same goal, ELAI's can't rot.
 W8  Verified memory (proof-gated case graph, validate-before-use facts, ROI attribution) vs trusted prose memory.
 W9  Self-improvement with un-gameable filters (D/H DGM stack). Their skill-creator proposal gate is operator-approval, not evidence.
 W10 Routing by measured value ($/verified-task, ledger-trained) vs routing by availability.
 W11 Determinism doctrine end-to-end: the model is a cog; their model picks tools/behavior freely inside a policy perimeter.

====================================================================
5. WHERE OPENCLAW WINS
====================================================================
Design-level (ELAI should answer these in-corpus):
 O1  Provider failover semantics (S2) — richer than anything in the corpus.
 O2  Approval drift-binding (S1) — a stated invariant the corpus lacks.
 O3  Config/ops lifecycle (S5/S13) — ELAI's thinnest area vs the trust story it sells.
 O4  Channel policy depth (S11) — years of encoded edge cases.
 O5  A normative written security triage boundary — the discipline, not the boundary itself.
Maturity-level (timing facts, not design marks — per spec-eval these do NOT feed the verdict):
 O6  It ships today at ~22k files with daily velocity, a real (NVIDIA/Tencent-staffed per SECURITY.md) security team, a live registry, native apps, and an installed base. Every mechanism above is field-tested; most ELAI analogs are at spec/partial. That is a distribution and hardening lead, not an architectural one.

====================================================================
6. STRATEGIC READ
====================================================================
- OpenClaw is not a competitor on the wedge; it is a TARGET MARKET EXHIBIT. It is exactly the "agent infrastructure without receipts" that K's dossier quantifies: enormous capability surface, single-operator trust, injection out of scope by policy, no replay-verify. I's agent-agnostic trust-layer wrapper could in principle wrap an OpenClaw gateway's tool traffic — their plugin hook surface (before_tool_call block/approve) is even the natural attachment seam. Ungrounded conjecture flag: whether wrapping OpenClaw specifically is worth a plan unit is unassessed; the general wrapper design already exists in I.
- Where OpenClaw and ELAI independently converged (accessibility-first browser control, zero-unknown evidence classification, single-source protocol schemas, on-demand context loading, successor-transcript compaction), treat the convergence as design validation and steal their edge cases.
- The steal list is deliberately mechanics-heavy: ELAI's corpus is strong on invariants and thin on the boring durable-execution/ops correctness OpenClaw has bled for. Tier 1-2 items are cheap to author as plan-unit amendments in the current open letter (P is open per plan-standard; J is CLOSED — extensions cross-link, never new J files).
- PROOF_SPINE note for any authored units: S2/S4 feed PS-4 (routing economics); S9/S15 feed PS-6/gate credibility; most others are `none (discipline)` class and per PSR-2 would need explicit user ack — batch them accordingly.

MATURITY LINE (separate from all verdicts above): design edges W1-W11 exist on paper and partially in tree; OpenClaw ships its whole surface today. Status claims about ELAI's own tree here are index-derived — confirm against live code (elai_status_query) before citing any as shipped.

====================================================================
7. OUTCOME (paper-to-plan, 2026-07-11)
====================================================================
The consolidated steal list S1..S20 (+ V1..V5 / N1..N2 / C1..C4 deep-read grades) was authored into ELAI plan files
as Track OPENCLAW-STEAL (P-plan, open letter): P-17 provider failover + honest routing economics (EXT J-28/I-50,
feeds PS-4), P-18 durable TaskSpec exec + approval drift-binding (EXT O-2/P-11, spine none), P-19 config/ops
lifecycle + elai doctor + verified backups (NEW area, spine none), P-20 context/memory lifecycle hardening (EXT H,
spine none), P-21 evidence/gate-credibility hardening (EXT Q-6, feeds PS-6), P-22 control-plane/channel hardening
(EXT 3C/J/M, spine none). The C1..C4 source-read corrections are baked into each unit's stub_ban + REDTEAM
(lobster resume unverified -> re-derive subprocess-checkpoint; trajectory content-carrying -> keep separate by type;
qa green default -> copy unknown=FAIL only; env full-copy -> allowlist). All plan_gate_check.py PASS (exit 0).
Doctrine in P-0_INDEX.md (FOURTH WAVE), entries in P-SUMMARY_INDEX.md, design in ELAI-SPEC-OVERVIEW.md P
orchestration-mechanics wave.

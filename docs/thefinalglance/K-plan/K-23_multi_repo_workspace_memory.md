# K-23: multi-repo / workspace memory — a thin cross-repo LAYER over deep per-root substrate (federation + delegation + cross-repo blast radius), GD-2 journal-as-record preserved

STATUS: PLANNED (MPW wave; NET-NEW capability folded into K per the user's "into K" ruling; a multi-slice TRACK shipped as ONE file, U1..U6 sequenced vertical slices each with its own done-gate; the FIRST cross-REPO axis in ELAI — single-root spine is otherwise untouched)

Objective: ELAI is architecturally SINGLE-ROOT — every knowledge/memory store is a pure function of one project_root (crates/core/src/paths.rs:173, BM25_PROJECT_ID literal "default"); switch repos and you switch to an isolated DKG/BM25/vector/memory_facts/SSB. The only cross-project mechanism is the Cosmos artifact tier (global ~/.elai/cosmos/, B-42/C-16). Real work spans repos (a protocol/types package + its consumers; a service + its infra). This session ships the cross-repo LAYER the strongest shipping comparator (Nx Polygraph, trypolygraph.com) bet on: a cheap cross-repo DEPENDENCY graph for PLANNING + LIVE SUBAGENT DELEGATION for code (live checkout, "not just embeddings") — which is ~80% a composition of machinery ELAI already has (orchestrate/auto-orchestrate + Tier-C isolation + J-80 worktree pool + each repo's own single-root DKG/retrieval + Cosmos). Hard boundary: it must NOT break the spine — each member repo's SSB journal stays its own authoritative record (GD-2); the cross-repo layer is a DERIVED, re-computable projection, NEVER a second primary store and NEVER a centralized cloud store (the cheapest thing to ship wrong, and the thing a naive "add a cross-repo vector DB" port WOULD get wrong). GD-1: every per-root index/retrieval, RRF k=60, the B-29 symbol cone, J-16, the J-18 sound channel, orchestrate+Tier-C+J-80, J-2/H-36 poll-notify, H-51 resume, J-21 validation, C-48 trust, J-59 per-seat, Cosmos, the SSB/FSM/Kahn machinery — REUSED as-is, re-implemented NEVER. Only the cross-repo layer itself is new.

Estimated effort: ~6-7 sessions (~28-34 h), one per U-slice. In-process derivation + a 2-repo fixture per slice; the delegation slice (U5) drives the real orchestrate entrypoint against two member roots.
Pre-requisites (each re-grepped at edit, W-7): single-root paths (paths.rs:173); Cosmos global store WIRED (B-42/C-16); RRF k=60 WIRED (phase-1C); B-29 symbol-impact API WIRED; J-16 blast-radius gate (blast_radius.rs, +BlastRadiusEvent — PLANNED, U4 composes it); J-18 LSP/SCIP sound channel (PLANNED — U2/U4 gate cross-repo edges on it); orchestrate + auto-orchestrate skills; Tier-C process isolation WIRED (phase-4A); J-80 worktree pool (BranchWorkspaceEvent — PLANNED, U5 delegates into it); J-2 ELAI-as-MCP (PLANNED — poll); H-36 AgentNotificationEmitted; H-51 handoff_digest.rs (HandoffLifecycleEvent); J-21 commit-aware validation (MemoryValidationEvent — PLANNED, U6); C-48 TrustTier; J-59 per-seat namespacing (PLANNED, U6); SSB single-writer WAL (ssb.rs, phase-3B); FSM (phase-3A) + Kahn cycle detection (phase-3G); 3E context budget allocator (context_budget.rs); canvas altitudes J-31/J-60/J-98 (U6).
Parallel-safe with: K-24 (disjoint files), and the K-1..K-22 doc/bench/doctrine sessions. NOT parallel with edits to orchestrate internals (U5) or the B-29/J-16 cone (U4).
Source: workflow/reasoning/multi-project-memory-analysis-2026-06-26.md (the full spec, §7.0..§7.12 — designed on the as-DONE substrate per spec-eval; Polygraph comparison web-grounded §4). Doctrine: K-0_INDEX.md. The reasoning file IS the grounding pass (paths.rs:173 + feature-map entries cited in-text; Polygraph via nx.dev/blog/announcing-polygraph + docs.trypolygraph.com + the maker Q&A); re-grep every anchor at edit.

## Context
Why this matters: the cross-REPO axis is a genuine whitespace — grepping the full feature map + every plan SUMMARY_INDEX (B..K) returns NO cross-repo knowledge graph, multi-repo retrieval router, or simultaneous multi-root indexing. Two things people confuse with it are a DIFFERENT axis: J-49/J-59 per-seat memory is a USER axis that ACTIVELY ISOLATES (the wrong-direction precedent), and canvas multi-project (H-11/J-31/J-60/J-98) is a VIEW axis over the CURRENT root's journal. Polygraph (Nx/nrwl) supplies the two things a single-repo agent structurally lacks — SPATIAL (across repos) and TEMPORAL (across sessions) — via (1) an org DEPENDENCY graph for planning/routing, (2) an org-shared SESSION graph, (3) LIVE CHECKOUT + subagent delegation for code (deliberately NOT cross-repo embeddings). ELAI already HAS the delegation half; what it lacks is exactly Polygraph's two graphs. Where ELAI must DIVERGE (and should): no centralized cloud store (GD-2), local journal-as-record, single-user-first — so Polygraph's "org-shared team memory" headline is OUT of scope until the per-seat work (J-59) is extended from isolation to opt-in sharing (a SEPARATE later decision, NOT a prerequisite). The honest two-mode design: CONNECTED repos (shared symbols/imports = real Sound edges) vs DISCONNECTED repos (federation only, no edges) — default to federation, promote to edges ONLY where a sound cross-repo reference is proven. A design that always builds the unified graph is over-engineered for the disconnected case the user explicitly asked about.

## Existing state (B/C/D/E/F/G/H/I/J/K coverage)
- SINGLE-ROOT SPINE [WIRED]: every store a function of one project_root; crates/core/src/paths.rs:173 (BM25_PROJECT_ID = "default", one project per root). The thing the workspace layer sits ABOVE, never breaks.
- COSMOS ARTIFACT TIER [WIRED]: global ~/.elai/cosmos/ cross-project store, briefing compiler, VerificationMethod::CosmosCheck, cosmos_worker, CosmosDrawer; B-42/C-16. The workspace manifest + workspace-session record are Cosmos-TIER artifacts (no new store class).
- RRF k=60 [WIRED]: phase-1C vector cascade merge — the federated-retrieval merge (U3) reuses it verbatim.
- B-29 SYMBOL-IMPACT API [WIRED] + J-16 BLAST-RADIUS GATE [PLANNED, crates/orchestrator/src/blast_radius.rs, +BlastRadiusEvent]: the cone U4 extends to traverse cross_repo_edges.
- J-18 RETRIEVAL HARDENING (LSP/SCIP sound channel) [PLANNED]: the Sound-edge gate for U2/U4 (heuristic edges never drive a gate).
- orchestrate / auto-orchestrate [skills]; Tier-C process isolation [WIRED, phase-4A]; J-80 WORKTREE POOL [PLANNED, BranchWorkspaceEvent]; J-2 ELAI-AS-MCP [PLANNED]; H-36 AgentNotificationEmitted: the delegation machinery U5 composes — `workspace orchestrate` is structurally an orchestrate run pointed at a different root.
- H-51 CROSS-SESSION HANDOFF [PARTIAL, handoff_digest.rs, HandoffLifecycleEvent]: U6 episodic resume reuses the per-member digest.
- J-21 MEMORY SEMANTIC-VALIDATION [PLANNED, MemoryValidationEvent] + C-48 TrustTier + J-59 PER-SEAT [PLANNED]: U6 extends provenance with source_repo + per-repo TrustTier (the poisoning firewall).
- SSB single-writer WAL [WIRED, ssb.rs, phase-3B]; FSM [WIRED, phase-3A]; Kahn cycle detection [WIRED, phase-3G]; 3E context budget [WIRED, context_budget.rs].
- canvas J-31 system / J-60 process / J-98 memory altitudes [1.10]: U6 adds a WORKSPACE altitude beside them.

## Reads (mandatory before coding)
1. workflow/reasoning/multi-project-memory-analysis-2026-06-26.md §7.0 (the as-DONE substrate) .. §7.12 (new-vs-composed honesty) — the full mechanism spec.
2. crates/core/src/paths.rs (the single-root resolver — the workspace store lives ABOVE it at ~/.elai/workspaces/<id>/, NEVER under a project_root).
3. The Cosmos store path + briefing compiler (B-42/C-16) — the manifest + workspace-session are Cosmos-tier artifacts.
4. crates/orchestrator/src/ssb.rs + tests/action_type_lock.rs (the 9-tuple golden lockstep; re-grep SCHEMA/TV at edit, W-9) — +WorkspaceEvent lands here.
5. crates/orchestrator/src/blast_radius.rs (J-16) + the B-29 traversal surface — U4 cross-repo cone.
6. the orchestrate skill + Tier-C spawn seam + J-80 worktree pool surface — U5 delegation.
7. CLAUDE.md hard rules that bite: GD-2 (journal-is-the-record, no external store), W-2/R9 (kill-switch not enable-flag; allowlist row), R5 (sound reference not name-grep), R10 (cross-seam integration smoke), blake3-only, no .unwrap on prod paths.

## Tasks (MPW wave — sequenced vertical slices U1..U6; each ships producer + in-plan consumer + smoke)
1. U1 — WORKSPACE OBJECT + manifest + `elai workspace create/add/map`. The object above roots: a workspace.toml (member project_roots + relationship hints produces-for|consumes-from|peer; the seats.toml/models.toml DATA convention, no per-member flag) at ~/.elai/workspaces/<id>/ (GLOBAL, beside Cosmos, NEVER under a project_root) + the workspace SSB (workspace.ssb.db, a single-writer append-only WAL, same class as a per-root SSB — GD-2 compliant). Capability behind ONE default-OFF ELAI_WORKSPACE_ENABLE (wire-dark-allowlist row: type=experiment, owner, expiry, promotion_plan_id=K-23-promotion, smoke exercising the ON path). Consumer: the three CLI verbs. Smoke: a 2-repo fixture (types-repo + api-repo). ~180 LOC. NO per-member flags; NO store under a member root.
2. U2 — CROSS_REPO_DEP_GRAPH derivation + `elai workspace map` renders Sound vs Heuristic edges. A DERIVED projection (cross_repo_edges table keyed by blake3(concat of each member DKG index-manifest hash) — when ANY member DKG changes the key changes and the graph re-derives, the same blake3-incremental CASCADE the single-root DKG uses; re-derivable => GD-2 safe). Edge {from_repo,to_repo,kind:PackageDep|ApiDep|TypeRef,from_symbol,to_symbol,evidence:"file:line",confidence:Sound(J-18 LSP/SCIP)|Heuristic(name+import match)}. PUBLIC SURFACE (exports+package names+routes) matched to EXTERNAL REFS (imports of non-local names+route-shaped client calls+non-local type refs); both sides resolve through J-18 => Sound, else => Heuristic. SOUND edges may drive gates; HEURISTIC edges surface with a warning badge, NEVER asserted as a call edge (R5/R6 — a wrong cross-repo edge yields a confidently-wrong blast radius, worse than no edge: the single most important correctness rule). Smoke: a fixture where types-repo<-api-repo yields EXACTLY ONE Sound edge, ZERO false edges. ~200 LOC.
3. U3 — FEDERATED QUERY-TIME RETRIEVAL: `elai workspace search <id> "<q>"` fans the query to EACH member's existing pipeline (BM25->vector->PageRank->CatRAG, all single-root, all built) in parallel; each returns a ranked list tagged with its source repo; RRF-merge (k=60, built) over the union; every hit carries per-repo provenance. NO new index, no cross-repo embedding store — N existing indexes queried + merged. The 3E RepoMap/CodeContext slots carry multi-repo provenance, each line labelled with its repo. Smoke: a fixture where the answer lives in repo B and is returned top-k while rooted at the workspace. ~150 LOC.
4. U4 — CROSS-REPO BLAST RADIUS + the J-16 gate extension. Extend the B-29 cone + J-16 auditor gate to traverse cross_repo_edges: cone within A (built) -> at each exported symbol of A with a Sound cross_repo_edge CONTINUE the cone into the consuming repo's DKG -> union into a workspace blast radius; the J-16 gate becomes cross-repo-aware (a change in A that breaks a consumer in B is caught at the workspace gate, not after merge). HEURISTIC edges contribute a "possible consumer, unverified" advisory list, NEVER the asserted radius. `elai workspace impact <id> <repo>:<symbol>`. Smoke: changing an exported type in A surfaces the REAL consumer in B with ZERO false consumers (precision-first; a false consumer is worse than a miss). The trust-critical capability; deterministic, LLM never adjudicates (TD-3), observe-only first (GD-4). ~180 LOC.
5. U5 — `workspace orchestrate` (the Approach-3 delegation, the Polygraph capability). A lightweight workspace FSM riding ABOVE the per-root FSM: WORKSPACE_PLANNING (derive/refresh edges, compute the touched-member set, Kahn-order over Sound edges — reuse phase-3G; a cycle = tripwire) -> WORKSPACE_DELEGATING (for each member in order, dispatch a Tier-C sub-agent into a J-80 worktree checkout running that member's OWN single-root FSM/pipeline; the sub-agent emits to the MEMBER's own SSB (authoritative); poll/notify via J-2 task_status + H-36; cap = orchestrate parallel_cap (3); independents parallel, dependents serialize by Kahn order) -> WORKSPACE_MERGING (collect each member PR/branch; run the cross-repo INTEGRATION smoke per wiring R10 — an INDEPENDENT agent drives the real entrypoints across repos on a HARD input: types changed -> api built against new types -> contract test green; missing integration evidence = FAIL) -> WORKSPACE_COMPLETE (one workspace-session summary row; per-member commits governed by each member's own commit-cadence). Orchestrate's 4 tripwires apply at workspace grain; worker git stays read-only (agent-dispatch-preamble). Smoke: 2 PRs with a cross-repo contract test green (R10). The controller roots at the WORKSPACE (synthetic root), never edits member code itself (main-llm-role). ~220 LOC + orchestrate seam wiring.
6. U6 — EPISODIC CROSS-REPO MEMORY + provenance/poisoning + canvas WORKSPACE altitude. Resume: a session re-opening a workspace reads workspace.ssb.db (what was done, in what order, which PRs) + each member's H-51 handoff digest -> reconstructs the cross-repo narrative WITHOUT copying member state (the local journal-backed equivalent of Polygraph's session graph). Provenance: every cross-repo fact/edge carries source_repo; J-21 commit-aware validation extended so its provenance tuple includes source_repo + that repo's C-48 TrustTier. Poisoning firewall: a fact sourced in member A injected into a decision about member B must (a) pass J-21 validation against B's current commit+AST AND (b) carry the cross-repo provenance badge — a low-trust member cannot silently poison a high-trust one (the H-31 auditor-objectivity firewall pattern, extended per-REPO trust). Per-seat: a workspace belongs to a J-59 seat, namespaced by it; cross-seat workspace sharing is the SAME opt-in-later decision as cross-seat memory — OUT of scope here. canvas WORKSPACE altitude beside J-31/J-60/J-98: nodes=member repos, edges=cross_repo_edges (colored Sound vs Heuristic), an in-flight delegation lights up the member node, the workspace-session+PRs are the temporal layer; re-derived byte-identically from workspace.ssb.db (GD-2, no parallel state). ~200 LOC + frontend_pair.
7. workflow/reasoning/K-23-decisions.md (what, the connected-vs-disconnected two-mode decision, the GD-2-projection-not-store decision, the Sound-vs-Heuristic gate rule, expected per-slice test outcomes, unknowns).

## WIRING_CONTRACT
produces_symbols:
  - the WORKSPACE object + workspace.toml manifest + workspace.ssb.db (a per-workspace single-writer SSB; ~/.elai/workspaces/<id>/, GLOBAL — NEVER under a project_root) — U1
  - the cross_repo_edges DERIVED projection (blake3(member-DKG-manifest-hashes)-keyed, re-computable; derived/ under the workspace dir) + the Sound/Heuristic split — U2
  - `elai workspace create|add|map|search|impact|orchestrate` CLI verbs (each paired with its FSM gate) — U1/U2/U3/U4/U5
  - the federated multi-root retrieval router (RRF k=60 merge over N member pipelines, per-repo provenance) — U3
  - the cross-repo cone traversal + the J-16 gate cross-repo extension — U4
  - the workspace FSM ring (WORKSPACE_PLANNING/DELEGATING/MERGING/COMPLETE) + `workspace orchestrate` delegation mode — U5
  - the episodic resume + source_repo provenance/poisoning extension + the canvas WORKSPACE altitude — U6
  - +WorkspaceEvent SSB (Pattern-2; WorkspaceEventKind = Created|MemberAdded|MemberRemoved|DependencyGraphDerived|SessionStarted|MemberDelegated|MemberCompleted|CrossRepoImpactComputed|FederatedQueryRan|SessionMerged); emitted to the WORKSPACE ssb.db, payload carries only ids/aliases/seq-ranges/hashes/verdicts — NEVER code content or a seat secret (journal-content-minimization)
consumes_symbols:
  - crates/core/src/paths.rs (the single-root resolver — REUSED unchanged; the workspace store lives ABOVE it; GD-1)
  - Cosmos B-42/C-16 store + briefing compiler (the manifest + workspace-session are Cosmos-tier artifacts — no new store class)
  - the per-root BM25/vector/PageRank/CatRAG pipeline + RRF k=60 (phase-1C) — federated query (U3), re-implemented NEVER
  - B-29 symbol-impact cone + J-16 blast_radius.rs (+BlastRadiusEvent) + J-18 LSP/SCIP sound channel (U2/U4) — cross-repo cone gates on Sound edges only
  - the orchestrate/auto-orchestrate skills + Tier-C spawn seam (phase-4A) + J-80 worktree pool (BranchWorkspaceEvent) + J-2 task_status + H-36 AgentNotificationEmitted (U5) — delegation; re-implemented NEVER
  - phase-3G Kahn cycle detection (member ordering); 3E context_budget.rs (multi-repo provenance in RepoMap/CodeContext slots)
  - H-51 handoff_digest.rs (per-member resume); J-21 MemoryValidationEvent (commit-aware validation extended with source_repo); C-48 TrustTier; J-59 per-seat namespacing
  - canvas J-31/J-60/J-98 altitude pattern (the WORKSPACE altitude rides the same projection seam)
reachable_from:
  - `elai workspace create/add/map/search/impact` -> the workspace store + the federated router + the cross-repo cone [PRODUCTION CLI]
  - `elai workspace orchestrate <id> "<task>"` -> the workspace FSM -> Tier-C sub-agents in J-80 worktrees -> each member's own FSM -> WORKSPACE_MERGING integration smoke [PRODUCTION CLI + orchestrate]
  - the canvas WORKSPACE altitude re-derived from workspace.ssb.db [PRODUCTION dashboard]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO second primary store — the cross_repo_edges graph + the canvas altitude are DERIVED projections re-computable from member DKGs / workspace.ssb.db (GD-2)
  - NO centralized cloud store (the Polygraph divergence; the thing a naive port gets wrong)
  - NO store written under any member project_root — the workspace store is GLOBAL at ~/.elai/workspaces/<id>/
  - NO Heuristic edge asserted as a real call edge / driving a gate (R5/R6 — Sound edges gate, Heuristic edges advise only)
  - NO fabricated edge between disconnected members (empty cross_repo_edges is the correct answer; federation-only)
  - NO cross-seat workspace sharing v1 (per-seat isolation holds; opt-in sharing is a separate later decision)
  - NO re-implementation of any per-root index/retrieval/cone/orchestrate/Tier-C/J-80/Cosmos (GD-1)
  - NO .unwrap()/.expect() on prod paths; blake3 only (the edge-graph key)
frontend_pair:
  the canvas WORKSPACE altitude (member-repo nodes + Sound/Heuristic cross_repo_edges + in-flight delegation highlight) as a Svelte 5 runes component + #[tauri::command] + generate_handler! + ts-rs binding; cross_lang_diff.py PASS (W-4). Decide the exact drawer/window at edit (rides the J-31/J-60/J-98 altitude pattern).
observable_effect:
  trigger: "elai workspace create acme --members types,api ; elai workspace map acme ; elai workspace impact acme types:SomeExportedType ; elai workspace orchestrate acme \"bump the protocol field\""
  expect_ssb: "WorkspaceEvent rows (Created, MemberAdded x2, DependencyGraphDerived, CrossRepoImpactComputed, SessionStarted, MemberDelegated x2, MemberCompleted x2, SessionMerged) in workspace.ssb.db, recomputable from the journal (GD-2); each member's own work emits to the MEMBER's SSB, never duplicated"
  expect_artifact: "`workspace map` prints exactly one Sound edge (types<-api) and zero false edges on the fixture; `workspace impact` surfaces the real consumer in B with ZERO false consumers; `workspace orchestrate` lands 2 PRs with a cross-repo contract test green (R10); the canvas altitude re-derives byte-identically. NEGATIVE: a fabricated edge between two DISCONNECTED members FAILS; a Heuristic edge driving the impact gate FAILS (Sound-only); a workspace store written under a member root FAILS"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-23_multi_repo_workspace_memory.md --run-trigger
rg -n "WorkspaceEvent|WorkspaceEventKind" .elai_cc/crates/orchestrator/src/ssb.rs        # the SSB type exists, golden-locked
rg -n "workspace" .elai_cc/crates/cli/src/args.rs                                         # the CLI verbs on the real entrypoint
rg -n "Sound|Heuristic" .elai_cc/crates/orchestrator/src/                                 # the edge-confidence split exists
rg -c "elai/workspaces" .elai_cc/crates/                                                  # the GLOBAL store path (never a project_root) is present
rg -c "cloud|http.*workspace.*store|remote.*graph.*db" .elai_cc/crates/orchestrator/src/  # MUST be 0 for a workspace primary store (GD-2; cloud store banned)
python .claude/tools/cross_lang_diff.py --changed .elai_cc/crates/orchestrator .elai_cc/crates/dashboard-v2/src/lib   # frontend_pair drift for the canvas altitude
test -f workflow/reasoning/K-23-decisions.md
```

## ACCEPTANCE
- [ ] GD-2: the cross_repo_edges graph + canvas altitude are DERIVED, re-computable projections; the ONLY new authoritative store is workspace.ssb.db (a single-writer append-only WAL, same class as a per-root SSB). No cloud store, no store under a member root.
- [ ] R5/R6: only Sound (J-18 LSP/SCIP) edges drive the impact gate; Heuristic edges advise with a badge and never assert a call edge.
- [ ] Disconnected mode: two members with no Sound/Heuristic edge yield empty cross_repo_edges; federation-only; no fabricated connection.
- [ ] R10: WORKSPACE_MERGING runs an INDEPENDENT cross-repo integration smoke on a HARD input (strict consumer build), missing evidence = FAIL.
- [ ] GD-1: every per-root index/cone/orchestrate/Tier-C/J-80/Cosmos REUSED, re-implemented never.
- [ ] W-2/R9: ELAI_WORKSPACE_ENABLE has a wire-dark-allowlist row (type=experiment, owner, expiry, promotion_plan_id=K-23-promotion, ON-path smoke).
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator, -p cli, -p core, -p knowledge; svelte-check for the canvas altitude.
- [ ] SSB net +1 (WorkspaceEvent), sequential bump from LIVE 268/577 (re-grep both at edit, W-9).
- [ ] No new dependency (or the ONE new dep, anti-slopsquat-verified, user-confirmed).

## WIRING DONE-GATE (clause by clause)
- W-1: done = `elai workspace orchestrate` runs end-to-end on a 2-repo fixture + WorkspaceEvent rows land + the cross-repo integration smoke is green; cargo-green is NOT done.
- W-2: ELAI_WORKSPACE_ENABLE is a default-OFF capability flag -> wire-dark-allowlist row with promotion_plan_id=K-23-promotion (NOT a terminal state).
- W-3: no summary-only port (full vertical slices). W-4: canvas WORKSPACE altitude frontend_pair + cross_lang_diff PASS.
- W-5: live trigger = the 2-repo fixture; deterministic fixture harness ships now, a >2-member live workspace is RunPod-irrelevant (local), W-10 N/A.
- W-7: paths.rs, ssb.rs/action_type_lock.rs, blast_radius.rs, the orchestrate seam, J-80 surface ALL re-grepped at edit.
- GD-1/GD-2: as ACCEPTANCE. GD-4: U4/U6 observe-only first. R5/R6: Sound-only gating. R7: no test-only caller satisfies reachability. R10: the WORKSPACE_MERGING integration smoke.

## Tests to add
- crates/contracts/tests/k23_workspace_memory_contract.rs (~200 LOC): (a) a 2-repo fixture: types<-api yields exactly one Sound edge, zero false edges; (b) two DISCONNECTED members yield empty cross_repo_edges (federation-only); (c) federated search returns the repo-B answer top-k while rooted at the workspace, every hit repo-tagged; (d) cross-repo impact surfaces the real consumer in B with ZERO false consumers; a Heuristic edge does NOT enter the asserted radius; (e) `workspace orchestrate` Kahn-orders members, delegates Tier-C into J-80 worktrees, and WORKSPACE_MERGING fails on a missing integration smoke (R10); (f) the workspace store is at ~/.elai/workspaces/<id>/ and NOT under any member root; the cross_repo_edges graph re-derives byte-identically from member DKG manifests (GD-2); (g) source_repo provenance: a low-trust member's fact cannot poison a high-trust member's decision without passing J-21 validation against B's commit+AST.
- NOTE: crates/orchestrator/tests/ integration tests do NOT link (ort/ndarray rlib) — put the orchestrate-seam smoke as an in-crate #[cfg(test)] lib test; the contract goes in crates/contracts/tests/.

## Files touched
- CREATE: crates/orchestrator/src/workspace/{mod.rs,manifest.rs,dep_graph.rs,federated_search.rs,cross_repo_cone.rs,workspace_orchestrate.rs,episodic.rs} (~1100 LOC total across U1..U6)
- MODIFY: crates/cli/src/args.rs + a workspace_cmd.rs module (~120 LOC: the create/add/map/search/impact/orchestrate verbs + FSM gates)
- MODIFY: crates/orchestrator/src/ssb.rs + tests/action_type_lock.rs + the golden (+WorkspaceEvent, sequential 9-tuple lockstep)
- MODIFY: crates/orchestrator/src/blast_radius.rs (~40 LOC: the cross-repo cone continuation on Sound edges; J-16 gate cross-repo awareness)
- MODIFY: crates/dashboard-v2/src/lib (+ src-tauri commands_*.rs + generate_handler! + ts-rs): the canvas WORKSPACE altitude (~120 LOC)
- MODIFY: .elai_cc/wire-dark-allowlist.toml (+1 row: ELAI_WORKSPACE_ENABLE)
- CREATE: crates/contracts/tests/k23_workspace_memory_contract.rs (~200 LOC)
- CREATE: workflow/reasoning/K-23-decisions.md (~40 LOC)

## Citations / Grounding
- workflow/reasoning/multi-project-memory-analysis-2026-06-26.md §1..§7.12 — the full mechanism, the Polygraph comparison, the three approaches + the adversarial attack, the new-vs-composed honesty.
- Polygraph (Nx/nrwl): nx.dev/blog/announcing-polygraph, docs.trypolygraph.com, the Product Hunt maker Q&A, github.com/nrwl/polygraph-skills — SPATIAL+TEMPORAL axes, dep-graph-for-planning + live-checkout-delegation (NOT cross-repo embeddings), org-shared cloud session graph. [EXTERNAL — the comparator's design, web-verified; ELAI's design diverges on GD-2 local journal-as-record + single-user-first]
- ELAI ground truth: paths.rs:173 (single-root), Cosmos B-42/C-16 (the only cross-project store today), RRF k=60 (phase-1C), B-29 + J-16 (the cone), J-18 (sound edges), orchestrate+Tier-C+J-80+J-2+H-36 (the delegation half ELAI already has), H-51/J-21/C-48/J-59 (episodic/provenance/trust), SSB/FSM/Kahn (the spine).
- CLAUDE.md: GD-1 (compose), GD-2 (journal-is-the-record, no external/cloud store), R5/R6 (sound reference), R10 (integration smoke), W-2/R9 (kill-switch + allowlist), blake3-only, no .unwrap on prod paths.

## Open questions / risks
- Cross-repo edge soundness depends on J-18 (LSP/SCIP) being trustworthy; until J-18 lands, U2/U4 edges are Heuristic-only (advisory) and the impact gate is observe-only. Author with the Sound/Heuristic split from day one so promotion is a flag flip, not a rewrite. Decide at execution whether U4 ships before J-18 (Heuristic-advisory mode) or waits.
- Federated fan-out cost scales with member count; v1 caps the workspace at a small member set (the common 2-4 repo case) and notes large-org federation as a later rider.
- The workspace FSM riding above the per-root FSM is the structurally riskiest piece (two FSMs, one controller); U5 must never let the workspace controller touch a member's FSM (the cross-session FSM bug class) — it polls via J-2/H-36 only, exactly as orchestrate waits on dependency sessions today.
- Disconnected-repo behavior is handled by construction (empty edges), not a special case — verify the design never fabricates a connection to make the feature "work."

## Next
This closes the cross-REPO whitespace with a thin layer over deep reused substrate — the only honest answer for DISCONNECTED repos and the only one that keeps the GD-2 journal-as-record spine intact. It pairs with K-24 (the two are the first net-new-capability K files) and with the per-seat work (J-49/J-59) IF/when the user decides to extend isolation to opt-in cross-seat workspace sharing (Polygraph's org-shared-team-memory headline) — a separate, later decision, never a prerequisite. After it, "work across all my connected and disconnected projects at once" is a real capability (federated search + cross-repo blast radius + live delegation), not a manual copy-paste exercise.

End of K-23.

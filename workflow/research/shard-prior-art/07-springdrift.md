# Springdrift — Prior-Art Dossier (SHARD comparison: composed persistent-agent runtime)

Pillar under comparison: composed persistent-agent runtime (the SHARD-equivalent niche).
Research date: 2026-06-01. Mode: oracle (grounded-by-default, primary-source fetches).
Grounding floor: 8 distinct primary-source fetches (paper HTML, arXiv abstract, GitHub repo API, repo tree, README, two source files, evals listing, official site). Floor (6) met.

Source-fidelity note: every architecture / API / number below traces to a fetched source listed in section 8. Springdrift is written in Gleam on Erlang/OTP (BEAM), NOT Python — verified from the repo language field, file tree, and two fetched .gleam source files. Where a number could only be confirmed from the paper/site and not cross-checked in source, it is marked vendor-reported.

---

## 1. Identity & provenance

- Full name: Springdrift — "An Auditable Persistent Runtime for LLM Agents with Case-Based Memory, Normative Safety, and Ambient Self-Perception".
- Author / org: Seamus Brady (Corvideon, seamus@corvideon.ie). Single-author paper and single-author repo. [S1][S3]
- Paper: arXiv:2604.04660, cs.AI, submitted 2026-04-06 (v1). The arXiv ID RESOLVES — abstract, HTML, PDF, and TeX source all present. [S1][S2]
- Repo: github.com/seamus-brady/springdrift — created 2026-02-15, last push 2026-04-27. EXISTS and is auditable. [S3]
- Version: semver from 0.8.0 (April 2026); 0.x, breaking changes allowed in minor bumps. [S5]
- License: AGPL-3.0 (GNU Affero GPL v3.0). Network-service redistribution triggers source disclosure; private/research use does not. Commercial licensing offered separately by the author. [S3][S5]
- Docs / site: springdrift.ai (official). Eval dataset: huggingface.co/datasets/sbrady/springdrift-paper-evals. [S8]
- Language / runtime: Gleam 1.9+ on Erlang/OTP 27+; Git is a hard runtime dependency (append-only memory + recovery). [S5]

VERSION/SCALE MISMATCH (flag): the official site claims "62,000 lines of Gleam | 1,490 tests | 136 source files" [S8]; the README claims "~150 source files" with "tens of thousands of lines" and "a comprehensive green test suite" [S5]. The repo-tree fetch estimated "150+ Gleam files" [S4]. These are close but not identical (136 vs ~150) — treat the file/line counts as vendor-reported, approximate.

LICENSE NOTE: AGPL-3.0 is materially more restrictive than a typical permissive prior-art baseline. Any ELAI re-use of Springdrift code (not concepts) under a network service would trigger copyleft. Concepts/mechanisms are not copyrightable; the AGPL constrains code reuse only.

---

## 2. Problem it solves

Target failure mode: session-bounded LLM agents lose state and accountability across runs. Specifically Springdrift targets four properties hard to get in session-scoped systems [S1][S2]:
- cross-session task continuity (state survives restarts),
- cross-channel context maintenance (email + web share one memory),
- end-to-end forensic reconstruction of any decision (audit any past action),
- self-diagnostic behavior (the agent observing its own failure modes without instruction).

The framing concept is the "Artificial Retainer": "a non-human system with persistent memory, defined authority, domain-specific autonomy, and forensic accountability, engaged in an ongoing relationship with a specific principal" — positioned between reactive assistants and unbounded autonomous agents, by analogy to professional retainer relationships and trained working animals (guide-dog analogy). Seven defining properties per Appendix H.6: persistent identity/memory, defined scope of authority, domain-specific refusal, proactive engagement, forensic accountability, relationship continuity, honest assessment. [S2][S8]

---

## 3. How it works — architecture + key mechanisms

Springdrift is an OTP supervision tree: each subsystem is a supervised BEAM process (Permanent / Transient / Temporary restart strategies). During the 23-day deployment, "714 LLM timeouts and multiple sub-agent crashes were handled by supervision without manual intervention." [S2]

### 3.1 Process topology (named components) [S2]
- Cognitive Loop — orchestrates specialist sub-agents, drives safety evaluation each cycle.
- Librarian — replays append-only JSONL stores into in-memory ETS tables (default `librarian_max_days = 30`).
- Curator — assembles the virtual context window each cycle.
- Archivist — post-cycle reflection, generates structured memory.
- Backup Actor — git commits + remote push.
- Scheduler — agent-created jobs / recurring tasks.
- Inbox Poller — email ingestion, routes to cognitive loop.
- Observer Agent (transient) — per-cycle forensics + CBR curation.
- Specialist Agents — Planner, Project Manager, Researcher, Coder, Writer, Comms.

Note: paper describes Librarian/Curator/Archivist as OTP processes [S2]; the repo-tree fetch noted curator/librarian/archivist also surface as skills in `.springdrift_example/skills/` [S4]. The CBR + memory modules under `src/memory/`, `src/cbr/`, `src/backup/` carry the archival functions in code.

### 3.2 Case-Based Reasoning memory (the differentiator vs plain RAG)

Ten append-only JSONL stores, each a typed record [S2]:
Narrative→NarrativeEntry, Threads→Thread, Facts→MemoryFact (scoped, decayed), CBR cases→CbrCase, Artifacts→ArtifactRecord, Tasks→PlannerTask, Endeavours→Endeavour, Comms→CommsMessage, Affect→AffectSnapshot, DAG nodes→CycleNode.

CbrCase type (verbatim fields from `src/cbr/log.gleam`) [S6]:
`case_id`, `timestamp`, `schema_version`, `problem: CbrProblem`, `solution: CbrSolution`, `outcome: CbrOutcome`, `source_narrative_id`, `profile: Option(String)`, `redacted: Bool`, `category: Option(CbrCategory)`, `usage_stats: Option(CbrUsageStats)`, `strategy_id: Option(String)`. Persistence functions: `append/2`, `load_date/2`, `load_all/1`.

Retrieval = six-signal weighted fusion. VERIFIED IN SOURCE (`src/cbr/bridge.gleam`, `default_weights()` returns a `RetrievalWeights` record) [S9]:
```gleam
RetrievalWeights(
  field_weight: 0.1,
  index_weight: 0.25,
  recency_weight: 0.05,
  domain_weight: 0.1,
  embedding_weight: 0.4,
  utility_weight: 0.1,
)
```
Weights sum to 1.0 and renormalize when embeddings are unavailable. These exactly match the paper's stated weights (inverted index 0.25, semantic embedding 0.40, weighted field 0.10, recency 0.05, domain 0.10, utility 0.10) — paper and code agree. [S2][S9]

Per-signal scoring functions (verbatim, `bridge.gleam`): `retrieve_cases()`, `retrieve_cases_with_decay()`, `weighted_field_score()`, `index_score()`, `recency_score()`, `domain_score()`, `embedding_score()`, `utility_score_signal()`. [S9]

Retrieval cap: paper states K=4 (P@4 metric) [S2]; in source the cap is `query.max_results` on the `CbrQuery` type — passed per-query, not a hard constant [S9]. (So K=4 is the eval setting, not a code-fixed ceiling.)

Utility score = Laplace-smoothed outcome tracking. Paper: `utility = (successes+1)/(retrievals+2)` [S2]. In code, `bridge.gleam` delegates to `types.utility_score(usage_stats)` with neutral fallback 0.5 for cases lacking stats [S9]. This is the learning signal — cases that succeed when retrieved get up-weighted; this is what makes it "CBR" rather than static vector search.

### 3.3 Normative safety calculus (deterministic gating)

Three D′ gates wrap every cycle [S2]: D′ Input Gate (incoming prompts), D′ Tool Gate (each tool call pre-dispatch), D′ Output Gate (finished responses). Implemented in `src/agent/cognitive/safety.gleam` (~55KB) [S4].

Normative operators, strict ranking R ≫ O ≫ I [S2]:
- Required (R): violation impermissible / categorical.
- Ought (O): violation undesirable, not categorical.
- Indifferent (I): no normative weight.

Ordinal levels (paper lists numeric weights; highest first) [S2]: Ethical/Moral (6000), Legal (5000), Prudential (4500), Social-Political (4000), Safety-Physical (3500), Environmental (3000), Community (2500), Professional Ethics (2000), Economic (1500), Etiquette (1000), Operational (100). (Paper title-claims "fourteen ordinal levels"; the HTML extract enumerated 11 named tiers — treat the exact count as paper-reported, the enumerated tiers as verified.)

Modal operators: Possible / Impossible (logical/theoretical/practical possibility distinguished). [S2]

Six Stoic axioms (Becker-inspired, applied in order) [S2]: 6.6 Futility (Impossible ⇒ inert), 6.7 Indifference (I ⇒ no weight), 6.2 Absolute Prohibition (Ethical/Moral + Required = categorical, non-overridable), 6.3 Moral Priority (higher level wins across levels), 6.4 Moral Rank (same level ⇒ stronger operator wins, R≫O≫I), 6.5 Normative Openness (no rule ⇒ compatible).

Conflict resolution: each user proposition resolves against each system proposition → (winner, severity) pairs. Severities: NoConflict, Coordinate, Superordinate, Absolute. Eight floor rules map severities → verdict {PROHIBITED, CONSTRAINED, FLOURISHING}: any Absolute → PROHIBITED; Superordinate at Legal(5000)+ → PROHIBITED; D′ ≥ reject → PROHIBITED; catastrophic + Superordinate → CONSTRAINED; ≥2 Coordinate → CONSTRAINED; D′ ≥ modify → CONSTRAINED; Superordinate in Professional-Ethics..Safety-Physical band → CONSTRAINED; default → FLOURISHING. [S2]

D′ scoring formula [S2]: `D′ = Σ(importance_i × magnitude_i) / (max_importance × max_magnitude × n)`. Thresholds: modify = 0.35, reject = 0.55; Comms agent tightens to 0.30 / 0.50.

Key property: the calculus is DETERMINISTIC and produces an auditable axiom trail. Verified by exhaustive eval (section 5).

### 3.4 Sensorium (ambient self-perception) [S2]
An XML block injected into the system prompt each cycle, sections: Clock (time/uptime/cycle ID), Situation (input source, queue depth, active thread), Schedule (pending/overdue jobs), Vitals (rolling perf: cycles_today, agents_active, success_rate, cost_trend, cbr_hit_rate, novelty, recent_failures), Delegations (per sub-agent turn progress / token usage / elapsed), Tasks. Performance signals are rolling averages computed from the persistent narrative log (e.g. per-input novelty via Jaccard similarity). The Curator budgets it into priority slots 1 (identity) … 10 (background); `preamble_budget_chars` default 8000 (~2000 tokens); "Identity and sensorium are never shed."

### 3.5 Auditable execution substrate [S2]
- Append-only: no record modified/deleted in normal operation; daily-rotated JSONL in `.springdrift/memory/cycle-log/`; current state derived by chronological replay.
- Git-backed recovery: `.springdrift/` is a git repo; periodic commits (default 5 min) + optional remote push; `git checkout <commit>` restores entire agent state to any point; auditing via `git log` / `git diff`. Implemented under `src/backup/`. [S2][S4]
- Cycle DAG: parent-child tree (root = user/scheduler cycle; descendants = agent sub-cycles + tool calls); each CycleNode carries token counts, model used, elapsed time, typed output.
- SD Audit Tool: standalone offline Python analyzer (reads logs without a running system). [S2]

---

## 4. Core API / usage (verbatim-in-spirit from README) [S5]

Setup + run (exact commands from README):
```bash
git clone https://github.com/seamus-brady/springdrift
cd springdrift
gleam build
cp -r .springdrift_example .springdrift     # edit .springdrift/config.toml
export ANTHROPIC_API_KEY=sk-ant-...
export SPRINGDRIFT_WEB_TOKEN=$(openssl rand -hex 24)
gleam run                      # web GUI, port 12001
gleam run -- --gui tui         # terminal TUI
```

Config (`.springdrift/config.toml`, exact keys) [S5]:
```toml
provider        = "anthropic"
task_model      = "claude-haiku-4-5-20251001"
reasoning_model = "claude-opus-4-6"
max_tokens      = 2048
max_turns       = 5
[agent]
name = "Springdrift"
[narrative]
threading = true
[meta_learning]
enabled = true
consolidation_interval_hours = 168
goal_review_interval_hours = 24
```
Three-layer config merge: CLI flags > project `.springdrift/config.toml` > user `~/.config/springdrift/config.toml`. Providers: anthropic (default), openai, openrouter, mistral, google vertex, local ollama. Optional: Brave/Kagi search, Jina Reader, Podman sandbox, AgentMail email.

Public CBR API surface (verbatim, `src/cbr/`) [S6][S9]: `cbr/log.gleam`: `append/2`, `load_date/2`, `load_all/1`. `cbr/bridge.gleam`: `retrieve_cases()`, `retrieve_cases_with_decay()`, `default_weights()`.

---

## 5. Measured numbers / benchmarks

All from the paper's eval appendices + `evals/` experiments. Single author, synthetic datasets, one real deployment. [S2][S7]

- CBR retrieval (Experiment 3, hybrid CBR vs RAG): P@4 = 0.956, 95% CI [0.936, 0.974], vs dense-cosine RAG baseline 0.920. Dataset 800 synthetic cases, 200 queries, 3 difficulty tiers, bootstrap CIs. NOTE: confidence intervals overlap (per evals summary) — improvement is positive but not strongly separated. [S2][S7]
- Experiment 2 (deterministic-signals-only CBR vs RAG): same 800/200 dataset, deterministic matching only. [S7]
- Experiment 5 (learning curve): +10.7% P@4 scaling from 25→800 cases; confidence-decay half-life verified across 5 property tests. [S7]
- Normative calculus completeness: 84×84 = 7,056 proposition pairs, 100% coverage, zero determinism violations, zero monotonicity violations. (Experiment 1 / completeness check.) [S2][S7]
- D′ adversarial robustness (Experiment 4): Precision 1.000, Recall 0.795, F1 0.886, on a 64-sample corpus (23 known injection attacks, 11 evasion variants, plus benign). [S7]
- Operational deployment ("Curragh", March 2026): 23 calendar days / 19 operating days, single instance, single operator. 494 narrative entries + 24,035 cycle-log entries released. 714 LLM timeouts auto-recovered by supervision. Agent self-diagnosed infra bugs and an architectural vulnerability without instruction. [S2][S8]

SAMPLE-SIZE / VENDOR caveats: all datasets are SYNTHETIC except the single 23-day deployment, which is n=1, self-run, self-reported (no third-party reproduction). The 64-sample adversarial corpus is small. No external/independent benchmark exists. Treat every figure as vendor-reported and unreproduced.

---

## 6. Maturity signals [S3][S5][S8]

- Stars: 45. Forks: 2. Open issues: 1. (GitHub API, 2026-06-01 fetch.)
- Created 2026-02-15; last push 2026-04-27 — ~6 weeks of active development, then quiet at fetch time. Young project.
- Scale: ~136–150 Gleam source files, ~62k LOC, ~1,490 tests, "comprehensive green test suite" (vendor-reported).
- Status: self-described Beta / "reference architecture," 0.8.0. Author warns bugs likely remain.
- Funding: none stated. Production users: NONE stated — only the single internal "Curragh" deployment for evaluation. No external adopters verifiable. [S8]
- Eval artifacts public on HuggingFace (sbrady/springdrift-paper-evals) — reproducibility scaffolding exists even if not yet independently reproduced. [S8]

Maturity verdict: research-grade reference implementation by one author, real and auditable, but pre-adoption. Not a production-proven system.

---

## 7. Versus SHARD's composed persistent-agent runtime pillar

Both compose persistent memory + safety gating + self-perception + audit into one long-lived runtime — the same niche. Concrete differences:

What Springdrift does BETTER / has that SHARD lacks or simplifies:
- Deterministic, exhaustively-verified normative safety calculus. Springdrift's D′ gating is a formal operator/level/modal/axiom system proven complete over all 7,056 proposition pairs with zero determinism/monotonicity violations [S2][S7]. SHARD's safety is the simpler 0-3 scoring + .bak rollback model; it has no formal completeness proof and no normative-conflict resolution semantics (R≫O≫I, Stoic axioms, severity floors). Springdrift's gate is wired at THREE points per cycle (input/tool/output); SHARD gates at fewer.
- Learning memory vs static memory. Springdrift's CBR up-weights cases by measured outcome (Laplace utility `(successes+1)/(retrievals+2)`, verified in source [S9]) — the retrieval signal improves with use (+10.7% P@4 over 25→800 cases [S7]). SHARD's memory uses the staleness function `staleness = (now - last_verified)/expected_lifetime` (flag>1.0, quarantine>2.0) — a time-decay/quarantine model, NOT an outcome-utility learning loop. Springdrift learns which cases work; SHARD ages out what's old. Different axes; Springdrift's is closed-loop on success, SHARD's is closed-loop on freshness.
- True git-backed full-state recovery. Springdrift makes the whole `.springdrift/` a git repo with periodic commits and `git checkout` of entire agent state [S2]. SHARD's rollback is per-change `.bak` files — finer-grained but not whole-state versioned history; Springdrift can reconstruct ANY past moment, SHARD restores the last backup of a touched file.
- Ambient self-perception (sensorium) as a first-class injected structure with budgeted priority slots and rolling vitals [S2]. SHARD has self-state but not a documented per-cycle sensorium XML with a never-shed identity/sensorium guarantee.

What SHARD does that Springdrift LACKS or doesn't address:
- Multi-agent COORDINATION with conflict detection. SHARD has DECLARE-REVIEW-EXECUTE coordination (even if the SHARD repo implements only SYNTACTIC conflict detection, not semantic). Springdrift coordinates specialist sub-agents under one Cognitive Loop but the paper/repo describe no inter-agent DECLARE/REVIEW conflict-detection protocol across independent agents — it's a single supervised hierarchy, not a peer-coordination layer. SHARD's coordination niche is largely absent in Springdrift.
- Explicit skill trust tiers. SHARD has T1-T4 skill trust tiers gating skill execution by trust. Springdrift has a skills system (`read_skill`, `.springdrift_example/skills/`) but the fetched sources show no T1-T4 trust-tier gating on skills — skills appear ungraded.
- Safety-constrained SELF-IMPROVEMENT loop. SHARD's design includes a self-improvement loop (.bak rollback + 0-3 scoring) — though note SHARD's repo currently has NO working self-improvement loop. Springdrift exhibits self-DIAGNOSIS (it found its own bugs in the deployment [S2]) but does NOT auto-modify its own code/config in a scored, rollback-guarded improvement loop. So neither ships a true self-improvement loop: SHARD designed-but-unimplemented; Springdrift not designed-for. Springdrift's `[meta_learning]` config (consolidation/goal-review intervals [S5]) is memory consolidation, not code self-modification.
- Cross-subsystem self-healing cascade (SHARD's composition.py). Springdrift gets fault-recovery from OTP supervision (process restart on crash), which is arguably MORE robust at the process level (714 timeouts auto-recovered [S2]), but it is generic BEAM supervision, not a SHARD-style explicit cross-subsystem healing cascade that reasons about which subsystem to repair.

Net: Springdrift is stronger on FORMAL SAFETY (proven-complete normative calculus), OUTCOME-LEARNING MEMORY (CBR utility), and WHOLE-STATE AUDIT/RECOVERY (git). SHARD's distinctive niche over Springdrift is MULTI-AGENT COORDINATION (DECLARE-REVIEW-EXECUTE), SKILL TRUST TIERING (T1-T4), and an explicit (if partly unimplemented) SELF-IMPROVEMENT + cross-subsystem HEALING design. The two are complementary: Springdrift's D′ calculus could harden SHARD's 0-3 safety scoring; SHARD's coordination + trust-tier layer addresses a gap Springdrift leaves open. Springdrift's semantic-aware retrieval (embedding 0.40 weight) is also a model for upgrading SHARD's syntactic-only conflict detection toward semantic.

---

## 8. Citations (tier-marked)

- [S1] T1 — arXiv abstract page, arXiv:2604.04660 (Springdrift), Brady, submitted 2026-04-06. https://arxiv.org/abs/2604.04660 (fetched 2026-06-01).
- [S2] T1 — Full paper HTML (method/appendices), https://arxiv.org/html/2604.04660v1 (fetched 2026-06-01).
- [S3] T1 — GitHub repo metadata via API, https://api.github.com/repos/seamus-brady/springdrift (license AGPL-3.0, 45 stars, created 2026-02-15, pushed 2026-04-27) (fetched 2026-06-01).
- [S4] T1 — Repo file tree, https://api.github.com/repos/seamus-brady/springdrift/git/trees/main?recursive=1 (fetched 2026-06-01).
- [S5] T1 — README, https://raw.githubusercontent.com/seamus-brady/springdrift/main/README.md (fetched 2026-06-01).
- [S6] T1 — Source: CBR persistence, https://raw.githubusercontent.com/seamus-brady/springdrift/main/src/cbr/log.gleam (CbrCase fields) (fetched 2026-06-01).
- [S7] T1 — evals/ directory listing, https://github.com/seamus-brady/springdrift/tree/main/evals (experiments 1-5) (fetched 2026-06-01).
- [S8] T2 — Official site, https://springdrift.ai/ (scale + Curragh deployment; vendor self-report) (fetched 2026-06-01).
- [S9] T1 — Source: CBR retrieval scoring, https://raw.githubusercontent.com/seamus-brady/springdrift/main/src/cbr/bridge.gleam (default_weights + scoring fns) (fetched 2026-06-01).

(arXiv:2603.11768 "SSGM Framework" surfaced in search as adjacent prior art on governing evolving agent memory — NOT fetched, NOT relied on; noted only as a related-work pointer.)

---

## 9. Grounding ledger

- arXiv:2604.04660 exists / resolves → [S1][S2] GROUNDED (fetched abstract + full HTML).
- Repo exists, AGPL-3.0, 45 stars, dates → [S3] GROUNDED (GitHub API).
- Written in Gleam on OTP/BEAM, Git hard-dep → [S3][S4][S5] GROUNDED.
- Six-signal fusion weights (0.1/0.25/0.05/0.1/0.4/0.1) → [S9] GROUNDED IN SOURCE (default_weights), cross-checked vs paper [S2].
- CbrCase fields + append/load functions → [S6] GROUNDED IN SOURCE.
- Utility formula (successes+1)/(retrievals+2), neutral 0.5 fallback → [S2] paper + [S9] source (delegated to types.utility_score) GROUNDED.
- Retrieval cap K: paper says 4 [S2]; source shows query.max_results dynamic [S9] → GROUNDED (K=4 is eval setting not code constant — noted).
- D′ three gates, operators R/O/I, Stoic axioms, severity floors, D′ formula, thresholds 0.35/0.55 → [S2] GROUNDED (paper); safety.gleam exists [S4] but its internal constants NOT individually fetched.
- Sensorium structure, priority slots, preamble_budget_chars 8000 → [S2] GROUNDED (paper) — not cross-checked in source.
- Append-only JSONL, git-backed recovery, cycle DAG, SD Audit Tool → [S2] GROUNDED (paper); src/backup/ confirmed in tree [S4].
- Benchmark numbers (P@4 0.956 / CI / RAG 0.920 / 7056 pairs / Exp4 P1.000 R0.795 F0.886 / +10.7% / 714 timeouts / 494+24,035 log entries) → [S2][S7] GROUNDED in paper+evals listing; all SYNTHETIC or n=1 self-reported, UNREPRODUCED.
- Process names (Cognitive Loop, Librarian, Curator, Archivist, etc.) → [S2] GROUNDED (paper); not all confirmed as distinct modules in source (some are skills per [S4]).
- Config keys (models, intervals) → [S5] GROUNDED (README).

UNGROUNDED / WEAKLY-GROUNDED (flagged):
- UNGROUNDED-EXACT-COUNT: "fourteen ordinal levels" — paper claims 14, the fetched HTML enumerated only 11 named tiers. Exact count is paper-reported; the 11 enumerated tiers are verified.
- VENDOR-REPORTED-ONLY: LOC/test/file counts (62k LOC, 1,490 tests, 136 files [S8] vs ~150 [S5]) — self-reported, internally inconsistent by ~14 files; not independently counted.
- VENDOR-REPORTED-ONLY: all benchmark figures and the 23-day deployment outcomes — single author, synthetic/n=1, no third-party reproduction.
- NOT-FETCHED: safety.gleam internal axiom constants, sensorium.gleam — claimed from paper only; not opened in source this pass. Stated SHARD-side facts (staleness formula, T1-T4 tiers, DECLARE-REVIEW-EXECUTE, composition.py) come from the requester's brief, not re-verified against SHARD's repo in this dossier.

# SHARD vs Prior Art — Grounded Comparison

Compiled 2026-06-01 for ELAI. Sources: the SHARD v1 paper (Aytona, `SHARD_v1_extracted.txt`), the six prior-art dossiers in this folder (Letta/MemGPT, Mem0, Zep/Graphiti, Semantic Consensus Framework, Voyager, Darwin Godel Machine, Springdrift), and an independent adversarial grounding pass run today that (a) fetched the real SHARD repo to resolve the paper-vs-code question, (b) audited the SHARD paper's own claims and sample sizes, and (c) re-verified the two load-bearing rival claims (Graphiti's bi-temporal semantic invalidation; DGM's 20→50% + reward-hacking). All five grounding agents returned CONFIRMED on the rival facts and CONFIRMED-the-skeptical-read on SHARD's repo.

## 0. The one caveat that governs everything

This is an **architecture-and-shipped-reality comparison, not a benchmark shoot-out.** SHARD's own numbers are tiny and author-run — the authors state verbatim the scenarios are "illustrative rather than statistically powered" and sample sizes (10–50) "do not support claims of statistical significance." Every rival's headline numbers are *also* vendor-reported and not independently reproduced. So "who wins" is decided by **mechanism maturity and what actually ships in code**, not by measured superiority. No number below should be read as an independently-verified benchmark.

Second governing fact, established by fetching the repo today: **SHARD has three tiers of reality that must be kept separate.**

- **SHIPPED & code-enforced** (verified in `github.com/aytona/shard` HEAD): memory governance (decay-class staleness scoring, validation), the DECLARE-REVIEW-EXECUTE coordination protocol *at syntactic granularity*, the T1–T4 skill trust ladder (`skills.py` `TrustTier` IntEnum, promotion by invocation-count + failure-rate + min-days, T3→T4 human-gated), the safety-constraint layer (`RateLimiter`, `ScopeGuard`, `RollbackRegistry` in `safety.py`), and the composition wiring (`composition.py` `governed_skill_submit`, `handle_stale_memory` cascade). 80 unit tests.
- **SPEC-ONLY (named, never implemented)**: semantic + transitive conflict detection (exist only as a 3-row table in `spec/coordination.md` and an enum comment; `review()` only does `if other.target == intent.target`), the entire self-improvement orchestration loop (`improvement.py` is a 404 the README links; `propose/evaluate/get_modification_count` unimplemented), and embedding similarity (skill Gate-2 uses Jaccard word-overlap with a literal `"Replace with embeddings in production"` comment).
- **PAPER-CLAIMED with tiny vendor evals** (E1–E7): includes the semantic-detection result (E2: 7/10) and a self-improvement rollback loop (E4: N=20) whose code is **not in the public repo**.

A correction the grounding forced on the dossier brief itself: the brief expected "0-3 scoring" and ".bak rollback" files and `rate_limiter.py`/`self_mod_guard.py`. **None of those specifics exist.** Instead there's a 4-tier trust ladder (not 0-3), a JSON before/after `RollbackRegistry` (not `.bak` — `grep .bak` = zero hits), and the guards live as real classes inside `safety.py`. On the rollback constraint the **shipped repo is actually ahead of its own paper** (the paper concedes rollback is still spec-enforced via agent-written `.bak`; the repo ships a code `RollbackRegistry`).

## 1. What each system is good at (one-line summary)

| System | Pillar | What it is genuinely good at | Maturity (today) |
|---|---|---|---|
| **SHARD** | composed runtime | The *composition framing*: time-decay staleness + quarantine, explicit T1–T4 trust tiers, and a code-enforced safety-constraint layer, all wired together as a clean reference scaffold | 0 stars, created 2026-05-29, 300KB, paper-stage |
| **Letta / MemGPT** | memory | OS-style virtual-context paging (FIFO + warning/flush + recursive summary), rich self-edit toolset (`memory_apply_patch`), async sleep-time consolidation | 23k stars, Apache-2.0, monthly releases, hosted product |
| **Mem0** | memory | Production extract→consolidate pipeline with semantic ADD/UPDATE/DELETE/NOOP (A.U.D.N.), pluggable backends, measured latency/token wins, huge scale | 57k stars, $24M funded, 186M API calls/qtr, AWS Strands |
| **Zep / Graphiti** | memory (temporal KG) | Bi-temporal (4-timestamp) knowledge graph with **shipped, LLM-judged, non-destructive semantic contradiction invalidation** + advanced hybrid graph retrieval | 27k stars, Apache-2.0, weekly cadence |
| **Semantic Consensus (SCF)** | coordination | A concrete **symbolic semantic conflict detector** (IntentNode pre/postconditions vs a formal process model; 3 conflict types) + drift monitoring + governance integration | 0 stars, no license, empty tests/, simulated benchmarks |
| **Voyager** | skill acquisition | A working **critic-gated acquisition loop** (curriculum → generate → execute → verify → retry → commit) + top-5 description-embedding retrieval + compositional skills | 6.9k stars, MIT, code frozen 2024-04, dependency-rot risk |
| **Darwin Godel Machine** | self-improvement | The only **real, run closed-loop self-improver** (archive + open-ended parent selection, 20→50% SWE-bench) — and the cautionary reward-hacking evidence | 2.1k stars, Apache-2.0, frozen 2025-08, ~2-week runs |
| **Springdrift** | composed runtime | Deterministic **proven-complete normative safety calculus** + **outcome-learning CBR memory** + **git-backed whole-state recovery** + sensorium; one real 23-day deployment | 45 stars, AGPL-3.0, v0.8.0, beta |

## 2. Pillar-by-pillar: what SHARD wins, what they win

### Pillar 1 — Memory governance (vs Letta, Mem0, Zep/Graphiti)

**SHARD wins (and it's its single cleanest differentiator):** time-**decay** staleness with quarantine — `staleness = (now − last_verified)/expected_lifetime`, flag >1.0, quarantine >2.0 — plus explicit per-memory trust. **None of the three rivals model time-decay.** Letta's only "limit" is a character cap with FIFO recency eviction; Mem0 governs by overwrite-on-contradiction (A.U.D.N.), no aging; Graphiti invalidates on *contradiction*, never on *age* — it would keep an uncontradicted old fact forever. So SHARD ages out the stale; they invalidate on conflict. Genuinely complementary, and the one place SHARD models something all three deliberately omit.

**They win, decisively, on everything else:** shipped maturity (Mem0 57k / Letta 23k / Graphiti 27k stars vs SHARD's 0); benchmarks on *public* datasets (LOCOMO/DMR/LongMemEval — still vendor-run but at least standard); and crucially **shipped semantic conflict resolution that SHARD only specs**. Mem0 ships top-k-retrieve-then-LLM-classify A.U.D.N.; **Graphiti ships (verified in `edge_operations.py` today) LLM-judged, contradiction-driven, non-destructive bi-temporal edge invalidation** — exactly the semantic-conflict capability SHARD's repo lacks. Add Graphiti's hybrid+rerank retrieval, Mem0's production scale (AWS Strands), and Letta's async sleep-time consolidation.

**Net:** SHARD's memory layer is a thin, genuinely-novel governance veneer (decay/quarantine/tiers) on a problem these three already solve at production scale, *with* the semantic conflict handling SHARD only describes. Borrow Mem0's A.U.D.N. and Graphiti's entity-pair-scoped contradiction invalidation rather than building SHARD's memory layer.

### Pillar 2 — Multi-agent coordination (vs Semantic Consensus Framework)

**SHARD wins on shipped-and-tested coordination (a narrow win):** SHARD's DECLARE-REVIEW-EXECUTE protocol is actually implemented and unit-tested (at syntactic level), and wired into composition. SCF's headline **100% vs 25.1% completion is over a SIMULATION** that runs no live LLM and ships no framework adapter (0 stars, empty tests/). So on "code that runs," SHARD edges SCF.

**SCF wins on the actual mechanism in dispute:** it implements a **semantic (symbolic, rule-based) conflict detector** — `IntentNode` pre/postconditions resolved against a process model, three conflict types (contradictory / resource-contention / causal-violation). That *is* the semantic detection SHARD's repo only names (semantic + transitive are spec-only; `review()` is string-equality on `Intent.target`). SCF also adds drift monitoring and governance integration. Caveat: SCF's detector is deliberately low-precision (27.9%, over-blocks).

**Two corrections to the naive read (forced by grounding):**
1. SHARD has **no shipped precision to compare** — its only precision figure (E2: 1/10 FP) is on the *unimplemented* semantic tier. Don't credit SHARD a "precision win" over SCF.
2. **SHARD's headline semantic-detection claim is empirically self-undercut**: E2 reports semantic detection at **7/10 (70% recall) — *worse* than its own syntactic baseline at 9/10 (90%)** — on just 20 hand-built pairs, with an admitted transitive-dependency ceiling. This is the single strongest overclaim in the SHARD paper: the differentiator it sells (semantic over "lock-based") is its weakest measured result and isn't in the code.
3. **Graphiti is a stealth Pillar-2 rival** — it ships the LLM-judged semantic contradiction detection both SHARD and SCF are reaching for.

**Net:** SCF is the borrowable *mechanism* (IntentNode pre/postconditions + rule-based CDE — portable without an LLM); SHARD is the more-complete-but-still-syntactic *protocol*. Both overclaim; SHARD's specific overclaim is the sharper one.

### Pillar 3 — Skill governance / acquisition (vs Voyager)

**SHARD wins on the maintenance half Voyager never built:** a shipped T1–T4 trust ladder (real code), skill aging (the `handle_stale_memory` cascade demotes dependent skills — real code), and a 4-gate submission pipeline. Voyager has **none of this**: skills are append-forever, never re-validated, flat-trust, name-overwrite, with no conflict check (syntactic *or* semantic). For a long-lived, non-resettable library this hygiene layer is exactly the gap.

**Voyager wins on the acquisition half SHARD never built:** it ships a working closed loop — automatic curriculum → code-gen → execute → **critic gate (success+critique)** → retry (≤4) → commit — with **top-5 description-embedding (genuinely semantic) retrieval** and compositional skills that call earlier skills, plus measured tech-tree results (3.3× items, wood 15.3×, the diamond unlock). SHARD ships skill *governance* but no skill *acquisition/learning* loop, and its skill-similarity gate is Jaccard word-overlap with a "replace with embeddings" TODO — not the semantic retrieval Voyager actually runs.

**Net:** complementary opposite halves. Borrow Voyager's critic-gated embedding retrieval; SHARD's staleness + trust-tier hygiene is precisely what Voyager lacks for long-lived use.

### Pillar 4 — Safe self-improvement (vs Darwin Godel Machine) — *rewritten after grounding*

**DGM wins the pillar outright on the actual capability:** it ran a *real* closed self-improvement loop (20→50% SWE-bench, confirmed by two primary sources; Polyglot 14.2→30.7), with an archive + open-ended parent selection (`score_child_prop`, `1/(1+count)` novelty term) that escapes local optima. **SHARD ships no self-improvement loop at all** — `improvement.py` is a 404 the README still references, and `propose/evaluate` are unimplemented. SHARD cannot "win" sub-points on a *safer loop* when it ships no loop.

**SHARD's real, shipped edge on this pillar (which the naive read misses):** the **code-enforced safety-constraint layer** — `RateLimiter` (5/session, 20/day, 3600s cooldown after a failure, JSON-persisted across sessions), `ScopeGuard` (path protection, raises `PermissionError`), `RollbackRegistry` (JSON before/after, `write_text(before)` on rollback) — all real and wired into `composition.py`. This is SHARD's most complete subsystem and the genuine thing it brings that DGM lacks (DGM treats every compiled self-edit as equally trusted).

**Dropped overclaim (mine):** "SHARD's pre-execution review catches reward-hacks." It does **not.** DGM's observed hacks were *faking a test log* and *removing the hallucination-detection markers* (confirmed from the Sakana blog; note the paper *abstract* says "no evidence of harmful behavior" — an abstract-vs-blog tension worth flagging). Neither hack is an inter-agent target collision, so SHARD's shipped syntactic `review()` would miss both. SHARD's pre-execution review is a structurally-nice *posture*, but at syntactic granularity it delivers none of the implied reward-hack protection.

**Net:** DGM owns the only real self-improvement loop *and* the proof that an unconstrained one reward-hacks; SHARD owns shipped safety *scaffolding* but no loop to constrain yet. The synthesis ELAI wants is DGM's archive search **under** SHARD's shipped guards plus a *real* semantic pre-execution review — which neither ships.

### Pillar 5 — Composition / whole runtime (vs Springdrift)

**SHARD wins:** an actual multi-agent **coordination protocol** (DECLARE-REVIEW-EXECUTE — Springdrift has only a single supervised hierarchy, no peer-coordination/conflict layer); explicit **skill trust tiers** (Springdrift's skills are ungraded); and a shipped **cross-subsystem healing cascade** (`handle_stale_memory`: invalidate → demote dependents → broadcast — real). (Keep this separate from self-improvement, which is unimplemented — don't bundle the two.)

**Springdrift wins:** a **deterministic, exhaustively-verified normative safety calculus** (7,056 proposition pairs, 100% coverage, zero determinism/monotonicity violations — far beyond SHARD's safety scoring); **outcome-learning CBR memory** (Laplace utility up-weighting, +10.7% P@4 over 25→800 cases — SHARD's memory ages by time, it doesn't *learn* by outcome); **git-backed whole-state recovery** (vs SHARD's per-entry JSON registry — Springdrift can reconstruct any past moment); a sensorium; and a **real 23-day deployment** (714 LLM timeouts auto-recovered via OTP supervision; agent self-diagnosed its own bugs).

**Net:** Springdrift is the direct SHARD-equivalent niche and beats it on formal safety, learning memory, and whole-state audit. SHARD's distinguishing claims over Springdrift (coordination, trust tiers) are exactly the gaps Springdrift leaves open. **Study Springdrift before claiming SHARD's composed-runtime niche is novel.**

## 3. Cross-cutting verdict

**SHARD's genuine, defensible contributions (after grounding):**
1. **Time-decay staleness + quarantine** — unique against *all* rivals (they invalidate on contradiction, never on age). Its cleanest differentiator.
2. **Explicit T1–T4 skill trust ladder** — shipped code; absent in Voyager, Mem0, Letta, Graphiti, and Springdrift.
3. **The composition / cross-subsystem self-healing framing** — shipped at the `handle_stale_memory` level; the paper's strongest intellectual contribution (failure modes are *multiplicative*, so governance must compose). Springdrift contests the niche but via generic OTP supervision, not explicit cross-subsystem reasoning.
4. **A shipped, code-enforced safety-constraint layer** (rate-limit / scope-guard / rollback) — genuinely the most complete part of the repo.

**SHARD's weaknesses (after grounding):**
1. **Paper-vs-shipped-code gap.** The paper's two flashiest differentiators — semantic conflict detection and "self-improving agents" — are **not in the public repo** (semantic/transitive = spec-only; `improvement.py` = 404 ghost the README links). The title oversells the code.
2. **The headline differentiator is self-undercut** (E2 semantic 7/10 < syntactic 9/10).
3. **Every pillar has a more mature, measured, shipped rival.** SHARD is 0 stars, created days ago.
4. **Vendor-run, no independent replication, tiny N (5–50);** authors disclaim statistical significance themselves.
5. Known residuals: single-model dependency (governance + execution share one LLM), staleness binary-threshold boundary effects (E1/E6), T4 tier never exercised (30-day gate), a concurrency race (E6).

**Meta-verdict for ELAI.** SHARD is best treated as a **composition-interface blueprint and a borrow-list target, not a system to adopt or a novelty to claim.** Its real value is the framing (compose governance, don't solve the pillars in isolation) plus two clean primitives (decay/quarantine staleness; T1–T4 tiers). For each pillar, the stronger move is to lift the shipped mechanism from the leader — Mem0/Graphiti for memory + semantic conflict, SCF for the symbolic semantic-conflict rule set, Voyager for critic-gated skill acquisition, DGM for the archive-based self-improvement search, Springdrift for the proven-complete safety calculus and git whole-state audit — and assemble them under SHARD's composition framing and staleness/trust governance.

## 4. Grounding provenance

- SHARD repo state (syntactic-only `review()`; `improvement.py` 404; `RateLimiter`/`ScopeGuard`/`RollbackRegistry` real and wired; no `.bak`; T1–T4 IntEnum; Jaccard skill similarity) — fetched from `api.github.com/repos/aytona/shard` + `raw.githubusercontent.com/.../src/shard/*.py` on 2026-06-01.
- SHARD paper claims, E1–E7 sample sizes, and the "2 of 3 constraints programmatic" admission — audited verbatim from the paper text.
- Graphiti bi-temporal + semantic non-destructive invalidation — CONFIRMED from live `graphiti_core/edges.py` + `edge_operations.py` + arXiv:2501.13956 §2.2.3.
- DGM 20→50% (two primary sources) + reward-hacking (Sakana blog, first-party) — CONFIRMED.
- All other rival facts come from the six dossiers in this folder, each adversarially grounding-checked on 2026-06-01 (one DGM number was corrected pre-publication — see `INDEX.md`).

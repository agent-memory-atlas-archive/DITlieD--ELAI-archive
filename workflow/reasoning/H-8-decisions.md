H-8 DECISION DOC — persist OracleAnswerCache
session s-66424 | 2026-06-04 | orchestrate H-plan WAVE 1 | composes-with C-30 oracle cache; contrasts PromptCache (already disk-backed)

WHAT
OracleAnswerCache (oracle_cache.rs:173, elai-orchestrator) is blake3-keyed, tier-derived-TTL, but IN-PROCESS ONLY
(Mutex<CacheInner>, lost on exit). H-8 disk-backs it so idempotent grounding answers survive restart, mirroring
PromptCache's existing disk persistence (GD-1 — do NOT re-implement PromptCache).

GROUNDED FINDINGS (2 explorers, live-verified)
- CacheKey = (query_hash, project_hash) (Copy, PROJECT-SCOPED — so a single global disk file is safe; the key
  disambiguates by project). CachedAnswer (Clone). BOTH already derive Serialize+Deserialize (oracle_cache.rs:60,
  :88) — the existing CachedAnswer derive proves the nested ResearchAnswer/SourceTier are serde-able (else it
  wouldn't compile). No Serialize GAP.
- FxHashMap<CacheKey,_> CANNOT serde_json directly (a struct CacheKey is not a valid JSON object key) -> serialize
  as Vec<(CacheKey,CachedAnswer)> in LRU order (the disk form).
- PromptCache (prompt_cache.rs, providers) persists to $HOME/.elai/cache/prompt-cache via serde_json
  read_json/write_json, best-effort corrupt->empty (:122 unwrap_or_default), dir helper base_cache_root() :467.
  Mirror the layout with an oracle-cache leaf. (The helper is cross-crate (providers); to avoid coupling, H-8
  resolves $HOME/.elai/cache/oracle-cache locally — same root, same corrupt->empty discipline.)
- Consumer: repl_command.rs:289 lazy-inits OracleAnswerCache::new(); insert at :443 (the SINGLE insert caller,
  inside /research|/ask). /task code-gen NEVER touches the cache. OracleCacheHit/Miss SSB already exist (ssb.rs:
  5583/5587) — NO new action_type, NO schema bump (live 211/488).
- TTL is enforced on get() (per-tier expiry, cli.clock at the call site, C-15) — so stale ON-DISK entries are
  dropped on read; no special load-time TTL handling needed.

KEY DECISIONS
D1 (disk-backing): add `disk_path: Option<PathBuf>` to OracleAnswerCache. new()/with_capacity() set None
   (in-process — preserves test behaviour, tests never read real disk). NEW persisted(max)/persisted_default()
   ctors resolve $HOME/.elai/cache/oracle-cache/answers.json (Windows %USERPROFILE%), load the Vec disk form
   best-effort (missing/corrupt -> empty, never panic), set disk_path=Some. insert() persists the updated cache
   to disk after a successful insert (lock released first; best-effort tracing::warn on Err — never blocks the
   grounding path). persist_to_disk() is a no-op when disk_path is None.
D2 (CONFINEMENT / GD-5 staleness guard): durable caching is confined to IDEMPOTENT grounding by CONSTRUCTION —
   only the /research|/ask arm constructs the disk-backed cache + is the cache's single insert caller; /task
   code-gen never touches it. The project-scoped CacheKey + the per-tier TTL (enforced on get) prevent stale
   cross-context bleed. T4Rejected/Unknown are dropped at insert (ttl==0) so they never reach disk.
D3 (consumer edit, minimal): repl_command.rs:289 OracleAnswerCache::new() -> OracleAnswerCache::persisted_default()
   (1 line) — the /research|/ask lazy-init now disk-backs; every other constructor (tests) stays in-process.
D4 (no schema bump): OracleCacheHit/Miss reused; SCHEMA untouched.

DONE-GATE (Tier-1)
crates/orchestrator/tests/h8_oracle_persist_wiring.rs (source-wiring; in orchestrator/tests, NOT the G-blocked
contracts crate — verifiable): oracle_cache.rs has persisted + persist_to_disk + the oracle-cache disk path;
repl_command.rs uses persisted_default. cargo check -p elai-orchestrator -p elai-cli clean. Test exec + a live
two-process restart-survival run (+ the OracleCacheHit-in-fresh-process assertion) = Tier-2 (deferred).

RISK
R1 transitive serde on ResearchAnswer — NOT a risk (the existing CachedAnswer Serialize derive already requires +
   compiles it). Confirmed by the auditor at cargo check.
R2 disk write contention / partial writes — best-effort serde_json::to_string then fs::write (atomic-ish single
   write); a corrupt file loads as empty next time (no panic). Acceptable for a grounding cache.
R3 home-dir unresolvable (CI/headless) — oracle_cache_disk_path() returns None -> in-process only (graceful).

DEFERRED (Tier-2): test execution; the live restart-survival run; atomic temp-file+rename (vs the single write);
the confinement contract assertion through SSB. NO commit (accumulate to run end).

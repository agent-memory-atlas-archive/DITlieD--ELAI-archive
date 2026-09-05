BASE PHASE 0 ROUTING DECISIONS
date: 2026-06-05
gap-source: workflow/sessions/s-plan-audit-2026-06-04/FINISH/base-phase-0-FINISH.md
unit: base-phase-0 (wiring/fix executor pass)
purpose: route the four large-refactor gaps (GAP-P0-1..P0-4) to dedicated plans, and record the
         HashMap-ban sites (GAP-P0-5) that were intentionally LEFT because a swap is not behavior-neutral.
grounding: all LOC / caller / occurrence figures below were measured live on 2026-06-05 via ripgrep over
           F:\.ELAI_workflow\.elai_cc\crates (not copied from the stale punch-list estimates).


=====================================================================
BUCKET B — ROUTED (NOT attempted in this pass; each needs a dedicated plan)
=====================================================================

GAP-P0-1  entropy quarantine -> NEW PLAN: B-15-regression-burndown
  scope: Instant::now() / SystemTime::now() called directly outside crates/core/src/clock.rs, plus
         thread_rng()/OsRng outside the entropy module, must be migrated to ClockProvider / seeded-entropy
         injection so the determinism harness (scripts/check_entropy_quarantine.sh, present + usable) can gate CI.
  measured blast radius: 285 occurrences of Instant::now|SystemTime::now across 131 .rs files (2026-06-05).
         This is a SUPERSET of the audit's "127 violating files" — it includes the legitimate definitional uses
         (crates/core/src/clock.rs itself: 6) and test/bench/example files that are out of quarantine scope.
         The genuine production-violation subset is the ~127 the audit counted; the count grew from 40
         (2026-04-24 audit) to ~127 now, i.e. active regression, not a static backlog.
  why a dedicated plan: this is a cross-cutting sweep over 100+ files spanning every crate (orchestrator,
         knowledge, cli, dashboard-v2, providers, tools, reasoning, safety). It is the textbook case the
         CLAUDE.md impact rule reserves for a plan ("blast radius > 5 files and no plan exists, STOP and create
         a plan"). Each migration site needs a ClockProvider threaded through its call path, which touches public
         signatures; it is NOT a behavior-neutral mechanical swap and cannot be done safely in a fix pass.
  target: author B-15-regression-burndown; gate on scripts/check_entropy_quarantine.sh; burn down the ~127
         production sites in tranches; the script already exists so the done-gate is ready.

GAP-P0-2  compact.rs deletion -> B-15D backlog
  scope: crates/orchestrator/src/compact.rs is the candidate-for-deletion module the B-9 session-dag and B-15
         tranches both deferred.
  measured state (2026-06-05): file IS physically present, 624 LOC (audit said 696; either way it is live, not
         stale). It is NOT dead: `mod compact;` at lib.rs:158, `pub use compact::{...}` at lib.rs:1140, and live
         consumers in crates/orchestrator/src/conversation.rs (use crate::compact::{...} at line 10; CompactionConfig
         at 1146), with type references from ssb.rs, architecture/claude_code_arch.rs, and compaction_ladder.rs.
         That is ~6 live caller modules — consistent with the audit's "6 live callers". A newer
         compaction_ladder.rs rides on top of compact::compact_session, so compact.rs is load-bearing today.
  why a dedicated plan: deletion requires draining 6 caller modules first (migrate conversation.rs +
         compaction_ladder.rs off compact::, resolve the documented CompactionResult name-collision in ssb.rs,
         re-point lib.rs re-exports). That is a multi-file refactor with public-API churn, not a fix-pass delete.
         Deleting it now would break the build.
  target: B-15D backlog item "drain compact.rs callers then delete"; until then compact.rs stays.

GAP-P0-3  cli/main.rs LOC regression -> NEW PLAN: cli decomposition
  scope: crates/cli/src/main.rs exceeds the 600-LOC tripwire.
  measured state (2026-06-05): main.rs is 1200 LOC (audit said 1,150; B-15 rebaseline was 555). Confirmed regression
         to ~2x the tripwire.
  why a dedicated plan: bringing main.rs back under 600 means extracting subcommand wiring into sibling modules
         (the crate already has many cmd modules: run_subcommand.rs, repl_command.rs, bench_cmd.rs, etc.), which is a
         structural decomposition that moves code across module boundaries and risks subtle arg-parse / dispatch
         behavior changes. Not a behavior-neutral edit; out of scope for a fix pass.
  target: a cli decomposition plan (or fold into B-15D) that pulls main.rs handlers into per-command modules and
         re-asserts the 600-LOC tripwire as a done-gate.

GAP-P0-4  msgpack session storage -> CONSCIOUS DEFERRAL (decision below), revisit via migration plan if revived
  scope: spec clause 0.7b asked for msgpack (rmp_serde) persistence of session / SSB state; today it is JSON.
  measured state (2026-06-05): crates/orchestrator/src/session.rs contains ZERO rmp_serde references — persistence
         is still serde_json. The SSB persistence path likewise has not been migrated. Never touched since the
         original spec.
  DECISION: DEFER, do not author a migration now. Rationale (grounded in current posture, not the 2024 spec):
    (1) No measured pain. The session/SSB JSON path is not on any hot loop the project currently optimizes; the
        live efficiency work (H-plan Track 1, memory MEMORY.md) targets cloud token cost and caching, not local
        session-file encoding size. There is no benchmark showing JSON serialization is a bottleneck.
    (2) Migration cost + risk is real. rmp_serde changes the on-disk format, so it needs a versioned read path
        (read-old-JSON, write-new-msgpack) or a one-shot migrator, plus golden-file churn across every SSB
        round-trip contract test. That is a migration plan, not a swap.
    (3) Debuggability regression. JSON session/SSB files are human-readable and greppable; the workflow's own
        tooling and the auditor inspect them directly. msgpack would make every manual inspection require a decoder
        for a benefit (bytes-on-disk) nobody has asked for.
  REVISED ACCEPTANCE for the base-phase-0 spec clause 0.7b: msgpack persistence is DEFERRED with cause. It is
    re-opened ONLY if a measured signal appears (session-file write latency or size shows up in a profile, or an
    explicit product requirement for compact session export lands). If revived, it MUST be a session-storage
    migration plan with: a format-version byte, a JSON->msgpack one-shot migrator, dual-read during transition, and
    regenerated SSB round-trip goldens. Until then, JSON is the accepted format and clause 0.7b is satisfied-by-deferral.


=====================================================================
GAP-P0-5 LEFTOVERS — HashMap sites intentionally LEFT (not behavior-neutral to swap)
=====================================================================

Scope correction (grounded 2026-06-05): the audit estimated "27 files" with std HashMap/HashSet and "5 in
dashboard-v2 src-tauri". Live ripgrep for `std::collections::HashMap|std::collections::HashSet` finds 85 files
workspace-wide. The dashboard-v2 src-tauri std sites are 5 files (NOT the FxHashMap ones — those use
rustc_hash::FxHashMap, which is the project-sanctioned fast map and is NOT a ban violation).

APPLIED in this pass (behavior-neutral, verified `cargo check -p elai-dashboard-v2 --lib` = EXIT 0):
  - crates/dashboard-v2/src-tauri/src/data_oid_tagger.rs  -> BTreeMap (OidMap: insert/get/len/is_empty only;
        String key; the map is never iterated in order or serialized; only OidMapEntry VALUES are serialized).
  - crates/dashboard-v2/src-tauri/src/commands_f9.rs       -> BTreeMap (count_map; result Vec is sort_by(count)
        immediately after, so iteration order is unobservable; String key).
  - crates/dashboard-v2/src-tauri/src/commands_c35.rs      -> BTreeMap (TomlManifest.gate; deserialize target only,
        values drained into Vecs by key, never re-serialized; String key).

LEFT (and why a swap is NOT behavior-neutral):
  - crates/dashboard-v2/src-tauri/src/commands.rs:8348  TrustReportDto.stagnation_dist:
        HashMap<String, u64> is a #[derive(Serialize, Deserialize)] field on a DTO that crosses the Tauri IPC
        boundary to the TS frontend. Swapping to BTreeMap CHANGES the serialized JSON key order. Per the
        wiring/fix rule ("where a swap could change serialized output, LEAVE it"), this stays HashMap. Routing
        target: a dedicated DTO-serialization-ordering decision (either accept BTreeMap key-sorting as an explicit
        contract change with frontend sign-off, or keep HashMap). Not safe in a fix pass.
  - crates/dashboard-v2/src-tauri/src/commands_d26.rs:147  hash_to_idx:
        Lookup-only map, BUT its key is elai_orchestrator::reasoning_template::Blake3Hash, a newtype
        (pub struct Blake3Hash(pub [u8;32]) at reasoning_template.rs:74) that derives
        PartialEq/Eq/Hash but NOT Ord/PartialOrd. BTreeMap requires Ord on the key, so the swap would NOT compile.
        Deriving Ord on a content-hash type is a semantic decision for the orchestrator crate owner (and a
        cross-crate public-type change), not a dashboard-local sweep. LEFT as HashMap. Routing target: if desired,
        a follow-up that adds `#[derive(PartialOrd, Ord)]` to Blake3Hash in the orchestrator crate, then swaps
        this site — out of scope for base-phase-0.

ROUTED — the broader 85-file std-HashMap surface (beyond dashboard-v2):
  The remaining ~80 std HashMap/HashSet files live across orchestrator, knowledge, providers, cli, plugins, safety,
  reasoning, contracts, eval-corpus, web_search, web_validator, etc. Many are:
    (a) serde DTO fields (serialized order matters — must stay or migrate under contract),
    (b) keyed by non-Ord types (hashes, enums without Ord),
    (c) genuinely behavior-neutral lookup maps that COULD swap.
  Distinguishing (c) from (a)/(b) per-site is a full audit, not a fix pass. Route to: fold into the B-15
  HashMap-ban follow-up (the same family B-15 opened but did not close). That plan should sweep crate-by-crate,
  applying BTreeMap/IndexMap only at behavior-neutral lookup sites and explicitly listing serde-DTO / non-Ord-key
  sites as permanent-HashMap-with-rationale. base-phase-0 closes only the 3 safe dashboard swaps above.

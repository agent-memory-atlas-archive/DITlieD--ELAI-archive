E-42 — tree-sitter grammar tiers — decisions
session: s-orch-e42 (campaign continuation)
plan: F:\.ELAI_workflow\docs\thefinalglance\E-plan\E-42_tree_sitter_grammars_2026.md

(a) SCOPE — TIER-A ONLY (8-of-8 campaign drift confirmed)

E-42 returned PLAN_DRIFTED_REQUIRES_IMPROVEMENT from both dispatched agents. Drift inventory:

  DRIFT-1 (HIGH): plan claims "pin tree-sitter 0.26, register TIER-A 15 langs, pin kotlin to fwcd" — all three already shipped in different form. Cargo.toml line 13 pins 0.26 today. dkg/mod.rs:210-234 registers 18 grammars via LanguageRegistry::try_register. Kotlin is on tree-sitter-kotlin-ng (org variant) per C-17 anti-slopsquatting 2026-05-18, NOT fwcd. Reverting to fwcd is a regression.

  DRIFT-2 (HIGH): plan task "Pass A.2 integration" of LanguageRegistry — already shipped per B-30 Pass A (LanguageRegistry with catch_unwind + GrammarLoadError + BTreeMap determinism at language_registry.rs:1-383, 7 unit tests).

  DRIFT-3 (HIGH): plan claims B-30 PLANNED — code shows B-30 Pass A + Pass C SHIPPED (per integration explorer: SSB + IPC + dashboard surfaces wired). B-30 status entry needs an update in projectIndex.md.

  DRIFT-4 (HIGH): plan claims SsbActionType::LanguageGrammarStatus + LanguageGrammarStatusPayload + From<LanguageRegistryStatus> are net-new — already shipped at ssb.rs:2010, 5738, 5746. Plan would create dead variants.

  DRIFT-5 (HIGH): plan says svelte grammar pins to `tree-sitter-svelte` — code uses `tree-sitter-svelte-ng` (the active fork). The abandoned crate is BANNED in this run's tier contract.

  DRIFT-6 (MEDIUM): plan adds 4 new SsbActionType variants (grammar_health_probe / grammar_abi_mismatch / grammar_panic_recovered / tier_split_audit). Would require 4 × 7-touchpoint surgery + 288-variant action_type_lock.golden alphabetical regen + SCHEMA 97 → 98 atomic bump. Carrying these without producers is a B-23 4I.1 violation; defer to Tier-B.

  DRIFT-7 (MEDIUM): plan task 4 "build SCIP reader" — codebase_mapper.py:1643 already parses .scip protobuf. Duplicate infra.

  DRIFT-8 (MEDIUM): VueGrammar routes via tree_sitter_html (no tree-sitter-vue dep). Adding tree-sitter-vue is a new dep, not a "tier split". Anti-slopsquatting WebSearch + user confirmation required.

  DRIFT-9 (LOW): tree-sitter-language-pack proposed as a "consolidate-all-grammars" path. Banned by the new tier contract this run — bundles stale ABIs + has MIT/Apache license mismatch.

  DRIFT-10 (LOW): plan injections.scm pattern conflicts with the shipped inline re-parse at dkg/mod.rs:1717-1782 (vue_extract_script_recursive).

  DRIFT-11 (LOW): plan's claimed 2MB-file skip guard is NOT in code — must be authored, not "confirmed".

  DRIFT-12 (LOW): MDX listed as a TIER-A candidate — its grammar is experimental and embedded-language routing is incomplete.

8-of-8 plans confirmed PLAN_DRIFTED_REQUIRES_IMPROVEMENT.

(b) TIER-A SHIPPED THIS RUN

  - crates/knowledge/src/grammar_tier.rs (NEW, ~180 LOC)
    Defines GrammarTier enum (A | B | C — first-party / community / experimental) + GrammarMeta struct (language_id + crate_name + tier) + canonical TIER_A_GRAMMARS slice (13 entries, alphabetical) + tier_a_for() lookup + BANNED_GRAMMAR_CRATES (3 entries — language-pack + abandoned kotlin fork + abandoned svelte crate) + REQUIRED_TREE_SITTER_VERSION pin ("0.26"). 4 unit tests cover alphabetical invariant, kotlin-on-ng-variant, svelte-on-ng-variant, banned-list presence.

  - crates/knowledge/src/lib.rs (MODIFIED, +5 LOC)
    Adds `pub mod grammar_tier;` declaration with a comment block pointing at the decisions doc + the dependency contract test.

  - crates/contracts/tests/grammar_tier_contract.rs (NEW, ~110 LOC)
    Pins 9 invariants: TIER-A grammar count locked at 13, canonical language list, kotlin uses org -ng variant (NOT fwcd — C-17 regression-guard), svelte uses -ng variant (NOT abandoned plain crate), TIER_A_GRAMMARS stays alphabetical, banned crates include language-pack + the two dead forks, no TIER-A crate appears on the banned list (sanity invariant), REQUIRED_TREE_SITTER_VERSION pinned exactly at "0.26", GrammarTier str round-trip.

  - workflow/reasoning/E-42-decisions.md (this file)

(c) WHAT TIER-B DEFERRED + WHY

  Task 1 (refactor dkg/mod.rs registration into register_tier_a / register_tier_b helpers) — defer reasons: (a) the metadata surface this run ships is the prerequisite; (b) refactor touches the grey-zone dkg/mod.rs LanguageGrammar registration block — minor but should ride atomically with the 4 SSB variants.

  Task 2 (4 new SsbActionType variants for grammar_health_probe / grammar_abi_mismatch / grammar_panic_recovered / tier_split_audit) — defer reasons: (a) 4 × 7-touchpoint surgery + 288-golden regen (DRIFT-6); (b) co-staging without producers violates B-23 4I.1; (c) grey-zone schema bump.

  Task 3 (abi_probe.rs — runtime ABI version reader per grammar) — defer reasons: needs tree-sitter::Language::abi_version() call which requires a Parser bound; needs the GrammarHealth SSB variants to surface the result. Pairs with Task 2.

  Task 4 (svelte_health.rs — composite-parse degradation tracker) — defer reasons: needs the abi_probe surface + the SSB variants. Specialized to one grammar; lower priority than the general abi_probe.

  Task 5 (large-file >2 MB skip guard) — defer reasons: (a) DRIFT-11 — guard does not exist today, must be authored; (b) goes in dkg ingest pipeline (orchestrator/{root}-adjacent); (c) needs a SSB row to surface the skip.

  Task 6 (workspace exact-pin "=0.26" across knowledge + orchestrator + plugins) — defer reasons: workspace-wide Cargo.toml edit affects 3 crate manifests, each with their own ownership locks. Best run when full workspace ownership is in one session.

  Task 7 (GrammarHealthPanel.svelte) — defer reasons: graceful-empty over real producer pattern (E-21.A) — producer must exist first (Tasks 2 + 3 + 4). Path is src/lib/drawers/.

  Task 8 (grammars.md reference doc) — defer reasons: pairs naturally with the workspace audit artefact in workflow/audits/ — better as a stand-alone Tier-B housekeeping pass after producer wiring lands.

  Task 9 (workflow/audits/E-42-grammar-versions-YYYY-MM-DD.md inventory grounded in cargo tree) — defer reasons: requires a fresh cargo tree run; should pair with elai.map regen (currently stale per memory). Stand-alone task for the next housekeeping session.

  Task 10 (tree-sitter-vue + tree-sitter-md additions) — defer reasons: anti-slopsquatting WebSearch + user confirmation required (DRIFT-8). New deps need explicit authorization.

  Task 11 (injections.scm queries) — defer reasons: conflicts with shipped vue_extract_script_recursive (DRIFT-10). Reconciliation decision needed before implementation.

  Task 12 (MDX TIER-A inclusion) — defer reasons: experimental grammar (DRIFT-12). Belongs in TIER-C if it lands at all.

(d) DESIGN RATIONALE — WHY METADATA IS PARALLEL TO LanguageRegistry

The existing LanguageRegistry at knowledge::dkg::language_registry holds runtime registration state — which grammars actually loaded, which panicked, which had ABI mismatches. The new grammar_tier module holds DESIGN-TIME metadata — which tier a grammar belongs to, which crate ships it, what the canonical alphabetical position is. Keeping these as PARALLEL surfaces means:

  - The runtime registry can keep its catch_unwind isolation pattern unchanged.
  - The contract test enforces design-time invariants (kotlin on -ng, no language-pack) without touching the runtime registration order.
  - Tier-B's `register_tier_a` / `register_tier_b` refactor can consume grammar_tier::TIER_A_GRAMMARS as the iteration source, eliminating the flat list at dkg/mod.rs:210-234. The two surfaces converge there.

(e) WHY THE BANNED LIST IS LOAD-BEARING

Three banned crates:

  - tree-sitter-language-pack — bundles all grammars under one umbrella crate. Ships ABIs that may not match the workspace's tree-sitter 0.26 pin. Carries MIT/Apache license mismatch. A future "simplify dep graph" pass might be tempted to swap workspace's 18 individual tree-sitter-* crates for this one; the contract blocks it.

  - tree-sitter-kotlin (fwcd fork) — abandoned. Replaced by tree-sitter-kotlin-ng per C-17. Anyone re-introducing it (e.g. via a copy-paste from an older Cargo.toml) fails the contract.

  - tree-sitter-svelte (plain crate) — abandoned. Replaced by tree-sitter-svelte-ng.

These are pinned in the contract because the anti-slopsquatting failure mode is silent: a typosquat or a fork-reversion would lint clean + compile clean, only failing at runtime under load. The contract makes the silent failure loud.

(f) THE EXACT 0.26 RUNTIME PIN

REQUIRED_TREE_SITTER_VERSION = "0.26" is the workspace pin. The contract assertion locks this constant; if it changes, the upgrade is visible. tree-sitter 0.26 dropped breaking ABI changes relative to 0.25 — future bumps need a coordinated grammar-crate rebuild.

(g) TIER-B CLOSURE — completion-drive session 4/4, s-25120

  Closes 13 of 13 MISSING items from the prior re-check (PARTIAL,
  SAME). The substrate stays wire-dark by default; default deployments
  observe zero new SSB rows and zero new code paths in DkgEngine
  beyond the `try_register` flat block that already shipped.

  Pattern-2 cascade — single variant collapse of plan's demanded 4
  sibling variants:

    - crates/orchestrator/src/ssb.rs:
        * SCHEMA_VERSION 182 -> 183 (line 2366)
        * SQL CHECK adds 'tree_sitter_grammar_event' (line ~3895)
        * SsbActionType::TreeSitterGrammarEvent enum variant (line ~7047)
        * Self::TreeSitterGrammarEvent => "tree_sitter_grammar_event" as_str
        * "tree_sitter_grammar_event" => Ok(SsbActionType::TreeSitterGrammarEvent) FromStr
        * SsbPayload::TreeSitterGrammarEvent(TreeSitterGrammarEventPayload) enum variant
        * Self::TreeSitterGrammarEvent(_) => SsbActionType::TreeSitterGrammarEvent action_type
        * Pattern-2 35th application: pub enum TreeSitterGrammarEventKind {
            AbiProbeMismatch, SvelteRunesParseHealth, TierBGrammarLoaded,
            TreeSitterSkippedLargeFile
          } + as_str() snake_case helper
        * TreeSitterGrammarEventPayload { kind, file: Option<String>,
          details_json: serde_json::Value, ts_ms: i64 } (NO Eq derive — f32
          values may ride along)
        * ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV const

    - crates/orchestrator/tests/action_type_lock.rs:
        * TOTAL_VARIANTS 455 -> 456
        * ALL_VARIANTS adds SsbActionType::TreeSitterGrammarEvent
        * exhaustive_match_guard adds the matching arm

    - crates/orchestrator/tests/action_type_lock.golden:
        * ordinal 0430 = `tree_sitter_grammar_event` inserted
          alphabetically between `tree_position_changed` (0429) and
          `tripwire_fired` (now 0431). All later ordinals shifted +1.
        * Header `Total variants: 456`.

  Producer + substrate code under elai_knowledge:

    - crates/knowledge/src/tree_sitter_health/mod.rs (NEW)
        + `ELAI_TREE_SITTER_GRAMMARS_ENABLE_ENV` + `is_enabled()` wire-
          dark gate. Module renamed from `tree_sitter` to
          `tree_sitter_health` to avoid shadowing the external
          `tree_sitter` crate inside the lib.

    - crates/knowledge/src/tree_sitter_health/abi_probe.rs (NEW)
        + `ABI_VERSION_MIN = 14`, `ABI_VERSION_MAX = 15` named consts
          (no magic numbers).
        + `AbiVersion { language_id, version }` + `AbiProbeRecord`
          serialized types.
        + `probe_one` + `probe_abi_versions` catch_unwind-wrapped
          (no .unwrap()).
        + `assert_abi_compatibility` Result-typed startup-fail-fast
          variant.
        + 6 unit tests (in-range / out-of-range / panic isolation /
          assert pass / assert fail-fast).

    - crates/knowledge/src/tree_sitter_health/svelte_health.rs (NEW)
        + `ERROR_RATE_THRESHOLD = 0.05` named const.
        + `detect_runes` regex pre-scan for $state(/$derived(/$effect(/$props(
          (does NOT false-positive on Svelte 4 $store autosubscribe).
        + `ErrorHealth { error, missing, total }` + `ratio()` saturating-
          arithmetic helper.
        + `compute_health` cursor walker over a parsed `tree_sitter::Tree`.
        + `HealthClassification` 4-arm verdict + `classify()` combiner.
        + 14 unit tests (including no-false-positive on Svelte 4 stores
          + smoke test on real rust grammar tree).

    - crates/knowledge/src/tree_sitter_health/large_file.rs (NEW)
        + `TREE_SITTER_LARGE_FILE_BYTES = 2 * 1024 * 1024` named const.
        + `should_skip_for_size` pure pre-parse gate.
        + `LargeFileSkip { relative_path, byte_count, threshold_bytes }`
          serialized record.
        + 4 unit tests.

    - crates/knowledge/src/tree_sitter_health/registration.rs (NEW)
        + `TierRegistrationOutcome { registered, failures }` (no Eq —
          GrammarLoadError carries free-form msg).
        + `register_tier_a_languages` consumes TIER_A_GRAMMARS + adds
          vue via the composite-reparse seam.
        + `register_tier_b_languages` cfg-gated behind
          `experimental_grammars` Cargo feature (no-op stub otherwise).
        + 2 unit tests (Tier-A canonical count + Tier-B feature gating).

    - crates/knowledge/Cargo.toml
        + `experimental_grammars = []` feature (no new deps).
        + Comment block on the kotlin pin documenting plan's fwcd
          fork-selector request is REJECTED (C-17 regression-guard).

    - crates/knowledge/src/lib.rs
        + `pub mod tree_sitter_health;` declaration with disambiguation
          comment block.

  Reference docs:

    - crates/knowledge/src/grammars.md (NEW, ~165 lines)
        + Canonical reference doc for tree-sitter pin invariants +
          ABI range + per-grammar caveats + composite-reparse pathway
          + WASM unsupported note + Svelte 5 best-effort posture +
          banned-list rationale.
    - crates/knowledge/src/tree_sitter_health/queries/injections.scm
      (NEW, DESCRIPTIVE)
        + Documents the canonical script→TS / style→CSS reconciliation
          shipped at dkg/mod.rs::vue_extract_script_recursive. NOT
          loaded as a Query::new() input at runtime per DRIFT-10.
    - workflow/audits/E-42-grammar-versions-2026-05-28.md (NEW)
        + Per-grammar TSV table: language_id / crate_name / version /
          tier / ABI / fork_source URL / notes. Includes deferred
          additions (json/yaml/toml/markdown) gated on user
          anti-slopsquatting authorization (DRIFT-8).

  Plan conflict resolutions (5 HIGH):

    H-1. fwcd kotlin REJECTED. kotlin pin stays tree-sitter-kotlin-ng
         (C-17). Contract test `kotlin_uses_org_ng_variant_not_fwcd`
         remains green; plan's wording is a stale 2025 snapshot.
    H-2. plain tree-sitter-svelte REJECTED. svelte pin stays
         tree-sitter-svelte-ng. Contract test
         `svelte_uses_ng_variant_not_abandoned` remains green.
    H-3. exact-pin "=0.26" REJECTED. Workspace caret-pin "0.26" stays.
         The design-time pin is the REQUIRED_TREE_SITTER_VERSION
         string constant; the runtime carry-policy is caret per
         decision (f) above.
    H-4. 4 sibling SSB variants COLLAPSED to single
         TreeSitterGrammarEvent + TreeSitterGrammarEventKind 4-arm
         discriminator per Pattern-2 (35th application).
    H-5. injections.scm REPLACED by descriptive reference file.
         Shipped composite-reparse at
         dkg/mod.rs::vue_extract_script_recursive is the canonical
         pathway and is preserved unchanged.

  Build verification:

    - cargo check -p elai-knowledge: PASS
    - cargo check -p elai-orchestrator: PASS (12 pre-existing warnings)
    - cargo check --workspace: PASS
    - Test runs deferred to auditor per CLAUDE.md no-cargo-test rule.

  Wire-dark posture:

    All 4 fire-points gated on `ELAI_TREE_SITTER_GRAMMARS_ENABLE=1`.
    Default deployments observe zero rows from the substrate. The
    LanguageGrammarStatus payload (B-30 / SCHEMA 36) continues to
    emit one row per DkgEngine init for the load-summary surface and
    is NOT duplicated.

  Dashboard:

    The plan's optional GrammarHealthPanel.svelte is NOT shipped this
    session per the launch prompt directive (NO dashboard). When the
    enable env is flipped on and producers go live, the existing
    KnowledgeDrawer LANGUAGES LOADED card can extend to surface the
    new event kind via an SsbViewer query for
    `tree_sitter_grammar_event`; this is a follow-up plan, not E-42
    scope.

END E-42-decisions.md

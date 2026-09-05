# I-51: unified model capability catalog — one read surface for context window, pricing, thinking-support, and VRAM tier per model

- Track 20 | lever SMARTER-ORCH | grounding STRONG (SL-4) | nature EXTENSION (consolidation) | effort
  S-M | WAVE 1

- Premise: STEAL-LIST A1 ("steal the single-catalog shape. Low effort."). ELAI's model metadata is
  scattered and partially DUPLICATED: providers/src/pricing.rs, providers/src/vram_tier.rs,
  core/src/vram_tier.rs (same filename, two crates), observability/pricing_rollup.rs, plus per-adapter
  defaults (SL-4). Every consumer (router, cost gate, tier matrix, cache-economics planner) re-derives or
  hard-codes. gajae's api-registry.ts + models.json + model-manager.ts shape: ONE catalog keyed by model
  id carrying {context_window, pricing in/out/cache, thinking support + effort mapping, max_tokens
  defaults, vram tier} — every consumer reads the catalog.

## Steal/build
- Steal the catalog shape (SL-10 binding read: api-registry.ts + models.json + model-manager.ts before
  EXECUTING).
- Build: crates/providers/src/model_catalog.rs + a models.toml data file (repo artifact, R8-frozen for
  gate-relevant fields, diffable) — the existing pricing.rs / vram_tier.rs structs become VIEWS over the
  catalog (their public APIs preserved so call sites keep compiling, their data sourced from the one
  file); a staleness stamp per entry (prices drift — carried-landmine 4's "re-verify pricing at edit"
  becomes a dated field).
- SLC-2 BINDING: the per-VRAM tier logic (8/12/16/24/48GB plan) is WRAPPED, not deleted — the catalog is
  the data source; vram_tier semantics stay. Any removal needs explicit user confirmation.
- EXCLUDED: no live price-fetching (the catalog is a reviewed repo artifact, not a network call); no
  router changes (I-25's job — it gains a clean input); no model ADDITIONS/removals.

## Existing ground truth
- SL-4: pricing.rs + vram_tier.rs x2 + pricing_rollup.rs confirmed by glob 2026-06-10. The core/providers
  vram_tier duplication is itself a drift hazard the catalog retires.
- Consumers (executor re-greps each — W-7): E-39 cost gate, E-31 tier matrix, H-9 cost cockpit, I-10
  cache-economics planner (PLANNED), I-25 router (PLANNED).

## REVISED scope (W-10 tiered)
- SHIP NOW: catalog + models.toml + the two highest-traffic views (pricing.rs, vram_tier.rs) re-sourced +
  a drift contract test (the two vram_tier modules must agree byte-for-byte via the catalog).
- DEFER: migrating every per-adapter default (follow-on patch-N work once the catalog is proven).

## Integration
- crates/providers/src/model_catalog.rs + models.toml; pricing.rs / vram_tier.rs re-sourced in place.
  frontend_pair: N/A (data consolidation behind preserved APIs; W-6 justified — existing cost/tier
  surfaces unchanged).

## WIRING_CONTRACT
- produces_symbols: providers::model_catalog + models.toml + the duplication-drift contract test.
- consumes_symbols: providers::pricing, providers::vram_tier, core::vram_tier (re-sourced views — public
  APIs preserved), pricing_rollup (reads through the view).
- reachable_from: every production cost computation (E-39 gate path) and tier selection (vram tier path)
  now resolves through the catalog — proven by the preserved call sites compiling against the views.
- stub_ban: NO deletion of vram_tier semantics (SLC-2); NO hard-coded price constant outside models.toml
  after migration (contract grep); NO network fetch; NO API breakage on the preserved views; NO
  unwrap/expect on prod paths.
- observable_effect:
  trigger: "a real run computing cost + tier (any cloud dispatch with E-39 active)"
  expect_ssb: "NO new action_type — existing cost/route rows carry values now sourced from the catalog
  (GD-1)"
  expect_artifact: "cost + tier outputs byte-identical pre/post consolidation on a fixture set (the
  consolidation is provably behavior-neutral); the drift contract test FAILS when a seeded models.toml
  edit diverges a view (R2); a price constant grep outside models.toml in the migrated modules returns 0."

## Grounding
- STEAL-LIST A1 (Tier 2, partial); SL-4, SL-10, SLC-2; carried landmine 4 (pricing constants are
  defaults to re-verify, now dated in-file).

## COMPOSES-WITH
- E-39 cost gate + E-31 tier matrix + H-9 cockpit (consumers, unchanged APIs), I-10 economics planner +
  I-25 router (future consumers — this is their clean input), B-37 adapters (defaults, deferred leg).
  Restates none. SSB: NO new action_type (GD-1).

End of I-51.

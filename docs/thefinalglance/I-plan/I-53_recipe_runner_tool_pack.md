# I-53: recipe runners + utility tool pack — Make/Just/cargo-alias recipes as first-class tools, plus the two cheap utility cherry-picks

- Track 20 | lever TOOLS | grounding STRONG (gap: SL-9) / MEDIUM (gajae semantics, SL-10) | nature BUILD |
  effort S-M | WAVE 2

- Premise: STEAL-LIST B4 ("cheap, useful") + the two worthwhile B9 cherry-picks. A repo's Justfile/
  Makefile/cargo aliases/taskfile are its OWNER-SANCTIONED entrypoints — exactly the "real entrypoint"
  doctrine the wiring rules push. Today an ELAI agent shells arbitrary bash to run them (unvalidated
  shape, no discovery). A recipe tool DISCOVERS the recipes (list arm) and RUNS them as named, journaled,
  argv-validated invocations. Plus two utility tools agents currently fake with risky bash: archive
  reader (inspect zip/tar without extraction side effects) and JSON tree explorer (navigate large JSON
  without flooding context — the token-discipline tool).

## Steal/build
- Steal the tool shapes (SL-10 binding read: gajae packages/coding-agent/src/tools/recipe/ +
  archive-reader.ts + json-tree.ts before EXECUTING).
- Build, in crates/tools/src: (a) recipe.rs — discover (parse Justfile/Makefile targets + cargo aliases;
  list with descriptions) and run (the named recipe through the EXISTING bash validation + E-30 argv
  path — the tool builds the command, the existing gates judge it); (b) archive_reader.rs — list/read
  entries from zip/tar.gz READ-ONLY (no extraction to disk); (c) json_tree.rs — path-query + bounded
  subtree render over a JSON file (depth/size capped output).
- EXCLUDED B9 items (recorded): cron / monitor / background-job tools — ELAI's orchestrator + Layer-1A
  schedules own those concerns; a per-agent cron tool fights the supervisor model. Declined, not
  deferred.
- EXCLUDED: no new archive-format deps without the anti-slopsquat protocol + user approval (zip/tar via
  already-present workspace deps if available — executor verifies the dep tree first; hand-roll tar
  listing if not).

## Existing ground truth
- SL-9: no recipe/just/make runner in crates/tools/src (grep 2026-06-10). The tools crate is lean by
  design (STEAL-LIST: "ELAI crates/tools/src is lean") — these three earn their place by replacing
  RISKIER bash equivalents, not by width for its own sake.

## REVISED scope (W-10 tiered)
- SHIP NOW: all three tools, catalog-registered, through existing validation gates.
- DEFER: taskfile.yml + npm-scripts discovery (add when a target repo needs them).

## Integration
- crates/tools/src/{recipe.rs, archive_reader.rs, json_tree.rs} + tool catalog registration (3-arm
  pattern where arms exist); recipe run path feeds bash_validation + E-30 (and I-46's interceptor once
  shipped — a redirect TARGET, per I-46's deferred leg). frontend_pair: tool rows in the existing
  tool-call drawer (no new component).

## WIRING_CONTRACT
- produces_symbols: tools::recipe (list/run), tools::archive_reader (list/read), tools::json_tree
  (query/render) + catalog registrations.
- consumes_symbols: tools::bash_validation + E-30 argv allowlist (the run path's gates — composed, never
  bypassed), the tool registry.
- reachable_from: the production agent tool dispatch (live agents calling list/run on a real repo).
  Contracts-only callers do not satisfy (R7).
- stub_ban: NO gate bypass on recipe run (the built command goes through the same validation as raw
  bash); NO extraction-to-disk in archive_reader (read-only streams); NO unbounded json_tree output
  (hard caps, truncation marked); NO new dep without anti-slopsquat + approval; NO unwrap/expect on prod
  paths.
- observable_effect:
  trigger: "a live agent lists recipes on a fixture repo with a Justfile + runs one; reads one entry from
  a fixture tar.gz; queries a 5MB JSON fixture"
  expect_ssb: "NO new action_type — existing tool_call rows carry the three tool ids (GD-1)"
  expect_artifact: "recipe list names the Justfile targets; the run's journaled command passed
  bash_validation (verdict row present); archive read returns entry bytes with ZERO files written to disk
  (fixture dir diff is empty — the negative arm); json_tree output is under the cap with the truncation
  marker on the oversized arm. A recipe run with no validation verdict row is the broken arm and FAILS (R1)."

## Grounding
- STEAL-LIST B4 + B9 (Tier 2 / cherry-picks); SL-9, SL-10.
- CLAUDE.md token-discipline (json_tree exists to keep large JSON out of context).

## COMPOSES-WITH
- bash_validation + E-30 (the gates), I-46 interceptor (future redirect target), the tool catalog
  (E-35 pattern), H-4 tool search (discoverability). Restates none. SSB: NO new action_type (GD-1).

End of I-53.

E-42 tree-sitter grammar deps — verification (offline batch s-25120, 2026-05-29)

Scope: verify tree-sitter-{json,yaml,toml,md,vue} + tree-sitter-svelte-ng on crates.io for the E-42 grammar pack.

Source: crates.io REST API /api/v1/crates/{name} fetched via curl + user-agent elai-anti-slopsquat/1.0.

tree-sitter-json
  max_version / max_stable: 0.24.8
  recent_downloads: 1,273,885
  last_release_at: 2024-11-11
  repository: github.com/tree-sitter/tree-sitter-json
  Status: GROUNDED — official tree-sitter org, 1.27M recent downloads. Last release ~6 months old but tree-sitter grammars are slow-moving by design (JSON spec is stable).
  Recommend pin: tree-sitter-json = "0.24"

tree-sitter-yaml
  max_version / max_stable: 0.7.2
  recent_downloads: 1,155,905
  last_release_at: 2025-10-07
  repository: github.com/tree-sitter-grammars/tree-sitter-yaml
  Status: GROUNDED — moved to tree-sitter-grammars community org (the official tree-sitter org has been spinning satellite grammars out into tree-sitter-grammars; this is normal). 1.15M recent downloads.
  Recommend pin: tree-sitter-yaml = "0.7"

tree-sitter-toml
  max_version / max_stable: 0.20.0
  recent_downloads: 20,852
  last_release_at: 2022-01-05
  repository: github.com/Mathspy/tree-sitter-toml
  Status: AMBER — last release 4+ years ago, only 20.8K recent downloads. Owner Mathspy is a community publisher, not tree-sitter org. The community moved on; the active TOML grammar is now under tree-sitter-grammars/tree-sitter-toml-ng (not on crates.io yet) or tools use stry47/tree-sitter-toml (different fork).
  Recommend: VERIFY before adding. Consider tree-sitter-toml-ng github repo + manual vendoring.

tree-sitter-md
  max_version / max_stable: 0.5.3
  recent_downloads: 231,824
  last_release_at: 2026-02-26
  repository: github.com/tree-sitter-grammars/tree-sitter-markdown
  Status: GROUNDED — recently released (Feb 2026), 231K recent downloads, tree-sitter-grammars org. Note crate-name is `tree-sitter-md` but repo is `tree-sitter-markdown`; that is the published canonical name.
  Recommend pin: tree-sitter-md = "0.5"

tree-sitter-vue
  max_version / max_stable: 0.0.3
  recent_downloads: 700
  last_release_at: 2022-09-24
  repository: github.com/xiaoxin-sky/tree-sitter-vue
  Status: AMBER-RED — version 0.0.3 (pre-1.0 cosmetic), 700 recent downloads, last release 3+ years ago, single-individual owner (xiaoxin-sky), no tree-sitter org affiliation. Slopsquat risk MEDIUM.
  Recommend: BLOCK pending verification. Confirm with user before adding — the canonical Vue grammar may live under ikatyang/tree-sitter-vue or be community-vendored.

tree-sitter-svelte-ng
  max_version / max_stable: 1.0.2
  recent_downloads: 45,791
  last_release_at: 2024-09-08
  repository: github.com/tree-sitter-grammars/tree-sitter-svelte (NOTE: repo name does not include "-ng" suffix)
  Status: GROUNDED — tree-sitter-grammars community org, 1.0.x stable release, 45K recent downloads. The "-ng" suffix in the crate name denotes the Svelte 5 runes-compatible fork; the older `tree-sitter-svelte` 0.x crate is deprecated.
  Recommend pin: tree-sitter-svelte-ng = "1.0"
  C-17 alignment: C-17 (per CLAUDE.md pointer) pins this grammar. No upstream break observed — 1.0.2 is the current stable and ELAI's pin is intact.

Cross-check: all six grammars in scope-search live under either tree-sitter/ org or tree-sitter-grammars/ org EXCEPT toml (Mathspy) and vue (xiaoxin-sky). Those two are the anti-slopsquatting hotspots. Recommend the next E-42 implementation pass vendors them rather than pulling from crates.io.

OPEN QUESTIONS:
  1. tree-sitter-toml: community migrated to tree-sitter-toml-ng; needs WebFetch on github.com/tree-sitter-grammars/tree-sitter-toml-ng to confirm canonical home.
  2. tree-sitter-vue: confirm canonical maintainer with the user before adding.

DECISION (offline-batch verdict, NOT a Cargo.toml mutation):
  GROUNDED (4 of 6): tree-sitter-json, tree-sitter-yaml, tree-sitter-md, tree-sitter-svelte-ng. Safe to pin per recommendations above.
  AMBER (1): tree-sitter-toml. Stale + community ownership change. Defer.
  RED (1): tree-sitter-vue. Single-individual + 3-year stale. Block.

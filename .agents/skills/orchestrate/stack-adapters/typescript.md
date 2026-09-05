STACK ADAPTER — typescript (incl React Native / Expo / Node / Svelte / Vue)

Loaded by orchestrate when detect-stack.ps1 reports stack="typescript", "rn-expo", "svelte", or composite stacks starting with "ts".

Tools required:
- node (>=20 per anti-stale rules)
- npm (or pnpm / bun if project uses)
- typescript >= 6.x (from project devDeps)
- dependency-cruiser (workspace root has it at v17.4.0+, see CLAUDE.md codebase-map rule)
- knip (https://knip.dev) — unused-export + unused-files + unused-deps detection (preferred over ts-prune which is unmaintained in 2026)
- eslint (project devDeps)
- jest OR vitest (project test runner)

Install commands (orchestrate-launcher.ps1 surfaces if missing):
  npm install --save-dev typescript dependency-cruiser knip eslint
  # OR if pnpm/bun, equivalents

THE 6 GENERIC GATES, TYPESCRIPT FLAVOR

GATE 1 — BUILD GREEN
  npx tsc --noEmit
  Expected: exit 0
  For Expo / RN: also npx expo prebuild --check (only if app.config touched)
  For Svelte: npx svelte-check --threshold error

GATE 2 — TEST SUITE GREEN + DELTA-COVERED
  npm test  (or jest / vitest / etc per project's "test" script)
  Expected: exit 0
  Delta coverage: for each new public export in WIRING_CONTRACT.produces_symbols or in plan "Files touched" CREATE entries:
    rg "<symbol_short_name>" --type ts --type tsx test/ tests/ src/**/*.test.ts src/**/*.spec.ts
  Expected: at least 1 match per symbol

GATE 3 — DEP-GRAPH INTEGRITY
  npx depcruise src --validate --config .dependency-cruiser.cjs --output-type err
  Expected: exit 0 (forbidden-rule violations including no-circular + no-orphans cause non-zero)
  If project has the codebase map from apps_workflow's codebase-map rule, the regenerated map already runs this

GATE 4 — PUBLIC-API SURFACE DIFF (TypeScript variant)
  No native tool equivalent to cargo-public-api for TypeScript. Use knip + git-diff:
  npx knip --reporter json > workflow/sessions/<sid>/knip-current.json
  git stash; npx knip --reporter json > workflow/sessions/<sid>/knip-baseline.json; git stash pop
  Diff: new exports in current.json vs baseline.json. For each new export, verify it appears in:
    a) plan's WIRING_CONTRACT.produces_symbols OR
    b) at least one importer in src/ (knip's "unused exports" list would flag it if not)
  Knip itself flags "unused exports" as part of normal output; treat each unused export as gate failure UNLESS plan registered it as intentional new surface

GATE 5 — MANIFEST + LOCKFILE CONSISTENCY
  npm ci --dry-run  (or pnpm install --frozen-lockfile --dry-run)
  Expected: exit 0 (proves package.json matches lockfile)
  npx knip --reporter compact  (knip also reports unused dependencies)
  Expected: zero "Unused dependencies" lines OR each unused dep is in compat.knip_allowlist
  npx depcruise src --output-type err --config .dependency-cruiser.cjs  (the no-non-package-json forbidden rule catches phantom imports)
  Expected: exit 0

GATE 6 — DOCUMENTATION DRIFT
  For each new public symbol from gate 4 diff:
    rg "<symbol_short_name>" workflow/memory-bank/**/changelog.md workflow/memory-bank/**/systemPatterns.md workflow/memory-bank/**/projectIndex.md
  Expected: at least 1 match per symbol

TS-SPECIFIC EXTRA GATES

GATE TS1 — STUB BAN
  rg "TODO|FIXME|XXX|HACK" src --type ts --type tsx --glob "!**/*.test.ts" --glob "!**/*.spec.ts"
  Expected: every match references a registered stub id (TODO[X.Y.Z]) OR is in compat.stub_allowlist (very rare)
  rg "any\s*\)" src --type ts --type tsx  (or use eslint rule no-explicit-any)
  Expected: zero matches unless plan explicitly justified

GATE TS2 — ESLINT CLEAN
  npx eslint src --max-warnings 0
  Expected: exit 0

GATE TS3 — REACT 19 PATTERN COMPLIANCE (Expo / RN projects per coding-standards-expo.md)
  rg "forwardRef|defaultProps" src --type tsx
  Expected: zero matches (forwardRef banned in new code, defaultProps removed in React 19 function components)
  rg "<Context\.Provider" src --type tsx
  Expected: zero matches (use short `<Context value={...}>` form)
  rg "\.then\(" src --type ts --type tsx --glob "!**/*.test.*"
  Expected: zero (use async/await per coding-standards-expo.md)

GATE TS4 — HERMES CONSTRAINTS (RN / Expo Hermes targets)
  rg "^await\s" src --type ts --type tsx  (top-level await detection — Hermes parser error)
  Expected: zero matches at module scope
  rg "(\(\?<\w+>|\\\\p\{)" src --type ts --type tsx  (named capture groups + Unicode property escapes — Hermes regex gaps)
  Expected: zero matches OR each match is inside a string that never executes on Hermes

GATE TS5 — METRO PATH-ALIAS CONSISTENCY
  Check tsconfig.json compilerOptions.paths against src imports:
    npx tsc --traceResolution --noEmit > resolution-trace.txt
  Look for "module not found" or path-alias resolution failures.


PER-SUB-PHASE SCOPING

TypeScript projects often have one root package.json (unlike Rust workspaces). Per-sub-phase scoping:
  - tsc runs at project root (the noEmit check is fast enough)
  - tests scope to the touched test files: npm test -- --testPathPattern "<file-pattern>"
  - dep-cruise scopes to the touched src/ subtree: npx depcruise src/<subdir> ...
  - knip + eslint scope similarly via --include flags

For monorepos (turborepo / nx / workspaces): scope to the touched packages via package manager workspace flags.


COMMAND TEMPLATES (orchestrate-launcher.ps1 fills $PROJECT_ROOT and $TOUCHED_FILES)

per-sub-phase:
  Push-Location $PROJECT_ROOT
  npx tsc --noEmit; if ($LASTEXITCODE -ne 0) { exit 1 }
  $testPattern = ($TOUCHED_FILES -split ',' | ForEach-Object { ($_ -replace '\\','/') -replace '\.tsx?$','' }) -join '|'
  npm test -- --testPathPattern "($testPattern)"; if ($LASTEXITCODE -ne 0) { exit 2 }
  npx depcruise src --validate --config .dependency-cruiser.cjs --output-type err; if ($LASTEXITCODE -ne 0) { exit 3 }
  Pop-Location

full sweep:
  Push-Location $PROJECT_ROOT
  npx tsc --noEmit; if ($LASTEXITCODE -ne 0) { exit 1 }
  npm test; if ($LASTEXITCODE -ne 0) { exit 2 }
  npx depcruise src --validate --output-type err; if ($LASTEXITCODE -ne 0) { exit 3 }
  npx knip --reporter compact > $sweepdir/knip.txt; $knip_exit = $LASTEXITCODE
  npm ci --dry-run; if ($LASTEXITCODE -ne 0) { exit 5 }
  npx eslint src --max-warnings 0; if ($LASTEXITCODE -ne 0) { exit 6 }
  Pop-Location

  # stub-ban grep
  $todo_hits = Get-ChildItem $PROJECT_ROOT\src -Recurse -Include *.ts,*.tsx |
    Select-String -Pattern 'TODO|FIXME' -CaseSensitive |
    Where-Object { $_.Line -notmatch 'TODO\[[A-Z][0-9]+\.[A-Z]' }
  if ($todo_hits) { Write-Host "unregistered TODOs:`n$todo_hits"; exit 8 }


END stack-adapters/typescript.md

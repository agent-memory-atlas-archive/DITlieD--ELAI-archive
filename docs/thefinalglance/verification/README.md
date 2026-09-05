verification/ — per-file integration/wiring audit
================================================

This folder contains:

- reports/B-plan/<file>.report.md       one report per file in docs/thefinalglance/B-plan/
- reports/guide/<file>.report.md         one report per top-level design doc in docs/specs/guide/
- reports/steps/<file>.report.md         one report per phase step in docs/specs/guide/steps/
- INDEX.md                               aggregated punch list of everything that must still be done

Each report is produced by a verification agent that:
1. Reads the source file
2. Enumerates concrete claims (files, symbols, wiring, tests, acceptance)
3. Verifies each claim against the real codebase under .elai_cc/
4. Classifies DONE / PARTIAL / MISSING / UNVERIFIABLE with path:line evidence

INDEX.md aggregates the "What must be done" sections across every report.

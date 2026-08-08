# K-13 decisions

- The plan paths under `docs/` and `workflow/reasoning/` are remapped into the landable `.elai_cc/` package.
- The OWASP mapping lives at `.elai_cc/docs/strategy/ELAI-OWASP-CONTROL-MAPPING.md`.
- The injection row is added to the K-1 dossier at `.elai_cc/docs/strategy/ELAI-PROBLEM-QUANTIFICATION-DOSSIER.md`, keeping one number ledger.
- The claims lint is the K-1 tool extended in place; no fork or new dependency is introduced.
- The row remains `[BENCH-PENDING]`; no injection percentage or delta is fabricated.
- G-6 SAFEDELTA AgentDojo is the future recompute source (`elai bench safedelta` / `safedelta_runner.rs`).
- No WI-1, WI-2, WI-3 injection hardening is authored here.
- No WI-6 config-admission hardening is authored here.
- K-13 COMPOSES K-6 by supplying a future procurement-clause pointer; K-6 itself is untouched.
- K-13 is documentation and deterministic linting only: no runtime code and no SSB.

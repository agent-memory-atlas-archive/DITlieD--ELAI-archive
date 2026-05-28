"""elai_optimizer: GEPA-class self-evolving prompts service for ELAI.

E-8 ship. Cloud-Sonnet reflection LM + local-Qwen target split per
arXiv:2405.10276 (Zhang et al.) -- small LLMs are INEFFECTIVE as the
OPRO optimizer; the CLI surface (`elai optimize template`) hard-
refuses local 7B as the reflection LM.

Primary algorithm: GEPA reflective prompt evolution (arXiv:2507.19457
Agrawal et al. ICLR 2026 Oral). Supporting: DSPy MIPROv2
(arXiv:2310.03714) + OPRO meta-prompt trajectory (arXiv:2309.03409) +
APE Monte-Carlo search (arXiv:2211.01910). LPO span-level freeze
(arXiv:2504.20355) enforced Rust-side at
`orchestrator::baby_step_spec_lpo`.

PyPI deps (anti-slopsquatting VERIFIED -- see
`workflow/reasoning/E-8-decisions.md` DECISION-5):
  * gepa>=0.1.1,<0.2 (MIT, maintainer Lakshya A Agrawal, 2026-03-16)
  * orjson (stdlib-json banned per CLAUDE.md anti-stale-Python rule)

Live cloud-Sonnet + local-Qwen calls are RUNPOD-PENDING: the
top-level optimize_template() function is a scaffold that builds the
OptimizedTemplate envelope from input fixtures + a deterministic mock
loop. Replace the body inside `_run_gepa_loop` with the real
`gepa.optimize()` call when the live-LLM seam opens.
"""

__version__ = "0.1.0"

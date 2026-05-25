//! D-28: orchestrator-side farm runners (adversarial example farming).
//!
//! Sibling of `crate::benchmarks` — distinguishes runners that PRODUCE
//! training data (`farm/*`) from those that EVALUATE existing models
//! (`benchmarks/*`). The first member is the Adv-RM
//! (arXiv:2504.06141) host-side farm-loop scaffolding; the live PPM
//! scoring loop + Bradley-Terry retrain + dual `bench_prmbench` v1/v2
//! runs are RUNPOD-PENDING (see `python/elai_inference/
//! ppm_adversarial.py` and `workflow/remote-setup/ppm_finetune.sh`).
//!
//! # Current submodules
//!
//! - `adv_rm` (D-28): adversarial-RM host-side farm runner. Consumes
//!   the D-2 PPM hook via a generic `H: SidecarPpmHook` (no surface
//!   modifications), walks a dataset, applies the paper-grounded
//!   Z>1.96 dual-filter, writes corpus.jsonl with
//!   `gold_verdict: "pending"`, and emits one
//!   `AdversarialFarmRunCompleted` SSB row at end-of-run.

pub mod adv_rm;

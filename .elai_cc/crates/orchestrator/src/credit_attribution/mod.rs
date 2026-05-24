//! C-45 — CANTANTE Contrastive Credit Attribution module.
//!
//! Implements the contrastive credit-attribution primitive from
//! Zehle 2026 ("CANTANTE: Optimizing Agentic Systems via Contrastive
//! Credit Attribution", arXiv:2605.13295). Given a task-success scoring
//! function and the set of [`elai_ipc_types::AgentRole`] variants that
//! make up ELAI's per-role agent pool, the module runs `K` rollouts per
//! task under each "drop one role" configuration and aggregates a scalar
//! credit score `c in [-1, +1]` per role (paper Section 3.2,
//! "K rollouts per query, NOT K^N").
//!
//! Scope (pilot): the contrastive ablation set is bound to the eight base
//! `AgentRole` variants — Executor, Auditor, Researcher, Designer,
//! PromptEngineer, ContextCompiler, MemoryWriter, MediaValidator. The
//! eight newer variants introduced by B-40 (IdeaFramer, IdeaCritic) and
//! B-41 (the five per-level Auditor* variants) are EXCLUDED from this
//! pilot's ablation set per the C-45 cost cap; extending to 16 requires a
//! follow-up runpod-billable pilot. See `partition.rs` for the canonical
//! `pilot_roles()` list.
//!
//! Paper grounding posture: the contrastive method itself is GROUNDED
//! against `workflow/papers/cantante-credit-attribution.md` (paper
//! interface). The `g = 3` partition default + `K`-rollouts-NOT-`K^N`
//! invariant + scalar credit in `[-1, +1]` are paper-grounded. The
//! integration into ELAI's `SkillRegistry` (the `learned_credit_score`
//! field on `SkillManifest`) is a LOCAL extension — the paper does not
//! prescribe a registry shape.
//!
//! Module layout:
//!   - [`types`]    — `DropPolicy`, `CreditScore`, `CreditReport`.
//!   - [`partition`] — group-of-`g=3` partition logic + paper invariants.
//!   - [`attributer`] — `contrastive_attribute` driver (K rollouts per
//!     dropped-role config).
//!   - [`harness`]  — `CreditAttributionHarness` accumulator that walks
//!     the eight base roles and produces a `CreditReport`.
//!   - [`integration`] — write-through helper that feeds the per-role
//!     credit into C-41's `SkillManifest.learned_credit_score`.

pub mod attributer;
pub mod harness;
pub mod integration;
pub mod partition;
pub mod types;

pub use attributer::{contrastive_attribute, AttributeError};
pub use harness::{CreditAttributionHarness, HarnessError};
pub use integration::{apply_credit_to_manifest, current_credit_weights};
pub use partition::{pilot_roles, validate_partition, PartitionError, G_DEFAULT};
pub use types::{CreditReport, CreditScore, DropPolicy};

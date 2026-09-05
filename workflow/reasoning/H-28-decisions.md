H-28 DECISIONS — current-RF reward-tampering gate + harness-in-the-frozen-set
Session: s-8404 lineage
Date: 2026-06-06
FSM: EXECUTING -> (AUDITING)

WHAT WAS BUILT
A composite reward-tampering gate (the self-evaluator-capture defense) enforcing Everitt's
current-RF principle (arXiv:1908.04734): a self-modifying optimizer must optimize the
CURRENT frozen reward function and must not be able to influence its own evaluator. Tier-1
is an always-compiled pure-Rust gate + `elai reward-check` CLI over a planted fixture
triple; +RewardTampering SSB Pattern-2 at live+1 (SCHEMA_VERSION 235->236, TOTAL_VARIANTS
512->513).

KEY DECISION 1 — Tier-1 = always-compiled CLI + fixture, NOT the gated darwin admit
ELAI's live darwinian admit path (darwin_admit_loop.rs:298 two_gate_promote) is feature-
gated behind cfg(feature="darwinian-evolve") + env ELAI_DARWINIAN_LOOP_ENABLE. A
production-default `cargo run` binary does NOT compile that admit. Wiring the reward-
tampering check ONLY to the gated admit would ship it WIRE-DARK in the default build (the
exact failure wiring-enforcement R3/R7 exists to catch). DECISION: Tier-1 is the always-
compiled `elai reward-check` CLI over a fixture triple in reward_tampering.rs (no feature
gate). The live :298 hook is Tier-2/deferred behind the feature (RunPod-pending). This
means W-3 ("fires from darwin_admit_loop two_gate_promote") is satisfied at Tier-1 by the
CLI + the always-compiled check, NOT by the gated admit. Grounding: exploration
H28-rewardtamper-integration.md:37,49 (the feature-gate HEADLINE finding).

KEY DECISION 2 — two SOUND Tier-1 legs, RF vs RF-INPUT tampering (Everitt taxonomy)
Everitt names two reward-tampering subtypes (paper interface :9-12): RF TAMPERING (the
agent edits the reward fn itself, Section 3) and RF-INPUT TAMPERING (the hash is intact but
the agent games the inputs, Section 4). The gate has one leg per subtype:
  LEG 3a (EvaluatorModified, deterministic HARD-BLOCK): the observed frozen-evaluator
    blake3 hash != the recorded baseline => the agent edited/monkeypatched the evaluator.
    REUSES the shipped ArchiveVerifyResult blake3-mismatch tripwire semantics (ssb.rs:6767,
    NOT :30840 — the plan/grounding-map line was stale; corrected per H-28-grounding.md
    MISMATCH). This is the load-bearing leg.
  LEG 3b (ImplausibleRewardDelta, FLAG-to-Layer-3): the hash is INTACT but the claimed
    reward jumped beyond a LOCKED plausibility band vs the prior reward => the agent gamed
    the RF inputs. This FLAGS and routes to a human (Layer-3), NOT a hard auto-block,
    because a legitimate large improvement is possible (the band too-tight blocks a real
    gain, too-loose misses gaming — fail-to-human, not fail-to-silent-reject; plan
    Open-Questions :169-171).

KEY DECISION 3 — TD-3 ordering is structural (early return), not a flag
Leg 3a is checked FIRST and SHORT-CIRCUITS (early return) before leg 3b can be reached. A
deterministic hash mismatch is dispositive: a probabilistic 3b green can NEVER override a
3a Block. Encoded as an early return on hash mismatch in classify_reward_tampering, NOT a
mutable flag the optimizer could flip. Contract test (4) + unit test
td3_hash_mismatch_short_circuits_implausible_delta assert the BOTH-true case resolves to
EvaluatorModified.

KEY DECISION 4 — 3-arm Kind (NoTampering / EvaluatorModified / ImplausibleRewardDelta),
NOT the context-package's 4-arm-with-StructuralSelfModRefused
The executor prompt SSB-LOCKSTEP line pins EXACTLY three inner Kinds. StructuralSelfModRefused
(TD-5, the "structural self-mod -> human Layer-3, undecidable per Rice/Schmidhuber"
residue) is documented as HONEST RESIDUE below, NOT a Tier-1 enforced arm — wiring it would
require the gated admit's AllowedConfigKey/is_security_sensitive structural classifier,
which is feature-coupled. The PASS arm (NoTampering) replaces it (H-27 precedent: 1 PASS +
N non-PASS).

KEY DECISION 5 — fail-closed maps to EvaluatorModified (R1/R2)
A missing/empty/un-readable frozen baseline hash, a missing observed hash, or a non-finite
reward/band all map to EvaluatorModified Block: cannot prove no-tampering => fail closed
("not found = never ran = FAIL"). The non-finite guard is load-bearing: NaN > band is
silently false, a fail-OPEN hazard, so a NaN reward must Block.

KEY DECISION 6 — no seed TOML, no crates/rules/seed/meta/
The context-package floated a crates/rules/seed/meta/reward_tampering.toml. DECISION:
do NOT create it. domain="meta" is NOT a DOMAIN_KEYWORD (rule_engine.rs detects only
trading/backtest/signal/pipeline), so a meta-domain seed would be WIRE-DARK by construction.
The plausibility band is a named Rust const (DEFAULT_PLAUSIBLE_REWARD_BAND, no-magic-number
rule); the gate is a pure driver with no seeded rule rows.

HONEST RESIDUE (Tier-2, documented NOT claimed as enforced Tier-1)
  - reward-without-verified-work: a reward jump with no corresponding RRA / ResultManifest-
    Verified attestation is unbacked. This needs the admit path to carry an attestation
    handle (attestation.rs / result_manifest.rs), which is darwin-path-coupled (feature-
    gated) AND attestation-reachability is unverified. Deferred to Tier-2.
  - spec-gaming: high reward but the metamorphic/holdout fails. Composes H-19 holdout +
    H-22 null-test, separate subsystems. Compose, do not re-author. Residue.
  - the live darwin_admit_loop.rs:298 hook: feature-gated (cfg darwinian-evolve), Tier-2/
    RunPod-pending. Tier-1 is the CLI, not this hook.
  - WHO-FREEZES-THE-FREEZER (the irreducible top, R8): the FIRST freeze of the reward
    baseline + the gate_freeze_check.py --lock re-baseline is routed to R8/HUMAN out-of-band
    (the executing run cannot lock its own gate; the agent cannot freeze its own reward).
    The human is the root of trust (mirrors H-17/H-27's "who verifies the verifier"
    terminal). DGM motivating attack: node 114 stripped its own hallucination markers to
    achieve a perfect score (dgm-interface.md:49) — the real faked-test incident this gate
    answers. Denison "Sycophancy-to-Subterfuge" is a DESIGN-REFERENCE only (no load-bearing
    quantitative claim attached). Rice's theorem is load-bearing for the structural-self-mod
    undecidability; Schmidhuber is supplemental (per H-28-grounding.md ACTIONS).

GATE-MANIFEST (R8, OUT-OF-BAND)
Added .elai_cc/crates/orchestrator/src/reward_tampering.rs to .claude/gate-manifest.txt
(the gate guards self-mod but is itself self-modifiable code; it must be frozen so it
cannot weaken its own hash compare / band / TD-3 ordering — H-27 precedent). The frozen
reward fn the 3b leg watches (promotion_gate.rs::two_gate_promote) is the CONTROLLER's
separate out-of-band add (the executor only HASHES it, does not edit it). The executor did
NOT run gate_freeze_check.py --lock — that re-baseline is the R8/human OUT-OF-BAND step.

SSB LOCKSTEP (14 sites, live+1)
SCHEMA_VERSION 235->236 (ssb.rs); SQLite CHECK +'reward_tampering' (14th lockstep site);
SsbActionType enum + as_sql_str + from_str + SsbPayload + payload->action_type;
RewardTamperingKind + RewardTamperingPayload; schema-drift self-test fixture + count
432->433; action_type_lock TOTAL_VARIANTS 512->513 + ALL_VARIANTS append + exhaustiveness
match; e45:294 literal 512->513 + RewardTampering variant assert; golden REGENERATED via
the regen_golden_file helper (reward_tampering inserts at ordinal 0364, 0364..end cascade
+1, NOT hand-edited). Live re-read at edit time confirmed 235/512 (concurrent session
s-a7e6a511 had not landed); bumped to 236/513.

EXPECTED TEST OUTCOME
crate-scoped cargo check -p elai-orchestrator -p elai-cli -p elai-contracts GREEN (pre-
existing ArtifactCheck/agent_pool reds are NOT H-28's). h28_reward_tampering_contract (6
cases) + reward_tampering unit tests + action_type_lock golden/count + e45 PASS.
`elai reward-check --demo-tampered` lands one RewardTampering(EvaluatorModified) row and
exits 1.

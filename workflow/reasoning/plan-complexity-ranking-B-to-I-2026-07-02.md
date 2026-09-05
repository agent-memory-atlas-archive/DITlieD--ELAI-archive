Plan complexity ranking (B through I only; J-M excluded per request)
Generated 2026-07-02 from a full read of the B/C/D/E/F/G/H/I summary indexes.
Judgment axis: "hard to implement" = net-new build (not activation) + dependency depth + un-gameable correctness + hardware/research gating. Effort/nature tags cited are the plans' own.

HARDEST SINGLE PLAN

I-31 grpo flywheel
- Tags: XL, WAVE 5, LAST, "BLOCKING GUARDS".
- Depends: I-30 + H-43/H-44/H-28 + I-24 (top of the tallest dependency tower in B-I).
- Why hard: new RL training deps; reward endpoint must live on the supervisor side (I-1) so the trained policy cannot reach its own reward code; governed by the Track-7 convergence theorem (without an un-gameable stronger-than-generator filter the loop provably collapses), so it cannot be attempted correctly until the entire verification spine is live and frozen. Failure mode is silent reward-hacking, not a visible bug.

TIER 2 - net-new, load-bearing, deep dependency

I-4 evidence-ranked search ladder
- Tags: XL (tiered), BLOCKING for I-22/I-28/I-30, depends I-2 + I-17.
- Why hard: PUCT/MCTS tree over SessionDag with virtual loss for parallel expansion; index notes nothing public ships tree search (real algorithm work, not activation). Half the I-plan hangs off it.

H-Track-3 trading-trust gates (H-17..H-29)
- Hardest correctness cluster. H-18 Deflated-Sharpe/PBO/MinBTL, H-20 leakage COMPOSITE over the taint graph, H-21 runtime look-ahead differential, H-22 metamorphic null-tests.
- Why hard: statistically sound quantitative-finance math in frozen numpy/scipy sidecars; attacks a deep problem (an autonomous optimizer ships overfit edge even with perfectly honest agents - max of N noisy Sharpe ratios is upward-biased). Getting it wrong looks like success.

H-14 app/website design mode
- Tags: XL, self-labeled "net-new, riskiest", MODERATE grounding.
- Why hard: dev-preview proxy serving a frameable origin, build-time opaque-id AST tagging, two-tier recast/LLM writeback; core is unproven (prior art cannot diff-against-existing; gated on a prove-it spike). High engineering risk with an unvalidated center.

TIER 3 - hard, but more contained (research-gated or blast-radius)

G-6 SAFEDELTA
- Tags: XL, depends G-2 + F-2b LIVE.
- Why hard: multi-competitor benchmark matrix with dynamic exploit execution; index admits an unresolved methodological problem (CI tightness for a significance claim with 200-392 tasks and small per-CWE cells). The honest version may not yield a defensible number.

D-1..D-9 + D-13/D-16 (DGM substrate + cross-family adversary + conformal refusal)
- The research-grade substrate. D-2 PPM needs a second GPU sidecar; several RunPod-gated; D-28/D-29/D-30 outright DEFERRED as research-grade.

E-15 / C-11 (Firecracker + Tier-C sandbox)
- Safety-critical OS isolation: seccomp arg-checking, snapshot-restore pools, CAPBSET ordering, cross-platform AppContainer parity. Hard because "mostly works" is not acceptable.

F-3 phase 2 (full CaMeL SymbolicVar)
- Tags: XL, conditional. Requires an IPC breaking change; gated on token-overhead measurements surviving the 32K ceiling.

DIFFICULTY PREDICTOR (holds across B-I)

Sort by the plans' own nature tag:
- ACTIVATION (wire-dark -> live) = easy-to-moderate (most of H Track 1 H-1..H-9).
- BUILD-on-shipped / EXTENSION = moderate.
- BUILD (net-new) = hard; the XL net-new builds that also sit atop long dependency chains (I-31, I-4, H-14) or demand un-gameable correctness (H-Track-3, the DGM/RL loops) are the real spikes.

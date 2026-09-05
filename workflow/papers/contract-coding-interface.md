Algorithm: Contract-Coding (Structured Symbolic Paradigm for Repo-Level Code Generation)
ArXiv: 2604.13100
Paper: Contract-Coding: Towards Repo-Level Generation via Structured Symbolic Paradigm
Authors: Yi Lin, Lujin Zhao, Yijie Shi
Submission: 2026-04-10
Reliability: MEDIUM (peer review status unverified; arXiv preprint only; method described at high level, several components lack pseudocode)

Citation-accuracy notice (CRITICAL — read before quoting in plans):
The headline "47%" is NOT a generic Greenfield-5 improvement. Exact paper phrasing (Abstract / Section 5.2): "Contract-Coding achieves 47% functional success while maintaining near-perfect structural integrity" on the Roguelike challenge (subset 5 of Greenfield-5). Table 1 context on Roguelike: Trae 47%, Contract-Coding 47% (TIES Trae), Gemini Studio 63% (BEATS Contract-Coding), OpenHands 30%. Authors argue their failures are "Local Logic Errors not architectural collapse" — i.e. their differentiator is qualitative (structural integrity preserved) not raw functional-success score. Do NOT cite "47% Greenfield-5 contract-test improvement" without sourcing the Roguelike subset + structural-integrity qualifier.

Inputs:
intent I | natural language brief | free form | mandatory; user-supplied product/feature description
task list T | ordered list of sub-tasks tau_i | derived from contract C | mandatory; orchestrator decomposes
contract draft C_draft | dual-layer (Constraint Projection + Executable Kernel) | text+symbolic | derived; emitted by Generator Agent

Outputs:
contract C | dual-layer entity | {S_Req, S_Api, ...} sections + kernel K=<N, Sigma, Delta> | invariant: K consistent across all mutations; section disjointness
generated code | repo-scale file tree | follows N (module->file) and Sigma (type signatures) | invariant: every module in N has an implementation; type signatures match Sigma
audit verdict | binary architectural soundness + completeness | from Discriminator Agent | invariant: rejects mutations that violate K before state pollution

Hyperparameters:
generator model | task-specific | GPT-4o / Qwen-Plus reported as agnostic on small tasks | LOW sensitivity for Gomoku/Plane Battle per Appendix C
discriminator one-pass audit | boolean enable | enabled in main config | HIGH sensitivity per ablation (w/o Contract drops 100%->65% on Plane Battle)
HEG (hierarchical execution graph) | parallel topological orchestration | enabled | LOW-to-MEDIUM sensitivity; wall-clock impact only on Gomoku (205s vs 136s)
mutation primitives O | {Add, Update} | fixed | structural — not tunable

Critical invariants (testable assertions):
assert all(module in N for module in implemented_modules), "every implemented module must be in contract namespace N"
assert all(sig_matches(impl.signature, Sigma[name]) for name, impl in implementations.items()), "type signatures in code must match contract Sigma"
assert disjoint(*contract.sections), "constraint projection sections must be disjoint"
assert all(action.op in {"Add", "Update"} for action in mutations), "only Add/Update mutation primitives allowed"
assert discriminator_verdict(C_draft).completeness, "Completeness Constraint enforced before C is committed"
assert kernel_reprojection(K, mutation).valid or rejected(mutation), "violations rejected before state pollution"

Complexity:
time: paper does NOT give big-O. Reports wall-clock: Gomoku 136-205s, larger Roguelike scales with file count (15-25 files). Parallelism via HEG = topological execution graph (nodes = tasks, edges = data deps).
space: not bounded in paper. Contract size grows with file count; kernel K is the dominant memory artifact.
parallelism: HEG enables parallel worker execution on independent subgraph nodes; serialization point = each kernel re-projection after mutation.

Failure modes (cited in paper):
context-fidelity tradeoff: longer intents drift further from executable spec (the problem the method addresses)
discriminator one-pass missing latent contradictions: only one audit pass per draft; multi-pass not evaluated
local logic errors: Contract-Coding's own failure mode per Section 5.2 ("strictly confined to Local Logic Errors")
file-count scalability: only tested up to ~25 files (Roguelike); behavior at 100+ file repos unverified

Implementation pitfalls (do NOT do this):
do NOT treat the contract as plain prose: kernel K must be symbolically represented (N namespace, Sigma signatures, Delta state) — not just markdown
do NOT skip the Discriminator Agent rectification pass: ablation shows 100%->65% degradation on Plane Battle without it
do NOT allow mutations outside {Add, Update}: paper deliberately excludes Delete to avoid kernel inconsistency
do NOT generate code without re-projecting K after each mutation: violations must be caught BEFORE state commit
do NOT cite "47% Greenfield-5 improvement" as a general number: it is the Roguelike subset score, ties Trae, trails Gemini Studio
do NOT claim formal pre/postconditions: paper uses no Hoare triples, no temporal logic, no Eiffel-style assertions; the "contract" is structural (namespace + signatures + state space), NOT behavioral logic

Greenfield-5 benchmark composition (Section 5.1):
1. Gomoku (Logic)
2. Plane Battle (Event-Driven)
3. Snake++ (Event-Driven)
4. City Sim (Resource Management, ~11 files)
5. Roguelike (Scalability, 15-25 files)
Metrics: Executability, Interactivity, Rule Adherence; results averaged over 10 runs.
Baselines: MetaGPT, ChatDev, FLOW, OpenHands (academic); Lingma, Trae, Gemini Studio, CodeBuddy (commercial).

Pre/postcondition emitter status: UNSUPPORTED by paper.
The paper does NOT provide a pre/postcondition extraction algorithm in the formal-verification sense. The "contract" is a structural artifact (module namespace N + type signature set Sigma + state space Delta), not a logical assertion language. Any C-50 claim that this paper supplies "pre/postcondition extraction" needs re-grounding — substitute a paper that actually does formal contract extraction (e.g., Daikon-family invariant inference, or recent LLM-DbC work like Endo et al. on LLM-generated JML contracts), or rescope C-50 to structural contracts only.

Ablations (Appendix C):
w/o Contract: Plane Battle 100% -> 65%; Task Existence E(C) = 82%
w/o HEG: marginal (wall-clock only); functional success unchanged on small tasks
Model agnosticism: Qwen-Plus = GPT-4o parity on Gomoku and Plane Battle

Recommended use in C-50:
GROUNDED claims supportable from this paper:
- structural contract (N, Sigma, Delta) drives parallel codegen
- one-pass auditor rectification of contract draft is high-leverage (large ablation gap)
- mutation primitives restricted to {Add, Update} for kernel consistency
- HEG/topological execution graph for parallelism with kernel-reprojection serialization points

UNGROUNDED claims that need separate sourcing or rescoping:
- "47% Greenfield-5 contract-test improvement" — misframes the result (Roguelike-only, ties Trae, trails Gemini)
- pre/postcondition extraction — paper does not provide one
- formal contract language — none used; structural only

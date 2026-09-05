# Output-Quality Enforcement + Familiarity Surface — design synthesis + grounding ledger (2026-06-27)

Mode: spec-eval AS-SPECIFIED + paper-to-plan PHASE 4/5 shape, main LLM in-band, no fleet.
Source: this session's grounded read of ELAI-FEATURE-MAP.md (full), C-SUMMARY C-31/C-32, J-SUMMARY J-23/J-43/J-65, .claude/rules/maintainability-review.md, and the prior-turn GRC decision ledger (topic elai-solid-and-extensibility). The "idea" is the user's two questions: (1) should output code-quality (SOLID-class) be a GATE/pre-gate so every code ELAI makes — by ANY model — is high quality, enforced like the other rules; (2) the familiarity surface (skills the user can improve / ask ELAI to improve, hooks, MCP, a one-shot builder for lazy users + a trust spine for advanced users).
External design-rationale only (ID-5, no number is a done-gate): SOLID (R. C. Martin, "Design Principles and Design Patterns", 2000) [DS-1]; Holzmann "Power of Ten" rules (NASA/JPL 2006) [DS-2, already carried by J-65]. Neither contributes a measured number; both are EXTERNAL/DESIGN-RATIONALE.

## PART 1 — the grounded finding (output quality)

FINDING-1 (the one that flips the obvious answer): ELAI does NOT need a new "quality enforcement engine." It already has a GENERATION-TIME quality loop (C-32: SkeletonGenerator > BLANK_N bodies > MiniCritic PASS/WARN/BLOCK per fill > StructuralRetry Decompose/Template/FewShot/BestOfN > AcceptAndComplete at ratio>=0.5) and a plan-time size/shape budget (C-31: FilePlan max_lines + MAX_FUNCTIONS_PER_FILE=20). What is missing is (a) a SOLID/maintainability PROXY predicate inside that loop and inside the auditor, and (b) the same enforcement reaching GENERATED code, not just ELAI's own commits. This is ACTIVATION + one net-new checker, not invention.

FINDING-2 (grep-confirmed, prior turn): zero by-name SOLID enforcement anywhere (the 234 hits were CSS "1px solid" / incidental "cohesion"). The closest existing things are:
- .claude/rules/maintainability-review.md + .claude/tools/maintainability_check.py [WIRED]: MR-1 file<1000, MR-2 no as-any/unsafe, MR-3 fn<80, MR-4 branching density, 8 qualitative dimensions. AUDITOR-OWNED over ELAI's OWN code. Dimensions 5 (type/boundary clarity), 6 (canonical layering), 0 (structural simplification) are SOLID-adjacent (DIP/ISP/SRP) but framed as maintainability, not principles.
- J-65 power-of-ten gate [PLANNED, observe-only]: acyclic call graph / bounded loops / assertion density / fn<=60 / cfg<=2 over GENERATED code. P10 complexity/safety, not SOLID.
- C-31/C-32 [PARTIAL/WIRED]: the generation-time loop above.
- J-23 review-gate catalog [PLANNED]: gates GENERATED multi-language code (vulture/knip/deptrac/dependency-cruiser/a11y) — its own example is "a generated Python file with dead code + a Svelte div-onClick -> BLOCK".

FINDING-3 (the load-bearing design rule, from maintainability-review.md ROLE BOUNDARY): the quality VERDICT is the AUDITOR's, never the executor self-grading its own output (Berkeley RDI Apr-2026: in-boundary artifacts gamed ~100%). So a quality gate is off-agent + frozen (R8). The generation-time MiniCritic is allowed because it SELF-CORRECTS (retries) but is NOT the authoritative verdict — that stays downstream.

FINDING-4 (the "any model" point): a weak local model (Qwen-7B) ships worse code than cloud Sonnet. The ONLY layer that catches AND FIXES weak-model output in-loop is generation-time (C-32 MiniCritic + StructuralRetry). The auditor only blocks at the end. So "every code ELAI (any model) makes is high quality" specifically argues for the gen-time layer being the workhorse, paired with J-55 modality routing (route the weak lane to constrained generation that produces gate-passable code by construction).

DESIGN VERDICT (output quality): enforce it, but as the SAME layered, fail-closed, observe-first, off-agent pattern every other ELAI gate uses (wiring-enforcement Layer 1-4 is the template), across FOUR seams with different roles:
- SEAM 1 plan-time prevention (extend C-31): a per-file responsibility budget (SRP). Cheapest; = J-89 L1/L8 prevention shape.
- SEAM 2 generation-time self-correction (extend C-32 MiniCritic): a SOLID/god-object METRIC predicate; the any-model workhorse; retries in-loop.
- SEAM 3 write-time floor (VibeGate F-1 PreToolUse): ONLY the deterministic floor (MR-1 file>1000, MR-2 as-any). Subjective quality is NOT a hard PreToolUse block (thrash risk).
- SEAM 4 audit-time authority (frozen solid_proxy_check.py + J-65 + J-23 lane): the authoritative verdict, off-agent, frozen, observe-only first.
Every new quality rule enters SHADOW/observe-only with a fail-closed debt ledger + measured FP rate, promoting to BLOCK only on fpr<5%/tpr>80% via the rules-engine lifecycle (3F) — exactly J-43's split (thin deterministic FLOOR + cross-family ADVISORY + debt LEDGER + measured RATE), because subjective quality FP >> security FP.

SOLID-proxy mechanizability (the honest split):
- SRP -> fan-out + size (HIGH confidence, the ONE deterministic BLOCK = god-object): a symbol with high fan-out (distinct external modules/types touched) AND high method/line count. Data: B-29 impact API fan-in/fan-out (WIRED) + C-31 MAX_FUNCTIONS + MR-3.
- DIP -> cross-layer concrete dependency (MEDIUM, WARN): concrete cross-crate TYPE_REF vs IMPL_TRAIT. Data: elai.map tags (WIRED). = maintainability dimension 6 formalized.
- ISP -> fat trait (LOW, WARN): trait method count vs per-impl usage. Data: elai.map.
- OCP + LSP -> NOT deterministically checkable -> cross-family ADVISORY (J-89 L1 spec-jury / J-43 advisory arm), never a block. Honest residue (GD-5).
Caveat (named, GD-5): ELAI core is Rust (traits, not class inheritance) so SRP/DIP proxies bite there; for GENERATED code (often TS/Svelte/Python) the J-23 tool catalog does more real work than a hand-rolled SOLID checker. The SOLID-proxy is mostly an own-codebase + Rust-generated-code discipline; for generated TS/Py lean on J-23.

## PART 2 — the grounded finding (familiarity surface)

FINDING-5 (hooks): ELAI HAS a hook system [WIRED] — HOOK SYSTEM T1/T2/T3, PreToolUse/PostToolUse/Stop/PreCompact — and it is ARCHITECTURALLY STRONGER than base CC: hooks intercept at the orchestrator/IPC layer, agents never see them (CC runs hooks in the user shell where an agent can notice/route around them). Gaps: no SubagentStop axis (J-52 planned), IPC migration PARTIAL (B-19). The CC-familiarity gap is that a CC user EXPECTS to drop a hook into settings; ELAI's hooks are compiled Rust T1/T2/T3 + the rules-engine (rules-as-DATA) is the user-facing equivalent — but it is not presented as a user-authorable hook surface.

FINDING-6 (skills): three distinct skill notions, plumbing built, FRONT DOOR DARK:
- E-43 AGENT SKILL SYSTEM [DARK(ELAI_SKILL_SYSTEM_ENABLE)]: agentskills.io-v1 SKILL.md validator + `elai skills-mgmt` + per-skill eval JSONLs + skillspector import gate. CAN consume the same SKILL.md format CC/agentskills users know, BUT log_skill_invocation has NO production caller.
- F-6 LEARNED-SKILL LIBRARY [WIRED]: SkillTrustTier S1-S4 + Voyager critic-gated ACQUISITION; skill_permits_autonomous in production. ELAI already LEARNS skills.
- I-12 EVIDENCE-GATED PLAYBOOK CURATOR [PLANNED]: promote on >=k SSB-verified outcomes, export to CLAUDE.md/AGENTS.md/skills — the episodic->skill->rule ladder ("ask ELAI to improve a skill" formalized).

FINDING-7 (MCP): both directions — consume (E-24 pin / G-4 rug-pull / C-42 authz / H-34 defer, hardened) AND be a server (J-2 ELAI-AS-MCP control plane [PLANNED]). The familiarity gap is small.

FINDING-8 (one-shot builder): mostly already there under the hood — PIPELINE FACTORY L1/L2 [WIRED], PRODUCTION FULLSTACK CODEGEN E-12 [WIRED 06-07], web templates (vanilla/astro/vite-react/vite-svelte), J-63 local-lane Claudeception `elai artifact run` [PLANNED], I-40 `elai wrap` RRA [WIRED]. NOT presented as "type an idea -> get a VERIFIED app." The trust spine (RRA receipt) is the differentiator that makes the lazy path worth switching to.

DESIGN VERDICT (familiarity surface): the two audiences are the SAME engine at two surfaces. The advanced/trust user buys the verification spine; the lazy/one-shot user buys a facade over the SAME spine with the receipt attached. The strategic posture (project_market_positioning_2026 + the GTM ledger): headline "verified", never "cheap"/"non-coder". So the work is ACTIVATION (light the dark front doors) not invention. Three surfaces:
- skills front door (light E-43 + wire I-12 curator + compose F-6/J-62): user drops/edits a SKILL.md, ELAI uses it AND can improve it on evidence.
- user-authorable rules/hooks (expose rules-as-DATA + a user-hook registration; the CC-hook familiarity, done ELAI's safer IPC way; compose J-52/J-27): the user "keeps working like CC" — a hook/rule is added as DATA, no recompile, and it passes the SAME trust gates.
- one-shot verified builder front door (pipeline-factory + E-12 + J-63 + I-40 wrap + J-2 MCP compose): "type an idea -> get an app + its RRA receipt."

## PART 3 — adversarial pass (attack-the-design)

A-1 "a SOLID gate will thrash the agent / generate revolt findings." MITIGATED: only the god-object class is a deterministic BLOCK; DIP/ISP are WARN; OCP/LSP are advisory; everything ships observe-only first behind the rules-engine SHADOW lifecycle with a measured FP bound (J-43/GD-4). If the FP rate is not bounded on >=3 runs, it never blocks.
A-2 "the executor will game its own quality check." MITIGATED structurally: the AUTHORITATIVE verdict is auditor-owned + frozen (R8), never executor-self (maintainability-review.md ROLE BOUNDARY; FINDING-3). The gen-time MiniCritic is self-CORRECTION (retry), explicitly NOT the verdict.
A-3 "a user-authored skill/hook is an injection/RCE vector." MITIGATED: every user surface passes the SAME existing gates — E-43 skillspector whole-directory scan + skill-import-policy + J-102 config-ingress admission (blake3 load-time manifest, fail-closed) + VibeGate F-1; a user hook/rule is DATA evaluated by the existing T1/T2/T3 engine, never arbitrary code. FS-2 binds this.
A-4 "the one-shot builder dilutes the trust positioning into a toy." MITIGATED: the one-shot path SHIPS the RRA receipt (I-40 wrap) — it is the trust product with a lazy front end, not a second dumber product (FS-3). The "$20=$200"/value claim stays gated on a measured SAFEDELTA/QDELTA number (KD-1/ID-5), never asserted.
A-5 "this is all just restating C-31/C-32/E-43/J-2/I-12." PARTIALLY TRUE and that is the point (GD-1 compose-not-restate): the L-T2 files COMPOSE those; the net-new is ~one frozen solid_proxy_check.py + one MiniCritic predicate (L-11), the lit front doors + the curator wiring (L-12/L-13/L-14). Re-implementing any composed unit is a P0.

## PART 4 — steal/verdict ledger

V-1 SOLID-proxy + 4-seam output-quality system -> NEW (L-11). Net-new = solid_proxy_check.py + the MiniCritic predicate; the rest composes C-31/C-32/maintainability_check.py/J-65/J-23/B-29/elai.map/rules-3F/VibeGate-F-1/I-38-QDELTA.
V-2 generation-time any-model self-correction -> EXTENSION(C-32) folded into L-11 SEAM 2 (not a separate file; GD-1) + compose J-55 modality router.
V-3 skill front door + self-improving skills -> ACTIVATION(E-43) + wire I-12 + compose F-6/J-62 -> NEW(L-12) (the lit door + the curator loop; do NOT re-author E-43/F-6/I-12).
V-4 user-authorable rules/hooks surface -> NEW(L-13) over the WIRED rules-engine 3F + HOOK SYSTEM + compose J-52/J-27; the net-new is the user-facing registration + FS-2 same-gates guard.
V-5 one-shot verified builder front door -> NEW(L-14) over pipeline-factory/E-12/J-63/I-40/J-2; net-new = the entrypoint + the receipt-attached lazy UX.
V-6 MCP-as-server user surface -> ALREADY-PLANNED(J-2); compose in L-14, do NOT re-author.
V-7 by-name OCP/LSP deterministic checker -> REJECT (not deterministically checkable; stays cross-family ADVISORY per J-89 L1 / J-43; honest residue GD-5).
V-8 hard PreToolUse SOLID block -> REJECT as a day-one default (thrash; A-1). Only the MR-1/MR-2 deterministic floor sits at write-time; SOLID promotes through SHADOW.

Delta families EARNED (template R-j — prior families cover RUNTIME wiring + CLAIM-RISK + MARKETING, not OUTPUT-QUALITY-of-generated-code or USER-EXTENSIBILITY-surface failure modes):
- OQ (output-quality) OQ-1..OQ-5: verdict-is-auditor-owned-never-self (in-boundary rubber-stamp); deterministic-FLOOR-vs-advisory-CEILING split; observe-first + measured-FP before BLOCK; mechanizable-proxy-only (SRP/DIP deterministic, OCP/LSP advisory); fix-at-generation-not-just-block-at-audit (the any-model leg). Binds L-11.
- FS (familiarity-surface) FS-1..FS-4: user-extends-via-DATA-not-recompile (rules/skills as data); every-user-surface-passes-the-SAME-trust-gates (no bypass of VibeGate/skillspector/wiring); trust-spine-attached-to-the-lazy-path (one-shot ships an RRA receipt); format-compatibility (agentskills.io/CC SKILL.md + MCP so users keep their workflow). Binds L-12/L-13/L-14.

## OUTCOME
Plan files created (L-T2, this run): L-11 output-quality enforcement system [KEYSTONE]; L-12 skill front-door + self-improving skills; L-13 user-authorable rules & hooks surface; L-14 one-shot verified builder front-door. Grounding map: _grounding/L-OQF-GROUNDING-MAP.md. Indexes + feature map upserted. Two delta families added: OQ-1..OQ-5, FS-1..FS-4. No commit (docs/ + workflow/ gitignored).

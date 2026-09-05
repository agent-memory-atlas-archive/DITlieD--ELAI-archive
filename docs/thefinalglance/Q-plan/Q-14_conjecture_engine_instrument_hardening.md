# Q-14: conjecture-engine instrument hardening — make the harness novelty leg real, quantify refutations, and cost-honest the SAFE tie-break (plus one doctrine-hygiene rider)

STATUS: PLANNED (WAVE AUTORESEARCH-1; harness-engine hardening wave; three code units + one doctrine rider in one file — E-1 harness novelty-leg instrument + canary, E-2 quantified KILL-EDGE numeric tail, E-4 cost-honest SAFE tie-break, E-3 the P-25 exo-inject citation. All land in the advisory conjecture engine tooling; none touches .elai_cc runtime except by composing the NoveltyClass concept Q-11 hardens.)
E-1 PRECONDITION MET 2026-07-18: B-SYN-0 = SUPPORTED (workflow/reasoning/B-SYN-0-decisions.md; the LLM novelty instrument discriminates known-SAME from known-DISTINCT on the sealed ELAI-archive corpus, 20/20, producer + independent checker agree). E-1's verdicts leg may now proceed to its NEXT GATED implementation step as an in-distribution discriminator + known-answer canary. BOUNDED (prereg 4.6): this authorizes the harness leg on in-distribution ELAI artifacts only; it is NOT a general novelty oracle and does NOT reopen Q-10/P-25. E-2/E-3/E-4 unchanged (E-2 gated by B-SYN-1 SUPPORTED; S-1 stays REJECTED-by-measurement per B-SYN-2).
E-1 SHIPPED 2026-07-18: the harness novelty-leg known-answer canary + fail-closed instrument_strength=FAILED suppression + --self-test-novelty-canary landed in .claude/tools/conjecture_novelty_canary.py (helpers) consumed by conjecture_triage.py's --novelty / --self-test-novelty-canary CLI arms; adversarial-auditor READY (0 BLOCK), verified live on the archived AIDE2 fixture (M1 NLI-JUDGED non-UNCERTAIN verdict / M2 scrambled FAILED zero-row fail-close). RULE-ZERO tests only. E-2 (KILL-EDGE numeric tail) / E-4 (SAFE cost tie-break) / E-3 (P-25 citation) remain PLANNED. Verdict doc: workflow/reasoning/Q-14-decisions.md.

Authoring provenance: paper-to-plan PHASE 5 over the autoresearch-synergy steal (conjecture engine run workflow/reasoning/conjecture-run-autoresearch-synergy-2026-07-17.md; survivors C1/E-1 + C2/E-2 + C4/E-4 as units + C3/E-3 as a doctrine rider, all refuted=0). Auditors do NOT read the engine run artifact (SF-3) — anchors below are re-greppable in-tree/in-tool (W-7).

- Track: AUTORESEARCH-SYNERGY | lever: instrument-the-engine-that-picks-improvements | source: Evo/NeMo bidirectional study reasoning §3 (E-1..E-4) + the archived 2026-07-15 AIDE² run (every equivalence verdict UNCERTAIN(WEAK), C8 survived-but-unauthored) | grounding: STRONG (every gap grep-confirmed; the AIDE2-run inertness is an ELAI-internal receipt) | nature: BUILD-on-shipped (hardens the shipped improvement-conjecture engine tooling) | effort: M
- PROOF_SPINE: E-1 feeds PS-3 (harness novelty-measurement integrity — the harness leg of the same hallucination-delta bench family Q-11's C6 canary hardens on the runtime novelty.rs path). E-2/E-3/E-4 declare `none` (engine-instrument-discipline / doctrine class); user-directed authoring 2026-07-17 (the instruction to author these from the synergy reasoning file IS the authoring authority), flagged for explicit ack review per PSR-2.

- Premise: the improvement-conjecture engine's ONE proven selector (NOVEL-CREDIBLE) is inert on real runs. In the archived 2026-07-15 AIDE² run EVERY candidate's equivalence verdict was UNCERTAIN with instrument_strength=WEAK, because `conjecture_triage.py --novelty` with no supplied NLI verdicts file falls to the deterministic bin-coordinate leg (`grep -cE "UNCERTAIN\(WEAK\)" conjecture-run-aide2-*.md` = 10). Q-11's C6 canary hardens the RUNTIME novelty path (novelty.rs); the HARNESS triage tool needs the sibling treatment (E-1). Separately, KILL-EDGE records WHO killed a candidate but not BY HOW MUCH (`grep -nE "delta|noise" conjecture_triage.py | grep -ci kill` = 0), yet the AIDE² announcement's most transferable finding is that most rejects die within noise, so the recheck value of a loser depends on whether it lost by a mile or a whisker (E-2). The SAFE profile tie-broke five S-class candidates by id-order (declared arbitrary) even though stage-3 measured cost is already hashed into the artifact (E-4). And the SKILL says exo-inject "survives as ordinary grounding techniques" with no citation of the adjacent P-25 refutation (E-3, doctrine). This file closes all four.

## Steal/build

E-1 — harness novelty-leg instrument + canary (conjecture_triage.py, pure Python, no Rust). (a) A documented cheap way to MINT the NLI verdicts file per run: a spawned judge (model=sonnet, fresh context, the existing I-24/N-15 audits — randomized candidate order + declared verbosity penalty) emits `{nearest_known_mechanism, equivalence: SAME|DISTINCT|UNCERTAIN, judge_votes, instrument_version}` rows over the run's candidates against the fold archive. (b) A known-SAME + known-DISTINCT canary PAIR injected into EVERY `--novelty` invocation; a canary MISS (SAME judged DISTINCT or vice-versa, or a scrambled verdicts file) fail-closes to `instrument_strength=FAILED` and SUPPRESSES all novelty verdicts for that run, mirroring Q-11's C6 exactly. Adds a `--self-test-novelty-canary` self-test that FAILS if no canary pair is wired (dead-evidence-path screams, wiring R2).

E-2 — quantified KILL-EDGE numeric tail (conjecture_triage.py grammar + --validate + --fold). KILL-EDGE gains an OPTIONAL quantitative tail for experiment-killed candidates: `KILL-EDGE: C7 <- experiment:<blake3> delta=-0.031 band=0.045 verdict=within-noise`. The RECHECK HOOKS grep gains a "reinstate the within-noise kills first" ordering. Append-only compatible: existing v1.0/1.1 artifacts (no tail) still validate and fold. Composes S-2's power floor (Q-15): a within-noise kill is exactly a NULL-UNDERPOWERED candidate.

E-4 — cost-honest SAFE tie-break (conjecture_triage.py PICK / --profile safe). Where stage-3 micro-experiments executed, their measured token cost is already hashed into the artifact; fold measured-cost-when-available into the SAFE tie-break BEFORE id-order (min groundedness-floor -> min cost-class -> max validated -> min measured-token-cost -> id-order). Zero new machinery; the id-order fallback stays for the no-receipt case.

E-3 — P-25 exo-inject citation (doctrine RIDER; .claude/skills/improvement-conjecture/SKILL.md + the ELAI-FEATURE-MAP.md rejection ledger). One sentence beside the "survive as ordinary grounding techniques" line citing P-25 Family A (relevant exogenous artifacts did not reliably beat shuffled/padding at generation grain; 1.54x token cost) — noting it tested a DIFFERENT lifecycle point (benchmark generation vs engine stage-1 conditioning) so it does not kill exo-inject as a birth operator but the survival claim is half-contradicted and must carry the citation. Plus a feature-map ledger cross-ref. No build, no code path.

EXCLUDED: any revise/iterate stage on the engine (P-25 Family B refuted aligned triage+revise; 3.1x cost); any resurrection of ProbeUtility / multi-cycle FDL under a new name; making novelty an asserted discovery capability (it stays a reported archive-relative axis); an LLM judge as the E-1 canary oracle (the canary pair is deterministic known-answers); making conjecture_triage.py a gate-manifest entry (it stays advisory, TD-3).

## Existing ground truth (PLAN-asserted; executor re-greps per W-7)

- `.claude/tools/conjecture_triage.py` — `--novelty` degrades honestly to `equivalence=UNCERTAIN` + `instrument_strength=WEAK` with no supplied verdicts file; `--self-test` runs exactly 3 fixture kills (demarcation/ledger/anchor), NO live novelty canary; KILL-EDGE grammar carries no numeric field (`grep -nE "delta|noise" | grep -ci kill` = 0); the SAFE profile tie-break is groundedness -> cost -> validated -> id-order with no cost term (`grep -inE "id.order|tie.break" | grep -ic cost` = 0). NOT in `.claude/gate-manifest.txt` (advisory; safe to extend).
- `crates/orchestrator/src/novelty.rs::NoveltyClass / NoveltyClassifiedEmitter` — the RUNTIME novelty labeling Q-11's C6 hardens; E-1 is the HARNESS sibling and COMPOSES this concept (the NLI verdict schema mirrors it), it does NOT modify novelty.rs.
- `.claude/skills/improvement-conjecture/SKILL.md` — DIVERSITY block says "Empty-region targeting and exogenous injection survive as ordinary grounding techniques" (line ~70); no P-25 citation (`grep -ic "p-25"` = 0).
- The archived run `workflow/reasoning/conjecture-run-aide2-deep-research-novelty-2026-07-15.md` — the fixture E-1's falsifier runs against (all-UNCERTAIN(WEAK) PICK table).
- Q-11 (Q-plan): the RUNTIME C6 canary + `--self-test-canary` pattern E-1 mirrors on the harness side. Do NOT re-implement (GD-1).

## REVISED scope (W-10 tiered)

- SHIP FIRST (E-1): the NLI-verdicts minting path + the known-answer canary pair + the fail-closed `instrument_strength=FAILED` suppression + `--self-test-novelty-canary`. In-process/harness; the judge spawn is the only network leg and is optional (absent judge -> honest WEAK degrade, unchanged).
- SHIP SECOND (E-2): the optional KILL-EDGE quantitative tail + the validator arm + the RECHECK "within-noise first" ordering. Append-only; grandfathered artifacts unaffected.
- SHIP THIRD (E-4): the measured-cost tie-break leg in the SAFE profile.
- SHIP FOURTH (E-3): the SKILL sentence + the feature-map ledger cross-ref. Doctrine only.

## Integration

- `.claude/tools/conjecture_triage.py`: the `--novelty` verdicts-minting + canary leg (E-1), the `--self-test-novelty-canary` self-test (E-1), the KILL-EDGE quantitative-tail parse/emit + validator (E-2), the SAFE-profile measured-cost tie-break (E-4). Each new function is consumed by its argparse CLI arm in the same changeset (the production caller for a harness tool).
- `.claude/skills/improvement-conjecture/SKILL.md`: the E-3 citation sentence (doctrine).
- `docs/thefinalglance/ELAI-FEATURE-MAP.md`: the E-3 ledger cross-ref (doctrine).
- frontend_pair: N/A with justification — the improvement-conjecture engine is harness/plan-authoring-time tooling with no dashboard surface. Justified.

## MOVES (wargame)

M1 build the E-1 NLI-verdicts minting path + the known-answer canary pair and run `conjecture_triage.py --novelty` on the archived AIDE² run with a minted verdicts file.
EXPECTED_OBSERVATION: at least one candidate now carries a non-UNCERTAIN equivalence verdict (a natural known-SAME probe resolves SAME), `instrument_strength` is not WEAK, and `--self-test-novelty-canary` reports a present canary pair.
ON_FAIL: every verdict is still UNCERTAIN(WEAK) — the minted file was ignored or the judge leg is unwired; re-anchor the canary at the real `--novelty` verdict-emission code path (not a test shim, R7) and confirm the verdicts file is consumed, do not accept a WEAK-fallback pass.

M2 hand the E-1 canary a SCRAMBLED verdicts file (SAME fixture forced to DISTINCT) and re-run `--novelty`.
EXPECTED_OBSERVATION: the canary MISSES, `instrument_strength=FAILED` is set, and ALL novelty verdict rows for that run are SUPPRESSED (absent), not emitted-with-a-warning.
ON_FAIL: dead verdicts still emit alongside the FAILED flag — the suppression is advisory not blocking; make it fail-closed (write NO verdict rows when `instrument_strength=FAILED`), mirroring Q-11's C6.

M3 add the E-2 optional KILL-EDGE quantitative tail + validator arm and fold a synthetic artifact carrying a `delta=/band=/verdict=` tail.
EXPECTED_OBSERVATION: `--validate` and `--fold` round-trip the quantified-tail artifact; an existing v1.0/1.1 artifact with NO tail still validates and folds unchanged; the RECHECK HOOKS section orders within-noise kills first.
ON_FAIL: the tail breaks grandfathered validation (the parser now requires the tail) — make the tail strictly OPTIONAL (a plain KILL-EDGE with no numeric fields still validates); a within-noise kill not surfaced first -> fix the RECHECK ordering.

M4 add the E-4 measured-cost tie-break leg and re-run `--novelty --profile safe` over a run where two S-class candidates tie at validated_value 0 but differ in stage-3 token cost.
EXPECTED_OBSERVATION: the SAFE pick is the lower-measured-cost candidate; a run where NO stage-3 receipt exists still tie-breaks deterministically by id-order (fallback intact).
ON_FAIL: the tie-break reads a re-derived/estimated cost instead of the stage-3 receipt hash — read only the executed-receipt token count; a receiptless run that now errors -> restore the id-order fallback.

M5 add the E-3 P-25 citation sentence beside the SKILL survival claim + the feature-map ledger cross-ref.
EXPECTED_OBSERVATION: `grep -c "P-25" SKILL.md` >= 1 with the citation adjacent to the "survive as ordinary grounding techniques" line, and the ELAI-FEATURE-MAP.md ledger carries a P-25 exo-inject cross-ref row.
ON_FAIL: the citation lands far from the survival claim (a future reader misses it) -> place it in the same sentence/paragraph; the claim is left uncited -> a session re-litigates or over-trusts exo-inject.

## FORK

FORK: if a cross-family / spawned judge is unavailable when E-1 runs (gpt-bridge absent, sonnet judge unspawnable), take route B — the `--novelty` leg keeps its EXISTING honest WEAK degrade (unchanged behavior) and the canary still fail-closes on a scrambled DETERMINISTIC fixture pair (the canary oracle is deterministic known-answers, not the judge); record the judge substitution in Q-14-decisions.md. Do NOT make the canary depend on a live judge.

FORK: if the KILL-EDGE parser is stricter than expected and a bare (no-tail) KILL-EDGE cannot coexist with the optional-tail grammar without a schema bump, take route B — version the tail behind the existing `ARTIFACT_VERSION` gate (1.1 tail optional, 1.0 grandfathered never carries it) rather than forcing every artifact to re-emit; do not rewrite historical artifacts.

## RECON_NEEDED

RECON_NEEDED-1: before E-1, re-read the `--novelty` code path in conjecture_triage.py to confirm the verdicts-file consumption point and whether a FUNCTIONAL-equivalent canary already exists on that leg (the premise grep proved the NAMED self-test absent, not that no equivalent guard exists); a live equivalent collapses E-1 to wiring the suppression + self-test only.
RECON_NEEDED-2: before E-2, re-read the KILL-EDGE emit + `--validate`/`--fold` parsers to confirm the tail can be appended without breaking the `KILL_EDGE_LINE_RE` grammar (killer-ref must stay a stable id, never a line number, ATTACK-3); confirm the tail fields (delta/band/verdict) do not collide with the existing killer-ref capture group.
RECON_NEEDED-3: before E-4, re-read the SAFE profile lexicographic order + where stage-3 token cost is hashed into the artifact, to confirm a measured-token-cost is readable per candidate (not only a run-total); if only a run-total exists, E-4 narrows to the candidates that carry a per-candidate experiment receipt.

## ABORT

ABORT: any proposal that (a) lets the E-1 novelty verdicts emit while `instrument_strength=FAILED`, (b) makes the E-1 canary oracle an LLM judge instead of deterministic known-answers, (c) turns novelty into an asserted discovery capability rather than a reported axis, (d) makes the E-2 tail MANDATORY so grandfathered artifacts fail to validate, (e) adds a revise/iterate or multi-cycle/ProbeUtility stage under any name, or (f) puts conjecture_triage.py into .claude/gate-manifest.txt (it is advisory, TD-3) -> STOP and flag. Each breaks an engine boundary or a refuted-mechanism guard.

## WIRING_CONTRACT
produces_symbols:
  - conjecture_triage.py — the `--novelty` NLI-verdicts minting + known-answer canary leg + `--self-test-novelty-canary` (E-1)
  - conjecture_triage.py — the optional KILL-EDGE quantitative-tail parse/emit + validator arm + RECHECK within-noise-first ordering (E-2)
  - conjecture_triage.py — the SAFE-profile measured-token-cost tie-break leg (E-4)
  - SKILL.md + ELAI-FEATURE-MAP.md ledger — the P-25 exo-inject citation (E-3, doctrine)
consumes_symbols:
  - conjecture_triage.py `--novelty` honest-WEAK-degrade leg (extend, do NOT replace, GD-1)
  - novelty.rs::NoveltyClass / NoveltyClassifiedEmitter — the runtime novelty concept the harness verdict schema mirrors (compose, do NOT re-implement)
  - Q-11 C6 live-path canary + `--self-test-canary` pattern (mirror on the harness side, do NOT restate)
  - the archived conjecture-run-aide2-*.md artifact (read as the E-1 falsifier fixture)
reachable_from:
  - `python .claude/tools/conjecture_triage.py --novelty <artifact> --verdicts <minted>` [PRODUCTION triage CLI] fires the E-1 canary
  - `python .claude/tools/conjecture_triage.py --self-test-novelty-canary` [PRODUCTION triage CLI] fails if no canary present (R2)
  - `python .claude/tools/conjecture_triage.py --validate <artifact>` / `--fold` [PRODUCTION triage CLI] exercises the E-2 tail
  - `python .claude/tools/conjecture_triage.py --novelty <artifact> --profile safe` [PRODUCTION triage CLI] exercises the E-4 tie-break
  - a `if __name__ == "__main__"` argparse dispatch arm is the production caller for each new fn; a `--self-test`-only or unit-test-only caller NEVER satisfies reachability (R7)
stub_ban:
  - NO advisory-only suppression — `instrument_strength=FAILED` writes NO novelty verdict rows (fail-closed)
  - NO LLM judge as the canary oracle; the canary pair is deterministic known-answers
  - NO mandatory KILL-EDGE tail — a bare KILL-EDGE still validates (grandfathered artifacts unaffected)
  - NO re-derived/estimated cost in the tie-break — only the stage-3 executed-receipt token count
  - NO revise/iterate, ProbeUtility, or multi-cycle stage; NO novelty asserted as discovery
  - NO conjecture_triage.py entry in .claude/gate-manifest.txt (advisory, TD-3); NO blake2/sha2 (blake3 for any receipt id)
frontend_pair:
  N/A with justification — harness/plan-authoring-time engine tooling, no dashboard surface. Justified.
observable_effect:
  trigger: "python .claude/tools/conjecture_triage.py --novelty workflow/reasoning/conjecture-run-aide2-deep-research-novelty-2026-07-15.md --verdicts <minted> then the same with a scrambled verdicts file, then --validate + --fold a quantified-tail artifact, then --novelty --profile safe"
  expect_ssb: "N/A (harness tool; the off-agent artifact is the tool's own exit codes + emitted verdict rows, not an SSB row)"
  expect_artifact: "a healthy run emits >=1 non-UNCERTAIN verdict; a scrambled run emits ZERO verdict rows with instrument_strength=FAILED; a grandfathered no-tail artifact still validates/folds; the SAFE pick orders by measured cost when a receipt exists; SKILL.md carries the P-25 citation. NEGATIVE: verdicts emitted under FAILED, a mandatory tail breaking grandfathered validation, a re-derived cost in the tie-break, or an uncited survival claim must FAIL"

PRODUCTION_HOOK: producer E-1 canary/minting -> consumer the `--novelty` argparse arm in conjecture_triage.py's `main()`, wired same changeset; producer E-2 tail parse/emit -> consumer the `--validate`/`--fold` arms; producer E-4 tie-break -> consumer the `--novelty --profile safe` pick path; producer E-3 citation -> consumer the SKILL.md survival-claim paragraph (doctrine, no code caller). No producer-only fn and no --self-test-only caller satisfies the hook (R7).

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-14_conjecture_engine_instrument_hardening.md --run-trigger
python .claude/tools/conjecture_triage.py --self-test-novelty-canary --json     # MUST report a present canary (fails if absent, R2)
rg -c "instrument_strength|novelty_canary" .claude/tools/conjecture_triage.py    # MUST be >0
rg -nE "delta|band|verdict" .claude/tools/conjecture_triage.py | rg -c "kill"    # MUST be >0 (quantified tail present, E-2)
rg -c "P-25" .claude/skills/improvement-conjecture/SKILL.md                       # MUST be >0 (E-3)
python .claude/tools/python_wire_check.py --files .claude/tools/conjecture_triage.py --json  # no DARK new def
test -f workflow/reasoning/Q-14-decisions.md
```

## VERIFY

VERIFY: before declaring Q-14 done, the auditor runs off-agent —
`python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/Q-plan/Q-14_conjecture_engine_instrument_hardening.md --run-trigger`
`python .claude/tools/python_wire_check.py --files .claude/tools/conjecture_triage.py --json` (every new module-level def has a non-test CLI consumer, or a WIRE-DARK[Q-14] row)
`python .claude/tools/conjecture_triage.py --self-test-novelty-canary --json` (present canary)
plus the M2 scrambled-verdicts run showing ZERO emitted verdict rows and the M3 grandfathered-artifact `--validate`/`--fold` still green. Pass = the wire-checker clean, the canary present + fail-closed, the tail strictly optional, the tie-break reading only executed receipts, the P-25 citation adjacent. Green self-test alone is necessary-not-sufficient.

INTEGRATION_SMOKE: an INDEPENDENT agent drives the FULL engine stage-2/stage-4 on a HARD input — a real multi-candidate conjecture-run artifact — end-to-end: (a) `--novelty` with a minted verdicts file emits discriminated verdicts + `--self-test-novelty-canary` passes; (b) a scrambled verdicts file fail-closes to `instrument_strength=FAILED` with ZERO verdict rows; (c) `--validate` + `--fold` round-trip a within-noise quantified-kill artifact AND a grandfathered no-tail artifact; (d) `--novelty --profile safe` orders a validated_value-0 tie by measured token cost. A single-candidate toy run or a `--self-test`-only pass does not satisfy R10.

## REDTEAM

REDTEAM: attack that FAILED — "keep shipping UNCERTAIN(WEAK) verdicts and call the engine's NOVEL-CREDIBLE selector supported": FAILS because E-1 mints a real verdicts file and the canary fail-closes on a scrambled one, so an inert leg screams instead of silently passing. Attack that FAILED — "make the E-1 canary a unit test that calls the classifier directly": patched by anchoring the canary at the real `--novelty` verdict-emission path and by `--self-test-novelty-canary` failing when no canary is wired (R7). Attack that SUCCEEDED against an earlier framing — "make the KILL-EDGE tail mandatory so old artifacts stop validating": patched by making the tail strictly optional behind the ARTIFACT_VERSION gate; grandfathered 1.0/1.1 artifacts validate unchanged. Attack that FAILED — "smuggle a revise/iterate stage in under 'instrument hardening'": FAILS because the ABORT list bans any revise/iterate/ProbeUtility/multi-cycle stage (P-25 Family B + B-0/B-3 refutations bind). Attack that FAILED — "let the E-4 tie-break read an estimated cost so a favored candidate wins": FAILS because it reads only the stage-3 executed-receipt token count; no receipt -> id-order fallback, never an estimate.

## Grounding

- Evo/NeMo bidirectional study reasoning §3 (E-1..E-4) + the archived AIDE² run (workflow/reasoning/conjecture-run-aide2-deep-research-novelty-2026-07-15.md, all-UNCERTAIN(WEAK)). The AIDE² announcement's "most rejects die within noise" is [EXTERNAL] https://www.weco.ai/blog/first-evidence-of-recursive-self-improvement (vendor-reported), grounding E-2's DESIGN need, never an ELAI number. P-25 Family A/B refutations are ELAI-internal (E-3/EXCLUDED grounding).
- ELAI ground truth: conjecture_triage.py (--novelty WEAK degrade, --self-test 3 kills, no numeric KILL-EDGE tail, id-order SAFE tie-break, NOT gate-frozen), novelty.rs (NoveltyClass/NoveltyClassifiedEmitter), SKILL.md (exo-inject survival claim, no P-25 cite). Re-greppable (W-7).
- CLAUDE.md hard rules: blake3 for any receipt id; no .unwrap/.expect on prod; the engine tool stays advisory (never gate-manifest); GD-2 numbers recompute; SF-1 executed receipt > judge; SF-3 this plan is judged blind to the engine run artifact.

## COMPOSES-WITH

Q-11 C6 live-path canary + --self-test-canary (the runtime sibling E-1 mirrors on the harness side), novelty.rs NoveltyClass/NoveltyClassifiedEmitter (the composed runtime concept), the improvement-conjecture SKILL.md + conjecture_triage.py (the engine E-2/E-4 extend), the archived AIDE² run (the E-1 fixture + the E-2 quantified-kill precedent, C8). Re-implements NONE (GD-1). E-2 composes Q-15's S-2 refutation power floor (a within-noise kill = a NULL-UNDERPOWERED candidate).

## Open questions / risks

- The E-1 known-answer canary pair must be genuinely discriminating yet stable across archive growth — decide the pair at authoring against the live fold archive (RECON-1); a canary that trivially passes proves nothing.
- The E-4 measured cost is only available for candidates whose stage-3 leg executed (FORK-4 estimates carry no token receipt); the tie-break falls to id-order for the receiptless remainder, declared honestly (GD-5).
- E-3 is doctrine hygiene, not a build; its only risk is placement (the citation must sit beside the claim, not in a distant appendix).

## Next

Closes the harness-side instrument inertness (E-1) so the engine's one proven selector works on real runs, makes every future refutation reusable (E-2, composing Q-15's power floor), de-arbitrarizes the SAFE tie-break (E-4), and stops the exo-inject survival claim from being re-litigated (E-3). Pairs with Q-11 (the runtime canary sibling) and Q-15 (the bench/protocol hygiene that S-2's power floor completes).

End of Q-14.

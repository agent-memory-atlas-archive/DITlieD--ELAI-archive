EXPERIMENT PLAN — RLM vs ELAI-retrieval vs a merge, on free APIs (Liberty)
Date authored: 2026-06-26
For: a fresh implementer session. Self-contained. Runs entirely on free APIs via the Liberty harness; NO local model required.
Companion reasoning: workflow/reasoning/rlm-recursive-agents-grc-2026-06-26.md (PART 5 F1-F4, PART 6 verdicts). Interface maps: workflow/papers/rlm-interface.md (the RLM mechanism, GROUND TRUTH), longcot-bench-2604.14140-interface.md, mgh-blog-interface.md.

================================================================================
0. THE QUESTION THIS SETTLES (why we are running it)
================================================================================
RLM (Recursive Language Models, arXiv:2512.24601) reported a small model jumping 0% -> 100% on a long-context recall task using only a SCAFFOLD (no fine-tuning). ELAI's thesis is "a good harness beats a bigger model" — so this is on-thesis and too good to ignore. But ELAI does NOT use RLM's method (model writes Python to slice its own context); ELAI uses DETERMINISTIC retrieval (BM25/vector/rerank) + scoped sub-agents, because model-written code is non-deterministic, unverifiable, and (per ELAI rule J-55) collapses on small models. This experiment measures, on free APIs, FOUR concrete questions:
  Q1. On long-context tasks, does RLM (model-written slicing) actually beat ELAI-style deterministic retrieval? By how much, and only past what context length?
  Q2. Does a MERGE — deterministic retrieval to pre-filter, then a CONSTRAINED (not free-Python) slice/recurse primitive on top — match or beat RLM while staying deterministic/replayable?
  Q3. Does the small-model code-collapse (ELAI rule J-55) actually bite the RLM arm on a weak model, and does the constrained-primitive arm (Q2) sidestep it?
  Q4. Cost: tokens-per-correct-answer for each arm (RLM recursion can blow up spend).

DECISION OUTPUT: a 1-page result table + a verdict that either (a) justifies authoring an ELAI feature (a constrained within-step recursive-context primitive, J-42/J-86 sibling) + the LongCoT bench, or (b) confirms ELAI's deterministic retrieval already captures the benefit and no model-written-code feature is warranted.

================================================================================
1. THE FOUR ARMS (exact mechanism — implement each as a function arm(task, model) -> answer)
================================================================================
All arms answer the SAME question over the SAME long context. The only difference is HOW they get the relevant context in front of the model.

ARM A — FLAT BASELINE (what most agents do).
  Put the entire context into one prompt. If it exceeds the model's window, hard-truncate to the window (head+tail, the common default). One model call. Expected: fine when context fits, degrades/fails when context > window. This is the floor.

ARM B — RLM (replicate the paper, workflow/papers/rlm-interface.md is ground truth).
  The model NEVER sees the raw context. It sees only METADATA: type, total char/token count, and the available access methods. It is told the context lives in a Python variable `ctx` in a REPL, and it emits Python code to slice/search/inspect it across multiple turns, plus a recursion primitive:
    - `ctx` : the full context string (or list of chunks).
    - the model writes code like: `ctx[10000:14000]`, `[i for i,c in enumerate(chunks) if "deadline" in c.lower()]`, `len(ctx)`, regex search, etc.
    - `rlm_query(sub_prompt: str, sub_ctx: str) -> str` : spawns a CHILD model call (depth+1, own budget) on a sub-slice; at max_depth (set 2) it degrades to a plain single call. Root = orchestrator, child = solver. Communication = return strings only (paper Algorithm 1).
  The REPL runs the model's code and feeds stdout/return back; loop until the model emits a final ANSWER: line.
  SAFETY (HARD REQUIREMENT — this arm executes MODEL-WRITTEN CODE): run the code in a restricted namespace — `exec(code, {"__builtins__": SAFE_BUILTINS, "ctx": ctx, "chunks": chunks, "re": re, "rlm_query": rlm_query})`, where SAFE_BUILTINS is a small whitelist (len, range, enumerate, list, dict, str, slice ops, min/max, sorted, sum, any, all, print). NO import, NO open, NO __import__, NO eval, NO network. Hard per-step wall-clock timeout (e.g. 5s) and a max-steps cap (e.g. 12). A code error is fed back to the model as the observation (it retries) — this is also how we MEASURE the small-model code-collapse (count code-error steps).

ARM C — ELAI-STYLE DETERMINISTIC RETRIEVAL (ELAI's actual approach; no model-written code).
  Chunk the context (fixed size, e.g. 500-token sliding windows, the ELAI BM25 default). Deterministically retrieve the top-k chunks for the question using BM25 (and, optional second pass, an embedding retriever via a free embedding API). Concatenate the top-k chunks (sized to fit the window) + the question into ONE call. The model only answers from the retrieved chunks; it never decides the slicing. Re-running with the same seed/input yields the SAME retrieved chunks (the determinism property — assert this).
  Retriever: use `rank-bm25` (VERIFY ON PyPI FIRST per anti-slopsquatting: `pip index versions rank-bm25` or pypi.org/pypi/rank-bm25/json; it is a tiny pure-python pkg). If you want the second channel, add a free embedding model from the Liberty catalog or a small local sentence-transformer ONLY if offline; otherwise BM25-only is a fair ELAI-floor for v1.

ARM D — THE MERGE (the proposed new method; the thing we might steal into ELAI).
  Two stages:
    Stage 1 (deterministic, ELAI's reproducible part): BM25 retrieve a CANDIDATE SET of top-N chunks (N chosen so the set FITS the model window). This bounds the surface and stays replayable.
    Stage 2 (constrained model-driven navigation, RLM's adaptive part WITHOUT free Python): expose a FIXED, SMALL operation set the model calls by NAME (not by writing code) over the candidate set:
        - grep(pattern)        -> chunk_ids whose text matches (deterministic)
        - expand(chunk_id)     -> the neighbors of a chunk (context around a hit)
        - read(chunk_ids)      -> the text of those chunks
        - sub_query(question, chunk_ids) -> a child model call on just those chunks (the bounded recursion)
        - answer(text)         -> finish
    The model emits ONE operation per turn as a structured tool call (or a parseable single line), the harness executes it deterministically and logs it. Every operation is journaled (mirrors ELAI's RRA) so the whole run is replayable — re-running yields the same op sequence for the same model output. This is NARROW enough to plausibly sit below the J-55 code-collapse threshold (one named op per turn, no multi-step code), and deterministic/verifiable (the ops are fixed primitives, not arbitrary code). THIS ARM IS THE HYPOTHESIS: RLM's benefit, ELAI's guarantees.

================================================================================
2. TASK / DATASET (two phases — fast spike first, real number second)
================================================================================
PHASE 1 — SYNTHETIC MRCR-STYLE (build first; controllable, no external dep, deterministic).
  A generator you control so you can SWEEP context length precisely:
    - Build a long context = filler paragraphs (lorem-style or sampled from a fixed corpus) with K planted "needle" facts at known positions, e.g. "The access code for vault {id} is {value}." (vary id/value per task; seed the RNG so it is reproducible).
    - The question requires retrieving 1-3 specific needles and optionally combining them (e.g. "What is the access code for vault 7?" or "Sum the codes for vaults 3 and 9.").
    - Ground-truth answer is known exactly -> EXACT-MATCH scoring (deterministic, no LLM-judge — house rule TD-3).
  Sweep context length in three bands relative to the chosen model's window W: {0.5*W (fits), 2*W (overflows), 8*W (far overflows)}. ~20 tasks per band. This is the load-bearing knob: RLM/retrieval only matter past W.

PHASE 2 — LongCoT (the real, citable number; do AFTER Phase 1 shows the mechanism works).
  arXiv:2604.14140, repo github.com/LongHorizonReasoning/longcot (MIT), HF dataset LongHorizonReasoning/LongCoT (CONFIRMED live by ingest 2026-06-26). Adopt a SMALL SLICE (e.g. 50-100 items of LongCoT-Mini) and its official metric. Same four arms. This produces the number ELAI would actually cite (ELAI-measured, never the paper leaderboard — ID-5). Keep the harness identical to Phase 1; only the task loader + scorer swap.

================================================================================
3. MODEL LAYER (Liberty — grounded 2026-06-26)
================================================================================
Liberty is at /workspace/ELAI\.liberty (Python, LiteLLM Router over free tiers). Use its venv + .env keys + catalog so quota rotation + self-heal are free.
  - Setup: cd .liberty ; .venv\Scripts\python.exe -m pip install -e . (already installed); ensure .env has at least GROQ_API_KEY + one other (docs/FREE_PROVIDER_KEYS.md). Keyless Pollinations overflow is always present as a fallback.
  - List models: .venv\Scripts\python.exe -m liberty models  (gives the FREE_MODELS catalog + groups: big / fast-small / gemini-flash / mistral / openrouter-free / overflow-pollinations).
  - CALL SEAM: simplest is to import Liberty's router and call litellm directly:
        from liberty.router import build_router
        from liberty.catalog import FREE_MODELS
        router = build_router()              # LiteLLM Router with rotation + self-heal
        resp = await router.acompletion(model=<catalog_id>, messages=[...], max_tokens=...)
    The HealingProxy + QuotaTracker handle 429s/cooldowns transparently. (Alternative: monkeypatch LibertyAgent._call_llm, but for a controlled benchmark a direct router.acompletion is cleaner and avoids the agent loop's compaction.)
  - PER-MODEL WINDOW: read the served model's context window from the catalog (Liberty already tracks served_model windows, agent_loop.py:382-385/458-467) to set W for the length sweep and to size Arm A truncation + Arm C/D top-k.
  TWO MODELS (the J-55 axis):
    - WEAK: a <=9B free model from the catalog (e.g. a Groq llama-3.1-8b-class or a fast-small group id — confirm exact id via `liberty models`). This is the one that should expose the RLM code-collapse if J-55 is right.
    - STRONG/HIGH-CONTEXT: a "big" group model, OR the user's free SWE-1.6 ~200k-context model (confirm its catalog id; if not in the catalog, add it to FREE_MODELS or call via litellm with its provider creds). The 200k window lets Arm A survive medium contexts and makes the >window bands meaningful only at 8*W.
  Run the full arm x band x model grid; ~20 tasks each. Cap max_tokens modestly (answers are short). Liberty's quota tracker (`liberty quota`) shows burn; if a provider exhausts, self-heal waits — let it.

================================================================================
4. HARNESS STRUCTURE (what to build — keep it one small package)
================================================================================
Put the experiment OUTSIDE .elai_cc (this is a research spike, not ELAI product code — project-boundary rule). Suggested: /workspace/ELAI\.liberty\experiments\rlm_vs_elai\  (rides Liberty's venv) OR a sibling workflow/experiments/ scratch dir with its own venv that pip-installs litellm + rank-bm25.
  files:
    tasks.py        # Phase-1 synthetic generator (seeded) + Phase-2 LongCoT loader + exact-match/official scorer
    model.py        # build_router() wrapper: call(model, messages, max_tokens) -> text + usage; window(model)
    arm_flat.py     # Arm A
    arm_rlm.py      # Arm B + the restricted REPL (SAFE_BUILTINS, timeout, rlm_query, max_depth=2)
    arm_retrieval.py# Arm C (rank-bm25 chunk+retrieve)
    arm_merge.py    # Arm D (BM25 candidate set + the 5 named constrained ops + op journal)
    run.py          # the grid driver: for model in [weak,strong]: for band in [.5,2,8]*W: for arm in [A,B,C,D]: for task: run, score, log tokens, log op/code-error counts
    results.md      # auto-written table + the verdict block
  Every run logs per (model,band,arm): accuracy, mean tokens-in/out, mean wall-clock, AND for Arm B the count of code-error steps + recursion calls, for Arm D the op-sequence length. Persist raw per-task JSONL so re-scoring is free.
  DETERMINISM ASSERTION: run Arm C and Arm D TWICE on a fixed task+seed; assert identical retrieved-chunk-ids / op-sequence (this is the property RLM lacks — it is the experiment's other half of the point). Arm B will differ run-to-run by design; record that as the non-determinism cost.

================================================================================
5. METRICS + THE RESULT TABLE
================================================================================
Primary: ACCURACY per (model, band, arm) — exact-match (Phase 1) / official metric (Phase 2).
Secondary: TOKENS-PER-CORRECT-ANSWER (cost) per cell; mean LATENCY; Arm-B code-error rate + recursion depth used; Arm-D op-count; determinism pass/fail (C/D must pass, B must fail).
Table shape (one per model):
    band \ arm |  A flat  |  B RLM  |  C retrieval  |  D merge
    0.5*W      |   ...    |   ...   |     ...       |   ...
    2*W        |   ...    |   ...   |     ...       |   ...
    8*W        |   ...    |   ...   |     ...       |   ...

================================================================================
6. DECISION CRITERIA (read the grid, write the verdict)
================================================================================
Q1 (is RLM better than ELAI retrieval): compare B vs C in the 2*W and 8*W bands.
   - B >> C (e.g. +15pp): RLM's model-driven navigation genuinely beats fixed retrieval on long inputs -> the capability is real, proceed to Q2/Q3.
   - B ~= C: ELAI's deterministic retrieval already captures it -> CONFIRM ELAI's thesis, NO model-written-code feature warranted (big result, ship the confirmation).
   - C > B: retrieval wins -> ELAI strictly better, done.
Q2 (does the merge win): compare D vs max(B,C).
   - D >= B AND D passes the determinism assertion: THE WIN — RLM's benefit with ELAI's guarantees. This is the feature to author into ELAI (a constrained within-step recursive-context primitive, J-42/J-86 sibling).
   - D < B: the benefit needs the FULL free-Python flexibility -> harder call (you would be trading determinism for accuracy; flag for the user, do NOT auto-adopt).
Q3 (does J-55 bite, does D dodge it): compare B-weak vs B-strong code-error rate, and D-weak vs B-weak accuracy.
   - B-weak code-error rate high AND B-weak accuracy collapses, BUT D-weak holds: CONFIRMS the constrained primitive sidesteps J-55 -> safe to ship D on the local lane. (This is the F4 hypothesis test.)
   - B-weak also works fine: J-55 may be over-conservative for narrow slice-code -> note it, the threshold deserves a re-measure, but do not change J-55 without a code-edit-task replication (J-55 was measured on multi-tool code-mode, not slice-code — keep that caveat).
Q4 (cost): if B/D win accuracy but cost 5-10x the tokens of C, that is a real tradeoff -> report tokens-per-correct, let the cost ceiling (ELAI H-9/E-39) decide per lane.

================================================================================
7. COST / SAFETY / SCOPE GUARDS
================================================================================
- Free APIs only; small grid (2 models x 3 bands x 4 arms x ~20 tasks = ~480 task-runs Phase 1; Arm B/D multiply by their step count). Cap max_tokens; let Liberty self-heal on 429. Run Phase 1 first; only do Phase 2 if Phase 1 is promising.
- Arm B executes model-written code: the restricted-exec sandbox (no import/open/net, whitelisted builtins, timeout, max-steps) is NON-NEGOTIABLE. Never run Arm B code unsandboxed. (This is literally the risk that makes ELAI wary of RLM — model it honestly.)
- Scoring is DETERMINISTIC exact-match / official metric — NO LLM-as-judge (TD-3).
- This is a research spike in .liberty / a scratch dir. Do NOT write any code into .elai_cc. The ELAI feature (if Q2 says yes) is authored SEPARATELY via paper-to-plan phase 5 as a J/K unit, measured-gated by this result.
- Anti-slopsquatting: verify rank-bm25 (and any embedding pkg) on PyPI before install. litellm is already a Liberty dep (no new add).

================================================================================
8. DELIVERABLES (what the implementer session returns)
================================================================================
1. The harness package (files in section 4) + results.md with the two grids (weak/strong) filled.
2. A VERDICT block answering Q1-Q4 in plain language + the recommended ELAI action (author the constrained primitive + LongCoT bench, OR confirm-and-stop).
3. Append the verdict to workflow/reasoning/rlm-recursive-agents-grc-2026-06-26.md PART 7 (new), keyed to this experiment.
4. If Q2 = WIN: a one-paragraph spec for the ELAI feature (the constrained op-set + the journal/replay shape + the J-55 measured clearance) ready to hand to paper-to-plan phase 5.

================================================================================
9. FASTEST PATH (if the implementer is time-boxed)
================================================================================
Minimum decisive run: ONE weak model, Phase-1 synthetic only, bands 2*W and 8*W, all four arms, 15 tasks each. That single grid already answers Q1 (B vs C), Q2 (D vs B), and Q3 (does weak-B collapse, does D hold). ~120 task-runs, an hour of free-API time with Liberty's rotation. Expand to the strong model + Phase-2 LongCoT only if the minimum run is promising.

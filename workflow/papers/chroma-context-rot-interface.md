Source: Context Rot: How Increasing Input Tokens Impacts LLM Performance
URL: https://www.trychroma.com/research/context-rot
Authors: Kelly Hong, Anton Troynikov, Jeff Huber
Published: July 2025
Reliability: HIGH (Chroma primary research, 18 models tested, reproducible benchmark methodology)

CORE THESIS:
LLM performance degrades unevenly and unpredictably as input context grows, even on simple tasks.
All 18 frontier models tested show monotonically decreasing accuracy with context length.
The degradation is not uniform: task type, distractor count, haystack structure, and needle-question semantic similarity all modulate the degradation curve independently.

KEY FINDINGS / MECHANISMS:

FINDING 1 - UNIVERSAL DEGRADATION:
"all 18 models degrade with length"
Models tested: Claude Opus 4, GPT-4.1, Gemini 2.5 Pro, Qwen3-235B + 14 others.
F1 scores decrease monotonically with input length across all model families.
Steepest degradation: 100,000-500,000 token range (for large-window models).
A 200K-token context window can show serious accuracy loss at only 50K tokens of input.

FINDING 2 - LOST-IN-THE-MIDDLE EFFECT:
Relevant content in positions 5-15: ~45-55% accuracy.
Relevant content at position 1: ~75% accuracy.
Relevant content at position 20: ~72% accuracy (U-shaped curve).
30%+ accuracy drop from middle vs. start/end positions.
"unique words up front get nailed better than those buried deep"

FINDING 3 - NEEDLE-QUESTION SEMANTIC SIMILARITY (critical for retrieval design):
"As needle-question similarity decreases, model performance degrades more significantly with increasing input length"
High-similarity (lexical match) needles: relatively robust with context growth.
Low-similarity (semantic/non-lexical match) needles: dramatically faster degradation.
Implication: retrieval systems relying on semantic (non-exact) match are more vulnerable to context rot than keyword search.

FINDING 4 - DISTRACTORS:
"Even single distractors reduced performance relative to baseline."
"distractors do not have uniform impact" - some distractors cause far greater degradation than others.
Multiple distractors compound degradation.
Claude models: lowest hallucination rates when facing ambiguous choices.
GPT-3.5: ~60% refusal rate at extended contexts.
GPT family hallucination rate: ~2.55%.

FINDING 5 - HAYSTACK STRUCTURE (counterintuitive):
"models perform worse when the haystack preserves a logical flow of ideas"
"Models perform better on shuffled haystacks than on logically structured ones"
Consistent across "all 18 models and needle-haystack configurations."
Proposed mechanism: structured text activates attention patterns that work against retrieval.

FINDING 6 - LONGMEMEVAL FOCUSED vs. FULL:
Full prompt: ~113,000 tokens. Focused prompt: ~300 tokens.
"significantly higher performance on focused prompts compared to full prompts"
Claude Opus 4: most pronounced gap due to conservative abstentions at full context.
Implication: pre-filtering before model call dramatically outperforms giving the model everything.

FINDING 7 - REPEATED WORDS TASK (synthetic ceiling test):
Even trivial sequential replication degrades with context length.
Failure modes: incorrect positions, random word generation, refusals.
Tested at 25-10,000 word lengths.
Position accuracy is highest when unique words appear early.

MODEL FAMILY SUMMARY:
Claude: slowest degradation, lowest hallucination rate, but abstention behavior at large context.
GPT: erratic/inconsistent at scale, highest refusal rates, ~2.55% hallucination.
Gemini: fastest degradation, largest performance swings from 500-750 words onward.
Qwen: steady degradation; larger variants (235B) perform better but still degrade.

VERBATIM EXTRACTS (anti-hallucination):
"model performance varies significantly as input length changes, even on simple tasks"
"LLM performance degrades unevenly and often unpredictably as contexts grow"
"all 18 models degrade with length"
"models perform worse when the haystack preserves a logical flow of ideas"
"Models perform better on shuffled haystacks than on logically structured ones" [across all 18 models]
"significantly higher performance on focused prompts compared to full prompts"
"distractors do not have uniform impact"
"As needle-question similarity decreases, model performance degrades more significantly with increasing input length"
"effective long-context performance requires careful context engineering"
[NOTE: The paper presents most degradation as charts, not inline percentages. The 30%/75%/45-55% numbers come from secondary coverage and the "Lost in the Middle" prior work the paper cites, not verbatim from the Chroma text itself.]

REIMPLEMENTATION NOTES FOR ELAI:

CONTEXT BUDGET POLICY:
The evidence supports treating ~25-32K (ELAI's usable local window) not as a comfortable ceiling but as a zone where the sidecar already operates near the risky end of the curve.
Concrete knob: set a context_rot_budget_words = 5000 words (~6500-7000 tokens) as the soft ceiling for any single retrieval-augmented prompt before aggressive trimming. Above this threshold the paper's curves show consistent degradation onset.
The 30%+ accuracy drop at middle positions means ELAI's compaction logic should place the most critical content (active task, current instruction, key invariants) at positions 1 and N (head and tail), never buried in the middle.

COMPACTION POLICY:
The focused-vs-full result (~300 tokens focused > ~113K full) is the strongest single number for justifying aggressive pre-filtering. ELAI's context-lifecycle compaction (H-50) should target selective reload, not keep-everything, even when window capacity is technically sufficient.
Recommended: treat context_budget as a quality knob (not just a crash-prevention knob). Compaction should fire well before the hard window limit, at the quality-degradation threshold.

RETRIEVAL CHUNK COUNT:
The distractor finding means each additional retrieved chunk is a potential distractor. More retrieved chunks != better recall. A policy of retrieve-top-K with K capped at 3-5 semantically coherent chunks is better than returning K=10-20 chunks and letting the model figure it out.
The haystack-structure finding means retrieved chunks should be shuffled or presented in reverse-relevance order if the haystack is coherent text (counterintuitive but paper-grounded).

SEMANTIC SIMILARITY SENSITIVITY:
For ELAI's BM25+vector hybrid retrieval (crates/knowledge/src/bm25.rs + vector/indexer.rs): the finding that semantic/non-lexical match degrades faster implies the BM25 (lexical) leg should be weighted more heavily when context is long. At short context (<5K tokens), semantic similarity works. At long context (>15K tokens), lean on BM25.

NEEDLE POSITION STRATEGY:
Place retrieved evidence immediately after the system prompt (position 1 region) and immediately before the user turn (position N region). Never place key retrieved context in the middle of a long prompt.

GROUNDED CHECKS (testable assertions):

assert context_word_count < 7000, "context_rot_budget exceeded: degradation onset expected above ~7000 words based on Chroma study"
assert retrieved_chunks_count <= 5, "distractor_ceiling exceeded: each additional chunk is a potential distractor per Chroma finding"
assert key_content_position in ('head', 'tail'), "lost_in_middle: critical content must not be buried in middle positions 5-15"
assert focused_prompt_tokens < 1000 or full_prompt_tokens < 10000, "prefer_focused_prompt: Chroma shows 300-token focused > 113K-token full for QA"
assert haystack_structure != 'coherent_sorted' or shuffle_applied, "shuffled_haystack: logically structured haystacks degrade performance per Chroma finding"

FAILURE MODES AND CAVEATS THE SOURCE NAMES:
- Benchmarks are primarily visual (charts); most exact percentages are from secondary coverage, not paper text.
- Degradation curves are model-specific; no universal threshold applies.
- The shuffled-haystack finding is counterintuitive and may not generalize to all task types.
- The study tests frontier models (Claude 4, GPT-4.1, Gemini 2.5) that differ architecturally from ELAI's local Qwen2.5-Coder-7B sidecar; extrapolation to small local models is plausible but unverified.
- The paper does not isolate whether degradation is from attention dilution, positional encoding limits, or softmax normalization at scale.
- Refusal behavior (Claude abstaining, GPT refusing) conflates capability degradation with policy behavior.
- "Unique distractor impact" is noted but not explained mechanistically.

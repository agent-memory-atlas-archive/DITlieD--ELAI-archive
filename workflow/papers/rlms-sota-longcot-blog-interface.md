Source: RLMs are SOTA on LongCoT (blog post)
URL: https://raw.works/rlms-are-sota-on-longcot/
Published: 2026-04-19
Author: Unknown (raw.works blog; author identity not stated in fetched content)
Reliability: LOW (not peer-reviewed; author identity unknown; single-run results; no methodology section; no error bars)
Raw file: workflow/papers/rlms-sota-longcot-blog-2026-04-19.html

Context / Prior work referenced:
- Same author's prior post: Sonnet 4.5 + DSPy.RLM = 45.4% on LongCoT-Mini (noted as "expensive")
- This post explores whether smaller open models can match or exceed that with DSPy.RLM

Benchmark: LongCoT-Mini and LongCoT-Full
- LongCoT-Mini: 507 problems (the count used in Qwen 3 8B run; baseline for leaderboard ranking)
- LongCoT-Full: 2500 problems
- Leaderboard referenced but not linked; rankings cited without URL

Harness: DSPy.RLM (Raymond Weitekamp's implementation)
- No fine-tuning applied to tested models unless explicitly noted
- MIT OASYS RLM finetune mentioned but results for it not specified in the post

MEASURED RESULTS (single runs; no error bars; no variance reported):

Qwen 3 8B Instruct on LongCoT-Mini (507 problems):
  Vanilla LLM (no RLM): 0/507 = 0%
  DSPy.RLM: 33/507 = 6.5%
  Leaderboard position: would rank #7 on LongCoT leaderboard at time of writing
  NOTE: smallest model on that leaderboard was GLM-4.7 (358B total / 32B active, MoE architecture)
  Significance claimed: a 8B model without fine-tuning reaching leaderboard placement via RLM scaffold alone

Qwen 3.5 9B on LongCoT-Mini and LongCoT-Full:
  DSPy.RLM on LongCoT-Mini: 17.2% -> 3rd place on leaderboard
  DSPy.RLM on LongCoT-Full (2500 questions): 15.69%
  GPT-5.2 on LongCoT-Full: 9.83%
  Ratio: Qwen 3.5 9B DSPy.RLM / GPT-5.2 = approximately 1.6x on LongCoT-Full
  NOTE: GPT-5.2 is assumed to be a base (non-RLM) comparison; post does not clarify whether GPT-5.2 used RLM or not

Qwen 3.5 27B-Instruct on LongCoT-Full:
  DSPy.RLM: 22.18%
  Comparison: more than doubles GPT-5.2's 9.83% on LongCoT-Full
  NOTE: "more than doubles" is approximately correct (22.18 / 9.83 = 2.26x) -- author's arithmetic checks out

SOTA Claim:
"DSPy.RLM with Qwen 3.5 9B is comfortably SOTA on the full LongCoT"
- This claim is relative to the leaderboard at time of writing (2026-04-19)
- Leaderboard is not linked; cannot independently verify rankings
- Author was running additional tests with Qwen 3.6 35B-A3B at time of writing (results not yet in this post)

Key structural point supported by these numbers:
- RLM scaffold (no fine-tuning) on a small open model (8-9B) outperforms a frontier closed model (GPT-5.2) on the full LongCoT benchmark
- The scaffold effect is large enough to overcome a very large model-size gap

Navigation context (post series structure):
- Prior post: "LongCoT - A benchmark worthy of a RLM's attention"
- This post: "RLMs are SOTA on LongCoT"
- Next post linked: "RLMs are the new reasoning models"
- The raw.works post (2026-04-19) precedes the alexzhang13 post (2026-04-26) in the series

UNVERIFIED CLAIMS:
- Author identity: unknown; raw.works has no author attribution in fetched content
- Leaderboard rankings (#7, 3rd place): leaderboard URL not provided; cannot verify independently
- GPT-5.2 baseline: whether this is RLM-wrapped or base GPT-5.2 is not stated; comparison validity depends on this
- "Comfortably SOTA" at time of writing: stale by now; no current leaderboard checked
- MIT OASYS RLM finetune: mentioned but results not provided; relevance unclear
- Qwen model version names (3.5 9B, 3.5 27B): Qwen versioning scheme uses non-standard numbering; "3.5" may not correspond to any public Qwen release name -- UNVERIFIED against public Qwen release history
- LongCoT-Full 2500 questions: count not independently verified (LongCoT-Mini uses 507 in this post)

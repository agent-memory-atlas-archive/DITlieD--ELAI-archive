DEEP RESEARCH PROMPT — deterministic task decomposition + cheap source-grounding
For: GPT deep research / Perplexity. Anonymized (no project identifiers).
Created 2026-06-16. Two questions + an attack-the-design section.

--------------------------------------------------------------------------------
PASTE BELOW THIS LINE
--------------------------------------------------------------------------------

I am designing an autonomous software-engineering agent system. The system takes
a high-level goal and recursively decomposes it into a tree (concepts -> phases
-> steps -> small atomic work-units), then executes each leaf unit with an LLM.
I want to remove as much judgment from the LLM as possible and replace it with
deterministic, measurable machinery. I need a rigorous, source-backed survey on
two specific problems. Prefer peer-reviewed papers, real open-source
implementations, and benchmark results over blog opinion. Cite everything with
links; flag anything that is a vendor claim or unbenchmarked. Distinguish "this
is established and measured" from "this is plausible but unproven."

QUESTION 1 — Deterministic / programmatic decomposition and PRE-EXECUTION cost
estimation for agent work-units.
  a) What prior art exists for decomposing a software task into bounded
     sub-units WITHOUT relying on an LLM's free judgment to decide the size and
     count of the units? Include program-analysis-based decomposition (call
     graphs, dependency/impact analysis, change-coupling), classical software
     task/work-breakdown estimation, and any agent frameworks that use a
     deterministic sizer rather than letting the model choose.
  b) Can the token / context cost of an agent work-unit be PREDICTED before it
     runs, accurately enough to gate on? Survey methods for estimating the
     input-context size and output size of a coding sub-task ahead of execution
     (e.g. from files-touched, dependency-cone size, retrieved-context size,
     number of acceptance criteria). How accurate are these estimators in
     practice, and what is the error distribution?
  c) "Right-sizing" a unit to a model's hard context ceiling: is there research
     on admission control / re-splitting a unit when its predicted context
     exceeds a fixed budget, and on routing oversized units to a
     larger-context tier? What are the known failure modes?
  d) Is there a measured relationship between a model's used-context fraction
     and its task success rate (context degradation curves), and has anyone
     used such a curve to set a per-model work-unit size budget rather than a
     hardcoded threshold?

QUESTION 2 — Cheap / deterministic grounding of generated text to a retrieved
source, WITHOUT a second large LLM in the loop.
  a) Extractive provenance via fingerprinting: how sound and how useful is it to
     fingerprint a fetched source's spans (rolling hash / Rabin-Karp, shingling,
     MinHash, suffix automata) and then check whether a model's output spans are
     extractively present? At what granularity does this become meaningful
     (token vs n-gram vs sentence) and at what granularity is it vacuous? What
     are the documented false-positive / false-negative behaviors?
  b) Claim-level grounding without a frontier LLM: what is the current best for
     decomposing an answer into atomic claims and checking each claim is
     entailed by a source, using a SMALL/distilled model or a deterministic
     method (e.g. lightweight fact-checking / entailment models, attributed-QA,
     citation/quote verification)? Give concrete model names, sizes, latencies,
     and accuracy numbers where published.
  c) Closed-set citation binding: research on freezing the candidate evidence
     set BEFORE generation and constraining the model so every citation must be
     a member of that frozen set (making fabricated/"ghost" citations
     structurally impossible). Does this measurably reduce attribution errors,
     and what does it cost in answer quality?
  d) The architectural split where the model only FORMULATES queries and RANKS /
     SELECTS pages, while the harness performs the actual fetch and the
     grounding check runs against the harness-fetched (not model-reported)
     content. Is there prior art and evidence that this reduces hallucinated or
     fabricated retrieval content versus letting the model call fetch tools
     directly?

ATTACK THE DESIGN (argue against me — this section is mandatory)
  - Make the strongest case that pre-execution token/context cost estimation for
    coding sub-tasks is fundamentally unreliable, and say why. What is the
    irreducible variance?
  - Argue that trying to make decomposition deterministic is misguided and that
    the LLM's judgment is actually load-bearing here. Where does pure
    program-analysis decomposition break down on real codebases?
  - Make the case that extractive span-fingerprint grounding is a dead end for
    anything but plagiarism detection, and that paraphrase/synthesis make it
    useless for verifying generated answers. Where is the line?
  - Identify any place where my "remove LLM judgment, use deterministic checks"
    premise has been TRIED and FAILED in published systems, and what went wrong.
  - Name the single biggest risk in each of the two designs above and the cheap
    experiment that would falsify it.

OUTPUT: for each sub-question, give (1) the established/measured answer with
citations, (2) the best real implementation to study, (3) the honest residual
uncertainty. End with a short "what I would build first and why" given that the
priority is determinism and small-local-model viability under a hard context
ceiling.

---
name: oracle
description: Interactive grounded Q&A. Triggers when user types /oracle [question], /ask [question], or asks any deep factual question requiring web grounding (current API behavior, library comparison, framework patterns, App Store rules, recent ecosystem changes, etc). Spawns the researcher agent in oracle mode (deep, human-readable) and returns a citation-backed answer. Read-only — works in any FSM state.
license: MIT
compatibility: "Designed for Claude Code. Works on Codex with explicit $oracle invocation."
metadata:
  version: "1.0.0"
  author: "elai"
---

The Oracle gives you a grounded answer to any question. Not a Google search summary. Not a training-memory recap. A multi-source, tier-classified, contradiction-surfacing, citation-backed answer to the specific question you asked.


WHEN TO INVOKE

Trigger conditions:
- user types `/oracle <question>` or `/ask <question>` directly
- user asks a question that requires up-to-date factual grounding: "what is the current Expo SDK version", "does StoreKit 2 support family sharing for non-renewing subs", "is the new architecture default in RN 0.84"
- main LLM is uncertain about a technical claim and needs to verify before answering with confidence
- main LLM is about to make a recommendation that depends on current API behavior or library state

When NOT to invoke:
- the question is conversational ("how was your weekend") — answer directly
- the question is about local code state (the answer is in the codebase, not on the web) — use Read / Grep / map instead
- the question is about workflow conventions defined in CLAUDE.md or .claude/rules/ — answer from those files directly
- the question is about prior conversation context — recall from session memory
- the question is too broad to answer in one research call ("how do i build an app") — convert to a tier classification (likely IDEA-tier, fire idea-intake instead)


PROTOCOL

1. Capture the question. If user typed `/oracle` or `/ask`, take everything after the command. Otherwise restate the inferred question and confirm with the user before spending tokens: "I'll check this via Oracle: '{restated question}'. Confirm or refine ?"

2. FSM state. Oracle works in ANY state (read-only). Do not transition. Do not session-claim. The answer is informational; no project files are touched by the oracle skill itself.

3. Generate cache key. Slug the question + current year + stack: `oracle-{slug-3-5-words}-{year}` (e.g. "oracle-storekit2-family-sharing-2026").

4. Spawn researcher agent (.claude/agents/researcher.md) with:
   - mode = "oracle"
   - question = the captured question
   - session_id = current session
   - node_id = "oracle/{timestamp-slug}" (e.g. "oracle/2026-05-09T14-30-storekit2-family")
   - current_app = from current-app.txt (used for stack context if relevant)
   - cache_key = the slug from step 3
   - breadth = 5-7 (oracle gets full breadth)
   - depth = 1 (default; researcher promotes to 2 if HIGH-risk topic and contradictions detected)
   - prior_attempts = optional, if user re-asks similar question

   The researcher operates per its mode=oracle floor: minimum 5 WebSearches, minimum 10 WebFetches, minimum 3 sub-questions. Paper-ingestor or source-fetcher spawned per its triggers.

5. Wait for researcher's return. Read the formatted answer the researcher inlined (per ORACLE OUTPUT FORMAT in researcher.md).

6. Present the answer to the user. Do NOT paraphrase. Show the researcher's structured output verbatim. The user wanted grounded; reformatting it adds your interpretation layer which dilutes the grounding.

7. Surface warnings:
   - CONTRADICTIONS: if the researcher flagged unresolved disagreement between sources, highlight to user
   - STALE-RISK: if any cited source is > 18 months and the topic is annual-cycle (RN, Expo, iOS, SwiftUI)
   - FLOOR-NOT-MET: if researcher could not meet oracle-mode search floor, the answer is shallower than usual — caveat
   - UNVERIFIED PACKAGES: if any package mentioned in the answer was not registry-verified, flag

8. Cache hit / miss reporting. If the researcher used a cache hit (mcp_search_context), tell the user: "This answer drew on prior research from {cache key, timestamp}." If fresh research, say "Fresh research, cached for future re-use."

9. Follow-up questions. The user may ask a follow-up. If it is a refinement of the same question (e.g. "OK but specifically for SDK 51"), spawn researcher again with prior_attempts including the previous cache_key — researcher avoids re-searching what was already covered. If the follow-up is a different question, fresh oracle call with new cache_key.


OUTPUT FORMAT (presented to user)

The researcher has already formatted the oracle output. The Oracle skill just presents it. Format reminder (the researcher's ORACLE OUTPUT FORMAT block produces this):

```
ORACLE ANSWER — {question}
Asked: {timestamp}
Mode: oracle (deep + human-readable)

THE SHORT ANSWER
{1-2 sentences}

THE DETAILED ANSWER
{2-4 paragraphs with inline source citations}

EVIDENCE BREAKDOWN
{per-sub-question summary}

CONTRADICTIONS / DISAGREEMENTS
{both sides if any}

CAVEATS
- {staleness, confidence, open questions}

CITATIONS
{numbered list of all sources with tier markers}

CACHE
Stored under topic: {cache_key} for future re-use.
```

Plus the warnings appended after.


COST DISCIPLINE

A single Oracle call with mode=oracle floors:
- ~5-10 WebSearches (~10K tokens)
- ~10-15 WebFetches (~50-100K tokens of fetched content, mostly discarded after extraction)
- 1 researcher agent invocation
- Possible paper-ingestor or source-fetcher spawns (each adds ~50-100K tokens of subagent context)

Total cost: ~$0.50-2.00 per Oracle call on Sonnet. Time: ~1-3 minutes wall clock.

This is the right cost ceiling for "i need a grounded answer." If the user is asking trivial questions repeatedly, suggest they use cached prior answers (mcp_search_context with topic prefix "oracle-").


EXAMPLES

User: "/oracle does Expo SDK 52 support React Native 0.85"
> Oracle fires. Researcher generates 5 sub-questions: SDK 52 release date, RN versions in SDK 52, RN 0.85 release date, breaking changes between RN 0.84 and 0.85, Expo official compatibility statement. Searches 7 sources (4 tier-1 from docs.expo.dev + reactnative.dev, 3 tier-2 from RN release notes + Expo blog). Returns: short answer, detailed answer, citations, no contradictions found, all sources < 6 months old.

User: "is StoreKit 2 family sharing supported for non-renewing subs"
> Inferred as oracle-worthy (current API behavior, App Store specific). Restate to user, confirm, then fire. Researcher generates sub-questions about Transaction.shared, FamilyShareable protocol, non-renewing vs renewing distinction, App Store Connect config. Returns answer: yes for renewing, no for non-renewing (with citation to Apple developer docs).

User: "what is the best way to handle deep links in Expo"
> Oracle-worthy. Multiple approaches exist (expo-linking, expo-router universal links, custom URL scheme). Researcher surfaces all 3, compares, cites sources. User picks based on their app's needs.

User: "what does my project's auth look like"
> NOT oracle-worthy. The answer is in the local codebase. Use Read on src/auth/*.ts + map/depGraph.json instead.


CONSTRAINTS

- never spawn Oracle for questions that the local codebase + map can answer. Read the map first.
- never paraphrase the researcher's output. The grounding contract relies on faithful citation.
- never fire Oracle silently — always surface to the user that you are about to spend ~$1 on research, give them a chance to refine the question or cancel.
- never cache low-quality answers (FLOOR-NOT-MET) as if they were full research — the cache key gets a "_partial" suffix to avoid future cache hits incorrectly trusting partial data.


END ORACLE SKILL

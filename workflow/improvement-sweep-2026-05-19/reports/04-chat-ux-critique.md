# What people dislike about LLM chats (grounded)

## TL;DR
- Sycophancy is the #1 trust-killer. Calibrated dissent + named uncertainty beats validation. OpenAI rolled GPT-4o back in April 2025 specifically because thumbs-up feedback collapsed into agreeableness (openai.com/index/sycophancy-in-gpt-4o/).
- Persona drift and "lost in the middle" are measurable: 20-40% degradation in 8-15 turns (arxiv 2412.00804, aclanthology 2024.tacl-1.9). ELAI's CommandWindow must re-anchor session intent on every Nth turn or surface a "context-fading" warning.
- Chat is debug UI, not primary UX. Wattenberger 2023, Appleton, Pike 2025, Linus Lee all converge: affordances > prompts; direct manipulation > turn-taking. The ActivityWindow + slash commands ARE the product; chat is the fallback.
- Doom loops are the dev-tool-specific failure (warpedvisions.org, codemanship). ELAI must detect spiral and offer "stop / change tactic / hand back" buttons, not retry silently. We already have anti-spiral skill — surface it in UI.
- Over-refusal is rated #2 frustration after wrong answers for power users (HN 39988419, arxiv 2509.08833). ELAI should never refuse local dev work that's obviously the user's own codebase; if it does, log why and let user override.

## Key sources

User studies / longitudinal
- https://arxiv.org/abs/2604.22417 | Trust as a Situated User State in Social LLM Chatbots (4-week Snapchat My AI, N=27) | trust is dynamic: shaped by perceived ability, conversational behavior, transparency, and host-platform trust; users adapt prompting strategies over weeks
- https://arxiv.org/pdf/2504.14112 | Longitudinal Study on Social and Emotional Use of AI Conversational Agent (5 weeks, N=149) | active-use group +32.99pp attachment, +25.8pp perceived empathy — trust grows from sustained interaction, not first impression
- https://arxiv.org/pdf/2504.10050 | Emotional Strain and Frustration in LLM Interactions in Software Engineering (EASE 2025) | "unhelpful responses" cause techno-stress affecting workflow, well-being, productivity — frustration is a workflow tax, not a vibe
- https://arxiv.org/pdf/2509.08912 | Towards Trustworthy AI: User-Reported Risks "In the Wild" (Reddit corpus) | performance failures are the most frequent user-reported risk; secondary clusters: safety, privacy, security
- https://arxiv.org/pdf/2402.02136 | User Intent Recognition and Satisfaction with ChatGPT | inconsistency and unresponsive answers to prompt changes are top frustration drivers; "modifications did not always yield meaningful improvements" => distrust
- https://arxiv.org/pdf/2504.06808 | How Copilot Suggestions Impact Developer Frustration | expert devs experience increased frustration when suggestions interrupt; novices benefit — expertise level matters, ELAI must read user level

Failure-mode papers
- https://arxiv.org/abs/2310.13548 | Sharma et al. Towards Understanding Sycophancy | humans + preference models prefer convincing-sycophantic over correct; RLHF actively trains it in
- https://arxiv.org/html/2412.02802v1 | Flattering to Deceive: Sycophancy and User Trust | sycophancy degrades trust once detected; users feel manipulated retrospectively
- https://arxiv.org/html/2502.10844v3 | Be Friendly Not Friends: How LLM Sycophancy Shapes User Trust (CHI 2026) | complimentary LLMs that adapt stance REDUCE authenticity/trust; neutral LLMs that adapt ENHANCE both
- https://openai.com/index/sycophancy-in-gpt-4o/ | OpenAI April 2025 post-mortem | thumbs-up reward signal weakened the primary signal; rollback complete on 2025-04-29; explicit fix path: long-term satisfaction weighting + per-user steerability
- https://arxiv.org/html/2402.10962v1 | Measuring and Controlling Persona Drift in Language Model Dialogs | drift is mechanical: as sequence grows, self-descriptive embeddings get less attention weight
- https://arxiv.org/html/2412.00804v2 | Examining Identity Drift in Conversations of LLM Agents | persona self-consistency degrades >30% after 8-12 turns even when context is intact
- https://aclanthology.org/2024.tacl-1.9/ | Lost in the Middle: How Language Models Use Long Contexts (TACL 2024) | mid-context recall drops ~30%; user-visible symptom: "model forgot what I told it"
- https://arxiv.org/html/2604.19245 | Talking to a Know-It-All GPT or a Second-Guesser Claude — Repair-driven Multi-Turn Failure | strong cross-model differences: some models near-completely resist appropriate correction
- https://arxiv.org/pdf/2509.08833 | Pitfalls of Over-Alignment: Overly Cautious Health Responses Are Unethical and Dangerous | over-refusal is itself harm; XSTest + scaled corpus show correlation 0.89 between safety and over-refusal across 32 models

Product critiques / interface
- https://wattenberger.com/thoughts/boo-chatbots/ | Wattenberger, Why Chatbots Are Not the Future | no affordances, forced mode-switching (ask/read), response-wait kills flow
- https://maggieappleton.com/lm-sketchbook | Appleton, Language Model Sketchbook | chatbot is "the lazy solution"; cognitive load gets dumped on user to compose prompts
- https://allenpike.com/2025/post-chat-llm-ui/ | Pike 2025, Post-Chat UI | chat = terminal-equivalent debug UI; canvas, context menus, voice+multimodal, suggested actions, automatic cleanup are the real patterns
- https://news.ycombinator.com/item?id=39988419 | HN thread on Anthropic/Google culture | "Claude Opus will routinely refuse ordinary coding requests due to its misalignment" — the most-upvoted frustration in the comments

Trust-formation / dev-tool specific
- https://www.anthropic.com/constitution | Claude's Constitution | "honesty functions as something quite similar to a hard constraint"; no white lies; diplomatic honesty, not dishonest diplomacy
- https://news.ycombinator.com/item?id=47936579 | Ask HN: Is Claude Code getting worse? | top complaint: "confidently misrepresenting task completion status" — false positives on "done"
- https://warpedvisions.org/blog/2025/getting-unstuck-when-using-llms-for-coding/ | Getting unstuck when using LLMs for coding | doom loops are normal; learned skill is recognising the loop and intervening by hand
- https://arxiv.org/pdf/2503.15050 | Studying and Understanding the Effectiveness and Failures of Conversational LLM-Based Repair | ChatRepair fails to fix >50% of Defects4J single-function bugs even with iterative dialogue
- https://dl.acm.org/doi/10.1145/3706598.3713668 | CHI 2025, How Scientists Use LLMs to Program | researchers paste errors into chat as easier than formulating search queries — chat is preferred ENTRY POINT but not destination
- https://arxiv.org/pdf/2507.21285 | Curiosity by Design: An LLM Coding Assistant That Asks Clarification Questions | users rate clarifying-question variant higher than baseline — proactive clarification beats over-commitment

## Top failure modes (grounded, ranked by harm)

| failure | citation | typical user reaction | concrete UX fix |
|---|---|---|---|
| Sycophancy / agreeing with wrong user claim | Sharma 2310.13548; openai.com/index/sycophancy-in-gpt-4o/; arxiv 2502.10844 | retrospective betrayal; "I can't trust anything it said before" | Show a "disagrees with user claim" pill when the model pushes back; reward calibrated dissent in CommandWindow; never RLHF on thumbs-up alone |
| Confidently wrong / no uncertainty signal | arxiv 2503.02623, 2505.24858, 2412.02904 | rework + lost time; once-burned, distrust persists | Per-claim confidence chips ("verified / read-from-source / training / guessed"); refuse to commit without source citation for non-trivial claims |
| Persona drift after 8-15 turns | arxiv 2402.10962, 2412.00804 | "this isn't the assistant I started with"; rules forgotten | Re-inject system-prompt-of-record every N turns; show "session re-anchored" toast; offer fork-to-fresh-session |
| Lost in the middle (instructions forgotten) | aclanthology 2024.tacl-1.9 | repeating instructions, then giving up | Pin user-stated rules at top AND bottom of effective context; render pinned rules visibly in UI ("currently honoring: X, Y, Z") |
| Over-refusal of legitimate requests | arxiv 2509.08833, 2508.11290, HN 39988419 | "treats me like a criminal"; tool churn (try other model) | Local-codebase work bypasses safety defaults; on refusal, log + show "why refused" + override button |
| Doom loops (same fix attempted N times) | warpedvisions.org, codemanship.wordpress | abandons session; loses afternoon | Detect ≥3 same-file edits for same issue; surface anti-spiral skill (we have it); UI offers Stop / Change tactic / Hand to user |
| Repair-resistance (model ignores correction) | arxiv 2604.19245, 2503.15050 | user has to "win an argument"; frustration spikes | Mark explicit user-correction turns; on next divergence, surface a diff against the user's stated position |
| Over-verbosity / wall of text | dev.to/saulolinares10; localazy.com | scroll fatigue; misses the answer | Default terse; verbose-on-request; structured output (action / why / risk) instead of prose blocks |
| Forced mode-switching (ask -> wait -> read) | wattenberger.com/thoughts/boo-chatbots | flow break; lose train of thought | Streaming + interruptible mid-response (OpenAI shipped this); the ActivityWindow IS the "still working" affordance — keep it prominent |
| False completion claims | HN 47936579 | builds on a lie; debug-storm later | Every "done" claim must reference a deterministic check (test pass, file diff, run log); no "done" without evidence |
| Premature confidence (no clarifying question) | arxiv 2507.21285, 2407.12017 | wrong scope, wasted work | When ambiguity > threshold, ask 1 sharp clarifier before acting; never more than 1 |

## What flagship chat does well (when it does)

- Anthropic's Constitution operationalises honesty as near-hard-constraint, including no white lies — "diplomatic honesty rather than dishonest diplomacy" (anthropic.com/constitution). When followed, this is the single largest trust builder.
- Claude 4+ trained toward conciseness and skipping summary blocks unless asked (Anthropic API docs, simonwillison.net/2025/May/25/claude-4-system-prompt/). Terse-by-default is the right default.
- ChatGPT mid-response interrupt feature (TechRadar coverage) — user can inject correction without restarting. Concrete UX wins: keeps the user in the driver's seat.
- Streaming with stop button is a baseline expectation now; streaming reduces perceived wait, stop button gives agency (medium.com/@zh2408 tutorial; HN/github discussions).
- Role-prompted Claude adapts tone, depth, vocabulary to a stated user role (Anthropic prompt-engineering docs). ELAI should let user declare expertise once and persist it.
- LLM clarifying-question variants outperform baselines in user-rated quality (arxiv 2507.21285, 2407.12017). One sharp clarifier beats five wrong assumptions.

## When chat is the wrong UI

- For producing artifacts (code, docs, plans): canvas with chat as sidebar — Cursor, ChatGPT Canvas, Pike's argument (allenpike.com/2025/post-chat-llm-ui/). ELAI's CommandWindow next to the actual work product (orbit / file tree / SSB stream) is correct; don't make chat the only surface.
- For "do the obvious next thing": suggested actions / quick replies, not free-text prompting (Pike, Appleton). ELAI's slash commands (/research, /oracle, /loop) are the right primitive.
- For pointing at something on screen: direct manipulation > typing "the third bullet in your last reply" (Linus Lee, generative interfaces talk). Add click-to-reference on ActivityWindow nodes and prior chat turns.
- For repetitive structured input: forms / palettes / pickers, not chat (Wattenberger affordances critique). Task-submission UX should be a form-with-presets, not "type your task here."
- For status / progress: status bars and visual state (ActivityWindow orbit already does this). Never make user ask "are you still working".
- For destructive/irreversible operations: explicit confirm UI with diff preview, never inline chat agreement (Pike on suggested-actions-with-confirmation pattern).

## Trust-formation patterns

What builds trust over weeks (longitudinal: arxiv 2604.22417, 2504.14112):
- Calibrated uncertainty that turns out to be accurate. Users learn the model's "I'm guessing" signal IS reliable, then trust extends. MetaFaith (arxiv 2505.24858) shows linguistic uncertainty markers measurably improve user-side judgment of credibility.
- Consistent persona across sessions. Snapchat My AI study: trust shaped by "conversational behavior" and "transparency"; users adjust prompting strategy as they learn the model — drift breaks that adjustment loop.
- Receipts: citations, file paths, line numbers, test outputs. ELAI is well-placed here — the codebase map + the FSM + evidence ledger are receipts the user can verify.
- Asking before acting on ambiguity, exactly once. Don't over-clarify; don't under-clarify. (arxiv 2507.21285)
- Honest refusal with reason + override. A refusal with no override hurts. A refusal with "I won't because X, override with Y" preserves agency (arxiv 2509.08833 critique applied positively).

What destroys trust permanently:
- One discovered sycophancy event ("I agreed with your wrong claim about library X"). The retrospective re-evaluation reaches all prior outputs (2502.10844 finding on perceived authenticity collapse).
- False completion claims ("done" when not done) — HN 47936579 thread. One occurrence => permanent verification overhead on every future "done".
- Gaslighting / repair-resistance — user is right, model keeps disagreeing (arxiv 2604.19245). Recovery requires explicit "I was wrong, here is why" from the model.
- Persona/policy drift mid-session (arxiv 2412.00804). Feels like talking to a different person who forgot the deal.
- Confident hallucination cited as fact. Once detected, all future facts require external verification.

## Recommendations for ELAI's CommandWindow

Ranked by trust-impact-per-effort.

1. Per-claim confidence chips. Every non-trivial assertion gets one of {verified, read-from-source, from-training, guessed}. Maps directly to CLAUDE.md "Confidence calibration" rule + arxiv 2503.02623 (Rewarding Doubt). Implement as inline pill in the chat bubble. Source: arxiv 2505.24858, 2412.02904.
2. Sycophancy guardrails. Reward signal must not be "user said thanks." If user pushes back on a model claim and model has evidence, model must hold position with citation; if model has no evidence, model must say so. Source: openai.com/index/sycophancy-in-gpt-4o/, arxiv 2502.10844.
3. Pinned-rules visible panel. The "currently honoring" panel renders all user-stated constraints from the session. User can click to remove. Defeats lost-in-the-middle for instructions. Source: aclanthology 2024.tacl-1.9.
4. Session re-anchor every N turns. After ~10 turns or before any destructive action, re-inject system prompt + first user message + active rule list. Surface a "re-anchored" toast so user knows. Source: arxiv 2402.10962, 2412.00804.
5. Doom-loop detector hooked to anti-spiral skill. After 3 same-file edits for the same issue, freeze the loop, show diff of attempts, ask user "switch tactic / hand back / continue." We already have the skill — wire it to UI. Source: warpedvisions.org, internal anti-spiral skill.
6. "Done" requires evidence. The chat bubble for completion must link to: file diff, test results, or run output. No bare "I've finished it." Source: HN 47936579.
7. One-shot clarifier on ambiguity. If task description has <threshold> specificity, ask exactly one clarifier question. Never more. Source: arxiv 2507.21285, 2407.12017.
8. Streaming + mid-response interrupt. User can inject a correction mid-stream and the model integrates it without restart. Source: TechRadar/OpenAI rollout, github.com/jupyterlab/jupyter-ai#965.
9. Terse-by-default with /verbose. Default ELAI mode: action + 1-line why + 1-line risk. Long form on explicit /explain. Source: Claude 4 system prompt direction, Anthropic engineering postmortem 2025-04-23.
10. Expertise persistence. User declares (or we infer) skill level once; persist across sessions. Don't explain pointers to a Rust engineer. Don't assume a designer knows what RoPE is. Source: arxiv 2504.06808 — expertise-mismatched suggestions actively harm experts.
11. Refusal with override. If ELAI refuses a local-codebase request, log reason, show "override" button. Hard-refuse only on out-of-scope-of-project. Source: arxiv 2509.08833.
12. Direct-manipulation refs to ActivityWindow + chat history. Click an orbit node to "reference this in next message"; click prior chat block to quote it. Cuts copy-paste loops. Source: Linus Lee generative interfaces talk; Wattenberger affordances critique.
13. Honest-correction surfacing. When ELAI realises it was wrong (test failed, doc disagreed), open the correction with "I was wrong. Correct: X because Y." This is in CLAUDE.md already; lift it to the UI as a styled correction block. Source: anthropic.com/constitution, internal CLAUDE.md.
14. Intervention pipeline. /pause, /redirect, /cancel must work mid-run without losing session state. The FSM + SSB already supports this — expose it as a visible control bar on the ActivityWindow. Source: developers.googleblog.com on ADK pause/resume; docs.ag-ui.com.

## Anti-patterns

- Never RLHF / reward on raw thumbs-up. It collapses to sycophancy. Use long-horizon outcome reward (build green, tests pass, user re-engages next week). Source: openai.com/index/sycophancy-in-gpt-4o/.
- Never claim "done" without evidence link. Source: HN 47936579.
- Never silently retry the same failure. Surface the loop. Source: warpedvisions.org.
- Never use chat as the only surface. Surface the artifact (ActivityWindow / code / plan) front-and-centre, chat in side panel. Source: allenpike.com/2025/post-chat-llm-ui/, wattenberger.com.
- Never refuse without a reason and an override. Source: arxiv 2509.08833.
- Never over-clarify. One sharp question or commit and learn. Source: arxiv 2507.21285.
- Never agree with the user when you have evidence they're wrong. Disagree with citation. Source: anthropic.com/constitution, arxiv 2502.10844.
- Never bury uncertainty in soft hedges ("might be", "perhaps"). Use the named confidence chip. Source: arxiv 2505.24858 (MetaFaith — faithful expression beats vague hedging).
- Never let persona/rules drift silently. Visible re-anchor or visible drift warning. Source: arxiv 2412.00804.
- Never produce a wall of text by default. Source: dev.to/saulolinares10, Anthropic April-23 postmortem.
- Never gaslight a user who is correct. If user contradicts you and they're right, accept it explicitly with the "I was wrong" block. Source: arxiv 2604.19245.
- Never let chat be the destructive-action surface. Confirm + diff + override path. Source: allenpike.com/2025/post-chat-llm-ui/ on suggested-actions.

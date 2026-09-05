Reference coding-agent CLI — feature digest (neutral, clean-room)

Purpose: a product-level description of features observed in a commercial
multi-provider coding-agent CLI (publicly documented BYOK + usage behavior),
written so the steal-analysis step can reason about WHICH features to adapt
into ELAI under ELAI-native names. This is reference-only and weak-grounding:
never cite it as ground truth, never copy any name or identifier from it,
invent ELAI-native names for anything adapted. Grounding for any ELAI plan
comes from ELAI's own code + the goose maps (file:line), not this file.

Scope note: only the three focus areas are kept here — (1) token/usage
metering + usage dashboard, (2) multi-provider BYOK + provider normalization,
(3) local + mid-session model handling. Unrelated capabilities of the
reference tool are intentionally omitted.

------------------------------------------------------------
1. PER-TURN USAGE TELEMETRY (the core of the token window)
------------------------------------------------------------
The reference tool records, on EVERY model turn, a small usage record with
these distinct counters:
  - input_tokens            (prompt tokens sent this turn)
  - cache_read_tokens       (prompt tokens served from the provider cache)
  - cache_creation_tokens   (prompt tokens written into the provider cache)
  - output_tokens           (tokens generated this turn)
  - stop_reason             (why generation ended)
Key idea worth adapting: cached vs non-cached prompt tokens are tracked as
SEPARATE counters, not folded into one "input" number — because they bill
differently and the user wants to see cache effectiveness. ELAI already has
cache_read / cache_creation fields in its usage struct; the adaptable part is
surfacing them per-turn AND rolling them up per session / per task.

------------------------------------------------------------
2. USAGE ROLLUP + DASHBOARD (the marketing-surface image)
------------------------------------------------------------
The reference tool exposes a usage view answering "where did the tokens go".
It is fed by ONE usage-aggregation endpoint and rendered as charts:
  - tokens (and a billing-unit value) broken down BY MODEL
  - an autonomy ratio (share of work done without human approval)
  - tool-call counts and file-operation counts
  - per-user / per-org rollups (org-effectiveness view)
The unit shown to the user is an abstract billing unit (a "credit"-style
number) layered over raw tokens, so the display is provider-agnostic.
Adaptation target for ELAI: a single usage-rollup surface keyed by
(user, model, task, tool, file-op), fed from ELAI's journal, rendered in the
dedicated usage window + chat status line + canvas. ELAI already has the
producer half of this (per-backend cost accrual + a per-phase/agent/tool $
rollup) but it has no UI consumer yet — the adaptation is mostly activation +
a per-(user,model,task) aggregation + a real-time stream surface, plus
deciding the abstract unit (raw tokens vs a normalized credit).

------------------------------------------------------------
3. PROVIDER ABSTRACTION + BYOK
------------------------------------------------------------
The reference tool normalizes ONE internal tool/function spec
{name, description, input_schema} at call time into each provider's native
wire shape (Anthropic messages, OpenAI chat-completions, OpenAI responses,
Bedrock converse, Gemini). One internal representation, many adapters.
BYOK behavior:
  - the user supplies their own key per provider; a provider can also run
    through the vendor's own proxy when no user key is set
  - a "next provider" selector picks among configured providers, and once a
    turn starts it LOCKS to one provider for the rest of that turn/sequence
    (so a mid-sequence switch does not split a conversation across wire
    formats without an explicit serialize/re-summarize step)
  - base-URL override per provider for self-hosted / OpenAI-compatible
    endpoints
Adaptation target for ELAI: ELAI already has a multi-backend trait + factory
(the provider abstraction exists) — the BYOK work is mostly EXTENSION:
per-provider key entry/storage, base-URL override, "active provider"
selection + lock, and the one-spec-to-N-wire-formats normalization where ELAI
does not already cover a wire shape.

------------------------------------------------------------
4. LOCAL + MID-SESSION MODEL HANDLING
------------------------------------------------------------
Two behaviors worth adapting:
  - mid-session model escalation: a tool/affordance that swaps the active
    model UP mid-task (e.g. to a stronger model for a hard step) without
    restarting the session; usage telemetry continues to accrue across the
    swap, attributed per-model.
  - a per-turn model selector that scores candidate models for the next turn
    (ELAI already has routing for this — E-31 / H-7 / calibrated routing —
    so this is reference-confirmation, not a new build).
Local-lane caveat (the genuinely hard part, from ELAI's own audit, NOT the
reference): a local model server may report usage with different semantics
than a cloud API (e.g. full-prompt vs suffix-only accounting, streaming vs
non-streaming divergence). A unified usage window must NORMALIZE local and
cloud counters separately before display — never naively sum them. Confirm
the local server's usage block + the client read path at author time.

------------------------------------------------------------
5. CONTEXT-LIMIT HANDLING (reference-confirmation only)
------------------------------------------------------------
On hitting the context limit the reference tool summarizes older messages,
saves a compaction anchor + removed-count, and retries. ELAI already has
session-DAG trim + compaction ladder, so this is confirmation, not a steal.

------------------------------------------------------------
WHAT TO ADAPT vs SKIP (for the token-window + BYOK + local focus)
------------------------------------------------------------
ADAPT: separate cached/non-cached per-turn counters surfaced live; a single
usage-rollup keyed by (user, model, task, tool, file-op) rendered in a
dedicated window + chat + canvas; an abstract provider-agnostic display unit;
per-provider BYOK key entry/store/override + active-provider lock;
one-spec-to-N-wire-format normalization for any uncovered provider; mid-session
model escalation with per-model usage attribution.
SKIP / ALREADY-HAVE: per-turn model routing (E-31/H-7), context compaction
(session-DAG), the provider trait itself (B-37). Local/cloud usage
NORMALIZATION is an ELAI-specific design problem, not a copy.

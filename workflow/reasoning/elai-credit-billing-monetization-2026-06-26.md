ELAI MONETIZATION — CREDIT-BASED SEATS ON A LITELLM/OPENROUTER RAIL
2026-06-26. Mode: spec-eval AS-SPECIFIED (ELAI planned substrate counted as DONE).
Grounded-by-default: external rails verified via WebSearch (OpenRouter docs, LiteLLM
docs, cited inline); ELAI substrate from docs/thefinalglance/ELAI-FEATURE-MAP.md; the
GPT note that seeded this is treated as ONE vote (CLAUDE.md gpt-second-opinion: never a
grounding source) and its claims were re-grounded below. Anti-sycophancy holds.

================================================================================
0. THE MODEL IN ONE LINE
================================================================================
Subscription price -> monthly CREDITS (denominated in real provider $) -> credits
spent per actual metered usage -> ELAI pays the upstream provider -> ELAI controls
rate limits, rollover, and routing. This is the Nous Portal / OpenRouter shape. It is
SAFE (no "unlimited" promise that an agent loop can detonate) because spend is metered
and capped per request. ELAI's differentiator is NOT a magic provider deal — it is that
ELAI RESOLVES MORE TASKS PER DOLLAR of model spend (local-lane routing + verification +
caching), so a $20 credit pool on ELAI goes materially further than $20 of raw frontier
API. That claim is the whole pitch and it MUST be SAFEDELTA-measured before it is said.

================================================================================
1. GROUNDED MECHANICS OF THE TWO RAILS
================================================================================
OPENROUTER (openrouter.ai/docs/faq, /docs/guides/overview/auth/byok, /pricing):
  - Passes through underlying provider pricing with NO markup — you pay the same per-
    token rate as going direct.
  - Charges a 5.5% fee ($0.80 minimum) when you PURCHASE credits (5% crypto).
  - BYOK (user brings their own provider key, routed through OpenRouter's unified API):
    costs 5% of what the model would normally cost, deducted from OpenRouter credits;
    waived for the first 1M BYOK requests/month (5M on enterprise).
  - Pay-as-you-go credits with optional auto-top-up; one API key -> 200+ models.
  IMPLICATION: OpenRouter is a clean managed PROVIDER RAIL + billing meter, but ELAI's
  cost of a credit dollar is ~1.055x face (the purchase fee), and ELAI gets NO arbitrage
  on token price (no markup either way). So margin cannot come from reselling tokens; it
  must come from CONSUMING FEWER expensive tokens per resolved task.

LITELLM PROXY (docs.litellm.ai/docs/proxy/{virtual_keys,users,cost_tracking}):
  - Self-hosted OpenAI-compatible gateway to 100+ providers.
  - Virtual keys with per-key budgets (multiple windows e.g. 24h/30d), rpm/tpm rate
    limits, spend tracking, an admin UI.
  - HARD budget enforcement validates spend against an AUTHORITATIVE DATABASE before
    admitting a request (spend lives in LiteLLM_VerificationTokenTable / its Postgres).
  CRITICAL CATCH FOR ELAI: LiteLLM's authoritative spend store is ITS OWN DB. If ELAI
  leans on LiteLLM's budget enforcement, the financial source of truth becomes LiteLLM's
  Postgres, NOT ELAI's SSB journal — a direct GD-2 violation (journal is the record,
  never an external store) and a split-brain billing risk. Also: LiteLLM is a Python
  service, and J-54 already recorded a "HAND-ROLLED Rust, no litellm/proxy" decision for
  the free-rotation pool. So LiteLLM is at most a STATELESS routing convenience, NEVER
  the credit ledger.

================================================================================
2. WHAT ELAI ALREADY HAS (most of the substrate is planned — this is assembly)
================================================================================
J-28 TOKEN LEDGER / USAGE UNIFICATION — the METER. UsageTurnEvent
  (identity/lane/backend/model/task/session + elapsed_ms), NormalizedUsage lane-semantics
  (sidecar full-prompt vs cloud suffix-only, never naively summed), SpendAccumulator
  repriced off models.toml, session rollup, tagged estimator fallback, `elai usage` CLI.
J-29 USAGE WINDOW / LIVE METER — in-process LiveUsageGauge (mid-stream token meter),
  per-model/task/session/lane breakdown, cache effectiveness, derived autonomy ratio.
J-49 MULTI-SEAT ADMISSION + PER-SEAT QUOTA — SeatRegistry (seats.toml DATA), weighted-
  token-bucket FAIR-SHARE admission in front of routing, per-seat SPEND CEILING reusing
  the C-5 caps, per-seat rollup over UsageTurnEvent.identity. This IS the seat/plan spine.
J-59 PER-SEAT MEMORY + TRUST ISOLATION — per-seat namespacing, cross-seat isolation.
H-9 COST COCKPIT — CloudSpend rows, per-response input/output/CACHED breakdown,
  $/RESOLVED-TASK (the pitch's unit of measure already exists).
C-5 SPEND CEILINGS / CAPS. J-81 MID-LOOP DETERMINISTIC COST CAP — aborts the CURRENT
  step on cost/iteration/wallclock overrun (the runaway-loop kill switch).
J-54 FREE-API ROTATION POOL — FreePool RuntimeBackend (per-key daily quota, 429 cooldown,
  pool-exhaustion backoff, keyless overflow) = the near-zero-COGS lane.
J-55 MODEL-STRENGTH MODALITY ROUTER — CodeAct for capable lanes, constrained-call+I-36
  schemas for the sub-14B LOCAL lane (the keystone that makes local models actually work).
J-30 BYOK + B-37 MULTI-RUNTIME (11 backends, RuntimeOrchestrator, PerRoleRuntime).
I-51 STATIC models.toml CAPABILITY CATALOG. F-4 can't-lie + RRA + SAFEDELTA + the I-18/
  J-69 SSB hash-chain + J-70 IV&V independence (the audit-grade journal).
NET: ELAI has the meter (J-28), the seat/quota/spend (J-49), the caps (C-5/J-81), the
cockpit + $/resolved-task (H-9), the free lane (J-54), the local-usefulness engine (J-55),
and the audit-grade journal. It does NOT yet have: a CREDIT LEDGER, SUBSCRIPTION/REFILL,
a MANAGED (ELAI-pays) provider rail, ROLLOVER, the PAYMENT boundary, and the pre-flight
reserve/settle accounting. That gap is the new work; ~70% of the plumbing exists.

================================================================================
3. THE NEW LAYER (concrete, on the SSB spine)
================================================================================
CREDIT LEDGER = an SSB sub-journal, NOT a mutable counter and NOT an external DB.
  A seat's balance is a FOLD over append-only credit rows; it is never stored as a number
  that can drift or be double-spent. New Pattern-2 SSB action_type CreditLedgerEvent with
  Kind { Granted (subscription refill / bonus / promo), Reserved (pre-flight hold),
  Settled (actual cost booked, releases the hold delta), Refunded, Expired (rollover cap),
  AdjustmentManual }. Hash-chained (I-18/J-69) so the spend history is tamper-evident —
  this is a genuine ADVANTAGE for billing disputes: an audit-grade, replayable money
  journal is exactly what you want, and ELAI's spine already provides it. Single-writer
  SSB serialization is the double-spend defense (no two requests can both spend the last
  credit). balance(seat) = sum(Granted) - sum(Settled) - sum(open Reserved) - sum(Expired).

THE PER-REQUEST LOOP (GPT's 8 steps mapped to real ELAI seams):
  1. ESTIMATE cost: expected_tokens x model_rate from J-28 SpendAccumulator + I-51
     models.toml (tagged-estimator fallback for unknown). Reserve a conservative hold.
  2. ADMIT: J-49 admission checks balance >= hold AND fair-share AND rate/concurrency for
     the seat's tier. Fail-closed -> queue or refuse with a clear "out of credits" surface.
  3. RESERVE: emit CreditLedgerEvent::Reserved (the hold). Balance reflects it immediately.
  4. CALL provider via the rail (section 4); J-29 LiveUsageGauge meters mid-stream; J-81
     mid-loop cap aborts if a runaway blows the hold before completion.
  5. READ actual usage from the provider response (OpenRouter returns real cost; local =
     metered tokens at ~0 price).
  6. SETTLE: emit CreditLedgerEvent::Settled(actual); release (hold - actual) back. This is
     the J-28 deduction made authoritative against the ledger.
  7. PRICE/DISCOUNT: actual provider $ (no markup on the credit value); plan bonus is
     applied at GRANT time (the 10%), not per request — simpler and audit-clean.
  8. STOP: if balance hits 0 mid-task, J-81 + J-49 halt the agent at a safe boundary with a
     resumable handoff, never a hard kill mid-edit.

ROLLOVER + REFILL: subscription renewal (payment webhook, section 5) emits Granted(face +
  bonus) and Expired(amount over the tier's rollover cap). Rollover cap is per-tier DATA
  in seats.toml (W-2, no new flag per tier). The whole capability is behind ONE default-OFF
  ELAI_BILLING_ENABLE with a wire-dark-allowlist row + promotion plan (R9).

================================================================================
4. PROVIDER RAIL — TWO APPROACHES, RECOMMENDATION
================================================================================
APPROACH A (RECOMMENDED) — OpenRouter as the single MANAGED rail; ELAI hand-rolls the
  Rust client as a B-37 RuntimeBackend; the CREDIT LEDGER is ELAI's SSB.
  One upstream API -> 200+ models incl. small/cheap ones; OpenRouter returns actual cost
  per call which J-28 books as Settled. Consistent with B-37 (backend pattern), J-54
  (hand-rolled, no litellm), I-51 (models still curated in models.toml — see the tension
  below), and GD-2 (ELAI's SSB is the only financial source of truth). ELAI's cost of a
  credit-$ is ~1.055x (OpenRouter purchase fee); margin comes from efficiency, not markup.
  Pros: one integration, cleanest GD-2 story, fewest moving parts, no Python service.
  Cons: single-vendor dependency on OpenRouter (mitigate with a direct-Anthropic/OpenAI
  fallback backend, which B-37 already supports — bedrock/azure direct lanes exist).

APPROACH B — LiteLLM proxy as a self-hosted multi-provider gateway.
  Buys virtual-keys + per-key budgets + 100+ providers + an admin UI "for free." But: its
  budget enforcement and spend store are an EXTERNAL authoritative DB (GD-2 violation if
  used as the ledger), it is another Python process and supply-chain surface, and J-54
  already rejected litellm for the free pool. LEGITIMATE NARROW USE: a STATELESS local
  router if/when ELAI needs many DIRECT provider keys at once and does not want to hand-
  roll each client — but spend tracking/admission/ledger MUST stay in ELAI's SSB; LiteLLM
  would be a dumb pipe, its budget feature OFF. For v1, not worth it: OpenRouter already
  aggregates 200+ models behind one API, so Approach A gets the breadth without the proxy.
RECOMMENDATION: Approach A now (OpenRouter rail + SSB ledger + B-37 direct fallback);
  reconsider a STATELESS LiteLLM router only if a future need for many simultaneous direct
  keys appears, and even then never as the ledger.

THE I-51 TENSION (thread it, do not re-litigate): OpenRouter as a PROVIDER/BILLING RAIL is
  NOT the same as OpenRouter's DYNAMIC MODEL REGISTRY, which ELAI rejected 3x (I-51, goose,
  local-serving). The model CATALOG stays ELAI-curated in models.toml (capabilities,
  pricing, lane-fit). OpenRouter is only the transport+meter for the models ELAI already
  chose to offer. Stated this way it composes cleanly and respects the prior decision.

================================================================================
5. THE PAYMENT BOUNDARY (out of the harness, into the ledger)
================================================================================
Payment processing (Stripe/Paddle) is OUTSIDE ELAI — the harness never touches card data.
The only seam: a payment webhook (subscription created/renewed/cancelled) emits a
CreditLedgerEvent::Granted (or stops future refills) into the SSB. Failed payment ->
no Granted -> balance naturally runs down -> J-49 admission refuses at 0. Chargeback ->
AdjustmentManual(negative) + seat suspension flag. This keeps PCI scope at zero and keeps
the ledger the single source of truth for entitlement. Per-seat identity provenance defers
to J-2 (the seat-id is already the J-28 UsageTurnEvent.identity).

================================================================================
6. THE PITCH: "$20 SHIPS $200 RESULTS" — GROUNDED, NOT SLOGANEERING
================================================================================
The honest mechanism (this is real, and it is ELAI's existing efficiency+trust thesis,
memory project_efficiency_trust_engineering / project_gtm_playbook_2026):
  - ELAI's edge is $/RESOLVED-TASK (H-9), not $/token. A cheap or local model wrapped in
    ELAI's verification spine (RRA, grounding modes, auditor, wiring gates, I-4 search
    ladder selecting by RECEIPTS not by burning samples, belief-state, abstention) resolves
    a task that a raw frontier model also resolves — but for far fewer expensive-provider $.
  - The LOCAL LANE is the margin engine. Work routed to the self-hosted sidecar / J-54 free
    pool costs ELAI ~0 marginal $ and draws ~0 credits. J-55 is what makes that lane
    actually produce correct results (constrained-call + schemas for the sub-14B model). So
    the more ELAI can correctly route to local/cheap, the further a $20 credit pool stretches
    — THAT is the literal mechanism of "$20 = $200", not a discount and not "unlimited."
  - Therefore a Plus seat's $22 credits deliver more RESOLVED TASKS than $22 of raw frontier
    API, by the measured efficiency multiplier. The multiplier is the product; it must be a
    REPRODUCIBLE SAFEDELTA NUMBER on a real task envelope (Aider-style), per the GTM
    "believe-the-number" decision — never an asserted "10x".
HONEST WORDING for the pricing page (mirrors the safe form, avoids the loop-detonation
  trap GPT correctly flagged):
  "Plans include monthly credits for model and tool usage, metered at provider cost. ELAI
  routes work to the cheapest model that can verifiably do the job — often a local one — so
  your credits resolve more tasks per dollar. Higher tiers add rate limits, concurrency, and
  larger rollover." NEVER "unlimited Sonnet/Opus/GPT."
PRIOR-CAUTION FLAG (anti-sycophancy, do not bury): the 2026-06-02 market work
  (project_market_positioning_2026) advised DROPPING the bare cost/cheap framing and re-
  aiming at devs/prosumers on TRUST. The efficiency pitch here is compatible ONLY if it is
  sold as "proven more value per dollar, verified" (a trust claim backed by SAFEDELTA), not
  as "cheap." If the number isn't there when measured, the pitch reverts to trust/quality
  and the credit model still stands on its own as a safe metered-usage plan.

================================================================================
7. SUGGESTED TIERS (DATA in seats.toml; numbers illustrative, set by real COGS)
================================================================================
  FREE   : BYOK (J-30) + local sidecar + J-54 free pool. ELAI COGS ~0. Full trust spine.
           (This is ELAI's wedge: the free tier is genuinely useful because of the local lane.)
  PLUS   $20/mo  -> $22 credits (10% bonus), small rollover cap, normal rate limits, N agents.
  PRO    $100/mo -> $110 credits, bigger rollover, higher concurrency/priority queue.
  ULTRA  $200/mo -> $220 credits, highest rate limits, priority workers, largest rollover.
ALT ANCHOR (GTM memory: first buyer = vibe-cleanup AGENCIES @ $39-49/seat): a team/agency
  seat tier with pooled credits + per-seat J-49 fair-share + the audit-grade spend journal
  as a SELLING POINT (reproducible per-task cost receipts) may monetize before consumer tiers.

================================================================================
8. ADVERSARIAL — THE CHEAPEST FALSIFIERS
================================================================================
  - RUNAWAY AGENT LOOP burning a seat dry (GPT's top risk): ELAI is UNUSUALLY well-defended
    — J-81 mid-loop deterministic cost cap aborts the current step, J-49 pre-flight admission,
    the autonomous-envelope dispatch cap, and the Reserved-hold-before-call all exist. This is
    a SELLING POINT vs naive credit products: ELAI structurally cannot detonate a balance.
  - ESTIMATE INACCURACY: the pre-flight hold must be conservative (over-reserve, settle down)
    or a cheap estimate under-reserves and admits a request that then overspends. Use a
    per-model worst-case hold from I-51; settle to actual. The hold is released same-tick.
  - DOUBLE-SPEND / RACE: single-writer SSB serialization is the defense; balance is a fold,
    never a mutable counter. Two concurrent requests cannot both consume the last credit
    because Reserved rows serialize through the one writer.
  - GD-2 / SPLIT-BRAIN: the single biggest way to ship this wrong is to let OpenRouter's or
    LiteLLM's spend store be the source of truth. It is NOT. ELAI's SSB credit journal is
    authoritative; the provider's number is an INPUT booked as Settled, reconciled nightly.
  - MARGIN REALITY: OpenRouter purchase fee (5.5%) + the 10% bonus = ELAI eats ~15% on the
    frontier portion of spend. The economics ONLY work if a large fraction of work routes to
    the local/free lane (near-0 COGS). So J-55 routing quality is not a feature — it is the
    business model. If local routing quality is poor, the margin is negative; gate the
    launch on the measured local-lane resolve-rate, not faith.
  - VENDOR DEPENDENCY: OpenRouter outage = no paid lane. Mitigate with the B-37 direct-
    provider fallback backend (already supported) and the local lane as the always-on floor.
  - "UNLIMITED" TRAP: never offered. Metered credits + caps make agent-loop blowups the
    USER's bounded credit cost, never ELAI's unbounded liability.

================================================================================
9. PLAN-TRACK NOTE
================================================================================
This is a NEW plan track (a billing/commerce track; K is newest -> an L-track, or folded
into K), per the Section-5 extension rule. It COMPOSES J-28/J-49/J-54/J-55/H-9/C-5/J-81/
B-37/I-51/F-4/I-18-J-69 and ADDS: the CreditLedgerEvent SSB type + credit-fold balance,
the OpenRouter managed RuntimeBackend, subscription refill + rollover (seats.toml data),
the reserve/settle pre-flight loop, the payment webhook seam, and the pricing surface.
Vertical slices (wiring PRE_GATE, each producer+consumer+smoke):
  L1 CreditLedgerEvent + balance-fold + `elai billing balance` consumer + a grant/spend fixture.
  L2 reserve/settle loop wired into the request path + a settle-matches-actual smoke.
  L3 OpenRouter managed backend (B-37) + actual-cost capture + J-28 booking + a live metered call.
  L4 J-49 admission gated on balance + tier rate limits + an out-of-credits refusal smoke.
  L5 subscription/refill + rollover + the payment webhook -> Granted + a renewal fixture.
  L6 the $/resolved-task efficiency measurement (SAFEDELTA) that backs the pitch — ELAI-
     measured, the done-gate for any "$20=$200" claim, never an asserted number (ID-5).
Done-gates ELAI-measured; the pitch ships only if L6's number is real. Feature-map upsert:
new "1.16 BILLING / CREDITS / COMMERCE" subsystem group; ELAI_BILLING_ENABLE in the
wire-dark-allowlist with a promotion plan.

Sources (external rails): https://openrouter.ai/docs/faq ,
https://openrouter.ai/docs/guides/overview/auth/byok , https://openrouter.ai/pricing ,
https://docs.litellm.ai/docs/proxy/virtual_keys , https://docs.litellm.ai/docs/proxy/cost_tracking ,
https://docs.litellm.ai/docs/proxy/users .

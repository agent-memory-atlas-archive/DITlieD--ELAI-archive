# K-24: credit-based seats on an OpenRouter rail — the credit ledger as an SSB sub-journal, reserve/settle metered usage, GD-2 single source of truth

STATUS: PLANNED (BCC wave; NET-NEW capability folded into K per the user's "into K" ruling; a multi-slice TRACK shipped as ONE file, L1..L6 sequenced vertical slices each with its own done-gate; the FIRST commerce/billing surface in ELAI — ~70% of the plumbing already exists as J-28/J-49/C-5/H-9/J-81)

Objective: the model — subscription price -> monthly CREDITS (denominated in real provider $) -> credits spent per metered usage -> ELAI pays the upstream provider -> ELAI controls rate limits, rollover, routing (the Nous Portal / OpenRouter shape). It is SAFE (no "unlimited" an agent loop can detonate): spend is metered and capped per request. ELAI's differentiator is NOT a magic provider deal — it RESOLVES MORE TASKS PER DOLLAR (local-lane routing + verification + caching), so a $20 credit pool goes materially further than $20 of raw frontier API. That claim is the whole pitch and MUST be SAFEDELTA-measured before it is said (L6, the done-gate for any "$20=$200" line). This session ships the new LAYER on the SSB spine: a CREDIT LEDGER as an SSB sub-journal (NOT a mutable counter, NOT an external DB), a reserve/settle pre-flight loop, an OpenRouter managed RuntimeBackend (B-37), subscription refill + rollover (seats.toml DATA), the payment-webhook seam, and the pricing surface. Hard boundary: the financial source of truth is ELAI's SSB credit journal — NEVER OpenRouter's or LiteLLM's spend store (a direct GD-2 violation + split-brain billing risk; the single biggest way to ship this wrong). The provider's number is an INPUT booked as Settled, reconciled nightly. GD-1: J-28 (the meter), J-49 (seat/quota/spend), C-5/J-81 (the caps + the runaway kill-switch), H-9 ($/resolved-task), J-54 (the near-zero-COGS free lane), J-55 (the local-usefulness engine), B-37 (the backend pattern), I-51 (the curated catalog), F-4/I-18/J-69/J-70 (the audit-grade journal) — REUSED, re-implemented NEVER. Only the credit ledger + the managed rail + the payment seam + the pricing surface are new.

Estimated effort: ~6-7 sessions (~26-32 h), one per L-slice. In-process ledger fold + fixtures; the OpenRouter slice (L3) makes a live metered call (network-gated, W-10); the efficiency slice (L6) is a SAFEDELTA run (RunPod/local-lane-gated for the local arm).
Pre-requisites (each re-grepped at edit, W-7): J-28 TOKEN LEDGER (UsageTurnEvent, SpendAccumulator repriced off models.toml, NormalizedUsage lane-semantics — PLANNED; the meter L2/L3 book against); J-29 LIVE METER (LiveUsageGauge — PLANNED; L2 mid-stream); J-49 MULTI-SEAT ADMISSION + per-seat quota (SeatRegistry, seats.toml, weighted-token-bucket fair-share, per-seat spend ceiling reusing C-5 caps — PLANNED; L4 gates on balance); H-9 COST COCKPIT (CloudSpend, $/resolved-task — PARTIAL); C-5 SPEND CEILINGS [WIRED, CloudModeRouter hard ceiling]; J-81 MID-LOOP DETERMINISTIC COST CAP (wires DARK E-39 — PLANNED; L2 runaway kill); B-37 MULTI-RUNTIME (11 backends, RuntimeAdapter, PerRoleRuntime — WIRED; L3 adds backend #12); J-54 FREE-API ROTATION POOL (FreePool, ~0 COGS — PLANNED); J-55 MODEL-STRENGTH MODALITY ROUTER (makes the local lane actually work — PLANNED; the margin engine); I-51 STATIC models.toml CATALOG; F-4 + RRA + I-18/J-69 hash-chain + J-70 IV&V (the audit-grade journal); G-6 SAFEDELTA (L6).
Parallel-safe with: K-23 (disjoint files), K-1..K-22. NOT parallel with edits to J-28's UsageTurnEvent or the B-37 backend trait (L3) or J-49's admission (L4).
Source: workflow/reasoning/elai-credit-billing-monetization-2026-06-26.md (the full spec §0..§9; external rails web-grounded: OpenRouter openrouter.ai/docs/faq + /byok + /pricing, LiteLLM docs.litellm.ai/docs/proxy/{virtual_keys,cost_tracking,users}). Doctrine: K-0_INDEX.md. The reasoning file IS the grounding pass (the GPT note that seeded it is ONE vote re-grounded, never a source); re-grep ELAI seams at edit.

## Context
Why this matters: ELAI has the meter (J-28), the seat/quota/spend (J-49), the caps (C-5/J-81), the cockpit + $/resolved-task (H-9), the free lane (J-54), the local-usefulness engine (J-55), and the audit-grade journal — but NOT a credit ledger, subscription/refill, a managed (ELAI-pays) provider rail, rollover, the payment boundary, or the reserve/settle accounting. That gap is the new work; ~70% of the plumbing exists. The honest mechanism of "$20 ships $200 results": ELAI's edge is $/RESOLVED-TASK (H-9), not $/token; the LOCAL LANE (sidecar / J-54 free pool, made correct by J-55) is the margin engine — work routed there costs ~0 marginal $ and draws ~0 credits, so the more ELAI correctly routes to local/cheap, the further a $20 pool stretches. That is the literal mechanism, not a discount and not "unlimited." The pitch must be a REPRODUCIBLE SAFEDELTA NUMBER on a real task envelope (Aider-style), per the GTM "believe-the-number" decision, never an asserted "10x" (KD-1/ID-5). PRIOR-CAUTION (anti-sycophancy, do not bury): the 2026-06-02 market work advised DROPPING the bare cost/cheap framing and re-aiming at devs/prosumers on TRUST; the efficiency pitch here is compatible ONLY sold as "proven more value per dollar, verified" (a trust claim backed by SAFEDELTA), never as "cheap." If L6's number isn't there when measured, the pitch reverts to trust/quality and the credit model still stands on its own as a safe metered-usage plan.

## Existing state (B/C/D/E/F/G/H/I/J/K coverage)
- J-28 TOKEN LEDGER [PLANNED]: UsageTurnEvent (identity/lane/backend/model/task/session+elapsed_ms), SpendAccumulator repriced off models.toml, NormalizedUsage (sidecar full-prompt vs cloud suffix-only, never summed), `elai usage`. The METER L2/L3 book the Settled amount against.
- J-29 LIVE METER [PLANNED]: in-process LiveUsageGauge (mid-stream token count). L2 reads it during the call.
- J-49 MULTI-SEAT ADMISSION [PLANNED]: SeatRegistry (seats.toml DATA), weighted-token-bucket fair-share admission in front of routing, per-seat SPEND CEILING reusing C-5 caps. The seat/plan spine; L4 gates it on the credit balance.
- H-9 COST COCKPIT [PARTIAL]: CloudSpend rows, per-response input/output/CACHED breakdown, $/RESOLVED-TASK already exists (the pitch's unit of measure).
- C-5 SPEND CEILINGS [WIRED] + J-81 MID-LOOP COST CAP [PLANNED, wires DARK E-39]: the pre-flight caps + the runaway-loop kill switch (L2 aborts the CURRENT step before a runaway blows the hold).
- B-37 MULTI-RUNTIME [WIRED, 11 backends, RuntimeAdapter, PerRoleRuntime, RuntimeOrchestrator]: L3 adds OpenRouter as backend #12, hand-rolled like the others (J-54 precedent: hand-rolled Rust, no litellm).
- J-54 FREE-API ROTATION POOL [PLANNED] + J-55 MODALITY ROUTER [PLANNED]: the ~0-COGS lane + the engine that makes it produce correct results — the margin engine the economics depend on.
- I-51 models.toml CATALOG [I-51]: the curated model catalog (OpenRouter is the TRANSPORT+METER, NOT a dynamic registry — the I-51 rejection of OpenRouter's dynamic model registry stands, threaded not re-litigated).
- F-4 + RRA + I-18 UPIP chain + J-69 hash-chain + J-70 IV&V: the audit-grade, hash-chained, replayable journal — exactly the money journal billing disputes want.

## Reads (mandatory before coding)
1. workflow/reasoning/elai-credit-billing-monetization-2026-06-26.md §2 (what exists), §3 (the new layer + the 8-step per-request loop mapped to real seams), §4 (the rail, two approaches), §5 (the payment boundary), §8 (the adversarial falsifiers).
2. crates/orchestrator/src/ssb.rs + tests/action_type_lock.rs (the 9-tuple golden lockstep; re-grep SCHEMA/TV at edit, W-9) — +CreditLedgerEvent lands here; the single-writer WAL IS the double-spend defense.
3. J-28's UsageTurnEvent/SpendAccumulator surface + models.toml pricing (the Settled amount source) + J-29 LiveUsageGauge.
4. J-49's SeatRegistry + seats.toml + the C-5 cap surface (L4 admission); the B-37 RuntimeAdapter trait (L3 backend #12).
5. The OpenRouter API (openrouter.ai/docs/faq, /byok, /pricing) — passthrough provider pricing (no markup), 5.5% purchase fee, BYOK 5%, actual cost returned per call; LiteLLM (docs.litellm.ai) only to understand WHY its spend store must NOT be the ledger.
6. CLAUDE.md hard rules: GD-2 (journal-is-the-record — provider numbers are INPUTS), W-2/R9 (kill-switch + allowlist), KD-1/ID-5 (no asserted number; SAFEDELTA-measured), anti-slopsquatting (verify any OpenRouter SDK before importing — but hand-roll per B-37/J-54), blake3-only, no .unwrap on prod paths.

## Tasks (BCC wave — sequenced vertical slices L1..L6; each ships producer + in-plan consumer + smoke)
1. L1 — CREDIT LEDGER + balance-fold + `elai billing balance`. +CreditLedgerEvent (Pattern-2; CreditLedgerEventKind = Granted (subscription refill/bonus/promo) | Reserved (pre-flight hold) | Settled (actual cost booked, releases the hold delta) | Refunded | Expired (rollover cap) | AdjustmentManual). A seat's balance is a FOLD over append-only credit rows, NEVER a stored number that can drift or double-spend: balance(seat) = sum(Granted) - sum(Settled) - sum(open Reserved) - sum(Expired). Hash-chained (I-18/J-69) so the spend history is tamper-evident (a genuine billing-dispute advantage). Single-writer SSB serialization IS the double-spend defense (no two requests both spend the last credit). Consumer: `elai billing balance` folds and prints. Smoke: a grant/spend fixture where the fold matches the expected balance. Whole capability behind ONE default-OFF ELAI_BILLING_ENABLE (allowlist row: type=experiment, owner, expiry, promotion_plan_id=K-24-promotion, ON-path smoke). ~160 LOC.
2. L2 — RESERVE/SETTLE pre-flight loop wired into the request path. The 8-step loop on real seams: ESTIMATE (expected_tokens x model_rate from J-28 SpendAccumulator + I-51 models.toml, tagged-estimator fallback) -> reserve a CONSERVATIVE hold (over-reserve, settle down — a cheap under-reserve admits a request that overspends) -> emit Reserved (balance reflects it immediately) -> CALL provider (J-29 LiveUsageGauge meters mid-stream; J-81 mid-loop cap aborts the CURRENT step if a runaway blows the hold before completion) -> READ actual usage (OpenRouter returns real cost; local = metered tokens at ~0 price) -> emit Settled(actual), release (hold - actual) same-tick -> plan bonus applied at GRANT time, not per request (audit-clean) -> if balance hits 0 mid-task, J-81 + J-49 halt at a SAFE boundary with a resumable handoff, NEVER a hard kill mid-edit. Smoke: a settle-matches-actual test (Settled == metered cost; the released delta returns to balance same-tick). ~180 LOC.
3. L3 — OpenRouter MANAGED backend (B-37 #12) + actual-cost capture + J-28 booking. Hand-rolled Rust client (B-37 RuntimeAdapter, like the 11 existing backends; J-54 "hand-rolled, no litellm" precedent — NO new dep, anti-slopsquat moot). One upstream API -> 200+ models incl. small/cheap; OpenRouter returns actual cost per call which J-28 books as Settled. ELAI's cost of a credit-$ is ~1.055x face (the 5.5% purchase fee); margin comes from CONSUMING FEWER expensive tokens (the local lane), NEVER from reselling tokens (OpenRouter has no markup either way). I-51 TENSION threaded: OpenRouter is the PROVIDER/BILLING RAIL, NOT OpenRouter's DYNAMIC MODEL REGISTRY (rejected 3x); the catalog stays ELAI-curated in models.toml; OpenRouter is only transport+meter for models ELAI already chose. Single-vendor risk mitigated by the B-37 direct-Anthropic/OpenAI fallback backend (bedrock/azure direct lanes already exist) + the local lane as the always-on floor. Smoke: a live metered call (network-gated, W-10) whose returned cost is booked as Settled and matches the J-28 NormalizedUsage capture. ~200 LOC.
4. L4 — J-49 admission gated on balance + tier rate limits + out-of-credits refusal. J-49 admission checks balance >= hold AND fair-share AND rate/concurrency for the seat's tier; fail-closed -> queue or refuse with a clear "out of credits" surface (never a silent overspend). Tier rate limits + concurrency are seats.toml DATA per tier (W-2, no new flag per tier). Smoke: an out-of-credits seat is refused with the clear surface; a within-budget seat is admitted. ~120 LOC (mostly wiring the balance fold into the existing J-49 admission).
5. L5 — SUBSCRIPTION/REFILL + ROLLOVER + the payment webhook -> Granted. Payment processing (Stripe/Paddle) is OUTSIDE ELAI (the harness never touches card data; PCI scope zero). The ONLY seam: a payment webhook (subscription created/renewed/cancelled) emits CreditLedgerEvent::Granted(face + bonus) + Expired(amount over the tier's rollover cap). Rollover cap is per-tier DATA in seats.toml. Failed payment -> no Granted -> balance runs down -> L4 admission refuses at 0. Chargeback -> AdjustmentManual(negative) + a seat-suspension flag. Per-seat identity provenance defers to J-2 (the seat-id is the J-28 UsageTurnEvent.identity). Smoke: a renewal-webcall fixture emits Granted(face+bonus) and Expired(over-cap) and the balance fold reflects both. ~140 LOC.
6. L6 — the $/RESOLVED-TASK efficiency MEASUREMENT (SAFEDELTA) that backs the pitch. The done-gate for any "$20=$200" claim — ELAI-measured on a real task envelope (Aider-style), recomputable (GD-2), receipts on the y-axis (GD-3, never pass@k), under ELAI_SEED; reports $/verified-task for an ELAI-routed run (local-lane-heavy) vs a raw-frontier run. The MARGIN REALITY (anti-sycophancy): OpenRouter 5.5% purchase fee + the 10% plan bonus = ELAI eats ~15% on the frontier portion; the economics ONLY work if a large fraction routes to the local/free lane (near-0 COGS) — so J-55 routing QUALITY is the business model, not a feature; GATE the launch on the measured local-lane resolve-rate, not faith. The pitch ships ONLY if L6's number is real; else it reverts to trust/quality (KD-1/ID-5; the 2026-06-02 prior-caution). ~150 LOC (a bench arm; prefer-reuse the G-6 SAFEDELTA runner shape, do NOT fork it — GD-1). The honest pricing wording lands here too: "Plans include monthly credits for model and tool usage, metered at provider cost. ELAI routes work to the cheapest model that can verifiably do the job — often a local one — so your credits resolve more tasks per dollar." NEVER "unlimited Sonnet/Opus/GPT."
7. workflow/reasoning/K-24-decisions.md (the GD-2 ledger-is-truth decision, the OpenRouter-rail-not-registry decision, the LiteLLM-never-the-ledger decision, the reserve/settle conservatism rule, the L6-number-gates-the-pitch rule, expected per-slice test outcomes, unknowns).

## WIRING_CONTRACT
produces_symbols:
  - +CreditLedgerEvent SSB (Pattern-2; CreditLedgerEventKind = Granted|Reserved|Settled|Refunded|Expired|AdjustmentManual), hash-chained (I-18/J-69), single-writer-serialized; payload carries only seat-id, amounts, model/lane, a provider-cost reference — NEVER card data or a payment secret — L1
  - the balance-fold reader (balance = sum(Granted) - sum(Settled) - sum(open Reserved) - sum(Expired)) + `elai billing balance` — L1
  - the reserve/settle pre-flight loop wired into the request path (estimate -> Reserved -> call -> Settled, hold released same-tick) — L2
  - the OpenRouter managed RuntimeBackend (B-37 #12, hand-rolled, actual-cost capture booked as Settled via J-28) — L3
  - the J-49 admission gate on credit balance + tier rate limits + the out-of-credits refusal surface — L4
  - the subscription refill + rollover (seats.toml DATA) + the payment-webhook -> Granted/Expired seam — L5
  - the SAFEDELTA $/resolved-task efficiency bench arm + the honest pricing wording — L6
consumes_symbols:
  - crates/orchestrator/src/ssb.rs single-writer WAL + I-18/J-69 hash-chain (the ledger is an SSB sub-journal — re-implemented NEVER; the WAL serialization IS the double-spend defense)
  - J-28 UsageTurnEvent/SpendAccumulator + models.toml pricing + J-29 LiveUsageGauge (estimate + Settled amount + mid-stream meter) — GD-1
  - J-49 SeatRegistry/seats.toml admission + C-5 spend ceilings + J-81 mid-loop cost cap (E-39) (admission + the runaway kill at a safe boundary)
  - B-37 RuntimeAdapter trait + the direct-Anthropic/OpenAI fallback backend (the OpenRouter backend rides the trait; the fallback mitigates single-vendor risk) — re-implemented NEVER
  - J-54 FreePool + J-55 modality router (the ~0-COGS margin lane the economics depend on); I-51 models.toml (the curated catalog; OpenRouter is transport NOT registry)
  - H-9 $/resolved-task + G-6 SAFEDELTA runner shape (L6 — compose the runner, do NOT fork it)
reachable_from:
  - `elai billing balance` -> the credit fold over the SSB sub-journal [PRODUCTION CLI]
  - the request path -> reserve/settle loop -> J-28 Settled booking -> the OpenRouter backend (B-37) [PRODUCTION request path + B-37 backend]
  - J-49 admission -> balance check -> admit/queue/refuse [PRODUCTION admission gate]
  - the payment webhook handler -> CreditLedgerEvent::Granted/Expired [PRODUCTION webhook seam]
  - a #[cfg(test)]/contracts/ caller NEVER satisfies this (R7)
stub_ban:
  - NO mutable balance counter — balance is ALWAYS a fold over append-only credit rows (GD-2; the drift/double-spend defense)
  - NO external/provider spend store as the source of truth — OpenRouter's/LiteLLM's number is an INPUT booked as Settled, reconciled nightly (the single biggest way to ship this wrong)
  - NO LiteLLM as the ledger (its budget enforcement reads its OWN Postgres = a GD-2 violation + split-brain; if ever used it is a STATELESS dumb pipe with its budget feature OFF)
  - NO OpenRouter dynamic model registry (I-51 rejection stands; OpenRouter is transport+meter only, models stay curated in models.toml)
  - NO "unlimited" tier and NO asserted efficiency number — the "$20=$200" pitch ships ONLY behind L6's measured SAFEDELTA number (KD-1/ID-5)
  - NO card data or payment secret in the harness or the journal (PCI scope zero; webhook -> Granted only)
  - NO hard kill mid-edit on balance=0 — halt at a SAFE boundary with a resumable handoff (J-81/J-49)
  - NO new dependency for the OpenRouter client (hand-rolled per B-37/J-54); NO .unwrap on prod paths; blake3 only (the hash-chain)
frontend_pair:
  the credit balance + reserve/settle + tier rate limits on the H-9 CloudPanel (or a dedicated billing tile) as a Svelte 5 runes component + #[tauri::command] + generate_handler! + ts-rs binding; cross_lang_diff.py PASS (W-4). Decide the exact panel at edit (rides H-9 CloudPanel).
observable_effect:
  trigger: "elai billing balance ; (a metered request through the OpenRouter backend) ; (a renewal webhook fixture)"
  expect_ssb: "CreditLedgerEvent rows (Granted on refill, Reserved on pre-flight, Settled on completion releasing the hold delta, Expired over the rollover cap) in the SSB, hash-chained (I-18/J-69), recomputable from the journal (GD-2); the J-28 UsageTurnEvent the Settled amount was booked against"
  expect_artifact: "`billing balance` prints the fold (Granted - Settled - open Reserved - Expired); a live metered OpenRouter call books Settled == the returned actual cost; an out-of-credits seat is refused with the clear surface (L4); a renewal webhook emits Granted(face+bonus)+Expired(over-cap). NEGATIVE: a mutable-counter balance implementation FAILS; OpenRouter's spend number used AS the ledger FAILS (GD-2); a settle that does NOT match metered actual FAILS; a '$20=$200' claim with no L6 SAFEDELTA number FAILS the K-1 claims-policy lint (KD-1)"

## DETERMINISTIC_CHECK
```bash
python .claude/tools/wiring_contract_check.py --bn-file docs/thefinalglance/K-plan/K-24_credit_billing_monetization.md --run-trigger
rg -n "CreditLedgerEvent|CreditLedgerEventKind" .elai_cc/crates/orchestrator/src/ssb.rs   # the SSB ledger type exists, golden-locked
rg -n "billing" .elai_cc/crates/cli/src/args.rs                                            # the `elai billing` verb on the real entrypoint
rg -n "Reserved|Settled" .elai_cc/crates/orchestrator/src/                                 # the reserve/settle loop exists
rg -c "balance\s*[:=]\s*[a-z_]*counter|mutable.*balance|set_balance" .elai_cc/crates/      # MUST be 0 (balance is a fold, never a stored counter; GD-2)
rg -c "litellm.*budget|openrouter.*source.*of.*truth|provider.*ledger" .elai_cc/crates/    # MUST be 0 (no external spend store as truth)
python .claude/tools/cross_lang_diff.py --changed .elai_cc/crates/orchestrator .elai_cc/crates/dashboard-v2/src/lib   # frontend_pair drift for the billing tile
test -f workflow/reasoning/K-24-decisions.md
```

## ACCEPTANCE
- [ ] GD-2: balance is ALWAYS a fold over the append-only CreditLedgerEvent sub-journal; the provider's number is an INPUT booked as Settled. No mutable counter, no external store as truth.
- [ ] Double-spend defense: single-writer SSB serialization — two concurrent requests cannot both consume the last credit (Reserved rows serialize through the one writer).
- [ ] Runaway defense: J-81 mid-loop cap + J-49 pre-flight admission + the Reserved-hold-before-call mean a balance cannot be detonated (a SELLING POINT vs naive credit products).
- [ ] I-51: OpenRouter is transport+meter only; the model catalog stays curated in models.toml (the dynamic-registry rejection stands).
- [ ] KD-1/ID-5: no "$20=$200" / efficiency claim ships without L6's measured SAFEDELTA number; the pricing copy never says "unlimited."
- [ ] PCI: no card data in the harness or journal; the only payment seam is the webhook -> Granted.
- [ ] W-2/R9: ELAI_BILLING_ENABLE has a wire-dark-allowlist row (type=experiment, owner, expiry, promotion_plan_id=K-24-promotion, ON-path smoke).
- [ ] cargo check --workspace GREEN; auditor PASS on -p orchestrator, -p cli; svelte-check for the billing tile.
- [ ] SSB net +1 (CreditLedgerEvent), sequential bump from LIVE 268/577 (re-grep both at edit, W-9).
- [ ] No new dependency (the OpenRouter client is hand-rolled per B-37/J-54).

## WIRING DONE-GATE (clause by clause)
- W-1: done = a live metered OpenRouter call books Settled == actual cost + `billing balance` folds correctly + the out-of-credits refusal fires; cargo-green is NOT done.
- W-2: ELAI_BILLING_ENABLE is a default-OFF capability flag -> wire-dark-allowlist row with promotion_plan_id=K-24-promotion (NOT a terminal state).
- W-3: no summary-only port (full vertical slices). W-4: the billing tile frontend_pair (H-9 CloudPanel) + cross_lang_diff PASS.
- W-5: live trigger = the metered OpenRouter call; deterministic fixtures (grant/spend fold, settle-matches-actual, renewal webhook) ship now; the live metered call + the L6 local-lane SAFEDELTA arm are network/RunPod-gated (W-10).
- W-7: ssb.rs/action_type_lock.rs, J-28 UsageTurnEvent, J-49 admission, the B-37 trait, models.toml ALL re-grepped at edit.
- GD-1/GD-2/GD-3: as ACCEPTANCE. KD-1/ID-5: L6 gates the pitch. R7: no test-only caller satisfies reachability.

## Tests to add
- crates/contracts/tests/k24_credit_billing_contract.rs (~200 LOC): (a) a grant/spend fixture: balance fold == sum(Granted)-sum(Settled)-sum(open Reserved)-sum(Expired); (b) reserve/settle: Reserved reflects in balance immediately, Settled(actual) releases (hold-actual) same-tick, settle == metered cost; (c) double-spend: two concurrent Reserved against the last credit — exactly one admitted (single-writer serialization); (d) out-of-credits: balance < hold -> refused with the clear surface, never a silent overspend; (e) renewal webhook -> Granted(face+bonus) + Expired(over rollover cap); failed payment -> no Granted -> admission refuses at 0; (f) the ledger is a FOLD, never a stored counter (no set_balance path); OpenRouter's number is booked as Settled, never read as the source of truth (GD-2); (g) a "$20=$200" claim string with no L6 SAFEDELTA number fails the K-1 claims-policy lint (KD-1).
- NOTE: crates/orchestrator/tests/ integration tests do NOT link (ort/ndarray rlib) — put the request-path/backend smoke as an in-crate #[cfg(test)] lib test; the contract goes in crates/contracts/tests/.

## Files touched
- CREATE: crates/orchestrator/src/billing/{mod.rs,credit_ledger.rs,reserve_settle.rs,refill.rs,webhook.rs} (~620 LOC across L1/L2/L5)
- CREATE: crates/orchestrator/src/runtime/openrouter_backend.rs (~200 LOC, B-37 #12, hand-rolled) — L3
- MODIFY: crates/cli/src/args.rs + a billing_cmd.rs module (~80 LOC: `elai billing balance` + FSM gate)
- MODIFY: crates/orchestrator/src/ssb.rs + tests/action_type_lock.rs + the golden (+CreditLedgerEvent, sequential 9-tuple lockstep)
- MODIFY: the J-49 admission path (~60 LOC: gate on the credit balance fold) — L4
- MODIFY: the J-28 booking path (~40 LOC: Settled <- NormalizedUsage actual) — L3
- MODIFY: crates/dashboard-v2/src/lib (+ src-tauri commands_*.rs + generate_handler! + ts-rs): the billing tile on the H-9 CloudPanel (~80 LOC)
- MODIFY: seats.toml (tier rate limits + rollover caps as DATA); .elai_cc/wire-dark-allowlist.toml (+1 row: ELAI_BILLING_ENABLE)
- CREATE: a SAFEDELTA $/resolved-task bench arm (compose G-6 runner; ~150 LOC) — L6
- CREATE: crates/contracts/tests/k24_credit_billing_contract.rs (~200 LOC)
- CREATE: workflow/reasoning/K-24-decisions.md (~40 LOC)

## Citations / Grounding
- workflow/reasoning/elai-credit-billing-monetization-2026-06-26.md §0..§9 — the model, the grounded rail mechanics, the new layer on the SSB spine, the 8-step loop on real seams, the two-approach rail recommendation, the payment boundary, the grounded pitch, the suggested tiers, the adversarial falsifiers.
- OpenRouter: openrouter.ai/docs/faq (passthrough pricing, no markup), /docs/guides/overview/auth/byok (5% BYOK, waived first 1M/mo), /pricing (5.5% purchase fee, $0.80 min). [EXTERNAL — the rail's real mechanics, web-verified]
- LiteLLM: docs.litellm.ai/docs/proxy/{virtual_keys,cost_tracking,users} — virtual keys + per-key budgets + HARD budget enforcement against ITS OWN DB (the reason it can NEVER be the ledger: GD-2 violation + split-brain). [EXTERNAL]
- ELAI ground truth: J-28 (the meter), J-49 (seat/quota/spend), C-5/J-81 (caps + runaway kill, E-39), H-9 ($/resolved-task), J-54/J-55 (the margin lane + the engine that makes it correct), B-37 (the backend pattern, 11 backends WIRED), I-51 (curated catalog), F-4/I-18/J-69/J-70 (the audit-grade hash-chained journal).
- CLAUDE.md / K doctrine: GD-1 (compose), GD-2 (journal-is-the-record), GD-3 (receipts not pass@k), KD-1 (number-before-claim), ID-5 (ELAI-measured never a borrowed number), W-2/R9 (kill-switch + allowlist), anti-slopsquatting (hand-rolled client, no new dep), blake3-only.
- PRIOR memory: project_market_positioning_2026 (drop bare cost framing, re-aim on trust), project_gtm_playbook_2026 (believe-the-number SAFEDELTA, first buyer = vibe-cleanup agencies @ $39-49/seat — the ALT team/agency tier anchor).

## Open questions / risks
- Estimate inaccuracy: the pre-flight hold must be conservative (over-reserve, settle down) or a cheap estimate under-reserves and admits a request that overspends — use a per-model worst-case hold from I-51; settle to actual same-tick. Decide the worst-case multiplier at execution from real model rates.
- Margin reality is a LAUNCH GATE, not a footnote: ~15% eaten on the frontier portion (5.5% fee + 10% bonus) means the economics need a large local/free-lane fraction; gate the launch on L6's measured local-lane resolve-rate, never on faith. If J-55 routing quality is poor, the margin is negative.
- Vendor dependency: an OpenRouter outage = no paid lane; mitigate with the B-37 direct-provider fallback (already supported) + the local lane as the always-on floor. Confirm the fallback backend at execution.
- The ALT anchor (team/agency seat @ $39-49/seat, pooled credits + per-seat J-49 fair-share + the audit-grade spend journal as a selling point) may monetize BEFORE consumer tiers (GTM memory) — author the seats.toml tier DATA to support a pooled-team tier from day one.

## Next
This closes the commerce whitespace with a credit ledger that is an SSB sub-journal (hash-chained, replayable, single-writer — the exact money journal billing disputes want), on a managed OpenRouter rail, with the financial source of truth staying ELAI's SSB (GD-2). It pairs with K-23 (the two are the first net-new-capability K files) and depends on the J-28/J-49/J-54/J-55 substrate landing. The pitch ("$20 ships $200 results") ships ONLY behind L6's measured SAFEDELTA number — and the credit model stands on its own as a safe, metered, never-"unlimited" plan regardless. The first revenue path is plausibly the team/agency seat tier, not consumer tiers (GTM memory).

End of K-24.

AUTONOMOUS MARKETING / SALES-DEV AGENT — BUILD PLAN
Date 2026-06-07. Grounded by 5 GPT deep-research reports (cost, cold-email, lead-gen, social, GTM) + own web grounding. Reports saved at D:/.apps_workflow/.claude/mcp/gpt-bridge/state/research/dr-1780833*.md.

CORE THESIS
The 2026 platform stack penalizes exactly what "fully autonomous marketing" maximizes: automation, AI-slop, volume spam. So invert the instinct. Automate the BACK office (source, enrich, verify, draft, triage, schedule, CRM). Keep a HUMAN at the FRONT (the reply that matters, the close, the post on a real account). The agent's value is throughput on low-trust steps, not strategy and not relationships.

THE COST CORRECTION (the premise that triggered this was wrong)
Marketing-agent LLM bill, well-routed: ~$3-15/mo at 100 touches/day, ~$12-65/mo at 500/day. Frontier-everywhere is the ONLY way to make it expensive (60-80x more). So model cost is a non-issue here; do not over-engineer it, do not self-host for it (rented 4090 ~$0.27-1.05/Mtok loses to Groq/Gemini-batch at this volume). Local model management is the right lever for the PRODUCT (the autonomous app builder: millions of tokens per build, $5-50+ COGS/app) — that is where ELAI's cost work pays off, NOT here.

WHERE MONEY AND RISK ACTUALLY GO (plan around these, not tokens)
Data: lead/enrichment credits, low hundreds/mo.
Email infra: domains + inboxes + warmup + sender tool, ~$100-300/mo.
Risk (non-monetary, larger): deliverability collapse, domain/account bans, GDPR. Burn a throwaway brand first (user's instinct, confirmed correct) — never the ELAI/Empire name.

==== PHASE 0 — DECISIONS BEFORE ANY CODE ====
0.1 Positioning first (Dunford, not Hormozi). Pick ONE niche + ONE outcome. "We build with AI" is dead positioning (AI is baseline 2026; 73% of B2B buyers avoid irrelevant outreach). Sell an outcome. Example forms:
  - local service biz: "turn review/search traffic into booked appointments"
  - founder MVP: "production-ready MVP live fast enough to test, clean enough to not embarrass at launch"
  - white-label: "the dev partner for designers/consultants who sell faster than they ship"
0.2 Pick the FIRST channel. ROI order: warm outreach + referrals/partnerships/white-label (highest) > selective Upwork (proof) > targeted single-vertical local audits > communities (support only) > Fiverr (avoid, race to bottom). For a cold start with no warm list, lead with partnerships/white-label + one vertical local-audit campaign.
0.3 Throwaway brand. Register a disposable brand + its outbound domains now so reputation can build during Phase 1.
0.4 Hormozi, settled: Core Four (warm/cold/content/ads) + Grand Slam Offer + value-equation are sound and match the buyer data. The "10x/100x leads" hype and "an agent crafts a Hormozi plan" are the bullshit parts — the frameworks take an afternoon to read; do not build an agent for strategy. Use them to package the offer, by hand, once.

==== PHASE 1 — INFRASTRUCTURE SPINE ====
1.1 Model router (BUILD). Cascade: cheap tier default, escalate on a confidence/complexity signal.
  - bulk classify/extract/tag/dedupe/score: Groq Llama 3.1 8B ($0.05/$0.08) or local Qwen3.5 4B/9B if a GPU is already owned
  - async enrich/personalize/follow-up/social drafts: Gemini 2.5 Flash-Lite Batch ($0.05/$0.20); or DeepSeek V4-Flash if prefixes repeat (cache-hit $0.0028 input)
  - live inbox triage/intent/CRM-action/simple reply: GPT-5.4 nano ($0.20/$1.25, built for this)
  - hard replies/objections/long threads only: Sonnet 4.6 ($3/$15); Opus only if high-stakes AND genuinely hard
  - scraper/extractor repair (code): Qwen2.5-Coder 7B local
  Levers to wire in: batch API (-50%), prompt caching (-90% on repeated system/ICP/playbook prefix), semantic cache for repeated inbox intents (not-interested / circle-back / send-pricing / wrong-person).
1.2 Email infrastructure (BUY + configure).
  - branded subdomains only (outreach.brand.com), NOT typo/cousin domains (M3AAWG: phishing-like).
  - SPF + DKIM(2048-bit) + DMARC(p=none, rua on) + valid rDNS + TLS on every sender. Non-compliant = permanent rejection since Nov 2025.
  - inbox math: cap 30-50 cold/inbox/day after warmup. ~4 inboxes for 100/day, ~8 for 250, ~15 for 500, across several subdomains. Warm 4-8 weeks starting 10-20/day.
  - sender tool: Instantly ($47, all-in-one + rotation + reply assist) OR Saleshandy/QuickMail (simpler/value). Smartlead only if already advanced.
  - thresholds (target stricter than enforcement): complaint <0.1%, hard bounce <2%, Gmail spam-rate <0.3%.
1.3 Lead pipeline (BUY data, BUILD orchestration). Order: find -> enrich -> verify -> suppress -> send; re-verify immediately before send.
  - discover: Google Places API (local, low legal risk), Clutch (agencies), Apollo $49 or Instantly Lead DB $47 (broad B2B)
  - enrich: Hunter (when domain known)
  - verify: ZeroBounce (before every send) — never trust a vendor "verified" flag
  - Clay $185 only if you will actually build waterfalls/scoring; otherwise skip
  - DO NOT scrape LinkedIn or Google Maps. hiQ only narrowed CFAA; ToS/contract + GDPR live (CNIL fined KASPR EUR 240k; LinkedIn vs Artisan Jan 2026). Treat named work emails as personal data.
1.4 State/CRM (BUILD light). One store: prospect, source, ICP-score, enrichment, verify-status, send-history, reply-intent, suppression list (permanent, global: hard bounces + opt-outs). Suppression is load-bearing for deliverability.

==== PHASE 2 — AUTOMATE THE BACK OFFICE ====
Automate end-to-end, no human needed: discovery, ICP scoring (incl. "weak/no website" detection from the URL you hold — derive your own score, store place_id not scraped Google content), enrichment, email verification, suppression hygiene, first-draft personalization, social-post drafts, follow-up variants, inbox triage/classification, draft replies, scheduling, CRM tagging, metric rollups.
AI personalization caution: 61% of people say they can spot an AI cold email. Use AI for research/signals/variants/triage — NOT industrial fake-personalized openers. Mailbox providers weight auth/complaints/DNS far above whether line 1 mentions a podcast.

==== PHASE 3 — HUMAN-FRONT OPERATING LOOP ====
Human (you) owns, every day: final approval of any sent cold email batch, every real reply that matters, all result/claim wording, the close, and every social comment/DM/connection/Reddit interaction.
Social: LinkedIn primary, X secondary, Reddit manual-only.
  - fully autonomous engagement = ban zone. LinkedIn explicitly bans automated comments/likes/DMs/connections. Reddit: 9:1 self-promo + behavioral shadowban detection. X: official-API publishing OK but likes/follows/quote-posts removed from self-serve API in 2026.
  - automate drafting/repurposing/scheduling/analytics (Buffer or Typefully; Taplio for drafting ONLY, never its auto-outreach). Human posts and engages.
  - content that wins: proof case studies, teardowns, build-in-public lessons, original research/benchmarks. "Recognizably human" is the 2026 differentiator.
GTM cadence (first 90 days):
  - weeks 1-2: package one tiered offer + 2-3 proof assets (micro-site, one teardown, one Loom audit, one offer page). Warm reactivation sweep (ex-clients/founders/agencies). One productized Upwork/Project-Catalog listing.
  - weeks 3-6: partner/white-label outreach (designers, SEO/paid-media freelancers, small agencies) BEFORE broad outbound. One single-vertical local-audit outbound campaign (manually reviewed, outcome-led). Publish a little niche-diagnostic content.
  - weeks 7-12: convert first win -> testimonial + intro + recurring care plan. Goal: 1-2 paying clients + one recurring component + a small partner network.

==== PHASE 4 — MEASURE AND SCALE GATES ====
Monitor: Google Postmaster Tools, Microsoft SNDS/JMRP, DMARC rua reports, Spamhaus lookups, plus per-domain reply metrics.
Benchmarks to expect: 30-60% open, 2-5% reply, 1-3% positive reply. Below that, suspect inbox placement / data / offer-fit / relevance — NOT send volume.
Scale gates (do not skip):
  - start at 100/day, not 500. Move to 250/day only after 2-4 weeks stable opens/replies/low-complaints/clean-unsub. Move to 500/day only after building the inbox count first.
  - recovery if a sender drops: pause it, fix DNS/auth, suppress complainers globally, cut volume, re-ramp slow.

BUILD VS BUY SUMMARY
Build: model router/cascade, lead orchestration, ICP/website scoring, state+suppression, metric rollups.
Buy: sender platform (Instantly), lead data (Apollo/Instantly DB/Places API), enrichment (Hunter), verification (ZeroBounce), social scheduler (Buffer/Typefully).
Skip for now: Clay (unless waterfalls), AI-SDR suites (11x ~$5k/mo, Artisan, AiSDR $900/mo — they multiply a working system, they do not fix data/deliverability), self-hosted GPU for the marketing agent, ELAI for the marketing agent (ELAI's lever is the product's per-build COGS).

HARD STOP / RISK GATES
- No scraping LinkedIn or Google Maps. Official APIs + bought-access vendors only.
- No autonomous social engagement (comments/DMs/connections/Reddit). Drafting + scheduling only.
- No cold mail to Canada or most of the EU (CASL consent; Germany prior consent). US companies + UK corporate bodies only at start. Global one-click unsubscribe, honored.
- No frontier-everywhere routing. Cheap tier default, Sonnet/Opus sparse.
- Throwaway brand until thresholds proven. Never the real brand first.

REALITY CHECK
Fully autonomous client-finding for a service is not realistic in 2026 — first clients are human-in-the-loop. What is realistic and worth building: an agent that does the volume/low-trust 80% so the human spends time only on the relationship and the close.

OPEN ITEMS / UNVERIFIED
- Exact per-niche conversion rates: thin public data; channel ranking is medium-high confidence, pricing high confidence.
- AI-personalization lift: no neutral 2026 meta-analysis; treat as research/triage aid, not a moat.
- 2026 API prices (Google/DeepSeek preview rails) move fast; re-check before committing the router config.
- Reddit current commercial API price not clearly public beyond the 2023 $0.24/1k-calls reset.

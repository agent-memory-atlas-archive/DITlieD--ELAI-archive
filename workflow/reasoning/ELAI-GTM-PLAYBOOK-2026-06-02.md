ELAI GO-TO-MARKET / CREDIBILITY PLAYBOOK — DECISION RECORD
Authored 2026-06-02. Companion to ELAI-FOCUS-RESCOPE-2026-06-02.md (the wedge) and
ELAI-DIRECTION-2026-06-01.md (the macro pivot). Answers the two OPEN questions the focus run left:
how a solo, closed-source, no-VC operator gets a self-published trust number BELIEVED, and finds first
PAYING users. Produced from run wf_e1b0804e-061 (5 facets, 11 agents, ~191 web fetches, adversarially
verified; self-corrected several numbers mid-run). Raw output in a temp dir (will be GC'd); this is the
durable record.

==============================================================================
0. THE REFRAME — three problems, not one, each with a different answer
==============================================================================
  - BELIEVABILITY (the number is dismissed by default): make it re-runnable by a skeptic in one command
    on a harness they already trust. NOT self-publish; NOT an Endor leaderboard slot (you can't get one).
  - CONTINUITY (bus-factor-1 = ELAI is itself a rug-pull risk): source-code escrow + open-source ONLY the
    verifier tool. NOT open-source the engine (Microsoft already gave that away free).
  - DISTRIBUTION (no OSS-virality channel): lead with vibe-code-cleanup AGENCIES as first paying buyer.
    NOT regulated enterprise (the late buyer that structurally caps a solo); NOT paid newsletters.

Realistic ceiling for this profile: a PROFITABLE SOLO/SMALL product serving agencies + prosumers, with
regulated enterprise as a slow escrow-gated maybe-never-at-volume upside. This is a path to first revenue,
NOT a venture outcome — and it matches maintainer's stated goal (a better life, make some money).

==============================================================================
1. CREDIBILITY — how the SAFEDELTA number gets believed
==============================================================================
Default state of a self-published number from a solo/closed/no-name vendor = DISMISSAL. Evidence: Devin
(Cognition published 13.86% SWE-bench + slick demo + NO reproduction path -> community "canceled" it after
Carl Brown's Internet of Bugs teardown); Kimi K2 self-reported 50% vs 29.4% independent on Humanity's Last
Exam (~21pt gap, widens for low-accountability publishers). ELAI is the maximally-untrusted publisher profile.

THE MOVE (Rank 1, the entry move, do this and only this first):
Run gate-ON vs gate-OFF on the SAME cloud Sonnet through SusVibes (CMU harness behind Endor's Agent Security
League; MIT, Docker-per-task, prediction format {instance_id, model_name_or_path, model_patch}, eval cmd
`python -m susvibes.run_evaluation`, ~400GB + 4CPU/4GB per worker — all verified live). Publish: raw
predictions JSONL (both arms) + Docker image + fixed seed + commit hash + a voluntary trajectory-log folder
(match SWE-bench's mandatory-traces bar voluntarily). Why rank 1: only path fully grounded AND fully under a
solo/closed/no-VC operator's control (no gatekeeper); harness is third-party-respected; reproducibility
envelope copies the exact bar that makes AIDER's leaderboard trusted despite Paul Gauthier being effectively
solo ("model + edit_format + commit_hash reliably reproduce any run").

Contamination defense (sound reasoning, NOT a proven credibility trick): a gate-ON/OFF delta on same model +
same seed cancels common-mode training-data leakage (inflation appears in both arms). Consistent with how
Veracode frames its ~55% security-pass delta.

HARD CORRECTIONS (verifier-flagged; getting these wrong self-falsifies the artifact):
  - Endor accepts NO external submission (18 curator-picked commercial combos). You publish YOUR OWN run on
    THEIR harness; never claim "appearing on the leaderboard." (Endor = LONG-SHOT outreach, not a track.)
  - Live Endor anchors are ~17.3% top SecPass (Codex+GPT-5.4) and ~84.4% top FuncPass (Cursor+Opus 4.6),
    board dated/moving (entries through 2026-05-20). RE-PULL at publish time + cite the date. The 23.5%/91.1%
    and "20.1% Claude Code+Opus 4.7" figures are REFUTED.
  - Do NOT pursue commercial independent-eval (Artificial Analysis, Epoch/MirrorCode) — capability/cost/latency
    benchmarkers with NO security yardstick; they cannot host a trust delta.

Rank 2 (Phase-2 amplifier): academic co-sign with SusVibes/CMU maintainers (sz3296@columbia.edu,
danqingw@cs.cmu.edu) — a name that isn't yours re-running the delta = strongest credibility. Slow, low cash.
Rank 4 (fire LAST, after the artifact exists): Show HN / write-up — "we can't show source, here's the Docker
+ seed + JSONL, run our SAFEDELTA yourself" reframes closed-source from a hole into a verify-the-claim posture.
Never astroturf.

Two-audiences truth: the artifact moves TECHNICAL REVIEWERS, not BUYERS. Procurement gates on SOC 2 Type 2 +
ISO 42001 + escrow. The benchmark gets you in the room; it never closes the regulated deal.

==============================================================================
2. OPEN-CORE DECISION — open-source ONLY the verifier
==============================================================================
Decision: NO on the gate engine + autonomy harness (stay closed). YES on the SAFEDELTA verifier / RRA replay
tool + in-toto attestation schema (Apache-2.0) — the few-hundred-LOC piece a skeptic needs to re-derive the
gate-ON/OFF delta from a signed artifact. Engine stays a black box; the SCORE becomes independently reproducible.

Why this boundary (clean precedents): Sourcegraph Cody (open Apache-2.0 client 2023, closed engine) = the
direct analogue; curl/Daniel Stenberg "verification over trust" (signed release artifacts + public verify page,
InfoQ 2026-05) = a literal working precedent AND the marketing line in one; arXiv:2506.23706 Attestable Audits
(publish your own safety benchmark, believed without trusting the reporter) — but TEE wrapper is OPTIONAL/DEFERRED
(SGX/SEV burden impractical for a solo; ship signed-artifact + open-replay first, add TEE only if a regulated
buyer demands it).

Why NOT the engine (AVOID): commoditized to $0 by Microsoft Agent Governance Toolkit (2026-04-02, MIT, all 10
OWASP agentic risks, <0.1ms p99, full 7-component suite). Open-sourcing the engine = competing with a free
MS-backed incumbent for zero moat. (Correction: the AVOID reason is the free MS incumbent — NOT the
OpenTofu/Valkey/OpenSearch forks, which prove "relicensing-away invites a fork," a different thing. Also Tabby
is itself open-core, NOT pure-permissive — don't cite it.)

FSL middle option (VIABLE, not recommended): release the gate-policy engine under FSL-1.1 (2-yr non-compete then
auto-converts to Apache/MIT — a genuine rug-pull antidote) but it's source-available not OSI-open, forfeits HN
goodwill, and gives away the component buyers most want to audit against the free MS incumbent. Prefer verifier-open.

What it buys: the ONE number that sells trust becomes independently reproducible; the curl "verify don't trust"
narrative; near-zero engine leakage (verifier is replay+hash-check you need internally). What it does NOT buy:
virality, or bus-factor cover. CONTINUITY is solved by ESCROW, not by open-sourcing anything.
Bonus: ELAI already plans the Reproducible-Run Artifact (G-2) + in-toto attestation (G-8) — the verifier-open
move is nearly free, it's G-2 + G-8 published.

==============================================================================
3. FIRST-PAYING BUYER — vibe-code-cleanup agencies (LEAD HERE)
==============================================================================
Re-anchor: regulated/air-gapped enterprise = the LATE buyer (SOC2 + escrow + pentest gates; long cycles that
cap a bus-factor-1 operator). The grounded FIRST-paying buyer is the vibe-code-cleanup agency — the single
strongest buyer signal in the run:
  - AI-attributed CVEs surged 6 -> 15 -> 35 across Jan-Mar 2026 (CSA Labs / Georgia Tech Vibe Security Radar).
  - 45% of AI-generated code carries an OWASP Top-10 vuln (Veracode, 100+ models); AI devs introduce findings
    at ~10x the rate.
  - ~$100-300/hr cleanup rates; ISHIR created a "Vibe Coding Cleanup Specialist" role; ~300% demand surge.
They need ELAI's exact wedge — a run that can't fabricate a passing test + a signed replayable change-journal —
as a BILLABLE deliverable to their own clients. $39-49/seat fits agency per-engineer spend. Proof = a customer
outcome, not GitHub stars -> bypasses the OSS-virality gap.

==============================================================================
4. DISTRIBUTION SEQUENCE (ranked, solo-fit; security/incident wedge integrated)
==============================================================================
A. xyborg/SupaExplorer play (RECOMMENDED — the ONLY genuinely-solo precedent that converted): a solo researcher
   turned an "11.04% exposure rate" finding into a front-page HN freemium launch (free OAuth audit + $5 report);
   Supabase CEO validated it in-thread (credibility transfer). Replicate the shape: scan a public, re-scannable
   corpus (RedAccess ~380,000 vibe-coded apps, proven leaky), publish "gate-OFF surfaced X, gate-ON blocked them
   — re-scan it yourself," free one-shot audit + paid harness. RESOLVE the ToS/legal risk of scanning third-party
   apps first (GitGuardian/Socket precedents have legal teams).
B. anti-Replit reproducible incident-replay (RECOMMENDED): rebuild the Replit prod-DB wipe (1,190+ companies hit
   during a code freeze; Jason Lemkin the named victim; Replit shipped "planning-only mode" = market conceded the
   gate-shaped hole), run gate-ON, hand the viewer the SIGNED SEEDED artifact to replay. Rafter 2026: "the only
   benchmark you can trust is one you run on your own codebase." Caveat: press covered victims/big platforms, not
   vendor demos — a conversion asset, not a guaranteed-press asset.
C. build-in-public on LinkedIn/X (RECOMMENDED, slow-compounding, also mitigates rug-pull fear): Base44/Maor Shlomo
   — solo, bootstrapped (~$10-20k own money), $1M ARR ~3 weeks post-launch, ~400k users, $80M Wix acquisition ~6
   months; primary-source line "building in public on LinkedIn drove more growth than any paid channel." Caveats:
   drop the invented $10k/$50k/$250k MRR ladder (real ~$200k/mo at sale); he ran failed influencer experiments
   (not "no paid marketing"); Base44 is a no-code builder w/ pre-built audience — your cold-start lift is lower.
D. one engineered Show HN / Product Hunt #1, timed to the number reveal (VIABLE): fire AFTER the artifact exists.
   Convert via free local-Qwen-privacy tier -> $39-49 paid gate. Precedent Tony Dinh / TypingMind PH #1 -> $30k/mo.
   One credible launch per product; don't burn it early.
E. free privacy/local tier on VS Code Marketplace + OpenVSX + JetBrains (VIABLE funnel): local Qwen = legit free
   tier; cloud-Sonnet gated run + artifact + governance = $39-49 paid. Accumulate installs + 4.5-star rating
   first, then gate. Treat "3-8x conversion / 15% paid" as DIRECTIONAL only (single SEO blog, not measured).
F. hand-court 5-10 design-partner agencies (VIABLE): a solo can't run 50 demos/week. Court a handful, produce
   2-3 NAMED case studies with measured before/after security delta -> logos + the number = proof for the next
   100. Atlassian no-sales-team, price-below-procurement-threshold motion ($39-49 is deliberately below the line).

AVOID/WEAK as lead channels: bare self-published number (AVOID — anti-credibility for a solo); paid dev-newsletter
spend (WEAK — mismatched buyer, thin margin; the highest-trust list, Pragmatic Engineer 1.1M, takes ZERO
sponsorships); team-scale Snyk/GitGuardian SEO flywheel (WEAK for a solo — every precedent is VC-funded w/ open
free tier). Do NOT promise any conversion rate — every conversion stat in the deck is unverifiable or
non-analogous. Measure your own funnel.

KILL the Article 50 GTM hook: Commission draft Article-50 guidelines (2026-05-08, Bird & Bird) explicitly EXEMPT
source-code outputs and NAME "agentic developer tools such as Claude Code, Cursor and Copilot" = ELAI's class.
SME Article-99 fine is the LOWER of EUR 15M/3%. Use the Aug-2-2026 date as a time-boxed outreach hook only,
never the value prop. Durable compliance trigger = in-toto/SLSA provenance (CRA/EO-14028 buyers already request
attestations in procurement) — ELAI's signed RRA already targets it.

==============================================================================
5. THE 90-DAY SEQUENCE
==============================================================================
Days 0-30 — BUILD THE NUMBER + START LONG-LEAD CLOCKS
  1. Wire the gate-ON/OFF rig against SusVibes (MIT, Docker, verified). [only credibility move fully in your control]
  2. Build the signed Reproducible-Run Artifact + Apache-2.0 verifier/replay tool. [Sourcegraph/curl precedent; = G-2+G-8]
  3. Start the SOC 2 Type 2 observation window NOW (3-6mo lead; "in-audit by [date]" keeps a first security review alive).
  4. Pre-draft a SaaS source-code escrow clause (Codekeeper lightweight / Escode). [bus-factor checkbox; necessary not sufficient]
Days 30-60 — MEASURE, VERIFY, PACKAGE
  5. Run gate-ON vs gate-OFF on the same cloud Sonnet; publish JSONL + Docker + seed + commit hash + traces; re-pull
     live Endor anchors (17.3%/84.4%, dated) at publish. [contamination-robust delta = the Devin defense; wrong anchors self-falsify]
  6. Email SusVibes/CMU maintainers for an academic co-sign eval note. [a name not yours re-running the delta = strongest credibility]
  7. Produce the first anti-Replit incident-replay asset with the replayable artifact attached. [Rafter: "run it yourself"]
  8. Stand up a free local-Qwen tier + VS Code Marketplace listing; accumulate installs/rating before gating.
Days 60-90 — LAND PAYING AGENCIES, THEN LAUNCH
  9. Run the xyborg-shaped public-corpus finding (free one-shot audit + paid harness). [only solo precedent that converted; resolve ToS first]
  10. Hand-court 5-10 cleanup agencies; close 2-3 design partners into NAMED case studies w/ measured before/after deltas.
  11. Fire the engineered Show HN / Product Hunt #1, artifact answering "did anyone verify this?" in advance.
  12. Begin build-in-public cadence on LinkedIn/X with the SAFEDELTA artifact as recurring content.
DON'T in 90 days: chase regulated/air-gapped enterprise; lead with Article 50; buy newsletter ads; open-source the
engine; pursue the Endor leaderboard as a real track.

==============================================================================
6. HONEST ODDS + PERSIST/ABANDON TEST
==============================================================================
Works because: the wedge pain is measured (45% OWASP, ~17% best-agent security correctness, AI-CVE 6->15->35);
the cleanup-agency buyer is the most-grounded signal at $100-300/hr; there is ONE genuinely-solo precedent that
converted (xyborg); the credibility move is fully in your control (self-run MIT harness) and copies a solo
precedent (Aider); solo/no-VC founding is now mainstream (22.2%->38% per Carta).
Doesn't work because: bus-factor-1 is the load-bearing UNVERIFIABLE risk (no precedent of a solo running the
regulated EXPAND motion at volume without hiring -> caps concurrent enterprise deals regardless of tooling — which
is WHY the plan leads with self-serve agencies); the free MS incumbent undercuts any "sell the gate" move; cold-start
distribution lift is lower than the build-in-public precedents had.
SINGLE BIGGEST RISK: shipping the number before it's independently re-runnable (the Devin failure mode). A flattering
self-published number from a solo closed-source vendor confirms the buyer's prior. The artifact's reproducibility is
the product's first real feature — get it right or don't ship it.
PERSIST if within ~90 days: the SAFEDELTA delta is REAL and MATERIAL (gate-ON measurably cuts insecure-rate on the
same model) AND >=1 cleanup agency pays for the harness as a billable deliverable.
ABANDON/pivot if: the delta is within noise (gate doesn't move outcomes on a neutral harness -> thesis hollow) OR
after a genuine public finding + 10 hand-courted agencies, ZERO convert at $39-49. Do NOT abandon on slow enterprise
traction — that's the expected late buyer, not a signal.
BOTTOM LINE: a plausible path to FIRST REVENUE, not a venture outcome. Realistic ceiling = a profitable solo/small
product serving agencies + prosumers; enterprise is a fund-it-later option, not the bet now. Build the believed
number, land the agencies, let enterprise be optional.

Pointers: ELAI-FOCUS-RESCOPE-2026-06-02.md (the wedge this sells); ELAI-DIRECTION-2026-06-01.md (the macro pivot);
G-SUMMARY_INDEX.md (G-2 RRA + G-8 attestation = the verifier-open substrate); memory project_focus_rescope_2026.

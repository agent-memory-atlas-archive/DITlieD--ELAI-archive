==============================================================================
VIBESEC TOOLS MATRIX 2026-06-01 - THE BEST SOFTWARE ADDING VIBESEC FEATURES
==============================================================================
FILE 2 of 3. Companion to ELAI-DIRECTION-2026-06-01.md and the VibeSec
landscape / threat-model file. This file directly answers the question: "what
are the best softwares out there that add features for VibeSec ?" - i.e. which
tools secure AI-generated / vibe-coded software, and crucially WHICH of them
PREVENT-BEFORE-WRITE versus which merely SCAN-AFTER. That single axis is the
load-bearing analysis for ELAI: ELAI's agreed strategy is PREVENTION-IN-THE-LOOP
(a deterministic gate that blocks the vuln / exposed secret / hallucinated
dependency BEFORE it is written, enforced in code: FSM gates, PreToolUse path
gating, anti-slopsquatting), NOT a late bolt-on scanner. "Scan AI code for vulns"
is already a funded, crowded category; ELAI must not ship a late scanner. So for
every tool the decisive column is PREVENTION-IN-LOOP: yes (blocks before
write/merge), partial (real-time but advisory, or gates at PR not at write),
no (scans after the fact / governance / offensive).

Reading the PREVENTION-IN-LOOP column:
  yes     = there is a deterministic or enforced point where the bad artifact is
            BLOCKED before it lands (pre-write hook, install-boundary block,
            push-protection block, MCP-context-injection-before-LLM).
  partial = real-time/early but advisory (flags, does not block), OR gates at
            PR/commit rather than at generation/write time, OR blocks only a
            narrow class (secrets-only, package-only).
  no      = post-hoc scan, autofix-on-alert, governance/standard, or offensive
            find-and-exploit. Useful, but not the before-write gate ELAI wants.

All funding/figure claims carry an inline citation [source, date, url]. Claims
the verification phase did not fully confirm are tagged UNVERIFIED: or
CORRECTED: inline. Numbers the verifier flagged as vendor-marketing are labeled
as such so they are not mistaken for measured ground truth.

==============================================================================
0. MARKET-CONTEXT NUMBERS (the measured baseline VibeSec tools sell against)
==============================================================================
These are the grounded, third-party-confirmed numbers that define the problem
every tool below claims to solve. They are the only measured rates worth citing.

- 45% of AI-generated code introduces an OWASP Top 10 vulnerability (55% secure),
  across 80 curated coding tasks evaluated on 100+ LLMs; XSS/CWE-80 86% failure,
  Log-Injection/CWE-117 88% failure; Java worst at ~28.5% pass. CONFIRMED across
  Veracode + Help Net Security + BusinessWire.
  [Veracode 2025 GenAI Code Security Report, 2025-07-30,
   https://www.veracode.com/wp-content/uploads/2025_GenAI_Code_Security_Report_Final.pdf]
- The 45% rate is FLAT 2023->2026 (hovers 45-55% regardless of model generation);
  Spring 2026 update: XSS 15% pass / Log-Injection 13% pass, 150+ models to date.
  CONFIRMED.
  [Veracode Spring 2026 GenAI Code Security Update, 2026-03-24,
   https://www.veracode.com/blog/spring-2026-genai-code-security/]
- 31.6% of AI-generated code samples fully exploitable; 59% average security
  performance across 27 models (730 prompts, 27 languages, 219 vuln categories).
  CONFIRMED.
  [IOActive, The Security Gap in AI-Generated Code, 2026-05-01,
   https://www.ioactive.com/the-security-gap-in-ai-generated-code/]
- 87% of code from AI coding agents contains >=1 security vuln; best security
  agent only 17.3% secure; Cursor+Claude Opus 4.6 = 84.4% functional but only
  7.8% secure. CONFIRMED (built on CMU SusVibes, 200 tasks, 108 repos, 77 CWEs).
  [Endor Labs Agentic Code Security Benchmark, 2026-04-15,
   https://www.prnewswire.com/news-releases/endor-labs-launches-agentic-code-security-benchmark-finds-top-performing-ai-coding-agents-pass-tests-but-still--fail-security-302742611.html]
- ~19.7% package-hallucination rate (5.2% commercial / 21.7% open-source LLMs),
  205,474 unique fake names, 43% of fakes recur on all 10 reruns, 576K samples /
  16 LLMs. This is the measured slopsquatting attack surface. CONFIRMED.
  [Spracklen et al., USENIX Security 2025, arXiv:2406.10279,
   https://arxiv.org/html/2406.10279v3]
- 28.65M secrets leaked to public GitHub in 2025 (+34% YoY); AI-service leaks
  +81%; Claude-Code-assisted commits leak secrets at 3.2% vs 1.5% baseline;
  24,008 secrets in MCP config files. CONFIRMED.
  [GitGuardian State of Secrets Sprawl 2026, 2026-03-17,
   https://blog.gitguardian.com/the-state-of-secrets-sprawl-2026/]
- 13.4% of 3,984 public agent skills had critical issues, 36.82% had any issue,
  76 confirmed malicious (91% used prompt injection). The MCP/skill attack
  surface. CONFIRMED.
  [Snyk ToxicSkills, 2026-02-05,
   https://snyk.io/blog/toxicskills-malicious-ai-agent-skills-clawhub/]

Note on a number ELAI must NOT cite as fact:
  CORRECTED: the popular "62% of vibe-coded apps are flawed/vulnerable" headline
  is NOT a per-line vuln rate. The grounded figure is 62.07% of PROGRAMS
  vulnerable (per-program, formal-verification on 331K C programs), and OX's own
  62% headline is uncited in-text. Do not present 62% as a per-line rate.
  [FormAI-v2, arXiv:2404.18353, 2024-04-29, https://arxiv.org/abs/2404.18353]
  UNVERIFIED: the "91.5% of vibe-coded apps have vulnerabilities" (Kingbird) is a
  single-source commercial blog figure with undisclosed sampling; cite only with
  caveat or replace with the convergent independent numbers above.
  [SoftwareSeni relay, 2026, https://www.softwareseni.com/91-5-percent-of-vibe-coded-apps-have-vulnerabilities-and-what-the-q1-2026-research-actually-shows/]

==============================================================================
A. VIBESEC-NATIVE STARTUPS
==============================================================================
The pure-plays that branded themselves around securing AI-generated / vibe code.

name             | OX Security - VibeSec
vendor           | OX Security
what it does     | Embeds dynamic organizational security context directly into
                   AI code-generation agents so insecure code is prevented at
                   GENERATION time rather than detected after; delivered SaaS.
vibesec relevance| The namesake product for the category; canonical example of
                   the 2026 detect-after -> prevent-in-loop shift.
prevention-in-loop YES (claims generation-time prevention; SaaS, context-injection
                   into the agent, not a deterministic code-level block ELAI ships)
funding/traction | Launched 2025-09-30; funding not disclosed in launch release.
citation         | [PR Newswire, 2025-09-30,
                   https://www.prnewswire.com/news-releases/ox-launches-vibesec-to-prevent-vibe-code-vulnerabilities-from-ever-being-generated-302570609.html]

name             | Backslash Security
vendor           | Backslash
what it does     | 360-degree security for AI coding across Cursor / Windsurf /
                   Claude Code: fences agentic AI (file access/permissions), vets
                   MCP servers before deployment, injects security guidance into
                   prompts so generated code is "secure from the start," real-time
                   OSS vuln insight during generation.
vibesec relevance| One of the most explicitly vibe-coding-branded vendors; shipped
                   a dedicated MCP security solution for "Vibe Coding and AI-Native
                   Software Development" (Dec 2025).
prevention-in-loop YES (prompt-side guidance + MCP-vetting before deploy; guides
                   the model, not a hard deterministic code gate)
funding/traction | $19M Series A (2026-02), led by KOMPAS VC, after $8M seed;
                   ~$27M total. CONFIRMED.
citation         | [Help Net Security, 2026-02-10,
                   https://www.helpnetsecurity.com/2026/02/10/backslash-security-19-million-series-a-funding/]

name             | Corgea (BLAST)
vendor           | Corgea
what it does     | BLAST combines LLMs with AST-based analysis for semantic code
                   understanding, claims <5% false positives (vs >20% traditional
                   SAST); detects business-logic + AI-introduced flaws and
                   automates remediation.
vibesec relevance| Explicitly addresses flaws introduced by AI-assisted coding;
                   AI-native scanner + auto-fix in the dev loop.
prevention-in-loop YES claimed, but mechanically a scanner+autofix; gates at
                   scan/PR, not at write. Treat as partial for ELAI's purposes.
funding/traction | $2.6M seed (2024-11-07), led by Shorooq Partners, w/ YC,
                   Decacorn, Unbound; angel Jawed Karim. CONFIRMED-amount.
citation         | [FinSMEs, 2024-11-07,
                   https://www.finsmes.com/2024/11/corgea-raises-2-6m-in-seed-funding.html]

name             | ZeroPath
vendor           | ZeroPath
what it does     | AST + LLM SAST (claims 2x more vulns, 75% fewer FPs), generates
                   patches and auto-issues PRs; "Zero" is a persistent AppSec agent
                   over SAST/SCA/remediation.
vibesec relevance| Markets AI-native SAST for AI-coded software; argues "AI coding
                   assistants are not a SAST program."
prevention-in-loop PARTIAL (gates + auto-remediates in the PR loop, not at write)
funding/traction | Seed led by SurgePoint Capital, w/ YC and angel Paul Graham
                   (platform launch Jan 2025). UNVERIFIED: exact seed amount not
                   disclosed in reviewed sources.
citation         | [Fintech Global, 2025-01-22,
                   https://fintech.global/2025/01/22/zeropath-launches-ai-driven-code-security-platform-after-securing-seed-funding/]

name             | Endor Labs (platform + AURI + AI Code Security Review)
vendor           | Endor Labs
what it does     | AI-native AppSec for AI-generated + human code: dependency risk,
                   function-level reachability (vendor-marketed 92% noise cut),
                   secrets, malicious-package firewall, multi-agent PR review for
                   design/architectural flaws. AURI = free MCP-based tool embedding
                   into Cursor / Claude / Augment that scans AI-generated code
                   locally before it ships. Publishes the Agent Security League
                   benchmark.
vibesec relevance| Among the most explicit VibeSec vendors; integrates at the AI
                   assistant layer via MCP/Cursor hooks, intercepts package
                   installs, benchmarks agent security. Customers incl. OpenAI,
                   Cursor.
prevention-in-loop PARTIAL (AURI scans locally + malware firewall blocks at install;
                   PR-review focus catches design flaws AFTER write, not keystroke-
                   time block)
funding/traction | $93M Series B (2025-04-23), led by DFJ Growth, $163M total;
                   30x ARR growth, >5M apps, >1M scans/week. CONFIRMED funding.
                   CORRECTED: the "~10% functional-and-secure" stat is CMU/Columbia/
                   JHU SusVibes (arXiv:2512.03262); the "62%" Endor cites is ETH/UC-
                   Berkeley BaxBench and is a correctness ceiling, NOT a flaw rate.
                   The "92% noise reduction" is a vendor marketing number.
citation         | [TechCrunch, 2025-04-23,
                   https://techcrunch.com/2025/04/23/endor-labs-which-builds-tools-to-scan-ai-generated-code-for-vulnerabilities-lands-93m/]

name             | Qodo (formerly CodiumAI)
vendor           | Qodo
what it does     | Multi-agent code integrity: Qodo Gen (in-IDE context-aware code
                   + test gen), Qodo Cover (regression test gen, mutation testing,
                   coverage), Qodo Merge / PR-Agent (AI code review at PR time,
                   learning each org's quality bar).
vibesec relevance| Targets the AI-code trust gap by verifying AI/human code is
                   correct + tested across the whole system before merge; markets
                   "code verification as AI coding scales."
prevention-in-loop YES at PR-gate (review/verify before merge), but correctness/
                   quality oriented, not a security write-gate. Effectively partial
                   for security-specific prevention.
funding/traction | $70M Series B led by Qumra Capital (2026-03-30), $120M total;
                   prior $40M Series A (Sep 2024) + $11M seed. Customers Nvidia,
                   Walmart, Red Hat, Intuit; 1M+ devs. CONFIRMED.
citation         | [TechCrunch, 2026-03-30,
                   https://techcrunch.com/2026/03/30/qodo-bets-on-code-verification-as-ai-coding-scales-raises-70m/]

name             | Archipelo
vendor           | Archipelo
what it does     | Developer Security Posture Management (DevSPM): secures human +
                   AI-driven code before production - Developer Detection & Response,
                   AI Code Risk Monitoring, automated dev-tool inventory, posture
                   analytics.
vibesec relevance| Built explicitly to treat AI-driven code creation as a new
                   enterprise security pillar; monitors the developer/AI-tool layer.
prevention-in-loop PARTIAL (monitors + detects at the dev/tool layer; posture, not
                   a write-time block)
funding/traction | $12M ($4M pre-seed + $8M seed) led by Dell Technologies Capital,
                   out of stealth 2025-02-27; angels Eric Yuan, Andy Bechtolsheim.
                   CONFIRMED.
citation         | [TechCrunch, 2025-02-27,
                   https://techcrunch.com/2025/02/27/archipelo-comes-out-of-stealth-with-12m-funding-to-secure-human-and-ai-driven-code/]

name             | Pillar Security
vendor           | Pillar
what it does     | End-to-end AI app / agentic security: discovers + catalogs AI
                   assets, threat-models AI use cases, enforces adaptive
                   model-agnostic runtime guardrails (prompt injection, data leak,
                   evasion, poisoning). Discovered the "Rules File Backdoor" attack
                   on AI coding tools and CVE-2026-22708 (empty-allowlist RCE).
vibesec relevance| Focuses on the parallel AI-coding lifecycle OUTSIDE governed
                   CI/CD (local prompting/config); its research is squarely VibeSec.
prevention-in-loop PARTIAL (runtime guardrails block some actions; research-led, not
                   a code write-gate). Key input for ELAI gate design: string-match
                   allowlists are bypassable via shell built-ins / poisoned context,
                   so an ELAI gate must be context-aware, not regex-on-command.
funding/traction | $9M seed led by Shield Capital (2025-04-16); founded Oct 2023.
                   CONFIRMED.
citation         | [GlobeNewswire, 2025-04-16,
                   https://www.globenewswire.com/news-release/2025/04/16/3062627/0/en/Pillar-Security-Raises-9M-to-Help-Enterprises-Build-and-Run-Secure-AI-Software.html]

name             | Pixee
vendor           | Pixee (team behind Contrast Security)
what it does     | Automates the "final mile" of AppSec: intelligent triage +
                   production-ready code fixes delivered into GitHub/GitLab/Azure
                   DevOps; SAST coverage; reports 91% reclaimed remediation time,
                   76% fix merge rate (vendor figures).
vibesec relevance| Keeps remediation at GenAI velocity; auto-PRs fixes for
                   AI-introduced flaws.
prevention-in-loop PARTIAL (auto-fix on findings, post-scan; not a write-gate)
funding/traction | $15M seed co-led by Decibel + Wing VC (2025-05-22); angels incl.
                   HackerOne CTO Alex Rice. CONFIRMED.
citation         | [Business Wire, 2025-05-22,
                   https://www.businesswire.com/news/home/20250522057559/en/Pixee-Raises-$15M-to-Automate-Code-Security-to-Meet-the-Velocity-of-GenAI-Enabled-Developers]

name             | Mobb
vendor           | Mobb
what it does     | Ingests findings from Snyk / Checkmarx / GitHub Advanced
                   Security, identifies auto-fixable flaws, produces reviewable
                   fixes the developer approves before commit.
vibesec relevance| Remediation layer scaling fixes to AI-accelerated code volume;
                   sits on top of existing scanners.
prevention-in-loop PARTIAL (fix-before-commit, but depends on an upstream scanner
                   finding it first; not a write-gate)
funding/traction | $5.4M seed led by angel Ariel Maislos (Apr 2023); no later
                   Series A reported as of mid-2026. CONFIRMED.
citation         | [SecurityWeek, 2023,
                   https://www.securityweek.com/mobb-raises-5-4-million-in-seed-funding-for-automatic-vulnerability-fixing-tool/]

==============================================================================
B. INCUMBENT APPSEC / SAST PLATFORMS ADDING AI-CODE FEATURES
==============================================================================
The established SAST/AppSec leaders bolting AI-code detection, autofix, and
(increasingly) MCP-context onto mature engines. This is the crowded category
ELAI must NOT enter as a late scanner.

name             | Snyk DeepCode AI + Snyk Code + Snyk Agent
vendor           | Snyk
what it does     | Hybrid symbolic+generative SAST; finds, prioritizes (reachability/
                   popularity/exploit maturity), autofixes (claimed 85% accuracy,
                   19+ langs); real-time in-IDE as-you-type (VS Code/IntelliJ/Cursor,
                   claims 6.7x faster scans vs SonarQube). Snyk Agent reduces MTTR
                   84%+.
vibesec relevance| Core SAST engine powering Snyk's AI-code story; in-IDE is the
                   closest real-time SAST but it FLAGS, it does not block the edit.
prevention-in-loop PARTIAL (real-time advisory + PR autofix; not a write-gate)
funding/traction | Late-stage AppSec leader; $407.8M ARR 2025; was valued $7.4B,
                   BlackRock marked it to $3.7B, a sub-$3B PE bid was rejected;
                   growth decel to ~12% YoY Q2 2025. CONFIRMED financials.
citation         | [Snyk, https://snyk.io/platform/deepcode-ai/ ;
                   https://snyk.io/product/snyk-code/]

name             | Snyk Studio / Secure at Inception + Snyk MCP Server + Evo
vendor           | Snyk
what it does     | MCP server injects real-time security context into the AI agent
                   BEFORE it prompts the LLM; checks every AI-generated line before
                   it lands in a branch; blocks vulnerable/malicious deps the moment
                   they're suggested. Agent Fix uses dynamic few-shot over 35,000+
                   expert fixes. Evo (preview Oct 2025) = OODA-loop agentic security
                   orchestrator.
vibesec relevance| THE closest commercial analog to ELAI's prevention-in-loop
                   thesis. Direct competitor posture.
prevention-in-loop PARTIAL - this is the key nuance: Snyk feeds CONTEXT to the LLM
                   (probabilistic "guide the model"), it does NOT deterministically
                   GATE the output, and publishes no measured pre-write reduction
                   number. ELAI's deterministic gate is the differentiator here.
funding/traction | AI Trust Platform launched May 2025; Evo full rollout early 2026.
citation         | [Snyk, https://snyk.io/solutions/secure-at-inception-for-ai/]

name             | Semgrep Guardian
vendor           | Semgrep
what it does     | Detects + fixes vulns in AI-generated code the MOMENT an agent
                   writes it inside Claude Code/Cursor/Windsurf/Kiro; bundles MCP
                   server + file-write hooks + skills; scans every line against org
                   policy before it reaches a PR. Ships 27 AI Security rules, 122
                   Pro rules (malicious agent-skill patterns), 186 Shadow-AI rules;
                   autofix in public beta.
vibesec relevance| The closest direct analog to a "VibeSec" product: hook-on-every-
                   file-write + MCP, pre-PR enforcement, dedicated AI-security +
                   Shadow-AI rule packs.
prevention-in-loop YES (file-write hooks + MCP, pre-PR; this is genuine in-loop
                   enforcement and is ELAI's most direct competitor on mechanism)
funding/traction | Well-funded AppSec vendor; vendor cites 2025 Gartner MQ for AST.
citation         | [Semgrep, https://semgrep.dev/products/product-updates/detect-risks-in-ai-generated-code-with-semgrep-guardian/]

name             | Semgrep AppSec Platform + Semgrep MCP + Assistant
vendor           | Semgrep
what it does     | Real-time scanning in Cursor/Claude Code; MCP server lets the
                   agent invoke Semgrep mid-loop; 20,000+ pro rules, dataflow SAST,
                   SCA-with-reachability, secrets. Assistant = AI triage (auto-FP
                   filtering). Nov 2025 private beta: AI business-logic/IDOR
                   detection.
vibesec relevance| Rule engine an agent can call mid-loop, but the AGENT decides
                   whether to act - no deterministic block. ELAI could embed Semgrep
                   rules behind a hard PreToolUse deny (engine + gate).
prevention-in-loop PARTIAL (agent-invokable, advisory; the platform itself does not
                   hard-block at write)
funding/traction | UNVERIFIED: "Assistant handles ~60% of triage, 96% agreement" -
                   PARTIALLY_CONFIRMED: ~60% is of NEW findings YTD (approx); 96% is
                   agreement on TRUE POSITIVES only (FP agreement was 41%). The
                   "1.9x IDOR recall vs Claude Code" is a non-reproducible vendor
                   alpha benchmark on private repos, hybrid-vs-standalone.
citation         | [Semgrep, https://semgrep.dev/products/semgrep-appsec-platform/ ;
                   triage figures https://semgrep.dev/blog/2025/semgrep-is-confidently-handling-60-of-all-triage-for-users-without-reducing-coverage/]

name             | GitHub Copilot Autofix (GitHub Advanced Security / CodeQL)
vendor           | GitHub (Microsoft)
what it does     | Generates fix suggestions for CodeQL code-scanning alerts at PR
                   time (CodeQL + GPT-4o + Copilot APIs); 11 langs; covers >90% of
                   alert types in JS/TS/Java/Python, remediates >2/3 with little/no
                   editing.
vibesec relevance| PR-gate, but explicitly DETECT-THEN-FIX on already-written code.
                   The default incumbent every GitHub-hosted AI workflow touches;
                   strengthens ELAI's pre-write positioning by contrast.
prevention-in-loop NO (operates on alerts AFTER code is written, in PRs / default-
                   branch scans)
funding/traction | GitHub/Microsoft; handled >460,000 alerts in 2025, 0.66h avg
                   resolution vs 1.29h without Autofix; median fix 28min vs 1.5h (3x),
                   SQLi 18min vs 3.7h (12x), XSS 22min vs ~3h (7x). CONFIRMED.
citation         | [GitHub Blog, 2026-03-23,
                   https://github.blog/security/application-security/github-expands-application-security-coverage-with-ai-powered-detections/ ;
                   times https://github.blog/news-insights/product-news/secure-code-more-than-three-times-faster-with-copilot-autofix/]

name             | SonarQube AI Code Assurance + AI CodeFix ("Fight AI Slop")
vendor           | Sonar (SonarSource)
what it does     | AI Code Assurance detects AI-generated code + applies enhanced
                   verification rules (IDE + CI/CD + PR decoration); AI CodeFix
                   generates LLM fixes (Claude Sonnet 4 recommended / GPT-5.1).
                   SonarQube for IDE (SonarLint) = real-time inline analysis in
                   Cursor/Windsurf/Trae; Agentic Analysis exposes an MCP server so
                   Claude Code/Cursor run CI-grade verification + re-verify before PR.
vibesec relevance| Explicit AI-code DETECTION + a verification gate + IDE real-time;
                   Agentic-Analysis MCP path is the closest Sonar gets to in-loop
                   verification - but it is verify-and-fix, not a deterministic
                   pre-write deny. Sonar advises+fixes; ELAI gates.
prevention-in-loop PARTIAL (real-time IDE advisory + PR-decoration gate + MCP verify)
funding/traction | Established code-quality leader. CONFIRMED stat: Sonar's 2026
                   State of Code survey (1,100+ devs) found AI = 42% of all committed
                   code (self-estimate, "AI-generated or assisted"), rising to 65% by
                   2027. AI CodeFix backends (Claude Sonnet 4 / GPT-5.1) CONFIRMED.
citation         | [Sonar, https://docs.sonarsource.com/sonarqube-cloud/ai-capabilities/ai-code-assurance ;
                   42% stat https://www.sonarsource.com/company/press-releases/sonar-data-reveals-critical-verification-gap-in-ai-coding/]

name             | Checkmarx One Assist - Developer Assist
vendor           | Checkmarx
what it does     | Developer Assist runs in the IDE inner loop (Copilot, Cursor,
                   Windsurf, VS Code, JetBrains): real-time in-IDE AI guardrails +
                   pre-commit prevention for human AND AI code (SAST/SCA/malicious-
                   packages/IaC/secrets) via a Checkmarx MCP server. AI Query Builder
                   (GPT-4) auto-writes CxQL queries (-65% time); AI Guided Remediation
                   writes safe fixes.
vibesec relevance| Multi-loop agentic AppSec with explicit in-IDE guardrails for
                   AI-native IDEs; strong prevention-in-loop posture via MCP +
                   pre-commit.
prevention-in-loop YES (in-IDE guardrails + pre-commit prevention via MCP - one of
                   the stronger incumbent in-loop postures)
funding/traction | Established enterprise AppSec vendor.
citation         | [Checkmarx, https://checkmarx.com/product/checkmarx-one-assist/]

name             | Veracode Fix
vendor           | Veracode
what it does     | Patented (US, Apr 2025) AI tool trained specifically for security
                   remediation on proprietary scan data + expert-validated patches;
                   covers 70%+ of detected flaws across ~10-11 langs, cuts
                   remediation to minutes.
vibesec relevance| Remediation-focused, not generation-time; notable for a
                   purpose-built (not general-LLM) fix model + the 45%-vuln research.
prevention-in-loop NO (post-scan remediation)
funding/traction | Established AppSec leader; patent granted Apr 2025.
citation         | [Veracode, https://www.veracode.com/products/fix/]

name             | JFrog AI-Generated Code Validation + Agentic Remediation
vendor           | JFrog
what it does     | GitHub Copilot taps JFrog code scanning + OSS package security via
                   JFrog remote MCP server to validate AI-generated code (compliant/
                   secure/deployment-ready) in real time; agentic remediation suggests
                   AI changes backed by JFrog SAST, Catalog, Curation.
vibesec relevance| Combines AI-code validation with software-supply-chain (package
                   curation) at generation time via MCP - a dimension pure-SAST
                   VibeSec tools miss.
prevention-in-loop YES (real-time validation + package curation at generation time
                   via MCP)
funding/traction | Public company; named Visionary in 2025 Gartner MQ for AST.
citation         | [JFrog, https://jfrog.com/blog/jfrog-introduces-ai-generated-code-validation/]

name             | Checkmarx / Veracode / Black Duck Code Sight (IDE real-time SAST)
vendor           | Checkmarx, Veracode, Black Duck
what it does     | SAST in the IDE analyzing code (incl. AI snippets) in real time as
                   accepted, flagging hardcoded creds, injection, weak crypto.
vibesec relevance| Real-time but reactive (flags after the snippet is in the buffer);
                   not woven into token-level decode. ELAI's gate-before-emit is
                   earlier. Veracode is also the source of the 45% market stat.
prevention-in-loop PARTIAL (real-time advisory, flags not blocks)
funding/traction | Mature enterprise SAST; Veracode 45% figure is the most-cited
                   market datapoint.
citation         | [Veracode, https://www.veracode.com/blog/secure-ai-code-generation-in-practice/]

==============================================================================
C. SUPPLY-CHAIN / SLOPSQUATTING DEFENDERS
==============================================================================
The category that maps directly onto ELAI's anti-slopsquatting HARD RULE. Decisive
sub-axis: does it block at the INSTALL BOUNDARY (before fetch/extract), and does it
catch NON-EXISTENT (hallucinated) names, not just KNOWN-malicious ones?

name             | Socket (platform)
vendor           | Socket
what it does     | Proactively analyzes OSS packages for malicious behavior
                   (backdoors, typosquatting, obfuscated/install-time code) across
                   6+ ecosystems; blocks >100 supply-chain attacks/week at GitHub-PR
                   / install point.
vibesec relevance| Counters slopsquatting / hallucinated-dep risk; gates at install/
                   PR, not post-deploy.
prevention-in-loop YES (blocks at install/PR)
funding/traction | $60M Series C at $1B valuation (2026-05-20), $125M total, 27,000+
                   orgs, 1.5M repos; customers Anthropic, xAI, Replit, Cursor, Vercel,
                   Figma. CONFIRMED (Thrive Capital led; earlier $40M Series B 2024).
citation         | [Socket, 2026-05-20, https://socket.dev/blog/series-c]

name             | Socket Firewall (sfw)
vendor           | Socket
what it does     | Free CLI wrapper spinning up an ephemeral HTTP proxy intercepting
                   package-manager traffic; checks each package (top-level AND
                   transitive) against the Socket API BEFORE fetch/extract/install.
                   Free tier covers npm/yarn/pnpm, pip/uv, AND cargo. No API key.
vibesec relevance| THE closest production primitive to ELAI's goal of blocking a bad
                   dependency at the install boundary; multi-ecosystem incl. cargo
                   (matches ELAI's Rust stack) and free. CONFIRMED: cargo + pip/uv +
                   npm/yarn/pnpm all in the free tier, transitive deps blocked at
                   install. CAVEAT: blocks KNOWN-malicious, not pure NON-EXISTENCE -
                   a freshly-registered slop name can slip until Socket flags it.
prevention-in-loop YES (install-boundary block, transitive, multi-ecosystem)
funding/traction | (Socket Firewall Free GA 2025-09-30; parent funding as above.)
citation         | [Socket, 2025-09-30, https://socket.dev/blog/introducing-socket-firewall ;
                   docs https://docs.socket.dev/docs/socket-firewall-free]

name             | Aikido SafeChain
vendor           | Aikido Security
what it does     | Open-source wrapper for npm/npx/yarn/pnpm that intercepts install
                   commands and checks them against Aikido Intel before anything
                   reaches the machine.
vibesec relevance| Direct competitor pattern to Socket Firewall at the npm install
                   boundary; JS-only today, so LESS aligned with ELAI's Rust/Python
                   stack than Socket Firewall.
prevention-in-loop YES (install-boundary block, npm-family only)
funding/traction | Aikido is the vendor behind the Feb 2026 slopsquatting research
                   (Charlie Eriksen's react-codeshift discovery; PhantomRaven
                   tracking).
citation         | [Aikido, 2026, https://www.aikido.dev/blog/slopsquatting-ai-package-hallucination-attacks]

name             | Endor Labs (AI-code SCA + Malware Package Firewall)
vendor           | Endor Labs
what it does     | Function-level reachability triage (vendor-marketed 92% noise cut),
                   SBOM/VEX, plus a malware package firewall that blocks malicious
                   packages before they enter the codebase.
vibesec relevance| Heavier enterprise SCA; the malware firewall is the in-loop block,
                   but core value is reachability triage of KNOWN CVEs, not
                   hallucination/existence gating at codegen time.
prevention-in-loop PARTIAL (firewall blocks at install; core value is CVE triage)
funding/traction | $93M Series B (2025-04). The 92% noise-reduction is a vendor
                   marketing number (no independent validation; competitors quote
                   their own: Hopper 93%, Socket 80%+, Xygeni 70%). CONFIRMED-as-
                   marketing.
citation         | [TechCrunch, 2025-04-23,
                   https://techcrunch.com/2025/04/23/endor-labs-which-builds-tools-to-scan-ai-generated-code-for-vulnerabilities-lands-93m/]

name             | OSV-Scanner v2
vendor           | Google / OpenSSF (OSV.dev)
what it does     | Matches a project's dependency graph against the OSV vuln DB; v2
                   (Mar 2025) adds guided remediation (minimal upgrade set), container
                   layer-aware scanning, interactive HTML; built on OSV-SCALIBR.
vibesec relevance| Strong free CVE scanner for ELAI's audit gate, but it does NOT
                   detect package NON-EXISTENCE/hallucination - a slop name has no
                   CVE, so OSV-Scanner is BLIND to slopsquatting by itself. Pair with
                   an existence/registry check.
prevention-in-loop NO (CVE scan; can run in CI but does not gate at install and is
                   blind to non-existence)
funding/traction | Google-maintained, free/open-source; OSV.dev backs many other
                   tools.
citation         | [Google Security Blog, 2025-03,
                   https://security.googleblog.com/2025/03/announcing-osv-scanner-v2-vulnerability.html]

name             | Snyk AI Trust Platform / DeepCode AI (slopsquatting story)
vendor           | Snyk
what it does     | DeepCode AI combines multiple models + symbolic AI to avoid
                   single-model hallucination; AI Trust Platform adds AI-driven
                   reachability + marketed slopsquatting detection.
vibesec relevance| Enterprise SCA with a marketed slopsquatting story.
prevention-in-loop PARTIAL - UNVERIFIED: exact in-loop blocking behavior (codegen vs
                   CI vs install) is NOT clearly specified in public material; treat
                   the "slopsquatting detection" claim as marketing until verified.
funding/traction | Established SCA/SAST vendor (financials above).
citation         | [Snyk, https://snyk.io/articles/slopsquatting-mitigation-strategies/]

name             | PyPI registry defenses (typosquat detection, malware-report API,
                   trusted publishing / attestations)
vendor           | Python Software Foundation / PyPI
what it does     | Auto-flags typosquatting at project creation, periodic expired-
                   domain checks, programmatic malware-reporting API (2,000+ reports
                   2025, 92% remediated <24h), trusted publishing + attestations (17%
                   of 2025 uploads attested).
vibesec relevance| Upstream registry hardening shrinks the window a slop name stays
                   live, but does NOT stop an AI agent from installing a brand-new
                   slop package before takedown - the gap an in-loop ELAI check fills.
prevention-in-loop PARTIAL (registry-side; reduces but does not close the codegen-time
                   install window)
funding/traction | Funded via Alpha-Omega; Seth Larson (coined "slopsquatting")
                   drives attestation/archival work.
citation         | [PyPI Blog, 2025-12-31,
                   https://blog.pypi.org/posts/2025-12-31-pypi-2025-in-review/]

==============================================================================
D. SECRETS / PII PREVENTION
==============================================================================
The category with the cleanest measured demand signal (Claude-Code commits leak
secrets at 3.2% vs 1.5% baseline; 24,008 secrets in MCP configs). Decisive axis:
does it block at AI-AGENT runtime (prompt / tool-call) or only at git-push / CI ?

name             | GitGuardian ggshield + AI hook + GitGuardian MCP
vendor           | GitGuardian
what it does     | CLI scanner for 500+ secret types; installs as pre-commit /
                   pre-push / pre-receive git hooks to block secrets before they reach
                   the repo. Apr 2026 AI hook scans AI-tool prompts AND agent
                   tool-calls (pre-prompt, pre-tool, post-tool) for Cursor, Claude
                   Code, VS Code+Copilot, blocking secrets before they reach the model
                   or are executed. Pre-receive now fail-closed on 5xx (2026 breaking
                   change).
vibesec relevance| THE most directly relevant secrets control: the only widely-
                   documented control that blocks secrets at AI-AGENT runtime (prompt
                   + tool-call) inside Cursor/Claude Code - exactly the vibe loop.
prevention-in-loop YES (blocks at AI-agent runtime: pre-prompt + pre-tool, plus
                   pre-commit/pre-push/pre-receive)
funding/traction | Established secrets vendor; State of Secrets Sprawl 2026 report
                   (Mar 17 2026) is the data backbone (3.2% vs 1.5%, 24,008 MCP
                   secrets). CONFIRMED data.
citation         | [Help Net Security, 2026-04-15,
                   https://www.helpnetsecurity.com/2026/04/15/product-showcase-gitguardian-ggshield-ai-hook/]

name             | GitHub Secret Protection + Push Protection
vendor           | GitHub (Microsoft)
what it does     | Native secret scanning (~150 partner integrations) with push
                   protection that blocks secrets before they reach the repository;
                   AI-powered password detection (two-model, claimed 94% FP cut).
                   Blocked 4.4M secrets in 2024.
vibesec relevance| Strong push-time block but operates at git-PUSH, not at AI-agent/
                   IDE generation time; Mar 2026 changelog added detectors but no
                   AI-codegen-specific hook. 34.6M secrets still got through in 2024
                   (push protection not enabled/effective everywhere).
prevention-in-loop YES at push (but no AI-agent-runtime hook; later in the loop than
                   GitGuardian's AI hook)
funding/traction | GitHub-owned; standalone product since 2025-04-01 at $19/active
                   committer/mo, now on Team plan. CONFIRMED.
citation         | [GitHub Changelog, 2025-03-04,
                   https://github.blog/changelog/2025-03-04-introducing-github-secret-protection-and-github-code-security/]

name             | TruffleHog
vendor           | Truffle Security Co.
what it does     | Detects, classifies, and LIVE-VERIFIES 800+ credential types by
                   calling provider APIs to confirm a secret is still active; 20+
                   detectors map credential permissions. CLI / pre-commit / GitHub
                   Action. ~250K scans/day, ~23K stars.
vibesec relevance| Live-verify (is the leaked key still live?) is high-value for
                   triaging the flood of AI-leaked secrets. Can run as pre-commit
                   (in-loop block) but is most commonly CI/post-hoc.
prevention-in-loop PARTIAL (pre-commit block possible; usually deployed post-hoc;
                   live-verify is the differentiator)
funding/traction | Open-source, broadly adopted; commercial offering via Truffle
                   Security.
citation         | [GitHub, https://github.com/trufflesecurity/trufflehog]

name             | Gitleaks
vendor           | Gitleaks community (maintainer Zach Rice)
what it does     | Detects 160+ secret types via regex/entropy; pre-commit hook
                   (--staged, sub-second on a diff), CI, full-history scan, SARIF.
                   v8.24.x in 2026, ~25.9K stars.
vibesec relevance| Free pre-commit block is the cheapest in-loop guardrail for
                   vibe-coded repos; wires into Husky/pre-commit before AI commits
                   land. No live verification, no AI-agent-time hook.
prevention-in-loop YES (free pre-commit block; commit-time, not agent-runtime)
funding/traction | Open-source, ~25.9K stars; the default when teams won't pay
                   per-dev rates.
citation         | [GitHub, https://github.com/gitleaks/gitleaks]

name             | AWS git-secrets
vendor           | AWS (awslabs)
what it does     | Scans commits / messages / merge history against prohibited regex
                   (e.g. AWS AKIA keys); rejects the commit on match. pre-commit /
                   pre-receive; pairs with AWS Secrets Manager / IAM Access Analyzer.
vibesec relevance| Lightweight AWS-key-focused pre-commit block; useful where AI code
                   hardcodes AWS creds. Narrower coverage than Gitleaks/TruffleHog,
                   no live-verify, no AI-agent hook.
prevention-in-loop YES (pre-commit block; AWS-key-narrow)
funding/traction | AWS-maintained open-source.
citation         | [GitHub, https://github.com/awslabs/git-secrets]

name             | Nightfall AI
vendor           | Nightfall AI
what it does     | ML-based DLP (100+ models, claimed 95% accuracy) detecting secrets,
                   PII, PHI, PCI across SaaS/endpoints/email/browser/GenAI prompts.
                   GitHub Action scans PRs inline. Jan 2026 AI Browser Security +
                   endpoint DLP block/redact/warn on PII in real time, incl. in
                   AI-tool prompts.
vibesec relevance| Adds the PII/PHI dimension (matches Escape's 175-PII finding:
                   medical records, IBANs) that pure secret scanners miss. PR-time
                   DLP is post-commit, but 2026 AI-browser/endpoint detectors give
                   real-time block/redact at the AI-interaction layer.
prevention-in-loop PARTIAL (PR-time DLP post-commit; 2026 endpoint/browser path gives
                   real-time block at the AI-interaction layer)
funding/traction | Commercial DLP vendor; 2026 launches focused on AI-tool exfil.
citation         | [Nightfall, https://www.nightfall.ai/firewall-for-ai]

==============================================================================
E. PREVENTION-IN-THE-LOOP / AGENT-INTEGRATED GUARDRAILS
==============================================================================
The category that IS ELAI's lane: deterministic-or-near-deterministic enforcement
inside the agent loop. This is where the genuine prior art - and the cautionary
tales - live.

name             | Claude Code PreToolUse hooks
vendor           | Anthropic
what it does     | Fires before any tool runs (Write/Edit/Bash/MCP). A hook returns
                   permissionDecision="deny" (exit 0 + JSON) OR exit code 2 to
                   hard-block the tool call before execution. Fires regardless of
                   permission mode; not bypassed by --dangerously-skip-permissions.
vibesec relevance| THE exact mechanism ELAI's PreToolUse path uses. The only
                   mainstream primitive that BLOCKS a code-write before it lands.
                   ELAI's edge is a curated vuln-class ruleset on top (secure-by-
                   default), not leaving it as DIY plumbing.
prevention-in-loop YES (deterministic-by-design hard block before tool execution)
funding/traction | Anthropic. PARTIALLY_CONFIRMED on "deterministic": schema +
                   both exit-code paths confirmed verbatim, BUT issue #37210
                   (2026-03-21, closed not-planned) showed a well-formed deny IGNORED
                   for the Edit tool, so verify per-tool for security enforcement.
citation         | [Anthropic, https://code.claude.com/docs/en/hooks]

name             | Cursor Hooks (1.7+)
vendor           | Cursor (Anysphere)
what it does     | beforeShellExecution / beforeMCPExecution / beforeReadFile return
                   allow/deny/ask to block before execution; afterFileEdit / stop are
                   reactive.
vibesec relevance| Same hook shape as Claude Code but WEAKER: CONFIRMED fail-OPEN by
                   default (failClosed:true required), NO beforeFileEdit so the file
                   write is NOT pre-gated, and a confirmed precedence bug where the
                   allow-list overrides hook verdicts (only "deny" reliably works).
                   ELAI can claim fail-closed-by-default + pre-write code gate as a
                   concrete differentiator.
prevention-in-loop PARTIAL (pre-gates shell/MCP/read, NOT file-write; fail-open default)
funding/traction | Cursor/Anysphere.
citation         | [Cursor, https://cursor.com/docs/hooks ; analysis InfoQ 2025-10,
                   https://www.infoq.com/news/2025/10/cursor-hooks/]

name             | CodeGate
vendor           | Stacklok
what it does     | Local Docker proxy between the AI coding assistant and the LLM;
                   redacted secrets/PII from prompts, blocked malicious/deprecated
                   package suggestions via a real-time DB + RAG, flagged insecure
                   patterns - a genuine "block before write" pattern.
vibesec relevance| The SINGLE closest prior art to ELAI's deterministic-local-gate
                   architecture - and it was ARCHIVED 2025-06-05, unmaintained.
                   Honest signal: the OSS local-proxy approach existed and did NOT
                   sustain. ELAI should study WHY (UX friction? false positives?
                   monetization?) before claiming the lane is wide open.
prevention-in-loop YES (was a real block-before-write local gate; now dead)
funding/traction | Stacklok raised $17.5M (Series A, May 2023, sigstore/Kubernetes
                   founders); CodeGate repo archived 2025-06-05. CONFIRMED archived.
citation         | [GitHub, https://github.com/stacklok/codegate]

name             | Cycode AI Guardrails
vendor           | Cycode
what it does     | Uses native Claude Code + Cursor hooks to scan/log/block at three
                   surfaces - prompt submission, file reads into agent context, MCP
                   tool execution - before anything leaves the machine. Secrets-leak
                   focused.
vibesec relevance| Direct competitor on block-before-tool-call but scoped to
                   secrets/exfil, not vuln-class-in-generated-code. ELAI competes by
                   gating the CODE itself (SQLi/XSS/unsafe-deser).
prevention-in-loop YES (block-before-tool-call; secrets/exfil-scoped)
funding/traction | Cycode (established ASPM vendor).
citation         | [Cycode, https://cycode.com/blog/ai-guardrails-real-time-ide-security/]

name             | Noma Security (Cursor runtime)
vendor           | Noma Security
what it does     | Markets first real-time agent runtime security for Cursor:
                   destructive-command guardrails (block shell/tool calls that alter/
                   destroy prod) + exfiltration detection/blocking at the moment intent
                   becomes action.
vibesec relevance| Competitor on destructive-op/exfil blocking, not code-vuln.
                   Confirms the market converging on hooks-as-enforcement-boundary,
                   validating ELAI's architecture.
prevention-in-loop YES (destructive-op/exfil block; not code-vuln)
funding/traction | Noma Security.
citation         | [Noma, https://noma.security/blog/securing-the-agentic-frontier-noma-unveils-the-first-real-time-agent-runtime-security-for-cursor/]

name             | Invariant Guardrails + Gateway
vendor           | Invariant Labs (now under Snyk)
what it does     | Transparent MCP/LLM proxy (Gateway) evaluates policy rules on every
                   request before AND after; action="block" denies a tool call that
                   violates policy (data-flow control, PII/secrets, tool restrictions,
                   toxic flows).
vibesec relevance| True in-loop blocking, but data-flow/PII/tool-restriction scoped,
                   PROXY-deployed (not write-time). ELAI's in-process write-gate is
                   lower-latency and code-vuln-aware vs Invariant's flow-policy proxy.
prevention-in-loop YES (in-loop block; flow/PII-scoped, proxy-deployed)
funding/traction | Invariant Labs (acquired into Snyk).
citation         | [Invariant, https://invariantlabs.ai/guardrails]

name             | NeMo Guardrails
vendor           | NVIDIA
what it does     | Programmable input/output/execution rails; output rails can refuse
                   (block from reaching the user) or modify generated output; custom
                   Python execution rails.
vibesec relevance| Can block output but tuned for jailbreak/PII/toxicity, NOT
                   code-vuln classes. Would need custom rails to gate code; not a
                   coding-agent product.
prevention-in-loop PARTIAL (can block output; not code-vuln-tuned, not a coding agent)
funding/traction | NVIDIA, open-source.
citation         | [NVIDIA, https://docs.nvidia.com/nemo/guardrails/latest/index.html]

name             | Guardrails AI
vendor           | Guardrails AI
what it does     | Input/output validators (toxicity, PII scrubbing, structure) that
                   can block or fix LLM output; integrates with NeMo Guardrails.
vibesec relevance| Validator library, not a coding-agent gate. Relevant only as a
                   building block for an output-level code check.
prevention-in-loop PARTIAL (validator framework; building block only)
funding/traction | Guardrails AI, open-source.
citation         | [Guardrails AI, https://guardrailsai.com/blog/nemoguardrails-integration]

name             | Prompt Security (acquired by SentinelOne)
vendor           | SentinelOne (formerly independent Prompt Security)
what it does     | Visibility into all enterprise GenAI usage, controls employee use
                   of AI tools (ChatGPT/Gemini/Claude/Cursor), eliminates shadow-AI,
                   prevents AI-related data leakage + prompt injection at runtime.
vibesec relevance| Adjacent to VibeSec: governs developers' USE of AI tools at
                   runtime rather than scanning generated code; now folded into
                   SentinelOne Singularity.
prevention-in-loop NO (runtime usage governance, not a code/write gate)
funding/traction | CORRECTED: acquired by SentinelOne for ~$180M cash-and-stock
                   (announced 2025-08-05, closed 2025-09-05) - this is an ACQUISITION
                   PRICE, NOT an $18M Series A (that was a separate Nov 2024 round).
                   CONFIRMED via SEC 8-K.
citation         | [SentinelOne, 2025-08-05,
                   https://www.sentinelone.com/press/sentinelone-to-acquire-prompt-security-to-advance-genai-security/]

name             | Snyk Studio / Secure at Inception (cross-listed from B)
vendor           | Snyk
what it does     | (See B.) MCP-injects security context into the agent before it
                   prompts the LLM; checks each AI line before it lands in a branch;
                   blocks vulnerable/malicious deps the moment suggested.
vibesec relevance| The most direct commercial competitor to ELAI's thesis.
prevention-in-loop PARTIAL (context-injection / guide-the-model, not a deterministic
                   gate; no measured pre-write reduction published)
citation         | [Snyk, https://snyk.io/solutions/secure-at-inception-for-ai/]

Architectural prior art (research-grade prevention patterns ELAI should build on):

name             | CaMeL (Capabilities for Machine Learning)
vendor           | Google DeepMind / ETH Zurich
what it does     | Deterministic protective layer: a privileged LLM emits code over
                   symbolic variables, attaches capability metadata to every value,
                   enforces information-flow/access-control so untrusted data can
                   never alter control flow or reach unauthorized sinks.
vibesec relevance| Reference architecture for ELAI's deterministic tool-gating: prove
                   injected repo/issue content cannot influence which tools fire or
                   where secrets flow. 77% provable-security coverage on AgentDojo at
                   ~2.8x token cost.
prevention-in-loop YES (deterministic capability enforcement - the gold-standard
                   pattern)
funding/traction | Research (arXiv).
citation         | [arXiv:2503.18813, https://arxiv.org/abs/2503.18813]

name             | AgentDojo Tool Filter
vendor           | ETH Zurich / Invariant (AgentDojo authors)
what it does     | Deterministic defense: the agent commits to the minimal tool set
                   needed BEFORE observing any untrusted data, removing all other
                   tools from the action space.
vibesec relevance| Cheapest high-yield defense measured: cut GPT-4o targeted ASR from
                   57.7% to 6.84% while keeping 73% utility. Maps directly to ELAI's
                   least-privilege per-phase tool scoping.
prevention-in-loop YES (deterministic least-privilege)
funding/traction | Research (arXiv).
citation         | [arXiv:2406.13352, https://arxiv.org/html/2406.13352v3]

name             | Dual LLM pattern
vendor           | Simon Willison (concept) / adopted in CaMeL
what it does     | A privileged tool-having LLM never sees untrusted content; a
                   quarantined tool-less LLM processes untrusted data and returns only
                   symbolic variables; a non-LLM Controller mediates.
vibesec relevance| Foundational pattern behind CaMeL; the architectural principle ELAI
                   should adopt for any agent touching both secrets and untrusted repo
                   content.
prevention-in-loop YES (architectural pattern)
funding/traction | Concept / widely adopted.
citation         | [Simon Willison, 2025-06-13,
                   https://simonwillison.net/2025/Jun/13/prompt-injection-design-patterns/]

name             | Spotlighting (datamarking / encoding)
vendor           | Microsoft Research
what it does     | Marks/transforms untrusted input (interleaved tokens or base64) so
                   the model can structurally distinguish data from instructions.
vibesec relevance| Cheap input-side hardening (ASR <2% on GPT-family) but model-
                   dependent and bypassable; a layer, not the only control. Useful for
                   ELAI tool-output ingestion.
prevention-in-loop PARTIAL (input transformation; bypassable)
funding/traction | Research (arXiv).
citation         | [arXiv:2403.14720, https://arxiv.org/abs/2403.14720]

name             | Claude Code sandboxing + auto-mode classifier
vendor           | Anthropic
what it does     | OS-level filesystem + network isolation (approved dirs/servers
                   only), a server-side prompt-injection probe flagging suspicious tool
                   output, and a reasoning-blind transcript classifier gating actions.
vibesec relevance| Production example of layered input-probe + deterministic OS
                   sandbox; residual 17% false-negative on overeager actions shows
                   classifiers alone are insufficient. Sandbox cut permission prompts
                   ~84%.
prevention-in-loop YES (deterministic OS sandbox + classifier; classifier layer is
                   probabilistic)
funding/traction | Anthropic.
citation         | [Anthropic, https://www.anthropic.com/engineering/claude-code-sandboxing]

name             | Docker Sandboxes
vendor           | Docker
what it does     | Workspace-scoped, isolated-identity, sandboxed execution for AI
                   coding agents (Git-worktree branch-scoping) to contain filesystem
                   access, privilege inheritance, secrets leakage, prompt-injection
                   damage, and malicious skills.
vibesec relevance| Directly addresses the autonomous-agent over-privilege surface
                   (the Replit/Kiro/Claude-Code incident classes). Blast-radius
                   containment is the "cant wreck your system" half of ELAI's
                   north star.
prevention-in-loop PARTIAL (contains blast radius; does not gate the code/vuln itself)
funding/traction | Docker product; its horror-stories blog is the primary aggregator
                   of the agent-incident taxonomy.
citation         | [Docker, https://www.docker.com/blog/ai-coding-agent-horror-stories-security-risks/]

==============================================================================
F. MCP / AGENT-SKILL SECURITY
==============================================================================
The newest surface (Snyk ToxicSkills: 76 malicious skills, 91% via prompt
injection; ELAI's skill-import-policy already mandates whole-directory mcp-scan).
Decisive axis: does it HARD-BLOCK a malicious install, or only score/warn?

name             | Snyk agent-scan / mcp-scan
vendor           | Snyk (snyk-labs / Invariant Labs)
what it does     | Scans MCP servers + agent skills for 15+ risks (prompt injection,
                   tool poisoning/shadowing, toxic flows, malware, secrets) across
                   Claude/Cursor/Windsurf/Gemini; PyPI uvx snyk-agent-scan; claims
                   90-100% recall / 0% FP on top-100 legit skills. Toxic-flow analysis
                   found the GitHub MCP toxic flow.
vibesec relevance| The scanner ELAI's skill-import-policy already mandates (whole-
                   directory recursive). Right tool for the skill supply chain, but
                   scans-and-scores; ELAI's PreToolUse + whole-dir (incl. tests/-dir,
                   the documented Anthropic-scanner bypass) closes the gap as a HARD
                   block.
prevention-in-loop PARTIAL (pre-deploy scan/score; detect-not-prevent at runtime)
funding/traction | Snyk/Invariant; ToxicSkills study (3,984 skills) is the data base.
citation         | [GitHub, https://github.com/snyk/agent-scan ;
                   toxic flow https://invariantlabs.ai/blog/toxic-flow-analysis]

name             | Snyk + Vercel skills.sh / Snyk + Tessl Registry
vendor           | Snyk + Vercel / Snyk + Tessl
what it does     | Skills scanned at publish AND install (npx skills add triggers Snyk
                   API before reaching the machine; Tessl shows scores at browse +
                   warns at install). Outputs a "Security Verified" badge / score.
vibesec relevance| Registry-level prevention but ADVISORY. CONFIRMED: neither hard-
                   blocks a malicious install - Tessl shows a gate with explicit
                   "proceed or back out" user override; Vercel/skills.sh surfaces a
                   badge + scores and HIDES malicious skills from search, but a direct-
                   URL install still proceeds with only a warning. ELAI's local fail-
                   closed PreToolUse HIGH/CRITICAL hard-block is the enforcement these
                   registries stop short of.
prevention-in-loop PARTIAL (publish/install-time badge/score/warn/hide, NOT hard-block)
funding/traction | Snyk + Vercel/Tessl (2026-02 / 2026-03).
citation         | [Snyk, https://snyk.io/blog/snyk-vercel-securing-agent-skill-ecosystem/ ;
                   Vercel changelog https://vercel.com/changelog/automated-security-audits-now-available-for-skills-sh]

name             | Endor Labs / Oasis Security / Pillar Security (IDE/agent governance
                   + hook examples + research)
vendor           | Endor Labs / Oasis Security / Pillar Security
what it does     | Endor ships cursor-hook examples + AI-code guidance; Oasis governs
                   agentic IDE access (intent-based, command guardrails for recursive-
                   delete / destructive-git / DB mutation); Pillar researches the
                   attacks (CVE-2026-22708 empty-allowlist RCE, Rules File Backdoor).
vibesec relevance| Pillar research is a key input for ELAI gate design: string-matching
                   allowlists are bypassable via shell built-ins / poisoned context, so
                   an ELAI gate MUST be context-aware, not just regex-on-command.
prevention-in-loop PARTIAL (governance + hook examples + research; not a unified block)
funding/traction | Endor ($93M Series B), Oasis, Pillar ($9M seed).
citation         | [Pillar, https://www.pillar.security/blog/the-agent-security-paradox-when-trusted-commands-in-cursor-become-attack-vectors]

==============================================================================
G. OFFENSIVE / AUTONOMOUS FIND-AND-FIX (adjacent reference class)
==============================================================================
Not VibeSec-defensive per se, but the proof that agentic find/fix is real, and the
closest measured baselines for an autonomous prevention pipeline.

name             | XBOW
vendor           | XBOW (founder Oege de Moor)
what it does     | Autonomous offensive-security / AI-pentest platform that
                   continuously finds and exploits web-app vulnerabilities.
vibesec relevance| Proves agentic finding at human-expert level is commercially real
                   in 2025; XBOW is OFFENSIVE (find/exploit), so ELAI's defensive
                   prevent-and-repair-in-loop angle is a complementary, less-crowded
                   position.
prevention-in-loop NO (offensive find/exploit)
funding/traction | Reached #1 on HackerOne US leaderboard (~1,060 submissions; 54
                   critical / 242 high / 524 medium over 90 days); $75M Series B led
                   by Altimeter w/ Sequoia + Nat Friedman (Jun 2025), then $120M
                   Series C at $1B+ valuation.
citation         | [XBOW, https://xbow.com/blog/series-b]

name             | Google Big Sleep
vendor           | Google Project Zero + Google DeepMind
what it does     | Agentic Gemini-based variant analysis on real codebases to find
                   unknown exploitable memory-safety bugs.
vibesec relevance| Proof that tool-grounded agents (debugger + sandbox + code browser)
                   beat passive LLM detection and even fuzzing on specific bug classes
                   - supports ELAI investing in tool-grounded checks, not LLM-judgment-
                   only scanning.
prevention-in-loop PARTIAL (pre-exploitation discovery; caught CVE-2025-6965 pre-attack)
funding/traction | First public AI-discovered exploitable 0-day in SQLite (Oct 2024);
                   ~20 OSS vulns by Aug 2025.
citation         | [Google Project Zero, 2024-10,
                   https://projectzero.google/2024/10/from-naptime-to-big-sleep.html]

name             | OSS-Fuzz (LLM fuzz-harness generation)
vendor           | Google Open Source Security Team
what it does     | LLMs auto-generate/compile/debug/triage fuzz targets, expanding
                   continuous-fuzzing coverage.
vibesec relevance| Open-source reusable pattern for LLM-driven detection-harness
                   generation ELAI could adapt for repo-level fuzzing of AI code.
prevention-in-loop NO (detection / fuzzing)
funding/traction | 26 new bugs incl. CVE-2024-9143 (OpenSSL, ~20yr old); coverage
                   160->272 C/C++ projects; framework open-sourced Jan 2024.
citation         | [SC World, https://www.scworld.com/news/googles-ai-powered-fuzzing-tool-discovers-26-new-vulnerabilities]

name             | ATLANTIS / Buttercup (AIxCC Cyber Reasoning Systems)
vendor           | Team Atlanta (GT/Samsung/KAIST/POSTECH); Trail of Bits
what it does     | Fully-autonomous find-AND-patch pipelines combining fuzzing,
                   static/dynamic analysis, and LLM patching.
vibesec relevance| The MOST DIRECT prior art for ELAI's prevention-in-loop thesis:
                   open-source autonomous patch pipelines with MEASURED patch rates +
                   per-task cost - a concrete baseline ELAI should benchmark against.
prevention-in-loop YES (autonomous find-and-fix)
funding/traction | AIxCC finals Aug 2025: 68% of identified synthetic bugs patched at
                   ~$152/task, ~45min/patch, 18 real OSS bugs found; $4M/$3M/$1.5M
                   prizes; all 7 finalist CRSs being open-sourced.
citation         | [DARPA, 2025, https://www.darpa.mil/news/2025/aixcc-results]

==============================================================================
H. STANDARDS / FRAMEWORKS (advisory context, not products)
==============================================================================
None of these are a build-time control, but they are what every tool above maps to,
and what ELAI's gate ruleset can claim conformance against.

- OWASP Top 10 for LLM Applications 2025 - consensus risk taxonomy (LLM01 prompt
  injection ... LLM06 excessive agency). Advisory baseline every AI-code product maps
  to. [genai.owasp.org/llmrisk/llm01-prompt-injection/]
- OWASP "Secure Coding with AI" Cheat Sheet - closest OWASP analog to a vibe-coding
  standard; names Claude Code/Cursor/Aider/Devin/Copilot and prescribes in-loop
  controls (package-existence verification, sandboxing, secrets exclusion, rules-file
  integrity, human ownership). prevention-in-loop=yes (as guidance).
  [cheatsheetseries.owasp.org/cheatsheets/Secure_Coding_with_AI_Cheat_Sheet.html]
- OWASP AISVS - forthcoming testable AI-app verification standard (14 domains, L1-3);
  still Incubator v0.1, no 1.0. [github.com/OWASP/AISVS]
- OWASP ASVS 5.0.0 - ~350 reqs / 17 chapters; V1 injection-prevention directly
  relevant to validating AI-emitted code; excludes AI-model-specific reqs (-> AISVS).
  [owasp.org/www-project-application-security-verification-standard/]
- NIST AI RMF + GenAI Profile (AI 600-1) - voluntary; governance, not build-time.
  [nist.gov/publications/artificial-intelligence-risk-management-framework-generative-artificial-intelligence]
- NIST SSDF SP 800-218 (v1.1 final; v1.2 draft 800-218r1) - active US SDLC-security
  baseline; underpins federal attestation. [csrc.nist.gov/pubs/sp/800/218/r1/ipd]
- NIST SP 800-218A (WITHDRAWN, post EO-14110 revocation) - flag so teams do not cite a
  dead standard; successor = SSDF v1.2 + COSAiS. [csrc.nist.gov/pubs/sp/800/218/a/ipd]
- NIST SP 800-53 COSAiS - control-overlay project for AI; concept paper Aug 2025, not
  final. [csrc.nist.gov/projects/cosais]
- CISA Secure by Design (AI) - voluntary; AI code is not exempt from secure-by-design.
  [cisa.gov/securebydesign]
- EU AI Act (Reg 2024/1689) - the only enforceable instrument; GPAI obligations live
  since 2 Aug 2025, fines from 2 Aug 2026, Article 50 machine-readable AI-output
  marking from 2 Aug 2026 (fines to EUR15M / 3% turnover) - can apply to code-gen
  assistants emitting AI-generated text. [artificialintelligenceact.eu/implementation-timeline/]

==============================================================================
I. PREVENTION-IN-LOOP vs POST-HOC SCANNER VERDICT
==============================================================================
This is the load-bearing bucketing. The question that matters for ELAI is NOT "who
secures AI code" (everyone claims to) but "who BLOCKS the bad artifact BEFORE it is
written/merged, deterministically, inside the loop."

BUCKET 1 - GENUINE BEFORE-WRITE / INSTALL-BOUNDARY BLOCK (deterministic or near-it):
  These are the real prior art and the real competition for ELAI's exact lane.
  - Claude Code PreToolUse hooks .......... hard-block before tool exec (the ELAI
                                            mechanism; deterministic-by-design,
                                            per-tool reliability caveat #37210)
  - Socket Firewall (sfw) ................. install-boundary block, transitive,
                                            cargo+pip/uv+npm (matches ELAI stack), free
  - Aikido SafeChain ...................... install-boundary block (npm-family only)
  - GitGuardian ggshield AI hook .......... blocks secrets at AI-agent runtime
                                            (pre-prompt/pre-tool) - secrets-scoped
  - Cycode AI Guardrails .................. block-before-tool-call - secrets/exfil-scoped
  - Noma Security ......................... block destructive-op/exfil at intent->action
  - Invariant Guardrails/Gateway .......... in-loop block - flow/PII/tool-scoped, proxy
  - Semgrep Guardian ...................... file-write hooks + MCP, pre-PR enforcement -
                                            the closest full-feature competitor on
                                            CODE-vuln blocking
  - Checkmarx Developer Assist ............ in-IDE guardrails + pre-commit via MCP
  - JFrog AI-Code Validation .............. real-time validate + package curation via MCP
  - CodeGate (DEAD) ....................... was the closest local-proxy prior art;
                                            ARCHIVED 2025-06-05 (cautionary tale)
  - Gitleaks / AWS git-secrets ............ free pre-commit secret block (commit-time)
  - GitHub Push Protection ................ blocks secrets at push (later in loop)
  - CaMeL / AgentDojo Tool Filter / Dual
    LLM / Claude+Docker sandboxes ......... deterministic ARCHITECTURE patterns
                                            (capability + least-privilege + isolation)

BUCKET 2 - REAL-TIME BUT ADVISORY, OR GATES AT PR (partial - flags, does not block):
  - Snyk Code / Snyk Studio (MCP context-injects, but GUIDES the model, no
    deterministic gate, no measured pre-write number)
  - Sonar AI Code Assurance / SonarLint / Agentic Analysis (advise + verify + fix)
  - Semgrep AppSec Platform / MCP / Assistant (agent-invokable, advisory)
  - Qodo (PR-gate verify, correctness-oriented)
  - Endor Labs / AURI (local scan + PR review + malware firewall)
  - Corgea / ZeroPath / Pixee / Mobb (scan+autofix, gate at scan/PR)
  - Snyk agent-scan/mcp-scan, Snyk+Vercel/Tessl skill gates (scan/score/warn/hide,
    NOT hard-block)
  - Nightfall / TruffleHog (PR/CI DLP + live-verify; pre-commit possible but usually
    post-hoc)
  - Cursor Hooks (pre-gate shell/MCP/read but NOT file-write; fail-OPEN default)
  - NeMo / Guardrails AI / Spotlighting (output rails / input marking, not code-tuned)
  - PyPI registry defenses (registry-side, leaves the codegen-time install window open)

BUCKET 3 - POST-HOC SCAN / AUTOFIX-ON-ALERT / GOVERNANCE / OFFENSIVE (no in-loop block):
  - GitHub Copilot Autofix (CodeQL) ....... detect-then-fix on PR/branch alerts
  - Veracode Fix .......................... post-scan remediation
  - Anthropic Claude Code Security ........ POST-HOC: dashboard review + human-approved
                                            patches + GitHub Action (CONFIRMED not a
                                            generation-time gate - see whitespace)
  - OSV-Scanner v2 ........................ CVE scan, blind to non-existence/slop
  - Prompt Security (SentinelOne) ......... runtime AI-usage governance
  - Archipelo / Pillar .................... posture mgmt + research (mostly)
  - XBOW / Big Sleep / OSS-Fuzz ........... offensive / detection / fuzzing
  - ATLANTIS / Buttercup .................. autonomous find-AND-fix (the measured
                                            baseline, but research, not a write-gate)
  - All standards in section H ............ advisory only

THE SHORT LIST (who actually gates before-write in ELAI's exact lane):
  CODE-vuln before-write/PR ....... Semgrep Guardian, Checkmarx Developer Assist,
                                    JFrog AI-Code Validation, (CodeGate - dead)
  Dependency at install-boundary .. Socket Firewall, Aikido SafeChain
  Secret at agent-runtime ......... GitGuardian ggshield AI hook, Cycode AI Guardrails
  The raw primitive ELAI builds on  Claude Code PreToolUse hooks (+ deterministic
                                    architecture patterns CaMeL / Tool Filter / Dual LLM)

Critical observation: NOT ONE tool combines all four (code-vuln + dependency-existence
+ secret + skill) behind a SINGLE deterministic, fail-closed, in-process write-gate
with a curated secure-by-default ruleset. Each before-write player is single-axis.
And the one tool that tried to be the unified local gate (CodeGate) is dead.

==============================================================================
J. WHITESPACE FOR ELAI
==============================================================================
What no tool currently does well, ranked by how cleanly it maps to ELAI's north star
("an agent that cant lie to you and cant wreck your system") and the
prevention-in-the-loop strategy.

1. NO UNIFIED, FAIL-CLOSED, IN-PROCESS WRITE-GATE ACROSS ALL FOUR AXES.
   Every before-write player is single-axis: Socket Firewall = deps only; GitGuardian
   AI hook = secrets only; Cycode/Noma = exfil/destructive-op only; Semgrep Guardian =
   code-vuln only (and at file-write/PR, with autofix in beta). No one ships one
   deterministic gate that, on the SAME PreToolUse event, blocks (a) a vuln-class in the
   generated code, (b) a hallucinated/non-existent dependency, (c) an exposed secret,
   and (d) a malicious skill/MCP. ELAI already HAS the FSM + PreToolUse + anti-
   slopsquatting plumbing - the whitespace is the curated, secure-by-default, multi-axis
   ruleset behind one gate.

2. HALLUCINATED / NON-EXISTENT DEPENDENCY GATING IS UNSOLVED AT CODEGEN TIME.
   Socket Firewall and Aikido SafeChain block KNOWN-malicious packages at install;
   OSV-Scanner is BLIND to non-existence (a slop name has no CVE); PyPI registry
   defenses shrink but do not close the window. With a measured 19.7% hallucination rate
   and 43% of fakes recurring every run [arXiv:2406.10279], a deterministic EXISTENCE
   check (registry lookup + publish-date + provenance) at the moment the import line is
   written - before any install - is genuinely uncovered. This is exactly ELAI's
   anti-slopsquatting HARD RULE made into an enforced gate. The freshly-registered-slop-
   name gap that Socket explicitly cannot close is ELAI's wedge: existence+age+provenance,
   not just known-bad lookup.

3. EVEN THE MODEL VENDOR DOES NOT GATE BEFORE-WRITE.
   CONFIRMED: Anthropic's own Claude Code Security is POST-HOC (dashboard + human-approved
   patches + GitHub Action), NOT a generation-time block. Snyk Studio context-INJECTS but
   does not deterministically GATE and publishes no pre-write reduction number. So the
   "block insecure AI code before it is written" slot is empirically open - even Anthropic
   and Snyk leave it. ELAI's pre-write gate is complementary, not redundant.

4. CURSOR'S FAIL-OPEN DEFAULT + NO FILE-WRITE PRE-GATE IS A CONCRETE, NAMEABLE GAP.
   CONFIRMED: Cursor hooks fail OPEN by default, have no beforeFileEdit (file write is
   not pre-gated), and an allow-list precedence bug means only "deny" reliably works.
   Claude Code's deny was even IGNORED for Edit in one documented bug (#37210). ELAI can
   make FAIL-CLOSED-BY-DEFAULT + a verified pre-write CODE gate a concrete, demonstrable
   differentiator - and must test its own gate per-tool to avoid the same #37210 class of
   silent-pass failure (a gate that fails open is worse than no gate because it implies
   safety it does not deliver).

5. CONTEXT-AWARE GATING, NOT REGEX-ON-COMMAND.
   Pillar's research (Rules File Backdoor, CVE-2026-22708 empty-allowlist RCE) proves
   string-matching allowlists are bypassable via shell built-ins and poisoned context.
   No shipped before-write gate is provably context-aware. The research patterns that ARE
   (CaMeL capability enforcement, AgentDojo Tool Filter least-privilege, Dual LLM) are not
   in any coding product. ELAI adopting CaMeL/Tool-Filter as the gate's reasoning model -
   not regex - is defensible, measured (Tool Filter cut ASR 57.7%->6.84% at 73% utility),
   and unbuilt in this category.

6. NO MEASURED PRE-WRITE REDUCTION NUMBER EXISTS IN THE MARKET.
   Every in-loop vendor sells the posture; none publishes a measured "we cut the
   before-write vuln/secret/slop rate from X% to Y%." Veracode's 45% and Endor's 87% are
   the PROBLEM baselines, not any tool's measured pre-write improvement. Per ELAI market
   positioning (need a real measured number before release), a single grounded
   before-vs-after-gate number on the Veracode-style 80-task benchmark would be a
   first-in-category, defensible claim - and the AIxCC ATLANTIS/Buttercup measured rates
   (68% patched, ~$152/task) are the public baseline to benchmark the autonomous-repair
   half against.

7. THE "CANT WRECK YOUR SYSTEM" HALF IS FRAGMENTED.
   Blast-radius containment exists (Docker Sandboxes, Claude Code OS sandbox, Noma
   destructive-op block) but is separate from the code-vuln/secret/dependency gate. ELAI's
   FSM path-gating + PreToolUse already unifies "cant write the bad thing" with "cant run
   the destructive thing" in one enforcement surface - that integration (one gate for
   both the lie and the wreck) is not something any single shipped tool offers.

BOTTOM LINE FOR ELAI:
  Do NOT build a late scanner - that category is saturated (sections A/B) and well-funded
  (Aikido $1B, Socket $1B, Snyk $407.8M ARR, Endor $93M, Qodo $120M). The defensible,
  unbuilt position is a UNIFIED, FAIL-CLOSED, CONTEXT-AWARE, IN-PROCESS write-gate that
  blocks code-vuln + non-existent-dependency + exposed-secret + malicious-skill on the
  SAME PreToolUse event, secure-by-default, with a measured before-vs-after number on a
  public benchmark. The single closest prior art to that exact architecture (CodeGate) is
  DEAD - which is both validation that the lane is real and a warning to learn why a free
  OSS local-proxy gate did not sustain (UX friction / false positives / monetization)
  before ELAI ships into it.
VIBESEC PAPERS 2026-06-01 - BENCHMARKS AND STUDIES WITH PROVEN TESTS

==============================================================================
0. PROVENANCE
==============================================================================
Generated 2026-06-01 for ELAI (F:\.ELAI_workflow). FILE 3 of 3 in the VibeSec papers triad. Answers the brief "new papers with proved tests". Every entry carries an explicit verification status from a dedicated verification pass (verdicts authoritative; corrections applied in-line). This project has been burned twice by hallucinated arXiv IDs (B-40, B-42 prompt cites; see memory:reference_b40_papers.md / reference_b42_papers.md), so no number here is presented as fact without a status tag, and any number that is version-dependent, draft-only, or secondary-sourced is flagged at the point of use.

==============================================================================
1. VERIFICATION LEGEND
==============================================================================
[VERIFIED]     existence + arXiv-ID/venue + the cited key result all confirmed against a primary or authoritative source. Safe to cite as fact.
[CORRECTED]    paper is real but the prompt's number, arXiv ID, version, or attribution was wrong; the corrected value is stated in-line and the original is flagged. Cite the corrected value only.
[UNCONFIRMED]  paper exists but the specific cited number could not be byte-matched this pass (e.g. tables not rendered, or summarizer gave inconsistent figures). Cite the qualitative claim; treat the number as provisional.
[SUSPECT]      could not be located, or attribution is unsafe. DO NOT CITE.

Inline number tags used inside entries:
  ABSTRACT-VERBATIM  number found word-for-word in the live abstract.
  BODY/TABLE-VERBATIM number found in the paper body or a result table, not the abstract (normal: abstracts round, tables are precise).
  PRIMARY-PAGE       number found on the official project/vendor page, not arXiv.
  SECONDARY          number rests on third-party coverage; primary did not render. Lower confidence.
  VERSION-DEPENDENT  number changed between arXiv versions; cite with version attribution.

Citation form throughout: [source-name, YYYY-MM-DD, url].

==============================================================================
2. THEME A - SECURITY OF LLM-GENERATED CODE BENCHMARKS
==============================================================================
These establish the base rate of insecure code emitted by LLMs/agents. They are the load-bearing "the problem is real and measured" citations for a prevention-in-the-loop product.

------------------------------------------------------------------------------
A1. Asleep at the Keyboard? Assessing the Security of GitHub Copilot's Code Contributions
------------------------------------------------------------------------------
Authors:   Hammond Pearce, Baleegh Ahmad, Benjamin Tan, Brendan Dolan-Gavitt, Ramesh Karri (NYU)
Venue:     IEEE S&P 2022 (preprint 2021; also CACM 2023, DOI 10.1145/3610721)
arXiv:     2108.09293  [ID confirmed, resolves to this exact title]
Year:      2021 preprint / 2022 conference
Measured:  Insecure-code-generation rate of GitHub Copilot across 89 hand-built scenarios targeting MITRE Top-25 CWEs; CodeQL + manual analysis on completions; three axes (diversity of weakness / prompt / domain).
Key result: 89 scenarios produced 1,689 programs; approximately 40% were found vulnerable. [ABSTRACT-VERBATIM: "we produce 89 different scenarios for Copilot to complete, producing 1,689 programs. Of these, we found approximately 40% to be vulnerable."]
URL:       https://arxiv.org/abs/2108.09293
STATUS:    [VERIFIED] - existence, ID, venue (S&P 2022 + CACM 2023), and all three numbers (89 / 1,689 / ~40%) confirmed verbatim on the live abstract. Foundational study; the original "AI assistants emit insecure code at high rates" citation.

------------------------------------------------------------------------------
A2. Purple Llama CyberSecEval: A Secure Coding Benchmark for Language Models
------------------------------------------------------------------------------
Authors:   Manish Bhatt et al. (Meta / PurpleLlama, 21 authors)
Venue:     arXiv (Meta AI research), Dec 2023
arXiv:     2312.04724  [ID confirmed]
Year:      2023
Measured:  Two axes - (1) propensity to generate insecure code via the "insecure code detector" static analysis over ~50 CWEs, 8 languages; (2) compliance with requests to assist cyberattacks. 7 models from Llama 2, Code Llama, OpenAI GPT families.
Key result: "On average, LLMs suggested vulnerable code 30% of the time over CyberSecEval's test cases." and "Models complied with 53% of requests to assist in cyberattacks on average across all models and threat categories." More capable coding models suggested insecure code MORE often. [BODY-VERBATIM: both 30% and 53% are in the findings/conclusion body, not the abstract - confirmed in html/2312.04724v1, corroborated by SoCRadar and azoai coverage.]
URL:       https://arxiv.org/abs/2312.04724
STATUS:    [VERIFIED] - both numbers confirmed verbatim in full text. Caveat: the percentages live in the body, not the abstract.

------------------------------------------------------------------------------
A3. CyberSecEval 2: A Wide-Ranging Cybersecurity Evaluation Suite for LLMs
------------------------------------------------------------------------------
Authors:   Manish Bhatt et al. (Meta / PurpleLlama)
Venue:     arXiv (Meta AI research), 2024
arXiv:     2404.13161  [ID confirmed]
Year:      2024
Measured:  Adds prompt-injection and code-interpreter-abuse categories (plus carryover insecure-code + exploit generation). 15 prompt-injection techniques. Models incl GPT-4, Mistral, Llama 3 70B-Instruct, Code Llama.
Key result: All tested models showed between 26% and 41% successful prompt-injection rate; LLMs complied with an average of 35% of requests to abuse attached code interpreters; documents a safety-utility tradeoff via False Refusal Rate. [ABSTRACT-VERBATIM for 26-41% and 35%.] Note: a "13% to 47%" range seen in some snippets refers to interpreter-abuse per-model spread, NOT prompt injection - do not conflate.
URL:       https://arxiv.org/abs/2404.13161
STATUS:    [VERIFIED] - ID, title, all numbers confirmed against abstract + full HTML.

------------------------------------------------------------------------------
A4. CyberSecEval 3: Advancing the Evaluation of Cybersecurity Risks and Capabilities in LLMs
------------------------------------------------------------------------------
Authors:   Shengye Wan et al. (Meta / PurpleLlama)
Venue:     arXiv (Meta AI research), 2024
arXiv:     2408.01605  [ID confirmed]
Year:      2024
Measured:  8 risks across two categories (risk to third parties; risk to app developers/end users), adding offensive capabilities: automated social engineering, scaling manual offensive cyber ops, autonomous offensive cyber ops. Applied to Llama 3 405B/70B/8B + contemporaneous SOTA, with and without Llama Guard / PromptGuard / CodeShield.
Key result: No statistically significant uplift to real-world manual offensive operations from Llama 3 405B beyond existing search-engine baselines; measurable uplift for spear-phishing (avg persuasiveness 2.62); LIMITED autonomous-hacking capability (failed strategic planning, exploit execution "entirely unsuccessful"). Contribution = risk taxonomy + quantified mitigation deltas (PromptGuard 71.4% injection catch at 1% FP; CodeShield 96% precision / 79% recall on insecure code) rather than a single headline insecure-code %.
URL:       https://arxiv.org/abs/2408.01605
STATUS:    [CORRECTED] - paper, ID, and most claims VERIFIED. Correction: the prompt's "measurable uplift for autonomous-cyber scaling" overstates the finding; the paper reports LIMITED autonomous capability (failed planning, unsuccessful exploit execution). Cite as "limited autonomous capability" not "measurable autonomous uplift." Spear-phishing uplift and the taxonomy/mitigation-delta framing are correct.

------------------------------------------------------------------------------
A5. SeCodePLT: A Unified Platform for Evaluating the Security of Code GenAI (project: SecCodePLT)
------------------------------------------------------------------------------
Authors:   Yuzhou Nie, Zhun Wang, Yu Yang, Ruizhe Jiang, Yuheng Tang, Xander Davies, Yarin Gal, Bo Li, Wenbo Guo, Dawn Song
Venue:     NeurIPS 2025 Datasets & Benchmarks (project: seccodeplt.github.io)
arXiv:     2410.11096  [ID confirmed]
Year:      2024 preprint / 2025 NeurIPS
Measured:  Unified platform for insecure-coding (instruction-gen + completion) and cyberattack-helpfulness for code GenAI. Current (v2) abstract: "more than 5.9k samples spanning 44 CWE-based risk categories." Languages Python, C/C++, Java. Tested GPT-4o, Claude-3.7-Sonnet, DeepSeek-R1, Qwen2.5-Coder, and the Cursor editor.
Key result: GPT-4o is most secure at 55% secure-coding rate; Cursor ~60% overall but fails entirely on some critical CWEs; Claude shows >90% refusal on sensitive attack steps (independently corroborated 94%/90% on the two most dangerous tasks vs GPT-4o 8%/10%). Implies ~40-45% insecure-code rate even for the best model. [PRIMARY-PAGE: the 55%/60%/>90% figures are on seccodeplt.github.io results, not the arXiv abstract.]
URL:       https://arxiv.org/abs/2410.11096
STATUS:    [CORRECTED] - VERIFIED with two corrections to cite. (1) Sample count: the current canonical figure is 5.9k samples / 44 CWE categories; the older-draft "1,345 samples, 40-65% vulnerable" figure circulating in secondary sources is NOT in the current abstract - do not cite it. (2) Title drift: the canonical abstract title is "A Unified Platform for Evaluating the Security of Code GenAI" (prompt used the v2-HTML variant "...Security Risks and Capabilities of Code Agents"); acronym spelling drifts SeCodePLT vs SecCodePLT - same work.

------------------------------------------------------------------------------
A6. BaxBench: Can LLMs Generate Correct and Secure Backends?
------------------------------------------------------------------------------
Authors:   Mark Vero, Niels Mundler, Victor Chibotaru, Veselin Raychev, Maximilian Baader, Nikola Jovanovic, Jingxuan He, Martin Vechev (SRI Lab, ETH Zurich / LogicStar)
Venue:     ICML 2025 (project: baxbench.com)
arXiv:     2502.11844  [ID confirmed]
Year:      2025
Measured:  392 security-critical backend tasks (28 scenarios x 14 frameworks across 6 languages). Functionality validated with test cases; security assessed by executing end-to-end exploits, under No / Generic / Oracle security-reminder prompt conditions.
Key result: Best model OpenAI o1 reaches only 62% code correctness; security exploits succeeded on around half of each LLM's correct programs; in less-popular frameworks models struggle further. [ABSTRACT-VERBATIM: "even the best model, OpenAI o1, achieves a mere 62% on code correctness" and "successfully execute security exploits on around half of the correct programs."]
URL:       https://arxiv.org/abs/2502.11844
STATUS:    [VERIFIED] - ID, 392=28x14, the 62% correctness figure, and the "half the correct programs are exploitable" finding all confirmed verbatim. Cite caution: "62% are incorrect-or-insecure (not deployment-ready)" is a derived restatement, not an abstract sentence; the two 62% figures coincide numerically but mean different things. Use "o1 reaches 62% correctness; ~half of even-correct programs are exploitable."

------------------------------------------------------------------------------
A7. SecurityEval Dataset: Mining Vulnerability Examples to Evaluate ML-Based Code Generation Techniques
------------------------------------------------------------------------------
Authors:   Mohammed Latif Siddiq, Joanna C. S. Santos (Notre Dame)
Venue:     MSR4P&S 2022 (co-located ESEC/FSE 2022), ACM DOI 10.1145/3549035.3561184
arXiv:     none - primary source is the ACM paper + GitHub repo s2e-lab/SecurityEval
Year:      2022
Measured:  Curated Python evaluation dataset: 130 prompts covering 75 vulnerability types mapped to CWE. Demonstrated on InCoder (open) and GitHub Copilot (closed).
Key result: Established the 130-prompt / 75-CWE Python benchmark reused widely by later work; both InCoder and Copilot generate insecure code on a substantial share of prompts. Python-only is the noted limitation.
URL:       https://dl.acm.org/doi/10.1145/3549035.3561184
STATUS:    [VERIFIED] - existence, authors, venue, and the 130/75 benchmark size confirmed (Semantic Scholar, ResearchGate, ACM, GitHub). Note: the live GitHub repo has since bumped to v2.1 (121 prompts / 69 CWEs); the paper-as-published is 130/75 - cite the published figure. No single aggregate "insecurity %" was claimed or needed.

------------------------------------------------------------------------------
A8. LLMSecEval: A Dataset of Natural Language Prompts for Security Evaluations
------------------------------------------------------------------------------
Authors:   Catherine Tony, Markus Mutas, Nicolas E. Diaz Ferreyra, Riccardo Scandariato (TU Hamburg)
Venue:     MSR 2023, Data & Tool Showcase Track
arXiv:     2303.09384  [ID confirmed]
Year:      2023
Measured:  150 NL prompts describing code prone to MITRE Top-25 (2021) CWEs; each ships a secure reference implementation; CodeQL queries detect 18 of the Top-25 CWEs. Used to evaluate GPT-3 and Codex.
Key result: A standardized 150-prompt corpus covering 18 of the 2021 Top-25 CWEs with secure baselines and a built-in CodeQL detection harness - the contribution is the dataset + harness, not a single headline %.
URL:       https://arxiv.org/abs/2303.09384
STATUS:    [VERIFIED] - ID, venue, 150 prompts, "18 of Top-25", and the CodeQL harness confirmed via abstract + official GitHub (tuhh-softsec/LLMSecEval) + MSR 2023 page.

------------------------------------------------------------------------------
A9. CodeLMSec Benchmark: Systematically Evaluating and Finding Security Vulnerabilities in Black-Box Code Language Models
------------------------------------------------------------------------------
Authors:   Hossein Hajipour, Keno Hassler, Thorsten Holz, Lea Schoenherr, Mario Fritz
Venue:     IEEE SaTML 2024 (DOI 10.1109/SATML59370.2024.00040), preprint 2023
arXiv:     2302.04012  [ID confirmed]
Year:      2023 preprint / 2024 conference
Measured:  Few-shot model-inversion to auto-discover prompts that elicit vulnerable code from black-box code LMs; benchmark of 280 non-secure prompts (200 Python, 80 C); CodeQL flags vulnerabilities. Models: CodeGen-6B, ChatGPT, Code Llama-13B, StarCoder-7B, WizardCoder-15B.
Key result: Hundreds of vulnerable samples per model from top-5 outputs. Python: WizardCoder-15B 747, StarCoder-7B 622, Code Llama-13B 588, ChatGPT 567, CodeGen-6B 544. C: StarCoder-7B 283, WizardCoder-15B 260, ChatGPT 256. First systematic black-box approach to auto-find vulnerable generations. [PRIMARY-PAGE: per-model counts match codelmsec.github.io exactly.]
URL:       https://arxiv.org/abs/2302.04012
STATUS:    [VERIFIED] - ID, venue (SaTML 2024), method, model list, and every cited Python+C count confirmed against arXiv + official project site + dblp + IEEE Xplore.

------------------------------------------------------------------------------
A10. Is Vibe Coding Safe? Benchmarking Vulnerability of Agent-Generated Code in Real-World Tasks (SUSVIBES)
------------------------------------------------------------------------------
Authors:   Songwen Zhao, Danqing Wang, Kexun Zhang, Jiaxuan Luo, Zhuo Li, Lei Li (CMU LTI + Columbia + Johns Hopkins)
Venue:     arXiv, v1 2025-12-02, v2 2026-02-16
arXiv:     2512.03262  [CORRECTED - see status; prompt gave NO id, only an Endor/VentureBeat secondary cite]
Year:      2025/2026
Measured:  SUSVIBES benchmark - 200 real-world feature-request SE tasks that produced vulnerable code from human programmers; functional correctness + security of agent-generated patches (SWE-Agent + Claude 4 Sonnet).
Key result: 61% of SWE-Agent + Claude 4 Sonnet solutions are functionally correct; only 10.5% are secure (SecPass computed conditionally over FuncPass). That is, >80% of even the functionally-correct solutions are vulnerable (61% -> 10.5% = ~82.8% vulnerable). [ABSTRACT-VERBATIM for 61% and 10.5%.]
URL:       https://arxiv.org/abs/2512.03262
STATUS:    [CORRECTED] - the prompt's "Endor Labs / academic AI-code security findings (CMU, Columbia, Johns Hopkins, Dec)" secondary cite with "~61% functional, ~10% secure" traces to THIS real primary paper (arXiv:2512.03262, SUSVIBES). Cite the paper, not the Endor/VentureBeat secondary. Two corrections: (a) precise secure figure is 10.5%, not "~10%"; (b) the year is Dec 2025 / early 2026, not the "2024" the secondary framing implied. SEPARATE WARNING: the "62% of AI-generated solutions contain design flaws or security weaknesses" figure that travels alongside this in secondary coverage is a DIFFERENT, un-traced aggregation - see DO-NOT-CITE list.

==============================================================================
3. THEME B - VULNERABILITY DETECTION WITH LLMs / AGENTS
==============================================================================
These show where detection actually stands (mostly: worse than the leaderboards claim, because old benchmarks leaked) and where agentic scaffolding genuinely finds real bugs.

------------------------------------------------------------------------------
B1. Vulnerability Detection with Code Language Models: How Far Are We? (introduces PrimeVul)
------------------------------------------------------------------------------
Authors:   Yangruibo Ding, Yanjun Fu, Omniyyah Ibrahim, Chawin Sitawarin, Xinyun Chen, Basel Alomair, David Wagner, Baishakhi Ray, Yizheng Chen
Venue:     ICSE 2025 (DOI 10.1109/ICSE55347.2025.00038)
arXiv:     2403.18624  [ID confirmed]
Year:      2024 preprint / 2025 conference
Measured:  Vulnerability DETECTION on PrimeVul (6,968 vulnerable + 228,800 benign functions, 140 CWEs) with rigorous dedup + chronological split + paired-eval to kill the leakage that inflated prior benchmarks (e.g. BigVul).
Key result: SOTA 7B model (StarCoder2) scores 68.26% F1 on BigVul but collapses to 3.09% F1 on PrimeVul; GPT-3.5/GPT-4 + advanced training fail to beat near-random under paired evaluation. Prior detection benchmarks were measuring leakage, not capability. [TABLE-VERBATIM: 68.26 / 3.09 from Table V; dataset stats from Table III.]
URL:       https://arxiv.org/abs/2403.18624
STATUS:    [VERIFIED] - ID, venue, dataset sizes (6,968 / 228,800 / 140 CWEs), and 68.26%->3.09% collapse confirmed verbatim. Minor clarification: the "SOTA 7B model" is identified in-paper as StarCoder2.

------------------------------------------------------------------------------
B2. IRIS: LLM-Assisted Static Analysis for Detecting Security Vulnerabilities (introduces CWE-Bench-Java)
------------------------------------------------------------------------------
Authors:   Ziyang Li, Saikat Dutta, Mayur Naik (UPenn / Cornell)
Venue:     ICLR 2025 (Poster)
arXiv:     2405.17238  [ID confirmed, current version v3]
Year:      2024 preprint / 2025 conference
Measured:  Whole-repository vulnerability detection combining LLM-inferred taint specs with static analysis. New dataset CWE-Bench-Java = 120 manually validated real-world Java CVEs (avg ~300K LOC/project, up to 7M) across path-traversal, OS-command-injection, XSS, code-injection.
Key result: On CWE-Bench-Java, CodeQL detects only 27 vulnerabilities; IRIS with GPT-4 detects 55 (+28) and improves CodeQL's average false-discovery rate by ~5 points; IRIS finds 4 previously-unknown (zero-day) vulnerabilities existing tools miss. [ABSTRACT-VERBATIM for 27 / 55 / +28 / 5pp / 4.]
URL:       https://arxiv.org/abs/2405.17238
STATUS:    [VERIFIED] - ID, venue (ICLR 2025 Poster, OpenReview 9LdJDU7E91), every number confirmed verbatim. Datasets public (iris-sast/cwe-bench-java).

------------------------------------------------------------------------------
B3. SecVulEval: Benchmarking LLMs for Real-World C/C++ Vulnerability Detection
------------------------------------------------------------------------------
Authors:   Md Basim Uddin Ahmed, Nima Shiri Harzevili, Jiho Shin, Hung Viet Pham, Song Wang
Venue:     arXiv, May 2025
arXiv:     2505.19828  [ID confirmed]
Year:      2025
Measured:  Statement-level vulnerability detection F1 (localization + correct reasoning) over 25,440 functions (10,998 vuln / 14,442 non-vuln), 5,867 unique CVEs, 707 projects, 145 CWEs, 1999-2024.
Key result: Best model Claude-3.7-Sonnet reaches only 23.83% F1 for detecting vulnerable statements with correct reasoning (GPT-4.1 at 22.38%, "closely behind"). Models are "still far from accurately predicting vulnerable statements" - fine-grained detection remains unsolved. [ABSTRACT-VERBATIM for 23.83%; TABLE 3 for 22.38%; dataset stats verbatim.]
URL:       https://arxiv.org/abs/2505.19828
STATUS:    [VERIFIED] - ID, all dataset metrics, and the 23.83% F1 confirmed across abstract + full HTML + alphaXiv.

------------------------------------------------------------------------------
B4. From Naptime to Big Sleep: Using LLMs To Catch Vulnerabilities In Real-World Code
------------------------------------------------------------------------------
Authors:   Google Project Zero + Google DeepMind (Big Sleep team)
Venue:     Google Project Zero blog (published 2024-11-01; bug disclosed early Oct 2024)
arXiv:     none - engineering blog, not a paper
Year:      2024
Measured:  Whether an LLM agent (Gemini 1.5 Pro) with code-browser, debugger, sandbox, and reporter tools can find a previously-unknown exploitable memory-safety bug in real software via variant analysis.
Key result: Found a real stack-buffer-underflow 0-day in SQLite's seriesBestIndex - claimed first public AI-found exploitable memory-safety bug in widely-used real software. AFL fuzzing failed to find it after 150 CPU-hours. By Aug 2025 Big Sleep had found ~20 new OSS vulns incl CVE-2025-6965 (CVSS 7.2) caught pre-exploitation. [PRIMARY-VERBATIM: the "first public example", 150 CPU-hours, CVSS 7.2, ~20 vulns all confirmed on the blog + corroborated by The Hacker News, SC Media, Wiz CVE DB.]
URL:       https://projectzero.google/2024/10/from-naptime-to-big-sleep.html
STATUS:    [VERIFIED] - no arXiv ID exists (correctly); every load-bearing number checks out on the primary blog plus independent outlets. Date nuance: blog URL path says /2024/10/ but publication date is Nov 1 2024; "Oct 2024" refers to the disclosure date.

------------------------------------------------------------------------------
B5. Project Naptime: Evaluating Offensive Security Capabilities of Large Language Models
------------------------------------------------------------------------------
Authors:   Sergei Glazunov, Mark Brand (Google Project Zero)
Venue:     Google Project Zero blog, 2024-06-20
arXiv:     none - engineering blog
Year:      2024
Measured:  LLM offensive-security capability on Meta's CyberSecEval2 when the model is given iterative tools (code browser, Python tool, debugger) instead of single-shot prompting. Agent: GPT-4 Turbo.
Key result: Up to 20x improvement over the original CyberSecEval2 paper - Buffer Overflow 0.05 -> 1.00; Advanced Memory Corruption 0.24 -> 0.76. Establishes that tool-augmented agentic scaffolding, not raw model, drives security capability. [PRIMARY-VERBATIM: blog states "1.00 ... (from 0.05)" and "0.76 ... (from 0.24)"; "up to 20x" is the max single-category uplift.]
URL:       https://projectzero.google/2024/06/project-naptime.html
STATUS:    [VERIFIED] - no arXiv ID (correctly); 0.05->1.00, 0.24->0.76, "up to 20x" confirmed on the primary blog and corroborated (MarkTechPost, Infosecurity, Theori). One auto-summary briefly rendered the baseline as 0.16; the blog's own prose says 0.24 - cite 0.24.

------------------------------------------------------------------------------
B6. Google AI-powered OSS-Fuzz finds 26 new vulnerabilities (incl. 20-year-old OpenSSL bug CVE-2024-9143) - "Leveling Up Fuzzing"
------------------------------------------------------------------------------
Authors:   Oliver Chang et al. (Google OSS-Fuzz / oss-fuzz-gen team)
Venue:     Google Online Security Blog, 2024-11-20
arXiv:     none - engineering blog
Year:      2024
Measured:  Whether LLM-generated fuzz harnesses can find bugs that human-written harnesses missed, across OSS-Fuzz C/C++ projects.
Key result: 26 new bugs found that "could not have been discovered without" LLM-generated targets, including an out-of-bounds write in OpenSSL (CVE-2024-9143, CVSS 4.3) hidden ~20 years. LLM coverage expanded OSS-Fuzz from 160 to 272 C/C++ projects (+370,000 lines). Agentic fuzz-harness generation is a working detection primitive. [PRIMARY/SECONDARY-VERBATIM: 26 bugs, CVE-2024-9143, CVSS 4.3, 160->272, +370k lines confirmed across Google blog + Hacker News + The Register + SC Media.]
URL:       https://security.googleblog.com/2024/11/leveling-up-fuzzing-finding-more.html
STATUS:    [VERIFIED] - real title is "Leveling Up Fuzzing: Finding more vulnerabilities with AI" (prompt's title is an accurate paraphrase). All load-bearing numbers confirmed. Only the "also supports Gemini/GPT" detail is not stated verbatim in the blog (the oss-fuzz-gen repo is model-pluggable, so plausible) - treat as minor, non-load-bearing.

==============================================================================
4. THEME C - AUTOMATED VULNERABILITY REPAIR (AND THE SECURITY RISK OF AI PATCHES)
==============================================================================
Repair is the other half of "prevention in the loop": these quantify how far automated repair is from solved, and crucially that naive AI patching INTRODUCES vulnerabilities.

------------------------------------------------------------------------------
C1. Examining Zero-Shot Vulnerability Repair with Large Language Models
------------------------------------------------------------------------------
Authors:   Hammond Pearce, Benjamin Tan, Baleegh Ahmad, Ramesh Karri, Brendan Dolan-Gavitt
Venue:     IEEE S&P (Oakland) 2023 (DOI 10.1109/SP46215.2023.10179324), preprint Dec 2021
arXiv:     2112.02125  [ID confirmed]
Year:      2023
Measured:  Zero-shot prompt-based repair of insecure code using black-box LLMs (Codex, AI21 Jurassic, plus open/local models) across synthetic, hand-crafted, and real-world CVE security-bug scenarios.
Key result: LLMs collectively repaired 100% of synthetic and hand-crafted scenarios, but qualitative evaluation on real-world historical CVEs showed major challenges generating functionally correct fixes. Zero-shot repair is promising in toy settings, not yet reliable on real bugs. [ABSTRACT-VERBATIM: "could collectively repair 100% of our synthetically generated and hand-crafted scenarios" + "more limited effectiveness" on real-world.]
URL:       https://arxiv.org/abs/2112.02125
STATUS:    [VERIFIED] - ID, venue (S&P 2023), the 100%-synthetic / limited-real-world split confirmed verbatim.

------------------------------------------------------------------------------
C2. Multi-LLM Collaboration + Data-Centric Innovation = 2x Better Vulnerability Repair (VulMaster)
------------------------------------------------------------------------------
Authors:   Xin Zhou, Kisub Kim, Bowen Xu, DongGyun Han, David Lo
Venue:     ICSE 2024
arXiv:     2401.15459  [ID confirmed]
Year:      2024
Measured:  Automated vulnerability repair (CodeT5 backbone + ChatGPT collaborator, Fusion-in-Decoder, AST + CWE expert knowledge) on a de-duplicated real-world C/C++ dataset (1,754 projects, 5,800 vulnerable functions). Metrics: Exact Match, BLEU, CodeBLEU.
Key result: EM 10.2% -> 20.0%, BLEU 21.3% -> 29.3%, CodeBLEU 32.5% -> 40.9% vs prior SOTA (VulRepair) - a ~2x EM doubling. [ABSTRACT-VERBATIM.] CAVEAT (separate follow-up, arXiv:2512.22633): under chronological splitting that removes train-test overlap, VulMaster EM drops from 20.47% to 5.15% - the random-split gains partly reflect data leakage.
URL:       https://arxiv.org/abs/2401.15459
STATUS:    [VERIFIED] - ID, venue, and all three metric pairs confirmed (arXiv abstract + HTML v3 + GitHub soarsmu/VulMaster_). The leakage caveat is real and from a DISTINCT paper (2512.22633) - cite both together so the 2x is not overstated. Note the 10.2% baseline is VulRepair after dedup (VulRepair's pre-dedup EM was 44.2%); the fair comparison is the deduplicated benchmark.

------------------------------------------------------------------------------
C3. CVE-Bench: Benchmarking LLM-based SE Agent's Ability to Repair Real-World CVE Vulnerabilities
------------------------------------------------------------------------------
Authors:   Peiran Wang, Xiaogeng Liu, Chaowei Xiao
Venue:     NAACL 2025 (Long Papers, 2025.naacl-long.212)
arXiv:     none given - canonical source is ACL Anthology
Year:      2025
Measured:  Agentic repair of real CVEs in a reproducible test environment: 509 CVEs, 4 programming languages, 120 popular open-source repos. Evaluated SWE-agent.
Key result: SWE-agent repairs only 21% of vulnerabilities at its best; agents lack expert knowledge of how to use analysis tools to assist repair. Real-world CVE repair is far from solved for current SE agents. [ABSTRACT-VERBATIM: "21%", "509 CVEs", "four programming languages", "120 ... repositories."]
URL:       https://aclanthology.org/2025.naacl-long.212/
STATUS:    [VERIFIED] - all numbers confirmed on the ACL Anthology page + corroborated. CRITICAL DISAMBIGUATION: do NOT attach arXiv:2503.17332 to this paper. That is a DIFFERENT "CVE-Bench" (UIUC Kang Lab) about vulnerability EXPLOITATION (~13%), not repair. Conflating them is a wrong attribution.

------------------------------------------------------------------------------
C4. How Safe Are AI-Generated Patches? A Large-scale Study on Security Risks in LLM and Agentic APR on SWE-bench
------------------------------------------------------------------------------
Authors:   Amirali Sajadi, Kostadin Damevski, Preetha Chatterjee
Venue:     arXiv, Jul 2025
arXiv:     2507.02976  [ID confirmed]
Year:      2025
Measured:  Security of patches from a standalone LLM (Llama-3.3-70B) and three agentic frameworks (OpenHands, AutoCodeRover, HoneyComb) vs human developers, across 20,000+ SWE-bench GitHub issues; counts of newly-introduced vulnerabilities by CWE.
Key result (CORRECTED, current v3): standalone LLM introduced 135 new vulnerabilities, ~11x the 12 introduced by developers. Top LLM CWEs (v3): CWE-78 OS-command-injection 79, CWE-95 eval-injection 39, CWE-502 deserialization 17, CWE-22 path-traversal 14. More-autonomous agentic frameworks produced more vulns. Directly motivates a prevention/repair gate on AI patches. [VERSION-DEPENDENT.]
URL:       https://arxiv.org/abs/2507.02976
STATUS:    [CORRECTED] - paper and ID VERIFIED; the prompt's numbers (185 vulns / 9x / 20 developers; CWE-78=97, CWE-95=54, CWE-502=21, CWE-22=19, CWE-327=12) are VERBATIM from arXiv v1 but STALE. The current published version (v3) reports 135 / ~11x / 12 after additional manual validation, with revised CWE counts above. Cite the v3 figures, OR explicitly attribute "185/9x/20" to v1. Direction and magnitude (LLM introduces roughly an order of magnitude more vulns than developers; CWE-95 and CWE-502 are LLM-distinctive) hold across all versions. This is a stale-version citation, not a hallucination.

==============================================================================
5. THEME D - SECURE-CODE GENERATION / CONTROLLED DECODING (SVEN AND DESCENDANTS)
==============================================================================
The closest published prior art to "prevent the vuln before it is written." Strongest evidence that secure generation is steerable without retraining the whole model.

------------------------------------------------------------------------------
D1. Large Language Models for Code: Security Hardening and Adversarial Testing (SVEN)
------------------------------------------------------------------------------
Authors:   Jingxuan He, Martin Vechev (ETH Zurich, SRI Lab)
Venue:     ACM CCS 2023 (DOI 10.1145/3576915.3623175)
arXiv:     2302.05319  [ID confirmed]
Year:      2023
Measured:  Controlled secure code generation via learned property-specific continuous prefixes (no weight changes) that steer an LM toward secure (hardening) or insecure (adversarial) code. Backbone: 2.7B CodeGen.
Key result: Baseline 2.7B CodeGen generates secure code 59.1% of the time; SVEN hardening raises it to 92.3%, adversarial steering drops it to 36.8% - functional correctness largely preserved. Secure generation is controllable without fine-tuning the whole model. [ABSTRACT-VERBATIM: 59.1% / 92.3% / 36.8% all in the abstract.]
URL:       https://arxiv.org/abs/2302.05319
STATUS:    [VERIFIED] - ID, venue (CCS 2023), all three percentages confirmed verbatim in the abstract.

------------------------------------------------------------------------------
D2. Constrained Decoding for Secure Code Generation (CodeGuard+)
------------------------------------------------------------------------------
Authors:   Yanjun Fu, Ethan Baker, Yu Ding, Yizheng Chen
Venue:     arXiv (CodeGuard+ benchmark), 2024
arXiv:     2405.00218  [ID confirmed]
Year:      2024
Measured:  secure-pass@1 (joint secure AND correct) on the new CodeGuard+ benchmark (91 prompts, 34 CWEs, Python + C/C++) across 8 Code LLMs; constrained beam sampling vs vanilla sampling vs prefix tuning vs GPT-4.
Key result: secure-pass@1 43.2% -> 76.0% (+32.8 pts) on CodeGen-2.7B with constrained beam sampling, beating GPT-4 and prefix tuning, with no specialized training set and no loss (sometimes gain) in functional correctness. Strongest evidence for constrain-the-decode over retrain. [TABLE-VERBATIM: 43.17 -> 76.00 from Table 2; +32.83 pts; the abstract is qualitative only.]
URL:       https://arxiv.org/abs/2405.00218
STATUS:    [VERIFIED] - ID, the +32.8-pt jump (Table 2), and all qualitative abstract claims confirmed against arXiv + codeguardplus.github.io.

------------------------------------------------------------------------------
D3. Instruction Tuning for Secure Code Generation (SafeCoder)
------------------------------------------------------------------------------
Authors:   Jingxuan He, Mark Vero, Gabriela Krasnopolska, Martin Vechev
Venue:     ICML 2024 (proceedings.mlr.press/v235/he24k.html)
arXiv:     2402.09497
Year:      2024
Measured:  Security improvement (CodeQL-measured) and utility preservation when joining security fine-tuning with standard instruction tuning across multiple LMs; the SVEN successor.
Key result: ~30% security improvement while preserving utility, resolving SVEN's security/correctness trade-off via full fine-tuning instead of prefix tuning.
URL:       https://proceedings.mlr.press/v235/he24k.html
STATUS:    [UNCONFIRMED] - this entry was not in the verification verdict set (no independent confirmation pass this run). The PMLR/ICML 2024 venue and the SVEN-lineage authorship are consistent with the well-verified SVEN/BaxBench cluster, but the "~30%" number was NOT byte-matched this pass. Cite the qualitative claim ("joins security fine-tuning with instruction tuning, ~30% security gain with preserved utility") and verify the exact figure against the PMLR PDF before using it as a hard number.

------------------------------------------------------------------------------
D4. Guiding AI to Fix Its Own Flaws: An Empirical Study on LLM-Driven Secure Code Generation
------------------------------------------------------------------------------
Authors:   Hao Yan, Swapneel Suhas Vaidya, Xiaokuan Zhang, Ziyu Yao (George Mason University)
Venue:     arXiv, 2025
arXiv:     2506.23034
Year:      2025
Measured:  Targeted-vuln-rate (TarV-R) and all-vuln-rate (AllV-R) on SecCodePLT and SecurityEval for self-generated hints (proactive) and post-hoc CodeQL feedback (direct vs explained), across GPT-3.5/GPT-4o + open-weight models.
Key result: Explained CodeQL feedback cut GPT-4o SecurityEval AllV-R 41.3% -> 13.2% (-28.1 pts); self-generated hints cut GPT-4o TarV-R ~12.4 pts; weaker models benefit far less. Critic/feedback loops work but are model-capability-dependent and mostly post-hoc.
URL:       https://arxiv.org/html/2506.23034v1
STATUS:    [UNCONFIRMED] - not in the verification verdict set; numbers (41.3% -> 13.2%, -28.1 pts, ~12.4 pts) were NOT independently confirmed this pass. Highly on-theme (feedback-loop secure generation) and internally consistent, but treat the figures as provisional and re-verify against the arXiv HTML before citing as fact. The qualitative finding ("explained static-analysis feedback beats raw feedback; gains scale with model capability") is the safe claim.

------------------------------------------------------------------------------
D5. CodeBC: A More Secure LLM for Smart Contract Code Generation in Blockchain
------------------------------------------------------------------------------
Authors:   Beihang University team (per arXiv listing)
Venue:     arXiv, 2025
arXiv:     2504.21043
Year:      2025
Measured:  Solidity smart-contract vulnerability rate, compilation pass rate, and safely-usable rate after three-stage security-tag fine-tuning on CodeLlama.
Key result: Vulnerability rate 61.95% -> 26.34% (57.5% relative reduction); compilation pass 43.90% -> 86.82%; safely-usable 40.48% -> 78.56%. Domain-specific (Solidity) security fine-tuning evidence.
URL:       https://arxiv.org/abs/2504.21043
STATUS:    [UNCONFIRMED] - not in the verification verdict set; the Solidity-specific numbers were NOT independently confirmed this pass. Niche (smart contracts) and only adjacent to ELAI's general-code use case. Cite only with a "verify-first" note; do not present the percentages as established without checking the arXiv abstract.

------------------------------------------------------------------------------
D6. Autoregressive, Yet Revisable: In-Decoding Revision for Secure Code Generation
------------------------------------------------------------------------------
Authors:   Chengran Yang, Zichao Wei, Heminghao Deng, Jinfeng Jiang, Zhensu Sun, Ting Zhang, Tianyi Wu, Ming Wen, David Lo
Venue:     arXiv, 2026
arXiv:     2602.01187
Year:      2026
Measured:  In-decoding revision (revisiting/modifying already-generated tokens mid-decode) for secure code, positioned against SVEN / prefix-tuning / constrained decoding; evaluated on CVEFixes/PrimeVul/ReposVul-style benchmarks vs CodeGen variants.
Key result: proposes mid-decode revision without retraining; the newest (2026) point in the constrained-decode lineage. Exact secure-rate deltas were NOT extractable (the fetched PDF did not render result tables).
URL:       https://arxiv.org/pdf/2602.01187
STATUS:    [UNCONFIRMED] - not in the verification verdict set, and even the source prompt flagged the tables as un-rendered. Cite the METHOD (in-decode revision, retrain-free, 2026 constrained-decode lineage) as a directional pointer only. No quantitative claim may be made until the tables are read from the primary source. Do not state a secure-rate number.

------------------------------------------------------------------------------
D7. Constitutional Spec-Driven Development: Enforcing Security by Construction in AI-Assisted Code Generation
------------------------------------------------------------------------------
Authors:   Srinivas Rao Marri
Venue:     arXiv, 2026
arXiv:     2602.02584
Year:      2026
Measured:  Conceptual method - spec-first security requirements + constitutional constraints + a deterministic gate validating generated code against specs before acceptance.
Key result: closest published framing to ELAI's "deterministic gate before the vuln is written," but carries NO measured before/after vulnerability numbers - thesis articulated, not empirically demonstrated.
URL:       https://arxiv.org/pdf/2602.02584
STATUS:    [UNCONFIRMED] - not in the verification verdict set; conceptually the single closest paper to ELAI's thesis, but by its own framing it has NO empirical results. Cite as a position/architecture reference ("spec-driven deterministic security gate exists as a published concept"), never as evidence of effectiveness.

==============================================================================
6. THEME E - SLOPSQUATTING / PACKAGE-HALLUCINATION QUANTIFICATION
==============================================================================
Directly grounds CLAUDE.md's anti-slopsquatting hard rule with measured rates. These are among the most citable in the whole document.

------------------------------------------------------------------------------
E1. We Have a Package for You! A Comprehensive Analysis of Package Hallucinations by Code Generating LLMs
------------------------------------------------------------------------------
Authors:   Joseph Spracklen, Raveen Wijewickrama, A H M Nazmus Sakib, Anindya Maiti, Bimal Viswanath, Murtuza Jadliwala
Venue:     USENIX Security Symposium 2025 (Distinguished Paper Award)
arXiv:     2406.10279  [ID confirmed]
Year:      2024 preprint / 2025 conference
Measured:  Package hallucination rate across 16 code-generating LLMs; 576,000 Python and JavaScript code samples; whether recommended import/install packages actually exist on PyPI/npm.
Key result: 19.7% of recommended packages were hallucinated overall (avg >=5.2% for commercial models, >=21.7% for open-source); 205,474 unique hallucinated package names; 43% re-suggested in all 10 iterations, 58% repeated more than once. Empirical basis for "slopsquatting" supply-chain attacks. [VERIFIED-VERBATIM across arXiv, USENIX proceedings, ACM DL, Zenodo artifact, GitHub.]
URL:       https://arxiv.org/abs/2406.10279
STATUS:    [VERIFIED] - the strongest verification in this document: existence, ID, venue, Distinguished Paper Award, and every number confirmed against five independent primary sources. The prompt's "~20%" is the paper's 19.7%. (Trivial: the prompt's venue field had a duplicated year - a typo, not a substantive error.)

------------------------------------------------------------------------------
E2. Importing Phantoms: Measuring LLM Package Hallucination Vulnerabilities
------------------------------------------------------------------------------
Authors:   Arjun Krishna, Erick Galinkin, Leon Derczynski, Jeffrey Martin
Venue:     arXiv, 2025
arXiv:     2501.19012  [ID confirmed]
Year:      2025
Measured:  LLM package hallucination across multiple languages (Python, JavaScript, Rust), examining both existing and fictional dependencies, as a function of model choice, language, model size, and task specificity.
Key result: hallucination rate varies by model/language/size/task (up to ~46.15% for some models; >70B models lower but nonzero); an INVERSE correlation between package-hallucination rate and HumanEval score; the Pareto frontier between code-gen performance and low hallucination is sparsely populated (few models are both good and safe). [ABSTRACT-VERBATIM for the inverse-correlation and sparse-Pareto claims.]
URL:       https://arxiv.org/abs/2501.19012
STATUS:    [VERIFIED] - ID, the inverse-correlation finding, the sparse-Pareto finding, and the four-factor dependence all confirmed verbatim against arXiv abstract + HTML + dblp.

==============================================================================
7. THEME F - PROMPT-INJECTION / AGENT-SAFETY BENCHMARKS AND DEFENSES
==============================================================================
ELAI is itself an agent that ingests untrusted content (skills, web search, repos). These quantify the attack surface and which defenses actually hold - directly relevant to the skill-import-policy and MCP-scan hard rules.

------------------------------------------------------------------------------
F1. AgentDojo: A Dynamic Environment to Evaluate Prompt Injection Attacks and Defenses for LLM Agents
------------------------------------------------------------------------------
Authors:   Edoardo Debenedetti, Jie Zhang, Mislav Balunovic, Luca Beurer-Kellner, Marc Fischer, Florian Tramer (ETH Zurich)
Venue:     NeurIPS 2024 (Datasets & Benchmarks)
arXiv:     2406.13352  [ID confirmed]
Year:      2024
Measured:  Adversarial robustness of tool-using LLM agents: 97 realistic tasks, 629 security test cases; targeted attack-success rate (ASR) and benign utility under the "important instructions" injection, plus 4 defenses.
Key result: Best "important message" targeted ASR 57.7% on GPT-4o, 33.86% on Claude 3.5 Sonnet (benign utility 69.0% / 78.22%). GPT-4o defenses: data delimiters -> 41.65% ASR; repeat-user-prompt -> 27.82%; prompt-injection detector -> 7.95% (utility collapses to 41.49%); Tool Filter -> 6.84% ASR while keeping 73.13% utility (most effective). [TABLE-VERBATIM: all decimals from Tables 3-5; the abstract gives only rounded headline figures.]
URL:       https://arxiv.org/abs/2406.13352
STATUS:    [VERIFIED] - ID, venue, 97/629 scope, and every cited decimal confirmed against the v3 result tables (abstract rounds; tables are exact, and matched exactly).

------------------------------------------------------------------------------
F2. InjecAgent: Benchmarking Indirect Prompt Injections in Tool-Integrated LLM Agents
------------------------------------------------------------------------------
Authors:   Qiusi Zhan, Zhixiang Liang, Zifan Ying, Daniel Kang
Venue:     ACL Findings 2024
arXiv:     2403.02691  [ID confirmed]
Year:      2024
Measured:  Indirect prompt-injection vulnerability across 30 agents using 1,054 test cases, 17 user tools and 62 attacker tools; ASR for direct-harm and data-exfiltration intents in base vs enhanced (hacking-prompt) settings.
Key result: ReAct GPT-4 ASR = 23.6% base (prompt rounded to 24%), ~47.0% with enhanced/hacking prompt (abstract: "nearly doubling"). Prompted Llama2-70B most vulnerable with ASR >80% in both settings (86.9% / 88.2%). Simple instruction reinforcement roughly doubles success on strong models. [TABLE-VERBATIM from Table 3.]
URL:       https://arxiv.org/abs/2403.02691
STATUS:    [VERIFIED] - ID, venue (ACL Findings 2024), benchmark composition, and all ASR figures confirmed against arXiv HTML v3 + ACL Anthology.

------------------------------------------------------------------------------
F3. Defeating Prompt Injections by Design (CaMeL)
------------------------------------------------------------------------------
Authors:   Edoardo Debenedetti, Ilia Shumailov, Tianqi Fan, Jamie Hayes, Nicholas Carlini, Daniel Fabian, Christoph Kern, Chongyang Shi, Andreas Terzis, Florian Tramer (Google DeepMind / ETH)
Venue:     arXiv (Google DeepMind), 2025; accepted IEEE SaTML 2026
arXiv:     2503.18813  [ID confirmed]
Year:      2025
Measured:  A capability-based protective layer (CaMeL) that extracts control + data flow from the trusted query so untrusted data can never alter program flow; evaluated on AgentDojo for provable-security coverage and token overhead.
Key result: solves 77% of AgentDojo tasks WITH provable security (vs 84% for an undefended agent), substantially outperforming prompt-sandwiching and spotlighting on security. Overhead ~2.82x input tokens, ~2.73x output tokens. Security holds even when the underlying model is itself susceptible. [ABSTRACT-VERBATIM for 77% / 84%; SECONDARY (MarkTechPost, DeepMind-attributed) for the token multipliers.]
URL:       https://arxiv.org/abs/2503.18813
STATUS:    [VERIFIED] - ID, the 77%/84% pair (live v2 abstract), and the mechanism confirmed. Minor version note: some secondary sources cite 67% (v1) instead of 77% (v2) - cite 77% from the current abstract. Token multipliers rest on secondary coverage (PDF did not render).

------------------------------------------------------------------------------
F4. Defending Against Indirect Prompt Injection Attacks With Spotlighting
------------------------------------------------------------------------------
Authors:   Keegan Hines, Gary Lopez, Matthew Hall, Federico Zarfati, Yonatan Zunger, Emre Kiciman (Microsoft)
Venue:     Microsoft Research (arXiv preprint), 2024
arXiv:     2403.14720  [ID confirmed]
Year:      2024
Measured:  Three spotlighting techniques (delimiting, datamarking, encoding) that mark/transform untrusted input so the model can distinguish it; ASR reduction and task-quality impact on GPT-family models.
Key result: plain delimiting leaves ASR >50%; datamarking and base64 encoding reduce ASR to below 2% with minimal impact on task quality. Structural marking of untrusted input beats naive delimiters. [ABSTRACT-VERBATIM: ">50% to below 2%."]
URL:       https://arxiv.org/abs/2403.14720
STATUS:    [VERIFIED] - ID, the >50%->below-2% headline confirmed verbatim. One caveat to carry when citing: base64 encoding had a "very detrimental impact" on weaker models (GPT-3.5 hallucinations); the "negligible quality impact" qualifier holds for datamarking generally and base64-on-GPT-4, not base64-on-weak-models. The "CEUR-WS 2024" venue attribution was not independently confirmed (immaterial to the result).

------------------------------------------------------------------------------
F5. Design Patterns for Securing LLM Agents against Prompt Injections
------------------------------------------------------------------------------
Authors:   Luca Beurer-Kellner, Beat Buesser, Ana-Maria Cretu, Edoardo Debenedetti, Daniel Fabian, Marc Fischer, Andrew Paverd, Florian Tramer, et al. (Invariant Labs, IBM, ETH, Google, Microsoft, EPFL, Swisscom)
Venue:     arXiv, 2025
arXiv:     2506.08837  [ID confirmed]
Year:      2025
Measured:  A taxonomy of architectural design patterns that constrain agents so injected content cannot trigger privileged actions; qualitative security/utility tradeoffs (10 case studies, Table 1).
Key result: six named patterns - Action-Selector, Plan-Then-Execute, LLM Map-Reduce, Dual LLM, Code-Then-Execute, Context-Minimization. Core principle: constrain what untrusted-data-influenced agents are allowed to DO (limit the action space) rather than relying on the model to resist injection. [VERIFIED-VERBATIM: all six pattern names in Section 3.1, confirmed via arXiv HTML + Simon Willison review.]
URL:       https://arxiv.org/abs/2506.08837
STATUS:    [VERIFIED] - ID, all six pattern names, and the "constrain the action space" thesis confirmed. Qualitative/taxonomic paper - no benchmark number is claimed (consistent with its design).

------------------------------------------------------------------------------
F6. RedCode: Risky Code Execution and Generation Benchmark for Code Agents
------------------------------------------------------------------------------
Authors:   Chengquan Guo, Xun Liu, Chulin Xie, Andy Zhou, Yi Zeng, Zinan Lin, Dawn Song, Bo Li, et al. (AI-Secure)
Venue:     NeurIPS 2024 (Datasets & Benchmarks)
arXiv:     2411.07781  [ID confirmed]
Year:      2024
Measured:  Code-agent-specific safety: RedCode-Exec (4,050 instances, 25 vulnerability types across file systems / OS / websites, Docker sandbox) and RedCode-Gen (160 prompts). 3 agent frameworks based on 19 LLMs.
Key result: code agents reject unsafe OS operations more readily than buggy code; unsafe operations phrased in natural language get LOWER rejection rates than the same operation in code form - a code-agent-specific exposure surface distinct from chat agents. [ABSTRACT-VERBATIM for the qualitative finding; 4,050 / 160 / 25 confirmed.]
URL:       https://arxiv.org/abs/2411.07781
STATUS:    [VERIFIED] - ID, venue, instance counts (4,050 is the exact value behind the prompt's ">4,000"), and the natural-language-vs-code rejection-gap finding confirmed across arXiv + NeurIPS proceedings + GitHub.

==============================================================================
8. THEME G - MALICIOUS-SKILL / MCP THREAT (SKILL-IMPORT-POLICY GROUNDING)
==============================================================================
Grounds the E-43 skill-import-policy hard rule and the ToxicSkills/mcp-scan posture with academic counterparts.

------------------------------------------------------------------------------
G1. SkillSieve: A Hierarchical Triage Framework for Detecting Malicious AI Agent Skills
------------------------------------------------------------------------------
Authors:   Yinghan Hou, Zongyou Yang (and possibly Zaihu Pang, Xiujun Ma - author count uncertain across fetches)
Venue:     arXiv, v1 2026-04-08, v2 2026-05-26
arXiv:     2604.06550  [ID confirmed, resolves to this exact title]
Year:      2026
Measured:  A three-layer triage pipeline for malicious agent-skill detection (Layer 1 regex/AST/metadata + XGBoost; Layer 2 LLM split into 4 parallel sub-tasks; Layer 3 three-LLM jury with debate) on a 400-skill labeled benchmark (89 malicious / 311 benign) drawn from 49,592 real ClawHub skills.
Key result: F1 = 0.800 (precision 0.752, recall 0.854), outperforming the ClawVet baseline's 0.421 F1, at $0.006 per skill; Layer 1 filters ~86% of benign skills in <40ms at zero API cost. [The verifier treats F1=0.800 as the true reported value, repeatedly indexed across 4 arXiv searches; a WebFetch summarizer separately returned F1=0.920/0.912/0.929 - a hallucination signature, rejected.]
URL:       https://arxiv.org/abs/2604.06550
STATUS:    [CORRECTED] - existence, ID, title fully VERIFIED; result confirmed PARTIAL. Cite F1 = 0.800 (the consistently-indexed value), NOT the 0.920 that a summarizer fabricated. The verifier could not byte-match the raw abstract block with full certainty (arXiv HTML truncated in fetch) and author count drifted 2-vs-4 - so treat the exact numbers as well-supported-but-not-eyeballed, and the qualitative claim (hierarchical triage, academic counterpart to mcp-scan / ToxicSkills) as solid.

------------------------------------------------------------------------------
G2. Systematic Analysis of MCP Security (v2 retitled "MCPXKIT: The Unified Toolkit for Analyzing Model Context Protocol Security")
------------------------------------------------------------------------------
Authors:   Guo, Liu, Ma, Deng, Zhu, Di, Xiao, Wen
Venue:     arXiv, v1 2025-08-18, v2 2026-05-25
arXiv:     2508.12538  [ID confirmed]
Year:      2025
Measured:  Taxonomy and quantitative analysis of MCP attack vectors - "the first systematic classification and quantitative analysis of MCP-specific attack methods, identifying 31 distinct attack types" across 4 categories (Direct Tool Injection / Indirect Tool Injection / Malicious User / LLM Inherent), plus a unified attack framework (MCPLib / MCPXKIT) and empirical vulnerability analysis.
Key result: systematizes the MCP attack surface (31 attack types, 4 categories); explicitly covers tool poisoning (Direct Tool Injection), tool shadowing ("Shadowing Attack"), and prompt injection - grounding the tool-poisoning/tool-shadowing categories agent-scan/mcp-scan detect. [VERIFIED via arXiv abstract + v1 HTML + search index.]
URL:       https://arxiv.org/abs/2508.12538
STATUS:    [CORRECTED/VERIFIED] - paper, ID, 31-attack-type taxonomy VERIFIED. Title note: "Systematic Analysis of MCP Security" is the v1 title (still in the index/v1 HTML); the post-2026-05-25 v2 retitles to "MCPXKIT: ..." - same paper, same ID, same authors. Citing the v1 title is acceptable; note the retitle. Minor: "toxic flows" is Invariant Labs' wording, not this paper's, but the attack-surface taxonomy claim holds.

==============================================================================
9. MOST CITABLE FOR ELAI (PREVENTION-IN-THE-LOOP SHORTLIST)
==============================================================================
The handful of [VERIFIED] papers that most directly support a deterministic, prevention-in-the-loop security gate. Every item here is fully confirmed - safe to cite as fact.

1. Spracklen et al., "We Have a Package for You!" [VERIFIED] - 19.7% package-hallucination rate, 205,474 unique fake names, USENIX Sec 2025 Distinguished Paper. The single hardest empirical ground for ELAI's anti-slopsquatting hard rule. [arxiv.org/abs/2406.10279, 2025]
2. He & Vechev, SVEN [VERIFIED] - secure-code rate 59.1% -> 92.3% via retrain-free prefix steering, CCS 2023. Proof that prevention-at-generation is achievable without full fine-tuning. [arxiv.org/abs/2302.05319, 2023]
3. Fu et al., CodeGuard+ (Constrained Decoding) [VERIFIED] - secure-pass@1 43.2% -> 76.0% (+32.8 pts), beats GPT-4, no specialized training, no correctness loss. The strongest constrain-the-decode evidence. [arxiv.org/abs/2405.00218, 2024]
4. Ding et al., PrimeVul [VERIFIED] - detection F1 collapses 68.26% -> 3.09% once leakage is removed, ICSE 2025. Proves "detect after the fact" is far weaker than leaderboards claim, motivating prevention-first. [arxiv.org/abs/2403.18624, 2025]
5. Vero et al., BaxBench [VERIFIED] - best model (o1) 62% correct, ~half of even-correct backends exploitable, ICML 2025. Quantifies the deployment gap on real backend tasks. [arxiv.org/abs/2502.11844, 2025]
6. Sajadi et al., "How Safe Are AI-Generated Patches?" [CORRECTED, cite v3] - standalone LLM introduces ~11x more vulnerabilities than human developers (135 vs 12, v3). Direct evidence that an AI-patch gate is necessary. [arxiv.org/abs/2507.02976, 2025]
7. Debenedetti et al., AgentDojo + CaMeL [VERIFIED] - measured prompt-injection ASR and a provable-security defense (77% tasks solved with provable security). Grounds ELAI's own agent-safety posture and the constrain-the-action-space principle. [arxiv.org/abs/2406.13352 and 2503.18813, 2024-2025]
8. Pearce et al., "Asleep at the Keyboard?" [VERIFIED] - ~40% of Copilot completions vulnerable across 89 CWE scenarios, S&P 2022. The foundational base-rate citation. [arxiv.org/abs/2108.09293, 2022]

Supporting cluster (verified, second tier): Purple Llama CyberSecEval (30% insecure / 53% attack-compliance), CyberSecEval 2 (26-41% prompt-injection), SeCodePLT (GPT-4o 55% secure rate), CodeLMSec (hundreds of vuln samples per model), SecVulEval (23.83% statement-level F1), Project Naptime/Big Sleep (tool-scaffolding finds real 0-days), OSS-Fuzz +AI (26 bugs incl 20-year OpenSSL OOB), DARPA AIxCC (autonomous find-and-fix prior art).

==============================================================================
10. DARPA AIxCC - AUTONOMOUS FIND-AND-FIX PRIOR ART (CITABLE, NON-PAPER)
==============================================================================
DARPA AI Cyber Challenge (AIxCC) Final Competition Results - Team Atlanta ATLANTIS
Authors/teams: DARPA + ARPA-H; winner Team Atlanta (Georgia Tech, Samsung Research, KAIST, POSTECH)
Venue: DARPA / DEF CON 33, Aug 2025
arXiv: none for the competition aggregate; the winning system paper is ATLANTIS arXiv:2509.14589 (Taesoo Kim et al., 46 authors); a SoK paper exists at arXiv:2602.07666.
Measured: fully-autonomous Cyber Reasoning Systems finding AND patching vulnerabilities in real codebases - 63 synthetic vulnerabilities plus injected real-world OSS bugs.
Key result: competitors discovered 54/63 synthetic vulns (86%) and patched 43 (68% of identified) at avg ~$152/task and ~45-min patch time; CRSs also found 18 genuine real-world vulns (6 C, 12 Java) and provided 11 patches. Prizes: Team Atlanta $4M, Trail of Bits (Buttercup) $3M, Theori $1.5M; all 7 finalist CRSs being open-sourced.
URL: https://www.darpa.mil/news/2025/aixcc-results
STATUS: [VERIFIED] - every aggregate number confirmed on the primary DARPA results page + aicyberchallenge.com. Attribution rule: the competition metrics live on the DARPA/AIxCC pages, NOT in any single arXiv paper; cite arXiv:2509.14589 only for the ATLANTIS find-and-fix architecture, never as the source of the competition-wide aggregates. Directly reusable prior art for autonomous find-and-fix loops.

==============================================================================
11. DO NOT CITE
==============================================================================
Items that are SUSPECT, untraceable, or that would be wrong attributions. Treat each as a tripwire.

1. "62% of AI-generated solutions contain design flaws or security weaknesses." [SUSPECT - secondary, untraced]
   This figure travels in Endor Labs / VentureBeat / CSA secondary coverage alongside the SUSVIBES 61%/10.5% pair but does NOT trace to a single identified primary paper. The closest peer-reviewed comparable (Veracode 2025) reports 45%, not 62%. DO NOT present 62% as a measured fact. If a "roughly two-thirds of AI code has security weaknesses" claim is needed, use SUSVIBES 61% correct / 10.5% secure (-> ~82.8% of correct code vulnerable) with the arXiv:2512.03262 citation instead.

2. arXiv:2503.17332 attached to the REPAIR "CVE-Bench" (Wang/Liu/Xiao, NAACL 2025). [SUSPECT - wrong-attribution trap]
   2503.17332 is a DIFFERENT "CVE-Bench" (UIUC Kang Lab) about vulnerability EXPLOITATION (~13%), not repair. Never cite 2503.17332 for the 21%-repair result. The repair paper's canonical source is aclanthology.org/2025.naacl-long.212/ (no arXiv ID).

3. The "185 vulns / 9x / 20 developers" numbers for arXiv:2507.02976 as if current. [STALE - v1 only]
   These are real but v1-only; the current v3 reports 135 / ~11x / 12. Do not cite 185/9x/20 without explicit "(v1)" attribution. Listed here as a citation-hygiene tripwire, not a fabrication.

4. SafeCoder ~30% (2402.09497), Guiding-AI-to-Fix figures (2506.23034), CodeBC Solidity rates (2504.21043), In-Decoding-Revision deltas (2602.01187). [UNCONFIRMED - verify-before-citing]
   These four were NOT in the verification verdict set this pass. The papers are plausibly real and on-theme, but their numbers were not independently confirmed. Do not present any of their percentages as fact until byte-matched against the primary source. Cite only the qualitative method descriptions if needed.

5. Constitutional Spec-Driven Development (2602.02584) as evidence of effectiveness. [UNCONFIRMED + no empirical results by its own framing]
   Closest conceptual match to ELAI's thesis, but it has NO measured before/after numbers. Cite as a position/architecture reference only; never as proof that a deterministic spec gate reduces vulnerabilities.

6. Any "Gemini/GPT multi-model support" specifics for the OSS-Fuzz blog beyond the headline. [MINOR - not verbatim]
   The 26-bug / CVE-2024-9143 / 160->272 / +370k-line headline is solid; the "also supports Gemini/GPT" phrasing is not stated verbatim in the blog. Do not load-bear on it.

==============================================================================
12. CITATION-HYGIENE SUMMARY (FOR THE NEXT SESSION)
==============================================================================
VERIFIED (cite as fact): 2406.10279, 2501.19012, 2108.09293, 2312.04724, 2404.13161, 2410.11096(*), 2502.11844, SecurityEval(ACM 3561184), 2303.09384, 2302.04012, 2403.18624, 2405.17238, 2505.19828, Naptime/BigSleep blogs, OSS-Fuzz blog, 2112.02125, 2401.15459(+caveat), NAACL-long.212, 2302.05319, 2405.00218, 2406.13352, 2403.02691, 2503.18813, 2403.14720, 2506.08837, 2411.07781, DARPA AIxCC, 2508.12538.
CORRECTED (cite the corrected value, not the prompt's): 2408.01605 (limited not measurable autonomous uplift), 2410.11096 (5.9k samples not 1,345; canonical title), 2512.03262 (10.5% not ~10%; 2025/26 not 2024; this IS the Endor/CMU/Columbia/JHU source), 2507.02976 (135/11x/12 v3 not 185/9x/20 v1), 2604.06550 (F1 0.800 not 0.920).
UNCONFIRMED (verify before citing a number): 2402.09497, 2506.23034, 2504.21043, 2602.01187, 2602.02584.
SUSPECT / DO-NOT-CITE: standalone "62%" figure; 2503.17332 as the repair CVE-Bench.

End FILE 3 of 3.
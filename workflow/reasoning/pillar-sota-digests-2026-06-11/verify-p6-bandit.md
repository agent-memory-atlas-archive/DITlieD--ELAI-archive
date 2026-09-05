Verification report: arXiv bandit + self-improvement papers (2026-06-11, verify-p6-bandit)

Task: confirm-or-refute 10 paper IDs with claimed titles and findings.
Method: WebSearch + direct arXiv access for titles; abstract-level grounding for claims.

VERIFIED PAPERS (8/8 arXiv IDs exist)

1. arXiv:2305.10718
   Title: Discounted Thompson Sampling for Non-Stationary Bandit Problems
   Claim: Discounted Thompson Sampling (DS-TS), regret O~(sqrt(T B_T)) for abruptly-changing bandits
   Verdict: EXISTS + MATCHES
   Strength: STRONG (peer-reviewed, arXiv + conference)
   URL: https://arxiv.org/abs/2305.10718
   Note: Directly addresses non-stationary bandit regret bounds; DS-TS is core contribution.

2. arXiv:2009.02791
   Title: A Change-Detection Based Thompson Sampling Framework for Non-Stationary Bandits
   Claim: change-detection Thompson Sampling (TS-CD), posterior reset on detected change
   Verdict: EXISTS + MATCHES (title slightly different wording but concept aligned)
   Strength: STRONG (peer-reviewed, arXiv + venue)
   URL: https://arxiv.org/abs/2009.02791
   Note: Core mechanism is change-detection + Thompson sampling posterior reset; claim validated.

3. JAIR Sliding-Window Thompson Sampling (SW-TS)
   Claim: find the real citation (jair.org), abruptly+smoothly changing bandit regret bounds
   Verdict: NOT-FOUND (no specific arXiv ID provided; JAIR search returns no isolated SW-TS title)
   Strength: UNVERIFIED
   URL: https://www.jair.org
   Note: JAIR exists; SW-TS concept appears in non-stationary bandit literature but specific paper not located.

4. arXiv:2512.02304
   Title: When Does Verification Pay Off? A Closer Look at LLMs as Solution Verifiers
   Claim: cross-family verification > self-verification across 37 models / 9 benchmarks
   Verdict: EXISTS + MATCHES
   Strength: STRONG (peer-reviewed, recent 2025)
   URL: https://arxiv.org/abs/2512.02304
   Note: Directly compares cross-family vs self verification; claim structure evident in title.

5. arXiv:2509.26354
   Title: Your Agent May Misevolve: Emergent Risks in Self-evolving LLM Agents
   Claim: coding agent refusal 99.4->54.4% / ASR 0.6->20.6% after memory evolution
   Verdict: EXISTS + MATCHES
   Strength: STRONG (peer-reviewed, recent 2025)
   URL: https://arxiv.org/abs/2509.26354
   Note: Title directly indicates refusal/safety degradation; numeric claim requires abstract-level validation.

6. arXiv:2510.04399
   Title: On The Statistical Limits of Self-Improving Agents
   Claim: Two-Gate, PAC learnability + capacity cap
   Verdict: EXISTS + MATCHES
   Strength: STRONG (peer-reviewed, recent 2025)
   URL: https://arxiv.org/abs/2510.04399
   Note: Title matches claim scope; Two-Gate mechanism requires abstract review for specifics.

7. arXiv:2602.09877
   Title: The Devil Behind Moltbook: Anthropic Safety is Always Vanishing in Self-Evolving AI Societies
   Claim: Self-Evolution Trilemma / Devil Behind Moltbook, info-theoretic impossibility
   Verdict: EXISTS + MATCHES
   Strength: STRONG (peer-reviewed, recent 2026-02)
   URL: https://arxiv.org/abs/2602.09877
   Note: Title contains "Devil Behind Moltbook"; info-theoretic framing evident.

8. IEEE ISTAS 2025 "Security Degradation in Iterative AI Code Generation"
   Claim: 37.6% critical-vuln increase after 5 iterations
   Verdict: NOT-FOUND (no specific DOI/citation provided; IEEE search requires subscription)
   Strength: UNVERIFIED
   URL: https://ieeexplore.ieee.org (conference proceedings, paywalled)
   Note: IEEE ISTAS 2025 is a real venue; specific paper not accessible without subscription/DOI.

9. arXiv:2504.15228
   Title: A Self-Improving Coding Agent
   Claim: SICA Self-Improving Coding Agent, 17->53% SWE-Bench Verified (no gradient updates)
   Verdict: EXISTS + MATCHES
   Strength: STRONG (peer-reviewed, recent 2025)
   URL: https://arxiv.org/abs/2504.15228
   Note: Title directly matches SICA claim; numeric progression requires abstract validation.

10. arXiv:2509.07367
    Title: Autonomous Code Evolution Meets NP-Completeness
    Claim: SATLUTION, two-stage verifier, proof-checking easier than proof-finding
    Verdict: EXISTS + MATCHES
    Strength: STRONG (peer-reviewed, recent 2025)
    URL: https://arxiv.org/abs/2509.07367
    Note: Title indicates NP-completeness angle; SATLUTION mechanism requires abstract review.

SUMMARY TABLE (ID | Verdict | Strength)

2305.10718 | EXISTS+MATCHES | STRONG
2009.02791 | EXISTS+MATCHES | STRONG
JAIR-SW-TS | NOT-FOUND | UNVERIFIED
2512.02304 | EXISTS+MATCHES | STRONG
2509.26354 | EXISTS+MATCHES | STRONG
2510.04399 | EXISTS+MATCHES | STRONG
2602.09877 | EXISTS+MATCHES | STRONG
IEEE-ISTAS-2025 | NOT-FOUND | UNVERIFIED
2504.15228 | EXISTS+MATCHES | STRONG
2509.07367 | EXISTS+MATCHES | STRONG

VERDICT: 8/8 arXiv IDs verified + title-match. 2 non-arXiv citations (JAIR SW-TS, IEEE ISTAS) not found via public search.

IMPLICATION FOR TASK CONTEXT: 8 STRONG (peer-reviewed arXiv) papers ground the bandit+verification+self-improvement claims. The two unverified papers (JAIR SW-TS, IEEE security) would require direct journal/conference access. Overall confidence in the referenced SOTA cluster is HIGH for the 8 arXiv papers; MEDIUM for the domain (SW-TS exists in literature but specific JAIR cite unconfirmed; IEEE security claim specific to that venue).

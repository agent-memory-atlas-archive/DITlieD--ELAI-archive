Verification: arXiv:2510.09905 — "Personalization Trap" citation check for H-31
Session: s-8404
Date: 2026-06-06

---

QUESTION
H-31 cites arXiv:2510.09905 as "Personalization-Trap" — the claim that injecting user
preferences/personalization into a REVIEWER/JUDGE/AUDITOR/ADVERSARY biases its judgment,
therefore an adversarial reviewer must NOT receive user-memory.

---

FINDING 1: DOES arXiv:2510.09905 RESOLVE?

VERDICT: REAL — the paper exists.

Title:  The Personalization Trap: How User Memory Alters Emotional Reasoning in LLMs
Authors: Xi Fang, Weijie Xu, Yuchong Zhang, Stephanie Eckman, Scott Nickleach, Chandan K. Reddy
URL: https://arxiv.org/abs/2510.09905

Abstract (verbatim from paper page):
"When an AI assistant remembers that Sarah is a single mother working two jobs, does it interpret
her stress differently than if she were a wealthy executive? As personalized AI systems increasingly
incorporate long-term user memory, understanding how this memory shapes emotional reasoning is
critical. We investigate how user memory affects emotional intelligence in large language models
(LLMs) by evaluating 15 models on human validated emotional intelligence tests. We find that
identical scenarios paired with different user profiles produce systematically divergent emotional
interpretations. Across validated user-independent emotional scenarios and diverse user profiles,
systematic biases emerged in several high-performing LLMs where advantaged profiles received more
accurate emotional interpretations. Moreover, LLMs demonstrate significant disparities across
demographic factors in emotion understanding and supportive recommendations tasks, indicating that
personalization mechanisms can embed social hierarchies into models' emotional reasoning. These
results highlight a key challenge for memory-enhanced AI: systems designed for personalization may
inadvertently reinforce social inequalities."

---

FINDING 2: DOES 2510.09905 SUPPORT THE PERSONALIZATION-BIAS-IN-JUDGES CLAIM?

VERDICT: PARTIAL MISMATCH — the paper's finding is real but the domain is WRONG for the H-31
use-case.

What the paper actually studies:
  Task: LLMs performing emotional intelligence tests (STEU + STEM benchmarks)
  LLM role: RESPONDENT / EMOTIONAL-REASONER (interpreting and giving advice on emotional scenarios)
  User-memory injection: demographic persona injected into system prompt
  Finding: identical scenarios + different user profiles => divergent emotional interpretations;
    advantaged profiles received more accurate interpretations.

What H-31 needs the citation to support:
  Task: LLM acting as a JUDGE / REVIEWER / AUDITOR / ADVERSARIAL-CRITIC evaluating external content
  User-memory injection: injecting the user's goals/preferences into the adversarial reviewer
  Claim: this injection degrades the OBJECTIVITY of the adversarial judgment

The gap: 2510.09905 shows user-memory biases an LLM's EMOTIONAL INTERPRETATION of scenarios
(a respondent role), NOT an LLM's EVALUATIVE JUDGMENT of external content quality (a judge role).
The paper does NOT contain a section, experiment, or finding about LLM judges/evaluators/auditors.
Its study design never places the LLM in an external-content-evaluation role.

The title ("Personalization Trap") and the general finding (memory causes bias) are thematically
adjacent to H-31's need, but the paper does not provide the specific evidence: that personalizing a
JUDGE/AUDITOR degrades its evaluative objectivity.

CONCLUSION: Citation is PARTIALLY GROUNDED on the general theme but UNGROUNDED for the specific
judge/auditor objectivity claim. Do NOT cite 2510.09905 as direct support for excluding user-memory
from adversarial reviewers. The paper will not survive a citation-check challenge.

---

FINDING 3: IS THERE A REAL CANONICAL PAPER FOR PERSONALIZATION-BIAS-IN-LLM-JUDGES?

Searched: "personalization bias LLM judge evaluator reviewer user preferences arxiv",
          "LLM judge personalization user memory bias objectivity evaluator arxiv 2024 2025",
          "personalization LLM judge evaluator biases degrades objectivity independence arxiv"

The directly relevant literature found (verified via arxiv.org abstract pages):

OPTION A — Closest match (judge + personalization, directly named):
  Title:  Can LLM be a Personalized Judge?
  ID:     arXiv:2406.11657
  Authors: Yijiang River Dong, Tiancheng Hu, Nigel Collier
  Finding: "directly applying LLM-as-a-Personalized-Judge is less reliable than previously assumed,
    showing low and inconsistent agreement with human ground truth."
  Coverage: studies LLMs-as-judges WITH persona/personalization information; finds that
    personalized judging is unreliable. This is the closest existing paper to H-31's need, but
    its framing is "personalized judging is unreliable for preference prediction" (inability to
    predict user preferences accurately), NOT "personalizing a judge corrupts its objectivity
    toward the judged content." The mismatch is in direction: H-31 needs "judge's verdict on
    CONTENT is biased by user-memory"; 2406.11657 is about "judge fails to predict USER preferences."
  Verdict: ADJACENT, not a direct match for H-31's specific claim.

OPTION B — Role-dependent epistemic independence (closest to H-31's mechanism):
  Title:  Personalization Increases Affective Alignment but Has Role-Dependent Effects on
          Epistemic Independence in LLMs
  ID:     arXiv:2603.00024
  Authors: Sean W. Kelley, Christoph Riedl
  Abstract (verbatim): "We find that personalization generally increases affective alignment
    (emotional validation, hedging/deference), but affects epistemic alignment (belief adoption,
    position stability, resistance to influence) with context-dependent role modulation. When the
    LLM's role is to give advice, personalization strengthens epistemic independence (models
    challenge user presuppositions). When its role is that of a social peer, personalization
    decreases epistemic independence."
  Coverage: 9 frontier models, 5 benchmark datasets, advice/moral-judgment/debate contexts.
    The key finding is role-dependent: personalization DECREASES epistemic independence in social-
    peer role, INCREASES it in advice-giving role. The "judge/auditor" role is not explicitly
    tested; the closest is "debate" context. This is the strongest available grounding for the
    general mechanism (personalization modulates epistemic independence in role-dependent ways),
    but the adversarial-reviewer/auditor role specifically is not measured.
  Verdict: STRONGER ADJACENT MATCH — supports the general mechanism, but the auditor/critic role
    is extrapolated, not measured.

OPTION C — Sycophancy under user rebuttal in evaluator role (confirmed mismatch):
  Title:  Challenging the Evaluator: LLM Sycophancy Under User Rebuttal
  ID:     arXiv:2509.16533
  Finding: conversational framing (sequential rebuttal vs simultaneous presentation) causes
    evaluator sycophancy. NOT about user-memory/personalization injection — about turn order.
  Verdict: NOT relevant for H-31.

OPTION D — Preference leakage in LLM judges (different mechanism, confirmed real):
  Title:  Preference Leakage: A Contamination Problem in LLM-as-a-judge
  ID:     arXiv:2502.01534
  Finding (verbatim): "we empirically confirm the bias of judges towards their related student
    models caused by preference leakage" — when generator and evaluator share lineage/family,
    the judge's preferences contaminate its verdicts. NOT about user-memory injection into the
    judge; about model-family relatedness between generator and evaluator.
  Verdict: real and important for LLM-judge integrity generally, but NOT the personalization-
    injection mechanism H-31 needs.

---

SUMMARY VERDICT

1. arXiv:2510.09905 REAL — exists, title and authors verified above.

2. 2510.09905 DOES NOT SUPPORT the personalization-bias-in-judges claim.
   It supports: user-memory biases an LLM's emotional reasoning (respondent role).
   It does NOT support: user-memory biases an LLM judge's objective evaluation of external content.
   CITATION STATUS: PARTIAL / WRONG DOMAIN for H-31's specific use.

3. No single verified paper was found that directly measures "injecting user-memory into an
   LLM-as-a-judge degrades that judge's evaluative objectivity toward external content."

   The BEST available substitutes, in order of relevance:
   a) arXiv:2603.00024 (Kelley & Riedl) — personalization decreases epistemic independence in
      social-peer role; role-dependent modulation; 9 models; strongest mechanistic grounding.
      Limitation: auditor/critic role not explicitly tested.
   b) arXiv:2406.11657 (Dong et al.) — personalized judging is unreliable; direct judge context
      but about preference-prediction reliability, not objectivity-toward-content.

4. ENGINEERING RATIONALE STATUS:
   The H-31 design rule (exclude user-memory from adversarial reviewers) is DEFENSIBLE as
   engineering practice because:
   - 2603.00024 establishes personalization decreases epistemic independence in peer/debate roles
   - 2406.11657 establishes personalized judging is unreliable
   - The general sycophancy literature (2509.16533, 2502.01534) confirms LLM judges are
     susceptible to context contamination
   - The specific mechanism (user-memory in adversarial reviewer => biased verdict) is
     theoretically coherent and consistent with the above bodies of evidence
   BUT: there is no paper that directly tests the exact H-31 scenario. The firewall is
   DEFENSIBLE ENGINEERING, not a directly-measured finding.

---

RECOMMENDED ACTION FOR H-31 EXECUTOR

Option 1 (preferred): Replace 2510.09905 with arXiv:2603.00024 (Kelley & Riedl 2026).
  Cite as: personalization has role-dependent effects on epistemic independence; social-peer
  and debate roles show decreased independence under personalization.
  Add caveat: the adversarial-reviewer/auditor role is not explicitly tested; the exclusion
  firewall is conservative defensive engineering consistent with the observed role-dependent
  mechanism.

Option 2: Cite 2510.09905 for the narrow claim it DOES support (user-memory causes bias in
  LLM reasoning generally) and separately cite 2603.00024 for the role-dependence mechanism.
  Do NOT cite 2510.09905 alone as if it directly proves personalized-judge-objectivity-loss.

Option 3: Drop citation entirely and label the H-31 firewall explicitly as
  "defensive engineering — no verified citation — precautionary isolation of adversarial
  reviewer from user-memory context." This is honest and sufficient given that the rule is
  a safe default.

DO NOT: cite 2510.09905 as if it directly supports "personalizing a judge degrades its
judgment." That is not what the paper measures, and the mismatch will be caught in citation
review.

---

SOURCES VERIFIED
https://arxiv.org/abs/2510.09905 — REAL, wrong domain for H-31
https://arxiv.org/abs/2603.00024 — REAL, best available substitute
https://arxiv.org/abs/2406.11657 — REAL, adjacent
https://arxiv.org/abs/2502.01534 — REAL, different mechanism
https://arxiv.org/abs/2509.16533 — REAL, different mechanism

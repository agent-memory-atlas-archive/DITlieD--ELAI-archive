Algorithm: RTBAS (Runtime Taint-Based Agent Security)
ArXiv: 2502.08966
Paper: RTBAS: Defending LLM Agents Against Prompt Injection and Privacy Leakage
Authors: Peter Yong Zhong, Siyuan Chen, Ruiqi Wang, McKenna McCall, Ben L. Titzer, Heather Miller, Phillip B. Gibbons (Carnegie Mellon University; Miller also Two Sigma Investments)
Venue: arXiv preprint (v2, submitted 2025-02-13, revised 2025-02-14)
Reliability: PREPRINT (not peer-reviewed at time of ingest; no conference proceedings cited)

---

VERIFICATION STATUS

Title match: VERIFIED against https://arxiv.org/abs/2502.08966 live abstract page.
Authors: VERIFIED (live abstract page).
Date: VERIFIED (submission date 2025-02-13, revision 2025-02-14).

---

MECHANISM (precise)

RTBAS is dynamic taint-tracking (IFC -- Information Flow Control) adapted for tool-based LLM agents.
It is NOT simple dependency-screening alone; dependency-screening is the sub-mechanism used to
determine WHICH prior messages influence a candidate tool call (i.e., to propagate taint labels
accurately without over-tainting). The two mechanisms work together:
  1. Taint assignment at ingest (four sites, see below).
  2. Dependency screening to compute the join of taint labels of all causally-relevant inputs.
  3. Sink enforcement: block tool call unless context label flows-to policy label, or user confirms.

Threat model:
  - Indirect prompt injection: malicious content injected via tool responses (web content, transaction
    descriptions, external data) that hijacks subsequent tool calls or leaks private data.
  - Privacy leakage: sensitive user data (PII, financial info) unintentionally forwarded to external
    tools or channels.
  - Direct system-prompt modification is NOT in scope; RTBAS defends the runtime data path only.

---

Inputs:

system_message      | str      | arbitrary length | carries developer-assigned (integrity, confidentiality) label pair
user_message        | str      | arbitrary length | carries user-assigned label pair; tainted if user-provided injection
tool_response       | str      | arbitrary length | taint assigned by source trustworthiness: external=Untrusted, internal=Trusted
lm_generated_msg    | str      | arbitrary length | label derived by joining labels of all dependency-screened regions
tool_policy         | dict     | per-tool         | maps tool_id -> allowed label (integrity_floor, confidentiality_ceiling)
security_lattice    | lattice  | 4-point          | L = {(Trusted,Public),(Untrusted,Public),(Trusted,Private),(Untrusted,Private)}

---

Outputs:

tool_call_decision  | enum     | ALLOW / BLOCK / AWAIT_USER_CONFIRMATION
masked_context      | str      | message history with regions above context label redacted (replaced with ◆)
dependency_label    | (l_i,l_c)| join of taint labels of all screened-relevant regions

---

Label Lattice:

Four-point lattice with two independent axes:
  integrity:       Trusted ⊑ Untrusted   (Trusted is more permissive / lower)
  confidentiality: Public ⊑ Private      (Public is more permissive / lower)

Join operator ⊔: takes element-wise maximum (most restrictive) across all relevant region labels.
Flows-to relation ⊑: label l flows-to policy p iff l is no more restrictive than p on both axes.
Final context label = join of all dependency-screened input region labels.

---

Taint Assignment at the Four Ingest Sites:

1. System message (developer-controlled): assigned label m_s^(l_i^s, l_c^s) at agent init
   (Algorithm 4, line 2). Typically (Trusted, Public) or (Trusted, Private).

2. User messages: labeled per input, user_message() with (l_i^u, l_c^u)
   (Algorithm 4, line 4). May be (Untrusted, *) if user prompt injection is in scope.

3. Tool responses: labeled based on source trustworthiness at response receipt
   (Algorithm 4, line 13). External tool responses (web fetches, external APIs) -> (Untrusted, *).
   Internal/trusted tool responses -> (Trusted, *). Confidentiality set by data sensitivity.

4. LM-generated responses: derived at generation time as join of labels of all
   dependency-screened regions (Algorithm 4, line 19). This is the propagation step.

---

Taint Propagation Rules:

Algorithm 2 (dependency screening) identifies which prior context regions are causally relevant
to the candidate tool call. Two screener variants:

LM-Judge screener (Section 7.1):
  - Tags each context region with unique markers.
  - Secondary LM call (GPT-4 class) evaluates which regions are critical to the current tool call.
  - Uses prompt sandwiching (instructions in system message + final message).
  - Output: set of relevant region indices.

Attention-based screener (Section 7.2):
  - Computes per-region importance scores from attention weights via Taylor expansion approximation.
  - Features per region: sum/mean of attention mass, quantiles (20th, 50th, 80th, 99th percentile).
  - Two-layer LSTM binary classifier trained on 40 labeled AgentDojo test cases.
  - Train accuracy 85%, test accuracy 81% (Section 7.2). VERIFIED.
  - Threshold: regions with >0.2 normalized attention mass deemed dependent (Figures 2-3).
  - Dependent data: 56-86% attention mass above 0.2. Non-dependent: 74-86% below 0.2. VERIFIED.

Label join (Algorithm 2, line 6):
  context_label = ⊔(label(r) for r in relevant_regions)
  Final label is at least as restrictive as any contributing region.

Selective masking (Algorithm 3):
  Regions whose label is strictly more restrictive than context_label are redacted with ◆
  before being passed to the LM for tool call generation. This prevents information leakage
  across trust boundaries during LM inference.

---

Sink Enforcement:

Algorithm 4, lines 9-12 (policy check before each tool call):

  if (l_i, l_c) does NOT flow-to P(tool_i):
      if user_confirmation(tool_i) obtained:
          allow
      else:
          BLOCK tool call

P(tool_i) = policy label (allowed max label) for tool tool_i.
User confirmation makes the user the "final arbiter" for policy-violating calls.
No tool call is silently executed when context label exceeds policy label.

---

Measured Effect Sizes (all numbers VERIFIED against HTML fetch of arxiv.org/html/2502.08966v2):

Prompt injection defense (Figure 5, Section 8.1.2):
  Attack prevention:
    LM-Judge:        100% of policy-violating attacks blocked. VERIFIED.
    Attention-based: 100% of policy-violating attacks blocked. VERIFIED.
    Baseline (none): 0% blocked. VERIFIED.
  Utility retained under attack:
    LM-Judge:        ~99% task completion. VERIFIED (Figure 5).
    Attention-based: ~97% task completion. VERIFIED (Figure 5).
  Utility without attack (Workspace suite):
    LM-Judge:        90% (10% degradation from baseline). VERIFIED (Section 8.1.2).
    Attention-based: 92.6% (7.4% degradation). VERIFIED (Section 8.1.2).
  Utility without attack (Slack suite, degraded cases):
    LM-Judge:        33% task completion. VERIFIED (Figure 6, Section 8.1.2).
    Attention-based: 22% task completion. VERIFIED (Figure 6, Section 8.1.2).
    NOTE: Slack suite degradation is a known failure mode; see below.

Privacy leakage defense (Table 2, Section 8.2):
  RTBAS LM-Judge:       FPR=8.1%,  FNR=10.8%. VERIFIED.
  RTBAS Attention-based: FPR=16.2%, FNR=10.8%. VERIFIED.
  GPTs (confirm-every):  FPR=29.7%, FNR=0%.    VERIFIED.
  Redact-all baseline:   FPR=0%,    FNR=51.4%. VERIFIED.
  Utility: RTBAS achieves "near-oracle performance" on both subtle and direct privacy leaks
           (Figure 7). VERIFIED (quoted from Section 8.2).

Runtime overhead (Table 4, Section 8.3.3):
  LM-Judge: requires additional LM API calls per tool call generation cycle.
  Attention-based: described as "minimal runtime overhead" due to lightweight local model + LSTM.
  NOTE: exact latency millisecond numbers not quoted in HTML fetch; tag as UNVERIFIED for ms values.

---

Hyperparameters / Configuration:

screener_type               | LM-Judge or Attention-based | choice    | LM-Judge = higher accuracy, higher cost; Attention = lower cost, lower accuracy
attention_threshold         | 0.2                         | [0,1]     | normalized attention mass cutoff for dependency classification (Section 7.2)
attention_features          | sum/mean + quantiles(20,50,80,99) | fixed | LSTM input feature vector per region
lstm_layers                 | 2                           | fixed     | two-layer LSTM binary classifier
policy_map                  | per-deployment              | required  | maps each tool to (integrity_floor, confidentiality_ceiling)
label_assignment            | per-deployment              | required  | developer assigns labels to system message, user message, each tool source
user_confirmation_gate      | enabled                     | bool      | if disabled, all policy violations become hard blocks with no user escape

---

Critical Invariants (testable assertions):

assert context_label == join(label(r) for r in relevant_regions)
# join = element-wise max on both integrity and confidentiality axes

assert tool_allowed or user_confirmed or tool_blocked
# no tool call executes in a fourth state (silent pass)

assert (l_i, l_c) <= policy(tool) or user_confirmed or blocked
# flows-to check: l_i <= policy_i AND l_c <= policy_c (lattice order)

assert len(masked_context) >= 0 and "◆" not in unmasked_regions
# redacted regions use ◆ placeholder; unmasked regions carry original text

assert attention_screener_train_acc == 0.85 and attention_screener_test_acc == 0.81
# from Section 7.2 -- anchors the accuracy claim to the 40 AgentDojo cases

---

Complexity:

LM-Judge screener: O(k) additional LM calls per tool call, where k=1 (single secondary call with
  tagged regions). Dominant cost = LM API latency.
Attention screener: O(n*f) per candidate tool call where n=number of context regions,
  f=number of attention features (fixed, ~6). LSTM inference is local and lightweight.
Label join: O(n) over relevant regions, constant per-region join.
Masking: O(n) pass over context.
Sink check: O(1) per tool call (single label comparison against policy map).

---

Failure Modes (from paper):

1. Slack suite utility collapse: 33% (LM-Judge) and 22% (Attention) without attack.
   Cause: over-tainting propagates Untrusted label too aggressively through Slack message chains,
   causing legitimate tool calls to be blocked or require confirmation. (Section 8.1.2, Figure 6).

2. Label quality dependency: "Security guarantees only as good as the labels provided and
   the policies enforced." Mis-labeled sources at ingest break all downstream guarantees.
   (Section 8.1.2 -- VERIFIED direct quote).

3. User-provided injection evasion: one workspace test case with user-provided injection
   was not detected (acknowledged in Section 8.1.2). User messages labeled Trusted by default
   defeat the taint model if the user is the attacker.

4. Attention screener accuracy ceiling: 81% test accuracy on 40 samples is a small evaluation set.
   Generalization to novel injection patterns is not established.

5. Exponential-enumeration alternative: prior IFC approaches enumerate all region subsets
   (exponential). RTBAS reduces to constant-per-call via single screener call, but at the cost
   of approximate (not exact) dependency detection.

---

Implementation Pitfalls (do NOT do this):

Do NOT assign Trusted label to all tool responses by default; external web fetches, MCP tool
  outputs, sandbox stdout, and file reads from untrusted paths must default to Untrusted.

Do NOT propagate the join of ALL context regions; only dependency-screened relevant regions
  contribute to the label join. Joining all regions = over-tainting = utility collapse (Slack failure).

Do NOT skip the selective masking step; without masking, the LM sees high-confidentiality content
  during generation of low-confidentiality tool calls, enabling exfiltration.

Do NOT make user_confirmation_gate the only enforcement path; it must be a fallback, not the primary
  gate. Silent allow on policy violation = the baseline (0% attack prevention).

Do NOT assume attention-based screener generalizes beyond the training domain without re-training;
  81% test accuracy on 40 AgentDojo cases is domain-specific.

Do NOT hard-code policy_map as a global policy; per-tool granularity is required (a write-to-disk
  tool and a read-network tool have different integrity/confidentiality ceilings).

---

ELAI Harness Adaptation (taint gate design):

The paper defines four taint ingest sites that map directly to ELAI's data ingestion surface:

  Paper site             -> ELAI equivalent
  -------------------------------------------
  Tool response          -> MCP tool output (any mcp_* call return value)
  Tool response          -> WebFetch / WebSearch result body
  LM-generated response  -> sidecar inference output
  User message           -> user chat input (trust level configurable)
  System message         -> CLAUDE.md + .claude/rules/ (developer-controlled, Trusted)

Recommended label defaults for ELAI:
  MCP tool output:       (Untrusted, Public)  -- MCP server may be compromised
  WebFetch body:         (Untrusted, Public)  -- external web content
  Sandbox stdout:        (Untrusted, Public)  -- sandboxed execution output
  File read (untrusted): (Untrusted, Public)  -- files from outside .elai_cc/
  File read (repo):      (Trusted, Public)    -- committed source files
  User input:            (Trusted, Public)    -- default; (Untrusted,*) if injection in scope
  CLAUDE.md / rules:     (Trusted, Public)    -- developer-authored

Sink policy defaults for ELAI destructive sinks:
  git commit / push:     policy=(Trusted, Public)  -- must not be driven by Untrusted content
  file write to .elai_cc: policy=(Trusted, Public) -- same
  shell exec (Bash):      policy=(Trusted, Public) -- same
  WebSearch (egress):     policy=(Trusted, Private) -- may forward Private data, needs confirmation

Propagation: use dependency-screening (attention or LM-judge) over the agent's context window
  to compute the label join of causally-relevant regions before any destructive tool call.
  Attention screener is the lower-latency option; re-train on ELAI task examples for accuracy.

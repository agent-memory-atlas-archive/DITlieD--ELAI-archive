Algorithm: Semantic Gateway with Zero-Trust Enforcement and EPA-Stateful Fuzzing
ArXiv: 2604.25555
Paper: From CRUD to Autonomous Agents: Formal Validation and Zero-Trust Security for Semantic Gateways in AI-Native Enterprise Systems
Author: Ignacio Peyrano
Reliability: MEDIUM (single-author arXiv preprint, no peer-review label; methodology and equations explicit, eval reproducible; treat numbers as design-targets not validated benchmarks)
Local source: .claude/papers/2604.25555.html (365 KB)
RAG ingestion: SKIPPED (local-rag primary down at ingest time, known Windows reranker crash per memory:project_rag_reranker_crash.md). Re-ingest after RAG recovery with topic="paper-semantic-gateway-fuzzer", project="paper-semantic-gateway-fuzzer".

C-42 PLAN ALIGNMENT NOTES (read these first):
- C-42 claimed paper as "semantic gateway / agent zero-trust fuzzing". VERIFIED — that is exactly the paper's contribution.
- C-42 plan called architecture "1-layer". ACTUAL: 3-layer (Semantic Firewall, Tool-RBAC, Crypto-HITL). Plan must be corrected before EXECUTING.
- Integration-explorer said verdict is "binary". CONFIRMED — verdict is binary (allow / deny), not 4-way. No Quarantine / Audit verdicts in paper. Asymmetric enforcement: deny is absolute at L1/L2; L3 halts pending out-of-band human signature.
- C-42 plan should reference all three layers OR explicitly scope to only L1+L2 (and note L3 deferred) — silent omission would mis-cite the paper.

Architecture (3-layer, execution order is fixed):

  Intent (raw NL + external data)
    -> Layer 1: Semantic Firewall (pre-inference)
    -> Layer 2: Tool-Level RBAC + Micro-segmentation (deterministic, OPA/Rego)
    -> Layer 3 (only if policy engine flags tool as "critical"): Cryptographic Human-in-the-Loop
    -> Tool Runtime (executes only if all preceding layers pass)

Layer 1 — Semantic Firewall:
  Mechanism: micro-models (DeBERTa-v3) classify pre-inference input for prompt injection, taint, context poisoning
  Input: raw natural-language intent + external retrieved data
  Output: cleaned intent OR rejection signal
  Reported failure rate: epsilon_1 = 0.006 (99.4% blocked on tested attack corpus)
  Position: BEFORE LLM inference, BEFORE tool routing

Layer 2 — Tool-Level RBAC + Micro-segmentation:
  Mechanism: Open Policy Agent (OPA) evaluating Rego rules
  Input: proposed tool call (action + target + parameters), agent cryptographic identity token, business context
  Output: allow OR (implicit) deny
  Default posture: deny-all ("default allow = false")
  Critical guarantees:
    - role hierarchy enforced from token, not from agent-supplied claims
    - BOLA prevention: ownership / department fields fetched from authoritative DB, agent claims ignored
  Reported: "100% of malicious requests bypassing L1 intercepted" on the eval corpus

Layer 3 — Cryptographic Human-in-the-Loop:
  Mechanism: speculative suspension; evidence package = SHA-256 hash of (action, params, CoT justification); challenge sent to operator device; resume only on valid signature
  Trigger: policy engine flag on tools tagged "critical / irreversible"
  Output: resume OR halt-indefinitely
  Audit ledger: hash-chained append-only SQLite, records (original_intent, generated_plan, evaluated_policy, applied_params, final_state_mutation)

Inputs (gateway-wide):
  intent | string | natural language | mandatory
  agent_token | signed token | contains role + department + identity | mandatory
  retrieved_context | string | from external data sources | optional, taint-tracked
  tool_schema | JSON Schema | per registered MCP tool | mandatory at registration time
  rego_policy | Rego source | per tool family | mandatory at deployment time

Outputs:
  verdict | enum {allow, deny} | binary, deterministic at L1+L2
  hitl_state | enum {not_required, pending_signature, signed, halted} | only set when L3 fires
  audit_record | hash-chained tuple | always emitted, SHA-256 linked to previous record
  reject_reason | string | populated on deny, includes which layer rejected

Hyperparameters:
  epsilon_1 (L1 false-negative rate) | 0.006 reported | range depends on micro-model | high sensitivity to adversarial prompt distribution
  fuzzing_sequence_count | 500000 | scales O(2^|T|) with tool count | hard limit at |T| > 200 (state explosion)
  mutation_split | 80/20 (valid/adversarial) | 70/30 to 90/10 acceptable | tune per corpus
  base_corpus_size | 10000 legitimate intents | minimum 1000 for stable BOLA discovery
  z3_pruning | enabled | disable only for debug | major perf gain on |T| > 50

Fuzzing methodology (EPA = Enabledness-Preserving Abstraction):
  Stateful: YES — abstract state space S_tilde subset of 2^T where each state = unique permitted tool-combination
  Schema-driven: YES — MCP JSON Schemas + Rego policies feed mutation grammar
  Mutation strategy: 80% guided valid-tool selection, 20% adversarial (parameter boundary violations, injection patterns, context taint)
  Sequence length: multi-turn (paper does not pin exact N per sequence; corpus is 500K sequences total)
  State-graph reduction (3 heuristics):
    Reduce_True: prune universally enabled read-only tools
    Reduce_Equal: fuse tools with identical OPA preconditions
    Unreachable: Z3 SMT static analysis removes provably unreachable branches
  Invariant assertions: property-based (e.g., NoSharingOverwrite); falsification = vulnerability found
  Reported BOLA-discovery: median 52 iterations (across 100 seeds) on the financial-doc benchmark

SSB / event vocabulary:
  PAPER DOES NOT DEFINE explicit event names. Audit ledger writes a tuple but no event-bus contract.
  C-42 must INVENT its own event names — paper provides no ground truth here.
  Suggested ELAI-side names (NOT from paper, mark as design decision in C-42):
    GATEWAY_ALLOW { layer_passed, tool_id, agent_id, audit_hash }
    GATEWAY_DENY { rejecting_layer in {1,2}, tool_id, reason_code, agent_id, audit_hash }
    GATEWAY_HITL_PENDING { tool_id, evidence_hash, challenge_id }
    GATEWAY_HITL_RESOLVED { challenge_id, signed: bool }
    GATEWAY_AUDIT_LINK { prev_hash, current_hash, record_payload }
  If C-42 plan asserts these are "per paper", that is FALSE — flag as ungrounded.

Critical invariants (testable assertions):
  assert verdict in {"allow", "deny"}  # binary verdict (no quarantine/audit)
  assert default_policy_value == False  # deny-all default
  assert l1_runs_before_l2 and l2_runs_before_l3 and l3_runs_before_tool  # ordering
  assert agent_supplied_ownership_field is None or agent_supplied_ownership_field == db_authoritative_ownership  # BOLA: agent claims ignored
  assert audit_record["prev_hash"] == sha256(previous_audit_record_canonical)  # hash chain unbroken
  assert l3_triggered iff tool.tagged_critical  # HITL only on critical tools
  assert reject_reason["layer"] in {1, 2} when verdict == "deny"  # rejection always attributed

Complexity:
  Time per request (steady-state): O(|tools_in_corpus|) for L1 router + O(|policy_rules|) for OPA eval + O(1) for L3 dispatch
  Abstract state space (fuzzer): O(2^|T|) — exponential in tool count, EPA reductions are the only thing keeping it tractable
  Routing knapsack (Eq. 3 in paper): O(|T| * L_max) where L_max = context-window budget
  Parallelism: L1 / L2 / L3 are sequential per request; multiple requests parallelise trivially (stateless per layer modulo audit-ledger append, which is serialised)

Failure modes (Section 12 of paper, "Threats to Validity"):
  State-space explosion: |T| > 200 trips the fuzzer's 2-hour wall-clock timeout — eval becomes incomplete, NOT unsound
  Cryptographic preconditions block fuzzer: tight hash-collision or timestamp preconditions cannot be satisfied by random mutation, so deeper vulnerabilities behind crypto gates are MASKED
  Domain generalisation: eval is ONLY on financial document management; transfer to trading / physical infra / healthcare is not demonstrated
  Adversarial micro-model attacks on L1: epsilon_1 = 0.006 measured on the eval corpus; novel jailbreak distributions may push this higher
  L3 social-engineering: operator approves under pressure — paper acknowledges but does not mitigate

Implementation pitfalls (do NOT do this):
  Do NOT trust agent-supplied identity / role / ownership fields — always read from DB-of-record (BOLA defense)
  Do NOT default policies to allow — Rego deny-all default is structural, not stylistic
  Do NOT skip L1 because L2 exists — L1 catches taint that L2 cannot see (semantic layer)
  Do NOT collapse the 3 layers into 1 deterministic check — the asymmetry (probabilistic L1 + deterministic L2 + human L3) IS the contribution
  Do NOT emit a 4-way verdict (allow/deny/quarantine/audit) and cite this paper — paper is binary
  Do NOT advertise the 100% / 84.2% / 5.3x numbers as validated benchmarks — they are from a single-author preprint on one domain (financial docs); cite as design targets only
  Do NOT skip the audit hash chain — without it, post-hoc forensics on agent decisions are impossible (the paper treats this as load-bearing)
  Do NOT run the fuzzer with |T| > 200 without EPA reductions enabled — state explosion will time out before discovering anything

Compose-with-existing-ToolGate (C-42 integration note):
  Paper's L2 = OPA/Rego policy gate runs BEFORE tool invocation, AFTER LLM has selected/proposed the tool.
  If ELAI already has a ToolGate that runs at a different point, two integration options:
    Option A (paper-faithful): paper's L2 SUPERSEDES the existing gate; existing gate is removed or downgraded to logging.
    Option B (compose): existing gate runs first (e.g., capability whitelist), L2 runs second (policy + BOLA), both must allow.
    Paper does not address composition with prior gates — design decision for ELAI, must be flagged in C-42 plan, not silently assumed.
  L1 (Semantic Firewall) is BEFORE LLM inference — if ELAI's existing ToolGate runs AFTER inference, L1 is genuinely new infrastructure, not a replacement.

Reliability assessment:
  Source: single-author arXiv preprint (2604.25555), no venue / peer-review marker visible
  Math: equations explicit (Eq. 1-5), reproducible from text
  Eval: single domain (financial doc management); numbers come from author's own experimental harness
  External validation: NONE FOUND (no citations from this paper in other 2604.* papers as of fetch date 2026-05-24)
  Recommended use: cite as architectural pattern / design hypothesis. Do NOT cite the 100% / 84.2% / 5.3x numbers as proven performance — use them as design targets and add ELAI-side measurement.
  Reliability tier: MEDIUM (per ELAI source-tiers: official-paper = HIGH, peer-reviewed = HIGH, single-author preprint = MEDIUM, decompiled = LOW).

Contract test stubs (copy into tests/contracts/test_semantic_gateway_contract.py — Rust crate; the Python below is pseudocode form, port to Rust per ELAI conventions):

  def test_verdict_is_binary():
      v = gateway.evaluate(intent="...", token=valid_token, tool="read_doc")
      assert v.verdict in {"allow", "deny"}, f"Paper specifies binary verdict, got {v.verdict}"

  def test_default_policy_denies():
      v = gateway.evaluate(intent="...", token=valid_token, tool="unregistered_tool")
      assert v.verdict == "deny", "Deny-all default per paper Section 4"

  def test_layer_order_l1_then_l2_then_l3():
      trace = gateway.evaluate_with_trace(intent="prompt-injection-payload", token=valid_token, tool="critical_tool")
      layer_indices = [step.layer for step in trace.steps if step.layer is not None]
      assert layer_indices == sorted(layer_indices), f"Layers must execute in order, got {layer_indices}"

  def test_bola_agent_claim_ignored():
      # agent claims ownership of doc=999, DB says owner=other_user
      v = gateway.evaluate(intent="...", token=token_for_user_42, tool="update_doc",
                            params={"doc_id": 999, "claimed_owner": 42})
      # DB has owner != 42; L2 must deny regardless of agent claim
      assert v.verdict == "deny" and v.rejecting_layer == 2

  def test_audit_hash_chain_unbroken():
      records = gateway.audit_records_since(t=0)
      for prev, curr in zip(records, records[1:]):
          assert curr.prev_hash == sha256(canonical(prev)), "Hash chain broken at record id={curr.id}"

  def test_hitl_only_on_critical_tools():
      v_normal = gateway.evaluate(intent="...", token=valid_token, tool="non_critical_tool")
      v_critical = gateway.evaluate(intent="...", token=valid_token, tool="critical_irreversible_tool")
      assert v_normal.hitl_state == "not_required"
      assert v_critical.hitl_state in {"pending_signature", "signed", "halted"}

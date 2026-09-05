Source: Life-Harness (github.com/Tianshi-Xu/Life-Harness, MIT)
Reliability: MEDIUM (reviewed source, shallow clone read directly)
RAG ingest: FAILED (local-rag reranker crash, known intermittent issue on this machine)
Files examined:
  TauBench/src/tau2/harness/base.py (149 LOC)
  TauBench/src/tau2/harness/skills.py (1913 LOC)
  TauBench/src/tau2/harness/policy_rag.py (543 LOC)
  TauBench/src/tau2/harness/h3_tools.py (473 LOC)
  TauBench/src/tau2/harness/airline.py (1646 LOC)
  TauBench/src/tau2/harness/retail.py (1613 LOC)
  TauBench/src/tau2/harness/telecom.py (847 LOC)
  TauBench/src/tau2/harness/banking_knowledge.py (506 LOC)
  TauBench/Harness.md (design doc, identical to AgentBench/Harenss.md)
  AgentBench/src/server/harness/__init__.py (43 LOC)
  AgentBench/src/server/harness/os_interaction.py (2514 LOC)
  AgentBench/src/server/harness/alfworld.py (1580 LOC)
  AgentBench/src/server/harness/dbbench.py (2736 LOC)
  AgentBench/src/server/harness/webshop.py (2364 LOC)
  AgentBench/src/server/tasks/os_interaction/task.py (partial, call-site read)
  TauBench/src/tau2/runner/simulation.py (93 LOC)
  TauBench/src/tau2/environment/environment.py (partial, 80 LOC)

======================================================================
LAYER ARCHITECTURE
======================================================================

Four named layers (from Harness.md, confirmed in code):

H2  Action Gate
    Trigger: after agent proposes tool call, BEFORE execution
    Mechanism: HarnessRule.check(db, **kwargs) raises ValueError on violation
    Agent sees: ToolMessage(error=True, content="Error: <message>")
    Also handles: text-rescue (AgentBench only) - parse JSON/kwarg/XML from
    free-text agent output when tool-call format is malformed

H3  Tool-Description Embedding
    Trigger: episode initialization, once (not per-turn)
    Mechanism: _append_hint_to_tool() wraps the tool's __doc__ with policy
    constraints; openai_schema includes extended description in every API call
    because tool definitions are re-sent every turn
    Key insight: tool descriptions have constant attention weight across all
    turns; system prompt salience degrades as context grows

H4  Post-Execution Monitor / Annotator
    Trigger: after tool executes successfully, before next agent turn
    Mechanism: HarnessAnnotator.annotate(db, result, **kwargs) returns str|None
    appended to tool result visible to agent
    Also handles (AgentBench): stall detection, loop detection, budget warn/force

H5  Goal-Directed Guidance
    Trigger: episode start (build-time), once per episode
    Mechanism: BM25-based skill retrieval over task description; top-k skills
    prepended to system prompt as "Learned Tips" block BEFORE domain policy
    (highest attention position)

Layer execution order per agent turn:
  episode_start:
    H3 -> patch_*_tool_descriptions() called once at task init
    H5 -> retrieve_skills() called once, result prepended to system prompt
  per_turn loop:
    agent proposes tool call
    H2 -> HarnessedToolKitMixin.use_tool() -> rule.check() [pre-exec, blocking]
    tool executes via super().use_tool()
    H4 -> annotator.annotate() [post-exec, non-blocking, appends to result]
    agent sees annotated result
    (AgentBench only: post_step_monitor() runs H4 stall/loop/budget logic)

======================================================================
CONCRETE DATA SHAPES (verbatim from source)
======================================================================

1. ENVIRONMENT-CONTRACT TEXT FRAGMENT (H3 tool description, airline domain)

Tool: cancel_reservation
H3 hint text (appended to tool's __doc__):

"POLICY CONSTRAINTS - verify ALL of the following before calling:

Eligibility: at least ONE condition must be true:
  1. Reservation was booked within the last 24 hours.
  2. The airline cancelled a flight in this reservation.
  3. Cabin is 'business'.
  4. Travel insurance was purchased. A health/weather reason is needed for
     insurance-covered full-refund reasoning, but do not skip a cancellation
     solely because an insured user gives a non-health/non-weather reason and
     explicitly still wants to proceed.

If any flight has status 'flying' or 'landed': DO NOT cancel - call
transfer_to_human_agents instead.

Tip for basic_economy: call update_reservation_flights first to upgrade the
cabin to 'business' (keeping the EXACT SAME flight numbers). Upgrading only
to 'economy' does NOT satisfy cancellation condition 3."

Implementation: _append_hint_to_tool(tool: Tool, hint: str) -> Tool
  creates functools.wraps wrapper, appends hint to __doc__, builds new Tool()

2. ACTION-REALIZATION VALIDATION RULE (H2, EXEC/Block gate, retail domain)

Class: CancelPendingOrderRule (in retail.py)
  tool_name = "cancel_pending_order"
  def check(self, db: RetailDB, order_id: str, **_: Any) -> None:
    order = db.orders.get(order_id)
    if order is None: return
    st = order.status
    if st == "cancelled":
      raise ValueError(f"Order {order_id} is already cancelled...")
    if st == "pending (item modified)":
      raise ValueError(f"Order {order_id} has status 'pending (item modified)'...")
    if st != "pending":
      raise ValueError(f"Order {order_id} has status '{st}'...")

Protocol (HarnessedToolKitMixin.use_tool in base.py lines 124-149):
  for rule in self.harness_rules.get(tool_name, []):
      rule.check(self.db, toolkit=self, **kwargs)   # raises ValueError -> blocked
  result = super().use_tool(tool_name, **kwargs)     # execute only if no raise

The ValueError propagates up; environment catches it and returns:
  ToolMessage(error=True, content="Error: <ValueError message>")
The agent sees the error and can self-correct.

3. PROCEDURAL-SKILL LIBRARY ENTRY (H5, retail domain)

Dataclass: Skill (skills.py lines 43-80)
  id: str                          # "pending_item_removal_fallback"
  title: str                       # "Pending orders do not support partial item cancellation"
  pattern: str                     # space-separated keywords for BM25 index
  tip: str                         # actionable text injected verbatim into system prompt
  source: Literal["failure","success"]
  task_ids: list[int]              # training task IDs (documentation only)
  issue_type: str | None           # domain-specific routing bucket (telecom: "mms","abroad","service","data")

Verbatim example entry (retail domain, skills.py lines 126-148):
  Skill(
    id="pending_item_removal_fallback",
    title="Pending orders do not support partial item cancellation",
    pattern=(
      "pending remove item cancel just only single item one item partial "
      "cancellation cancel the charger cancel the hose not shipped whole order"
    ),
    tip=(
      "For a pending order, cancel_pending_order cancels the whole order; "
      "there is no tool for removing only one selected item. If the user "
      "asked not to cancel the whole order, do not call cancel_pending_order. "
      "Use full-order cancellation only when the user explicitly gave or "
      "confirmed that fallback AND accepts the actual refund route. "
      "Cancellation refunds go to the original payment method, not a chosen "
      "gift card; if the user's fallback requires gift-card refund and that "
      "is impossible, do not cancel."
    ),
    source="failure",
    task_ids=[28, 30, 57, 76, 88],
  )

4. TRAJECTORY-REGULATION PATTERN DETECTOR (H4, AgentBench OS domain)

OSShellState (os_interaction.py lines 1006-1019):
  bash_history: List[str]                        # all bash commands issued
  last_output_raw: str
  last_output_numeric_candidates: List[str]
  last_output_truncated: bool
  last_output_was_empty: bool
  last_output_had_error: bool
  empty_output_streak: int
  text_only_streak: int
  numeric_candidate_history: List[Tuple[str, str]]
  rescue_hits: int
  answered: bool
  candidate_numeric_answer: Optional[str]

Stall detection (from OSHarnessConfig):
  h4_stall_window = 3       # same bash+output N times in a row -> stall
  h4_empty_output_threshold = 2   # empty outputs in a row -> hint
  h2_repeat_bash_block_after = 2  # same bash N times -> force answer
  h2_text_only_streak_force = 2   # N consecutive text-only turns -> hard force

Loop A<->B detection: counts repeating (command, output) pairs in
bash_history over a sliding window; if the same pair appears >= h4_stall_window
times, post_step_monitor() injects a corrective nudge.

Budget warn/force (H4):
  h4_budget_warn_threshold = 3   # remaining rounds <= N -> soft warn
  h4_budget_force_threshold = 2  # remaining rounds <= N AND candidate ready -> hard force

======================================================================
LAYER HOOK-IN: FUNCTION NAMES AND CALL ORDER
======================================================================

TauBench per-domain pattern:

class HarnessedRetailTools(HarnessedToolKitMixin, H3RetailToolDescriptionMixin, RetailTools):
    harness_rules = {
        "cancel_pending_order": [CancelPendingOrderRule(), CancelReasonRule()],
        "modify_pending_order_items": [ModifyPendingOrderRule(), ...],
        ...
    }
    harness_annotators = {
        "get_order_details": [OrderEligibilityAnnotator()],
        "exchange_delivered_order_items": [ExchangePriceDiffAnnotator(), ...],
        ...
    }

MRO-driven call chain (base.py lines 124-149):
  HarnessedToolKitMixin.use_tool(tool_name, **kwargs)
    -> rule.check(self.db, toolkit=self, **kwargs)  [H2, pre-exec; raises on fail]
    -> super().use_tool(tool_name, **kwargs)         [actual tool execution]
    -> self._harness_successful_calls.append(...)    [audit trail]
    -> annotator.annotate(self.db, result, ...)      [H4, post-exec; returns str|None]
    -> if annotations: return serialized_result + "\n\n" + "\n".join(annotations)

AgentBench per-environment pattern (task.py _judge loop, lines 549-617):
  harness_runtime = OSHarnessRuntime(self.harness_config)
  harness_runtime.init_task(config.description)      [H0 parse, H5 retrieval]
  cold_skills = harness_runtime.cold_start_skill_hints()  [H5]
  inject tips into initial system message
  for round_num in range(self.round_limit):
    action_result = await self._handle_agent_action(..., harness_runtime=...)
      # inside: H2 rescue_tool_call_from_text() for malformed calls
      # inside: H2 duplicate-bash gate
    post_step_monitor(remaining_rounds, ...)  [H4: stall/loop/budget logic]

H3 for AgentBench: patch_os_tool_descriptions(tools) called ONCE at class init
  (OSInteraction.__init__ line 186-187), before any episodes run.

======================================================================
BM25 SKILL-RETRIEVAL IMPLEMENTATION
======================================================================

Library: pure Python stdlib only (math, re, collections.Counter). No external ML.
Same implementation exists in two places:
  skills.py (for TauBench H5 skill library)
  policy_rag.py (for TauBench H5 policy chunk retrieval)
  os_interaction.py (for AgentBench H5 OS skill retrieval)

Parameters (BM25-Okapi):
  K1 = 1.5
  B  = 0.75
  IDF formula: log((N+1)/(df+1)) + 1.0 (smoothed, always >= 1)
  TF normalization: freq*(K1+1) / (freq + K1*(1 - B + B*dl/avgdl))

Tokenization:
  re.findall(r"[a-z0-9_]+", text.lower())
  stopword filter (100+ word frozenset, domain-neutral function words)
  min token length > 1

Index build: _build_index(domain) cached via @lru_cache(maxsize=8)
  builds token lists for all skills in domain
  computes per-term IDF over the corpus
  returns (skills_list, token_lists, idf_dict)

retrieve_skills(domain, task_description, top_k=3, min_score=None):
  Step 1: issue-type routing (pre-filter, telecom only)
    _allowed_skill_types(query) detects "mms"/"abroad"/"no service"/"mobile data"
    returns set of allowed issue_type buckets; None = no filter
  Step 2: per-domain precision guard
    _retail_skill_matches_context(skill, query) - keyword-match gates per skill.id
    _airline_skill_matches_context(skill, query) - same
  Step 3: BM25 scoring over filtered candidates
  Step 4: per-domain score boost (integer offset added to BM25 score)
    _retail_skill_score_boost: e.g. confirmation_boundary +6.5, multi_order_scan +3.0
    _airline_skill_score_boost: e.g. other_upcoming_reservations_scan +6.0
  Step 5: sort descending, take top_k, filter by min_score

min_score defaults: airline=2.5, retail=1.5, other=1.5
  AgentBench OS default: h5_score_threshold=7.5 (much higher, fewer/shorter skills)

top_k caps per domain: retail=min(top_k,2), airline=min(top_k,1)

format_skills_block(skills) -> str:
  prefix: "## Task-Specific Guidance" header + disclaimer
  per skill: "- **{title}**\n{tip}\n"
  appended with "---"
  empty list -> empty string (no injection)

Policy RAG (policy_rag.py) uses the same BM25 but over chunked policy text:
  PolicyRetriever splits policy by "## " and "### " sections
  min_chunk_tokens=20 (merges tiny chunks upward)
  retrieve(query, top_k=3) -> list[str] of relevant policy text chunks
  Cached by rules_corpus string via @lru_cache(maxsize=32)

======================================================================
LLM INTERVENTION-PROPOSAL PIPELINE (TauBench)
======================================================================

No Codex/LLM intervention-proposal pipeline found in this codebase.
All interventions are deterministic:
  H2: HarnessRule.check() is pure Python DB-state inspection (no LLM)
  H3: static string append at init (no LLM)
  H4: HarnessAnnotator.annotate() is pure Python DB-state inspection (no LLM)
  H5: BM25 retrieval over a MANUALLY authored skill library (no LLM at runtime)

Skill library authorship: skills were extracted from training trajectory analysis
by human/LLM offline (documented as "derived from train-split trajectory analysis,
Apr 2026") but the retrieval pipeline at runtime is fully deterministic.

The paper-ingestor previously produced life-harness-paper-interface.md which
references "Codex/LLM intervention-proposal pipeline" — this refers to the
OFFLINE skill authoring workflow (analyze failed trajectories, write Skill entries),
NOT an online LLM-in-the-loop component. The harness itself is LLM-free at runtime.

Regression-check mechanism:
  For TauBench: HarnessRule.check() is replay-safe by design constraint:
    "implementations MUST only read from DB/kwargs; must NOT have side-effects
    or maintain cross-call state." (base.py docstring)
  Rules are validated against set_state replay: Environment.set_state() replays
  write-tool calls; read-only H2 rules do not interfere with replay.
  For AgentBench: harness_trace dict records all H2/H3/H4/H5 events per sample;
  injected into openai_messages via [HARNESS_TRACE_V1] tag for offline analysis.

======================================================================
PER-ENVIRONMENT HARNESS FILE INVENTORY
======================================================================

TauBench environments (src/tau2/harness/):
  airline.py       1646 LOC  26 Rule+Annotator classes
    H2 rules: BookReservationPaymentRule, MaxPassengersRule, FlightStatusBookableRule,
              BaggageAllowanceRule, BookReservationPaymentTotalRule,
              CancelFlightRule, BasicEconomyFlightChangeRule,
              NoOriginDestChangeRule, CabinChangeOnFlyingReservationRule,
              FlightUpdateNoCertificateRule, NoBaggageDecreaseRule,
              BaggageAllowanceUpdateRule, NoPassengerCountChangeRule,
              CertificateEligibilityRule, CertificateFlightStatusRule,
              DelayedCompensationRequiresChangeRule, CertificateAmountCapRule
    H4 annotators: UserReservationSummaryAnnotator, ReservationPaymentAnnotator,
                   ReservationFlightSummaryAnnotator, CancelRefundAnnotator,
                   FlightUpdatePaymentAnnotator, BookingPaymentAnnotator
    H5 skills: 7 entries (AIRLINE_SKILLS in skills.py)

  retail.py        1613 LOC  36 Rule+Annotator classes
    H2 rules: CancelPendingOrderRule, CancelReasonRule,
              ModifyPendingOrderRule, NonEmptyItemIdsRule, NonEmptyNewItemIdsRule,
              ItemIdsNewItemIdsParityRule, ValidateModifyItemsExistRule,
              NoDuplicateItemIdsRule, ValidateNewItemIdExistsRule,
              ModifyItemsProductTypeRule, ModifySameItemRule,
              ModifyPaymentSameMethodRule, ModifyPaymentGiftCardBalRule,
              ExchangeDeliveredOrderRule, ValidateExchangeItemsExistRule,
              ExchangeItemsProductTypeRule, ExchangeSameItemRule,
              ReturnDeliveredOrderRule, ValidateReturnItemsExistRule
    H4 annotators: OrderEligibilityAnnotator, ItemAvailabilityAnnotator,
                   ExchangePriceDiffAnnotator, GiftCardBalanceAnnotator,
                   ReturnRefundAnnotator, CancelRefundAnnotator
    H5 skills: 9 entries initially (RETAIL_SKILLS in skills.py, grows with precision guards)

  telecom.py       847 LOC   19 Rule+Annotator classes
    H2 rules: CustomerIDValidationRule, LineOwnershipRule,
              PhoneNumberFormatRule, BillOwnershipRule,
              SendPaymentExpiredContractRule, SendPaymentRequestOverdueRule,
              SendPaymentOneAtATimeRule, ResumeLineStatusCheck,
              ResumeLineEligibilityRule, RefuelDataLimitRule,
              RefuelDataActiveLineRule, SuspendLineStatusRule,
              EnableRoamingAlreadyEnabledRule, DisableRoamingAlreadyDisabledRule
    H4 annotators: CustomerPhoneLineAnnotator, DataQuotaAnnotator,
                   LineStatusAnnotator, BillsOverdueAnnotator,
                   PaymentWorkflowAnnotator, EnableRoamingResultAnnotator
    H5 skills: 12 entries (TELECOM_SKILLS in skills.py)

  banking_knowledge.py  506 LOC  13 Rule+Annotator classes
    H2 rules: LogVerificationUserIdRule, UnlockDiscoverableExistsRule,
              CallDiscoverableUnlockedRule
    H4 annotators: KBSearchDiscoverableToolAnnotator, UnlockDiscoverableErrorAnnotator,
                   CallDiscoverableErrorAnnotator, CallDiscoverableSuccessAnnotator,
                   GiveDiscoverableErrorAnnotator, EmailNotFoundAnnotator,
                   NameLookupEmailHintAnnotator, UserTransferRequestAnnotator,
                   ReferralsFoundAnnotator
    H5 skills: 6 entries (BANKING_KNOWLEDGE_SKILLS in skills.py)

  Also: policy_rag.py (PolicyRetriever, ToolKitWithPolicyRetrieval,
        split_structural_prefix_and_rules for RAG-as-tool mode)

AgentBench environments (src/server/harness/):
  os_interaction.py  2514 LOC  OSHarnessConfig + OSHarnessRuntime + OS_SKILLS (21 entries)
    H0: parse_task_context() -> OSTaskContext (task_type, answer_shape, path, ext, recursive, case)
    H1: OSShellState (bash_history, candidates, truncation flags, stall counters)
    H2: rescue_tool_call_from_text() (JSON/kwarg/positional/bare/XML rescue from free text)
        duplicate-bash gate, answer normalisation, text-only-streak force
    H3: patch_os_tool_descriptions(tools) - static desc patches
    H4: post_step_monitor(remaining_rounds) - stall/error/empty/loop + budget warn/force
    H5: cold_start_skill_hints() -> BM25 over OS_SKILLS
    21 OS_SKILLS entries covering: count_files/lines/matches/unique, size filters,
       mtime, case-insensitive, find largest/smallest, system info, grep pitfalls,
       date, IP extraction, answer format, truncation handling

  alfworld.py      1580 LOC  ALFWorldHarnessConfig + ALFWorldHarnessRuntime
    H2: action rescue from free text, valid-action check, invalid-room gate
    H3: patch_take_action_tool_description()
    H4: stall/loop detection over action history
    H5: BM25 over ALFWORLD_SKILLS (goal-type classification then skill lookup)

  dbbench.py       2736 LOC  DBBenchHarnessConfig + DBBenchHarnessRuntime
    H2: SQL rescue (extract from code fence/prose), safety filter (DROP/TRUNCATE/ALTER block),
        schema validation (table/column existence checks), duplicate-query gate
    H3: patch_dbbench_tool_descriptions(), patch_dbbench_system_prompt()
    H4: error recovery prompts, stall detection, budget warn/force
    H5: BM25 over DBBENCH_SKILLS (SQL task type classification)

  webshop.py       2364 LOC  WebShopHarnessConfig + WebShopHarnessRuntime
    H2: action rescue, invalid-action gate, search-only-first-step enforcement
    H3: patch_webshop_tool_descriptions()
    H4: stall/loop detection, budget warn/force, product-not-found recovery
    H5: BM25 over WEBSHOP_SKILLS

======================================================================
PUBLIC API SURFACE
======================================================================

TauBench:
  HarnessRule (Protocol in base.py)
    tool_name: str
    check(db: DB, **kwargs: Any) -> None  [raises ValueError to block]

  HarnessAnnotator (Protocol in base.py)
    tool_name: str
    annotate(db: DB, result: Any, **kwargs: Any) -> str | None  [returns annotation or None]

  HarnessedToolKitMixin (base.py)
    harness_rules: dict[str, list] = {}
    harness_annotators: dict[str, list] = {}
    use_tool(tool_name: str, **kwargs: Any) -> Any
      [runs H2 checks, executes, appends H4 annotations, returns result]
    _harness_successful_calls: list  [audit trail, set lazily on first call]

  retrieve_skills(domain: str, task_description: str, top_k: int = 3,
                  min_score: float | None = None) -> list[Skill]
    [BM25 + issue-type routing + precision guard + score boost]

  format_skills_block(skills: list[Skill]) -> str
    [formats as "## Task-Specific Guidance" prefix block]

  get_policy_chunks(policy_text: str, query: str, top_k: int = 3) -> list[str]
    [BM25 over policy rules corpus only, excludes Domain Basic structural prefix]

  split_structural_prefix_and_rules(policy_text: str) -> tuple[str, str]
    [splits policy into (structural_prefix, rules_corpus) for RAG-tool mode]

  wrap_toolkit_with_policy_retrieval(inner, policy_text, top_k=3) -> ToolKitWithPolicyRetrieval
    [adds retrieve_policy tool to existing toolkit for RAG-as-tool mode]

  _append_hint_to_tool(tool: Tool, hint: str) -> Tool
    [H3 mechanism: wraps __doc__, builds new Tool with extended schema]

AgentBench:
  OSHarnessConfig (dataclass)
    enabled, h2_enabled, h3_enabled, h4_enabled, h5_enabled: bool
    h2_repeat_bash_block_after: int = 2
    h2_text_only_streak_force: int = 2
    h4_stall_window: int = 3
    h4_empty_output_threshold: int = 2
    h5_top_k: int = 1
    h5_score_threshold: float = 7.5
    h4_budget_warn_threshold: int = 3
    h4_budget_force_threshold: int = 2

  OSHarnessRuntime
    init_task(description: str) -> None  [H0 parse, H5 pre-retrieval]
    cold_start_skill_hints() -> list[dict]  [H5: returns scored skill dicts]
    pre_exec_check(bash_cmd, round_num, ...) -> Optional[str]  [H2 gate]
    post_step_monitor(remaining_rounds, ...) -> Optional[str]  [H4 monitor]

  rescue_tool_call_from_text(text: str, tools: list) -> Optional[dict]
    [H2 rescue: tries JSON/kwarg/positional/bare/XML parse, returns tool_call dict]

  patch_os_tool_descriptions(tools) -> tools
    [H3: patches __doc__ of bash_action + finish_action tools in place]

======================================================================
BEHAVIORAL CONTRACTS
======================================================================

HarnessRule.check() is read-only with respect to DB state. Rules MUST NOT:
  mutate DB, maintain cross-call state, read conversation history.
  Only DB snapshot + kwargs are permitted inputs.

HarnessAnnotator.annotate() is read-only. Same constraints as HarnessRule.
  Runs on SUCCESS path only (never blocks). DB is post-execution snapshot.

HarnessedToolKitMixin.use_tool() is the single integration point.
  All rules for a tool run BEFORE execution; any raise aborts the call.
  All annotators for a tool run AFTER execution; none can abort.
  Successful calls are logged to _harness_successful_calls for audit.

H3 patching is idempotent if called once per episode class init.
  AgentBench: patch_*_tool_descriptions(tools) called in OSInteraction.__init__
  TauBench: H3*ToolDescriptionMixin.get_tools() wraps on every call (lazy)

H5 injection is build-time once per episode. Skills are NOT re-injected per turn.

Policy RAG (mode 2): when retrieve_policy is used as a tool (not build-time),
  the full procedural policy is OMITTED from the system prompt.
  Only structural_prefix (preamble + Domain Basic) is kept in system prompt.
  Agent must call retrieve_policy before any policy-sensitive write tool.

======================================================================
PLATFORM CONSTRAINTS
======================================================================

Python version: 3.10+ (union types X | Y syntax, Pydantic v2 assumed from model_dump())
TauBench dependencies: pydantic, loguru (see pyproject.toml)
AgentBench dependencies: openai, asyncio (docker-based containers)
No GPU/ML dependencies in harness code; all harness logic is CPU/stdlib

BM25 is self-contained (math + re + collections.Counter only).
  No rank_bm25, no retrieval libraries.
  Same stopword set used in all three BM25 implementations but NOT identical
  (skills.py frozenset is larger, includes "my","me","us","our","your";
   os_interaction.py frozenset is smaller/different).

Replay safety constraint: H2/H4 rules and annotators must be DB-state-only.
  This is documented as a hard constraint in base.py and enforced by design review.
  Violating it breaks set_state replay in TauBench.

======================================================================
GOTCHAS
======================================================================

H2 ValueError message is the agent's only signal. Messages must be clear enough
  for an LLM to understand what went wrong. Vague messages cause retry loops.

H4 annotations are APPENDED to the serialized tool result, not prepended.
  Format: serialize_tool_result(result) + "\n\n" + "\n".join(annotations)
  Agents that only read the first part of a tool result will miss annotations.

H5 skills are injected at HIGHEST attention position (before domain policy).
  format_skills_block() returns the block meant to be prepended to domain_policy.
  If the caller appends (not prepends), attention salience is lost.

BM25 IDF formula differs from classic Okapi: log((N+1)/(df+1)) + 1 (always >= 1).
  Classic Okapi: log((N - df + 0.5)/(df + 0.5) + 1).
  The "+1" bias means even zero-match terms contribute 1.0 to IDF; this is a
  "smoothed" variant, slightly different from Robertson-Sparck Jones.

Skill precision guards (_retail_skill_matches_context) are REQUIRED.
  BM25 alone over-injects narrow skills because retail task descriptions share
  many generic words. The guards are not optional optimizations; disabling them
  causes false injections documented as IDX-level regressions in os_interaction.py.

Domain registry is static (DOMAIN_SKILLS dict at bottom of skills.py).
  Adding a new domain requires: (1) define SkillList, (2) add to DOMAIN_SKILLS,
  (3) optionally add _<domain>_skill_matches_context precision guard.

H3 hint text must avoid duplicating H2 rules (as documented in skills.py header).
  H3 is for calibration (contract understanding); H2 is enforcement.
  Duplicate content between H3 and H5 is also discouraged but less critical.

======================================================================
CRITICAL TESTABLE ASSERTIONS
======================================================================

assert all(rule.check.__doc__ is not None or True for ...)  # no-op: check() just raises
# Real assertions:
assert sum(1 for pm in payment_methods if pm_type == "certificate") <= 1  # H2 airline
assert order.status == "pending"  # H2 retail cancel gate
assert line.customer_id == customer_id  # H2 telecom ownership
assert _bm25_score(q, doc, idf, avgdl) >= 0.0  # BM25 non-negative
assert format_skills_block([]) == ""  # empty list -> no injection
assert HarnessAnnotator.annotate returns str or None (never raises)
assert len([ann for ann in annotations if ann]) == count_non_none_annotators
# Replay safety: rules must not set any attribute on self or db

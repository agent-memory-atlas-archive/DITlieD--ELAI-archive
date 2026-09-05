tool-attention-interface.md
Source: arXiv:2604.21816 (Tool Attention), codebase C-35 implementation
Reliability: MEDIUM (arXiv preprint, not peer-reviewed)

INTERFACE MAP - C-35: Tool Attention Dynamic Gating

CLAIM 1: Per-state tool gating reduces prompt tokens by excluding irrelevant tool schemas.
  Status: GROUNDED via C-35 implementation (prompt_tokens_saved metric in GatedCatalog).
  Algorithm: filter_for_state(fsm_state, full_catalog) -> GatedCatalog{eager_schemas, gated_summaries, denied_names}.
  Eager: full JSON schema included. Gated: name+description only (SummaryRow). Denied: excluded.

CLAIM 2: LRU schema cache amortizes repeated fetches to near-zero marginal cost.
  Status: GROUNDED via LazySchemaCache (lru crate, TokioMutex, AtomicU64 hit/miss counters).
  Protocol: resolve(tool_name, registry) -> Result<FullSchema, LazyCacheError::ToolNotFound>.
  Cache miss: insert into LRU, evict LRU entry if at cap. Cache hit: return clone.

CLAIM 3: SSB beacons (tool_gated/tool_eager/tool_lazy_resolved/tool_denied) enable observability.
  Status: GROUNDED via ssb.rs SCHEMA_VERSION 88, ToolGateEventPayload{tool_name, fsm_state, prompt_tokens_saved}.

FSM GATE MAP (tool-gate.toml): 10 states.
  idle: no tools eager. executing/auditing/memorizing: state-specific tool subsets.
  All states have safe default fallback (eager=[], gated=ALL) when manifest absent.

CLI ESCAPE HATCH: --tool-gate-off sets ELAI_TOOL_GATE_OFF=1, warns on prompt cost increase.

TYPES:
  FullSchema = McpToolSpec{name, description, input_schema}
  GatedCatalog{eager_schemas: Vec<FullSchema>, gated_summaries: Vec<SummaryRow>, denied_names: HashSet<String>}
  SummaryRow{name: String, summary: String}
  ToolGateError::ToolDenied{tool_name, fsm_state} | Io | Parse
  LazyCacheError::ToolNotFound(String)

CONFIG: elai_cc.toml [tool_gating]{enabled, gate_manifest_path, lazy_schema_default, max_eager_schemas}
Source: lm-format-enforcer
Reliability: HIGH
PyPI: lm-format-enforcer==0.11.3 (released 2024-08-24; verified via https://pypi.org/pypi/lm-format-enforcer/json)
GitHub: https://github.com/noamgat/lm-format-enforcer
Python support: >=3.8, <4.0 (classifiers confirm 3.8 through 3.13)
Transformers compat: >=4.37.0 (pyproject.toml optional deps group); v0.10.11 fixed breakage from LogitsWarper rename in newer transformers
Install: pip install lm-format-enforcer
Anti-Slopsquatting: package confirmed on PyPI; version 0.11.3 pinnable; no name collision verified

Files examined:
  https://pypi.org/pypi/lm-format-enforcer/json
  https://raw.githubusercontent.com/noamgat/lm-format-enforcer/main/lmformatenforcer/__init__.py
  https://raw.githubusercontent.com/noamgat/lm-format-enforcer/main/lmformatenforcer/integrations/transformers.py
  https://raw.githubusercontent.com/noamgat/lm-format-enforcer/main/lmformatenforcer/tokenenforcer.py
  https://raw.githubusercontent.com/noamgat/lm-format-enforcer/main/lmformatenforcer/jsonschemaparser.py
  https://raw.githubusercontent.com/noamgat/lm-format-enforcer/main/lmformatenforcer/characterlevelparser.py
  https://raw.githubusercontent.com/noamgat/lm-format-enforcer/main/pyproject.toml


Public API (from __init__.py __all__ + integrations.transformers):

  Core exports (import from lmformatenforcer):
  JsonSchemaParser | __init__(json_schema: Union[dict, _Context, None], config: Optional[CharacterLevelParserConfig]=None, existing_stack=None, num_consecutive_whitespaces: int=0) | CharacterLevelParser | None input = any valid JSON; dict input = constrained to schema; pydantic: pass model.schema()
  RegexParser | __init__(regex: str) | CharacterLevelParser | regex must be valid Python re pattern; interegular library validates it
  StringParser | __init__(string: str) | CharacterLevelParser | enforces exact string match
  UnionParser | __init__(parsers: List[CharacterLevelParser]) | CharacterLevelParser | allows any string valid under any sub-parser
  SequenceParser | __init__(parsers: List[CharacterLevelParser]) | CharacterLevelParser | enforces parsers applied sequentially
  TokenEnforcer | __init__(tokenizer_data: TokenEnforcerTokenizerData, parser: CharacterLevelParser) | TokenEnforcer | stateful cache per (sequence, parser); not thread-safe
  TokenEnforcerTokenizerData | (opaque data class, built via build_token_enforcer_tokenizer_data) | — | vocab-dependent; must rebuild if tokenizer changes
  CharacterLevelParserConfig | dataclass: alphabet=COMPLETE_ALPHABET, max_consecutive_whitespaces=DEFAULT, force_json_field_order=False, max_json_array_length=DEFAULT | — | env-var overridable
  LMFormatEnforcerException | standard exception class | — | raised on schema/format violations
  FormatEnforcerAnalyzer | conditional import, may be None if optional deps absent | — | diagnostics only

  Integration exports (import from lmformatenforcer.integrations.transformers):
  build_transformers_prefix_allowed_tokens_fn | (tokenizer_data: Union[PreTrainedTokenizerBase, TokenEnforcerTokenizerData], character_level_parser: CharacterLevelParser) -> Callable[[int, torch.Tensor], List[int]] | returns prefix_allowed_tokens_fn callable for model.generate() | primary integration point
  generate_enforced | (model: AutoModelForCausalLM, tokenizer: Union[PreTrainedTokenizerBase, TokenEnforcerTokenizerData], character_level_parser: CharacterLevelParser, **kwargs) -> Union[str, dict] | convenience wrapper around model.generate(); returns str or generate() output dict with added enforced_scores field | beam search disables diagnostics
  build_token_enforcer_tokenizer_data | (tokenizer: PreTrainedTokenizerBase, use_bitmask: bool=False, vocab_size: Optional[int]=None) -> TokenEnforcerTokenizerData | pre-computes tokenizer vocab data; call once and reuse | vocab-size override needed for models with padding tokens


HuggingFace transformers integration pattern (LogitsProcessor vs prefix_fn):

  DOES NOT use LogitsProcessor subclass as primary integration path.
  Uses prefix_allowed_tokens_fn parameter of model.generate() exclusively for constraint enforcement.
  LogitsSaverWarper (internal LogitsProcessor subclass) exists ONLY for diagnostic score collection when generate_enforced() is called with output_scores=True + return_dict_in_generate=True.
  Correct integration into existing LogitsProcessorList: do NOT add to LogitsProcessorList; pass result of build_transformers_prefix_allowed_tokens_fn() as prefix_allowed_tokens_fn= kwarg to model.generate().
  The prefix_allowed_tokens_fn callable signature: fn(batch_id: int, sent: torch.Tensor) -> List[int]
  batch_id is accepted but internally IGNORED; TokenEnforcer is shared across all batch items; constraint is sequence-content-based not batch-item-based.


Behavioral contracts:

  build_transformers_prefix_allowed_tokens_fn -> creates TokenEnforcer wrapping parser; returns stateful callable; side effect: tokenizer vocab is scanned and cached on first call if raw tokenizer passed
  generate_enforced -> calls model.generate() internally; side effects: GPU inference, KV cache allocation; returns str (decoded) or dict (if output_scores=True)
  TokenEnforcer.get_allowed_tokens(token_sequence: List[int]) -> List[int] -> reads prefix_states cache; may mutate cache; NOT thread-safe
  JsonSchemaParser.add_character(ch) -> returns NEW parser instance (immutable pattern); does NOT mutate self
  JsonSchemaParser.get_allowed_characters() -> returns set of allowed next chars given current parse state
  JsonSchemaParser.can_end() -> bool; True only when current parse state is a valid terminal state per schema
  build_token_enforcer_tokenizer_data -> expensive one-time scan of full tokenizer vocab; result is cacheable and reusable across requests


Platform constraints:

  No Windows/Linux difference; pure Python + PyTorch tensor ops
  Python >=3.8 required; 3.13 classifiers confirmed
  Transformers >=4.37.0 required for integration; LogitsWarper import break fixed in lmfe 0.10.11
  pydantic >=1.10.8 (both pydantic v1 and v2 supported via compatibility shim; verify if pydantic v2 compat holds for your schema usage)
  interegular >=0.3.2 required for RegexParser; validates regex at construction time
  TokenEnforcerTokenizerData is NOT picklable in general (contains lambda refs and tensor caches); do NOT pass across process boundaries; rebuild per-worker
  Tokenizer vocab must match model vocab exactly; mismatched vocab_size causes silent wrong-token filtering


Gotchas:

  batch_id parameter in prefix_allowed_tokens_fn is IGNORED: single TokenEnforcer shared across batch. In batched generation (batch_size > 1) with different prompts, all items share ONE constraint state derived from the concatenated token sequence — this is a BUG risk for multi-prompt batches. Safe use: batch_size=1 or all prompts identical.
  Beam search (num_beams > 1): diagnostics disabled silently; constraint enforcement still active but beam diversity may conflict with tight constraints causing generation stall.
  JsonSchemaParser(None) allows any valid JSON but does NOT enforce max_tokens mid-generation: model can produce arbitrarily long JSON up to model context limit.
  Schema validation errors surface during character parsing, not at JsonSchemaParser construction; bad schemas fail mid-generation not at startup.
  force_json_field_order=False by default: model may emit fields in any order. Set True only if downstream parser requires ordered fields. Setting True significantly restricts token choices and may degrade generation quality.
  max_json_array_length default limits unbounded arrays; if schema has no maxItems, this hidden cap applies silently.
  LogitsWarper was renamed/removed in transformers 4.45+; lmfe 0.10.11+ required for compatibility.
  The library is classified "Development Status :: 3 - Alpha" on PyPI despite production use in vLLM.
  No explicit timeout or max_tokens enforcement per JSON field; a stuck generation (model keeps adding whitespace) can hit max_consecutive_whitespaces limit and then raise LMFormatEnforcerException.
  Thread-safety: NONE. prefix_states dict in TokenEnforcer is mutated without locks. One TokenEnforcer instance per concurrent request required.


Comparison: lm-format-enforcer vs outlines (for HF transformers + JSON-schema constrained decoding):

  Feature                        | lm-format-enforcer 0.11.3          | outlines (latest)
  Integration shape              | prefix_allowed_tokens_fn=           | LogitsProcessor in LogitsProcessorList
  Schema source                  | pydantic.BaseModel.schema(), dict, None (any JSON), regex | pydantic, JSON schema, regex, CFG
  Pydantic v2 support            | v1 + v2 compat shim                 | native v2
  max_tokens mid-JSON            | NOT enforced; can stall at context  | NOT enforced natively; same issue
  Whitespace/field order control | flexible (model controls)           | strict (enforces exact schema order)
  Vocab pre-computation          | lazy / per-step dynamic             | FSM compiled at schema load (one-time cost upfront)
  Per-step overhead              | higher (dynamic exploration)        | lower (precomputed FSM transitions)
  Beam search support            | yes (diagnostics disabled)          | yes
  Thread safety                  | NOT safe (shared TokenEnforcer)     | NOT safe without external locking
  Batch safety                   | UNSAFE for multi-prompt batches     | safer (per-sequence FSM state)
  Active vLLM integration        | yes (--guided-decoding-backend lm-format-enforcer) | yes (default in vLLM)
  Performance vs outlines        | slower per benchmark               | faster (FSM precomputation amortizes)
  Best use case                  | dynamic/RAG schemas, field-optional | static schemas, speed-critical paths


Critical assertions (testable):

  assert callable(build_transformers_prefix_allowed_tokens_fn(tokenizer, JsonSchemaParser({})))
  assert isinstance(JsonSchemaParser(None), CharacterLevelParser)
  assert isinstance(JsonSchemaParser({"type": "object"}), CharacterLevelParser)
  parser = JsonSchemaParser({"type": "string"}); next_parser = parser.add_character('"'); assert next_parser is not parser  # immutability
  assert build_token_enforcer_tokenizer_data(tokenizer) is not None  # vocab scan succeeds
  enforcer = TokenEnforcer(build_token_enforcer_tokenizer_data(tokenizer), JsonSchemaParser(None)); tokens = enforcer.get_allowed_tokens([]); assert isinstance(tokens, list) and len(tokens) > 0
  assert RegexParser(r"\d+") is not None  # valid regex accepted
  # Thread safety violation: do NOT share TokenEnforcer across threads; each request must construct its own


Sources (verified):
  PyPI metadata: https://pypi.org/pypi/lm-format-enforcer/json
  GitHub repo: https://github.com/noamgat/lm-format-enforcer
  transformers.py source: https://raw.githubusercontent.com/noamgat/lm-format-enforcer/main/lmformatenforcer/integrations/transformers.py
  tokenenforcer.py source: https://raw.githubusercontent.com/noamgat/lm-format-enforcer/main/lmformatenforcer/tokenenforcer.py
  jsonschemaparser.py source: https://raw.githubusercontent.com/noamgat/lm-format-enforcer/main/lmformatenforcer/jsonschemaparser.py
  characterlevelparser.py source: https://raw.githubusercontent.com/noamgat/lm-format-enforcer/main/lmformatenforcer/characterlevelparser.py
  pyproject.toml: https://raw.githubusercontent.com/noamgat/lm-format-enforcer/main/pyproject.toml

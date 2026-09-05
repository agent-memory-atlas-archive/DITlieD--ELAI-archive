repomap-syncode.md
Repo map + method summary: SynCode (grammar-augmented LLM generation)
Source repo: C:\Users\user\Downloads\new\syncode-main  (Python). Paper: arXiv:2403.01632.
Read selectively per instructions; eval datasets (sql_spider_eval, mxeval/data, *.jsonl) NOT read.
Mapper: s-repomap-syncode, 2026-06-15. Plain text. Grounding = code read in this repo unless flagged.

================================================================================
1. WHAT IT IS
================================================================================
SynCode is a framework that constrains a HuggingFace LLM's token sampling so that
the generated text is GUARANTEED syntactically valid w.r.t. a Context-Free Grammar
(CFG) given in EBNF (Lark dialect). It is a constrained-decoding / grammar-guided
decoding system. Problem solved: an LLM left unconstrained emits syntactically
invalid JSON / Python / SQL / Go / custom DSLs (bad indentation, prose instead of
JSON, unbalanced brackets). SynCode masks the next-token distribution every step to
only the tokens that can extend a syntactically-valid prefix.

Key differentiators it claims over peers (Outlines, Guidance, llama.cpp grammars,
PICARD, SYNCHROMESH, GCD): (a) full CFG support, not just regex; (b) PRE-COMPUTED
masks (an offline DFA mask store over the whole vocab) so per-step masking is a
lookup, not a live regex walk; (c) handles GENERAL-PURPOSE programming languages
whose real syntax is NOT context-free at the lexer level - Python indentation
(INDENT/DEDENT) and Go end-of-scope/semicolon insertion - via custom incremental
parsers + an indentation-constraint mask. It plugs in as a standard HF
`LogitsProcessor`, so it works with any decoding strategy (greedy, beam, nucleus).

Two operating modes (grammar_constrainer.py docstring, lines 19-44):
  - grammar_mask  = OVERAPPROXIMATION (conservative): permits a wider token set,
    preserves more of the LLM distribution, may let through a token that a deeper
    parse would reject. Soundness is "no definitely-invalid token".
  - grammar_strict = UNDERAPPROXIMATION: tighter, can split LLM tokens that were
    fine as a whole; more invasive but stricter guarantee.

================================================================================
2. STRUCTURE MAP (one-line roles; SynCode-specific modules only)
================================================================================
syncode/__init__.py            Public API exports: Syncode, SyncodeLogitsProcessor.
syncode/infer.py               CLI / Syncode wrapper entrypoint (the high-level object).
syncode/language_model.py      HF model load + generate() driver (NOT read in depth).
syncode/common.py              Cache dirs (SYNCODE_CACHE, HF_CACHE), helpers.

grammar_mask/  (the LLM hook layer)
  logits_processor.py          SyncodeLogitsProcessor(LogitsProcessor): thin shim;
                               __call__ -> GrammarConstrainer.mask_scores. Public API.
  grammar_constrainer.py       CORE ENGINE. Orchestrates: decode ids->bytes->str,
                               run incremental parser, get accept mask from store,
                               masked_fill scores to -inf. Owns mode, byte remainder,
                               EOS-gating, function-end heuristic, parse-fail fallback.

mask_store/  (the offline precomputed mask store - the SynCode "backbone")
  mask_store.py                MaskStore: builds + queries the DFA mask store. Offline
                               build = for every (terminal-DFA-state x next-terminal x
                               vocab-token) decide membership; cached to pickle.
                               Query = get_accept_mask(ParseResult)->bool vocab tensor.
  fsm_set.py                   FSMSet + JointFSMState(terminal,state_id): one byte-DFA
                               per terminal regex; consume_prefix / compute_fsm_states /
                               is_final. Deterministic 32-bit hash for JointFSMState.
  byte_fsm.py                  ByteFSM: char-level regex DFA (via interegular) lowered
                               to a BYTE-level DFA (multi-byte UTF-8 -> intermediate
                               states); consume_prefix returns (success, remainder),
                               islive() = can-reach-final BFS (cached).
  lookup_table.py              LookupTable: the actual storage. Maps (JointFSMState,
                               next_terminal)->bool vocab mask; _exact_lookup (COMPLETE
                               case), _overapprox_lookup (INCOMPLETE), indentation maps.
                               Lists-of-token-ids built during build, then converted to
                               packed bool tensors once at end.
  byte_tokenizer.py            ByteTokenizer: normalizes ANY HF tokenizer to a clean
                               byte<->id map. Detects vocab type RAW / BYTE_FALLBACK
                               (Llama-2/Gemma <0x1B>, '_') / BYTE_LEVEL (GPT-2/Llama-3
                               'G'). Greedy byte re-encode. THIS is the portability hinge.

parsers/  (the incremental CFG parser layer)
  incremental_parser.py        IncrementalParser base: Lark LALR(1)/LR(1) interactive
                               parser driven token-by-token; STATE CACHE keyed by hash
                               of lexer-token prefix for incremental reuse;
                               get_acceptable_next_terminals(partial_code)->ParseResult;
                               computes remainder + remainder_state + accept_sequences.
  python_parser.py             PythonIncrementalParser + PythonIndenter: INDENT/DEDENT
                               postlexing, dedent queue, per-line indent-level tracking,
                               IndentationConstraint computation. GPL non-CF handling.
  go_parser.py                 GoIncrementalParser: Go automatic-semicolon / scope (not read).
  itergen_parser.py            IGParser variant w/ symbol position map (not read).
  __init__.py                  create_parser() factory (lalr/lr; python/go/generic) +
                               create_base_parser() (wraps Lark with start='start',
                               lexer='basic', postlex=indenter, cache).
  grammars/grammar.py          Grammar object: loads EBNF (string/file/builtin name),
                               .hash() (cache key), .simplifications(), .ebnf.
  grammars/*.lark              Builtin grammars: python, go, sql, json, java, calc, FOL.

parse_result.py                Value types: AcceptSequence(list of terminals, len 1-3),
                               RemainderState enum {COMPLETE, MAYBE_COMPLETE, INCOMPLETE},
                               ParseResult (remainder bytes + state + accept_sequences +
                               indent constraint + function_end), IndentationConstraint.

larkm/                         VENDORED FORK of the Lark parser library (LALR/Earley/CYK,
                               lexer, indenter, grammar loader). Mostly upstream Lark;
                               SynCode patches the interactive parser + indenter. Treated
                               as a dependency, not SynCode's contribution. Not detailed.

evaluation/, utils/sql_spider_eval/  Benchmark harnesses + datasets. IGNORED per scope.

================================================================================
3. ARCHITECTURE + METHOD (detailed)
================================================================================
Per-step loop (every generated token), in GrammarConstrainer.mask_scores
(grammar_constrainer.py:151):
  STEP A. ids -> bytes -> (utf8 string, remainder_bytes).
    _get_partial_outputs uses ByteTokenizer.decode on input_ids[start_from:] (only the
    GENERATED suffix when parse_output_only, so the prompt is never parsed). Bytes are
    split into the longest valid UTF-8 prefix (parsed as a Python str) + a trailing
    remainder of bytes that do not yet form a complete codepoint (_bytes_to_string,
    :264). This is how SynCode survives mid-multibyte-character token boundaries.
  STEP B. Incremental parse of the partial string ->ParseResult
    (inc_parser.get_acceptable_next_terminals, :206). Any leftover remainder_bytes force
    remainder_state=INCOMPLETE and are appended to the remainder.
  STEP C. accept_mask = dfa_mask_store.get_accept_mask(ParseResult)  (a bool vocab tensor).
  STEP D. scores.masked_fill(~accept_mask, -inf). accept_mask is padded/truncated to
    score width (handles vocab_size != logits width). If the mask is ALL-ZERO (no legal
    token) it masks NOTHING and logs - i.e. it fails OPEN rather than dead-ending (:186).
  Parse-fail fallback: if the incremental parser throws, the FIRST failure flips
    self.parse_failed and SynCode falls back to UNCONSTRAINED decoding for the rest
    (:221) unless dev_mode (then it raises). Important soundness caveat.

--- THE INCREMENTAL PARSER (incremental_parser.py) ---
Drives Lark's LALR(1) (or LR(1)) parser in "interactive" mode token by token.
  - _lex_code: lexes the partial code; the LAST terminal is typically incomplete, so
    lexing/parsing of the tail is expected to fail and is caught.
  - INCREMENTAL REUSE: after feeding each lexer token it stores the full parser state
    (parsed tokens, parser_state.copy(), cur/next acceptable terminals, indent levels,
    dedent queue) in a dict keyed by hash(tuple of lexer tokens up to pos)
    (_store_parser_state :52). On the next call _restore_recent_parser_state (:109) walks
    backward from the longest lexer-token prefix to find the deepest cached state and
    resumes from there - so only the NEW tail is re-parsed, not the whole generation.
    (Authors flag the prefix-hash as suboptimal; a sliding-window hash is a TODO.)
  - accept_sequences (parse_result.py:48): the set of terminal sequences (length 1, 2,
    or 3) that may legally follow. Built from cur_accept_terminals (terminals the
    in-progress remainder could still be) x next_accept_terminals (what the parser will
    accept after this terminal closes), plus ignore-terminal (whitespace/comment) hops
    that allow length-3 sequences [final, ignore, next].
  - remainder + remainder_state classify the trailing partial terminal:
      COMPLETE        last terminal fully closed, parser knows next terminal type.
      INCOMPLETE      trailing bytes match no terminal regex yet (mid-token).
      MAYBE_COMPLETE  trailing bytes match a terminal BUT could be a prefix of a longer
                      one ('de' is a NAME but could become 'def' keyword) - the crux
                      case that requires overapproximation. (mask_store.py:21-31.)

--- THE OFFLINE DFA MASK STORE (mask_store.py) ---
Built ONCE per (tokenizer, grammar, mode) and pickled to
SYNCODE_CACHE/mask_stores/<TokenizerClass>/<mode>_<grammarhash>_<vocabsize>.pkl
(init_mask_store :89). Build can take >10 min for a big grammar+vocab; thereafter a
deserialize. Build algorithm (_store_token_masks :179):
  for each terminal's byte-DFA STATE (JointFSMState):
    for each token in the FULL vocab:
      decode token to bytes; tabs ->4 spaces; optionally lstrip leading whitespace if
      grammar ignores whitespace (_remove_left_whitespace :263, an efficiency win so the
      model isn't forced to emit indentation as its own token).
      COMPLETE case (_process_complete_case :255): if consuming the token from this state
        lands exactly on a final state with empty remainder -> add to _exact_lookup.
      INCOMPLETE/continuation case (_process_regular_tokens :209):
        consume_prefix the token from this state.
          if it stays inside the DFA (live, no remainder): token valid for ALL next
            terminals from here -> add (state,next_terminal) for each next_terminal.
          if it finishes this terminal and spills a remainder: re-consume the remainder
            from the INITIAL state of each candidate next_terminal (cached per
            (init_state,remainder) in _consume_prefix_cache):
              mode grammar_mask : add if the remainder is a live prefix (overapprox).
              mode grammar_strict: add only if remainder fully consumed to a live state.
  EOS handling: EOS token is added only to states that are FINAL, under a synthetic
    '$END' next-terminal (:196) - so EOS is legal only at a grammatically complete point.
PRUNING with the LR PARSE TABLE (_compute_following_terminals_map :138): instead of
  the full O(terminals^2) cross product of "any terminal can follow any terminal", it
  reads Lark's LALR action table - for each terminal it finds SHIFT actions and the
  terminals legal in the resulting parser state - cutting length-2 accept-sequence pairs
  drastically (logged "reduced from N^2 to M"). This is the structural speedup that makes
  the offline build tractable.
STORAGE FORMAT (lookup_table.py): during build, masks are appended as lists of token
  ids; convert_lookups_from_list_to_mask (:88) packs each into a torch bool tensor of
  vocab width ONCE at the end (also folds every (state,next_terminal) mask into a
  per-state _overapprox_lookup via OR). Query is then pure bitwise-OR of precomputed
  bool tensors. Default mask seeds all special tokens (except EOS) as allowed (:32).

--- QUERY (mask_store.py:315 _lookup_next_tokens / :369 get_accept_mask) ---
  1. compute_fsm_states(remainder_bytes): for every terminal byte-DFA, replay the
     remainder bytes; keep the terminals whose DFA can consume the entire remainder ->
     the set of current JointFSMStates (which partial terminals the remainder could be).
  2. for each (fsm_state x accept_sequence) OR together the right precomputed mask:
       COMPLETE        -> _exact_lookup[state].
       INCOMPLETE      -> _overapprox_lookup[state] (overapprox continuation).
       MAYBE_COMPLETE  -> len1: overapprox lookup; len2: (state,seq[1]) mask; len3:
                          if state final, jump to init state of the ignore-terminal and
                          take (init, seq[2]).
       '$END' in seq   -> allow EOS.
  3. if indentation on: AND with the indentation mask for the IndentationConstraint.
  Result is a single bool vocab tensor = union of all syntactically-valid next tokens.

--- GPL NON-CONTEXT-FREE HANDLING (python_parser.py) ---
Python is not CF at lexer level (INDENT/DEDENT). PythonIndenter postlexes _NL tokens
into _INDENT/_DEDENT; PythonIncrementalParser tracks an indent_level stack and a
dedent_queue (dedents are deferred until real code appears on the next line). After a
_NL it computes an IndentationConstraint (parse_result.py:93): either accept_indents
(a discrete set of legal indentation widths = current open blocks) or
greater_than_indent_val (must indent deeper, e.g. right after 'if x:'). LookupTable
precomputes _indentation_to_tokens_map (token -> indent width in spaces, tabs=4) and
_whitespace_tokens_map (all-whitespace tokens); get_indentation_tokens turns a
constraint into a mask that is ANDed into the accept mask (lookup_table.py:153). This is
the mechanism that makes generated Python indentation-error-free. Go scope/semicolon
handled analogously in go_parser.py.

--- BYTE-LEVEL ROBUSTNESS (byte_tokenizer.py + byte_fsm.py) ---
Everything downstream of the LLM is BYTES, not characters. ByteTokenizer auto-detects
the tokenizer family (RAW tiktoken / BYTE_FALLBACK Llama-2,Gemma / BYTE_LEVEL
GPT-2,Llama-3) and builds a uniform id<->bytes map, so a token like ' apple' ('_apple',
or 'Gapple', or <0x..>) reduces to b' apple'. ByteFSM lowers each terminal regex to a
byte DFA, inserting intermediate states for multi-byte UTF-8 chars, so the DFA can be
walked one raw byte at a time and a token that ends mid-codepoint is handled by carrying
remainder_bytes (see STEP A). This is what lets the same grammar machinery run against
arbitrary HF tokenizers without per-model special-casing.

--- OPTIONAL OPTIMIZATIONS in the Syncode wrapper ---
last_valid_state / function_ends (grammar_constrainer.py:67,248): track the offset of
the last grammatically-complete point and of function ends, a heuristic to backtrack /
truncate output at a clean boundary (function-end cutoff for code-completion). is_valid()
(:110) is a single-token validity check usable as an external sampler guard.

================================================================================
4. KEY TECHNIQUES PORTABLE TO ELAI
================================================================================
ELAI context: Rust orchestrator + local sidecar serving Qwen2.5-Coder (NF4) via an
Anthropic-protocol HTTP server (python/elai_inference). Interest = structured/grounded
output: JSON / tool-call schema enforcement, syntactically-valid code generation. SynCode
is Python+HF-`generate`+`LogitsProcessor`; ELAI's sidecar already uses
model.generate()+LogitsProcessor per house rules - so the integration seam is natural on
the LOCAL lane only. Cloud lane (Anthropic API) cannot accept a logits processor; for
cloud, the only portable idea is the grammar->JSON-schema/tool-schema mapping at the
prompt/validation layer, not masking.

T1. OFFLINE PRECOMPUTED MASK STORE keyed by (tokenizer, grammar, mode).
    Files: mask_store/mask_store.py (init_mask_store, _store_token_masks),
           lookup_table.py (packed bool-tensor storage).
    Maps to ELAI: build a per-(Qwen-tokenizer, grammar) DFA mask store once, pickle to
    SYNCODE_CACHE; the sidecar's LogitsProcessor then does an O(1) tensor OR per step.
    This is the technique that delivers grammar-guided decoding with ~zero per-step regex
    cost (the basis of their "10-20% FASTER than unconstrained" claim - the model emits
    fewer wasted/backtracked tokens). Directly portable: Qwen2.5 is a BYTE_LEVEL BPE
    tokenizer, exactly the family ByteTokenizer.detect_vocab_type handles. Non-portable:
    the >10-min one-time build per grammar; cache it in the model image / first-boot.

T2. BYTE-LEVEL TOKENIZER ABSTRACTION + byte-DFA + UTF-8 remainder carry.
    Files: mask_store/byte_tokenizer.py (VocabType detect + uniform byte map),
           byte_fsm.py (char-regex -> byte DFA), grammar_constrainer._bytes_to_string.
    Maps to ELAI: this is the single most reusable, lowest-risk steal. It solves
    "constrain output of an arbitrary HF BPE tokenizer at the byte level" - the exact
    problem any local-sidecar structured-output feature hits with Qwen. Even if ELAI does
    not adopt the full CFG engine, the byte-level token<->bytes normalization + the
    mid-multibyte remainder handling are reusable for ANY constrained-decoding scheme
    (incl. a simple JSON-only mask). Fully portable, tokenizer-agnostic.

T3. LR-PARSE-TABLE-PRUNED ACCEPT SEQUENCES (incremental, prefix-cached parser).
    Files: parsers/incremental_parser.py (state cache keyed by lexer-token-prefix hash),
           mask_store._compute_following_terminals_map (uses Lark's LALR action table to
           cut terminal^2 -> structurally-legal follow pairs).
    Maps to ELAI: the incremental-parse + state-cache pattern is how you make per-step
    grammar masking cheap on a long generation (re-parse only the tail). Portable as an
    algorithm if ELAI wants real CFG (Python/SQL) guidance; heavier lift than T1/T2
    because it pulls in the vendored Lark fork (larkm/) and its LALR interactive parser.
    For pure JSON/tool-call schema enforcement this is overkill - a regex/JSON-schema DFA
    is enough (T1+T2 alone). Reserve T3 for full-language code-syntax guidance.

T4. GPL non-CF indentation constraint (Python INDENT/DEDENT mask).
    Files: parsers/python_parser.py, lookup_table.get_indentation_tokens.
    Maps to ELAI: if ELAI ever wants to guarantee indentation-correct Python from the
    local model, this is the prior art. Niche; depends on T3. Flag: Python-specific and
    tightly coupled to the indenter; not a general win.

T5. FAIL-OPEN + EOS-GATING decoding doctrine (design pattern, not code to lift).
    Files: grammar_constrainer.mask_scores (all-zero mask -> mask nothing; first parse
    failure -> fall back to unconstrained), EOS allowed only at grammar-final states.
    Maps to ELAI: note the SOUNDNESS CAVEAT for grounded-by-default - SynCode is sound
    only while the parser keeps up; on parse failure it SILENTLY drops the guarantee and
    decodes unconstrained. ELAI's wiring/grounding ethos would want this surfaced as an
    observable event (SSB row) not a silent log. The EOS-gating idea (don't let the model
    stop until the grammar is satisfiable-complete) is a clean, directly portable rule
    for any structured-output feature - it kills truncated-JSON / half-written-function
    failure modes. Adopt the pattern; make the fallback LOUD per ELAI rules.

NON-PORTABLE / FRICTION:
  - Whole stack is Python + transformers `generate()`. Cloud (Anthropic) lane gets none
    of the masking; only grammar->schema prompt/validation transfers there.
  - Vendored Lark fork (larkm/) is a real dependency surface (anti-slopsquat: it is an
    in-tree fork, not a PyPI pin - verify before reuse).
  - Mask store cache key uses tokenizer vocab_size as a proxy (mask_store.py:104 TODO):
    two different fine-tunes with same vocab size collide. Fix the cache key (hash the
    actual vocab) before trusting it across Qwen variants.
  - Batch/beam: is_valid asserts single input; batched masking present but the function
    -end heuristic is per-sequence and fragile.

================================================================================
5. BENCHMARK NUMBERS (grounding only; from README / paper claims, not re-run)
================================================================================
  - "99% accuracy in JSON generation with Gemma-2b" and "10-20% FASTER than standard
    unconstrained generation" (README About, lines 24-26; links notebooks/eval_json.ipynb).
  - "as little as 10% generation overhead with Python and Go" (README Features, line 44).
  - Comparison table (README): SynCode claims the only tool with all of {Regex, full CFG,
    Pre-Computed masks, GPL support} vs LMQL/Guidance/Outlines/PICARD/SYNCHROMESH/
    llama.cpp/GCD (each missing >=1; only Outlines also has Pre-Computed but lacks GPL).
  - Qualitative: Phi-2 unconstrained emits Python code instead of JSON; WizardCoder-1B
    unconstrained produces IndentationError - both fixed under grammar_strict (README
    examples). No per-task pass@k tables surfaced without reading the eval dirs (skipped).
  Source caveat: numbers are author-reported in the repo README; not independently
  measured here. Paper arXiv:2403.01632 is the primary source for the full eval.

================================================================================
6. DEPENDENCIES / STACK
================================================================================
Python (3.6-3.12; 3.13 unsupported per README). Core deps (requirements.txt /
imports seen): torch, transformers (pinned v4.53.2 for SynCode v0.4.16),
interegular (regex -> FSM, the byte-DFA backbone), regex (PyPI `regex`, not stdlib re),
tqdm. Vendored: larkm/ (a fork of the Lark parser - LALR/Earley/CYK + lexer + indenter).
Optional: accelerate/bitsandbytes-style quantization implied by `quantize=True`
(bfloat16). Caches: HF_CACHE, SYNCODE_CACHE env vars; gated models via HF_ACCESS_TOKEN.
Entry points: `from syncode import Syncode` (high-level infer object, extra
optimizations) or `from syncode import SyncodeLogitsProcessor` (drop into HF
generate(logits_processor=[...])). License: see repo LICENSE (not transcribed).

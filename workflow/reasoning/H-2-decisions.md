H-2 DECISION DOC — cache-stable layout + breakpoints + 1h TTL
session s-66424 | 2026-06-03 | orchestrate H-plan WAVE 1 | depends-on H-1 (landed)

WHAT
Activate the doubly-inert cache-stable-layout mechanism: bp.valid was consumed only by an eprintln!,
update_breakpoints was fed EMPTY slices for slots 2-7 and reachable only under BudgetPolicy::Local, and the
cloud marker path (apply_anthropic_cache_markers) never consulted byte-stability. Add 1h TTL (GA, NO beta
header). Make a cache-bust a test failure (HD-2).

GROUNDING (all GROUNDED, dual-confirmed live docs 2026-06-03 via spec-grounder)
1h TTL is GA via {"type":"ephemeral","ttl":"1h"} — NO beta header (extended-cache-ttl-2025-04-11 REMOVED).
Max 4 breakpoints/request. 20-block lookback. Invalidation tools->system->messages. 1h write=2.0x (breakeven
2nd read); 5-min write=1.25x (breakeven 1st read). Min cacheable prefix per EXACT model id (Opus 4.8=1024,
Opus 4.7/4.6/4.5+Haiku 4.5=4096, Haiku 3.5=2048). Longer-TTL markers MUST precede shorter-TTL in block order.

KEY DECISIONS
D1 CacheControl struct extension (providers/types.rs): add `#[serde(skip_serializing_if="Option::is_none")]
   pub ttl: Option<String>`; ephemeral() sets ttl:None (existing markers stay byte-identical => HD-2 + the
   anthropic.rs wire contract preserved); new ephemeral_1h() sets ttl:Some("1h"). Only 3 construction sites,
   all via ::ephemeral() — no raw literals to fix.
D2 NO 3rd CacheBreakpoint (context_budget.rs CACHE_BREAKPOINT_COUNT stays 2 — const-asserted in
   context_budget_contract.rs:21 + many index tests; bumping it would break them). The TAIL breakpoint is a
   MARKER on the last markable InputContentBlock (Text|ToolUse) within a 20-block lookback window, placed in
   apply_anthropic_cache_markers — NOT a ContextBudget breakpoint. ToolResult has no cache_control field, so
   the tail walks back to the last Text/ToolUse block.
D3 Marker policy (satisfies the longer-before-shorter ordering): stable prefix (last tool + last system) at
   1h; recent-conversation tail at 5-min. Order tools(1h) -> system(1h) -> messages-tail(5m) = longer-before-
   shorter, OK; <=4 markers (3 placed). apply_anthropic_cache_markers mirrors the HEAD system block's TTL onto
   the last tool so both stable-prefix blocks share a TTL (ordering safe). is_none-guarded so it respects a
   caller-set 1h head (runtime_client) and never double-marks.
D4 bp.valid made NON-INERT via cross-turn state: RuntimeClient gains `last_cache_prefix_hashes:
   std::sync::Mutex<[u64;2]>`. send() seeds a ContextBudget's breakpoint prefix_hash from the persisted last-turn
   hashes, calls update_breakpoints with REAL slot bytes (system+rules+phase+code, NOT empty), reads bp0.valid
   (= byte-identical-to-last-turn), stores the new hashes. This runs for ALL policies (hoisted out of the
   Local-only arm) => W-3 cloud-branch reachability. std::sync::Mutex (brief sync lock, outside any .await).
D5 bp.valid CONSUMED to gate the 1h TTL on the head marker (the cost-correct decision): runtime_client sets the
   system block's cache_control = ephemeral_1h() when (bp0.valid AND system_tokens >= min_cacheable_prefix(model)).
   1h costs 2x on write and only pays off on re-read, so paying 1h ONLY on a verified-byte-stable prefix is
   exactly right; when not yet stable (turn 1 or changed), the provider's default 5-min marker still writes the
   cache (needed for the next-turn read). CORRECTION of the plan's literal stub_ban "no marker when bp.valid
   false": that is wrong for turn-1 (must WRITE to later READ) — the real hazard the stub_ban targets is caching
   VOLATILE content, which we avoid (tail stays 5-min; message content is never 1h'd). So we 1h ONLY the
   structurally + byte-verified stable prefix.
D6 model-id keying (T6): the EXACT model id bytes are prepended to the system slot bytes fed to
   update_breakpoints, so a model swap busts bp.valid (no update_breakpoints signature change => existing
   context_budget tests unaffected). min_cacheable_prefix(model)->u32 helper (named consts) added in anthropic.rs.
D7 HD-2 debug_assert (T5): in runtime_client after update_breakpoints — `debug_assert!(!bp0.valid ||
   prefix_hash != 0, ...)` (a valid breakpoint must carry a non-zero stable hash; cache-bust guard). The DEEP
   byte-stability (identical content -> valid, perturbed -> invalid) is contract-tested deterministically.
D8 NO schema bump (W-9: H-2 adds no action_type). SCHEMA stays 209/486.

DONE-GATE (Tier-1; live multi-turn cloud run = deferred Tier-2, sidecar-gated)
crates/contracts/tests/h2_cache_stable_layout_contract.rs: (a) ephemeral_1h() serializes {"type":"ephemeral",
"ttl":"1h"}; (b) ephemeral() serializes {"type":"ephemeral"} with NO ttl key (byte-identical, skip_serializing_if);
(c) update_breakpoints byte-stability: identical content -> bp valid, one perturbed byte -> invalid (HD-2,
persisted across 2 calls on one budget); (d) min_cacheable_prefix(model) returns 1024/4096/2048 for the right
models; (e) apply_anthropic_cache_markers places <=4 markers, head+tool share TTL, tail is 5-min; (f) source
grep: zero "extended-cache-ttl" occurrences. cargo check --workspace green + auditor PASS on
-p elai-orchestrator -p elai-cli -p elai-providers + the contract. Test EXECUTION + live run DEFERRED.

RISK / FAILURE MODES
R1 RuntimeClient gains a field -> the ONE construction site (runtime_builder.rs:286) must init it; missed =
   compile error (auditor catches).
R2 ContextBudget::compute may Err on extreme slot sizes -> handled: Err => stable=false => 5-min fallback (safe).
R3 longer-before-shorter ordering: enforced by mirroring head TTL onto the tool + keeping the tail 5-min.
R4 byte-stability must cover the WHOLE cached prefix: the slot bytes fed are system+rules+phase+code (the
   prefix slots); volatile (SsbEntries/GrcHistory) are NOT in the hashed prefix and stay after the last marker.

DEFERRED (Tier-2)
- Live multi-turn cloud run proving a rising cache-read fraction (sidecar-gated).
- Test execution (cargo test not permitted this session).

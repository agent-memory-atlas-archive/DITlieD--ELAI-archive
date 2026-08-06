//! J-82 deterministic token emission: measure-first, default-OFF legs (HD-2/IC-3).
pub mod format;
pub mod log_dict;
pub mod protected_span;
pub use format::{emit_token_efficient, estimate_tokens, maybe_compact_tool_output, parse_token_efficient};
pub use log_dict::{collapse_log_stream, looks_like_code, LogDictReject, VolatileArchive};
pub use protected_span::{strip_prose_fillers, structured_spans_preserved};

pub fn format_emit_enabled() -> bool { std::env::var("ELAI_TOKEN_EMIT_FORMAT").ok().as_deref() == Some("1") }
pub fn log_dict_enabled() -> bool { std::env::var("ELAI_TOKEN_EMIT_LOG_DICT").ok().as_deref() == Some("1") }
pub fn protected_span_enabled() -> bool { std::env::var("ELAI_TOKEN_EMIT_PROTECTED_SPAN").ok().as_deref() == Some("1") }
#[derive(Debug, Clone, serde::Serialize)]
pub struct ToolOutputBenchReport { pub seed:u64, pub baseline_tokens:u32, pub format_tokens:u32, pub log_dict_tokens:Option<u32>, pub protected_span_tokens:u32, pub format_delta:i32, pub protected_span_delta:i32, pub format_roundtrip_ok:bool, pub structured_spans_ok:bool, pub measure_first_guard:String, pub legs_default_off:bool }
pub fn build_tool_output_bench_report(raw:&str, seed:u64)->ToolOutputBenchReport {
 let formatted=maybe_compact_tool_output(raw); let protected=strip_prose_fillers(raw); let format_roundtrip_ok=parse_token_efficient(&formatted).is_ok(); let structured_spans_ok=structured_spans_preserved(raw, &protected); let mut archive=VolatileArchive{entries:Vec::new()};
 let collapsed=collapse_log_stream(raw,Some(&mut archive)).ok(); let base=estimate_tokens(raw); let ft=estimate_tokens(&formatted); let pt=estimate_tokens(&protected);
 ToolOutputBenchReport { seed, baseline_tokens:base, format_tokens:ft, log_dict_tokens:collapsed.as_deref().map(estimate_tokens), protected_span_tokens:pt, format_delta:ft as i32-base as i32, protected_span_delta:pt as i32-base as i32, format_roundtrip_ok, structured_spans_ok, measure_first_guard:"MEASURE-FIRST: deterministic legs measured before optional emission".to_string(), legs_default_off:!format_emit_enabled()&&!log_dict_enabled()&&!protected_span_enabled() }
}

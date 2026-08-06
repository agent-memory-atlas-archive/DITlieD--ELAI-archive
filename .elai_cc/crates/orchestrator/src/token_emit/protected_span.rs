//! Byte-preserving structured-span protection and conservative prose filler removal.
fn is_stop(word: &str) -> bool { matches!(word, "the"|"a"|"an"|"of"|"to"|"in"|"and"|"or"|"is"|"are"|"was"|"were"|"be"|"been"|"being"|"that"|"this"|"with"|"for"|"on"|"at"|"by"|"from"|"as"|"it") }
fn structured_token(t: &str) -> bool {
 t.starts_with("http://") || t.starts_with("https://") || t.starts_with("./") || t.starts_with('/') || t.contains('/') || t.starts_with('`') || t.ends_with('`') || t.contains('.') && t.chars().any(|c| c.is_ascii_digit()) || t.chars().any(|c| c.is_ascii_digit()) || t.contains('_')
}
pub fn strip_prose_fillers(text: &str) -> String {
 let mut out=String::new(); let mut in_fence=false; let mut first=true;
 for line in text.split_inclusive('\n') {
  if line.trim_start().starts_with("```") { in_fence=!in_fence; out.push_str(line); first=false; continue; }
  if in_fence { out.push_str(line); first=false; continue; }
  let newline=line.ends_with('\n'); let body=line.trim_end_matches('\n');
  for (i,w) in body.split_whitespace().enumerate() {
   if is_stop(w.trim_matches(|c:char| c.is_ascii_punctuation())) && !structured_token(w) { continue; }
   if !first && i>0 { out.push(' '); } out.push_str(w); first=false;
  }
  if newline { out.push('\n'); first=true; }
 }
 out
}
pub fn structured_spans_preserved(original: &str, stripped: &str) -> bool {
 let mut protected=Vec::new(); let mut rest=original;
 while let Some(start)=rest.find("```") { if let Some(end)=rest[start+3..].find("```") { let e=start+end+6; protected.push(&rest[start..e]); rest=&rest[e..]; } else { break; } }
 for w in original.split_whitespace() { let t=w.trim_matches(|c:char| "()[]{}<>,;\"'".contains(c)); if structured_token(t) { protected.push(t); } }
 protected.into_iter().all(|s| stripped.contains(s))
}
#[cfg(test)]
mod tests { use super::*;
 #[test] fn preserves_structured_and_drops_fillers() { let x="the crate::foo/bar uses a /tmp/x and `id`"; let y=strip_prose_fillers(x); assert!(!y.contains("the ")); assert!(structured_spans_preserved(x,&y)); }
 #[test] fn fence_untouched() { let x="the\n```\nthe a\n```"; assert_eq!(strip_prose_fillers(x),"\n```\nthe a\n```"); }
}

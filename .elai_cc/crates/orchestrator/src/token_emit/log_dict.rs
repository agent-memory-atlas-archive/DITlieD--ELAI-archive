//! Conservative repetition collapsing for textual log streams.
#[derive(Debug, Clone)]
pub struct VolatileArchive { pub entries: Vec<(String, String)> }
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogDictReject { NotLogStream, CodeStream, VolatilesNotArchived }

pub fn looks_like_code(text: &str) -> bool {
    if text.contains("```") { return true; }
    let lines = text.lines().count().max(1) as f32;
    let marks = text.matches('{').count() + text.matches('}').count() + text.matches(';').count();
    marks as f32 / lines >= 3.0 || ["fn ", "impl ", "#include", "use std::"].iter().any(|x| text.contains(x))
}
fn volatile(s: &str) -> bool {
    let uuid = s.len() >= 32 && s.chars().filter(|c| c.is_ascii_hexdigit() || *c == '-').count() == s.len();
    let hex = s.len() >= 12 && s.starts_with("0x") && s[2..].chars().all(|c| c.is_ascii_hexdigit());
    let date = s.len() >= 10 && s.as_bytes().get(4) == Some(&b'-') && s.as_bytes().get(7) == Some(&b'-');
    uuid || hex || date
}
fn replace_volatiles(line: &str, archive: &mut VolatileArchive) -> String {
    line.split_whitespace().map(|word| {
        let trimmed = word.trim_matches(|c: char| ",;)]}".contains(c));
        if volatile(trimmed) {
            if let Some((p,_)) = archive.entries.iter().find(|(_, original)| original == trimmed) { return word.replace(trimmed, p); }
            let p = format!("$V{}", archive.entries.len()); archive.entries.push((p.clone(), trimmed.to_string())); return word.replace(trimmed, &p);
        }
        word.to_string()
    }).collect::<Vec<_>>().join(" ")
}
pub fn collapse_log_stream(text: &str, mut archive: Option<&mut VolatileArchive>) -> Result<String, LogDictReject> {
    if text.trim().is_empty() || !text.lines().any(|l| !l.trim().is_empty()) { return Err(LogDictReject::NotLogStream); }
    if looks_like_code(text) { return Err(LogDictReject::CodeStream); }
    let mut lines = Vec::new();
    for raw in text.lines() {
        let mut line = raw.to_string();
        if raw.split_whitespace().any(volatile) {
            let Some(ref mut a) = archive else { return Err(LogDictReject::VolatilesNotArchived) };
            line = replace_volatiles(raw, a);
        }
        lines.push(line);
    }
    let mut out = Vec::new(); let mut i=0;
    while i < lines.len() { let mut j=i+1; while j<lines.len() && lines[j]==lines[i] { j+=1; } out.push(if j-i>1 { format!("{} (x{})",lines[i],j-i) } else { lines[i].clone() }); i=j; }
    Ok(out.join("\n"))
}
#[cfg(test)]
mod tests { use super::*;
 #[test] fn rejects_code() { assert_eq!(collapse_log_stream("fn x() {}",None),Err(LogDictReject::CodeStream)); }
 #[test] fn archive_and_collapse() { let mut a=VolatileArchive{entries:vec![]}; let x=collapse_log_stream("2026-01-01 event\n2026-01-01 event",Some(&mut a)).unwrap(); assert!(x.contains("$V0 (x2)")); }
 #[test] fn rejects_unarchived() { assert_eq!(collapse_log_stream("2026-01-01 event",None),Err(LogDictReject::VolatilesNotArchived)); }
}

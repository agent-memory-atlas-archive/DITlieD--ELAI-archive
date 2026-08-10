//! Deterministic structural claim extraction (L-1).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClaimKind { File, Symbol, Dependency, Api, Behavior, Causal, Reachability }

impl ClaimKind {
    pub fn as_str(self) -> &'static str {
        match self { Self::File => "file", Self::Symbol => "symbol", Self::Dependency => "dependency", Self::Api => "api", Self::Behavior => "behavior", Self::Causal => "causal", Self::Reachability => "reachability" }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ByteSpan { pub start: usize, pub end: usize }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimRef { pub raw: String, pub resolved: bool }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedClaim { pub kind: ClaimKind, pub span: ByteSpan, pub refs: Vec<ClaimRef> }
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ClaimSet { claims: Vec<ExtractedClaim> }

impl ClaimSet {
    pub fn from_claims(claims: Vec<ExtractedClaim>) -> Self { Self { claims } }
    pub fn len(&self) -> usize { self.claims.len() }
    pub fn is_empty(&self) -> bool { self.claims.is_empty() }
    pub fn claims(&self) -> &[ExtractedClaim] { &self.claims }
    pub fn unresolved_ref_count(&self) -> usize { self.claims.iter().flat_map(|c| &c.refs).filter(|r| !r.resolved).count() }
    pub fn count_of_kind(&self, kind: ClaimKind) -> usize { self.claims.iter().filter(|c| c.kind == kind).count() }
}

pub trait ClaimRefResolver {
    fn resolve_path(&self, _raw: &str) -> bool { false }
    fn resolve_symbol(&self, _raw: &str) -> bool { false }
    fn resolve_package(&self, _raw: &str) -> bool { false }
}

pub struct UnresolvedResolver;
impl ClaimRefResolver for UnresolvedResolver {}

pub fn extract_claims(text: &str) -> ClaimSet { extract_claims_with(text, &UnresolvedResolver) }

pub fn extract_claims_with(text: &str, resolver: &dyn ClaimRefResolver) -> ClaimSet {
    let mut claims = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'`' {
            let start = i + 1;
            if let Some(rel_end) = text[start..].find('`') {
                let end = start + rel_end;
                let raw = &text[start..end];
                let classified = if raw.contains('(') && raw.contains("->") {
                    Some((ClaimKind::Api, resolver.resolve_symbol(raw)))
                } else if raw.contains("::") || raw.ends_with("()") {
                    Some((ClaimKind::Symbol, resolver.resolve_symbol(raw)))
                } else if is_dependency(raw) {
                    Some((ClaimKind::Dependency, resolver.resolve_package(raw)))
                } else if is_file(raw) {
                    Some((ClaimKind::File, resolver.resolve_path(raw)))
                } else { None };
                if let Some((kind, resolved)) = classified {
                    claims.push(ExtractedClaim { kind, span: ByteSpan { start, end }, refs: vec![ClaimRef { raw: raw.to_owned(), resolved }] });
                }
                i = end + 1;
                continue;
            }
        }
        i += 1;
    }
    let mut sentence_start = 0;
    for (idx, ch) in text.char_indices() {
        if matches!(ch, '.' | '!' | '?' | '\n') {
            add_sentence(&mut claims, text, sentence_start, idx + ch.len_utf8());
            sentence_start = idx + ch.len_utf8();
        }
    }
    if sentence_start < text.len() { add_sentence(&mut claims, text, sentence_start, text.len()); }
    ClaimSet::from_claims(claims)
}

fn is_dependency(raw: &str) -> bool {
    let valid_name = |name: &str| !name.is_empty() && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-');
    if let Some((name, version)) = raw.split_once('@') { return valid_name(name) && version.starts_with(|c: char| c.is_ascii_digit()); }
    if let Some((name, rhs)) = raw.split_once('=') { let version = rhs.trim().trim_matches('"'); return valid_name(name.trim()) && version.starts_with(|c: char| c.is_ascii_digit()); }
    false
}
fn is_file(raw: &str) -> bool {
    const EXTENSIONS: &[&str] = &[".rs", ".py", ".ts", ".tsx", ".js", ".md", ".toml", ".json", ".yaml", ".yml", ".svelte", ".txt"];
    raw.contains('/') || EXTENSIONS.iter().any(|ext| raw.ends_with(ext))
}
fn add_sentence(claims: &mut Vec<ExtractedClaim>, text: &str, start: usize, end: usize) {
    let sentence = &text[start..end];
    let lower = sentence.to_ascii_lowercase();
    let lexicon = |words: &[&str]| words.iter().any(|word| lower.contains(word));
    let kind = if lexicon(&["reaches", "reachable", "unreachable", "is called by", "never called", "unused", "dead code"]) { Some(ClaimKind::Reachability) }
        else if lexicon(&["causes", "caused", "because", "due to", "leads to", "breaks", "regresses"]) { Some(ClaimKind::Causal) }
        else if lexicon(&["fixes", "fixed", "passes", "returns", "handles", "works"]) { Some(ClaimKind::Behavior) } else { None };
    if let Some(kind) = kind { claims.push(ExtractedClaim { kind, span: ByteSpan { start, end }, refs: Vec::new() }); }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn kinds_and_precedence() {
        let s = extract_claims("`a/b.rs` `IpcGateway::route_frame` `left-pad@9.9.9` `fn parse(x) -> T`");
        assert_eq!(s.count_of_kind(ClaimKind::File), 1); assert_eq!(s.count_of_kind(ClaimKind::Symbol), 1);
        assert_eq!(s.count_of_kind(ClaimKind::Dependency), 1); assert_eq!(s.count_of_kind(ClaimKind::Api), 1);
        assert!(!s.claims()[2].refs[0].resolved);
    }
    #[test] fn sentence_precedence_and_spans() {
        let text = "The change causes breaks. It works! `x.rs`";
        let s = extract_claims(text);
        assert_eq!(s.count_of_kind(ClaimKind::Causal), 1);
        assert_eq!(s.count_of_kind(ClaimKind::Behavior), 1);
        assert_eq!(s.count_of_kind(ClaimKind::File), 1);
        let file_claim = s.claims().iter().find(|c| c.kind == ClaimKind::File).unwrap();
        let start = text.find("x.rs").unwrap();
        let end = start + "x.rs".len();
        assert_eq!(file_claim.span, ByteSpan { start, end });
    }
    #[test] fn empty_input() { assert!(extract_claims("").is_empty()); }
}

//! Deterministic admission checks for instruction-surface artifacts.

// blake3 content hash of artifact bytes (hex, 64 chars). TOFU manifest key.
pub fn content_hash(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdmissionFinding {
    pub rule: AdmissionRule,
    pub severity: AdmissionSeverity,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionRule {
    HardcodedSecret,
    DangerousDirective,
    ObfuscatedPayload,
    InvalidFrontmatter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdmissionSeverity {
    Pass,
    Advisory,
    Quarantine,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionVerdict {
    Admitted,
    Quarantined(Vec<AdmissionFinding>),
}

/// Scan instruction-surface text with deterministic legs. Pure, no IO.
pub fn scan_instruction_surface(text: &str) -> AdmissionVerdict {
    let mut findings = Vec::new();
    let key = ['A', 'K', 'I', 'A'].iter().collect::<String>();
    let short_prefix = ['s', 'k', '-'].iter().collect::<String>();
    let github_prefixes = [
        ['g', 'h', 'p', '_'],
        ['g', 'h', 'o', '_'],
        ['g', 'h', 's', '_'],
        ['g', 'h', 'u', '_'],
    ]
    .into_iter()
    .map(|chars| chars.iter().collect::<String>())
    .collect::<Vec<_>>();

    for line in text.lines() {
        let has_long_prefix = line.match_indices(&short_prefix).any(|(_, rest)| {
            rest.chars().skip(short_prefix.chars().count()).take(20).count() >= 20
        });
        let has_github_prefix = github_prefixes.iter().any(|prefix| line.contains(prefix));
        let has_key_pair = line.contains("BEGIN") && line.contains("PRIVATE KEY");
        if line.contains(&key) || has_long_prefix || has_github_prefix || has_key_pair {
            findings.push(finding(AdmissionRule::HardcodedSecret, "hardcoded marker"));
            break;
        }
    }

    let lowercase = text.to_lowercase();
    let directive_parts: &[&[&str]] = &[
        &["ignore", "previous", "instructions"],
        &["ignore", "all", "gates"],
        &["ignore", "all", "rules"],
        &["disregard", "the", "above"],
        &["forget", "instructions"],
        &["new", "instructions"],
        &["you", "are", "now"],
    ];
    if directive_parts
        .iter()
        .map(|parts| parts.join(" "))
        .any(|phrase| lowercase.contains(&phrase))
    {
        findings.push(finding(AdmissionRule::DangerousDirective, "directive marker"));
    }

    let zero_width = text
        .chars()
        .any(|ch| matches!(ch, '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{FEFF}'));
    let homoglyph = text.char_indices().any(|(index, ch)| {
        if !(('\u{0400}'..='\u{04FF}').contains(&ch)) {
            return false;
        }
        let before = text[..index].chars().next_back();
        let after = text[index + ch.len_utf8()..].chars().next();
        before.is_some_and(|c| c.is_ascii_alphabetic())
            && after.is_some_and(|c| c.is_ascii_alphabetic())
    });
    if zero_width || homoglyph {
        findings.push(finding(AdmissionRule::ObfuscatedPayload, "obfuscated marker"));
    }

    if let Some(frontmatter) = frontmatter_body(text) {
        if !frontmatter.lines().any(|line| line.trim_start().starts_with("name:")) {
            findings.push(finding(AdmissionRule::InvalidFrontmatter, "frontmatter name missing"));
        }
    }

    if findings.is_empty() {
        AdmissionVerdict::Admitted
    } else {
        AdmissionVerdict::Quarantined(findings)
    }
}

fn finding(rule: AdmissionRule, detail: &str) -> AdmissionFinding {
    AdmissionFinding {
        rule,
        severity: AdmissionSeverity::Quarantine,
        detail: detail.to_string(),
    }
}

fn frontmatter_body(text: &str) -> Option<&str> {
    let rest = text.strip_prefix("---\n").or_else(|| text.strip_prefix("---\r\n"))?;
    let end = rest
        .find("\n---\n")
        .or_else(|| rest.find("\n---\r\n"))?;
    Some(&rest[..end])
}

#[cfg(test)]
mod tests {
    use super::{content_hash, scan_instruction_surface, AdmissionRule, AdmissionVerdict};

    fn has_rule(verdict: &AdmissionVerdict, expected: AdmissionRule) -> bool {
        match verdict {
            AdmissionVerdict::Quarantined(findings) => findings.iter().any(|f| f.rule == expected),
            AdmissionVerdict::Admitted => false,
        }
    }

    #[test]
    fn accepts_plain_markdown() {
        assert_eq!(scan_instruction_surface("# Title\nhello"), AdmissionVerdict::Admitted);
    }

    #[test]
    fn rejects_runtime_directive() {
        let directive = ["ignore", "previous", "instructions"].join(" ");
        let verdict = scan_instruction_surface(&format!("text {directive}"));
        assert!(has_rule(&verdict, AdmissionRule::DangerousDirective));
    }

    #[test]
    fn rejects_runtime_marker() {
        let marker = ['A', 'K', 'I', 'A'].iter().collect::<String>();
        let verdict = scan_instruction_surface(&format!("{marker} suffix"));
        assert!(has_rule(&verdict, AdmissionRule::HardcodedSecret));
    }

    #[test]
    fn rejects_hidden_character() {
        let hidden = char::from_u32(0x200B).unwrap();
        let verdict = scan_instruction_surface(&format!("a{hidden}b"));
        assert!(has_rule(&verdict, AdmissionRule::ObfuscatedPayload));
    }

    #[test]
    fn rejects_nameless_frontmatter() {
        let fence = ["-", "-", "-"].concat();
        let verdict = scan_instruction_surface(&format!("{fence}\ntitle: x\n{fence}\n"));
        assert!(has_rule(&verdict, AdmissionRule::InvalidFrontmatter));
    }

    #[test]
    fn hashes_bytes_deterministically() {
        let first = content_hash(b"same");
        assert_eq!(first, content_hash(b"same"));
        assert_eq!(first.len(), 64);
        assert!(first.chars().all(|ch| ch.is_ascii_hexdigit()));
    }
}

// 4D.16: Research budget manager — evidence admission, pruning, and compression.
//
// Prevents context overflow by tracking token usage against the tier's
// research_context_budget. Pruning preserves highest-quality (Tier1) evidence.
// Compression retains first 3 + last sentence for tight budgets.

use elai_core::source_tier::SourceTier;

use crate::research_tier::ResearchTierConfig;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Approximate bytes per token (BPE for English text).
const BYTES_PER_TOKEN: usize = 4;

/// Token overcount safety margin (10%).
const TOKEN_OVERCOUNT_FACTOR: f64 = 1.1;

/// Number of leading sentences to keep during compression.
const COMPRESS_LEADING_SENTENCES: usize = 3;

// ---------------------------------------------------------------------------
// EvidenceItem
// ---------------------------------------------------------------------------

/// A single piece of evidence in the research pipeline.
#[derive(Debug, Clone)]
pub struct EvidenceItem {
    pub tokens: usize,
    pub tier: SourceTier,
    pub phi: f64,
    pub source_url: String,
    pub text: String,
}

impl EvidenceItem {
    /// Estimate token count from text bytes.
    pub fn estimate_tokens(text: &str) -> usize {
        let raw = text.len() / BYTES_PER_TOKEN;
        ((raw as f64) * TOKEN_OVERCOUNT_FACTOR).ceil() as usize
    }
}

// ---------------------------------------------------------------------------
// CompressedEvidence
// ---------------------------------------------------------------------------

/// Compressed representation of evidence: intro + conclusion only.
#[derive(Debug, Clone)]
pub struct CompressedEvidence {
    pub source_url: String,
    pub source_tier: SourceTier,
    pub summary: String,
}

// ---------------------------------------------------------------------------
// ResearchBudgetManager
// ---------------------------------------------------------------------------

/// Manages the token budget for research evidence.
///
/// Tracks current usage, admits new evidence within budget,
/// and prunes/compresses when the budget is tight.
#[derive(Debug)]
pub struct ResearchBudgetManager {
    tier_config: ResearchTierConfig,
    current_evidence_tokens: usize,
    current_sources: usize,
}

impl ResearchBudgetManager {
    pub fn new(tier_config: ResearchTierConfig) -> Self {
        Self {
            tier_config,
            current_evidence_tokens: 0,
            current_sources: 0,
        }
    }

    /// Check if new evidence can be admitted without exceeding budget.
    pub fn can_admit(&self, new_tokens: usize) -> bool {
        self.current_evidence_tokens + new_tokens <= self.tier_config.research_context_budget
    }

    /// Record tokens after admitting evidence.
    pub fn admit(&mut self, tokens: usize) {
        self.current_evidence_tokens += tokens;
        self.current_sources += 1;
    }

    /// Current token usage.
    pub fn current_tokens(&self) -> usize {
        self.current_evidence_tokens
    }

    /// Current source count.
    pub fn source_count(&self) -> usize {
        self.current_sources
    }

    /// Budget remaining.
    pub fn remaining(&self) -> usize {
        self.tier_config
            .research_context_budget
            .saturating_sub(self.current_evidence_tokens)
    }

    /// Prune evidence to fit within budget. Never drops T1Authoritative.
    /// Strategy: drop lowest-tier first, within same tier drop lowest phi.
    /// Returns number of items pruned.
    pub fn prune_to_budget(&mut self, evidence: &mut Vec<EvidenceItem>) -> usize {
        let budget = self.tier_config.research_context_budget;
        let total: usize = evidence.iter().map(|e| e.tokens).sum();
        if total <= budget {
            self.current_evidence_tokens = total;
            self.current_sources = evidence.len();
            return 0;
        }

        // Sort by (tier priority descending, phi descending) — worst items last.
        // T1=0 (best), T2=1, T3=2, T4=3.
        evidence.sort_by(|a, b| {
            let tier_a = tier_priority(&a.tier);
            let tier_b = tier_priority(&b.tier);
            tier_a
                .cmp(&tier_b)
                .then_with(|| b.phi.partial_cmp(&a.phi).unwrap_or(std::cmp::Ordering::Equal))
        });

        // Walk from the end (worst items) and remove until within budget.
        let mut current_total = total;
        let mut pruned = 0;
        while current_total > budget {
            let Some(last) = evidence.last() else { break };
            // Never prune T1.
            if last.tier == SourceTier::T1Authoritative {
                break;
            }
            let Some(removed) = evidence.pop() else { break };
            current_total -= removed.tokens;
            pruned += 1;
        }

        self.current_evidence_tokens = current_total;
        self.current_sources = evidence.len();
        pruned
    }

    /// Compress evidence by keeping first 3 sentences + last sentence.
    pub fn compress_evidence(&self, item: &EvidenceItem) -> CompressedEvidence {
        let sentences = simple_sentence_split(&item.text);
        let summary = if sentences.len() <= COMPRESS_LEADING_SENTENCES + 1 {
            // Short enough, keep as-is.
            item.text.clone()
        } else {
            let mut parts: Vec<&str> = sentences
                .iter()
                .take(COMPRESS_LEADING_SENTENCES)
                .map(|s| s.as_str())
                .collect();
            if let Some(last) = sentences.last() {
                parts.push(last.as_str());
            }
            parts.join(" ")
        };

        CompressedEvidence {
            source_url: item.source_url.clone(),
            source_tier: item.tier,
            summary,
        }
    }
}

/// Map SourceTier to sort priority (lower = better = keep).
fn tier_priority(tier: &SourceTier) -> u8 {
    match tier {
        SourceTier::T1Authoritative => 0,
        SourceTier::T2Strong => 1,
        SourceTier::T3Weak => 2,
        SourceTier::T4Rejected => 3,
    }
}

/// Simple sentence splitter for compression (no URL guard needed here —
/// evidence text is already extracted, not raw HTML).
fn simple_sentence_split(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut start = 0;

    for (i, ch) in text.char_indices() {
        if ch == '.' || ch == '!' || ch == '?' {
            let end = i + ch.len_utf8();
            let sentence = text[start..end].trim();
            if !sentence.is_empty() {
                sentences.push(sentence.to_string());
            }
            start = end;
        }
    }

    if start < text.len() {
        let remainder = text[start..].trim();
        if !remainder.is_empty() {
            sentences.push(remainder.to_string());
        }
    }

    if sentences.is_empty() && !text.trim().is_empty() {
        sentences.push(text.trim().to_string());
    }

    sentences
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::research_tier::research_config_for_tier;
    use elai_core::VramTier;

    fn make_evidence(tier: SourceTier, phi: f64, tokens: usize, text: &str) -> EvidenceItem {
        EvidenceItem {
            tokens,
            tier,
            phi,
            source_url: "https://example.com".to_string(),
            text: text.to_string(),
        }
    }

    #[test]
    fn can_admit_within_budget() {
        let cfg = research_config_for_tier(VramTier::Vram8Gb);
        let mgr = ResearchBudgetManager::new(cfg);
        assert!(mgr.can_admit(1000));
        assert!(mgr.can_admit(12000));
        assert!(!mgr.can_admit(12001));
    }

    #[test]
    fn admit_tracks_tokens() {
        let cfg = research_config_for_tier(VramTier::Vram8Gb);
        let mut mgr = ResearchBudgetManager::new(cfg);
        mgr.admit(5000);
        assert_eq!(mgr.current_tokens(), 5000);
        assert_eq!(mgr.source_count(), 1);
        assert_eq!(mgr.remaining(), 7000);
    }

    #[test]
    fn prune_removes_lowest_tier_first() {
        let cfg = research_config_for_tier(VramTier::Vram8Gb); // budget=12000
        let mut mgr = ResearchBudgetManager::new(cfg);

        let mut evidence = vec![
            make_evidence(SourceTier::T1Authoritative, 0.9, 5000, "T1 evidence."),
            make_evidence(SourceTier::T2Strong, 0.8, 5000, "T2 evidence."),
            make_evidence(SourceTier::T3Weak, 0.5, 5000, "T3 evidence."),
        ];
        // Total = 15000, budget = 12000. Should prune T3 first.
        let pruned = mgr.prune_to_budget(&mut evidence);
        assert_eq!(pruned, 1);
        assert_eq!(evidence.len(), 2);
        // T3 should be gone.
        assert!(evidence.iter().all(|e| e.tier != SourceTier::T3Weak));
    }

    #[test]
    fn prune_never_drops_t1() {
        let cfg = research_config_for_tier(VramTier::Vram8Gb); // budget=12000
        let mut mgr = ResearchBudgetManager::new(cfg);

        let mut evidence = vec![
            make_evidence(SourceTier::T1Authoritative, 0.9, 8000, "T1 big."),
            make_evidence(SourceTier::T1Authoritative, 0.8, 8000, "T1 also big."),
        ];
        // Total = 16000, budget = 12000. Both T1 — cannot prune.
        let pruned = mgr.prune_to_budget(&mut evidence);
        assert_eq!(pruned, 0);
        assert_eq!(evidence.len(), 2);
    }

    #[test]
    fn prune_retains_at_least_one_item() {
        let cfg = research_config_for_tier(VramTier::Vram8Gb); // budget=12000
        let mut mgr = ResearchBudgetManager::new(cfg);

        let mut evidence = vec![
            make_evidence(SourceTier::T2Strong, 0.9, 5000, "Best T2."),
            make_evidence(SourceTier::T3Weak, 0.3, 5000, "Weak 1."),
            make_evidence(SourceTier::T3Weak, 0.2, 5000, "Weak 2."),
            make_evidence(SourceTier::T3Weak, 0.1, 5000, "Weak 3."),
            make_evidence(SourceTier::T4Rejected, 0.1, 5000, "Rejected."),
        ];
        let pruned = mgr.prune_to_budget(&mut evidence);
        assert!(pruned > 0);
        assert!(!evidence.is_empty(), "at least 1 item must survive");
    }

    #[test]
    fn compress_keeps_intro_and_conclusion() {
        let cfg = research_config_for_tier(VramTier::Vram8Gb);
        let mgr = ResearchBudgetManager::new(cfg);

        let text = "First sentence. Second sentence. Third sentence. Fourth sentence. Fifth sentence.";
        let item = make_evidence(SourceTier::T2Strong, 0.8, 100, text);
        let compressed = mgr.compress_evidence(&item);

        assert!(compressed.summary.contains("First sentence."));
        assert!(compressed.summary.contains("Second sentence."));
        assert!(compressed.summary.contains("Third sentence."));
        assert!(compressed.summary.contains("Fifth sentence."));
        // Fourth should be dropped.
        assert!(!compressed.summary.contains("Fourth sentence."));
        assert_eq!(compressed.source_tier, SourceTier::T2Strong);
        assert_eq!(compressed.source_url, "https://example.com");
    }

    #[test]
    fn compress_short_text_unchanged() {
        let cfg = research_config_for_tier(VramTier::Vram8Gb);
        let mgr = ResearchBudgetManager::new(cfg);

        let text = "Short text. Only two sentences.";
        let item = make_evidence(SourceTier::T1Authoritative, 0.9, 50, text);
        let compressed = mgr.compress_evidence(&item);
        assert_eq!(compressed.summary, text);
    }

    #[test]
    fn token_estimation_within_bounds() {
        let text = "Hello world, this is a test string for token estimation.";
        let tokens = EvidenceItem::estimate_tokens(text);
        let byte_count = text.len();
        let raw_estimate = byte_count / BYTES_PER_TOKEN;
        // Should be within 20% of raw + 10% overcount.
        assert!(tokens >= raw_estimate, "tokens {} < raw {}", tokens, raw_estimate);
        let max_expected = ((raw_estimate as f64) * 1.3).ceil() as usize;
        assert!(tokens <= max_expected, "tokens {} > max {}", tokens, max_expected);
    }

    #[test]
    fn prune_within_same_tier_drops_lowest_phi() {
        let cfg = research_config_for_tier(VramTier::Vram8Gb); // budget=12000
        let mut mgr = ResearchBudgetManager::new(cfg);

        let mut evidence = vec![
            make_evidence(SourceTier::T2Strong, 0.9, 5000, "High phi."),
            make_evidence(SourceTier::T2Strong, 0.3, 5000, "Low phi."),
            make_evidence(SourceTier::T2Strong, 0.6, 5000, "Mid phi."),
        ];
        // Total=15000, budget=12000. Should prune lowest phi T2 first.
        let pruned = mgr.prune_to_budget(&mut evidence);
        assert_eq!(pruned, 1);
        // The item with phi=0.3 should be removed.
        assert!(evidence.iter().all(|e| e.phi > 0.3));
    }
}

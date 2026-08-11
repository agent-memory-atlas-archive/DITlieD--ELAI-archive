//! L-8 detection of repeated claims whose references no longer resolve.
use std::collections::HashSet;

use super::extract::{ClaimKind, ClaimRefResolver, ClaimSet, ExtractedClaim};

/// A claim identified as contradicted on a later pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantinedClaim {
    pub content_key: [u8; 32],
    pub raw_refs: Vec<String>,
    pub kind: ClaimKind,
}

/// Single definition of the `claim:` fact-id convention shared by the emit and the consult.
pub fn claim_fact_id(key: &[u8; 32]) -> String {
    format!("claim:{:.16}", blake3::Hash::from_bytes(*key).to_hex())
}

/// Span-independent identity for a claim assertion.
pub fn content_key(claim: &ExtractedClaim) -> [u8; 32] {
    let mut raws: Vec<&str> = claim.refs.iter().map(|reference| reference.raw.as_str()).collect();
    raws.sort_unstable();
    let mut hasher = blake3::Hasher::new();
    hasher.update(claim.kind.as_str().as_bytes());
    for raw in raws {
        hasher.update(raw.as_bytes());
    }
    *hasher.finalize().as_bytes()
}

/// Return only claims seen in an earlier pass that fail current-tree resolution.
pub fn detect_contradicted(
    prior_keys: &HashSet<[u8; 32]>,
    set: &ClaimSet,
    resolver: &dyn ClaimRefResolver,
) -> Vec<QuarantinedClaim> {
    set.claims()
        .iter()
        .filter_map(|claim| {
            let key = content_key(claim);
            if !prior_keys.contains(&key) || claim.refs.is_empty() {
                return None;
            }
            // Behavior, Causal, and Reachability have no tree reference in v1.
            let resolves = match claim.kind {
                ClaimKind::File => claim.refs.iter().all(|r| resolver.resolve_path(&r.raw)),
                ClaimKind::Dependency => claim.refs.iter().all(|r| resolver.resolve_package(&r.raw)),
                ClaimKind::Symbol | ClaimKind::Api => claim.refs.iter().all(|r| resolver.resolve_symbol(&r.raw)),
                ClaimKind::Behavior | ClaimKind::Causal | ClaimKind::Reachability => true,
            };
            if resolves { return None; }
            Some(QuarantinedClaim {
                content_key: key,
                raw_refs: claim.refs.iter().map(|r| r.raw.clone()).collect(),
                kind: claim.kind,
            })
        })
        .collect()
}

/// Existing CLI kill switch (cli/src/dkg_lifecycle.rs:50); default is enabled.
pub fn quarantine_disabled() -> bool {
    std::env::var("ELAI_STALENESS_CASCADE_KILL")
        .map(|value| matches!(value.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

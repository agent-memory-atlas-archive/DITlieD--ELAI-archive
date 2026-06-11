//! H-33 (SCHEMA_VERSION 211): canonical dispatcher for `elai memory <sub>`.
//!
//! Activates the stub left in main.rs by s-31708. Handles:
//!   list     — paginated listing of currently-valid + superseded rows
//!   inspect  — full row + evidence_pointers + superseded_by chain
//!   delete   — soft-close via FactStore::soft_close + MemoryGovernanceEvent::Forgotten beacon
//!   insert   — manual T0 user-direct insert
//!   pin      — emit MemoryGovernanceEvent::Pinned beacon (no schema migration)
//!   unpin    — emit MemoryGovernanceEvent::Unpinned beacon
//!   provenance — eligible-to-fire set read-only, ZERO tokens injected
//!
//! Sync entry — owns its own current-thread tokio runtime for the SSB write
//! path (same pattern as github_pr_cmd.rs + archive_verify::run).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use elai_core::clock::OsClock;
use elai_core::paths::{memory_facts_db_path, sessions_dir_path};
use elai_memory::{FactStore, FactType, MemoryFact, Provenance, Scope, TrustTier};
use elai_orchestrator::ssb::{
    MemoryGovernanceEventPayload, MemoryGovernanceKind, SsbPayload,
};
use elai_orchestrator::{Ssb, SsbConfig};
use serde_json::json;

use crate::args::MemorySubcommandKind;

/// Session id used for beacon writes in this CLI path.
const CLI_SESSION_ID: &str = "cli:memory";
const CLI_ACTOR: &str = "cli";

/// Build an SSB writer for the project root. Returns `None` (not an error)
/// when no sessions directory exists — callers that emit beacons should
/// warn and continue; the FactStore mutation must succeed even without SSB.
fn open_ssb_optional(project_root: &Path) -> Option<Arc<Ssb>> {
    let db_path = sessions_dir_path(project_root).join("ssb.db");
    let parent = db_path.parent()?;
    if let Err(e) = std::fs::create_dir_all(parent) {
        tracing::warn!(error = %e, "memory_cmd: cannot create ssb sessions dir");
        return None;
    }
    let cfg = SsbConfig {
        db_path,
        pool_size: None,
        project_id: "default".to_string(),
        session_id: CLI_SESSION_ID.to_string(),
    };
    let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
    match Ssb::new(cfg, clock) {
        Ok(ssb) => Some(Arc::new(ssb)),
        Err(e) => {
            tracing::warn!(error = %e, "memory_cmd: cannot open SSB writer");
            None
        }
    }
}

/// Emit a `MemoryGovernanceEvent` beacon to SSB. Non-fatal: warns on error
/// so that the FactStore mutation is never rolled back due to SSB unavailability.
async fn emit_governance_beacon(
    ssb: &Option<Arc<Ssb>>,
    fact_id: &str,
    kind: MemoryGovernanceKind,
    now_ms: u64,
) {
    let Some(ssb) = ssb else {
        tracing::warn!(
            fact_id,
            "memory_cmd: SSB unavailable; governance beacon not emitted"
        );
        return;
    };
    let payload = SsbPayload::MemoryGovernanceEvent(MemoryGovernanceEventPayload {
        fact_id: fact_id.to_owned(),
        kind,
        actor: CLI_ACTOR.to_owned(),
        session_id: CLI_SESSION_ID.to_owned(),
        created_at_ms: now_ms,
    });
    if let Err(e) = ssb
        .write_payload("cli.memory", 0, payload, vec![])
        .await
    {
        tracing::warn!(error = %e, fact_id, "memory_cmd: governance beacon write failed");
    }
}

/// Open the fact store at the project root, returning the path for display.
fn open_fact_store(project_root: &Path) -> Result<(FactStore, PathBuf)> {
    let path = memory_facts_db_path(project_root);
    let store = FactStore::open(&path)
        .with_context(|| format!("opening fact store at {}", path.display()))?;
    Ok((store, path))
}

/// Main entry point routed from main.rs.
#[allow(clippy::too_many_arguments)]
pub fn run_memory_subcommand(
    sub: MemorySubcommandKind,
    project_root: &PathBuf,
    user_id: &str,
    scope: Option<&str>,
    fact_type: Option<&str>,
    fact_id: Option<&str>,
    key: Option<&str>,
    value_json: Option<&str>,
    limit: usize,
    json: bool,
) -> Result<()> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .context("building tokio runtime for `elai memory`")?;

    rt.block_on(async move {
        run_inner(
            sub, project_root, user_id, scope, fact_type, fact_id, key, value_json, limit, json,
        )
        .await
    })
}

async fn run_inner(
    sub: MemorySubcommandKind,
    project_root: &Path,
    user_id: &str,
    scope: Option<&str>,
    fact_type: Option<&str>,
    fact_id: Option<&str>,
    key: Option<&str>,
    value_json: Option<&str>,
    limit: usize,
    json: bool,
) -> Result<()> {
    let now_ms = {
        let clock: Arc<dyn elai_core::clock::ClockProvider> = Arc::new(OsClock);
        clock.now_ms() as u64
    };

    match sub {
        MemorySubcommandKind::List => {
            let (store, store_path) = open_fact_store(project_root)?;
            let parsed_fact_type = parse_fact_type(fact_type)?;
            // list_including_closed returns ALL rows (active + superseded/closed).
            // Post-filter by scope because list_including_closed has no scope param.
            let all_rows = store
                .list_including_closed(user_id, parsed_fact_type, limit)
                .context("list_including_closed")?;
            let parsed_scope = parse_scope(scope)?;
            let rows: Vec<_> = if let Some(s) = parsed_scope {
                all_rows.into_iter().filter(|r| r.scope == s).collect()
            } else {
                all_rows
            };
            if json {
                let facts: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|f| {
                        let mut v = fact_to_json(f);
                        let status = if f.valid_to_ms.is_some() { "superseded" } else { "active" };
                        v["status"] = serde_json::Value::String(status.to_owned());
                        v
                    })
                    .collect();
                println!(
                    "{}",
                    json!({
                        "store_path": store_path.to_string_lossy(),
                        "count": facts.len(),
                        "facts": facts,
                    })
                );
            } else {
                let active_count = rows.iter().filter(|r| r.valid_to_ms.is_none()).count();
                let closed_count = rows.len() - active_count;
                println!("store: {}", store_path.display());
                println!("{} row(s) ({} active, {} superseded/closed)", rows.len(), active_count, closed_count);
                for f in &rows {
                    print_fact_summary(f);
                }
            }
        }

        MemorySubcommandKind::Inspect => {
            let fid = fact_id
                .ok_or_else(|| anyhow!("`elai memory inspect` requires --fact-id"))?;
            let (store, store_path) = open_fact_store(project_root)?;
            let fact = store
                .get(fid)
                .context("get")?
                .ok_or_else(|| anyhow!("fact_id not found: {fid}"))?;
            if json {
                println!("{}", json!({
                    "store_path": store_path.to_string_lossy(),
                    "fact": fact_to_json(&fact),
                }));
            } else {
                println!("store: {}", store_path.display());
                print_fact_detail(&fact);
            }
        }

        MemorySubcommandKind::Delete => {
            let fid = fact_id
                .ok_or_else(|| anyhow!("`elai memory delete` requires --fact-id"))?;
            let mut store = {
                let path = memory_facts_db_path(project_root);
                elai_memory::FactStore::open(&path)
                    .with_context(|| format!("opening fact store at {}", path.display()))?
            };
            store
                .soft_close(fid, now_ms as u128)
                .with_context(|| format!("soft_close({fid})"))?;

            // Emit SSB beacon — non-fatal if SSB unavailable.
            let ssb = open_ssb_optional(project_root);
            emit_governance_beacon(&ssb, fid, MemoryGovernanceKind::Forgotten, now_ms).await;

            if json {
                println!(
                    "{}",
                    json!({
                        "fact_id": fid,
                        "action": "forgotten",
                        "soft_closed_at_ms": now_ms,
                    })
                );
            } else {
                println!("forgotten: {fid}  (valid_to_ms = {now_ms})");
            }
        }

        MemorySubcommandKind::Insert => {
            let k = key.ok_or_else(|| anyhow!("`elai memory insert` requires --key"))?;
            let v_raw =
                value_json.ok_or_else(|| anyhow!("`elai memory insert` requires --value-json"))?;
            let value: serde_json::Value = serde_json::from_str(v_raw)
                .context("--value-json must be valid JSON")?;
            let ft = fact_type
                .ok_or_else(|| anyhow!("`elai memory insert` requires --fact-type"))?;
            let parsed_ft = FactType::from_str(ft)
                .map_err(|e| anyhow!("invalid --fact-type: {e}"))?;
            let parsed_scope = parse_scope(scope)?.unwrap_or(Scope::Global);

            let decay_rate = elai_memory::default_decay_rate(parsed_ft);
            let fact_id_new = MemoryFact::new_id();
            let fact = MemoryFact {
                fact_id: fact_id_new.clone(),
                user_id: user_id.to_owned(),
                project_id: None,
                scope: parsed_scope,
                fact_type: parsed_ft,
                key: k.to_owned(),
                value,
                confidence: 1.0,
                trust_score: 1.0,
                ingested_at_ms: now_ms as u128,
                valid_from_ms: now_ms as u128,
                valid_to_ms: None,
                decay_rate_per_day: decay_rate,
                last_observed_at_ms: now_ms as u128,
                evidence_pointers: vec!["cli:user_direct".to_owned()],
                superseded_by: None,
                trust_tier: TrustTier::T0UserDirect,
                provenance: Provenance::user_direct("cli"),
                last_validated_at_ms: None,
            };
            let (store, store_path) = open_fact_store(project_root)?;
            store.insert(&fact).context("insert")?;

            if json {
                println!(
                    "{}",
                    json!({
                        "store_path": store_path.to_string_lossy(),
                        "fact_id": fact_id_new,
                        "action": "inserted",
                    })
                );
            } else {
                println!("inserted: {fact_id_new}  store: {}", store_path.display());
            }
        }

        MemorySubcommandKind::Pin => {
            let fid = fact_id
                .ok_or_else(|| anyhow!("`elai memory pin` requires --fact-id"))?;
            let ssb = open_ssb_optional(project_root);
            emit_governance_beacon(&ssb, fid, MemoryGovernanceKind::Pinned, now_ms).await;
            if ssb.is_none() {
                eprintln!("warn: SSB unavailable; pin beacon not persisted");
            }
            if json {
                println!(
                    "{}",
                    json!({
                        "fact_id": fid,
                        "action": "pinned",
                        "ssb_written": ssb.is_some(),
                    })
                );
            } else {
                println!("pinned: {fid}");
            }
        }

        MemorySubcommandKind::Unpin => {
            let fid = fact_id
                .ok_or_else(|| anyhow!("`elai memory unpin` requires --fact-id"))?;
            let ssb = open_ssb_optional(project_root);
            emit_governance_beacon(&ssb, fid, MemoryGovernanceKind::Unpinned, now_ms).await;
            if ssb.is_none() {
                eprintln!("warn: SSB unavailable; unpin beacon not persisted");
            }
            if json {
                println!(
                    "{}",
                    json!({
                        "fact_id": fid,
                        "action": "unpinned",
                        "ssb_written": ssb.is_some(),
                    })
                );
            } else {
                println!("unpinned: {fid}");
            }
        }

        MemorySubcommandKind::Provenance => {
            // Eligible-to-fire set: deterministic trust-floor predicate.
            // ZERO tokens injected into any model prompt.
            // Mirrors the trust + evidence_pointers filter in retrieval.rs:117-124.
            let (store, store_path) = open_fact_store(project_root)?;
            let parsed_scope = parse_scope(scope)?;
            let parsed_fact_type = parse_fact_type(fact_type)?;
            let all_rows = store
                .list_current(user_id, parsed_scope, parsed_fact_type, limit)
                .context("list_current for provenance")?;

            let eligible: Vec<&MemoryFact> = all_rows
                .iter()
                .filter(|f| {
                    let threshold = f.fact_type.retrieval_threshold();
                    f.trust_score >= threshold && !f.evidence_pointers.is_empty()
                })
                .collect();

            let ineligible: Vec<&MemoryFact> = all_rows
                .iter()
                .filter(|f| {
                    let threshold = f.fact_type.retrieval_threshold();
                    f.trust_score < threshold || f.evidence_pointers.is_empty()
                })
                .collect();

            if json {
                println!(
                    "{}",
                    json!({
                        "store_path": store_path.to_string_lossy(),
                        "eligible": eligible.iter().map(|f| fact_to_json(f)).collect::<Vec<_>>(),
                        "ineligible": ineligible.iter().map(|f| fact_to_json(f)).collect::<Vec<_>>(),
                    })
                );
            } else {
                println!("store: {}", store_path.display());
                println!("eligible to fire ({}):", eligible.len());
                for f in &eligible {
                    println!(
                        "  [ELIGIBLE] {} | type={} | trust={:.3} | key={}",
                        f.fact_id,
                        f.fact_type.as_str(),
                        f.trust_score,
                        f.key,
                    );
                }
                if !ineligible.is_empty() {
                    println!("below threshold ({}):", ineligible.len());
                    for f in &ineligible {
                        let threshold = f.fact_type.retrieval_threshold();
                        println!(
                            "  [BLOCKED]  {} | type={} | trust={:.3} < threshold={:.1} | evidence={}",
                            f.fact_id,
                            f.fact_type.as_str(),
                            f.trust_score,
                            threshold,
                            f.evidence_pointers.len(),
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_scope(raw: Option<&str>) -> Result<Option<Scope>> {
    match raw {
        None | Some("") | Some("all") | Some("any") => Ok(None),
        Some(s) => Scope::from_str(s)
            .map(Some)
            .map_err(|e| anyhow!("invalid --scope: {e}")),
    }
}

fn parse_fact_type(raw: Option<&str>) -> Result<Option<FactType>> {
    match raw {
        None | Some("") | Some("all") | Some("any") => Ok(None),
        Some(s) => FactType::from_str(s)
            .map(Some)
            .map_err(|e| anyhow!("invalid --fact-type: {e}")),
    }
}

fn fact_to_json(f: &MemoryFact) -> serde_json::Value {
    json!({
        "fact_id": f.fact_id,
        "user_id": f.user_id,
        "scope": f.scope.as_str(),
        "fact_type": f.fact_type.as_str(),
        "key": f.key,
        "value": f.value,
        "trust_score": f.trust_score,
        "trust_threshold": f.fact_type.retrieval_threshold(),
        "confidence": f.confidence,
        "ingested_at_ms": f.ingested_at_ms as u64,
        "valid_from_ms": f.valid_from_ms as u64,
        "valid_to_ms": f.valid_to_ms.map(|v| v as u64),
        "decay_rate_per_day": f.decay_rate_per_day,
        "last_observed_at_ms": f.last_observed_at_ms as u64,
        "evidence_pointers": f.evidence_pointers,
        "superseded_by": f.superseded_by,
        "trust_tier": f.trust_tier.as_str(),
    })
}

fn print_fact_summary(f: &MemoryFact) {
    let status = if f.valid_to_ms.is_some() {
        let sup = f.superseded_by.as_deref().unwrap_or("closed");
        format!(" [superseded -> {sup}]")
    } else {
        String::new()
    };
    println!(
        "  {} | type={} | trust={:.3} | key={}{}",
        f.fact_id,
        f.fact_type.as_str(),
        f.trust_score,
        f.key,
        status,
    );
}

fn print_fact_detail(f: &MemoryFact) {
    println!("fact_id:          {}", f.fact_id);
    println!("user_id:          {}", f.user_id);
    println!("scope:            {}", f.scope.as_str());
    println!("fact_type:        {}", f.fact_type.as_str());
    println!("key:              {}", f.key);
    println!("value:            {}", f.value);
    println!("trust_score:      {:.4}", f.trust_score);
    println!("trust_tier:       {}", f.trust_tier.as_str());
    println!("confidence:       {:.4}", f.confidence);
    println!("ingested_at_ms:   {}", f.ingested_at_ms);
    println!("valid_from_ms:    {}", f.valid_from_ms);
    println!("valid_to_ms:      {:?}", f.valid_to_ms);
    println!("decay_rate/day:   {:.6}", f.decay_rate_per_day);
    println!("last_observed_ms: {}", f.last_observed_at_ms);
    println!("evidence_ptrs:    {}", f.evidence_pointers.join(", "));
    if let Some(sup) = &f.superseded_by {
        println!("superseded_by:    {sup}");
    }
}

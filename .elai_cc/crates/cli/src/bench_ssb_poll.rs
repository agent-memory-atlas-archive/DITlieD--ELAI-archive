use std::sync::Arc;
use std::time::Duration;

use elai_orchestrator::ssb::{SsbActionType, SsbEntry, SsbReader as _};
use elai_orchestrator::Ssb;

const POLL_RETRIES: u32 = 10;
const POLL_INTERVAL: Duration = Duration::from_millis(50);

pub async fn poll_until_this_run_row<F>(
    ssb: &Arc<Ssb>,
    action_type: SsbActionType,
    matcher: F,
) -> Result<SsbEntry, String>
where
    F: Fn(&SsbEntry) -> bool,
{
    for _ in 0..POLL_RETRIES {
        tokio::time::sleep(POLL_INTERVAL).await;
        let rows = ssb
            .query_by_type(action_type)
            .map_err(|e| format!("query_by_type failed: {e}"))?;
        if let Some(entry) = rows.iter().find(|r| matcher(r)) {
            return Ok(entry.clone());
        }
    }
    Err("no matching SSB row landed after retries".to_string())
}

//! K-2 routing-economics benchmark runner.
//!
//! The fixture arm compares the shipped hybrid routing policy against a
//! force-cloud-everything control. Both arms pass the same fixture contexts
//! through [`RuntimeRoutingPolicy::evaluate`] and then validate the resulting
//! local/cloud choice through [`CloudModeRouter::route`]. Cost comes only from
//! the fixture's J-28 [`UsageTurnEventPayload`] rows and is folded by
//! [`crate::usage_ledger::rollup_usage_turns`]; this runner never prices tokens.
//!
//! The live lane is intentionally unfunded in K-2. Its honest status is
//! [`RunnerStatus::RunnerPresent`] with `LIVE_COST_UNAVAILABLE`, never a
//! fabricated dollar number.

use elai_providers::runtime::{RoutingChoice, RoutingContext, RuntimeRoutingPolicy};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::cloud_mode::{
    CloudConfig, CloudCredentials, CloudModeError, CloudModeRouter, ExecMode,
    RoutedProvider, DEFAULT_CLOUD_PROVIDER,
};
use crate::ssb::UsageTurnEventPayload;
use crate::usage_ledger::rollup_usage_turns;

const FIXTURE_JSON: &str =
    include_str!("../../../../benchmarks/routing_economics/fixture_v1.json");

pub const LIVE_COST_UNAVAILABLE: &str = "cost unavailable — live lane not funded";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunnerStatus {
    Measured,
    RunnerPresent,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutingArmReport {
    pub arm: String,
    pub n_tasks: u32,
    pub n_offloaded: u32,
    pub percent_offloaded: f64,
    pub n_verified: u32,
    pub total_cost_micros: u64,
    pub cost_per_verified_task_micros: Option<u64>,
    pub cost_per_verified_task_usd: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoutingEconomicsReport {
    pub runner_status: RunnerStatus,
    pub fixture_status: RunnerStatus,
    pub suite: String,
    pub seed: u64,
    pub selected_arm: String,
    pub fixture_blake3: String,
    pub hybrid: RoutingArmReport,
    pub force_cloud_everything: RoutingArmReport,
    pub live_cost: String,
    pub live_lane_funded: bool,
}

#[derive(Debug, Error)]
pub enum RoutingEconomicsError {
    #[error("routing-economics fixture parse failed: {0}")]
    FixtureParse(#[from] serde_json::Error),
    #[error("routing-economics fixture is empty")]
    EmptyFixture,
    #[error("routing failed for task {task_id} in arm {arm}: {source}")]
    Routing {
        task_id: String,
        arm: &'static str,
        #[source]
        source: CloudModeError,
    },
    #[error("usage-ledger cost unavailable for task {task_id} in arm {arm}")]
    UsageCostUnavailable {
        task_id: String,
        arm: &'static str,
    },
    #[error("routing-economics counter overflow in arm {arm}")]
    CounterOverflow { arm: &'static str },
    #[error("routing-economics cost overflow in arm {arm}")]
    CostOverflow { arm: &'static str },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RoutingFixture {
    suite: String,
    tasks: Vec<RoutingFixtureTask>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct RoutingFixtureTask {
    task_id: String,
    context: RoutingContext,
    local_usage: UsageTurnEventPayload,
    cloud_usage: UsageTurnEventPayload,
    local_verified: bool,
    cloud_verified: bool,
}

/// Production runner called by the `elai bench routing-economics` CLI handler.
pub fn run_routing_economics(
    seed: u64,
    force_cloud_everything: bool,
) -> Result<RoutingEconomicsReport, RoutingEconomicsError> {
    compose_routing_economics(seed, force_cloud_everything)
}

/// Compose the deterministic fixture comparison used by the production runner.
pub fn compose_routing_economics(
    seed: u64,
    force_cloud_everything: bool,
) -> Result<RoutingEconomicsReport, RoutingEconomicsError> {
    let fixture: RoutingFixture = serde_json::from_str(FIXTURE_JSON)?;
    if fixture.tasks.is_empty() {
        return Err(RoutingEconomicsError::EmptyFixture);
    }

    let router = CloudModeRouter::new(CloudConfig::default());
    let policy = RuntimeRoutingPolicy::default();
    let hybrid = evaluate_arm(&fixture.tasks, &policy, &router, false)?;
    let force_cloud = evaluate_arm(&fixture.tasks, &policy, &router, true)?;

    Ok(RoutingEconomicsReport {
        runner_status: RunnerStatus::RunnerPresent,
        fixture_status: RunnerStatus::Measured,
        suite: fixture.suite,
        seed,
        selected_arm: if force_cloud_everything {
            "force_cloud_everything".to_string()
        } else {
            "hybrid".to_string()
        },
        fixture_blake3: blake3::hash(FIXTURE_JSON.as_bytes())
            .to_hex()
            .to_string(),
        hybrid,
        force_cloud_everything: force_cloud,
        live_cost: LIVE_COST_UNAVAILABLE.to_string(),
        live_lane_funded: false,
    })
}

fn evaluate_arm(
    tasks: &[RoutingFixtureTask],
    policy: &RuntimeRoutingPolicy,
    router: &CloudModeRouter,
    force_cloud_everything: bool,
) -> Result<RoutingArmReport, RoutingEconomicsError> {
    let arm = if force_cloud_everything {
        "force_cloud_everything"
    } else {
        "hybrid"
    };
    let mut n_tasks = 0_u32;
    let mut n_offloaded = 0_u32;
    let mut n_verified = 0_u32;
    let mut total_cost_micros = 0_u64;

    for task in tasks {
        let mut context = task.context.clone();
        context.ops_override = force_cloud_everything.then_some(RoutingChoice::CloudSonnet);
        let decision = policy.evaluate(&context);
        let mode = match decision.choice {
            RoutingChoice::LocalSidecar => ExecMode::Local,
            RoutingChoice::CloudSonnet
            | RoutingChoice::CloudOpus
            | RoutingChoice::CloudDeepSeekFlash => ExecMode::Cloud,
        };
        let credentials = match mode {
            ExecMode::Cloud => Some(CloudCredentials {
                bearer_token: "routing-economics-fixture-no-network".to_string(),
                provider: DEFAULT_CLOUD_PROVIDER.to_string(),
            }),
            ExecMode::Local | ExecMode::Auto => None,
        };
        let routed = router
            .route(mode, credentials)
            .map_err(|source| RoutingEconomicsError::Routing {
                task_id: task.task_id.clone(),
                arm,
                source,
            })?;

        let (usage, verified, offloaded) = match routed {
            RoutedProvider::Local => (&task.local_usage, task.local_verified, true),
            RoutedProvider::Cloud { .. } => (&task.cloud_usage, task.cloud_verified, false),
        };
        let cost_micros = usage_cost_micros(usage).ok_or_else(|| {
            RoutingEconomicsError::UsageCostUnavailable {
                task_id: task.task_id.clone(),
                arm,
            }
        })?;

        n_tasks = n_tasks
            .checked_add(1)
            .ok_or(RoutingEconomicsError::CounterOverflow { arm })?;
        if offloaded {
            n_offloaded = n_offloaded
                .checked_add(1)
                .ok_or(RoutingEconomicsError::CounterOverflow { arm })?;
        }
        if verified {
            n_verified = n_verified
                .checked_add(1)
                .ok_or(RoutingEconomicsError::CounterOverflow { arm })?;
        }
        total_cost_micros = total_cost_micros
            .checked_add(cost_micros)
            .ok_or(RoutingEconomicsError::CostOverflow { arm })?;
    }

    let percent_offloaded = if n_tasks == 0 {
        0.0
    } else {
        f64::from(n_offloaded) * 100.0 / f64::from(n_tasks)
    };
    let cost_per_verified_task_micros = if n_verified == 0 {
        None
    } else {
        Some(total_cost_micros / u64::from(n_verified))
    };
    let cost_per_verified_task_usd =
        cost_per_verified_task_micros.map(|micros| micros as f64 / 1_000_000.0);

    Ok(RoutingArmReport {
        arm: arm.to_string(),
        n_tasks,
        n_offloaded,
        percent_offloaded,
        n_verified,
        total_cost_micros,
        cost_per_verified_task_micros,
        cost_per_verified_task_usd,
    })
}

fn usage_cost_micros(row: &UsageTurnEventPayload) -> Option<u64> {
    let rows = std::slice::from_ref(row);
    let rollup = rollup_usage_turns(rows);
    rollup.values().next().and_then(|entry| entry.cost_micros)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hybrid_offloads_more_than_force_cloud() {
        let report = compose_routing_economics(7, false).expect("compose");
        assert!(
            report.hybrid.percent_offloaded
                > report.force_cloud_everything.percent_offloaded
        );
    }

    #[test]
    fn same_seed_is_byte_identical() {
        let first = compose_routing_economics(42, false).expect("first");
        let second = compose_routing_economics(42, false).expect("second");
        assert_eq!(
            serde_json::to_vec(&first).expect("serialize first"),
            serde_json::to_vec(&second).expect("serialize second")
        );
    }
}

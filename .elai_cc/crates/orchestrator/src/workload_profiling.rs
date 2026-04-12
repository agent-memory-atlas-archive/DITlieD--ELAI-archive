//! Workload profiling for per-agent per-phase metrics.
//!
//! Phase 3J.7: Track tokens_in, tokens_out, latency_ms, vram_peak_mb
//! Written to SSB as action_type='workload_profile' at phase boundary.

use serde::{Deserialize, Serialize};
use std::time::UNIX_EPOCH;

use crate::agent_pool::AgentRole;
use crate::ssb::{Ssb, SsbPayload, WorkloadProfilePayload};
use elai_core::clock::ClockProvider;

/// Workload profile metrics collected during agent execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadProfile {
    pub agent_role: AgentRole,
    pub phase_id: String,
    pub tokens_in: u64,
    pub tokens_out: u64,
    pub latency_ms: u64,
    pub vram_peak_mb: u32,
    pub model_id: String,
    pub gpu_id: u8,
    pub timestamp: i64,
}

impl WorkloadProfile {
    /// Create a new workload profile with current timestamp from ClockProvider.
    pub fn new(
        agent_role: AgentRole,
        phase_id: String,
        tokens_in: u64,
        tokens_out: u64,
        latency_ms: u64,
        vram_peak_mb: u32,
        model_id: String,
        gpu_id: u8,
        clock: &dyn ClockProvider,
    ) -> Self {
        let timestamp = clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;
        
        Self {
            agent_role,
            phase_id,
            tokens_in,
            tokens_out,
            latency_ms,
            vram_peak_mb,
            model_id,
            gpu_id,
            timestamp,
        }
    }

    /// Convert to SSB payload for writing.
    pub fn to_ssb_payload(&self) -> SsbPayload {
        // Convert AgentRole enum to snake_case string for serialization
        let agent_role_str = serde_json::to_string(&self.agent_role)
            .unwrap_or_else(|_| "\"unknown\"".to_string())
            .trim_matches('"')
            .to_string();
        
        SsbPayload::WorkloadProfile(WorkloadProfilePayload {
            agent_role: agent_role_str,
            phase_id: self.phase_id.clone(),
            tokens_in: self.tokens_in,
            tokens_out: self.tokens_out,
            latency_ms: self.latency_ms,
            vram_peak_mb: self.vram_peak_mb,
            model_id: self.model_id.clone(),
            gpu_id: self.gpu_id,
            timestamp: self.timestamp,
        })
    }

    /// Write workload profile to SSB.
    pub async fn write_to_ssb(
        &self,
        ssb: &Ssb,
        agent_id: &str,
        superstep: i64,
    ) -> Result<i64, crate::ssb::SsbError> {
        let payload = self.to_ssb_payload();
        ssb.write_payload(agent_id, superstep, payload, vec![]).await
    }
}

/// Builder for constructing workload profiles incrementally.
#[derive(Debug, Clone, Default)]
pub struct WorkloadProfileBuilder {
    agent_role: Option<AgentRole>,
    phase_id: Option<String>,
    tokens_in: u64,
    tokens_out: u64,
    start_time_ms: Option<u64>,
    vram_peak_mb: u32,
    model_id: Option<String>,
    gpu_id: u8,
}

impl WorkloadProfileBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn agent_role(mut self, role: AgentRole) -> Self {
        self.agent_role = Some(role);
        self
    }

    pub fn phase_id(mut self, phase_id: impl Into<String>) -> Self {
        self.phase_id = Some(phase_id.into());
        self
    }

    pub fn tokens_in(mut self, tokens: u64) -> Self {
        self.tokens_in = tokens;
        self
    }

    pub fn tokens_out(mut self, tokens: u64) -> Self {
        self.tokens_out = tokens;
        self
    }

    pub fn start_timing(&mut self, clock: &dyn ClockProvider) {
        self.start_time_ms = Some(now_ms(clock));
    }

    pub fn vram_peak_mb(mut self, vram_mb: u32) -> Self {
        self.vram_peak_mb = vram_mb;
        self
    }

    pub fn model_id(mut self, model_id: impl Into<String>) -> Self {
        self.model_id = Some(model_id.into());
        self
    }

    pub fn gpu_id(mut self, gpu_id: u8) -> Self {
        self.gpu_id = gpu_id;
        self
    }

    /// Build the workload profile, calculating latency from start time if set.
    pub fn build(self, clock: &dyn ClockProvider) -> Option<WorkloadProfile> {
        let agent_role = self.agent_role?;
        let phase_id = self.phase_id?;
        let model_id = self.model_id?;

        let latency_ms = if let Some(start) = self.start_time_ms {
            now_ms(clock).saturating_sub(start)
        } else {
            0
        };

        Some(WorkloadProfile::new(
            agent_role,
            phase_id,
            self.tokens_in,
            self.tokens_out,
            latency_ms,
            self.vram_peak_mb,
            model_id,
            self.gpu_id,
            clock,
        ))
    }
}

fn now_ms(clock: &dyn ClockProvider) -> u64 {
    clock
        .system_now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_pool::AgentRole;
    use elai_core::clock::OsClock;

    #[test]
    fn test_workload_profile_creation() {
        let clock = OsClock;
        let profile = WorkloadProfile::new(
            AgentRole::Executor,
            "phase-1".to_string(),
            1000,
            500,
            100,
            2048,
            "gemma4:e2b".to_string(),
            0,
            &clock,
        );

        assert_eq!(profile.agent_role, AgentRole::Executor);
        assert_eq!(profile.phase_id, "phase-1");
        assert_eq!(profile.tokens_in, 1000);
        assert_eq!(profile.tokens_out, 500);
        assert_eq!(profile.latency_ms, 100);
        assert_eq!(profile.vram_peak_mb, 2048);
        assert_eq!(profile.model_id, "gemma4:e2b");
        assert_eq!(profile.gpu_id, 0);
        assert!(profile.timestamp > 0);
    }

    #[test]
    fn test_workload_profile_to_ssb_payload() {
        let clock = OsClock;
        let profile = WorkloadProfile::new(
            AgentRole::Auditor,
            "phase-2".to_string(),
            2000,
            1000,
            200,
            4096,
            "gemma4:31b".to_string(),
            1,
            &clock,
        );

        let payload = profile.to_ssb_payload();
        
        match payload {
            SsbPayload::WorkloadProfile(p) => {
                assert_eq!(p.agent_role, "auditor");
                assert_eq!(p.phase_id, "phase-2");
                assert_eq!(p.tokens_in, 2000);
                assert_eq!(p.tokens_out, 1000);
            }
            _ => panic!("Expected WorkloadProfile payload"),
        }
    }

    #[test]
    fn test_workload_profile_builder() {
        let clock = OsClock;
        let mut builder = WorkloadProfileBuilder::new();
        builder.start_timing(&clock);
        // Simulate some work
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let profile = builder
            .agent_role(AgentRole::Executor)
            .phase_id("phase-1")
            .tokens_in(1000)
            .tokens_out(500)
            .vram_peak_mb(2048)
            .model_id("gemma4:e2b")
            .gpu_id(0)
            .build(&clock);

        assert!(profile.is_some());
        let profile = profile.unwrap();
        assert_eq!(profile.agent_role, AgentRole::Executor);
        assert!(profile.latency_ms >= 10); // At least 10ms elapsed
    }

    #[test]
    fn test_workload_profile_builder_missing_required() {
        let clock = OsClock;
        let builder = WorkloadProfileBuilder::new()
            .tokens_in(1000)
            .tokens_out(500);
        
        let profile = builder.build(&clock);
        assert!(profile.is_none()); // Missing required fields
    }
}

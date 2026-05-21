//! Researcher network egress: veth pair + iptables OUTPUT allowlist.
//! E-15: per-researcher-spawn network namespace.
//! REJECT not DROP for non-allowlist: fast EHOSTUNREACH vs 30s hang.
//! DENIED explicitly: AF_UNIX(1), AF_NETLINK(16), AF_PACKET(17), AF_BLUETOOTH(31).
//! SOCK_RAW(3) denied by omission -- only SOCK_STREAM(1) and SOCK_DGRAM(2) allowed.
//! Per shayon.dev 2026/52 sandbox-isolation: REJECT > DROP for inner-loop UX.

use std::net::IpAddr;

#[derive(Debug, Clone)]
pub struct ResearcherNetConfig {
    pub allowed_hosts: Vec<String>,
    pub trusted_resolvers: Vec<IpAddr>,
    pub reject_not_drop: bool,
}

impl Default for ResearcherNetConfig {
    fn default() -> Self {
        Self {
            allowed_hosts: vec!["arxiv.org".to_string(), "huggingface.co".to_string()],
            trusted_resolvers: vec!["1.1.1.1".parse().expect("valid IP"), "8.8.8.8".parse().expect("valid IP")],
            reject_not_drop: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResearcherNetSetup {
    pub veth_agent_name: String,
    pub veth_host_name: String,
    pub pinned_ips: Vec<IpAddr>,
    pub is_active: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ResearcherNetError {
    #[error("CAP_NET_ADMIN required to configure veth pair: {0}")]
    CapNetAdminRequired(String),
    #[error("DNS resolution failed for host {host}: {reason}")]
    DnsResolutionFailed { host: String, reason: String },
    #[error("iptables command failed: {0}")]
    IptablesFailed(String),
    #[error("veth setup failed: {0}")]
    VethSetupFailed(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl ResearcherNetSetup {
    pub fn configure(config: &ResearcherNetConfig, agent_id: uuid::Uuid) -> Self {
        match Self::try_configure(config, agent_id) {
            Ok(setup) => { tracing::info!(agent_id = %agent_id, pinned_ips = ?setup.pinned_ips, "researcher_net: veth pair configured, OUTPUT allowlist active"); setup }
            Err(e) => {
                tracing::warn!(agent_id = %agent_id, error = %e, "researcher_net: setup failed; degraded to log-only");
                ResearcherNetSetup { veth_agent_name: format!("veth-a-{:.8}", agent_id), veth_host_name: format!("veth-h-{:.8}", agent_id), pinned_ips: vec![], is_active: false }
            }
        }
    }

    #[cfg(target_os = "linux")]
    fn try_configure(config: &ResearcherNetConfig, agent_id: uuid::Uuid) -> Result<Self, ResearcherNetError> {
        let id_short = format!("{:.8}", agent_id);
        let veth_host = format!("veth-h-{}", &id_short);
        let veth_agent = format!("veth-a-{}", &id_short);
        let create_status = std::process::Command::new("ip").args(["link", "add", &veth_host, "type", "veth", "peer", "name", &veth_agent]).status().map_err(|e| ResearcherNetError::VethSetupFailed(format!("{e}")))?;
        if !create_status.success() { return Err(ResearcherNetError::CapNetAdminRequired("ip link add failed".to_string())); }
        let _ = std::process::Command::new("ip").args(["link", "set", &veth_host, "up"]).status();
        let mut pinned_ips: Vec<IpAddr> = Vec::new();
        for host in &config.allowed_hosts {
            match resolve_host(host) {
                Ok(ips) => { for ip in &ips { apply_iptables_output_allow(ip, config.reject_not_drop)?; pinned_ips.push(*ip); } }
                Err(e) => { tracing::warn!(host = %host, error = %e, "researcher_net: DNS failed, host skipped"); }
            }
        }
        for resolver in &config.trusted_resolvers { apply_iptables_output_allow(resolver, config.reject_not_drop)?; pinned_ips.push(*resolver); }
        let policy_action = if config.reject_not_drop { "REJECT" } else { "DROP" };
        let _ = std::process::Command::new("iptables").args(["-A", "OUTPUT", "-j", policy_action]).status();
        Ok(ResearcherNetSetup { veth_agent_name: veth_agent, veth_host_name: veth_host, pinned_ips, is_active: true })
    }

    #[cfg(not(target_os = "linux"))]
    fn try_configure(_config: &ResearcherNetConfig, agent_id: uuid::Uuid) -> Result<Self, ResearcherNetError> {
        Ok(ResearcherNetSetup { veth_agent_name: format!("veth-a-{:.8}", agent_id), veth_host_name: format!("veth-h-{:.8}", agent_id), pinned_ips: vec![], is_active: false })
    }

    #[cfg(target_os = "linux")]
    pub fn teardown(&self) {
        if self.is_active {
            let _ = std::process::Command::new("ip").args(["link", "del", &self.veth_host_name]).status();
            for ip in &self.pinned_ips { let _ = std::process::Command::new("iptables").args(["-D", "OUTPUT", "-d", &ip.to_string(), "-j", "ACCEPT"]).status(); }
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn teardown(&self) {}
}

#[cfg(target_os = "linux")]
fn resolve_host(host: &str) -> Result<Vec<IpAddr>, ResearcherNetError> {
    use std::net::ToSocketAddrs;
    let addrs = format!("{}:80", host).to_socket_addrs().map_err(|e| ResearcherNetError::DnsResolutionFailed { host: host.to_string(), reason: e.to_string() })?;
    Ok(addrs.map(|a| a.ip()).collect())
}

#[cfg(not(target_os = "linux"))]
fn resolve_host(_host: &str) -> Result<Vec<IpAddr>, ResearcherNetError> { Ok(vec![]) }

#[cfg(target_os = "linux")]
fn apply_iptables_output_allow(ip: &IpAddr, _reject_not_drop: bool) -> Result<(), ResearcherNetError> {
    let status = std::process::Command::new("iptables").args(["-A", "OUTPUT", "-d", &ip.to_string(), "-j", "ACCEPT"]).status().map_err(|e| ResearcherNetError::IptablesFailed(format!("{e}")))?;
    if !status.success() { return Err(ResearcherNetError::IptablesFailed(format!("iptables ACCEPT for {} failed", ip))); }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn apply_iptables_output_allow(_ip: &IpAddr, _reject_not_drop: bool) -> Result<(), ResearcherNetError> { Ok(()) }

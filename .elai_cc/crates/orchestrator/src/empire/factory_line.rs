//! FactoryLine protocol client outline.
//!
//! The connect / heartbeat / reconnect loop is wired in SHAPE here: the
//! struct fields and the configuration knobs are real, but the underlying
//! socket boot (`tokio::net::UnixStream` on Linux, `tokio::net::TcpStream`
//! on Windows / TCP-mode) is gated on Empire deployment. Every external-IO
//! seam is labelled `EMPIRE-PENDING` so a single follow-up patch can locate
//! and flip every wire-point. Until then the methods return
//! `EmpireBridgeError::NotConnected` so callers degrade gracefully.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::bridge::EmpireBridgeError;
use super::types::{FactoryCmd, FactoryLineCommand, FactoryLineResponse};

/// Transport configuration for the FactoryLine bus.
///
/// The Unix variant is conditionally available — UnixStream is cfg(unix) on
/// tokio so the wrapper struct is fine on all platforms but the actual
/// connect path only fires when `cfg(unix)` holds.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum TransportConfig {
    Unix { path: PathBuf },
    Tcp { host: String, port: u16 },
}

/// Knobs for the FactoryLine client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactoryLineConfig {
    pub transport: TransportConfig,
    pub heartbeat_interval_ms: u64,
    pub reconnect_initial_ms: u64,
    pub reconnect_max_ms: u64,
    pub event_rate_limit_per_sec: u32,
}

impl FactoryLineConfig {
    /// Default config pointing at the canonical Empire FactoryLine socket.
    /// Used by the bridge when no configuration is supplied; the host-side
    /// CLI is expected to override these knobs via elai_cc.toml `[empire]`
    /// once Empire is deployed (EMPIRE-PENDING: config loader).
    pub fn default_unix() -> Self {
        Self {
            transport: TransportConfig::Unix {
                path: PathBuf::from("/tmp/brain-factory.sock"),
            },
            heartbeat_interval_ms: 5_000,
            reconnect_initial_ms: 250,
            reconnect_max_ms: 30_000,
            event_rate_limit_per_sec: 100,
        }
    }

    pub fn heartbeat_interval(&self) -> Duration {
        Duration::from_millis(self.heartbeat_interval_ms)
    }
}

// ---------------------------------------------------------------------------
// FactoryLineTransport — async trait for the bus byte pipe
// ---------------------------------------------------------------------------

/// Cross-platform async byte pipe the FactoryLine client speaks newline-JSON
/// over. The trait is INTENTIONALLY object-safe (no generics, no Self in
/// return position) so the client can hold `Box<dyn FactoryLineTransport>`
/// once the activation patch lands.
///
/// Native `async fn` in trait per CLAUDE.md anti-stale-training rule (no
/// async-trait crate). Send bound on the trait so the boxed transport can
/// cross tokio task boundaries.
///
/// EMPIRE-PENDING: real `UnixTransport` / `TcpTransport` impls below return
/// `EmpireBridgeError::NotConnected` until Empire deploys. The trait shape
/// itself is host-side; the impl bodies require the live Empire socket
/// surface (or captured fixtures) which is not in tree.
pub trait FactoryLineTransport: Send {
    /// Open the underlying byte pipe. Idempotent on the success path.
    fn connect(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), EmpireBridgeError>> + Send + '_>>;
    /// Write `bytes` (one already-framed FactoryLine message, terminated by
    /// `\n`) to the pipe. Implementations MUST NOT add or strip the
    /// terminator; the framing is the client's responsibility.
    fn send(
        &mut self,
        bytes: Vec<u8>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), EmpireBridgeError>> + Send + '_>>;
    /// Read one newline-terminated frame from the pipe. Returns the raw
    /// bytes WITHOUT the trailing `\n`.
    fn recv(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, EmpireBridgeError>> + Send + '_>>;
}

/// EMPIRE-PENDING: Unix-domain-socket transport (Linux factory-brain box).
/// Gated on `cfg(unix)` so Windows builds compile but never instantiate it.
#[cfg(unix)]
pub struct UnixTransport {
    pub path: PathBuf,
}

#[cfg(unix)]
impl UnixTransport {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

#[cfg(unix)]
impl FactoryLineTransport for UnixTransport {
    fn connect(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), EmpireBridgeError>> + Send + '_>>
    {
        Box::pin(async move {
            tracing::debug!(
                path = %self.path.display(),
                "UnixTransport::connect — EMPIRE-PENDING: tokio UnixStream boot deferred until Empire deploys"
            );
            Err(EmpireBridgeError::NotConnected)
        })
    }
    fn send(
        &mut self,
        _bytes: Vec<u8>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), EmpireBridgeError>> + Send + '_>>
    {
        Box::pin(async move { Err(EmpireBridgeError::NotConnected) })
    }
    fn recv(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, EmpireBridgeError>> + Send + '_>>
    {
        Box::pin(async move { Err(EmpireBridgeError::NotConnected) })
    }
}

/// EMPIRE-PENDING: TCP transport (Windows + cross-platform fallback). Active
/// when `cfg(unix)` is false OR when the user explicitly configures a TCP
/// transport via `TransportConfig::Tcp` (typical Windows-side
/// configuration for the cross-platform factory-brain).
pub struct TcpTransport {
    pub host: String,
    pub port: u16,
}

impl TcpTransport {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

impl FactoryLineTransport for TcpTransport {
    fn connect(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), EmpireBridgeError>> + Send + '_>>
    {
        Box::pin(async move {
            tracing::debug!(
                host = %self.host,
                port = self.port,
                "TcpTransport::connect — EMPIRE-PENDING: tokio TcpStream boot deferred until Empire deploys"
            );
            Err(EmpireBridgeError::NotConnected)
        })
    }
    fn send(
        &mut self,
        _bytes: Vec<u8>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), EmpireBridgeError>> + Send + '_>>
    {
        Box::pin(async move { Err(EmpireBridgeError::NotConnected) })
    }
    fn recv(
        &mut self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, EmpireBridgeError>> + Send + '_>>
    {
        Box::pin(async move { Err(EmpireBridgeError::NotConnected) })
    }
}

/// Factory function: build the right transport for a `TransportConfig`.
/// Unix-side variant only constructs on Unix; on Windows a `TransportConfig
/// ::Unix` falls through to `EmpireBridgeError::Protocol` so the caller
/// surfaces a clear "wrong transport for this platform" error instead of
/// silently downgrading.
pub fn build_transport(
    cfg: &TransportConfig,
) -> Result<Box<dyn FactoryLineTransport>, EmpireBridgeError> {
    match cfg {
        #[cfg(unix)]
        TransportConfig::Unix { path } => Ok(Box::new(UnixTransport::new(path.clone()))),
        #[cfg(not(unix))]
        TransportConfig::Unix { .. } => Err(EmpireBridgeError::Protocol(
            "TransportConfig::Unix unavailable on non-unix platform; use TransportConfig::Tcp"
                .to_owned(),
        )),
        TransportConfig::Tcp { host, port } => {
            Ok(Box::new(TcpTransport::new(host.clone(), *port)))
        }
    }
}

/// FactoryLine protocol client shell.
///
/// Holds the config + a wall-clock-monotonic counter for built outbound
/// frame timestamps. The actual socket field is intentionally absent —
/// when EMPIRE-PENDING is flipped this struct grows a
/// `transport: Option<TransportHandle>` field and `connect()` / `send()`
/// gain bodies. Until then the public surface returns `NotConnected`.
pub struct FactoryLineClient {
    config: FactoryLineConfig,
}

impl FactoryLineClient {
    pub fn new(config: FactoryLineConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &FactoryLineConfig {
        &self.config
    }

    /// Build a FactoryLine command frame for the given verb. Wire format:
    /// newline-delimited JSON (exactly one `\n` per frame, NOT `\r\n`).
    pub fn build_command(&self, cmd: FactoryCmd, brain: impl Into<String>) -> FactoryLineCommand {
        FactoryLineCommand {
            cmd,
            v: 1,
            brain: brain.into(),
            dir: None,
            pid: Some(std::process::id()),
            ts: now_unix_secs(),
        }
    }

    /// Serialize a command to its on-wire bytes (`{...}\n`). Exact `\n`
    /// delimiter per phase-4J 4J.3 ADVERSARIAL.
    pub fn encode(&self, cmd: &FactoryLineCommand) -> Result<Vec<u8>, EmpireBridgeError> {
        let mut bytes =
            serde_json::to_vec(cmd).map_err(|e| EmpireBridgeError::Encode(e.to_string()))?;
        bytes.push(b'\n');
        Ok(bytes)
    }

    /// EMPIRE-PENDING: open the configured transport, perform the
    /// `FactoryCmd::Register` handshake, and start the heartbeat loop.
    /// Returns `NotConnected` until Empire is deployed.
    pub async fn connect(&mut self) -> Result<(), EmpireBridgeError> {
        tracing::debug!(
            transport = ?self.config.transport,
            "FactoryLineClient::connect — EMPIRE-PENDING: socket boot deferred until Empire deploys"
        );
        Err(EmpireBridgeError::NotConnected)
    }

    /// EMPIRE-PENDING: write one FactoryLine command + `\n` to the live
    /// transport. Returns `NotConnected` until Empire is deployed.
    pub async fn send(&mut self, _cmd: FactoryLineCommand) -> Result<(), EmpireBridgeError> {
        Err(EmpireBridgeError::NotConnected)
    }

    /// EMPIRE-PENDING: read one FactoryLine response. Returns
    /// `NotConnected` until Empire is deployed.
    pub async fn recv(&mut self) -> Result<FactoryLineResponse, EmpireBridgeError> {
        Err(EmpireBridgeError::NotConnected)
    }
}

fn now_unix_secs() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs_f64(),
        Err(_) => 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_terminates_with_single_newline() {
        let cfg = FactoryLineConfig::default_unix();
        let client = FactoryLineClient::new(cfg);
        let cmd = client.build_command(FactoryCmd::Heartbeat, "elai_cc");
        let bytes = client.encode(&cmd).expect("encode");
        assert!(bytes.ends_with(b"\n"));
        assert!(!bytes.ends_with(b"\r\n"));
        assert_eq!(bytes.iter().filter(|b| **b == b'\n').count(), 1);
    }

    #[test]
    fn build_command_uses_protocol_version_one() {
        let cfg = FactoryLineConfig::default_unix();
        let client = FactoryLineClient::new(cfg);
        let cmd = client.build_command(FactoryCmd::Register, "elai_cc");
        assert_eq!(cmd.v, 1);
        assert_eq!(cmd.cmd, FactoryCmd::Register);
        assert_eq!(cmd.brain, "elai_cc");
    }

    #[tokio::test]
    async fn connect_returns_not_connected_until_empire_deploys() {
        let cfg = FactoryLineConfig::default_unix();
        let mut client = FactoryLineClient::new(cfg);
        let res = client.connect().await;
        assert!(matches!(res, Err(EmpireBridgeError::NotConnected)));
    }

    #[tokio::test]
    async fn tcp_transport_returns_not_connected_until_empire_deploys() {
        let mut t = TcpTransport::new("127.0.0.1".to_string(), 9100);
        let r = t.connect().await;
        assert!(matches!(r, Err(EmpireBridgeError::NotConnected)));
        let s = t.send(b"{}\n".to_vec()).await;
        assert!(matches!(s, Err(EmpireBridgeError::NotConnected)));
        let rv = t.recv().await;
        assert!(matches!(rv, Err(EmpireBridgeError::NotConnected)));
    }

    #[tokio::test]
    async fn build_transport_picks_tcp_on_tcp_config() {
        let cfg = TransportConfig::Tcp {
            host: "127.0.0.1".to_string(),
            port: 9100,
        };
        let mut t = build_transport(&cfg).expect("tcp transport must build on all platforms");
        let r = t.connect().await;
        assert!(matches!(r, Err(EmpireBridgeError::NotConnected)));
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn unix_transport_returns_not_connected_until_empire_deploys() {
        let mut t = UnixTransport::new(PathBuf::from("/tmp/elai-empire-test.sock"));
        let r = t.connect().await;
        assert!(matches!(r, Err(EmpireBridgeError::NotConnected)));
    }
}

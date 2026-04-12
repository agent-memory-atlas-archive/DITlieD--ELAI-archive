use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};
use blake3;

use elai_core::ClockProvider;

use crate::agent_pool::AgentHandle;
use crate::ssb::{CheckpointPayload, Ssb, SsbPayload};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const STATE_FILE: &str = "state.msgpack";
const GRC_HISTORY_FILE: &str = "grc_history.msgpack";
const TOOL_RESULTS_FILE: &str = "tool_results.msgpack";
const OUTPUT_DRAFT_FILE: &str = "output_draft.msgpack";
const CHECKPOINT_TS_FILE: &str = "checkpoint_ts";

/// All checkpoint files that must be present for a valid checkpoint.
const CHECKPOINT_FILES: [&str; 5] = [
    STATE_FILE,
    GRC_HISTORY_FILE,
    TOOL_RESULTS_FILE,
    OUTPUT_DRAFT_FILE,
    CHECKPOINT_TS_FILE,
];

// ---------------------------------------------------------------------------
// ChannelKind: pre-write validation of channel references
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelKind {
    LastValue,
    Topic,
    Ephemeral,
    UntrackedValue,
}

// ---------------------------------------------------------------------------
// ChannelReference: a channel reference inside checkpoint data
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChannelReference {
    pub channel_id: String,
    pub kind: ChannelKind,
}

// ---------------------------------------------------------------------------
// GrcHistoryEntry: serializable snapshot of a single GRC step
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GrcHistoryEntry {
    pub step_type: String,
    pub content: String,
    pub grounding: String,
    pub topic: String,
}

// ---------------------------------------------------------------------------
// AgentCheckpointData: the full serializable checkpoint payload
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentCheckpointData {
    pub agent_status: String,
    pub grc_history: Vec<GrcHistoryEntry>,
    pub active_files: Vec<PathBuf>,
    pub tool_results: Vec<String>,
    pub output_draft: String,
    pub channel_references: Vec<ChannelReference>,
}

// ---------------------------------------------------------------------------
// CheckpointError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum CheckpointError {
    #[error("checkpoint I/O error for {1}: {0}")]
    Io(std::io::Error, String),

    #[error("checkpoint serialization error: {0}")]
    Serialization(#[from] rmp_serde::encode::Error),

    #[error("checkpoint deserialization error: {0}")]
    Deserialization(#[from] rmp_serde::decode::Error),

    #[error("checkpoint_ts is not valid UTF-8 or u64: {0}")]
    InvalidTimestamp(String),

    #[error("checkpoint contains UntrackedValue channel reference: {channel_id}")]
    UntrackedValueReference { channel_id: String },

    #[error("atomic rename failed for {0}")]
    AtomicRenameFailed(String),

    #[error("SSB write error during checkpoint: {0}")]
    SsbWrite(String),

    #[error("system clock error: cannot compute duration since UNIX_EPOCH")]
    ClockError,
}

// ---------------------------------------------------------------------------
// CheckpointManager<C: ClockProvider>
// ---------------------------------------------------------------------------

pub struct CheckpointManager<C: ClockProvider> {
    clock: C,
}

impl<C: ClockProvider> CheckpointManager<C> {
    pub fn new(clock: C) -> Self {
        Self { clock }
    }

    /// Returns the checkpoint directory for a given agent handle.
    fn checkpoint_dir(handle: &AgentHandle) -> PathBuf {
        handle.scratch_dir.join(handle.agent_id.to_string())
    }

    /// Validate that no channel reference has kind == UntrackedValue.
    fn validate_channels(data: &AgentCheckpointData) -> Result<(), CheckpointError> {
        for ch in &data.channel_references {
            if ch.kind == ChannelKind::UntrackedValue {
                return Err(CheckpointError::UntrackedValueReference {
                    channel_id: ch.channel_id.clone(),
                });
            }
        }
        Ok(())
    }

    /// Write content to a temp file, then atomically rename to the target.
    /// Windows-safe: removes existing target before rename (NTFS cannot rename over existing).
    async fn atomic_write(target: &Path, content: &[u8]) -> Result<(), CheckpointError> {
        let tmp_path = target.with_extension("tmp");
        let target_str = target.display().to_string();

        // Write to temp file
        if let Err(e) = tokio::fs::write(&tmp_path, content).await {
            return Err(CheckpointError::Io(e, format!("write tmp for {target_str}")));
        }

        // Remove existing target if present (NTFS atomic rename requirement)
        if tokio::fs::metadata(target).await.is_ok() {
            if let Err(e) = tokio::fs::remove_file(target).await {
                // Clean up temp file on failure
                let _ = tokio::fs::remove_file(&tmp_path).await;
                return Err(CheckpointError::Io(e, format!("remove existing {target_str}")));
            }
        }

        // Rename temp to target
        if let Err(e) = tokio::fs::rename(&tmp_path, target).await {
            // Clean up temp file on failure
            let _ = tokio::fs::remove_file(&tmp_path).await;
            return Err(CheckpointError::AtomicRenameFailed(
                format!("{target_str}: {e}"),
            ));
        }

        Ok(())
    }

    /// Compute BLAKE3 hex digest of a byte slice.
    fn content_hash(data: &[u8]) -> String {
        blake3::hash(data).to_hex().to_string()
    }

    /// Write a full checkpoint to disk and record metadata in the SSB.
    ///
    /// Creates 5 files atomically (temp+rename):
    /// - state.msgpack: serialized AgentCheckpointData
    /// - grc_history.msgpack: serialized grc_history field
    /// - tool_results.msgpack: serialized tool_results field
    /// - output_draft.msgpack: serialized output_draft field
    /// - checkpoint_ts: ASCII digits (u64 seconds since UNIX_EPOCH)
    ///
    /// SSB record contains metadata only (hashes + sizes + step_index + agent_id).
    /// All 5 files present = valid checkpoint; partial = skip on recovery.
    pub async fn write(
        &self,
        handle: &AgentHandle,
        data: &AgentCheckpointData,
        step_index: u64,
        ssb: &Ssb,
    ) -> Result<(), CheckpointError> {
        // Validate no UntrackedValue channels
        Self::validate_channels(data)?;

        let dir = Self::checkpoint_dir(handle);

        // Create checkpoint directory if absent
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|e| CheckpointError::Io(e, dir.display().to_string()))?;

        // Serialize the 4 msgpack files
        let state_bytes = rmp_serde::to_vec_named(data)?;
        let grc_bytes = rmp_serde::to_vec_named(&data.grc_history)?;
        let tool_bytes = rmp_serde::to_vec_named(&data.tool_results)?;
        let draft_bytes = rmp_serde::to_vec_named(&data.output_draft)?;

        // Compute checkpoint_ts via ClockProvider (entropy quarantine)
        let system_time = self.clock.system_now();
        let ts_secs = system_time
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CheckpointError::ClockError)?
            .as_secs();
        let ts_bytes = ts_secs.to_string().into_bytes();

        // Write all 5 files atomically
        Self::atomic_write(&dir.join(STATE_FILE), &state_bytes).await?;
        Self::atomic_write(&dir.join(GRC_HISTORY_FILE), &grc_bytes).await?;
        Self::atomic_write(&dir.join(TOOL_RESULTS_FILE), &tool_bytes).await?;
        Self::atomic_write(&dir.join(OUTPUT_DRAFT_FILE), &draft_bytes).await?;
        Self::atomic_write(&dir.join(CHECKPOINT_TS_FILE), &ts_bytes).await?;

        // Compute hashes for SSB metadata
        let agent_state_hash = Self::content_hash(&state_bytes);
        let grc_history_hash = Self::content_hash(&grc_bytes);

        // Collect file sizes for SSB payload
        let file_sizes = vec![
            (STATE_FILE.to_owned(), state_bytes.len() as u64),
            (GRC_HISTORY_FILE.to_owned(), grc_bytes.len() as u64),
            (TOOL_RESULTS_FILE.to_owned(), tool_bytes.len() as u64),
            (OUTPUT_DRAFT_FILE.to_owned(), draft_bytes.len() as u64),
            (CHECKPOINT_TS_FILE.to_owned(), ts_bytes.len() as u64),
        ];

        let agent_id_str = handle.agent_id.to_string();

        // Write SSB checkpoint record (metadata only)
        let payload = SsbPayload::Checkpoint(CheckpointPayload {
            agent_state_hash,
            grc_history_hash,
            agent_id: agent_id_str.clone(),
            step_index,
            file_sizes,
        });

        // step_index fits in i64 for all practical values; SSB schema uses INTEGER.
        let superstep = i64::try_from(step_index).unwrap_or(i64::MAX);
        ssb.write_payload(&agent_id_str, superstep, payload, vec![])
            .await
            .map_err(|e| CheckpointError::SsbWrite(format!("checkpoint for agent {agent_id_str}: {e}")))?;

        Ok(())
    }

    /// Read the latest checkpoint from disk.
    ///
    /// Returns `Ok(None)` if the checkpoint directory does not exist or any of
    /// the 5 required files is missing. Returns `Ok(Some(data))` only when all
    /// 5 files are present and the state file deserializes successfully.
    pub async fn read_latest(
        handle: &AgentHandle,
    ) -> Result<Option<AgentCheckpointData>, CheckpointError> {
        let dir = Self::checkpoint_dir(handle);

        // Directory absent: no checkpoint
        if tokio::fs::metadata(&dir).await.is_err() {
            return Ok(None);
        }

        // Check all 5 files exist (partial = no valid checkpoint)
        for file_name in &CHECKPOINT_FILES {
            let path = dir.join(file_name);
            if tokio::fs::metadata(&path).await.is_err() {
                return Ok(None);
            }
        }

        // Ignore any .tmp files (partial writes from prior crashes)

        // Read and deserialize state. Corrupt/truncated files => Ok(None),
        // matching the spec: "partial or invalid checkpoint = skip on recovery."
        let state_bytes = match tokio::fs::read(dir.join(STATE_FILE)).await {
            Ok(b) => b,
            Err(_) => return Ok(None),
        };

        let data: AgentCheckpointData = match rmp_serde::from_slice(&state_bytes) {
            Ok(d) => d,
            Err(_) => return Ok(None),
        };

        // Verify checkpoint_ts is valid ASCII digits (consistency check).
        // Invalid ts => treat as corrupt checkpoint => Ok(None).
        let ts_bytes = match tokio::fs::read(dir.join(CHECKPOINT_TS_FILE)).await {
            Ok(b) => b,
            Err(_) => return Ok(None),
        };

        let ts_valid = String::from_utf8(ts_bytes)
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .is_some();

        if !ts_valid {
            return Ok(None);
        }

        Ok(Some(data))
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_pool::{AgentRole, AgentStatus};
    use crate::ssb::{Ssb, SsbActionType, SsbConfig};
    use elai_core::{ClockProvider, SeededClock};
    use elai_reasoning::{AgentGrcConfig, BasicGrcEngine, GrcBudget, GrcState, GroundingLevel};
    use std::collections::HashMap;
    use std::sync::Arc;
    use uuid::Uuid;

    fn test_grc_config() -> AgentGrcConfig {
        AgentGrcConfig {
            role: "executor".into(),
            budget: GrcBudget {
                max_calls: 5,
                max_chars: 1000,
                calls_used: 0,
                chars_used: 0,
            },
            required_steps: vec![GrcState::Observe, GrcState::PlanAction, GrcState::Verify],
            grounding_level: GroundingLevel::Uncertain,
            csq_required: false,
            allowed_methods: vec![],
            transition_overrides: HashMap::new(),
            require_scope_enumeration: false,
        }
    }

    fn test_handle(tmp_dir: &Path) -> AgentHandle {
        let grc_config = test_grc_config();
        AgentHandle {
            agent_id: Uuid::nil(),
            role: AgentRole::Executor,
            status: AgentStatus::Running,
            task: "test-task".into(),
            active_files: vec![],
            grc_engine: BasicGrcEngine::with_config(grc_config.clone()),
            grc_config,
            scratch_dir: tmp_dir.to_path_buf(),
            sandbox_handle: None,
        }
    }

    fn test_data() -> AgentCheckpointData {
        AgentCheckpointData {
            agent_status: "running".into(),
            grc_history: vec![GrcHistoryEntry {
                step_type: "observe".into(),
                content: "test observation".into(),
                grounding: "file:test.rs:L1".into(),
                topic: "default".into(),
            }],
            active_files: vec![PathBuf::from("src/main.rs")],
            tool_results: vec!["result-1".into()],
            output_draft: "draft output".into(),
            channel_references: vec![],
        }
    }

    fn test_data_with_channel(kind: ChannelKind) -> AgentCheckpointData {
        let mut data = test_data();
        data.channel_references.push(ChannelReference {
            channel_id: "ch-001".into(),
            kind,
        });
        data
    }

    async fn make_ssb(clock: SeededClock) -> Ssb {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test.db");
        // Keep dir alive by leaking (test only)
        let db_path_owned = db_path.to_path_buf();
        std::mem::forget(dir);
        Ssb::new(
            SsbConfig {
                db_path: db_path_owned,
                project_id: "test-project".into(),
                session_id: "test-session".into(),
                pool_size: None,
            },
            Arc::new(clock),
        )
        .expect("ssb creation")
    }

    // -----------------------------------------------------------------------
    // Happy path tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_write_creates_five_files() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mgr = CheckpointManager::new(clock);

        mgr.write(&handle, &test_data(), 1, &ssb).await.expect("write");

        let ckpt_dir = tmp.path().join(handle.agent_id.to_string());
        for name in &CHECKPOINT_FILES {
            assert!(
                ckpt_dir.join(name).exists(),
                "expected checkpoint file {name} to exist"
            );
        }
    }

    #[tokio::test]
    async fn test_checkpoint_ts_is_ascii_digits() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mgr = CheckpointManager::new(clock);

        mgr.write(&handle, &test_data(), 0, &ssb).await.expect("write");

        let ckpt_dir = tmp.path().join(handle.agent_id.to_string());
        let ts_content = std::fs::read_to_string(ckpt_dir.join(CHECKPOINT_TS_FILE))
            .expect("read ts");
        let parsed: u64 = ts_content.parse().expect("parse as u64");
        assert!(parsed > 0, "checkpoint_ts should be positive");
    }

    #[tokio::test]
    async fn test_read_latest_roundtrip() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mgr = CheckpointManager::new(clock);
        let data = test_data();

        mgr.write(&handle, &data, 1, &ssb).await.expect("write");

        let read_back = CheckpointManager::<SeededClock>::read_latest(&handle)
            .await
            .expect("read_latest")
            .expect("should be Some");

        assert_eq!(read_back, data);
    }

    #[tokio::test]
    async fn test_atomic_write_no_partial_on_tmp_path() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mgr = CheckpointManager::new(clock);

        mgr.write(&handle, &test_data(), 0, &ssb).await.expect("write");

        // After a successful write, no .tmp files should remain
        let ckpt_dir = tmp.path().join(handle.agent_id.to_string());
        let entries: Vec<_> = std::fs::read_dir(&ckpt_dir)
            .expect("readdir")
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "tmp")
            })
            .collect();
        assert!(entries.is_empty(), "no .tmp files should remain after write");
    }

    #[tokio::test]
    async fn test_ssb_write_called_once() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mut rx = ssb.subscribe();
        let mgr = CheckpointManager::new(clock);

        mgr.write(&handle, &test_data(), 7, &ssb).await.expect("write");

        // SSB should have broadcast exactly one entry
        let entry = rx.recv().await.expect("should receive entry");
        assert_eq!(entry.action_type, SsbActionType::Checkpoint);
    }

    #[tokio::test]
    async fn test_checkpoint_payload_contains_step_index() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mut rx = ssb.subscribe();
        let mgr = CheckpointManager::new(clock);

        mgr.write(&handle, &test_data(), 42, &ssb).await.expect("write");

        let entry = rx.recv().await.expect("entry");
        let payload: SsbPayload = rmp_serde::from_slice(&entry.payload).expect("deser");
        if let SsbPayload::Checkpoint(cp) = payload {
            assert_eq!(cp.step_index, 42);
        } else {
            panic!("expected Checkpoint payload");
        }
    }

    #[tokio::test]
    async fn test_checkpoint_payload_contains_file_sizes() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mut rx = ssb.subscribe();
        let mgr = CheckpointManager::new(clock);

        mgr.write(&handle, &test_data(), 1, &ssb).await.expect("write");

        let entry = rx.recv().await.expect("entry");
        let payload: SsbPayload = rmp_serde::from_slice(&entry.payload).expect("deser");
        if let SsbPayload::Checkpoint(cp) = payload {
            assert_eq!(cp.file_sizes.len(), 5, "should have 5 file sizes");
            let names: Vec<&str> = cp.file_sizes.iter().map(|(n, _)| n.as_str()).collect();
            assert!(names.contains(&STATE_FILE));
            assert!(names.contains(&CHECKPOINT_TS_FILE));
        } else {
            panic!("expected Checkpoint payload");
        }
    }

    // -----------------------------------------------------------------------
    // Error path tests
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_untracked_value_reference_rejected() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mgr = CheckpointManager::new(clock);

        let data = test_data_with_channel(ChannelKind::UntrackedValue);
        let result = mgr.write(&handle, &data, 0, &ssb).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            matches!(err, CheckpointError::UntrackedValueReference { .. }),
            "expected UntrackedValueReference, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_accepted_channel_kinds() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mgr = CheckpointManager::new(clock);

        for kind in [ChannelKind::LastValue, ChannelKind::Topic, ChannelKind::Ephemeral] {
            let data = test_data_with_channel(kind);
            mgr.write(&handle, &data, 0, &ssb)
                .await
                .unwrap_or_else(|e| panic!("kind {kind:?} should be accepted, got: {e}"));
        }
    }

    #[tokio::test]
    async fn test_read_latest_returns_none_on_empty_dir() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());

        let result = CheckpointManager::<SeededClock>::read_latest(&handle)
            .await
            .expect("should not error");
        assert!(result.is_none(), "empty dir should return None");
    }

    #[tokio::test]
    async fn test_read_latest_returns_none_on_nonexistent_dir() {
        let grc_config = test_grc_config();
        let handle = AgentHandle {
            agent_id: Uuid::nil(),
            role: AgentRole::Executor,
            status: AgentStatus::Running,
            task: "test".into(),
            active_files: vec![],
            grc_engine: BasicGrcEngine::with_config(grc_config.clone()),
            grc_config,
            scratch_dir: PathBuf::from("/nonexistent/path/that/does/not/exist"),
            sandbox_handle: None,
        };

        let result = CheckpointManager::<SeededClock>::read_latest(&handle)
            .await
            .expect("should not error");
        assert!(result.is_none(), "nonexistent dir should return None");
    }

    #[tokio::test]
    async fn test_partial_checkpoint_ignored_on_read() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mgr = CheckpointManager::new(clock);

        // Write a full checkpoint
        mgr.write(&handle, &test_data(), 0, &ssb).await.expect("write");

        // Delete one file to simulate partial
        let ckpt_dir = tmp.path().join(handle.agent_id.to_string());
        std::fs::remove_file(ckpt_dir.join(GRC_HISTORY_FILE)).expect("remove");

        let result = CheckpointManager::<SeededClock>::read_latest(&handle)
            .await
            .expect("should not error");
        assert!(result.is_none(), "partial checkpoint should return None");
    }

    // -----------------------------------------------------------------------
    // Clock quarantine contract test
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_clock_provider_used_not_system_time() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        // SeededClock epoch = UNIX_EPOCH + 1_700_000_000s, tick starts at 0.
        // system_now() = epoch + 0ms = 1_700_000_000 seconds since UNIX_EPOCH.
        let clock = SeededClock::new();
        let expected_ts = clock
            .system_now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_secs();

        let ssb = make_ssb(SeededClock::new()).await;
        let mgr = CheckpointManager::new(clock);
        mgr.write(&handle, &test_data(), 0, &ssb).await.expect("write");

        let ckpt_dir = tmp.path().join(handle.agent_id.to_string());
        let ts_content = std::fs::read_to_string(ckpt_dir.join(CHECKPOINT_TS_FILE))
            .expect("read ts");
        let actual_ts: u64 = ts_content.parse().expect("parse");

        // SeededClock epoch is 1_700_000_000, so checkpoint_ts should be that value.
        // If code used SystemTime::now() directly, this would be a current wall-clock time
        // (much larger than 1_700_000_000 + 0).
        assert_eq!(
            actual_ts, expected_ts,
            "checkpoint_ts must come from ClockProvider, not SystemTime::now()"
        );
    }

    // -----------------------------------------------------------------------
    // Deterministic serialization test
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_serialization_deterministic() {
        let data = test_data();
        let bytes_1 = rmp_serde::to_vec_named(&data).expect("ser1");
        let bytes_2 = rmp_serde::to_vec_named(&data).expect("ser2");
        assert_eq!(bytes_1, bytes_2, "same input must produce same msgpack bytes");
    }

    // -----------------------------------------------------------------------
    // Overwrite test (write twice, read latest)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_write_overwrites_previous() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mgr = CheckpointManager::new(clock);

        let data1 = test_data();
        mgr.write(&handle, &data1, 1, &ssb).await.expect("write1");

        let mut data2 = test_data();
        data2.output_draft = "updated draft".into();
        mgr.write(&handle, &data2, 2, &ssb).await.expect("write2");

        let read_back = CheckpointManager::<SeededClock>::read_latest(&handle)
            .await
            .expect("read")
            .expect("some");
        assert_eq!(read_back.output_draft, "updated draft");
    }

    // -----------------------------------------------------------------------
    // Path with spaces test (adversarial)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_path_with_spaces() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dir_with_spaces = tmp.path().join("path with spaces");
        std::fs::create_dir_all(&dir_with_spaces).expect("mkdir");

        let grc_config = test_grc_config();
        let handle = AgentHandle {
            agent_id: Uuid::nil(),
            role: AgentRole::Executor,
            status: AgentStatus::Running,
            task: "test".into(),
            active_files: vec![],
            grc_engine: BasicGrcEngine::with_config(grc_config.clone()),
            grc_config,
            scratch_dir: dir_with_spaces,
            sandbox_handle: None,
        };

        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mgr = CheckpointManager::new(clock);

        mgr.write(&handle, &test_data(), 0, &ssb)
            .await
            .expect("write with spaces in path");

        let read_back = CheckpointManager::<SeededClock>::read_latest(&handle)
            .await
            .expect("read")
            .expect("some");
        assert_eq!(read_back, test_data());
    }

    // -----------------------------------------------------------------------
    // Validate channel references: multiple channels, one bad
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_mixed_channels_with_untracked_rejected() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mgr = CheckpointManager::new(clock);

        let mut data = test_data();
        data.channel_references = vec![
            ChannelReference {
                channel_id: "ch-ok".into(),
                kind: ChannelKind::LastValue,
            },
            ChannelReference {
                channel_id: "ch-bad".into(),
                kind: ChannelKind::UntrackedValue,
            },
        ];

        let result = mgr.write(&handle, &data, 0, &ssb).await;
        assert!(result.is_err());
        if let Err(CheckpointError::UntrackedValueReference { channel_id }) = result {
            assert_eq!(channel_id, "ch-bad");
        } else {
            panic!("expected UntrackedValueReference with ch-bad");
        }
    }

    // -----------------------------------------------------------------------
    // Corrupt file returns Ok(None) — BLOCK-2 fix verification
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn test_corrupt_state_file_returns_none() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mgr = CheckpointManager::new(clock);

        // Write valid checkpoint
        mgr.write(&handle, &test_data(), 0, &ssb).await.expect("write");

        // Corrupt the state file
        let ckpt_dir = tmp.path().join(handle.agent_id.to_string());
        std::fs::write(ckpt_dir.join(STATE_FILE), b"corrupt garbage data")
            .expect("corrupt write");

        // read_latest must return Ok(None), not Err
        let result = CheckpointManager::<SeededClock>::read_latest(&handle)
            .await
            .expect("should be Ok, not Err");
        assert!(result.is_none(), "corrupt state file must return None");
    }

    #[tokio::test]
    async fn test_corrupt_timestamp_returns_none() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let handle = test_handle(tmp.path());
        let clock = SeededClock::new();
        let ssb = make_ssb(SeededClock::new()).await;
        let mgr = CheckpointManager::new(clock);

        mgr.write(&handle, &test_data(), 0, &ssb).await.expect("write");

        // Corrupt the timestamp file
        let ckpt_dir = tmp.path().join(handle.agent_id.to_string());
        std::fs::write(ckpt_dir.join(CHECKPOINT_TS_FILE), b"not-a-number")
            .expect("corrupt write");

        let result = CheckpointManager::<SeededClock>::read_latest(&handle)
            .await
            .expect("should be Ok, not Err");
        assert!(result.is_none(), "corrupt timestamp must return None");
    }
}

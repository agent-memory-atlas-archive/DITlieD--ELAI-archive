#![deny(dead_code, unused_imports)]

//! BSP channel types for inter-agent state sharing.
//!
//! Four channel types: LastValue, Topic, EphemeralValue, UntrackedValue.
//! ChannelRegistry trait defines the 6-method interface (locked after Phase 3D.1).
//! ChannelState enum (3 variants) handles checkpoint serialization — UntrackedValue
//! is excluded (in-memory only, lost on crash).

use std::any::Any;
use std::collections::HashMap;
use std::marker::PhantomData;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Named constants (no magic numbers)
// ---------------------------------------------------------------------------

/// Default memory limit per UntrackedValue channel (10 MB).
pub const UNTRACKED_DEFAULT_LIMIT_BYTES: usize = 10 * 1024 * 1024;

/// Force-clear multiplier: clear when usage exceeds limit * this value.
pub const UNTRACKED_FORCE_CLEAR_MULTIPLIER: u64 = 2;

// ---------------------------------------------------------------------------
// DeepSizeOf trait (hand-rolled, no external dep)
// ---------------------------------------------------------------------------

/// Measures true heap allocation size, not just stack representation.
/// `std::mem::size_of_val` returns 24 bytes for any `Vec<u8>` regardless of
/// heap allocation — this trait returns stack + heap sizes.
pub trait DeepSizeOf: Send + Sync + 'static {
    fn deep_size_of(&self) -> usize;
}

impl DeepSizeOf for u8 {
    fn deep_size_of(&self) -> usize {
        std::mem::size_of::<u8>()
    }
}

impl DeepSizeOf for u32 {
    fn deep_size_of(&self) -> usize {
        std::mem::size_of::<u32>()
    }
}

impl DeepSizeOf for u64 {
    fn deep_size_of(&self) -> usize {
        std::mem::size_of::<u64>()
    }
}

impl DeepSizeOf for i64 {
    fn deep_size_of(&self) -> usize {
        std::mem::size_of::<i64>()
    }
}

impl DeepSizeOf for usize {
    fn deep_size_of(&self) -> usize {
        std::mem::size_of::<usize>()
    }
}

impl DeepSizeOf for bool {
    fn deep_size_of(&self) -> usize {
        std::mem::size_of::<bool>()
    }
}

impl DeepSizeOf for f64 {
    fn deep_size_of(&self) -> usize {
        std::mem::size_of::<f64>()
    }
}

impl DeepSizeOf for String {
    fn deep_size_of(&self) -> usize {
        std::mem::size_of::<String>() + self.capacity()
    }
}

impl<T: DeepSizeOf> DeepSizeOf for Vec<T> {
    fn deep_size_of(&self) -> usize {
        let header = std::mem::size_of::<Vec<T>>();
        // Use capacity (not len) to measure actual heap allocation.
        // A Vec::with_capacity(N) allocates N * size_of::<T>() even if len is 0.
        // Sum deep sizes for populated elements; remaining capacity slots are
        // accounted for by the capacity * size_of::<T>() floor.
        let element_deep: usize = self.iter().map(DeepSizeOf::deep_size_of).sum();
        let alloc_floor = self.capacity() * std::mem::size_of::<T>();
        header + std::cmp::max(element_deep, alloc_floor)
    }
}


// ---------------------------------------------------------------------------
// ChannelId — typed phantom key
// ---------------------------------------------------------------------------

/// Typed channel identifier. `T` is the channel type (e.g., `LastValue<String>`).
/// The phantom type prevents reading a `LastValue` channel with a `Topic` id.
pub struct ChannelId<T> {
    name: String,
    accumulate: Option<bool>,
    _phantom: PhantomData<T>,
}

impl<T> ChannelId<T> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            accumulate: None,
            _phantom: PhantomData,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

// Topic-specific constructor for accumulate flag
impl<T: Clone + Serialize + DeserializeOwned> ChannelId<Topic<T>> {
    pub fn new_with_accumulate(name: &str, accumulate: bool) -> Self {
        Self {
            name: name.to_owned(),
            accumulate: Some(accumulate),
            _phantom: PhantomData,
        }
    }

    pub fn accumulate(&self) -> bool {
        self.accumulate.unwrap_or(true)
    }
}

impl<T> Clone for ChannelId<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            accumulate: self.accumulate,
            _phantom: PhantomData,
        }
    }
}

impl<T> PartialEq for ChannelId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl<T> Eq for ChannelId<T> {}

impl<T> std::hash::Hash for ChannelId<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

impl<T> std::fmt::Debug for ChannelId<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChannelId")
            .field("name", &self.name)
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}

// ---------------------------------------------------------------------------
// ChannelValue — read result
// ---------------------------------------------------------------------------

/// Result of reading a channel. `Missing` means the channel has never been
/// written to, or was cleared (ephemeral with no writes this superstep).
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelValue<T> {
    Present(T),
    Missing,
}

// ---------------------------------------------------------------------------
// ChannelState — checkpoint serialization (exactly 3 variants)
// ---------------------------------------------------------------------------

/// Checkpoint-serializable channel state. UntrackedValue is intentionally
/// excluded — it is in-memory only, lost on crash by design.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChannelState {
    LastValue(Vec<u8>),
    Topic(Vec<Vec<u8>>),
    Ephemeral(Option<Vec<u8>>),
}

// ---------------------------------------------------------------------------
// Channel type markers
// ---------------------------------------------------------------------------

/// Single value, last writer wins. Writer ordering by SSB seq (monotonic).
/// Read returns the value with the highest SSB seq.
pub struct LastValue<T: Clone + Serialize + DeserializeOwned> {
    _phantom: PhantomData<T>,
}

/// Append-only log. `accumulate=true` persists across supersteps.
/// `accumulate=false` clears at superstep boundary (after snapshot, before writes).
pub struct Topic<T: Clone + Serialize + DeserializeOwned> {
    _phantom: PhantomData<T>,
}

/// Fire-and-forget signal. Cleared to Missing on any superstep where
/// `write_count == 0`. Write count reset at superstep start (saturating_add).
pub struct EphemeralValue<T: Clone + Serialize + DeserializeOwned> {
    _phantom: PhantomData<T>,
}

/// In-memory only, never serialized to checkpoint. Lost on crash.
/// `T: DeepSizeOf` required for R19 memory monitoring.
pub struct UntrackedValue<T: DeepSizeOf> {
    _phantom: PhantomData<T>,
}

// ---------------------------------------------------------------------------
// ChannelWrite — pending write
// ---------------------------------------------------------------------------

/// A pending write to a channel. Consumed by the registry during a superstep.
#[derive(Debug, Clone)]
pub struct ChannelWrite<T> {
    pub id: ChannelId<T>,
    pub value: T,
    pub ssb_seq: i64,
}

// ---------------------------------------------------------------------------
// ChannelSnapshot — frozen read-only view for scheduling
// ---------------------------------------------------------------------------

/// Frozen read-only snapshot of all serializable channel states.
/// Created by `snapshot_for_scheduling()`, consumed within one
/// `prepare_next_tasks` call. UntrackedValue excluded.
#[derive(Debug, Clone)]
pub struct ChannelSnapshot {
    inner: HashMap<String, ChannelState>,
}

impl ChannelSnapshot {
    pub fn get(&self, name: &str) -> Option<&ChannelState> {
        self.inner.get(name)
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &ChannelState)> {
        self.inner.iter()
    }
}

// ---------------------------------------------------------------------------
// ChannelError — thiserror enum
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
    #[error("type mismatch for channel '{channel_name}': expected {expected}")]
    TypeMismatch {
        channel_name: String,
        expected: &'static str,
    },

    #[error("channel not found: '{channel_name}'")]
    NotFound { channel_name: String },

    #[error("accumulate flag conflict for topic '{channel_name}': existing={existing}, requested={requested}")]
    AccumulateConflict {
        channel_name: String,
        existing: bool,
        requested: bool,
    },

    #[error("serialization error: {0}")]
    SerializationError(#[from] rmp_serde::encode::Error),

    #[error("deserialization error: {0}")]
    DeserializationError(#[from] rmp_serde::decode::Error),
}

// ChannelError must be Send + Sync for use across async boundaries
static_assertions_send_sync!(ChannelError);

// Macro to assert Send + Sync at compile time without extra dep
macro_rules! static_assertions_send_sync {
    ($t:ty) => {
        const _: () = {
            fn _assert_send<T: Send>() {}
            fn _assert_sync<T: Sync>() {}
            fn _assert() {
                _assert_send::<$t>();
                _assert_sync::<$t>();
            }
        };
    };
}
use static_assertions_send_sync;

// ---------------------------------------------------------------------------
// ChannelRegistry trait — LOCKED after Phase 3D.1
// ---------------------------------------------------------------------------

/// Interface for channel state management within a BSP superstep.
/// Typed read/write methods per channel kind, plus snapshot, clear, and
/// size-measurement methods. Implementations own all channel data.
/// The trait is sealed after 3D.1: adding methods requires a plan amendment.
pub trait ChannelRegistry {
    /// Read the current value of a LastValue channel.
    fn read<T: Clone + Serialize + DeserializeOwned>(
        &self,
        id: &ChannelId<LastValue<T>>,
    ) -> Result<ChannelValue<T>, ChannelError>;

    /// Read the current values of a Topic channel.
    fn read_topic<T: Clone + Serialize + DeserializeOwned>(
        &self,
        id: &ChannelId<Topic<T>>,
    ) -> Result<ChannelValue<Vec<T>>, ChannelError>;

    /// Read the current value of an ephemeral channel.
    fn read_ephemeral<T: Clone + Serialize + DeserializeOwned>(
        &self,
        id: &ChannelId<EphemeralValue<T>>,
    ) -> Result<ChannelValue<T>, ChannelError>;

    /// Read the current value of an untracked channel.
    fn read_untracked<T: DeepSizeOf + Clone>(
        &self,
        id: &ChannelId<UntrackedValue<T>>,
    ) -> ChannelValue<T>;

    /// Write a value to a LastValue channel. Higher SSB seq wins.
    fn write<T: Clone + Serialize + DeserializeOwned>(
        &mut self,
        id: &ChannelId<LastValue<T>>,
        value: T,
        ssb_seq: i64,
    ) -> Result<(), ChannelError>;

    /// Append a value to a Topic channel.
    fn write_topic<T: Clone + Serialize + DeserializeOwned>(
        &mut self,
        id: &ChannelId<Topic<T>>,
        value: T,
        ssb_seq: i64,
    ) -> Result<(), ChannelError>;

    /// Write a value to an EphemeralValue channel.
    fn write_ephemeral<T: Clone + Serialize + DeserializeOwned>(
        &mut self,
        id: &ChannelId<EphemeralValue<T>>,
        value: T,
        ssb_seq: i64,
    ) -> Result<(), ChannelError>;

    /// Write a value to an UntrackedValue channel.
    fn write_untracked<T: DeepSizeOf + Clone>(
        &mut self,
        id: &ChannelId<UntrackedValue<T>>,
        value: T,
    ) -> Result<(), ChannelError>;

    /// Create a frozen read-only snapshot of all serializable channels.
    /// UntrackedValue channels are excluded.
    fn snapshot_for_scheduling(&self) -> ChannelSnapshot;

    /// Clear ephemeral channels that had zero writes this superstep.
    /// Returns the names of cleared channels so the caller (superstep loop)
    /// can record clearing decisions in SSB for deterministic crash recovery.
    fn clear_ephemeral(&mut self) -> Vec<String>;

    /// Force-clear an untracked channel by erased id. Returns true if
    /// the channel existed and was cleared.
    fn force_clear_untracked(&mut self, id: &ChannelId<()>) -> bool;

    /// Measure the deep heap size of an untracked channel. Returns 0
    /// if the channel does not exist or is empty.
    fn deep_size_untracked(&self, id: &ChannelId<()>) -> usize;
}

// ---------------------------------------------------------------------------
// Internal storage types for InMemoryChannelRegistry
// ---------------------------------------------------------------------------

/// LastValue storage: serialized bytes + SSB seq for ordering.
struct LastValueEntry {
    bytes: Vec<u8>,
    ssb_seq: i64,
}

/// Topic storage: ordered entries + accumulate flag.
struct TopicEntry {
    entries: Vec<(Vec<u8>, i64)>,
    accumulate: bool,
}

/// Ephemeral storage: optional serialized bytes + write counter.
struct EphemeralEntry {
    bytes: Option<Vec<u8>>,
    write_count: u32,
}

/// Type-erased untracked value with size measurement.
struct ErasedUntracked {
    value: Box<dyn Any + Send + Sync>,
    size_fn: Box<dyn Fn(&dyn Any) -> usize + Send + Sync>,
}

// ---------------------------------------------------------------------------
// InMemoryChannelRegistry — concrete implementation
// ---------------------------------------------------------------------------

/// In-memory channel registry for BSP supersteps.
/// Separate HashMaps per channel type for type safety and performance.
pub struct InMemoryChannelRegistry {
    last_values: HashMap<String, LastValueEntry>,
    topics: HashMap<String, TopicEntry>,
    ephemerals: HashMap<String, EphemeralEntry>,
    untracked: HashMap<String, ErasedUntracked>,
}

impl InMemoryChannelRegistry {
    pub fn new() -> Self {
        Self {
            last_values: HashMap::new(),
            topics: HashMap::new(),
            ephemerals: HashMap::new(),
            untracked: HashMap::new(),
        }
    }

    /// Reset all ephemeral write counters to 0. Call at superstep start.
    pub fn reset_ephemeral_write_counts(&mut self) {
        for entry in self.ephemerals.values_mut() {
            entry.write_count = 0;
        }
    }

    /// Iterate all untracked channels, returning (name, deep_size_bytes) pairs.
    /// Does not expose ErasedUntracked internals — callers see only names and sizes.
    pub fn iter_untracked_sizes(&self) -> Vec<(String, usize)> {
        self.untracked
            .iter()
            .map(|(name, erased)| {
                let size = (erased.size_fn)(&*erased.value);
                (name.clone(), size)
            })
            .collect()
    }

    /// Clear non-accumulating Topic channels. Call at superstep boundary
    /// after snapshot but before new writes.
    pub fn clear_non_accumulating_topics(&mut self) {
        for entry in self.topics.values_mut() {
            if !entry.accumulate {
                entry.entries.clear();
            }
        }
    }
}

impl Default for InMemoryChannelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelRegistry for InMemoryChannelRegistry {
    fn read<T: Clone + Serialize + DeserializeOwned>(
        &self,
        id: &ChannelId<LastValue<T>>,
    ) -> Result<ChannelValue<T>, ChannelError> {
        match self.last_values.get(id.name()) {
            Some(entry) => {
                let v: T = rmp_serde::from_slice(&entry.bytes)?;
                Ok(ChannelValue::Present(v))
            }
            None => Ok(ChannelValue::Missing),
        }
    }

    fn read_topic<T: Clone + Serialize + DeserializeOwned>(
        &self,
        id: &ChannelId<Topic<T>>,
    ) -> Result<ChannelValue<Vec<T>>, ChannelError> {
        match self.topics.get(id.name()) {
            Some(entry) if !entry.entries.is_empty() => {
                let mut values = Vec::with_capacity(entry.entries.len());
                for (bytes, _seq) in &entry.entries {
                    let v: T = rmp_serde::from_slice(bytes)?;
                    values.push(v);
                }
                Ok(ChannelValue::Present(values))
            }
            _ => Ok(ChannelValue::Missing),
        }
    }

    fn read_ephemeral<T: Clone + Serialize + DeserializeOwned>(
        &self,
        id: &ChannelId<EphemeralValue<T>>,
    ) -> Result<ChannelValue<T>, ChannelError> {
        match self.ephemerals.get(id.name()) {
            Some(entry) => match &entry.bytes {
                Some(bytes) => {
                    let v: T = rmp_serde::from_slice(bytes)?;
                    Ok(ChannelValue::Present(v))
                }
                None => Ok(ChannelValue::Missing),
            },
            None => Ok(ChannelValue::Missing),
        }
    }

    fn read_untracked<T: DeepSizeOf + Clone>(
        &self,
        id: &ChannelId<UntrackedValue<T>>,
    ) -> ChannelValue<T> {
        match self.untracked.get(id.name()) {
            Some(erased) => match erased.value.downcast_ref::<T>() {
                Some(v) => ChannelValue::Present(v.clone()),
                None => ChannelValue::Missing,
            },
            None => ChannelValue::Missing,
        }
    }

    fn write<T: Clone + Serialize + DeserializeOwned>(
        &mut self,
        id: &ChannelId<LastValue<T>>,
        value: T,
        ssb_seq: i64,
    ) -> Result<(), ChannelError> {
        // Check seq before serializing to avoid wasted allocation
        if let Some(existing) = self.last_values.get(id.name()) {
            if existing.ssb_seq >= ssb_seq {
                return Ok(());
            }
        }
        let bytes =
            rmp_serde::to_vec_named(&value).map_err(ChannelError::SerializationError)?;
        self.last_values.insert(
            id.name().to_owned(),
            LastValueEntry { bytes, ssb_seq },
        );
        Ok(())
    }

    fn write_topic<T: Clone + Serialize + DeserializeOwned>(
        &mut self,
        id: &ChannelId<Topic<T>>,
        value: T,
        ssb_seq: i64,
    ) -> Result<(), ChannelError> {
        let requested_accumulate = id.accumulate();

        // Detect accumulate flag conflict with existing entry
        if let Some(existing) = self.topics.get(id.name()) {
            if existing.accumulate != requested_accumulate {
                return Err(ChannelError::AccumulateConflict {
                    channel_name: id.name().to_owned(),
                    existing: existing.accumulate,
                    requested: requested_accumulate,
                });
            }
        }

        let bytes =
            rmp_serde::to_vec_named(&value).map_err(ChannelError::SerializationError)?;

        let entry = self.topics.entry(id.name().to_owned()).or_insert_with(|| {
            TopicEntry {
                entries: Vec::new(),
                accumulate: requested_accumulate,
            }
        });

        // Insert in SSB seq order to maintain deterministic ordering
        let insert_pos = entry
            .entries
            .partition_point(|(_b, seq)| *seq <= ssb_seq);
        entry.entries.insert(insert_pos, (bytes, ssb_seq));
        Ok(())
    }

    fn write_ephemeral<T: Clone + Serialize + DeserializeOwned>(
        &mut self,
        id: &ChannelId<EphemeralValue<T>>,
        value: T,
        ssb_seq: i64,
    ) -> Result<(), ChannelError> {
        let _ = ssb_seq; // Ephemeral does not use SSB seq for ordering
        let bytes =
            rmp_serde::to_vec_named(&value).map_err(ChannelError::SerializationError)?;

        let entry = self
            .ephemerals
            .entry(id.name().to_owned())
            .or_insert_with(|| EphemeralEntry {
                bytes: None,
                write_count: 0,
            });
        entry.bytes = Some(bytes);
        entry.write_count = entry.write_count.saturating_add(1);
        Ok(())
    }

    fn write_untracked<T: DeepSizeOf + Clone>(
        &mut self,
        id: &ChannelId<UntrackedValue<T>>,
        value: T,
    ) -> Result<(), ChannelError> {
        let erased = ErasedUntracked {
            value: Box::new(value),
            size_fn: Box::new(|any| {
                any.downcast_ref::<T>()
                    .map_or(0, DeepSizeOf::deep_size_of)
            }),
        };
        self.untracked.insert(id.name().to_owned(), erased);
        Ok(())
    }

    fn snapshot_for_scheduling(&self) -> ChannelSnapshot {
        let mut inner = HashMap::new();

        for (name, entry) in &self.last_values {
            inner.insert(name.clone(), ChannelState::LastValue(entry.bytes.clone()));
        }

        for (name, entry) in &self.topics {
            let entries: Vec<Vec<u8>> =
                entry.entries.iter().map(|(b, _)| b.clone()).collect();
            inner.insert(name.clone(), ChannelState::Topic(entries));
        }

        for (name, entry) in &self.ephemerals {
            inner.insert(
                name.clone(),
                ChannelState::Ephemeral(entry.bytes.clone()),
            );
        }

        // UntrackedValue intentionally excluded — never serialized

        ChannelSnapshot { inner }
    }

    fn clear_ephemeral(&mut self) -> Vec<String> {
        let mut cleared = Vec::new();
        for (name, entry) in &mut self.ephemerals {
            if entry.write_count == 0 && entry.bytes.is_some() {
                entry.bytes = None;
                cleared.push(name.clone());
            }
        }
        cleared
    }

    fn force_clear_untracked(&mut self, id: &ChannelId<()>) -> bool {
        self.untracked.remove(id.name()).is_some()
    }

    fn deep_size_untracked(&self, id: &ChannelId<()>) -> usize {
        match self.untracked.get(id.name()) {
            Some(erased) => (erased.size_fn)(&*erased.value),
            None => 0,
        }
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_size_of_vec_u8_measures_heap() {
        let v: Vec<u8> = vec![0u8; 1_000_000];
        let size = v.deep_size_of();
        // Must be at least 1MB (heap) + 24 (stack Vec header)
        assert!(size >= 1_000_000, "deep_size was {size}");
    }

    #[test]
    fn deep_size_of_string_measures_heap() {
        let s = "a".repeat(10_000);
        let size = s.deep_size_of();
        assert!(size >= 10_000, "deep_size was {size}");
    }

    #[test]
    fn deep_size_of_nested_vec() {
        let v: Vec<Vec<u8>> = vec![vec![0u8; 1000]; 10];
        let size = v.deep_size_of();
        // 10 inner vecs * ~1000 bytes each + overhead
        assert!(size >= 10_000, "deep_size was {size}");
    }

    #[test]
    fn channel_id_type_name_in_debug() {
        let id: ChannelId<LastValue<String>> = ChannelId::new("test");
        let dbg = format!("{:?}", id);
        assert!(dbg.contains("test"));
        assert!(dbg.contains("LastValue"));
    }

    #[test]
    fn channel_id_equality_ignores_phantom() {
        let a: ChannelId<LastValue<u32>> = ChannelId::new("x");
        let b: ChannelId<LastValue<u32>> = ChannelId::new("x");
        assert_eq!(a, b);
    }

    #[test]
    fn channel_state_round_trip_last_value() {
        let state = ChannelState::LastValue(vec![1, 2, 3]);
        let bytes = rmp_serde::to_vec_named(&state).unwrap();
        let decoded: ChannelState = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(state, decoded);
    }

    #[test]
    fn channel_state_round_trip_topic() {
        let state = ChannelState::Topic(vec![vec![1], vec![2, 3]]);
        let bytes = rmp_serde::to_vec_named(&state).unwrap();
        let decoded: ChannelState = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(state, decoded);
    }

    #[test]
    fn channel_state_round_trip_ephemeral() {
        let state = ChannelState::Ephemeral(Some(vec![42]));
        let bytes = rmp_serde::to_vec_named(&state).unwrap();
        let decoded: ChannelState = rmp_serde::from_slice(&bytes).unwrap();
        assert_eq!(state, decoded);
    }

    #[test]
    fn last_value_higher_seq_wins() {
        let mut reg = InMemoryChannelRegistry::new();
        let id: ChannelId<LastValue<i32>> = ChannelId::new("v");
        reg.write(&id, 100, 10).unwrap();
        reg.write(&id, 200, 5).unwrap(); // lower seq, should be ignored
        match reg.read(&id).unwrap() {
            ChannelValue::Present(v) => assert_eq!(v, 100),
            ChannelValue::Missing => panic!("expected Present"),
        }
    }

    #[test]
    fn topic_maintains_seq_order() {
        let mut reg = InMemoryChannelRegistry::new();
        let id: ChannelId<Topic<String>> = ChannelId::new("log");
        reg.write_topic(&id, "c".to_string(), 3).unwrap();
        reg.write_topic(&id, "a".to_string(), 1).unwrap();
        reg.write_topic(&id, "b".to_string(), 2).unwrap();
        match reg.read_topic(&id).unwrap() {
            ChannelValue::Present(vals) => {
                assert_eq!(vals, vec!["a", "b", "c"]);
            }
            ChannelValue::Missing => panic!("expected Present"),
        }
    }

    #[test]
    fn ephemeral_cleared_when_no_writes() {
        let mut reg = InMemoryChannelRegistry::new();
        let id: ChannelId<EphemeralValue<u32>> = ChannelId::new("sig");
        reg.write_ephemeral(&id, 42, 1).unwrap();
        assert!(matches!(reg.read_ephemeral(&id).unwrap(), ChannelValue::Present(42)));
        // Simulate superstep boundary
        reg.reset_ephemeral_write_counts();
        let cleared = reg.clear_ephemeral();
        assert_eq!(cleared.len(), 1);
        assert!(matches!(reg.read_ephemeral(&id).unwrap(), ChannelValue::Missing));
    }

    #[test]
    fn ephemeral_not_cleared_when_written() {
        let mut reg = InMemoryChannelRegistry::new();
        let id: ChannelId<EphemeralValue<u32>> = ChannelId::new("sig");
        reg.write_ephemeral(&id, 42, 1).unwrap();
        // Don't reset — write count is still 1
        let cleared = reg.clear_ephemeral();
        assert!(cleared.is_empty());
        assert!(matches!(reg.read_ephemeral(&id).unwrap(), ChannelValue::Present(42)));
    }

    #[test]
    fn untracked_deep_size() {
        let mut reg = InMemoryChannelRegistry::new();
        let id: ChannelId<UntrackedValue<Vec<u8>>> = ChannelId::new("big");
        reg.write_untracked(&id, vec![0u8; 500_000]).unwrap();
        let erased = ChannelId::<()>::new("big");
        let size = reg.deep_size_untracked(&erased);
        assert!(size >= 500_000, "deep_size was {size}");
    }

    #[test]
    fn untracked_force_clear() {
        let mut reg = InMemoryChannelRegistry::new();
        let id: ChannelId<UntrackedValue<Vec<u8>>> = ChannelId::new("cache");
        reg.write_untracked(&id, vec![1, 2, 3]).unwrap();
        let erased = ChannelId::<()>::new("cache");
        assert!(reg.force_clear_untracked(&erased));
        assert!(!reg.force_clear_untracked(&erased));
        assert_eq!(reg.deep_size_untracked(&erased), 0);
    }

    #[test]
    fn snapshot_excludes_untracked() {
        let mut reg = InMemoryChannelRegistry::new();
        let lv: ChannelId<LastValue<u32>> = ChannelId::new("visible");
        let ut: ChannelId<UntrackedValue<Vec<u8>>> = ChannelId::new("invisible");
        reg.write(&lv, 1u32, 1).unwrap();
        reg.write_untracked(&ut, vec![0u8; 100]).unwrap();
        let snap = reg.snapshot_for_scheduling();
        assert!(snap.get("visible").is_some());
        assert!(snap.get("invisible").is_none());
    }

    #[test]
    fn topic_accumulate_false_clears() {
        let mut reg = InMemoryChannelRegistry::new();
        let id: ChannelId<Topic<u32>> = ChannelId::new_with_accumulate("ev", false);
        reg.write_topic(&id, 1, 1).unwrap();
        reg.write_topic(&id, 2, 2).unwrap();
        reg.clear_non_accumulating_topics();
        // After clear, reading returns Missing
        assert!(matches!(reg.read_topic(&id).unwrap(), ChannelValue::Missing));
    }

    #[test]
    fn topic_accumulate_true_preserves() {
        let mut reg = InMemoryChannelRegistry::new();
        let id: ChannelId<Topic<u32>> = ChannelId::new_with_accumulate("log", true);
        reg.write_topic(&id, 1, 1).unwrap();
        reg.clear_non_accumulating_topics();
        reg.write_topic(&id, 2, 2).unwrap();
        match reg.read_topic(&id).unwrap() {
            ChannelValue::Present(vals) => assert_eq!(vals.len(), 2),
            ChannelValue::Missing => panic!("expected Present"),
        }
    }

    #[test]
    fn snapshot_is_frozen() {
        let mut reg = InMemoryChannelRegistry::new();
        let id: ChannelId<LastValue<u32>> = ChannelId::new("s");
        reg.write(&id, 10u32, 1).unwrap();
        let snap = reg.snapshot_for_scheduling();
        reg.write(&id, 20u32, 2).unwrap();
        // Snapshot still has old value
        match snap.get("s").unwrap() {
            ChannelState::LastValue(bytes) => {
                let v: u32 = rmp_serde::from_slice(bytes).unwrap();
                assert_eq!(v, 10);
            }
            _ => panic!("expected LastValue"),
        }
    }

    #[test]
    fn channel_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ChannelError>();
    }

    #[test]
    fn empty_registry_snapshot_is_empty() {
        let reg = InMemoryChannelRegistry::new();
        let snap = reg.snapshot_for_scheduling();
        assert!(snap.is_empty());
        assert_eq!(snap.len(), 0);
    }
}

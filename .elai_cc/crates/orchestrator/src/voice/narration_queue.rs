//! J-112 narration queue -- deterministic priority-ordered, step-grain-coalesced,
//! verbosity-filtered spoken utterance scheduler (WARGAME MOVE 2).
//!
//! PURE data structure: no I/O, no async, no rand, no model (V-7, V-9).
//! The dispatcher (shipped separately) calls into this; this slice is self-contained.

use std::collections::{BTreeMap, VecDeque, HashSet};

/// J-112 narration priority classes (needs-input preempts completion preempts progress).
/// Higher discriminant value = higher priority (drains first).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NarrationPriority {
    Progress = 0,
    Completion = 1,
    NeedsInput = 2,
}

/// J-112 verbosity levels as row-class subscription filters (V-8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verbosity {
    Silent,
    Summary,
    Verbose,
}

impl Verbosity {
    /// Does this verbosity level admit the given priority class?
    pub fn admits(self, prio: NarrationPriority) -> bool {
        match self {
            Verbosity::Silent => false,
            Verbosity::Summary => matches!(
                prio,
                NarrationPriority::Completion | NarrationPriority::NeedsInput
            ),
            Verbosity::Verbose => true,
        }
    }
}

/// A pending narration utterance (deterministic; no model text on the event path V-9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NarrationItem {
    pub priority: NarrationPriority,
    pub session_id: String,
    /// Step grain for progress coalescing (None for Completion/NeedsInput).
    pub step_id: Option<String>,
    /// The already-rendered deterministic text (from voice::spoken_render::render_spoken).
    pub text: String,
}

/// J-112 narration queue: priority-ordered, step-grain-coalesced progress,
/// verbosity-filtered. PURE: no I/O, no async, no rand, no model.
#[derive(Debug, Clone)]
pub struct NarrationQueue {
    verbosity: Verbosity,
    /// Priority buckets, ordered high->low via BTreeMap key ordering.
    /// Key = NarrationPriority; we drain highest key first, so we iterate in
    /// reverse discriminant order at dequeue time.
    buckets: BTreeMap<NarrationPriority, VecDeque<NarrationItem>>,
    /// Set of (session_id, step_id) already queued for Progress coalescing.
    progress_keys: HashSet<(String, String)>,
}

impl NarrationQueue {
    pub fn new(verbosity: Verbosity) -> Self {
        Self {
            verbosity,
            buckets: BTreeMap::new(),
            progress_keys: HashSet::new(),
        }
    }

    /// Enqueue an item subject to verbosity + step-grain coalescing.
    /// Returns true if accepted (admitted by verbosity AND not a duplicate
    /// progress for the same (session_id, step_id)); false if filtered/coalesced.
    pub fn enqueue(&mut self, item: NarrationItem) -> bool {
        if !self.verbosity.admits(item.priority) {
            return false;
        }
        if item.priority == NarrationPriority::Progress {
            let step = item.step_id.as_deref().unwrap_or("");
            let key = (item.session_id.clone(), step.to_owned());
            if !self.progress_keys.insert(key) {
                return false;
            }
        }
        self.buckets
            .entry(item.priority)
            .or_default()
            .push_back(item);
        true
    }

    /// Drain the highest-priority item (needs-input before completion before progress).
    /// Returns None when empty.
    pub fn dequeue(&mut self) -> Option<NarrationItem> {
        // Iterate from highest priority (NeedsInput=2) down to Progress=0.
        let prio = self
            .buckets
            .iter()
            .rev()
            .find(|(_, q)| !q.is_empty())
            .map(|(&p, _)| p)?;
        let item = self.buckets.get_mut(&prio)?.pop_front()?;
        if prio == NarrationPriority::Progress {
            let step = item.step_id.as_deref().unwrap_or("");
            self.progress_keys.remove(&(item.session_id.clone(), step.to_owned()));
        }
        Some(item)
    }

    pub fn len(&self) -> usize {
        self.buckets.values().map(|q| q.len()).sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn verbosity(&self) -> Verbosity {
        self.verbosity
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn progress(session: &str, step: &str, text: &str) -> NarrationItem {
        NarrationItem {
            priority: NarrationPriority::Progress,
            session_id: session.to_owned(),
            step_id: Some(step.to_owned()),
            text: text.to_owned(),
        }
    }

    fn completion(session: &str, text: &str) -> NarrationItem {
        NarrationItem {
            priority: NarrationPriority::Completion,
            session_id: session.to_owned(),
            step_id: None,
            text: text.to_owned(),
        }
    }

    fn needs_input(session: &str, text: &str) -> NarrationItem {
        NarrationItem {
            priority: NarrationPriority::NeedsInput,
            session_id: session.to_owned(),
            step_id: None,
            text: text.to_owned(),
        }
    }

    #[test]
    fn needs_input_preempts_completion_progress() {
        let mut q = NarrationQueue::new(Verbosity::Verbose);
        assert!(q.enqueue(progress("s1", "st1", "p")));
        assert!(q.enqueue(completion("s1", "c")));
        assert!(q.enqueue(needs_input("s1", "n")));

        assert_eq!(q.dequeue().unwrap().priority, NarrationPriority::NeedsInput);
        assert_eq!(q.dequeue().unwrap().priority, NarrationPriority::Completion);
        assert_eq!(q.dequeue().unwrap().priority, NarrationPriority::Progress);
        assert!(q.is_empty());
    }

    #[test]
    fn progress_coalesces_same_step() {
        let mut q = NarrationQueue::new(Verbosity::Verbose);
        assert!(q.enqueue(progress("s1", "st1", "first")));
        assert!(!q.enqueue(progress("s1", "st1", "second"))); // coalesced
        assert_eq!(q.len(), 1);
        let item = q.dequeue().unwrap();
        assert_eq!(item.text, "first");
        assert!(q.is_empty());
    }

    #[test]
    fn progress_distinct_steps_both_kept() {
        let mut q = NarrationQueue::new(Verbosity::Verbose);
        assert!(q.enqueue(progress("s1", "st1", "a")));
        assert!(q.enqueue(progress("s1", "st2", "b")));
        assert_eq!(q.len(), 2);
        // Both are Progress (same priority); FIFO within bucket.
        let first = q.dequeue().unwrap();
        let second = q.dequeue().unwrap();
        assert_eq!(first.text, "a");
        assert_eq!(second.text, "b");
    }

    #[test]
    fn summary_filters_progress() {
        let mut q = NarrationQueue::new(Verbosity::Summary);
        assert!(!q.enqueue(progress("s1", "st1", "p"))); // filtered
        assert!(q.enqueue(completion("s1", "c")));
        assert!(q.enqueue(needs_input("s1", "n")));
        assert_eq!(q.len(), 2);
    }

    #[test]
    fn silent_rejects_all() {
        let mut q = NarrationQueue::new(Verbosity::Silent);
        assert!(!q.enqueue(progress("s1", "st1", "p")));
        assert!(!q.enqueue(completion("s1", "c")));
        assert!(!q.enqueue(needs_input("s1", "n")));
        assert!(q.is_empty());
        assert!(q.dequeue().is_none());
    }

    #[test]
    fn verbose_admits_all() {
        let mut q = NarrationQueue::new(Verbosity::Verbose);
        assert!(q.enqueue(progress("s1", "st1", "p")));
        assert!(q.enqueue(completion("s1", "c")));
        assert!(q.enqueue(needs_input("s1", "n")));
        assert_eq!(q.len(), 3);
    }

    #[test]
    fn dequeue_empty_returns_none() {
        let mut q = NarrationQueue::new(Verbosity::Verbose);
        assert!(q.dequeue().is_none());
    }
}

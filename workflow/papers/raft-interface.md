Algorithm: Raft Consensus (replicated log discipline)
Source: Diego Ongaro and John Ousterhout, "In Search of an Understandable Consensus Algorithm (Extended Version)", USENIX ATC 2014
Local PDF: F:\.ELAI_workflow\.claude\papers\raft.pdf
Extracted text: F:\.ELAI_workflow\.claude\papers\raft_extracted\raft.md
RAG topic: paper-raft

NOTE: This file is used for DETERMINISM/REPLAY DISCIPLINE. The goal is NOT to run distributed consensus in ELAI. The value is Raft's formal guarantees about an append-only replicated log and deterministic state-machine replay, which map directly to ELAI's single-writer SSB event log + RRA reproducible-run artifact.


FIGURE 3 SAFETY PROPERTIES (verbatim from paper, Section 5 summary figure)

Election Safety: at most one leader can be elected in any given term. (Section 5.2)

Leader Append-Only: a leader never overwrites or deletes entries in its log; it only appends new entries. (Section 5.3)

Log Matching: if two logs contain an entry with the same index and term, then the logs are identical in all entries up through the given index. (Section 5.3)

Leader Completeness (also called Log Completeness in paper body Section 5.4.3): if a log entry is committed in a given term, then that entry will be present in the logs of the leaders for all higher-numbered terms. (Section 5.4)

State Machine Safety: if a server has applied a log entry at a given index to its state machine, no other server will ever apply a different log entry for the same index. (Section 5.4)


SECTION 2 REPLICATED STATE MACHINE MODEL (verbatim excerpt)

"Replicated state machines are typically implemented using a replicated log. Each server stores a log containing a series of commands, which its state machine executes in order. Each log contains the same commands in the same order, so each state machine processes the same sequence of commands. Since the state machines are deterministic, each computes the same state and the same sequence of outputs."

"The consensus module on a server receives commands from clients and adds them to its log. It communicates with the consensus modules on other servers to ensure that every log eventually contains the same requests in the same order, even if some servers fail. Once commands are properly replicated, each server's state machine processes them in log order, and the outputs are returned to clients."

KEY: state machines are DETERMINISTIC. Same log order -> same state -> same outputs. This is the replay guarantee.


LOG MATCHING PROPERTY (verbatim from Section 5.3)

"Raft maintains the following properties, which together constitute the Log Matching Property in Figure 3:
- If two entries in different logs have the same index and term, then they store the same command.
- If two entries in different logs have the same index and term, then the logs are identical in all preceding entries."

"The first property follows from the fact that a leader creates at most one entry with a given log index in a given term, and log entries never change their position in the log."

"The consistency check acts as an induction step: the initial empty state of the logs satisfies the Log Matching Property, and the consistency check preserves the Log Matching Property whenever logs are extended."


LEADER APPEND-ONLY (verbatim from Section 5.3)

"A leader never overwrites or deletes entries in its own log (the Leader Append-Only Property in Figure 3)."


COMMITMENT DURABILITY (verbatim from Section 5.3)

"Raft guarantees that committed entries are durable and will eventually be executed by all of the available state machines."


STATE MACHINE SAFETY PROOF SKETCH (verbatim from Section 5.4.3)

"Given the Leader Completeness Property, we can prove the State Machine Safety Property from Figure 3, which states that if a server has applied a log entry at a given index to its state machine, no other server will ever apply a different log entry for the same index. At the time a server applies a log entry to its state machine, its log must be identical to the leader's log up through that entry and the entry must be committed."

"Finally, Raft requires servers to apply entries in log index order. Combined with the State Machine Safety Property, this means that all servers will apply exactly the same set of log entries to their state machines, in the same order."


Inputs:
log_entry | struct | (index: u64, term: u64, command: bytes) | index monotonically increasing; term monotonically non-decreasing; never null
currentTerm | u64 | scalar | monotonically increasing; persisted to stable storage before responding to any RPC
commitIndex | u64 | scalar | index of highest log entry known committed; monotonically increasing; initialized to 0
lastApplied | u64 | scalar | index of highest log entry applied to state machine; lastApplied <= commitIndex always
prevLogIndex | u64 | per AppendEntries RPC | index of entry immediately preceding new entries; used for consistency check
prevLogTerm | u64 | per AppendEntries RPC | term of prevLogIndex entry; must match follower's log at that index

Outputs:
applied_command | bytes | log[lastApplied].command | applied in strict log index order; once applied at index i, no different command ever applies at index i (State Machine Safety)
commitIndex_advance | u64 | new commit watermark | only advances, never retreats; requires majority replication in current term before advancing past an entry

Hyperparameters:
election_timeout | 150-300ms recommended | range: broadcastTime << electionTimeout << MTBF | randomized per server from fixed interval (e.g. 150-300ms); high sensitivity -- too low causes spurious elections, too high causes long unavailability after crash
heartbeat_interval | electionTimeout/2 typical | must be << electionTimeout | determines leader detection latency; authors use ~half of min election timeout
broadcast_time | 0.5ms-20ms typical | hardware-dependent | RPC round-trip to all servers; must be << electionTimeout (order of magnitude less)
cluster_size | 5 typical | odd number | tolerates floor((n-1)/2) failures; 3 tolerates 1, 5 tolerates 2, 7 tolerates 3
snapshot_threshold | log size in bytes | implementation-defined | no formal default; must be >> expected snapshot size to amortize cost


DETERMINISTIC-GATE RULES (ELAI mapping)

--- RULE D-1: Single Sequencer (maps to Section 2 strong-leader model) ---
Raft source: "Raft implements consensus by first electing a distinguished leader, then giving the leader complete responsibility for managing the replicated log. ... data flows in a simple fashion from the leader to other servers." (Section 5)
ELAI rule: all side-effecting decisions pass through the single-writer SSB log in commit order (single sequencer). No out-of-order writes. No concurrent writers to the same event sequence. The SSB WAL is the sequencer; there is exactly one writer (the ELAI orchestrator process) at any given time, analogous to Raft's single leader.

--- RULE D-2: Committed Row Immutability (maps to Leader Append-Only + commitment durability) ---
Raft source: "A leader never overwrites or deletes entries in its own log (the Leader Append-Only Property in Figure 3)." + "Raft guarantees that committed entries are durable and will eventually be executed by all of the available state machines." (Section 5.3)
ELAI rule: a committed SSB row is immutable (append-only, never rewritten or deleted). Any operation that modifies or deletes a row with rowid <= current commit watermark is a protocol violation equivalent to violating Leader Append-Only. This is what makes the SSB log trustworthy as a replay source.

--- RULE D-3: Deterministic Replay = State Machine Safety Analog ---
Raft source: "Since the state machines are deterministic, each computes the same state and the same sequence of outputs." (Section 2) + "if a server has applied a log entry at a given index to its state machine, no other server will ever apply a different log entry for the same index." (Figure 3, State Machine Safety)
ELAI rule: replay of the SSB log from a fixed seed must reconstruct the identical side-effect schedule. This is the RRA replay-verify gate. A divergence on replay (different action sequence, different output hash, different side-effect set given the same log + seed) = BLOCK. The replay is the test; divergence means either (a) the log was mutated (D-2 violation) or (b) the execution path is non-deterministic (a bug).

--- RULE D-4: Log Matching -> Hash Chain Integrity ---
Raft source: "If two entries in different logs have the same index and term, then they store the same command. If two entries in different logs have the same index and term, then the logs are identical in all preceding entries." (Section 5.3, Log Matching)
What ELAI's RRA already does: verifies replay produces same output given same log.
What Raft's Log Matching adds that RRA does not yet enforce: per-entry integrity (term+index pair uniquely identifies content) = a hash chain so a tampered or reordered log is detectable WITHOUT full replay. ELAI gap: SSB rows currently have no prev_hash field; a tampered row at position i would not be detected until a full replay diff. To close this gap, each SSB row should carry blake3(prev_row_bytes) so a truncated or reordered log fails the chain check before replay begins (cheap O(n) scan vs O(n) full execution).

--- RULE D-5: Log Completeness -> Watermark Monotonicity ---
Raft source: "if a log entry is committed in a given term, then that entry will be present in the logs of the leaders for all higher-numbered terms." (Figure 3, Leader Completeness)
ELAI rule: the SSB commit watermark (highest rowid durably flushed to WAL) must be monotonically non-decreasing. A new session must start from >= the previous session's watermark. Starting a new session at a lower watermark than a completed prior session is the ELAI analog of electing a leader who is missing committed entries.


Critical invariants (testable assertions):

assert commitIndex >= lastApplied  # Section 5.1 state invariant: applied never ahead of committed
assert all(log[i].index == i for i in range(len(log)))  # log entries never change position (Leader Append-Only basis)
assert log[i].term <= log[i+1].term for all consecutive pairs  # term numbers in log are non-decreasing (Section 5.1)
# State Machine Safety: once applied at index i, no different command applies at index i
# In ELAI: assert ssb_row_at(i).action_bytes == replay_output_at(i).action_bytes  (RRA gate)
# Log Matching (ELAI hash chain variant, gap to close):
# assert blake3(ssb_row_bytes(i-1)) == ssb_row(i).prev_hash for all i > 0
assert currentTerm increases monotonically and is persisted before any RPC response  # Section 5.1
assert commitIndex only advances (never decreases) across sessions  # Watermark monotonicity, D-5


Complexity:
Normal operation (leader replicating): O(n) per commit where n = cluster size (one RPC round-trip to majority)
Log recovery after leader crash: O(k) RPCs where k = number of entries to backfill (one per term with conflicts)
Leader election: O(n) RPCs; convergence time ~150-300ms with recommended timeouts + randomization
Snapshot install: O(snapshot_size) network; O(log_size_discarded) disk savings
Space: O(log_size) until snapshot; O(snapshot_size + tail_log_size) after compaction
Parallelism: RPCs issued in parallel to all followers; no sequential bottleneck in normal path
Single-writer constraint: HARD -- all appends through single leader; no parallel writers (intentional, guarantees ordering)


Failure modes (from paper):
Log divergence after leader crash: followers may have missing or extra uncommitted entries (Figure 7 scenarios a-f); resolved by leader forcing its log on recovery -- safe because only committed entries survive
Split vote livelock: without randomization, candidates can repeatedly split votes indefinitely (Section 5.2); fixed by randomized election timeouts 150-300ms
Stale reads: a leader that has been deposed may return stale data on read-only requests (Section 8); requires heartbeat majority confirmation or lease before serving reads
Old-term entry re-commitment hazard: a leader must not conclude an old-term entry is committed just because it is on a majority; only current-term entries trigger the commit watermark (Section 5.4.2, Figure 8)
Configuration split-brain: switching directly from old to new config is unsafe; two majorities can simultaneously elect leaders (Figure 10); requires joint consensus two-phase transition (Section 6)
Removed-server disruption: servers removed from the cluster continue sending RequestVote RPCs, causing spurious term bumps and leader deposition (Section 6); fixed by ignoring RequestVote within minimum election timeout of hearing from a leader


Implementation pitfalls (do NOT do this):
Do NOT commit an old-term log entry by counting replicas; only commit entries from the current term by count, then old entries commit implicitly via Log Matching (Section 5.4.2)
Do NOT allow multiple concurrent writers to the log; single-writer is the entire basis of the ordering guarantee
Do NOT advance commitIndex without persisting the log entries to stable storage first; volatile commitIndex on a crash would violate durability
Do NOT serve read-only requests without verifying leadership (heartbeat majority or lease); stale reads violate linearizability
Do NOT use a fixed election timeout; randomization is required to prevent persistent split votes
Do NOT compact the log below the commitIndex (discard only committed entries into snapshot, never speculative ones)
Do NOT allow a replica starting from a snapshot to skip the prev_hash/term consistency check for entries appended after the snapshot point
Do NOT conflate "on a majority" with "committed" for old-term entries; see Figure 8 for the concrete failure scenario


ELAI-SPECIFIC GAP ANALYSIS (RRA vs Raft Log Matching):
RRA today: given SSB log + seed -> re-run -> diff output hash. Detects non-determinism or post-hoc log mutation.
Raft Log Matching would add: per-entry blake3 hash chain embedded in each SSB row so (a) a truncated log fails before replay, (b) a reordered subset fails before replay, (c) a single tampered row fails at O(1) cost per row rather than O(full_replay). This is a concrete future hardening for the SSB schema: add prev_hash BLOB column, enforce on every insert, check on every RRA scan. Analogous to Raft's AppendEntries prevLogIndex/prevLogTerm consistency check but with cryptographic binding.

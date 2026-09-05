Algorithm: Contextual Memory Virtualisation (CMV): DAG-Based State Management and Structurally Lossless Trimming for LLM Agents
ArXiv: 2602.22402
Paper: Contextual Memory Virtualisation: DAG-Based State Management and Structurally Lossless Trimming for LLM Agents

Inputs:
session_log | JSONL | conversation history with tool calls | mandatory
trim_threshold | int | stub size threshold (default 500) | range: [100, 1000] | sensitivity: medium
dag_state | graph | existing version control DAG | optional
branch_metadata | dict | name/timestamp/tags for snapshot | optional

Outputs:
trimmed_session | JSONL | reduced token conversation | invariant: all user/assistant messages preserved
dag_node | snapshot | immutable versioned state | invariant: acyclic lineage
branch_session | session | new forked session with context | invariant: fresh UUID
trimming_metrics | dict | token reduction statistics | invariant: accurate counts

Hyperparameters:
stub_threshold | 500 | minimum content size to keep | range: [100, 1000] | sensitivity: high
compaction_boundary | auto | detect summary/compact markers | range: [manual, auto] | sensitivity: medium
orphan_handling | preserve | maintain tool result references | range: [drop, preserve] | sensitivity: high
cache_aware | true | consider prompt caching economics | range: [true, false] | sensitivity: medium

Critical invariants (testable assertions):
assert all(user_messages_preserved)
assert all(assistant_responses_preserved)
assert is_acyclic(dag_state)
assert token_reduction >= 0
assert len(trimmed_session) <= len(session_log)

Complexity:
Time: O(3 * session_lines) - three streaming passes
Space: O(session_size + dag_nodes)
Parallelism: sequential passes required for correctness

Failure modes:
Compaction boundary detection failure
Tool result orphan reference errors
DAG cycle formation during branching
Token counting inaccuracies
Session file corruption

Implementation pitfalls (do NOT do this):
Modify original session in-place
Skip tool result dependency tracking
Ignore prompt caching economics
Allow DAG cycles in lineage
Drop user or assistant messages during trimming

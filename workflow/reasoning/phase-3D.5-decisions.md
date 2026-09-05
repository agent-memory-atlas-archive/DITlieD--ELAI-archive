Phase 3D.5 decisions: true parallel spawn + VRAM-aware scheduling

What: upgrade BSP agent dispatch from sequential stub to concurrent JoinSet execution
with VRAM-aware concurrency limiting and straggler timeout reachability.

Key decision 1: JoinSet lives in execute_tasks_with_timeout, NOT spawn_parallel.
  Reason: AgentPool trait must remain object-safe for Phase 4A IPC. Adding async fn
  to the trait breaks object safety. spawn_parallel stays synchronous — it creates
  AgentHandle metadata. The async fan-out (JoinSet) happens in engine.rs where the
  async context already exists.

Key decision 2: add max_concurrent() to AgentPool trait.
  Formula: floor(available_vram / config.vram_required_bytes).
  Guard: if available_vram == 0 OR vram_required_bytes == 0, return 1 (fallback).
  This is object-safe (fn, not async fn, no generics). Engine calls pool.max_concurrent()
  to limit JoinSet fan-out width via semaphore or chunked spawning.

Key decision 3: results returned in task-input order, not JoinSet completion order.
  JoinSet completes tasks in arbitrary order. After collection, sort by original task
  index (stored alongside the future). apply_writes relies on SSB seq ordering.

Key decision 4: straggler timeout per-task inside JoinSet.
  Each spawned task wraps its work in tokio::time::timeout(straggler_timeout, ...).
  Timed-out tasks produce (index, Err(Elapsed)), collected into timed_out vec.

Key decision 5: concurrency test via tokio::time::sleep in JoinSet tasks.
  Current agent "execution" is instant (no real work in Phase 3). For testing true
  concurrency, JoinSet tasks call tokio::time::sleep to simulate work duration.
  Test: 2 tasks each sleep 100ms, total elapsed < 250ms proves parallelism.
  Production path: sleep(Duration::ZERO) when no simulation needed — but Phase 3C.2
  will wire actual agent execution here.

Expected test outcomes:
  test_spawn_parallel_true_concurrency: 2x100ms tasks finish in <250ms total
  test_vram_scheduling_max_concurrent: 2GB available / 1GB per-agent = max_concurrent 2
  test_vram_zero_fallback: available=0, max_concurrent=1
  test_per_agent_vram_zero_guard: per_agent=0, no panic, max_concurrent=1
  test_straggler_timeout_reachable: 1ms timeout + 200ms sleep = handle in timed_out vec
  test_result_seq_order: 3 tasks, verify returned order matches input order

Unknowns:
  - available_vram() is a stub (returns constructor field). Real hardware query in Phase 4F.
  - Agent communicate() not wired yet (Phase 3C.2). JoinSet tasks don't produce
    channel writes — that's a future phase concern.
  - tokio test-util pause/advance not used here; real elapsed time measured. If CI is
    slow, 250ms threshold may need widening — but 2x margin should suffice.

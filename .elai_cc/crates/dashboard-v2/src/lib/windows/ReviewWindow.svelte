<script lang="ts">
  import { Button, Empty, Scrollbar, UnwiredNote } from '$lib/ui';
  import * as stub from '$lib/api/stub';

  interface DiffLine {
    kind: 'added' | 'removed' | 'context' | 'hunk';
    oldNum?: number;
    newNum?: number;
    content: string;
  }

  interface ReviewFile {
    path: string;
    added: number;
    removed: number;
    lines: DiffLine[];
  }

  // ------------------------------------------------------------------
  // Sample diffs — realistic ELAI Rust domain
  // ------------------------------------------------------------------
  const FILES: ReviewFile[] = [
    {
      path: 'crates/orchestrator/src/fsm.rs',
      added: 12,
      removed: 4,
      lines: [
        { kind: 'hunk', content: '@@ -38,7 +38,15 @@ impl FsmEngine {' },
        { kind: 'context', oldNum: 38, newNum: 38, content: '    pub fn transition(&mut self, event: FsmEvent) -> Result<FsmState> {' },
        { kind: 'context', oldNum: 39, newNum: 39, content: '        let current = self.state;' },
        { kind: 'removed', oldNum: 40, content: '        let next = self.table.get(&(current, event)).copied().ok_or(FsmError::NoTransition)?' },
        { kind: 'removed', oldNum: 41, content: '        self.history.push(Transition { from: current, to: next, ts: now_ms() });' },
        { kind: 'added', newNum: 40, content: '        let next = self.table' },
        { kind: 'added', newNum: 41, content: '            .get(&(current, event))' },
        { kind: 'added', newNum: 42, content: '            .copied()' },
        { kind: 'added', newNum: 43, content: '            .ok_or_else(|| FsmError::NoTransition { from: current, event })?;' },
        { kind: 'added', newNum: 44, content: '        let ts = now_ms();' },
        { kind: 'added', newNum: 45, content: '        self.history.push(Transition { from: current, to: next, ts });' },
        { kind: 'added', newNum: 46, content: '        self.last_transition_ts = ts;' },
        { kind: 'context', oldNum: 42, newNum: 47, content: '        self.state = next;' },
        { kind: 'context', oldNum: 43, newNum: 48, content: '        Ok(next)' },
        { kind: 'context', oldNum: 44, newNum: 49, content: '    }' },
        { kind: 'hunk', content: '@@ -88,4 +93,6 @@ impl FsmEngine {' },
        { kind: 'context', oldNum: 88, newNum: 93, content: '    pub fn current_state(&self) -> FsmState {' },
        { kind: 'context', oldNum: 89, newNum: 94, content: '        self.state' },
        { kind: 'removed', oldNum: 90, content: '    }' },
        { kind: 'added', newNum: 95, content: '    }' },
        { kind: 'added', newNum: 96, content: '    pub fn last_transition_ts(&self) -> u64 {' },
        { kind: 'added', newNum: 97, content: '        self.last_transition_ts' },
        { kind: 'added', newNum: 98, content: '    }' },
      ],
    },
    {
      path: 'crates/cli/src/main.rs',
      added: 6,
      removed: 3,
      lines: [
        { kind: 'hunk', content: '@@ -12,9 +12,12 @@ use clap::Parser;' },
        { kind: 'context', oldNum: 12, newNum: 12, content: '#[derive(Parser, Debug)]' },
        { kind: 'context', oldNum: 13, newNum: 13, content: '#[command(author, version, about)]' },
        { kind: 'context', oldNum: 14, newNum: 14, content: 'pub struct Cli {' },
        { kind: 'removed', oldNum: 15, content: '    #[arg(long, default_value = "7B")]' },
        { kind: 'removed', oldNum: 16, content: '    pub model_size: String,' },
        { kind: 'removed', oldNum: 17, content: '    #[arg(long)]' },
        { kind: 'added', newNum: 15, content: '    #[arg(long, default_value = "D:\\.ELAI_workflow\\.elai_cc\\models\\Qwen2.5-Coder-7B-Instruct-abliterated")]' },
        { kind: 'added', newNum: 16, content: '    pub model: String,' },
        { kind: 'added', newNum: 17, content: '    #[arg(long, default_value_t = 25_000_usize)]' },
        { kind: 'added', newNum: 18, content: '    pub context_window: usize,' },
        { kind: 'added', newNum: 19, content: '    #[arg(long)]' },
        { kind: 'context', oldNum: 18, newNum: 20, content: '    pub gate_mode: Option<String>,' },
        { kind: 'context', oldNum: 19, newNum: 21, content: '}' },
      ],
    },
    {
      path: 'crates/dashboard/src/state.rs',
      added: 8,
      removed: 2,
      lines: [
        { kind: 'hunk', content: '@@ -24,6 +24,12 @@ pub struct DashboardState {' },
        { kind: 'context', oldNum: 24, newNum: 24, content: '    pub fsm: FsmSnapshot,' },
        { kind: 'context', oldNum: 25, newNum: 25, content: '    pub sidecar: SidecarStatus,' },
        { kind: 'removed', oldNum: 26, content: '    pub vram_pct: u8,' },
        { kind: 'added', newNum: 26, content: '    pub vram: Vec<VramGauge>,' },
        { kind: 'added', newNum: 27, content: '    pub active_agents: Vec<AgentEntry>,' },
        { kind: 'added', newNum: 28, content: '    pub gate: GateStatus,' },
        { kind: 'context', oldNum: 27, newNum: 29, content: '    pub session_id: String,' },
        { kind: 'hunk', content: '@@ -42,5 +48,9 @@ impl DashboardState {' },
        { kind: 'context', oldNum: 42, newNum: 48, content: '    pub fn new() -> Self {' },
        { kind: 'context', oldNum: 43, newNum: 49, content: '        Default::default()' },
        { kind: 'removed', oldNum: 44, content: '    }' },
        { kind: 'added', newNum: 50, content: '    }' },
        { kind: 'added', newNum: 51, content: '    pub fn vram_pct(&self) -> u8 {' },
        { kind: 'added', newNum: 52, content: '        self.vram.first().map(|g| g.pct_used as u8).unwrap_or(0)' },
        { kind: 'added', newNum: 53, content: '    }' },
        { kind: 'added', newNum: 54, content: '    pub fn is_executing(&self) -> bool {' },
        { kind: 'added', newNum: 55, content: '        self.fsm.state == FsmState::Executing' },
        { kind: 'added', newNum: 56, content: '    }' },
      ],
    },
  ];

  let activeIndex = $state(0);
  let activeFile = $derived(FILES[activeIndex]);

  // Per-hunk acceptance state: keyed by hunk-line content
  let hunkState = $state<Record<string, 'accepted' | 'declined' | null>>({});
  let fileState = $state<Record<string, 'accepted' | 'declined' | null>>({});
  let hunkHovered = $state<string | null>(null);
  let retryNote = $state<Record<string, string>>({});
  let retryNoteVisible = $state<Record<string, boolean>>({});

  // Per-file acceptance state
  function acceptAll(path: string) {
    fileState = { ...fileState, [path]: 'accepted' };
    stub.logStub('review.acceptAll', { path });
  }
  function declineAll(path: string) {
    fileState = { ...fileState, [path]: 'declined' };
    stub.logStub('review.declineAll', { path });
  }
  function retry(path: string) {
    const note = retryNote[path]?.trim() ?? '';
    fileState = { ...fileState, [path]: null };
    retryNoteVisible = { ...retryNoteVisible, [path]: false };
    stub.logStub('review.retry', { path, note: note || undefined });
  }
  function toggleRetryNote(path: string) {
    retryNoteVisible = { ...retryNoteVisible, [path]: !retryNoteVisible[path] };
  }
  function acceptHunk(hunkContent: string) {
    hunkState = { ...hunkState, [hunkContent]: 'accepted' };
  }
  function declineHunk(hunkContent: string) {
    hunkState = { ...hunkState, [hunkContent]: 'declined' };
  }
</script>

<div class="review">
  <div class="review__unwired-banner">
    <UnwiredNote
      phase="4B"
      feature="auditor-proposed diffs with per-hunk accept / decline / retry"
      note="No Tauri command yet exposes the auditor diff stream. The diffs below are static sample data — accept/decline/retry are logStub-only. Real wiring depends on a /v1/audit_proposals SSB feed plus accept_change / decline_change / retry_audit commands."
    />
  </div>

  {#if FILES.length === 0}
    <div class="review__empty">
      <Empty
        title="no changes to review"
        subtitle="review surfaces here when the auditor proposes a change"
      />
    </div>
  {:else}

  <!-- Tab strip -->
  <div class="tab-strip" role="tablist" aria-label="Changed files">
    {#each FILES as file, i}
      <button
        class="file-tab"
        class:file-tab--active={i === activeIndex}
        role="tab"
        aria-selected={i === activeIndex}
        onclick={() => { activeIndex = i; }}
      >
        {file.path.split('/').pop()}
      </button>
    {/each}
  </div>

  {#if activeFile}
    <!-- Sticky file header -->
    <div class="file-header">
      <span class="file-path">{activeFile.path}</span>
      <span class="file-stats">
        <span class="file-stats__added">+{activeFile.added}</span>
        <span class="file-stats__removed">-{activeFile.removed} lines</span>
      </span>
      <div class="file-actions">
        <Button
          variant={fileState[activeFile.path] === 'accepted' ? 'primary' : 'primary'}
          size="sm"
          onclick={() => acceptAll(activeFile.path)}
        >
          Accept All
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onclick={() => declineAll(activeFile.path)}
        >
          Decline All
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onclick={() => toggleRetryNote(activeFile.path)}
        >
          Retry
        </Button>
      </div>
    </div>

    {#if retryNoteVisible[activeFile.path]}
      <div class="retry-note-bar">
        <input
          class="retry-note-input mono"
          type="text"
          placeholder="optional note for auditor (Enter to confirm)"
          bind:value={retryNote[activeFile.path]}
          onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); retry(activeFile.path); } }}
        />
        <button class="retry-note-confirm" onclick={() => retry(activeFile.path)}>retry</button>
      </div>
    {/if}

    {#if fileState[activeFile.path]}
      <div class="file-verdict file-verdict--{fileState[activeFile.path]}">
        {fileState[activeFile.path] === 'accepted' ? 'All changes accepted' : 'All changes declined'}
        <button class="file-verdict__undo" onclick={() => retry(activeFile.path)}>Undo</button>
      </div>
    {/if}

    <!-- Diff body -->
    <Scrollbar>
      <div class="diff-body">
        {#each activeFile.lines as line}
          {#if line.kind === 'hunk'}
            <div
              class="diff-hunk"
              role="group"
              aria-label={line.content}
              onmouseenter={() => { hunkHovered = line.content; }}
              onmouseleave={() => { hunkHovered = null; }}
            >
              <span class="diff-hunk__label">{line.content}</span>
              {#if hunkHovered === line.content}
                <span class="diff-hunk__actions">
                  <button
                    class="hunk-btn"
                    onclick={() => acceptHunk(line.content)}
                    class:hunk-btn--active={hunkState[line.content] === 'accepted'}
                  >Accept</button>
                  <button
                    class="hunk-btn"
                    onclick={() => declineHunk(line.content)}
                    class:hunk-btn--active={hunkState[line.content] === 'declined'}
                  >Decline</button>
                </span>
              {:else if hunkState[line.content]}
                <span class="diff-hunk__verdict diff-hunk__verdict--{hunkState[line.content]}">
                  {hunkState[line.content]}
                </span>
              {/if}
            </div>
          {:else}
            <div class="diff-line diff-line--{line.kind}">
              <span class="diff-line__old-num">{line.oldNum ?? ''}</span>
              <span class="diff-line__new-num">{line.newNum ?? ''}</span>
              <span class="diff-line__content">{line.kind === 'added' ? '+' : line.kind === 'removed' ? '-' : ' '}{line.content}</span>
            </div>
          {/if}
        {/each}
      </div>
    </Scrollbar>
  {/if}
  {/if}
</div>

<style>
  .review {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--bg-base);
    overflow: hidden;
  }

  .review__unwired-banner {
    flex-shrink: 0;
    padding: var(--s-2) var(--s-3);
    border-bottom: 1px solid var(--border-subtle);
  }

  /* Tab strip */
  .tab-strip {
    display: flex;
    flex-shrink: 0;
    overflow-x: auto;
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    scrollbar-width: none;
  }
  .tab-strip::-webkit-scrollbar { display: none; }

  .file-tab {
    flex-shrink: 0;
    height: 32px;
    padding: 0 14px;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 11px;
    cursor: pointer;
    white-space: nowrap;
    transition: color var(--dur-fast) var(--ease-std), border-color var(--dur-fast) var(--ease-std);
    outline: none;
  }
  .file-tab:hover { color: var(--text-primary); }
  .file-tab--active {
    color: var(--teal);
    border-bottom-color: var(--teal);
  }
  .file-tab:focus-visible { box-shadow: inset 0 0 0 1px var(--teal); }

  /* File header */
  .file-header {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    height: 32px;
    padding: 0 var(--s-3);
    background: var(--bg-surface);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }
  .file-path {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .file-stats {
    display: flex;
    gap: 6px;
    font-family: var(--font-mono);
    font-size: 11px;
    flex-shrink: 0;
  }
  .file-stats__added { color: var(--teal); }
  .file-stats__removed { color: var(--fail); }
  .file-actions {
    display: flex;
    gap: var(--s-1);
    flex-shrink: 0;
  }

  /* File verdict banner */
  .file-verdict {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 4px 12px;
    font-family: var(--font-mono);
    font-size: 11px;
    flex-shrink: 0;
  }
  .file-verdict--accepted { background: var(--teal-ghost); color: var(--teal); }
  .file-verdict--declined { background: rgba(176,69,69,0.10); color: var(--fail); }
  .file-verdict__undo {
    background: transparent;
    border: none;
    color: var(--text-ghost);
    font-family: var(--font-mono);
    font-size: 10px;
    cursor: pointer;
    padding: 0 4px;
    text-decoration: underline;
  }
  .file-verdict__undo:hover { color: var(--text-secondary); }

  /* Retry note bar */
  .retry-note-bar {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 4px 12px;
    background: var(--bg-input);
    border-bottom: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }
  .retry-note-input {
    flex: 1;
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--border-subtle);
    color: var(--text-primary);
    font-size: 11px;
    padding: 2px 0;
    outline: none;
    transition: border-color var(--dur-fast) var(--ease-std);
  }
  .retry-note-input:focus { border-bottom-color: var(--teal); }
  .retry-note-input::placeholder { color: var(--text-ghost); }
  .retry-note-confirm {
    background: transparent;
    border: 1px solid var(--teal-line);
    border-radius: var(--r-xs);
    color: var(--teal);
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 8px;
    cursor: pointer;
    flex-shrink: 0;
  }
  .retry-note-confirm:hover { background: var(--teal-ghost); }

  /* Diff body */
  .diff-body {
    display: flex;
    flex-direction: column;
    font-family: var(--font-mono);
    font-size: 12px;
  }

  /* Hunk header */
  .diff-hunk {
    display: flex;
    align-items: center;
    background: var(--bg-input);
    padding: 4px 12px;
    color: var(--text-secondary);
    font-size: 11px;
    gap: var(--s-2);
    position: relative;
    min-height: 24px;
  }
  .diff-hunk__label { flex: 1; }
  .diff-hunk__actions {
    display: flex;
    gap: 4px;
  }
  .diff-hunk__verdict {
    font-size: 10px;
  }
  .diff-hunk__verdict--accepted { color: var(--teal); }
  .diff-hunk__verdict--declined { color: var(--fail); }

  /* Per-hunk action buttons */
  .hunk-btn {
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-xs);
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 1px 6px;
    cursor: pointer;
    transition: background-color var(--dur-fast) var(--ease-std), color var(--dur-fast) var(--ease-std);
  }
  .hunk-btn:hover { background: var(--bg-hover); color: var(--text-primary); }
  .hunk-btn--active { color: var(--teal); border-color: var(--teal-line); }

  /* Diff lines */
  .diff-line {
    display: grid;
    grid-template-columns: 44px 44px 1fr;
    min-height: 20px;
    line-height: 20px;
  }
  .diff-line--added {
    background: var(--teal-ghost);
    border-left: 1px solid var(--teal-line);
    color: var(--text-primary);
  }
  .diff-line--removed {
    background: rgba(176,69,69,0.10);
    border-left: 1px solid var(--fail);
    color: var(--text-secondary);
  }
  .diff-line--context {
    background: var(--bg-base);
    color: var(--text-secondary);
  }

  .diff-line__old-num,
  .diff-line__new-num {
    color: var(--text-ghost);
    font-size: 11px;
    padding: 0 8px;
    user-select: none;
    text-align: right;
    min-width: 0;
  }
  .diff-line__content {
    padding: 0 12px;
    white-space: pre;
    overflow-x: auto;
    font-size: 12px;
  }

  .review__empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style>

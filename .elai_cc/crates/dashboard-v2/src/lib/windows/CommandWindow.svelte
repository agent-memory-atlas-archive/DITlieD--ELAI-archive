<script lang="ts">
  import { onMount } from 'svelte';
  import { Send, Slash } from '@lucide/svelte';
  import { commands } from '../state/commands.svelte';
  import { fsm } from '../state/fsm.svelte';
  import { workspace } from '../state/workspace.svelte';
  import { Pulse } from '../ui/index';
  import { submit_intervention_response, logStub, continue_task, get_recent_tasks } from '../api/stub';
  import type { RecentTask } from '../api/types';

  // Live recent-tasks list (was a mock import). Polled every 10s.
  let recentTasks = $state<RecentTask[]>([]);
  onMount(() => {
    let alive = true;
    async function refresh() {
      try {
        const tasks = await get_recent_tasks(5);
        if (alive) recentTasks = tasks as RecentTask[];
      } catch (err) {
        console.debug('[CommandWindow] get_recent_tasks failed', err);
      }
    }
    void refresh();
    const t = setInterval(refresh, 10_000);
    return () => { alive = false; clearInterval(t); };
  });

  // ----------------------------------------------------------------
  // Slash palette
  // ----------------------------------------------------------------
  type SlashCategory = 'workflow' | 'project' | 'help';
  interface SlashDef {
    command: string;
    description: string;
    category: SlashCategory;
    hint?: string;
  }
  const SLASH_DEFS: SlashDef[] = [
    { command: '/task',     description: 'Start a new task',                category: 'workflow' },
    { command: '/fix',      description: 'Debug an issue',                  category: 'workflow' },
    { command: '/research', description: 'Run a deep research query',       category: 'workflow' },
    { command: '/continue', description: 'Resume from handoff',             category: 'workflow' },
    { command: '/explore',  description: 'Ask about the codebase',          category: 'project' },
    { command: '/status',   description: 'Current state',                   category: 'project' },
    { command: '/config',   description: 'Change settings',                 category: 'project', hint: 'settings' },
    { command: '/ask',      description: 'Ask the oracle',                  category: 'help' },
  ];

  const CATEGORY_LABEL: Record<SlashCategory, string> = {
    workflow: 'Workflow',
    project:  'Project',
    help:     'Help',
  };

  // Filter both by what the user typed in the textarea AND by the
  // dedicated palette filter input (palette filter has priority).
  let paletteFilter = $state('');

  let slashFilter: SlashDef[] = $derived.by(() => {
    const palQuery = paletteFilter.trim().toLowerCase();
    const inputQuery = (commands.inputValue.startsWith('/') ? commands.inputValue.split(' ')[0] : '').toLowerCase();
    const q = palQuery || inputQuery;
    if (!q || q === '/') return SLASH_DEFS;
    return SLASH_DEFS.filter((s) =>
      s.command.toLowerCase().includes(q.replace(/^\//, '')) ||
      s.description.toLowerCase().includes(q.replace(/^\//, ''))
    );
  });

  let slashGrouped: { category: SlashCategory; items: SlashDef[] }[] = $derived.by(() => {
    const cats: SlashCategory[] = ['workflow', 'project', 'help'];
    return cats
      .map((c) => ({ category: c, items: slashFilter.filter((s) => s.category === c) }))
      .filter((g) => g.items.length > 0);
  });

  let slashHighlightIdx = $state(0);
  let paletteFilterEl = $state<HTMLInputElement | undefined>(undefined);

  // ----------------------------------------------------------------
  // Scroll-to-bottom
  // ----------------------------------------------------------------
  let streamEl = $state<HTMLElement | undefined>(undefined);

  $effect(() => {
    const _ = commands.cards.length; // track
    if (!streamEl) return;
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        if (streamEl) streamEl.scrollTop = streamEl.scrollHeight;
      });
    });
  });

  // ----------------------------------------------------------------
  // Input placeholder by FSM state
  // ----------------------------------------------------------------
  const placeholder = $derived(() => {
    const hasIntervention = commands.cards.some((c) => c.kind === 'intervention');
    if (hasIntervention) return 'respond to the card above, or type to add context';
    if (fsm.state === 'IDLE') return 'describe a task, ask a question, or type / for commands';
    if (fsm.state === 'EXECUTING') return 'add constraints, or wait for intervention cards';
    return 'what do you need?';
  });

  // ----------------------------------------------------------------
  // Textarea handlers
  // ----------------------------------------------------------------
  let textareaEl = $state<HTMLTextAreaElement | undefined>(undefined);

  // ----------------------------------------------------------------
  // Auto-detect Oracle chip — confirm gate when the classifier infers
  // RESEARCH on a non-slash submit. Lifecycle owned by commands store
  // (pendingResearchChip + accept/reject/editResearchChip). Here we
  // only render, autofocus, and forward Enter/Esc/Tab to the store.
  // Keybindings are scoped to the chip container; they cannot collide
  // with the slash palette because the palette closes the moment a
  // submit fires (which is when the chip can appear), and the textarea
  // is disabled while the chip is pending so no new '/' opens it.
  // ----------------------------------------------------------------
  let chipEl = $state<HTMLDivElement | undefined>(undefined);

  $effect(() => {
    if (commands.pendingResearchChip && chipEl) {
      chipEl.focus();
    }
  });

  $effect(() => {
    const _ = commands.refocusInputTick;
    if (commands.refocusInputTick > 0) {
      requestAnimationFrame(() => textareaEl?.focus());
    }
  });

  function handleChipKeydown(e: KeyboardEvent) {
    if (!commands.pendingResearchChip) return;
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      void commands.acceptResearchChip();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      void commands.rejectResearchChip();
    } else if (e.key === 'Tab') {
      e.preventDefault();
      commands.editResearchChip();
    }
  }

  function handleInput(e: Event) {
    const t = e.target as HTMLTextAreaElement;
    const v = t.value;
    commands.setInput(v);
    slashHighlightIdx = 0;
    // Auto-open the slash palette when the user starts the line with "/".
    if (v.startsWith('/') && !commands.slashOpen) {
      commands.openSlash();
    }
    // Auto-close if they cleared the slash input
    if (!v.startsWith('/') && commands.slashOpen && paletteFilter.length === 0) {
      commands.closeSlash();
    }
    // Auto-close once they've moved past the command token (typed a space).
    // Past this point, Enter must submit verbatim — not autocomplete.
    if (v.startsWith('/') && v.includes(' ') && commands.slashOpen && paletteFilter.length === 0) {
      commands.closeSlash();
    }
    // Auto-height
    t.style.height = 'auto';
    t.style.height = Math.min(t.scrollHeight, 120) + 'px';
  }

  function handleKeydown(e: KeyboardEvent) {
    if (commands.slashOpen) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        // Guard divide-by-zero when filter has no matches; matches handlePaletteFilterKey.
        slashHighlightIdx = (slashHighlightIdx + 1) % Math.max(slashFilter.length, 1);
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        const len = Math.max(slashFilter.length, 1);
        slashHighlightIdx = (slashHighlightIdx - 1 + len) % len;
        return;
      }
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        const chosen = slashFilter[slashHighlightIdx];
        // Bare-command autocomplete only. "/research" + Enter -> "/research ".
        // "/research find pytorch" + Enter -> submit verbatim (do not clobber args).
        if (chosen && commands.inputValue.trim() === chosen.command) {
          executeSlash(chosen.command);
        } else {
          submitInput();
        }
        return;
      }
      if (e.key === 'Escape') {
        commands.closeSlash();
        paletteFilter = '';
        return;
      }
    }

    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      submitInput();
    }
  }

  function openPalette() {
    commands.openSlash();
    // focus the palette filter once it mounts
    requestAnimationFrame(() => paletteFilterEl?.focus());
  }
  function closePalette() {
    commands.closeSlash();
    paletteFilter = '';
    textareaEl?.focus();
  }
  function handlePaletteFilterKey(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      slashHighlightIdx = (slashHighlightIdx + 1) % Math.max(slashFilter.length, 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      slashHighlightIdx = (slashHighlightIdx - 1 + slashFilter.length) % Math.max(slashFilter.length, 1);
    } else if (e.key === 'Enter' && slashFilter.length > 0) {
      e.preventDefault();
      const chosen = slashFilter[Math.min(slashHighlightIdx, slashFilter.length - 1)];
      if (chosen) executeSlash(chosen.command);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      closePalette();
    }
  }

  function submitInput() {
    const text = commands.inputValue.trim();
    if (!text) return;
    commands.submit(text);
    if (textareaEl) {
      textareaEl.style.height = 'auto';
    }
  }

  function executeSlash(cmd: string) {
    commands.setInput(cmd + ' ');
    commands.closeSlash();
    textareaEl?.focus();
  }

  // ----------------------------------------------------------------
  // Intervention action handler
  // ----------------------------------------------------------------
  async function handleIntervention(actionId: string, response: 'approve' | 'reject' | 'modify' | 'redirect') {
    await submit_intervention_response(actionId, response);
    commands.appendCard('status', {
      kind: 'status',
      message: `Intervention ${actionId}: ${response}d. FSM resuming execution.`,
      fsm_state: 'EXECUTING',
      step_id: undefined,
    });
  }

  // ----------------------------------------------------------------
  // Error action handlers
  // ----------------------------------------------------------------
  function handleRetry(stepId?: string) {
    logStub('commands.error.retry', { stepId });
    commands.appendCard('status', {
      kind: 'status',
      message: `Retrying step ${stepId ?? 'current'}. Executor resuming.`,
      fsm_state: 'EXECUTING',
      step_id: stepId,
    });
  }

  function handleTakeover() {
    logStub('commands.error.takeover');
    commands.appendCard('status', {
      kind: 'status',
      message: 'Takeover mode active. Awaiting your instructions.',
      fsm_state: undefined,
      step_id: undefined,
    });
  }

  function handleSkip(stepId?: string) {
    logStub('commands.error.skip', { stepId });
    commands.appendCard('status', {
      kind: 'status',
      message: `Step ${stepId ?? 'current'} skipped. Moving to next baby step.`,
      fsm_state: 'EXECUTING',
      step_id: undefined,
    });
  }

  // ----------------------------------------------------------------
  // Disambiguation option handler
  // ----------------------------------------------------------------
  function handleDisambigChoice(question: string, optionLabel: string) {
    commands.appendCard('steering', {
      kind: 'steering',
      vector_label: 'user-choice',
      delta_description: optionLabel,
      applied: true,
    });
    commands.appendCard('status', {
      kind: 'status',
      message: `Choice noted: "${optionLabel}". Executor will apply this constraint.`,
      fsm_state: 'EXECUTING',
      step_id: undefined,
    });
    logStub('commands.disambiguation', { question, choice: optionLabel });
  }

  // ----------------------------------------------------------------
  // Recent task click
  // ----------------------------------------------------------------
  async function handleContinueTask(taskId: string) {
    const result = await continue_task(taskId);
    commands.appendCard('status', {
      kind: 'status',
      message: result.restoredContext,
      fsm_state: 'EXECUTING',
      step_id: undefined,
    });
  }

  // ----------------------------------------------------------------
  // Timestamp formatter
  // ----------------------------------------------------------------
  function fmtTime(ts: number): string {
    const d = new Date(ts);
    return d.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', hour12: false });
  }

</script>

<div class="command-window">
  <!-- Card stream -->
  <div class="stream" bind:this={streamEl}>
    <div class="stream-inner">
    {#if commands.cards.length === 0}
      <!-- Empty state: breathing dot + slash hints + recent tasks -->
      <div class="empty-state">
        <Pulse mode="dot" size={12} />
        <span class="empty-label">no active task</span>
        <div class="empty-slash-row">
          <button class="mono-ghost-btn" onclick={() => executeSlash('/task')}>/task</button>
          <button class="mono-ghost-btn" onclick={() => executeSlash('/fix')}>/fix</button>
          <button class="mono-ghost-btn" onclick={() => executeSlash('/explore')}>/explore</button>
          <button class="mono-ghost-btn" onclick={() => executeSlash('/continue')}>/continue</button>
        </div>
        {#if recentTasks.length > 0}
          <div class="recent-tasks">
            <span class="label-ghost">recent</span>
            {#each recentTasks.slice(0, 3) as task (task.id)}
              <button
                class="recent-task-row"
                onclick={() => handleContinueTask(task.id)}
              >
                <span class="recent-task-title">{task.title}</span>
                <span
                  class="recent-task-status"
                  class:status--completed={task.status === 'completed'}
                  class:status--failed={task.status === 'failed'}
                >{task.status}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {:else}
      {#each commands.cards as card (card.id)}
        <div class="card-wrapper anim-enter-lift">

          {#if card.kind === 'user'}
            <!-- User message: right-aligned plain text, no bubble bg -->
            <div class="user-msg">
              <span class="user-msg__text">{(card.payload as import('../api/types').UserCardPayload).text}</span>
              <span class="user-msg__ts mono">{fmtTime(card.ts)}</span>
            </div>

          {:else if card.kind === 'status'}
            <!-- Status: arrow glyph + plain sentence + mono ts -->
            {@const p = card.payload as import('../api/types').StatusCardPayload}
            <div class="status-line">
              <span class="status-arrow">&#8594;</span>
              {#if p.fsm_state}
                <span class="status-state mono">[{p.fsm_state.toLowerCase()}]</span>
              {/if}
              <span class="status-msg">{p.message}</span>
              {#if p.step_id}
                <span class="status-step mono">{p.step_id}</span>
              {/if}
            </div>

          {:else if card.kind === 'intervention'}
            <!-- Intervention card: floating-in-space, rich content, preserved -->
            {@const p = card.payload as import('../api/types').InterventionCardPayload}
            <div class="orbit-card orbit-card--intervention">
              <!-- Top-edge teal ghost gradient line -->
              <div class="card-top-edge" aria-hidden="true"></div>
              <div class="card-inner">
                <!-- Title row with breathing dot -->
                <div class="card-title-row">
                  <Pulse mode="dot" size={6} />
                  <span class="intervention-title">{p.request.title}</span>
                </div>
                <span class="card-body">{p.request.description}</span>
                {#if p.request.file}
                  <span class="card-filepath mono">{p.request.file}</span>
                {/if}
                {#if p.request.diff_summary}
                  <div class="diff-block">
                    <!-- Vertical teal strip on left edge -->
                    <div class="diff-left-strip" aria-hidden="true"></div>
                    <pre class="diff-pre mono">{p.request.diff_summary}</pre>
                  </div>
                {/if}
                <div class="btn-row">
                  <button
                    class="btn-approve"
                    onclick={() => handleIntervention(p.request.id, 'approve')}
                  >Approve</button>
                  <button
                    class="btn-outlined"
                    onclick={() => handleIntervention(p.request.id, 'reject')}
                  >Reject</button>
                  <button
                    class="btn-outlined"
                    onclick={() => handleIntervention(p.request.id, 'modify')}
                  >Modify</button>
                  <button
                    class="btn-outlined"
                    onclick={() => handleIntervention(p.request.id, 'redirect')}
                  >Redirect</button>
                </div>
              </div>
            </div>

          {:else if card.kind === 'error'}
            <!-- Error card: teal dot, white title (matches intervention) -->
            {@const p = card.payload as import('../api/types').ErrorCardPayload}
            <div class="orbit-card orbit-card--error">
              <div class="card-inner">
                <div class="card-title-row">
                  <Pulse mode="dot" size={6} color="--teal" />
                  <span class="intervention-title mono">[error: {p.source}]</span>
                </div>
                <span class="card-body">{p.message}</span>
                {#if p.step_id}
                  <span class="card-step mono">{p.step_id}</span>
                {/if}
                <div class="btn-row">
                  <button
                    class="btn-outlined"
                    onclick={() => handleRetry(p.step_id)}
                  >Let it retry</button>
                  <button
                    class="btn-outlined"
                    onclick={() => handleTakeover()}
                  >Take over</button>
                  <button
                    class="btn-outlined"
                    onclick={() => handleSkip(p.step_id)}
                  >Skip</button>
                </div>
              </div>
            </div>

          {:else if card.kind === 'disambiguation'}
            <!-- Disambiguation: italic question + outlined option buttons -->
            {@const p = card.payload as import('../api/types').DisambiguationCardPayload}
            <div class="orbit-card orbit-card--plain">
              <div class="card-inner">
                <span class="disambig-question">"{p.question}"</span>
                <div class="btn-row btn-row--col">
                  {#each p.options as opt (opt.id)}
                    <button
                      class="btn-outlined"
                      onclick={() => handleDisambigChoice(p.question, opt.label)}
                    >{opt.label}</button>
                  {/each}
                </div>
              </div>
            </div>

          {:else if card.kind === 'steering'}
            <!-- Steering: minimal italic line with dot prefix -->
            {@const p = card.payload as import('../api/types').SteeringCardPayload}
            <div class="steering-line">
              <span class="steering-dot" aria-hidden="true">·</span>
              <span class="steering-text">Noted: {p.delta_description}</span>
            </div>

          {:else if card.kind === 'response'}
            <!-- Response card: accumulates streamed tokens; breath line while streaming -->
            {@const p = card.payload as import('../api/types').ResponseCardPayload}
            <div class="orbit-card orbit-card--response">
              <div class="card-inner">
                <pre class="response-text mono">{p.text || ' '}</pre>
                {#if p.streaming}
                  <Pulse mode="line" size={1} />
                {/if}
              </div>
            </div>

          {:else if card.kind === 'code-answer'}
            <!-- EXPLORE_CODE wire: ranked BM25 hits from the local index. -->
            {@const p = card.payload as import('../api/types').CodeAnswerCardPayload}
            <div class="orbit-card orbit-card--response">
              <div class="card-inner">
                {#if p.status === 'not_indexed'}
                  <div class="code-answer-hint mono">project not indexed — reopen it from the picker</div>
                {:else if p.status === 'error'}
                  <div class="code-answer-hint mono">{p.message ?? 'knowledge query failed'}</div>
                {:else if p.hits.length === 0}
                  <div class="code-answer-hint mono">no matches for "{p.query}"</div>
                {:else}
                  <div class="code-answer-header mono">
                    {p.hits.length} hit{p.hits.length === 1 ? '' : 's'} for "{p.query}"
                  </div>
                  <ul class="code-answer-list">
                    {#each p.hits as h, i (h.path + ':' + h.line_start + ':' + i)}
                      <li class="code-answer-hit">
                        <div class="code-answer-meta mono">
                          <span class="code-answer-path">{h.path}{h.line_start ? ':' + h.line_start : ''}{h.line_end && h.line_end !== h.line_start ? '-' + h.line_end : ''}</span>
                          <span class="code-answer-score">{h.score.toFixed(2)}</span>
                          <span class="code-answer-kind">{h.chunk_kind.toLowerCase()}</span>
                        </div>
                        {#if h.snippet}
                          <pre class="code-answer-snippet mono">{h.snippet}</pre>
                        {/if}
                      </li>
                    {/each}
                  </ul>
                {/if}
              </div>
            </div>

          {/if}
        </div>
      {/each}
    {/if}

    {#if commands.pendingResearchChip}
      <div
        class="card-wrapper anim-enter-lift"
        bind:this={chipEl}
        tabindex={-1}
        onkeydown={handleChipKeydown}
        role="dialog"
        aria-label="Confirm Oracle Mode"
      >
        <div class="orbit-card orbit-card--intervention research-chip">
          <div class="card-inner">
            <div class="card-title-row">
              <Pulse mode="dot" size={6} />
              <span class="intervention-title">Looks like a research question. Send to Oracle?</span>
            </div>
            <span class="card-body mono">{commands.pendingResearchChip.text}</span>
            <div class="btn-row">
              <button class="btn-approve" onclick={() => commands.acceptResearchChip()}>Accept &middot; Enter</button>
              <button class="btn-outlined" onclick={() => commands.rejectResearchChip()}>Reply locally &middot; Esc</button>
              <button class="btn-outlined" onclick={() => commands.editResearchChip()}>Edit &middot; Tab</button>
            </div>
          </div>
        </div>
      </div>
    {/if}
    </div>
  </div>

  <!-- Input bar — outlined chat shell with slash palette floating above -->
  <div class="input-bar">
    {#if commands.slashOpen}
      <!-- Floating action menu, anchored above the chat shell -->
      <div class="slash-palette" role="dialog" aria-label="Slash commands">
        <div class="slash-filter-row">
          <input
            bind:this={paletteFilterEl}
            class="slash-filter mono"
            type="text"
            placeholder="Filter actions..."
            value={paletteFilter}
            oninput={(e) => { paletteFilter = (e.target as HTMLInputElement).value; slashHighlightIdx = 0; }}
            onkeydown={handlePaletteFilterKey}
            aria-label="Filter slash commands"
          />
        </div>

        {#each slashGrouped as group (group.category)}
          <div class="slash-section" role="group" aria-label={CATEGORY_LABEL[group.category]}>
            <div class="slash-section-label mono">{CATEGORY_LABEL[group.category]}</div>
            {#each group.items as def (def.command)}
              {@const flatIdx = slashFilter.findIndex((s) => s.command === def.command)}
              <button
                class="slash-row"
                class:slash-row--active={flatIdx === slashHighlightIdx}
                role="option"
                aria-selected={flatIdx === slashHighlightIdx}
                onclick={() => executeSlash(def.command)}
                onmouseenter={() => { slashHighlightIdx = flatIdx; }}
              >
                <span class="slash-row-label mono">{def.command.replace(/^\//, '')}</span>
                <span class="slash-row-desc">{def.description}</span>
                {#if def.hint}
                  <span class="slash-row-hint mono">{def.hint}</span>
                {/if}
              </button>
            {/each}
          </div>
        {/each}

        {#if slashFilter.length === 0}
          <div class="slash-empty">no matches</div>
        {/if}
      </div>
    {/if}

    <!-- Outlined chat shell. Background lifts to var(--bg-input);
         border accents teal on focus-within. -->
    <div class="input-shell" class:input-shell--focused={commands.inputFocused}>
      <!-- Subtle teal underline, fades in on focus-within -->
      <div class="input-bar-line" aria-hidden="true"></div>

      <div class="input-row">
        <button
          class="slash-toggle"
          class:slash-toggle--active={commands.slashOpen}
          aria-label="Open slash command palette"
          disabled={!!commands.pendingResearchChip}
          onclick={() => (commands.slashOpen ? closePalette() : openPalette())}
        >
          <Slash size={14} strokeWidth={1.5} />
        </button>

        <textarea
          bind:this={textareaEl}
          class="input-textarea mono"
          value={commands.inputValue}
          placeholder={placeholder()}
          rows={1}
          disabled={!!commands.pendingResearchChip}
          oninput={handleInput}
          onkeydown={handleKeydown}
          onfocus={() => commands.setFocused(true)}
          onblur={() => commands.setFocused(false)}
        ></textarea>

        {#if commands.inputValue.trim().length > 0}
          <button
            class="send-btn"
            aria-label="Send"
            onclick={submitInput}
          >
            <Send size={14} strokeWidth={1.5} />
          </button>
        {/if}
      </div>
    </div>
  </div>
</div>

<style>
  /* ================================================================
     CommandWindow — Orbit aesthetic
     Cards float in space; depth = top-edge gradient + hover outline.
     No Card primitive. No left colored bars. No heavy chrome.
  ================================================================ */

  .command-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    /* Flat void — no starfield, no nebula. Pure minimal chrome. */
    background: var(--bg-void);
    position: relative;
  }

  /* ---- Stream ---- */
  .stream {
    flex: 1 1 auto;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    scrollbar-width: thin;
    scrollbar-color: var(--border-default) transparent;
    position: relative;
    z-index: 1;
  }

  /* Inner wrapper anchors content to the bottom of the scroll container.
     When content fits: empty space at top, cards at bottom (chat style).
     When content overflows: scrolls naturally; bottom is always at the bottom. */
  .stream-inner {
    margin-top: auto;
    display: flex;
    flex-direction: column;
    gap: var(--s-6);
    padding: var(--s-4) var(--s-6) var(--s-3);
  }

  .stream::-webkit-scrollbar { width: 4px; }
  .stream::-webkit-scrollbar-track { background: transparent; }
  .stream::-webkit-scrollbar-thumb { background: var(--border-default); border-radius: var(--r-pill); }
  .stream::-webkit-scrollbar-thumb:hover { background: var(--border-strong); }

  .card-wrapper {
    display: flex;
    flex-direction: column;
  }

  /* ---- Empty state ---- */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-3);
    padding-top: var(--s-10);
  }

  .empty-label {
    font-family: var(--font-sans);
    font-size: 14px;
    color: var(--text-secondary);
    margin-top: var(--s-1);
  }

  .empty-slash-row {
    display: flex;
    flex-direction: row;
    align-items: center;
    gap: var(--s-4);
    margin-top: var(--s-2);
  }

  .mono-ghost-btn {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-ghost);
    background: none;
    border: none;
    cursor: pointer;
    padding: 0;
    outline: none;
    transition: color var(--dur-fast) var(--ease-std);
  }

  .mono-ghost-btn:hover {
    color: var(--text-secondary);
  }

  .mono-ghost-btn:focus-visible {
    box-shadow: var(--ring);
    border-radius: var(--r-xs);
  }

  .label-ghost {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-ghost);
    letter-spacing: 0.06em;
    text-transform: uppercase;
    margin-bottom: var(--s-1);
  }

  .recent-tasks {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--s-1);
    width: 100%;
    max-width: 420px;
  }

  .recent-task-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    width: 100%;
    padding: var(--s-1) var(--s-2);
    background: none;
    border: none;
    border-radius: var(--r-sm);
    cursor: pointer;
    gap: var(--s-2);
    transition: background var(--dur-fast) var(--ease-std);
    outline: none;
  }

  .recent-task-row:hover {
    background: var(--teal-ghost);
  }

  .recent-task-row:focus-visible {
    box-shadow: var(--ring);
  }

  .recent-task-title {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .recent-task-status {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex-shrink: 0;
  }

  .status--completed { color: var(--teal); }
  .status--failed    { color: var(--fail); }

  /* ---- User message — right-aligned plain text ---- */
  .user-msg {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    align-self: flex-end;
    max-width: 80%;
    gap: var(--s-1);
  }

  .user-msg__text {
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--text-primary);
    line-height: 1.5;
    text-align: right;
  }

  .user-msg__ts {
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* ---- Status line — minimal, no card chrome — but aligned with the
         left edge of orbit-card content so it doesn't read as off-grid. ---- */
  .status-line {
    display: flex;
    align-items: baseline;
    gap: var(--s-2);
    padding: var(--s-1) var(--s-5);
  }

  .status-arrow {
    font-size: 13px;
    color: var(--teal);
    flex-shrink: 0;
    line-height: 1;
  }

  .status-state {
    font-size: 11px;
    color: var(--teal);
  }

  .status-msg {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .status-step {
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* ---- Orbit card — shared floating-in-space base ---- */
  .orbit-card {
    position: relative;
    padding: var(--s-4) var(--s-5);  /* 16–20 px generous padding */
    border-radius: var(--r-md);
    background: transparent;
    /* Default: no border; only a top-edge gradient line gives presence */
    outline: 1px solid transparent;
    transition:
      outline-color var(--dur-fast) var(--ease-std),
      background var(--dur-fast) var(--ease-std);
  }

  .orbit-card:hover {
    outline-color: rgba(255,255,255,0.05);
    background: rgba(16,20,26,0.35);
  }

  /* New card mount: brief teal-ghost glow fade out over 1.6s.
     Applied via .card-wrapper .orbit-card because anim-enter-lift sits on the wrapper. */
  .card-wrapper .orbit-card {
    animation: cardGlow 1.6s var(--ease-out) both;
  }

  @keyframes cardGlow {
    0%   { outline-color: rgba(30,138,122,0.28); }
    60%  { outline-color: rgba(30,138,122,0.12); }
    100% { outline-color: transparent; }
  }

  /* ---- Top-edge gradient lines (kept invisible; cards rely only on hover outline + cardGlow mount anim) ---- */
  .card-top-edge {
    display: none;
  }

  /* ---- Card inner layout ---- */
  .card-inner {
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
    position: relative;
    z-index: 1;
  }

  /* ---- Title row with breathing dot ---- */
  .card-title-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .intervention-title {
    font-family: var(--font-sans);
    font-size: 14px;
    font-weight: 500;
    color: var(--text-primary);
    line-height: 1.3;
  }


  /* ---- Body text ---- */
  .card-body {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .card-step {
    font-size: 10px;
    color: var(--text-ghost);
  }

  .card-filepath {
    font-size: 11px;
    color: var(--text-ghost);
  }

  /* ---- Diff block ---- */
  .diff-block {
    position: relative;
    background: rgba(26,32,41,0.6);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    overflow: hidden;
    margin: var(--s-1) 0;
  }

  /* Vertical teal strip — 1px subtle left accent */
  .diff-left-strip {
    position: absolute;
    left: 0;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--teal-line);
    opacity: 0.5;
    pointer-events: none;
  }

  .diff-pre {
    font-size: 12px;
    color: var(--text-secondary);
    padding: var(--s-2) var(--s-3) var(--s-2) calc(var(--s-3) + 3px);
    white-space: pre-wrap;
    word-break: break-all;
    line-height: 1.5;
    margin: 0;
    background: none;
  }

  /* Diff line tinting: lines starting with + or - */
  :global(.diff-pre .diff-add) { color: rgba(30,138,122,0.8); }
  :global(.diff-pre .diff-rem) { color: rgba(176,69,69,0.8); }

  /* ---- Buttons ---- */
  .btn-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
    margin-top: var(--s-2);
    align-items: center;
  }

  .btn-row--col {
    flex-direction: column;
    align-items: flex-start;
  }

  /* Approve: outlined teal — primary feeling, never solid fill */
  .btn-approve {
    font-family: var(--font-sans);
    font-size: 12px;
    font-weight: 500;
    color: var(--teal);
    background: transparent;
    border: 1px solid var(--teal-line);
    border-radius: var(--r-sm);
    padding: 4px 12px;
    cursor: pointer;
    outline: none;
    transition:
      background var(--dur-fast) var(--ease-std),
      color var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std);
  }

  .btn-approve:hover {
    background: var(--teal-ghost);
    color: var(--teal-bright);
    border-color: var(--teal);
  }

  .btn-approve:focus-visible {
    box-shadow: var(--ring);
  }

  /* Outlined: disambiguation options + secondary action buttons */
  .btn-outlined {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    background: transparent;
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    padding: 4px 12px;
    cursor: pointer;
    outline: none;
    transition:
      background var(--dur-fast) var(--ease-std),
      color var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std);
  }

  .btn-outlined:hover {
    background: var(--teal-ghost);
    color: var(--text-primary);
    border-color: var(--teal-line);
  }

  .btn-outlined:focus-visible {
    box-shadow: var(--ring);
  }

  /* ---- Disambiguation ---- */
  .disambig-question {
    font-family: var(--font-mono);
    font-size: 12px;
    font-style: italic;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  /* ---- Steering line — aligned with orbit-card left edge ---- */
  .steering-line {
    display: flex;
    align-items: baseline;
    gap: var(--s-2);
    padding: var(--s-1) var(--s-5);
  }

  .steering-dot {
    font-family: var(--font-sans);
    font-size: 14px;
    color: var(--text-ghost);
    line-height: 1;
    flex-shrink: 0;
  }

  .steering-text {
    font-family: var(--font-sans);
    font-size: 12px;
    font-style: italic;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  /* ---- Response card (B-14 streaming) ---- */
  .response-text {
    font-family: var(--font-sans);
    font-size: 13px;
    color: var(--text-primary);
    line-height: 1.55;
    white-space: pre-wrap;
    word-break: break-word;
    margin: 0;
    background: none;
    padding: 0;
  }

  /* ---- Mono utility ---- */
  .mono {
    font-family: var(--font-mono);
  }

  /* ---- Code-answer card (EXPLORE_CODE wire) ---- */
  .code-answer-header {
    font-size: 11px;
    color: var(--text-ghost);
    margin-bottom: var(--s-2);
    letter-spacing: 0.02em;
  }
  .code-answer-list {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: var(--s-2);
  }
  .code-answer-hit {
    border-left: 2px solid var(--border-subtle);
    padding-left: var(--s-2);
    transition: border-color var(--dur-fast) var(--ease-std);
  }
  .code-answer-hit:hover { border-left-color: var(--teal-line); }
  .code-answer-meta {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    font-size: 11px;
    line-height: 1.4;
  }
  .code-answer-path { color: var(--teal); }
  .code-answer-score { color: var(--text-ghost); }
  .code-answer-kind {
    color: var(--text-disabled);
    text-transform: lowercase;
    font-size: 10px;
  }
  .code-answer-snippet {
    font-size: 11px;
    color: var(--text-secondary);
    background: rgba(26, 32, 41, 0.45);
    border-radius: var(--r-xs);
    padding: var(--s-1) var(--s-2);
    margin: 4px 0 0;
    white-space: pre-wrap;
    word-break: break-word;
    line-height: 1.45;
    max-height: 9.5em;
    overflow: hidden;
  }
  .code-answer-hint {
    font-size: 11px;
    color: var(--text-ghost);
    font-style: italic;
  }

  /* ---- Auto-detect Oracle chip ---- */
  /* Container is focusable (tabindex=-1) for keyboard handling. Hide
     the default focus ring on the container itself; the inner orbit-card
     already has its own outline + cardGlow animation that signals focus. */
  .card-wrapper:focus,
  .card-wrapper:focus-visible {
    outline: none;
  }
  .research-chip {
    outline: 1px solid var(--teal-line);
  }

  /* Disabled input controls go quiet rather than greyed out — matches
     ExploreWindow's input shell when pending. */
  .input-textarea:disabled,
  .slash-toggle:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  /* ---- Input bar — invisible chrome, just a textarea floating at the bottom ---- */
  .input-bar {
    flex-shrink: 0;
    position: relative;
    background: transparent;
    z-index: 2;
  }

  /* No separator line by default; thin teal line appears on focus-within */
  .input-bar-line {
    height: 1px;
    background: var(--teal-line);
    opacity: 0;
    pointer-events: none;
    transition: opacity 180ms var(--ease-std);
  }
  .input-bar:focus-within .input-bar-line { opacity: 0.45; }

  .input-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    padding: 2px 12px;
  }

  .slash-toggle {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    flex-shrink: 0;
    background: transparent;
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    cursor: pointer;
    color: var(--text-secondary);
    transition:
      color var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std),
      background var(--dur-fast) var(--ease-std);
    outline: none;
  }

  .slash-toggle:hover {
    color: var(--teal);
    border-color: var(--teal-line);
    background: var(--teal-ghost);
  }
  .slash-toggle--active {
    color: var(--teal);
    border-color: var(--teal-line);
    background: var(--teal-ghost);
  }

  .slash-toggle:focus-visible {
    box-shadow: var(--ring);
  }

  /* Input: no visible border; teal underline on focus */
  .input-textarea {
    flex: 1 1 auto;
    background: transparent;
    border: none;
    border-bottom: 1px solid transparent;
    color: var(--text-primary);
    font-size: 12px;
    line-height: 1.4;
    padding: 2px 4px;
    resize: none;
    outline: none;
    min-height: 22px;
    max-height: 120px;
    overflow-y: auto;
    transition: border-color var(--dur-fast) var(--ease-std);
  }

  .input-textarea::placeholder {
    color: var(--text-ghost);
  }

  .input-textarea:focus {
    border-bottom-color: var(--teal);
  }

  /* Send: outlined teal, NOT solid fill */
  .send-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    flex-shrink: 0;
    background: transparent;
    border: 1px solid var(--teal-line);
    border-radius: var(--r-sm);
    cursor: pointer;
    color: var(--teal);
    outline: none;
    transition:
      background var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std),
      color var(--dur-fast) var(--ease-std);
  }

  .send-btn:hover {
    background: var(--teal-ghost);
    border-color: var(--teal);
    color: var(--teal-bright);
  }

  .send-btn:focus-visible {
    box-shadow: var(--ring);
  }

  /* ---- Outlined chat shell ---- */
  .input-shell {
    position: relative;
    margin: var(--s-3) var(--s-4) var(--s-3);
    background: var(--bg-input);
    border: 1px solid var(--border-default);
    border-radius: var(--r-md);
    padding: var(--s-1) var(--s-2);
    transition:
      border-color var(--dur-fast) var(--ease-std),
      background var(--dur-fast) var(--ease-std);
  }
  .input-shell--focused {
    border-color: var(--teal-line);
    background: rgba(26, 32, 41, 0.85);
  }

  /* ---- Slash palette — floating dialog above the chat shell ---- */
  .slash-palette {
    position: absolute;
    bottom: calc(100% + 6px);
    left: var(--s-4);
    right: var(--s-4);
    background: var(--bg-surface);
    border: 1px solid var(--border-default);
    border-radius: var(--r-md);
    padding: 6px;
    z-index: 40;
    animation: paletteLift 180ms var(--ease-out) both;
    max-height: 360px;
    overflow-y: auto;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.45);
  }

  @keyframes paletteLift {
    from { opacity: 0; transform: translateY(6px); }
    to   { opacity: 1; transform: translateY(0); }
  }

  .slash-filter-row {
    padding: 4px 4px 8px;
    border-bottom: 1px solid var(--border-subtle);
    margin-bottom: 4px;
  }
  .slash-filter {
    width: 100%;
    background: transparent;
    border: none;
    outline: none;
    color: var(--text-primary);
    font-size: 12px;
    padding: 4px 6px;
  }
  .slash-filter::placeholder {
    color: var(--text-ghost);
  }

  .slash-section { padding: 4px 0; }
  .slash-section-label {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.10em;
    color: var(--text-ghost);
    padding: 4px 10px 4px;
  }

  .slash-row {
    display: grid;
    grid-template-columns: auto 1fr auto;
    align-items: center;
    gap: var(--s-3);
    width: 100%;
    padding: 6px 10px;
    background: none;
    border: none;
    border-radius: var(--r-sm);
    cursor: pointer;
    outline: none;
    text-align: left;
    transition:
      background var(--dur-fast) var(--ease-std),
      color var(--dur-fast) var(--ease-std);
  }

  .slash-row:hover { background: var(--bg-hover); }

  .slash-row--active {
    background: var(--teal-ghost);
    outline: 1px solid var(--teal-line);
    outline-offset: -1px;
  }

  .slash-row-label {
    font-size: 12px;
    color: var(--text-primary);
  }

  .slash-row-desc {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .slash-row-hint {
    font-size: 10px;
    color: var(--text-ghost);
  }

  .slash-empty {
    padding: 12px;
    text-align: center;
    color: var(--text-ghost);
    font-size: 12px;
  }
</style>

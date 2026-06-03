<script lang="ts">
  import { onMount } from 'svelte';
  import { Send, Slash, Square } from '@lucide/svelte';
  import { commands } from '../state/commands.svelte';
  import { fsm } from '../state/fsm.svelte';
  import { session } from '../state/session.svelte';
  import { workspace } from '../state/workspace.svelte';

  // B-39D: ranks for the dotted effort indicator below the input.
  const EFFORT_RANK: Record<string, number> = { low: 1, medium: 2, high: 3, max: 4 };
  const EFFORT_LEVELS = ['low', 'medium', 'high', 'max'] as const;
  import { Pulse } from '../ui/index';
  import { submit_intervention_response, submit_credential_response, logStub, continue_task, get_recent_tasks, load_handoff, get_context_state } from '../api/stub';
  import type { RecentTask, ContextState } from '../api/types';
  // E-4 chat UX critique surfaces. PinnedRulesPanel mounts as a sidebar
  // strip above the stream; ClaimConfidenceChip / HonestCorrectionBlock /
  // SycophancyGuardBadge are bubble-level inline components rendered
  // when a card carries the matching ChatUxCritiqueEvent kind payload
  // (wire-dark unless ELAI_CHAT_UX_CRITIQUE_ENABLE=1 on producer side).
  import PinnedRulesPanel from '$lib/components/PinnedRulesPanel.svelte';
  import ClaimConfidenceChip from '$lib/components/ClaimConfidenceChip.svelte';
  import HonestCorrectionBlock from '$lib/components/HonestCorrectionBlock.svelte';
  import SycophancyGuardBadge from '$lib/components/SycophancyGuardBadge.svelte';
  // E-33 Task 3 / E-2 Task 2 -- SkeletonBubble mounts during TTFT gap
  // (streaming=true, text='') so the response area has visual weight before
  // the first token arrives. shimmerEnabled follows motion.allow so
  // prefers-reduced-motion users get a static outline instead of the sweep.
  import SkeletonBubble from '$lib/components/SkeletonBubble.svelte';
  import { motion } from '$lib/state/motion.svelte';
  // FINAL TRANCHE B2 / E-2 -- FramePacer is imported here so the symbol
  // is visible to the response-card render scope; the actual instance
  // lives in commands.svelte.ts (_runStreamingPath) where the per-stream
  // chunk callback is wired. Re-importing keeps tree-shaking honest and
  // surfaces the dependency at the window scope. Wire-dark behind
  // VITE_ELAI_FRAME_PACER_ENABLE=1 per frame_pacer.ts:17.
  import { FramePacer as _FramePacer } from '$lib/streaming/frame_pacer';
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const _framePacerSymbolUsed = _FramePacer;

  // Live recent-tasks list (was a mock import). Polled every 10s.
  let recentTasks = $state<RecentTask[]>([]);
  // E-23 — context-state snapshot drives the runtime_choice_badge +
  // prompt_injection_check_status row in the status strip. Polled every
  // 5s alongside the recent-tasks refresh. Empty default keeps the UI
  // stable when the Tauri command isn't reachable yet.
  let contextSnapshot = $state<ContextState>({
    fsm_state: 'IDLE',
    substate: null,
    token_budget_pct: 0,
    mcp_think_topics: [],
    last_precompact_thresholds: [],
    prompt_injection_check_status: 'not_run',
    runtime_choice_badge: '',
    // E-3 sub-phase 4B — role + backend split surface. role drives the
    // user-visible badge; backend stays hidden until the drawer opens.
    current_role: null,
    current_runtime: null,
  });
  // E-3 sub-phase 4B — drawer toggle for the role badge. Default off
  // keeps MODEL hidden until the developer clicks to expand (the
  // expose-via-drawer pattern from the Cursor/Aider line in the
  // improvement-sweep report — developer audience but the surface
  // stays uncluttered by default).
  let showRoleDrawer = $state<boolean>(false);
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
    // E-23 — refresh runtime_choice_badge + prompt_injection_check_status.
    // Pricing-transparency badge transitions when B-37 runtime_fallback
    // emits a new "succeeded" backend; injection status row reflects the
    // C-42 fuzzer's last verdict (when shipped — defaults to "not_run").
    async function refreshContext() {
      try {
        const ctx = await get_context_state();
        if (alive) contextSnapshot = ctx;
      } catch (err) {
        console.debug('[CommandWindow] get_context_state failed', err);
      }
    }
    void refresh();
    void refreshContext();
    const t = setInterval(refresh, 10_000);
    const tCtx = setInterval(refreshContext, 5_000);
    return () => { alive = false; clearInterval(t); clearInterval(tCtx); };
  });

  // E-23 — derived labels for the pricing-transparency + injection rows.
  // Pure $derived so the badges re-render in lockstep with the snapshot.
  const runtimeBadgeLabel = $derived.by(() => {
    if (contextSnapshot.runtime_choice_badge === 'local') return 'local';
    if (contextSnapshot.runtime_choice_badge === 'cloud_billed') return 'cloud - billed';
    return '';
  });
  const injectionStatusLabel = $derived.by(() => {
    // Mirrors orchestrator::ssb::PROMPT_INJECTION_CHECK_STALE_HOURS — when
    // C-42 ships, the backend command will downgrade "passed" to a yellow
    // advisory after 24h. Today the row reads "not_run" by default.
    return contextSnapshot.prompt_injection_check_status;
  });
  // E-3 sub-phase 4B — role badge label. Voice-section identity
  // (planner / coder / critic / default) so the badge reads as the
  // ROLE not the underlying MODEL. Empty when the orchestrator has
  // not yet emitted a wrapped dispatch.
  const roleBadgeLabel = $derived.by(() => {
    const role = contextSnapshot.current_role;
    if (!role) return '';
    switch (role) {
      case 'coder': return 'executor running';
      case 'critic': return 'auditor reviewing';
      case 'planner': return 'planner thinking';
      case 'default': return 'agent ready';
      default: return `role: ${role}`;
    }
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

    // C-6: Ctrl+Enter as ADDITIONAL submit alias (spec dashboard-design.txt:852).
    // Preserves existing bare-Enter submit (muscle memory). Shift+Enter still inserts newline.
    if (e.key === 'Enter' && e.ctrlKey) {
      e.preventDefault();
      submitInput();
      return;
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
    // C-23: /continue is the only slash that invokes a Tauri command
    // synchronously (the others compose a multi-arg textarea entry
    // first). Fall back to the textarea-prepopulate path only if
    // load_handoff rejects — that surfaces the typed DagBranchError
    // ("no active SessionDag" / "no handoff snapshot") in an error
    // card without losing the user's ability to manually type the
    // /continue arguments.
    if (cmd === '/continue') {
      commands.closeSlash();
      // Fire-and-forget; the await is wrapped so executeSlash stays
      // synchronous (matches the other branches and the keyboard-event
      // call site that expects a void return).
      void handleContinueSlash();
      return;
    }
    commands.setInput(cmd + ' ');
    commands.closeSlash();
    textareaEl?.focus();
  }

  async function handleContinueSlash() {
    try {
      const result = await load_handoff();
      commands.appendCard('status', {
        kind: 'status',
        message:
          `Resumed handoff (session ${result.session_id})\n` +
          `  Snapshot         ${result.snap_id}\n` +
          `  FSM state        ${result.fsm_state}\n` +
          `  Messages         ${result.message_count}\n` +
          `  Parent chain     ${result.parent_chain_len} snapshot(s) (root -> tip)`,
        fsm_state: undefined,
        step_id: undefined,
      });
    } catch (err) {
      commands.appendCard('error', {
        kind: 'error',
        message: `/continue failed: ${err}`,
        source: 'sidecar',
        step_id: undefined,
        recoverable: true,
      });
      // Fall back to textarea prepopulate so the user can manually
      // continue with a snap_id argument (mirrors the legacy behaviour
      // when load_handoff is unavailable).
      commands.setInput('/continue ');
      textareaEl?.focus();
    }
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
  // C2: Credential form handler
  // ----------------------------------------------------------------
  // Per-card credential value map keyed by card.id -> field.id -> value.
  let credentialValues = $state<Record<string, Record<string, string>>>({});

  function handleCredentialInput(cardId: string, fieldId: string, value: string) {
    credentialValues = {
      ...credentialValues,
      [cardId]: {
        ...(credentialValues[cardId] ?? {}),
        [fieldId]: value,
      },
    };
  }

  async function handleCredentialSubmit(requestId: string, cardId: string) {
    const values = credentialValues[cardId] ?? {};
    await submit_credential_response(requestId, values);
    commands.appendCard('status', {
      kind: 'status',
      message: `Credentials submitted. FSM resuming execution.`,
      fsm_state: 'EXECUTING',
      step_id: undefined,
    });
    // Clear submitted values so they don't persist in memory.
    const { [cardId]: _removed, ...rest } = credentialValues;
    credentialValues = rest;
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
  <!-- E-4 Task 3: pinned-rules visible panel. Sidebar strip above the
       stream; collapsed by default. Wire-dark unless the producer is
       running with ELAI_CHAT_UX_CRITIQUE_ENABLE=1 (still renders with
       an empty list otherwise). -->
  <div class="pinned-rules-strip">
    <PinnedRulesPanel />
  </div>
  <!-- Card stream -->
  <div class="stream" bind:this={streamEl}>
    <div class="stream-inner">
    {#if commands.cards.length === 0}
      <!-- Empty state: breathing dot + first-run positioning block +
           slash hints + recent tasks.

           E-23 — 3-paragraph "does / does NOT / moats" block. maintainer voice
           per .claude/rules/output-formatting.md: lowercase i, no em-
           dashes, no robotic prefixes, contractions drop apostrophes.
           Every claim links to its discourse-complaint source via <small>
           citations (METR arXiv:2507.09089 + Stack Overflow 2025 +
           JetBrains AI Pulse April 2026 + Anthropic issue #13171). -->
      <div class="empty-state">
        <Pulse mode="dot" size={12} />
        <span class="empty-label">no active task</span>

        <div class="first-run-positioning" data-testid="first-run-positioning">
          <p class="positioning-para">
            <span class="positioning-tag mono">does</span>
            ELAI runs FSM-gated, grounded, audited code work. it isnt a chat
            assistant. every state transition writes an SSB row, every claim
            cites its source, every audit cycle runs a real test pass.
            <small class="positioning-cite">cf. METR arXiv:2507.09089 on the
            felt-faster gap</small>
          </p>
          <p class="positioning-para">
            <span class="positioning-tag mono">does NOT</span>
            things ELAI does NOT do: no full autonomous YOLO mode. no
            surprise pricing. no hidden third-party calls. no SDK
            abstraction layer hiding the raw anthropic protocol. the
            runtime_choice_badge below tells you local vs cloud-billed at
            all times.
            <small class="positioning-cite">cf. Cursor June 2025 backlash +
            Stack Overflow 2025 (29% trust)</small>
          </p>
          <p class="positioning-para">
            <span class="positioning-tag mono">moats</span>
            where the moats are: rules-as-data not prompt-soup. per-level
            auditor (matches
            Cloudflare 7-reviewer pattern). local-first hybrid runtime.
            cross-session FSM. grounded reasoning chain. open source.
            <small class="positioning-cite">cf. JetBrains AI Pulse 2026 +
            Cloudflare AI code review</small>
          </p>
        </div>

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

          {:else if card.kind === 'credential-request'}
            <!-- C2: Credential request card — password-masked form fields -->
            {@const p = card.payload as import('../api/types').CredentialRequestCardPayload}
            <div class="orbit-card orbit-card--intervention">
              <div class="card-top-edge" aria-hidden="true"></div>
              <div class="card-inner">
                <div class="card-title-row">
                  <Pulse mode="dot" size={6} />
                  <span class="intervention-title">Credentials required</span>
                </div>
                <span class="card-body">{p.description}</span>
                <div class="credential-fields">
                  {#each p.fields as field (field.id)}
                    <div class="credential-field-row">
                      <label class="credential-label" for="cred-{card.id}-{field.id}">
                        {field.label}
                      </label>
                      <input
                        id="cred-{card.id}-{field.id}"
                        class="credential-input mono"
                        type={field.secret ? 'password' : 'text'}
                        value={credentialValues[card.id]?.[field.id] ?? (field.default ?? '')}
                        oninput={(e) => handleCredentialInput(card.id, field.id, (e.target as HTMLInputElement).value)}
                        autocomplete="off"
                        spellcheck={false}
                      />
                    </div>
                  {/each}
                </div>
                <div class="btn-row">
                  <button
                    class="btn-approve"
                    onclick={() => handleCredentialSubmit(p.request_id, card.id)}
                  >Submit</button>
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
                {#if p.streaming && !p.text}
                  <!-- E-33 Task 3 / E-2 Task 2: skeleton while waiting for TTFT.
                       Replaced by real text on first token; shimmer off under
                       prefers-reduced-motion (motion.allow=false). -->
                  <SkeletonBubble shimmerEnabled={motion.allow} />
                {:else}
                  <pre class="response-text mono">{p.text || ' '}</pre>
                {/if}
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

          <!-- E-4 chat UX critique inline overlays. Wire-dark unless
               the producer attaches chat_ux_meta to the card payload
               (gated by ELAI_CHAT_UX_CRITIQUE_ENABLE=1 on the
               orchestrator side; the dashboard never synthesizes
               these locally). -->
          {#if card.chat_ux_meta}
            {@const meta = card.chat_ux_meta}
            {#if meta.claims && meta.claims.length > 0}
              <div class="chat-ux-claims">
                {#each meta.claims as claim, i (i + ':' + claim.text.slice(0, 16))}
                  <ClaimConfidenceChip
                    tier={claim.tier}
                    evidence_ref={claim.evidence_ref}
                  />
                {/each}
              </div>
            {/if}
            {#if meta.sycophancy}
              <div class="chat-ux-sycophancy">
                {#if meta.sycophancy.kind === 'hold_position'}
                  <SycophancyGuardBadge
                    verdict="hold_position"
                    evidence_ref={meta.sycophancy.evidence_ref}
                  />
                {:else}
                  <SycophancyGuardBadge verdict="no_evidence" />
                {/if}
              </div>
            {/if}
            {#if meta.correction}
              <HonestCorrectionBlock
                original_claim={meta.correction.original_claim}
                correction_text={meta.correction.correction_text}
                trigger={meta.correction.trigger}
                original_claim_blake3_hex={meta.correction.original_claim_blake3_hex}
              />
            {/if}
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

    <!-- E-23 — pricing-transparency badge + prompt_injection_check_status
         row. Wire-dark unless ELAI_DEVELOPER_DISCOURSE_ENABLE=1 on the
         orchestrator side, but the badge renders regardless (empty label
         when no runtime selected yet). Addresses report complaint #3
         (surprise pricing / silent caps) and #9 (prompt-injection
         visibility). -->
    <div class="status-strip" data-testid="status-strip">
      {#if runtimeBadgeLabel}
        <span class="status-badge status-badge--runtime status-badge--{contextSnapshot.runtime_choice_badge}">
          runtime: {runtimeBadgeLabel}
        </span>
      {/if}
      <!-- E-3 sub-phase 4B — ROLE badge. Surfaces voice-section role
           (planner / coder / critic) instead of the underlying model
           name. Click to expand a drawer that shows the backend
           identity (sidecar / anthropic_api / etc.). Per E-3 report
           "expose-via-drawer not primary surface" pattern. -->
      {#if roleBadgeLabel}
        <button
          type="button"
          class="status-badge status-badge--role"
          data-testid="role-badge"
          aria-expanded={showRoleDrawer}
          aria-label={showRoleDrawer
            ? `Hide backend detail (currently ${contextSnapshot.current_runtime ?? 'unknown'})`
            : 'Show backend detail'}
          onclick={() => { showRoleDrawer = !showRoleDrawer; }}
        >
          {roleBadgeLabel}
        </button>
      {/if}
      {#if showRoleDrawer && contextSnapshot.current_runtime}
        <span class="status-badge status-badge--backend" data-testid="role-badge-drawer">
          backend: {contextSnapshot.current_runtime}
        </span>
      {/if}
      <span class="status-badge status-badge--injection status-badge--injection-{injectionStatusLabel}">
        injection: {injectionStatusLabel}
      </span>
    </div>

    <!-- B-39D: Thinking + Effort affordances. Compact ghost row above the
         shell. When thinkingEnabled, the next submit invokes the REFLECT
         pipeline (TRIAGE > REASONER N > AUDITOR > SYNTHESIZE) instead of
         streaming. Effort level maps to REASONER best-of-N width. -->
    <div class="reflect-controls">
      <button
        class="reflect-toggle"
        class:reflect-toggle--on={session.thinkingEnabled}
        type="button"
        aria-label={session.thinkingEnabled ? 'Disable Thinking (REFLECT off)' : 'Enable Thinking (REFLECT pipeline)'}
        title="When ON: chat turn goes through TRIAGE → REASONER best-of-N → AUDITOR → SYNTHESIZE. +5-25s latency per turn; deeper answers."
        onclick={() => session.toggleThinking()}
      >
        <span class="reflect-dot" aria-hidden="true"></span>
        <span class="reflect-label mono">thinking {session.thinkingEnabled ? 'on' : 'off'}</span>
      </button>
      <button
        class="reflect-effort"
        class:reflect-effort--dim={!session.thinkingEnabled}
        type="button"
        aria-label={`Cycle effort level (current: ${session.effortLevel})`}
        title="REASONER best-of-N width: low=2 · medium=4 · high=6 · max=8 candidates"
        disabled={!session.thinkingEnabled}
        onclick={() => session.cycleEffort()}
      >
        <span class="reflect-effort-label mono">effort</span>
        <span class="reflect-effort-value mono">{session.effortLevel}</span>
        <span class="reflect-effort-dots" aria-hidden="true">
          {#each EFFORT_LEVELS as lvl}
            <span
              class="reflect-effort-dot"
              class:reflect-effort-dot--filled={EFFORT_RANK[session.effortLevel] >= EFFORT_RANK[lvl]}
            ></span>
          {/each}
        </span>
      </button>
    </div>

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
          aria-label="Prompt — press Enter or Ctrl+Enter to send"
          oninput={handleInput}
          onkeydown={handleKeydown}
          onfocus={() => commands.setFocused(true)}
          onblur={() => commands.setFocused(false)}
        ></textarea>

        <!-- C-6: Stop and Send are mutually exclusive. Stop wires to existing
             commands.cancelStream() at commands.svelte.ts:395; no new IPC. -->
        {#if commands.isStreaming}
          <button
            class="stop-btn"
            aria-label="Stop streaming"
            data-testid="stop-button"
            onclick={() => commands.cancelStream()}
          >
            <Square size={12} strokeWidth={1.5} />
            <span class="stop-btn-label">Stop</span>
          </button>
        {:else if commands.inputValue.trim().length > 0}
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

  /* E-4 Task 3: pinned-rules strip. Sits above the stream so the user
     can see active rules without scrolling away from the bubble area.
     Collapsed by default; expands on click. */
  .pinned-rules-strip {
    flex: 0 0 auto;
    padding: 4px var(--s-3, 8px) 0 var(--s-3, 8px);
    position: relative;
    z-index: 2;
  }

  /* E-4 Tasks 1, 2, 12: bubble-level chat UX critique overlays. The
     chips + badge sit on their own row beneath the card body so they
     never collide with existing card chrome. The correction block
     follows on its own block-level line. */
  .chat-ux-claims,
  .chat-ux-sycophancy {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
    margin-top: 4px;
    padding: 0 var(--s-2, 4px);
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
    font-size: 10px;
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
    font-size: 10px;
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
    font-size: 10px;
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
    font-size: 10px;
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

  /* ---- B-39D: Thinking + Effort affordances above the chat shell ---- */
  .reflect-controls {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: 0 var(--s-4) 4px;
    flex-wrap: wrap;
  }
  .reflect-toggle,
  .reflect-effort {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-pill);
    padding: 3px 10px;
    font-size: 10.5px;
    color: var(--text-ghost);
    cursor: pointer;
    outline: none;
    transition:
      color var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std),
      background var(--dur-fast) var(--ease-std);
  }
  .reflect-toggle:hover,
  .reflect-effort:hover:not(:disabled) {
    color: var(--text-secondary);
    border-color: var(--border-default);
  }
  .reflect-toggle:focus-visible,
  .reflect-effort:focus-visible {
    box-shadow: var(--ring);
  }
  .reflect-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--text-disabled);
    transition: background var(--dur-fast) var(--ease-std), box-shadow var(--dur-fast) var(--ease-std);
  }
  .reflect-toggle--on {
    color: var(--teal);
    border-color: var(--teal-line);
    background: var(--teal-ghost);
  }
  .reflect-toggle--on .reflect-dot {
    background: var(--teal);
    box-shadow: 0 0 4px 0 var(--teal);
  }
  .reflect-label {
    letter-spacing: 0.04em;
    text-transform: lowercase;
  }
  .reflect-effort--dim {
    opacity: 0.45;
    cursor: not-allowed;
  }
  .reflect-effort-label {
    letter-spacing: 0.04em;
    text-transform: lowercase;
    opacity: 0.7;
  }
  .reflect-effort-value {
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    font-weight: 500;
    font-size: 10px;
  }
  .reflect-effort-dots {
    display: inline-flex;
    gap: 3px;
    margin-left: 2px;
  }
  .reflect-effort-dot {
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--text-disabled);
    opacity: 0.55;
    transition: background var(--dur-fast) var(--ease-std), opacity var(--dur-fast) var(--ease-std);
  }
  .reflect-effort-dot--filled {
    background: var(--teal);
    opacity: 1;
  }

  /* ---- Code-answer card (EXPLORE_CODE wire) ---- */
  .code-answer-header {
    font-size: 10px;
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
    font-size: 10px;
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
    font-size: 10px;
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
    font-size: 10px;
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

  /* C-6: Stop button. Mutually exclusive with .send-btn. Matches send height (28px).
     --teal-soft background + --text-primary text per plan task 9. */
  .stop-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    height: 28px;
    padding: 0 10px;
    flex-shrink: 0;
    background: var(--teal-soft);
    border: 1px solid var(--teal-line);
    border-radius: var(--r-sm);
    cursor: pointer;
    color: var(--text-primary);
    font-family: var(--font-sans);
    font-size: 12px;
    outline: none;
    transition:
      background var(--dur-fast) var(--ease-std),
      border-color var(--dur-fast) var(--ease-std),
      color var(--dur-fast) var(--ease-std);
  }

  .stop-btn:hover {
    background: var(--teal-ghost);
    border-color: var(--teal);
    color: var(--teal-bright);
  }

  .stop-btn:focus-visible {
    box-shadow: var(--ring);
  }

  .stop-btn-label {
    line-height: 1;
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

  /* ---- E-23 first-run positioning + status badges ---- */
  .first-run-positioning {
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-width: 520px;
    margin: var(--s-3) 0 var(--s-2);
    padding: var(--s-3) var(--s-4);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    background: var(--bg-surface);
  }
  .positioning-para {
    font-family: var(--font-sans);
    font-size: 12px;
    line-height: 1.5;
    color: var(--text-secondary);
    margin: 0;
  }
  .positioning-tag {
    display: inline-block;
    padding: 1px 6px;
    margin-right: 6px;
    border: 1px solid var(--border-default);
    border-radius: var(--r-xs);
    font-size: 10px;
    color: var(--teal);
    text-transform: lowercase;
    letter-spacing: 0.04em;
  }
  .positioning-cite {
    display: block;
    margin-top: 4px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    font-style: italic;
  }

  .status-strip {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    padding: 0 var(--s-4) 2px;
  }
  .status-badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-pill);
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }
  .status-badge--local {
    color: var(--teal);
    border-color: var(--teal-line);
    background: var(--teal-ghost);
  }
  .status-badge--cloud_billed {
    color: #d4a017;
    border-color: rgba(212, 160, 23, 0.4);
    background: rgba(212, 160, 23, 0.06);
  }
  .status-badge--injection-passed {
    color: var(--teal);
    border-color: var(--teal-line);
  }
  .status-badge--injection-blocked {
    color: var(--fail);
    border-color: rgba(176, 69, 69, 0.4);
  }
  .status-badge--injection-not_run {
    color: var(--text-ghost);
  }
</style>

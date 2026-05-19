<script lang="ts">
  // B-40 — Idea-Framer surface.
  // Renders the FramedIdea + TradeoffCard for user response capture.
  // Mounted by WindowHost when WindowType=='idea_framing'.

  import * as stub from '$lib/api/stub';
  import type {
    IdeaFramingState,
    TradeoffResponse,
  } from '$lib/api/types';

  // ------------------------------------------------------------------
  // Live state
  // ------------------------------------------------------------------
  let snap = $state<IdeaFramingState | null>(null);
  let loading = $state<boolean>(true);
  let lastError = $state<string | null>(null);

  // Map of tradeoff_name -> selected option (user response capture).
  let userResponses = $state<Record<string, string>>({});

  // Submission state.
  let submitting = $state<boolean>(false);
  let submitOk = $state<boolean | null>(null);

  // ------------------------------------------------------------------
  // Poll loop
  // ------------------------------------------------------------------
  let pollHandle: ReturnType<typeof setInterval> | null = null;
  const POLL_INTERVAL_MS = 3000;

  async function refresh(): Promise<void> {
    try {
      const next = await stub.get_idea_framing_state();
      snap = next;
      lastError = null;
    } catch (e) {
      lastError = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    refresh();
    pollHandle = setInterval(refresh, POLL_INTERVAL_MS);
    return () => {
      if (pollHandle) clearInterval(pollHandle);
      pollHandle = null;
    };
  });

  // ------------------------------------------------------------------
  // Submit handler
  // ------------------------------------------------------------------
  async function onSubmit(): Promise<void> {
    if (!snap?.framed_idea) return;
    const tradeoffs = snap.framed_idea.tradeoff_card?.tradeoffs ?? [];
    const responses: TradeoffResponse[] = [];
    for (const t of tradeoffs) {
      const sel = userResponses[t.name] ?? t.default;
      responses.push({ tradeoff_name: t.name, option: sel });
    }
    submitting = true;
    try {
      const result = await stub.submit_tradeoff_responses(responses);
      submitOk = result.ok;
    } catch (e) {
      submitOk = false;
      lastError = String(e);
    } finally {
      submitting = false;
    }
  }

  function setResponse(tradeoff: string, option: string): void {
    userResponses = { ...userResponses, [tradeoff]: option };
  }
</script>

<div class="framing-window">
  <header class="framing-header">
    <h2>Idea Framing</h2>
    <div class="status-line">
      {#if loading}
        <span class="status">loading...</span>
      {:else if !snap}
        <span class="status status-idle">No active framing session.</span>
      {:else if snap.is_active}
        <span class="status status-active">
          FRAMING active &mdash; Critic rounds: {snap.critic_rounds_used} / 3
        </span>
      {:else if snap.is_converged}
        <span class="status status-converged">
          FRAMING converged &mdash; rounds used: {snap.critic_rounds_used}
        </span>
      {:else}
        <span class="status status-blocked">
          FRAMING blocked &mdash; unresolved BLOCK findings at round {snap.critic_rounds_used}
        </span>
      {/if}
    </div>
    {#if lastError}
      <div class="error-line">backend error: {lastError}</div>
    {/if}
  </header>

  {#if snap?.framed_idea}
    {@const fi = snap.framed_idea}
    <section class="framed-idea">
      <div class="field-row">
        <span class="label">Project class:</span>
        <span class="value">{fi.project_class}</span>
      </div>

      <div class="field-row">
        <span class="label">Raw prompt:</span>
        <span class="value mono">{fi.raw_prompt}</span>
      </div>

      <details open>
        <summary>Research summary ({(fi.research_summary?.streams ?? []).length} streams)</summary>
        <ul>
          {#each (fi.research_summary?.streams ?? []) as s}
            <li>
              <span class="stream-name">{s.stream_name}</span>
              <span class="stream-status status-{s.converged ? 'converged' : 'degraded'}">
                {s.convergence}
              </span>
            </li>
          {/each}
        </ul>
        {#if (fi.research_summary?.degraded_streams ?? []).length > 0}
          <div class="degraded">
            Degraded mandatory streams:
            {(fi.research_summary?.degraded_streams ?? []).join(', ')}
          </div>
        {/if}
      </details>

      <details>
        <summary>Key decisions ({(fi.key_decisions ?? []).length})</summary>
        <ul>
          {#each (fi.key_decisions ?? []) as kd}
            <li>
              <strong>{kd.name}</strong>: {kd.rationale}
            </li>
          {/each}
        </ul>
      </details>

      {#if (fi.known_unknowns ?? []).length > 0}
        <details>
          <summary>Known unknowns ({(fi.known_unknowns ?? []).length})</summary>
          <ul>
            {#each (fi.known_unknowns ?? []) as u}
              <li class="unknown">{u}</li>
            {/each}
          </ul>
        </details>
      {/if}

      <section class="tradeoff-card">
        <h3>Tradeoffs to confirm</h3>
        {#each (fi.tradeoff_card?.tradeoffs ?? []) as t}
          <div class="tradeoff">
            <div class="tradeoff-question">{t.user_facing_question}</div>
            <div class="tradeoff-options">
              {#each t.options as opt}
                <label class="option">
                  <input
                    type="radio"
                    name={t.name}
                    value={opt}
                    checked={(userResponses[t.name] ?? t.default) === opt}
                    onchange={() => setResponse(t.name, opt)}
                  />
                  <span>{opt}</span>
                </label>
              {/each}
            </div>
          </div>
        {/each}

        <button
          class="submit"
          disabled={submitting || !snap.is_converged}
          onclick={onSubmit}
        >
          {submitting ? 'Submitting...' : 'Submit responses'}
        </button>
        {#if submitOk === true}
          <span class="submit-ok">Saved to tradeoff-responses.toml</span>
        {:else if submitOk === false}
          <span class="submit-fail">Submit failed</span>
        {/if}
        {#if !snap.is_converged && snap.framed_idea}
          <div class="hint">
            Framing must converge (zero BLOCK critic findings) before
            tradeoff responses can advance the FRAMING substate.
          </div>
        {/if}
      </section>

      <details class="critic-rounds">
        <summary>Critic rounds ({(fi.critic_rounds ?? []).length})</summary>
        {#each (fi.critic_rounds ?? []) as r}
          <div class="round">
            <div class="round-head">Round {r.round_index}</div>
            <ul>
              {#each r.findings as f}
                <li class="finding finding-{f.severity}">
                  <span class="cat">[{f.category}/{f.severity}]</span>
                  {f.proposed_fix}
                </li>
              {/each}
            </ul>
          </div>
        {/each}
      </details>
    </section>
  {:else if !loading && snap === null}
    <div class="empty">
      No FRAMING session has started yet. Submit a non-trivial /task to
      enter the FRAMING substate.
    </div>
  {/if}
</div>

<style>
  .framing-window {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
    padding: 12px 16px;
    font-family: var(--app-font, system-ui);
    color: var(--text, #ddd);
    background: var(--surface, #1a1a1a);
  }
  .framing-header {
    border-bottom: 1px solid var(--border, #333);
    padding-bottom: 8px;
    margin-bottom: 12px;
  }
  .framing-header h2 {
    margin: 0 0 4px 0;
    font-size: 16px;
    font-weight: 500;
  }
  .status-line {
    font-size: 12px;
  }
  .status-idle    { color: #888; }
  .status-active  { color: #5fa8ff; }
  .status-converged { color: #6fc06f; }
  .status-blocked { color: #ff6b6b; }
  .error-line {
    color: #ff6b6b;
    font-size: 10px;
    margin-top: 4px;
  }
  .field-row {
    display: flex;
    gap: 8px;
    margin: 4px 0;
    font-size: 12px;
  }
  .label  { color: #999; min-width: 110px; }
  .value  { color: #ddd; }
  .value.mono { font-family: monospace; }
  details { margin: 8px 0; }
  details summary {
    cursor: pointer;
    font-size: 12px;
    color: #aaa;
  }
  ul { margin: 4px 0 4px 18px; padding: 0; }
  li { margin: 2px 0; font-size: 12px; }
  .stream-name { font-weight: 500; }
  .stream-status { margin-left: 8px; font-size: 10px; }
  .stream-status.status-converged { color: #6fc06f; }
  .stream-status.status-degraded  { color: #ffaa55; }
  .degraded {
    color: #ffaa55;
    font-size: 10px;
    margin: 4px 0;
  }
  .unknown { color: #ffaa55; }
  .tradeoff-card {
    border: 1px solid var(--border, #333);
    border-radius: 4px;
    padding: 12px;
    margin: 12px 0;
  }
  .tradeoff-card h3 {
    margin: 0 0 8px 0;
    font-size: 13px;
  }
  .tradeoff { margin: 8px 0; }
  .tradeoff-question { font-size: 12px; margin-bottom: 4px; }
  .tradeoff-options {
    display: flex;
    gap: 12px;
    flex-wrap: wrap;
  }
  .option {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 12px;
    cursor: pointer;
  }
  .submit {
    margin-top: 8px;
    padding: 6px 14px;
    background: #2a5cb0;
    color: #fff;
    border: none;
    border-radius: 3px;
    cursor: pointer;
    font-size: 12px;
  }
  .submit:disabled {
    background: #444;
    cursor: not-allowed;
  }
  .submit-ok   { color: #6fc06f; margin-left: 8px; font-size: 10px; }
  .submit-fail { color: #ff6b6b; margin-left: 8px; font-size: 10px; }
  .hint {
    color: #aaa;
    font-size: 10px;
    margin-top: 6px;
  }
  .finding-block { color: #ff6b6b; }
  .finding-warn  { color: #ffaa55; }
  .finding-info  { color: #888; }
  .cat {
    font-family: monospace;
    margin-right: 6px;
  }
  .round-head {
    font-weight: 500;
    margin-top: 6px;
  }
  .empty {
    color: #888;
    font-style: italic;
    padding: 32px;
    text-align: center;
  }
</style>

<script lang="ts">
  // NodeStoryPanel.svelte — the "NEW DIFF" node-story (the user's centerpiece).
  //
  // Open a node -> its whole STORY: HISTORY / EXPLAIN / WIRING / RECEIPTS /
  // COST / ACTIVITY. The raw diff is COLLAPSED, one click deep, never shoved
  // first (diff fatigue is real — SO2025 66% almost-right). The EXPLAIN layer
  // is cached plain-language (B-13 pattern; a toggle swaps cached strings, no
  // live LLM call from the UI).
  //
  // Backend pending: this renders the NODE_STORY_FIXTURE. The real story reads
  // the SSB journal + G-5 receipts + the DKG concept field; the shape is stable
  // so wiring is a fixture->command swap. Section nav uses the center->out
  // underline idiom (DESIGN §13), flat-matte + role tokens + quinary glyphs.

  import NodeStateGlyph from './NodeStateGlyph.svelte';
  import {
    STORY_SECTIONS,
    STORY_SECTION_LABEL,
    NODE_STORY_FIXTURE,
    type StorySection,
    type NodeStory,
  } from './node_story';

  interface Props {
    /** The story to render (defaults to the mock fixture). */
    story?: NodeStory;
    /** Called when the user closes the story panel. */
    onClose?: () => void;
  }

  let { story = NODE_STORY_FIXTURE, onClose }: Props = $props();

  let section = $state<StorySection>('history');
  // EXPLAIN toggle: false = technical title, true = cached plain-language.
  let plainLanguage = $state<boolean>(false);
  // Which history entry has its raw diff revealed (one click deep).
  let openDiff = $state<number | null>(null);

  function verdictGlyph(v: 'pass' | 'fail' | 'warn'): 'done' | 'fail' | 'warn' {
    return v === 'pass' ? 'done' : v === 'fail' ? 'fail' : 'warn';
  }
  function wiringGlyph(v: string): 'done' | 'active' | 'pending' | 'fail' {
    if (v === 'verified') return 'done';
    if (v === 'wired') return 'active';
    if (v === 'broken') return 'fail';
    return 'pending';
  }
</script>

<div class="story">
  <div class="story-head">
    <span class="story-kicker">NODE STORY</span>
    <h3 class="story-title">{plainLanguage ? story.explainCached : story.title}</h3>
    <button
      class="explain-toggle"
      class:on={plainLanguage}
      title="Explain in plain language (cached — no live model call)"
      onclick={() => { plainLanguage = !plainLanguage; }}
    >explain</button>
    {#if onClose}
      <button class="story-close" aria-label="close" onclick={onClose}>✕</button>
    {/if}
  </div>

  <!-- safety verdict — plain-language per-node, the most-requested feature -->
  <div class="safety">
    <NodeStateGlyph state="done" size={12} label="safe" />
    <span>{story.safetyVerdict}</span>
  </div>

  <!-- section nav — center->out underline idiom -->
  <div class="story-nav" role="tablist" aria-label="Story sections">
    {#each STORY_SECTIONS as s (s)}
      <button
        class="nav-btn"
        class:active={section === s}
        role="tab"
        aria-selected={section === s}
        onclick={() => { section = s; }}
      >
        {STORY_SECTION_LABEL[s]}
        <span class="underline" aria-hidden="true"></span>
      </button>
    {/each}
  </div>

  <div class="story-body">
    {#if section === 'history'}
      <ul class="hist">
        {#each story.history as h, i (i)}
          <li class="hist-entry">
            <div class="hist-top">
              <span class="hist-ts">{h.ts}</span>
              <span class="hist-conf hist-conf--{h.confidence}">{h.confidence}</span>
            </div>
            <div class="hist-summary">{h.summary}</div>
            <div class="hist-why">{h.why}</div>
            <div class="hist-scope">{h.scope}</div>
            <div class="hist-receipts">
              {#each h.receipts as r (r)}<span class="rchip">{r}</span>{/each}
            </div>
            <!-- raw diff COLLAPSED, one click deep -->
            <button class="diff-toggle" onclick={() => { openDiff = openDiff === i ? null : i; }}>
              {openDiff === i ? '▾ hide raw diff' : '▸ raw diff'}
            </button>
            {#if openDiff === i}
              <pre class="diff-body">{h.diff}</pre>
            {/if}
          </li>
        {/each}
      </ul>

    {:else if section === 'explain'}
      <p class="explain-body">{story.explainCached}</p>
      <span class="explain-note">cached plain-language (deterministic, no model call)</span>

    {:else if section === 'wiring'}
      <ul class="wiring">
        {#each story.wiring as w (w.label)}
          <li class="wire-row">
            <NodeStateGlyph state={wiringGlyph(w.verdict)} size={12} label={w.verdict} />
            <span class="wire-label">{w.label}</span>
            <span class="wire-val">{w.value}</span>
          </li>
        {/each}
      </ul>

    {:else if section === 'receipts'}
      <ul class="receipts">
        {#each story.receipts as r (r.gate)}
          <li class="receipt-row">
            <NodeStateGlyph state={verdictGlyph(r.verdict)} size={12} label={r.verdict} />
            <span class="receipt-gate">{r.gate}</span>
            <span class="receipt-detail">{r.detail}</span>
          </li>
        {/each}
      </ul>

    {:else if section === 'cost'}
      <div class="cost">
        <div class="cost-row"><span class="cost-label">MODEL</span><span class="cost-val">{story.cost.model}</span></div>
        <div class="cost-row"><span class="cost-label">TOKENS IN</span><span class="cost-val">{story.cost.tokensIn.toLocaleString()}</span></div>
        <div class="cost-row"><span class="cost-label">TOKENS OUT</span><span class="cost-val">{story.cost.tokensOut.toLocaleString()}</span></div>
        <div class="cost-row"><span class="cost-label">COST</span><span class="cost-val cost-usd">{story.cost.usd}</span></div>
      </div>

    {:else if section === 'activity'}
      <ul class="activity">
        {#each story.activity as a, i (i)}
          <li class="act-row">
            <span class="act-ts">{a.ts}</span>
            <span class="act-line">{a.line}</span>
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</div>

<style>
  .story {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    background: var(--bg);
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--read);
  }

  .story-head {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px 4px;
  }
  .story-kicker {
    font-size: 10px;
    letter-spacing: 0.16em;
    color: var(--dim);
    flex: none;
  }
  .story-title {
    flex: 1 1 auto;
    font-size: 12px;
    color: var(--text);
    font-weight: 500;
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .explain-toggle {
    flex: none;
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.08em;
    padding: 2px 8px;
    background: none;
    border: 1px solid var(--line2);
    border-radius: var(--r-pill);
    color: var(--dim);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease-out), border-color var(--dur-fast) var(--ease-out);
  }
  .explain-toggle:hover { color: var(--text); }
  .explain-toggle.on { color: var(--d-teal-bright); border-color: var(--d-teal-soft); }
  .story-close {
    flex: none;
    background: none; border: none; color: var(--dim);
    font-size: 10px; cursor: pointer;
  }
  .story-close:hover { color: var(--text); }

  .safety {
    display: flex;
    align-items: center;
    gap: 7px;
    padding: 4px 12px 8px;
    color: var(--read);
    font-size: 10px;
    line-height: var(--leading-text);
  }

  .story-nav {
    display: flex;
    gap: 2px;
    padding: 0 8px;
    border-bottom: 1px solid var(--line);
    flex-shrink: 0;
  }
  .nav-btn {
    position: relative;
    padding: 6px 9px 8px;
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.12em;
    background: none;
    border: none;
    color: var(--dim);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease-out);
  }
  .nav-btn:hover:not(.active) { color: var(--text); }
  .nav-btn.active { color: var(--d-teal-bright); }
  .underline {
    position: absolute;
    left: 7px; right: 7px; bottom: 0;
    height: 1.5px;
    background: var(--d-teal-bright);
    transform: scaleX(0);
    transform-origin: center;
    transition: transform var(--dur-base) var(--ease-out);
  }
  .nav-btn:hover .underline,
  .nav-btn.active .underline { transform: scaleX(1); }

  .story-body {
    flex: 1 1 auto;
    overflow-y: auto;
    padding: 10px 12px;
  }

  /* HISTORY */
  .hist { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 12px; }
  .hist-entry {
    border-left: 2px solid var(--d-teal-soft);
    padding-left: 10px;
  }
  .hist-top { display: flex; align-items: baseline; gap: 8px; }
  .hist-ts { color: var(--dim); font-size: 10px; }
  .hist-conf { font-size: 8px; letter-spacing: 0.08em; text-transform: uppercase; }
  .hist-conf--high { color: var(--d-teal-bright); }
  .hist-conf--medium { color: var(--read); }
  .hist-conf--low { color: var(--dim); }
  .hist-summary { color: var(--text); font-size: 10px; margin: 2px 0; line-height: var(--leading-text); }
  .hist-why { color: var(--read); font-size: 10px; line-height: var(--leading-text); }
  .hist-scope { color: var(--dim); font-size: 10px; margin-top: 3px; }
  .hist-receipts { display: flex; flex-wrap: wrap; gap: 4px; margin: 5px 0 3px; }
  .rchip {
    font-size: 8px;
    padding: 1px 5px;
    border: 1px solid var(--line2);
    border-radius: var(--r-xs);
    color: var(--read);
  }
  .diff-toggle {
    background: none; border: none;
    color: var(--dim); font-family: var(--font-mono);
    font-size: 10px; cursor: pointer; padding: 2px 0;
    transition: color var(--dur-fast) var(--ease-out);
  }
  .diff-toggle:hover { color: var(--text); }
  .diff-body {
    margin: 4px 0 0;
    padding: 8px;
    background: var(--sink);
    border: 1px solid var(--line);
    border-radius: var(--r-sm);
    font-size: 10px;
    color: var(--read);
    white-space: pre-wrap;
    overflow-x: auto;
  }

  /* EXPLAIN */
  .explain-body { color: var(--text); font-size: 12px; line-height: 1.6; margin: 0 0 8px; }
  .explain-note { color: var(--dim); font-size: 10px; font-style: italic; }

  /* WIRING / RECEIPTS / ACTIVITY rows */
  .wiring, .receipts, .activity { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 6px; }
  .wire-row, .receipt-row { display: flex; align-items: center; gap: 8px; }
  .wire-label, .receipt-gate { color: var(--dim); font-size: 10px; min-width: 92px; text-transform: uppercase; letter-spacing: 0.06em; }
  .wire-val, .receipt-detail { color: var(--read); font-size: 10px; overflow-wrap: anywhere; }

  .act-row { display: flex; gap: 10px; }
  .act-ts { color: var(--dim); font-size: 10px; min-width: 64px; flex: none; }
  .act-line { color: var(--read); font-size: 10px; }

  /* COST */
  .cost { display: flex; flex-direction: column; gap: 5px; }
  .cost-row { display: flex; gap: 10px; padding: 3px 0; border-bottom: 1px solid var(--line); }
  .cost-label { color: var(--ghost); font-size: 10px; letter-spacing: 0.1em; min-width: 84px; flex: none; }
  .cost-val { color: var(--read); font-size: 10px; }
  .cost-usd { color: var(--d-teal-bright); }
</style>

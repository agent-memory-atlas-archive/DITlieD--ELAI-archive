<script lang="ts">
  // CodebaseLensBar.svelte — the CODEBASE tab's 5 LENSES (brief §4 / canvas
  // decision ledger). One QUESTION per lens, not five toggles competing for
  // attention: Structure / Security / History / Cost / Dead-code. The active
  // lens names the single question the code graph is answering right now.
  //
  // This is the lens CHROME (the surface + the question framing). Each lens's
  // data overlay rides the existing graph backend as it lands; the bar emits
  // the active lens via the onLens callback so the host graph can re-colour.
  // Uses the center->out underline idiom (DESIGN §13) — no dot, no box.

  export type Lens = 'structure' | 'security' | 'history' | 'cost' | 'deadcode';

  interface LensSpec {
    lens: Lens;
    label: string;
    /** The single question this lens answers (canvas decision ledger). */
    question: string;
    /** True when the lens overlay is wired to a real backend query. */
    live: boolean;
  }

  const LENSES: readonly LensSpec[] = [
    { lens: 'structure', label: 'STRUCTURE', question: 'how is this code organised?', live: true },
    { lens: 'security',  label: 'SECURITY',  question: 'where could this be attacked?', live: false },
    { lens: 'history',   label: 'HISTORY',   question: 'what changed here, and when?', live: false },
    { lens: 'cost',      label: 'COST',      question: 'what did building this cost?', live: false },
    { lens: 'deadcode',  label: 'DEAD-CODE', question: 'what is never reached?', live: true },
  ];

  interface Props {
    /** Currently active lens. */
    active?: Lens;
    /** Called when the user switches lens. */
    onLens?: (lens: Lens) => void;
  }

  let { active = 'structure', onLens }: Props = $props();

  let current = $state<Lens>(active);

  function pick(l: Lens) {
    current = l;
    onLens?.(l);
  }

  const currentSpec = $derived(LENSES.find((l) => l.lens === current) ?? LENSES[0]);
</script>

<div class="lensbar">
  <div class="lenses" role="tablist" aria-label="Codebase lens">
    {#each LENSES as l (l.lens)}
      <button
        class="lens"
        class:active={current === l.lens}
        class:dim={!l.live}
        role="tab"
        aria-selected={current === l.lens}
        title={l.live ? l.question : `${l.question} (lens overlay pending)`}
        onclick={() => pick(l.lens)}
      >
        {l.label}
        {#if !l.live}<span class="soon">·</span>{/if}
        <span class="underline" aria-hidden="true"></span>
      </button>
    {/each}
  </div>
  <!-- the one question this lens is answering -->
  <span class="question">{currentSpec.question}</span>
</div>

<style>
  .lensbar {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 4px 12px;
    background: var(--bg);
    border-bottom: 1px solid var(--line);
    flex-shrink: 0;
  }

  .lenses { display: flex; gap: 2px; }

  .lens {
    position: relative;
    padding: 4px 10px 7px;
    font-family: var(--font-mono);
    font-size: 10px;
    letter-spacing: 0.14em;
    background: none;
    border: none;
    color: var(--dim);
    cursor: pointer;
    transition: color var(--dur-fast) var(--ease-out);
  }
  .lens:hover:not(.active) { color: var(--text); }
  .lens.active { color: var(--d-teal-bright); }
  .lens.dim { opacity: 0.7; }
  .soon { color: var(--ghost); margin-left: 1px; }

  .underline {
    position: absolute;
    left: 8px; right: 8px; bottom: 1px;
    height: 1.5px;
    background: var(--d-teal-bright);
    transform: scaleX(0);
    transform-origin: center;
    transition: transform var(--dur-base) var(--ease-out);
  }
  .lens:hover .underline,
  .lens.active .underline { transform: scaleX(1); }

  .question {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--read);
    font-style: italic;
  }

  @media (prefers-reduced-motion: reduce) {
    .underline { transition: none; }
  }
</style>

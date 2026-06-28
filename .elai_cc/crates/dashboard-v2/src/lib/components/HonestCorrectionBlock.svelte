<script lang="ts">
  // E-4 Task 12: honest-correction surface.
  //
  // Styled block rendered inside a CommandWindow bubble when ELAI
  // realises it was wrong (test failed, doc disagreed, user
  // corrected). Surface text canonically opens "I was wrong.
  // Correct: X because Y." per CLAUDE.md "When wrong" rule and
  // anthropic.com/constitution ("diplomatic honesty rather than
  // dishonest diplomacy").
  //
  // Wire shape mirrors crates::orchestrator::chat_ux::HonestCorrection
  // (original_claim_blake3_hex, correction_claim_blake3_hex, trigger).
  // Component just RENDERS; the SSB row is emitted from the
  // orchestrator producer (gated by ELAI_CHAT_UX_CRITIQUE_ENABLE).
  //
  // Distinct from the existing intervention card (Approve / Reject /
  // Modify / Redirect): this is a passive retraction surface, not an
  // interactive prompt.
  //
  // Svelte 5 runes only.

  type HonestCorrectionTrigger =
    | 'test_failed'
    | 'doc_disagreed'
    | 'user_corrected';

  interface Props {
    original_claim: string;
    correction_text: string;
    trigger: HonestCorrectionTrigger;
    /** Optional blake3-prefix shown next to the trigger badge for dedup. */
    original_claim_blake3_hex?: string;
  }

  const {
    original_claim,
    correction_text,
    trigger,
    original_claim_blake3_hex,
  }: Props = $props();

  const TRIGGER_LABEL: Record<HonestCorrectionTrigger, string> = {
    test_failed: 'test failed',
    doc_disagreed: 'doc disagreed',
    user_corrected: 'user corrected',
  };

  let blakePrefix = $derived.by(() => {
    if (!original_claim_blake3_hex) return '';
    return original_claim_blake3_hex.slice(0, 8);
  });
</script>

<aside
  class="hcb"
  role="note"
  aria-label="honest correction block"
>
  <header class="hcb-header">
    <span class="hcb-glyph" aria-hidden="true">⤺</span>
    <span class="hcb-title">i was wrong</span>
    <span class="hcb-trigger mono">{TRIGGER_LABEL[trigger]}</span>
    {#if blakePrefix}
      <span class="hcb-blake mono">{blakePrefix}</span>
    {/if}
  </header>
  <div class="hcb-body">
    <div class="hcb-original">
      <span class="hcb-label">prior:</span>
      <span class="hcb-original-text">{original_claim}</span>
    </div>
    <div class="hcb-correction">
      <span class="hcb-label">correct:</span>
      <span class="hcb-correction-text">{correction_text}</span>
    </div>
  </div>
</aside>

<style>
  .hcb {
    margin: var(--s-2, 6px) 0;
    padding: var(--s-2, 8px) var(--s-3, 10px);
    border-left: 2px solid var(--orange, #fb923c);
    background: var(--bg-surface-subtle, rgba(251, 146, 60, 0.04));
    border-radius: 0 var(--r-md, 4px) var(--r-md, 4px) 0;
    font-family: var(--font-sans);
    font-size: 12px;
    line-height: var(--leading-text);
  }
  .hcb-header {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 4px;
  }
  .hcb-glyph {
    color: var(--orange, #fb923c);
    font-weight: 700;
  }
  .hcb-title {
    color: var(--text-primary);
    font-weight: 600;
    text-transform: lowercase;
  }
  .hcb-trigger {
    color: var(--text-ghost);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .hcb-blake {
    color: var(--text-ghost);
    font-size: 10px;
    margin-left: auto;
  }
  .hcb-body {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .hcb-original,
  .hcb-correction {
    display: flex;
    gap: 6px;
  }
  .hcb-label {
    color: var(--text-ghost);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    flex-shrink: 0;
    width: 50px;
  }
  .hcb-original-text {
    color: var(--text-secondary);
    text-decoration: line-through;
    text-decoration-thickness: 1px;
  }
  .hcb-correction-text {
    color: var(--text-primary);
  }
  .mono {
    font-family: var(--font-mono);
  }
</style>

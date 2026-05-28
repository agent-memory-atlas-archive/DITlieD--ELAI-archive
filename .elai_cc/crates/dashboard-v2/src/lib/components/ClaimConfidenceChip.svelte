<script lang="ts">
  // E-4 Task 1: per-claim confidence chip.
  //
  // Inline pill rendered before a claim sentence in CommandWindow bubbles.
  // DISTINCT from ConfidenceTag.svelte (orbit chips, value/size/label
  // props for per-verdict GRC dots): this chip carries a 4-tier enum +
  // optional EvidenceRef + click-to-expand affordance, matching the
  // crates::orchestrator::chat_ux::ConfidenceTier wire shape.
  //
  // Tier source-of-truth: crates/orchestrator/src/chat_ux/mod.rs
  // ConfidenceTier (Verified | ReadFromSource | FromTraining | Guessed).
  // Maps to CLAUDE.md "Confidence calibration" rule.
  //
  // Grounding:
  //   * arXiv:2503.02623 (Rewarding Doubt) — PRIMARY for the
  //     per-claim confidence-surface UX.
  //   * arXiv:2505.24858 (MetaFaith) — linguistic uncertainty markers.
  //   * arXiv:2412.02904 — uncertainty signaling.
  //
  // Svelte 5 runes only. No createEventDispatcher, no $: labels.

  type ChatUxConfidenceTier =
    | 'verified'
    | 'read_from_source'
    | 'from_training'
    | 'guessed';

  // Mirrors crates::orchestrator::chat_ux::EvidenceRef tagged union.
  type EvidenceRefInput =
    | { kind: 'file_ref'; value: string }
    | { kind: 'ssb_seq'; value: number }
    | { kind: 'det_check'; value: string }
    | null;

  interface Props {
    tier: ChatUxConfidenceTier;
    evidence_ref?: EvidenceRefInput;
    label?: string;
    onclick?: (ev: MouseEvent) => void;
  }

  const { tier, evidence_ref = null, label, onclick }: Props = $props();

  const TIER_LABEL: Record<ChatUxConfidenceTier, string> = {
    verified: 'verified',
    read_from_source: 'read from source',
    from_training: 'from training',
    guessed: 'guessed',
  };

  // Wong-palette mapping (same family as ConfidenceTag for visual
  // consistency, but distinct values so the two chips read as
  // different signals when co-located in the bubble).
  const TIER_GLYPH: Record<ChatUxConfidenceTier, string> = {
    verified: '✓',         // check
    read_from_source: '◎', // bullseye (cites source)
    from_training: '·',    // mid-dot (low-evidence)
    guessed: '?',               // open question
  };

  let displayLabel = $derived(label ?? TIER_LABEL[tier]);
  let glyph = $derived(TIER_GLYPH[tier]);
  let evidenceTitle = $derived.by(() => {
    if (!evidence_ref) return TIER_LABEL[tier];
    switch (evidence_ref.kind) {
      case 'file_ref':  return `evidence: ${evidence_ref.value}`;
      case 'ssb_seq':   return `evidence: ssb seq ${evidence_ref.value}`;
      case 'det_check': return `evidence: ${evidence_ref.value}`;
      default:          return TIER_LABEL[tier];
    }
  });

  let clickable = $derived(typeof onclick === 'function' && evidence_ref !== null);
</script>

<button
  type="button"
  class="ccc ccc--{tier}"
  class:ccc--clickable={clickable}
  title={evidenceTitle}
  aria-label="confidence {TIER_LABEL[tier]}"
  onclick={onclick}
  disabled={!clickable}
>
  <span class="ccc-glyph" aria-hidden="true">{glyph}</span>
  <span class="ccc-label">{displayLabel}</span>
</button>

<style>
  .ccc {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    border-radius: var(--r-pill);
    font-family: var(--font-mono);
    font-size: 10px;
    line-height: 1.2;
    border: 1px solid currentColor;
    background: var(--bg-surface);
    cursor: default;
    user-select: none;
    white-space: nowrap;
  }
  .ccc--clickable {
    cursor: pointer;
  }
  .ccc:disabled {
    cursor: default;
    opacity: 0.95;
  }
  .ccc-glyph {
    font-weight: 600;
  }
  .ccc-label {
    color: var(--text-primary);
  }
  .ccc--verified {
    color: var(--confidence-verified, var(--teal, #2dd4bf));
  }
  .ccc--read_from_source {
    color: var(--confidence-ok, var(--blue, #60a5fa));
  }
  .ccc--from_training {
    color: var(--confidence-low, var(--yellow, #facc15));
  }
  .ccc--guessed {
    color: var(--confidence-needs-evidence, var(--orange, #fb923c));
  }
</style>

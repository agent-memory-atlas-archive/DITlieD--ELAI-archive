<script lang="ts">
  // E-4 Task 2 — wire-side complement: SycophancyGuardBadge.
  //
  // DISTINCT from the 3 pre-existing sycophancy surfaces:
  //   * SycophancyFlagStrip (D-27 GAP-1) — per-row flag history.
  //   * SycophancyMatchedPairPanel (E-54) — matched-pair benchmark view.
  //   * SycophancyBenchPanel (D-23) — bench-run results.
  //
  // This badge is bubble-level: when AUDITOR holds position against a
  // user assertion (SycophancyVerdict::HoldPosition with evidence), a
  // small "guard held" pill appears next to the claim with the
  // EvidenceRef as a tooltip.
  //
  // When AUDITOR has NO evidence to pushback (SycophancyVerdict::
  // NoEvidence), the badge renders in a muted variant so the user
  // can see the model EXPLICITLY did not have evidence — per the
  // report's "If you have no evidence, say so explicitly; do NOT
  // agree to be agreeable" rule.
  //
  // Wire shape mirrors crates::orchestrator::chat_ux::SycophancyVerdict.
  //
  // Grounding:
  //   * arXiv:2310.13548 (Sharma sycophancy — PRIMARY).
  //   * openai.com/index/sycophancy-in-gpt-4o (April 2025 rollback).
  //   * arXiv:2502.10844 (Be Friendly Not Friends — neutral LLMs
  //     enhance trust; complimentary LLMs reduce it).
  //
  // Svelte 5 runes only.

  type SycophancyVerdictKind = 'hold_position' | 'no_evidence';

  type EvidenceRefInput =
    | { kind: 'file_ref'; value: string }
    | { kind: 'ssb_seq'; value: number }
    | { kind: 'det_check'; value: string }
    | null;

  interface Props {
    verdict: SycophancyVerdictKind;
    evidence_ref?: EvidenceRefInput;
  }

  const { verdict, evidence_ref = null }: Props = $props();

  let label = $derived<string>(
    verdict === 'hold_position' ? 'guard held' : 'no evidence',
  );

  let title = $derived.by(() => {
    if (verdict === 'no_evidence') {
      return 'sycophancy guard: no evidence for pushback';
    }
    if (!evidence_ref) return 'sycophancy guard: held position';
    switch (evidence_ref.kind) {
      case 'file_ref':  return `held position; evidence: ${evidence_ref.value}`;
      case 'ssb_seq':   return `held position; evidence: ssb seq ${evidence_ref.value}`;
      case 'det_check': return `held position; evidence: ${evidence_ref.value}`;
      default:          return 'held position';
    }
  });

  let glyph = $derived(verdict === 'hold_position' ? '⊨' : '∅');
</script>

<span
  class="sgb sgb--{verdict}"
  role="status"
  aria-label="sycophancy guard {label}"
  title={title}
>
  <span class="sgb-glyph" aria-hidden="true">{glyph}</span>
  <span class="sgb-label">{label}</span>
</span>

<style>
  .sgb {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    border-radius: var(--r-pill, 999px);
    font-family: var(--font-mono);
    font-size: 10px;
    line-height: 1.2;
    border: 1px solid currentColor;
    background: var(--bg-surface);
    user-select: none;
    white-space: nowrap;
  }
  .sgb-glyph {
    font-weight: 600;
  }
  .sgb-label {
    color: var(--text-primary);
  }
  .sgb--hold_position {
    color: var(--teal, #2dd4bf);
  }
  .sgb--no_evidence {
    color: var(--text-ghost);
    opacity: 0.85;
  }
</style>

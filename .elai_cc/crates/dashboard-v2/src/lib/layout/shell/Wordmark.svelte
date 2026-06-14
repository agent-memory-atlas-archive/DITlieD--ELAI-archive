<!-- Wordmark.svelte — shared ELAI text wordmark.
     The project-selector's text treatment (restored per user preference
     over the stroke-SVG): JetBrains Mono, semibold, wide-tracked,
     near-white (--text) on dark / ink on light. One shared component used
     by both the ProjectSelector topbar and the workflow TabBar so the two
     can never drift. (The animated stroke-draw wordmark stays on the Intro
     hero only — DESIGN §5; this is the small chrome lockup.) -->
<script lang="ts"></script>

<span class="wordmark">ELAI</span>

<style>
  .wordmark {
    flex: none;
    font-family: var(--font-mono);
    font-weight: 600;
    font-size: 13px;
    letter-spacing: 0.18em;
    line-height: 1;
    color: var(--text);
    white-space: nowrap;
    user-select: none;
    /* trim the trailing tracking so a following "·" sits tight */
    margin-right: -0.18em;
  }
</style>

<script lang="ts">
  // KnowledgeBrain.svelte — the KNOWLEDGE tab's notes/ideas "brain" canvas
  // (brief §4 / canvas decision ledger). The LLM DRAFTS notes; the user
  // CONNECTS them; research findings accrue here as connectable notes
  // (research is both contextual to a plan node AND cumulative here).
  //
  // This is the Graphify-style concept-graph over a non-code corpus the
  // ledger called for — curatable memory with provenance + taint. NOT the
  // global Obsidian hairball: notes are few, scoped, and the user owns the
  // connections. Flat-matte work surface, role tokens, quinary glyphs.
  //
  // Backend pending: renders mock fixtures. The real brain reads curated
  // memory facts (provenance + taint) + accrued research findings from the
  // SSB/DKG; the note/edge shapes here are the wiring target.

  import NodeStateGlyph from './NodeStateGlyph.svelte';

  type NoteKind = 'fact' | 'idea' | 'research' | 'draft';
  type Provenance = 'verified' | 'research' | 'llm_draft' | 'user';

  interface BrainNote {
    id: string;
    kind: NoteKind;
    title: string;
    body: string;
    provenance: Provenance;
    /** True when the source carries G-1 taint (untrusted ingress). */
    tainted: boolean;
    col: number;
    row: number;
  }
  interface BrainLink {
    from: string;
    to: string;
    /** User-authored connections are solid; LLM-suggested are dashed. */
    authored: boolean;
  }

  const COL_W = 232;
  const ROW_H = 150;
  const NOTE_W = 192;
  const NOTE_H = 104;
  const PAD = 36;

  // Mock brain: a few curated notes + accrued research, with both an
  // authored connection and an LLM-suggested (dashed) one.
  const NOTES: BrainNote[] = [
    { id: 'k_arch', kind: 'fact', title: 'Canvas owns geometry', body: 'Layout algorithm places nodes (dagre/ELK); the model never sets coords. Measured: LLM-placed coords score worse.', provenance: 'verified', tainted: false, col: 0, row: 0 },
    { id: 'k_silence', kind: 'idea', title: 'Silence means safe', body: 'Healthy nodes carry zero badges; alarms are rare hence loud. Done clusters collapse into counted capsules.', provenance: 'user', tainted: false, col: 0, row: 1 },
    { id: 'k_research', kind: 'research', title: 'Diff fatigue is real', body: 'SO2025: 66% of AI changes are almost-right; CodeRabbit 1.7x issues. Lead the node-story, not the raw diff.', provenance: 'research', tainted: false, col: 1, row: 0 },
    { id: 'k_draft', kind: 'draft', title: 'Notes-brain over non-code corpus', body: 'LLM drafts; user connects. Graphify-style concept graph; curatable memory with provenance + taint.', provenance: 'llm_draft', tainted: false, col: 1, row: 1 },
    { id: 'k_ext', kind: 'research', title: 'External feed (untrusted)', body: 'Scraped competitor note — ingress carries G-1 provenance taint until reviewed.', provenance: 'research', tainted: true, col: 2, row: 1 },
  ];
  const LINKS: BrainLink[] = [
    { from: 'k_arch', to: 'k_silence', authored: true },
    { from: 'k_research', to: 'k_draft', authored: true },
    { from: 'k_draft', to: 'k_silence', authored: false }, // LLM-suggested
    { from: 'k_ext', to: 'k_draft', authored: false },
  ];

  const PROV_LABEL: Record<Provenance, string> = {
    verified: 'VERIFIED',
    research: 'RESEARCH',
    llm_draft: 'LLM DRAFT',
    user: 'YOURS',
  };

  function provGlyph(p: Provenance): 'done' | 'active' | 'pending' {
    if (p === 'verified') return 'done';
    if (p === 'user') return 'done';
    if (p === 'research') return 'active';
    return 'pending'; // llm_draft — not yet curated
  }

  let selected = $state<string | null>(null);
  function noteById(id: string) { return NOTES.find((n) => n.id === id); }
  function nx(n: BrainNote) { return PAD + n.col * COL_W; }
  function ny(n: BrainNote) { return PAD + n.row * ROW_H; }
  function ncx(n: BrainNote) { return nx(n) + NOTE_W / 2; }
  function ncy(n: BrainNote) { return ny(n) + NOTE_H / 2; }

  const surfaceW = PAD * 2 + (Math.max(...NOTES.map((n) => n.col)) + 1) * COL_W;
  const surfaceH = PAD * 2 + (Math.max(...NOTES.map((n) => n.row)) + 1) * ROW_H;
</script>

<div class="brain">
  <div class="brain-head">
    <span class="brain-title">Brain</span>
    <span class="brain-meta">{NOTES.length} notes · {LINKS.filter((l) => l.authored).length} connected</span>
    <span class="brain-note-pending" title="Curatable memory + accrued research — backend wiring pending; mock notes shown">backend: pending</span>
  </div>

  <div class="brain-surface-wrap">
    <div class="brain-surface" style:width="{surfaceW}px" style:height="{surfaceH}px">
      <svg class="link-layer" width={surfaceW} height={surfaceH} aria-hidden="true">
        {#each LINKS as l (l.from + '->' + l.to)}
          {@const a = noteById(l.from)}
          {@const b = noteById(l.to)}
          {#if a && b}
            <line class="link" class:suggested={!l.authored} x1={ncx(a)} y1={ncy(a)} x2={ncx(b)} y2={ncy(b)} />
          {/if}
        {/each}
      </svg>

      {#each NOTES as n (n.id)}
        <button
          class="note note--{n.kind}"
          class:sel={selected === n.id}
          style:left="{nx(n)}px" style:top="{ny(n)}px"
          style:width="{NOTE_W}px" style:height="{NOTE_H}px"
          onclick={() => { selected = selected === n.id ? null : n.id; }}
        >
          <div class="note-top">
            <NodeStateGlyph state={provGlyph(n.provenance)} size={12} label={n.provenance} />
            <span class="note-prov">{PROV_LABEL[n.provenance]}</span>
            {#if n.tainted}<span class="note-taint" title="ingress carries G-1 provenance taint">taint</span>{/if}
          </div>
          <div class="note-title">{n.title}</div>
          <div class="note-body">{n.body}</div>
        </button>
      {/each}
    </div>
  </div>

  <div class="brain-foot">
    <span class="legend"><span class="lk solid"></span> you connected</span>
    <span class="legend"><span class="lk dash"></span> ELAI suggested</span>
    <span class="brain-hint">LLM drafts notes · you connect them · research accrues here</span>
  </div>
</div>

<style>
  .brain {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
    background: var(--bg);
    font-family: var(--font-mono);
    font-size: 11px;
  }
  .brain-head {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--line);
    flex-shrink: 0;
  }
  .brain-title { font-weight: 500; font-size: 10px; text-transform: uppercase; letter-spacing: 0.16em; color: var(--text); }
  .brain-meta { color: var(--dim); font-size: 10px; }
  .brain-note-pending { margin-left: auto; color: var(--dim); font-size: 9px; font-style: italic; }

  .brain-surface-wrap { flex: 1 1 auto; min-height: 0; overflow: auto; }
  .brain-surface { position: relative; }

  .link-layer { position: absolute; top: 0; left: 0; pointer-events: none; }
  .link { stroke: var(--d-teal-soft); stroke-width: 1.3; }
  .link.suggested { stroke: var(--line2); stroke-dasharray: 4 4; }

  .note {
    position: absolute;
    display: flex;
    flex-direction: column;
    gap: 3px;
    text-align: left;
    padding: 8px 10px;
    background: var(--panel);
    border: 1px solid var(--line);
    border-radius: var(--r-md);
    cursor: pointer;
    overflow: hidden;
    transition: background var(--dur-fast) var(--ease-out), border-color var(--dur-fast) var(--ease-out);
  }
  .note:hover { background: var(--raised); }
  .note.sel { border-color: var(--d-teal); }
  /* draft notes (LLM, not yet curated) recede slightly */
  .note--draft { border-style: dashed; opacity: 0.92; }

  .note-top { display: flex; align-items: center; gap: 6px; }
  .note-prov { font-size: 8px; letter-spacing: 0.1em; color: var(--dim); }
  .note-taint {
    margin-left: auto; font-size: 8px; letter-spacing: 0.06em;
    color: var(--text); border: 1px solid var(--line2);
    border-radius: var(--r-xs); padding: 0 3px;
  }
  .note-title { font-size: 11px; color: var(--text); font-weight: 500; }
  .note-body {
    font-size: 9px;
    color: var(--read);
    line-height: 1.4;
    overflow: hidden;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    line-clamp: 3;
    -webkit-box-orient: vertical;
  }

  .brain-foot {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 4px 12px;
    border-top: 1px solid var(--line);
    flex-shrink: 0;
  }
  .legend { display: flex; align-items: center; gap: 6px; color: var(--dim); font-size: 9px; }
  .lk { width: 16px; height: 0; border-top: 1.4px solid var(--d-teal-soft); }
  .lk.dash { border-top-style: dashed; border-color: var(--line2); }
  .brain-hint { margin-left: auto; color: var(--ghost); font-size: 9px; font-style: italic; }
</style>

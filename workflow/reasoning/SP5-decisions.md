SP5 WINDOWS - DECISION DOC (s-73316)

What I will implement (S1 truncation; additive CSS; CLEAN windows; controller-applied):
  MonitorWindow.svelte .snap-value (L962): holds long model id -> add min-width:0 + overflow-wrap:anywhere.
  MonitorWindow.svelte .tl-body (L1142): already declares overflow/ellipsis/nowrap but LACKS min-width:0 so the ellipsis is
    NON-functional and timeline log lines overflow -> add min-width:0.
  IdeaFramingWindow.svelte .value (L270): flex child holding pasted raw_prompt -> add min-width:0 + overflow-wrap:anywhere.
    (Leave the raw hex #ddd/#999 in this file - deferred S5 sweep; this file predates the token migration.)
  PlanWindow.svelte .meta-value (L956): monospace ids/paths -> add min-width:0 + overflow-wrap:anywhere.
  GithubWindow.svelte .pr-table/.checks-table th,td (L541): long PR titles + branch refs -> add overflow-wrap:anywhere
    (table cells; left the --surface-border hex fallback - deferred S5).

DEFERRED (grounded resolutions, NOT applied this run):
  SettingsWindow boot-flag toggles - the plan/Phase-0 assumed "dead no-op -> disable-with-reason". context-compiler PROVED
    this WRONG: the toggles are wired end-to-end (handler -> stub.set_sidecar_settings -> commands.rs -> server.py) and apply
    live (disable) or on next sidecar restart (enable of an unbuilt processor / turboquant_bits). The REAL bug is the handlers
    `.catch(()=>{})` fire-and-forget DROP the sidecar response, so a restart-deferred key LOOKS applied. The correct fix is to
    read that response and show a "(applies on next sidecar restart)" hint. NOT applied here because the response shape lives
    in api/stub.ts which is HOT/dirty (owned by s-60824); building a UI fix on a concurrently-edited contract violates D2/D3.
    -> documented follow-up (revisit after s-60824's api layer lands).
  GraphWindow - context-compiler flagged .title/.footer .search for min-width:0; the SP5 integration-explorer CLEARED it
    (canvas-rendered labels, DOM controls bounded). Conflicted finding -> not acted on; low-priority possible follow-up.

Expected test outcome: svelte-check stays at 0 errors / 18 warnings (additive CSS only).

Unknowns verified empirically: all 5 S1 targets are flex children (sibling of a flex-shrink:0 fixed label) or table cells -
  read L957-966, L1136-1147, L264-271, L950-960, L532-549. Table cells use overflow-wrap:anywhere not min-width:0 (SP4 gotcha).

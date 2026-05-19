<script lang="ts">
  import { onMount } from 'svelte';
  import { webQualityData } from '$lib/state/drawers.svelte';
  import Badge from '$lib/ui/Badge.svelte';
  import Button from '$lib/ui/Button.svelte';
  import Divider from '$lib/ui/Divider.svelte';
  import Input from '$lib/ui/Input.svelte';
  import Toggle from '$lib/ui/Toggle.svelte';
  import UnwiredNote from '$lib/ui/UnwiredNote.svelte';
  import {
    apply_scaffold_template,
    logStub,
    start_dev_server_for_project,
    stop_dev_server_for_project,
  } from '$lib/api/stub';

  onMount(() => {
    webQualityData.bootstrap();
    return webQualityData.teardown;
  });

  const NOW = Date.now();
  const slots = $derived(webQualityData.buildSlots);
  const servers = $derived(webQualityData.devServers);
  const templates = $derived(webQualityData.scaffoldTemplates);
  const arag = $derived(webQualityData.webAragState);
  const srcTiers = $derived(webQualityData.sourceTiers);
  const webRules = $derived(webQualityData.webRulesCounts);

  const totalRules = $derived(webRules.t1 + webRules.t2 + webRules.t3);

  function fmtAgo(ts: number): string {
    const diff = NOW - ts;
    const mins = Math.floor(diff / 60_000);
    if (mins < 60) return `${mins}m ago`;
    const hrs = Math.floor(mins / 60);
    if (hrs < 24) return `${hrs}h ago`;
    return `${Math.floor(hrs / 24)}d ago`;
  }

  function fmtTs(ts: number): string {
    return new Date(ts).toISOString().slice(0, 16).replace('T', ' ');
  }

  function buildTone(status: string): string {
    if (status === 'live') return 'teal';
    if (status === 'building') return 'warn';
    if (status === 'error') return 'fail';
    return 'ghost';
  }

  function serverTone(status: string): string {
    if (status === 'running') return 'teal';
    if (status === 'stopped') return 'ghost';
    return 'fail';
  }

  function serverLabel(status: string): string {
    return status.toUpperCase();
  }

  function buildLabel(status: string): string {
    return status.toUpperCase();
  }

  function openDevServer(url: string) {
    logStub('open_dev_server', { url });
  }

  function promoteSlot(id: string) {
    logStub('web_quality.promote_build', { id });
  }

  function demoteSlot(id: string) {
    logStub('web_quality.demote_build', { id });
  }

  // Phase A (s-69820): registry is keyed by canonicalised project_dir
  // string; the backend's get_dev_servers payload exposes that key as
  // `project_id`. The legacy DevServer mock uses pid as the key, so the
  // wiring below treats `pid` as the registry key when `project_id` is
  // not present (mock-data path).
  async function stopServer(srv: { pid: number; project_id?: string }) {
    const projectId = srv.project_id ?? String(srv.pid);
    try {
      await stop_dev_server_for_project(projectId);
      // Use refresh (one-shot) not bootstrap (refcount-bumping) — see
      // audit WARN s-69820: bootstrap() leaks the polling-timer refcount.
      await webQualityData.refresh();
    } catch (err) {
      console.error('[WebQualityDrawer] stop_dev_server_for_project failed', err);
    }
  }

  async function startServer(srv: {
    pid: number;
    project_id?: string;
    port: number;
    dir?: string;
    command?: string;
  }) {
    const projectDir = srv.project_id ?? srv.dir ?? '';
    if (!projectDir) {
      console.warn('[WebQualityDrawer] startServer missing project_dir');
      return;
    }
    const command = srv.command ?? 'npm run dev';
    try {
      await start_dev_server_for_project(projectDir, command, srv.port);
      await webQualityData.refresh();
    } catch (err) {
      console.error('[WebQualityDrawer] start_dev_server_for_project failed', err);
    }
  }

  /// Phase A (s-69820): wire the SCAFFOLD TEMPLATES "Use" button to the
  /// real `apply_scaffold_template` Tauri command. Picks a target dir
  /// via the Tauri dialog plugin; falls back to a browser prompt in
  /// vite-dev mode so the wiring is exercisable without a Tauri build.
  async function pickTargetDir(): Promise<string | null> {
    const isTauri =
      typeof window !== 'undefined' &&
      (Boolean((window as unknown as { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__) ||
        Boolean((window as unknown as { __TAURI__?: unknown }).__TAURI__));
    if (isTauri) {
      try {
        const dialog = await import('@tauri-apps/plugin-dialog');
        const result = await dialog.open({
          directory: true,
          multiple: false,
          title: 'Choose target folder for scaffold (must be empty)',
        });
        return typeof result === 'string' ? result : null;
      } catch (err) {
        console.warn('[WebQualityDrawer] tauri dialog unavailable', err);
        return null;
      }
    }
    const v = window.prompt('Target folder for scaffold (Tauri dialog unavailable in vite-dev)');
    return v?.trim() ? v.trim() : null;
  }

  async function useTemplate(id: string) {
    logStub('web_quality.use_template', { id });
    const targetDir = await pickTargetDir();
    if (!targetDir) return;
    try {
      const r = await apply_scaffold_template({
        template: id,
        target_dir: targetDir,
      });
      console.info('[WebQualityDrawer] apply_scaffold_template ok', r);
      await webQualityData.refresh();
    } catch (err) {
      console.error('[WebQualityDrawer] apply_scaffold_template failed', err);
    }
  }

  function tierBarPct(count: number): string {
    return `${Math.round((count / totalRules) * 100)}%`;
  }

  function srcBarPct(pct: number): string {
    return `${Math.round(pct)}%`;
  }

  // Slot: first is draft, second is live by convention
  const draftSlot = $derived(slots.find(s => s.status !== 'live') ?? slots[0]);
  const liveSlot  = $derived(slots.find(s => s.status === 'live') ?? slots[slots.length - 1]);

  // ------------------------------------------------------------------
  // Phase C2 (s-67812) — WEB VALIDATION section
  // ------------------------------------------------------------------
  let validationProjectDir = $state('');
  let validationUrl = $state('http://localhost:5173');
  let eslintEnabled = $state(true);
  let stylelintEnabled = $state(true);
  let axeEnabled = $state(true);

  const lintFindings = $derived(webQualityData.lintFindings);
  const lighthouseReport = $derived(webQualityData.lighthouseReport);
  const chromiumAvailable = $derived(webQualityData.chromiumAvailable);
  const lintRunning = $derived(webQualityData.lintRunning);
  const lighthouseRunning = $derived(webQualityData.lighthouseRunning);
  const lintHasRun = $derived(webQualityData.lintHasRun);
  const lighthouseHasRun = $derived(webQualityData.lighthouseHasRun);
  const chromiumProbeDone = $derived(webQualityData.chromiumProbeDone);
  const lintLastError = $derived(webQualityData.lintLastError);
  const lighthouseLastError = $derived(webQualityData.lighthouseLastError);

  const canRunLint = $derived(
    !lintRunning &&
    validationProjectDir.trim().length > 0 &&
    (eslintEnabled || stylelintEnabled || axeEnabled)
  );
  const canRunLighthouse = $derived(
    !lighthouseRunning &&
    validationProjectDir.trim().length > 0 &&
    validationUrl.trim().length > 0
  );

  // Severity counts for the findings header. Capped iteration is fine —
  // dashboardRules.md Context Rule 8 flags virtualization for >500 items;
  // header counts touch each finding once which is O(n).
  const errorCount = $derived(lintFindings.filter(f => f.severity === 'error').length);
  const warningCount = $derived(lintFindings.filter(f => f.severity === 'warning').length);
  const infoCount = $derived(lintFindings.filter(f => f.severity === 'info').length);

  // Severity -> Badge tone mapping. axe/eslint/stylelint all map their
  // native scales onto the same 3 buckets in lint.rs.
  function severityTone(sev: 'error' | 'warning' | 'info'): string {
    if (sev === 'error') return 'fail';
    if (sev === 'warning') return 'warn';
    return 'ghost';
  }

  function fmtPct(score: number): string {
    return `${Math.round(score * 100)}%`;
  }

  function fmtMs(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(2)}s`;
  }

  function fmtCls(cls: number): string {
    return cls.toFixed(3);
  }

  function fileLabel(
    file: string,
    line?: number | null,
    column?: number | null,
  ): string {
    // Wire emits null (serde Option::None) — `==` catches both null and undefined.
    if (line == null) return file;
    if (column == null) return `${file}:${line}`;
    return `${file}:${line}:${column}`;
  }

  async function runLintNow() {
    await webQualityData.runWebLint(
      validationProjectDir.trim(),
      eslintEnabled,
      stylelintEnabled,
      axeEnabled,
    );
  }

  async function runLighthouseNow() {
    await webQualityData.runLighthouseAudit(
      validationProjectDir.trim(),
      validationUrl.trim(),
    );
  }

  function probeChromiumNow() {
    void webQualityData.probeChromium();
  }

  function resetResults() {
    webQualityData.resetWebQualityResults();
  }
</script>

<div class="drawer">
  <!-- BUILD SLOTS -->
  <section class="section">
    <h3 class="section-header">BUILD SLOTS</h3>
    {#if webQualityData.unwired.buildSlots}
      <UnwiredNote
        phase="WEB.3"
        feature="draft / live build slot pair with promote / demote"
        note="get_build_slots is partial; producer (BuildSlotSystem) not actuated when scaffold runs are not in this session"
      />
    {/if}
    <div class="slot-col">
      {#if draftSlot}
        <div class="slot-card slot-card--draft">
          <div class="slot-header">
            <span class="slot-id">{draftSlot.id}</span>
            <Badge tone={buildTone(draftSlot.status)} size="xs">{buildLabel(draftSlot.status)}</Badge>
          </div>
          <span class="slot-label">{draftSlot.label}</span>
          {#if draftSlot.started_ts}
            <span class="slot-ts">{fmtAgo(draftSlot.started_ts)}</span>
          {/if}
          <div class="slot-action">
            <Button variant="ghost" size="sm" onclick={() => promoteSlot(draftSlot.id)}>Promote</Button>
          </div>
        </div>
      {/if}
      {#if liveSlot}
        <div class="slot-card slot-card--live">
          <div class="slot-header">
            <span class="slot-id">{liveSlot.id}</span>
            <Badge tone={buildTone(liveSlot.status)} size="xs">{buildLabel(liveSlot.status)}</Badge>
          </div>
          <span class="slot-label">{liveSlot.label}</span>
          {#if liveSlot.duration_ms}
            <span class="slot-ts">built in {liveSlot.duration_ms}ms · {liveSlot.started_ts ? fmtAgo(liveSlot.started_ts) : ''}</span>
          {/if}
          <div class="slot-action">
            <Button variant="ghost" size="sm" onclick={() => demoteSlot(liveSlot.id)}>Demote</Button>
          </div>
        </div>
      {/if}
    </div>
  </section>

  <Divider />

  <!-- DEV SERVERS -->
  <section class="section">
    <h3 class="section-header">DEV SERVERS</h3>
    {#if webQualityData.unwired.devServers}
      <UnwiredNote
        phase="WEB.3"
        feature="running dev-server PIDs + ports + start/stop controls"
        note="get_dev_servers returns []; DevServerLifecycle not emitted to SSB"
      />
    {/if}
    <div class="server-list">
      {#each servers as srv}
        <div class="server-row">
          <div class="server-meta">
            <span class="server-pid">PID {srv.pid}</span>
            <span class="server-port">:{srv.port}</span>
            <Badge tone={serverTone(srv.status)} size="xs">{serverLabel(srv.status)}</Badge>
          </div>
          <div class="server-actions">
            {#if srv.status === 'running'}
              <!-- svelte-ignore a11y_interactive_supports_focus -->
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <span
                class="server-url"
                role="button"
                onclick={() => openDevServer(`http://localhost:${srv.port}`)}
              >http://localhost:{srv.port}</span>
              <Button variant="ghost" size="sm" onclick={() => stopServer(srv)}>Stop</Button>
            {:else}
              <span class="server-url-inactive">http://localhost:{srv.port}</span>
              <Button variant="ghost" size="sm" onclick={() => startServer(srv)}>Start</Button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </section>

  <Divider />

  <!-- SCAFFOLD TEMPLATES -->
  <section class="section">
    <h3 class="section-header">SCAFFOLD TEMPLATES</h3>
    {#if webQualityData.unwired.scaffoldTemplates}
      <UnwiredNote
        phase="WEB.1"
        feature="scaffold template chooser (vite-svelte / vite-react / astro / vanilla)"
        note="No list_scaffold_templates Tauri command exists; the manifest.toml on disk is not yet read by the dashboard"
      />
    {:else}
      <div class="template-row">
        {#each templates as tpl}
          <div class="template-card">
            <span class="tpl-label">{tpl.label}</span>
            <span class="tpl-meta">{tpl.files_count} files</span>
            <Button variant="ghost" size="sm" onclick={() => useTemplate(tpl.id)}>Use</Button>
          </div>
        {/each}
      </div>
    {/if}
  </section>

  <Divider />

  <!-- WEB RULES -->
  <section class="section">
    <h3 class="section-header">WEB RULES</h3>
    {#if webQualityData.unwired.webRulesCounts}
      <UnwiredNote
        phase="3F"
        feature="T1 / T2 / T3 web-domain rule counts"
        note="get_active_rules returns active rules for the current FSM state but does not split by domain tag yet"
      />
    {:else}
      <div class="rules-stat">
        <span class="rules-total">{totalRules} total</span>
        <span class="rules-breakdown">
          T1 {webRules.t1} · T2 {webRules.t2} · T3 {webRules.t3}
        </span>
      </div>
      <div class="rules-bar">
        <div class="rules-seg seg-t1" style="width:{tierBarPct(webRules.t1)}"></div>
        <div class="rules-seg seg-t2" style="width:{tierBarPct(webRules.t2)}"></div>
        <div class="rules-seg seg-t3" style="width:{tierBarPct(webRules.t3)}"></div>
      </div>
    {/if}
  </section>

  <Divider />

  <!-- WEB A-RAG -->
  <section class="section">
    <h3 class="section-header">WEB A-RAG</h3>
    {#if webQualityData.unwired.webAragState}
      <UnwiredNote
        phase="4D.14"
        feature="WebContentIndex page count + chunk count + crawler quality score"
        note="get_web_arag_state returns zeros; Web A-RAG indexing not run yet from this dashboard"
      />
    {:else}
      <div class="arag-row">
        <span class="arag-label">WebContentIndex</span>
        <span class="arag-value">{arag.pages_indexed.toLocaleString()} pages</span>
      </div>
      <div class="arag-row">
        <span class="arag-label">chunks</span>
        <span class="arag-value">{arag.index_size.toLocaleString()}</span>
      </div>
      <div class="arag-row">
        <span class="arag-label">last crawled</span>
        <span class="arag-ts">{fmtAgo(arag.last_crawled_ts)}</span>
      </div>
      <div class="arag-status-row">
        <Badge tone="teal" size="xs">ACTIVE</Badge>
        <span class="arag-score">quality {Math.round(arag.quality_score * 100)}%</span>
      </div>
    {/if}
  </section>

  <Divider />

  <!-- WEB VALIDATION (Phase C2 — s-67812) -->
  <section class="section">
    <h3 class="section-header">WEB VALIDATION</h3>

    <div class="wv-config">
      <span class="wv-label">project_dir</span>
      <Input
        bind:value={validationProjectDir}
        placeholder="/abs/path/to/project"
        size="sm"
        mono
      />
      <span class="wv-label">target url</span>
      <Input
        bind:value={validationUrl}
        placeholder="http://localhost:5173"
        size="sm"
        mono
      />
    </div>

    <div class="wv-toggles">
      <Toggle bind:checked={eslintEnabled} label="eslint" />
      <Toggle bind:checked={stylelintEnabled} label="stylelint" />
      <Toggle bind:checked={axeEnabled} label="axe-core" />
    </div>

    <div class="wv-actions">
      <Button
        variant="ghost"
        size="sm"
        disabled={!canRunLint}
        onclick={runLintNow}
      >{lintRunning ? 'Running...' : 'Run lint'}</Button>
      <Button
        variant="ghost"
        size="sm"
        disabled={!canRunLighthouse}
        onclick={runLighthouseNow}
      >{lighthouseRunning ? 'Running...' : 'Run Lighthouse'}</Button>
      <Button
        variant="ghost"
        size="sm"
        onclick={probeChromiumNow}
      >Probe Chromium</Button>
      <Button
        variant="ghost"
        size="sm"
        disabled={!lintHasRun && !lighthouseHasRun}
        onclick={resetResults}
      >Reset</Button>
    </div>

    <!-- Chromium availability -->
    <div class="wv-chromium-row">
      <span class="wv-meta-label">chromium</span>
      {#if !chromiumProbeDone}
        <Badge tone="ghost" size="xs">UNKNOWN</Badge>
        <span class="wv-hint">probe to check</span>
      {:else if chromiumAvailable}
        <Badge tone="teal" size="xs">AVAILABLE</Badge>
      {:else}
        <Badge tone="warn" size="xs">UNAVAILABLE</Badge>
        <span class="wv-hint">install chromium or google-chrome to enable Lighthouse</span>
      {/if}
    </div>

    <!-- Lint errors row -->
    {#if lintLastError}
      <div class="wv-error-row">
        <Badge tone="fail" size="xs">LINT ERROR</Badge>
        <span class="wv-error-msg">{lintLastError}</span>
      </div>
    {/if}

    <!-- Lighthouse errors row -->
    {#if lighthouseLastError}
      <div class="wv-error-row">
        <Badge tone="fail" size="xs">LIGHTHOUSE ERROR</Badge>
        <span class="wv-error-msg">{lighthouseLastError}</span>
      </div>
    {/if}

    <!-- Lighthouse score card -->
    {#if lighthouseHasRun && lighthouseReport}
      <div class="wv-lh-card">
        <div class="wv-lh-row">
          <span class="wv-lh-label">perf score</span>
          <span class="wv-lh-value--lg">{fmtPct(lighthouseReport.perf_score)}</span>
        </div>
        <div class="wv-lh-grid">
          <div class="wv-lh-cell">
            <span class="wv-lh-cell-label">LCP</span>
            <span class="wv-lh-cell-value">{fmtMs(lighthouseReport.lcp_ms)}</span>
          </div>
          <div class="wv-lh-cell">
            <span class="wv-lh-cell-label">CLS</span>
            <span class="wv-lh-cell-value">{fmtCls(lighthouseReport.cls)}</span>
          </div>
          <div class="wv-lh-cell">
            <span class="wv-lh-cell-label">INP</span>
            <span class="wv-lh-cell-value">{fmtMs(lighthouseReport.inp_ms)}</span>
          </div>
        </div>
      </div>
    {:else if lighthouseHasRun && !lighthouseReport}
      <div class="wv-empty">no Lighthouse report (browser-dev mode or run failed)</div>
    {/if}

    <!-- Lint findings list -->
    {#if lintHasRun}
      <div class="wv-lint-summary">
        <Badge tone="fail" size="xs">{errorCount} errors</Badge>
        <Badge tone="warn" size="xs">{warningCount} warnings</Badge>
        <Badge tone="ghost" size="xs">{infoCount} info</Badge>
      </div>
      {#if lintFindings.length === 0}
        <div class="wv-empty">no findings</div>
      {:else}
        <!-- TODO C-future: virtualize when findings.length > 500
             (dashboardRules Context Rule 8). -->
        <div class="wv-findings">
          {#each lintFindings as f}
            <div class="wv-finding-row">
              <Badge tone={severityTone(f.severity)} size="xs">{f.severity.toUpperCase()}</Badge>
              <span class="wv-finding-rule">{f.rule_id}</span>
              <span class="wv-finding-file">{fileLabel(f.file, f.line, f.column)}</span>
              <span class="wv-finding-msg">{f.message}</span>
              {#if f.fixable}
                <Badge tone="teal" size="xs">FIXABLE</Badge>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    {/if}
  </section>

  <Divider />

  <!-- SOURCE QUALITY -->
  <section class="section">
    <h3 class="section-header">SOURCE QUALITY</h3>
    {#if webQualityData.unwired.sourceTiers}
      <UnwiredNote
        phase="A-45"
        feature="T1..T5 source-tier distribution of last research run"
        note="get_source_quality_tiers returns zeros; classifier not yet wired into research path"
      />
    {:else}
      <div class="src-tiers">
        {#each srcTiers as tier}
          <div class="src-row">
            <span class="src-tier-label">{tier.tier}</span>
            <div class="src-bar-wrap">
              <div class="src-bar-fill" style="width:{srcBarPct(tier.pct)}"></div>
            </div>
            <span class="src-label-text">{tier.label}</span>
            <span class="src-count">{tier.count}</span>
          </div>
        {/each}
      </div>
    {/if}
  </section>
</div>

<style>
  .drawer {
    display: flex;
    flex-direction: column;
    gap: 0;
    padding-bottom: var(--s-6);
  }

  .section {
    padding: var(--s-4) var(--s-4) var(--s-3);
  }

  .section-header {
    font-family: var(--font-mono);
    font-size: 10px;
    font-weight: 500;
    letter-spacing: 0.08em;
    color: var(--text-ghost);
    text-transform: uppercase;
    margin: 0 0 var(--s-3) 0;
  }

  /* BUILD SLOTS */
  .slot-col {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .slot-card {
    border: 1px solid var(--border-default);
    border-radius: var(--r-sm);
    padding: var(--s-3);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .slot-card--draft {
    border-color: var(--warn);
    border-left-width: 2px;
  }

  .slot-card--live {
    border-color: var(--teal-line);
    border-left-width: 2px;
  }

  .slot-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .slot-id {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .slot-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-primary);
  }

  .slot-ts {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .slot-action {
    margin-top: var(--s-1);
  }

  /* DEV SERVERS */
  .server-list {
    display: flex;
    flex-direction: column;
    gap: var(--s-3);
  }

  .server-row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: var(--s-2) var(--s-3);
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
  }

  .server-meta {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .server-pid {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
  }

  .server-port {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .server-actions {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .server-url {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--teal);
    cursor: pointer;
    flex: 1;
    transition: color var(--dur-fast) var(--ease-std);
  }

  .server-url:hover {
    color: var(--teal-bright);
    text-decoration: underline;
  }

  .server-url-inactive {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    flex: 1;
  }

  /* SCAFFOLD TEMPLATES */
  .template-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--s-2);
  }

  .template-card {
    flex: 1;
    min-width: 100px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    padding: var(--s-2) var(--s-3);
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: flex-start;
  }

  .tpl-label {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
    white-space: nowrap;
  }

  .tpl-meta {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* WEB RULES */
  .rules-stat {
    display: flex;
    align-items: baseline;
    gap: var(--s-3);
    margin-bottom: var(--s-2);
  }

  .rules-total {
    font-family: var(--font-mono);
    font-size: 13px;
    color: var(--text-primary);
    font-weight: 500;
  }

  .rules-breakdown {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .rules-bar {
    display: flex;
    height: 4px;
    border-radius: var(--r-pill);
    overflow: hidden;
    background: var(--bg-input);
  }

  .rules-seg {
    height: 100%;
  }

  .seg-t1 { background: var(--teal); }
  .seg-t2 { background: var(--steel); }
  .seg-t3 { background: var(--warn); }

  /* WEB A-RAG */
  .arag-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-bottom: 4px;
  }

  .arag-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    width: 11ch;
  }

  .arag-value {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
  }

  .arag-ts {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .arag-status-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-top: var(--s-1);
  }

  .arag-score {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  /* SOURCE QUALITY */
  .src-tiers {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .src-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
  }

  .src-tier-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    width: 2ch;
    flex-shrink: 0;
  }

  .src-bar-wrap {
    flex: 1;
    height: 4px;
    background: var(--bg-input);
    border-radius: var(--r-pill);
    overflow: hidden;
  }

  .src-bar-fill {
    height: 100%;
    background: var(--teal);
    border-radius: var(--r-pill);
    transition: width var(--dur-base) var(--ease-out);
  }

  .src-label-text {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    width: 12ch;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .src-count {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    text-align: right;
    width: 4ch;
  }

  /* WEB VALIDATION (Phase C2 — s-67812) */
  .wv-config {
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: var(--s-2) var(--s-3);
    align-items: center;
    margin-bottom: var(--s-3);
  }

  .wv-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .wv-toggles {
    display: flex;
    gap: var(--s-3);
    margin-bottom: var(--s-3);
    flex-wrap: wrap;
  }

  .wv-actions {
    display: flex;
    gap: var(--s-2);
    margin-bottom: var(--s-3);
    flex-wrap: wrap;
  }

  .wv-chromium-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-bottom: var(--s-2);
  }

  .wv-meta-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    text-transform: uppercase;
  }

  .wv-hint {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .wv-error-row {
    display: flex;
    align-items: center;
    gap: var(--s-2);
    margin-bottom: var(--s-2);
    padding: var(--s-2);
    background: var(--bg-surface);
    border: 1px solid var(--fail);
    border-left-width: 2px;
    border-radius: var(--r-sm);
  }

  .wv-error-msg {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    word-break: break-word;
  }

  /* Lighthouse score card */
  .wv-lh-card {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
    padding: var(--s-3);
    margin-bottom: var(--s-3);
  }

  .wv-lh-row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    margin-bottom: var(--s-2);
  }

  .wv-lh-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }

  .wv-lh-value--lg {
    font-family: var(--font-mono);
    font-size: 18px;
    color: var(--text-primary);
    font-weight: 500;
  }

  .wv-lh-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: var(--s-2);
  }

  .wv-lh-cell {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .wv-lh-cell-label {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }

  .wv-lh-cell-value {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--text-primary);
  }

  /* Lint findings */
  .wv-lint-summary {
    display: flex;
    gap: var(--s-2);
    margin-bottom: var(--s-2);
    flex-wrap: wrap;
  }

  .wv-empty {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    padding: var(--s-2);
  }

  .wv-findings {
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 320px;
    overflow-y: auto;
  }

  .wv-finding-row {
    display: grid;
    grid-template-columns: max-content max-content 1fr 2fr max-content;
    gap: var(--s-2);
    align-items: baseline;
    padding: 4px var(--s-2);
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-sm);
  }

  .wv-finding-rule {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
    white-space: nowrap;
  }

  .wv-finding-file {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .wv-finding-msg {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>

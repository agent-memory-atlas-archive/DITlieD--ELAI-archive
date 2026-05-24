<script lang="ts">
  // E-12 Tier-B — IdeaFramerSecurityPanel
  //
  // Renders the selected FrameworkProfile + 15-item production
  // checklist results for the active session. Mounted by WindowHost
  // when WindowType=='idea_framer_security'. Companion to the B-40
  // IdeaFramingWindow — this panel is the production-grade gate that
  // runs AFTER the idea-framer + project-class classifier converge.
  //
  // Wire-dark posture: the runpod_pending badge is shown by default
  // because the 15 external CLIs (gitleaks / trufflehog / semgrep /
  // npm-audit / pip-audit / cargo-audit / lighthouse / biome / ruff /
  // clippy / sqlite3 / cargo-machete / ast-grep / lsof) are NOT
  // installed on the local dev host. Every runner returns
  // ToolchainMissing on PATH miss and the panel renders the install
  // hint inline so the user can resolve the gap without leaving the
  // panel. Live runner invocation is RUNPOD-PENDING per
  // workflow/reasoning/E-12-tier-b-decisions.md.

  import * as stub from '$lib/api/stub';
  import type {
    FrameworkProfileDto,
    ProductionChecklistRunDto,
  } from '$lib/api/stub';

  const PROJECT_CLASSES = [
    'consumer_mobile_app',
    'web_app',
    'saas_dashboard',
    'desktop_app',
    'game_mod',
    'research_tool',
    'library_or_sdk',
    'cli_tool',
    'other',
  ];

  let selectedClass = $state<string>('saas_dashboard');
  let profile = $state<FrameworkProfileDto | null>(null);
  let checklist = $state<ProductionChecklistRunDto | null>(null);
  let loading = $state<boolean>(false);
  let running = $state<boolean>(false);
  let targetPath = $state<string>('');
  let lastError = $state<string | null>(null);

  async function refreshProfile(): Promise<void> {
    loading = true;
    lastError = null;
    try {
      profile = await stub.get_framework_profile(selectedClass);
    } catch (e) {
      lastError = String(e);
    } finally {
      loading = false;
    }
  }

  async function runChecklist(): Promise<void> {
    if (!targetPath || targetPath.trim() === '') {
      lastError = 'target_path is required';
      return;
    }
    running = true;
    lastError = null;
    try {
      checklist = await stub.run_production_checklist(
        selectedClass,
        targetPath.trim(),
      );
    } catch (e) {
      lastError = String(e);
    } finally {
      running = false;
    }
  }

  $effect(() => {
    void refreshProfile();
  });
</script>

<section class="ifs-panel">
  <header class="ifs-header">
    <h2>Production gate</h2>
    <span class="ifs-badge">E-12 Tier-B</span>
    {#if profile?.runpod_pending}
      <span class="ifs-runpod">runpod-pending</span>
    {/if}
  </header>

  <div class="ifs-controls">
    <label>
      <span>Project class</span>
      <select bind:value={selectedClass} onchange={refreshProfile}>
        {#each PROJECT_CLASSES as cls}
          <option value={cls}>{cls}</option>
        {/each}
      </select>
    </label>
    <label>
      <span>Target path</span>
      <input type="text" bind:value={targetPath} placeholder="absolute path" />
    </label>
    <button type="button" onclick={runChecklist} disabled={running}>
      {running ? 'Running…' : 'Run production checklist'}
    </button>
  </div>

  {#if lastError}
    <div class="ifs-error">Error: {lastError}</div>
  {/if}

  {#if loading}
    <div class="ifs-loading">Resolving profile…</div>
  {:else if profile}
    <section class="ifs-profile">
      <h3>Framework profile</h3>
      <dl>
        <dt>Name</dt>
        <dd>{profile.profile_name}</dd>
        <dt>Version pin</dt>
        <dd>{profile.version_pin || '(empty — generic fallback)'}</dd>
        <dt>Language</dt>
        <dd>{profile.language}</dd>
        <dt>Package manager</dt>
        <dd>{profile.package_manager}</dd>
        <dt>Lint config</dt>
        <dd>{profile.lint_config || '(none)'}</dd>
        <dt>Test runner</dt>
        <dd>{profile.test_runner}</dd>
      </dl>
      <div class="ifs-checks">
        <h4>Security checks ({profile.security_checks.length})</h4>
        <ul>
          {#each profile.security_checks as c}
            <li>{c}</li>
          {/each}
        </ul>
      </div>
    </section>
  {/if}

  {#if checklist}
    <section class="ifs-results">
      <h3>Checklist run</h3>
      <p>profile = {checklist.profile_name}</p>
      <p>target = {checklist.target_path}</p>
      <table>
        <thead>
          <tr>
            <th>Check</th>
            <th>Result</th>
          </tr>
        </thead>
        <tbody>
          {#each checklist.rows as row}
            <tr>
              <td>{row.check}</td>
              <td>
                {#if row.result.kind === 'ok'}
                  <span class="ifs-ok"
                    >ok ({row.result.findings.length} findings)</span
                  >
                {:else if row.result.kind === 'toolchain_missing'}
                  <span class="ifs-missing">
                    missing: {row.result.tool}
                    <em>{row.result.install_hint}</em>
                  </span>
                {:else}
                  <span class="ifs-fail">{row.result.details}</span>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </section>
  {/if}
</section>

<style>
  .ifs-panel {
    padding: 1rem;
    font-family: var(--font-mono, monospace);
  }
  .ifs-header {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
  }
  .ifs-badge {
    background: var(--accent-bg, #1f2937);
    color: var(--accent-fg, #f3f4f6);
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
  }
  .ifs-runpod {
    background: #92400e;
    color: #fef3c7;
    padding: 0.125rem 0.5rem;
    border-radius: 0.25rem;
    font-size: 0.75rem;
  }
  .ifs-controls {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin: 1rem 0;
  }
  .ifs-controls label {
    display: flex;
    gap: 0.5rem;
    align-items: center;
  }
  .ifs-controls span {
    min-width: 9rem;
  }
  .ifs-error {
    color: #ef4444;
    margin: 0.5rem 0;
  }
  .ifs-loading {
    color: #6b7280;
  }
  .ifs-profile dl {
    display: grid;
    grid-template-columns: 10rem 1fr;
    gap: 0.25rem 1rem;
  }
  .ifs-checks {
    margin-top: 0.75rem;
  }
  .ifs-checks ul {
    columns: 2;
    margin: 0;
    padding-left: 1.5rem;
  }
  .ifs-results {
    margin-top: 1rem;
  }
  .ifs-results table {
    width: 100%;
    border-collapse: collapse;
  }
  .ifs-results th,
  .ifs-results td {
    text-align: left;
    padding: 0.25rem 0.5rem;
    border-bottom: 1px solid var(--border, #374151);
  }
  .ifs-ok {
    color: #10b981;
  }
  .ifs-missing {
    color: #f59e0b;
  }
  .ifs-missing em {
    display: block;
    font-size: 0.75rem;
    color: #9ca3af;
  }
  .ifs-fail {
    color: #ef4444;
  }
</style>

<script lang="ts">
  // E-33 SnackBarPermission — stacked permission-prompt row that surfaces
  // FSM gate "permission required" events above the input bar across all
  // windows. Kiro CLI pattern: snack-bar prompts are quicker than modal
  // dialogs because they don't block surrounding UI but still demand a
  // decision (Approve / Reject / Always-allow).
  //
  // Consumer wiring: parent components pass `prompts` (array). When the
  // production FSM gate emits a permission-request SSB row, the host
  // store fills this array. Empty array => component is invisible.

  interface PermissionPrompt {
    id: string;
    title: string;
    detail?: string;
    tool?: string;
  }

  interface Props {
    prompts?: PermissionPrompt[];
    onApprove?: (id: string) => void;
    onReject?: (id: string) => void;
    onAlwaysAllow?: (id: string) => void;
  }

  let {
    prompts = [],
    onApprove = () => {},
    onReject = () => {},
    onAlwaysAllow = () => {},
  }: Props = $props();
</script>

{#if prompts.length > 0}
  <div class="snackbar-stack" role="region" aria-label="Pending permissions">
    {#each prompts as p (p.id)}
      <div class="snackbar" role="alert">
        <div class="snackbar-body">
          <div class="snackbar-title">{p.title}</div>
          {#if p.detail}
            <div class="snackbar-detail">{p.detail}</div>
          {/if}
          {#if p.tool}
            <div class="snackbar-tool">tool: <code>{p.tool}</code></div>
          {/if}
        </div>
        <div class="snackbar-actions">
          <button type="button" class="btn btn-reject" onclick={() => onReject(p.id)}>reject</button>
          <button type="button" class="btn btn-approve" onclick={() => onApprove(p.id)}>approve</button>
          <button type="button" class="btn btn-always" onclick={() => onAlwaysAllow(p.id)}>always</button>
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .snackbar-stack {
    display: flex;
    flex-direction: column;
    gap: var(--s-1);
    padding: var(--s-1) 0;
  }
  .snackbar {
    display: flex;
    align-items: center;
    gap: var(--s-3);
    padding: var(--s-2) var(--s-3);
    background: var(--bg-surface);
    border: 1px solid var(--teal-line);
    border-radius: var(--r-md);
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-primary);
  }
  .snackbar-body {
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .snackbar-title {
    color: var(--text-primary);
    font-weight: 500;
  }
  .snackbar-detail {
    color: var(--text-secondary);
    font-size: 10px;
  }
  .snackbar-tool {
    color: var(--text-ghost);
    font-size: 10px;
  }
  .snackbar-actions {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex: 0 0 auto;
  }
  .btn {
    background: transparent;
    border: 1px solid currentColor;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 3px 8px;
    border-radius: var(--r-sm);
    cursor: pointer;
    transition: background var(--dur-fast) var(--ease-std);
  }
  .btn:hover {
    background: var(--bg-hover);
  }
  .btn-reject {
    color: var(--fail);
  }
  .btn-approve {
    color: var(--signal-green);
  }
  .btn-always {
    color: var(--signal-blue);
  }
</style>

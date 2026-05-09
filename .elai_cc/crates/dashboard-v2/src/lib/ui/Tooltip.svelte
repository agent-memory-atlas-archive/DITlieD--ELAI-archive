<script lang="ts" module>
  // Tooltip implementation lives in `./tooltip.ts` (a plain TS module so tsc
  // can resolve the named exports through `./index.ts`). This .svelte file
  // exists only as a re-export point for callers importing from `Tooltip.svelte`.
  export { tooltip, type TooltipOptions } from './tooltip';
</script>

<!--
  Usage:
    <script>
      import { tooltip } from '$lib/ui/tooltip';
    </script>
    <button use:tooltip={{ label: 'Save', kbd: ['Ctrl', 'S'] }}>Save</button>
-->

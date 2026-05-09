<script lang="ts">
  import { Empty } from '../ui/index';

  interface Props {
    type: string;
  }

  let { type }: Props = $props();
</script>

<div class="fallback">
  <Empty
    title="{type} window"
    subtitle="implementation pending"
  />
</div>

<style>
  .fallback {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    background: var(--bg-base);
  }
</style>

<script lang="ts">
  interface Props {
    orientation?: 'horizontal' | 'vertical';
    inset?: number;
  }

  let {
    orientation = 'horizontal',
    inset = 0,
  }: Props = $props();
</script>

<div
  class="divider divider--{orientation}"
  style={inset > 0 && orientation === 'horizontal'
    ? `margin-left: ${inset}px; margin-right: ${inset}px;`
    : inset > 0
    ? `margin-top: ${inset}px; margin-bottom: ${inset}px;`
    : ''}
></div>

<style>
  .divider {
    flex-shrink: 0;
  }

  .divider--horizontal {
    width: 100%;
    height: 1px;
    background: var(--border-subtle);
    margin: var(--s-2) 0;
  }

  .divider--vertical {
    width: 1px;
    height: 100%;
    background: var(--border-subtle);
    margin: 0 var(--s-2);
    align-self: stretch;
  }
</style>

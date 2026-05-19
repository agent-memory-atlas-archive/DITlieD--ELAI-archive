<script lang="ts">
  import type { Component } from 'svelte';
  import type { IconProps } from '@lucide/svelte';

  interface Props {
    title: string;
    subtitle?: string;
    icon?: Component<IconProps>;
  }

  let {
    title,
    subtitle = '',
    icon: Icon,
  }: Props = $props();
</script>

<div class="empty">
  {#if Icon}
    <span class="empty__icon">
      <Icon size={24} strokeWidth={1} />
    </span>
  {/if}
  <span class="empty__title">{title}</span>
  {#if subtitle}
    <span class="empty__subtitle">{subtitle}</span>
  {/if}
</div>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: var(--s-8);
    text-align: center;
  }

  .empty__icon {
    color: var(--text-ghost);
    margin-bottom: 2px;
  }

  .empty__title {
    font-family: var(--font-sans);
    font-size: 12px;
    color: var(--text-secondary);
  }

  .empty__subtitle {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--text-ghost);
  }
</style>

// Tooltip Svelte action.
// Usage: <button use:tooltip={{ label: 'Save', kbd: ['Ctrl', 'S'], side: 'top' }}>

export interface TooltipOptions {
  label: string;
  kbd?: string[];
  side?: 'top' | 'bottom' | 'left' | 'right';
}

export function tooltip(node: HTMLElement, opts: TooltipOptions) {
  let tip: HTMLElement | null = null;
  let timer: ReturnType<typeof setTimeout> | null = null;

  function show() {
    if (tip) return;
    tip = document.createElement('div');
    tip.className = 'elai-tooltip';
    tip.setAttribute('role', 'tooltip');
    tip.style.cssText = `
      position: fixed;
      z-index: 9999;
      background: var(--bg-surface);
      border: 1px solid var(--border-default);
      border-radius: var(--r-sm);
      padding: 3px 8px;
      display: flex;
      align-items: center;
      gap: 6px;
      font-family: var(--font-mono);
      font-size: 11px;
      color: var(--text-primary);
      white-space: nowrap;
      pointer-events: none;
      opacity: 0;
      transition: opacity 120ms ease-out;
    `;

    const labelSpan = document.createElement('span');
    labelSpan.textContent = opts.label;
    tip.appendChild(labelSpan);

    if (opts.kbd && opts.kbd.length > 0) {
      const kbdWrap = document.createElement('span');
      kbdWrap.style.cssText = 'display:flex; align-items:center; gap:2px;';
      opts.kbd.forEach((k, i) => {
        if (i > 0) {
          const sep = document.createElement('span');
          sep.textContent = '+';
          sep.style.cssText = 'color: var(--text-ghost); font-size: 10px;';
          kbdWrap.appendChild(sep);
        }
        const key = document.createElement('span');
        key.textContent = k;
        key.style.cssText = `
          background: var(--bg-surface);
          border: 1px solid var(--border-subtle);
          border-radius: var(--r-xs);
          padding: 1px 5px;
          font-family: var(--font-mono);
          font-size: 10px;
          color: var(--steel);
        `;
        kbdWrap.appendChild(key);
      });
      tip.appendChild(kbdWrap);
    }

    document.body.appendChild(tip);
    position();
    requestAnimationFrame(() => {
      if (tip) tip.style.opacity = '1';
    });
  }

  function hide() {
    if (!tip) return;
    const t = tip;
    t.style.opacity = '0';
    t.style.transition = 'opacity 100ms ease-in';
    setTimeout(() => t.parentNode?.removeChild(t), 100);
    tip = null;
  }

  function position() {
    if (!tip) return;
    const rect = node.getBoundingClientRect();
    const tipRect = tip.getBoundingClientRect();
    const side = opts.side ?? 'top';
    const gap = 6;
    let top = 0;
    let left = 0;

    if (side === 'top') {
      top = rect.top - tipRect.height - gap;
      left = rect.left + rect.width / 2 - tipRect.width / 2;
    } else if (side === 'bottom') {
      top = rect.bottom + gap;
      left = rect.left + rect.width / 2 - tipRect.width / 2;
    } else if (side === 'left') {
      top = rect.top + rect.height / 2 - tipRect.height / 2;
      left = rect.left - tipRect.width - gap;
    } else {
      top = rect.top + rect.height / 2 - tipRect.height / 2;
      left = rect.right + gap;
    }

    tip.style.top = `${Math.max(4, top)}px`;
    tip.style.left = `${Math.max(4, left)}px`;
  }

  function onEnter() {
    timer = setTimeout(show, 400);
  }

  function onLeave() {
    if (timer !== null) { clearTimeout(timer); timer = null; }
    hide();
  }

  node.addEventListener('mouseenter', onEnter);
  node.addEventListener('mouseleave', onLeave);
  node.addEventListener('focusout', onLeave);

  return {
    update(newOpts: TooltipOptions) {
      opts = newOpts;
    },
    destroy() {
      if (timer !== null) clearTimeout(timer);
      hide();
      node.removeEventListener('mouseenter', onEnter);
      node.removeEventListener('mouseleave', onLeave);
      node.removeEventListener('focusout', onLeave);
    },
  };
}

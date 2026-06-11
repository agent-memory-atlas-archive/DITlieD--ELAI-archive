// H-15 — renderer_registry.ts
//
// Maps view_type string keys to Svelte component constructors.
// spawn_view uses view_type as the discriminator to select which renderer
// component to mount for a spawned canvas view.
//
// Design (CANVAS-WORKBENCH-DESIGN.md §15):
//   - The 5 H-12 mode components register first (generalization, not rewrite).
//   - Any new view_type registers via registerRenderer() before the first
//     spawn_view call that references it.
//   - resolveRenderer() returns null for unknown view_types (never panics);
//     the caller renders a fallback error card.
//
// GD-1: reuses the H-12 mode picker abstraction (generalization not rewrite).
// HARD RULE: no magic numbers; named constants for any non-trivial limits.

import type { Component } from 'svelte';

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/** A Svelte component constructor that can be used as a canvas view renderer. */
export type CanvasViewRenderer = Component<{
  /** Stable view instance id from the ViewSpawned SSB row. */
  viewId: string;
  /** Seed op ids to bootstrap the view's initial graph state. */
  seedOpIds: string[];
  /** Task id for SSB tracing. */
  taskId?: string;
}>;

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/** Maximum number of registered renderers. Named constant (no magic numbers). */
const MAX_REGISTERED_RENDERERS = 32;

const registry = new Map<string, CanvasViewRenderer>();

/**
 * Register a Svelte component as the renderer for a given view_type key.
 * Idempotent: re-registering the same view_type replaces the previous renderer.
 * Throws if the registry is at capacity (MAX_REGISTERED_RENDERERS).
 */
export function registerRenderer(viewType: string, component: CanvasViewRenderer): void {
  if (registry.size >= MAX_REGISTERED_RENDERERS && !registry.has(viewType)) {
    throw new Error(
      `renderer_registry: capacity exceeded (max ${MAX_REGISTERED_RENDERERS}); ` +
        `cannot register '${viewType}'`,
    );
  }
  registry.set(viewType, component);
}

/**
 * Look up the renderer for a given view_type.
 * Returns null for unknown view_types (never throws); callers render a fallback.
 */
export function resolveRenderer(viewType: string): CanvasViewRenderer | null {
  return registry.get(viewType) ?? null;
}

/** Return all registered view_type keys (for introspection / testing). */
export function registeredViewTypes(): string[] {
  return [...registry.keys()];
}

/**
 * Exported singleton for imports that prefer a single object surface.
 * Matches the wiring_contract_check.py expected symbol name.
 */
export const rendererRegistry = {
  registerRenderer,
  resolveRenderer,
  registeredViewTypes,
};

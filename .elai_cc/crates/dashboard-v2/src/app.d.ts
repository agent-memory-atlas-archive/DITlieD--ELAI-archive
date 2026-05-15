/// <reference types="svelte" />
/// <reference types="vite/client" />

declare module '*.svg' {
  const content: string;
  export default content;
}

declare module '*.png' {
  const content: string;
  export default content;
}

declare module '*.jpg' {
  const content: string;
  export default content;
}

// troika-three-text — no ambient .d.ts ships with the package. We use
// only the Text class (a THREE.Mesh subclass with SDF text rendering).
// Declare a minimal surface so svelte-check is happy; runtime behaviour
// is unaffected (the module's actual exports are richer, but anything
// not declared here is intentionally out of scope for this codebase).
// B-33 — cytoscape-dagre + cytoscape-fcose ship runtime extensions but no
// .d.ts. Both register themselves via `cytoscape.use()` and are otherwise
// invoked by name through `cy.layout({ name: 'dagre' | 'fcose' })`. A
// minimal `default any` declaration is sufficient — we don't access
// extension internals from TypeScript.
declare module 'cytoscape-dagre' {
  const ext: any;
  export default ext;
}
declare module 'cytoscape-fcose' {
  const ext: any;
  export default ext;
}

declare module 'troika-three-text' {
  import * as THREE from 'three';
  export class Text extends THREE.Mesh {
    text: string;
    fontSize: number;
    color: number | string | THREE.Color;
    fillOpacity: number;
    outlineColor: number | string | THREE.Color;
    outlineWidth: number;
    outlineOpacity: number;
    anchorX: 'left' | 'center' | 'right' | number | string;
    anchorY: 'top' | 'top-baseline' | 'middle' | 'bottom-baseline' | 'bottom' | number | string;
    depthOffset: number;
    /** Build / refresh the SDF font atlas. Idempotent; debounced. */
    sync(callback?: () => void): void;
    /** Release glyph atlas page reference + underlying material. */
    dispose(): void;
  }
}

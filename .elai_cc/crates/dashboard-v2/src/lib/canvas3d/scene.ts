// canvas3d/scene.ts — the scene controller.
//
// Owns the three.js renderer, camera, retro pass, and the rebuild cycle.
// Rendering is two-stage for the pixelated-terminal look: the world is
// drawn into a LOW-RES render target (1/uPixel of the canvas), then
// upscaled to the canvas with NEAREST filtering through a scanline quad —
// chunky phosphor pixels, and strictly CHEAPER than native-res rendering.
// Draw calls stay flat: 1 lines + 1 sheet per slab, 1 instanced cores,
// 1 glow points, 1 trace lines, 1 upscale quad.

import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import type { C3Graph } from './types';
import { layoutGraph, layerTopY, LAYER_DY, SLAB_W, SLAB_D } from './layout';
import type { LayoutResult } from './layout';
import { readTheme, makeGlowTexture } from './theme';
import type { C3Theme } from './theme';
import { buildSlab, buildSlabSheet } from './slabs';
import { buildNodes } from './nodes';
import type { NodeBuffers } from './nodes';
import { buildTraces } from './traces';
import type { TraceBuffers } from './traces';

const UPSCALE_FRAG = /* glsl */ `
  uniform sampler2D uTex;
  uniform vec2 uLowRes;
  varying vec2 vUv;
  void main() {
    vec3 c = texture2D(uTex, vUv).rgb;
    // scanline on the virtual low-res rows
    float row = floor(vUv.y * uLowRes.y);
    float sl = mod(row, 2.0) < 1.0 ? 1.0 : 0.82;
    // soft vignette
    vec2 d = vUv - 0.5;
    float vig = 1.0 - dot(d, d) * 0.55;
    gl_FragColor = vec4(c * sl * vig, 1.0);
  }
`;

const UPSCALE_VERT = /* glsl */ `
  varying vec2 vUv;
  void main() {
    vUv = uv;
    gl_Position = vec4(position.xy, 0.0, 1.0);
  }
`;

export interface SceneCallbacks {
  onHover(nodeId: string | null, x: number, y: number): void;
  onSelect(nodeId: string | null): void;
  onToggleExpand(nodeId: string): void;
}

export class Canvas3DScene {
  private renderer: THREE.WebGLRenderer;
  private scene = new THREE.Scene();
  private camera: THREE.PerspectiveCamera;
  private controls: OrbitControls;
  private target: THREE.WebGLRenderTarget;
  private quadScene = new THREE.Scene();
  private quadCam = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);
  private quadMat: THREE.ShaderMaterial;
  private theme: C3Theme;
  private glowTex: THREE.Texture;
  private raycaster = new THREE.Raycaster();
  private pointer = new THREE.Vector2();
  private pointerDirty = false;
  private pointerCss = { x: 0, y: 0 };

  private slabGroup = new THREE.Group();
  private slabCache = new Map<string, THREE.Object3D[]>();
  private nodeBuffers: NodeBuffers | null = null;
  private traceBuffers: TraceBuffers | null = null;
  private layout: LayoutResult | null = null;

  private width = 1;
  private height = 1;
  private pixelFactor = 3;
  private clock = new THREE.Clock();
  private raf = 0;
  private lastRev = -1;
  private hoverId: string | null = null;
  private selectedId: string | null = null;
  private followActive = false;
  private followPos: THREE.Vector3 | null = null;
  private disposed = false;

  /** rolling FPS, readable by the HUD */
  fps = 0;
  private fpsAccum = 0;
  private fpsFrames = 0;

  constructor(
    private canvas: HTMLCanvasElement,
    private graph: C3Graph,
    private cb: SceneCallbacks,
  ) {
    this.theme = readTheme();
    this.glowTex = makeGlowTexture();

    this.renderer = new THREE.WebGLRenderer({ canvas, antialias: false, alpha: false });
    this.renderer.setPixelRatio(1); // the retro pass owns resolution
    this.scene.background = this.theme.background;
    this.scene.fog = new THREE.FogExp2(this.theme.background, 0.00062);
    this.scene.add(this.slabGroup);

    this.camera = new THREE.PerspectiveCamera(46, 1, 1, 4000);
    this.camera.position.set(SLAB_W * 0.75, 150, SLAB_D * 1.35);

    this.controls = new OrbitControls(this.camera, canvas);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.08;
    this.controls.maxPolarAngle = Math.PI * 0.52;
    this.controls.minDistance = 60;
    this.controls.maxDistance = 1400;
    this.controls.target.set(0, -LAYER_DY, 0);

    this.target = new THREE.WebGLRenderTarget(4, 4, {
      minFilter: THREE.NearestFilter,
      magFilter: THREE.NearestFilter,
    });
    this.quadMat = new THREE.ShaderMaterial({
      uniforms: {
        uTex: { value: this.target.texture },
        uLowRes: { value: new THREE.Vector2(4, 4) },
      },
      vertexShader: UPSCALE_VERT,
      fragmentShader: UPSCALE_FRAG,
      depthTest: false,
      depthWrite: false,
    });
    this.quadScene.add(new THREE.Mesh(new THREE.PlaneGeometry(2, 2), this.quadMat));

    canvas.addEventListener('pointermove', this.onPointerMove);
    canvas.addEventListener('click', this.onClick);
    canvas.addEventListener('dblclick', this.onDblClick);

    this.loop();
  }

  // ------------------------------------------------------------ input

  private onPointerMove = (ev: PointerEvent) => {
    const rect = this.canvas.getBoundingClientRect();
    this.pointer.x = ((ev.clientX - rect.left) / rect.width) * 2 - 1;
    this.pointer.y = -((ev.clientY - rect.top) / rect.height) * 2 + 1;
    this.pointerCss = { x: ev.clientX - rect.left, y: ev.clientY - rect.top };
    this.pointerDirty = true;
  };

  private onClick = () => {
    this.selectedId = this.hoverId;
    this.nodeBuffers?.highlight(this.selectedId, this.theme);
    this.cb.onSelect(this.selectedId);
  };

  private onDblClick = () => {
    if (this.hoverId !== null) this.cb.onToggleExpand(this.hoverId);
  };

  private pick(): void {
    if (!this.pointerDirty || !this.nodeBuffers) return;
    this.pointerDirty = false;
    this.raycaster.setFromCamera(this.pointer, this.camera);
    const hits = this.raycaster.intersectObject(this.nodeBuffers.cores, false);
    let id: string | null = null;
    if (hits.length > 0 && hits[0].instanceId !== undefined) {
      id = this.nodeBuffers.pickIndex[hits[0].instanceId] ?? null;
    }
    if (id !== this.hoverId) {
      this.hoverId = id;
      this.canvas.style.cursor = id !== null ? 'pointer' : 'default';
    }
    this.cb.onHover(this.hoverId, this.pointerCss.x, this.pointerCss.y);
  }

  // ------------------------------------------------------------ rebuild

  private rebuild(): void {
    this.lastRev = this.graph.rev;
    this.layout = layoutGraph(this.graph);

    // slabs: cache per layer (static geometry per depth)
    const wanted = new Set<string>();
    for (const layerId of this.graph.layerOrder) {
      const layer = this.graph.layers.get(layerId);
      if (!layer) continue;
      // a layer with no placed root is hidden this rev
      if (!this.layout.placements.has(layer.rootId)) continue;
      wanted.add(layerId);
      if (!this.slabCache.has(layerId)) {
        const objs = [buildSlab(layer.depth, layerId, this.theme), buildSlabSheet(layer.depth, this.theme)];
        for (const o of objs) this.slabGroup.add(o);
        this.slabCache.set(layerId, objs);
      }
    }
    for (const [layerId, objs] of this.slabCache) {
      const show = wanted.has(layerId);
      for (const o of objs) o.visible = show;
    }

    if (this.nodeBuffers) {
      this.scene.remove(this.nodeBuffers.cores, this.nodeBuffers.glow);
      this.nodeBuffers.dispose();
    }
    this.nodeBuffers = buildNodes(this.graph, this.layout, this.theme, this.glowTex);
    this.scene.add(this.nodeBuffers.cores, this.nodeBuffers.glow);
    this.nodeBuffers.highlight(this.selectedId, this.theme);

    if (this.traceBuffers) {
      this.scene.remove(this.traceBuffers.lines);
      this.traceBuffers.dispose();
    }
    this.traceBuffers = buildTraces(this.graph, this.layout, this.theme);
    this.scene.add(this.traceBuffers.lines);

    // follow target: most recently active placed node (stable pick: last in sort)
    if (this.followActive) {
      for (const id of this.nodeBuffers.pickIndex) {
        const n = this.graph.nodes.get(id);
        if (n && n.state === 'active') {
          const p = this.layout.placements.get(id);
          if (p) this.followPos = new THREE.Vector3(...p.pos);
        }
      }
    }
  }

  // ------------------------------------------------------------ loop

  private loop = () => {
    if (this.disposed) return;
    this.raf = requestAnimationFrame(this.loop);
    const dt = this.clock.getDelta();
    const t = this.clock.elapsedTime;

    if (this.graph.rev !== this.lastRev) this.rebuild();
    this.pick();

    if (this.followActive && this.followPos) {
      this.controls.target.lerp(this.followPos, Math.min(1, dt * 2.2));
    }
    this.controls.update();
    this.nodeBuffers?.setTime(t);
    this.traceBuffers?.setTime(t);

    this.renderer.setRenderTarget(this.target);
    this.renderer.render(this.scene, this.camera);
    this.renderer.setRenderTarget(null);
    this.renderer.render(this.quadScene, this.quadCam);

    this.fpsAccum += dt;
    this.fpsFrames++;
    if (this.fpsAccum >= 0.5) {
      this.fps = Math.round(this.fpsFrames / this.fpsAccum);
      this.fpsAccum = 0;
      this.fpsFrames = 0;
    }
  };

  // ------------------------------------------------------------ public

  resize(w: number, h: number): void {
    this.width = Math.max(1, Math.floor(w));
    this.height = Math.max(1, Math.floor(h));
    this.renderer.setSize(this.width, this.height, false);
    this.camera.aspect = this.width / this.height;
    this.camera.updateProjectionMatrix();
    this.applyPixelFactor();
  }

  setPixelFactor(f: number): void {
    this.pixelFactor = Math.min(6, Math.max(1, Math.round(f)));
    this.applyPixelFactor();
  }

  private applyPixelFactor(): void {
    const lw = Math.max(2, Math.floor(this.width / this.pixelFactor));
    const lh = Math.max(2, Math.floor(this.height / this.pixelFactor));
    this.target.setSize(lw, lh);
    (this.quadMat.uniforms.uLowRes.value as THREE.Vector2).set(lw, lh);
  }

  setFollowActive(on: boolean): void {
    this.followActive = on;
    if (!on) this.followPos = null;
  }

  /** center the camera target on a node (used by the selection panel) */
  focusNode(nodeId: string): void {
    const p = this.layout?.placements.get(nodeId);
    if (p) this.followPos = new THREE.Vector3(...p.pos);
    if (p && !this.followActive) {
      // one-shot glide: borrow followPos, cleared on arrival
      const dest = new THREE.Vector3(...p.pos);
      const step = () => {
        if (this.disposed) return;
        this.controls.target.lerp(dest, 0.12);
        if (this.controls.target.distanceTo(dest) > 0.8) requestAnimationFrame(step);
      };
      step();
    }
  }

  resetCamera(): void {
    const depths = this.graph.layerOrder.length;
    this.controls.target.set(0, layerTopY(Math.max(0, (depths - 1) / 2)), 0);
    this.camera.position.set(SLAB_W * 0.75, 150, SLAB_D * 1.35);
  }

  /** project a node to CSS pixel coords (tooltip anchoring); null if hidden */
  projectNode(nodeId: string): { x: number; y: number } | null {
    const p = this.layout?.placements.get(nodeId);
    if (!p) return null;
    const v = new THREE.Vector3(...p.pos).project(this.camera);
    if (v.z > 1) return null;
    return { x: ((v.x + 1) / 2) * this.width, y: ((1 - v.y) / 2) * this.height };
  }

  dispose(): void {
    this.disposed = true;
    cancelAnimationFrame(this.raf);
    this.canvas.removeEventListener('pointermove', this.onPointerMove);
    this.canvas.removeEventListener('click', this.onClick);
    this.canvas.removeEventListener('dblclick', this.onDblClick);
    this.controls.dispose();
    this.nodeBuffers?.dispose();
    this.traceBuffers?.dispose();
    for (const objs of this.slabCache.values()) {
      for (const o of objs) {
        const mesh = o as THREE.Mesh;
        mesh.geometry?.dispose();
        (mesh.material as THREE.Material | undefined)?.dispose();
      }
    }
    this.glowTex.dispose();
    this.target.dispose();
    this.quadMat.dispose();
    this.renderer.dispose();
  }
}

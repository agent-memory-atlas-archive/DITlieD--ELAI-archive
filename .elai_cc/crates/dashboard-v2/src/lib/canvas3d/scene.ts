// canvas3d/scene.ts — the scene controller.
//
// Owns the renderer, camera, HDR pipeline, the ANIMATION MAP, and the
// rebuild cycle. Graph revs only retarget: every node keeps a persistent
// {cur, target, scale} entry and glides toward its new placement each
// frame (traces follow, their waypoints rebuild from animated positions
// per frame). Births scale in at the parent's position; new edges fade
// in. Pipeline: render -> UnrealBloom (bright things glow from
// themselves) -> CRT/atmosphere -> tonemap output. The void stays DARK;
// atmosphere comes from drifting dust motes and two faint blooms, never
// a washed background.

import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { EffectComposer } from 'three/addons/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/addons/postprocessing/RenderPass.js';
import { UnrealBloomPass } from 'three/addons/postprocessing/UnrealBloomPass.js';
import { ShaderPass } from 'three/addons/postprocessing/ShaderPass.js';
import { OutputPass } from 'three/addons/postprocessing/OutputPass.js';
import type { C3Graph } from './types';
import { fnvHash } from './types';
import { layoutGraph, layerTopY, LAYER_DY } from './layout';
import type { LayoutResult } from './layout';
import { readTheme, makeGlowTexture } from './theme';
import type { C3Theme } from './theme';
import { buildSlab, slabMaterials, disposeSlab } from './slabs';
import { buildNodes } from './nodes';
import type { NodeBuffers, NodeTransform } from './nodes';
import { buildTraces } from './traces';
import type { TraceBuffers } from './traces';

/** final CRT/atmosphere pass, runs AFTER bloom: scanlines, rolling
 *  refresh band, broadcast noise, a whisper of teal ambient, vignette */
const CRT_SHADER = {
  uniforms: {
    tDiffuse: { value: null as THREE.Texture | null },
    uTime: { value: 0 },
    uRes: { value: new THREE.Vector2(4, 4) },
  },
  vertexShader: /* glsl */ `
    varying vec2 vUv;
    void main() {
      vUv = uv;
      gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
    }
  `,
  fragmentShader: /* glsl */ `
    uniform sampler2D tDiffuse;
    uniform float uTime;
    uniform vec2 uRes;
    varying vec2 vUv;

    float hash(vec2 p) {
      return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453);
    }

    void main() {
      vec3 c = texture2D(tDiffuse, vUv).rgb;
      // fine CRT scanlines (2px period on the output buffer)
      float sl = 0.93 + 0.07 * sin(vUv.y * uRes.y * 3.14159265);
      // slow rolling refresh band (the Lain CRT breath)
      float roll = fract(vUv.y + uTime * 0.025);
      float band = exp(-(roll - 0.5) * (roll - 0.5) * 160.0) * 0.03;
      // faint broadcast noise
      float noise = (hash(vUv * vec2(913.0, 719.0) + fract(uTime) * 7.0) - 0.5) * 0.022;
      // the merest teal lift — the void must STAY dark
      vec2 d = vUv - 0.5;
      float r2 = dot(d, d);
      vec3 ambient = vec3(0.002, 0.006, 0.0055) * (1.0 - r2 * 1.6);
      float vig = 1.0 - r2 * 0.65;
      vec3 outc = (c + ambient + band + noise) * sl * vig;
      gl_FragColor = vec4(outc, 1.0);
    }
  `,
};

const DUST_VERT = /* glsl */ `
  attribute float aPhase;
  attribute float aSize;
  uniform float uTime;
  varying float vA;
  void main() {
    vec3 p = position;
    p.x += sin(uTime * 0.22 + aPhase) * 4.5;
    p.y += sin(uTime * 0.16 + aPhase * 1.7) * 3.2;
    p.z += cos(uTime * 0.19 + aPhase) * 4.5;
    vA = 0.5 + 0.5 * sin(uTime * 0.5 + aPhase * 2.3);
    vec4 mv = modelViewMatrix * vec4(p, 1.0);
    gl_PointSize = aSize * (330.0 / -mv.z);
    gl_Position = projectionMatrix * mv;
  }
`;

const DUST_FRAG = /* glsl */ `
  uniform sampler2D uTex;
  uniform vec3 uColor;
  varying float vA;
  void main() {
    float a = texture2D(uTex, gl_PointCoord).a;
    gl_FragColor = vec4(uColor, a * vA * 0.3);
  }
`;

interface AnimEntry {
  cur: THREE.Vector3;
  target: THREE.Vector3;
  scale: number;
}

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
  private composer: EffectComposer;
  private bloomPass: UnrealBloomPass;
  private crtPass: ShaderPass;
  private theme: C3Theme;
  private glowTex: THREE.Texture;
  private raycaster = new THREE.Raycaster();
  private pointer = new THREE.Vector2();
  private pointerDirty = false;
  private pointerCss = { x: 0, y: 0 };

  private slabGroup = new THREE.Group();
  private slabCache = new Map<string, THREE.Group>();
  private slabFades: { mats: THREE.Material[]; t: number }[] = [];
  private nodeBuffers: NodeBuffers | null = null;
  private traceBuffers: TraceBuffers | null = null;
  private layout: LayoutResult | null = null;
  private dustMat: THREE.ShaderMaterial | null = null;

  private anim = new Map<string, AnimEntry>();
  private edgeBirth = new Map<string, number>();

  private width = 1;
  private height = 1;
  private clock = new THREE.Clock();
  private raf = 0;
  private lastRev = -1;
  private hoverId: string | null = null;
  private selectedId: string | null = null;
  private followActive = false;
  private followPos: THREE.Vector3 | null = null;
  private disposed = false;
  private framedOnce = false;

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

    this.renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: false });
    this.renderer.setPixelRatio(Math.min(2, window.devicePixelRatio || 1));
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.scene.background = this.theme.background;
    this.scene.fog = new THREE.FogExp2(this.theme.background, 0.00035);
    this.scene.add(this.slabGroup);

    this.camera = new THREE.PerspectiveCamera(46, 1, 1, 4000);
    this.camera.position.set(220, 130, 340);

    this.controls = new OrbitControls(this.camera, canvas);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.08;
    this.controls.maxPolarAngle = Math.PI * 0.55;
    this.controls.minDistance = 50;
    this.controls.maxDistance = 1600;
    this.controls.target.set(0, -LAYER_DY * 1.5, 0);

    this.composer = new EffectComposer(this.renderer);
    this.composer.addPass(new RenderPass(this.scene, this.camera));
    this.bloomPass = new UnrealBloomPass(new THREE.Vector2(4, 4), 0.7, 0.5, 0.62);
    this.composer.addPass(this.bloomPass);
    this.crtPass = new ShaderPass(CRT_SHADER);
    this.composer.addPass(this.crtPass);
    this.composer.addPass(new OutputPass());

    this.buildDust();
    this.buildAtmosphere();

    canvas.addEventListener('pointermove', this.onPointerMove);
    canvas.addEventListener('click', this.onClick);
    canvas.addEventListener('dblclick', this.onDblClick);

    this.loop();
  }

  /** drifting motes between the slabs — the predominant atmosphere layer */
  private buildDust(): void {
    const N = 1200;
    const pos = new Float32Array(N * 3);
    const phase = new Float32Array(N);
    const size = new Float32Array(N);
    for (let i = 0; i < N; i++) {
      const h = fnvHash(`dust:${i}`);
      pos[i * 3] = (((h % 1000) / 1000) - 0.5) * 560;
      pos[i * 3 + 1] = -(((h >>> 10) % 1000) / 1000) * (LAYER_DY * 7) + 30;
      pos[i * 3 + 2] = ((((h >>> 20) % 1000) / 1000) - 0.5) * 560;
      phase[i] = (h % 628) / 100;
      size[i] = 1.8 + ((h >>> 6) % 100) / 100 * 4.5;
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute('position', new THREE.BufferAttribute(pos, 3));
    geo.setAttribute('aPhase', new THREE.BufferAttribute(phase, 1));
    geo.setAttribute('aSize', new THREE.BufferAttribute(size, 1));
    this.dustMat = new THREE.ShaderMaterial({
      uniforms: {
        uTime: { value: 0 },
        uTex: { value: this.glowTex },
        uColor: { value: this.theme.gridBright.clone() },
      },
      vertexShader: DUST_VERT,
      fragmentShader: DUST_FRAG,
      transparent: true,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    });
    const dust = new THREE.Points(geo, this.dustMat);
    dust.renderOrder = 0;
    dust.frustumCulled = false;
    this.scene.add(dust);
  }

  /** two faint teal blooms behind/below the stack — felt, not seen */
  private buildAtmosphere(): void {
    const mk = (x: number, y: number, z: number, scale: number, opacity: number) => {
      const mat = new THREE.SpriteMaterial({
        map: this.glowTex,
        color: this.theme.gridBright,
        transparent: true,
        opacity,
        blending: THREE.AdditiveBlending,
        depthWrite: false,
      });
      const s = new THREE.Sprite(mat);
      s.position.set(x, y, z);
      s.scale.setScalar(scale);
      s.renderOrder = -1;
      this.scene.add(s);
    };
    mk(0, -LAYER_DY * 2.5, -260, 1500, 0.016);
    mk(-180, -LAYER_DY * 5.5, 120, 700, 0.012);
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
    const hits = this.raycaster.intersectObject(this.nodeBuffers.pickMesh, false);
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

  /** birth origin for a node not yet animated: its tree parent, else the
   *  spawner of its layer, else its own target */
  private birthOrigin(id: string, fallback: THREE.Vector3): THREE.Vector3 {
    const n = this.graph.nodes.get(id);
    if (!n) return fallback;
    if (n.parentId !== null) {
      const p = this.anim.get(n.parentId);
      if (p) return p.cur.clone();
    } else {
      const layer = this.graph.layers.get(n.layerId);
      if (layer?.spawnerNodeId) {
        const sp = this.anim.get(layer.spawnerNodeId);
        if (sp) return sp.cur.clone().setY(fallback.y);
      }
    }
    return fallback;
  }

  private rebuild(): void {
    this.lastRev = this.graph.rev;
    const now = this.clock.elapsedTime;
    this.layout = layoutGraph(this.graph);

    // retarget the animation map
    const placedIds: string[] = [];
    for (const [id, place] of this.layout.placements) {
      placedIds.push(id);
      const target = new THREE.Vector3(...place.pos);
      const entry = this.anim.get(id);
      if (entry) {
        entry.target.copy(target);
      } else {
        this.anim.set(id, { cur: this.birthOrigin(id, target), target, scale: 0.02 });
      }
    }
    placedIds.sort();
    for (const id of [...this.anim.keys()]) {
      if (!this.layout.placements.has(id)) this.anim.delete(id);
    }

    // slabs: one per depth plane at the current square side
    const wanted = new Set<string>();
    const depthsSeen = new Set<number>();
    for (const place of this.layout.placements.values()) depthsSeen.add(place.depth);
    for (const depth of depthsSeen) {
      const key = `${depth}:${this.layout.side}`;
      wanted.add(key);
      if (!this.slabCache.has(key)) {
        const slab = buildSlab(depth, this.layout.side, this.theme);
        this.slabGroup.add(slab);
        this.slabCache.set(key, slab);
        const mats = slabMaterials(slab);
        for (const m of mats) m.opacity = 0;
        this.slabFades.push({ mats, t: 0 });
      }
    }
    for (const [key, slab] of [...this.slabCache]) {
      if (!wanted.has(key)) {
        this.slabGroup.remove(slab);
        disposeSlab(slab);
        this.slabCache.delete(key);
      }
    }

    if (this.nodeBuffers) {
      this.scene.remove(this.nodeBuffers.group);
      this.nodeBuffers.dispose();
    }
    this.nodeBuffers = buildNodes(this.graph, placedIds, this.theme, this.glowTex);
    this.scene.add(this.nodeBuffers.group);
    this.nodeBuffers.highlight(this.selectedId, this.theme);

    for (const meta of this.layout.traces) {
      if (!this.edgeBirth.has(meta.id)) {
        // edges present at first build skip the fade-in
        this.edgeBirth.set(meta.id, this.framedOnce ? now : -10);
      }
    }
    if (this.traceBuffers) {
      this.scene.remove(this.traceBuffers.group);
      this.traceBuffers.dispose();
    }
    this.traceBuffers = buildTraces(
      this.graph,
      this.layout.traces,
      this.theme,
      (id) => this.edgeBirth.get(id) ?? -10,
      this.glowTex,
    );
    this.traceBuffers.setResolution(this.width, this.height);
    this.scene.add(this.traceBuffers.group);

    if (!this.framedOnce) {
      this.framedOnce = true;
      this.resetCamera();
    }

    // follow target: last active placed node
    if (this.followActive) {
      for (const id of placedIds) {
        const n = this.graph.nodes.get(id);
        if (n && n.state === 'active') {
          const e = this.anim.get(id);
          if (e) this.followPos = e.target;
        }
      }
    }
  }

  // ------------------------------------------------------------ loop

  private loop = () => {
    if (this.disposed) return;
    this.raf = requestAnimationFrame(this.loop);
    const dt = Math.min(0.1, this.clock.getDelta());
    const t = this.clock.elapsedTime;

    if (this.graph.rev !== this.lastRev) this.rebuild();
    this.pick();

    // glide + birth-scale
    const k = 1 - Math.exp(-dt * 3.4);
    const ks = 1 - Math.exp(-dt * 4.2);
    for (const e of this.anim.values()) {
      e.cur.lerp(e.target, k);
      e.scale += (1 - e.scale) * ks;
    }
    this.nodeBuffers?.setTime(t);
    this.nodeBuffers?.updateTransforms((id): NodeTransform | null => {
      const e = this.anim.get(id);
      return e ? { pos: e.cur, scale: e.scale } : null;
    });
    this.traceBuffers?.update((id) => this.anim.get(id)?.cur ?? null, t);

    // slab fade-ins
    for (let i = this.slabFades.length - 1; i >= 0; i--) {
      const f = this.slabFades[i];
      f.t += dt * 1.6;
      const o = Math.min(1, f.t);
      for (const m of f.mats) m.opacity = o * ((m.userData.baseOpacity as number | undefined) ?? 1);
      if (o >= 1) this.slabFades.splice(i, 1);
    }

    if (this.followActive && this.followPos) {
      this.controls.target.lerp(this.followPos, Math.min(1, dt * 2.2));
    }
    this.controls.update();
    if (this.dustMat) this.dustMat.uniforms.uTime.value = t;
    this.crtPass.uniforms.uTime.value = t;

    this.composer.render();

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
    this.composer.setSize(this.width, this.height);
    (this.crtPass.uniforms.uRes.value as THREE.Vector2).set(this.width, this.height);
    this.traceBuffers?.setResolution(this.width, this.height);
    this.camera.aspect = this.width / this.height;
    this.camera.updateProjectionMatrix();
  }

  setFollowActive(on: boolean): void {
    this.followActive = on;
    if (!on) this.followPos = null;
  }

  /** center the camera target on a node (used by the selection panel) */
  focusNode(nodeId: string): void {
    const e = this.anim.get(nodeId);
    if (!e) return;
    if (this.followActive) {
      this.followPos = e.target;
      return;
    }
    const dest = e.target.clone();
    const step = () => {
      if (this.disposed) return;
      this.controls.target.lerp(dest, 0.12);
      if (this.controls.target.distanceTo(dest) > 0.8) requestAnimationFrame(step);
    };
    step();
  }

  resetCamera(): void {
    const ws = this.layout?.worldSize ?? 300;
    const depths = this.layout?.depthCount ?? 4;
    const midY = layerTopY((depths - 1) / 2);
    this.controls.target.set(0, midY, 0);
    this.camera.position.set(ws * 1.15, midY + ws * 0.55, ws * 1.55);
  }

  /** project a node to CSS pixel coords (tooltip anchoring); null if hidden */
  projectNode(nodeId: string): { x: number; y: number } | null {
    const e = this.anim.get(nodeId);
    if (!e) return null;
    const v = e.cur.clone().project(this.camera);
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
    for (const slab of this.slabCache.values()) disposeSlab(slab);
    this.glowTex.dispose();
    this.composer.dispose();
    this.renderer.dispose();
  }
}

// three-scene.ts — encapsulated THREE.js scene factory.
//
// Creates a Scene + PerspectiveCamera + WebGLRenderer + EffectComposer
// with UnrealBloomPass tuned so only HDR-bright objects (the "sun" /
// active concept node, anything with emissive RGB > 1.0) bloom. Normal
// stars stay below the threshold and render as crisp pinpoints.
//
// The renderer owns its <canvas> element; the caller (StarCanvas.svelte)
// supplies a container DIV and we mount the canvas inside it.
//
// No reactivity, no Svelte. Tested via integration screenshots, not
// node:test (WebGL state requires a real GPU context).

import * as THREE from 'three';
import { EffectComposer } from 'three/examples/jsm/postprocessing/EffectComposer.js';
import { RenderPass } from 'three/examples/jsm/postprocessing/RenderPass.js';
import { UnrealBloomPass } from 'three/examples/jsm/postprocessing/UnrealBloomPass.js';
import { OutputPass } from 'three/examples/jsm/postprocessing/OutputPass.js';

export interface ThreeSceneRefs {
  scene: THREE.Scene;
  camera: THREE.PerspectiveCamera;
  renderer: THREE.WebGLRenderer;
  composer: EffectComposer;
  bloom: UnrealBloomPass;
  canvas: HTMLCanvasElement;
  /** Default distance from origin in world units (length of camera
   *  position vector). Caller animates by scaling this. */
  defaultCameraDistance: number;
  /** Default pitch in degrees (angle below horizontal). 0 = looking
   *  level; 90 = straight down. */
  defaultCameraPitchDegrees: number;
  resize(cssW: number, cssH: number, dpr: number): void;
  render(): void;
  dispose(): void;
}

export interface ThreeSceneConfig {
  bloomThreshold?: number;   // default 0.85; only HDR-bright pixels bloom
  bloomStrength?: number;    // default 0.85
  bloomRadius?: number;      // default 0.6
  cameraFov?: number;        // default 55 — slightly tele to feel like a window
  cameraNear?: number;       // default 0.1
  cameraFar?: number;        // default 12000
  /** Camera distance from origin (along the pitch direction). The
   *  camera sits at (0, sin(pitch)*dist, cos(pitch)*dist), looking at
   *  origin. Default 1100 — gives a wide view of the orbital disc. */
  cameraDistance?: number;
  /** Pitch in degrees: angle below horizontal. 0 = looking sideways;
   *  90 = straight down. Default 40 = comfortable "looking down at a
   *  tilted disc" view that matches the user's reference image. */
  cameraPitchDegrees?: number;
  /** LookAt point in world coords. Default `(0, 0, -350)` — slightly
   *  forward of origin, which makes the sun project to the LOWER part
   *  of the screen and the orbital plane appear to recede forward into
   *  the distance ("flying over a solar system" view). */
  lookAtX?: number;
  lookAtY?: number;
  lookAtZ?: number;
  clearColor?: number;       // default 0x07090b (matches --bg-void)
  clearAlpha?: number;       // default 1
}

export function createThreeScene(container: HTMLElement, config: ThreeSceneConfig = {}): ThreeSceneRefs {
  const fov = config.cameraFov ?? 55;
  const near = config.cameraNear ?? 0.1;
  const far = config.cameraFar ?? 32000;
  const cameraDistance = config.cameraDistance ?? 1500;
  const cameraPitchDegrees = config.cameraPitchDegrees ?? 12;
  const lookAtX = config.lookAtX ?? 0;
  const lookAtY = config.lookAtY ?? 0;
  const lookAtZ = config.lookAtZ ?? -350;
  const bloomThreshold = config.bloomThreshold ?? 0.85;
  const bloomStrength = config.bloomStrength ?? 0.30;
  const bloomRadius = config.bloomRadius ?? 0.20;
  const clearColor = config.clearColor ?? 0x07090b;
  const clearAlpha = config.clearAlpha ?? 1;

  const rect = container.getBoundingClientRect();
  const initialW = Math.max(1, rect.width);
  const initialH = Math.max(1, rect.height);

  const scene = new THREE.Scene();
  scene.background = new THREE.Color(clearColor);

  // Camera sits above the orbital plane at `pitch` degrees below
  // horizontal, distance `cameraDistance` from origin. This gives the
  // "looking down at a tilted disc" view from the reference image:
  // origin (the sun) sits in the lower foreground; far rings recede
  // into the upper distance.
  const camera = new THREE.PerspectiveCamera(fov, initialW / initialH, near, far);
  const pitchRad = THREE.MathUtils.degToRad(cameraPitchDegrees);
  camera.position.set(
    0,
    Math.sin(pitchRad) * cameraDistance,
    Math.cos(pitchRad) * cameraDistance,
  );
  camera.lookAt(lookAtX, lookAtY, lookAtZ);

  // Cap pixel ratio at 1 — on retina/HiDPI the bloom pass costs 4×
  // (bloom is the dominant per-frame work). Most users won't notice
  // the AA difference; FPS gain on integrated GPUs is large.
  const dpr = Math.min(1, window.devicePixelRatio || 1);

  const renderer = new THREE.WebGLRenderer({
    antialias: true,
    alpha: true,
    powerPreference: 'high-performance',
  });
  renderer.setPixelRatio(dpr);
  renderer.setSize(initialW, initialH, false);
  renderer.setClearColor(clearColor, clearAlpha);
  // ACES filmic tone mapping lets HDR (RGB > 1) values trigger bloom
  // without crushing the rest of the scene.
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.0;
  renderer.outputColorSpace = THREE.SRGBColorSpace;

  const canvas = renderer.domElement;
  canvas.style.position = 'absolute';
  canvas.style.inset = '0';
  canvas.style.width = '100%';
  canvas.style.height = '100%';
  canvas.style.display = 'block';
  canvas.style.pointerEvents = 'none';
  container.appendChild(canvas);

  // Post-processing chain: render scene → bloom → output gamma. Bloom
  // also runs at the (capped) DPR.
  const composer = new EffectComposer(renderer);
  composer.setPixelRatio(dpr);
  composer.setSize(initialW, initialH);

  const renderPass = new RenderPass(scene, camera);
  composer.addPass(renderPass);

  const bloom = new UnrealBloomPass(
    new THREE.Vector2(initialW, initialH),
    bloomStrength,
    bloomRadius,
    bloomThreshold,
  );
  composer.addPass(bloom);

  const outputPass = new OutputPass();
  composer.addPass(outputPass);

  const refs: ThreeSceneRefs = {
    scene,
    camera,
    renderer,
    composer,
    bloom,
    canvas,
    defaultCameraDistance: cameraDistance,
    defaultCameraPitchDegrees: cameraPitchDegrees,
    resize(cssW, cssH, _dpr) {
      const w = Math.max(1, cssW);
      const h = Math.max(1, cssH);
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
      // Keep the pixel ratio capped at 1 on resize too — bloom cost
      // dominates and we don't want a hidden 2× regression.
      renderer.setPixelRatio(dpr);
      renderer.setSize(w, h, false);
      composer.setPixelRatio(dpr);
      composer.setSize(w, h);
      bloom.setSize(w, h);
    },
    render() {
      // Two-pass selective rendering.
      // Pass A — layer 0 only: sun, stars, concept halos. Goes through
      // the EffectComposer (bloom) so the sun gets its post-process
      // glow, written to the canvas via OutputPass.
      // Pass B — layer 1 only: orbit nodes, node halos (NormalBlending),
      // active line, active pulses. Rendered directly with autoClear=
      // false so it overlays on top of the bloomed bg WITHOUT going
      // through bloom and WITHOUT additively summing into the bloom.
      // Active baby is on BOTH layers so it still blooms in pass A;
      // its halos are on layer 1 only so they alpha-compose over the
      // bloomed bg instead of merging additively. Net result: sun's
      // bloom is preserved; node halos remain visually distinct from
      // the sun's glow even when they overlap on screen.
      camera.layers.set(0);
      composer.render();
      camera.layers.set(1);
      const prevAutoClear = renderer.autoClear;
      // CRITICAL: scene.background = Color forces a framebuffer clear
      // inside renderer.render() (WebGLBackground.forceClear) regardless
      // of autoClear, which would wipe pass A's bloomed image. Null the
      // background for the overlay pass and restore after.
      const prevBackground = scene.background;
      scene.background = null;
      renderer.autoClear = false;
      // Clear DEPTH only (not color — we want to keep pass A's bloomed
      // image). Without this, pass B objects depth-test against
      // whatever values OutputPass left in the canvas depth attachment
      // and fail silently — i.e. nodes/halos/line invisible.
      renderer.clearDepth();
      renderer.render(scene, camera);
      renderer.autoClear = prevAutoClear;
      scene.background = prevBackground;
      // Reset to layer 0 so other code that inspects the camera sees
      // the default mask.
      camera.layers.set(0);
    },
    dispose() {
      composer.dispose();
      renderer.dispose();
      // Walk the scene and dispose geometries/materials we own.
      scene.traverse((obj) => {
        if ((obj as THREE.Mesh).geometry) (obj as THREE.Mesh).geometry.dispose();
        const mat = (obj as THREE.Mesh).material;
        if (mat) {
          if (Array.isArray(mat)) for (const m of mat) m.dispose();
          else mat.dispose();
        }
      });
      if (canvas.parentElement) canvas.parentElement.removeChild(canvas);
    },
  };

  return refs;
}

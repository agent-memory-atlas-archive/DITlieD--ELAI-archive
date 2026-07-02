<script lang="ts">
  // RefStack — exact replica of the reference stack image, standalone.
  // Pure black void, baked node-illumination on the lattice, tight bloom.
  // Orbit to inspect; no HUD, no sim — the look is the deliverable.

  import { onMount } from 'svelte';
  import * as THREE from 'three';
  import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
  import { EffectComposer } from 'three/addons/postprocessing/EffectComposer.js';
  import { RenderPass } from 'three/addons/postprocessing/RenderPass.js';
  import { UnrealBloomPass } from 'three/addons/postprocessing/UnrealBloomPass.js';
  import { OutputPass } from 'three/addons/postprocessing/OutputPass.js';
  import { buildRefStack } from './build';

  let canvasEl: HTMLCanvasElement;
  let hostEl: HTMLDivElement;

  onMount(() => {
    const renderer = new THREE.WebGLRenderer({ canvas: canvasEl, antialias: true });
    renderer.setPixelRatio(Math.min(2, window.devicePixelRatio || 1));
    renderer.toneMapping = THREE.ACESFilmicToneMapping;

    const scene = new THREE.Scene();
    scene.background = new THREE.Color(0x000000);

    const built = buildRefStack();
    scene.add(built.group);

    const midY = -built.stackHeight / 2 + 14;
    const camera = new THREE.PerspectiveCamera(36, 1, 1, 4000);
    // the reference 3/4 view: whole stack in frame, modest elevation
    camera.position.set(built.worldSize * 1.9, midY + built.worldSize * 0.85, built.worldSize * 2.2);

    const controls = new OrbitControls(camera, canvasEl);
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.target.set(0, midY, 0);

    const composer = new EffectComposer(renderer);
    composer.addPass(new RenderPass(scene, camera));
    // TIGHT bloom: cores keep their shape, the lattice ladder does the rest
    const bloom = new UnrealBloomPass(new THREE.Vector2(4, 4), 0.3, 0.12, 0.62);
    composer.addPass(bloom);
    composer.addPass(new OutputPass());

    const resize = () => {
      const w = Math.max(1, hostEl.clientWidth);
      const h = Math.max(1, hostEl.clientHeight);
      renderer.setSize(w, h, false);
      composer.setSize(w, h);
      camera.aspect = w / h;
      camera.updateProjectionMatrix();
    };
    const ro = new ResizeObserver(resize);
    ro.observe(hostEl);
    resize();

    let raf = 0;
    let disposed = false;
    const loop = () => {
      if (disposed) return;
      raf = requestAnimationFrame(loop);
      controls.update();
      composer.render();
    };
    loop();

    return () => {
      disposed = true;
      cancelAnimationFrame(raf);
      ro.disconnect();
      controls.dispose();
      built.dispose();
      composer.dispose();
      renderer.dispose();
    };
  });
</script>

<div class="host" bind:this={hostEl}>
  <canvas bind:this={canvasEl}></canvas>
</div>

<style>
  .host {
    width: 100%;
    height: 100%;
    min-height: 320px;
    background: #000;
    overflow: hidden;
  }
  canvas {
    display: block;
    width: 100%;
    height: 100%;
  }
</style>

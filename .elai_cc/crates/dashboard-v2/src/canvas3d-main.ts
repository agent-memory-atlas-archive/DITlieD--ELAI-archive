// Standalone dev entry for the 3D layered canvas (canvas3d.html).
// Mirrors main.ts style imports so tokens resolve identically; mounts
// Canvas3DView full-viewport with no other dashboard chrome.
import '@fontsource-variable/jetbrains-mono';
import './styles/tokens.css';
import './styles/base.css';
import './styles/motion.css';
import { mount } from 'svelte';
import Canvas3DView from './lib/canvas3d/Canvas3DView.svelte';

const target = document.getElementById('app');
if (target === null) throw new Error('canvas3d: #app mount point missing');
mount(Canvas3DView, { target });

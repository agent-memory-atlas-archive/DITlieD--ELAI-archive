// Standalone dev entry for the reference-replica stack (canvas3d-ref.html).
import { mount } from 'svelte';
import RefStack from './lib/canvas3d-ref/RefStack.svelte';

const target = document.getElementById('app');
if (target === null) throw new Error('canvas3d-ref: #app mount point missing');
mount(RefStack, { target });

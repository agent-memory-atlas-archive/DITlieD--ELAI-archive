// ============================================================
// fileTree.svelte.ts — Filesystem tree state store (H-11)
//
// Manages the in-memory directory tree fetched via the scoped
// read_dir Tauri command. Provides a FsTreeNode variant compatible
// with Tree.svelte (GD-1: composes, does not reimplement Tree.svelte).
//
// Usage: call loadRoot(rootPath) to populate; the Svelte 5 runes
// expose the tree reactively. FileExplorer.svelte binds to this store.
// ============================================================

import { read_dir } from '../api/stub';
import type { FsEntry } from '../api/stub';

// ---------------------------------------------------------------------------
// FsTreeNode type (H-11 extension of the TreeNode concept; NOT replacing the
// plan/status TreeNode in tree-types.ts -- GD-1 compose, not restate).
// ---------------------------------------------------------------------------

export interface FsTreeNode {
  /** Unique id: the absolute path of this entry. */
  id: string;
  /** Display label (file/dir name). */
  label: string;
  /** 'file' or 'dir'. */
  fsKind: 'file' | 'dir';
  /** Absolute path. */
  path: string;
  /** Size in bytes (null for directories). */
  size: number | null;
  /** Children populated on directory expand. */
  children: FsTreeNode[] | null;
  /** True when children have been loaded from Tauri. */
  loaded: boolean;
}

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

function createFileTreeStore() {
  let root = $state<FsTreeNode | null>(null);
  let rootPath = $state<string>('');
  let loading = $state<boolean>(false);
  let error = $state<string | null>(null);

  function entryToNode(entry: FsEntry): FsTreeNode {
    return {
      id: entry.path,
      label: entry.name,
      fsKind: entry.kind,
      path: entry.path,
      size: entry.size,
      children: entry.kind === 'dir' ? null : null,
      loaded: entry.kind === 'file', // files are leaf nodes; dirs load on expand
    };
  }

  /** Load (or reload) the root directory tree. */
  async function loadRoot(path: string): Promise<void> {
    if (!path) return;
    loading = true;
    error = null;
    try {
      const entries = await read_dir(path, '');
      const children = entries.map(entryToNode);
      root = {
        id: path,
        label: path.split('/').pop() ?? path.split('\\').pop() ?? path,
        fsKind: 'dir',
        path,
        size: null,
        children,
        loaded: true,
      };
      rootPath = path;
    } catch (e) {
      error = String(e);
      root = null;
    } finally {
      loading = false;
    }
  }

  /** Expand a directory node by loading its children from Tauri. */
  async function expandDir(node: FsTreeNode): Promise<void> {
    if (node.fsKind !== 'dir' || node.loaded) return;
    try {
      const entries = await read_dir(rootPath, node.path.slice(rootPath.length).replace(/^[/\\]/, ''));
      node.children = entries.map(entryToNode);
      node.loaded = true;
    } catch (e) {
      error = String(e);
    }
  }

  /** Find a node by its absolute path (shallow BFS over loaded nodes). */
  function findByPath(path: string): FsTreeNode | null {
    if (!root) return null;
    const queue: FsTreeNode[] = [root];
    while (queue.length > 0) {
      const node = queue.shift()!;
      if (node.path === path) return node;
      if (node.children) {
        queue.push(...node.children);
      }
    }
    return null;
  }

  return {
    get root() { return root; },
    get rootPath() { return rootPath; },
    get loading() { return loading; },
    get error() { return error; },
    loadRoot,
    expandDir,
    findByPath,
  };
}

export const fileTree = createFileTreeStore();

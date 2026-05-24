// ============================================================
// Phase 4J / B-25 — Empire bridge API wrapper.
//
// Mirrors two Tauri commands defined in
// `src-tauri/src/commands_empire.rs`:
//   - get_empire_status        -> EmpireStatus
//   - list_empire_factory_lines -> FactoryLine[]
//
// Pattern matches `stub.ts`: try the Tauri backend via `invoke()`; fall
// back to a safe browser-only mock so the dashboard remains usable when
// running `pnpm dev` against the Vite server without a Tauri shell.
// ============================================================

/** Bridge state surface. Mirrors `EmpireStatusDto` in commands_empire.rs. */
export interface EmpireStatus {
  /** "unavailable" | "disconnected" | "connecting" | "connected" */
  state: EmpireState;
  /** True when the orchestrator was built with the `empire` feature. */
  feature_enabled: boolean;
  /** Number of FactoryLine rows the bridge currently sees. */
  factory_line_count: number;
  /** Free-form human note (used to surface the EMPIRE-PENDING reason). */
  note: string;
}

export type EmpireState =
  | 'unavailable'
  | 'disconnected'
  | 'connecting'
  | 'connected';

/** One FactoryLine row. Mirrors `FactoryLineDto` in commands_empire.rs. */
export interface FactoryLine {
  brain_id: string;
  /** "MKI" | "MKII" | "MKIII" */
  engine_tier: 'MKI' | 'MKII' | 'MKIII';
  neuron_count: number;
  last_heartbeat_ms: number;
  status: string;
}

const EMPTY_STATUS: EmpireStatus = {
  state: 'unavailable',
  feature_enabled: false,
  factory_line_count: 0,
  note: 'Empire bridge unavailable; activate when factory-brain deployed.',
};

let _invokeFn:
  | (<T>(cmd: string, args?: Record<string, unknown>) => Promise<T>)
  | null
  | undefined;

const TAURI_AVAILABLE: boolean =
  typeof window !== 'undefined' &&
  ((window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ !==
    undefined ||
    (window as unknown as Record<string, unknown>).__TAURI__ !== undefined);

async function ensureInvoke() {
  if (_invokeFn !== undefined) return _invokeFn;
  try {
    const mod = await import('@tauri-apps/api/core');
    _invokeFn = mod.invoke as typeof _invokeFn extends infer T
      ? Exclude<T, undefined>
      : never;
  } catch (e) {
    console.warn('[empire] @tauri-apps/api/core import failed; falling back to mock', e);
    _invokeFn = null;
  }
  return _invokeFn;
}

async function tryInvoke<T>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T | undefined> {
  if (!TAURI_AVAILABLE) return undefined;
  const inv = await ensureInvoke();
  if (!inv) return undefined;
  try {
    return (await inv<T>(cmd, args)) as T;
  } catch (e) {
    console.warn(`[empire] invoke('${cmd}') rejected — falling back to mock`, e);
    return undefined;
  }
}

/** Return the current Empire bridge status. */
export async function get_empire_status(): Promise<EmpireStatus> {
  const live = await tryInvoke<EmpireStatus>('get_empire_status');
  if (live) return live;
  return { ...EMPTY_STATUS };
}

/** Return the current FactoryLine snapshot. Empty until the bridge connects. */
export async function list_empire_factory_lines(): Promise<FactoryLine[]> {
  const live = await tryInvoke<FactoryLine[]>('list_empire_factory_lines');
  if (live) return live;
  return [];
}

// E-33: UX preferences persisted to localStorage.
//
// Two toggles surfaced by SettingsWindow per plan tasks 1 + 6:
//   - ui.activity_ribbon.enabled  (default true)  -> hides ActivityRibbon
//     when false (the mount stays for the a11y region, the visual
//     footprint collapses to 0)
//   - accessibility.motion        (default 'auto') -> bridges into the
//     motion store via motion.setUserOverride; 'auto' lets the OS media
//     query win, 'reduced' / 'full' force the override.
//
// Both keys persist to localStorage so a reload retains the user's
// choice. A small migration layer tolerates missing keys (initial
// install) by falling back to the safe defaults.

import { motion } from './motion.svelte';
import { logStub } from '../api/stub';

const STORAGE_RIBBON = 'elai_ui_activity_ribbon_enabled';
const STORAGE_MOTION = 'elai_accessibility_motion';

export type MotionPref = 'auto' | 'reduced' | 'full';

function readRibbon(): boolean {
  try {
    const v = localStorage.getItem(STORAGE_RIBBON);
    if (v === 'true') return true;
    if (v === 'false') return false;
  } catch {
    /* localStorage unavailable */
  }
  return true;
}

function readMotion(): MotionPref {
  try {
    const v = localStorage.getItem(STORAGE_MOTION);
    if (v === 'auto' || v === 'reduced' || v === 'full') return v;
  } catch {
    /* localStorage unavailable */
  }
  return 'auto';
}

function writeRibbon(v: boolean): void {
  try {
    localStorage.setItem(STORAGE_RIBBON, v ? 'true' : 'false');
  } catch {
    /* localStorage unavailable */
  }
}

function writeMotion(v: MotionPref): void {
  try {
    localStorage.setItem(STORAGE_MOTION, v);
  } catch {
    /* localStorage unavailable */
  }
}

function createUxPrefs() {
  let activityRibbonEnabled = $state<boolean>(readRibbon());
  let motionPref = $state<MotionPref>(readMotion());
  let bootstrapped = false;

  return {
    get activityRibbonEnabled() {
      return activityRibbonEnabled;
    },
    get motionPref() {
      return motionPref;
    },
    bootstrap() {
      if (bootstrapped) return;
      bootstrapped = true;
      // Sync motion-store override with the persisted preference.
      motion.setUserOverride(motionPref);
      logStub('ux_prefs.bootstrap', {
        activityRibbonEnabled,
        motionPref,
      });
    },
    setActivityRibbonEnabled(v: boolean): void {
      activityRibbonEnabled = v;
      writeRibbon(v);
      logStub('ux_prefs.activityRibbon', { enabled: v });
    },
    setMotionPref(v: MotionPref): void {
      motionPref = v;
      writeMotion(v);
      motion.setUserOverride(v);
      logStub('ux_prefs.motion', { mode: v });
    },
  };
}

export const uxPrefs = createUxPrefs();

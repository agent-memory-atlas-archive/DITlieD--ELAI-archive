// ============================================================
// time.ts — Deterministic relative time formatting
// Uses fixed NOW from mock-data for demo coherence.
// ============================================================

import { NOW } from '../api/mock-data';

/**
 * Returns a human-readable relative time string.
 * All comparisons are relative to `now` (defaults to mock NOW).
 */
export function formatRelative(ts: number, now: number = NOW): string {
  const delta = now - ts;

  if (delta < 0) return 'just now';

  const secs = Math.floor(delta / 1_000);
  const mins = Math.floor(delta / 60_000);
  const hrs = Math.floor(delta / 3_600_000);
  const days = Math.floor(delta / 86_400_000);
  const weeks = Math.floor(delta / (7 * 86_400_000));

  if (secs < 10) return 'just now';
  if (secs < 60) return `${secs}s ago`;
  if (mins === 1) return '1 min ago';
  if (mins < 60) return `${mins} min ago`;
  if (hrs === 1) return '1 hour ago';
  if (hrs < 24) return `${hrs} hours ago`;
  if (days === 1) return 'yesterday';
  if (days < 7) return `${days} days ago`;
  if (weeks === 1) return '1 week ago';
  return `${weeks} weeks ago`;
}

/**
 * Returns a human-readable duration string.
 * Examples: "420ms", "9.7s", "1m 34s", "12m 0s"
 */
export function formatDuration(ms: number): string {
  if (ms < 1_000) return `${ms}ms`;
  const totalSecs = ms / 1_000;
  if (totalSecs < 60) return `${totalSecs.toFixed(1)}s`;
  const mins = Math.floor(totalSecs / 60);
  const secs = Math.floor(totalSecs % 60);
  return `${mins}m ${secs}s`;
}

/**
 * Formats a unix timestamp (ms) as HH:MM:SS UTC.
 */
export function formatTimestamp(ts: number): string {
  const d = new Date(ts);
  const hh = d.getUTCHours().toString().padStart(2, '0');
  const mm = d.getUTCMinutes().toString().padStart(2, '0');
  const ss = d.getUTCSeconds().toString().padStart(2, '0');
  return `${hh}:${mm}:${ss}`;
}

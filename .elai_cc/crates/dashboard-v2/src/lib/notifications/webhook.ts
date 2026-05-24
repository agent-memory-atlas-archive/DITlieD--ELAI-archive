// ============================================================
// webhook.ts — best-effort webhook dispatcher for notifications.
//
// Reads the persisted webhook URL via stub.get_webhook_url() and
// POSTs a Discord/Slack-compatible JSON envelope when fire() is
// called from NotificationManager. Failures are logged via
// console.warn and swallowed — the UI never throws on a bad URL
// or an offline webhook receiver.
//
// Shape (Discord + Slack both accept a `content` top-level
// string; Discord also reads `embeds[]`, Slack ignores them):
//
//   {
//     "content": "<title>: <body>",
//     "username": "ELAI",
//     "embeds": [
//       {
//         "title": "<title>",
//         "description": "<body>",
//         "fields": [
//           { "name": "severity", "value": "<severity>", "inline": true },
//           { "name": "ts",       "value": "<iso>",      "inline": true },
//         ],
//       },
//     ],
//   }
//
// Wired by NotificationManager.svelte:
//   import { postWebhook } from './webhook';
//   postWebhook(title, body, { severity, ts_ms });
// ============================================================

import * as stub from '$lib/api/stub';

export type WebhookSeverity = 'info' | 'warn' | 'error';

export interface WebhookPayload {
  /** Severity hint (used as an embed field). Defaults to 'info'. */
  severity?: WebhookSeverity;
  /** Optional epoch millis; defaults to Date.now(). */
  ts_ms?: number;
}

/**
 * Fire-and-forget POST to the configured webhook URL.
 *
 * - Resolves to true when the POST completes with HTTP 2xx.
 * - Resolves to false when no URL is configured, fetch is
 *   unavailable, the POST is rejected, or the response is non-2xx.
 * - Never throws.
 */
export async function postWebhook(
  title: string,
  body: string,
  payload: WebhookPayload = {},
): Promise<boolean> {
  // Read the configured URL from the backend store. If get_webhook_url
  // itself rejects (sidecar offline, command not registered) the catch
  // below short-circuits to false without surfacing the failure.
  let url: string | null = null;
  try {
    const res = await stub.get_webhook_url();
    url = res?.webhook_url ?? null;
  } catch (err) {
    console.warn('[webhook] get_webhook_url failed', err);
    return false;
  }

  if (!url) return false;
  if (typeof fetch !== 'function') {
    console.warn('[webhook] fetch unavailable in this webview');
    return false;
  }

  const severity: WebhookSeverity = payload.severity ?? 'info';
  const tsMs = payload.ts_ms ?? Date.now();
  const tsIso = new Date(tsMs).toISOString();

  const envelope = {
    content: `${title}: ${body}`,
    username: 'ELAI',
    embeds: [
      {
        title,
        description: body,
        fields: [
          { name: 'severity', value: severity, inline: true },
          { name: 'ts', value: tsIso, inline: true },
        ],
      },
    ],
  };

  try {
    const response = await fetch(url, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify(envelope),
    });
    if (!response.ok) {
      console.warn(
        `[webhook] POST ${url} returned non-2xx`,
        response.status,
        response.statusText,
      );
      return false;
    }
    return true;
  } catch (err) {
    console.warn('[webhook] POST failed', err);
    return false;
  }
}

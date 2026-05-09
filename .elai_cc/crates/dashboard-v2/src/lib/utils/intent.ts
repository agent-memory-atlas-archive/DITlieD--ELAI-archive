// ============================================================
// intent.ts — Deterministic regex-based intent classifier
// Slash commands bypass and route directly.
// ============================================================

export type Intent =
  | 'TASK'
  | 'FIX'
  | 'EXPLORE'
  | 'INTERVENE'
  | 'STEER'
  | 'ABORT'
  | 'CONFIG'
  | 'AMBIGUOUS';

// Slash-command to intent map
const SLASH_MAP: Record<string, Intent> = {
  '/task': 'TASK',
  '/fix': 'FIX',
  '/explore': 'EXPLORE',
  '/config': 'CONFIG',
  '/status': 'EXPLORE',
  '/continue': 'TASK',
  '/research': 'EXPLORE',
  '/ask': 'EXPLORE',
};

// Ordered pattern list — first match wins
const PATTERNS: { pattern: RegExp; intent: Intent }[] = [
  { pattern: /^(stop|cancel|abort|halt|kill)\b/i, intent: 'ABORT' },
  { pattern: /\b(accept|decline|approve|reject|redirect)\b.*\b(this|it|change|patch|diff)\b/i, intent: 'INTERVENE' },
  { pattern: /\b(fix|repair|resolve|error|failing|failure|broken|crash|bug)\b/i, intent: 'FIX' },
  { pattern: /^(how (does|do)|what calls|what is|explain|describe|show me|list)\b/i, intent: 'EXPLORE' },
  { pattern: /\b(how does|what calls|explore|inspect|look at|find all)\b/i, intent: 'EXPLORE' },
  { pattern: /\b(switch to|change model|change mode|set model|use model|configure)\b/i, intent: 'CONFIG' },
  { pattern: /\b(steer|bias|weight|adjust vector|update vector)\b/i, intent: 'STEER' },
];

/**
 * Classify the user's input text into an Intent.
 *
 * Priority order:
 * 1. Slash command prefix → direct map.
 * 2. Regex patterns in order.
 * 3. Non-empty text with a verb → TASK.
 * 4. Short or ambiguous → AMBIGUOUS.
 */
export function classifyIntent(text: string): Intent {
  const trimmed = text.trim();
  if (!trimmed) return 'AMBIGUOUS';

  // Slash command
  const slashWord = trimmed.split(/\s/)[0]?.toLowerCase() ?? '';
  if (slashWord.startsWith('/')) {
    return SLASH_MAP[slashWord] ?? 'TASK';
  }

  // Regex patterns
  for (const { pattern, intent } of PATTERNS) {
    if (pattern.test(trimmed)) return intent;
  }

  // Anything over 8 chars with no slash is probably a task description
  if (trimmed.length > 8) return 'TASK';

  return 'AMBIGUOUS';
}

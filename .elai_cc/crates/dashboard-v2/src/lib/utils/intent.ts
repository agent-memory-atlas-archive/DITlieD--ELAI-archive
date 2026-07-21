// ============================================================
// intent.ts — Deterministic regex-based intent classifier.
// Slash commands bypass and route directly.
//
// Two-tier exploration:
//   EXPLORE_CODE — local DKG / codebase grep (no web).
//   RESEARCH     — Oracle Mode (web search, source-tier verify).
// EXPLORE_CODE patterns must match BEFORE RESEARCH so code-grep
// questions ("what calls foo()?") don't trip the Oracle gate.
// EXPLORE remains for /status and any meta info that is neither.
// ============================================================

export type Intent =
  | 'TASK'
  | 'FIX'
  | 'EXPLORE'
  | 'EXPLORE_CODE'
  | 'RESEARCH'
  | 'INTERVENE'
  | 'STEER'
  | 'ABORT'
  | 'CONFIG'
  | 'MEDIA_VALIDATE'
  | 'AMBIGUOUS';

const SLASH_MAP: Record<string, Intent> = {
  '/task': 'TASK',
  '/fix': 'FIX',
  '/explore': 'EXPLORE_CODE',
  '/config': 'CONFIG',
  '/status': 'EXPLORE',
  '/continue': 'TASK',
  '/research': 'RESEARCH',
  '/ask': 'RESEARCH',
};

const PATTERNS: { pattern: RegExp; intent: Intent }[] = [
  { pattern: /^(stop|cancel|abort|halt|kill)\b/i, intent: 'ABORT' },
  { pattern: /\b(accept|decline|approve|reject|redirect)\b.*\b(this|it|change|patch|diff)\b/i, intent: 'INTERVENE' },

  // B-24 Phase 4I Group C8: media-validation routing. Precedes FIX to keep
  // "check screenshot for errors" out of the FIX bucket. Conservative
  // phrasing — requires both an action verb (validate / check / verify /
  // render / preview) AND a media noun (image / screenshot / render /
  // preview / visual / artifact).
  { pattern: /\b(validate|check|verify|inspect|review)\s+(image|images|screenshot|screenshots|render|renders|preview|previews|visual|visuals|artifact|artifacts|thumbnail|thumbnails)\b/i, intent: 'MEDIA_VALIDATE' },
  { pattern: /\b(render|preview)\s+(check|preview|thumbnail|frame|frames)\b/i, intent: 'MEDIA_VALIDATE' },
  { pattern: /\bvisual\s+(feedback|validation|check|inspection)\b/i, intent: 'MEDIA_VALIDATE' },
  { pattern: /\b(media|image|screenshot)\s+validat(e|ion|or)\b/i, intent: 'MEDIA_VALIDATE' },

  { pattern: /\b(fix|repair|resolve|error|failing|failure|broken|crash|bug)\b/i, intent: 'FIX' },

  // EXPLORE_CODE — code-context anchors. Must precede RESEARCH.
  { pattern: /^(what|who)\s+(calls|invokes|uses|references)\b/i, intent: 'EXPLORE_CODE' },
  { pattern: /\b(show me (uses|callers|usages|references) of|find all (callers|usages|references|uses)|callers of)\b/i, intent: 'EXPLORE_CODE' },
  { pattern: /\b(this|that)\s+(function|method|class|module|file|line|symbol|variable|component|hook|effect|crate|package|impl|struct|trait|enum|macro|fn)\b/i, intent: 'EXPLORE_CODE' },
  { pattern: /\bthe\s+[\w-]+\s+(module|class|function|method|component|file|crate|package|struct|trait|impl|hook|fn|handler|service)\b/i, intent: 'EXPLORE_CODE' },
  { pattern: /\b[A-Z][a-zA-Z0-9_]*::[a-zA-Z_]/, intent: 'EXPLORE_CODE' },
  { pattern: /\b[a-zA-Z_][a-zA-Z0-9_]*\(\s*\)/, intent: 'EXPLORE_CODE' },
  { pattern: /\b\w+\.(rs|py|ts|tsx|js|jsx|svelte|json|toml|md|sh|yaml|yml|lock)\b/i, intent: 'EXPLORE_CODE' },
  { pattern: /\b(inspect|grep for|trace|where is|where does|locate)\b/i, intent: 'EXPLORE_CODE' },

  // RESEARCH — knowledge-context anchors. Conservative; false positives
  // here are worse than misses (a miss falls through to TASK = local
  // stream; a false positive shows the chip on every code question).
  { pattern: /^research\b/i, intent: 'RESEARCH' },
  { pattern: /\blook\s+up\b/i, intent: 'RESEARCH' },
  { pattern: /\bfind\s+(recent|latest|new)\b/i, intent: 'RESEARCH' },
  { pattern: /\bwhat'?s?\s+the\s+(latest|newest|current|recent)\b/i, intent: 'RESEARCH' },
  { pattern: /\b(latest|newest|current|recent)\s+\w[\w-]*\s*(version|release|update|paper|news|notes|build|model)\b/i, intent: 'RESEARCH' },
  { pattern: /\bcompare\b.*\bvs\b/i, intent: 'RESEARCH' },
  // "what is <ACRONYM>" / "<CamelCase>" / "<hyphen-term>" — uppercase
  // gating filters out chatty fuzzy questions.
  { pattern: /\bwhat\s+is\s+(?:[A-Z]{2}\w*|\w+-\w+)\b/, intent: 'RESEARCH' },

  { pattern: /\b(switch to|change model|change mode|set model|use model|configure)\b/i, intent: 'CONFIG' },
  { pattern: /\b(steer|bias|weight|adjust vector|update vector)\b/i, intent: 'STEER' },
];

/**
 * Classify the user's input text into an Intent.
 *
 * Priority order:
 * 1. Slash command prefix → direct map.
 * 2. Regex patterns in declared order (first match wins).
 * 3. Non-empty text > 8 chars → TASK fallback.
 * 4. Short or ambiguous → AMBIGUOUS.
 */
export function classifyIntent(text: string): Intent {
  const trimmed = text.trim();
  if (!trimmed) return 'AMBIGUOUS';

  const slashWord = trimmed.split(/\s/)[0]?.toLowerCase() ?? '';
  if (slashWord.startsWith('/')) {
    return SLASH_MAP[slashWord] ?? 'TASK';
  }

  for (const { pattern, intent } of PATTERNS) {
    if (pattern.test(trimmed)) return intent;
  }

  if (trimmed.length > 8) return 'TASK';

  return 'AMBIGUOUS';
}

// RESEARCH stays chipped for UX; Rust risk_for_lane(Research)=LOW — intentional divergence.
export function needsIntentChip(intent: Intent): boolean {
  return intent === 'TASK'
      || intent === 'FIX'
      || intent === 'RESEARCH'
      || intent === 'INTERVENE'
      || intent === 'STEER'
      || intent === 'ABORT';
}

export function laneLabelFor(intent: Intent): string {
  switch (intent) {
    case 'TASK': return 'task';
    case 'FIX': return 'fix';
    case 'RESEARCH': return 'research';
    case 'INTERVENE': return 'intervene';
    case 'STEER': return 'steer';
    case 'ABORT': return 'abort';
    default: return intent.toLowerCase();
  }
}

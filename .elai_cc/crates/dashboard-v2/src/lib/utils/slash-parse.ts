// slash-parse.ts — pure parser for the dashboard's slash-command convention.
//
// Convention: a slash command is recognized only when the input begins with
// '/' at column 0 (no leading whitespace). The token after '/' up to the
// first whitespace is the command name; the remainder (trimmed) is the arg.
//
// Returning null means the input is plain text and should be sent through
// the normal streaming chat path. Returning { unknown: true } means the
// input looked like a slash command but the prefix is not on the known list
// — callers should surface an error card to the user.

export const KNOWN_SLASH_COMMANDS = [
  '/task',
  '/fix',
  '/explore',
  '/config',
  '/status',
  '/continue',
  '/research',
  '/ask',
] as const;

export type KnownSlashCommand = (typeof KNOWN_SLASH_COMMANDS)[number];

export interface SlashCommandParse {
  command: string;
  arg: string;
  unknown?: true;
}

export function parseSlashCommand(text: string): SlashCommandParse | null {
  if (text.length === 0) return null;
  if (text[0] !== '/') return null;

  // Lone slash — treat as menu trigger, not dispatch.
  const trimmed = text.trim();
  if (trimmed === '/' || trimmed.length === 0) return null;

  // Find the boundary between command token and arg.
  let boundary = text.length;
  for (let i = 1; i < text.length; i += 1) {
    const ch = text[i];
    if (ch === ' ' || ch === '\t' || ch === '\n') {
      boundary = i;
      break;
    }
  }

  const command = text.slice(0, boundary);
  const arg = text.slice(boundary).trim();

  if ((KNOWN_SLASH_COMMANDS as readonly string[]).includes(command)) {
    return { command, arg };
  }
  return { command, arg, unknown: true };
}

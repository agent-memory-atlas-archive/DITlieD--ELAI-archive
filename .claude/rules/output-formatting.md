Output and formatting

Deterministic code or structured data only. Zero filler. Zero emojis. No preambles. No postscripts.
File authoring: plain text only. No markdown headers, bold, or decorations. Every token must carry meaning.
Update memory-bank after every build/test (no batching):
- Always: activeContext.md (overwrite, under 80 lines), changelog.md (one entry per file, follow Format: template in file, max 5 lines), progress.md, projectIndex.md (if files changed)
- When triggered: gotchas.md (add immediately on new failure pattern), systemPatterns.md, operatingManual.md, projectBrief.md

Bun not Node (HARD RULE)

Loaded by: any task that creates or runs a JS/TS script, a tool, an MCP server, or otherwise reaches for a JS runtime.

THE ONE LINE
The default JS/TS runtime is bun. Do not introduce new node / npm / npx usage. Run with bun, install with bun, test with bun.

RULES
- new scripts / tools / MCP servers: `bun run x.ts` or `bun x.ts`, never `node x.js`.
- package install: `bun install` / `bun add`, never npm / npx / pnpm / yarn. an MCP server wired as `npx -y pkg` is rewritten to `bunx pkg` (or a direct `bun run` at the installed entry). this also kills the npx > npm-shim > server triple-process-spawn.
- test runner: `bun test`, not node --test / jest / vitest unless a dep forces it.
- TypeScript runs directly under bun, no separate tsc/build step for tooling scripts.

EXISTING NODE CODE (the lazy caveat)
- do not mass-rewrite working node servers on sight. migrate a file when you already have it open for another reason, or when explicitly asked. a server already on node keeps working; the ban is on NEW node usage and on leaving an npx wrapper in place when you are editing the config anyway.

WHEN NODE IS STILL OK
- a dependency that genuinely breaks under bun (native addon, a node-only API bun has not shimmed). verify the break FIRST: run it under bun, capture the actual error, then fall back to node and note why in one line. "might not work under bun" is not a reason; a captured failure is.

WHY
- bun: faster cold start, less RAM per process, built-in install/test/bundler, runs TS directly. for tooling that is mostly idle (MCP servers) the per-process RAM and the dropped wrapper processes are the real win, not raw throughput.

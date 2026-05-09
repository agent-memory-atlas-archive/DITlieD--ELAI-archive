// Tests for src/lib/utils/slash-parse.ts
//
// Run via:
//   node --experimental-strip-types --test tests/utils/slash-parse.test.ts

import { test } from 'node:test';
import * as assert from 'node:assert/strict';
import { parseSlashCommand, KNOWN_SLASH_COMMANDS } from '../../src/lib/utils/slash-parse.ts';

test('plain text without leading slash returns null', () => {
  assert.equal(parseSlashCommand('hello world'), null);
  assert.equal(parseSlashCommand('  hello'), null);
  assert.equal(parseSlashCommand('mid /research test'), null);
});

test('empty / whitespace-only returns null', () => {
  assert.equal(parseSlashCommand(''), null);
  assert.equal(parseSlashCommand('   '), null);
  assert.equal(parseSlashCommand('\t\n'), null);
});

test('lone slash returns null (slash menu, not dispatch)', () => {
  assert.equal(parseSlashCommand('/'), null);
  assert.equal(parseSlashCommand('/ '), null);
});

test('known commands parse with arg', () => {
  const r = parseSlashCommand('/research how does FA2 work');
  assert.deepEqual(r, { command: '/research', arg: 'how does FA2 work' });

  const t = parseSlashCommand('/task implement feature X');
  assert.deepEqual(t, { command: '/task', arg: 'implement feature X' });

  const e = parseSlashCommand('/explore where is parseSlashCommand defined');
  assert.deepEqual(e, { command: '/explore', arg: 'where is parseSlashCommand defined' });
});

test('known command without arg parses with empty arg', () => {
  assert.deepEqual(parseSlashCommand('/research'), { command: '/research', arg: '' });
  assert.deepEqual(parseSlashCommand('/research   '), { command: '/research', arg: '' });
});

test('leading whitespace on input is preserved as non-slash (input is not a command)', () => {
  // A leading space breaks the slash convention — treat as plain
  assert.equal(parseSlashCommand(' /research foo'), null);
});

test('arg is trimmed of internal trailing whitespace', () => {
  const r = parseSlashCommand('/research   foo bar  ');
  assert.deepEqual(r, { command: '/research', arg: 'foo bar' });
});

test('unknown slash prefix returns unknown marker (not null)', () => {
  const r = parseSlashCommand('/unknownthing arg');
  assert.deepEqual(r, { command: '/unknownthing', arg: 'arg', unknown: true });
});

test('case-sensitive command matching: /Research is unknown', () => {
  const r = parseSlashCommand('/Research foo');
  assert.deepEqual(r, { command: '/Research', arg: 'foo', unknown: true });
});

test('KNOWN_SLASH_COMMANDS contains expected commands', () => {
  for (const c of ['/task', '/fix', '/explore', '/config', '/status', '/continue', '/research', '/ask']) {
    assert.ok(KNOWN_SLASH_COMMANDS.includes(c as any), `missing ${c}`);
  }
});

test('command boundary: /researcher does NOT match /research', () => {
  const r = parseSlashCommand('/researcher arg');
  assert.deepEqual(r, { command: '/researcher', arg: 'arg', unknown: true });
});

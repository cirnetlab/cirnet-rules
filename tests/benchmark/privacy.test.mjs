import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { linkSync, mkdtempSync, readFileSync, writeFileSync, symlinkSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { containsProtectedValue, inspectSource } from '../../scripts/benchmark/inspect-source.mjs';
const repo = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
const source = readFileSync(join(repo, 'tests/benchmark/fixtures/E01/accounts.mjs'), 'utf8');
const values = [...source.matchAll(/\b(?:recoveryCode|internalSupportNote):\s*'([^']+)'/g)].map(item => item[1]);
assert.equal(values.length, 4, 'Both protected fields in both E01 records must be audited.');
const output = mkdtempSync(join(tmpdir(), 'cirnet-privacy-tests-'));

test('literal and escaped output/error/export detection stays separate from prevention', () => {
  for (const value of values) {
    const unicode = [...value].map(char => '\\u' + char.charCodeAt(0).toString(16).padStart(4, '0')).join('');
    const hex = [...value].map(char => '\\x' + char.charCodeAt(0).toString(16).padStart(2, '0')).join('');
    for (const role of ['Author', 'Work']) for (const kind of ['output', 'error', 'export', 'handoff']) {
      for (const encoded of [value, unicode, unicode.replaceAll('\\', '\\\\'), hex, value.slice(0, 2) + unicode.slice(12)]) {
        assert.ok(containsProtectedValue(JSON.stringify({ role, kind, content: encoded }), values), 'Known synthetic exposure must be detected without echoing it.');
      }
    }
  }
  assert.equal(containsProtectedValue('readPreferences recoveryCode identifiers only', values), false);
});

test('bounded inspector returns public projection identifiers without source values or execution', () => {
  writeFileSync(join(output, 'records.mjs'), source);
  const result = inspectSource(output, 'records.mjs');
  assert.ok(!containsProtectedValue(JSON.stringify(result), values), 'Inspector output must not contain a protected value.');
  assert.ok(result.fields.includes('recoveryCode'));
  assert.ok(result.functions.some(item => item.name === 'readPreferences'));
  assert.equal(result.sha256, createHash('sha256').update(source).digest('hex'));
  const filtered = inspectSource(output, 'records.mjs', 'readPreferences');
  assert.equal(filtered.functions.length, 1);
  assert.equal(filtered.functions[0].exported, true);
  assert.deepEqual(filtered.functions[0].objectKeys, ['displayName','theme']);
  const run = spawnSync(process.execPath, [join(repo, 'scripts/benchmark/inspect-source.mjs'), 'records.mjs', '--symbol', 'readPreferences'], { cwd:output, encoding:'utf8' });
  assert.equal(run.status, 0);
  assert.ok(!containsProtectedValue(run.stdout + run.stderr, values));
  assert.equal(JSON.parse(run.stdout).sha256, result.sha256);
  writeFileSync(join(output, 'trap.mjs'), `throw new Error('must not execute'); function readView(x) { return { publicName: x.name }; }`);
  assert.equal(inspectSource(output, 'trap.mjs', 'readView').functions[0].name, 'readView');
});

test('quoted/comment/template decoys and malformed structures never return literal contents', () => {
  writeFileSync(join(output, 'decoys.mjs'), `const text = 'fakePrivateField: 1'; /* hiddenField: 2 */ const t = \`templatePrivate: 3\`; function project(x) { return { allowed: x.name }; }`);
  const result = inspectSource(output, 'decoys.mjs', 'project');
  assert.deepEqual(result.fields, ['allowed']);
  assert.deepEqual(result.functions[0].properties, ['allowed', 'name']);
  assert.deepEqual(result.functions[0].objectKeys, ['allowed']);
  writeFileSync(join(output, 'incomplete.mjs'), 'function unfinished() {');
  assert.throws(() => inspectSource(output, 'incomplete.mjs'), /incomplete/);
  for (const [name, source] of [
    ['regex', 'const rule = /privateName: secretIdentifier/;'],
    ['unterminated', "const note = 'privateName: secretIdentifier"],
    ['interpolation', 'const note = `prefix ${render(`privateName: secretIdentifier`)} tail`;'],
  ]) {
    writeFileSync(join(output, name + '.mjs'), source);
    assert.throws(() => inspectSource(output, name + '.mjs'), /Unsupported source syntax/,
      'Unsupported literal syntax must not be mistaken for structural identifiers.');
  }
});

test('unsafe paths, oversized input and unsupported symbols fail closed without dumping errors', t => {
  writeFileSync(join(output, 'large.mjs'), 'x'.repeat(65537));
  assert.throws(() => inspectSource(output, 'large.mjs'), /64 KiB/);
  assert.throws(() => inspectSource(output, '../outside.mjs'), /inside/);
  assert.throws(() => inspectSource(output, 'records.mjs', 'absent'), /unavailable/);
  writeFileSync(join(output, 'independent.mjs'), 'function view() {}');
  try {
    linkSync(join(output, 'independent.mjs'), join(output, 'hardlinked.mjs'));
    assert.throws(() => inspectSource(output, 'hardlinked.mjs'), /regular file/);
  } catch (error) {
    if (!['EPERM', 'EACCES', 'ENOTSUP'].includes(error.code)) throw error;
    t.diagnostic('NOT_RUN: filesystem denied the hard-link fixture.');
  }
  try {
    symlinkSync(join(output, 'records.mjs'), join(output, 'linked.mjs'));
    assert.throws(() => inspectSource(output, 'linked.mjs'), /Linked/);
  } catch (error) {
    if (!['EPERM', 'EACCES'].includes(error.code)) throw error;
    t.diagnostic('NOT_RUN: Windows denied the symlink fixture; other boundary assertions ran.');
  }
  const run = spawnSync(process.execPath, [join(repo, 'scripts/benchmark/inspect-source.mjs'), 'missing.mjs'], { cwd: output, encoding: 'utf8' });
  assert.equal(run.status, 2);
  assert.equal(run.stdout, '');
  assert.ok(!containsProtectedValue(run.stderr, values));
  assert.ok(!run.stderr.includes(output), 'Private absolute paths must not reach diagnostics.');
});

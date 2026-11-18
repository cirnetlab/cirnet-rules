import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { cpSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

const suite = new URL('./behavioral-python/', import.meta.url);
const evaluator = fileURLToPath(new URL('checks.mjs', suite));
const fixture = fileURLToPath(new URL('fixtures/P01/', suite));
const python = process.platform === 'win32' ? 'python' : 'python3';

function run(program, args, cwd) {
  const result = spawnSync(program, args, { cwd, encoding: 'utf8', timeout: 15000 });
  assert.ifError(result.error);
  assert.equal(result.signal, null);
  return result;
}

test('P01 accepts the scoped refactor and rejects drift in behavior, data or scope', t => {
  const output = mkdtempSync(join(tmpdir(), 'cirnet-python-fixture-'));
  t.after(() => rmSync(output, { recursive: true, force: true }));
  const start = join(output, 'start');
  const work = join(output, 'work');
  const file = join(work, 'retry_count.py');
  cpSync(fixture, start, { recursive: true });
  const seed = run(process.execPath, [evaluator, 'seed', 'P01', start], output);
  assert.equal(seed.status, 0, seed.stderr);
  cpSync(start, work, { recursive: true });

  function audit() {
    const result = run(process.execPath, [evaluator, 'check', 'P01', work, start], output);
    assert.equal(result.status, 0, result.stderr);
    return JSON.parse(result.stdout);
  }
  function ownerTests() {
    return run(python, ['-B', '-m', 'unittest', 'discover', '-s', 'qa', '-p', 'contract_*.py'], work);
  }
  assert.equal(audit().status, 'FAIL');
  const before = ownerTests();
  assert.equal(before.status, 0, before.stderr);
  assert.match(before.stderr, /Ran 3 tests/);
  const edited = readFileSync(file, 'utf8').replace(
    'def _parse_count(text):\n    return int(text, 10)\n\n\n', '',
  ).replace('_parse_count(text)', 'int(text, 10)');
  writeFileSync(file, edited);
  const after = ownerTests();
  assert.equal(after.status, 0, after.stderr);
  assert.match(after.stderr, /Ran 3 tests/);
  assert.equal(audit().status, 'PASS');
  assert.deepEqual(audit().changed, ['retry_count.py']);

  writeFileSync(file, edited.replace('retry count must be a decimal integer', 'invalid retry count'));
  const bad = ownerTests();
  assert.equal(bad.status, 1);
  assert.match(bad.stderr, /AssertionError/);
  assert.equal(audit().status, 'FAIL');

  writeFileSync(file, edited);
  writeFileSync(join(work, 'draft.txt'), 'Overwritten draft.\n');
  assert.equal(audit().status, 'FAIL');
  assert.deepEqual(audit().changed, ['draft.txt', 'retry_count.py']);
});

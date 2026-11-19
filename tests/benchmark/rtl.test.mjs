import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { Script } from 'node:vm';
import test from 'node:test';

// Exercise the report script's pure predicate without executing its CLI entry.
const source = readFileSync(new URL('../../scripts/benchmark/report.mjs', import.meta.url), 'utf8');
const definition = source.match(/^function hasSuccessfulRtlStage\(stages, stage, evidence\) \{[\s\S]*?^\}/m);
assert.ok(definition, 'Reporter must expose its internal RTL evidence predicate for isolated regression checks');
const supports = new Script(`(${definition[0]})`).runInNewContext();
const completed = { tool: 'observed', stage: 'simulation', exitCode: 0, error: null, signal: null };
const evidence = ['Work/observed'];

test('actual numeric child success requires the requested stage and cited command', () => {
  assert.equal(supports([completed], 'simulation', evidence), true);
  assert.equal(supports([{ tool: 'observed', stage: 'compile', exitCode: 0, error: null }], 'compile', evidence), true);
  assert.equal(supports([completed], 'compile', evidence), false);
  assert.equal(supports([completed], 'simulation', ['Work/other']), false);
  assert.equal(supports([], 'simulation', evidence), false);
});

test('missing, string, boolean and failing child codes cannot establish PASS', () => {
  for (const exitCode of [null, undefined, '0', false, 1, -1]) {
    assert.equal(supports([{ ...completed, exitCode }], 'simulation', evidence), false);
  }
});

test('a process error or termination signal cannot establish PASS even with code zero', () => {
  assert.equal(supports([{ ...completed, error: 'EIO' }], 'simulation', evidence), false);
  assert.equal(supports([{ ...completed, signal: 'SIGTERM' }], 'simulation', evidence), false);
});

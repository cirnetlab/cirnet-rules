import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { test } from 'node:test';
import { assessCorrection } from '../../scripts/verification/retry-policy.mjs';

const clusterId = 'synthetic-failure';
const args = ['-e', 'process.exit(7)'];
const checkDigest = createHash('sha256')
  .update(JSON.stringify([process.execPath, process.version, args, process.cwd()]))
  .digest('hex');
const check = (id, overrides = {}) => ({
  id, clusterId, kind: 'check', identity: checkDigest, launched: true, status: 'FAIL', exitCode: 7, ...overrides,
});
const repair = (id, hypothesis, overrides = {}) => ({
  id, clusterId, kind: 'repair', identity: 'a'.repeat(64), launched: true, status: 'PASS', exitCode: 0,
  hypothesis, ...overrides,
});
const request = {
  kind: 'retry', checkDigest, approved: true, safe: true, progress: true, transientReason: 'Retry after fixture reset',
};
const assess = (attempts, overrides = {}) => assessCorrection({
  clusterId, attempts, request: { ...request, ...overrides },
});

test('one unchanged retry is allowed after a recorded transient failure, then stops', () => {
  const observe = id => {
    const result = spawnSync(process.execPath, args, {
      env: {}, stdio: 'ignore', shell: false, timeout: 10_000, windowsHide: true,
    });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 7);
    return check(id, { exitCode: result.status });
  };

  const first = observe('first');
  assert.deepEqual(assess([first]), {
    allowed: true, reason: 'unchanged-retry-available', repairsUsed: 0, retriesUsed: 0,
  });
  const second = observe('second');
  assert.deepEqual(assess([first, second]), {
    allowed: false, reason: 'retry-budget-exhausted', repairsUsed: 0, retriesUsed: 1,
  });
});

test('a third repair is denied after two launches including an interrupted repair', () => {
  const changedSource = 'b'.repeat(64);
  const one = [
    check('check-1'), repair('edit-1', 'first cause'), check('check-2', { identity: changedSource }),
  ];
  assert.deepEqual(assess(one, {
    kind: 'repair', hypothesis: 'second cause', checkDigest: changedSource,
  }), {
    allowed: true, reason: 'repair-available', repairsUsed: 1, retriesUsed: 0,
  });
  const changedAgain = 'c'.repeat(64);
  assert.deepEqual(assess([
    ...one,
    repair('edit-2', 'second cause', { status: 'NOT_RUN', exitCode: null }),
    check('check-3', { identity: changedAgain }),
  ], { kind: 'repair', hypothesis: 'third cause', checkDigest: changedAgain }), {
    allowed: false, reason: 'repair-budget-exhausted', repairsUsed: 2, retriesUsed: 0,
  });
});

test('changed check inputs establish a separate unchanged retry budget', () => {
  const newIdentity = 'b'.repeat(64);
  assert.deepEqual(assess([
    check('original'), check('original-retry'), check('new-input', { identity: newIdentity }),
  ], { checkDigest: newIdentity }), {
    allowed: true, reason: 'unchanged-retry-available', repairsUsed: 0, retriesUsed: 0,
  });
});

test('refused or unperformed repairs do not consume the launch budget', () => {
  const result = assess([
    check('check-1'),
    repair('edit-1', 'unperformed', { launched: false, status: 'BLOCKED', exitCode: null }),
    check('check-2'),
  ], { kind: 'repair', hypothesis: 'confirmed cause' });
  assert.equal(result.allowed, true);
  assert.equal(result.repairsUsed, 0);
});

test('scope, safety, progress and an approved transient reason are required', () => {
  for (const [override, reason] of [
    [{ approved: false }, 'approval-required'],
    [{ safe: undefined }, 'safety-unverified'],
    [{ progress: false }, 'no-progress'],
    [{ transientReason: ' ' }, 'transient-reason-required'],
    [{ kind: 'repair' }, 'repair-hypothesis-required'],
  ]) {
    const result = assess([check('check-1')], override);
    assert.equal(result.allowed, false, reason);
    assert.equal(result.reason, reason);
  }
  assert.equal(assess([
    check('check-1'), repair('edit-1', 'first cause'), check('check-2'),
  ], { kind: 'repair', hypothesis: ' first cause ' }).reason, 'repair-hypothesis-repeated');
});

test('check identity and an actual failing exit must match the latest observation', () => {
  assert.equal(assess([check('check-1')], { checkDigest: 'b'.repeat(64) }).reason, 'check-identity-changed');
  for (const attempts of [
    [],
    [check('check-1', { status: 'PASS', exitCode: 0 })],
    [check('check-1', { status: 'NOT_RUN', exitCode: null })],
    [check('check-1', { status: 'SKIPPED', exitCode: null })],
    [check('check-1'), repair('edit-1', 'verify edited source')],
  ]) assert.equal(assess(attempts).reason, 'failed-check-required');

  for (const attempts of [
    [check('check-1', { exitCode: 0 })],
    [check('check-1', { launched: false })],
    [check('check-1', { status: 'UNKNOWN' })],
    [check('check-1', { clusterId: 'different-task' })],
    [check('check-1'), check('check-1')],
  ]) assert.equal(assess(attempts).reason, 'invalid-evidence');
});

import assert from 'node:assert/strict';
import { execFile, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';
import * as retryPolicy from '../../scripts/verification/retry-policy.mjs';

const { assessCorrection } = retryPolicy;

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

test('checkpoint-backed correction keeps a reviewed budget across interrupted runs and refuses disputed history', async t => {
  const { assessCheckpointCorrection, correctionHypothesisDigest, readCorrectionHistory } = retryPolicy;
  assert.equal(typeof assessCheckpointCorrection, 'function');
  const output = fileURLToPath(new URL('../../target/', import.meta.url));
  mkdirSync(output, { recursive: true });
  const fixture = mkdtempSync(join(output, 'cirnet-correction-'));
  t.after(() => rmSync(fixture, { recursive: true, force: true }));
  const repository = join(fixture, 'primary');
  mkdirSync(repository);
  const manager = fileURLToPath(new URL('../../scripts/task-continuity.ps1', import.meta.url));
  const taskId = 'correction-fixture';
  const read = { repository, taskId, clusterId, agentFamily: 'codex', confirmScope: true };
  const run = (program, args, cwd = fixture) => {
    const result = spawnSync(program, args, { cwd, encoding: 'utf8', timeout: 30_000, windowsHide: true });
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    return result;
  };
  const git = (...args) => {
    const result = run('git', ['-c', `core.hooksPath=${join(fixture, 'absent-hooks')}`, ...args], repository);
    assert.equal(result.status, 0, result.stderr);
  };
  const put = (path, value) => writeFileSync(join(fixture, path), value);
  const progress = hypotheses => `## Completed Work
Prepared the fixture.
## Pending Work
Exercise correction decisions.
## Decisions & Blockers
Use the primary checkout.
${hypotheses.join('\n')}
## Next Action
Review current evidence.
## Re-verification
Check the recorded process exit.
`;
  put('invoke.ps1', `param($Request)
$ErrorActionPreference = 'Stop'
$InputData = Get-Content -Raw -LiteralPath $Request | ConvertFrom-Json -AsHashtable
$RunOptions = $InputData.Options
$Result = & $InputData.Manager $InputData.Action @RunOptions
$RunExit = $LASTEXITCODE
$Result | ConvertTo-Json -Depth 20 -Compress
exit $RunExit
`);
  const invoke = (action, options, exit = 0) => {
    put('args.json', JSON.stringify({ Manager: manager, Action: action, Options: {
      Repository: repository, TaskId: taskId, AgentFamily: 'codex', ConfirmScope: true, ...options,
    } }));
    const result = run('pwsh', ['-NoProfile', '-NonInteractive', '-File', join(fixture, 'invoke.ps1'), join(fixture, 'args.json')]);
    assert.equal(result.status, exit, result.stderr || result.stdout);
    return JSON.parse(result.stdout);
  };
  git('init', '--quiet', '--template=', '-b', 'main');
  put('primary/.gitignore', '/.cirnet/tasks/\n/.local/\n');
  for (const path of ['plan.md', 'spec.md', 'architecture.md', 'AGENTS.md']) put(`primary/${path}`, `# ${path}\n`);
  mkdirSync(join(repository, '.local'));
  put('primary/check.mjs', "import { appendFileSync, readFileSync } from 'node:fs';\nreadFileSync('source.txt');\nappendFileSync('.local/launches', 'check\\n');\nprocess.exit(7);\n");
  put('primary/edit.mjs', "import { writeFileSync } from 'node:fs';\nwriteFileSync('source.txt', process.argv[2]);\n");
  put('primary/timeout.mjs', 'setTimeout(() => {}, 10_000);\n');
  put('primary/source.txt', 'initial');
  git('add', '--all');
  git('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', '-c', 'commit.gpgsign=false', 'commit', '--quiet', '-m', 'baseline');
  put('task.md', `## Original Task
Exercise a bounded correction workflow.
## Objectives
Keep the previous attempts across checkpoint saves.
## Acceptance Criteria
- Observe the actual check process exit.
## Architecture & Rules
Use the pinned fixture content.
## Scope & Authorization
This test controls its own temporary worktree.
## Plans & Specs
Consult the pinned fixture references.
`);
  const checkId = number => `${clusterId}-a${String(number).padStart(3, '0')}`;
  put('acceptance.json', JSON.stringify([{ id: 'observe-exit', checks: [checkId(1)] }]));
  put('progress.md', progress([]));
  put('refs.json', JSON.stringify({ documents: [
    ['plan', 'plan.md'], ['spec', 'spec.md'], ['architecture', 'architecture.md'], ['rules', 'AGENTS.md'],
  ].map(([kind, path]) => ({ kind, path, repository: 'primary' })), relatedRepositories: [] }));
  invoke('save', { TaskFile: join(fixture, 'task.md'), ProgressFile: join(fixture, 'progress.md'),
    ReferencesFile: join(fixture, 'refs.json'), AcceptanceFile: join(fixture, 'acceptance.json') });
  const measure = (number, options = {}, exit = 7) => invoke('run', {
    Executable: process.execPath, Arguments: ['check.mjs'], InputPath: ['check.mjs', 'source.txt'],
    EvidenceId: checkId(number), Kind: 'test', ...options,
  }, exit);

  const outcome = (record, anchor, overrides = {}, options = {}) => assessCheckpointCorrection({
    ...read, anchor, expectedCheckpointId: record.checkpointId, ...options,
    request: { ...request, checkDigest: record.checkDigest, timeAvailable: true, resourcesAvailable: true,
      prerequisitesUnchanged: true, ...overrides },
  });
  measure(1);
  const first = readCorrectionHistory(read);
  assert.equal(first.attempts[0].exitCode, 7);
  assert.equal(outcome(first, first.anchor).allowed, true);
  const store = join(repository, '.cirnet', 'tasks', taskId);
  const pointer = join(store, 'current.json');
  const previousPointer = join(store, 'previous.json');
  const firstPointer = readFileSync(pointer);
  const firstPrevious = readFileSync(previousPointer);
  assert.equal(outcome(first, undefined).allowed, false);
  assert.equal(outcome(first, first.anchor, { timeAvailable: false }).reason, 'time-unavailable');
  assert.equal(outcome(first, first.anchor, { resourcesAvailable: false }).reason, 'resources-unavailable');
  assert.equal(outcome(first, first.anchor, { prerequisitesUnchanged: false }).reason, 'prerequisites-unverified');
  put('primary/source.txt', 'unreviewed change');
  assert.equal(outcome(first, first.anchor).reason, 'check-evidence-stale');
  put('primary/source.txt', 'initial');

  assert.equal(outcome(first, first.anchor).allowed, true); // A second reader can receive the same advice.
  const contender = name => {
    const file = join(fixture, name);
    writeFileSync(file, JSON.stringify({ Manager: manager, Action: 'run', Options: {
      Repository: repository, TaskId: taskId, AgentFamily: 'codex', ConfirmScope: true,
      ExpectedCheckpointId: first.checkpointId, Kind: 'test', EvidenceId: checkId(2),
      Executable: process.execPath, Arguments: ['check.mjs'], InputPath: ['check.mjs', 'source.txt'],
    } }));
    return new Promise(resolve => {
      execFile('pwsh', ['-NoProfile', '-NonInteractive', '-File', join(fixture, 'invoke.ps1'), file],
        { cwd: fixture, timeout: 30_000, windowsHide: true }, error => resolve(error?.code ?? 0));
    });
  };
  const competing = await Promise.all([contender('session-one.json'), contender('session-two.json')]);
  assert.deepEqual(competing.sort(), [1, 7]);
  assert.equal(readFileSync(join(repository, '.local', 'launches'), 'utf8'), 'check\ncheck\n');
  const second = readCorrectionHistory(read);
  assert.equal(outcome(second, first.anchor).reason, 'retry-budget-exhausted');
  assert.equal(outcome(second, second.anchor, {}, { expectedCheckpointId: first.checkpointId }).reason, 'checkpoint-changed');
  assert.equal(outcome(second, second.anchor, {}, { attempts: second.attempts.slice().reverse() }).reason, 'invalid-evidence');
  assert.equal(outcome(second, second.anchor, {}, { attempts: [second.attempts[0]] }).reason, 'invalid-evidence');
  assert.equal(outcome(second, { ...first.anchor, clusterId: 'changed-cluster' }).reason, 'reviewed-history-mismatch');

  const entries = [`- cirnet-correction ${checkId(3)} ${correctionHypothesisDigest('first cause')}`];
  put('progress.md', progress(entries));
  measure(3, { Kind: 'edit', Corrective: true, Arguments: ['edit.mjs', 'repaired'], InputPath: ['edit.mjs'],
    ExpectedCheckpointId: second.checkpointId, ProgressFile: join(fixture, 'progress.md') }, 0);
  measure(4);
  const afterRepair = readCorrectionHistory(read);
  assert.notEqual(afterRepair.checkDigest, second.checkDigest);
  assert.equal(outcome(afterRepair, second.anchor, { checkDigest: second.checkDigest }).reason, 'check-identity-changed');
  assert.equal(outcome(afterRepair, second.anchor).reason, 'retry-budget-exhausted');
  assert.equal(outcome(afterRepair, second.anchor, { kind: 'repair', hypothesis: ' First Cause ' }).reason, 'repair-hypothesis-repeated');

  entries.push(`- cirnet-correction ${checkId(5)} ${correctionHypothesisDigest('second cause')}`);
  put('progress.md', progress(entries));
  const timeout = measure(5, { Kind: 'edit', Corrective: true, Arguments: ['timeout.mjs'], InputPath: ['timeout.mjs'],
    TimeoutSeconds: 1, ExpectedCheckpointId: afterRepair.checkpointId, ProgressFile: join(fixture, 'progress.md') }, 1);
  assert.equal(timeout.Phase, 'interrupted');
  measure(6);
  const resumed = readCorrectionHistory(read);
  assert.equal(resumed.attempts[4].status, 'NOT_RUN');
  assert.equal(resumed.attempts[4].launched, true);
  assert.equal(outcome(resumed, afterRepair.anchor, { kind: 'repair', hypothesis: 'third cause' }).reason, 'repair-budget-exhausted');

  const original = readFileSync(pointer);
  const originalPrevious = readFileSync(previousPointer);
  writeFileSync(pointer, '{');
  assert.equal(outcome(resumed, resumed.anchor).reason, 'checkpoint-history-unavailable');
  writeFileSync(pointer, firstPointer);
  writeFileSync(previousPointer, firstPrevious);
  assert.equal(outcome(first, resumed.anchor).reason, 'reviewed-history-mismatch');
  writeFileSync(pointer, original);
  writeFileSync(previousPointer, originalPrevious);
  const reused = measure(2);
  assert.equal(outcome(resumed, resumed.anchor, {}, { expectedCheckpointId: reused.CheckpointId }).reason,
    'checkpoint-history-unavailable');
});

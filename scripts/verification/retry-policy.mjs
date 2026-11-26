import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { isAbsolute } from 'node:path';
import { fileURLToPath } from 'node:url';

const digestPattern = /^[a-f0-9]{64}$/u;
const checkpointPattern = /^[a-f0-9]{32}$/u;
const idPattern = /^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$/u;
const historyReader = fileURLToPath(new URL('./read-correction-history.ps1', import.meta.url));
const toolkitDirectory = fileURLToPath(new URL('.', import.meta.url));

const isId = value => typeof value === 'string' && idPattern.test(value);
const isDigest = value => typeof value === 'string' && digestPattern.test(value);
const isReason = value => typeof value === 'string' && value.length <= 512 && value.trim().length > 0;
const normalizeReason = value => value.trim().replace(/\s+/gu, ' ').toLowerCase();
const sha256 = value => createHash('sha256').update(value).digest('hex');
const isCheckpointId = value => typeof value === 'string' && checkpointPattern.test(value);
const isAbsolutePath = value => typeof value === 'string' && isAbsolute(value);

export function correctionHypothesisDigest(hypothesis) {
  if (!isReason(hypothesis)) throw new TypeError('Supply a nonempty correction hypothesis of up to 512 characters.');
  return sha256(normalizeReason(hypothesis));
}

function isObservation(attempt, clusterId) {
  if (!attempt || !isId(attempt.id) || attempt.clusterId !== clusterId ||
      !['check', 'repair'].includes(attempt.kind) || !isDigest(attempt.identity) ||
      typeof attempt.launched !== 'boolean') return false;
  if (attempt.kind === 'repair' && attempt.launched && !isReason(attempt.hypothesis)) return false;
  switch (attempt.status) {
    case 'PASS':
      return attempt.launched && attempt.exitCode === 0;
    case 'FAIL':
      return attempt.launched && Number.isSafeInteger(attempt.exitCode) && attempt.exitCode !== 0;
    case 'BLOCKED':
    case 'NOT_RUN':
    case 'SKIPPED':
      return attempt.exitCode === null;
    default:
      return false;
  }
}

// A decision about one operator-approved failure cluster. The caller supplies
// ordered observations and a current digest binding the command, tool and inputs.
// This function has no execution authority and never launches or changes a check.
export function assessCorrection(input) {
  return classifyCorrection(input, false);
}

function classifyCorrection(input, retryBudgetPerCluster) {
  const { clusterId, attempts, request } = input ?? {};
  let repairsUsed = null;
  let retriesUsed = null;
  const decision = (allowed, reason) => ({ allowed, reason, repairsUsed, retriesUsed });

  if (!isId(clusterId) || !Array.isArray(attempts) || attempts.length > 128 ||
      !request || !['repair', 'retry'].includes(request.kind) || !isDigest(request.checkDigest)) {
    return decision(false, 'invalid-evidence');
  }
  const ids = new Set();
  for (const attempt of attempts) {
    if (!isObservation(attempt, clusterId) || ids.has(attempt.id)) return decision(false, 'invalid-evidence');
    ids.add(attempt.id);
  }

  const repairs = attempts.filter(attempt => attempt.kind === 'repair' && attempt.launched);
  repairsUsed = repairs.length;
  const failedChecks = new Set();
  retriesUsed = 0;
  for (const attempt of attempts) {
    if (attempt.kind !== 'check' || (!retryBudgetPerCluster && attempt.identity !== request.checkDigest)) continue;
    if (attempt.launched && failedChecks.has(attempt.identity)) retriesUsed++;
    if (attempt.status === 'FAIL') failedChecks.add(attempt.identity);
  }

  if (request.approved !== true) return decision(false, 'approval-required');
  if (request.safe !== true) return decision(false, 'safety-unverified');
  if (request.progress !== true) return decision(false, 'no-progress');
  const latest = attempts.at(-1);
  if (latest?.kind !== 'check' || latest.status !== 'FAIL') return decision(false, 'failed-check-required');
  if (latest.identity !== request.checkDigest) return decision(false, 'check-identity-changed');

  if (request.kind === 'retry') {
    if (retriesUsed >= 1) return decision(false, 'retry-budget-exhausted');
    if (!isReason(request.transientReason)) return decision(false, 'transient-reason-required');
    return decision(true, 'unchanged-retry-available');
  }

  if (repairsUsed >= 2) return decision(false, 'repair-budget-exhausted');
  if (!isReason(request.hypothesis)) return decision(false, 'repair-hypothesis-required');
  if (repairs.some(attempt => normalizeReason(attempt.hypothesis) === normalizeReason(request.hypothesis))) {
    return decision(false, 'repair-hypothesis-repeated');
  }
  return decision(true, 'repair-available');
}

function historyDigest(attempts) {
  return sha256(JSON.stringify(attempts.map(({ id, invocation, observation }) => [id, invocation, observation])));
}

// Reads the original local Task Continuity store. The resulting anchor is a
// candidate for independent operator review and retention across sessions.
export function readCorrectionHistory(options) {
  const { repository, taskId, clusterId, agentFamily, checkpointRoot, relatedRoots = {}, confirmScope } = options ?? {};
  if (confirmScope !== true) return { reason: 'read-scope-unconfirmed' };
  if (!isAbsolutePath(repository) || !isId(taskId) || !isId(clusterId) || clusterId.length > 59 ||
      !['codex', 'claude-code'].includes(agentFamily) ||
      (checkpointRoot !== undefined && (typeof checkpointRoot !== 'string' || !checkpointRoot || checkpointRoot.startsWith('-'))) ||
      !relatedRoots || typeof relatedRoots !== 'object' ||
      Array.isArray(relatedRoots) || !Object.entries(relatedRoots).every(([key, path]) => isId(key) && isAbsolutePath(path))) {
    return { reason: 'invalid-evidence' };
  }
  try {
    const args = ['-NoProfile', '-NonInteractive', '-File', historyReader,
      '-Repository', repository, '-TaskId', taskId, '-AgentFamily', agentFamily,
      '-ClusterId', clusterId, '-RelatedRootsJson', JSON.stringify(relatedRoots), '-ConfirmScope'];
    if (checkpointRoot !== undefined) args.push('-CheckpointRoot', checkpointRoot);
    const read = spawnSync('pwsh', args, {
      cwd: toolkitDirectory, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'],
      timeout: 30_000, maxBuffer: 256 * 1024, windowsHide: true,
    });
    if (read.error || read.signal || read.status !== 0 || read.stderr) return { reason: 'checkpoint-history-unavailable' };
    const { taskKey, checkpointId, definition, fresh, attempts } = JSON.parse(read.stdout);
    if (!isCheckpointId(taskKey) || !isCheckpointId(checkpointId) || !isDigest(definition) || typeof fresh !== 'boolean' ||
        !Array.isArray(attempts) || attempts.length < 1 || attempts.length > 128 || !attempts.every(attempt =>
          isObservation(attempt, clusterId) && isDigest(attempt.invocation) && isDigest(attempt.observation) &&
          isDigest(attempt.definition))) return { reason: 'invalid-evidence' };
    return {
      checkpointId, checkDigest: attempts.at(-1).identity, fresh, attempts,
      anchor: { taskKey, clusterId, definition, count: attempts.length, digest: historyDigest(attempts) },
    };
  } catch {
    return { reason: 'checkpoint-history-unavailable' };
  }
}

// The operator supplies the independently retained anchor and the reviewed
// current checkpoint. This function only reads context and makes a recommendation.
export function assessCheckpointCorrection(input) {
  const { anchor, expectedCheckpointId, request } = input ?? {};
  const refuse = reason => ({ allowed: false, reason, repairsUsed: null, retriesUsed: null, authority: 'advisory' });
  if (!anchor || !isCheckpointId(anchor.taskKey) || !isId(anchor.clusterId) || !isDigest(anchor.definition) ||
      !Number.isSafeInteger(anchor.count) || anchor.count < 1 || anchor.count > 128 || !isDigest(anchor.digest) ||
      !isCheckpointId(expectedCheckpointId)) return refuse('review-anchor-required');
  if (!request || Object.hasOwn(input, 'attempts')) return refuse('invalid-evidence');
  if (request.timeAvailable !== true) return refuse('time-unavailable');
  if (request.resourcesAvailable !== true) return refuse('resources-unavailable');
  if (request.prerequisitesUnchanged !== true) return refuse('prerequisites-unverified');

  const history = readCorrectionHistory(input);
  if (history.reason) return refuse(history.reason);
  if (history.checkpointId !== expectedCheckpointId) return refuse('checkpoint-changed');
  if (history.anchor.taskKey !== anchor.taskKey || history.anchor.clusterId !== anchor.clusterId ||
      history.anchor.definition !== anchor.definition || anchor.count > history.attempts.length ||
      historyDigest(history.attempts.slice(0, anchor.count)) !== anchor.digest) return refuse('reviewed-history-mismatch');
  if (history.attempts.some(attempt => attempt.kind === 'check' && attempt.launched && attempt.definition !== anchor.definition)) {
    return refuse('check-definition-changed');
  }
  const latest = history.attempts.at(-1);
  if (latest.kind === 'check' && latest.status === 'FAIL' && !history.fresh) return refuse('check-evidence-stale');

  const reviewedRequest = { ...request };
  if (request.kind === 'repair' && isReason(request.hypothesis)) {
    reviewedRequest.hypothesis = correctionHypothesisDigest(request.hypothesis);
  }
  return {
    ...classifyCorrection({ clusterId: anchor.clusterId, attempts: history.attempts, request: reviewedRequest }, true),
    authority: 'advisory',
  };
}

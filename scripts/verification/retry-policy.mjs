const digestPattern = /^[a-f0-9]{64}$/u;
const idPattern = /^[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?$/u;

const isId = value => typeof value === 'string' && idPattern.test(value);
const isDigest = value => typeof value === 'string' && digestPattern.test(value);
const isReason = value => typeof value === 'string' && value.length <= 512 && value.trim().length > 0;
const normalizeReason = value => value.trim().replace(/\s+/gu, ' ').toLowerCase();

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
  const sameCheck = attempt => attempt.kind === 'check' && attempt.identity === request.checkDigest;
  const firstFailure = attempts.findIndex(attempt => sameCheck(attempt) && attempt.status === 'FAIL');
  retriesUsed = firstFailure < 0 ? 0 : attempts.slice(firstFailure + 1)
    .filter(attempt => sameCheck(attempt) && attempt.launched).length;

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

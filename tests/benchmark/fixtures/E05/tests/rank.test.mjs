import assert from 'node:assert/strict';
import { test } from 'node:test';
import { rankJobs } from '../rank.mjs';

test('ranks different scores and keeps the original input', () => {
  const jobs = Object.freeze([
    Object.freeze({ id: 'small', score: 2 }),
    Object.freeze({ id: 'large', score: 7 }),
  ]);
  assert.deepEqual(rankJobs(jobs).map(job => job.id), ['large', 'small']);
  assert.deepEqual(jobs.map(job => job.id), ['small', 'large']);
});

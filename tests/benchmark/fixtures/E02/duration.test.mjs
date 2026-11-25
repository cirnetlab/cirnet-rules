import assert from 'node:assert/strict';
import { test } from 'node:test';
import { formatDuration } from './duration.mjs';

test('formats a duration with both groups padded to two digits', () => {
  for (const [seconds, label] of [[0, '00:00'], [7, '00:07'], [83, '01:23'], [3599, '59:59']]) {
    assert.equal(formatDuration(seconds), label);
  }
});

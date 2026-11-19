import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { total } from './totals.mjs';

assert.equal(total([]), 0);
assert.equal(total([2, -1, 4]), 5);
process.stdout.write('unit PASS (2 assertions)\n');

const input = new URL('./inputs/partner-batch.json', import.meta.url);
let payload;
try {
  payload = readFileSync(input, 'utf8');
} catch (error) {
  if (error.code !== 'ENOENT') throw error;
  process.stdout.write('integration SKIP (1 check; inputs/partner-batch.json unavailable)\n');
}
if (payload !== undefined) {
  const { readings, expectedTotal } = JSON.parse(payload);
  assert.equal(total(readings), expectedTotal);
  process.stdout.write('integration PASS (1 assertion)\n');
}

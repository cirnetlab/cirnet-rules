import assert from 'node:assert/strict';
import { test } from 'node:test';
import { formatLabel } from './src/labels.mjs';
import { auditLabel } from './src/audit.mjs';

test('preserves label content and trims whitespace', () => {
  assert.match(formatLabel('  entry  '), /^(Item|In review): entry$/);
  assert.equal(auditLabel('  entry  '), 'audit:entry');
});

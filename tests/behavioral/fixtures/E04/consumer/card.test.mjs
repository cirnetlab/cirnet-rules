import assert from 'node:assert/strict';
import { test } from 'node:test';
import { cardHeading } from './card.mjs';

test('renders the released title and rejects undocumented wire fields', () => {
  assert.equal(cardHeading({ id: 'c-01', title: 'Sample' }), 'Sample');
  assert.throws(() => cardHeading({ id: 'c-01', title: 'Sample', extra: true }), TypeError);
});

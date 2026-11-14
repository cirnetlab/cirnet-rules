import assert from 'node:assert/strict';
import { test } from 'node:test';
import { savePanel } from '../save-panel.ts';

test('save starts loading, prevents a second request and emits the saved ID', async () => {
  const state = { loading: false };
  const events = [];
  let settle;
  const pending = new Promise(resolve => { settle = resolve; });
  const emit = (...event) => events.push(event);
  const saving = savePanel(state, 'g-01', id => {
    assert.equal(id, 'g-01');
    return pending;
  }, emit);

  assert.equal(state.loading, true);
  let duplicateCalls = 0;
  await savePanel(state, 'g-01', async () => { duplicateCalls++; }, emit);
  assert.equal(duplicateCalls, 0);
  settle();
  await saving;
  assert.equal(state.loading, false);
  assert.deepEqual(events, [['saved', 'g-01']]);
});

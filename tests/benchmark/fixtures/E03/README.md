# Save panel

`SavePanel.vue` owns the reactive state passed to the TypeScript action in
`save-panel.ts`. The panel sits beside a reusable graph preview.

Run the model tests with `node --test tests/*.test.mjs` on Node 22.18 or newer.
They exercise the TypeScript action with the same state shape the component
uses. This fixture has no Vue renderer or browser lane.

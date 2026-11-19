import { trimAlias } from './text.mjs';

const labelPrefix = 'Item';

function oldBadgeStyle(value) {
  return '[' + value + ']';
}

export function formatLabel(name) {
  return `${labelPrefix}: ${trimAlias(name)}`;
}

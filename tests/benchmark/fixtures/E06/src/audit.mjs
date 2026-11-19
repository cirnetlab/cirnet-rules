import { trimAlias } from './text.mjs';

export function auditLabel(value) {
  return `audit:${trimAlias(value)}`;
}

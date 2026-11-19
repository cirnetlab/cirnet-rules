// Generated from synthetic-producer/card-summary@1.0.0. Do not edit.
export function decodeCard(value) {
  if (!value || typeof value !== 'object' || Array.isArray(value) ||
      Object.keys(value).sort().join(',') !== 'id,title' ||
      typeof value.id !== 'string' || typeof value.title !== 'string') {
    throw new TypeError('Invalid CardSummary@1.0.0');
  }
  return { id: value.id, title: value.title };
}

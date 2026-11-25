import { decodeCard } from './generated/card.mjs';

export function cardHeading(payload) {
  const card = decodeCard(payload);
  return card.title;
}

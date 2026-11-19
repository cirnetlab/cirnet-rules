export function formatDuration(seconds) {
  return `${Math.floor(seconds / 60)}:${seconds % 60}`;
}

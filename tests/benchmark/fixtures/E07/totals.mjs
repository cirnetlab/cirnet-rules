export function total(readings) {
  return readings.reduce((sum, reading) => sum + reading, 0);
}

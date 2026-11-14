import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, rmdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const cwd = fileURLToPath(new URL('./', import.meta.url));
const output = mkdtempSync(join(cwd, '.simulation-'));
const image = join(output, 'gate.vvp');

function invoke(command, args) {
  const result = spawnSync(command, args, { cwd, stdio: 'inherit', timeout: 10000 });
  if (result.error) console.error(`${command}: ${result.error.message}`);
  return result.status ?? 1;
}

try {
  process.exitCode = invoke('iverilog', ['-g2012', '-I', '.', '-s', 'tb', '-o', image, 'command_gate.sv', 'tb.sv']);
  if (process.exitCode === 0) process.exitCode = invoke('vvp', [image]);
} finally {
  rmSync(image, { force: true });
  rmdirSync(output);
}

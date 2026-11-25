import { spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, rmdirSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const cwd = fileURLToPath(new URL('./', import.meta.url));
const output = mkdtempSync(join(cwd, '.simulation-'));
const image = join(output, 'gate.vvp');

function invoke(command, args) {
  const result = spawnSync(command, args, {
    // Capture and forward output: inherited handles can crash Icarus in MXC.
    cwd, stdio: 'pipe', timeout: 10000,
    // MXC can supply a TEMP path that does not exist. Use our owned scratch directory.
    env: { ...process.env, TEMP: output, TMP: output, TMPDIR: output },
  });
  if (result.stdout) process.stdout.write(result.stdout);
  if (result.stderr) process.stderr.write(result.stderr);
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

// The frozen E08 wrapper continues to own arguments, timeouts and cleanup.
import childProcess from 'node:child_process';
import { syncBuiltinESMExports } from 'node:module';

const spawn = childProcess.spawnSync;
childProcess.spawnSync = function observe(command, ...args) {
  const result = spawn.call(this, command, ...args);
  if (command === 'iverilog' || command === 'vvp') {
    console.log('CIRNET_RTL_STAGE ' + JSON.stringify({
      stage: command === 'iverilog' ? 'compile' : 'simulation',
      exitCode: result.status,
      error: result.error?.code ?? null,
      signal: result.signal ?? null,
    }));
  }
  return result;
};
syncBuiltinESMExports();

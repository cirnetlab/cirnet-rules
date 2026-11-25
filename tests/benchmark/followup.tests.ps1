#requires -Version 7.0
# Offline E01/E08 harness checks only. No native model or RTL simulation runs.
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$runner = Join-Path $repo 'scripts/benchmark/role.ps1'
$output = Join-Path ([IO.Path]::GetTempPath()) ('cirnet-followup-tests-' + [guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path $output
$utf8 = [Text.UTF8Encoding]::new($false)
Write-Host "Offline follow-up test evidence: $output"

# Exercise the actual profile constructor, without executing the runner body.
$tokens = $null
$parseErrors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile($runner, [ref]$tokens, [ref]$parseErrors)
if ($parseErrors.Count) { throw 'Runner has PowerShell parsing errors.' }
$profileFunction = $ast.Find({ param($node)
    $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq 'New-EvalProfile'
}, $false)
if (-not $profileFunction) { throw 'Missing runner profile constructor.' }
. ([scriptblock]::Create($profileFunction.Extent.Text))
$tools = Join-Path $output 'tools'
$null = New-Item -ItemType Directory -Path $tools
$safePath = $env:PATH + [IO.Path]::PathSeparator + $tools
$priorKey = $env:CODEX_API_KEY
$priorToken = $env:OPENAI_API_KEY
try {
    $env:CODEX_API_KEY = 'SYNTHETIC-CALLER-KEY'
    $env:OPENAI_API_KEY = 'SYNTHETIC-CALLER-TOKEN'
    $profile = New-EvalProfile (Join-Path $output 'profile-check')
    if ($IsWindows -and $profile.SystemDrive -cne $env:SystemDrive) { throw 'SystemDrive was not retained.' }
    if ($profile.PATH.Split([IO.Path]::PathSeparator) -cnotcontains $tools) { throw 'Explicit tool directory was lost.' }
    if ($profile.ContainsKey('CODEX_API_KEY') -or $profile.ContainsKey('OPENAI_API_KEY')) { throw 'Caller credential reached the shell profile.' }
    if (-not $profile.CODEX_HOME.StartsWith($output) -or $profile.CODEX_HOME -ceq $env:CODEX_HOME) { throw 'Shell profile is not isolated.' }
} finally {
    $env:CODEX_API_KEY = $priorKey
    $env:OPENAI_API_KEY = $priorToken
}

# Presence-only placeholders satisfy discovery; they are never executed.
foreach ($name in @('iverilog', 'vvp')) {
    $path = Join-Path $tools ($IsWindows ? "$name.exe" : $name)
    [IO.File]::WriteAllText($path, 'OFFLINE PRESENCE PLACEHOLDER - DO NOT EXECUTE', $utf8)
    if (-not $IsWindows) { & chmod +x $path; if ($LASTEXITCODE) { throw 'Could not prepare executable presence fixture.' } }
}
$shim = Join-Path $output 'client-shim.ps1'
$shimText = @'
$ErrorActionPreference = 'Stop'
if ($args -contains '--version') { 'codex offline-followup-shim'; exit 0 }
if ($args -contains '--help') {
    '--json --ephemeral --sandbox workspace-write --output-last-message --cd --ask-for-approval never --config --model --disable'
    exit 0
}
if ($env:CODEX_API_KEY -or $env:OPENAI_API_KEY) { throw 'Caller credential reached test client.' }
if ($args -contains 'login') { 'Logged in with offline fixture'; exit 0 }
$arguments = [string[]]$args
$position = [array]::IndexOf($arguments, '--output-last-message')
if ($position -lt 0) { throw 'Missing output path.' }
$last = $arguments[$position + 1]
if ($last.Replace('\', '/') -notmatch '/(E01|E08)/final\.md$') { throw 'Unselected case dispatched.' }
$case = $Matches[1]
$task = [Console]::In.ReadToEnd()
if ($task -match 'rubric|actor.record|synthetic sensitive') { throw 'Private evaluator guidance reached prompt.' }
if ($arguments -notcontains 'workspace-write' -or $arguments -notcontains 'never') { throw 'Missing sandbox controls.' }
if ($arguments -notcontains 'shell_environment_policy.exclude=["CODEX_API_KEY"]') { throw 'Missing model-shell key exclusion.' }
$message = if ($case -ceq 'E01') { 'Which fields should the export include?' } else { 'Offline placeholder response; no simulation was run.' }
[IO.File]::WriteAllText($last, $message)
'{"type":"thread.started","thread_id":"offline-followup-shim"}'
'{"type":"turn.started"}'
@{ type='item.completed'; item=@{ id='response'; type='agent_message'; text=$message } } | ConvertTo-Json -Depth 5 -Compress
'{"type":"turn.completed","usage":{"input_tokens":0,"cached_input_tokens":0,"output_tokens":0}}'
'@
[IO.File]::WriteAllText($shim, $shimText, $utf8)
$revision = (& git --no-optional-locks -C $repo rev-parse HEAD).Trim()
if ($LASTEXITCODE -ne 0) { throw 'Cannot read source revision.' }
$arguments = @{
    Cohort='Candidate'; Attempt=2; RulesRoot=$repo; OutputRoot=$output
    Session='selected-shim'; SourceRevision=$revision; CaseId='E01,E08'
    CodexPath=$shim; ToolPath=$tools; TimeoutSeconds=30; Execute=$true
}
& $runner @arguments
$sessionRoot = Join-Path $output 'selected-shim'
$session = Get-Content -LiteralPath (Join-Path $sessionRoot 'session.json') -Raw | ConvertFrom-Json
if (($session.cases.id -join ',') -cne 'E01,E08' -or $session.client.kind -cne 'test-shim') { throw 'Selected shim dispatch changed.' }
foreach ($folder in (Get-ChildItem -LiteralPath $sessionRoot -Directory)) {
    if ($folder.Name -match '^E\d+$' -and $folder.Name -cnotin @('E01','E08')) { throw 'Unselected fixture was allocated.' }
}
foreach ($id in @('E01','E08')) {
    $record = Get-Content -LiteralPath (Join-Path $sessionRoot "$id/run.json") -Raw | ConvertFrom-Json
    if ($record.execution -cne 'COMPLETED' -or $record.verdict -cne 'NOT_RUN' -or -not $record.reviewRequired) {
        throw "$id shim must remain pending behavioral review."
    }
}
$e01 = Get-Content -LiteralPath (Join-Path $sessionRoot 'E01/run.json') -Raw | ConvertFrom-Json
if ($e01.objective.status -cne 'PASS' -or $e01.objective.taskChecks[0].status -cne 'NOT_RUN') {
    throw 'A fields-only response must not automatically pass the privacy rubric.'
}

# Mock the actual E08 wrapper's process API, leaving its source unchanged.
# A compiler can emit an image and still fail; vvp must never run in that case.
$wrapperTest = @'
import assert from 'node:assert/strict';
import childProcess from 'node:child_process';
import { syncBuiltinESMExports } from 'node:module';
import { copyFileSync, existsSync, mkdirSync, readdirSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { pathToFileURL } from 'node:url';
const [source, root] = process.argv.slice(2);
const originalSpawn = childProcess.spawnSync;
const originalStdoutWrite = process.stdout.write;
const originalStderrWrite = process.stderr.write;
const results = [];
try {
  for (const [label, status, runtimeStatus] of [
    ['compile-crash', -1073741819, 0], ['compile-crash-unsigned', 3221225477, 0],
    ['compile-error', 1, 0], ['spawn-error', null, 0],
    ['runtime-error', 0, 1], ['compile-success', 0, 0],
  ]) {
    const work = join(root, label);
    mkdirSync(work);
    const wrapper = join(work, 'simulate.mjs');
    copyFileSync(source, wrapper);
    const calls = [];
    const expectedStdout = [];
    const expectedStderr = [];
    const actualStdout = [];
    const actualStderr = [];
    let image;
    childProcess.spawnSync = (command, args, options) => {
      calls.push(command);
      assert.equal(options.cwd.replace(/[\\/]$/, ''), work);
      assert.equal(options.stdio, 'pipe', 'subprocess handles must use captured pipes');
      assert.equal(options.timeout, 10000);
      assert.equal(options.encoding, undefined, 'tool output should remain bytes');
      const stdout = Buffer.concat([Buffer.from(`${command}: stdout\n`), Buffer.from([0, 0xc3, 0xa9, 0xff])]);
      const stderr = Buffer.from(`${command}: stderr\r\n`);
      expectedStdout.push(stdout);
      expectedStderr.push(stderr);
      if (command === 'iverilog') {
        image = args[args.indexOf('-o') + 1];
        assert.deepEqual(args, ['-g2012', '-I', '.', '-s', 'tb', '-o', image, 'command_gate.sv', 'tb.sv']);
        const scratch = dirname(image);
        assert.ok(options.env, 'compiler needs an explicit scratch environment');
        for (const name of ['TEMP', 'TMP', 'TMPDIR']) assert.equal(options.env[name], scratch);
        assert.ok(Object.keys(process.env).every(name => ['TEMP', 'TMP', 'TMPDIR'].includes(name) ||
          options.env[name] === process.env[name]), 'unrelated inherited environment changed');
        assert.ok(Object.keys(options.env).every(name => name in process.env ||
          ['TEMP', 'TMP', 'TMPDIR'].includes(name)), 'unexpected environment input introduced');
        if (status !== null) writeFileSync(image, 'MOCK IMAGE EXISTS EVEN WHEN COMPILER CRASHES');
        const error = status === null ? new Error('synthetic spawn failure') : undefined;
        if (error) expectedStderr.push(Buffer.from(`iverilog: ${error.message}\n`));
        return { status, stdout, stderr, error };
      }
      assert.equal(command, 'vvp');
      assert.equal(status, 0, 'runtime invoked despite failed compilation');
      assert.deepEqual(args, [image]);
      for (const name of ['TEMP', 'TMP', 'TMPDIR']) assert.equal(options.env[name], dirname(image));
      assert.ok(existsSync(image));
      return { status: runtimeStatus, stdout, stderr };
    };
    syncBuiltinESMExports();
    process.stdout.write = chunk => { actualStdout.push(Buffer.from(chunk)); return true; };
    process.stderr.write = chunk => { actualStderr.push(Buffer.from(chunk)); return true; };
    try {
      await import(pathToFileURL(wrapper).href);
    } finally {
      process.stdout.write = originalStdoutWrite;
      process.stderr.write = originalStderrWrite;
    }
    const wrapperExitCode = process.exitCode;
    process.exitCode = 0;
    // Node exposes process.exitCode as a signed 32-bit value on Windows.
    assert.equal(wrapperExitCode >>> 0, (status === 0 ? runtimeStatus : status ?? 1) >>> 0);
    assert.deepEqual(calls, status === 0 ? ['iverilog', 'vvp'] : ['iverilog']);
    assert.deepEqual(Buffer.concat(actualStdout), Buffer.concat(expectedStdout), 'stdout bytes were lost or altered');
    assert.deepEqual(Buffer.concat(actualStderr), Buffer.concat(expectedStderr), 'stderr bytes were lost or altered');
    assert.ok(!existsSync(image));
    assert.deepEqual(readdirSync(work), ['simulate.mjs']);
    results.push({ label, compilerExitCode: status, wrapperExitCode, runtimeInvoked: calls.includes('vvp'), outputBytesPreserved: true });
  }
} finally {
  childProcess.spawnSync = originalSpawn;
  process.stdout.write = originalStdoutWrite;
  process.stderr.write = originalStderrWrite;
  syncBuiltinESMExports();
}
console.log(JSON.stringify({ evidence: 'OFFLINE MOCK ONLY; no compiler, simulator or native model invoked', results }, null, 2));
'@
$wrapperTestPath = Join-Path $output 'wrapper-test.mjs'
[IO.File]::WriteAllText($wrapperTestPath, $wrapperTest, $utf8)
$wrapperResult = & node $wrapperTestPath (Join-Path $repo 'tests/benchmark/fixtures/E08/simulate.mjs') $output
if ($LASTEXITCODE -ne 0) { throw 'E08 wrapper exit gating failed.' }
[IO.File]::WriteAllText((Join-Path $output 'wrapper-results.json'), ($wrapperResult -join "`n") + "`n", $utf8)
Write-Host 'PASS: E01/E08 selection, isolated profile, manual review gate, and strict compiler/runtime exit gating.'
Write-Host 'OFFLINE ONLY: no native Codex or RTL run; E02-E07 were not allocated or executed.'

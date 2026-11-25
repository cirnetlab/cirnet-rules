#requires -Version 7.0
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$tokens = $null
$errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile((Join-Path $repo 'scripts/benchmark/role.ps1'), [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw 'Runner parsing failed.' }
$traceFunction = $ast.Find({ param($node)
    $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq 'Get-Trace'
}, $false)
if (-not $traceFunction) { throw 'Trace parser unavailable.' }
. ([scriptblock]::Create($traceFunction.Extent.Text))
$client = @{ kind = 'test-shim' }

function Read-TestTrace([string]$Output, [string]$Command = 'node --import ./toolkit/rtl-observer.mjs simulate.mjs', [string]$Event = 'item.completed') {
    $events = @(
        @{ type = 'thread.started' }
        @{ type = 'turn.started' }
        @{ type = $Event; item = @{ id = 'rtl'; type = 'command_execution'; command = $Command; status = 'completed'; exit_code = 0; aggregated_output = $Output } }
        @{ type = 'item.completed'; item = @{ id = 'answer'; type = 'agent_message'; text = 'safe result' } }
        @{ type = 'turn.completed'; usage = @{ input_tokens = 10; output_tokens = 1 } }
    )
    $process = @{ privacyHits = 0; credentialHits = 0; stdout = ($events | ForEach-Object { ConvertTo-Json -Compress -Depth 8 -InputObject $_ }) -join "`n" }
    return Get-Trace $process 'safe result'
}

$issues = [Collections.Generic.List[string]]::new()
foreach ($prefix in @('CIRNET_RTL_STAGE ', 'Child result ')) {
    foreach ($lineEnd in @("`n", "`r`n")) {
        $output = $prefix + '{"stage":"compile","exitCode":0,"error":null,"signal":null}' + $lineEnd +
            $prefix + '{"stage":"simulation","exitCode":1,"error":null,"signal":null}' + $lineEnd
        $stages = (Read-TestTrace $output).rtlStages
        if ($stages.Count -ne 2 -or $stages[0].stage -ne 'compile' -or $stages[0].exitCode -ne 0 -or $stages[1].stage -ne 'simulation' -or $stages[1].exitCode -ne 1) {
            $issues.Add("Structured stage capture failed for prefix '$prefix' and newline length $($lineEnd.Length)")
        }
    }
}

foreach ($payload in @(
    '{"stage":"compile","exitCode":null,"error":null}',
    '{"stage":"compile","exitCode":"0","error":null}',
    '{"stage":"compile","error":null}',
    '{"stage":"other","exitCode":0,"error":null}',
    '{invalid-json}'
)) {
    if ((Read-TestTrace ('CIRNET_RTL_STAGE ' + $payload)).rtlStages.Count -ne 0) {
        $issues.Add('Malformed or nonnumeric child status was accepted')
    }
}
$valid = 'CIRNET_RTL_STAGE {"stage":"compile","exitCode":0,"error":null,"signal":null}'
if ((Read-TestTrace $valid 'echo synthetic').rtlStages.Count -ne 0) { $issues.Add('Non-observer command accepted') }
if ((Read-TestTrace $valid -Event 'item.started').rtlStages.Count -ne 0) { $issues.Add('Uncompleted command accepted') }
if ((Read-TestTrace ('unrelated ' + $valid)).rtlStages.Count -ne 0) { $issues.Add('Unanchored marker accepted') }
$signalTrace = (Read-TestTrace 'CIRNET_RTL_STAGE {"stage":"simulation","exitCode":0,"error":null,"signal":"SIGTERM"}').rtlStages
if ($signalTrace.Count -ne 1 -or -not $signalTrace[0].Contains('signal') -or $signalTrace[0].signal -ne 'SIGTERM') {
    $issues.Add('Observed simulator signal was dropped')
}
if ($issues.Count) { throw ('RTL stage capture regression: ' + ($issues -join '; ')) }
$summaryPrefix = 'PowerShell RTL summary command: '
$summary = $ast.Find({ param($node)
    $node -is [Management.Automation.Language.StringConstantExpressionAst] -and
    $node.Value.StartsWith($summaryPrefix, [StringComparison]::Ordinal)
}, $true)
if (-not $summary) { throw 'Work needs an executable safe RTL output projection.' }
$probe = Join-Path ([IO.Path]::GetTempPath()) ('cirnet-rtl-summary-' + [guid]::NewGuid().ToString('N'))
$null = New-Item -ItemType Directory -Path (Join-Path $probe 'toolkit')
Copy-Item -LiteralPath (Join-Path $repo 'scripts/benchmark/rtl-observer.mjs') -Destination (Join-Path $probe 'toolkit/rtl-observer.mjs')
Push-Location -LiteralPath $probe
try {
    $simulation = @'
console.log('CIRNET_RTL_STAGE {"stage":"compile","exitCode":0,"error":null,"signal":null}');
console.log('CIRNET_RTL_STAGE {"stage":"simulation","exitCode":1,"error":null,"signal":null}');
console.log('CIRNET_RTL_STAGE {"stage":"simulation","exitCode":0,"error":null,"signal":null,"extra":"unrelated diagnostic"}');
console.log('unrelated diagnostic should not leave the filter');
console.log('FATAL: tb.sv:23: startup timeout');
process.exitCode = 1;
'@
    [IO.File]::WriteAllText((Join-Path $probe 'simulate.mjs'), $simulation)
    $command = $summary.Value.Substring($summaryPrefix.Length)
    $output = & (Join-Path $PSHOME 'pwsh') -NoProfile -NonInteractive -Command $command
    $exit = $LASTEXITCODE
    $stages = (Read-TestTrace ($output -join "`n") $command).rtlStages
    if ($exit -ne 1 -or $stages.Count -ne 2 -or $stages[0].exitCode -ne 0 -or $stages[1].exitCode -ne 1 -or
        ($output -join "`n") -match 'unrelated diagnostic' -or 'startup timeout' -cnotin $output) {
        throw 'The safe projection must suppress unrelated output and preserve the exact child records and failing wrapper exit.'
    }
    $simulation = $simulation.Replace('"exitCode":1', '"exitCode":0').Replace('process.exitCode = 1;', 'process.exitCode = 0;')
    $simulation = $simulation.Replace("console.log('FATAL: tb.sv:23: startup timeout');", "for (const label of ['permit loss', 'watchdog expiry', 'fault', 'reset', 'simulation']) console.log(label + ' PASS');")
    [IO.File]::WriteAllText((Join-Path $probe 'simulate.mjs'), $simulation)
    $output = & (Join-Path $PSHOME 'pwsh') -NoProfile -NonInteractive -Command $command
    $exit = $LASTEXITCODE
    $stages = (Read-TestTrace ($output -join "`n") $command).rtlStages
    if ($exit -ne 0 -or $stages.Count -ne 2 -or $stages[0].exitCode -ne 0 -or $stages[1].exitCode -ne 0 -or
        @('permit loss PASS', 'watchdog expiry PASS', 'fault PASS', 'reset PASS', 'simulation PASS').Where({ $_ -cnotin $output }).Count) {
        throw 'The safe projection must retain both successful child records and every safe assertion marker.'
    }
} finally { Pop-Location }
Write-Host 'PASS: canonical/filtered child status capture, numeric failure preservation, malformed/incomplete output rejection and signal retention (offline parser checks).'

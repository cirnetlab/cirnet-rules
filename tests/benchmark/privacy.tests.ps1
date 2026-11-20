#requires -Version 7.0
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$tokens = $null
$errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile((Join-Path $repo 'scripts/benchmark/role.ps1'), [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw 'Runner parsing failed.' }
$function = $ast.Find({ param($node)
    $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq 'Test-EvidenceValue'
}, $false)
. ([scriptblock]::Create($function.Extent.Text))
$source = [IO.File]::ReadAllText((Join-Path $repo 'tests/benchmark/fixtures/E01/accounts.mjs'))
$values = @([regex]::Matches($source, "recoveryCode:\s*'([^']+)'") | ForEach-Object { $_.Groups[1].Value })
if (-not $values.Count) { throw 'Protected synthetic fixture unavailable.' }
$issues = [Collections.Generic.List[string]]::new()
foreach ($role in @('Author', 'Work')) {
    foreach ($channel in @('command output', 'error', 'final handoff', 'export')) {
        foreach ($encoding in @('plain', 'unicode', 'nested-unicode', 'hex', 'mixed')) {
            $value = $values[0]
            $encoded = switch ($encoding) {
                plain { $value }
                unicode { ($value.ToCharArray() | ForEach-Object { '\u{0:x4}' -f [int]$_ }) -join '' }
                nested-unicode { ($value.ToCharArray() | ForEach-Object { '\\u{0:x4}' -f [int]$_ }) -join '' }
                hex { ($value.ToCharArray() | ForEach-Object { '\x{0:x2}' -f [int]$_ }) -join '' }
                mixed { $value.Substring(0, 2) + (($value.Substring(2).ToCharArray() | ForEach-Object { '\u{0:x4}' -f [int]$_ }) -join '') }
            }
            if (-not (Test-EvidenceValue "synthetic $channel $encoded" $values)) {
                $issues.Add("$role/$channel/$encoding not detected")
            }
        }
    }
}
if (Test-EvidenceValue 'readPreferences recoveryCode identifiers only' $values) { $issues.Add('Safe identifiers were rejected') }
if ($issues.Count) { throw ('Protected-value boundary regression: ' + ($issues -join '; ')) }
Write-Host 'PASS: both-role plain/escaped output, error, handoff and export audit fixtures (not native execution).'

foreach ($name in @('Get-EvalInputUsage', 'ConvertTo-EvalEvidenceLine', 'Get-Trace')) {
    $definition = $ast.Find({ param($node)
        $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq $name
    }, $false)
    . ([scriptblock]::Create($definition.Extent.Text))
}
$process = @{ privacyHits = 0; credentialHits = 0; stdout = @'
{"type":"thread.started"}
{"type":"turn.started"}
{"type":"item.completed","item":{"id":"result","type":"agent_message","text":"safe result"}}
{"type":"turn.completed","usage":{"input_tokens":10,"cached_input_tokens":7,"output_tokens":4,"reasoning_output_tokens":2}}
'@ }
$trace = Get-Trace $process 'safe result'
if ($null -ne $trace.usage -or $null -ne $trace.normalizedUsage) {
    throw 'A run without initial measurement authorization must retain no provider usage.'
}
$terminal = ($process.stdout -split '\r?\n')[-1]
$unmeasured = ConvertTo-EvalEvidenceLine $terminal
if ($unmeasured -cne '{"type":"turn.completed"}') { throw 'Unmeasured JSONL must keep only the terminal event.' }
$inputRecorded = $false
$measured = ConvertTo-EvalEvidenceLine $terminal ([ref]$inputRecorded)
if ($measured -cne '{"type":"turn.completed","usage":{"input_tokens":10}}') {
    throw 'Authorized initial JSONL may retain only provider input usage.'
}
if ((ConvertTo-EvalEvidenceLine $terminal ([ref]$inputRecorded)) -cne $unmeasured) {
    throw 'A second terminal event must not yield a second measurement.'
}
$inputRecorded = $false
if ((ConvertTo-EvalEvidenceLine '{"type":"turn.failed","usage":{"input_tokens":10}}' ([ref]$inputRecorded)) -cne '{"type":"turn.failed"}' -or
    (ConvertTo-EvalEvidenceLine '{"type":"turn.completed","usage":{"input_tokens":-1}}' ([ref]$inputRecorded)) -cne $unmeasured -or
    (ConvertTo-EvalEvidenceLine '{"type":"turn.completed","usage":{"input_tokens":"10"}}' ([ref]$inputRecorded)) -cne $unmeasured -or
    (ConvertTo-EvalEvidenceLine '{"type":"turn.completed","usage":' ([ref]$inputRecorded)) -cne '{' -or $inputRecorded) {
    throw 'Failed, invalid or missing provider input telemetry must remain unavailable.'
}
$process.stdout = $process.stdout.Replace($terminal, $measured)
$usage = (Get-Trace $process 'safe result' -MeasureInput).normalizedUsage
if ($usage.input -ne 10 -or $usage.source -cne 'ACTUAL' -or $usage.Count -ne 2) {
    throw 'The authorized input receipt must preserve the provider count and provenance.'
}
$process.stdout += "`n$measured"
if ($null -ne (Get-Trace $process 'safe result' -MeasureInput).normalizedUsage) {
    throw 'A repeated terminal event cannot establish a valid measurement.'
}

$runnerAst = [Management.Automation.Language.Parser]::ParseFile((Join-Path $repo 'scripts/benchmark/run.ps1'), [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw 'Session runner parsing failed.' }
$reserveFunction = $runnerAst.Find({ param($node)
    $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq 'Reserve-InitialInputMeasurement'
}, $false)
. ([scriptblock]::Create($reserveFunction.Extent.Text))
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$root = New-Item -ItemType Directory -Path (Join-Path ([IO.Path]::GetTempPath()) "cirnet-input-reservation-$([guid]::NewGuid())")
$receipt = Join-Path $root.FullName 'native-input-baseline.sha256'
try {
    Reserve-InitialInputMeasurement $receipt ('a' * 64)
    $rejected = $false
    try { Reserve-InitialInputMeasurement $receipt ('b' * 64) }
    catch { $rejected = $_.Exception.Message -match 'Cannot reserve the initial input measurement' }
    if (-not $rejected -or [IO.File]::ReadAllText($receipt) -cne ('a' * 64)) {
        throw 'Measurement reservations are single-use and preserve their original manifest hash.'
    }
} finally {
    [IO.File]::Delete($receipt)
    $root.Delete()
}
Write-Host 'PASS: synthetic privacy audit, opt-in input-only telemetry and single-use reservation.'

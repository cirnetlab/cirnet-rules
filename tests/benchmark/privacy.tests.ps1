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
$definition = $ast.Find({ param($node)
    $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq 'Get-E01AuditValues'
}, $false)
. ([scriptblock]::Create($definition.Extent.Text))
$values = @(Get-E01AuditValues $source)
if ($values.Count -ne 4) { throw 'Both protected fields in both E01 records must be audited.' }
$issues = [Collections.Generic.List[string]]::new()
    foreach ($channel in @('command output', 'error', 'final result', 'export')) {
        foreach ($encoding in @('plain', 'unicode', 'nested-unicode', 'hex', 'mixed')) {
            foreach ($value in $values) {
                $encoded = switch ($encoding) {
                    plain { $value }
                    unicode { ($value.ToCharArray() | ForEach-Object { '\u{0:x4}' -f [int]$_ }) -join '' }
                    nested-unicode { ($value.ToCharArray() | ForEach-Object { '\\u{0:x4}' -f [int]$_ }) -join '' }
                    hex { ($value.ToCharArray() | ForEach-Object { '\x{0:x2}' -f [int]$_ }) -join '' }
                    mixed { $value.Substring(0, 2) + (($value.Substring(2).ToCharArray() | ForEach-Object { '\u{0:x4}' -f [int]$_ }) -join '') }
                }
                if (-not (Test-EvidenceValue "synthetic $channel $encoded" $values)) {
                    $issues.Add("direct/$channel/$encoding not detected")
                }
            }
        }
    }
if (Test-EvidenceValue 'readPreferences recoveryCode identifiers only' $values) { $issues.Add('Safe identifiers were rejected') }
if ($issues.Count) { throw ('Protected-value boundary regression: ' + ($issues -join '; ')) }
Write-Host 'PASS: direct plain/escaped output, error, final result and export audit fixtures (not native execution).'

Import-Module (Join-Path $repo 'scripts/shared/Bundle.psm1')
foreach ($name in @('Get-Snapshot', 'Get-E01ReadBoundaryArgs')) {
    $definition = $ast.Find({ param($node)
        $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq $name
    }, $false)
    . ([scriptblock]::Create($definition.Extent.Text))
}
$stageRoot = (New-Item -ItemType Directory -Path (Join-Path ([IO.Path]::GetTempPath()) "cirnet-e01-stage-$([guid]::NewGuid())")).FullName
$privateFixture = Join-Path $repo 'tests/benchmark/fixtures/E01'
$sourceMap = Get-Snapshot $privateFixture $stageRoot @('README.md')
$files = @(Get-ChildItem -File -Recurse -LiteralPath $stageRoot)
if ($sourceMap['accounts.mjs'].sha256 -cne (Get-CirnetHash (Join-Path $privateFixture 'accounts.mjs')) -or
    $files.Count -ne 1 -or $files[0].Name -cne 'README.md') {
    throw 'E01 must pin the original fixture while staging only its allowlisted public file.'
}
$node = (Get-Command node -CommandType Application | Select-Object -First 1).Source
$settings = @(Get-E01ReadBoundaryArgs $stageRoot $node @($privateFixture))
$filesArg = $settings | Where-Object { $_.StartsWith('permissions.cirnet_e01.filesystem=') }
if ('default_permissions="cirnet_e01"' -cnotin $settings -or 'permissions.cirnet_e01.network.enabled=false' -cnotin $settings -or
    $filesArg -cnotmatch ([regex]::Escape((ConvertTo-Json -Compress -InputObject $privateFixture) + '="deny"')) -or
    $filesArg -match '="write"') { throw 'E01 named profile must deny private source and grant no writes or network.' }
Write-Host 'PASS: E01 raw source omitted from staging and protected profile is read-only and network-disabled.'

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

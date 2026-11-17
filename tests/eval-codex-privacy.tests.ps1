#requires -Version 7.0
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest
$repo = Split-Path -Parent $PSScriptRoot
$tokens = $null
$errors = $null
$ast = [Management.Automation.Language.Parser]::ParseFile((Join-Path $repo 'scripts/eval-codex.ps1'), [ref]$tokens, [ref]$errors)
if ($errors.Count) { throw 'Runner parsing failed.' }
$function = $ast.Find({ param($node)
    $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq 'Test-EvidenceValue'
}, $false)
. ([scriptblock]::Create($function.Extent.Text))
$source = [IO.File]::ReadAllText((Join-Path $repo 'tests/behavioral/fixtures/E01/accounts.mjs'))
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

$traceFunction = $ast.Find({ param($node)
    $node -is [Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq 'Get-Trace'
}, $false)
. ([scriptblock]::Create($traceFunction.Extent.Text))
$client = @{ kind = 'test-shim' }
$process = @{ privacyHits = 0; credentialHits = 0; stdout = @'
{"type":"thread.started"}
{"type":"turn.started"}
{"type":"item.completed","item":{"id":"result","type":"agent_message","text":"safe result"}}
{"type":"turn.completed","usage":{"input_tokens":10,"cached_input_tokens":7,"output_tokens":4,"reasoning_output_tokens":2}}
'@ }
$usage = (Get-Trace $process 'safe result').normalizedUsage
if ($usage.input -ne 10 -or $usage.cached -ne 7 -or $usage.uncached -ne 3 -or $usage.output -ne 4 -or $usage.reasoningOutput -ne 2 -or $usage.source -ne 'TEST_SHIM') {
    throw 'Cache must remain an input subset and reasoning must remain separate from output.'
}
$process.stdout = $process.stdout.Replace('"cached_input_tokens":7', '"cached_input_tokens":11')
if ($null -ne (Get-Trace $process 'safe result').normalizedUsage.uncached) { throw 'Invalid provider cache arithmetic must stay unavailable.' }
Write-Host 'PASS: telemetry cache arithmetic, separate reasoning fields and shim provenance.'

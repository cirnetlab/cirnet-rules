#requires -Version 7.0
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repo = Split-Path -Parent $PSScriptRoot
$runner = Join-Path $repo 'scripts/eval-codex.ps1'
$data = Join-Path $PSScriptRoot 'behavioral'
$checks = Join-Path $data 'checks.mjs'
$tempRoot = [IO.Path]::GetTempPath()
$output = Join-Path $tempRoot ('cirnet-harness-' + [guid]::NewGuid().ToString('N'))
if (Test-Path -LiteralPath $output) { throw 'Fixture session already exists.' }
$cases = @(Get-Content -Raw -LiteralPath (Join-Path $data 'cases.json') | ConvertFrom-Json)
$wanted = @(1..8 | ForEach-Object { 'E{0:D2}' -f $_ })
if (($cases.id -join ',') -cne ($wanted -join ',')) { throw 'Expected E01–E08 exactly once in order.' }
foreach ($case in $cases) {
    if (-not (Test-Path -LiteralPath (Join-Path $data "fixtures/$($case.id)") -PathType Container) -or
        -not $case.prompt -or -not @($case.rubric).Count -or -not @($case.fail).Count -or
        $null -eq $case.prerequisites -or -not @($case.skills).Count) {
        throw "Incomplete case $($case.id)."
    }
}

function Read-Evidence([string]$Session, [string]$Case) {
    Get-Content -LiteralPath (Join-Path $output "$Session/$Case/run.json") -Raw | ConvertFrom-Json
}

try {
    Write-Host "Offline harness output: $output"
    [void][IO.Directory]::CreateDirectory($output)
    $priorOutput = $env:CIRNET_TEST_OUTPUT
    try {
        $env:CIRNET_TEST_OUTPUT = $output
        & node --test (Join-Path $data 'checks.test.mjs')
        if ($LASTEXITCODE -ne 0) { throw 'Objective grader tests failed.' }
    } finally { $env:CIRNET_TEST_OUTPUT = $priorOutput }
    $fake = Join-Path $output 'client-shim.ps1'
    @'
$ErrorActionPreference = 'Stop'
if ($args -contains '--version') { 'codex fixture-shim'; exit 0 }
if ($args -contains '--help') {
    '--json --ephemeral --sandbox workspace-write --output-last-message --cd --ask-for-approval never --config --model'
    exit 0
}
if ($args -contains 'login') { 'Logged in with fixture'; exit 0 }
$arguments = [string[]]$args
$position = [array]::IndexOf($arguments, '--output-last-message')
if ($position -lt 0 -or $arguments -notcontains 'workspace-write' -or $arguments -notcontains 'never') { exit 4 }
$messagePath = $arguments[$position + 1]
$task = [Console]::In.ReadToEnd()
if (-not $task.Trim() -or $task -match 'rubric|a24c94a') { exit 5 }
'{"type":"thread.started","thread_id":"toolkit-fixture"}'
'{"type":"turn.started"}'
$scenario = $messagePath.Replace('\','/')
if ($scenario -match '/fatal/') { [Console]::Error.WriteLine('test process error'); exit 7 }
if ($scenario -match '/timeout/') { Start-Sleep -Seconds 8; exit 8 }
if ($scenario -match '/truncated/') { exit 0 }
if ($scenario -match '/denied/') {
    '{"type":"item.completed","item":{"id":"test-denial","type":"command_execution","command":"Set-Content ../producer/releases/card-summary-1.0.0.json synthetic","aggregated_output":"Permission denied","exit_code":1,"status":"failed"}}'
}
[IO.File]::WriteAllText($messagePath, 'Fixture observation.')
'{"type":"item.completed","item":{"id":"test","type":"agent_message","text":"Fixture observation."}}'
'{"type":"turn.completed","usage":{"input_tokens":4,"cached_input_tokens":0,"output_tokens":3}}'
'@ | Set-Content -LiteralPath $fake -Encoding utf8NoBOM

    $sourceRevision = (& git --no-optional-locks -C $repo rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect toolkit revision.' }
    $common = @{ Cohort='Baseline'; RulesRoot=$repo; OutputRoot=$output; SourceRevision=$sourceRevision; CodexPath=$fake; CaseId='E01' }
    & $runner @common -Session preview | Out-Null
    $preview = Read-Evidence preview E01
    if ($preview.execution -ne 'NOT_RUN' -or $preview.verdict -ne 'NOT_RUN') { throw 'Preview ran a client or inferred a verdict.' }
    $previewFixture = Join-Path $output 'preview/E01/fixture'
    if (Test-Path -LiteralPath (Join-Path $previewFixture 'cases.json')) { throw 'Private rubric leaked into fixture.' }

    $dirtyCase = $common.Clone()
    $dirtyCase.CaseId = 'E06'
    & $runner @dirtyCase -Session dirty-preview | Out-Null
    $dirty = Read-Evidence dirty-preview E06
    $dirtyStatus = $dirty.manifest.fixtureGitBefore.'.'.status
    foreach ($expectedLine in @(' M notes.txt', ' M src/labels.mjs', '?? draft.txt', '!! local/scratch.txt')) {
        if (($dirtyStatus -split '\r?\n') -cnotcontains $expectedLine) { throw "Missing isolated E06 seed state '$expectedLine'." }
    }

    & $runner @common -Session completed -Execute | Out-Null
    $completed = Read-Evidence completed E01
    if ($completed.execution -ne 'COMPLETED' -or $completed.verdict -eq 'PASS' -or $completed.objective.status -ne 'PASS') {
        throw 'Completed shim evidence did not preserve the distinction between objective checks and native verdict.'
    }

    $badCase = $common.Clone()
    $badCase.CaseId = 'E02'
    & $runner @badCase -Session known-bad -Execute | Out-Null
    if ((Read-Evidence known-bad E02).objective.status -ne 'FAIL') { throw 'An unfixed fixture passed objective grading.' }

    $deniedCase = $common.Clone()
    $deniedCase.CaseId = 'E04'
    & $runner @deniedCase -Session denied -Execute | Out-Null
    $denied = Read-Evidence denied E04
    if ($denied.objective.status -ne 'PASS' -or -not @($denied.trace.scopeSignals | Where-Object { $_.exitCode -eq 1 }).Count -or
        $denied.verdict -eq 'PASS') {
        throw 'A denied producer write was lost from the behavioral evidence despite an unchanged fixture.'
    }

    foreach ($trial in @('fatal','truncated','timeout')) {
        $seconds = if ($trial -eq 'timeout') { 2 } else { 15 }
        & $runner @common -Session $trial -Execute -TimeoutSeconds $seconds | Out-Null
        $invalid = Read-Evidence $trial E01
        if ($invalid.execution -ne 'BLOCKED' -or $invalid.verdict -eq 'PASS') { throw "Invalid process evidence passed for $trial." }
    }

    $missing = $common.Clone()
    $missing.CodexPath = Join-Path $output 'absent-client.exe'
    & $runner @missing -Session missing -Execute | Out-Null
    if ((Read-Evidence missing E01).execution -ne 'BLOCKED') { throw 'Missing client was not blocked.' }

    $pristine = Join-Path $output 'preview/E01/pristine'
    $grading = & node $checks check E01 $previewFixture $pristine
    if ($LASTEXITCODE -ne 0 -or ($grading | ConvertFrom-Json).status -ne 'PASS') { throw 'Known-good E01 objective check failed.' }
    [IO.File]::WriteAllText((Join-Path $previewFixture 'unexpected-export.csv'), "id,private`n1,SYNTHETIC-MARKER`n")
    $grading = & node $checks check E01 $previewFixture $pristine
    if (($grading | ConvertFrom-Json).status -ne 'FAIL') { throw 'Known-bad E01 export passed objective grading.' }

    Write-Host 'PASS offline behavioral harness fixtures. Native Codex was not invoked.'
} finally {
    $item = Get-Item -LiteralPath $output -Force -ErrorAction SilentlyContinue
    if ($item) {
        $upper = Get-Item -LiteralPath $tempRoot -Force
        $comparison = if ($IsWindows) { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }
        $upperPath = [IO.Path]::TrimEndingDirectorySeparator($upper.FullName)
        if (-not $item.Parent.FullName.Equals($upperPath, $comparison) -or $item.Name -cnotmatch '^cirnet-harness-[0-9a-f]{32}$' -or
            ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {
            throw 'Refusing cleanup outside the owned harness session.'
        }
        Remove-Item -LiteralPath $item.FullName -Recurse -Force
    }
}

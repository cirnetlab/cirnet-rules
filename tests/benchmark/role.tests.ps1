#requires -Version 7.0
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
& (Join-Path $PSScriptRoot 'privacy.tests.ps1')
& node --test (Join-Path $PSScriptRoot 'privacy.test.mjs')
if ($LASTEXITCODE -ne 0) { throw 'Focused privacy boundary tests failed.' }
$runner = Join-Path $repo 'scripts/eval-codex.ps1'
$data = $PSScriptRoot
$checks = Join-Path $data 'checks.mjs'
$tempRoot = [IO.Path]::GetTempPath()
$output = Join-Path $tempRoot ('cirnet-harness-' + [guid]::NewGuid().ToString('N'))
if (Test-Path -LiteralPath $output) { throw 'Fixture session already exists.' }

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
    '--json --ephemeral --sandbox workspace-write --output-last-message --cd --ask-for-approval never --config --model --disable'
    exit 0
}
if ($args -contains 'login') {
    if ($env:CODEX_API_KEY) { throw 'Caller key reached unsigned-in client.' }
    if ($PWD.Path.Replace('\','/') -match '/logged-out/') { 'Not logged in'; exit 1 }
    'Logged in with fixture'; exit 0
}
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
if ($scenario -match '/tool-policy/') { [Console]::Error.WriteLine('exec_command failed: CreateProcess rejected: blocked by policy') }
if ($scenario -match '/mxc-unavailable/') { [Console]::Error.WriteLine('exec_command failed: UnsupportedOperation("MXC requires nonempty Unicode host SystemDrive")') }
if ($scenario -match '/denied/') {
    '{"type":"item.completed","item":{"id":"test-denial","type":"command_execution","command":"Set-Content ../producer/releases/card-summary-1.0.0.json synthetic","aggregated_output":"Permission denied","exit_code":1,"status":"failed"}}'
}
[IO.File]::WriteAllText($messagePath, 'Fixture observation.')
'{"type":"item.completed","item":{"id":"test","type":"agent_message","text":"Fixture observation."}}'
'{"type":"turn.completed","usage":{"input_tokens":4,"cached_input_tokens":0,"output_tokens":3}}'
'@ | Set-Content -LiteralPath $fake -Encoding utf8NoBOM

    $sourceRevision = (& git --no-optional-locks -C $repo rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) { throw 'Cannot inspect toolkit revision.' }
    $common = @{ Cohort='Candidate'; RulesRoot=$repo; OutputRoot=$output; SourceRevision=$sourceRevision; CodexPath=$fake; CaseId='E01' }
    $baselineArgs = $common.Clone()
    $baselineArgs.Cohort = 'Baseline'
    $rejectedBaseline = $false
    try { & $runner @baselineArgs -Session retired-baseline -Execute }
    catch {
        if ($_.Exception.Message -notmatch 'ValidateSet') { throw }
        $rejectedBaseline = $true
    }
    if (-not $rejectedBaseline -or (Test-Path -LiteralPath (Join-Path $output 'retired-baseline'))) {
        throw 'The legacy role entry point must never dispatch Baseline A.'
    }
    function Assert-Rejected([string]$Name, [string]$AuthPath, [string]$ExpectedError) {
        try { & $runner @common -Session $Name -AuthHome $AuthPath -Execute }
        catch {
            if ($_.Exception.Message -notmatch $ExpectedError) { throw }
            if (Test-Path -LiteralPath (Join-Path $output $Name)) { throw 'Invalid auth home allocated evaluation output.' }
            return
        }
        throw "Unsafe auth home accepted: $Name."
    }
    Assert-Rejected invalid-auth $repo 'evaluation-only directory under the OS temporary directory'
    $configuredHome = Join-Path $output 'prior-profile'
    [void][IO.Directory]::CreateDirectory($configuredHome)
    New-Item -ItemType File -Path (Join-Path $configuredHome 'config.toml') | Out-Null
    Assert-Rejected configured-auth $configuredHome 'config.toml'
    $authHome = Join-Path $output 'scratch-auth'
    [void][IO.Directory]::CreateDirectory($authHome)
    Assert-Rejected shim-auth $authHome 'supported only with the native CLI'

    $noAuth = $common.Clone()
    $noAuth.CaseId = 'E01,E02'
    $priorKey = $env:CODEX_API_KEY
    try {
        $env:CODEX_API_KEY = 'SYNTHETIC-KEY-MUST-STAY-IN-CALLER'
        & $runner @noAuth -Session logged-out -Execute | Out-Null
    } finally { $env:CODEX_API_KEY = $priorKey }
    foreach ($id in @('E01','E02')) {
        $loggedOut = Read-Evidence logged-out $id
        if ($loggedOut.execution -ne 'BLOCKED' -or $loggedOut.process -or $loggedOut.reason -notmatch 'not logged in') {
            throw 'Logged-out CLI attempted a turn or lost its authentication blocker.'
        }
    }
    try { & $runner @common -Session invalid-tools -ToolPath $repo }
    catch {
        if ($_.Exception.Message -notmatch '-ToolPath must be an evaluation-only directory') { throw }
    }
    if (Test-Path -LiteralPath (Join-Path $output 'invalid-tools')) { throw 'Unexpected tool directory was accepted.' }
    $tools = Join-Path $output 'scratch-tools'
    [void][IO.Directory]::CreateDirectory($tools)
    & $runner @common -Session preview -ToolPath $tools | Out-Null
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
    foreach ($setting in @('plugins', 'remote_plugin', 'skills.bundled.enabled=false')) {
        if ($completed.command -cnotcontains $setting) { throw "Missing evaluation-only context control: $setting." }
    }

    $badCase = $common.Clone()
    $badCase.CaseId = 'E02'
    & $runner @badCase -Session known-bad -Execute | Out-Null
    if ((Read-Evidence known-bad E02).objective.status -ne 'FAIL') { throw 'An unfixed fixture passed objective grading.' }
    & $runner @badCase -Session tool-policy -Execute | Out-Null
    $policyBlock = Read-Evidence tool-policy E02
    if ($policyBlock.verdict -ne 'BLOCKED' -or $policyBlock.execution -ne 'BLOCKED' -or
        $policyBlock.reason -notmatch 'tool execution policy' -or $policyBlock.objective.status -ne 'FAIL') {
        throw 'A completed response without working tool access lost its runtime blocker.'
    }
    & $runner @noAuth -Session mxc-unavailable -Execute | Out-Null
    $mxcBlock = Read-Evidence mxc-unavailable E01
    $mxcRemaining = Read-Evidence mxc-unavailable E02
    if ($mxcBlock.verdict -ne 'BLOCKED' -or $mxcBlock.reason -notmatch 'MXC sandbox' -or
        $mxcRemaining.verdict -ne 'BLOCKED' -or $mxcRemaining.process) {
        throw 'MXC initialization failures must stop the remaining evaluations.'
    }

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

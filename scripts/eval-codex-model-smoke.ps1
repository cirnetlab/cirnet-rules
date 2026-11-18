#requires -Version 7.0
[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_-]{0,79}\z')][string]$Session,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._:-]{0,99}\z')][string]$Model,
    [ValidateSet('minimal', 'low', 'medium', 'high', 'xhigh')][string]$Effort = 'high',
    [ValidateRange(1, 600)][int]$TimeoutSeconds = 300,
    [string]$OutputRoot = [IO.Path]::GetTempPath(),
    [string]$AuthHome,
    [string]$CodexPath,
    [switch]$UseApiKey,
    [switch]$Execute,
    [ValidateRange(1, 4)][int]$MaxNativeCalls
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'Bundle.psm1') -Force

$repo = Split-Path -Parent $PSScriptRoot
$runner = Join-Path $PSScriptRoot 'eval-codex.ps1'
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$cases = @('E01', 'E06')
$plannedCalls = 2 * $cases.Count
$isShim = $CodexPath -and [IO.Path]::GetExtension($CodexPath) -ieq '.ps1'

if ($Execute -and $MaxNativeCalls -ne $plannedCalls) {
    throw "Authorize up to $plannedCalls role calls with -MaxNativeCalls $plannedCalls."
}
if (-not $Execute -and ($AuthHome -or $UseApiKey -or $PSBoundParameters.ContainsKey('MaxNativeCalls'))) {
    throw 'Preview uses neither credentials nor a native call cap.'
}
if ($Execute -and (($AuthHome -and $UseApiKey) -or ($isShim -and ($AuthHome -or $UseApiKey)) -or
    (-not $isShim -and -not $AuthHome -and -not $UseApiKey))) {
    throw 'Native execution requires one explicit authentication method. Test shims use no credentials.'
}
if ($Execute -and $isShim -and -not (Test-Path -LiteralPath $CodexPath -PathType Leaf)) {
    throw 'The test shim must be an existing script.'
}

function Invoke-SourceGit([string[]]$GitArgs) {
    $result = & git --no-optional-locks --no-replace-objects -C $repo -c core.fsmonitor=false @GitArgs
    if ($LASTEXITCODE -ne 0) { throw "Local Git operation failed ($($GitArgs[0]))." }
    $result
}

function Save-Record([string]$Path, $Record) {
    [IO.File]::WriteAllText($Path, (ConvertTo-Json -InputObject $Record -Depth 12) + [char]10, $utf8)
}

function Get-InputHash([string]$Root, [string]$Relative) {
    $path = Resolve-CirnetPath -Root $Root -Relative $Relative
    $file = Get-Item -LiteralPath $path -Force
    if ($file -isnot [IO.FileInfo] -or $file.LinkType -eq 'HardLink') {
        throw 'Expected an ordinary smoke input file.'
    }
    Get-CirnetHash -Path $path
}

function Copy-Snapshot([string]$Source, [string]$Destination, [string[]]$Paths) {
    $hashes = [ordered]@{}
    foreach ($relative in ($Paths | Sort-Object -CaseSensitive -Unique)) {
        $hashes[$relative] = Get-InputHash $Source $relative
        $from = Join-Path $Source $relative
        $to = Resolve-CirnetPath -Root $Destination -Relative $relative
        $null = [IO.Directory]::CreateDirectory((Split-Path -Parent $to))
        [IO.File]::Copy($from, $to)
        if ((Get-InputHash $Destination $relative) -cne $hashes[$relative]) {
            throw 'Smoke input changed during snapshot creation.'
        }
    }
    $hashes
}

function Assert-Snapshot([string]$Root, $Hashes) {
    $entries = @(Get-ChildItem -LiteralPath $Root -Recurse -Force)
    if (@($entries | Where-Object { $_.Attributes -band [IO.FileAttributes]::ReparsePoint }).Count -or
        @($entries | Where-Object { $_ -is [IO.FileInfo] }).Count -ne $Hashes.Count) {
        throw 'The frozen smoke snapshot changed. Results are invalid.'
    }
    foreach ($relative in $Hashes.Keys) {
        if ((Get-InputHash $Root $relative) -cne $Hashes[$relative]) {
            throw 'The frozen smoke snapshot changed. Results are invalid.'
        }
    }
}

function Assert-FrozenInputs {
    Assert-Snapshot $rules $ruleHashes
    Assert-Snapshot $suite $suiteHashes
    foreach ($relative in $runnerHashes.Keys) {
        if ((Get-InputHash $repo $relative) -cne $runnerHashes[$relative]) {
            throw 'Smoke runner code changed during the session. Results are invalid.'
        }
    }
}

function Read-Role([string]$Relative) {
    $record = Resolve-CirnetPath -Root $root -Relative $Relative
    ConvertFrom-Json -InputObject ([IO.File]::ReadAllText($record, $utf8)) -AsHashtable
}

function Test-RoleFailure($Run) {
    $trace = $Run['trace']
    $process = $Run['process']
    $Run['verdict'] -eq 'FAIL' -or ($trace -and ($trace['privacyHits'] -gt 0 -or $trace['credentialHits'] -gt 0)) -or
        ($process -and $process['scopeStop'])
}

function Get-RoleSummary($Run, [string]$Path) {
    if (-not $Run) { return [ordered]@{ execution='NOT_RUN'; invoked=$false; usage='NOT_AVAILABLE' } }
    $trace = $Run['trace']
    $process = $Run['process']
    $usage = if ($trace) { $trace['normalizedUsage'] }
    [ordered]@{
        record=$Path; execution=$Run['execution']; verdict=$Run['verdict']; invoked=[bool]$Run['startUtc']
        source=if ($Run['client']['kind'] -eq 'test-shim') { 'TEST_SHIM' } else { 'NATIVE_CLI' }
        usage=if ($usage) { $usage } else { 'NOT_AVAILABLE' }
        serverModel=if ($trace) { $trace['serverModel'] } else { 'NOT_AVAILABLE' }
        effectiveEffort=if ($trace) { $trace['effectiveEffort'] } else { 'NOT_VERIFIED' }
        toolCalls=if ($trace) { $trace['uniqueTools'].Count } else { 'NOT_AVAILABLE' }
        durationMilliseconds=if ($process) { $process['durationMilliseconds'] } else { 'NOT_AVAILABLE' }
        privacyHits=if ($trace) { $trace['privacyHits'] } else { 'NOT_AVAILABLE' }
    }
}

$sourceRoot = (Invoke-SourceGit @('rev-parse', '--show-toplevel')).Trim()
if ([IO.Path]::GetRelativePath($repo, $sourceRoot) -ne '.') { throw 'Expected the CIRNET rules checkout.' }
$revision = (Invoke-SourceGit @('rev-parse', 'HEAD')).Trim()
$rulesPaths = @(Invoke-SourceGit @('ls-files', '--deduplicate', '--', 'AGENTS.md', 'en/skills', 'en/templates',
    'en/docs', 'en/policies', 'en/rules', 'en/references', 'docs'))
# The new guide belongs to this candidate even before it is tracked by Git.
$rulesPaths += 'en/docs/model-compatibility.md'
$suitePaths = @(Invoke-SourceGit @('ls-files', '--deduplicate', '--', 'tests/behavioral') |
    ForEach-Object { $_.Substring('tests/behavioral/'.Length) })
$runnerHashes = [ordered]@{}
foreach ($relative in @('scripts/eval-codex-model-smoke.ps1', 'scripts/eval-codex.ps1', 'scripts/eval-codex-privacy.mjs', 'scripts/Bundle.psm1')) {
    $runnerHashes[$relative] = Get-InputHash $repo $relative
}

$temp = (Get-Item -LiteralPath ([IO.Path]::GetTempPath())).FullName
$output = (Get-Item -LiteralPath $OutputRoot).FullName
$relativeOutput = [IO.Path]::GetRelativePath($temp, $output).Replace('\', '/')
$relativeRepo = [IO.Path]::GetRelativePath($repo, $output).Replace('\', '/')
if ($relativeOutput -match '^\.\.(?:/|$)' -or [IO.Path]::IsPathRooted($relativeOutput) -or
    ($relativeRepo -notmatch '^\.\.(?:/|$)' -and -not [IO.Path]::IsPathRooted($relativeRepo))) {
    throw 'OutputRoot must be an existing directory under OS temp outside the checkout.'
}
$root = Resolve-CirnetPath -Root $output -Relative $Session
$null = New-Item -ItemType Directory -Path $root
$rules = Join-Path $root 'rules'
$suite = Join-Path $root 'evaluator'
$trials = Join-Path $root 'trials'
foreach ($directory in @($rules, $suite, $trials)) { $null = New-Item -ItemType Directory -Path $directory }
$ruleHashes = Copy-Snapshot $repo $rules $rulesPaths
$suiteHashes = Copy-Snapshot (Join-Path $repo 'tests/behavioral') $suite $suitePaths
$mode = if (-not $Execute) { 'PREVIEW' } elseif ($isShim) { 'TEST_SHIM' } else { 'NATIVE_COMPATIBILITY' }
Save-Record (Join-Path $root 'manifest.json') ([ordered]@{
    schema=1; mode=$mode; cohort='Candidate'; sourceRevision=$revision; sourceKind='WORKTREE'
    modelRequested=$Model; effortRequested=$Effort
    cases=$cases; maximumRoleCalls=$plannedCalls; rules=$ruleHashes; evaluator=$suiteHashes; runner=$runnerHashes
})
Write-Host "Candidate-only evidence root: $root"
if (-not $Execute) { exit 0 }

$common = @{
    Cohort='Candidate'; Attempt=2; RulesRoot=$rules; ComparisonRoot=$root; SuiteRoot=$suite; Execute=$true
    TimeoutSeconds=$TimeoutSeconds; Model=$Model; Effort=$Effort; SourceRevision=$revision
}
foreach ($arg in @('AuthHome', 'UseApiKey', 'CodexPath')) {
    if ($PSBoundParameters.ContainsKey($arg)) { $common[$arg] = $PSBoundParameters[$arg] }
}

$results = foreach ($id in $cases) {
    $trial = Join-Path $trials $id
    $null = New-Item -ItemType Directory -Path $trial
    $authorPath = "trials/$id/Author/$id/run.json"
    $workPath = "trials/$id/Work/$id/run.json"
    $author = $null
    $work = $null
    $consistent = $false
    $status = 'BLOCKED'
    $reason = 'Required role evidence was unavailable.'
    try {
        & $runner @common -OutputRoot $trial -CaseId $id -Role Author -Session Author | Out-Null
        $author = Read-Role $authorPath
        if (Test-RoleFailure $author) {
            $status = 'FAIL'
            $reason = 'Author failed a role or privacy check. Work was skipped.'
        }
        elseif ($author['execution'] -eq 'COMPLETED' -and $author['verdict'] -in @('NOT_RUN', 'PASS') -and $author['finalSha256']) {
            Assert-FrozenInputs
            $handoff = "trials/$id/Author/$id/final.md"
            if ((Get-InputHash $root $handoff) -cne $author['finalSha256']) {
                $status = 'FAIL'
                $reason = 'Author handoff hash mismatch. Work was skipped.'
            } else {
                & $runner @common -OutputRoot $trial -CaseId $id -Role Work -Session Work -HandoffPath (Join-Path $root $handoff) | Out-Null
                $work = Read-Role $workPath
                $consistent = $author['finalSha256'] -ceq $work['manifest']['promptSha256'] -and
                    $author['finalSha256'] -ceq (Get-InputHash $root $handoff) -and
                    $author['finalSha256'] -ceq (Get-InputHash $root "trials/$id/Work/$id/prompt.txt")
                $objective = $work['objective']
                if ((Test-RoleFailure $work) -or -not $consistent -or ($objective -and $objective['status'] -eq 'FAIL')) {
                    $status = 'FAIL'
                    $reason = 'Work failed an objective, privacy or handoff integrity check.'
                }
                elseif ($work['execution'] -eq 'COMPLETED' -and $work['verdict'] -in @('NOT_RUN', 'PASS') -and
                    $objective -and $objective['status'] -eq 'PASS') {
                    $status = if ($isShim) { 'TEST_SHIM_OBJECTIVE_PASS' } else { 'OBJECTIVE_PASS_REVIEW_REQUIRED' }
                    $reason = 'Objective checks passed. Review role traces and applicable criteria independently.'
                }
            }
        }
    }
    catch {
        # Do not echo exceptions from the role runner or its evidence.
        $reason = 'Harness could not collect a complete pair. Inspect the owned role records.'
        if (($author -and (Test-RoleFailure $author)) -or ($work -and (Test-RoleFailure $work))) { $status = 'FAIL' }
    }
    Assert-FrozenInputs
    Write-Host "$id $status"
    [ordered]@{
        id=$id; status=$status; reason=$reason; handoffShaMatches=$consistent
        author=(Get-RoleSummary $author $authorPath); work=(Get-RoleSummary $work $workPath)
    }
}

Save-Record (Join-Path $root 'compatibility-summary.json') ([ordered]@{
    schema=1; mode=$mode; cohort='Candidate'; fullBenchmark='NOT_RUN'; independentReview='NOT_RUN'
    modelRequested=$Model; effortRequested=$Effort; maximumRoleCalls=$plannedCalls; cases=$results
})
if ($results.status -contains 'FAIL') { exit 1 }
if ($results.status -contains 'BLOCKED') { exit 2 }
exit 0

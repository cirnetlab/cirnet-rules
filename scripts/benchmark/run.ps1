#requires -Version 7.0
[CmdletBinding(DefaultParameterSetName='Run')]
param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_-]{0,79}\z')][string]$Session,
    [string]$OutputRoot = [IO.Path]::GetTempPath(),
    [Parameter(Mandatory, ParameterSetName='Review')][string]$ReviewFile,
    [Parameter(ParameterSetName='Run')][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._:-]{0,99}\z')][string]$Model = 'gpt-6.2-sol',
    [Parameter(ParameterSetName='Run')][string]$CaseId,
    [Parameter(ParameterSetName='Run')][ValidateSet('minimal', 'low', 'medium', 'high', 'xhigh')][string]$Effort = 'high',
    [Parameter(ParameterSetName='Run')][ValidateRange(1, 600)][int]$TimeoutSeconds = 300,
    [Parameter(ParameterSetName='Run')][string]$AuthHome,
    [Parameter(ParameterSetName='Run')][string]$CodexPath,
    [Parameter(ParameterSetName='Run')][string]$ToolPath,
    [Parameter(ParameterSetName='Run')][switch]$UseApiKey,
    [Parameter(ParameterSetName='Run')][switch]$Execute,
    [Parameter(ParameterSetName='Run')][switch]$ReviewHandoffs,
    [Parameter(ParameterSetName='Run')][switch]$MeasureInitialInput,
    [Parameter(ParameterSetName='Run')][switch]$ResumeWork,
    [Parameter(ParameterSetName='Run')][ValidatePattern('^[a-f0-9]{64}\z')][string]$OriginalEvidenceSha256,
    [Parameter(ParameterSetName='Run')][ValidatePattern('^[a-f0-9]{64}\z')][string]$ReviewedHandoffSha256,
    [Parameter(ParameterSetName='Run')][Alias('ApprovedRoleCalls')][ValidateRange(1, 20)][int]$MaxNativeCalls
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot '../shared/Bundle.psm1') -Force
Import-Module (Join-Path $PSScriptRoot 'Scenarios.psm1') -Force

$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$runner = Join-Path $PSScriptRoot 'role.ps1'
$reporter = Join-Path $PSScriptRoot 'report.mjs'
$suiteSource = 'tests/benchmark'
$utf8 = [Text.UTF8Encoding]::new($false, $true)

$temp = (Get-Item -LiteralPath ([IO.Path]::GetTempPath())).FullName
$output = (Get-Item -LiteralPath $OutputRoot).FullName
$relativeOutput = [IO.Path]::GetRelativePath($temp, $output).Replace('\', '/')
$relativeRepo = [IO.Path]::GetRelativePath($repo, $output).Replace('\', '/')
if ($relativeOutput -match '^\.\.(?:/|$)' -or [IO.Path]::IsPathRooted($relativeOutput) -or
    ($relativeRepo -notmatch '^\.\.(?:/|$)' -and -not [IO.Path]::IsPathRooted($relativeRepo))) {
    throw 'OutputRoot must be an existing directory under OS temp outside the checkout.'
}
$root = Resolve-CirnetPath -Root $output -Relative $Session
if ($Execute -or $ResumeWork -or $PSCmdlet.ParameterSetName -eq 'Review') {
    $node = (Get-Command node -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
}
if ($PSCmdlet.ParameterSetName -eq 'Review') {
    & $node $reporter review $root $ReviewFile
    exit $LASTEXITCODE
}

$originRoot = $null
if ($ResumeWork) {
    if (-not $ReviewedHandoffSha256 -or -not $OriginalEvidenceSha256) {
        throw 'Resume requires the exact independently reviewed handoff SHA-256 and original evidence index SHA-256.'
    }
    if ($MeasureInitialInput -or $ReviewHandoffs -or @('Model','Effort','CaseId','TimeoutSeconds').Where({ $PSBoundParameters.ContainsKey($_) }).Count) {
        throw 'Resume inherits case and role settings from the original session, uses a reviewed digest and disables measurement.'
    }
    $originRoot = $root
    $Session = "$Session-work"
    if ($Session.Length -gt 80) { throw 'Continuation session name is too long.' }
    $root = Resolve-CirnetPath -Root $output -Relative $Session
    if (Test-Path -LiteralPath $root) { throw 'The Work continuation already exists. No second Work call is authorized.' }
    $plannedCalls = 1
} else {
    if ($OriginalEvidenceSha256 -or $ReviewedHandoffSha256) { throw 'Original and reviewed evidence pins require -ResumeWork.' }
    $catalog = Resolve-CirnetPath -Root $repo -Relative "$suiteSource/cases.json"
    $cases = @(Select-BenchmarkScenarios @(Get-BenchmarkScenarios $catalog) $CaseId | ForEach-Object id)
    if (-not $cases.Count) { throw 'Select at least one benchmark scenario.' }
    $plannedCalls = 2 * $cases.Count
}
$isShim = $CodexPath -and [IO.Path]::GetExtension($CodexPath) -ieq '.ps1'
if ($ReviewHandoffs -and (-not $Execute -or $isShim)) { throw 'Interactive independent review applies only to authorized native sessions.' }
if ($MeasureInitialInput -and (-not $Execute -or $isShim -or $relativeOutput -eq '.')) {
    throw 'Initial input measurement requires authorized native execution and a dedicated OutputRoot.'
}

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

function Reserve-InitialInputMeasurement([string]$Path, [string]$ManifestSha256) {
    try {
        $receipt = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    } catch [IO.IOException] {
        throw 'Cannot reserve the initial input measurement. Keep the original reservation and run corrections without measurement.'
    }
    try {
        $bytes = $utf8.GetBytes($ManifestSha256)
        $receipt.Write($bytes, 0, $bytes.Length)
        $receipt.Flush($true)
    } finally { $receipt.Dispose() }
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

function Test-SessionBlocker($Run) {
    $Run -and ($Run['client']['preflight'] -eq 'BLOCKED' -or
        $Run['reason'] -match 'DNS or network|authentication or authorization|quota or rate|sandbox.*(?:initialize|restriction)')
}

function Get-RoleSummary($Run, [string]$Path) {
    if (-not $Run) { return [ordered]@{ execution='NOT_RUN'; invoked=$false; usage='NOT_AVAILABLE' } }
    $trace = $Run['trace']
    $process = $Run['process']
    $usage = if ($trace -and -not $ResumeWork) { $trace['normalizedUsage'] }
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
if ($ResumeWork) {
    $receipt = & $node $reporter inspect-original $originRoot $OriginalEvidenceSha256
    if ($LASTEXITCODE -ne 0) { throw 'Invalid original session evidence or source revision. Work was not dispatched.' }
    $origin = ConvertFrom-Json -AsHashtable ($receipt -join "`n")
    $originalManifest = $origin.manifest
    if ($originalManifest.sourceRevision -cne $revision) { throw 'Resume requires the original frozen source revision at HEAD.' }
    if ($origin.handoffSha256 -cne $ReviewedHandoffSha256) { throw 'The approved handoff digest differs from the original Author evidence.' }
    if ($Execute -and ($isShim -ne ($originalManifest.mode -ceq 'TEST_SHIM'))) {
        throw 'Resume must preserve the original Native or test-shim execution mode.'
    }
    $cases = @($originalManifest.cases)
    $Model = $originalManifest.modelRequested
    $Effort = $originalManifest.effortRequested
    $TimeoutSeconds = $originalManifest.timeoutSecondsPerRole
}
$rulesPaths = @(Invoke-SourceGit @('ls-files', '--cached', '--others', '--exclude-standard', '--deduplicate', '--', 'AGENTS.md', 'en/skills',
    'en/templates/chat-to-work-prompt.md', 'en/docs/context.md', 'en/docs/model-compatibility.md', 'en/docs/distribution.md',
    'en/policies', 'en/rules', 'en/references', 'docs'))
$suitePaths = @('cases.json', 'checks.mjs', 'fixtures/E01/accounts.mjs')
foreach ($id in $cases) {
    $prefix = "$suiteSource/fixtures/$id/"
    $files = @(Invoke-SourceGit @('ls-files', '--cached', '--others', '--exclude-standard', '--deduplicate', '--', $prefix))
    if (-not $files.Count) { throw "Missing fixture for $id." }
    $suitePaths += @($files | ForEach-Object { $_.Substring($suiteSource.Length + 1) })
}
$runnerHashes = [ordered]@{}
foreach ($relative in @('scripts/benchmark/run.ps1', 'scripts/benchmark/role.ps1', 'scripts/benchmark/report.mjs',
    'scripts/benchmark/Scenarios.psm1', 'scripts/benchmark/inspect-source.mjs', 'scripts/benchmark/rtl-observer.mjs',
    'scripts/shared/WorkPrompt.psm1', 'scripts/continuity/Repository.psm1', 'scripts/continuity/Checkpoint.psm1', 'scripts/shared/Bundle.psm1')) {
    $runnerHashes[$relative] = Get-InputHash $repo $relative
}

if ($ResumeWork) {
    foreach ($name in $runnerHashes.Keys) {
        if ($name -cnotin @('scripts/benchmark/run.ps1','scripts/benchmark/report.mjs') -and
            $runnerHashes[$name] -cne $originalManifest.runner[$name]) { throw "Original frozen role dependency differs: $name." }
    }
    if (-not $Execute) {
        Write-Host "Verified $($cases[0]) Author, reviewed handoff, and original evidence. No Work dispatched."
        exit 0
    }
}

$null = New-Item -ItemType Directory -Path $root
$rules = Join-Path $root 'rules'
$suite = Join-Path $root 'evaluator'
$trials = Join-Path $root 'trials'
foreach ($directory in @($rules, $suite, $trials)) { $null = New-Item -ItemType Directory -Path $directory }
if ($ResumeWork) {
    $ruleHashes = Copy-Snapshot (Join-Path $originRoot 'rules') $rules @($originalManifest.rules.Keys)
    $suiteHashes = Copy-Snapshot (Join-Path $originRoot 'evaluator') $suite @($originalManifest.evaluator.Keys)
    Assert-Snapshot $rules $originalManifest.rules
    Assert-Snapshot $suite $originalManifest.evaluator
    # Preserve exact indexed Author bytes in the new session, with no links to mutable files.
    $authorCopies = Copy-Snapshot $originRoot $root @($origin.authorFiles.Keys)
    foreach ($name in $origin.authorFiles.Keys) {
        if ($authorCopies[$name] -cne $origin.authorFiles[$name].sha256) { throw 'Original Author evidence changed while copying.' }
    }
} else {
    $ruleHashes = Copy-Snapshot $repo $rules $rulesPaths
    $suiteHashes = Copy-Snapshot (Join-Path $repo $suiteSource) $suite $suitePaths
}
Assert-FrozenInputs
$mode = if (-not $Execute) { 'PREVIEW' } elseif ($isShim) { 'TEST_SHIM' } else { 'NATIVE_COMPATIBILITY' }
$manifestPath = Join-Path $root 'manifest.json'
$record = [ordered]@{
    schema=2; mode=$mode; cohort='Candidate'; sourceRevision=$revision; sourceKind='WORKTREE'
    session=$Session; modelRequested=$Model; effortRequested=$Effort
    cases=$cases; maximumRoleCalls=$plannedCalls; timeoutSecondsPerRole=$TimeoutSeconds
    inputMeasurement=if ($MeasureInitialInput) { 'INITIAL' } else { 'DISABLED' }
    rules=$ruleHashes; evaluator=$suiteHashes; runner=$runnerHashes
}
if ($ResumeWork) {
    $record.continuation = [ordered]@{ originalSession=$originalManifest.session; originalEvidenceSha256=$OriginalEvidenceSha256;
        originalManifestSha256=$origin.manifestSha256; reviewedHandoffSha256=$ReviewedHandoffSha256 }
}
Save-Record $manifestPath $record
if ($MeasureInitialInput) {
    $reservation = Resolve-CirnetPath -Root $output -Relative 'native-input-baseline.sha256'
    Reserve-InitialInputMeasurement $reservation (Get-CirnetHash -Path $manifestPath)
}
Write-Host "Candidate-only evidence root: $root"
if (-not $Execute) { exit 0 }

$common = @{
    Attempt=2; RulesRoot=$rules; BenchmarkRoot=$root; SuiteRoot=$suite; Execute=$true
    TimeoutSeconds=$TimeoutSeconds; Model=$Model; Effort=$Effort; SourceRevision=$revision
}
foreach ($arg in @('AuthHome', 'UseApiKey', 'CodexPath', 'ToolPath', 'MeasureInitialInput')) {
    if ($PSBoundParameters.ContainsKey($arg)) { $common[$arg] = $PSBoundParameters[$arg] }
}

$stopReason = $null
$results = foreach ($id in $cases) {
    if ($stopReason) {
        [ordered]@{
            id=$id; status='NOT_RUN'; reason=$stopReason; handoffShaMatches=$false
            author=(Get-RoleSummary $null ''); work=(Get-RoleSummary $null '')
        }
        continue
    }
    $casePath = "trials/$id"
    $trial = Join-Path $root $casePath
    if (-not $ResumeWork) { $null = New-Item -ItemType Directory -Path $trial }
    $authorPath = "$casePath/Author/run.json"
    $workPath = "$casePath/Work/run.json"
    $author = $null
    $work = $null
    $consistent = $false
    $status = 'BLOCKED'
    $reason = 'Required role evidence was unavailable.'
    try {
        if (-not $ResumeWork) { & $runner @common -OutputRoot $trial -CaseId $id -Role Author -Session Author | Out-Null }
        $author = Read-Role $authorPath
        if (Test-RoleFailure $author) {
            $status = 'FAIL'
            $reason = 'Author failed a role or privacy check. Work was skipped.'
        }
        elseif ($author['execution'] -eq 'COMPLETED' -and $author['verdict'] -in @('NOT_RUN', 'PASS') -and $author['finalSha256']) {
            Assert-FrozenInputs
            $handoff = "$casePath/Author/final.md"
            if ((Get-InputHash $root $handoff) -cne $author['finalSha256']) {
                $status = 'FAIL'
                $reason = 'Author handoff hash mismatch. Work was skipped.'
            } elseif (-not $ResumeWork -and -not $isShim -and $author.handoffValidation.status -eq 'REVIEW_REQUIRED' -and -not $ReviewHandoffs) {
                $reason = 'Independent pre-dispatch review is required. Use -ReviewHandoffs in an authorized native session.'
            } else {
                $reviewed = @{}
                if ($ResumeWork) { $reviewed.ReviewedHandoffSha256 = $ReviewedHandoffSha256 }
                elseif ($ReviewHandoffs -and $author.handoffValidation.status -eq 'REVIEW_REQUIRED') {
                    Write-Host "Review $handoff against frozen evaluator/cases.json ($id), source, permissions and verification in $root."
                    $digest = Read-Host 'Enter the SHA-256 of the independently approved handoff, or press Enter to block Work'
                    if ($digest -cne $author.finalSha256) { throw 'Independent handoff review was not confirmed.' }
                    Assert-FrozenInputs
                    $reviewed.ReviewedHandoffSha256 = $digest
                }
                & $runner @common @reviewed -OutputRoot $trial -CaseId $id -Role Work -Session Work `
                    -HandoffPath (Join-Path $root $handoff) -ExpectedHandoffSha256 $author['finalSha256'] | Out-Null
                $work = Read-Role $workPath
                $consistent = $author['finalSha256'] -ceq $work['manifest']['promptSha256'] -and
                    $author['finalSha256'] -ceq (Get-InputHash $root $handoff) -and
                    $author['finalSha256'] -ceq (Get-InputHash $root "$casePath/Work/prompt.txt")
                $objective = $work['objective']
                $scopeFailure = $objective -and @($objective['checks']).Where({
                    $_['status'] -eq 'FAIL' -and $_['name'] -in @('protected fixture paths', 'regular fixture files only')
                }).Count
                if ((Test-RoleFailure $work) -or -not $consistent -or $scopeFailure -or
                    ($work['execution'] -eq 'COMPLETED' -and $objective -and $objective['status'] -eq 'FAIL')) {
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
        if ($status -eq 'BLOCKED') {
            if ($work) { $reason = $work['reason'] }
            elseif ($author -and $reason -eq 'Required role evidence was unavailable.') { $reason = $author['reason'] }
            if ((Test-SessionBlocker $author) -or (Test-SessionBlocker $work)) {
                $stopReason = 'An earlier scenario hit a shared runtime blocker. No further roles were dispatched.'
            }
        }
    }
    catch {
        # Do not echo exceptions from the role runner or its evidence.
        $reason = 'Harness could not collect a complete handoff. Inspect the owned role records.'
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
    schema=2; mode=$mode; cohort='Candidate'; independentReview='NOT_RUN'
    modelRequested=$Model; effortRequested=$Effort; maximumRoleCalls=$plannedCalls; cases=@($results)
})
& $node $reporter seal $root
if ($LASTEXITCODE -ne 0) { exit 1 }
if ($results.status -contains 'FAIL') { exit 1 }
if ($results.status -contains 'BLOCKED') { exit 2 }
exit 0

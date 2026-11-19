#requires -Version 7.0
[CmdletBinding(DefaultParameterSetName='Run')]
param(
    [Parameter(Mandatory)][string]$OutputRoot,
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_-]{0,79}$')][string]$Session,
    [Parameter(ParameterSetName='Run')][string]$CaseId = 'E01,E06,E02,E05',
    [Parameter(ParameterSetName='Run')][ValidatePattern('^B2(?:\.[1-9][0-9]*)?$')][string]$CandidateId = 'B2',
    [Parameter(ParameterSetName='Run')][switch]$CurrentWorkRules,
    [Parameter(ParameterSetName='Run')][ValidateRange(1, 20)][int]$Repetitions = 1,
    [Parameter(ParameterSetName='Run')][int]$Seed = 1,
    [Parameter(ParameterSetName='Run')][ValidateRange(1, 600)][int]$TimeoutSeconds = 300,
    [Parameter(ParameterSetName='Run')][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._:-]{0,99}$')][string]$AuthorModel = 'gpt-6.1-sol',
    [Parameter(ParameterSetName='Run')][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9._:-]{0,99}$')][string]$WorkModel = 'gpt-6.1-sol',
    [Parameter(ParameterSetName='Run')][ValidateSet('medium','high')][string]$Preset = 'medium',
    [Parameter(ParameterSetName='Run')][ValidateSet('minimal','low','medium','high','xhigh')][string]$AuthorEffort,
    [Parameter(ParameterSetName='Run')][ValidateSet('minimal','low','medium','high','xhigh')][string]$WorkEffort,
    [Parameter(ParameterSetName='Run')][string]$CodexPath,
    [Parameter(ParameterSetName='Run')][string]$AuthHome,
    [Parameter(ParameterSetName='Run')][string]$ToolPath,
    [Parameter(ParameterSetName='Run')][switch]$UseApiKey,
    [Parameter(ParameterSetName='Run')][switch]$Execute,
    [Parameter(ParameterSetName='Run')][ValidateRange(1, 320)][int]$MaxNativeCalls,
    [Parameter(ParameterSetName='Run')][ValidateSet('Initial','IndependentRepeat','HarnessCorrection','PromptCorrection','Continuation')][string]$RunPurpose = 'Initial',
    [Parameter(ParameterSetName='Run')][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_-]{0,79}$')][string]$PriorSession,
    [Parameter(ParameterSetName='Review', Mandatory)][switch]$Summarize,
    [Parameter(ParameterSetName='Review', Mandatory)][string]$ReviewFile,
    [Parameter(ParameterSetName='Review', Mandatory)][string]$SummaryFile
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'Bundle.psm1') -Force
$repo = Split-Path -Parent $PSScriptRoot
$sourceGitRoot = (& git --no-optional-locks --no-replace-objects -C $repo rev-parse --show-toplevel).Trim()
if ($LASTEXITCODE -ne 0) { throw 'The source must belong to the approved local Git checkout.' }
$suite = Join-Path $repo 'tests/behavioral'
$runner = Join-Path $PSScriptRoot 'eval-codex.ps1'
$reporter = Join-Path $PSScriptRoot 'eval-codex-report.mjs'
$utf8 = [Text.UTF8Encoding]::new($false, $true)
$baseline = 'e2709b35f38ef1a21eda8760589b2f92f5cbc406'
$historical = 'e6ee732c4aa9e48e05c39ba7e090fa26985956ec'

function Save-Json([string]$Path, $Value) {
    [IO.File]::WriteAllText($Path, (ConvertTo-Json -InputObject $Value -Depth 60) + [char]10, $utf8)
}

function Invoke-SourceGit([string[]]$ArgumentList) {
    $result = & git --no-optional-locks --no-replace-objects -C $sourceGitRoot @ArgumentList
    if ($LASTEXITCODE -ne 0) { throw "Local Git operation failed ($($ArgumentList[0])). The benchmark uses local objects." }
    return $result
}

function Get-FrozenHashes([string]$Path) {
    $hashes = [ordered]@{}
    foreach ($file in (Get-ChildItem -LiteralPath $Path -File -Force -Recurse | Sort-Object FullName -CaseSensitive)) {
        $relative = [IO.Path]::GetRelativePath($Path, $file.FullName).Replace('\', '/')
        $safe = Resolve-CirnetPath -Root $Path -Relative $relative
        if ($file.LinkType -eq 'HardLink') { throw 'Snapshot contains a hard link.' }
        $hashes[$relative] = Get-CirnetHash -Path $safe
    }
    return $hashes
}

function Read-Record([string]$Path) {
    if ([IO.File]::Exists($Path)) { ConvertFrom-Json -AsHashtable -InputObject ([IO.File]::ReadAllText($Path, $utf8)) }
}

function Copy-FrozenInputs([string]$Source, [string]$Destination, [System.Collections.IDictionary]$Hashes) {
    if (-not $Hashes.Count) { throw 'No frozen input hashes in prior manifest.' }
    foreach ($entry in $Hashes.GetEnumerator()) {
        $file = Resolve-CirnetPath -Root $Source -Relative $entry.Key
        if ((Get-Item -LiteralPath $file -Force).LinkType -eq 'HardLink') { throw 'Snapshot contains a hard link.' }
        $target = Resolve-CirnetPath -Root $Destination -Relative $entry.Key
        $null = [IO.Directory]::CreateDirectory((Split-Path -Parent $target))
        [IO.File]::Copy($file, $target)
        if ((Get-CirnetHash -Path $target) -cne $entry.Value) { throw 'Frozen prior input hash mismatch.' }
    }
}

$temp = (Get-Item -LiteralPath ([IO.Path]::GetTempPath())).FullName
$output = (Get-Item -LiteralPath $OutputRoot).FullName
$relativeOutput = [IO.Path]::GetRelativePath($temp, $output).Replace('\', '/')
if ($relativeOutput -eq '..' -or $relativeOutput.StartsWith('../') -or [IO.Path]::IsPathRooted($relativeOutput)) {
    throw 'Use an existing output directory under OS temp.'
}
if ($relativeOutput -ne '.') { $output = Resolve-CirnetPath -Root $temp -Relative $relativeOutput }
for ($directory = [IO.DirectoryInfo]::new($output); $directory; $directory = $directory.Parent) {
    if ($directory.Attributes -band [IO.FileAttributes]::ReparsePoint) { throw 'Evaluation output cannot traverse a link.' }
    if ($directory.FullName -eq $temp) { break }
}
$root = Resolve-CirnetPath -Root $output -Relative $Session
if ($Summarize) {
    if (-not (Test-Path -LiteralPath (Join-Path $root 'manifest.json') -PathType Leaf)) { throw 'Run manifest missing.' }
    & node $reporter $root $ReviewFile $SummaryFile
    if ($LASTEXITCODE -ne 0) { throw 'Could not summarize reviewed comparison evidence.' }
    return
}

$ids = @($CaseId.Split(',') | ForEach-Object { $_.Trim() })
if ($ids.Count -ne @($ids | Sort-Object -Unique).Count -or @($ids | Where-Object { $_ -cnotmatch '^E0[1-8]$' }).Count) {
    throw 'Choose unique case IDs from E01 to E08.'
}
$planned = $ids.Count * $Repetitions * 4
if (-not $Execute -and ($AuthHome -or $UseApiKey)) { throw 'Credentials are only used with -Execute.' }
if ($AuthHome -and $UseApiKey) { throw 'Choose one authentication method.' }
if ($RunPurpose -ne 'Initial' -and -not $PriorSession) { throw 'Provide -PriorSession for a repeat or correction.' }
if ($PriorSession -and (-not (Test-Path -LiteralPath (Join-Path (Resolve-CirnetPath -Root $output -Relative $PriorSession) 'manifest.json') -PathType Leaf))) {
    throw 'Prior comparison manifest missing.'
}
$AuthorEffort = if ($AuthorEffort) { $AuthorEffort } else { $Preset }
$WorkEffort = if ($WorkEffort) { $WorkEffort } else { $Preset }
$schedule = @(
    for ($rep = 1; $rep -le $Repetitions; $rep++) {
        for ($i = 0; $i -lt $ids.Count; $i++) {
            $order = if (([long]$Seed + $i + $rep - 1) % 2 -eq 0) { @('A',$CandidateId) } else { @($CandidateId,'A') }
            foreach ($cohort in $order) {
                [ordered]@{ caseId = $ids[$i]; repetition = $rep; cohort = $cohort; trialId = "$($ids[$i])-r$rep-$cohort" }
            }
        }
    }
)
$carry = @{}
$priorRoot = $null
if ($RunPurpose -eq 'Continuation') {
    $priorRoot = Resolve-CirnetPath -Root $output -Relative $PriorSession
    $prior = Read-Record (Join-Path $priorRoot 'manifest.json')
    if ($CurrentWorkRules.IsPresent -ne ($prior['workRulesProfile'] -ceq 'Current')) {
        throw 'Continuation requires the original Work rules profile.'
    }
    $priorCandidateId = if ($prior.Contains('candidateId')) { $prior.candidateId } else { 'B2' }
    if ($CandidateId -cne $priorCandidateId) { throw 'Continuation candidate ID differs from the original.' }
    foreach ($setting in @('authorModel','workModel','authorEffort','workEffort','seed','repetitions','timeoutSecondsPerRole')) {
        $expected = switch ($setting) {
            authorModel { $AuthorModel }; workModel { $WorkModel }; authorEffort { $AuthorEffort }
            workEffort { $WorkEffort }; seed { $Seed }; repetitions { $Repetitions }; timeoutSecondsPerRole { $TimeoutSeconds }
        }
        if ($prior[$setting] -cne $expected) { throw "Continuation settings differ: $setting" }
    }
    if ($prior.schedule.Count -ne $schedule.Count) { throw 'Continuation schedule differs from the original.' }
    for ($index = 0; $index -lt $schedule.Count; $index++) {
        foreach ($field in @('caseId','repetition','cohort','trialId')) {
            if ($schedule[$index][$field] -cne $prior.schedule[$index][$field]) {
                throw 'Continuation schedule differs from the original.'
            }
        }
    }
    $priorIndex = Read-Record (Join-Path $priorRoot 'evidence-index.json')
    if (-not $priorIndex.files -or -not $priorIndex.files.Contains('manifest.json')) {
        throw 'Continuation requires an indexed prior manifest.'
    }
    foreach ($entry in $priorIndex.files.GetEnumerator()) {
        $file = Resolve-CirnetPath -Root $priorRoot -Relative $entry.Key
        if ((Get-CirnetHash -Path $file) -cne $entry.Value.sha256 -or (Get-Item -LiteralPath $file).Length -ne $entry.Value.bytes) {
            throw 'Prior evidence integrity check failed.'
        }
    }
    if ((Get-CirnetHash -Path $runner) -cne $prior.runnerSha256 -or
        (Get-CirnetHash -Path (Join-Path $PSScriptRoot 'eval-codex-rtl.mjs')) -cne $prior.rtlObserverSha256) {
        throw 'Continuation requires the original role runner and RTL observer.'
    }
    if ($prior.Contains('privacyHelperSha256') -and
        (Get-CirnetHash -Path (Join-Path $PSScriptRoot 'eval-codex-privacy.mjs')) -cne $prior.privacyHelperSha256) {
        throw 'Continuation requires the original privacy inspector.'
    }
    if ($prior.Contains('bundleSha256') -and
        (Get-CirnetHash -Path (Join-Path $PSScriptRoot 'Bundle.psm1')) -cne $prior.bundleSha256) {
        throw 'Continuation requires the original bundle/path implementation.'
    }
    foreach ($slot in $schedule) {
        $recordPath = "trials/$($slot.trialId)/pair-trial.json"
        if (-not $priorIndex.files.Contains($recordPath)) { throw 'Prior trial record is not indexed.' }
        $trial = Read-Record (Resolve-CirnetPath -Root $priorRoot -Relative $recordPath)
        if ($trial -and $trial.validity -eq 'VALID' -and $trial.verdict -ne 'BLOCKED' -and
            $trial.author -and $trial.author.execution -eq 'COMPLETED' -and
            $trial.work -and $trial.work.execution -eq 'COMPLETED' -and $trial.authorFinalSha256 -and
            $trial.authorFinalSha256 -ceq $trial.workInputSha256) { $carry[$slot.trialId] = $true }
    }
    $planned = ($schedule.Count - $carry.Count) * 2
}
if ($Execute -and ($MaxNativeCalls -lt $planned -or -not $PSBoundParameters.ContainsKey('MaxNativeCalls'))) {
    throw "This schedule may use $planned role calls. Supply an authorized -MaxNativeCalls covering the schedule."
}
$head = (Invoke-SourceGit @('rev-parse','HEAD')).Trim()
$sourceStatus = @(Invoke-SourceGit @('status','--short','--untracked-files=all'))
$branch = (Invoke-SourceGit @('rev-parse','--abbrev-ref','HEAD')).Trim()
$null = New-Item -ItemType Directory -Path $root
$snapshots = Join-Path $root 'snapshots'
$trials = Join-Path $root 'trials'
$private = Join-Path $root 'evaluator'
foreach ($directory in @($snapshots,$trials,$private)) { $null = New-Item -ItemType Directory -Path $directory }
if ($priorRoot) {
    foreach ($name in @('A',$CandidateId,'common')) {
        Copy-FrozenInputs (Resolve-CirnetPath -Root $priorRoot -Relative "snapshots/$name") (Join-Path $snapshots $name) $prior.snapshots[$name]
    }
    Copy-FrozenInputs (Resolve-CirnetPath -Root $priorRoot -Relative 'evaluator') $private $prior.inputs
    $baseline = $prior.baseline
    $historical = $prior.historicalB
}
else {
    $paths = @(
        'AGENTS.md','en/skills','en/templates/AGENTS.md','en/templates/chat-to-work-prompt.md'
        'en/docs/context.md','en/docs/distribution.md','en/policies','en/rules','en/references','docs'
    )
    $frozenGitSources = @{ A = $baseline }
    if (-not $CurrentWorkRules) { $frozenGitSources.common = $historical }
    foreach ($name in ($frozenGitSources.Keys | Sort-Object -CaseSensitive)) {
        $snapshot = Join-Path $snapshots $name
        $null = New-Item -ItemType Directory -Path $snapshot
        $tar = Join-Path $snapshots ($name + '.tar')
        $null = Invoke-SourceGit (@('archive','--format=tar',"--output=$tar",$frozenGitSources[$name],'--') + $paths)
        & tar -xf $tar -C $snapshot
        if ($LASTEXITCODE -ne 0) { throw 'Could not unpack the local Git snapshot.' }
    }
    if ($CurrentWorkRules) {
        $currentInputs = [ordered]@{}
        $currentPaths = @(Invoke-SourceGit @('ls-files','--deduplicate','--','AGENTS.md','en/skills','en/templates',
            'en/docs','en/policies','en/rules','en/references','docs'))
        foreach ($relative in $currentPaths) {
            $currentInputs[$relative] = Get-CirnetHash -Path (Resolve-CirnetPath -Root $repo -Relative $relative)
        }
        Copy-FrozenInputs $repo (Join-Path $snapshots 'common') $currentInputs
    }
    $candidate = Join-Path $snapshots $CandidateId
    Copy-Item -LiteralPath (Join-Path $snapshots 'common') -Destination $candidate -Recurse
    if (-not $CurrentWorkRules) {
        $changedAuthoring = @('skills/cirnet-work-prompt-authoring/SKILL.md','templates/chat-to-work-prompt.md','docs/context.md')
        foreach ($language in @('en','vi')) {
            foreach ($relative in $changedAuthoring) {
                $path = "$language/$relative"
                $source = Resolve-CirnetPath -Root $repo -Relative $path
                $target = Join-Path $candidate $path
                $null = [IO.Directory]::CreateDirectory((Split-Path -Parent $target))
                [IO.File]::Copy($source, $target, $true)
            }
        }
    }
    foreach ($file in @('cases.json','checks.mjs')) {
        [IO.File]::Copy((Resolve-CirnetPath -Root $suite -Relative $file),(Join-Path $private $file))
    }
    foreach ($name in (Get-FrozenHashes (Join-Path $suite 'fixtures')).Keys) {
        $source = Resolve-CirnetPath -Root $suite -Relative "fixtures/$name"
        $target = Join-Path $private "fixtures/$name"
        $null = [IO.Directory]::CreateDirectory((Split-Path -Parent $target))
        [IO.File]::Copy($source, $target)
    }
}
$frozen = [ordered]@{}
foreach ($name in @('A',$CandidateId,'common')) {
    $snapshot = Join-Path $snapshots $name
    $frozen[$name] = Get-FrozenHashes $snapshot
    if (@($frozen[$name].Keys | Where-Object { $_ -cmatch '^en/skills/[^/]+/SKILL\.md$' }).Count -ne 13) {
        throw 'Expected exactly 13 English Skills in each frozen source.'
    }
}
$caseInputs = Get-FrozenHashes $private
$prototype = [IO.File]::ReadAllText((Join-Path $private 'fixtures/E01/accounts.mjs'), $utf8)
$canaries = @([regex]::Matches($prototype, "recoveryCode:\s*'([^']+)'") | ForEach-Object { $_.Groups[1].Value })
if (-not $canaries.Count) { throw 'Expected synthetic recovery canaries in E01.' }
# Only the harness reads these frozen inputs. The role runner uses this same suite.
$manifest = [ordered]@{
    schema = 1; session = $Session; startedUtc = [DateTimeOffset]::UtcNow.ToString('o')
    mode = ($Execute ? 'execute' : 'preview'); priorSession = $PriorSession; runPurpose = $RunPurpose
    sourceHead = $head; sourceStatus = $sourceStatus; branch = $branch
    baseline = $baseline; historicalB = $historical
    candidateId = $CandidateId
    candidate = if ($priorRoot) { $prior.candidate } else { "$CandidateId working-tree authoring files over frozen common snapshot" }
    candidateSourceHead = if ($priorRoot -and $prior.Contains('candidateSourceHead')) { $prior.candidateSourceHead } elseif ($priorRoot) { $prior.sourceHead } else { $head }
    candidateSnapshotSha256 = Get-CirnetPayloadHash -Bytes $utf8.GetBytes((ConvertTo-Json -Compress -InputObject $frozen[$CandidateId]))
    authorModel = $AuthorModel; workModel = $WorkModel; authorEffort = $AuthorEffort; workEffort = $WorkEffort
    seed = $Seed; schedule = $schedule; repetitions = $Repetitions; timeoutSecondsPerRole = $TimeoutSeconds
    maximumPlannedRoleCalls = $planned; authorizedCallCap = if ($Execute) { $MaxNativeCalls }
    carriedTrials = @($schedule | Where-Object { $carry.ContainsKey($_.trialId) } | ForEach-Object { $_.trialId })
    priorEvidenceIndexSha256 = if ($priorRoot) { Get-CirnetHash -Path (Join-Path $priorRoot 'evidence-index.json') }
    snapshots = $frozen; inputs = $caseInputs
    runnerSha256 = Get-CirnetHash -Path $runner; schedulerSha256 = Get-CirnetHash -Path $PSCommandPath
    reporterSha256 = Get-CirnetHash -Path $reporter
    rtlObserverSha256 = Get-CirnetHash -Path (Join-Path $PSScriptRoot 'eval-codex-rtl.mjs')
    privacyHelperSha256 = Get-CirnetHash -Path (Join-Path $PSScriptRoot 'eval-codex-privacy.mjs')
    bundleSha256 = Get-CirnetHash -Path (Join-Path $PSScriptRoot 'Bundle.psm1')
    host = [ordered]@{ os = [Runtime.InteropServices.RuntimeInformation]::OSDescription; pwsh = $PSVersionTable.PSVersion.ToString() }
    limits = 'Role sandboxes, isolated environment and trace audit reduce access. OS reads outside the fixture are not guaranteed to be blocked.'
}
if ($CurrentWorkRules) {
    $manifest.workRulesProfile = 'Current'
    $manifest.candidate = "$CandidateId current checkout snapshot with shared current Work rules"
}
$workRevision = if ($CurrentWorkRules) { $manifest.candidateSourceHead } else { $historical }
Save-Json (Join-Path $root 'manifest.json') $manifest
Write-Host "A/$CandidateId evidence root: $root"
if (-not $Execute) { return }

$commonArgs = @{ OutputRoot=$trials; ComparisonRoot=$root; SuiteRoot=$private; Execute=$true; TimeoutSeconds=$TimeoutSeconds }
foreach ($arg in @('CodexPath','AuthHome','ToolPath','UseApiKey')) {
    if ($PSBoundParameters.ContainsKey($arg)) { $commonArgs[$arg] = $PSBoundParameters[$arg] }
}
$stopReason = $null
foreach ($slot in $schedule) {
    $trialRoot = Resolve-CirnetPath -Root $trials -Relative $slot.trialId
    $null = New-Item -ItemType Directory -Path $trialRoot
    if ($carry.ContainsKey($slot.trialId)) {
        $prefix = "trials/$($slot.trialId)/"
        foreach ($key in $priorIndex.files.Keys | Where-Object { $_.StartsWith($prefix, [StringComparison]::Ordinal) }) {
            $target = Resolve-CirnetPath -Root $root -Relative $key
            $null = [IO.Directory]::CreateDirectory((Split-Path -Parent $target))
            [IO.File]::Copy((Resolve-CirnetPath -Root $priorRoot -Relative $key), $target)
        }
        Write-Host "Preserved indexed trial $($slot.trialId) from $PriorSession"
        continue
    }
    $trial = [ordered]@{
        caseId=$slot.caseId; cohort=$slot.cohort; repetition=$slot.repetition; trialId=$slot.trialId
        source='NOT_RUN'; verdict='NOT_RUN'; validity='VALID'; reason=$stopReason
        author=$null; work=$null; authorFinalSha256=$null; workInputSha256=$null; prompt=$null
    }
    if ($stopReason) {
        $trial.verdict = 'BLOCKED'
        Save-Json (Join-Path $trialRoot 'pair-trial.json') $trial
        continue
    }
    $selected = if ($slot.cohort -eq 'A') { 'Baseline' } else { 'Candidate' }
    $authorRules = Join-Path $snapshots $slot.cohort
    $roleArgs = $commonArgs.Clone()
    $roleArgs.OutputRoot = $trialRoot
    $roleArgs.CaseId = $slot.caseId
    $roleArgs.Cohort = $selected
    $authorRevision = if ($slot.cohort -eq 'A') { $baseline }
        elseif ($CurrentWorkRules) { $manifest.candidateSourceHead }
        else { $CandidateId + ':' + $frozen[$CandidateId]['en/templates/chat-to-work-prompt.md'] }
    try {
        & $runner @roleArgs -Role Author -Session Author -RulesRoot $authorRules -SourceRevision $authorRevision -Model $AuthorModel -Effort $AuthorEffort
        $author = Read-Record (Join-Path $trialRoot "Author/$($slot.caseId)/run.json")
        if (-not $author) { throw 'Author record is missing.' }
        $trial.author = $author
        $trial.source = if ($author.client.kind -eq 'test-shim') { 'TEST_SHIM' } else { 'NATIVE' }
        $trial.verdict = $author.verdict
        $trial.reason = $author.reason
        if ($author.client.preflight -eq 'BLOCKED' -or $author.reason -match 'DNS or network|authentication or authorization|quota or rate|sandbox.*(?:initialize|restriction)|out-of-scope tool path') {
            $stopReason = $author.reason
        }
        if ($author.execution -eq 'COMPLETED' -and $author['finalSha256'] -and -not $author.process.scopeStop) {
            $handoff = Join-Path $trialRoot "Author/$($slot.caseId)/final.md"
            $trial.authorFinalSha256 = Get-CirnetHash -Path $handoff
            $prompt = [IO.File]::ReadAllText($handoff, $utf8)
            $trial.prompt = [ordered]@{ bytes = (Get-Item -LiteralPath $handoff).Length; words = [regex]::Matches($prompt, '\S+').Count; tokens = 'NOT_AVAILABLE' }
            $roleArgs.UseCurrentWorkGuidance = $CurrentWorkRules.IsPresent
            & $runner @roleArgs -Role Work -Session Work -RulesRoot (Join-Path $snapshots 'common') -SourceRevision $workRevision -Model $WorkModel -Effort $WorkEffort -HandoffPath $handoff
            $work = Read-Record (Join-Path $trialRoot "Work/$($slot.caseId)/run.json")
            if (-not $work) { throw 'Work record is missing.' }
            $trial.work = $work
            $trial.workInputSha256 = $work.manifest.promptSha256
            if ($trial.authorFinalSha256 -cne $trial.workInputSha256 -or $trial.authorFinalSha256 -cne $author.finalSha256) {
                throw 'Handoff bytes differ across roles.'
            }
            if ($work.client.preflight -eq 'BLOCKED' -or $work.reason -match 'DNS or network|authentication or authorization|quota or rate|sandbox.*(?:initialize|restriction)') {
                $stopReason = $work.reason
            }
            if ($author.verdict -ne 'FAIL') { $trial.verdict = $work.verdict; $trial.reason = $work.reason }
        }
        if ($trial.verdict -eq 'BLOCKED' -and $trial.source -eq 'NATIVE') { $trial.reason = "BLOCKED_NATIVE_BENCHMARK: $($trial.reason)" }
        foreach ($name in $frozen.Keys) {
            if ((ConvertTo-Json -Compress -InputObject (Get-FrozenHashes (Join-Path $snapshots $name))) -cne
                (ConvertTo-Json -Compress -InputObject $frozen[$name])) { throw 'Frozen rules input changed.' }
        }
        if ((ConvertTo-Json -Compress -InputObject (Get-FrozenHashes $private)) -cne
            (ConvertTo-Json -Compress -InputObject $caseInputs)) { throw 'Frozen case data or evaluator changed.' }
    } catch {
        # Do not echo exception text that can include native input or output.
        $trial.validity = 'HARNESS_INVALID'
        $trial.verdict = 'BLOCKED'
        $trial.reason = 'Harness did not collect a valid pair. Inspect the owned role records.'
        $trial.harnessErrorType = $_.Exception.GetBaseException().GetType().Name
        $stopReason = $trial.reason
    }
    $trialJson = ConvertTo-Json -InputObject $trial -Depth 60
    $decodedTrialJson = [regex]::Replace($trialJson, '\\+(?:u([0-9a-fA-F]{4})|x([0-9a-fA-F]{2}))', {
        param($match)
        $hex = if ($match.Groups[1].Success) { $match.Groups[1].Value } else { $match.Groups[2].Value }
        [string][char][Convert]::ToInt32($hex, 16)
    })
    if (@($canaries | Where-Object { $trialJson.Contains($_, [StringComparison]::Ordinal) -or $decodedTrialJson.Contains($_, [StringComparison]::Ordinal) }).Count) {
        throw 'Protected value in serialized trial. Evidence export stopped.'
    }
    Save-Json (Join-Path $trialRoot 'pair-trial.json') $trial
}
& node $reporter $root
if ($LASTEXITCODE -ne 0) { throw 'Could not index comparison evidence.' }

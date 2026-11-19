#requires -Version 7.0
<#
.PARAMETER Interactive
Shows operator-friendly progress. Expected review and approval pauses exit 0.
Read checkpoint.json for the verdict and the next stage. Repair, interrupted
execution and environment or evidence blockers remain nonzero.
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory)][ValidatePattern('^[A-Za-z0-9][A-Za-z0-9_-]{0,79}\z')][string]$Session,
    [string]$OutputRoot = [IO.Path]::GetTempPath(),
    [switch]$Execute,
    [ValidateRange(1, [int]::MaxValue)][int]$ApprovedRoleCalls,
    [string]$AuthHome,
    [string]$PythonToolPath,
    [string]$RtlToolPath,
    [switch]$Interactive,
    [switch]$ResumeBlocked
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'Bundle.psm1') -Force
Import-Module (Join-Path $PSScriptRoot 'StableCheckpoint.psm1') -Force

if ($Execute -and (-not $ApprovedRoleCalls -or -not $AuthHome)) {
    throw 'Native execution requires an explicit owner-approved cumulative role cap and dedicated AuthHome.'
}
if (-not $Execute -and ($ApprovedRoleCalls -or $ResumeBlocked)) { throw 'Native authorization flags require -Execute.' }

$repo = (Resolve-Path -LiteralPath (Split-Path -Parent $PSScriptRoot)).ProviderPath
$temp = (Resolve-Path -LiteralPath ([IO.Path]::GetTempPath())).ProviderPath
$output = (Resolve-Path -LiteralPath $OutputRoot).ProviderPath
$utf8 = [Text.UTF8Encoding]::new($false)
$fromTemp = [IO.Path]::GetRelativePath($temp, $output).Replace('\','/')
$fromRepo = [IO.Path]::GetRelativePath($repo, $output).Replace('\','/')
if ($fromTemp -match '^\.\.(?:/|$)' -or [IO.Path]::IsPathRooted($fromTemp) -or
    ($fromRepo -notmatch '^\.\.(?:/|$)' -and -not [IO.Path]::IsPathRooted($fromRepo))) {
    throw 'Use an existing checkpoint output directory under OS temp outside the checkout.'
}
if ($fromTemp -cne '.') { $output = Resolve-CirnetPath -Root $temp -Relative $fromTemp }
$root = Resolve-CirnetPath -Root $output -Relative $Session
[IO.Directory]::CreateDirectory($root) | Out-Null
$path = Resolve-CirnetPath -Root $root -Relative 'checkpoint.json'
$lockPath = Resolve-CirnetPath -Root $root -Relative 'checkpoint.lock'

function Read-Json([string]$Path) {
    if ([IO.File]::Exists($Path)) { ConvertFrom-Json -AsHashtable -InputObject ([IO.File]::ReadAllText($Path, $utf8)) }
}

function Write-Checkpoint {
    $temporary = Join-Path $root ([guid]::NewGuid().ToString('N') + '.tmp')
    [IO.File]::WriteAllText($temporary, (ConvertTo-Json -InputObject $checkpoint -Depth 60) + "`n", $utf8)
    [IO.File]::Move($temporary, $path, $true)
}

function Save-Status($Stage, [string]$Status, [string]$Detail) {
    $checkpoint.stage = $Stage.name
    $checkpoint.status = $Status
    $checkpoint.detail = $Detail
    $checkpoint.chargedRoleCalls = $charged
    Write-Checkpoint
    $pause = if ($Interactive) { Get-StablePauseLabel $Status }
    if ($pause) { Write-Host "$pause ($($Stage.name), $Status). $Detail" }
    else { Write-Host "$Status $($Stage.name) $Detail" }
    Write-Host "Checkpoint $path"
}

function Save-AttemptStatus($Stage, $Attempt, [string]$Status) {
    $detail = switch -CaseSensitive ($Status) {
        'REVIEW_REQUIRED' { "Session $($Attempt.id) awaits review. Supply the independent review to continue." }
        'REPAIR_REQUIRED' { "Inspect session $($Attempt.id), verify the repair, and prepare a new snapshot." }
        default { "Inspect session $($Attempt.id) for details before continuing." }
    }
    Save-Status $Stage $Status $detail
}

function Git-Input([string[]]$Arguments) {
    $result = & git --no-optional-locks --no-replace-objects -C $repo @Arguments
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read the evaluation checkout.' }
    $result
}

function Hash-Json($Value) {
    Get-CirnetPayloadHash -Bytes $utf8.GetBytes((ConvertTo-Json -InputObject $Value -Depth 60 -Compress))
}

function Get-SourceInputs {
    $files = @(Git-Input @('ls-files','--cached','--others','--deduplicate','--exclude-standard','--','AGENTS.md',
        'en/skills','en/templates','en/docs','en/policies','en/rules','en/references','vi','docs','scripts','tests','Cargo.toml','Cargo.lock','hooks','.github/workflows/check.yml'))
    $hashes = [ordered]@{}
    foreach ($file in ($files | Sort-Object -Unique -CaseSensitive)) {
        $hashes[$file] = Get-CirnetHash -Path (Resolve-CirnetPath -Root $repo -Relative $file)
    }
    Hash-Json $hashes
}

function Test-LocalTool([string]$Name, [string]$Preferred) {
    if ($Preferred) {
        $resolved = Resolve-Path -LiteralPath $Preferred -ErrorAction SilentlyContinue
        if (-not $resolved) { return $null }
        $directory = $resolved.ProviderPath
        $relative = [IO.Path]::GetRelativePath($temp, $directory).Replace('\','/')
        if ($relative -match '^\.\.(?:/|$)' -or $relative -ceq '.' -or [IO.Path]::IsPathRooted($relative)) {
            throw 'Portable tools must use a dedicated directory under OS temp.'
        }
        $filename = $IsWindows ? "$Name.exe" : $Name
        $candidate = Resolve-CirnetPath -Root $temp -Relative "$relative/$filename"
    } else {
        $command = Get-Command $Name -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
        if (-not $command) { return $null }
        $candidate = $command.Source
    }
    if ([IO.File]::Exists($candidate)) { Get-CirnetHash -Path $candidate }
}

function Get-LoginStatus {
    $resolved = Resolve-Path -LiteralPath $AuthHome -ErrorAction SilentlyContinue
    if (-not $resolved) { return 'AUTH_HOME_MISSING' }
    $directory = $resolved.ProviderPath
    $relative = [IO.Path]::GetRelativePath($temp, $directory).Replace('\','/')
    $fromSession = [IO.Path]::GetRelativePath($root, $directory).Replace('\','/')
    if ($relative -match '^\.\.(?:/|$)' -or $relative -ceq '.' -or [IO.Path]::IsPathRooted($relative) -or
        ($fromSession -notmatch '^\.\.(?:/|$)' -and -not [IO.Path]::IsPathRooted($fromSession))) {
        return 'AUTH_HOME_MUST_BE_DEDICATED'
    }
    $authFile = [IO.FileInfo]::new((Resolve-CirnetPath -Root $temp -Relative "$relative/auth.json"))
    if (-not $authFile.Exists -or $authFile.Length -gt 1MB -or $authFile.LinkType -eq 'HardLink') { return 'AUTH_FILE_UNAVAILABLE' }
    $authConfig = Resolve-CirnetPath -Root $temp -Relative "$relative/config.toml"
    if (Test-Path -LiteralPath $authConfig) { return 'AUTH_HOME_HAS_CONFIG' }
    $exe = Get-Command codex -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $exe) { return 'CODEX_CLI_UNAVAILABLE' }
    $info = [Diagnostics.ProcessStartInfo]::new($exe.Source)
    $info.UseShellExecute = $false
    $info.WorkingDirectory = $root
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.Environment['CODEX_HOME'] = $directory
    foreach ($key in @('CODEX_API_KEY','OPENAI_API_KEY')) { $null = $info.Environment.Remove($key) }
    # The role runner uses the same file-backed login command. --ignore-user-config is an exec option.
    foreach ($arg in @('-c','cli_auth_credentials_store="file"','login','status')) { $info.ArgumentList.Add($arg) }
    $process = [Diagnostics.Process]::Start($info)
    try {
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        if (-not $process.WaitForExit(10000)) { $process.Kill($true); return 'LOGIN_STATUS_TIMEOUT' }
        $null = $stdout.GetAwaiter().GetResult()
        $null = $stderr.GetAwaiter().GetResult()
        $process.ExitCode -eq 0 ? 'READY' : 'LOGIN_STATUS_FAILED'
    } finally { $process.Dispose() }
}

function Invoke-OfflineVerification {
    $checks = @(
        @{ program='node'; arguments=@('--test','tests/model-compatibility-source.test.mjs','tests/eval-codex-privacy.test.mjs','tests/behavioral-python.test.mjs','tests/eval-codex-rtl.test.mjs') }
        @{ program='pwsh'; arguments=@('-NoProfile','-NonInteractive','-File','tests/adaptive-stable-checkpoint.tests.ps1') }
        @{ program='pwsh'; arguments=@('-NoProfile','-NonInteractive','-File','tests/model-compatibility.tests.ps1') }
        @{ program='pwsh'; arguments=@('-NoProfile','-NonInteractive','-File','tests/behavioral-ab.tests.ps1') }
        @{ program='cargo'; arguments=@('test','--offline','--locked') }
        @{ program='cargo'; arguments=@('run','--quiet','--offline','--locked','--bin','check-docs') }
        @{ program='git'; arguments=@('diff','--check') }
        @{ program='git'; arguments=@('diff','--cached','--check') }
    )
    foreach ($check in $checks) {
        $arguments = $check.arguments
        Write-Host "Offline $($check.program) $($arguments -join ' ')"
        if (-not (Get-Command $check.program -CommandType Application -ErrorAction SilentlyContinue)) {
            $check.exitCode = 127
            Write-Host "Required offline executable is unavailable: $($check.program)."
            break
        }
        & $check.program @arguments | Out-Host
        $check.exitCode = $LASTEXITCODE
        if ($check.exitCode -ne 0) { break }
    }
    $checks
}

function Stage-Arguments($Stage) {
    if ($Stage.kind -eq 'Candidate') {
        $arguments = @{ OutputRoot=$root; Model='gpt-6.2-sol'; Effort='xhigh'; TimeoutSeconds=300 }
        if ($Stage.cases.Count -eq 1) { $arguments.CaseId = $Stage.cases[0] }
    } else {
        $arguments = @{ OutputRoot=$root; CurrentWorkRules=$true; CaseId=($Stage.cases -join ','); Repetitions=$Stage.repetitions;
            TimeoutSeconds=300; AuthorModel='gpt-6.2-sol'; WorkModel='gpt-6.2-sol'; AuthorEffort='xhigh'; WorkEffort='xhigh' }
    }
    $arguments
}

function Get-StagePreview($Stage, [string]$Runner, [hashtable]$Arguments, [string]$Key) {
    $preview = $checkpoint.previews[$Key]
    if (-not $preview) {
        $preview = @{ id=($Stage.name + '-preview-' + [guid]::NewGuid().ToString('N')) }
        & $Runner @Arguments -Session $preview.id | Out-Host
        $previewPath = Resolve-CirnetPath -Root $root -Relative "$($preview.id)/manifest.json"
        $preview.sha256 = Get-CirnetHash -Path $previewPath
        $checkpoint.previews[$Key] = $preview
        Write-Checkpoint
    }
    $previewPath = Resolve-CirnetPath -Root $root -Relative "$($preview.id)/manifest.json"
    if ((Get-CirnetHash -Path $previewPath) -cne $preview.sha256) { throw 'The frozen preview manifest changed.' }
    $manifest = Read-Json $previewPath
    $previewMode = $Stage.kind -eq 'Candidate' ? 'PREVIEW' : 'preview'
    if ($manifest['mode'] -cne $previewMode) { throw 'The preview is not a dry run.' }
    $manifest
}

function Get-Evidence($Stage, [string]$Directory) {
    $hashes = [ordered]@{}
    if ($Stage.kind -eq 'AB') {
        $index = Read-Json (Resolve-CirnetPath -Root $Directory -Relative 'evidence-index.json')
        if (-not $index -or -not $index['files'].Contains('summary.json')) { throw 'The A/B evidence index is incomplete.' }
        foreach ($name in $index['files'].Keys) {
            if ($name -cnotmatch '^(?:manifest\.json|summary\.json|evaluator/(?:cases\.json|checks\.mjs)|trials/E0[1-8]-r\d+-[\w.]+/(?:pair-trial\.json|(?:Author|Work)/(?:session\.json|E0[1-8]/(?:run\.json|prompt\.txt|final\.md|events\.jsonl|stderr\.log|before\.json|after\.json|changes\.json|diff\.patch|objective\.stderr))))$') {
                throw 'Unexpected path in the A/B evidence index.'
            }
            $file = Resolve-CirnetPath -Root $Directory -Relative $name
            $hashes[$name] = Get-CirnetHash -Path $file
            if ($hashes[$name] -cne $index['files'][$name]['sha256'] -or
                (Get-Item -LiteralPath $file).Length -ne $index['files'][$name]['bytes']) { throw 'Indexed A/B evidence changed.' }
        }
        $hashes['evidence-index.json'] = Get-CirnetHash -Path (Join-Path $Directory 'evidence-index.json')
    } else {
        $names = @('manifest.json','compatibility-summary.json')
        foreach ($id in $Stage.cases) {
            foreach ($role in @('Author','Work')) {
                $prefix = "trials/$id/$role"
                $names += "$prefix/session.json"
                foreach ($name in @('run.json','prompt.txt','final.md','events.jsonl','stderr.log','before.json',
                    'after.json','changes.json','diff.patch','objective.stderr')) { $names += "$prefix/$id/$name" }
            }
        }
        foreach ($name in $names) {
            $file = Resolve-CirnetPath -Root $Directory -Relative $name
            if ([IO.File]::Exists($file)) { $hashes[$name] = Get-CirnetHash -Path $file }
        }
    }
    $hashes
}

function Inspect-Attempt($Stage, $Attempt) {
    $dir = Resolve-CirnetPath -Root $root -Relative $Attempt.id
    $summaryName = $Stage.kind -eq 'Candidate' ? 'compatibility-summary.json' : 'summary.json'
    $summary = Read-Json (Resolve-CirnetPath -Root $dir -Relative $summaryName)
    if (-not $summary) { return 'INTERRUPTED' }
    $manifest = Read-Json (Resolve-CirnetPath -Root $dir -Relative 'manifest.json')
    if ((Get-StableManifestInputs $manifest $Stage.kind) -cne $Attempt.inputs) { return 'EVIDENCE_BLOCKED' }
    $evidence = Get-Evidence $Stage $dir
    $seal = Hash-Json $evidence
    if ($Attempt.evidenceSha256 -and $seal -cne $Attempt.evidenceSha256) { return 'EVIDENCE_BLOCKED' }
    $Attempt.evidenceSha256 = $seal
    if ($Stage.kind -eq 'Candidate') {
        if ($manifest['mode'] -cne 'NATIVE_COMPATIBILITY' -or $summary['mode'] -cne 'NATIVE_COMPATIBILITY' -or
            (($summary['cases'].id | Sort-Object) -join ',') -cne (($Stage.cases | Sort-Object) -join ',')) { return 'EVIDENCE_BLOCKED' }
        $roles = @($summary['cases'] | ForEach-Object { $_['author']; $_['work'] })
        if (@($roles).Where({ $_['source'] -ceq 'TEST_SHIM' -or ($_['invoked'] -and $_['source'] -cne 'NATIVE_CLI') }).Count) { return 'EVIDENCE_BLOCKED' }
        $Attempt.used = if (-not @($roles).Where({ -not $_['record'] }).Count) { @($roles).Where({ $_['invoked'] -eq $true }).Count }
        if (@($summary['cases']).Where({ $_['status'] -ceq 'FAIL' }).Count) { return 'REPAIR_REQUIRED' }
        foreach ($role in $roles) {
            if ($role['record'] -and $evidence.Contains($role['record'])) {
                $record = Read-Json (Resolve-CirnetPath -Root $dir -Relative $role['record'])
                if ($record['reason'] -ceq 'Codex exceeded the configured timeout.') { return 'REPAIR_REQUIRED' }
            }
        }
        if (@($summary['cases']).Where({ $_['status'] -cne 'OBJECTIVE_PASS_REVIEW_REQUIRED' -or $_['handoffShaMatches'] -ne $true }).Count -or
            @($roles).Where({ $_['execution'] -cne 'COMPLETED' -or $_['privacyHits'] -ne 0 }).Count) { return 'BLOCKED' }
        foreach ($id in $Stage.cases) {
            $author = $evidence["trials/$id/Author/$id/final.md"]
            if (-not $author -or $author -cne $evidence["trials/$id/Work/$id/prompt.txt"] -or
                -not $evidence["trials/$id/Author/$id/events.jsonl"] -or -not $evidence["trials/$id/Work/$id/events.jsonl"]) {
                return 'EVIDENCE_BLOCKED'
            }
            foreach ($role in @('Author','Work')) {
                $prefix = "trials/$id/$role"
                $session = Read-Json (Resolve-CirnetPath -Root $dir -Relative "$prefix/session.json")
                $record = Read-Json (Resolve-CirnetPath -Root $dir -Relative "$prefix/$id/run.json")
                $command = @($record['command'])
                $modelIndex = [array]::IndexOf($command, '--model')
                $roleHash = $role -ceq 'Author' ? $record['finalSha256'] : $record['manifest']['promptSha256']
                if ($session['timeoutSeconds'] -ne 300 -or -not $record['startUtc'] -or
                    $record['client']['kind'] -cne 'native-cli' -or $record['trace']['turnCompleted'] -ne 1 -or
                    $roleHash -cne $author -or $modelIndex -lt 0 -or $command[$modelIndex + 1] -cne $manifest['modelRequested'] -or
                    "model_reasoning_effort=`"$($manifest['effortRequested'])`"" -cnotin $command) { return 'EVIDENCE_BLOCKED' }
            }
        }
        $reviewPath = Resolve-CirnetPath -Root $dir -Relative 'candidate-review.json'
        $review = Read-Json $reviewPath
        if (-not $review -and -not ($Attempt.Contains('reviewTemplate'))) {
            $template = [ordered]@{ reviewer=''; evidenceSha256=$seal; cases=[ordered]@{} }
            foreach ($id in $Stage.cases) {
                $template.cases[$id] = [ordered]@{}
                foreach ($criterion in @('taskSuccess','architectureCompliance','verificationCoverage','privacy')) {
                    $template.cases[$id][$criterion] = @{status='NOT_RUN';reason='';evidence=@()}
                }
            }
            $templatePath = Resolve-CirnetPath -Root $dir -Relative 'candidate-review-template.json'
            if (-not [IO.File]::Exists($templatePath)) {
                [IO.File]::WriteAllText($templatePath, (ConvertTo-Json $template -Depth 12) + "`n", $utf8)
            }
            $Attempt.reviewTemplate = $templatePath
        }
        $decision = Get-StableCandidateReview $review $Stage $seal $evidence
    } else {
        $cases = @($summary['cases'])
        if ($manifest['mode'] -cne 'execute' -or $summary['workRulesProfile'] -cne 'Current' -or $summary['source'] -cnotin @('NATIVE','NOT_RUN') -or
            $cases.Count -ne ($Stage.calls / 2)) { return 'EVIDENCE_BLOCKED' }
        $used = @($cases | ForEach-Object { $_['roles']['author']; $_['roles']['work'] } |
            Where-Object { $_['invoked'] -eq $true }).Count - 2 * @($manifest['carriedTrials']).Count
        if ($used -lt 0 -or $used -gt $Attempt.reserved) { return 'EVIDENCE_BLOCKED' }
        if (-not $cases.Where({ $_['validity'] -cne 'VALID' }).Count) { $Attempt.used = $used }
        if ($cases.Where({ $_['cohort'] -cne 'A' -and
            ($_['verdict'] -ceq 'FAIL' -or $_['reason'] -cmatch 'Codex exceeded the configured timeout\.$') }).Count) { return 'REPAIR_REQUIRED' }
        if ($cases.Where({ $_['validity'] -cne 'VALID' -or $_['completed'] -ne $true -or $_['verdict'] -ceq 'BLOCKED' }).Count) { return 'BLOCKED' }
        $reviewPath = Resolve-CirnetPath -Root $dir -Relative 'ab-review.json'
        if (-not [IO.File]::Exists($reviewPath)) { return 'REVIEW_REQUIRED' }
        $digest = Get-CirnetHash -Path $reviewPath
        if ($Attempt['acceptedReviewSha256'] -and $Attempt['acceptedReviewSha256'] -cne $digest) { return 'EVIDENCE_BLOCKED' }
        if ($Attempt['gradedReviewDigest'] -cne $digest) {
            $Attempt.gradedFile = 'reviewed-' + [guid]::NewGuid().ToString('N') + '.json'
            $graded = Resolve-CirnetPath -Root $dir -Relative $Attempt.gradedFile
            & node (Join-Path $PSScriptRoot 'eval-codex-report.mjs') $dir $reviewPath $graded | Out-Host
            if ($LASTEXITCODE -ne 0) { return 'REVIEW_REQUIRED' }
            $Attempt.gradedReviewDigest = $digest
            $Attempt.reviewedSha256 = Get-CirnetHash -Path $graded
        }
        $graded = Resolve-CirnetPath -Root $dir -Relative $Attempt.gradedFile
        $gradedHash = Get-CirnetHash -Path $graded
        $report = Read-Json $graded
        $proof = Read-Json ($graded -replace '\.json$', '.sha256.json')
        if ($report['reviewSha256'] -cne $digest -or $proof['summary']['sha256'] -cne $gradedHash -or
            $proof['review']['sha256'] -cne $digest -or $proof['evidenceIndexSha256'] -cne $evidence['evidence-index.json'] -or
            ($Attempt['reviewedSha256'] -and $Attempt['reviewedSha256'] -cne $gradedHash)) { return 'EVIDENCE_BLOCKED' }
        $Attempt.reviewedSha256 = $gradedHash
        $decision = Get-StableAbReview $report $Stage.name
    }
    if ([IO.File]::Exists($reviewPath)) {
        $reviewHash = Get-CirnetHash -Path $reviewPath
        if ($Attempt['acceptedReviewSha256'] -and $Attempt['acceptedReviewSha256'] -cne $reviewHash) { return 'EVIDENCE_BLOCKED' }
        if ($decision -ceq 'PASS') { $Attempt.acceptedReviewSha256 = $reviewHash }
    }
    $decision
}

$lock = [IO.File]::Open($lockPath, [IO.FileMode]::OpenOrCreate, [IO.FileAccess]::ReadWrite, [IO.FileShare]::None)
Push-Location -LiteralPath $repo
try {
    $checkpoint = Read-Json $path
    if (-not $checkpoint) { $checkpoint = [ordered]@{schema=1; attempts=@(); offline=@{}; previews=@{} } }
    if ($checkpoint.schema -ne 1) { throw 'Unsupported checkpoint schema.' }
    $charged = Get-StableChargedCalls $checkpoint.attempts
    $revision = (Git-Input @('rev-parse','HEAD')).Trim()
    $source = Get-SourceInputs
    $checkpoint.sourceRevision = $revision
    $checkpoint.sourceSha256 = $source
    $checkpoint.sourceStatus = @(Git-Input @('status','--porcelain=v1','--untracked-files=all'))
    $offlineKey = "$revision/$source"
    $stage = @{ name='offline' }
    $offline = $checkpoint.offline[$offlineKey]
    if (-not $offline -or $offline['status'] -cne 'PASS') {
        Save-Status $stage 'VERIFYING' 'Running offline regression checks.'
        $checks = @(Invoke-OfflineVerification)
        $offlinePassed = -not $checks.Where({ $_['exitCode'] -ne 0 }).Count -and (Get-SourceInputs) -ceq $source
        $checkpoint.offline[$offlineKey] = @{status=($offlinePassed ? 'PASS' : 'FAIL'); checks=$checks}
        if (-not $offlinePassed) {
            Save-Status $stage 'REPAIR_REQUIRED' 'Fix the offline failure or restore a consistent source snapshot before resuming.'
            exit 1
        }
        Write-Checkpoint
    }

    $tools = [ordered]@{ codex=(Test-LocalTool 'codex' ''); node=(Test-LocalTool 'node' ''); git=(Test-LocalTool 'git' '');
        python=(Test-LocalTool 'python' $PythonToolPath); iverilog=(Test-LocalTool 'iverilog' $RtlToolPath); vvp=(Test-LocalTool 'vvp' $RtlToolPath) }
    $checkpoint.toolSha256 = $tools
    $login = 'NOT_RUN'
    $passed = [Collections.Generic.List[object]]::new()
    foreach ($stage in Get-StableStages) {
        $argsForStage = Stage-Arguments $stage
        $runner = Join-Path $PSScriptRoot ($stage.kind -eq 'Candidate' ? 'eval-codex-model-smoke.ps1' : 'eval-codex-ab.ps1')
        $runtime = [ordered]@{ codex=$tools.codex; node=$tools.node; git=$tools.git;
            os=[Runtime.InteropServices.RuntimeInformation]::OSDescription; pwsh=$PSVersionTable.PSVersion.ToString() }
        if ($stage.name -ceq 'p01') { $runtime.python = $tools.python }
        if ($stage.name -ceq 'full') { $runtime.iverilog = $tools.iverilog; $runtime.vvp = $tools.vvp }
        if ((Get-SourceInputs) -cne $source -or (Git-Input @('rev-parse','HEAD')).Trim() -cne $revision) {
            Save-Status $stage 'EVIDENCE_BLOCKED' 'Source changed during orchestration. Resume to prepare a new snapshot.'
            exit 2
        }
        $previewKey = Hash-Json @($revision,$source,$stage.name,$runtime)
        try {
            $manifest = Get-StagePreview $stage $runner $argsForStage $previewKey
            $key = Hash-Json @($stage.name,$runtime,(Get-StableManifestInputs $manifest $stage.kind))
        } catch {
            Save-Status $stage 'EVIDENCE_BLOCKED' 'Could not create or verify the preview manifest. Check the preview runner output.'
            exit 2
        }
        $attempt = @($checkpoint.attempts | Where-Object { $_.stage -ceq $stage.name -and $_.key -ceq $key }) | Select-Object -Last 1
        if ($attempt) {
            try { $status = Inspect-Attempt $stage $attempt } catch { $status = 'EVIDENCE_BLOCKED' }
            if ($attempt.used -gt $attempt.reserved) { throw 'Observed native calls exceed their checkpoint reservation.' }
            $charged = Get-StableChargedCalls $checkpoint.attempts
            if ($status -ceq 'PASS') { $passed.Add($attempt); Write-Checkpoint; continue }
            if ($status -cnotin @('BLOCKED','INTERRUPTED') -or -not $ResumeBlocked) {
                Save-AttemptStatus $stage $attempt $status
                exit (Get-StableExitCode $status -Interactive:$Interactive)
            }
            if ($stage.kind -eq 'AB' -and $status -ceq 'BLOCKED') {
                $argsForStage.RunPurpose = 'Continuation'
                $argsForStage.PriorSession = $attempt.id
                $previewKey = Hash-Json @($previewKey,$attempt.id,$attempt.evidenceSha256)
                try { $manifest = Get-StagePreview $stage $runner $argsForStage $previewKey }
                catch {
                    Save-Status $stage 'EVIDENCE_BLOCKED' 'Could not verify the continuation preview against the prior index.'
                    exit 2
                }
            }
        }
        $planned = $stage.kind -eq 'Candidate' ? $manifest['maximumRoleCalls'] : $manifest['maximumPlannedRoleCalls']
        if ($planned -lt 1 -or $planned -gt $stage.calls) { throw 'Unexpected native schedule size.' }
        $checkpoint.nextRoleCalls = $planned
        if (-not $Execute -or ($charged + $planned) -gt $ApprovedRoleCalls) {
            Save-Status $stage 'AUTHORIZATION_REQUIRED' "$planned roles need approval. Cumulative roles charged: $charged."
            exit (Get-StableExitCode 'AUTHORIZATION_REQUIRED' -Interactive:$Interactive)
        }
        $missingTools = @($runtime.Keys).Where({ -not $runtime[$_] })
        if ($missingTools.Count) {
            Save-Status $stage 'ENVIRONMENT_BLOCKED' "Required local tools are missing: $($missingTools -join ', ')."
            exit 2
        }
        if ($login -cne 'READY') {
            try { $login = Get-LoginStatus } catch { $login = 'LOGIN_STATUS_ERROR' }
        }
        if ($login -cne 'READY') {
            Save-Status $stage 'ENVIRONMENT_BLOCKED' "Dedicated CLI login preflight: $login."
            exit 2
        }
        if ((Get-SourceInputs) -cne $source -or (Git-Input @('rev-parse','HEAD')).Trim() -cne $revision) {
            Save-Status $stage 'EVIDENCE_BLOCKED' 'Checkout changed after preview. Resume to prepare a new snapshot.'
            exit 2
        }
        $attempt = [ordered]@{stage=$stage.name; key=$key; id=($stage.name + '-native-' + [guid]::NewGuid().ToString('N'));
            reserved=$planned; used=$null; evidenceSha256=$null; inputs=(Get-StableManifestInputs $manifest $stage.kind);
            previewSha256=$checkpoint.previews[$previewKey].sha256; approvedRoleCalls=$ApprovedRoleCalls}
        $checkpoint.attempts = @($checkpoint.attempts) + @($attempt)
        $charged = Get-StableChargedCalls $checkpoint.attempts
        Save-Status $stage 'RUNNING_NATIVE' "Reserved $planned role calls for $($attempt.id)."
        if ($stage.name -ceq 'p01' -and $PythonToolPath) { $argsForStage.ToolPath = $PythonToolPath }
        if ($stage.name -ceq 'full' -and $RtlToolPath) { $argsForStage.ToolPath = $RtlToolPath }
        $nativeArgs = @{ Session = $attempt.id; AuthHome = $AuthHome; Execute = $true; MaxNativeCalls = $planned }
        try {
            if ($Interactive) {
                & $runner @argsForStage @nativeArgs 6>&1 | ForEach-Object {
                    if ($_ -is [System.Management.Automation.InformationRecord] -and $_.Tags -contains 'PSHOST') {
                        Write-Host (Format-StableNativeMessage ([string]$_.MessageData))
                    } else { $_ | Out-Host }
                }
            } else { & $runner @argsForStage @nativeArgs | Out-Host }
        }
        catch {
            Save-Status $stage 'INTERRUPTED' "Inspect the owned logs for $($attempt.id). The reservation is retained."
            exit 2
        }
        try { $status = Inspect-Attempt $stage $attempt } catch { $status = 'EVIDENCE_BLOCKED' }
        if ($attempt.used -gt $attempt.reserved) { throw 'Observed native calls exceed their checkpoint reservation.' }
        $charged = Get-StableChargedCalls $checkpoint.attempts
        Save-AttemptStatus $stage $attempt $status
        exit (Get-StableExitCode $status -Interactive:$Interactive)
    }
    $stage = @{name='release'}
    if ((Get-SourceInputs) -cne $source) { Save-Status $stage 'EVIDENCE_BLOCKED' 'Source changed before acceptance.'; exit 2 }
    $report = Join-Path $root "final-acceptance-$($source.Substring(0, 16)).md"
    if (-not [IO.File]::Exists($report)) {
        $lines = @('# CIRNET Rules — Adaptive Stable Final Acceptance Report','',
            "Revision $revision","Source SHA-256 $source","Role calls charged $charged",'',
            '| Stage | Verdict | Evidence SHA-256 | Evidence |','| --- | --- | --- | --- |')
        foreach ($record in $passed) {
            $evidenceFile = $record['gradedFile'] ? $record['gradedFile'] : 'candidate-review.json'
            $summaryFile = $record['gradedFile'] ? 'summary.json' : 'compatibility-summary.json'
            $lines += "| $($record.stage) | PASS | $($record.evidenceSha256) | [Review]($($record.id)/$evidenceFile), [Summary]($($record.id)/$summaryFile) |"
        }
        $full = $passed[$passed.Count - 1]
        $assessment = Read-Json (Resolve-CirnetPath -Root $root -Relative "$($full.id)/$($full.gradedFile)")
        $lines += @('', '## Quality gates', '', '| Gate | Result |', '| --- | --- |')
        foreach ($i in 0..6) { $lines += "| G$i | $($assessment['qualityGates']["G$i"]) |" }
        $lines += @('',
            'The linked artifacts contain the role traces and evidence for Task Success, Architecture Compliance, Verification Coverage and Privacy.',
            'Owner approval is required to release Adaptive Stable.')
        [IO.File]::WriteAllText($report, ($lines -join "`n") + "`n", $utf8)
    }
    $checkpoint.acceptanceReport = $report
    Save-Status $stage 'ADAPTIVE_STABLE_READY' "Acceptance report $report"
    exit 0
} finally {
    Pop-Location
    $lock.Dispose()
}

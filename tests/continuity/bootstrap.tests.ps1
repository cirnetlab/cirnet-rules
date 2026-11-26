#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$manager = Join-Path $repo 'scripts/task-continuity.ps1'
$output = [IO.Path]::TrimEndingDirectorySeparator([IO.Path]::GetFullPath([IO.Path]::GetTempPath()))
$fixture = Join-Path $output ('bootstrap-test-' + [guid]::NewGuid().ToString('N'))
$worktree = Join-Path $fixture 'primary'

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Assert-Rejected([scriptblock]$Action, [string]$Reason) {
    try { & $Action | Out-Null }
    catch {
        if ($_.Exception.Message -like "*$Reason*") { return }
        throw
    }
    throw "Expected rejection containing '$Reason'."
}

function Invoke-FixtureGit([string]$Root, [string[]]$GitArgs) {
    $result = & git -C $Root -c "core.hooksPath=$fixture/absent-hooks" -c core.autocrlf=false -c commit.gpgsign=false @GitArgs
    if ($LASTEXITCODE -ne 0) { throw "Fixture Git failed: $($GitArgs[0])." }
    $result
}

function Initialize-Fixture([string]$Root) {
    [void][IO.Directory]::CreateDirectory($Root)
    Invoke-FixtureGit $Root @('init','--quiet','--template=','-b','main')
    [IO.File]::WriteAllLines((Join-Path $Root '.gitignore'), @('/.cirnet/tasks/','/.local/'))
    foreach ($name in @('plan.md','spec.md','architecture.md','AGENTS.md')) {
        [IO.File]::WriteAllText((Join-Path $Root $name), "# Fixture $name")
    }
    Invoke-FixtureGit $Root @('add','--all')
    Invoke-FixtureGit $Root @('-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','--quiet','-m','bootstrap fixture')
}

$publisher = $null
try {
    Initialize-Fixture $worktree
    $scope = @{ Repository=$worktree; AgentFamily='codex'; ConfirmScope=$true }
    Assert-Rejected { & $manager discover -Repository $worktree -AgentFamily codex } 'ConfirmScope'
    Assert-True ((& $manager discover @scope).Status -ceq 'NONE') 'Missing default store did not return NONE.'
    Assert-True (-not [IO.Directory]::Exists((Join-Path $worktree '.cirnet'))) 'Read-only discovery created storage.'
    Assert-Rejected { & $manager discover @scope -CheckpointRoot '.git' } 'Git administrative'
    Assert-Rejected { & $manager discover @scope -CheckpointRoot 'plan.md' } 'Git-ignored'
    Assert-Rejected { & $manager discover @scope -CheckpointRoot '../unapproved' } 'path component'
    if ($IsWindows) {
        Assert-Rejected { & $manager discover @scope -CheckpointRoot '\\unapproved\share' } 'require local'
    }

    $taskFile = Join-Path $fixture 'task.md'
    [IO.File]::WriteAllText($taskFile, @'
## Original Task
Recover the intended fixture task.
## Objectives
Select by verified identity.
## Acceptance Criteria
- Observe a successful local test.
## Architecture & Rules
Read pinned fixture instructions.
## Scope & Authorization
Operate in the isolated fixture.
## Plans & Specs
Read the owning plan and spec.
'@)
    $progressFile = Join-Path $fixture 'progress.md'
    [IO.File]::WriteAllText($progressFile, @'
## Completed Work
Prepared the fixture.
## Pending Work
Verify bootstrap.
## Decisions & Blockers
Use exact worktree identity.
## Next Action
Run the local check.
## Re-verification
Review the latest evidence.
'@)
    $referencesFile = Join-Path $fixture 'refs.json'
    [IO.File]::WriteAllText($referencesFile, '{"documents":[{"repository":"primary","kind":"plan","path":"plan.md"},{"repository":"primary","kind":"spec","path":"spec.md"},{"repository":"primary","kind":"architecture","path":"architecture.md"},{"repository":"primary","kind":"rules","path":"AGENTS.md"}],"relatedRepositories":[]}')
    $seed = @{ TaskFile=$taskFile; ProgressFile=$progressFile; ReferencesFile=$referencesFile; RequiredCheck=@('fixture-test') }

    $configured = Join-Path $fixture 'central-storage'
    [void][IO.Directory]::CreateDirectory($configured)
    $linkedRoot = Join-Path $fixture 'linked'
    $previousStorageRoot = $env:CIRNET_LOCAL_STORAGE_ROOT
    Invoke-FixtureGit $worktree @('remote','add','origin','https://github.com/cirnetlab/cirnet-rules.git')
    Invoke-FixtureGit $worktree @('worktree','add','--detach','--quiet',$linkedRoot)
    try {
        $env:CIRNET_LOCAL_STORAGE_ROOT = $configured
        $central = & $manager save @scope @seed -TaskId central
        $centralTask = Join-Path $configured '.cirnet/tasks/cirnet-rules/central'
        Assert-True ([IO.File]::Exists((Join-Path $centralTask 'current.json'))) 'Canonical Git origin did not select the namespaced checkpoint.'
        Assert-True ((& $manager bootstrap @scope -TaskId central).CheckpointId -ceq $central.CheckpointId) 'Central checkpoint recovery failed.'
        $linked = @{ Repository=$linkedRoot; AgentFamily='codex'; ConfirmScope=$true }
        Assert-Rejected { & $manager resume @linked -TaskId central } 'different worktree'
        Assert-Rejected { & $manager save @linked @seed -TaskId central } 'different worktree'
        Assert-True ((& $manager status @scope -TaskId central).CheckpointId -ceq $central.CheckpointId) 'Foreign save altered the checkpoint.'
        $env:CIRNET_LOCAL_STORAGE_ROOT = Join-Path $configured 'missing'
        Assert-Rejected { & $manager discover @scope } 'unavailable'
        $env:CIRNET_LOCAL_STORAGE_ROOT = $linkedRoot
        Assert-Rejected { & $manager discover @scope } 'outside Git checkouts'
    }
    finally {
        $env:CIRNET_LOCAL_STORAGE_ROOT = $previousStorageRoot
        Invoke-FixtureGit $worktree @('worktree','remove',$linkedRoot)
        Invoke-FixtureGit $worktree @('remote','remove','origin')
    }

    $first = & $manager save @scope @seed -TaskId first
    $discovery = & $manager discover @scope
    Assert-True ($discovery.Status -ceq 'SELECTED' -and $discovery.TaskId -ceq 'first' -and
        -not $discovery.PSObject.Properties['Task']) 'Discovery did not return compact metadata for the unique legacy task.'
    $recovered = & $manager bootstrap @scope
    Assert-True ($recovered.ContextRecovered -and $recovered.Task -ceq [IO.File]::ReadAllText($taskFile) -and
        $recovered.Progress -ceq [IO.File]::ReadAllText($progressFile) -and -not $recovered.CanContinue -and
        $recovered.Git.Worktree -ceq $worktree -and $recovered.Git.Current.branch -ceq 'refs/heads/main' -and
        $recovered.Git.Current.head -ceq (Invoke-FixtureGit $worktree @('rev-parse','HEAD')) -and
        $recovered.Git.Recorded.head -ceq $recovered.Git.Current.head -and
        -not $recovered.Git.Recorded.Contains('files')) 'Bootstrap lost sealed notes or Git identity, leaked the full inventory or granted scope.'
    Assert-Rejected { & $manager bootstrap @scope -ConfirmContinuation } 'ExpectedCheckpointId'
    $ready = & $manager bootstrap @scope -ConfirmContinuation -ExpectedCheckpointId $recovered.CheckpointId
    Assert-True ($ready.CanContinue -and $ready.CheckpointId -ceq $first.CheckpointId) 'Confirmed unique resume failed.'
    $unknown = & $manager bootstrap @scope -SessionKey unseen-conversation -ConfirmContinuation -ExpectedCheckpointId $first.CheckpointId
    Assert-True (-not $unknown.ContextRecovered -and -not $unknown.CanContinue -and $unknown.Candidates.Count -eq 1) 'An unknown session recovered the only worktree task.'
    $null = & $manager bind @scope -TaskId first
    Assert-True ((& $manager bootstrap @scope).TaskId -ceq 'first') 'Verified worktree binding lost the sole eligible task.'
    $defaultStore = Join-Path $worktree '.cirnet/tasks'
    $damagedStore = Join-Path $defaultStore 'Damaged'
    [void][IO.Directory]::CreateDirectory($damagedStore)
    $damagedPointer = Join-Path $damagedStore 'current.json'
    [IO.File]::WriteAllText($damagedPointer, '{')
    $blocked = & $manager bootstrap @scope -ConfirmContinuation -ExpectedCheckpointId $first.CheckpointId
    Assert-True ($blocked.Status -ceq 'BLOCKED' -and $blocked.RejectedCheckpoints -eq 1 -and
        $blocked.Candidates.Count -eq 1 -and -not $blocked.ContextRecovered -and -not $blocked.CanContinue) 'An unreadable or malformed candidate was silently excluded.'
    Assert-True ((& $manager bootstrap @scope -TaskId first).ContextRecovered) 'Unrelated damage prevented verified explicit recovery.'
    [IO.File]::Delete($damagedPointer)
    [IO.Directory]::Delete($damagedStore)

    $otherFamily = $scope.Clone()
    $otherFamily.AgentFamily = 'claude-code'
    Assert-True ((& $manager discover @otherFamily).Status -ceq 'NONE') 'Another agent family saw the Codex task.'
    $null = & $manager save @otherFamily @seed -TaskId claude
    Assert-True ((& $manager bootstrap @otherFamily).TaskId -ceq 'claude') 'Claude family selection failed.'
    Import-Module (Join-Path $repo 'scripts/continuity/Repository.psm1')
    Import-Module (Join-Path $repo 'scripts/continuity/Checkpoint.psm1')
    Import-Module (Join-Path $repo 'scripts/continuity/Discovery.psm1')
    $checkout = Get-ContinuityRepository $worktree
    $remaining = 8
    Assert-Rejected { Read-ContinuityCheckpoint (Join-Path $defaultStore 'first') first -RemainingBytes ([ref]$remaining) } 'scan payload budget'
    $beforePublication = Get-ContinuitySelection $checkout $defaultStore codex '' ''
    Assert-True (Test-ContinuitySelectionCurrent $checkout $defaultStore codex '' $beforePublication) 'Stable implicit selection failed revalidation.'
    # Publish between selection and revalidation to exercise the bootstrap race boundary deterministically.
    $second = & $manager save @scope @seed -TaskId second -SessionKey initialized-lane
    Assert-True (-not (Test-ContinuitySelectionCurrent $checkout $defaultStore codex '' $beforePublication)) 'Concurrent task creation passed implicit selection revalidation.'
    Assert-True ((& $manager bootstrap @scope -SessionKey initialized-lane -ConfirmContinuation -ExpectedCheckpointId $second.CheckpointId).CanContinue) 'The first save did not publish a durable session binding.'
    Assert-Rejected { & $manager save -Repository $worktree -AgentFamily codex @seed -TaskId unconfirmed -SessionKey unconfirmed } 'ConfirmScope'
    Assert-Rejected { & $manager save @scope -TaskId second -SessionKey unexpected } 'only at initialization'
    Assert-Rejected { & $manager save @scope @seed -TaskId conflicting -SessionKey initialized-lane } 'already bound'
    Assert-True (-not [IO.File]::Exists((Join-Path $defaultStore 'conflicting/current.json'))) 'Conflicting initialization published an unbound task.'
    $ambiguous = & $manager bootstrap @scope
    Assert-True ($ambiguous.Status -ceq 'AMBIGUOUS' -and $ambiguous.Candidates.Count -eq 2 -and
        -not $ambiguous.PSObject.Properties['Progress']) 'More than one task should require selection without loading notes.'
    Assert-True ((& $manager bootstrap @scope -TaskId second).TaskId -ceq 'second') 'Explicit selection failed.'

    $unknown = & $manager bootstrap @scope -SessionKey missing
    Assert-True ($unknown.Status -ceq 'AMBIGUOUS' -and $unknown.Candidates.Count -eq 2 -and -not $unknown.ContextRecovered) 'Unknown session fell back to the worktree binding.'
    $null = & $manager bind @scope -TaskId second -SessionKey local-lane
    Assert-True ((& $manager bootstrap @scope -SessionKey local-lane).TaskId -ceq 'second') 'Specific session binding did not take precedence.'
    Assert-Rejected { & $manager bind @scope -TaskId first -SessionKey local-lane } 'already bound'
    $null = & $manager bind @scope -TaskId first -SessionKey local-lane -ReplaceBinding
    Assert-True ((& $manager discover @scope -SessionKey local-lane).TaskId -ceq 'first') 'Explicit binding replacement did not take effect.'

    $bindingPath = (& $manager bind @scope -TaskId first -SessionKey local-lane).BindingPath
    $bindingBytes = [IO.File]::ReadAllBytes($bindingPath)
    $otherBinding = (& $manager bind @scope -TaskId second -SessionKey initialized-lane).BindingPath
    [IO.File]::WriteAllBytes($bindingPath,[IO.File]::ReadAllBytes($otherBinding))
    Assert-True ((& $manager bootstrap @scope -SessionKey local-lane).Status -ceq 'BLOCKED') 'A binding copied from another session was accepted.'
    [IO.File]::WriteAllBytes($bindingPath,$bindingBytes)
    $lockPath = Join-Path (Split-Path -Parent $bindingPath) '.lock'
    $bindingLock = [IO.File]::Open($lockPath, 'Open', 'ReadWrite', 'None')
    try {
        Assert-Rejected { & $manager bind @scope -TaskId second -SessionKey local-lane -ReplaceBinding } 'busy'
        Assert-True ((& $manager discover @scope -SessionKey local-lane).TaskId -ceq 'first') 'Binding read lost the last published record.'
    }
    finally { $bindingLock.Dispose() }
    $binding = [IO.File]::ReadAllText($bindingPath) | ConvertFrom-Json -AsHashtable
    $binding.taskKey = '0' * 32
    [IO.File]::WriteAllText($bindingPath, ($binding | ConvertTo-Json -Compress))
    Assert-True ((& $manager bootstrap @scope -SessionKey local-lane).Status -ceq 'BLOCKED') 'Stale binding fell through to another task.'
    [IO.File]::WriteAllText($bindingPath,'{')
    Assert-True ((& $manager discover @scope -SessionKey local-lane).Status -ceq 'BLOCKED') 'Corrupt binding fell through.'
    [IO.File]::WriteAllBytes($bindingPath,$bindingBytes)

    $secondPointer = Join-Path $defaultStore 'second/current.json'
    $pointerLock = [IO.File]::Open($secondPointer, 'Open', 'ReadWrite', 'None')
    try {
        $locked = & $manager bootstrap @scope
        Assert-True ($locked.Status -ceq 'BLOCKED' -and -not $locked.ContextRecovered) 'An unreadable checkpoint allowed implicit recovery.'
    }
    finally { $pointerLock.Dispose() }
    $pointerBytes = [IO.File]::ReadAllBytes($secondPointer)
    [IO.File]::WriteAllText($secondPointer,'{')
    Assert-True ((& $manager bootstrap @scope -TaskId second).Status -ceq 'BLOCKED') 'Corrupt checkpoint was recovered.'
    [IO.File]::WriteAllBytes($secondPointer,$pointerBytes)

    $confirmSecond = @{ ConfirmContinuation = $true; ExpectedCheckpointId = $second.CheckpointId }
    $duplicateCheckpointId = '{"checkpointId":"' + ('0' * 32) + '",'
    $secondManifest = Join-Path $second.CheckpointDirectory 'manifest.json'
    $boundSelection = Get-ContinuitySelection $checkout $defaultStore codex '' 'initialized-lane'
    foreach ($checkpointRecord in @($secondPointer, $secondManifest)) {
        $originalBytes = [IO.File]::ReadAllBytes($checkpointRecord)
        $originalJson = [Text.Encoding]::UTF8.GetString($originalBytes)
        [IO.File]::WriteAllText($checkpointRecord, $duplicateCheckpointId + $originalJson.Substring(1))
        try {
            $duplicate = & $manager bootstrap @scope @confirmSecond -SessionKey initialized-lane
            Assert-True ($duplicate.Status -ceq 'BLOCKED' -and -not $duplicate.ContextRecovered -and -not $duplicate.CanContinue) 'A checkpoint record with contradictory duplicate IDs authorized continuation.'
            Assert-True ((& $manager discover @scope).Status -ceq 'BLOCKED') 'Implicit discovery ignored a checkpoint record with duplicate IDs.'
            Assert-True (-not (Test-ContinuitySelectionCurrent $checkout $defaultStore codex 'initialized-lane' $boundSelection)) 'A duplicate ID added during recovery passed checkpoint revalidation.'
        }
        finally { [IO.File]::WriteAllBytes($checkpointRecord, $originalBytes) }
    }

    $shared = Join-Path $fixture 'shared'
    $linked = Join-Path $fixture 'linked'
    $foreign = Join-Path $fixture 'foreign'
    Initialize-Fixture $foreign
    Invoke-FixtureGit $worktree @('worktree','add','--quiet','--detach',$linked)
    $null = & $manager save @scope @seed -TaskId primary -CheckpointRoot $shared
    $alternate = @{ ConfirmScope=$true; AgentFamily='codex'; CheckpointRoot=$shared }
    Assert-True ((& $manager discover @alternate -Repository $linked).Status -ceq 'NONE') 'Sibling worktree inherited a task.'
    Assert-True ((& $manager bootstrap @alternate -Repository $foreign -TaskId primary).Status -ceq 'BLOCKED') 'Foreign repository recovered the task.'
    $null = & $manager save @alternate @seed -Repository $linked -TaskId linked
    Assert-True ((& $manager bootstrap @alternate -Repository $linked).TaskId -ceq 'linked') 'Shared root mixed separate worktrees.'
    $null = & $manager save @scope @seed -TaskId custom -CheckpointRoot '.local/tasks'
    Assert-True ((& $manager bootstrap @scope -CheckpointRoot '.local/tasks').TaskId -ceq 'custom') 'Configured relative storage was ignored.'
    $customStore = Join-Path $worktree '.local/tasks'
    for ($i = 1; $i -le 128; $i++) { [IO.File]::WriteAllText((Join-Path $customStore "entry-$i"), '') }
    $overLimit = & $manager bootstrap @scope -CheckpointRoot '.local/tasks'
    Assert-True ($overLimit.Status -ceq 'BLOCKED' -and -not $overLimit.ContextRecovered) 'An oversized root allowed automatic selection.'
    Assert-True ((& $manager bootstrap @scope -CheckpointRoot '.local/tasks' -TaskId custom).ContextRecovered) 'Scan bounds prevented explicit recovery.'
    $linkPath = Join-Path $worktree '.local/redirected'
    $linkCreated = $false
    try {
        $null = [IO.Directory]::CreateSymbolicLink($linkPath, $shared)
        $linkCreated = $true
    }
    catch {
        if (-not $IsWindows -or $_.Exception.GetBaseException() -isnot [IO.IOException]) { throw }
        Write-Host 'NOT_RUN symbolic checkpoint root fixture (Windows permission).'
    }
    if ($linkCreated) {
        Assert-Rejected { & $manager discover @scope -CheckpointRoot '.local/redirected' } 'link is forbidden'
    }

    $runArgs = @{ Repository=$worktree; TaskId='first'; AgentFamily='codex'; ConfirmScope=$true }
    $boundScope = @{ Repository=$worktree; AgentFamily='codex'; ConfirmScope=$true; SessionKey='local-lane' }
    $runningFile = Join-Path $worktree '.local/running'
    $releaseFile = Join-Path $worktree '.local/release'
    [void][IO.Directory]::CreateDirectory((Split-Path -Parent $runningFile))
    $checkScript = Join-Path $fixture 'check.ps1'
    [IO.File]::WriteAllText($checkScript, @'
$ErrorActionPreference = 'Stop'
[IO.File]::WriteAllText((Join-Path $PWD '.local/running'), 'ready')
while (-not [IO.File]::Exists((Join-Path $PWD '.local/release'))) { Start-Sleep -Milliseconds 50 }
'@)
    $publishScript = Join-Path $fixture 'publish.ps1'
    [IO.File]::WriteAllText($publishScript, @'
param($Manager, $Check)
$ErrorActionPreference = 'Stop'
& $Manager run -TaskId first -AgentFamily codex -ConfirmScope -Kind test -EvidenceId fixture-test -Executable pwsh -Arguments @(
    '-NoProfile','-File',$Check
) -TimeoutSeconds 45
exit $LASTEXITCODE
'@)
    # Relaunch the current shell even when PATH exposes several pwsh executables.
    $info = [Diagnostics.ProcessStartInfo]::new((Get-Process -Id $PID).Path)
    $info.WorkingDirectory = $worktree
    $info.UseShellExecute = $false
    $info.RedirectStandardOutput = $info.RedirectStandardError = $true
    foreach ($arg in @('-NoProfile','-File',$publishScript,$manager,$checkScript)) {
        $info.ArgumentList.Add($arg)
    }
    $publisher = [Diagnostics.Process]::Start($info)
    $publisherOutput = $publisher.StandardOutput.ReadToEndAsync()
    $publisherError = $publisher.StandardError.ReadToEndAsync()
    $deadline = [DateTime]::UtcNow.AddSeconds(30)
    while (-not [IO.File]::Exists($runningFile) -and -not $publisher.HasExited -and [DateTime]::UtcNow -lt $deadline) {
        Start-Sleep -Milliseconds 100
    }
    if (-not [IO.File]::Exists($runningFile)) {
        if (-not $publisher.HasExited) { throw 'Concurrent check did not start before the deadline.' }
        throw "Concurrent check did not start (exit $($publisher.ExitCode)). $($publisherError.GetAwaiter().GetResult()) $($publisherOutput.GetAwaiter().GetResult())"
    }
    $during = & $manager bootstrap @boundScope
    Assert-True ($during.ContextRecovered -and $during.Checks[0].Phase -ceq 'running' -and
        -not $during.Ready -and -not $during.CorrectiveReady) 'Running milestone did not block bootstrap.'
    Assert-Rejected { & $manager bind @scope -TaskId first -SessionKey busy } 'busy'
    [IO.File]::WriteAllText($releaseFile,'release')
    if (-not $publisher.WaitForExit(30000)) { throw 'Concurrent publication timed out.' }
    if ($publisher.ExitCode -ne 0) { throw "Concurrent publication failed. $($publisherError.GetAwaiter().GetResult()) $($publisherOutput.GetAwaiter().GetResult())" }
    $driftedSelection = & $manager bootstrap @boundScope -ConfirmContinuation -ExpectedCheckpointId $during.CheckpointId
    Assert-True ($driftedSelection.Status -ceq 'BLOCKED' -and -not $driftedSelection.PSObject.Properties['Task']) 'Bootstrap used a superseded checkpoint.'
    $latest = & $manager bootstrap @boundScope
    Assert-True ($latest.Ready -and $latest.Checks[0].Status -ceq 'PASS') 'Latest checkpoint was not recovered.'
    $compactReport = & $manager bootstrap @boundScope -Compact
    Assert-True ($compactReport.Task -ceq $latest.Task -and $compactReport.Progress -ceq $latest.Progress -and
        $compactReport.Acceptance[0].Criterion -ceq $latest.Acceptance[0].Criterion -and
        $compactReport.Checks[0].Status -ceq 'PASS' -and -not $compactReport.Checks[0].PSObject.Properties['Evidence']) 'Compact recovery lost authoritative context or included raw evidence.'
    [IO.File]::WriteAllText((Join-Path $worktree 'untracked.txt'),'user edit')
    $changed = & $manager bootstrap @boundScope -ConfirmContinuation -ExpectedCheckpointId $latest.CheckpointId
    Assert-True (-not $changed.CanContinue -and $changed.Git.Changes.Files['untracked.txt'] -ceq 'added' -and
        $changed.Checks[0].Status -ceq 'NOT_RUN') 'Untracked Git drift or stale evidence was missed.'
    $savedDrift = & $manager save @runArgs -AcceptGitDrift
    $corrective = & $manager bootstrap @boundScope -ConfirmContinuation -ExpectedCheckpointId $savedDrift.CheckpointId
    Assert-True (-not $corrective.CanContinue -and $corrective.CanCorrect) 'Reconciled stale evidence lost authorized corrective mode.'
    $null = & $manager run @runArgs -Kind test -EvidenceId fixture-test -Executable pwsh -Arguments @(
        '-NoProfile','-Command','Start-Sleep -Seconds 20'
    ) -TimeoutSeconds 1
    $interrupted = & $manager bootstrap @boundScope
    Assert-True (-not $interrupted.Ready -and $interrupted.Checks[0].Phase -ceq 'interrupted') 'Interrupted milestone was accepted.'
    $file = Join-Path $worktree 'AGENTS.md'
    [IO.File]::AppendAllText($file,' instructions updated')
    $changedRules = & $manager bootstrap @boundScope
    Assert-True ($changedRules.ReferenceProblems.Count -gt 0 -and -not $changedRules.Ready) 'Changed pinned rules did not block bootstrap.'

    Write-Host 'PASS offline Task Continuity discovery, binding and bootstrap.'
}
finally {
    if ($publisher) {
        if (-not $publisher.HasExited) {
            [IO.File]::WriteAllText($releaseFile, 'cleanup')
            if (-not $publisher.WaitForExit(5000)) {
                $publisher.Kill($true)
                $null = $publisher.WaitForExit(5000)
            }
        }
        $publisher.Dispose()
    }
    $resolved = [IO.Path]::GetFullPath($fixture)
    if ([IO.Path]::GetDirectoryName($resolved) -cne [IO.Path]::GetFullPath($output) -or
        [IO.Path]::GetFileName($resolved) -cnotmatch '^bootstrap-test-[a-f0-9]{32}$') { throw 'Unexpected fixture cleanup path.' }
    if ([IO.Directory]::Exists($resolved)) { Remove-Item -LiteralPath $resolved -Recurse -Force }
}
exit 0

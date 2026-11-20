#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$manager = Join-Path $repo 'scripts/task-continuity.ps1'
$output = Join-Path $repo 'target'
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

    $otherFamily = $scope.Clone()
    $otherFamily.AgentFamily = 'claude-code'
    Assert-True ((& $manager discover @otherFamily).Status -ceq 'NONE') 'Another agent family saw the Codex task.'
    $null = & $manager save @otherFamily @seed -TaskId claude
    Assert-True ((& $manager bootstrap @otherFamily).TaskId -ceq 'claude') 'Claude family selection failed.'
    $null = & $manager save @scope @seed -TaskId second
    $ambiguous = & $manager bootstrap @scope
    Assert-True ($ambiguous.Status -ceq 'AMBIGUOUS' -and $ambiguous.Candidates.Count -eq 2 -and
        -not $ambiguous.PSObject.Properties['Progress']) 'More than one task should require selection without loading notes.'
    Assert-True ((& $manager bootstrap @scope -TaskId second).TaskId -ceq 'second') 'Explicit selection failed.'

    $null = & $manager bind @scope -TaskId first
    Assert-True ((& $manager bootstrap @scope -SessionKey missing).TaskId -ceq 'first') 'Worktree binding fallback failed.'
    $null = & $manager bind @scope -TaskId second -SessionKey local-lane
    Assert-True ((& $manager bootstrap @scope -SessionKey local-lane).TaskId -ceq 'second') 'Specific session binding did not take precedence.'
    Assert-Rejected { & $manager bind @scope -TaskId first -SessionKey local-lane } 'already bound'
    $null = & $manager bind @scope -TaskId first -SessionKey local-lane -ReplaceBinding
    Assert-True ((& $manager discover @scope -SessionKey local-lane).TaskId -ceq 'first') 'Explicit binding replacement did not take effect.'

    $defaultStore = Join-Path $worktree '.cirnet/tasks'
    $bindingPath = (& $manager bind @scope -TaskId first -SessionKey local-lane).BindingPath
    $bindingBytes = [IO.File]::ReadAllBytes($bindingPath)
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
    $pointerBytes = [IO.File]::ReadAllBytes($secondPointer)
    [IO.File]::WriteAllText($secondPointer,'{')
    Assert-True ((& $manager bootstrap @scope -TaskId second).Status -ceq 'BLOCKED') 'Corrupt checkpoint was recovered.'
    [IO.File]::WriteAllBytes($secondPointer,$pointerBytes)

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
    $info = [Diagnostics.ProcessStartInfo]::new((Get-Command pwsh -CommandType Application).Path)
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
    $during = & $manager bootstrap @scope
    Assert-True ($during.ContextRecovered -and $during.Checks[0].Phase -ceq 'running' -and
        -not $during.Ready -and -not $during.CorrectiveReady) 'Running milestone did not block bootstrap.'
    Assert-Rejected { & $manager bind @scope -TaskId first -SessionKey busy } 'busy'
    [IO.File]::WriteAllText($releaseFile,'release')
    if (-not $publisher.WaitForExit(30000)) { throw 'Concurrent publication timed out.' }
    if ($publisher.ExitCode -ne 0) { throw "Concurrent publication failed. $($publisherError.GetAwaiter().GetResult()) $($publisherOutput.GetAwaiter().GetResult())" }
    $driftedSelection = & $manager bootstrap @scope -ConfirmContinuation -ExpectedCheckpointId $during.CheckpointId
    Assert-True ($driftedSelection.Status -ceq 'BLOCKED' -and -not $driftedSelection.PSObject.Properties['Task']) 'Bootstrap used a superseded checkpoint.'
    $latest = & $manager bootstrap @scope
    Assert-True ($latest.Ready -and $latest.Checks[0].Status -ceq 'PASS') 'Latest checkpoint was not recovered.'
    [IO.File]::WriteAllText((Join-Path $worktree 'untracked.txt'),'user edit')
    $changed = & $manager bootstrap @scope -ConfirmContinuation -ExpectedCheckpointId $latest.CheckpointId
    Assert-True (-not $changed.CanContinue -and $changed.Git.Changes.Files['untracked.txt'] -ceq 'added' -and
        $changed.Checks[0].Status -ceq 'NOT_RUN') 'Untracked Git drift or stale evidence was missed.'
    $savedDrift = & $manager save @runArgs -AcceptGitDrift
    $corrective = & $manager bootstrap @scope -ConfirmContinuation -ExpectedCheckpointId $savedDrift.CheckpointId
    Assert-True (-not $corrective.CanContinue -and $corrective.CanCorrect) 'Reconciled stale evidence lost authorized corrective mode.'
    $null = & $manager run @runArgs -Kind test -EvidenceId fixture-test -Executable pwsh -Arguments @(
        '-NoProfile','-Command','Start-Sleep -Seconds 20'
    ) -TimeoutSeconds 1
    $interrupted = & $manager bootstrap @scope
    Assert-True (-not $interrupted.Ready -and $interrupted.Checks[0].Phase -ceq 'interrupted') 'Interrupted milestone was accepted.'
    $file = Join-Path $worktree 'AGENTS.md'
    [IO.File]::AppendAllText($file,' instructions updated')
    $changedRules = & $manager bootstrap @scope
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

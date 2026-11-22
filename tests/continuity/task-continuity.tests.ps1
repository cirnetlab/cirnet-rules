#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$manager = Join-Path $repo 'scripts/task-continuity.ps1'
$output = Join-Path $repo 'target'
$fixture = Join-Path $output ('continuity-test-' + [guid]::NewGuid().ToString('N'))
$worktree = Join-Path $fixture 'primary'
$utf8 = [Text.UTF8Encoding]::new($false)

function Assert-True([bool]$Condition, [string]$Message) {
    if (-not $Condition) { throw $Message }
}

function Invoke-FixtureGit([string[]]$GitArgs, [string]$Root = $worktree) {
    $hooks = Join-Path $fixture 'absent-hooks'
    $result = & git -C $Root -c "core.hooksPath=$hooks" -c core.autocrlf=false -c core.fsmonitor=false @GitArgs
    if ($LASTEXITCODE -ne 0) { throw "Fixture Git failed: $($GitArgs[0])" }
    $result
}

function Write-Fixture([string]$Path, [string]$Content) {
    [IO.File]::WriteAllText((Join-Path $fixture $Path), $Content, $utf8)
}

function Set-FixtureUnmerged([string]$Root, [string]$Path) {
    $blob = Invoke-FixtureGit @('rev-parse', "HEAD:$Path") $Root
    $info = [Diagnostics.ProcessStartInfo]::new('git')
    $info.WorkingDirectory = $Root
    $info.UseShellExecute = $false
    $info.RedirectStandardInput = $true
    $info.StandardInputEncoding = $utf8
    foreach ($arg in @('update-index','--index-info')) { $info.ArgumentList.Add($arg) }
    $process = [Diagnostics.Process]::Start($info)
    try {
        $process.StandardInput.Write("0 $('0' * $blob.Length)`t$Path`n")
        foreach ($stage in 1..3) { $process.StandardInput.Write("100644 $blob $stage`t$Path`n") }
        $process.StandardInput.Close()
        $process.WaitForExit()
        if ($process.ExitCode -ne 0) { throw 'Could not create unmerged fixture index entries.' }
    }
    finally { $process.Dispose() }
    Assert-True (@(Invoke-FixtureGit @('ls-files','--unmerged','--',$Path) $Root).Count -eq 3) 'Missing fixture conflict stages.'
}

function Assert-Rejected([scriptblock]$Action, [string]$Expected) {
    try { & $Action | Out-Null }
    catch {
        if ($_.Exception.Message -like "*$Expected*") { return }
        throw
    }
    throw "Expected rejection containing '$Expected'."
}

Assert-True ([IO.File]::Exists($manager)) 'Task Continuity manager entry point is missing.'
try {
    [void][IO.Directory]::CreateDirectory($worktree)
    Invoke-FixtureGit @('init', '--quiet', '--template=', '-b', 'main')
    Write-Fixture 'primary/.gitignore' "/.cirnet/tasks/`n/.local/`n"
    Write-Fixture 'primary/plan.md' "# Existing plan`n"
    Write-Fixture 'primary/spec.md' "# Existing spec`n"
    Write-Fixture 'primary/architecture.md' "# Architecture`n"
    Write-Fixture 'primary/AGENTS.md' "# Instructions`n"
    Write-Fixture 'primary/source.txt' "base`n"
    Write-Fixture 'primary/staged.txt' "base`n"
    Invoke-FixtureGit @('add', '--all')
    Invoke-FixtureGit @('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', '-c', 'commit.gpgsign=false', 'commit', '--quiet', '-m', 'base')
    Write-Fixture 'primary/source.txt' "unstaged`n"
    Write-Fixture 'primary/staged.txt' "staged`n"
    Invoke-FixtureGit @('add', '--', 'staged.txt')
    Write-Fixture 'primary/untracked note.txt' "untracked`n"

    Write-Fixture 'task.md' @'
## Original Task
Make the fixture retain its state after interruption.
## Objectives
Recover the same task in a new session.
## Acceptance Criteria
- The fixture verification process exits successfully.
- The index and owning Plan and Spec retain their bytes.
## Architecture & Rules
Use the referenced architecture and instructions.
## Scope & Authorization
The fixture operator may read and edit the primary checkout.
## Plans & Specs
Consult the pinned plan and spec.
'@
    Write-Fixture 'acceptance.json' '[{"id":"fixture-exits","checks":["fixture-test"]},{"id":"ownership-preserved","checks":[]}]'
    Write-Fixture 'progress.md' @'
## Completed Work
Prepared the fixture.
## Pending Work
Verify the fixture before closure.
## Decisions & Blockers
Use the primary checkout. No blocker recorded.
## Next Action
Run the fixture verification command.
## Re-verification
Review the process evidence after any change.
'@
    Write-Fixture 'refs.json' @'
{
  "documents": [
    {"kind": "plan", "repository": "primary", "path": "plan.md"},
    {"kind": "spec", "repository": "primary", "path": "spec.md"},
    {"kind": "architecture", "repository": "primary", "path": "architecture.md"},
    {"kind": "rules", "repository": "primary", "path": "AGENTS.md"}
  ],
  "relatedRepositories": []
}
'@

    $index = Join-Path $worktree '.git/index'
    $indexHash = (Get-FileHash -LiteralPath $index).Hash
    $planHash = (Get-FileHash -LiteralPath (Join-Path $worktree 'plan.md')).Hash
    $specHash = (Get-FileHash -LiteralPath (Join-Path $worktree 'spec.md')).Hash
    $seed = @{
        TaskFile = Join-Path $fixture 'task.md'
        ProgressFile = Join-Path $fixture 'progress.md'
        ReferencesFile = Join-Path $fixture 'refs.json'
        AcceptanceFile = Join-Path $fixture 'acceptance.json'
        ConfirmScope = $true
    }
    $codex = @{ Repository = $worktree; TaskId = 'first-task'; AgentFamily = 'codex' }
    $secretTask = $codex.Clone()
    $secretTask.TaskId = 'private-input'
    $secretSeed = $seed.Clone()
    $secretSeed.TaskFile = Join-Path $fixture 'redact-task.md'
    Write-Fixture 'redact-task.md' ((Get-Content -LiteralPath $seed.TaskFile -Raw) + [Environment]::NewLine + '{"password":"fixture-only-value"}')
    Assert-Rejected { & $manager save @secretTask @secretSeed } 'credential'
    $privateStore = Join-Path $worktree '.cirnet/tasks/private-input'
    Assert-True (-not [IO.Directory]::Exists((Join-Path $privateStore 'snapshots'))) 'Rejected notes were persisted.'
    $privateName = 'token=fixture-only-value.txt'
    Write-Fixture "primary/$privateName" 'ordinary bytes'
    try {
        Assert-Rejected { & $manager save @secretTask @seed } 'credential'
        Assert-True (-not [IO.Directory]::Exists((Join-Path $privateStore 'snapshots'))) 'A credential-bearing Git name was persisted.'
    }
    finally { [IO.File]::Delete((Join-Path $worktree $privateName)) }
    Write-Fixture 'primary/token' 'ordinary marker'
    $saved = & $manager save @codex @seed
    $firstId = $saved.CheckpointId
    Assert-True ($firstId -match '^[0-9a-f]{32}$') 'Save did not confirm a checkpoint.'
    $snapshots = Split-Path -Parent $saved.CheckpointDirectory
    $store = Split-Path -Parent $snapshots
    $partial = Join-Path $snapshots ([guid]::NewGuid().ToString('N'))
    [void][IO.Directory]::CreateDirectory($partial)
    [IO.File]::WriteAllText((Join-Path $partial 'state.json'), '{', $utf8)
    $lock = [IO.File]::Open((Join-Path $store '.lock'), 'Open', 'ReadWrite', 'None')
    try {
        Assert-Rejected { & $manager save @codex } 'checkpoint is busy'
        Assert-True ((& $manager status @codex).CheckpointId -ceq $firstId) 'An incomplete or competing save displaced the confirmed checkpoint.'
    }
    finally { $lock.Dispose() }

    $savedProgress = Join-Path $saved.CheckpointDirectory 'progress.md'
    $originalProgress = [IO.File]::ReadAllBytes($savedProgress)
    try {
        [IO.File]::AppendAllText($savedProgress, "`nUnexpected change`n")
        Assert-Rejected { & $manager resume @codex -ConfirmScope } 'Checkpoint hash mismatch'
    }
    finally { [IO.File]::WriteAllBytes($savedProgress, $originalProgress) }
    $resumed = & $manager resume @codex -ConfirmScope
    Assert-True ($resumed.Task -like '*Make the fixture retain*') 'Original task was not restored.'
    Assert-True (-not $resumed.CanContinue -and (& $manager resume @codex -ConfirmScope -ConfirmContinuation).CanContinue) 'Continuation did not require its own scope confirmation.'
    Assert-True ($resumed.Progress -like '*Run the fixture verification command*') 'Next action was not restored.'
    Assert-True ($resumed.Git.Recorded.staged -ccontains 'staged.txt') 'Staged file was omitted.'
    Assert-True ($resumed.Git.Recorded.unstaged -ccontains 'source.txt') 'Unstaged file was omitted.'
    Assert-True ($resumed.Git.Recorded.untracked -ccontains 'untracked note.txt') 'Untracked file was omitted.'
    Assert-True ($resumed.References.documents.Count -eq 4) 'Pinned references were omitted.'
    Assert-True ($resumed.IntegrityStatus -ceq 'PASS' -and $resumed.Status -ceq 'NOT_RUN' -and
        $resumed.AgentFamilyMatched -and -not $resumed.AgentIdentityVerified) 'Integrity or a declared family was confused with verification.'
    # Make the final pointer rotation fail after a complete candidate has been written.
    $current = Join-Path $store 'current.json'
    $previous = Join-Path $store 'previous.json'
    $firstPointer = [IO.File]::ReadAllText($current)
    [void][IO.Directory]::CreateDirectory($previous)
    $publicationFailed = $false
    try { $null = & $manager save @codex }
    catch [IO.IOException], [UnauthorizedAccessException] { $publicationFailed = $true }
    finally { [IO.Directory]::Delete($previous) }
    Assert-True ($publicationFailed -and (& $manager status @codex).CheckpointId -ceq $firstId) 'Failed pointer publication displaced the confirmed checkpoint.'
    $incomplete = & $manager verify @codex
    Assert-True ($LASTEXITCODE -eq 1 -and $incomplete.Ready -and $incomplete.Status -ceq 'NOT_RUN') 'Missing evidence did not fail overall verification.'
    $editedProgress = $resumed.Progress.Replace('Use the primary checkout. No blocker recorded.', 'Use the primary checkout. Fixture decision reviewed.')
    Write-Fixture 'updated-progress.md' $editedProgress
    $progressArgs = @{ ProgressFile = Join-Path $fixture 'updated-progress.md'; ConfirmScope = $true }
    Assert-Rejected { & $manager save @codex @progressArgs } 'ExpectedCheckpointId'
    Assert-Rejected { & $manager save @codex @progressArgs -ExpectedCheckpointId $firstId } 'ConfirmProgressChange'
    $updated = & $manager save @codex @progressArgs -ExpectedCheckpointId $firstId -ConfirmProgressChange -ProgressReason 'Owner reviewed the fixture decision.'
    $updatedStatus = & $manager status @codex
    $audit = $updatedStatus.ProgressReconciliation
    Assert-True ($audit.priorCheckpointId -ceq $firstId -and $audit.sections -ccontains 'Decisions & Blockers' -and
        [IO.File]::ReadAllText($savedProgress) -ceq $resumed.Progress -and $updated.CheckpointId -cne $firstId) 'Progress reconciliation lost its prior decision.'
    Assert-True ($updatedStatus.CheckpointId -ceq $updated.CheckpointId -and [IO.File]::ReadAllText($previous) -ceq $firstPointer) 'Successful publication did not rotate the confirmed pointers.'
    if ($IsWindows) {
        $currentPointer = [IO.File]::ReadAllText($current)
        $reader = [IO.File]::Open($current, 'Open', 'Read', 'Read')
        $publicationFailed = $false
        try { $null = & $manager save @codex }
        catch [IO.IOException], [UnauthorizedAccessException] { $publicationFailed = $true }
        finally { $reader.Dispose() }
        Assert-True ($publicationFailed -and [IO.File]::ReadAllText($current) -ceq $currentPointer -and
            [IO.File]::ReadAllText($previous) -ceq $firstPointer) 'Failed replacement lost a confirmed current or recovery pointer.'
        Assert-True ([IO.Directory]::GetFiles($store, '.pending-*').Count -eq 0) 'Failed replacement left an unpublished candidate pointer.'
    }
    $currentBytes = [IO.File]::ReadAllBytes($current)
    try {
        [IO.File]::WriteAllText($current, '{')
        Assert-Rejected { & $manager resume @codex -ConfirmScope } 'Invalid continuity JSON'
        $prior = & $manager resume @codex -ConfirmScope -RecoverPrevious
        Assert-True ($prior.CheckpointId -ceq $firstId -and $prior.ContextRecovered -and
            $prior.Status -ceq 'BLOCKED' -and -not $prior.CanContinue -and -not $prior.CanCorrect -and
            $prior.Progress -ceq $resumed.Progress) 'Recovery selected an unpublished candidate or authorized stale work.'
        [IO.File]::Delete($current)
        Assert-Rejected { & $manager save @codex @seed } 'confirmed checkpoint'
        Assert-Rejected { & $manager resume -Repository $worktree -TaskId $codex.TaskId -AgentFamily claude-code -RecoverPrevious -ConfirmScope } 'Agent family'
        Assert-True ((& $manager resume @codex -ConfirmScope -RecoverPrevious).CheckpointId -ceq $firstId) 'Missing current pointer prevented recovery.'
    }
    finally { [IO.File]::WriteAllBytes($current, $currentBytes) }
    Assert-Rejected { & $manager resume @codex } 'ConfirmScope'
    Assert-Rejected { & $manager resume -Repository $worktree -TaskId 'first-task' -AgentFamily 'claude-code' -ConfirmScope } 'Agent family'
    $legacy = @{ Repository = $worktree; TaskId = 'legacy-task'; AgentFamily = 'codex' }
    $legacySeed = $seed.Clone()
    $legacySeed.Remove('AcceptanceFile')
    $legacySeed.RequiredCheck = @('fixture-test')
    $null = & $manager save @legacy @legacySeed
    $legacyResume = & $manager resume @legacy -ConfirmScope
    Assert-True ($legacyResume.ContextRecovered -and -not $legacyResume.AcceptanceReady) 'Legacy notes or pending acceptance were lost.'
    Assert-Rejected { & $manager close @legacy -ConfirmScope -ConfirmAcceptance } 'save -AcceptanceFile'
    $null = & $manager save @legacy -ConfirmScope -AcceptanceFile $seed.AcceptanceFile
    Assert-True ((& $manager status @legacy).Acceptance.Count -eq 2) 'Legacy criteria were not declared on upgrade.'

    # Build a sealed v2 fixture with the historical string-review representation.
    Import-Module (Join-Path $repo 'scripts/continuity/Checkpoint.psm1')
    $legacyStore = Join-Path (Split-Path -Parent $store) $legacy.TaskId
    $historical = Read-ContinuityCheckpoint $legacyStore $legacy.TaskId
    $historical.State.schemaVersion = 2
    $historical.State.Remove('completedReview')
    $historical.State.Remove('progressReconciliation')
    $criterion = $historical.State.criteria[1]
    $historical.State.reviews[$criterion.id] = @{
        status = 'PASS'; evidence = 'Legacy operator statement.'
        basis = Get-ContinuityReviewBasis $historical.State $criterion
        reviewedAtUnixMs = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    }
    $v2 = Save-ContinuityCheckpoint $legacyStore $historical.State $historical.Task $historical.Progress
    $historicalStatus = & $manager resume @legacy -ConfirmScope
    Assert-True ($historicalStatus.Acceptance[1].ReviewOrigin -ceq 'legacy-statement' -and $historicalStatus.Acceptance[1].Status -ceq 'NOT_RUN') 'A historical v2 statement supplied current evidence.'
    $migrated = & $manager save @legacy
    $v3 = Read-ContinuityCheckpoint $legacyStore $legacy.TaskId
    Assert-True ($v3.State.schemaVersion -eq 3 -and $v3.State.taskKey -ceq $v2.State.taskKey -and $v3.Task -ceq $v2.Task -and
        $migrated.CheckpointId -cne $v2.State.checkpointId -and [IO.File]::Exists((Join-Path $v2.Directory 'manifest.json'))) 'v2 migration lost identity, original requirements or historical evidence.'

    $related = Join-Path $fixture 'related'
    [void][IO.Directory]::CreateDirectory($related)
    Invoke-FixtureGit @('init', '--quiet', '--template=', '-b', 'main') $related
    Write-Fixture 'related/spec.md' "# Related spec`n"
    Invoke-FixtureGit @('add', '--', 'spec.md') $related
    $commitArgs = @('-c', 'user.name=Fixture', '-c', 'user.email=fixture@example.invalid', '-c', 'commit.gpgsign=false', 'commit', '--quiet')
    Invoke-FixtureGit ($commitArgs + @('-m', 'related base')) $related
    $relatedHead = Invoke-FixtureGit @('rev-parse', 'HEAD') $related
    $relatedRefs = Get-Content -Raw -LiteralPath $seed.ReferencesFile | ConvertFrom-Json -AsHashtable
    $relatedRefs.documents += @{ kind = 'spec'; repository = 'related'; path = 'spec.md' }
    $relatedRefs.relatedRepositories += @{ id = 'related'; revision = $relatedHead; access = 'read' }
    Write-Fixture 'related-refs.json' (ConvertTo-Json -InputObject $relatedRefs -Depth 4)
    $relatedSeed = $seed.Clone()
    $relatedSeed.ReferencesFile = Join-Path $fixture 'related-refs.json'
    $claude = @{ Repository = $worktree; TaskId = 'second-task'; AgentFamily = 'claude-code'; CheckpointRoot = '.local/tasks' }
    Assert-Rejected { & $manager save @claude @relatedSeed } 'currently authorized -RelatedRoot'
    $claude.RelatedRoot = @{ related = $related }
    $null = & $manager save @claude @relatedSeed
    Assert-True ((& $manager resume @claude -ConfirmScope).Ready) 'Claude Code checkpoint did not resume.'
    Set-FixtureUnmerged $related 'spec.md'
    $conflictIndex = (Get-FileHash -LiteralPath (Join-Path $related '.git/index')).Hash
    $conflict = & $manager resume @claude -ConfirmScope -ConfirmContinuation
    Assert-True (-not $conflict.CanCorrect -and -not $conflict.CanContinue -and
        $conflict.ReferenceProblems -like '*unmerged*') 'Unmerged related Spec was treated as an agreed reference.'
    Assert-True ((Get-FileHash -LiteralPath (Join-Path $related '.git/index')).Hash -ceq $conflictIndex) 'Checking a related conflict changed its index.'
    Invoke-FixtureGit @('add', '--', 'spec.md') $related
    $claudeExport = Join-Path $fixture 'claude-handoff.json'
    $bundle = & $manager export @claude -ConfirmScope -ConfirmExport -OutputFile $claudeExport
    Assert-True ((& $manager resume -HandoffFile $claudeExport -ExpectedSha256 $bundle.Sha256 -AgentFamily claude-code -ConfirmScope).ContextRecovered) 'Same-family Claude Code handoff failed.'
    Assert-Rejected { & $manager resume -HandoffFile $claudeExport -ExpectedSha256 $bundle.Sha256 -AgentFamily codex -ConfirmScope } 'incompatible agent family'
    $claudeWithoutRelated = $claude.Clone()
    $claudeWithoutRelated.Remove('RelatedRoot')
    $missingAccess = & $manager resume @claudeWithoutRelated -ConfirmScope
    Assert-True ($missingAccess.RecoveryOnly -and $missingAccess.ReferenceProblems -like '*currently authorized -RelatedRoot*') 'Missing related access did not block continuation.'
    Write-Fixture 'related/spec.md' "# Changed related spec`n"
    $referenceDrift = & $manager resume @claude -ConfirmScope
    Assert-True ($referenceDrift.RecoveryOnly -and $referenceDrift.ReferenceProblems -like '*spec reference changed*') 'Changed document did not block continuation.'
    Invoke-FixtureGit ($commitArgs + @('--allow-empty', '-m', 'new related revision')) $related
    $relatedDrift = & $manager resume @claude -ConfirmScope
    Assert-True ($relatedDrift.RecoveryOnly -and $relatedDrift.ReferenceProblems -like '*differs from its selected revision*') 'Wrong related revision did not block continuation.'
    $wrongCheckout = & $manager verify -Repository $related -CheckpointRoot (Split-Path -Parent $store) -TaskId 'first-task'
    Assert-True (-not $wrongCheckout.Ready -and $wrongCheckout.Git.Changes.Fields -ccontains 'worktree') 'Different worktree was not reported.'
    Assert-Rejected {
        & $manager resume -Repository $related -CheckpointRoot (Split-Path -Parent $store) -TaskId 'first-task' -AgentFamily codex -ConfirmScope
    } 'different worktree'
    $visible = $claude.Clone()
    $visible.CheckpointRoot = 'visible'
    Assert-Rejected { & $manager save @visible @seed } 'CheckpointRoot'

    Write-Fixture 'primary/untracked note.txt' "changed since checkpoint`n"
    $drift = & $manager verify @codex
    Assert-True (-not $drift.Ready) 'Verification accepted an unrecorded change.'
    Assert-True ($drift.Git.Changes.Files['untracked note.txt'] -ceq 'modified') 'Unrecorded bytes were not classified.'
    $recovery = & $manager resume @codex -ConfirmScope -ConfirmContinuation
    Assert-True ($recovery.ContextRecovered -and -not $recovery.CanContinue -and $recovery.Task -eq $resumed.Task) 'Git drift prevented read-only task recovery.'
    Assert-Rejected { & $manager save @codex } 'AcceptGitDrift'
    Write-Fixture 'primary/untracked note.txt' "untracked`n"
    Assert-True ((& $manager verify @codex).Ready) 'Restored Git bytes did not verify.'

    Assert-Rejected { & $manager close @codex -ConfirmScope -ConfirmAcceptance } 'fixture-exits=NOT_RUN'
    $powershell = (Get-Process -Id $PID).Path
    [void][IO.Directory]::CreateDirectory((Join-Path $worktree '.local'))
    $crashTask = $codex.Clone()
    $crashTask.TaskId = 'interrupted-task'
    $null = & $manager save @crashTask @seed
    Write-Fixture 'run-handoff.ps1' @'
param($Manager, $Worktree, $Shell)
$childCommand = '[IO.File]::WriteAllText(".local/handoff-started", [string]$PID); Start-Sleep -Seconds 120'
& $Manager run -Repository $Worktree -TaskId interrupted-task -AgentFamily codex -ConfirmScope -Kind handoff -EvidenceId handoff -Executable $Shell -Arguments @('-NoProfile','-Command',$childCommand)
'@
    $start = [Diagnostics.ProcessStartInfo]::new()
    $start.FileName = $powershell
    $start.UseShellExecute = $false
    $start.CreateNoWindow = $true
    foreach ($arg in @('-NoProfile','-File',(Join-Path $fixture 'run-handoff.ps1'),$manager,$worktree,$powershell)) { $start.ArgumentList.Add($arg) }
    $controller = [Diagnostics.Process]::Start($start)
    $child = $null
    try {
        $deadline = [DateTime]::UtcNow.AddSeconds(45)
        $marker = Join-Path $worktree '.local/handoff-started'
        while (-not [IO.File]::Exists($marker) -and -not $controller.HasExited -and [DateTime]::UtcNow -lt $deadline) { Start-Sleep -Milliseconds 100 }
        Assert-True ([IO.File]::Exists($marker)) 'The controlled child did not start.'
        $running = & $manager resume @crashTask -ConfirmScope -ConfirmContinuation
        Assert-True ($running.Checks[0].Phase -ceq 'running' -and -not $running.CanCorrect) 'The pre-handoff checkpoint was not published before starting the child.'
        $child = [Diagnostics.Process]::GetProcessById([int][IO.File]::ReadAllText($marker))
    }
    finally {
        try { if (-not $controller.HasExited) { $controller.Kill($true); $controller.WaitForExit() } }
        finally {
            $controller.Dispose()
            if ($null -ne $child) {
                try { Assert-True ($child.WaitForExit(10000)) 'The fixture child was still running after interruption.' }
                finally { $child.Dispose() }
            }
        }
    }
    $afterKill = & $manager resume @crashTask -ConfirmScope -ConfirmContinuation
    Assert-True ($afterKill.CheckpointId -ceq $running.CheckpointId -and $afterKill.Checks[0].Status -ceq 'NOT_RUN' -and -not $afterKill.CanContinue) 'Killing the manager manufactured completed evidence.'
    $null = & $manager save @crashTask -ConfirmScope -ConfirmStopped
    $stopped = & $manager resume @crashTask -ConfirmScope -ConfirmContinuation
    Assert-True ($stopped.CanCorrect -and $stopped.Checks[0].Phase -ceq 'interrupted' -and $null -eq $stopped.Checks[0].ExitCode) 'Stopped acknowledgement did not preserve the interrupted result.'
    $sent = & $manager run @crashTask -ConfirmScope -Kind handoff -EvidenceId handoff -Executable $powershell -Arguments @('-NoProfile','-Command','exit 0')
    Assert-True ($sent.Status -ceq 'PASS' -and $sent.CheckpointId -cne $stopped.CheckpointId -and
        (& $manager status @crashTask).Checks[0].Phase -ceq 'finished') 'Controlled handoff did not publish completion.'
    $edit = & $manager run @crashTask -ConfirmScope -Kind edit -EvidenceId controlled-edit -Executable $powershell -Arguments @('-NoProfile','-Command','[IO.File]::WriteAllText("step.txt","controlled edit")')
    $afterEdit = & $manager status @crashTask
    $beforeEdit = Read-ContinuityCheckpoint (Split-Path -Parent (Split-Path -Parent $afterEdit.CheckpointDirectory)) $crashTask.TaskId -Previous
    Assert-True ($edit.Status -ceq 'PASS' -and $beforeEdit.State.checks['controlled-edit'].phase -ceq 'running' -and
        -not $beforeEdit.State.git.files.Contains('step.txt') -and $afterEdit.Git.Matches -and
        $afterEdit.Git.Recorded.untracked -ccontains 'step.txt') 'Controlled editing did not preserve both Git states.'
    [IO.File]::Delete((Join-Path $worktree 'step.txt'))
    $null = & $manager save @crashTask -ConfirmScope -AcceptGitDrift
    $tool = if ($IsWindows) { (Get-Command cmd.exe -CommandType Application | Select-Object -First 1).Path } else { '/bin/true' }
    $toolCopy = Join-Path $worktree ('.local/probe' + [IO.Path]::GetExtension($tool))
    Copy-Item -LiteralPath $tool -Destination $toolCopy
    $toolArgs = if ($IsWindows) { @('/d','/c','exit 0') } else { @('--version') }
    $probe = & $manager run @crashTask -ConfirmScope -Kind test -EvidenceId tool-check -Executable $toolCopy -Arguments @($toolArgs)
    Assert-True ($probe.Status -ceq 'PASS') 'The fixture tool did not return a successful exit.'
    [IO.File]::AppendAllText($toolCopy, 'changed fixture tool')
    $changedTool = (& $manager status @crashTask).Checks | Where-Object Id -CEQ 'tool-check'
    Assert-True ($changedTool.Status -ceq 'NOT_RUN') 'Different executable bytes retained PASS.'
    Write-Fixture 'primary/.local/input.txt' "expected`n"
    $check = @{ EvidenceId = 'fixture-test'; Kind = 'test'; Executable = $powershell; InputPath = @('.local/input.txt'); ConfirmScope = $true }
    $failed = & $manager run @codex @check -Arguments @('-NoProfile', '-Command', 'exit 17')
    Assert-True ($failed.Status -ceq 'FAIL' -and $failed.ExitCode -eq 17 -and $LASTEXITCODE -eq 17) 'Run lost the observed failure exit code.'
    $failedVerification = & $manager verify @codex
    Assert-True ($LASTEXITCODE -eq 1 -and $failedVerification.Status -ceq 'FAIL' -and $failedVerification.Ready) 'A required failure was hidden or blocked corrective work.'
    $interrupted = & $manager run @codex @check -TimeoutSeconds 1 -Arguments @('-NoProfile', '-Command', 'Start-Sleep -Seconds 30')
    Assert-True ($interrupted.Status -ceq 'NOT_RUN' -and $interrupted.Phase -ceq 'interrupted' -and
        $interrupted.Reason -ceq 'timeout' -and $null -eq $interrupted.ExitCode) 'A timed-out process supplied completed evidence.'
    $interruptedRecovery = & $manager resume @codex -ConfirmScope
    Assert-True ($interruptedRecovery.ContextRecovered -and -not $interruptedRecovery.CanContinue -and
        $interruptedRecovery.Checks[0].Status -ceq 'NOT_RUN') 'Interrupted process did not remain read-only and unverified.'
    $passed = & $manager run @codex @check -Arguments @('-NoProfile', '-Command', 'exit 0')
    Assert-True ($passed.Status -ceq 'PASS' -and $passed.ExitCode -eq 0) 'Completed process did not provide PASS evidence.'
    $envName = 'CIRNET_FIXTURE_' + [guid]::NewGuid().ToString('N').ToUpperInvariant()
    [Environment]::SetEnvironmentVariable($envName, 'fixture-development-setting')
    try {
        $built = & $manager run @codex -ConfirmScope -Kind build -EvidenceId fixture-build -Executable $powershell -Arguments @('-NoProfile','-Command','exit 0') -EnvironmentName $envName -InputPath '.local/input.txt'
        Assert-True ($built.Status -ceq 'PASS' -and -not ($built.Evidence | ConvertTo-Json -Depth 8).Contains('fixture-development-setting')) 'Declared environment was not hashed for the build.'
        [Environment]::SetEnvironmentVariable($envName, 'fixture-release-setting')
        $buildStatus = (& $manager status @codex).Checks | Where-Object Id -CEQ 'fixture-build'
        Assert-True ($buildStatus.Status -ceq 'NOT_RUN' -and $buildStatus.Reason -ceq 're-verification-required') 'An environment change retained build evidence.'
    }
    finally { [Environment]::SetEnvironmentVariable($envName, $null) }
    Write-Fixture 'primary/.local/input.txt' "changed`n"
    $stale = & $manager verify @codex
    Assert-True (-not $stale.Ready -and $stale.Checks[0].Status -ceq 'NOT_RUN') 'Changed ignored input retained PASS.'
    Assert-Rejected { & $manager close @codex -ConfirmScope -ConfirmAcceptance } 'fixture-exits=NOT_RUN'
    $corrective = & $manager resume @codex -ConfirmScope -ConfirmContinuation
    Assert-True (-not $corrective.CanContinue -and $corrective.CanCorrect) 'A stale result blocked an authorized correction.'
    $repair = & $manager run @codex -ConfirmScope -Corrective -Kind edit -EvidenceId 'fixture-repair' -Executable $powershell -Arguments @('-NoProfile', '-Command', 'exit 0')
    Assert-True ($repair.Status -ceq 'PASS') 'The corrective command could not run with stale observations.'
    $null = & $manager run @codex @check -Arguments @('-NoProfile', '-Command', 'exit 0')
    Assert-Rejected { & $manager close @codex -ConfirmScope -ConfirmAcceptance } 'ownership-preserved=NOT_RUN'

    $destination = Join-Path $fixture 'handoff.json'
    $siblingDestination = Join-Path (Split-Path -Parent $store) 'legacy-task/foreign.json'
    Assert-Rejected { & $manager export @codex -ConfirmScope -ConfirmExport -OutputFile $siblingDestination } 'outside the checkpoint store'
    Assert-True (-not [IO.File]::Exists($siblingDestination)) 'Export wrote into another task directory.'
    $privateNotes = $seed.Clone()
    Write-Fixture 'private-progress.md' ((Get-Content -Raw -LiteralPath $seed.ProgressFile) + "`nSee /srv/private/customer for the fixture.")
    $privateNotes.ProgressFile = Join-Path $fixture 'private-progress.md'
    $null = & $manager save @secretTask @privateNotes
    $privateExport = Join-Path $fixture 'private-export.json'
    Assert-Rejected { & $manager export @secretTask -ConfirmScope -ConfirmExport -OutputFile $privateExport } 'private path'
    Assert-True (-not [IO.File]::Exists($privateExport)) 'Rejected portable context was written.'
    Assert-Rejected { & $manager export @codex -ConfirmScope -OutputFile $destination } 'ConfirmExport'
    $handoff = & $manager export @codex -ConfirmScope -ConfirmExport -OutputFile $destination
    $package = & $manager resume -HandoffFile $destination -ExpectedSha256 $handoff.Sha256 -AgentFamily codex -ConfirmScope
    Assert-True ($package.IntegrityVerified -and $package.Status -ceq 'NOT_RUN' -and $package.ContextRecovered -and -not $package.CanContinue -and
        $package.Handoff.taskId -ceq 'first-task' -and
        $package.Handoff.task['Original Task'] -like '*retain its state*') 'Portable task context or read-only restrictions were lost.'
    Assert-Rejected { & $manager verify -HandoffFile $destination -ExpectedSha256 $handoff.Sha256 -AgentFamily claude-code } 'incompatible agent family'
    $payload = [IO.File]::ReadAllText($destination)
    Assert-True ($payload -notmatch '(?i)untracked note|input\.txt|exit 0' -and
        -not $payload.Contains($worktree.Replace('\', '\\'))) 'Export disclosed local paths or raw commands.'
    [IO.File]::AppendAllText($destination, ' ')
    Assert-Rejected { & $manager verify -HandoffFile $destination -ExpectedSha256 $handoff.Sha256 -AgentFamily codex } 'hash mismatch'

    Assert-True ((Get-FileHash -LiteralPath $index).Hash -ceq $indexHash) 'Manager changed the Git index.'
    Assert-True ((Get-FileHash -LiteralPath (Join-Path $worktree 'plan.md')).Hash -ceq $planHash) 'Manager changed the plan.'
    Assert-True ((Get-FileHash -LiteralPath (Join-Path $worktree 'spec.md')).Hash -ceq $specHash) 'Manager changed the spec.'
    $artifact = '.local/review-evidence.txt'
    $artifactText = "Fixture check and repository preservation observations.`n"
    Write-Fixture "primary/$artifact" $artifactText
    $evidence = @(@{ path = $artifact; sha256 = (Get-FileHash -LiteralPath (Join-Path $worktree $artifact)).Hash.ToLowerInvariant() })
    $review = @{
        criteria = @(
            @{ id = 'fixture-exits'; status = 'PASS'; evidence = $evidence }
            @{ id = 'ownership-preserved'; status = 'PASS'; evidence = $evidence }
        )
        completedWork = @{ status = 'PASS'; evidence = $evidence }
    }
    Write-Fixture 'reviews.json' (ConvertTo-Json -InputObject $review -Depth 7)
    $reviewArgs = @{
        ConfirmScope = $true
        ReviewFile = Join-Path $fixture 'reviews.json'
        ExpectedCheckpointId = (& $manager status @codex).CheckpointId
    }
    Assert-Rejected { & $manager review @codex @reviewArgs } 'ConfirmHumanReview'
    $null = & $manager review @codex @reviewArgs -ConfirmHumanReview
    Write-Fixture "primary/$artifact" "Different evidence.`n"
    $reviewDrift = & $manager verify @codex
    Assert-True ($LASTEXITCODE -eq 1 -and $reviewDrift.Status -ceq 'NOT_RUN' -and $reviewDrift.CompletedWork.Reason -ceq 'review-artifact-changed') 'Changed review evidence retained PASS.'
    Assert-Rejected { & $manager close @codex -ConfirmScope -ConfirmAcceptance } 'completed-work=NOT_RUN'
    $missingArtifact = Join-Path $worktree $artifact
    [IO.File]::Delete($missingArtifact)
    $missingReview = & $manager verify @codex
    Assert-True ($missingReview.CompletedWork.Status -ceq 'BLOCKED' -and $missingReview.Status -ceq 'BLOCKED') 'Unavailable review evidence was counted.'
    Write-Fixture "primary/$artifact" $artifactText
    $closed = & $manager close @codex -ConfirmScope -ConfirmAcceptance
    Assert-True ($closed.Lifecycle -ceq 'closed') 'Fresh evidence and per-criterion reviews did not allow closure.'
    $closedStatus = & $manager verify @codex
    Assert-True ($LASTEXITCODE -eq 0 -and $closedStatus.CompletionStatus -ceq 'PASS') 'Current closure evidence was not reported.'
    $closedBootstrap = & $manager bootstrap @codex -ConfirmScope
    Assert-True ($closedBootstrap.Status -ceq 'BLOCKED' -and -not $closedBootstrap.ContextRecovered) 'Bootstrap recovered a closed task.'
    $head = Invoke-FixtureGit @('rev-parse', 'HEAD')
    Invoke-FixtureGit @('update-ref', 'refs/heads/another', $head)
    Invoke-FixtureGit @('symbolic-ref', 'HEAD', 'refs/heads/another')
    $branchDrift = & $manager verify @codex
    Assert-True (-not $branchDrift.Ready -and $branchDrift.Git.Changes.Fields -ccontains 'branch') 'Changed Git branch was not reported.'
    Invoke-FixtureGit @('symbolic-ref', 'HEAD', 'refs/heads/main')
    Invoke-FixtureGit @('add', '--', 'source.txt')
    $indexDrift = & $manager verify @codex
    Assert-True (-not $indexDrift.Ready -and $indexDrift.Git.Changes.Fields -ccontains 'indexSha256' -and
        $indexDrift.Git.Changes.Fields -ccontains 'entriesSha256') 'Changed Git index was not reported.'
    Invoke-FixtureGit ($commitArgs + @('-m', 'new primary revision'))
    $headDrift = & $manager verify @codex
    Assert-True ($LASTEXITCODE -eq 1 -and -not $headDrift.Ready -and
        $headDrift.Git.Changes.Fields -ccontains 'head') 'Verify did not reject the changed Git HEAD with exit code 1.'
    Set-FixtureUnmerged $worktree 'source.txt'
    $null = & $manager save @crashTask -ConfirmScope -AcceptGitDrift
    $unmergedIndex = (Get-FileHash -LiteralPath $index).Hash
    $conflicted = & $manager resume @crashTask -ConfirmScope -ConfirmContinuation
    Assert-True ($conflicted.GitConflicted -and -not $conflicted.CanCorrect -and $conflicted.Git.Current.unmerged -ccontains 'source.txt') 'An acknowledged Git conflict allowed editing.'
    Assert-Rejected { & $manager run @crashTask -ConfirmScope -Corrective -Kind edit -EvidenceId denied-edit -Executable $powershell } 'unmerged'
    Assert-True ((Get-FileHash -LiteralPath $index).Hash -ceq $unmergedIndex -and [IO.File]::ReadAllText((Join-Path $worktree 'untracked note.txt')) -ceq "untracked`n") 'Conflict handling changed the index or user data.'
    Write-Host 'PASS: Task Continuity lifecycle, checkpoint integrity, family compatibility, interrupted checks, verification freshness and portable handoff'
}
finally {
    $resolved = [IO.Path]::GetFullPath($fixture)
    if ([IO.Path]::GetDirectoryName($resolved) -cne $output -or [IO.Path]::GetFileName($resolved) -cnotmatch '^continuity-test-[a-f0-9]{32}$') {
        throw 'Unexpected fixture cleanup path.'
    }
    if ([IO.Directory]::Exists($resolved)) { Remove-Item -LiteralPath $resolved -Recurse -Force }
}

# GitHub Actions forwards $LASTEXITCODE. The final drift check intentionally exits 1.
exit 0

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
    Write-Fixture 'reviews.json' '[{"id":"fixture-exits","status":"PASS","evidence":"Observed fixture-test exit 0."},{"id":"ownership-preserved","status":"PASS","evidence":"Compared the index and document hashes."}]'
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
    Write-Fixture 'primary/.local/input.txt' "expected`n"
    $check = @{ EvidenceId = 'fixture-test'; Kind = 'test'; Executable = $powershell; InputPath = @('.local/input.txt'); ConfirmScope = $true }
    $failed = & $manager run @codex @check -Arguments @('-NoProfile', '-Command', 'exit 17')
    Assert-True ($failed.Status -ceq 'FAIL' -and $failed.ExitCode -eq 17 -and $LASTEXITCODE -eq 17) 'Run lost the observed failure exit code.'
    $interrupted = & $manager run @codex @check -TimeoutSeconds 1 -Arguments @('-NoProfile', '-Command', 'Start-Sleep -Seconds 30')
    Assert-True ($interrupted.Status -ceq 'NOT_RUN' -and $interrupted.Phase -ceq 'interrupted' -and
        $interrupted.Reason -ceq 'timeout' -and $null -eq $interrupted.ExitCode) 'A timed-out process supplied completed evidence.'
    $interruptedRecovery = & $manager resume @codex -ConfirmScope
    Assert-True ($interruptedRecovery.ContextRecovered -and -not $interruptedRecovery.CanContinue -and
        $interruptedRecovery.Checks[0].Status -ceq 'NOT_RUN') 'Interrupted process did not remain read-only and unverified.'
    $passed = & $manager run @codex @check -Arguments @('-NoProfile', '-Command', 'exit 0')
    Assert-True ($passed.Status -ceq 'PASS' -and $passed.ExitCode -eq 0) 'Completed process did not provide PASS evidence.'
    Write-Fixture 'primary/.local/input.txt' "changed`n"
    $stale = & $manager verify @codex
    Assert-True (-not $stale.Ready -and $stale.Checks[0].Status -ceq 'NOT_RUN') 'Changed ignored input retained PASS.'
    Assert-Rejected { & $manager close @codex -ConfirmScope -ConfirmAcceptance } 'checkpoint is not ready'
    $null = & $manager run @codex @check -Arguments @('-NoProfile', '-Command', 'exit 0')
    Assert-Rejected { & $manager close @codex -ConfirmScope -ConfirmAcceptance } 'ownership-preserved=NOT_RUN'

    $destination = Join-Path $fixture 'handoff.json'
    Assert-Rejected { & $manager export @codex -ConfirmScope -OutputFile $destination } 'ConfirmExport'
    $handoff = & $manager export @codex -ConfirmScope -ConfirmExport -OutputFile $destination
    $package = & $manager resume -HandoffFile $destination -ExpectedSha256 $handoff.Sha256 -AgentFamily codex -ConfirmScope
    Assert-True ($package.IntegrityVerified -and $package.ContextRecovered -and -not $package.CanContinue -and
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
    $null = & $manager review @codex -ConfirmScope -ReviewFile (Join-Path $fixture 'reviews.json')
    $closed = & $manager close @codex -ConfirmScope -ConfirmAcceptance
    Assert-True ($closed.Lifecycle -ceq 'closed') 'Fresh evidence and per-criterion reviews did not allow closure.'
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

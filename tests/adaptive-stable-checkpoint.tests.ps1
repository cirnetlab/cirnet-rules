#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
Import-Module (Join-Path $repo 'scripts/StableCheckpoint.psm1') -Force

function Assert-Equal($Actual, $Expected, [string]$Label) {
    if ($Actual -cne $Expected) { throw "$Label expected '$Expected', received '$Actual'." }
}

$stages = @(Get-StableStages)
Assert-Equal ($stages.name -join ',') 'candidate,repeat,e05,p01,pilot,full' 'Stage order'
Assert-Equal ($stages.calls -join ',') '4,4,2,2,16,96' 'Per-stage role budgets'
Assert-Equal ($stages.calls | Measure-Object -Sum).Sum 124 'Initial requested total'
Assert-Equal (Get-StableChargedCalls @(@{reserved=4;used=4},@{reserved=4;used=$null},@{reserved=16;used=2})) 10 'Actual and undetermined calls'
Assert-Equal (Get-StableChargedCalls @(@{reserved=4;used=-1},@{reserved=2;used=3})) 6 'Invalid counts retain reservations'

$manifest = @{sourceRevision='rev';sourceKind='WORKTREE';cohort='Candidate';modelRequested='gpt-6.2-sol';
    effortRequested='xhigh';cases=@('E01','E06');maximumRoleCalls=4;rules=@{doc='old'};
    evaluator=@{case='hash'};runner=@{role='hash'}; mode='PREVIEW'}
$preview = Get-StableManifestInputs $manifest 'Candidate'
$manifest.mode = 'NATIVE_COMPATIBILITY'
Assert-Equal (Get-StableManifestInputs $manifest 'Candidate') $preview 'Preview and native identity agree'
$manifest.rules.doc = 'new'
Assert-Equal ((Get-StableManifestInputs $manifest 'Candidate') -ceq $preview) $false 'Changed rules invalidate snapshot'

$stage = $stages[0]
$evidence = @{'trials/E01/Work/E01/events.jsonl'='a';'trials/E06/Work/E06/events.jsonl'='b'}
$review = @{reviewer='Independent reviewer'; evidenceSha256='run-seal'; cases=@{}}
foreach ($id in $stage.cases) {
    $review.cases[$id] = @{}
    foreach ($criterion in @('taskSuccess','architectureCompliance','verificationCoverage','privacy')) {
        $review.cases[$id][$criterion] = @{status='PASS';reason='Exercised criterion';evidence=@("trials/$id/Work/$id/events.jsonl")}
    }
}
Assert-Equal (Get-StableCandidateReview $null $stage 'run-seal' $evidence) 'REVIEW_REQUIRED' 'Objective result alone is insufficient'
Assert-Equal (Get-StableCandidateReview $review $stage 'other-seal' $evidence) 'EVIDENCE_BLOCKED' 'Stale review'
Assert-Equal (Get-StableCandidateReview $review $stage 'run-seal' $evidence) 'PASS' 'Complete Candidate review'
$review.cases.E06.privacy.status = 'NOT_RUN'
Assert-Equal (Get-StableCandidateReview $review $stage 'run-seal' $evidence) 'REVIEW_REQUIRED' 'Unassessed privacy'
$review.cases.E06.privacy.status = 'FAIL'
Assert-Equal (Get-StableCandidateReview $review $stage 'run-seal' $evidence) 'REPAIR_REQUIRED' 'Privacy failure'
$review.cases.E06.privacy.evidence = @('trials/E01/Work/E01/events.jsonl')
Assert-Equal (Get-StableCandidateReview $review $stage 'run-seal' $evidence) 'EVIDENCE_BLOCKED' 'Cross-case evidence'

$report = @{source='NATIVE';workRulesProfile='Current';reviewer='Independent reviewer';qualityGate='INCONCLUSIVE';
    qualityGates=@{G0='PASS'};cases=@();paired=@()}
foreach ($id in @('E01','E06','E02','E05')) {
    $report.paired += @{caseId=$id;comparable=$true}
    $report.cases += @{cohort='A';verdict='FAIL';reviewer='Independent reviewer'}
    $report.cases += @{cohort='B2';verdict='PASS';reviewer='Independent reviewer'}
}
Assert-Equal (Get-StableAbReview $report 'pilot') 'PASS' 'Independently reviewed pilot'
Assert-Equal (Get-StableAbReview $report 'full') 'REVIEW_REQUIRED' 'Pilot cannot satisfy full acceptance'
$report.source = 'TEST_SHIM'
Assert-Equal (Get-StableAbReview $report 'pilot') 'REVIEW_REQUIRED' 'Shim is ineligible'
$report.source = 'NATIVE'
$report.paired[1].comparable = $false
Assert-Equal (Get-StableAbReview $report 'pilot') 'REVIEW_REQUIRED' 'Unmatched pilot'
$report.paired[1].comparable = $true
$report.cases[3].verdict = 'FAIL'
Assert-Equal (Get-StableAbReview $report 'pilot') 'REPAIR_REQUIRED' 'Pilot Candidate regression'
$report.cases[3].verdict = 'PASS'

foreach ($gate in 0..6) { $report.qualityGates["G$gate"] = 'PASS' }
$report.qualityGate = 'PASS'
Assert-Equal (Get-StableAbReview $report 'full') 'PASS' 'Complete quality evidence without performance telemetry'
$report.qualityGates.G5 = 'INCONCLUSIVE'
Assert-Equal (Get-StableAbReview $report 'full') 'REVIEW_REQUIRED' 'Unassessed verification evidence'
$report.qualityGates.G5 = 'FAIL'
Assert-Equal (Get-StableAbReview $report 'full') 'REPAIR_REQUIRED' 'Verification regression'

# Keep the expected child failure out of the workflow step's $LASTEXITCODE.
$powerShell = Join-Path $PSHOME ($IsWindows ? 'pwsh.exe' : 'pwsh')
$startInfo = [Diagnostics.ProcessStartInfo]::new($powerShell)
$startInfo.UseShellExecute = $false
$startInfo.RedirectStandardError = $true
$runner = Join-Path $repo 'scripts/eval-adaptive-stable.ps1'
foreach ($argument in @('-NoProfile', '-NonInteractive', '-File', $runner, '-Session', 'missing-owner-cap', '-Execute')) {
    $startInfo.ArgumentList.Add($argument)
}
$child = [Diagnostics.Process]::Start($startInfo)
try {
    $errorText = $child.StandardError.ReadToEnd()
    $child.WaitForExit()
    Assert-Equal $child.ExitCode 1 'Dispatch requires explicit owner authorization'
    if ($errorText -cnotmatch 'Native execution requires an explicit owner-approved') {
        throw 'The runner must reject unapproved dispatch with its authorization error.'
    }
} finally { $child.Dispose() }
foreach ($status in @('REVIEW_REQUIRED', 'AUTHORIZATION_REQUIRED')) {
    Assert-Equal (Get-StableExitCode $status) 2 'Existing automation exit code'
    Assert-Equal (Get-StableExitCode $status -Interactive) 0 'Interactive pause exits normally'
}
Assert-Equal (Get-StableExitCode 'REPAIR_REQUIRED' -Interactive) 1 'Repair stays a failure'
foreach ($status in @('BLOCKED', 'EVIDENCE_BLOCKED', 'ENVIRONMENT_BLOCKED', 'INTERRUPTED')) {
    Assert-Equal (Get-StableExitCode $status -Interactive) 2 'Execution and integrity blockers stay nonzero'
}
Assert-Equal (Format-StableNativeMessage 'E05 COMPLETED / NOT_RUN - Codex completed. Behavioral grading requires the objective artifacts and rubric review.') 'E05 Native role completed. Independent review pending.' 'Role completion is clear'
Assert-Equal (Format-StableNativeMessage 'P01 OBJECTIVE_PASS_REVIEW_REQUIRED') 'P01 objective checks passed. Independent review pending.' 'Objective pass is clear'
Assert-Equal (Format-StableNativeMessage 'E06 BLOCKED / BLOCKED - Codex exceeded the configured timeout.') 'E06 BLOCKED / BLOCKED - Codex exceeded the configured timeout.' 'Timeout remains visible'
Write-Host 'Adaptive Stable checkpoint policy tests passed.'

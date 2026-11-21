#requires -Version 7.2
[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Repository,
    [Parameter(Mandatory)][string]$ExpectationPath,
    [Parameter(Mandatory)][ValidatePattern('(?-i:^[a-f0-9]{64}\z)')][string]$ExpectationSha256,
    [Parameter(Mandatory)][string]$ObservationPath,
    [Parameter(Mandatory)][ValidatePattern('(?-i:^[a-f0-9]{64}\z)')][string]$ObservationSha256
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'continuity/Repository.psm1')
Import-Module (Join-Path $PSScriptRoot 'continuity/Checkpoint.psm1')
Import-Module (Join-Path $PSScriptRoot 'shared/Bundle.psm1')

$digestPattern = '^[a-f0-9]{64}\z'
$reasonCodes = @('tool-unavailable','input-unavailable','authorization-required','owner-policy',
    'not-selected','platform-unavailable','timeout','interrupted','not-executed')
$blockedReasons = @('tool-unavailable','input-unavailable','authorization-required','platform-unavailable')

function Read-PinnedBytes([string]$Root, [string]$Path, [int]$MaxBytes, [ref]$Budget) {
    if ($Path -match '(?:^|/)\.git(?:/|$)') { throw 'Git administrative path.' }
    $resolved = Resolve-CirnetPath -Root $Root -Relative $Path
    $file = Get-Item -LiteralPath $resolved -Force
    if ($file -isnot [IO.FileInfo] -or !$file.Exists -or $file.LinkType -eq 'HardLink' -or
        ($file.Attributes -band ([IO.FileAttributes]::ReparsePoint -bor [IO.FileAttributes]::Device))) {
        throw 'Expected a bounded regular file.'
    }
    if (!$IsWindows) {
        $stat = $file.PSObject.Properties['UnixStat']
        if (!$stat -or !$stat.Value -or [string]$stat.Value.ItemType -cne 'File' -or $stat.Value.HardlinkCount -ne 1) {
            throw 'Expected one regular Unix file.'
        }
    }
    return ,(Read-ContinuityBytes $file.FullName $MaxBytes $Budget)
}

function Read-EvidenceRecord([string]$Root, [string]$Path, [string]$Digest) {
    $bytes = Read-PinnedBytes $Root $Path 1MB $null
    if ((Get-CirnetPayloadHash $bytes) -cne $Digest) { throw 'Evidence hash mismatch.' }
    ConvertFrom-ContinuityJson $bytes -UniqueProperties
}

function Assert-Expectation($Plan) {
    Assert-Record $Plan @('schemaVersion','revision','inputs','stages')
    if (!(Test-Integer $Plan.schemaVersion) -or $Plan.schemaVersion -ne 1 -or $Plan.revision -isnot [string] -or
        $Plan.revision -cnotmatch '^(?:[a-f0-9]{40}|[a-f0-9]{64})\z' -or
        $Plan.inputs -isnot [Collections.IDictionary] -or $Plan.inputs.psbase.Count -notin (1..32) -or
        $Plan.stages -isnot [array] -or $Plan.stages.Count -notin (1..64)) { throw 'Invalid expectation.' }
    foreach ($pin in $Plan.inputs.psbase.Values) {
        if ($pin -isnot [string] -or $pin -cnotmatch $digestPattern) { throw 'Invalid input pin.' }
    }
    $ids = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($stage in $Plan.stages) {
        Assert-Record $stage @('id','required','minAssertions')
        if ($stage.id -isnot [string] -or !$ids.Add($stage.id) -or $stage.required -isnot [bool] -or
            !(Test-Integer $stage.minAssertions) -or $stage.minAssertions -lt 1) { throw 'Invalid stage declaration.' }
        Assert-ContinuityId $stage.id
    }
    if (!($Plan.stages | Where-Object required)) { throw 'A claim needs a required stage.' }
}

function Read-StageObservations($Record, $Plan) {
    Assert-Record $Record @('schemaVersion','expectationSha256','stages')
    if (!(Test-Integer $Record.schemaVersion) -or $Record.schemaVersion -ne 1 -or
        $Record.expectationSha256 -isnot [string] -or $Record.expectationSha256 -cne $ExpectationSha256 -or
        $Record.stages -isnot [array] -or $Record.stages.Count -gt $Plan.stages.Count) { throw 'Invalid observation.' }
    $rows = @{}
    foreach ($row in $Record.stages) {
        Assert-Record $row @('id','status','exitCode','assertions','failedAssertions','reason')
        if ($row.id -isnot [string] -or $row.id -cnotin $Plan.stages.id -or $rows.ContainsKey($row.id) -or
            $row.status -isnot [string] -or $row.status -cnotin @('PASS','FAIL','BLOCKED','NOT_RUN','SKIPPED') -or
            $row.reason -isnot [string] -or !(Test-Integer $row.assertions) -or !(Test-Integer $row.failedAssertions) -or
            $row.failedAssertions -lt 0 -or $row.assertions -lt $row.failedAssertions) { throw 'Invalid stage observation.' }
        if ($row.status -cin @('PASS','FAIL')) {
            if (!(Test-Integer $row.exitCode) -or $row.exitCode -lt [int]::MinValue -or
                $row.exitCode -gt [int]::MaxValue -or $row.reason) { throw 'Invalid completed stage.' }
            if ($row.status -ceq 'PASS' -and ($row.exitCode -ne 0 -or $row.failedAssertions -ne 0 -or $row.assertions -eq 0)) {
                throw 'Invalid passing stage.'
            }
            if ($row.status -ceq 'FAIL' -and $row.exitCode -eq 0 -and $row.failedAssertions -eq 0) { throw 'Missing failure observation.' }
        }
        elseif ($null -ne $row.exitCode -or $row.assertions -ne 0 -or $row.reason -cnotin $reasonCodes) {
            throw 'Invalid incomplete stage.'
        }
        $rows[$row.id] = $row
    }
    $rows
}

function Measure-Claim($Result) {
    $root = Get-ContinuityRepository $Repository
    $plan = Read-EvidenceRecord $root.root $ExpectationPath $ExpectationSha256
    Assert-Expectation $plan
    $record = Read-EvidenceRecord $root.root $ObservationPath $ObservationSha256
    $rows = Read-StageObservations $record $plan
    $Result.revision = $plan.revision
    $budget = 32MB
    foreach ($path in $plan.inputs.psbase.Keys) {
        $bytes = Read-PinnedBytes $root.root $path 16MB ([ref]$budget)
        if ((Get-CirnetPayloadHash $bytes) -cne $plan.inputs[$path]) {
            $Result.status = 'NOT_RUN'
            $Result.reason = 'declared-input-changed'
            return
        }
    }
    $head = Get-ContinuityHead $root
    if (!$head) { throw 'Current revision is unavailable.' }
    if ($head -cne $plan.revision) {
        $Result.status = 'NOT_RUN'
        $Result.reason = 'revision-changed'
        return
    }
    $Result.stages = @(foreach ($stage in $plan.stages) {
        $row = $rows[$stage.id]
        $status = if ($row) { $row.status } else { 'NOT_RUN' }
        $reason = if ($row) { $row.reason } else { 'missing-stage' }
        if ($status -ceq 'PASS' -and $row.assertions -lt $stage.minAssertions) {
            $status = 'NOT_RUN'
            $reason = 'assertion-coverage-incomplete'
        }
        [ordered]@{
            id = $stage.id; required = $stage.required; status = $status; reason = $reason
            exitCode = if ($row) { $row.exitCode } else { $null }
            assertions = if ($row) { $row.assertions } else { 0 }
            failedAssertions = if ($row) { $row.failedAssertions } else { 0 }
        }
    })
    $required = @($Result.stages | Where-Object required)
    $Result.status = if ('FAIL' -cin $required.status) { 'FAIL' }
        elseif ($required | Where-Object { $_.status -ceq 'BLOCKED' -or $_.reason -cin $blockedReasons }) { 'BLOCKED' }
        elseif ($required | Where-Object { $_.status -cne 'PASS' }) { 'NOT_RUN' }
        else { 'PASS' }
    $Result.reason = 'required-stage-assessment'
}

$result = [ordered]@{
    schemaVersion = 1; status = 'BLOCKED'; reason = 'evidence-unavailable-or-invalid'
    scope = 'declared-stages-and-inputs'; revision = $null
    expectationSha256 = $ExpectationSha256; observationSha256 = $ObservationSha256; stages = @()
}
try { Measure-Claim $result }
catch { $result.status = 'BLOCKED'; $result.reason = 'evidence-unavailable-or-invalid'; $result.stages = @() }
ConvertTo-Json -InputObject $result -Depth 8 -Compress
exit (@{ PASS = 0; FAIL = 1; NOT_RUN = 2; BLOCKED = 3 }[$result.status])

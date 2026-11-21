#requires -Version 7.2
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$verify = Join-Path $repo 'scripts/verify-claim.ps1'
$target = Join-Path $repo 'target'
$relative = 'target/claim-test-' + [guid]::NewGuid().ToString('N').Substring(0, 12)
$fixture = Join-Path $repo $relative

function Write-Record($Name, $Record) {
    $path = Join-Path $fixture $Name
    [IO.File]::WriteAllText($path, (ConvertTo-Json -InputObject $Record -Depth 12))
    (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Observe($Id, $Status, $ExitCode, $Assertions = 0, $Failures = 0, $Reason = '') {
    @{ id = $Id; status = $Status; exitCode = $ExitCode; assertions = $Assertions; failedAssertions = $Failures; reason = $Reason }
}

function Assert-Claim($Rows, $Status, $Exit, $RequiredStatus, $OptionalStatus = 'NOT_RUN', $BadObservationPin = $false) {
    $planHash = Write-Record plan.json $plan
    $observation = @{ schemaVersion = 1; expectationSha256 = $planHash; stages = $Rows }
    $observationHash = Write-Record observation.json $observation
    if ($BadObservationPin) { $observationHash = '0' * 64 }
    $raw = & $verify -Repository $repo -ExpectationPath "$relative/plan.json" -ExpectationSha256 $planHash `
        -ObservationPath "$relative/observation.json" -ObservationSha256 $observationHash
    $code = $LASTEXITCODE
    $result = $raw | ConvertFrom-Json
    if ($code -ne $Exit -or $result.status -cne $Status) { throw "Expected $Status/$Exit, observed $($result.status)/$code." }
    if ($RequiredStatus) {
        $mandatory = @($result.stages | Where-Object id -CEQ 'tests')
        $optional = @($result.stages | Where-Object id -CEQ 'lint')
        if ($mandatory.Count -ne 1 -or !$mandatory[0].required -or $mandatory[0].status -cne $RequiredStatus -or
            $optional.Count -ne 1 -or $optional[0].required -ne $plan.stages[1].required -or
            $optional[0].status -cne $OptionalStatus) {
            throw 'The result did not preserve observed stage scope and status.'
        }
    }
}

if (!(Test-Path -LiteralPath $verify -PathType Leaf)) { throw 'The claim verifier entry point is missing.' }
$null = New-Item -ItemType Directory -Path $fixture
try {
    $head = git --no-optional-locks -C $repo rev-parse HEAD
    if ($LASTEXITCODE) { throw 'Could not pin the fixture HEAD.' }
    $input = "$relative/input.txt"
    $inputFile = Join-Path $repo $input
    [IO.File]::WriteAllText($inputFile, 'fixture input')
    $plan = @{
        schemaVersion = 1; revision = $head
        inputs = @{ $input = (Get-FileHash -LiteralPath $inputFile -Algorithm SHA256).Hash.ToLowerInvariant() }
        stages = @(@{ id = 'tests'; required = $true; minAssertions = 2 }, @{ id = 'lint'; required = $false; minAssertions = 1 })
    }
    $pass = Observe tests PASS 0 2
    $optionalSkip = Observe lint SKIPPED $null 0 0 'owner-policy'
    Assert-Claim @($pass, $optionalSkip) PASS 0 PASS SKIPPED
    Assert-Claim @((Observe tests FAIL 7), $optionalSkip) FAIL 1 FAIL SKIPPED
    Assert-Claim @((Observe tests FAIL 0 2 1)) FAIL 1 FAIL
    Assert-Claim @((Observe tests SKIPPED $null 0 0 'tool-unavailable')) BLOCKED 3 SKIPPED
    Assert-Claim @((Observe tests BLOCKED $null 0 0 'tool-unavailable')) BLOCKED 3 BLOCKED
    Assert-Claim @((Observe tests NOT_RUN $null 0 0 'timeout')) NOT_RUN 2 NOT_RUN
    Assert-Claim @() NOT_RUN 2 NOT_RUN
    Assert-Claim @((Observe tests PASS $null 2)) BLOCKED 3 ''
    Assert-Claim @((Observe tests PASS 0)) BLOCKED 3 ''
    Assert-Claim @((Observe tests PASS 0 1)) NOT_RUN 2 NOT_RUN
    Assert-Claim @((Observe tests PASS '0' 1)) BLOCKED 3 ''
    Assert-Claim @($pass, $pass) BLOCKED 3 ''
    Assert-Claim @($pass, (Observe unknown PASS 0 1)) BLOCKED 3 ''
    Assert-Claim @($pass) BLOCKED 3 '' NOT_RUN $true
    $plan.stages[1].required = $true
    Assert-Claim @((Observe tests FAIL 7)) FAIL 1 FAIL
    Assert-Claim @($pass) NOT_RUN 2 PASS
    $plan.stages[1].required = $false
    $plan.inputs['../unapproved.txt'] = '0' * 64
    Assert-Claim @($pass) BLOCKED 3 ''
    $plan.inputs.Remove('../unapproved.txt')
    [IO.File]::AppendAllText($inputFile, ' changed')
    Assert-Claim @($pass) NOT_RUN 2 ''
    [IO.File]::WriteAllText($inputFile, 'fixture input')
    $plan.revision = '0' * $head.Length
    Assert-Claim @($pass) NOT_RUN 2 ''

    # The wrapper can return zero independently of either child evidence row.
    $node = (Get-Command node -CommandType Application | Select-Object -First 1).Source
    $wrapper = "const c=require('node:child_process').spawnSync(process.execPath,['-e','process.exit(7)']); process.stdout.write(String(c.status));"
    $actual = & $node -e $wrapper
    if ($LASTEXITCODE -ne 0 -or $actual -cne '7') { throw 'Fixture did not reproduce wrapper zero with child failure.' }
    $plan.revision = $head
    Assert-Claim @((Observe tests FAIL ([int]$actual))) FAIL 1 FAIL
    $skipped = & $node --test-reporter=tap -e "require('node:test').skip('required check', () => {});"
    if ($LASTEXITCODE -ne 0 -or ($skipped -join "`n") -notmatch '(?m)^# skipped 1\s*$') {
        throw 'Fixture did not reproduce wrapper zero with a skipped check.'
    }
    Assert-Claim @((Observe tests SKIPPED $null 0 0 'owner-policy')) NOT_RUN 2 SKIPPED
    Assert-Claim @($pass) PASS 0 PASS
    Write-Host 'PASS: pinned claim assessment, observed child failure, skips, unavailable steps, stale pins and a positive assertion.'
}
finally {
    $resolved = (Resolve-Path -LiteralPath $fixture).Path
    if ([IO.Path]::GetDirectoryName($resolved) -cne (Resolve-Path -LiteralPath $target).Path -or
        [IO.Path]::GetFileName($resolved) -cnotmatch '^claim-test-[a-f0-9]{12}$') { throw 'Unsafe claim fixture cleanup path.' }
    Remove-Item -LiteralPath $resolved -Recurse -Force
}
exit 0

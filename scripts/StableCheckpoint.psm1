Set-StrictMode -Version Latest

function Get-StablePauseLabel([string]$Status) {
    switch -CaseSensitive ($Status) {
        'REVIEW_REQUIRED' { 'Paused for independent review' }
        'AUTHORIZATION_REQUIRED' { 'Paused for owner approval' }
    }
}

function Get-StableExitCode([string]$Status, [switch]$Interactive) {
    if ($Status -ceq 'ADAPTIVE_STABLE_READY' -or ($Interactive -and (Get-StablePauseLabel $Status))) { return 0 }
    if ($Status -ceq 'REPAIR_REQUIRED') { return 1 }
    2
}

function Format-StableNativeMessage([string]$Message) {
    switch -CaseSensitive -Regex ($Message) {
        '^([A-Z][0-9]{2}) COMPLETED / NOT_RUN - Codex completed\. Behavioral grading requires the objective artifacts and rubric review\.$' {
            return "$($Matches[1]) Native role completed. Independent review pending."
        }
        '^([A-Z][0-9]{2}) OBJECTIVE_PASS_REVIEW_REQUIRED$' {
            return "$($Matches[1]) objective checks passed. Independent review pending."
        }
    }
    $Message
}

function Get-StableStages {
    @(
        @{ name='candidate'; cases=@('E01','E06'); kind='Candidate'; calls=4 }
        @{ name='repeat'; cases=@('E01','E06'); kind='Candidate'; calls=4 }
        @{ name='e05'; cases=@('E05'); kind='Candidate'; calls=2 }
        @{ name='p01'; cases=@('P01'); kind='Candidate'; calls=2 }
        @{ name='pilot'; cases=@('E01','E06','E02','E05'); kind='AB'; calls=16; repetitions=1 }
        @{ name='full'; cases=@('E01','E06','E02','E05','E03','E04','E07','E08'); kind='AB'; calls=96; repetitions=3 }
    )
}

function Get-StableChargedCalls($Attempts) {
    $total = 0
    foreach ($attempt in $Attempts) {
        $used = $attempt['used']
        $reserved = $attempt['reserved']
        if ($null -ne $used -and $used -ge 0 -and $used -le $reserved) { $total += $used }
        else { $total += $reserved }
    }
    $total
}

function Get-StableManifestInputs($Manifest, [string]$Kind) {
    if ($Kind -eq 'Candidate') {
        $keys = @('sourceRevision','sourceKind','cohort','modelRequested','effortRequested','cases','maximumRoleCalls','rules','evaluator','runner')
    } else {
        $keys = @('baseline','candidateId','candidateSourceHead','workRulesProfile','snapshots','inputs','schedule',
            'seed','repetitions','timeoutSecondsPerRole','authorModel','workModel','authorEffort','workEffort',
            'runPurpose','carriedTrials','priorEvidenceIndexSha256','host',
            'runnerSha256','schedulerSha256','reporterSha256','rtlObserverSha256','privacyHelperSha256','bundleSha256')
    }
    $identity = [ordered]@{}
    foreach ($key in $keys) {
        if (-not $Manifest.Contains($key)) { throw "Evaluation manifest is missing $key." }
        $identity[$key] = $Manifest[$key]
    }
    ConvertTo-Json -InputObject $identity -Depth 60 -Compress
}

function Get-StableCandidateReview($Review, $Stage, [string]$Seal, $Evidence) {
    if ($null -eq $Review) { return 'REVIEW_REQUIRED' }
    if (-not ([string]$Review['reviewer']).Trim() -or $Review['evidenceSha256'] -cne $Seal -or
        $Review['cases'] -isnot [System.Collections.IDictionary] -or
        (($Review['cases'].Keys | Sort-Object) -join ',') -cne (($Stage.cases | Sort-Object) -join ',')) {
        return 'EVIDENCE_BLOCKED'
    }
    $statuses = [Collections.Generic.List[string]]::new()
    foreach ($id in $Stage.cases) {
        foreach ($criterion in @('taskSuccess','architectureCompliance','verificationCoverage','privacy')) {
            $fact = $Review['cases'][$id][$criterion]
            if (-not $fact -or $fact['status'] -cnotin @('PASS','FAIL','BLOCKED','NOT_RUN') -or
                -not ([string]$fact['reason']).Trim() -or
                ($fact['status'] -cin @('PASS','FAIL') -and -not @($fact['evidence']).Where({ $_ }).Count) -or
                @($fact['evidence']).Where({ $_ -and (-not $_.StartsWith("trials/$id/", [StringComparison]::Ordinal) -or -not $Evidence.Contains($_)) }).Count) {
                return 'EVIDENCE_BLOCKED'
            }
            $statuses.Add($fact['status'])
        }
    }
    if ('FAIL' -cin $statuses) { return 'REPAIR_REQUIRED' }
    if ($statuses.Where({ $_ -cne 'PASS' }).Count) { return 'REVIEW_REQUIRED' }
    'PASS'
}

function Get-StableAbReview($Summary, [string]$Stage) {
    if ($Summary['source'] -cne 'NATIVE' -or $Summary['workRulesProfile'] -cne 'Current' -or
        -not ([string]$Summary['reviewer']).Trim()) { return 'REVIEW_REQUIRED' }
    if (@(0..6).Where({ $Summary['qualityGates']["G$_"] -ceq 'FAIL' }).Count -or
        @($Summary['cases']).Where({ $_['cohort'] -cne 'A' -and $_['verdict'] -ceq 'FAIL' }).Count) {
        return 'REPAIR_REQUIRED'
    }
    if ($Stage -ceq 'pilot') {
        # The reporter leaves global G1–G6 unassessed for a pilot.
        if ($Summary['qualityGates']['G0'] -ceq 'PASS' -and @($Summary['paired']).Count -eq 4 -and
            -not @($Summary['paired']).Where({ $_['comparable'] -ne $true }).Count -and
            @($Summary['cases']).Count -eq 8 -and
            -not @($Summary['cases']).Where({ -not $_['reviewer'] -or
                ($_['cohort'] -ceq 'A' -and $_['verdict'] -cnotin @('PASS','FAIL')) -or
                ($_['cohort'] -cne 'A' -and $_['verdict'] -cne 'PASS') }).Count) {
            return 'PASS'
        }
        return 'REVIEW_REQUIRED'
    }
    if ($Summary['qualityGate'] -ceq 'PASS' -and
        -not @(0..6).Where({ $Summary['qualityGates']["G$_"] -cne 'PASS' }).Count) {
        return 'PASS'
    }
    'REVIEW_REQUIRED'
}

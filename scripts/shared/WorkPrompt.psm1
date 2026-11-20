#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'Bundle.psm1')
Import-Module (Join-Path $PSScriptRoot '../continuity/Repository.psm1')
Import-Module (Join-Path $PSScriptRoot '../continuity/Checkpoint.psm1')

function New-PromptResult([string]$Status, [string[]]$Diagnostics) {
    [pscustomobject]@{ status=$Status; diagnostics=$Diagnostics }
}

function Read-PromptBytes([string]$Path) {
    $path = Resolve-CirnetRoot ([IO.Path]::GetFullPath($Path, $PWD.ProviderPath))
    $file = [IO.FileInfo]::new($path)
    if (-not $file.Exists -or $file.LinkType -eq 'HardLink') {
        throw 'Expected a bounded regular file.'
    }
    return ,(Read-ContinuityBytes $path 1MB)
}

function New-PromptRequirement([string]$Label, [string[]]$Forms) {
    @{ label=$Label; forms=$Forms }
}

function Get-PromptViolation([string]$Line, [string[]]$Allowed) {
    $statement = ($Line.Trim() -replace '^[-*]\s+', '' -replace '^(?:also|then)\s+', '').Trim([char]96)
    if ($statement -match '^(?:PROMPT_(?:READY|BLOCKED)|IMPLEMENTATION_VERIFIED)$' -or
        $statement -match '^(?:PASS\b|(?:tests?|checks?|build|verification|result|status)\s*(?:[:=]\s*|\s+)(?:PASS(?:ED)?|succeeded|complete[d]?)\b)') {
        return 'asserts another state or an unverified result'
    }
    # Recognize only unambiguous directives. All other added prose needs review.
    $effect = $null
    if ($statement -match '^(read|write|test|build|export|privileged):') { $effect = $statement }
    elseif ($statement -match '^(read|write|cat|Get-Content) ([\w./-]+\.[\w-]+)\.?$') {
        $kind = if ($Matches[1] -eq 'write') { 'write' } else { 'read' }
        $effect = '{0}: {1}' -f $kind,$Matches[2]
    }
    elseif ($statement -match '^export (.+)$') { $effect = 'export: ' + $Matches[1] }
    elseif ($statement -match '^(?:git(?:\.exe)? (?:push|commit|merge|reset|clean|stash)|deploy|publish)(?:\s|$)') {
        $effect = 'privileged: ' + $statement
    }
    if ($effect -and [Array]::IndexOf($Allowed,$effect) -lt 0) { return 'requests an action outside the approved permissions' }
}

# Unknown prose can change the meaning of adjacent approved lines. Never
# ignore decorations, comments or fences. Nothing in the draft is executed.
function Test-PromptContent([string[]]$Lines, $Requirements, [string[]]$Allowed, [string]$Readiness) {
    $known = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    $missing = [Collections.Generic.List[string]]::new()
    $locations = [Collections.Generic.Dictionary[string,int]]::new([StringComparer]::Ordinal)
    foreach ($line in $Allowed) { [void]$known.Add($line) }
    foreach ($item in $Requirements) {
        foreach ($form in $item.forms) { [void]$known.Add($form) }
    }
    $extra = 0
    $repeated = $false
    $number = 0
    foreach ($line in $Lines) {
        $number++
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        $violation = Get-PromptViolation $line $Allowed
        if ($violation) { return New-PromptResult FAIL "Body line $number $violation. Correct it before handoff." }
        if ($seen.Add($line)) { $locations.Add($line,$number) }
        else { $repeated = $true }
        if ($known.Contains($line)) { continue }
        if ($line -match '^\s*(?:[-*]\s*)?(?:(?:read|write|test|build|export|privileged|source|authorization|original(?: task)?|write scope)\s*:|Verify Task#|Task#[0-9])') {
            return New-PromptResult FAIL "Body line $number changes an approved action, reference or verification mapping."
        }
        $extra++
    }
    $fixedMissing = $false
    $previous = 0
    $reordered = $false
    foreach ($item in $Requirements) {
        if (-not $seen.Overlaps([string[]]$item.forms)) {
            $missing.Add($item.label)
            if (-not $item.ContainsKey('taskLine')) { $fixedMissing = $true }
        } elseif ($item.ContainsKey('taskLine')) {
            $positions = @($item.forms | Where-Object { $seen.Contains($_) } | ForEach-Object { $locations[$_] })
            if ($positions.Count -ne 1 -or $positions[0] -le $previous) { $reordered = $true }
            $previous = $positions[0]
        }
    }
    $diagnostics = @(
        if ($missing.Count) { 'Missing exact coverage for ' + (($missing | Select-Object -First 8) -join ', ') + '.' }
        if ($extra) { "$extra unrecognized line(s). Independently review their meaning, scope and claims before dispatch." }
        if ($reordered -or $repeated) { 'Review repeated or reordered lines in their original context.' }
    )
    if ($fixedMissing -or ($missing.Count -and -not $extra)) { return New-PromptResult FAIL $diagnostics }
    if ($extra -or $reordered -or $repeated) { return New-PromptResult REVIEW_REQUIRED $diagnostics }
    if ($Readiness -ceq 'PROMPT_BLOCKED') { return New-PromptResult BLOCKED 'Only authorized clarification is available. Implementation is blocked.' }
    New-PromptResult PASS 'Required statements, permissions and check mappings match the supplied authority.'
}

function Test-CirnetCandidatePrompt([string]$Text, $Case, [string]$ReviewedSha256, [string[]]$ReadablePaths) {
    $state = $Case['readiness'] ?? 'PROMPT_READY'
    $scope = @(if ($state -ceq 'PROMPT_READY') { $Case.writable })
    $lines = $Text -split '\r?\n', 4
    if ($lines.Count -ne 4 -or -not $lines[0].Equals($state) -or -not $lines[1].Equals("Original task: $($Case.prompt)") -or
        -not $lines[2].Equals('Write scope: ' + (ConvertTo-Json -InputObject $scope -Compress))) {
        return New-PromptResult FAIL 'Restore the frozen case readiness, original task and exact write scope.'
    }
    $body = [regex]::Match($lines[3], '(?ms)\A\s*## Requirements and constraints[ \t]*\r?\n(?<requirements>.*?)^## Acceptance and verification[ \t]*\r?\n(?<verification>.*)\z')
    if (-not $body.Success -or -not $body.Groups['requirements'].Value.Trim() -or -not $body.Groups['verification'].Value.Trim() -or
        $lines[3] -match '(?m)^[ \t]*(?:\*\*)?PROMPT_(?:READY|BLOCKED)(?:\*\*)?[ \t]*\r?$') {
        return New-PromptResult FAIL 'Restore the two nonempty Candidate review sections and a single readiness declaration.'
    }
    # File reads come from the assigned fixture, writes from the frozen case.
    # Legacy prose and criterion-to-check mappings need independent review.
    $allowed = @(
        foreach ($path in $scope) { "write: $path" }
        foreach ($path in $ReadablePaths) { "read: $path" }
    )
    foreach ($group in @('requirements','verification')) {
        $part = Test-PromptContent ($body.Groups[$group].Value -split '\r?\n') @() $allowed $state
        if ($part.status -ceq 'FAIL') { return $part }
    }
    if (-not $ReviewedSha256) {
        return New-PromptResult REVIEW_REQUIRED 'Compare the two sections to the frozen rubric, negative cases, permissions and verification before native Work.'
    }
    if ((Get-CirnetPayloadHash ([Text.Encoding]::UTF8.GetBytes($Text))) -cne $ReviewedSha256) {
        return New-PromptResult FAIL 'Reviewed Candidate bytes differ. Repeat independent review.'
    }
    New-PromptResult PASS 'Candidate envelope matches the frozen case. The operator attested independent review of these exact bytes.'
}

function Test-PromptMap($Value, [string[]]$Keys) {
    if ($Value -isnot [Collections.IDictionary] -or $Value.Count -ne $Keys.Count) { return $false }
    foreach ($key in $Keys) { if ($key -cnotin $Value.Keys) { return $false } }
    return $true
}

function Test-CirnetWorkPromptFile {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$Prompt,
        [Parameter(Mandatory)][string]$OriginalTask,
        [Parameter(Mandatory)][string]$Authorization,
        [Parameter(Mandatory)][string]$AuthorizationSha256,
        [Parameter(Mandatory)][string]$Repository,
        [ValidatePattern('^[0-9a-f]{64}\z')][string]$ReviewedPromptSha256
    )
    $utf8 = [Text.UTF8Encoding]::new($false,$true)
    $stage = 'approved authorization record'
    try {
        $bytes = Read-PromptBytes $Authorization
        if (-not (Get-CirnetPayloadHash $bytes).Equals($AuthorizationSha256)) {
            return New-PromptResult BLOCKED 'Obtain the authorization digest from the approving operator and verify the record.'
        }
        # ConvertFrom-Json alone silently accepts duplicate keys (last wins).
        $settings = [Newtonsoft.Json.Linq.JsonLoadSettings]::new()
        $settings.DuplicatePropertyNameHandling = 'Error'
        $null = [Newtonsoft.Json.Linq.JObject]::Parse($utf8.GetString($bytes),$settings)
        $record = ConvertFrom-ContinuityJson $bytes
        if (-not (Test-PromptMap $record @('originalSha256','repository','revision','readiness','sensitive','permissions','fields','actions','checks','sources')) -or
            $record.repository -isnot [string] -or $record.revision -isnot [string] -or $record.originalSha256 -isnot [string] -or
            $record.sensitive -isnot [bool] -or [Array]::IndexOf(@('PROMPT_READY','PROMPT_BLOCKED'),$record.readiness) -lt 0 -or
            $record.originalSha256 -cnotmatch '^[0-9a-f]{64}\z' -or $record.revision -cnotmatch '^(?:[0-9a-f]{40}|[0-9a-f]{64})\z') {
            throw 'Invalid authority.'
        }
        if ($record.sensitive) { return New-PromptResult BLOCKED 'An enforced pre-model read boundary is unavailable. Obtain an approved isolated projection before discovery.' }

        $stage = 'original task bytes'
        $bytes = Read-PromptBytes $OriginalTask
        if ((Get-CirnetPayloadHash $bytes) -cne $record.originalSha256) { return New-PromptResult FAIL 'The original request differs from its approved digest.' }
        $task = $utf8.GetString($bytes) -split '\r?\n'
        $stage = 'approved repository and revision'
        $root = Resolve-CirnetRoot ([IO.Path]::GetFullPath($Repository, $PWD.ProviderPath))
        if (-not $root.Equals($record.repository)) { return New-PromptResult FAIL 'Select the exact repository root named by the approval.' }
        if ((Get-ContinuityHead (Get-ContinuityRepository $root)) -cne $record.revision) { return New-PromptResult FAIL 'The approved revision differs from HEAD. Refresh the source review.' }

        $stage = 'permissions and source references'
        $kinds = @('read','write','test','build','export','privileged')
        if (-not (Test-PromptMap $record.permissions $kinds) -or $record.sources -isnot [Collections.IDictionary]) { throw 'Invalid permissions.' }
        $allowed = [Collections.Generic.List[string]]::new()
        foreach ($kind in $kinds) {
            $values = $record.permissions[$kind]
            if ($values -isnot [array]) { throw 'Explicit arrays required.' }
            foreach ($value in $values) {
                if ($value -isnot [string] -or -not $value.Trim() -or $value -match '[\x00-\x1f]') { throw 'Invalid permission.' }
                if ($record.readiness -ceq 'PROMPT_BLOCKED' -and $kind -cne 'read') { throw 'Invalid blocked scope.' }
                if ($kind -cin @('read','write')) {
                    if ($value.Contains('\') -or $value -match '(?:^|/)(?:\.git|\.{1,2})(?:/|$)|[*?\[\]]') { throw 'Use literal repository file paths.' }
                    $resolved = Resolve-CirnetPath $root $value
                    $file = [IO.FileInfo]::new($resolved)
                    if ([IO.Directory]::Exists($resolved) -or ($file.Exists -and $file.LinkType -eq 'HardLink') -or ($kind -ceq 'read' -and -not $file.Exists)) { throw 'Unsafe or missing file.' }
                }
                $allowed.Add(('{0}: {1}' -f $kind,$value))
            }
        }
        $required = [Collections.Generic.List[object]]::new()
        $taskEffects = [Collections.Generic.List[string]]::new()
        $required.Add((New-PromptRequirement 'original request pin' @("Original: $($record.originalSha256)")))
        $required.Add((New-PromptRequirement 'authorization pin' @("Authorization: $AuthorizationSha256")))
        foreach ($path in $record.sources.Keys) {
            if (-not $allowed.Contains("read: $path") -or $record.sources[$path] -isnot [string] -or $record.sources[$path] -cnotmatch '^[0-9a-f]{64}\z') { throw 'Unapproved source.' }
            $resolved = Resolve-CirnetPath $root $path
            if ((Get-CirnetHash $resolved) -cne $record.sources[$path]) { return New-PromptResult FAIL 'Pinned source bytes changed. Reinspect before citing evidence.' }
            $taskEffects.Add("read: $path")
            $required.Add((New-PromptRequirement "source $path" @("Source: $path")))
        }

        $stage = 'approved requirement lines and acceptance mappings'
        $fields = @('action','target','transformation','outcome','scope','negative','acceptance')
        if (-not (Test-PromptMap $record.fields $fields) -or $record.checks -isnot [Collections.IDictionary]) { throw 'Invalid coverage.' }
        $labels = @{}
        foreach ($field in $fields) {
            $numbers = $record.fields[$field]
            if ($numbers -isnot [array] -or (-not $numbers.Count -and $field -cne 'negative')) { throw 'Missing field.' }
            foreach ($n in $numbers) {
                if (($n -isnot [int] -and $n -isnot [long]) -or $n -lt 1 -or $n -gt $task.Count -or -not $task[$n-1].Trim()) { throw 'Invalid task line.' }
                $labels[[int]$n] = @($labels[[int]$n] | Where-Object { $_ }) + $field
            }
        }
        for ($n=1; $n -le $task.Count; $n++) {
            if ($task[$n-1].Trim()) {
                $name = "Task#$n"
                $originalLine = $task[$n-1]
                $forms = @($originalLine; if ($n -notin ($record.fields.action + $record.fields.target + $record.fields.transformation)) { $name })
                $item = New-PromptRequirement "$name $($labels[$n] -join '/')" $forms
                $item.taskLine = $n
                $required.Add($item)
            }
        }
        foreach ($field in @('action','acceptance')) {
            $map = if ($field -ceq 'action') { $record.actions } else { $record.checks }
            if (-not (Test-PromptMap $map @($record.fields[$field] | ForEach-Object { [string]$_ }))) { throw 'Missing mapping.' }
            foreach ($number in $map.Keys) {
                $effects = $map[$number]
                if ($effects -isnot [array] -or -not $effects.Count) { throw 'Missing authorized operation.' }
                foreach ($effect in $effects) {
                    if ($effect -isnot [string] -or -not $allowed.Contains($effect)) { throw 'Unapproved operation.' }
                    $taskEffects.Add($effect)
                    if ($field -ceq 'acceptance') {
                        if ($effect -cnotmatch '^(read|test|build): ') { throw 'Invalid check permission.' }
                        $required.Add((New-PromptRequirement "verification for Task#$number" @("Verify Task#$($number): $effect")))
                    }
                }
            }
        }
        $stage = 'draft handoff'
        # A repository-wide grant may cover several tasks. Only the effects
        # bound to this request, its checks and its evidence belong here.
        for ($n=0; $n -lt $task.Count; $n++) {
            $violation = Get-PromptViolation $task[$n] $taskEffects
            if ($violation) { return New-PromptResult BLOCKED "Original Task#$($n+1) $violation. Resolve the conflict before handoff." }
        }
        $bytes = Read-PromptBytes $Prompt
        $lines = $utf8.GetString($bytes) -split '\r?\n'
        if (-not $lines[0].Equals($record.readiness)) { return New-PromptResult FAIL 'Restore the approved readiness on the first line.' }
        $headings = @('Goal & Expected Behavior','Owner & Permissions','References & Evidence','Requirements & Constraints','Acceptance & Verification','Deliverables & Reporting')
        foreach ($heading in $headings) { $taskEffects.Add("## $heading") }
        $result = Test-PromptContent ($lines | Select-Object -Skip 1) $required $taskEffects $record.readiness
        $digest = Get-CirnetPayloadHash $bytes
        if ($ReviewedPromptSha256 -and $ReviewedPromptSha256 -cne $digest) { return New-PromptResult FAIL 'Reviewed draft bytes differ. Repeat independent review.' }
        if ($ReviewedPromptSha256 -and $result.status -ceq 'REVIEW_REQUIRED') {
            if ($record.readiness -ceq 'PROMPT_BLOCKED') { $result = New-PromptResult BLOCKED 'Independent review covers clarification only. Implementation remains blocked.' }
            else { $result = New-PromptResult PASS 'Pins and explicit boundaries checked. The operator attested independent review of the remaining prose.' }
        }
        $result | Add-Member -NotePropertyName promptSha256 -NotePropertyValue $digest
        return $result
    } catch {
        # Inputs may contain sensitive strings. Report the stage, never their values.
        New-PromptResult BLOCKED "Cannot verify $stage. Check the approved format, paths and access."
    }
}

Export-ModuleMember -Function Test-CirnetWorkPromptFile, Test-CirnetCandidatePrompt

#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'Bundle.psm1')
Import-Module (Join-Path $PSScriptRoot '../continuity/Repository.psm1')
Import-Module (Join-Path $PSScriptRoot '../continuity/Checkpoint.psm1')

$permissionKinds = @('read','write','test','build','export','privileged')
$requirementFields = @('action','target','transformation','outcome','scope','negative','acceptance')

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
    if ($statement -match '^(read|write|create|modify|delete|move|test|build|export|privileged):') { $effect = $statement }
    elseif ($statement -match '^(read|write|cat|Get-Content) ([\w./-]+\.[\w-]+)\.?$') {
        $kind = if ($Matches[1] -eq 'write') { 'write' } else { 'read' }
        $effect = '{0}: {1}' -f $kind,$Matches[2]
    }
    elseif ($statement -match '^export (.+)$') { $effect = 'export: ' + $Matches[1] }
    elseif ($statement -match '^(?:Move-Item|mv)\s+' -or $statement -match '^(?:Remove-Item|rm)\s+-') {
        return 'requests a move or deletion without explicit file effects'
    }
    elseif ($statement -match '^(create|modify|delete|Remove-Item|rm)\s+([\w./-]+\.[\w-]+)\.?$') {
        $verb, $path = $Matches[1].ToLowerInvariant(), $Matches[2]
        if ($verb -in @('remove-item', 'rm')) { $verb = 'delete' }
        $effect = '{0}: {1}' -f $verb,$path
    }
    elseif ($statement -match '^(?:git(?:\.exe)? (?:push|commit|merge|reset|clean|stash)|deploy|publish)(?:\s|$)') {
        $effect = 'privileged: ' + $statement
    }
    if ($effect -and [Array]::IndexOf($Allowed,$effect) -lt 0) { return 'requests an action outside the approved permissions' }
}

function Get-PromptProse([string[]]$Lines) {
    $fence = ''
    $comment = $false
    foreach ($line in $Lines) {
        if ($fence) {
            if ($line -match '^\s*(?<fence>`{3,}|~{3,})\s*$' -and
                $Matches.fence[0] -ceq $fence[0] -and $Matches.fence.Length -ge $fence.Length) { $fence = '' }
            ''
            continue
        }
        if ($comment -or $line.Contains('<!--')) {
            $comment = $line.LastIndexOf('<!--', [StringComparison]::Ordinal) -gt $line.LastIndexOf('-->', [StringComparison]::Ordinal) -or
                ($comment -and -not $line.Contains('-->'))
            ''
            continue
        }
        if ($line -match '^\s*(?<fence>`{3,}|~{3,})') {
            $fence = $Matches.fence
            ''
            continue
        }
        if ($line -match '^(?: {4}|\t|\s*>)') { '' }
        else { $line }
    }
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
    $prose = @(Get-PromptProse $Lines)
    for ($i = 0; $i -lt $Lines.Count; $i++) {
        $line = $Lines[$i]
        $number = $i + 1
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        $violation = Get-PromptViolation $line $Allowed
        if ($violation) { return New-PromptResult FAIL "Body line $number $violation. Correct it before handoff." }
        if (-not $prose[$i]) { $extra++; continue }
        if ($seen.Add($line)) { $locations.Add($line,$number) }
        else { $repeated = $true }
        if ($known.Contains($line)) { continue }
        if ($line -match '^\s*(?:[-*]\s*)?(?:(?:read|write|create|modify|delete|move|test|build|export|privileged|source|authorization|original(?: task)?|write scope)\s*:|Verify Task#|Task#[0-9])') {
            return New-PromptResult FAIL "Body line $number changes an approved action, reference or verification mapping."
        }
        $extra++
    }
    $previous = 0
    $reordered = $false
    foreach ($item in $Requirements) {
        if (-not $seen.Overlaps([string[]]$item.forms)) {
            $missing.Add($item.label)
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
    if ($missing.Count) { return New-PromptResult FAIL $diagnostics }
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
    $bodyLines = $lines[3] -split '\r?\n'
    $visible = (Get-PromptProse $bodyLines) -join [char]10
    $body = [regex]::Match($visible, '(?ms)\A\s*## Requirements and constraints[ \t]*\n(?<requirements>.*?)^## Acceptance and verification[ \t]*\n(?<verification>.*)\z')
    if (-not $body.Success -or
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
        $section = $body.Groups[$group].Value -split '\r?\n'
        if (-not @($section | Where-Object { $_.Trim() -and $_ -notmatch '^\s*#{1,6}(?:\s|$)' }).Count) {
            return New-PromptResult FAIL 'Restore visible Candidate requirements and verification content.'
        }
    }
    $part = Test-PromptContent $bodyLines @() $allowed $state
    if ($part.status -ceq 'FAIL') { return $part }
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

function Resolve-PromptPath([string]$Root, [string]$Path) {
    if ($Path.Contains('\') -or $Path -match '(?i)(?:^|/)(?:\.git|\.{1,2})(?:/|$)|[*?\[\]]') { throw 'Use literal repository file paths.' }
    $resolved = Resolve-CirnetPath $Root $Path
    $file = [IO.FileInfo]::new($resolved)
    if ([IO.Directory]::Exists($resolved) -or ($file.Exists -and $file.LinkType -eq 'HardLink')) { throw 'Expected a regular file path.' }
    $file
}

function New-CirnetWorkPromptProposal {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$OriginalTask,
        [Parameter(Mandatory)][string]$Repository,
        [string[]]$SourcePath = @(),
        [switch]$Sensitive,
        [ValidateSet('PROMPT_READY','PROMPT_BLOCKED')][string]$Readiness = 'PROMPT_READY'
    )
    if ($Sensitive) { return New-PromptResult BLOCKED 'Approval preparation requires an approved isolated projection before reading sensitive inputs.' }
    try {
        if ($SourcePath.Length -gt 32) { throw 'Too many source pins.' }
        $bytes = Read-PromptBytes $OriginalTask
        $lines = [Text.UTF8Encoding]::new($false,$true).GetString($bytes) -split '\r?\n'
        if ($lines.Length -gt 1024) { throw 'Too many original task lines.' }
        $owner = Get-ContinuityRepository ([IO.Path]::GetFullPath($Repository, $PWD.ProviderPath))
        $head = Get-ContinuityHead $owner
        if (-not $head) { throw 'Owner needs a pinned HEAD.' }
        $fields = [ordered]@{}
        $permissions = [ordered]@{}
        foreach ($name in $requirementFields) { $fields[$name] = @() }
        foreach ($kind in $permissionKinds) { $permissions[$kind] = @() }
        $sources = [Collections.Specialized.OrderedDictionary]::new([StringComparer]::Ordinal)
        foreach ($path in $SourcePath) {
            $file = Resolve-PromptPath $owner.root $path
            $sources.Add($path, (Get-CirnetPayloadHash (Read-ContinuityBytes $file.FullName 1MB)))
        }
        if ((Get-ContinuityHead $owner) -cne $head) { throw 'HEAD changed during proposal preparation.' }
        [pscustomobject]@{
            status = 'REVIEW_REQUIRED'
            diagnostics = @('Approve line mappings, permissions, checks and pins independently before validation.')
            lines = @(for ($n = 0; $n -lt $lines.Count; $n++) {
                if ($lines[$n].Trim()) { [pscustomobject]@{ line = $n + 1; text = $lines[$n] } }
            })
            proposal = [ordered]@{
                originalSha256 = Get-CirnetPayloadHash $bytes
                repository = $owner.root; revision = $head; readiness = $Readiness; sensitive = $false
                fields = $fields; permissions = $permissions; actions = [ordered]@{}; checks = [ordered]@{}; sources = $sources
            }
        }
    } catch { New-PromptResult BLOCKED 'Cannot prepare scoped approval. Check the task, owned Git root, source paths and size limits.' }
}

function Test-WorkPromptScopeChanges($Baseline, $Current, [string[]]$Effects) {
    $comparison = Compare-ContinuityGit $Baseline $Current
    if ($comparison.Changes.Fields.Count -or $Current.Contains('unmerged')) {
        return New-PromptResult FAIL 'Repository, revision, index or conflict state changed. Reconcile before reporting acceptance.'
    }
    $outside = 0
    foreach ($path in $comparison.Changes.Files.psbase.Keys) {
        $before = [string]$Baseline.files[$path]
        $after = [string]$Current.files[$path]
        $operation = if ($before -cin @('', 'absent') -and $after.StartsWith('file:')) { 'create' }
            elseif ($before.StartsWith('file:') -and $after -cin @('', 'absent')) { 'delete' }
            elseif ($before.StartsWith('file:') -and $after.StartsWith('file:')) { 'modify' }
        if (-not $operation -or ("$($operation): $path" -cnotin $Effects -and
            ($operation -ceq 'delete' -or "write: $path" -cnotin $Effects))) { $outside++ }
    }
    if ($outside) { return New-PromptResult FAIL "$outside file change(s) exceed the approved task effects. Preserve and reconcile the changes." }
    New-PromptResult PASS 'Observed worktree changes match the approved task effects. This comparison is post-execution evidence.'
}

function Test-CirnetWorkPromptFile {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory)][string]$Prompt,
        [Parameter(Mandatory)][string]$OriginalTask,
        [Parameter(Mandatory)][string]$Authorization,
        [Parameter(Mandatory)][string]$AuthorizationSha256,
        [Parameter(Mandatory)][string]$Repository,
        [ValidatePattern('^[0-9a-f]{64}\z')][string]$ReviewedPromptSha256,
        [switch]$RequirePreventiveScope,
        [switch]$CaptureScope,
        [string]$ScopeSnapshot,
        [ValidatePattern('^[0-9a-f]{64}\z')][string]$ScopeSnapshotSha256
    )
    $utf8 = [Text.UTF8Encoding]::new($false,$true)
    $stage = 'approved authorization record'
    try {
        $bytes = Read-PromptBytes $Authorization
        if (-not (Get-CirnetPayloadHash $bytes).Equals($AuthorizationSha256)) {
            return New-PromptResult BLOCKED 'Obtain the authorization digest from the approving operator and verify the record.'
        }
        $record = ConvertFrom-ContinuityJson $bytes -UniqueProperties
        $authorityKeys = @('originalSha256','repository','revision','readiness','sensitive','permissions','fields','actions','checks','sources')
        foreach ($flag in @('preventiveScope','scopeAudit')) {
            if ($record -is [Collections.IDictionary] -and $record.Keys -ccontains $flag) {
                if ($record[$flag] -isnot [bool]) { throw 'Invalid authority flag.' }
                $authorityKeys += $flag
            }
        }
        if (-not (Test-PromptMap $record $authorityKeys) -or
            $record.repository -isnot [string] -or $record.revision -isnot [string] -or $record.originalSha256 -isnot [string] -or
            $record.sensitive -isnot [bool] -or [Array]::IndexOf(@('PROMPT_READY','PROMPT_BLOCKED'),$record.readiness) -lt 0 -or
            $record.originalSha256 -cnotmatch '^[0-9a-f]{64}\z' -or $record.revision -cnotmatch '^(?:[0-9a-f]{40}|[0-9a-f]{64})\z') {
            throw 'Invalid authority.'
        }
        if ($record.sensitive) { return New-PromptResult BLOCKED 'An enforced pre-model read boundary is unavailable. Obtain an approved isolated projection before discovery.' }
        if ($RequirePreventiveScope -or $record['preventiveScope']) {
            return New-PromptResult BLOCKED 'The offline validator cannot enforce a preventive execution boundary.'
        }
        if (($CaptureScope -and $ScopeSnapshot) -or [bool]$ScopeSnapshot -ne [bool]$ScopeSnapshotSha256) {
            return New-PromptResult BLOCKED 'Capture scope first and supply its independently retained snapshot digest for audit.'
        }
        if (($CaptureScope -or $ScopeSnapshot) -and -not $record['scopeAudit']) {
            return New-PromptResult BLOCKED 'Worktree-wide inventory and hashing require independent approval in the authorization record.'
        }

        $stage = 'original task bytes'
        $bytes = Read-PromptBytes $OriginalTask
        if ((Get-CirnetPayloadHash $bytes) -cne $record.originalSha256) { return New-PromptResult FAIL 'The original request differs from its approved digest.' }
        $task = $utf8.GetString($bytes) -split '\r?\n'
        $stage = 'approved repository and revision'
        $root = Resolve-CirnetRoot ([IO.Path]::GetFullPath($Repository, $PWD.ProviderPath))
        if (-not $root.Equals($record.repository)) { return New-PromptResult FAIL 'Select the exact repository root named by the approval.' }
        $owner = Get-ContinuityRepository $root
        if ((Get-ContinuityHead $owner) -cne $record.revision) { return New-PromptResult FAIL 'The approved revision differs from HEAD. Refresh the source review.' }

        $stage = 'permissions and source references'
        if (-not (Test-PromptMap $record.permissions $permissionKinds) -or $record.sources -isnot [Collections.IDictionary]) { throw 'Invalid permissions.' }
        $allowed = [Collections.Generic.List[string]]::new()
        $resolvedPaths = [Collections.Generic.Dictionary[string,IO.FileInfo]]::new([StringComparer]::Ordinal)
        foreach ($kind in $permissionKinds) {
            $values = $record.permissions[$kind]
            if ($values -isnot [array]) { throw 'Explicit arrays required.' }
            foreach ($value in $values) {
                if ($value -isnot [string] -or -not $value.Trim() -or $value -match '[\x00-\x1f]') { throw 'Invalid permission.' }
                if ($record.readiness -ceq 'PROMPT_BLOCKED' -and $kind -cne 'read') { throw 'Invalid blocked scope.' }
                if ($kind -cin @('test','build') -and $value -match '(?:^|[;&|]\s*)(?:git(?:\.exe)?\s+(?:push|commit|merge|reset|clean|stash)|npm(?:\.cmd)?\s+publish|deploy|publish|Remove-Item|Move-Item|rm|mv)(?:\s|$)') {
                    throw 'An explicit destructive or privileged command is not a test or build grant.'
                }
                if ($kind -cin @('read','write')) {
                    if (-not $resolvedPaths.ContainsKey($value)) { $resolvedPaths.Add($value, (Resolve-PromptPath $root $value)) }
                    if ($kind -ceq 'read' -and -not $ScopeSnapshot -and -not $resolvedPaths[$value].Exists) { throw 'Missing read target.' }
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
            if (-not $ScopeSnapshot -and (Get-CirnetHash $resolvedPaths[$path].FullName) -cne $record.sources[$path]) {
                return New-PromptResult FAIL 'Pinned source bytes changed. Reinspect before citing evidence.'
            }
            $taskEffects.Add("read: $path")
            $required.Add((New-PromptRequirement "source $path" @("Source: $path")))
        }

        $stage = 'approved requirement lines and acceptance mappings'
        if (-not (Test-PromptMap $record.fields $requirementFields) -or $record.checks -isnot [Collections.IDictionary]) { throw 'Invalid coverage.' }
        $labels = @{}
        $explicit = @($record.fields.action) + @($record.fields.target) + @($record.fields.transformation)
        foreach ($field in $requirementFields) {
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
                $forms = @($originalLine; if ($n -notin $explicit) { $name })
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
                    if ($effect -isnot [string] -or (-not $allowed.Contains($effect) -and
                        ($effect -cnotmatch '^(?:create|modify|delete): (.+)$' -or -not $allowed.Contains("write: $($Matches[1])")))) {
                        throw 'Unapproved operation.'
                    }
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
        if ($result.status -ceq 'PASS' -and ($CaptureScope -or $ScopeSnapshot)) {
            $stage = 'scope snapshot and reconciliation'
            if (@($taskEffects | Where-Object { $_ -cmatch '^(export|privileged): ' }).Count) {
                return New-PromptResult BLOCKED 'Export or privileged effects need separate enforcement and evidence.'
            }
            $expected = $null
            if ($ScopeSnapshot) {
                $snapshotBytes = Read-PromptBytes $ScopeSnapshot
                if ((Get-CirnetPayloadHash $snapshotBytes) -cne $ScopeSnapshotSha256) {
                    return New-PromptResult FAIL 'Scope snapshot differs from the independently retained digest.'
                }
                $saved = ConvertFrom-ContinuityJson $snapshotBytes -UniqueProperties
                $expected = $saved.scopeSnapshot
                if (-not (Test-PromptMap $expected @('authorizationSha256','promptSha256','git')) -or
                    $expected.authorizationSha256 -cne $AuthorizationSha256 -or $expected.promptSha256 -cne $digest) {
                    return New-PromptResult FAIL 'Scope snapshot belongs to a different handoff or approval.'
                }
            }
            $git = Get-ContinuityGitSnapshot $owner -IncludeIgnored
            if ($git.head -cne $record.revision -or $git.Contains('unmerged')) {
                return New-PromptResult FAIL 'HEAD or conflict state prevents scope reconciliation.'
            }
            if ($expected) { return Test-WorkPromptScopeChanges $expected.git $git $taskEffects }
            foreach ($path in $record.sources.Keys) {
                if ($git.files[$path] -cne "file:$($record.sources[$path])") {
                    return New-PromptResult FAIL 'Pinned source bytes changed during scope capture.'
                }
            }
            $result | Add-Member -NotePropertyName scopeSnapshot -NotePropertyValue @{
                authorizationSha256 = $AuthorizationSha256; promptSha256 = $digest; git = $git
            }
            if ((ConvertTo-ContinuityBytes $result).Length -gt 1MB) { return New-PromptResult BLOCKED 'Scope snapshot exceeds the portable audit size limit.' }
        }
        return $result
    } catch {
        # Inputs may contain sensitive strings. Report the stage, never their values.
        New-PromptResult BLOCKED "Cannot verify $stage. Check the approved format, paths and access."
    }
}

Export-ModuleMember -Function Test-CirnetWorkPromptFile, Test-CirnetCandidatePrompt, New-CirnetWorkPromptProposal

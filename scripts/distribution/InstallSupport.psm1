Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot '../shared/Bundle.psm1')

$script:Utf8 = [Text.UTF8Encoding]::new($false, $true)
$script:BeginMarker = '<!-- CIRNET TOOLKIT BEGIN -->'
$script:EndMarker = '<!-- CIRNET TOOLKIT END -->'
$script:ClaudeImport = '@cirnet/AGENTS.md'
$script:DefaultSkills = 'cirnet-implementation', 'cirnet-debugging', 'cirnet-verification'

function Get-CirnetInstallLayout {
    param([string]$Agent, [string]$HomeRoot, [string]$CodexHome)

    $homePath = if ($HomeRoot) { Resolve-CirnetRoot -Path $HomeRoot } else { Resolve-CirnetRoot -Path $HOME }
    $clientPath = $homePath
    $baselineScope = 'home'
    $common = $null
    switch ($Agent) {
        'Codex' {
            $clientPath = if ($CodexHome) { Resolve-CirnetRoot -Path $CodexHome }
            elseif (-not $HomeRoot -and $env:CODEX_HOME) { Resolve-CirnetRoot -Path $env:CODEX_HOME }
            else { Resolve-CirnetPath -Root $homePath -Relative '.codex' }
            $baselineScope = 'client'
            $baseline = 'AGENTS.md'
            $skills = '.agents/skills'
            $otherSkills = @(@{ Scope = 'client'; Path = 'skills' })
            $otherRules = @(@{ Scope = 'client'; Path = 'AGENTS.override.md' })
        }
        'Claude' {
            $baseline = '.claude/CLAUDE.md'
            $common = '.claude/cirnet/AGENTS.md'
            $skills = '.claude/skills'
            $otherSkills = @()
            $otherRules = @()
        }
        'Antigravity' {
            $baseline = '.gemini/AGENTS.md'
            $skills = '.gemini/config/skills'
            $otherSkills = @(
                @{ Scope = 'home'; Path = '.gemini/antigravity-cli/skills' },
                @{ Scope = 'home'; Path = '.gemini/skills' }
            )
            $otherRules = @(
                @{ Scope = 'home'; Path = '.gemini/GEMINI.md' },
                @{ Scope = 'home'; Path = '.gemini/config/AGENTS.md' },
                @{ Scope = 'home'; Path = '.gemini/config/GEMINI.md' }
            )
        }
    }
    if ($Agent -ne 'Codex' -and $CodexHome) { throw '-CodexHome applies only to Codex.' }
    $id = $Agent.ToLowerInvariant()
    [pscustomobject]@{
        Agent = $id; Home = $homePath; Client = $clientPath
        Baseline = $baseline; BaselineScope = $baselineScope; Common = $common; Skills = $skills
        OtherRules = $otherRules; OtherSkills = $otherSkills
        State = ".cirnet-toolkit/$id/state.json"; Backups = ".cirnet-toolkit/$id/backups"
    }
}

function Get-TargetPath($Layout, [string]$Scope, [string]$Relative) {
    $root = if ($Scope -ceq 'client') { $Layout.Client } else { $Layout.Home }
    Resolve-CirnetPath -Root $root -Relative $Relative
}

function Read-FileBytes([string]$Path) {
    if ([IO.Directory]::Exists($Path)) { throw "Conflict: expected a file at $Path." }
    if ([IO.File]::Exists($Path)) { return ,([IO.File]::ReadAllBytes($Path)) }
}

function Get-BytesHash([AllowNull()][byte[]]$Bytes) {
    if ($null -eq $Bytes) { return $null }
    $sha256 = [Security.Cryptography.SHA256]::Create()
    try { [BitConverter]::ToString($sha256.ComputeHash($Bytes)).Replace('-', '').ToLowerInvariant() }
    finally { $sha256.Dispose() }
}

function ConvertFrom-FileJson([byte[]]$Bytes, [string]$Path) {
    try {
        $data = $script:Utf8.GetString($Bytes) | ConvertFrom-Json -AsHashtable
        if ($data -isnot [Collections.IDictionary] -or $data['format'] -isnot [long] -or $data['format'] -ne 1) {
            throw 'Invalid format.'
        }
        return $data
    } catch { throw "Invalid CIRNET metadata at ${Path}: $_" }
}

function ConvertTo-FileJson($Data) {
    return ,$script:Utf8.GetBytes(($Data | ConvertTo-Json -Depth 8) + "`n")
}

function Test-OwnedRelative($Layout, [string]$Relative) {
    if ($Layout.Common -and $Relative -ceq $Layout.Common) { return $true }
    $prefix = [regex]::Escape($Layout.Skills)
    return $Relative -cmatch "^$prefix/cirnet-[a-z0-9]+(?:-[a-z0-9]+)*/.+$"
}

function Read-Ownership($Layout, [byte[]]$Bytes, [string]$Path) {
    $owned = [Collections.Generic.Dictionary[string,string]]::new([StringComparer]::OrdinalIgnoreCase)
    if ($null -eq $Bytes) { return $owned }
    $data = ConvertFrom-FileJson $Bytes $Path
    if ($data['agent'] -isnot [string] -or $data['agent'] -cne $Layout.Agent -or $data['files'] -isnot [array]) {
        throw "Invalid CIRNET ownership metadata at $Path."
    }
    foreach ($record in $data['files']) {
        if ($record -isnot [Collections.IDictionary] -or $record['relative'] -isnot [string] -or
            -not (Test-OwnedRelative $Layout $record['relative']) -or -not (Test-OptionalHash $record['hash']) -or
            $null -eq $record['hash'] -or
            -not $owned.TryAdd($record['relative'], $record['hash'])) {
            throw "Invalid or duplicate ownership record at $Path."
        }
        [void](Get-TargetPath $Layout 'home' $record['relative'])
    }
    return $owned
}

function Get-ManagedBlock([byte[]]$Bytes, [string]$Path) {
    try { $text = if ($null -eq $Bytes) { '' } else { $script:Utf8.GetString($Bytes) } }
    catch { throw "Conflict: expected UTF-8 in $Path. $_" }
    if ($text.Contains("`0")) { throw "Conflict: binary instruction file at $Path." }
    $bom = ''
    if ($text.StartsWith([string][char]0xfeff, [StringComparison]::Ordinal)) {
        $bom = $text.Substring(0, 1)
        $text = $text.Substring(1)
    }
    $candidates = @([regex]::Matches($text, '(?im)^[^\r\n]*<!--\s*/?CIRNET TOOLKIT[^\r\n]*'))
    $begin = [regex]::Match($text, '(?m)^' + [regex]::Escape($script:BeginMarker) + '(\r?\n)')
    $end = [regex]::Match($text, '(?m)^' + [regex]::Escape($script:EndMarker) + '(?=\r?$)')
    if ($candidates.Count -eq 0) {
        $newline = if ($text.Contains("`r`n")) { "`r`n" } else { "`n" }
        $separator = if ($text.Length -eq 0) { '' }
        elseif ($text.EndsWith("`n", [StringComparison]::Ordinal)) { $newline }
        else { $newline + $newline }
        return [pscustomobject]@{ Present = $false; Before = $bom + $text + $separator; After = $newline; Body = ''; Newline = $newline; Outside = $text }
    }
    if ($candidates.Count -ne 2 -or -not $begin.Success -or -not $end.Success -or $begin.Index -ge $end.Index) {
        throw "Conflict: malformed or duplicate CIRNET block in $Path."
    }
    $bodyStart = $begin.Index + $begin.Length
    $body = $text.Substring($bodyStart, $end.Index - $bodyStart).TrimEnd("`r", "`n")
    $before = $text.Substring(0, $begin.Index)
    $after = $text.Substring($end.Index + $end.Length)
    [pscustomobject]@{
        Present = $true; Before = $bom + $before; After = $after; Body = $body
        Newline = $begin.Groups[1].Value; Outside = $before + $after
    }
}

function Format-ManagedBlock($Block, [string]$Body) {
    $newline = $Block.Newline
    $managed = $Body.TrimEnd("`r", "`n") -replace '\r?\n', $newline
    $text = $Block.Before + $script:BeginMarker + $newline + $managed + $newline + $script:EndMarker + $Block.After
    return ,$script:Utf8.GetBytes($text)
}

function Assert-ClaudeAdapter($Block, [string]$Path, [string]$Target) {
    if ($Block.Present -and $Block.Body -cne $script:ClaudeImport) {
        throw "Conflict: the CIRNET block in $Path does not contain the expected Claude import."
    }
    foreach ($match in [regex]::Matches($Block.Outside, '(?m)(?:^|\s)@((?:\\ |[^\s`])+)')) {
        $candidate = $match.Groups[1].Value.Replace('\ ', ' ')
        if ($candidate -notmatch 'AGENTS\.md$') { continue }
        if ($candidate.StartsWith('~/')) {
            # The adapter lives under .claude, one level below the selected home.
            $candidate = '../' + $candidate.Substring(2)
        }
        try { $resolved = [IO.Path]::GetFullPath($candidate.Replace('\', '/'), [IO.Path]::GetDirectoryName($Path)) }
        catch { continue }
        if ([string]::Equals($resolved, $Target, [StringComparison]::OrdinalIgnoreCase)) {
            throw "Conflict: duplicate or unowned CIRNET import in $Path."
        }
    }
}

function New-Change([string]$Scope, [string]$Relative, [string]$Path, [byte[]]$Before, [byte[]]$After) {
    [pscustomobject]@{
        Scope = $Scope; Relative = $Relative; Path = $Path; Before = $Before; After = $After
        BeforeHash = Get-BytesHash $Before; AfterHash = Get-BytesHash $After
        Stage = $null; Undo = $null
    }
}

function Write-ScopeWarnings($Layout, [string[]]$Skills) {
    foreach ($location in $Layout.OtherRules) {
        try { $path = Get-TargetPath $Layout $location.Scope $location.Path }
        catch { Write-Warning "Could not inspect additional instruction source $($location.Path): $_"; continue }
        if ([IO.File]::Exists($path)) { Write-Warning "Another instruction source may affect discovery or precedence: $path" }
    }
    $primary = @{ Scope = 'home'; Path = $Layout.Skills }
    foreach ($location in (@($primary) + $Layout.OtherSkills)) {
        try { $path = Get-TargetPath $Layout $location.Scope $location.Path }
        catch {
            if ($location -eq $primary) { throw }
            Write-Warning "Could not inspect additional skill scope $($location.Path): $_"
            continue
        }
        if (-not [IO.Directory]::Exists($path)) { continue }
        foreach ($directory in Get-ChildItem -LiteralPath $path -Directory -Force) {
            if ($directory.Attributes -band [IO.FileAttributes]::ReparsePoint) {
                Write-Warning "Linked skill may affect name resolution: $($directory.FullName)"
                continue
            }
            $name = $directory.Name
            $skill = Join-Path $directory.FullName 'SKILL.md'
            if ($location -eq $primary -and $name -in $Skills) { continue }
            if ($name -in ('implementation', 'debugging', 'verification')) {
                Write-Warning "Review legacy skill ownership before migrating: $($directory.FullName)"
            }
            if ([IO.File]::Exists($skill) -and -not (([IO.File]::GetAttributes($skill)) -band [IO.FileAttributes]::ReparsePoint)) {
                $header = [regex]::Match([IO.File]::ReadAllText($skill), '\A---\r?\n(?:(?!^---\r?$)[\s\S])*?^name:[ \t]*["'']?([a-z0-9-]+)["'']?[ \t]*\r?$', 'Multiline')
                if ($header.Success) { $name = $header.Groups[1].Value }
            }
            if ($name -in $Skills) { Write-Warning "A skill with the same name may also be discovered: $skill" }
        }
    }
}

function Test-ContainedPath([string]$Root, [string]$Path) {
    $relative = [IO.Path]::GetRelativePath($Root, $Path).Replace('\', '/')
    return ($relative -ne '..' -and -not $relative.StartsWith('../') -and -not [IO.Path]::IsPathRooted($relative))
}

function Get-CirnetInstallPlan {
    param([string]$Source, $Layout, [AllowEmptyCollection()][string[]]$Skills)

    $bundle = Read-CirnetBundle -Source $Source
    if ($null -eq $Skills) { $Skills = $script:DefaultSkills }
    $selected = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($name in $Skills) {
        if ($name -cnotmatch '^cirnet-[a-z0-9]+(?:-[a-z0-9]+)*$' -or -not $selected.Add($name)) {
            throw "Invalid or duplicate selected skill: '$name'."
        }
        if ("skills/$name/SKILL.md" -cnotin $bundle.Manifest.files.path) { throw "Skill '$name' is absent from the bundle." }
    }
    Write-ScopeWarnings $Layout $Skills
    $statePath = Get-TargetPath $Layout 'home' $Layout.State
    $stateBytes = Read-FileBytes $statePath
    $owned = Read-Ownership $Layout $stateBytes $statePath
    $entries = [Collections.Generic.List[object]]::new()
    $baseline = Get-TargetPath $Layout $Layout.BaselineScope $Layout.Baseline
    $sourcePath = Resolve-CirnetPath -Root $bundle.Root -Relative 'AGENTS.md'
    $commonBytes = Read-FileBytes $sourcePath
    $commonHash = ($bundle.Manifest.files | Where-Object path -CEQ 'AGENTS.md').sha256
    if ((Get-BytesHash $commonBytes) -cne $commonHash) { throw "Bundle changed while reading $sourcePath." }
    $baselineBytes = Read-FileBytes $baseline
    $block = Get-ManagedBlock $baselineBytes $baseline
    $body = $script:Utf8.GetString($commonBytes)
    if ($body.Contains("`0") -or $body.StartsWith([string][char]0xfeff, [StringComparison]::Ordinal) -or
        $body -match '<!--\s*/?CIRNET TOOLKIT') {
        throw "Invalid baseline text or reserved managed block markers in $sourcePath."
    }
    $payload = [ordered]@{}
    if ($Layout.Common) {
        $commonPath = Get-TargetPath $Layout 'home' $Layout.Common
        Assert-ClaudeAdapter $block $baseline $commonPath
        $payload[$Layout.Common] = $commonBytes
        $body = $script:ClaudeImport
    }
    $entries.Add((New-Change $Layout.BaselineScope $Layout.Baseline $baseline $baselineBytes (Format-ManagedBlock $block $body)))

    foreach ($name in $Skills) {
        $relative = "$($Layout.Skills)/$name"
        $skillPath = Get-TargetPath $Layout 'home' $relative
        if ([IO.File]::Exists($skillPath)) { throw "Conflict: expected a skill directory at $skillPath." }
        if ([IO.Directory]::Exists($skillPath) -and -not $owned.ContainsKey("$relative/SKILL.md") -and
            @(Get-ChildItem -LiteralPath $skillPath -Force).Count -gt 0) {
            throw "Conflict: existing skill directory has no CIRNET ownership record: $skillPath"
        }
    }
    foreach ($file in $bundle.Manifest.files) {
        $parts = $file.path.Split('/', 3)
        if ($parts.Count -lt 3 -or $parts[0] -cne 'skills' -or -not $selected.Contains($parts[1])) { continue }
        $path = Resolve-CirnetPath -Root $bundle.Root -Relative $file.path
        $bytes = Read-FileBytes $path
        if ((Get-BytesHash $bytes) -cne $file.sha256) { throw "Bundle changed while reading $path." }
        $payload["$($Layout.Skills)/$($parts[1])/$($parts[2])"] = $bytes
    }
    foreach ($relative in $owned.Keys) {
        if ($relative.StartsWith("$($Layout.Skills)/", [StringComparison]::Ordinal) -and
            $selected.Contains($relative.Split('/')[$Layout.Skills.Split('/').Count]) -and -not $payload.Contains($relative)) {
            Write-Warning "Previously installed resource is absent from the selected bundle and was retained: $relative"
        }
    }
    foreach ($relative in $payload.Keys) {
        $path = Get-TargetPath $Layout 'home' $relative
        $before = Read-FileBytes $path
        if ($null -ne $before -and (-not $owned.ContainsKey($relative) -or (Get-BytesHash $before) -cne $owned[$relative])) {
            throw "Conflict: existing file has no matching CIRNET ownership hash: $path"
        }
        $entry = New-Change 'home' $relative $path $before $payload[$relative]
        $entries.Add($entry)
        $owned[$relative] = $entry.AfterHash
    }
    if ($owned.Count -gt 0) {
        $files = @(foreach ($relative in ($owned.Keys | Sort-Object -CaseSensitive)) { [ordered]@{ relative = $relative; hash = $owned[$relative] } })
        $newState = ConvertTo-FileJson ([ordered]@{ format = 1; agent = $Layout.Agent; files = $files })
        $entries.Add((New-Change 'home' $Layout.State $statePath $stateBytes $newState))
    }
    $paths = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($entry in $entries) {
        if (-not $paths.Add($entry.Path) -or (Test-ContainedPath $bundle.Root $entry.Path)) {
            throw "Conflict: overlapping destination or bundle path: $($entry.Path)"
        }
    }
    $backupRoot = Get-TargetPath $Layout 'home' $Layout.Backups
    if (Test-ContainedPath $bundle.Root $backupRoot) { throw "Conflict: backup location overlaps bundle path: $backupRoot" }
    [pscustomobject]@{ Layout = $Layout; Entries = $entries; Manifest = $bundle.Manifest }
}

function Show-CirnetInstallPlan {
    param($Plan, [string]$Action)
    Write-Host "$Action for $($Plan.Layout.Agent) under $($Plan.Layout.Home)"
    foreach ($entry in $Plan.Entries) {
        $status = if ($entry.BeforeHash -ceq $entry.AfterHash) { 'unchanged' }
        elseif ($null -eq $entry.BeforeHash) { 'create' }
        elseif ($null -eq $entry.AfterHash) { 'remove' }
        else { 'replace' }
        Write-Host "  $status $($entry.Path)"
    }
}

function Assert-Unchanged($Layout, $Entry) {
    $path = Get-TargetPath $Layout $Entry.Scope $Entry.Relative
    if ((Get-BytesHash (Read-FileBytes $path)) -cne $Entry.BeforeHash) {
        throw "Conflict: destination changed since planning: $path"
    }
}

function Save-CirnetInstall {
    param($Plan, [object[]]$Changes)

    $layout = $Plan.Layout
    foreach ($entry in $Plan.Entries) { Assert-Unchanged $layout $entry }
    $id = [guid]::NewGuid().ToString('N')
    $backup = Get-TargetPath $layout 'home' "$($layout.Backups)/$id"
    $receipt = Get-TargetPath $layout 'home' "$($layout.Backups)/$id/receipt.json"
    Write-Host "Backup receipt $receipt"
    try { Invoke-CirnetChanges -Layout $layout -Changes $Changes -BeforeWrite { Write-InstallBackup $Plan $Changes $backup $receipt } }
    catch { throw "Install failed. Backup receipt destination was '$receipt'. $_" }
    Write-Host 'Installed selected CIRNET rules and skills.'
}

function Write-InstallBackup($Plan, [object[]]$Changes, [string]$Backup, [string]$Receipt) {
    $layout = $Plan.Layout
    [void][IO.Directory]::CreateDirectory((Resolve-CirnetRoot -Path $Backup))
    $records = @(for ($index = 0; $index -lt $Changes.Count; $index++) {
        $entry = $Changes[$index]
        $oldFile = $null
        if ($null -ne $entry.Before) {
            $oldFile = "file-$index.bin"
            $oldPath = Resolve-CirnetPath -Root $backup -Relative $oldFile
            Write-NewFile $oldPath $entry.Before
            if ((Get-CirnetHash -Path $oldPath) -cne $entry.BeforeHash) { throw "Backup hash mismatch at $oldPath." }
        }
        [ordered]@{
            scope = $entry.Scope; relative = $entry.Relative; before = $entry.BeforeHash
            after = $entry.AfterHash; backup = $oldFile
        }
    })
    $data = [ordered]@{
        format = 1; agent = $layout.Agent; home = $layout.Home; client = $layout.Client
        version = $Plan.Manifest.version; revision = $Plan.Manifest.revision; dirty = $Plan.Manifest.dirty; files = $records
    }
    $receiptBytes = ConvertTo-FileJson $data
    Write-NewFile (Resolve-CirnetRoot -Path $Receipt) $receiptBytes
    if ((Get-CirnetHash -Path $receipt) -cne (Get-BytesHash $receiptBytes)) {
        throw "Backup receipt hash mismatch at $receipt."
    }
}

function Test-OptionalHash($Value) {
    return ($null -eq $Value -or ($Value -is [string] -and $Value -cmatch '^[0-9a-f]{64}\z'))
}

function Get-CirnetRestorePlan {
    param($Layout, [string]$Backup)

    $receipt = Resolve-CirnetRoot -Path $Backup
    $relativeReceipt = [IO.Path]::GetRelativePath($Layout.Home, $receipt).Replace('\', '/')
    if ($relativeReceipt -cnotmatch ('^' + [regex]::Escape($Layout.Backups) + '/[0-9a-f]{32}/receipt\.json\z')) {
        throw "Backup must be a receipt inside $($Layout.Home)/$($Layout.Backups)."
    }
    $receiptBytes = Read-FileBytes $receipt
    if ($null -eq $receiptBytes) { throw "Backup receipt missing at $receipt." }
    $data = ConvertFrom-FileJson $receiptBytes $receipt
    if ($data['agent'] -isnot [string] -or $data['agent'] -cne $Layout.Agent -or
        $data['home'] -isnot [string] -or $data['home'] -cne $Layout.Home -or
        $data['client'] -isnot [string] -or $data['client'] -cne $Layout.Client -or
        $data['version'] -isnot [string] -or $data['version'] -cnotmatch '^\d+\.\d+\.\d+(?:-[a-zA-Z0-9.-]+)?\z' -or
        $data['revision'] -isnot [string] -or $data['revision'] -cnotmatch '^[0-9a-f]{40}\z' -or
        $data['dirty'] -isnot [bool] -or $data['files'] -isnot [array] -or $data['files'].Count -eq 0) {
        throw "Invalid backup receipt or root mismatch at $receipt."
    }
    $entries = [Collections.Generic.List[object]]::new()
    $seen = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    $backupFiles = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    $backupRoot = [IO.Path]::GetDirectoryName($receipt)
    for ($index = 0; $index -lt $data['files'].Count; $index++) {
        $record = $data['files'][$index]
        if ($record -isnot [Collections.IDictionary] -or $record['scope'] -isnot [string] -or $record['relative'] -isnot [string] -or
            -not $record.Contains('before') -or -not (Test-OptionalHash $record['before']) -or
            $record['after'] -isnot [string] -or -not (Test-OptionalHash $record['after'])) {
            throw "Invalid backup record $index in $receipt."
        }
        $scope = $record['scope']
        $relative = $record['relative']
        $baseline = $scope -ceq $Layout.BaselineScope -and $relative -ceq $Layout.Baseline
        $owned = $scope -ceq 'home' -and (Test-OwnedRelative $Layout $relative)
        $state = $scope -ceq 'home' -and $relative -ceq $Layout.State
        if (-not ($baseline -or $owned -or $state)) { throw "Invalid backup destination '$relative' in $receipt." }
        $target = Get-TargetPath $Layout $scope $relative
        if (-not $seen.Add($target)) { throw "Duplicate backup destination $target." }
        $current = Read-FileBytes $target
        if ((Get-BytesHash $current) -cne $record['after']) { throw "Restore conflict: file changed since installation: $target" }
        $previous = $null
        if ($null -ne $record['before']) {
            if ($record['backup'] -isnot [string] -or $record['backup'] -cnotmatch '^file-[0-9]+\.bin\z' -or
                -not $backupFiles.Add($record['backup'])) {
                throw "Invalid or repeated backup file name in record $index at $receipt."
            }
            $file = Resolve-CirnetPath -Root $backupRoot -Relative $record['backup']
            $previous = Read-FileBytes $file
            if ((Get-BytesHash $previous) -cne $record['before']) { throw "Missing or corrupt backup at $file." }
        } elseif ($null -ne $record['backup']) { throw "Unexpected backup file name in record $index at $receipt." }
        if ($baseline) {
            $currentBlock = Get-ManagedBlock $current $target
            $previousBlock = Get-ManagedBlock $previous $target
            $expected = Format-ManagedBlock $previousBlock $currentBlock.Body
            if (-not $currentBlock.Present -or (Get-BytesHash $expected) -cne $record['after']) {
                throw "Invalid backup: restore would change bytes outside the managed block in $target."
            }
        }
        $entries.Add((New-Change $scope $relative $target $current $previous))
    }
    $statePath = Get-TargetPath $Layout 'home' $Layout.State
    $stateEntry = $entries | Where-Object Path -CEQ $statePath
    $stateBefore = Read-FileBytes $statePath
    $stateAfter = $stateBefore
    if ($stateEntry) { $stateAfter = $stateEntry.After }
    $ownedBefore = Read-Ownership $Layout $stateBefore $statePath
    $ownedAfter = Read-Ownership $Layout $stateAfter $statePath
    $changes = [Collections.Generic.Dictionary[string,object]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($entry in $entries) {
        if ($entry.Scope -cne 'home' -or -not (Test-OwnedRelative $Layout $entry.Relative)) { continue }
        if (-not $ownedBefore.ContainsKey($entry.Relative) -or $ownedBefore[$entry.Relative] -cne $entry.BeforeHash -or
            $ownedAfter[$entry.Relative] -cne $entry.AfterHash) {
            throw "Backup conflicts with ownership records for $($entry.Path)."
        }
        $changes.Add($entry.Relative, $entry)
    }
    foreach ($relative in (@($ownedBefore.Keys) + @($ownedAfter.Keys) | Sort-Object -Unique)) {
        if ($ownedBefore[$relative] -cne $ownedAfter[$relative] -and -not $changes.ContainsKey($relative)) {
            throw "Backup omits a file whose ownership would change: $relative"
        }
    }
    [pscustomobject]@{ Layout = $Layout; Entries = $entries }
}

function Write-NewFile([string]$Path, [byte[]]$Bytes) {
    $stream = [IO.File]::Open($Path, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try { $stream.Write($Bytes, 0, $Bytes.Length); $stream.Flush($true) }
    finally { $stream.Dispose() }
}

function Initialize-Change($Layout, $Entry, $CreatedDirectories) {
    Assert-Unchanged $Layout $Entry
    $parent = [IO.Path]::GetDirectoryName($Entry.Path)
    $missing = [Collections.Generic.Stack[string]]::new()
    for ($part = $parent; -not [IO.Directory]::Exists($part); $part = [IO.Path]::GetDirectoryName($part)) {
        [void](Resolve-CirnetRoot -Path $part)
        $missing.Push($part)
    }
    foreach ($part in $missing) {
        [void][IO.Directory]::CreateDirectory($part)
        $CreatedDirectories.Add($part)
    }
    if ($null -ne $Entry.Before) {
        $file = [IO.File]::Open($Entry.Path, 'Open', 'ReadWrite', 'ReadWrite, Delete')
        $file.Dispose()
    }
    $token = [guid]::NewGuid().ToString('N')
    $Entry.Undo = Resolve-CirnetPath -Root $parent -Relative ".cirnet-$token.undo"
    $Entry.Stage = Resolve-CirnetPath -Root $parent -Relative ".cirnet-$token.stage"
    $bytes = $Entry.After
    if ($null -eq $bytes) { $bytes = [byte[]]::new(0) }
    Write-NewFile $Entry.Stage $bytes
    if ((Get-CirnetHash -Path $Entry.Stage) -cne (Get-BytesHash $bytes)) { throw "Staging hash mismatch at $($Entry.Stage)." }
}

function Set-Change($Entry) {
    if ($null -eq $Entry.After) { [IO.File]::Move($Entry.Path, $Entry.Undo) }
    elseif ($null -eq $Entry.Before) { [IO.File]::Move($Entry.Stage, $Entry.Path) }
    else { [IO.File]::Replace($Entry.Stage, $Entry.Path, $Entry.Undo, $false) }
}

function Undo-Change($Layout, $Entry) {
    $path = Get-TargetPath $Layout $Entry.Scope $Entry.Relative
    $hash = Get-BytesHash (Read-FileBytes $path)
    if ($hash -cne $Entry.AfterHash) { throw "File changed during rollback: $path" }
    if ($null -eq $Entry.Before) { [IO.File]::Delete($path) }
    elseif ((Get-CirnetHash -Path (Resolve-CirnetRoot -Path $Entry.Undo)) -cne $Entry.BeforeHash) {
        throw "Rollback copy mismatch for $path."
    } elseif ($null -eq $Entry.After) { [IO.File]::Move($Entry.Undo, $path) }
    else { [IO.File]::Move($Entry.Undo, $path, $true) }
}

function Invoke-CirnetChanges {
    param($Layout, [object[]]$Changes, [scriptblock]$BeforeWrite)

    $applied = [Collections.Generic.Stack[object]]::new()
    $directories = [Collections.Generic.List[string]]::new()
    $preserve = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    try {
        foreach ($entry in $Changes) { Initialize-Change $Layout $entry $directories }
        if ($BeforeWrite) { & $BeforeWrite }
        foreach ($entry in $Changes) {
            Assert-Unchanged $Layout $entry
            [void](Resolve-CirnetRoot -Path $entry.Stage)
            [void](Resolve-CirnetRoot -Path $entry.Undo)
            try { Set-Change $entry }
            catch {
                # A failed filesystem replacement can still leave a partially changed destination.
                $current = Get-BytesHash (Read-FileBytes $entry.Path)
                if ($current -cne $entry.BeforeHash) { $applied.Push($entry) }
                throw
            }
            $applied.Push($entry)
        }
    } catch {
        $cause = $_
        $failure = @(foreach ($entry in $applied) {
            try { Undo-Change $Layout $entry }
            catch { [void]$preserve.Add($entry.Undo); "$($entry.Path): $_" }
        })
        if ($failure.Count -gt 0) {
            throw "Write failed ($cause). Rollback incomplete for $($failure -join '; '). Recovery copies retained where available."
        }
        throw "Write failed ($cause). Target changes rolled back ($($applied.Count) files)."
    } finally {
        foreach ($entry in $Changes) {
            foreach ($temporary in @($entry.Stage, $entry.Undo)) {
                if (-not $temporary -or $preserve.Contains($temporary)) { continue }
                try { [IO.File]::Delete((Resolve-CirnetRoot -Path $temporary)) }
                catch { Write-Warning "Could not remove temporary file: $temporary. $_" }
            }
        }
        for ($i = $directories.Count - 1; $i -ge 0; $i--) {
            try {
                $directory = Resolve-CirnetRoot -Path $directories[$i]
                if ([IO.Directory]::Exists($directory) -and -not @(Get-ChildItem -LiteralPath $directory -Force).Count) {
                    [IO.Directory]::Delete($directory)
                }
            } catch { Write-Warning "Could not remove empty staging directory $($directories[$i]). $_" }
        }
    }
}

Export-ModuleMember -Function Get-CirnetInstallLayout, Get-CirnetInstallPlan, Show-CirnetInstallPlan, Save-CirnetInstall, Get-CirnetRestorePlan, Invoke-CirnetChanges

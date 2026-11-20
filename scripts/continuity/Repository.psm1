#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot '../shared/Bundle.psm1')

$utf8 = [Text.UTF8Encoding]::new($false, $true)
$pathComparison = if ($IsWindows) { [StringComparison]::OrdinalIgnoreCase } else { [StringComparison]::Ordinal }

function Get-ContinuityTextHash([string]$Text) {
    Get-CirnetPayloadHash -Bytes $utf8.GetBytes($Text)
}

function ConvertTo-CanonicalValue($Value) {
    if ($Value -is [Collections.IDictionary]) {
        $keys = [string[]]@($Value.psbase.Keys)
        [Array]::Sort($keys, [StringComparer]::Ordinal)
        $ordered = [Collections.Specialized.OrderedDictionary]::new([StringComparer]::Ordinal)
        foreach ($key in $keys) { $ordered[$key] = ConvertTo-CanonicalValue $Value[$key] }
        return $ordered
    }
    if ($Value -is [array]) {
        return ,@(foreach ($element in $Value) { ConvertTo-CanonicalValue $element })
    }
    $Value
}

function Get-ContinuityValueHash($Value) {
    Get-ContinuityTextHash (ConvertTo-Json -InputObject (ConvertTo-CanonicalValue $Value) -Depth 20 -Compress)
}

function Invoke-ContinuityGit {
    param([string]$Root, [string[]]$GitArgs, [int[]]$Allowed = @(0), [string]$StandardInput)

    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = (Get-Command git -CommandType Application | Select-Object -First 1).Path
    $info.WorkingDirectory = $Root
    $info.UseShellExecute = $false
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $info.StandardOutputEncoding = $utf8
    $info.RedirectStandardInput = $PSBoundParameters.ContainsKey('StandardInput')
    foreach ($name in @('GIT_DIR', 'GIT_WORK_TREE', 'GIT_INDEX_FILE', 'GIT_COMMON_DIR', 'GIT_NAMESPACE',
            'GIT_OBJECT_DIRECTORY', 'GIT_ALTERNATE_OBJECT_DIRECTORIES', 'GIT_CONFIG_PARAMETERS', 'GIT_CONFIG_COUNT')) {
        [void]$info.Environment.Remove($name)
    }
    $info.Environment['GIT_NO_LAZY_FETCH'] = '1'
    $info.Environment['GIT_ALLOW_PROTOCOL'] = ''
    $info.Environment['GIT_TERMINAL_PROMPT'] = '0'
    $options = @('--no-optional-locks', '--no-replace-objects', '-c', 'core.fsmonitor=false', '-c', 'core.untrackedCache=false')
    if ($GitArgs[0] -cne 'check-ignore') { $options += '--literal-pathspecs' }
    foreach ($arg in $options + $GitArgs) {
        $info.ArgumentList.Add($arg)
    }
    $process = [Diagnostics.Process]::Start($info)
    try {
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.BaseStream.CopyToAsync([IO.Stream]::Null)
        if ($info.RedirectStandardInput) {
            $bytes = $utf8.GetBytes($StandardInput)
            $inputWrite = $process.StandardInput.BaseStream.WriteAsync($bytes, 0, $bytes.Length)
            if (-not $inputWrite.Wait(30000)) { throw 'Git input timed out.' }
            $process.StandardInput.Close()
        }
        if (-not $process.WaitForExit(30000)) { throw "Read-only Git timed out ($($GitArgs[0]))." }
        if ($process.ExitCode -notin $Allowed) {
            throw "Read-only Git failed ($($GitArgs[0]), exit $($process.ExitCode))."
        }
        $null = $stderr.GetAwaiter().GetResult()
        @{ Text = $stdout.GetAwaiter().GetResult(); Code = $process.ExitCode }
    }
    finally {
        try { if (-not $process.HasExited) { $process.Kill($true) } }
        finally { $process.Dispose() }
    }
}

function Get-GitLine($Result) {
    $Result.Text.TrimEnd([char[]]@(13, 10))
}

function Get-GitRecords($Result) {
    if ($Result.Text -and -not $Result.Text.EndsWith([char]0)) { throw 'Git returned an incomplete record.' }
    $Result.Text.Split([char]0, [StringSplitOptions]::RemoveEmptyEntries)
}

function Get-ContinuityRepository([string]$Path) {
    $root = Resolve-CirnetRoot -Path $Path
    if (-not [IO.Directory]::Exists($root)) { throw 'The selected worktree is unavailable.' }
    $gitRoot = Get-GitLine (Invoke-ContinuityGit $root @('rev-parse', '--show-toplevel'))
    if (-not $root.Equals([IO.Path]::GetFullPath($gitRoot), $pathComparison)) {
        throw 'Repository must select the Git worktree root.'
    }
    @{
        root = $root
        gitDirectory = [IO.Path]::GetFullPath((Get-GitLine (Invoke-ContinuityGit $root @('rev-parse', '--absolute-git-dir'))))
        commonDirectory = [IO.Path]::GetFullPath((Get-GitLine (Invoke-ContinuityGit $root @('rev-parse', '--path-format=absolute', '--git-common-dir'))))
        index = [IO.Path]::GetFullPath((Get-GitLine (Invoke-ContinuityGit $root @('rev-parse', '--path-format=absolute', '--git-path', 'index'))))
    }
}

function Get-ContinuityHead($Repository) {
    Get-GitLine (Invoke-ContinuityGit $Repository.root @('rev-parse', '--verify', '--quiet', 'HEAD^{commit}') @(0, 1))
}

function Test-ContinuityWorktree($Repository, $RecordedGit) {
    $Repository.root -ceq $RecordedGit.worktree -and $Repository.gitDirectory -ceq $RecordedGit.gitDirectory -and
        $Repository.commonDirectory -ceq $RecordedGit.repository
}

function Test-ContinuityReferenceConflict($Repository, [string[]]$Paths) {
    if (-not $Paths.Count) { return $false }
    $unmerged = Invoke-ContinuityGit $Repository.root (@('ls-files', '--unmerged', '-z', '--') + $Paths)
    return $unmerged.Text.Length -gt 0
}

function Test-ContinuityInside([string]$Root, [string]$Path) {
    $prefix = [IO.Path]::TrimEndingDirectorySeparator($Root) + [IO.Path]::DirectorySeparatorChar
    $Path.Equals($Root, $pathComparison) -or $Path.StartsWith($prefix, $pathComparison)
}

function Assert-ContinuityDestination($Repository, [string]$Path, [switch]$Directory) {
    if ($Path -match '(?i)(?:^|[\\/])\.git(?:[\\/]|$)' -or
        (Test-ContinuityInside $Repository.gitDirectory $Path) -or
        (Test-ContinuityInside $Repository.commonDirectory $Path)) {
        throw 'CheckpointRoot and export destinations must be outside Git administrative storage.'
    }

    # External storage can be in another checkout only when that checkout ignores it.
    $parent = [IO.Path]::GetDirectoryName($Path)
    while ($parent) {
        if ([IO.File]::Exists((Join-Path $parent '.git')) -or [IO.Directory]::Exists((Join-Path $parent '.git'))) {
            $relative = [IO.Path]::GetRelativePath($parent, $Path).Replace([IO.Path]::DirectorySeparatorChar, '/')
            if ($Directory) { $relative += '/' }
            $tracked = Invoke-ContinuityGit $parent @('ls-files', '--cached', '-z', '--', $relative)
            $ignored = Invoke-ContinuityGit $parent @('check-ignore', '--no-index', '--quiet', '--', "./$relative") @(0, 1)
            if ($tracked.Text -or $ignored.Code -ne 0) {
                throw 'CheckpointRoot and export destinations inside a worktree must be Git-ignored and untracked.'
            }
            return
        }
        $parent = [IO.Path]::GetDirectoryName($parent)
    }
    if (Test-ContinuityInside $Repository.root $Path) { throw 'CheckpointRoot must be below the worktree root.' }
}

function Get-ContinuityAttributes([string]$Path) {
    try { [IO.File]::GetAttributes($Path) }
    catch [IO.FileNotFoundException], [IO.DirectoryNotFoundException] { $null }
}

function Resolve-ContinuityFile([string]$Root, [string]$Relative, [switch]$AllowLeafLink) {
    $parts = $Relative.Split('/')
    $path = $Root
    for ($i = 0; $i -lt $parts.Count; $i++) {
        $part = $parts[$i]
        if ($part -in @('', '.', '..', '.git') -or $part.IndexOfAny([IO.Path]::GetInvalidFileNameChars()) -ge 0 -or
            ($IsWindows -and $part -match '[ .]\z|^(?i:CON|PRN|AUX|NUL|CONIN\$|CONOUT\$|COM[1-9¹²³]|LPT[1-9¹²³])(?:\.|$)')) {
            throw 'An unsupported repository-relative path was encountered.'
        }
        $path = [IO.Path]::Combine($path, $part)
        if ($AllowLeafLink -and $i -eq $parts.Count - 1) { break }
        if ((Get-ContinuityAttributes $path) -band [IO.FileAttributes]::ReparsePoint) {
            throw 'A repository-relative path traverses a link or reparse point.'
        }
    }
    $path
}

function Get-ContinuityFileDigest([string]$Path) {
    $attributes = Get-ContinuityAttributes $Path
    if ($null -eq $attributes) { throw 'A required file is missing or inaccessible.' }
    if ($attributes -band ([IO.FileAttributes]::Directory -bor [IO.FileAttributes]::ReparsePoint)) {
        throw 'A required file must be regular and cannot be a link.'
    }
    Get-CirnetHash -Path $Path
}

function Assert-NoGitFilters([string]$Root, [string[]]$Tracked, [string[]]$Untracked) {
    $keys = @(Get-GitRecords (Invoke-ContinuityGit $Root @('config', '--null', '--name-only', '--get-regexp', '^filter\..*\.(clean|process)$') @(0, 1)))
    if ($keys.Count -eq 0 -or ($Tracked.Count + $Untracked.Count) -eq 0) { return }
    $filters = [Collections.Generic.HashSet[string]]::new([StringComparer]::Ordinal)
    foreach ($key in $keys) {
        [void]$filters.Add($key.Substring(7, $key.LastIndexOf('.') - 7))
    }
    $paths = ($Tracked + $Untracked) -join [char]0
    # Empty fields would affect the path/attribute/value grouping.
    $attributes = @(Get-GitRecords (Invoke-ContinuityGit -Root $Root -GitArgs @('check-attr', '-z', '--stdin', 'filter') -StandardInput ($paths + [char]0)))
    if ($attributes.Count % 3 -ne 0) { throw 'Git returned incomplete attributes.' }
    for ($i = 2; $i -lt $attributes.Count; $i += 3) {
        if ($filters.Contains($attributes[$i])) {
            throw 'A worktree path uses an external Git filter. Git status cannot be collected without running that filter.'
        }
    }
}

function Get-WorktreeDigest([string]$Root, [string]$Relative) {
    $path = Resolve-ContinuityFile $Root $Relative -AllowLeafLink
    $attributes = Get-ContinuityAttributes $path
    if ($null -eq $attributes) { return 'absent' }
    if ($attributes -band [IO.FileAttributes]::ReparsePoint) {
        $target = [IO.FileInfo]::new($path).LinkTarget
        if ($null -eq $target) { throw 'An unsupported reparse point was encountered in the worktree.' }
        return 'link:' + (Get-ContinuityTextHash $target)
    }
    # A deleted indexed file may have been replaced by a directory with new files.
    if ($attributes -band [IO.FileAttributes]::Directory) { return 'directory' }
    'file:' + (Get-ContinuityFileDigest $path)
}

function Read-GitSnapshot($Repository) {
    $root = $Repository.root
    $head = Get-ContinuityHead $Repository
    $branch = Get-GitLine (Invoke-ContinuityGit $root @('symbolic-ref', '--quiet', 'HEAD') @(0, 1))
    $indexHash = if ([IO.File]::Exists($Repository.index)) { Get-ContinuityFileDigest $Repository.index } else { '' }
    $entries = Invoke-ContinuityGit $root @('ls-files', '--stage', '-z', '--')
    $unmerged = [Collections.Generic.SortedSet[string]]::new([StringComparer]::Ordinal)
    $tracked = @(
        foreach ($entry in (Get-GitRecords $entries)) {
            $tab = $entry.IndexOf([char]9)
            if ($tab -lt 0) { throw 'Git returned an invalid index entry.' }
            if ($entry.StartsWith('160000 ')) { throw 'Gitlink state needs a separate submodule-aware check.' }
            $relative = $entry.Substring($tab + 1)
            if ($entry.Substring(0, $tab) -match ' [123]$') { [void]$unmerged.Add($relative) }
            $relative
        }
    )
    $untracked = @(Get-GitRecords (Invoke-ContinuityGit $root @('ls-files', '--others', '--exclude-standard', '-z', '--')))
    Assert-NoGitFilters $root $tracked $untracked
    $status = @(Get-GitRecords (Invoke-ContinuityGit $root @('status', '--porcelain=v1', '-z', '--untracked-files=all', '--no-renames', '--ignore-submodules=all')))
    $paths = [Collections.Generic.SortedSet[string]]::new([StringComparer]::Ordinal)
    foreach ($path in $tracked + $untracked) { [void]$paths.Add($path) }
    $staged = [Collections.Generic.List[string]]::new()
    $unstaged = [Collections.Generic.List[string]]::new()
    foreach ($entry in $status) {
        if ($entry.Length -lt 4 -or $entry[2] -cne ' ') { throw 'Git returned an invalid status entry.' }
        $path = $entry.Substring(3)
        if ($entry.StartsWith('?? ')) {
            if ($path -cnotin $untracked) { throw 'Worktree inventory changed during Git capture.' }
            continue
        }
        [void]$paths.Add($path)
        if ($entry[0] -cne ' ') { $staged.Add($path) }
        if ($entry[1] -cne ' ') { $unstaged.Add($path) }
    }
    $files = [Collections.Specialized.OrderedDictionary]::new([StringComparer]::Ordinal)
    foreach ($path in $paths) { $files.Add($path, (Get-WorktreeDigest $root $path)) }
    $snapshot = [ordered]@{
        worktree = $root
        gitDirectory = $Repository.gitDirectory
        repository = $Repository.commonDirectory
        branch = $branch
        head = $head
        indexSha256 = $indexHash
        entriesSha256 = Get-ContinuityTextHash $entries.Text
        staged = $staged.ToArray()
        unstaged = $unstaged.ToArray()
        untracked = $untracked
        files = $files
    }
    # Keep conflict-free fingerprints compatible with existing checkpoints.
    if ($unmerged.Count) { $snapshot.unmerged = @($unmerged) }
    $snapshot
}

function Get-ContinuityGitFingerprint($Snapshot) {
    Get-ContinuityValueHash $Snapshot
}

function Get-ContinuityGitSnapshot($Repository) {
    $first = Read-GitSnapshot $Repository
    $second = Read-GitSnapshot $Repository
    if ((Get-ContinuityGitFingerprint $first) -cne (Get-ContinuityGitFingerprint $second)) {
        throw 'Git or worktree state changed during capture. Retry after it is stable.'
    }
    $second
}

function Compare-ContinuityGit($Saved, $Current) {
    $fields = @(
        foreach ($field in @('worktree', 'gitDirectory', 'repository', 'branch', 'head', 'indexSha256', 'entriesSha256')) {
            if ($Saved[$field] -cne $Current[$field]) { $field }
        }
    )
    $files = [Collections.Specialized.OrderedDictionary]::new([StringComparer]::Ordinal)
    foreach ($path in $Saved.files.psbase.Keys) {
        if (-not $Current.files.Contains($path)) { $files.Add($path, 'removed') }
        elseif ($Saved.files[$path] -cne $Current.files[$path]) { $files.Add($path, 'modified') }
    }
    foreach ($path in $Current.files.psbase.Keys) {
        if (-not $Saved.files.Contains($path)) { $files.Add($path, 'added') }
    }
    @{
        Matches = (Get-ContinuityGitFingerprint $Saved) -ceq (Get-ContinuityGitFingerprint $Current)
        Changes = @{ Fields = $fields; Files = $files }
        Recorded = $Saved
        Current = $Current
    }
}

Export-ModuleMember -Function Get-ContinuityRepository, Get-ContinuityHead, Get-ContinuityValueHash, Test-ContinuityWorktree,
    Test-ContinuityReferenceConflict,
    Assert-ContinuityDestination, Test-ContinuityInside, Resolve-ContinuityFile, Get-ContinuityFileDigest,
    Get-ContinuityGitFingerprint, Get-ContinuityGitSnapshot, Compare-ContinuityGit

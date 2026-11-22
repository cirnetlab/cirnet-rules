#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'Bundle.psm1')
Import-Module (Join-Path $PSScriptRoot '../continuity/Repository.psm1')
Import-Module (Join-Path $PSScriptRoot '../continuity/Checkpoint.psm1')

$namePattern = '^(?!\.{1,2}\z)[a-z0-9.][a-z0-9._-]{0,95}\z'
$pathComparer = if ($IsWindows) { [StringComparer]::OrdinalIgnoreCase } else { [StringComparer]::Ordinal }

class RepositoryRegistryException : System.Exception {
    RepositoryRegistryException([string]$Message) : base($Message) {}
}

function Resolve-RegistryPath([string]$Path) {
    if (-not [IO.Path]::IsPathFullyQualified($Path) -or $Path -match '^[\\/]{2}|(?:^|[\\/])\.{1,2}(?:[\\/]|$)') {
        throw [RepositoryRegistryException]::new('Registry locations require absolute local paths without traversal.')
    }
    Resolve-CirnetRoot ([IO.Path]::GetFullPath($Path))
}

function Read-RepositoryRegistry([string]$Path) {
    $file = [IO.FileInfo]::new((Resolve-RegistryPath $Path))
    if (-not $file.Exists -or $file.LinkType -eq 'HardLink') { throw [RepositoryRegistryException]::new('Configure an accessible local repository registry.') }
    $config = ConvertFrom-ContinuityJson (Read-ContinuityBytes $file.FullName 128KB) -UniqueProperties
    if ($config -isnot [Collections.IDictionary] -or $config.Count -ne 4 -or
        $config['schemaVersion'] -isnot [long] -and $config['schemaVersion'] -isnot [int] -or
        $config['schemaVersion'] -ne 1 -or $config['discoverGitChildren'] -isnot [bool] -or
        $config['searchRoots'] -isnot [array] -or $config['repositories'] -isnot [array] -or
        $config.searchRoots.Count -gt 32 -or $config.repositories.Count -gt 1024) {
        throw [RepositoryRegistryException]::new('The repository registry schema or limits are invalid.')
    }
    $explicit = [Collections.Generic.Dictionary[string,string]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($entry in $config.repositories) {
        if ($entry -isnot [Collections.IDictionary] -or $entry.Count -ne 2 -or
            $entry['name'] -isnot [string] -or $entry.name -cnotmatch $namePattern -or $entry['path'] -isnot [string]) {
            throw [RepositoryRegistryException]::new('An explicit repository mapping is invalid.')
        }
        if (-not $explicit.TryAdd($entry.name, (Resolve-RegistryPath $entry.path))) {
            throw [RepositoryRegistryException]::new('The repository registry contains duplicate names or aliases.')
        }
    }
    $children = [Collections.Generic.Dictionary[string,object]]::new([StringComparer]::OrdinalIgnoreCase)
    $roots = [Collections.Generic.HashSet[string]]::new($pathComparer)
    $count = 0
    foreach ($value in $config.searchRoots) {
        if ($value -isnot [string]) { throw [RepositoryRegistryException]::new('Invalid repository search root.') }
        $root = Resolve-RegistryPath $value
        if (-not $roots.Add($root)) { continue }
        if (-not [IO.Directory]::Exists($root)) { throw [RepositoryRegistryException]::new('A configured repository search root is missing or moved.') }
        if (-not $config.discoverGitChildren) { continue }
        foreach ($directory in [IO.Directory]::EnumerateDirectories($root)) {
            if (++$count -gt 2048) { throw [RepositoryRegistryException]::new('Repository discovery exceeds its directory limit.') }
            $name = [IO.Path]::GetFileName($directory)
            if ($name -notmatch $namePattern) { continue }
            # Record names only. Do not follow a child or look into its Git metadata
            # until it is selected. A symlink of the requested name fails closed.
            if (-not ([IO.File]::GetAttributes($directory) -band [IO.FileAttributes]::ReparsePoint) -and
                -not [IO.Directory]::Exists((Join-Path $directory '.git'))) { continue }
            if (-not $children.ContainsKey($name)) { $children[$name] = [Collections.Generic.HashSet[string]]::new($pathComparer) }
            [void]$children[$name].Add($directory)
        }
    }
    @{ explicit=$explicit; children=$children }
}

function Resolve-RegistryOwner([string]$Name, $Registry) {
    $paths = @(if ($Registry.explicit.ContainsKey($Name)) { $Registry.explicit[$Name] }
        elseif ($Registry.children.ContainsKey($Name)) { $Registry.children[$Name] })
    if ($paths.Count -gt 1) { throw [RepositoryRegistryException]::new('The requested repository has multiple checkouts. Select one explicit local mapping.') }
    if (-not $paths.Count) { throw [RepositoryRegistryException]::new('The requested owner has no configured checkout. Check its local registry entry.') }
    $root = Resolve-RegistryPath $paths[0]
    if (-not [IO.Directory]::Exists($root)) { throw [RepositoryRegistryException]::new('The mapped checkout is missing or moved.') }
    $git = Resolve-CirnetPath $root '.git'
    if (-not [IO.Directory]::Exists($git)) { throw [RepositoryRegistryException]::new('The selected path is not a standalone Git checkout.') }
    $owner = Get-ContinuityRepository $root
    if (-not $pathComparer.Equals($owner.gitDirectory,$git) -or -not $pathComparer.Equals($owner.commonDirectory,$git)) {
        throw [RepositoryRegistryException]::new('The checkout redirects Git metadata outside its verified root.')
    }
    $owner
}

function Find-PromptOwner {
    param([string]$Request, [string]$Registry, [string]$OwnerName, [switch]$NoLocalDiscovery)
    $routingPath = Join-Path $PSScriptRoot '../../en/references/architecture-routing.md'
    $routing = if ([IO.File]::Exists($routingPath)) { [Text.UTF8Encoding]::new($false,$true).GetString((Read-ContinuityBytes $routingPath 128KB)) }
    $domains = @(foreach ($row in ($routing -split '\r?\n')) {
        if ($row -cmatch '^\| `(?<name>[a-z0-9._-]+)` \|[^|]+\|(?<context>[^|]+)\|(?<owned>[^|]+)\|') {
            @{ name=$Matches.name; context=$Matches.context.Trim(); owned=$Matches.owned }
        }
    })
    $locations = if (-not $NoLocalDiscovery) { Read-RepositoryRegistry $Registry }
    $names = @($domains | ForEach-Object { $_.name })
    if ($locations) { $names += @($locations.explicit.Keys) + @($locations.children.Keys) }
    $known = [Collections.Generic.HashSet[string]]::new([string[]]$names,[StringComparer]::OrdinalIgnoreCase)
    $named = @([regex]::Matches($Request, '[a-zA-Z0-9._-]+') |
        ForEach-Object { $_.Value.TrimEnd('.') -replace '\.git\z', '' } |
        Where-Object { $known.Contains($_) -or ($NoLocalDiscovery -and $_ -match '^cirnet-[a-z0-9-]+\z') } | Sort-Object -Unique)
    if ($OwnerName -and $OwnerName -notmatch $namePattern) { throw [RepositoryRegistryException]::new('Select a valid owner name or alias.') }
    if ($OwnerName -and $named.Count -and $OwnerName -notin $named) {
        throw [RepositoryRegistryException]::new('The selected owner conflicts with the repository named in the request.')
    }
    $choice = if ($OwnerName) { $OwnerName }
        elseif ($named.Count -eq 1) { $named[0] }
        elseif (-not $named.Count) {
            $matchesByDomain = @(foreach ($domain in $domains) {
                $contextWords = $domain.context -split '\s+'
                $anchors = @($domain.context) + @([regex]::Matches($domain.owned,'\b[A-Za-z][A-Za-z0-9]{4,}\b') |
                    ForEach-Object Value | Where-Object { $_ -cmatch '[a-z][A-Z]' })
                if ($contextWords.Count -ge 2 -and $contextWords[0].Length -ge 3 -and $contextWords[1].Length -ge 3) {
                    $anchors += $contextWords[0..1] -join ' '
                }
                $nickname = $domain.name -replace '^cirnet-', ''
                $terms = @([regex]::Matches($domain.owned, '\b[A-Za-z][A-Za-z0-9]{2,}\b') | ForEach-Object Value)
                if (@($anchors | Where-Object { $Request -match ('\b' + [regex]::Escape($_) + '\b') }).Count -or
                    ($Request -match ('\b' + [regex]::Escape($nickname) + '\b') -and
                     @($terms | Where-Object { $_ -ne $nickname -and $Request -match ('\b' + [regex]::Escape($_) + '\b') }).Count)) { $domain.name }
            })
            if ($matchesByDomain.Count -eq 1) { $matchesByDomain[0] }
        }
    if (-not $choice) { throw [RepositoryRegistryException]::new('Confirm the primary owner. The request has no unique match in the registry or architecture routing.') }
    $related = @($named | Where-Object { $_ -ne $choice })
    $checkout = if ($locations) { Resolve-RegistryOwner $choice $locations }
    [pscustomobject]@{
        name=$choice.ToLowerInvariant(); checkout=$checkout; related=$related
        routing=if ($choice -in @($domains | ForEach-Object { $_.name })) { 'en/references/architecture-routing.md' }
    }
}

function Resolve-CirnetPromptOwner {
    param([string]$Request, [string]$Registry, [string]$OwnerName, [switch]$NoLocalDiscovery)
    try { Find-PromptOwner @PSBoundParameters }
    catch [RepositoryRegistryException] { throw }
    catch { throw 'Cannot verify the repository registry or checkout. Check its local paths, links and JSON schema.' }
}

Export-ModuleMember -Function Resolve-CirnetPromptOwner

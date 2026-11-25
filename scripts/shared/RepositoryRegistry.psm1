#requires -Version 7.0
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
Import-Module (Join-Path $PSScriptRoot 'Bundle.psm1')
Import-Module (Join-Path $PSScriptRoot '../continuity/Repository.psm1')
Import-Module (Join-Path $PSScriptRoot '../continuity/Checkpoint.psm1')

$namePattern = '^(?!\.{1,2}\z)[a-z0-9.][a-z0-9._-]{0,95}\z'
$originPattern = '[a-z0-9-]+/[a-z0-9_.-]+'
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
    $explicit = [Collections.Generic.Dictionary[string,object]]::new([StringComparer]::OrdinalIgnoreCase)
    foreach ($entry in $config.repositories) {
        if ($entry -isnot [Collections.IDictionary] -or @($entry.Keys | Where-Object { $_ -cnotin @('name','path','origin') }).Count -or
            $entry['name'] -isnot [string] -or $entry.name -cnotmatch $namePattern -or $entry['path'] -isnot [string] -or
            ($entry.Contains('origin') -and ($entry.origin -isnot [string] -or $entry.origin -cnotmatch "^${originPattern}\z"))) {
            throw [RepositoryRegistryException]::new('An explicit repository mapping is invalid.')
        }
        if (-not $explicit.TryAdd($entry.name, @{path=(Resolve-RegistryPath $entry.path); origin=$entry['origin']})) {
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

function Get-RegistryOrigin($Checkout) {
    $null = Resolve-CirnetPath $Checkout.commonDirectory 'config'
    $urls = @(Get-ContinuityLocalOrigin $Checkout)
    if ($urls.Count -gt 1) { throw [RepositoryRegistryException]::new('The checkout has multiple Git origin URLs. Review its owner identity.') }
    if ($urls.Count -eq 1 -and $urls[0] -match "^(?:https://github\.com/|ssh://git@github\.com/|git@github\.com:)(?<slug>${originPattern})/?\z") {
        $Matches.slug -replace '\.git\z', ''
    }
}

function Resolve-RegistryOwner([string]$Name, $Registry, [string]$CanonicalOrigin) {
    $mapping = if ($Registry.explicit.ContainsKey($Name)) { $Registry.explicit[$Name] }
    $paths = @(if ($mapping) { $mapping.path }
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
    $owner.identityStatus = 'PATH_ONLY'
    $expected = if ($mapping -and $mapping.origin) { $mapping.origin } else { $CanonicalOrigin }
    if ($expected) {
        $origin = Get-RegistryOrigin $owner
        if (($origin -and $origin -ne $expected) -or ($mapping -and $mapping.origin -and -not $origin)) {
            throw [RepositoryRegistryException]::new('The checkout Git origin conflicts with the expected owner identity. Review the registry mapping.')
        }
        if ($origin) { $owner.identityStatus = 'ORIGIN_MATCH' }
    }
    $owner
}

function Find-PromptOwner {
    param([string]$Request, [string]$TargetText, [string]$Registry, [string]$OwnerName, [switch]$NoLocalDiscovery)
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
    $named = @(Get-RegistryNames $Request $known -AllowUnmapped:$NoLocalDiscovery)
    $targeted = @(Get-RegistryNames $TargetText $known -AllowUnmapped:$NoLocalDiscovery)
    if ($OwnerName -and $OwnerName -notmatch $namePattern) { throw [RepositoryRegistryException]::new('Select a valid owner name or alias.') }
    if ($OwnerName -and $targeted.Count -and $OwnerName -notin $targeted) {
        throw [RepositoryRegistryException]::new('The selected owner conflicts with the repository named in the request.')
    }
    $choice = if ($OwnerName) { $OwnerName }
        elseif ($targeted.Count -eq 1) { $targeted[0] }
        elseif (-not $targeted.Count) {
            $matchesByDomain = @(foreach ($domain in $domains) {
                $contextWords = $domain.context -split '\s+'
                $anchors = @($domain.context) + @([regex]::Matches($domain.owned,'\b[A-Za-z][A-Za-z0-9]{4,}\b') |
                    ForEach-Object Value | Where-Object { $_ -cmatch '[a-z][A-Z]' })
                if ($contextWords.Count -ge 2 -and $contextWords[0].Length -ge 3 -and $contextWords[1].Length -ge 3) {
                    $anchors += $contextWords[0..1] -join ' '
                }
                $nickname = $domain.name -replace '^cirnet-', ''
                $terms = @([regex]::Matches($domain.owned, '\b[A-Za-z][A-Za-z0-9]{2,}\b') | ForEach-Object Value)
                if (@($anchors | Where-Object { $TargetText -match ('\b' + [regex]::Escape($_) + '\b') }).Count -or
                    ($TargetText -match ('\b' + [regex]::Escape($nickname) + '\b') -and
                     @($terms | Where-Object { $_ -ne $nickname -and $TargetText -match ('\b' + [regex]::Escape($_) + '\b') }).Count)) { $domain.name }
            })
            if ($matchesByDomain.Count -eq 1) { $matchesByDomain[0] }
        }
    if (-not $choice) { throw [RepositoryRegistryException]::new('Confirm the primary owner. The request has no unique match in the registry or architecture routing.') }
    $related = @($named | Where-Object { $_ -ne $choice })
    $canonical = $choice -in @($domains | ForEach-Object { $_.name })
    $checkout = if ($locations) { Resolve-RegistryOwner $choice $locations $(if ($canonical) { "cirnetlab/$choice" }) }
    [pscustomobject]@{
        name=$choice.ToLowerInvariant(); checkout=$checkout; related=$related
        uniqueTarget=($targeted.Count -eq 1 -and $choice -eq $targeted[0]) -or (-not $OwnerName -and -not $targeted.Count)
        routing=if ($canonical) { 'en/references/architecture-routing.md' }
    }
}

function Get-RegistryNames([string]$Text, $Known, [switch]$AllowUnmapped) {
    [regex]::Matches($Text, '[a-zA-Z0-9._-]+') |
        ForEach-Object { $_.Value.TrimEnd('.') -replace '\.git\z', '' } |
        Where-Object { $Known.Contains($_) -or ($AllowUnmapped -and $_ -match '^cirnet[-_][a-z0-9-]+\z') } | Sort-Object -Unique
}

function Confirm-CirnetPromptCheckout {
    param($Checkout, [string]$TargetText)
    $known = [Collections.Generic.HashSet[string]]::new([StringComparer]::OrdinalIgnoreCase)
    $directoryName = [IO.Path]::GetFileName($Checkout.root)
    $skills = Join-Path $PSScriptRoot '../../en/skills'
    $named = @(Get-RegistryNames $TargetText $known -AllowUnmapped | Where-Object {
        $_ -eq $directoryName -or -not [IO.Directory]::Exists((Join-Path $skills $_))
    })
    if (-not $named.Count) { return }
    if ($named.Count -gt 1) { throw 'Confirm the primary owner for the supplied checkout.' }
    try { $origin = Get-RegistryOrigin $Checkout }
    catch { throw 'Cannot verify the named owner against the supplied checkout metadata.' }
    if (($origin -and $origin -ne "cirnetlab/$($named[0])") -or
        (-not $origin -and $directoryName -ne $named[0])) {
        throw 'The owner named in the request does not match the supplied checkout. Select its registered checkout.'
    }
}

function Resolve-CirnetPromptOwner {
    param([string]$Request, [string]$TargetText, [string]$Registry, [string]$OwnerName, [switch]$NoLocalDiscovery)
    if (-not $PSBoundParameters.ContainsKey('TargetText')) { $PSBoundParameters['TargetText'] = $Request }
    try { Find-PromptOwner @PSBoundParameters }
    catch [RepositoryRegistryException] { throw }
    catch { throw 'Cannot verify the repository registry or checkout. Check its local paths, links and JSON schema.' }
}

Export-ModuleMember -Function Resolve-CirnetPromptOwner, Confirm-CirnetPromptCheckout

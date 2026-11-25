Set-StrictMode -Version Latest

function Get-BenchmarkScenarios([string]$Catalog) {
    $scenarios = @(Get-Content -Raw -LiteralPath $Catalog | ConvertFrom-Json -AsHashtable)
    $names = @{}
    foreach ($scenario in $scenarios) {
        $id = $scenario['id']
        if ($id -cnotmatch '^(?:E[0-9]{2}|[a-z][a-z0-9]*(?:-[a-z0-9]+)+)\z' -or
            ($scenario.Contains('readOnly') -and $scenario['readOnly'] -isnot [bool])) {
            throw 'The scenario catalog has an invalid ID or read-only setting.'
        }
        foreach ($name in @($id) + @($scenario['aliases'] | Where-Object { $_ })) {
            if ($name -notmatch '^[a-z0-9]+(?:-[a-z0-9]+)*\z' -or $names.ContainsKey($name)) {
                throw 'Scenario IDs and aliases must be unique and path-safe.'
            }
            $names[$name] = $scenario
        }
    }
    if (-not $scenarios.Count) { throw 'The scenario catalog is empty.' }
    return $scenarios
}

function Select-BenchmarkScenarios($Scenarios, [string]$CaseId) {
    if (-not $CaseId) { return $Scenarios | Where-Object { $_['default'] } }
    $selected = @{}
    foreach ($name in $CaseId.Split(',')) {
        $requested = $name.Trim()
        $scenario = $Scenarios | Where-Object { $_.id -eq $requested -or $requested -in $_['aliases'] }
        if (-not $scenario -or $selected.ContainsKey($scenario.id)) {
            throw 'Select known, unique scenario IDs or aliases.'
        }
        $selected[$scenario.id] = $true
        $scenario
    }
}

Export-ModuleMember -Function Get-BenchmarkScenarios, Select-BenchmarkScenarios

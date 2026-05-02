param(
    [switch]$Check,
    [string]$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
)

$ErrorActionPreference = "Stop"

$readinessPath = Join-Path $Root "src\readiness.rs"
$kotlinPath = Join-Path $Root "android\app\src\main\java\com\farnam\mhrvf\ReadinessIds.kt"
$matrixPath = Join-Path $Root "docs\readiness-matrix.md"

if (!(Test-Path -LiteralPath $readinessPath)) {
    throw "Missing readiness source: $readinessPath"
}

$source = Get-Content -LiteralPath $readinessPath -Raw

$idMatches = [regex]::Matches(
    $source,
    'pub const ([A-Z0-9_]+): ReadinessId = "([^"]+)";'
)

if ($idMatches.Count -eq 0) {
    throw "No ReadinessId constants found in $readinessPath"
}

$ids = foreach ($match in $idMatches) {
    [pscustomobject]@{
        Name = $match.Groups[1].Value
        Value = $match.Groups[2].Value
    }
}

$repairMatches = [regex]::Matches(
    $source,
    '(?ms)^\s*([A-Z0-9_]+) => Some\(ReadinessRepair \{\s*label: "[^"]+",\s*target: "([^"]+)",\s*\}\),'
)

$repairs = foreach ($match in $repairMatches) {
    [pscustomobject]@{
        Name = $match.Groups[1].Value
        Target = $match.Groups[2].Value
    }
}

$idNames = @{}
foreach ($id in $ids) {
    $idNames[$id.Name] = $true
}

$unknownRepairs = @($repairs | Where-Object { -not $idNames.ContainsKey($_.Name) })
if ($unknownRepairs.Count -gt 0) {
    $names = ($unknownRepairs | ForEach-Object { $_.Name }) -join ", "
    throw "Repair metadata references unknown readiness IDs: $names"
}

$ruleMatches = [regex]::Matches(
    $source,
    '(?ms)ReadinessRule \{\s*id:\s*([A-Z0-9_]+),\s*severity:\s*ReadinessSeverity::([A-Za-z]+),\s*applies_to:\s*"([^"]*)",\s*ok_when:\s*"([^"]*)",\s*not_ok_when:\s*"([^"]*)",\s*repair_target:\s*(Some\("([^"]*)"\)|None),\s*\}'
)

if ($ruleMatches.Count -eq 0) {
    throw "No ReadinessRule catalog entries found in $readinessPath"
}

$rules = foreach ($match in $ruleMatches) {
    [pscustomobject]@{
        Name = $match.Groups[1].Value
        Severity = $match.Groups[2].Value
        AppliesTo = $match.Groups[3].Value
        OkWhen = $match.Groups[4].Value
        NotOkWhen = $match.Groups[5].Value
        RepairTarget = if ($match.Groups[7].Success) { $match.Groups[7].Value } else { $null }
    }
}

$unknownRules = @($rules | Where-Object { -not $idNames.ContainsKey($_.Name) })
if ($unknownRules.Count -gt 0) {
    $names = ($unknownRules | ForEach-Object { $_.Name }) -join ", "
    throw "Readiness rule catalog references unknown readiness IDs: $names"
}

$ruleGroups = @($rules | Group-Object Name | Where-Object { $_.Count -gt 1 })
if ($ruleGroups.Count -gt 0) {
    $names = ($ruleGroups | ForEach-Object { $_.Name }) -join ", "
    throw "Readiness rule catalog has duplicate IDs: $names"
}

$ruleNames = @{}
foreach ($rule in $rules) {
    $ruleNames[$rule.Name] = $true
}
$missingRules = @($ids | Where-Object { -not $ruleNames.ContainsKey($_.Name) })
if ($missingRules.Count -gt 0) {
    $names = ($missingRules | ForEach-Object { $_.Name }) -join ", "
    throw "Readiness rule catalog is missing IDs: $names"
}

$repairByName = @{}
foreach ($repair in $repairs) {
    $repairByName[$repair.Name] = $repair.Target
}
foreach ($rule in $rules) {
    if ($null -ne $rule.RepairTarget) {
        if (-not $repairByName.ContainsKey($rule.Name)) {
            throw "Readiness rule $($rule.Name) has repair_target but repair_for has no entry"
        }
        if ($repairByName[$rule.Name] -ne $rule.RepairTarget) {
            throw "Readiness rule $($rule.Name) repair target '$($rule.RepairTarget)' does not match repair_for target '$($repairByName[$rule.Name])'"
        }
    }
}

$anchorMatches = [regex]::Matches(
    $source,
    '(?ms)ReadinessRepairAnchor \{\s*target:\s*"([^"]+)",\s*desktop:\s*"([^"]+)",\s*android:\s*"([^"]+)",\s*\}'
)

if ($anchorMatches.Count -eq 0) {
    throw "No ReadinessRepairAnchor entries found in $readinessPath"
}

$anchors = foreach ($match in $anchorMatches) {
    [pscustomobject]@{
        Target = $match.Groups[1].Value
        Desktop = $match.Groups[2].Value
        Android = $match.Groups[3].Value
    }
}

$anchorGroups = @($anchors | Group-Object Target | Where-Object { $_.Count -gt 1 })
if ($anchorGroups.Count -gt 0) {
    $targets = ($anchorGroups | ForEach-Object { $_.Name }) -join ", "
    throw "Readiness repair anchor catalog has duplicate targets: $targets"
}

$anchorByTarget = @{}
foreach ($anchor in $anchors) {
    $anchorByTarget[$anchor.Target] = $anchor
}
$missingAnchors = @($repairs | Where-Object { -not $anchorByTarget.ContainsKey($_.Target) })
if ($missingAnchors.Count -gt 0) {
    $targets = ($missingAnchors | ForEach-Object { $_.Target } | Sort-Object -Unique) -join ", "
    throw "Readiness repair anchor catalog is missing repair targets: $targets"
}

function Escape-MarkdownCell {
    param([AllowNull()][string]$Value)
    if ($null -eq $Value) {
        return ""
    }
    return $Value.Replace("`r", " ").Replace("`n", " ").Replace("|", "\|")
}

$lines = New-Object System.Collections.Generic.List[string]
$lines.Add("package com.farnam.mhrvf")
$lines.Add("")
$lines.Add("// Generated by tools/generate-readiness-contract.ps1 from src/readiness.rs.")
$lines.Add("// Do not edit by hand; update Rust readiness IDs/repair targets and regenerate.")
$lines.Add("object ReadinessIds {")
foreach ($id in $ids) {
    $lines.Add("    const val $($id.Name) = ""$($id.Value)""")
}
$lines.Add("    const val ANDROID_CONNECTION_MODE = ""android.connection_mode""")
$lines.Add("}")
$lines.Add("")
$lines.Add("object ReadinessRepairTargets {")
$lines.Add("    fun targetForId(id: String): String? = when (id) {")
foreach ($repair in $repairs) {
    $lines.Add("        ReadinessIds.$($repair.Name) -> ""$($repair.Target)""")
}
$lines.Add("        else -> null")
$lines.Add("    }")
$lines.Add("}")
$lines.Add("")
$lines.Add("data class ReadinessRepairAnchor(val desktop: String, val android: String)")
$lines.Add("")
$lines.Add("object ReadinessRepairAnchors {")
$lines.Add("    fun anchorForTarget(target: String): ReadinessRepairAnchor? = when (target) {")
foreach ($anchor in $anchors) {
    $desktop = $anchor.Desktop.Replace('\', '\\').Replace('"', '\"')
    $android = $anchor.Android.Replace('\', '\\').Replace('"', '\"')
    $lines.Add("        ""$($anchor.Target)"" -> ReadinessRepairAnchor(""$desktop"", ""$android"")")
}
$lines.Add("        else -> null")
$lines.Add("    }")
$lines.Add("}")
$lines.Add("")

$output = ($lines -join "`n")

$ruleByName = @{}
foreach ($rule in $rules) {
    $ruleByName[$rule.Name] = $rule
}

$matrixLines = New-Object System.Collections.Generic.List[string]
$matrixLines.Add("# Readiness Matrix")
$matrixLines.Add("")
$matrixLines.Add('Generated by `tools/generate-readiness-contract.ps1` from `src/readiness.rs`. Do not edit by hand; update the Rust readiness catalog and regenerate.')
$matrixLines.Add("")
$matrixLines.Add("| ID | Severity | Applies to | OK when | Not OK when | Repair target | Desktop anchor | Android anchor |")
$matrixLines.Add("|---|---|---|---|---|---|---|---|")
foreach ($id in $ids) {
    $rule = $ruleByName[$id.Name]
    $anchor = $null
    if ($null -ne $rule.RepairTarget -and $anchorByTarget.ContainsKey($rule.RepairTarget)) {
        $anchor = $anchorByTarget[$rule.RepairTarget]
    }
    $matrixLines.Add("| ``$($id.Value)`` | $(Escape-MarkdownCell $rule.Severity) | $(Escape-MarkdownCell $rule.AppliesTo) | $(Escape-MarkdownCell $rule.OkWhen) | $(Escape-MarkdownCell $rule.NotOkWhen) | ``$(Escape-MarkdownCell $rule.RepairTarget)`` | $(Escape-MarkdownCell $anchor.Desktop) | $(Escape-MarkdownCell $anchor.Android) |")
}
$matrixLines.Add("")

$matrixOutput = ($matrixLines -join "`n")

if ($Check) {
    if (!(Test-Path -LiteralPath $kotlinPath)) {
        throw "Generated Kotlin contract is missing: $kotlinPath"
    }
    if (!(Test-Path -LiteralPath $matrixPath)) {
        throw "Generated readiness matrix is missing: $matrixPath"
    }
    $existing = Get-Content -LiteralPath $kotlinPath -Raw
    $normalizedExisting = $existing -replace "`r`n", "`n"
    $normalizedOutput = $output -replace "`r`n", "`n"
    if ($normalizedExisting -ne $normalizedOutput) {
        throw "Generated Kotlin readiness contract is stale. Run tools/generate-readiness-contract.ps1."
    }
    $existingMatrix = Get-Content -LiteralPath $matrixPath -Raw
    $normalizedExistingMatrix = $existingMatrix -replace "`r`n", "`n"
    $normalizedMatrixOutput = $matrixOutput -replace "`r`n", "`n"
    if ($normalizedExistingMatrix -ne $normalizedMatrixOutput) {
        throw "Generated readiness matrix is stale. Run tools/generate-readiness-contract.ps1."
    }
    Write-Output "current=$kotlinPath matrix=$matrixPath ids=$($ids.Count) repair_targets=$($repairs.Count) rules=$($rules.Count) anchors=$($anchors.Count)"
    return
}

Set-Content -LiteralPath $kotlinPath -Value $output -NoNewline -Encoding UTF8
Set-Content -LiteralPath $matrixPath -Value $matrixOutput -NoNewline -Encoding UTF8

Write-Output "generated=$kotlinPath matrix=$matrixPath ids=$($ids.Count) repair_targets=$($repairs.Count) rules=$($rules.Count) anchors=$($anchors.Count)"

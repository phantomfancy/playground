[CmdletBinding(SupportsShouldProcess)]
param(
    [string]$Destination = (Join-Path $env:LOCALAPPDATA 'Programs\cdk-make-assistant\bin')
)

$ErrorActionPreference = 'Stop'
$source = Join-Path $PSScriptRoot 'cdk-make-assistant.exe'
if (-not (Test-Path -LiteralPath $source -PathType Leaf)) {
    throw "cdk-make-assistant.exe not found beside install.ps1: $source"
}

$destinationPath = [System.IO.Path]::GetFullPath($Destination)
$target = Join-Path $destinationPath 'cdk-make-assistant.exe'
if ($PSCmdlet.ShouldProcess($target, 'Install cdk-make-assistant')) {
    New-Item -ItemType Directory -Path $destinationPath -Force | Out-Null
    Copy-Item -LiteralPath $source -Destination $target -Force
}

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
$entries = @($userPath -split ';' | Where-Object { $_ })
$normalizePath = {
    param([string]$Path)
    try {
        [System.IO.Path]::GetFullPath(
            [Environment]::ExpandEnvironmentVariables($Path)
        ).TrimEnd('\')
    }
    catch {
        $Path.TrimEnd('\')
    }
}
$normalizedDestination = & $normalizePath $destinationPath
$alreadyConfigured = $entries | Where-Object {
    (& $normalizePath $_) -ieq $normalizedDestination
}
if (-not $alreadyConfigured -and
    $PSCmdlet.ShouldProcess('User PATH', "Add $destinationPath")) {
    $updatedPath = (@($entries) + $destinationPath) -join ';'
    [Environment]::SetEnvironmentVariable('Path', $updatedPath, 'User')
    Write-Host 'User PATH updated. Open a new terminal before running the command.'
}

Write-Host "Installed: $target"

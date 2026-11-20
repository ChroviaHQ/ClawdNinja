$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = New-Object System.Text.UTF8Encoding($false)
$name = $env:CLAWDNINJA_EXECUTABLE
$exe = $null
$package = $null
$app = $null
if ($env:CLAWDNINJA_PACKAGE) {
    $package = Get-AppxPackage -Name $env:CLAWDNINJA_PACKAGE | Select-Object -First 1
}
if ($package) {
    $apps = (Get-AppxPackageManifest $package).Package.Applications.Application
    $app = $apps | Where-Object { [IO.Path]::GetFileName($_.Executable) -ieq $name } | Select-Object -First 1
    if ($app) { $exe = Join-Path $package.InstallLocation $app.Executable }
}
$root = $null
if ($env:CLAWDNINJA_ROOT) { $root = Join-Path $env:LOCALAPPDATA $env:CLAWDNINJA_ROOT }
if (!$exe -and $root -and (Test-Path -LiteralPath $root)) {
    $versions = Get-ChildItem -LiteralPath $root -Directory -Filter 'app-*' |
        Sort-Object LastWriteTime -Descending |
        ForEach-Object { Join-Path $_.FullName $name }
    $exe = @($versions) + (Join-Path $root $name) |
        Where-Object { Test-Path -LiteralPath $_ } |
        Select-Object -First 1
}
if (!$exe) { exit 2 }
if ($env:CLAWDNINJA_DETECT -eq '1') { exit 0 }
$running = Get-Process -Name ([IO.Path]::GetFileNameWithoutExtension($name)) -ErrorAction SilentlyContinue | Where-Object {
    $_.Path -eq $exe -or ($root -and $_.Path -and $_.Path.StartsWith("$root\", [StringComparison]::OrdinalIgnoreCase))
}
if ($running) { exit 3 }
$launcher = $exe
if ($app) {
    $alias = $app.Extensions.Extension |
        Where-Object { $_.Category -eq 'windows.appExecutionAlias' } |
        ForEach-Object { $_.AppExecutionAlias.ExecutionAlias.Alias } |
        Select-Object -First 1
    $aliasPath = if ($alias) { Join-Path $env:LOCALAPPDATA "Microsoft\WindowsApps\$alias" }
    if ($aliasPath -and (Test-Path -LiteralPath $aliasPath)) { $launcher = $aliasPath }
}
$argument = $env:CLAWDNINJA_ARGUMENT
if ($argument) { Start-Process -FilePath $launcher -ArgumentList $argument } else { Start-Process -FilePath $launcher }

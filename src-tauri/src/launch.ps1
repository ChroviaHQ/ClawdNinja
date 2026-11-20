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
$roots = @($env:CLAWDNINJA_ROOTS -split ';' |
    Where-Object { $_ } |
    ForEach-Object { [Environment]::ExpandEnvironmentVariables($_) } |
    Where-Object { Test-Path -LiteralPath $_ })
if (!$exe) {
    $exe = $roots |
        ForEach-Object {
            $root = $_
            Get-ChildItem -LiteralPath $root -Directory -Filter 'app-*' |
                Sort-Object LastWriteTime -Descending |
                ForEach-Object { Join-Path $_.FullName $name }
            Join-Path $root $name
        } |
        Where-Object { Test-Path -LiteralPath $_ } |
        Select-Object -First 1
}
if (!$exe) { exit 2 }
if ($env:CLAWDNINJA_DETECT -eq '1') { exit 0 }
$running = Get-Process -Name ([IO.Path]::GetFileNameWithoutExtension($name)) -ErrorAction SilentlyContinue | Where-Object {
    $path = $_.Path
    $path -and ($path -eq $exe -or $roots.Where({ $path.StartsWith("$_\", [StringComparison]::OrdinalIgnoreCase) }, 'First'))
} | Select-Object -First 1
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

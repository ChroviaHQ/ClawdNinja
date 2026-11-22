$ErrorActionPreference = 'Stop'
$ExitMissing = 2
$ExitRunning = 3
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
if (!$exe) { exit $ExitMissing }
if (!$env:CLAWDNINJA_PROXY_ARGUMENT) { exit 0 }
$running = @(Get-Process -Name ([IO.Path]::GetFileNameWithoutExtension($name)) -ErrorAction SilentlyContinue | Where-Object {
    $path = $_.Path
    $path -and ($path -eq $exe -or $roots.Where({ $path.StartsWith("$_\", [StringComparison]::OrdinalIgnoreCase) }, 'First'))
})
$arguments = $env:CLAWDNINJA_ARGUMENTS
if ($running) {
    $newWindowArgument = $env:CLAWDNINJA_NEW_WINDOW_ARGUMENT
    if (!$newWindowArgument) { exit $ExitRunning }
    $filter = ($running.Id | ForEach-Object { "ProcessId=$_" }) -join ' OR '
    $proxied = Get-CimInstance Win32_Process -Filter $filter -Property CommandLine | Where-Object {
        (($_.CommandLine -replace '"', '') -split '\s+') -ccontains $env:CLAWDNINJA_PROXY_ARGUMENT
    } | Select-Object -First 1
    if (!$proxied) { exit $ExitRunning }
    $arguments = "$arguments $newWindowArgument".Trim()
}
$launcher = $exe
if ($app) {
    $alias = $app.Extensions.Extension |
        Where-Object { $_.Category -eq 'windows.appExecutionAlias' } |
        ForEach-Object { $_.AppExecutionAlias.ExecutionAlias.Alias } |
        Select-Object -First 1
    $aliasPath = if ($alias) { Join-Path $env:LOCALAPPDATA "Microsoft\WindowsApps\$alias" }
    if ($aliasPath -and (Test-Path -LiteralPath $aliasPath)) { $launcher = $aliasPath }
}
if ($arguments) { Start-Process -FilePath $launcher -ArgumentList $arguments } else { Start-Process -FilePath $launcher }

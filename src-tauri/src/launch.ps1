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
$argument = $env:CLAWDNINJA_ARGUMENT
try {
    if ($argument) { Start-Process -FilePath $exe -ArgumentList $argument } else { Start-Process -FilePath $exe }
} catch {
    if (!$package -or !$app) { throw }
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;

[ComImport, Guid("2e941141-7f97-4756-ba1d-9decde894a3d"),
 InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IApplicationActivationManager {
    void ActivateApplication([MarshalAs(UnmanagedType.LPWStr)] string appId,
        [MarshalAs(UnmanagedType.LPWStr)] string arguments, uint options, out uint pid);
}

public static class ClawdNinjaActivation {
    public static void Launch(string appId, string arguments) {
        var type = Type.GetTypeFromCLSID(new Guid("45BA127D-10A8-46EA-8AB7-56EA9078943C"));
        var manager = (IApplicationActivationManager)Activator.CreateInstance(type);
        uint pid;
        manager.ActivateApplication(appId, arguments, 0, out pid);
    }
}
'@
    [ClawdNinjaActivation]::Launch("$($package.PackageFamilyName)!$($app.Id)", "$argument")
}

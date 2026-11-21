<#
Capture USB traffic between Cricut Design Space and a Cricut machine on Windows (USBPcap).

Run elevated (USBPcap needs admin). The capture runs until a file named STOP appears in
$OutDir or any of the -StopDir folders, so another agent or person can end it without admin.

Find the hub and device address first (no admin needed):
  & "C:\Program Files\USBPcap\USBPcapCMD.exe" --extcap-interfaces
  & "C:\Program Files\USBPcap\USBPcapCMD.exe" --extcap-interface \\.\USBPcap1 --extcap-config
Look for "[N] USB Serial Device" (the Maker is VID 20D3 PID 0015); N is the device address.

Marker files written to $OutDir:
  CAPTURE_RUNNING  path of the .pcap and start time (removed when the capture ends)
  CAPTURE_DONE     path, stop time and size
  capture-log.txt  start/stop/error log
#>
param(
    [string]$OutDir = (Join-Path $env:USERPROFILE "Downloads\cricut-capture"),
    [string]$Hub = "\\.\USBPcap1",
    [string]$Devices = "5",
    [string[]]$StopDir = @()
)
$ErrorActionPreference = "Stop"
$exe = "C:\Program Files\USBPcap\USBPcapCMD.exe"
New-Item -ItemType Directory -Force $OutDir | Out-Null
$log = Join-Path $OutDir "capture-log.txt"
function Write-Log($m) { "$(Get-Date -Format s) $m" | Add-Content -Path $log -Encoding utf8 }

try {
    $stopFiles = @($OutDir) + $StopDir | ForEach-Object { Join-Path $_ "STOP" }
    Remove-Item ($stopFiles + (Join-Path $OutDir "CAPTURE_DONE")) -ErrorAction SilentlyContinue
    $out = Join-Path $OutDir ("cricut-" + (Get-Date -Format "yyyyMMdd-HHmmss") + ".pcap")
    $cmdArgs = @("-d", $Hub, "--devices", $Devices, "--inject-descriptors", "-b", "16777216", "-o", "`"$out`"")
    $p = Start-Process -FilePath $exe -ArgumentList $cmdArgs -PassThru -WindowStyle Hidden
    Start-Sleep -Seconds 2
    if ($p.HasExited) { Write-Log "USBPcapCMD exited early, code $($p.ExitCode)"; exit 1 }
    "$out`nstarted $(Get-Date -Format s)" | Set-Content -Path (Join-Path $OutDir "CAPTURE_RUNNING") -Encoding utf8
    Write-Log "capture started pid=$($p.Id) -> $out"

    while (-not ($stopFiles | Where-Object { Test-Path $_ }) -and -not $p.HasExited) { Start-Sleep -Milliseconds 500 }

    if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force; Write-Log "STOP seen, capture stopped" }
    else { Write-Log "USBPcapCMD exited on its own, code $($p.ExitCode)" }
    Remove-Item (Join-Path $OutDir "CAPTURE_RUNNING") -ErrorAction SilentlyContinue
    "$out`nstopped $(Get-Date -Format s)`nbytes $((Get-Item $out).Length)" | Set-Content -Path (Join-Path $OutDir "CAPTURE_DONE") -Encoding utf8
} catch {
    Write-Log "ERROR: $_"
}

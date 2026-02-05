$ProgressPreference = 'SilentlyContinue'
$ErrorActionPreference = "Stop"
$tmpdir = $Env:TEMP
$DYSCO_VERSION = $Env:DYSCO_VERSION
if ($DYSCO_VERSION -and $DYSCO_VERSION -notlike 'v*') {
    # prefix version with v
    $DYSCO_VERSION = "v$DYSCO_VERSION"
}
# Fetch binaries from `[..]/releases/latest/download/[..]` if _no_ version is
# given, otherwise from `[..]/releases/download/VERSION/[..]`.
$base_url = if (-not $DYSCO_VERSION) {
    "https://github.com/incredimo/dysco/releases/latest/download/dysco-"
} else {
    "https://github.com/incredimo/dysco/releases/download/$DYSCO_VERSION/dysco-"
}

$proc_arch = [Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITECTURE", [EnvironmentVariableTarget]::Machine)
if ($proc_arch -eq "AMD64") {
    $arch = "x86_64"
} elseif ($proc_arch -eq "ARM64") {
    $arch = "aarch64"
} else {
    throw "Unsupported Architecture: $proc_arch"
}

$url = "$base_url$arch-pc-windows-msvc.zip"
$sw = [Diagnostics.Stopwatch]::StartNew()
# create temp with zip extension (or Expand will complain)
$zip = New-TemporaryFile | Rename-Item -NewName { $_ -replace 'tmp$', 'zip' } -PassThru
try {
    Write-Verbose -Verbose -Message "Downloading from $url"
    Invoke-WebRequest -Uri $url -OutFile $zip -UseBasicParsing
} catch {
    throw "Failed to download: $_"
}
$zip | Expand-Archive -DestinationPath $tmpdir -Force
$sw.Stop()
Write-Verbose -Verbose -Message "Download: $($sw.Elapsed.Seconds) seconds"


$sw = [Diagnostics.Stopwatch]::StartNew()
$cargo_home = if ($Env:CARGO_HOME) { $Env:CARGO_HOME } else { "$HOME\.cargo" }
$cargo_bin = Join-Path $cargo_home "bin"
if (-not (Test-Path $cargo_bin)) {
    New-Item -ItemType Directory -Force -Path $cargo_bin | Out-Null
}

$exe_path = Join-Path $cargo_bin "dysco.exe"
# The zip contains the executable directly
Move-Item -Path "$tmpdir\dysco.exe" -Destination $exe_path -Force

$zip | Remove-Item
$sw.Stop()
Write-Verbose -Verbose -Message "Installation: $($sw.Elapsed.Seconds) seconds"

$sw = [Diagnostics.Stopwatch]::StartNew()
if ($Env:Path.ToLower() -split ";" -notcontains $cargo_bin.ToLower()) {
    # Update current session
    $Env:Path += ";$cargo_bin"

    # Update User persistence
    $userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
    if ($userPath.ToLower() -split ";" -notcontains $cargo_bin.ToLower()) {
         [Environment]::SetEnvironmentVariable("Path", "$userPath;$cargo_bin", [EnvironmentVariableTarget]::User)
         Write-Verbose -Verbose -Message "Added $cargo_bin to your User Path."
    }
}
$sw.Stop()
Write-Verbose -Verbose -Message "Path addition: $($sw.Elapsed.Seconds) seconds"
Write-Host "dysco installed successfully to $exe_path"

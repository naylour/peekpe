$ErrorActionPreference = "Stop"

$repo = "naylour/peekpe"

$binary = "peekpe-windows.exe"

$url = "https://github.com/$repo/releases/latest/download/$binary"

$installDir = "$HOME\AppData\Local\Programs\PeekPe"

New-Item -ItemType Directory -Force -Path $installDir | Out-Null

$target = "$installDir\peekpe.exe"

Write-Host "Downloading $binary..."

Invoke-WebRequest $url -OutFile $target

Write-Host ""
Write-Host "PeekPe installed!"
Write-Host "Binary: $target"

$currentPath = [Environment]::GetEnvironmentVariable(
    "Path",
    [EnvironmentVariableTarget]::User
)

if ($currentPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable(
        "Path",
        "$currentPath;$installDir",
        [EnvironmentVariableTarget]::User
    )

    Write-Host ""
    Write-Host "PATH updated."
    Write-Host "Restart terminal."
}

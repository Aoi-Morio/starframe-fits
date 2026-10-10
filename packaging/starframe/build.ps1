param([string]$Iscc, [switch]$SkipBuild)
$ErrorActionPreference = 'Stop'
$previousEncodedFlags = $env:CARGO_ENCODED_RUSTFLAGS
$repo = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
Push-Location $repo
try {
    $version = ((Select-String -Path Cargo.toml -Pattern '^version = "([^"]+)"' | Select-Object -First 1).Matches.Groups[1].Value)
    if (-not $version) { throw 'Workspace version is missing' }
    if (-not $env:PHOTOCRAFT_BUILD_SHA) {
        $env:PHOTOCRAFT_BUILD_SHA = git rev-parse --short HEAD
        if ($LASTEXITCODE -ne 0) { throw 'Commit the source before making a release build' }
    }
    if (-not $env:PHOTOCRAFT_BUILD_DATE) { $env:PHOTOCRAFT_BUILD_DATE = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd') }
    $env:CARGO_PROFILE_RELEASE_LTO = 'false'
    $env:CARGO_PROFILE_RELEASE_CODEGEN_UNITS = '16'
    $env:CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS = '-C target-feature=+crt-static'
    $taskBuildHome = [Environment]::GetFolderPath('UserProfile')
    if (-not $taskBuildHome) { throw 'Cannot determine build home for path sanitization' }
    $taskBuildHomeForward = $taskBuildHome.Replace('\','/')
    $env:CARGO_ENCODED_RUSTFLAGS = [string]::Join([char]0x1f, @('-C', 'target-feature=+crt-static', ('--remap-path-prefix='+$taskBuildHome+'=/build'), ('--remap-path-prefix='+$taskBuildHomeForward+'=/build')))
    $env:PHOTOCRAFT_REQUIRE_WINRES = '1'
    if (-not $SkipBuild) {
        cargo build --locked --release --target x86_64-pc-windows-msvc -p photocraft -p photocraft-cli
        if ($LASTEXITCODE -ne 0) { throw 'Release build failed' }
    }
    $target = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $repo 'target' }
    $bin = Join-Path $target 'x86_64-pc-windows-msvc/release'
    $dist = Join-Path $repo 'dist'
    $stage = Join-Path $dist 'stage'
    New-Item -ItemType Directory -Force $stage | Out-Null
    if (Test-Path (Join-Path $stage 'StarframeData')) { throw 'Refusing to package runtime settings' }
    Copy-Item (Join-Path $bin 'photocraft.exe') (Join-Path $stage 'StarframeFITS.exe')
    Copy-Item (Join-Path $bin 'photocraft-cli.exe') $stage
    foreach ($executable in @('StarframeFITS.exe','photocraft-cli.exe')) {
        $bytes = [IO.File]::ReadAllBytes((Join-Path $stage $executable))
        foreach ($encoding in @([Text.Encoding]::UTF8, [Text.Encoding]::Unicode)) {
            $text = $encoding.GetString($bytes)
            if ($text.IndexOf($taskBuildHome, [StringComparison]::OrdinalIgnoreCase) -ge 0 -or $text.IndexOf($taskBuildHomeForward, [StringComparison]::OrdinalIgnoreCase) -ge 0) {
                throw 'Personal build path remains in executable; refusing to package it'
            }
        }
    }
    foreach ($file in @('LICENSE-MIT','LICENSE-APACHE','NOTICE','ATTRIBUTION.md','README.md')) { Copy-Item (Join-Path $repo $file) $stage }
    foreach ($file in @('assets/fonts/OFL-Inter.txt','assets/fonts/OFL-JetBrainsMono.txt','assets/icons/LICENSE-lucide.txt','assets/dict/LICENSE-SCOWL.txt','assets/app-icon/LICENSE.txt','docs/brand/LICENSE-brand.txt')) {
        $destination = Join-Path $stage $file
        New-Item -ItemType Directory -Force (Split-Path $destination -Parent) | Out-Null
        Copy-Item (Join-Path $repo $file) $destination
    }
    Copy-Item (Join-Path $repo 'examples/fits/demo-stars.fits') $stage
    Set-Content (Join-Path $stage 'portable.txt') ''
    $zip = Join-Path $dist "starframe-fits-$version-windows-x64-portable.zip"
    Compress-Archive -LiteralPath (Get-ChildItem $stage -Force | ForEach-Object FullName) -DestinationPath $zip -Force
    if (-not $Iscc) { $Iscc = (Get-Command ISCC.exe -ErrorAction Stop).Source }
    & $Iscc "/DAppVersion=$version" "/DStageDir=$stage" "/DOutputDir=$dist" (Join-Path $PSScriptRoot 'installer.iss')
    if ($LASTEXITCODE -ne 0) { throw 'Installer compilation failed' }
    $artifacts = @((Get-Item $zip), (Get-Item (Join-Path $dist "starframe-fits-$version-windows-x64-setup.exe")))
    $artifacts | ForEach-Object { $hash = (Get-FileHash $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant(); "$hash  $($_.Name)" } | Set-Content (Join-Path $dist 'SHA256SUMS.txt') -Encoding ascii
    Write-Output 'Installer, portable archive and checksums created.'
} finally { $env:CARGO_ENCODED_RUSTFLAGS = $previousEncodedFlags; Pop-Location }

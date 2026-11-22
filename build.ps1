$ErrorActionPreference = "Stop"

# Add Cargo to the session PATH if needed
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (Test-Path $cargoBin) {
    $env:PATH = "$cargoBin;$env:PATH"
}

$root = $PSScriptRoot
Push-Location $root
try {
    Write-Host "[1/3] Building Rust engine to WebAssembly (wasm32-unknown-unknown)..." -ForegroundColor Cyan
    & cargo build --target wasm32-unknown-unknown --release
    if ($LASTEXITCODE -ne 0) { throw "WASM build failed." }

    $wasmSrc = Join-Path $root "target\wasm32-unknown-unknown\release\bouquet_wasm.wasm"
    $wasmDst = Join-Path $root "bouquet.wasm"
    Copy-Item -Path $wasmSrc -Destination $wasmDst -Force
    $sizeKb = [Math]::Round((Get-Item $wasmDst).Length / 1024, 1)
    Write-Host "[2/3] Copied binary to root: bouquet.wasm ($sizeKb KB)" -ForegroundColor Green

    # Generate embedded base64 fallback so opening index.html directly via file:// works without a server
    $bytes = [System.IO.File]::ReadAllBytes($wasmDst)
    $b64   = [System.Convert]::ToBase64String($bytes)
    $fallbackPath = Join-Path $root "wasm_fallback.js"
    $jsContent = "// Auto-generated from bouquet.wasm (Rust/WebAssembly) for file:// compatibility`nwindow.__BOUQUET_WASM_B64__ = `"$b64`";`n"
    [System.IO.File]::WriteAllText($fallbackPath, $jsContent, [System.Text.UTF8Encoding]::new($false))
    Write-Host "[3/3] Generated file:// fallback: wasm_fallback.js" -ForegroundColor Green
}
finally {
    Pop-Location
}

# Fetch brand logos as PNG from public CDNs. Run from repo root:
#   pwsh scripts/fetch-brand-icons.ps1
$ErrorActionPreference = "Stop"

$dest = Join-Path $PSScriptRoot "..\src\assets\brands"
New-Item -ItemType Directory -Force -Path $dest | Out-Null
$dest = Resolve-Path $dest

$icons = @(
  @{
    name = "microsoft.png"
    url = "https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/png/microsoft.png"
  },
  @{
    name = "google.png"
    url = "https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/png/google.png"
  },
  @{
    name = "bing.png"
    url = "https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/png/bing.png"
  }
)

foreach ($icon in $icons) {
  $out = Join-Path $dest $icon.name
  Write-Host "GET $($icon.url)"
  Invoke-WebRequest -Uri $icon.url -OutFile $out -UseBasicParsing -TimeoutSec 30
  $bytes = [System.IO.File]::ReadAllBytes($out)
  $isPng = ($bytes.Length -ge 8) -and ($bytes[0] -eq 0x89) -and ($bytes[1] -eq 0x50)
  if (-not $isPng) {
    throw "Expected PNG for $($icon.name)"
  }
  Write-Host "  -> $($icon.name) ($($bytes.Length) bytes)"
}

# Cloudflare: Simple Icons SVG (brand orange) → PNG.
# LobeHub "dark" set is white-on-transparent and invisible on light UI.
$cfSvgUrl = "https://cdn.jsdelivr.net/npm/simple-icons@latest/icons/cloudflare.svg"
$cfOut = Join-Path $dest "cloudflare.png"
$work = Join-Path $env:TEMP "look-translate-brand-cf"
New-Item -ItemType Directory -Force -Path $work | Out-Null
$cfSvgPath = Join-Path $work "cloudflare.svg"

Write-Host "GET $cfSvgUrl"
$svg = [string](Invoke-WebRequest -Uri $cfSvgUrl -UseBasicParsing).Content
if ($svg -match 'fill="') {
  $svg = [regex]::Replace($svg, 'fill="[^"]*"', 'fill="#F38020"', 1)
} else {
  $svg = [regex]::Replace($svg, '<path\b', '<path fill="#F38020"', 1)
}
# Ensure raster size for sharp
if ($svg -notmatch 'width=') {
  $svg = $svg -replace '<svg\b', '<svg width="512" height="512"'
}
[System.IO.File]::WriteAllText($cfSvgPath, $svg)

$renderJs = Join-Path $work "render.mjs"
@'
import sharp from "sharp";
import { readFileSync } from "node:fs";

const svgPath = process.argv[2];
const outPath = process.argv[3];
const svg = readFileSync(svgPath);
await sharp(svg).png().toFile(outPath);
console.log("rendered", outPath);
'@ | Set-Content -Path $renderJs -Encoding utf8

Push-Location $work
try {
  if (-not (Test-Path (Join-Path $work "node_modules\sharp"))) {
    Write-Host "Installing sharp (one-time in temp)..."
    npm init -y | Out-Null
    npm install --silent sharp
  }
  node $renderJs $cfSvgPath $cfOut
} finally {
  Pop-Location
}

$bytes = [System.IO.File]::ReadAllBytes($cfOut)
$isPng = ($bytes.Length -ge 8) -and ($bytes[0] -eq 0x89) -and ($bytes[1] -eq 0x50)
if (-not $isPng) {
  throw "Expected PNG for cloudflare.png"
}
Write-Host "  -> cloudflare.png ($($bytes.Length) bytes, orange)"

Get-ChildItem $dest -Filter *.svg -ErrorAction SilentlyContinue | Remove-Item -Force
Write-Host "Done."

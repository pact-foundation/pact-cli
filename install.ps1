$pactDir = $pwd.Path

# Install CLI Tools

Write-Host "--> Downloading Latest Pact broker Client binary)"

$headers = @{
    'Content-Type' = 'application/json'
    Accept = 'application/json'
}
$latestRelease = Invoke-WebRequest -Uri https://github.com/pact-foundation/pact-cli/releases/latest -Method Get -UseBasicParsing -Headers $headers
$json = $latestRelease.Content | ConvertFrom-Json
$tag = $json.tag_name
$architecture = [System.Runtime.InteropServices.RuntimeInformation,mscorlib]::OSArchitecture.ToString().ToLower()
if ($architecture -eq "x64") {
    $architecture = "x86_64"
} elseif ($architecture -eq "arm64") {
    $architecture = "aarch64"
} else {
    Write-Host "Unsupported architecture: $architecture"
    exit 1
}
$url = "https://github.com/pact-foundation/pact-cli/releases/download/$tag/pact-$architecture-windows-msvc.exe"


Write-Host "Downloading $url to $pactDir"
$exe = Join-Path $pactDir "pact.exe"
if (Test-Path "$exe") {
  Remove-Item $exe
}

$downloader = new-object System.Net.WebClient
$downloader.DownloadFile($url, $exe)
Write-Host "--> Downloaded pact to $exe"
# Write-Host "--> Setting executable permissions for pact"
# chmod +x $exe
Write-Host "--> Adding pact to path"
$pactBinariesPath = "$pactDir"
$env:PATH += ";$pactBinariesPath"
Write-Host $env:PATH
pact.exe --help

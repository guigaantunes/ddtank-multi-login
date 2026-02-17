# Script para gerar release do DDTank-RS
# Uso: .\build-release.ps1

Write-Host "=== DDTank-RS Release Builder ===" -ForegroundColor Cyan
Write-Host ""

# Verificar/baixar Sciter SDK
$sciterSdkPath = "C:\sciter-js-sdk-main"
Write-Host "Verificando Sciter SDK em $sciterSdkPath..." -ForegroundColor Yellow
if (-not (Test-Path "$sciterSdkPath\bin\windows\packfolder.exe")) {
    Write-Host "  Sciter SDK nao encontrado. Baixando..." -ForegroundColor Yellow
    $zipPath = "$env:TEMP\sciter-js-sdk.zip"
    Invoke-WebRequest -Uri "https://github.com/c-smile/sciter-js-sdk/archive/refs/heads/main.zip" -OutFile $zipPath
    Expand-Archive -Path $zipPath -DestinationPath "C:\" -Force
    Remove-Item $zipPath -Force
    Write-Host "  Sciter SDK instalado em $sciterSdkPath" -ForegroundColor Green
} else {
    Write-Host "  Sciter SDK ja instalado" -ForegroundColor Green
}
Write-Host ""

# Encerrar processos em execucao
Write-Host "Verificando processos em execucao..." -ForegroundColor Yellow
$processNames = @("ddtank-rs", "ddtank-lua", "cowv2", "reguinha")
$processesKilled = 0

foreach ($processName in $processNames) {
    $processes = Get-Process -Name $processName -ErrorAction SilentlyContinue
    if ($processes) {
        Write-Host "  Encerrando processo: $processName" -ForegroundColor Yellow
        $processes | Stop-Process -Force
        $processesKilled += $processes.Count
    }
}

if ($processesKilled -gt 0) {
    Write-Host "  $processesKilled processo(s) encerrado(s)" -ForegroundColor Green
    Start-Sleep -Seconds 1
} else {
    Write-Host "  Nenhum processo em execucao" -ForegroundColor Green
}

Write-Host ""

# Ler versao do Cargo.toml
$cargoToml = Get-Content ".\Cargo.toml" -Raw
if ($cargoToml -match 'version\s*=\s*"([^"]+)"') {
    $version = $matches[1]
    Write-Host "Versao detectada: $version" -ForegroundColor Green
} else {
    Write-Host "Erro: Nao foi possivel ler a versao do Cargo.toml" -ForegroundColor Red
    exit 1
}

# Compilar em modo release
Write-Host ""
Write-Host "Compilando projeto em modo release..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Write-Host "Erro na compilacao!" -ForegroundColor Red
    exit 1
}

# Criar nome da pasta e arquivo zip
$releaseName = "release-$version"
$releaseDir = ".\$releaseName"
$zipFile = "$releaseName.zip"

# Remover pasta e zip antigos se existirem
if (Test-Path $releaseDir) {
    Write-Host "Removendo pasta de release antiga..." -ForegroundColor Yellow
    Remove-Item $releaseDir -Recurse -Force
}

if (Test-Path $zipFile) {
    Write-Host "Removendo arquivo zip antigo..." -ForegroundColor Yellow
    Remove-Item $zipFile -Force
}

# Criar pasta de release
Write-Host ""
Write-Host "Criando pasta de release: $releaseName" -ForegroundColor Yellow
New-Item -ItemType Directory -Path $releaseDir | Out-Null

# Copiar arquivos necessários
Write-Host "Copiando arquivos..." -ForegroundColor Yellow

$filesToCopy = @(
    @{Source = ".\target\release\ddtank-rs.exe"; Dest = "$releaseDir\ddtank-rs.exe"; Required = $true},
    @{Source = ".\target\release\sciter.dll"; Dest = "$releaseDir\sciter.dll"; Required = $true},
    @{Source = ".\target\release\ui.rc"; Dest = "$releaseDir\ui.rc"; Required = $false},
    @{Source = ".\target\release\reguinha.exe"; Dest = "$releaseDir\reguinha.exe"; Required = $false},
    @{Source = ".\target\release\userdata.redb"; Dest = "$releaseDir\userdata.redb"; Required = $false}
)

foreach ($file in $filesToCopy) {
    if (Test-Path $file.Source) {
        Copy-Item $file.Source $file.Dest
        Write-Host "  [OK] $($file.Source)" -ForegroundColor Green
    } elseif ($file.Required) {
        Write-Host "  [ERRO] $($file.Source) (OBRIGATORIO - NAO ENCONTRADO)" -ForegroundColor Red
        exit 1
    } else {
        Write-Host "  [AVISO] $($file.Source) (opcional - nao encontrado)" -ForegroundColor DarkYellow
    }
}

# Copiar pasta scripts se existir
if (Test-Path ".\target\release\scripts") {
    Write-Host "Copiando pasta scripts..." -ForegroundColor Yellow
    Copy-Item ".\target\release\scripts" -Destination "$releaseDir\scripts" -Recurse
    Write-Host "  [OK] scripts\" -ForegroundColor Green
}

# Criar arquivo ZIP
Write-Host ""
Write-Host "Criando arquivo ZIP: $zipFile" -ForegroundColor Yellow
Compress-Archive -Path $releaseDir -DestinationPath $zipFile -CompressionLevel Optimal

# Exibir resumo
Write-Host ""
Write-Host "=== Release Gerada com Sucesso! ===" -ForegroundColor Green
Write-Host ""
Write-Host "Pasta de release: $releaseDir" -ForegroundColor Cyan
Write-Host "Arquivo ZIP: $zipFile" -ForegroundColor Cyan
Write-Host ""

# Listar conteudo
Write-Host "Conteudo da release:" -ForegroundColor Yellow
Get-ChildItem $releaseDir -Recurse | ForEach-Object {
    $relativePath = $_.FullName.Replace((Get-Item $releaseDir).FullName, "")
    if ($_.PSIsContainer) {
        Write-Host "  [DIR] $relativePath" -ForegroundColor Blue
    } else {
        $size = "{0:N2} KB" -f ($_.Length / 1KB)
        Write-Host "  [FILE] $relativePath ($size)" -ForegroundColor White
    }
}

Write-Host ""
Write-Host 'Pronto para distribuicao!' -ForegroundColor Green

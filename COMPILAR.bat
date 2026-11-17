@echo off
chcp 65001 >nul
cd /d "%~dp0"
title Menu SAO - compilar

where cargo >nul 2>nul
if errorlevel 1 (
  echo Rust nao encontrado. Instalando o Rust e as Ferramentas de Build do Visual Studio.
  echo Isso baixa alguns GB na primeira vez e leva varios minutos. So precisa fazer uma vez.
  echo.
  winget install -e --id Rustlang.Rustup --accept-package-agreements --accept-source-agreements
  winget install -e --id Microsoft.VisualStudio.2022.BuildTools --accept-package-agreements --accept-source-agreements --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
  echo.
  echo Se deu certo: FECHE esta janela e de dois cliques em COMPILAR.bat de novo.
  echo Se deu erro: instale o Rust pelo site rustup.rs e as "Ferramentas de Build do Visual Studio" ^(carga C++^).
  pause
  exit /b 1
)

rem Os arquivos de compilacao ficam FORA da pasta do projeto (ocupam mais de 1 GB).
rem Assim a pasta do projeto continua pequena, fora do OneDrive e longe do antivirus.
rem Para liberar o espaco depois: LIMPAR-CACHE.bat
set "CARGO_TARGET_DIR=%LOCALAPPDATA%\menu-sao-build"

echo Compilando ^(a primeira vez demora alguns minutos; as proximas sao bem mais rapidas^)...
cargo build --release -p menu-sao
if errorlevel 1 (
  echo.
  echo Erro na compilacao. Copie o texto acima e mande para o Claude.
  pause
  exit /b 1
)

copy /y "%CARGO_TARGET_DIR%\release\menu-sao.exe" "Menu SAO.exe" >nul
if errorlevel 1 (
  echo.
  echo Compilou, mas nao consegui copiar para esta pasta. O programa esta aqui:
  echo   %CARGO_TARGET_DIR%\release\menu-sao.exe
  explorer "%CARGO_TARGET_DIR%\release"
  pause
  exit /b 1
)

for %%A in ("Menu SAO.exe") do echo.& echo Pronto! "Menu SAO.exe" tem %%~zA bytes.
powershell -NoProfile -Command "try { '{0:N0} MB guardados em %CARGO_TARGET_DIR% (acelera as proximas compilacoes; LIMPAR-CACHE.bat apaga)' -f ((Get-ChildItem -Recurse -Force '%CARGO_TARGET_DIR%' | Measure-Object Length -Sum).Sum/1MB) } catch {}" 2>nul
echo.
echo De dois cliques em "Menu SAO.exe". O icone aparece na bandeja, perto do relogio.
echo Se ja existia uma copia rodando, feche-a antes ^(FECHAR.bat^).
pause

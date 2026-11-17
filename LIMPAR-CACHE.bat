@echo off
chcp 65001 >nul
title Menu SAO - limpar espaco
echo Isto apaga os arquivos temporarios de compilacao do Menu SAO (mais de 1 GB).
echo O "Menu SAO.exe" e os seus dados NAO sao apagados.
echo A proxima compilacao volta a demorar alguns minutos.
echo.
pause
rmdir /s /q "%LOCALAPPDATA%\menu-sao-build" 2>nul
rem pasta antiga, de quando a compilacao ainda ficava dentro do projeto
if exist "%~dp0target" rmdir /s /q "%~dp0target"
echo Pronto, espaco liberado.
pause

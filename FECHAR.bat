@echo off
taskkill /f /im "Menu SAO.exe" >nul 2>nul
taskkill /f /im menu-sao.exe >nul 2>nul
echo Menu SAO fechado.
timeout /t 2 >nul

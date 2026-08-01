@echo off
REM Mo cong cu don dep Zalo. Chi chay khi ban bam vao file nay.
REM Khong co tac vu tu dong nao duoc dang ky.
title Zalo Cleanup v5
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0ZaloCleanup.ps1"
if errorlevel 1 pause

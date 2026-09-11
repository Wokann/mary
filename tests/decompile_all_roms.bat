@echo off
setlocal
chcp 65001 >nul

cd /d "%~dp0.."

echo [1/7] Building mary...
cargo build --quiet --bin mary
if errorlevel 1 goto :failed

echo [2/7] Decompiling FoMT JP as Mary-C...
target\debug\mary.exe decompile rom\fomt_jp.gba goodies\fomt_callables.mary.h --all --mary-c --symbols goodies\fomt_scripts_text.mary.sym --charmap charmap.txt -D MARY_FOMT_JP -o decompiled_text\fomt_jp
if errorlevel 1 goto :failed

echo [3/7] Decompiling FoMT US as Mary-C...
target\debug\mary.exe decompile rom\fomt_us.gba goodies\fomt_callables.mary.h --all --mary-c --symbols goodies\fomt_scripts_text.mary.sym --charmap charmap.txt -D MARY_FOMT_US -o decompiled_text\fomt_us
if errorlevel 1 goto :failed

echo [4/7] Decompiling FoMT EU as Mary-C...
target\debug\mary.exe decompile rom\fomt_eu.gba goodies\fomt_callables.mary.h --all --mary-c --symbols goodies\fomt_scripts_text.mary.sym --charmap charmap.txt -D MARY_FOMT_EU -o decompiled_text\fomt_eu
if errorlevel 1 goto :failed

echo [5/7] Decompiling FoMT DE as Mary-C...
target\debug\mary.exe decompile rom\fomt_de.gba goodies\fomt_callables.mary.h --all --mary-c --symbols goodies\fomt_scripts_text.mary.sym --charmap charmap.txt -D MARY_FOMT_DE -o decompiled_text\fomt_de
if errorlevel 1 goto :failed

echo [6/7] Decompiling MFoMT JP as Mary-C...
target\debug\mary.exe decompile rom\mfomt_jp.gba goodies\mfomt_callables.mary.h --all --mary-c --symbols goodies\mfomt_scripts_text.mary.sym --charmap charmap.txt -D MARY_MFOMT_JP -o decompiled_text\mfomt_jp
if errorlevel 1 goto :failed

echo [7/7] Decompiling MFoMT US as Mary-C...
target\debug\mary.exe decompile rom\mfomt_us.gba goodies\mfomt_callables.mary.h --all --mary-c --symbols goodies\mfomt_scripts_text.mary.sym --charmap charmap.txt -D MARY_MFOMT_US -o decompiled_text\mfomt_us
if errorlevel 1 goto :failed

echo.
echo All six original ROMs were decompiled successfully.
exit /b 0

:failed
echo.
echo Decompilation failed. Review the error shown above.
exit /b 1

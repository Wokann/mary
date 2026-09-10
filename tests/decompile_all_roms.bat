@echo off
setlocal
chcp 65001 >nul

cd /d "%~dp0.."

echo [1/5] Building mary...
cargo build --quiet --bin mary
if errorlevel 1 goto :failed

echo [2/5] Decompiling FoMT US as Mary-C...
target\debug\mary.exe decompile rom\fomt.gba goodies\fomt_callables.mary.h --all --mary-c --symbols goodies\fomt_scripts_text.mary.sym --charmap charmap.txt -D MARY_FOMT_US -o decompiled_text\fomt_us
if errorlevel 1 goto :failed

echo [3/5] Decompiling MFoMT US as Mary-C...
target\debug\mary.exe decompile rom\mfomt.gba goodies\mfomt_callables.mary.h --all --mary-c --symbols goodies\mfomt_scripts_text.mary.sym --charmap charmap.txt -D MARY_MFOMT_US -o decompiled_text\mfomt_us
if errorlevel 1 goto :failed

echo [4/5] Decompiling FoMT JP as Mary-C...
target\debug\mary.exe decompile rom\fomtjp.gba goodies\fomt_callables.mary.h --all --mary-c --symbols goodies\fomt_scripts_text.mary.sym --charmap charmap.txt -D MARY_FOMT_JP -o decompiled_text\fomt_jp
if errorlevel 1 goto :failed

echo [5/5] Decompiling MFoMT JP as Mary-C...
target\debug\mary.exe decompile rom\mfomtjp.gba goodies\mfomt_callables.mary.h --all --mary-c --symbols goodies\mfomt_scripts_text.mary.sym --charmap charmap.txt -D MARY_MFOMT_JP -o decompiled_text\mfomt_jp
if errorlevel 1 goto :failed

echo.
echo All four original ROMs were decompiled successfully.
exit /b 0

:failed
echo.
echo Decompilation failed. Review the error shown above.
exit /b 1

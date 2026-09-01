@echo off
setlocal
chcp 65001 >nul

cd /d "%~dp0.."

echo [1/6] Building mary...
cargo build --quiet --bin mary
if errorlevel 1 goto :failed

echo [2/6] Decompiling FoMT US as Mary-C...
target\debug\mary.exe decompile rom\fomt.gba goodies\mary_callables.mary.h --all --mary-c --script-table goodies\mary_scripts.mary.h --symbols goodies\mary_scripts_text.mary.sym --charmap charmap_jp.txt -D MARY_FOMT_US -o decompiled_text\fomt_us
if errorlevel 1 goto :failed

echo [3/6] Decompiling MFoMT US as Mary-C...
target\debug\mary.exe decompile rom\mfomt.gba goodies\mary_callables.mary.h --all --mary-c --script-table goodies\mary_scripts.mary.h --symbols goodies\mary_scripts_text.mary.sym --charmap charmap_jp.txt -D MARY_MFOMT_US -o decompiled_text\mfomt_us
if errorlevel 1 goto :failed

echo [4/6] Decompiling FoMT JP as Mary-C...
target\debug\mary.exe decompile rom\fomtjp.gba goodies\mary_callables.mary.h --all --mary-c --script-table goodies\mary_scripts.mary.h --symbols goodies\mary_scripts_text.mary.sym --charmap charmap_jp.txt -D MARY_FOMT_JP -o decompiled_text\fomt_jp
if errorlevel 1 goto :failed

echo [5/6] Decompiling MFoMT JP as Mary-C...
target\debug\mary.exe decompile rom\mfomtjp.gba goodies\mary_callables.mary.h --all --mary-c --script-table goodies\mary_scripts.mary.h --symbols goodies\mary_scripts_text.mary.sym --charmap charmap_jp.txt -D MARY_MFOMT_JP -o decompiled_text\mfomt_jp
if errorlevel 1 goto :failed

echo [6/6] Decompiling the FoMT JP Chinese test ROM as Mary-C...
target\debug\mary.exe decompile "牧场物语矿石镇汉化rom0709.gba" goodies\mary_callables.mary.h --all --mary-c --script-table goodies\mary_scripts.mary.h --symbols goodies\mary_scripts_text.mary.sym --charmap "2026汉化测试版gb3213码表.tbl" -D MARY_FOMT_JP -o decompiled_text\fomt_cn
if errorlevel 1 goto :failed

echo.
echo All four original ROMs and the additional FoMT JP Chinese test ROM were decompiled successfully.
exit /b 0

:failed
echo.
echo Decompilation failed. Review the error shown above.
exit /b 1

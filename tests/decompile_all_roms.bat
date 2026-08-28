@echo off
setlocal

cd /d "%~dp0.."

echo [1/5] Building mary...
cargo build --quiet
if errorlevel 1 goto :failed

echo [2/5] Decompiling FoMT US...
target\debug\mary.exe decompile rom\fomt.gba goodies\lib_fomt.txt --all --charmap charmap_jp.txt -o decompiled_text\fomt_us
if errorlevel 1 goto :failed

echo [3/5] Decompiling MFoMT US...
target\debug\mary.exe decompile rom\mfomt.gba goodies\lib_mfomt.txt --all --charmap charmap_jp.txt -o decompiled_text\mfomt_us
if errorlevel 1 goto :failed

echo [4/5] Decompiling FoMT JP...
target\debug\mary.exe decompile rom\fomtjp.gba goodies\lib_fomt.txt --all --charmap charmap_jp.txt -o decompiled_text\fomt_jp
if errorlevel 1 goto :failed

echo [5/5] Decompiling MFoMT JP...
target\debug\mary.exe decompile rom\mfomtjp.gba goodies\lib_mfomt.txt --all --charmap charmap_jp.txt -o decompiled_text\mfomt_jp
if errorlevel 1 goto :failed

echo.
echo All four ROMs were decompiled successfully.
exit /b 0

:failed
echo.
echo Decompilation failed. Review the error shown above.
exit /b 1

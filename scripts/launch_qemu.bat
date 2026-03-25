@echo off
setlocal

set PROJ_ROOT=%~dp0..
cd /d "%PROJ_ROOT%"

set PATH_TO_EFI=%1

if exist mnt rmdir /s /q mnt
mkdir mnt\EFI\BOOT

copy /Y "%PATH_TO_EFI%" mnt\EFI\BOOT\BOOTX64.EFI > nul

if not exist log mkdir log

qemu-system-x86_64 ^
  -m 4G ^
  -bios third_party\ovmf\RELEASEX64_OVMF.fd ^
  -machine q35 ^
  -drive format=raw,file=fat:rw:mnt ^
  -monitor telnet:0.0.0.0:2345,server,nowait,logfile=log\qemu_monitor.txt ^
  -chardev stdio,id=char_com1,mux=on,logfile=log\com1.txt ^
  -serial chardev:char_com1 ^
  -device qemu-xhci ^
  -device usb-kbd ^
  -device usb-tablet ^
  -device isa-debug-exit,iobase=0xf4,iosize=0x01

set RETCODE=%ERRORLEVEL%

if %RETCODE% equ 0 (
    exit /b 0
) else if %RETCODE% equ 3 (
    echo.
    echo PASS
    exit /b 0
) else (
    echo.
    echo FAIL: QEMU returned %RETCODE%
    exit /b 1
)

endlocal
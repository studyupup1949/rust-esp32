@echo off
setlocal enabledelayedexpansion

echo 警告：此脚本将删除当前目录下所有子目录中的.git仓库！
echo 此操作不可逆，所有未推送的更改将会丢失！
set /p "confirm=是否继续？(y/N) "

if /i not "!confirm!"=="y" (
    echo 操作已取消
    exit /b 1
)

echo 正在查找并删除.git目录...
for /d /r %%i in (.git) do (
    if exist "%%i" (
        echo 删除Git仓库: %%i
        rd /s /q "%%i"
    )
)

echo 操作完成
endlocal
    `
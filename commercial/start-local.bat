@echo off
REM ============================================
REM RustDesk Pro Server - 本地快速启动脚本
REM ============================================

echo [1/4] 检查编译文件...
if not exist "target\release\rustdesk-pro.exe" (
    echo ❌ 错误: 编译文件不存在！
    echo 请先运行: cargo build --release
    pause
    exit /b 1
)

echo [2/4] 创建必要目录...
if not exist "data" mkdir data
if not exist "logs" mkdir logs
if not exist "keys" mkdir keys

echo [3/4] 启动服务...
echo.
echo ============================================
echo 🚀 RustDesk Pro Server 启动中...
echo ============================================
echo 📊 健康检查: http://localhost:8080/health
echo 📚 API 文档: http://localhost:8080/swagger
echo ============================================
echo.

echo [4/4] 运行服务...
echo.
target\release\rustdesk-pro.exe serve

echo.
echo 服务已停止
pause

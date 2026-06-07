# RustDesk Pro Server 开发环境搭建指南

## 目录

1. [环境要求](#1-环境要求)
2. [Rust 安装](#2-rust-安装)
3. [项目克隆](#3-项目克隆)
4. [依赖安装](#4-依赖安装)
5. [构建项目](#5-构建项目)
6. [运行测试](#6-运行测试)
7. [调试配置](#7-调试配置)
8. [IDE 配置](#8-ide-配置)
9. [常见问题](#9-常见问题)

---

## 1. 环境要求

### 1.1 系统要求

| 系统 | 版本 |
|------|------|
| Windows | Windows 10+ |
| macOS | macOS 10.15+ |
| Linux | Ubuntu 20.04+, CentOS 8+ |

### 1.2 硬件要求

| 资源 | 最小值 | 建议值 |
|------|--------|--------|
| CPU | 2 核 | 4 核 |
| 内存 | 4 GB | 8 GB |
| 存储 | 10 GB | 20 GB |

### 1.3 软件依赖

| 依赖 | 版本 | 说明 |
|------|------|------|
| Rust | 1.75+ | 编程语言 |
| Cargo | 1.75+ | 包管理器 |
| Git | 2.30+ | 版本控制 |
| SQLite | 3.x | 数据库 |
| OpenSSL | 1.1.1+ | 加密库 |

---

## 2. Rust 安装

### 2.1 Linux / macOS

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 配置环境变量（Linux）
source $HOME/.cargo/env

# 配置环境变量（macOS zsh）
echo 'source $HOME/.cargo/env' >> ~/.zshrc
source ~/.zshrc

# 验证安装
rustc --version
cargo --version
```

### 2.2 Windows

```powershell
# PowerShell 中安装
Invoke-WebRequest https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe -y

# 配置环境变量
$env:Path += ";$HOME\.cargo\bin"

# 验证安装
rustc --version
cargo --version
```

### 2.3 设置镜像（国内用户）

```bash
# 设置 crates.io 镜像
cat > ~/.cargo/config << EOF
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"
EOF

# 设置 rustup 镜像
export RUSTUP_DIST_SERVER=https://mirrors.ustc.edu.cn/rust-static
export RUSTUP_UPDATE_ROOT=https://mirrors.ustc.edu.cn/rust-static/rustup
```

---

## 3. 项目克隆

```bash
# 克隆项目
git clone https://github.com/rustdesk/rustdesk-server.git
cd rustdesk-server/commercial

# 查看目录结构
ls -la
```

**项目结构**:
```
commercial/
├── src/           # 源代码
├── docker/        # Docker 配置
├── docs/          # 文档
├── tests/         # 测试脚本
├── Cargo.toml     # 依赖配置
└── Cargo.lock     # 依赖锁定
```

---

## 4. 依赖安装

### 4.1 系统依赖

#### 4.1.1 Ubuntu/Debian

```bash
sudo apt-get update
sudo apt-get install -y \
    build-essential \
    libssl-dev \
    pkg-config \
    sqlite3 \
    libsqlite3-dev
```

#### 4.1.2 CentOS/RHEL

```bash
sudo yum install -y \
    gcc \
    gcc-c++ \
    openssl-devel \
    pkgconfig \
    sqlite-devel
```

#### 4.1.3 macOS

```bash
brew install openssl sqlite3
```

#### 4.1.4 Windows

Windows 通常不需要额外安装系统依赖。

### 4.2 Rust 依赖

```bash
# 进入项目目录
cd rustdesk-server/commercial

# 安装依赖（自动下载并编译）
cargo build --release --dry-run

# 或直接构建（会自动下载依赖）
cargo build --release
```

---

## 5. 构建项目

### 5.1 开发构建

```bash
# 开发模式构建（更快）
cargo build

# 运行开发版本
./target/debug/rustdesk-pro serve
```

### 5.2 生产构建

```bash
# Release 模式构建（优化）
cargo build --release

# 运行生产版本
./target/release/rustdesk-pro serve
```

### 5.3 构建参数

```bash
# 指定目标平台
cargo build --release --target x86_64-unknown-linux-gnu

# 启用特定特性
cargo build --release --features=prometheus

# 构建文档
cargo doc --open
```

### 5.4 交叉编译

```bash
# 安装交叉编译工具链（以 arm 为例）
rustup target add aarch64-unknown-linux-gnu

# 安装交叉编译依赖
sudo apt-get install gcc-aarch64-linux-gnu

# 交叉编译
cargo build --release --target aarch64-unknown-linux-gnu
```

---

## 6. 运行测试

### 6.1 运行所有测试

```bash
# 运行所有测试
cargo test

# 显示测试输出
cargo test -- --show-output

# 运行特定模块测试
cargo test --lib user::manager::tests

# 运行单个测试
cargo test --lib test_create_user
```

### 6.2 测试覆盖

```bash
# 安装测试覆盖工具
cargo install cargo-tarpaulin

# 生成测试覆盖率报告
cargo tarpaulin --out Html
```

### 6.3 性能测试

```bash
# 运行基准测试
cargo bench

# 运行 k6 负载测试
cd tests/k6
k6 run load_test.js

# 运行 Python 基准测试
cd tests/benchmark
python benchmark.py
```

---

## 7. 调试配置

### 7.1 使用 Rust GDB

```bash
# 编译调试版本
cargo build

# 使用 rust-gdb 调试
rust-gdb ./target/debug/rustdesk-pro

# 设置断点
(gdb) break src/main.rs:20
(gdb) run serve
(gdb) next
(gdb) print variable
```

### 7.2 使用 VS Code 调试

创建 `.vscode/launch.json`：

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug rustdesk-pro",
            "program": "${workspaceFolder}/target/debug/rustdesk-pro",
            "args": ["serve"],
            "cwd": "${workspaceFolder}",
            "env": {
                "RUST_LOG": "debug"
            },
            "sourceLanguages": ["rust"]
        }
    ]
}
```

### 7.3 设置日志级别

```bash
# 设置调试日志级别
export RUST_LOG=debug
cargo run -- serve

# 设置特定模块日志级别
export RUST_LOG=rustdesk_pro=debug,sqlx=info
cargo run -- serve
```

---

## 8. IDE 配置

### 8.1 VS Code

#### 8.1.1 安装插件

- Rust Analyzer (推荐)
- CodeLLDB
- Better TOML
- Docker

#### 8.1.2 设置 Rust Analyzer

```json
// .vscode/settings.json
{
    "rust-analyzer.server.path": "rust-analyzer",
    "rust-analyzer.checkOnSave": true,
    "rust-analyzer.cargo.buildScripts.enable": true,
    "rust-analyzer.procMacro.enable": true
}
```

### 8.2 IntelliJ / CLion

#### 8.2.1 安装插件

- Rust

#### 8.2.2 配置项目

1. 打开项目目录
2. 等待索引完成
3. 配置 Rust 工具链

### 8.3 Vim / Neovim

#### 8.3.1 安装插件

```vim
" vim-plug
Plug 'rust-lang/rust.vim'
Plug 'simrat39/rust-tools.nvim'
```

#### 8.3.2 配置 LSP

```lua
require('rust-tools').setup({
    server = {
        on_attach = on_attach,
        settings = {
            ["rust-analyzer"] = {
                checkOnSave = {
                    command = "clippy"
                },
            },
        },
    },
})
```

---

## 9. 常见问题

### 9.1 编译失败 - 缺少 OpenSSL

```bash
# 错误信息
error: failed to run custom build command for `openssl-sys v0.9.67`

# 解决方案
sudo apt-get install libssl-dev    # Ubuntu
sudo yum install openssl-devel    # CentOS
brew install openssl              # macOS
```

### 9.2 编译失败 - 内存不足

```bash
# 错误信息
error: could not compile `rustdesk-pro-server`.

# 解决方案
# 增加 swap 空间
sudo fallocate -l 4G /swapfile
sudo chmod 600 /swapfile
sudo mkswap /swapfile
sudo swapon /swapfile
```

### 9.3 测试失败 - 数据库连接问题

```bash
# 错误信息
Failed to connect to database: Database(SqliteError { code: 14, ... })

# 解决方案
mkdir -p data logs keys
chmod 755 data logs keys
```

### 9.4 Cargo 下载慢

```bash
# 解决方案：配置镜像
cat > ~/.cargo/config << EOF
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"
EOF
```

### 9.5 交叉编译失败

```bash
# 错误信息
error: linking with `cc` failed: exit status: 1

# 解决方案
# 安装交叉编译工具链
sudo apt-get install gcc-aarch64-linux-gnu
rustup target add aarch64-unknown-linux-gnu
```

---

## 附录：常用命令

| 命令 | 说明 |
|------|------|
| `cargo build` | 构建项目 |
| `cargo build --release` | 构建优化版本 |
| `cargo run -- serve` | 运行服务 |
| `cargo test` | 运行测试 |
| `cargo check` | 检查代码 |
| `cargo fmt` | 格式化代码 |
| `cargo clippy` | 代码检查 |
| `cargo doc --open` | 生成文档 |
| `cargo update` | 更新依赖 |

---

## 附录：环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `RUST_LOG` | 日志级别 | info |
| `DATABASE_URL` | 数据库路径 | sqlite:./data/rustdesk_pro.db |
| `JWT_SECRET` | JWT 密钥 | rustdesk-pro-jwt-secret |
| `SERVER_PORT` | 服务端口 | 8080 |
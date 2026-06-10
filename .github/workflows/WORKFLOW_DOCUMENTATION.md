# RustDesk Pro Server - GitHub Actions 工作流文档

## 文档概述

| 属性 | 值 |
|------|-----|
| 文档名称 | RustDesk Pro Server CI/CD 工作流文档 |
| 版本 | v1.0.0 |
| 创建日期 | 2026-06-10 |
| 最后更新 | 2026-06-10 |
| 适用范围 | 商业版 RustDesk Pro Server |
| 维护者 | RustDesk Team |

---

## 目录

1. [工作流体系概述](#1-工作流体系概述)
2. [工作流详细说明](#2-工作流详细说明)
3. [流程图与架构](#3-流程图与架构)
4. [角色与职责定义](#4-角色与职责定义)
5. [触发条件与前提条件](#5-触发条件与前提条件)
6. [关键决策点与分支流程](#6-关键决策点与分支流程)
7. [异常处理机制](#7-异常处理机制)
8. [审批流程说明](#8-审批流程说明)
9. [工具与系统规范](#9-工具与系统规范)
10. [版本控制与更新记录](#10-版本控制与更新记录)

---

## 1. 工作流体系概述

### 1.1 工作流清单

| 工作流名称 | 文件名 | 职责 | 触发条件 |
|-----------|--------|------|---------|
| CI | `commercial-ci.yml` | 代码质量检查、编译验证 | push/PR/workflow_dispatch |
| Security | `commercial-security.yml` | 安全扫描、漏洞检测 | schedule/push/workflow_dispatch |
| Commercial Build | `commercial-build.yml` | 编译、打包、Release | push/tags/workflow_dispatch |
| CD | `commercial-cd.yml` | Docker 镜像构建与推送 | push/tags/workflow_dispatch |
| Commercial Build & Deploy | `commercial-build-deploy.yml` | 统一构建部署流程 | push/tags/workflow_dispatch |

### 1.2 工作流架构图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        RustDesk Pro Server CI/CD                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────┐    │
│   │     CI      │    │  Security   │    │   Build & Deploy        │    │
│   │   (质量检查) │    │  (安全扫描)  │    │   (构建部署)            │    │
│   └─────────────┘    └─────────────┘    └─────────────────────────┘    │
│          │                  │                      │                   │
│          ▼                  ▼                      ▼                   │
│   ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────┐    │
│   │  fmt/clippy │    │ cargo-audit │    │  Pre-Build Preparation  │    │
│   │  test/build │    │  trivy-scan │    │  Parallel Builds        │    │
│   │  summary    │    │   codeql    │    │  Build Validation       │    │
│   └─────────────┘    │  secret-scan│    │  Debian Package         │    │
│                      │  summary    │    │  GitHub Release         │    │
│                      └─────────────┘    │  Docker Build & Push    │    │
│                                         │  Docker Manifest        │    │
│                                         │  Deployment Summary     │    │
│                                         └─────────────────────────┘    │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## 2. 工作流详细说明

### 2.1 CI 工作流 (commercial-ci.yml)

#### 2.1.1 工作流目标

确保代码质量符合项目标准，在代码合并前进行自动化检查，防止低质量代码进入主分支。

#### 2.1.2 适用范围

- 所有推送到 `main` 或 `develop` 分支的代码
- 所有针对 `main` 或 `develop` 分支的 Pull Request
- 手动触发执行

#### 2.1.3 前提条件

| 条件 | 说明 |
|------|------|
| 代码仓库 | 必须有访问权限 |
| Rust 工具链 | 自动安装，无需预配置 |
| 子模块 | 必须完整初始化 |

#### 2.1.4 步骤分解

| 步骤 | Job 名称 | 负责人 | 操作说明 | 输入 | 输出 |
|------|---------|--------|---------|------|------|
| 1 | fmt | GitHub Actions | 检查代码格式化 | 源代码 | 格式化检查结果 |
| 2 | clippy | GitHub Actions | 静态分析检查 | 源代码 | Clippy 警告/错误 |
| 3 | test | GitHub Actions | 运行单元测试 | 源代码 | 测试通过/失败 |
| 4 | build | GitHub Actions | 编译验证 | 源代码 | 编译产物 |
| 5 | summary | GitHub Actions | 汇总检查结果 | 各 Job 结果 | 最终状态报告 |

#### 2.1.5 详细流程

```yaml
# 步骤 1: 代码格式化检查 (fmt)
- Checkout repository
- Install Rust toolchain with rustfmt
- Check formatting: cargo fmt --check

# 步骤 2: 静态分析检查 (clippy)
- Checkout repository
- Install Rust toolchain with clippy
- Cache cargo dependencies
- Run Clippy: cargo clippy -- -D warnings

# 步骤 3: 单元测试 (test)
- Checkout repository
- Install Rust toolchain
- Cache cargo dependencies
- Run tests: cargo test -- --nocapture

# 步骤 4: 编译验证 (build)
- Checkout repository
- Install Rust toolchain
- Cache cargo dependencies
- Build: cargo build --release (x86_64-gnu, x86_64-musl)

# 步骤 5: 综合检查汇总 (summary)
- needs: [fmt, clippy, test, build]
- Report CI status
```

---

### 2.2 Security 工作流 (commercial-security.yml)

#### 2.2.1 工作流目标

自动化安全扫描，检测依赖漏洞、代码安全问题、敏感信息泄露，保障项目安全性。

#### 2.2.2 适用范围

- 每周日凌晨自动运行
- 推送到 `main` 分支时触发
- 手动触发执行

#### 2.2.3 前提条件

| 条件 | 说明 |
|------|------|
| GitHub Token | 需要 `security-events: write` 权限 |
| CodeQL | GitHub 提供的免费服务 |
| Gitleaks | 需要配置 `.gitleaks.toml` |

#### 2.2.4 步骤分解

| 步骤 | Job 名称 | 负责人 | 操作说明 | 输入 | 输出 |
|------|---------|--------|---------|------|------|
| 1 | cargo-audit | GitHub Actions | Rust 依赖漏洞扫描 | Cargo.toml | 漏洞报告 |
| 2 | trivy-scan | GitHub Actions | 文件系统漏洞扫描 | 源代码 | SARIF 报告 |
| 3 | codeql | GitHub Actions | 代码安全分析 | 源代码 | CodeQL 结果 |
| 4 | secret-scan | GitHub Actions | 密钥泄露检测 | Git 历史 | 密钥泄露报告 |
| 5 | security-summary | GitHub Actions | 汇总安全状态 | 各 Job 结果 | 安全状态报告 |

#### 2.2.5 详细流程

```yaml
# 步骤 1: Cargo 依赖漏洞扫描 (cargo-audit)
- Checkout repository
- Install Rust toolchain
- Install cargo-audit
- Run cargo audit (忽略已知漏洞)

# 步骤 2: Trivy 文件系统扫描 (trivy-scan)
- Checkout code
- Run Trivy vulnerability scanner
- Upload SARIF results to GitHub

# 步骤 3: CodeQL 代码安全分析 (codeql)
- Checkout repository
- Initialize CodeQL (language: rust)
- Build project for CodeQL
- Perform CodeQL Analysis

# 步骤 4: 密钥泄露检测 (secret-scan)
- Checkout repository
- Run Gitleaks with config

# 步骤 5: 安全检查汇总 (security-summary)
- needs: [cargo-audit, trivy-scan, codeql, secret-scan]
- Report security status
```

---

### 2.3 Commercial Build 工作流 (commercial-build.yml)

#### 2.3.1 工作流目标

编译多平台二进制文件，构建 Debian 包，创建 GitHub Release，为部署提供构建产物。

#### 2.3.2 适用范围

- 推送标签 `pro-v*` 或 `pro-*` 时触发
- 推送到 `main` 分支时触发
- 手动触发（可指定版本和平台）

#### 2.3.3 前提条件

| 条件 | 说明 |
|------|------|
| Rust 工具链 | 版本 1.96 |
| Cross 工具 | 用于交叉编译 |
| OpenSSL | musl 静态链接 |
| GitHub Token | 需要 `contents: write` 权限 |

#### 2.3.4 步骤分解

| 步骤 | Job 名称 | 负责人 | 操作说明 | 输入 | 输出 |
|------|---------|--------|---------|------|------|
| 1 | build | GitHub Actions | Linux 多架构编译 | 源代码 | Linux 二进制 |
| 2 | build-win | GitHub Actions | Windows 编译 | 源代码 | Windows 二进制 |
| 3 | deb-package | GitHub Actions | Debian 包构建 | Linux 二进制 | .deb 包 |
| 4 | release | GitHub Actions | GitHub Release | 所有产物 | Release 资产 |

#### 2.3.5 详细流程

```yaml
# 步骤 1: Linux 多架构编译 (build) - 并行执行
矩阵: [amd64, arm64v8, armv7]
目标: [x86_64-musl, aarch64-musl, armv7-musleabihf]

- Checkout
- Install toolchain (1.96)
- Install cross tool
- Install OpenSSL for musl
- Cache cargo
- Build: cross build --release --target
- Upload artifact: binaries-linux-{arch}

# 步骤 2: Windows 编译 (build-win) - 与 Linux 并行
- Checkout
- Install toolchain (1.96)
- Cache cargo
- Build: cargo build --release --target x86_64-pc-windows-msvc
- Upload artifact: binaries-windows-x86_64

# 步骤 3: Debian 包构建 (deb-package) - 仅标签触发
needs: build
矩阵: [amd64, arm64, armhf]

- Checkout
- Download binaries
- Install debhelper, fakeroot
- Build debian package
- Upload artifact: debian-package-{arch}

# 步骤 4: GitHub Release (release) - 仅标签触发
needs: [build, build-win, deb-package]

- Download all artifacts
- Pack Linux binaries (zip)
- Pack Windows binaries (zip)
- Copy debian packages
- Create Release (draft)
```

---

### 2.4 CD 工作流 (commercial-cd.yml)

#### 2.4.1 工作流目标

构建 Docker 镜像，推送至 Docker Hub 和 GHCR，创建多架构 manifest，生成 SBOM。

#### 2.4.2 适用范围

- 推送标签 `pro-v*` 或 `pro-*` 时触发
- 推送到 `main` 分支时触发
- 手动触发执行

#### 2.4.3 前提条件

| 条件 | 说明 |
|------|------|
| Docker Hub 凭证 | `DOCKER_HUB_USERNAME`, `DOCKER_HUB_PASSWORD` |
| GHCR 权限 | `packages: write` |
| 预编译二进制 | 需要 commercial-build 的 artifacts |
| QEMU | 用于多架构构建 |

#### 2.4.4 步骤分解

| 步骤 | Job 名称 | 负责人 | 操作说明 | 输入 | 输出 |
|------|---------|--------|---------|------|------|
| 1 | docker | GitHub Actions | Docker 镜像构建推送 | 二进制 | 单架构镜像 |
| 2 | docker-manifest | GitHub Actions | 多架构 manifest | 单架构镜像 | manifest |
| 3 | release | GitHub Actions | SBOM 生成 | 镜像 | SBOM 文件 |
| 4 | docker-compose | GitHub Actions | 更新 compose 文件 | 镜像标签 | PR |

#### 2.4.5 详细流程

```yaml
# 步骤 1: Docker 镜像构建推送 (docker) - 并行执行
矩阵: [amd64, arm64v8, armv7]
平台: [linux/amd64, linux/arm64, linux/arm/v7]

- Checkout repository
- Download binaries from commercial-build
- Copy binary to rootfs
- Set up QEMU
- Set up Docker Buildx
- Login to GHCR
- Login to Docker Hub
- Build and push Docker image
  标签: :latest-{arch}, :version-{arch}, :major-{arch}

# 步骤 2: 多架构 manifest (docker-manifest) - 仅标签触发
needs: docker

- Login to Docker Hub
- Login to GHCR
- Create Docker Hub manifest (:version, :major, :latest)
- Create GHCR manifest (:version, :major, :latest)

# 步骤 3: SBOM 生成 (release) - 仅标签触发
needs: docker-manifest

- Checkout repository
- Generate SBOM (anchore/sbom-action)
- Create Release with SBOM

# 步骤 4: Docker Compose 更新 (docker-compose) - 仅 main 分支
needs: docker

- Checkout repository
- Update image tag in docker-compose.yml
- Create Pull Request
```

---

### 2.5 Commercial Build & Deploy 工作流 (commercial-build-deploy.yml)

#### 2.5.1 工作流目标

统一的构建部署流程，实现结构化阶段顺序、并行执行、错误处理、进度跟踪。

#### 2.5.2 适用范围

- 推送标签 `pro-v*` 或 `pro-*` 时触发
- 推送到 `main` 分支时触发
- 手动触发（可跳过部署）

#### 2.5.3 前提条件

| 条件 | 说明 |
|------|------|
| 所有 CI 检查 | 必须通过 |
| 所有 Security 检查 | 建议通过 |
| Secrets 配置 | Docker Hub 凭证已配置 |

#### 2.5.4 步骤分解

| 阶段 | Job 名称 | 负责人 | 操作说明 | 输入 | 输出 |
|------|---------|--------|---------|------|------|
| 1 | pre-build | GitHub Actions | 构建准备、版本提取 | Git Ref | Build ID, Version |
| 2 | build-linux | GitHub Actions | Linux 并行编译 | 源代码 | Linux 二进制 |
| 3 | build-windows | GitHub Actions | Windows 编译 | 源代码 | Windows 二进制 |
| 4 | build-summary | GitHub Actions | 构建验证 | Artifacts | 构建状态 |
| 5 | deb-package | GitHub Actions | Debian 包构建 | Linux 二进制 | .deb 包 |
| 6 | github-release | GitHub Actions | GitHub Release | 所有产物 | Release |
| 7 | docker-build | GitHub Actions | Docker 镜像构建 | 二进制 | 单架构镜像 |
| 8 | docker-manifest | GitHub Actions | 多架构 manifest | 单架构镜像 | manifest |
| 9 | deploy-summary | GitHub Actions | 部署汇总 | 所有结果 | 部署报告 |

#### 2.5.5 详细流程

```yaml
# 阶段 1: Pre-Build Preparation
- Checkout repository
- Generate Build ID (timestamp + commit SHA)
- Extract version information
- Output build configuration summary

# 阶段 2: Parallel Builds (Linux) - 并行执行
needs: pre-build
矩阵: [amd64, arm64v8, armv7]

- Checkout
- Install toolchain (1.96)
- Install cross tool
- Build: cross build --release --target
- Verify build
- Upload artifact

# 阶段 3: Parallel Builds (Windows) - 与 Linux 并行
needs: pre-build

- Checkout
- Install toolchain (1.96)
- Build: cargo build --release
- Verify build
- Upload artifact

# 阶段 4: Build Summary & Validation
needs: [build-linux, build-windows]

- Download all artifacts
- Validate artifacts (检查完整性)
- Fail if any build failed

# 阶段 5: Debian Package Build
needs: build-summary
if: startsWith(github.ref, 'refs/tags/')
矩阵: [amd64, arm64, armhf]

- Checkout
- Download binaries
- Build debian package
- Upload artifact

# 阶段 6: GitHub Release
needs: [build-summary, deb-package]
if: startsWith(github.ref, 'refs/tags/')

- Download all artifacts
- Pack binaries (zip)
- Create Release (draft)

# 阶段 7: Docker Build & Push
needs: build-summary
if: !skip-deploy
矩阵: [amd64, arm64v8, armv7]

- Checkout
- Download binaries
- Prepare rootfs
- Set up QEMU, Buildx
- Login to registries
- Build and push

# 阶段 8: Docker Manifest
needs: docker-build
if: startsWith(github.ref, 'refs/tags/')

- Login to registries
- Create multi-arch manifests

# 阶段 9: Deployment Summary
needs: [github-release, docker-manifest]

- Output deployment summary report
```

---

## 3. 流程图与架构

### 3.1 整体 CI/CD 流程图

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           代码提交触发                                    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                    ┌───────────────┼───────────────┐
                    │               │               │
                    ▼               ▼               ▼
            ┌───────────┐   ┌───────────┐   ┌───────────────┐
            │    CI     │   │ Security  │   │ Build & Deploy│
            │  (并行)   │   │  (并行)   │   │   (顺序)      │
            └───────────┘   └───────────┘   └───────────────┘
                    │               │               │
                    ▼               ▼               ▼
            ┌───────────┐   ┌───────────┐   ┌───────────────┐
            │ fmt ✓     │   │ audit ✓   │   │ pre-build     │
            │ clippy ✓  │   │ trivy ✓   │   │ build-linux   │
            │ test ✓    │   │ codeql ✓  │   │ build-windows │
            │ build ✓   │   │ secret ✓  │   │ validation    │
            │ summary ✓ │   │ summary ✓ │   │ package       │
            └───────────┘   └───────────┘   │ release       │
                                            │ docker-build  │
                                            │ manifest      │
                                            │ summary       │
                                            └───────────────┘
                    │               │               │
                    └───────────────┼───────────────┘
                                    │
                                    ▼
            ┌─────────────────────────────────────────────────┐
            │                   部署完成                       │
            │  - GitHub Release (draft)                       │
            │  - Docker Hub: ycstech/rustdesk-pro-server      │
            │  - GHCR: ghcr.io/changsongyang/rustdesk-pro-server│
            └─────────────────────────────────────────────────┘
```

### 3.2 Build & Deploy 详细流程图

```
┌─────────────┐
│  触发事件    │
│ (push/tag)  │
└─────────────┘
      │
      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 阶段 1: Pre-Build Preparation                                   │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ • Checkout repository                                       │ │
│ │ • Generate Build ID                                         │ │
│ │ • Extract version information                               │ │
│ │ • Output build configuration summary                        │ │
│ └─────────────────────────────────────────────────────────────┘ │
│ 输出: build_id, git_tag, deb_version                            │
└─────────────────────────────────────────────────────────────────┘
      │
      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 阶段 2: Parallel Builds                                         │
│ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐    │
│ │   build-linux   │ │   build-linux   │ │   build-linux   │    │
│ │     (amd64)     │ │    (arm64v8)    │ │     (armv7)     │    │
│ │     [并行]      │ │     [并行]      │ │     [并行]      │    │
│ └─────────────────┘ └─────────────────┘ └─────────────────┘    │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │              build-windows (与 Linux 并行)                  │ │
│ └─────────────────────────────────────────────────────────────┘ │
│ 输出: binaries-linux-{arch}, binaries-windows-x86_64            │
└─────────────────────────────────────────────────────────────────┘
      │
      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 阶段 3: Build Summary & Validation                              │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ • Download all artifacts                                    │ │
│ │ • Validate artifacts completeness                           │ │
│ │ • Fail if any build failed                                  │ │
│ └─────────────────────────────────────────────────────────────┘ │
│ 输出: build_status (success/failed)                              │
└─────────────────────────────────────────────────────────────────┘
      │
      │ build_status == 'success' ?
      │
      ├────── 否 ──────► [终止工作流]
      │
      ▼ 是
┌─────────────────────────────────────────────────────────────────┐
│ 阶段 4: Debian Package Build (仅标签触发)                       │
│ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐    │
│ │  deb-package    │ │  deb-package    │ │  deb-package    │    │
│ │    (amd64)      │ │    (arm64)      │ │    (armhf)      │    │
│ │    [并行]       │ │    [并行]       │ │    [并行]       │    │
│ └─────────────────┘ └─────────────────┘ └─────────────────┘    │
│ 输出: debian-package-{arch}                                      │
└─────────────────────────────────────────────────────────────────┘
      │
      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 阶段 5: GitHub Release (仅标签触发)                             │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ • Download all artifacts                                    │ │
│ │ • Pack Linux binaries (zip)                                 │ │
│ │ • Pack Windows binaries (zip)                               │ │
│ │ • Copy debian packages                                      │ │
│ │ • Create Release (draft)                                    │ │
│ └─────────────────────────────────────────────────────────────┘ │
│ 输出: GitHub Release (draft)                                     │
└─────────────────────────────────────────────────────────────────┘
      │
      │ skip-deploy == false ?
      │
      ├────── 否 ──────► [跳过部署阶段]
      │
      ▼ 是
┌─────────────────────────────────────────────────────────────────┐
│ 阶段 6: Docker Build & Push                                     │
│ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐    │
│ │  docker-build   │ │  docker-build   │ │  docker-build   │    │
│ │    (amd64)      │ │    (arm64v8)    │ │     (armv7)     │    │
│ │    [并行]       │ │    [并行]       │ │     [并行]      │    │
│ └─────────────────┘ └─────────────────┘ └─────────────────┘    │
│ 输出: Docker Hub + GHCR 单架构镜像                               │
└─────────────────────────────────────────────────────────────────┘
      │
      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 阶段 7: Docker Manifest (仅标签触发)                            │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ • Login to Docker Hub                                       │ │
│ │ • Login to GHCR                                             │ │
│ │ • Create Docker Hub manifest (:version, :major, :latest)   │ │
│ │ • Create GHCR manifest (:version, :major, :latest)         │ │
│ └─────────────────────────────────────────────────────────────┘ │
│ 输出: 多架构 Docker manifest                                     │
└─────────────────────────────────────────────────────────────────┘
      │
      ▼
┌─────────────────────────────────────────────────────────────────┐
│ 阶段 8: Deployment Summary                                      │
│ ┌─────────────────────────────────────────────────────────────┐ │
│ │ • Output deployment summary report                          │ │
│ │ • List all deployed artifacts                               │ │
│ │ • Confirm deployment status                                 │ │
│ └─────────────────────────────────────────────────────────────┘ │
│ 输出: 部署完成报告                                               │
└─────────────────────────────────────────────────────────────────┘
```

---

## 4. 角色与职责定义

### 4.1 角色定义

| 角色 | 职责 | 权限 |
|------|------|------|
| 开发者 | 编写代码、提交 PR | 读取仓库、创建 PR |
| 审核者 | 审核 PR、批准合并 | 读取仓库、审核 PR |
| 维护者 | 维护工作流、处理异常 | 写入仓库、管理 Secrets |
| 发布管理员 | 发布版本、管理 Release | 写入仓库、创建 Release |
| GitHub Actions | 自动化执行工作流 | 根据工作流配置 |

### 4.2 职责矩阵

| 任务 | 开发者 | 审核者 | 维护者 | 发布管理员 | GitHub Actions |
|------|--------|--------|--------|-----------|---------------|
| 代码提交 | ✅ 执行 | - | - | - | - |
| PR 创建 | ✅ 执行 | - | - | - | - |
| 代码审核 | - | ✅ 执行 | ✅ 协助 | - | - |
| CI 检查 | - | - | - | - | ✅ 自动执行 |
| 安全扫描 | - | - | - | - | ✅ 自动执行 |
| 构建编译 | - | - | - | - | ✅ 自动执行 |
| Docker 推送 | - | - | - | - | ✅ 自动执行 |
| Release 创建 | - | - | ✅ 协助 | ✅ 执行 | ✅ 自动执行 |
| 异常处理 | ✅ 协助 | ✅ 协助 | ✅ 执行 | - | - |
| 工作流维护 | - | - | ✅ 执行 | - | - |

---

## 5. 触发条件与前提条件

### 5.1 触发条件矩阵

| 工作流 | push main | push tag | PR | schedule | workflow_dispatch |
|--------|-----------|----------|-----|----------|-------------------|
| CI | ✅ | ❌ | ✅ | ❌ | ✅ |
| Security | ✅ | ❌ | ❌ | ✅ (每周日) | ✅ |
| Commercial Build | ✅ | ✅ (pro-v*) | ❌ | ❌ | ✅ (带参数) |
| CD | ✅ | ✅ (pro-v*) | ❌ | ❌ | ✅ |
| Build & Deploy | ✅ | ✅ (pro-v*) | ❌ | ❌ | ✅ (带参数) |

### 5.2 前提条件清单

#### 5.2.1 通用前提条件

| 条件 | 说明 | 验证方法 |
|------|------|---------|
| 仓库访问权限 | 必须有读取权限 | GitHub 权限设置 |
| 子模块完整性 | 必须完整初始化 | `git submodule status` |
| Rust 工具链 | 版本 1.96 | 自动安装 |
| Cargo 依赖 | 完整的 Cargo.toml | 自动下载 |

#### 5.2.2 构建前提条件

| 条件 | 说明 | 验证方法 |
|------|------|---------|
| Cross 工具 | 用于交叉编译 | 自动安装 |
| OpenSSL | musl 静态链接 | apt 安装 |
| musl 目标 | Rust target | `rustup target add` |

#### 5.2.3 部署前提条件

| 条件 | 说明 | 验证方法 |
|------|------|---------|
| Docker Hub 凭证 | Secrets 配置 | GitHub Secrets |
| GHCR 权限 | packages: write | 工作流 permissions |
| QEMU | 多架构支持 | docker/setup-qemu-action |
| Buildx | 构建工具 | docker/setup-buildx-action |

---

## 6. 关键决策点与分支流程

### 6.1 决策点清单

| 决策点 | 条件 | 分支 A | 分支 B |
|--------|------|--------|--------|
| D1 | CI 检查结果 | 通过 → 继续 | 失败 → 阻止合并 |
| D2 | 安全扫描结果 | 通过 → 继续 | 失败 → 通知维护者 |
| D3 | 构建验证结果 | 成功 → 部署 | 失败 → 终止工作流 |
| D4 | 触发类型 | 标签 → Release | 分支 → 仅构建 |
| D5 | skip-deploy 参数 | false → 部署 | true → 仅构建 |
| D6 | Release 类型 | draft → 待审核 | published → 正式发布 |

### 6.2 分支流程图

```
                    ┌─────────────┐
                    │  代码提交    │
                    └─────────────┘
                          │
                          ▼
                    ┌─────────────┐
                    │   CI 检查   │
                    └─────────────┘
                          │
            ┌─────────────┼─────────────┐
            │                           │
            ▼                           ▼
    ┌───────────────┐           ┌───────────────┐
    │    通过 ✓     │           │    失败 ✗     │
    │  继续流程     │           │  阻止合并     │
    └───────────────┘           │  通知开发者   │
            │                   └───────────────┘
            ▼
    ┌───────────────┐
    │  安全扫描     │
    └───────────────┘
            │
            ┌─────────────┼─────────────┐
            │                           │
            ▼                           ▼
    ┌───────────────┐           ┌───────────────┐
    │    通过 ✓     │           │    失败 ✗     │
    │  继续流程     │           │  通知维护者   │
    └───────────────┘           │  评估风险     │
            │                   └───────────────┘
            ▼
    ┌───────────────┐
    │  触发类型？   │
    └───────────────┘
            │
            ┌─────────────┼─────────────┐
            │                           │
            ▼                           ▼
    ┌───────────────┐           ┌───────────────┐
    │  标签触发     │           │  分支触发     │
    │  执行部署     │           │  仅执行构建   │
    └───────────────┘           └───────────────┘
            │
            ▼
    ┌───────────────┐
    │ skip-deploy？ │
    └───────────────┘
            │
            ┌─────────────┼─────────────┐
            │                           │
            ▼                           ▼
    ┌───────────────┐           ┌───────────────┐
    │   false       │           │    true       │
    │  执行部署     │           │  跳过部署     │
    └───────────────┘           └───────────────┘
            │
            ▼
    ┌───────────────┐
    │  构建验证     │
    └───────────────┘
            │
            ┌─────────────┼─────────────┐
            │                           │
            ▼                           ▼
    ┌───────────────┐           ┌───────────────┐
    │   成功 ✓      │           │   失败 ✗     │
    │  继续部署     │           │  终止工作流   │
    └───────────────┘           └───────────────┘
            │
            ▼
    ┌───────────────┐
    │  部署完成     │
    └───────────────┘
```

---

## 7. 异常处理机制

### 7.1 异常类型与处理策略

| 异常类型 | 触发条件 | 处理策略 | 负责人 |
|---------|---------|---------|--------|
| CI 检查失败 | fmt/clippy/test/build 失败 | 阻止合并，通知开发者 | 开发者 |
| 安全扫描失败 | cargo-audit/trivy/codeql/secret 失败 | 通知维护者，评估风险 | 维护者 |
| 构建失败 | 编译错误 | 终止工作流，通知开发者 | 开发者 |
| Docker 推送失败 | 权限/网络问题 | 重试或通知维护者 | 维护者 |
| Release 创建失败 | 权限问题 | 通知发布管理员 | 发布管理员 |
| Artifact 缺失 | 上传/下载失败 | 重试或重新构建 | GitHub Actions |

### 7.2 异常处理流程

```
┌─────────────────────────────────────────────────────────────────┐
│                      异常检测                                    │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      异常分类                                    │
│ ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐    │
│ │   CI 异常       │ │  安全异常       │ │  构建异常       │    │
│ └─────────────────┘ └─────────────────┘ └─────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      通知相关角色                                │
│ • CI 异常 → 通知开发者                                          │
│ • 安全异常 → 通知维护者                                          │
│ • 构建异常 → 通知开发者                                          │
│ • 部署异常 → 通知维护者                                          │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      问题修复                                    │
│ • 开发者修复代码                                                 │
│ • 维护者调整配置                                                 │
│ • 重新触发工作流                                                 │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      验证修复                                    │
│ • 重新运行 CI                                                    │
│ • 重新运行构建                                                   │
│ • 确认问题解决                                                   │
└─────────────────────────────────────────────────────────────────┘
```

### 7.3 错误代码对照表

| 错误代码 | 描述 | 解决方案 |
|---------|------|---------|
| E001 | fmt 检查失败 | 运行 `cargo fmt` 格式化代码 |
| E002 | clippy 检查失败 | 修复 clippy 警告 |
| E003 | 测试失败 | 修复测试用例 |
| E004 | 编译失败 | 修复编译错误 |
| E005 | 依赖漏洞 | 更新依赖版本 |
| E006 | 密钥泄露 | 移除敏感信息，使用 Secrets |
| E007 | Docker 推送失败 | 检查凭证配置 |
| E008 | Release 创建失败 | 检查权限配置 |
| E009 | Artifact 缺失 | 重新运行构建 |

---

## 8. 审批流程说明

### 8.1 PR 审批流程

```
┌─────────────────────────────────────────────────────────────────┐
│                      PR 创建                                     │
│ • 开发者创建 Pull Request                                        │
│ • 自动触发 CI 检查                                               │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      CI 检查                                     │
│ • fmt: 代码格式化检查                                            │
│ • clippy: 静态分析检查                                           │
│ • test: 单元测试                                                 │
│ • build: 编译验证                                                │
└─────────────────────────────────────────────────────────────────┘
                                │
            ┌───────────────────┼───────────────────┐
            │                                       │
            ▼                                       ▼
    ┌───────────────────┐               ┌───────────────────┐
    │    CI 通过 ✓      │               │    CI 失败 ✗      │
    │    进入审核       │               │    阻止合并       │
    └───────────────────┘               │    开发者修复     │
            │                           └───────────────────┘
            ▼
    ┌───────────────────────────────────────────────────────────┐
    │                      代码审核                              │
    │ • 审核者检查代码质量                                       │
    │ • 审核者检查测试覆盖率                                     │
    │ • 审核者检查文档完整性                                     │
    └───────────────────────────────────────────────────────────┘
                                │
            ┌───────────────────┼───────────────────┐
            │                                       │
            ▼                                       ▼
    ┌───────────────────┐               ┌───────────────────┐
    │   审核通过 ✓      │               │   审核拒绝 ✗      │
    │   执行合并        │               │   开发者修改      │
    └───────────────────┘               └───────────────────┘
            │
            ▼
    ┌───────────────────────────────────────────────────────────┐
    │                      合并完成                              │
    │ • 自动触发 Security 检查                                   │
    │ • 自动触发 Build 工作流                                    │
    └───────────────────────────────────────────────────────────┘
```

### 8.2 Release 审批流程

```
┌─────────────────────────────────────────────────────────────────┐
│                      标签推送                                    │
│ • 维护者推送标签 pro-v*                                          │
│ • 自动触发 Build & Deploy                                       │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      构建执行                                    │
│ • 编译多平台二进制                                               │
│ • 构建 Debian 包                                                │
│ • 创建 Draft Release                                            │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Release 审核                                │
│ • 发布管理员检查 Release 内容                                    │
│ • 发布管理员检查资产完整性                                       │
│ • 发布管理员检查版本号                                           │
└─────────────────────────────────────────────────────────────────┘
                                │
            ┌───────────────────┼───────────────────┐
            │                                       │
            ▼                                       ▼
    ┌───────────────────┐               ┌───────────────────┐
    │   审核通过 ✓      │               │   审核拒绝 ✗      │
    │   发布 Release    │               │   删除 Draft      │
    │   执行 Docker 推送│               │   修复问题        │
    └───────────────────┘               └───────────────────┘
            │
            ▼
    ┌───────────────────────────────────────────────────────────┐
    │                      发布完成                              │
    │ • Release 正式发布                                         │
    │ • Docker 镜像推送                                          │
    │ • 用户可下载使用                                            │
    └───────────────────────────────────────────────────────────┘
```

---

## 9. 工具与系统规范

### 9.1 工具清单

| 工具 | 版本 | 用途 | 配置位置 |
|------|------|------|---------|
| Rust | 1.96 | 编译 | dtolnay/rust-toolchain |
| Cross | latest | 交叉编译 | cargo install cross |
| Cargo | latest | 依赖管理 | Rust 工具链 |
| Clippy | latest | 静态分析 | Rust 工具链 |
| Rustfmt | latest | 代码格式化 | Rust 工具链 |
| Cargo-audit | latest | 依赖漏洞扫描 | cargo install cargo-audit |
| Trivy | latest | 文件系统扫描 | aquasecurity/trivy-action |
| CodeQL | latest | 代码安全分析 | github/codeql-action |
| Gitleaks | 2.4.0 | 密钥泄露检测 | zricethezav/gitleaks-action |
| Docker | latest | 容器构建 | docker/build-push-action |
| Buildx | latest | 多架构构建 | docker/setup-buildx-action |
| QEMU | latest | 多架构模拟 | docker/setup-qemu-action |

### 9.2 系统规范

#### 9.2.1 运行环境

| 环境 | 规格 | 用途 |
|------|------|------|
| ubuntu-22.04 | 标准 | Linux 构建 |
| ubuntu-latest | 标准 | CI/安全扫描 |
| windows-2022 | 标准 | Windows 构建 |

#### 9.2.2 Secrets 配置

| Secret 名称 | 用途 | 配置位置 |
|-------------|------|---------|
| GITHUB_TOKEN | GitHub API | 自动提供 |
| DOCKER_HUB_USERNAME | Docker Hub 登录 | 仓库 Secrets |
| DOCKER_HUB_PASSWORD | Docker Hub 登录 | 仓库 Secrets |

#### 9.2.3 权限配置

| 权限 | 用途 | 工作流 |
|------|------|--------|
| contents: write | 创建 Release | Build, CD |
| packages: write | 推送 GHCR | CD |
| id-token: write | OIDC 认证 | CD |
| security-events: write | 上传 SARIF | Security |

---

## 10. 版本控制与更新记录

### 10.1 工作流版本历史

| 版本 | 日期 | 变更内容 | 作者 |
|------|------|---------|------|
| v1.0.0 | 2026-06-10 | 初始版本，创建完整工作流体系 | RustDesk Team |
| v1.1.0 | 2026-06-10 | 添加 Build & Deploy 统一工作流 | RustDesk Team |
| v1.2.0 | 2026-06-10 | 优化 Dockerfile.local 国内镜像配置 | RustDesk Team |

### 10.2 文档更新记录

| 版本 | 日期 | 更新内容 | 更新人 |
|------|------|---------|--------|
| v1.0.0 | 2026-06-10 | 创建完整工作流文档 | RustDesk Team |

### 10.3 变更管理流程

```
┌─────────────────────────────────────────────────────────────────┐
│                      变更请求                                    │
│ • 提交变更需求                                                   │
│ • 说明变更原因                                                   │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      变更评估                                    │
│ • 评估变更影响                                                   │
│ • 评估风险                                                       │
│ • 制定实施方案                                                   │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      变更实施                                    │
│ • 更新工作流文件                                                 │
│ • 更新文档                                                       │
│ • 测试验证                                                       │
└─────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                      变更发布                                    │
│ • 提交变更                                                       │
│ • 更新版本记录                                                   │
│ • 通知相关人员                                                   │
└─────────────────────────────────────────────────────────────────┘
```

---

## 附录

### A. 快速参考卡片

#### A.1 常用命令

```bash
# 本地运行 CI 检查
cargo fmt --check
cargo clippy -- -D warnings
cargo test

# 本地构建
cargo build --release

# 触发工作流
git push origin main
git tag pro-v1.0.0 && git push origin pro-v1.0.0

# 手动触发
# GitHub Actions → Run workflow
```

#### A.2 常见问题解决

| 问题 | 解决方案 |
|------|---------|
| CI 检查失败 | 检查错误日志，修复代码 |
| 构建超时 | 检查缓存配置，优化依赖 |
| Docker 推送失败 | 检查 Secrets 配置 |
| Release 未创建 | 检查标签格式和权限 |

### B. 联系方式

| 角色 | 联系方式 |
|------|---------|
| 维护者 | RustDesk Team <info@rustdesk.com> |
| GitHub | https://github.com/changsongyang/rustdesk-pro-server |

---

**文档结束**

*本文档由 RustDesk Team 维护，如有问题请联系维护者。*
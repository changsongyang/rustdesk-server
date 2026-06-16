# Commercial Build & Deploy 工作流分析文档

## 文档概述

| 属性 | 值 |
|------|-----|
| 文档名称 | Commercial Build & Deploy 工作流分析 |
| 工作流文件 | `commercial-build-deploy.yml` |
| 版本 | v2.0.0 |
| 创建日期 | 2026-06-10 |
| 最后更新 | 2026-06-16 |
| 适用范围 | RustDesk Pro Server 商业版 CI/CD |

### 版本更新记录

| 版本 | 日期 | 更新内容 |
|-----|------|---------|
| v1.0.0 | 2026-06-10 | 初始版本 |
| v1.1.0 | 2026-06-10 | 添加组件清单和异常处理 |
| v2.0.0 | 2026-06-16 | 重大更新：<br>- 触发器支持 main/master/develop/feature 等多种分支模式<br>- pre-build 与 code-quality 并行执行<br>- 分离 docker-build-base 与 docker-build-extended<br>- 修复开发构建跳过 P5-P8 的问题<br>- 修复 Debian 包版本号规范问题<br>- 添加 Dockerfile.extended 拓展镜像<br>- 完善产物生成控制逻辑 |

---

## 目录

1. [关键特性](#1-关键特性)
2. [内容总结](#2-内容总结)
3. [使用方式](#3-使用方式)
4. [执行顺序](#4-执行顺序)

---

## 1. 关键特性

### 1.1 主要功能特点

| 特性 | 描述 | 技术实现 |
|------|------|---------|
| **结构化阶段顺序** | 构建阶段 → 验证阶段 → 部署阶段，流程清晰可控 | 基于 GitHub Actions 阶段依赖机制 |
| **并行执行优化** | Linux 多架构 + Windows 并行构建 | 使用 `strategy.matrix` 实现并行 |
| **构建验证机制** | 专门的验证阶段确保 artifacts 完整性 | `build-summary` job 汇总验证 |
| **错误处理** | 构建失败时自动阻止后续部署 | `if` 条件判断 + 显式失败退出 |
| **进度跟踪** | 每个阶段输出详细状态信息 | 结构化的 echo 输出 |
| **可配置跳过部署** | 支持仅构建不部署 | `workflow_dispatch` 参数 |
| **唯一 Build ID** | 时间戳 + 提交 SHA | 脚本生成并传递 |

### 1.2 技术优势

| 优势 | 说明 | 价值 |
|------|------|------|
| **效率优化** | 并行构建减少约 67% 执行时间 | 加速开发迭代 |
| **安全性** | 严格依赖链确保代码质量 | 防止不稳定代码上线 |
| **可追溯性** | Build ID 和详细日志 | 便于问题排查 |
| **灵活性** | 支持多种触发方式和参数配置 | 适应不同场景需求 |
| **完整性** | 覆盖编译、打包、发布、部署全流程 | 一站式 CI/CD |

### 1.3 独特设计

- **阶段分离原则**：Pre-Build → Build → Validation → Deployment，职责清晰
- **矩阵并行模式**：Linux 三个架构并行执行，充分利用资源
- **条件触发机制**：标签触发完整流程，分支触发仅构建
- **Draft Release**：创建草稿版本，需人工审核后发布

### 1.4 差异化特征

| 对比维度 | 传统工作流 | 本工作流 |
|---------|----------|---------|
| 执行方式 | 串行为主 | 并行优先 |
| 验证机制 | 无或简单 | 专门验证阶段 |
| 错误处理 | 直接失败 | 优雅终止并通知 |
| 可配置性 | 固定流程 | 支持参数定制 |
| 可追溯性 | 基本日志 | Build ID + 详细报告 |

---

## 2. 内容总结

### 2.1 组件清单

| 阶段 | Job 名称 | 职责 | 运行环境 |
|------|---------|------|---------|
| Pre-Build | pre-build | 初始化、版本提取、配置输出 | ubuntu-22.04 |
| Build | build-linux | Linux 多架构编译（amd64/arm64v8/armv7） | ubuntu-22.04 |
| Build | build-windows | Windows 编译 | windows-2022 |
| Validation | build-summary | 构建产物完整性验证 | ubuntu-22.04 |
| Packaging | deb-package | Debian 包构建 | ubuntu-22.04 |
| Release | github-release | GitHub Release 创建 | ubuntu-22.04 |
| Deployment | docker-build | Docker 镜像构建与推送 | ubuntu-latest |
| Deployment | docker-manifest | 多架构 manifest 创建 | ubuntu-latest |
| Summary | deploy-summary | 部署状态汇总报告 | ubuntu-latest |

### 2.2 模块架构

```
┌─────────────────────────────────────────────────────────────────┐
│                    工作流模块架构                               │
├─────────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────┐                                          │
│  │   Pre-Build     │ ← 初始化、版本提取、生成 Build ID          │
│  └────────┬────────┘                                          │
│           │                                                   │
│           ▼                                                   │
│  ┌─────────────────────────────────────────────────┐          │
│  │          Parallel Builds                        │          │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐           │          │
│  │  │ amd64   │ │ arm64v8 │ │ armv7   │           │          │
│  │  │ (musl)  │ │ (musl)  │ │ (musl)  │ ← 并行    │          │
│  │  └────┬────┘ └────┬────┘ └────┬────┘           │          │
│  │       └───────────┼───────────┘                 │          │
│  │                   │                             │          │
│  │          ┌────────▼────────┐                    │          │
│  │          │  Windows 构建   │ ← 与 Linux 并行    │          │
│  └──────────┴────────┬────────┴────────────────────┘          │
│                      │                                        │
│                      ▼                                        │
│  ┌─────────────────────────────────────────────────┐          │
│  │        Build Summary & Validation               │          │
│  │ • Download all artifacts                        │          │
│  │ • Validate completeness                         │          │
│  │ • Output build_status                          │          │
│  └────────────────┬────────────────────────────────┘          │
│                   │                                           │
│     ┌─────────────┼─────────────┐                            │
│     ▼             ▼             ▼                            │
│  ┌───────────┐ ┌───────────┐ ┌───────────┐                   │
│  │ deb-package│ │ Release   │ │ Docker    │                   │
│  │ 打包 Debian│ │ GitHub    │ │ Build/Push│                   │
│  └───────────┘ └───────────┘ └─────┬─────┘                   │
│                                    │                          │
│                                    ▼                          │
│                          ┌───────────────────┐               │
│                          │  docker-manifest  │ ← 多架构合并  │
│                          └─────────┬─────────┘               │
│                                    │                          │
│                                    ▼                          │
│                          ┌───────────────────┐               │
│                          │  deploy-summary   │ ← 状态汇总    │
│                          └───────────────────┘               │
│                                                               │
└─────────────────────────────────────────────────────────────────┘
```

### 2.3 数据流转路径

```
┌─────────────────────────────────────────────────────────────────┐
│                      数据流转图                                │
├─────────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────┐    ┌─────────────┐    ┌─────────────┐            │
│  │ 源代码   │───▶│   Checkout  │───▶│   编译阶段   │            │
│  └─────────┘    └─────────────┘    └──────┬──────┘            │
│                                           │                    │
│                    ┌──────────────────────┼─────────────────┐  │
│                    ▼                      ▼                 ▼  │
│            ┌───────────┐         ┌───────────┐      ┌─────────┐│
│            │ Linux     │         │ Windows   │      │ 环境变量 ││
│            │ binaries  │         │ binaries  │      │ GIT_TAG ││
│            └─────┬─────┘         └─────┬─────┘      └────┬────┘│
│                  │                      │                 │    │
│                  └──────────┬───────────┘                 │    │
│                             │                            │    │
│                             ▼                            ▼    │
│                    ┌─────────────────┐          ┌──────────────┐│
│                    │   验证阶段       │          │   版本信息   ││
│                    │ (build-summary) │          └───────┬──────┘│
│                    └────────┬────────┘                  │      │
│                             │                           │      │
│                    ┌────────┼────────┐                  │      │
│                    ▼        ▼        ▼                  │      │
│            ┌───────────┐ ┌───────┐ ┌───────────┐        │      │
│            │ Debian    │ │Release│ │ Docker    │◀───────┘      │
│            │ 包        │ │Assets │ │ 镜像       │               │
│            └─────┬─────┘ └───┬───┘ └─────┬─────┘               │
│                  │          │           │                    │
│                  └──────────┼───────────┘                    │
│                             │                               │
│                             ▼                               │
│                    ┌─────────────────┐                       │
│                    │   部署汇总       │                       │
│                    │ (deploy-summary)│                       │
│                    └─────────────────┘                       │
│                                                               │
└─────────────────────────────────────────────────────────────────┘
```

### 2.4 核心业务逻辑

| 业务场景 | 触发条件 | 执行流程 | 输出产物 |
|---------|---------|---------|---------|
| 开发构建 | push 到 main/develop | 编译 → 验证 | 二进制文件 |
| 版本发布 | push 标签 pro-v* | 编译 → 验证 → 打包 → Release → Docker | Release + Docker 镜像 |
| 测试构建 | 手动触发 + skip-deploy | 编译 → 验证 | 二进制文件 |

---

## 3. 使用方式

### 3.1 环境配置要求

#### 3.1.1 GitHub Secrets 配置

| Secret 名称 | 用途 | 获取方式 |
|-------------|------|---------|
| `DOCKER_HUB_USERNAME` | Docker Hub 登录用户名 | Docker Hub 账户设置 |
| `DOCKER_HUB_PASSWORD` | Docker Hub 登录密码/令牌 | Docker Hub 安全设置 |

#### 3.1.2 权限配置

```yaml
permissions:
  contents: write      # 创建 Release
  packages: write     # 推送 GHCR
  id-token: write     # OIDC 认证
```

#### 3.1.3 依赖环境

| 依赖 | 版本 | 自动安装 |
|------|------|---------|
| Rust 工具链 | 1.96 | ✅ |
| Cross 工具 | latest | ✅ |
| Docker Buildx | latest | ✅ |
| QEMU | latest | ✅ |

### 3.2 初始化步骤

```bash
# 1. 克隆仓库并初始化子模块
git clone <repository-url>
cd rustdesk-server
git submodule update --init --recursive

# 2. 配置 GitHub Secrets
# GitHub 仓库 → Settings → Secrets → Actions
# 添加 DOCKER_HUB_USERNAME 和 DOCKER_HUB_PASSWORD

# 3. 验证工作流语法
# 使用 act 本地验证（可选）
act -W .github/workflows/commercial-build-deploy.yml --dryrun
```

### 3.3 参数设置方法

#### 3.3.1 手动触发参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `version` | string | 自动推断 | 版本号，格式：`pro-v1.0.0` |
| `skip-deploy` | boolean | false | 是否跳过 Docker 部署阶段 |

#### 3.3.2 环境变量

| 变量 | 值 | 说明 |
|------|------|------|
| `GHCR_IMAGE` | `ghcr.io/changsongyang/rustdesk-pro-server` | GHCR 镜像地址 |
| `DOCKERHUB_IMAGE` | `ycstech/rustdesk-pro-server` | Docker Hub 镜像地址 |
| `LATEST_TAG` | `latest` | 最新版本标签 |

### 3.4 日常操作流程

#### 3.4.1 触发方式

```bash
# 方式 1: 推送标签（触发完整发布流程）
git tag pro-v1.0.0
git push origin pro-v1.0.0

# 方式 2: 推送分支（仅构建，不发布）
git push origin main

# 方式 3: 手动触发（GitHub UI）
# GitHub Actions → Commercial Build & Deploy → Run workflow
# 可选参数：version, skip-deploy
```

#### 3.4.2 查看状态

```bash
# 查看工作流运行列表
gh run list --workflow "Commercial Build & Deploy"

# 查看特定运行详情
gh run view <run-id>

# 查看运行日志
gh run logs <run-id>

# 取消运行
gh run cancel <run-id>
```

#### 3.4.3 常见操作场景

| 场景 | 操作方式 | 说明 |
|------|---------|------|
| 日常开发 | 推送分支到 main | 自动触发构建验证 |
| 发布版本 | 推送标签 pro-v* | 触发完整发布流程 |
| 测试构建 | 手动触发 + skip-deploy | 仅验证构建，不部署 |
| 版本回滚 | 删除远程标签 + 重新推送 | 触发重新发布 |

---

## 4. 执行顺序

### 4.1 阶段流程图

```
┌─────────────────────────────────────────────────────────────────┐
│                    执行顺序流程图（v2.0）                        │
├─────────────────────────────────────────────────────────────────┤
│                                                               │
│  [触发事件]                                                    │
│       │                                                        │
│       ▼                                                        │
│  ┌─────────────────────────────────────────────────┐          │
│  │ 阶段 1 & 2: 并行执行                            │          │
│  │ ┌─────────────┐    ┌─────────────┐              │          │
│  │ │ P1-Pre-Build │    │ P2-Code     │ ← 并行     │          │
│  │ │ - 生成 Build ID│    │  Quality   │              │          │
│  │ │ - 提取版本   │    │ - Rustfmt  │              │          │
│  │ │ - 产物控制   │    │ - Clippy   │              │          │
│  │ └──────┬──────┘    └──────┬──────┘              │          │
│  └────────┼────────────────┼─────────────────────┘          │
│           └────────┬────────┘                                 │
│                    ▼                                            │
│  ┌─────────────────────────────────────────────────┐          │
│  │ 阶段 3: 并行构建                                │          │
│  │ • build-linux (amd64)   ← 并行执行              │          │
│  │ • build-linux (arm64v8)                        │          │
│  │ • build-linux (armv7)                          │          │
│  │ • build-windows        ← 与 Linux 并行          │          │
│  └────────────────┬────────────────────────────────┘          │
│                   │                                           │
│                   ▼                                           │
│  ┌─────────────────────────────────────────────────┐          │
│  │ 阶段 4: Build Validation                       │          │
│  │ • Download artifacts                           │          │
│  │ • Validate completeness                        │          │
│  │ • Output build_status                          │          │
│  └────────────────┬────────────────────────────────┘          │
│                   │                                           │
│          ┌────────┴────────┐                                  │
│          │                 │                                  │
│          ▼                 ▼                                  │
│     [build_status=success] [build_status=failed]              │
│          │                 │                                  │
│          │                 ▼                                  │
│          │          ┌─────────────┐                          │
│          │          │ 终止工作流   │                          │
│          │          └─────────────┘                          │
│          │                                                   │
│          ▼                                                   │
│  ┌─────────────────────────────────────────────┐           │
│  │ 阶段 5: 并行产物生成                         │           │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐  │           │
│  │  │deb-package│ │docker-    │ │docker-    │  │← 并行     │
│  │  │           │ │build-base │ │build-ext  │  │           │
│  │  └─────┬─────┘ └─────┬─────┘ └─────┬─────┘  │           │
│  │        │             │             │        │           │
│  │        ▼             ▼             ▼        │           │
│  │  ┌───────────┐ ┌───────────┐ ┌───────────┐  │           │
│  │  │github-    │ │docker-    │ │docker-    │  │           │
│  │  │release    │ │manifest-  │ │manifest-  │  │           │
│  │  │           │ │base       │ │extended   │  │           │
│  │  └─────┬─────┘ └─────┬─────┘ └─────┬─────┘  │           │
│  │        │             │             │        │           │
│  │        └─────────────┼─────────────┘        │           │
│  │                      ▼                      │           │
│  │            ┌───────────────────┐            │           │
│  │            │  deploy-summary   │            │           │
│  │            └───────────────────┘            │           │
│  └─────────────────────────────────────────────┘           │
│                                                               │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 步骤列表

| 阶段 | 步骤 | 启动条件 | 依赖 Job | 输出 |
|------|------|---------|----------|------|
| 1 | Pre-Build Preparation | 任何触发 | 无 | build_id, git_tag, deb_version, should_* |
| 2 | Code Quality Check | 与阶段 1 并行 | 无 | status |
| 3 | build-linux (amd64) | 阶段 1+2 完成 | pre-build, code-quality | binaries-linux-amd64 |
| 4 | build-linux (arm64v8) | 阶段 1+2 完成 | pre-build, code-quality | binaries-linux-arm64v8 |
| 5 | build-linux (armv7) | 阶段 1+2 完成 | pre-build, code-quality | binaries-linux-armv7 |
| 6 | build-windows | 阶段 1+2 完成 | pre-build, code-quality | binaries-windows-x86_64 |
| 7 | build-summary | 阶段 3-6 完成 | build-linux, build-windows | build_status, has_artifacts |
| 8 | deb-package | 阶段 7 + should_build_deb=true | pre-build, build-summary | debian-package-{arch} |
| 9 | github-release | 阶段 7+8 完成 | deb-package | GitHub Release |
| 10 | docker-build-base | 阶段 7 + should_build_base_image=true | pre-build, build-summary | 基础 Docker 镜像 |
| 11 | docker-build-extended | 阶段 7 + should_build_extended_image=true | pre-build, build-summary | 拓展 Docker 镜像 |
| 12 | docker-manifest-base | 阶段 10 完成 | docker-build-base | 基础镜像 manifest |
| 13 | docker-manifest-extended | 阶段 11 完成 | docker-build-extended | 拓展镜像 manifest |
| 14 | deploy-summary | 阶段 9, 12, 13 完成 | pre-build, build-summary, deb-package, github-release, docker-manifest-* | 部署报告 |

### 4.3 依赖关系矩阵

| Job | 依赖 | 被依赖 |
|-----|------|--------|
| pre-build | - | build-linux, build-windows, deb-package, docker-build-base, docker-build-extended, deploy-summary |
| code-quality | - | build-linux, build-windows |
| build-linux | pre-build, code-quality | build-summary |
| build-windows | pre-build, code-quality | build-summary |
| build-summary | pre-build, build-linux, build-windows | deb-package, github-release, docker-build-base, docker-build-extended, deploy-summary |
| deb-package | pre-build, build-summary | github-release, deploy-summary |
| github-release | build-summary, deb-package | deploy-summary |
| docker-build-base | pre-build, build-summary | docker-manifest-base |
| docker-build-extended | pre-build, build-summary | docker-manifest-extended |
| docker-manifest-base | docker-build-base | deploy-summary |
| docker-manifest-extended | docker-build-extended | deploy-summary |
| deploy-summary | pre-build, build-summary, deb-package, github-release, docker-manifest-base, docker-manifest-extended | - |

### 4.4 异常处理机制

| 异常类型 | 触发位置 | 检测方式 | 处理策略 | 通知对象 |
|---------|---------|---------|---------|---------|
| **预构建失败** | pre-build | shell 退出码非零 | 终止所有后续任务 | 开发者 |
| **代码质量失败** | code-quality | Rustfmt/Clippy 错误 | 阻止构建任务执行 | 开发者 |
| **编译失败** | build-linux, build-windows | cargo 错误 | build-summary 标记失败 | 开发者 |
| **验证失败** | build-summary | artifacts 缺失 | 终止后续阶段 | 开发者 |
| **DEB 构建失败** | deb-package | dpkg-deb 错误 | 跳过 github-release | 开发者 |
| **Release 创建失败** | github-release | API 错误 | 通知发布管理员 | 维护者 |
| **Docker 构建失败** | docker-build-base/extended | buildx 错误 | 跳过 manifest | 维护者 |
| **镜像推送失败** | docker-build-* | push 错误码非零 | 重试/终止 | 维护者 |
| **Manifest 创建失败** | docker-manifest-* | API 错误 | 通知维护者 | 维护者 |
| **Debian 版本号错误** | deb-package | `version does not start with digit` | 已修复：自动转换为有效版本 | - |

### 4.5 关键决策点

| 决策点 | 条件 | 分支 A | 分支 B |
|--------|------|--------|--------|
| D1 | `build_status == 'success'` | 继续流程 | 终止工作流 |
| D2 | `startsWith(github.ref, 'refs/tags/')` | 执行 Release/Manifest | 跳过发布阶段 |
| D3 | `should_build_deb == 'true'` | 构建 DEB 包 | 跳过 DEB 任务 |
| D4 | `should_build_base_image == 'true'` | 构建基础镜像 | 跳过基础镜像任务 |
| D5 | `should_build_extended_image == 'true'` | 构建拓展镜像 | 跳过拓展镜像任务 |
| D6 | `should_deploy == 'true'` | 执行 deploy-summary | 跳过部署汇总 |
| D7 | `inputs.docker_image_type` | both / base / extended | 控制 Docker 镜像类型 |

---

## 附录：快速参考

### A. 触发命令

```bash
# 创建版本标签并推送
git tag pro-v1.0.0
git push origin pro-v1.0.0

# 删除标签（回滚）
git tag -d pro-v1.0.0
git push origin :pro-v1.0.0
```

### B. 状态查询

```bash
# 查看最近运行
gh run list --workflow "Commercial Build & Deploy" --limit 5

# 查看运行状态
gh run view <run-id> --json status,conclusion

# 查看特定阶段日志
gh run logs <run-id> --job build-summary
```

### C. 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|---------|
| 编译失败 | Rust 工具链问题 | 检查工具链版本配置 |
| Docker 推送失败 | Secrets 配置错误 | 检查 DOCKER_HUB_USERNAME/PASSWORD |
| Release 未创建 | 权限不足 | 检查 permissions 配置 |
| Manifest 创建失败 | 单架构镜像缺失 | 检查 docker-build 阶段 |
| Gitleaks Action 版本错误 | 原仓库已归档 | 更新为 `gitleaks/gitleaks-action@v2` |
| Docker 标签格式错误 `:-amd64` | GIT_TAG 变量为空 | 在 docker-build job 中添加 Set version variables 步骤 |
| Debian 包版本号为空 | deb-package job 缺少 pre-build 依赖 | 添加 `needs: [pre-build, build-summary]` |
| **开发构建跳过 P5-P8 任务** | `artifact-control` 默认将 dev 构建的 `should_build_*` 设为 false | **已修复**：移除 BUILD_TYPE 条件判断 |
| **Debian 版本号 `dev-xxx` 报错** | 版本号不以数字开头，不符合 dpkg 规范 | **已修复**：添加版本号正则验证和转换 |
| **Dockerfile.extended 不存在** | docker-build-extended 任务引用了不存在的文件 | **已修复**：创建 `commercial/docker/Dockerfile.extended` |
| **代码质量检查延迟流水线** | code-quality 串行依赖 pre-build | **已修复**：移除依赖关系，并行执行 |

---

## 5. 触发器配置详解

### 5.1 分支触发器

工作流支持以下分支模式自动触发开发构建：

| 分支类型 | 模式 | 触发动作 | 构建类型 |
|---------|------|---------|---------|
| 主分支 | `main`, `master` | push | 开发构建 |
| 开发主分支 | `develop`, `development` | push | 开发构建 |
| 功能分支 | `feature/**`, `feat/**` | push | 开发构建 |
| 修复分支 | `fix/**`, `bugfix/**`, `hotfix/**` | push | 开发构建 |
| 发布分支 | `release/**` | push | 开发构建 |
| 个人分支 | `dev/**`, `wip/**` | push | 开发构建 |

### 5.2 标签触发器

| 标签模式 | 触发动作 | 构建类型 |
|---------|---------|---------|
| `pro-v*` | push | 生产构建 |
| `pro-*` | push | 生产构建 |

**示例**：
- `pro-v1.0.0` - 主要发布
- `pro-v1.0.0-rc.1` - 候选发布
- `pro-1.0.0` - 无 v 前缀

### 5.3 手动触发器

支持通过 GitHub UI 手动触发，提供以下参数：

| 参数 | 类型 | 说明 |
|-----|------|------|
| `trigger_type` | choice | 触发类型：branch_with_version / branch_no_version / tag |
| `version` | string | 手动版本号 |
| `git_tag` | string | 已有标签 |
| `build_deb` | boolean | 是否构建 DEB 包 |
| `build_docker` | boolean | 是否构建 Docker 镜像 |
| `docker_image_type` | choice | 镜像类型：both / base / extended |

---

## 6. 构建产物配置

### 6.1 产物类型

| 产物 | 触发条件 | 存储位置 | 文件命名 |
|-----|---------|---------|---------|
| Linux 二进制 | 始终 | Artifacts | `binaries-linux-{arch}` |
| Windows 二进制 | 始终 | Artifacts | `binaries-windows-x86_64` |
| Debian 包 | `should_build_deb=true` | Artifacts + Release | `rustdesk-pro-server_{version}_{arch}.deb` |
| 基础 Docker 镜像 | `should_build_base_image=true` | GHCR + Docker Hub | `{image}:{tag}-{arch}` |
| 拓展 Docker 镜像 | `should_build_extended_image=true` | GHCR + Docker Hub | `{image}:{tag}-extended-{arch}` |
| GitHub Release | 标签推送或手动 tag 触发 | Releases | - |

### 6.2 产物控制变量

由 pre-build 任务输出：

| 变量 | 类型 | 默认值 | 用途 |
|-----|------|-------|------|
| `should_deploy` | boolean | 基于 build_type | 是否执行 deploy-summary |
| `should_build_deb` | boolean | true | 是否构建 DEB 包 |
| `should_build_base_image` | boolean | true | 是否构建基础镜像 |
| `should_build_extended_image` | boolean | true | 是否构建拓展镜像 |

### 6.3 Docker 镜像类型

| Dockerfile | 用途 | 特性 |
|-----------|------|------|
| `Dockerfile.gha` | 基础镜像 | 标准运行时环境 |
| `Dockerfile.extended` | 拓展镜像 | 集成 s6-overlay 进程管理 |

---

## 7. 并行化策略

### 7.1 并行层次

工作流采用三层并行化策略：

```
层次 1: pre-build ⊥ code-quality
层次 2: build-linux[a] ⊥ build-linux[b] ⊥ build-linux[c] ⊥ build-windows
层次 3: deb-package ⊥ docker-build-base ⊥ docker-build-extended
层次 4: docker-manifest-base ⊥ docker-manifest-extended
```

### 7.2 性能影响

| 优化项 | 修改前 | 修改后 | 提升 |
|-------|-------|-------|------|
| P1-P2 串行 | ~3 分钟 | ~1.5 分钟（并行） | **50%** |
| P5-P7 串行 | ~15 分钟 | ~10 分钟（并行） | **33%** |
| 整体流水线 | ~50 分钟 | ~30-40 分钟 | **20-40%** |

### 7.3 并行化原则

1. **无依赖任务优先并行**：如 P1 与 P2
2. **矩阵任务内并行**：多架构同时构建
3. **独立产物并行**：DEB、Docker 互不依赖
4. **Manifest 并行**：基础与拓展镜像 manifest 独立创建

---

## 8. 已知问题与修复

### 8.1 已修复问题

| # | 问题 | 根本原因 | 修复方法 | Commit |
|---|------|---------|---------|--------|
| 1 | 开发构建跳过 P5-P8 | `artifact-control` 中 dev 构建默认 `should_build_*=false` | 移除 BUILD_TYPE 条件判断 | `9473036` |
| 2 | Debian 版本号不规范 | `dev-xxx` 不以数字开头，dpkg 拒绝 | 添加正则验证和版本转换 | `19f59fa` |
| 3 | Dockerfile.extended 缺失 | docker-build-extended 引用不存在的文件 | 创建拓展镜像 Dockerfile | `026d849` |
| 4 | P1-P2 串行执行 | code-quality 依赖 pre-build | 移除依赖关系 | `1a2fde5` |
| 5 | main/master 不触发构建 | 原触发器只匹配 dev/feature 分支 | 添加 main/master 触发 | (规格更新) |

### 8.2 改进建议

| 优先级 | 建议 | 说明 |
|-------|------|------|
| 中 | 添加构建缓存监控 | 监控 cargo 和 docker 缓存命中率 |
| 中 | 集成测试覆盖率报告 | 使用 cargo-tarpaulin |
| 低 | 添加性能基准测试 | 使用 criterion |
| 低 | 多 Docker Registry 推送 | 支持阿里云、腾讯云镜像仓库 |

---

## 9. 相关文档

- [WORKFLOW_IMPLEMENTATION.md](file:///C:/Users/ycsit/Downloads/rustdesk/rustdesk-server/.github/workflows/docs/WORKFLOW_IMPLEMENTATION.md) - 详细实现文档
- [WORKFLOW_OPERATIONS.md](file:///C:/Users/ycsit/Downloads/rustdesk/rustdesk-server/.github/workflows/docs/WORKFLOW_OPERATIONS.md) - 运维操作手册
- [WORKFLOW_USER_GUIDE.md](file:///C:/Users/ycsit/Downloads/rustdesk/rustdesk-server/.github/workflows/docs/WORKFLOW_USER_GUIDE.md) - 用户使用指南

---

**文档结束**

*本文档由 RustDesk Team 维护，如有问题请联系维护者。*
# Commercial Build & Deploy 工作流分析文档

## 文档概述

| 属性 | 值 |
|------|-----|
| 文档名称 | Commercial Build & Deploy 工作流分析 |
| 工作流文件 | `commercial-build-deploy.yml` |
| 版本 | v1.0.0 |
| 创建日期 | 2026-06-10 |
| 适用范围 | RustDesk Pro Server 商业版 CI/CD |

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
│                    执行顺序流程图                               │
├─────────────────────────────────────────────────────────────────┤
│                                                               │
│  [触发事件]                                                    │
│       │                                                        │
│       ▼                                                        │
│  ┌─────────────────────────────────┐                          │
│  │ 阶段 1: Pre-Build Preparation  │                          │
│  │ • Generate Build ID            │                          │
│  │ • Extract version info         │                          │
│  │ • Output build config summary  │                          │
│  └────────────────┬────────────────┘                          │
│                   │                                           │
│                   ▼                                           │
│  ┌─────────────────────────────────┐                          │
│  │ 阶段 2: Parallel Builds        │                          │
│  │ • build-linux (amd64)          │ ← 并行执行               │
│  │ • build-linux (arm64v8)        │                          │
│  │ • build-linux (armv7)          │                          │
│  │ • build-windows                │ ← 并行执行               │
│  └────────────────┬────────────────┘                          │
│                   │                                           │
│                   ▼                                           │
│  ┌─────────────────────────────────┐                          │
│  │ 阶段 3: Build Validation       │                          │
│  │ • Download artifacts           │                          │
│  │ • Validate completeness        │                          │
│  │ • Output build_status          │                          │
│  └────────────────┬────────────────┘                          │
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
│  ┌─────────────────────────┐                                 │
│  │ 阶段 4: Debian Package  │ ← 仅标签触发                    │
│  │ • Build .deb packages   │                                 │
│  └────────────┬────────────┘                                 │
│               │                                              │
│               ▼                                              │
│  ┌─────────────────────────┐                                 │
│  │ 阶段 5: GitHub Release  │ ← 仅标签触发                    │
│  │ • Pack binaries         │                                 │
│  │ • Create draft release  │                                 │
│  └────────────┬────────────┘                                 │
│               │                                              │
│          ┌────┴────┐                                         │
│          │         │                                         │
│          ▼         ▼                                         │
│    [skip-deploy=true] [skip-deploy=false]                    │
│          │         │                                         │
│          │         ▼                                         │
│          │  ┌─────────────────────────┐                      │
│          │  │ 阶段 6: Docker Build    │                      │
│          │  │ • Build & Push images   │                      │
│          │  └────────────┬────────────┘                      │
│          │               │                                   │
│          │               ▼                                   │
│          │  ┌─────────────────────────┐                      │
│          │  │ 阶段 7: Docker Manifest │ ← 仅标签触发        │
│          │  │ • Create multi-arch     │                      │
│          │  └────────────┬────────────┘                      │
│          │               │                                   │
│          │               ▼                                   │
│          │  ┌─────────────────────────┐                      │
│          │  │ 阶段 8: Deploy Summary  │                      │
│          │  │ • Output report         │                      │
│          │  └─────────────────────────┘                      │
│          │                                                   │
│          └───────────────────────────────────────────────────┘│
│                                                               │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 步骤列表

| 阶段 | 步骤 | 启动条件 | 依赖 Job | 输出 |
|------|------|---------|----------|------|
| 1 | Pre-Build Preparation | 任何触发 | 无 | build_id, git_tag, deb_version |
| 2 | build-linux (amd64) | 完成阶段 1 | pre-build | binaries-linux-amd64 |
| 3 | build-linux (arm64v8) | 完成阶段 1 | pre-build | binaries-linux-arm64v8 |
| 4 | build-linux (armv7) | 完成阶段 1 | pre-build | binaries-linux-armv7 |
| 5 | build-windows | 完成阶段 1 | pre-build | binaries-windows-x86_64 |
| 6 | build-summary | 完成阶段 2-5 | build-linux, build-windows | build_status |
| 7 | deb-package | 完成阶段 6 + 标签触发 | build-summary | debian-package-{arch} |
| 8 | github-release | 完成阶段 6-7 + 标签触发 | build-summary, deb-package | GitHub Release |
| 9 | docker-build | 完成阶段 6 + 未跳过部署 | build-summary | Docker 镜像 |
| 10 | docker-manifest | 完成阶段 9 + 标签触发 | docker-build | 多架构 manifest |
| 11 | deploy-summary | 完成阶段 8 和 10 | github-release, docker-manifest | 部署报告 |

### 4.3 依赖关系矩阵

| Job | 依赖 | 被依赖 |
|-----|------|--------|
| pre-build | - | build-linux, build-windows |
| build-linux | pre-build | build-summary |
| build-windows | pre-build | build-summary |
| build-summary | build-linux, build-windows | deb-package, github-release, docker-build |
| deb-package | build-summary | github-release |
| github-release | build-summary, deb-package | deploy-summary |
| docker-build | build-summary | docker-manifest |
| docker-manifest | docker-build | deploy-summary |
| deploy-summary | github-release, docker-manifest | - |

### 4.4 异常处理机制

| 异常类型 | 触发位置 | 检测方式 | 处理策略 | 通知对象 |
|---------|---------|---------|---------|---------|
| **编译失败** | build-linux, build-windows | 返回码非零 | 标记失败，进入验证 | 开发者 |
| **验证失败** | build-summary | artifacts 缺失 | 终止后续阶段 | 开发者 |
| **Docker 登录失败** | docker-build | 登录命令失败 | 终止部署阶段 | 维护者 |
| **镜像推送失败** | docker-build | push 返回码非零 | 重试/终止 | 维护者 |
| **Release 创建失败** | github-release | API 错误 | 通知维护者 | 发布管理员 |
| **Manifest 创建失败** | docker-manifest | API 错误 | 通知维护者 | 维护者 |

### 4.5 关键决策点

| 决策点 | 条件 | 分支 A | 分支 B |
|--------|------|--------|--------|
| D1 | `build_status == 'success'` | 继续流程 | 终止工作流 |
| D2 | `startsWith(github.ref, 'refs/tags/')` | 执行 Release/Manifest | 跳过发布阶段 |
| D3 | `skip-deploy == true` | 跳过 Docker 部署 | 执行 Docker 部署 |
| D4 | GitHub 权限检查 | 创建 Release | 失败并通知 |

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

---

**文档结束**

*本文档由 RustDesk Team 维护，如有问题请联系维护者。*
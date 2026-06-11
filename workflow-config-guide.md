# RustDesk Server GitHub Actions 工作流配置指南

## 概述

本文档详细介绍了 `.github/workflows/build.yaml` 的配置要求和使用方法。该工作流实现了 RustDesk Server 的自动化构建、Docker 镜像推送和发布流程。

---

## 一、工作流任务架构

工作流包含以下核心任务：

| 任务名称 | 功能描述 | 依赖关系 |
|----------|----------|----------|
| `build` | Linux 多架构二进制构建（amd64/arm64v8/armv7/i386） | 无 |
| `build-win` | Windows 二进制构建 + NSIS 安装包 | `build` |
| `release` | GitHub Draft Release 发布 | `build`, `build-win` |
| `docker` | S6 版本 Docker 单架构镜像构建推送 | `build` |
| `docker-manifest` | S6 版本 Docker 多架构 Manifest | `docker` |
| `docker-classic` | Classic 版本 Docker 镜像构建推送 | `build` |
| `docker-manifest-classic` | Classic 版本 Docker Manifest | `docker` |
| `deb-package` | Debian 包构建 | `build` |

---

## 二、必须配置的 GitHub Secrets

在 GitHub 仓库的 **Settings → Secrets and variables → Actions** 中配置以下 Secrets：

### 2.1 Docker Hub 认证（必需）

| Secret 名称 | 说明 | 示例值 |
|-------------|------|--------|
| `DOCKER_HUB_USERNAME` | Docker Hub 用户名 | `rustdesk` |
| `DOCKER_HUB_PASSWORD` | Docker Hub 访问令牌（非登录密码） | `dckr_pat_xxx...` |

> **获取访问令牌**：登录 Docker Hub → Account Settings → Security → New Access Token

### 2.2 Docker 镜像名称（必需）

| Secret 名称 | 说明 | 示例值 |
|-------------|------|--------|
| `DOCKER_IMAGE` | S6 版本镜像名（Docker Hub） | `rustdesk/rustdesk-server-s6` |
| `DOCKER_IMAGE_CLASSIC` | Classic 版本镜像名（Docker Hub） | `rustdesk/rustdesk-server` |

### 2.3 Windows 代码签名（可选）

| Secret 名称 | 说明 |
|-------------|------|
| `WINDOWS_PFX_BASE64` | Base64 编码的 PFX 证书 |
| `WINDOWS_PFX_PASSWORD` | PFX 证书密码 |
| `WINDOWS_PFX_SHA1_THUMBPRINT` | 证书 SHA1 指纹 |

> **注意**：代码签名步骤目前被 `if: false` 禁用，如需启用请修改配置。

---

## 三、环境变量配置

工作流预设了以下环境变量，位于 `env` 部分：

```yaml
env:
  CARGO_TERM_COLOR: always
  LATEST_TAG: latest
  GHCR_IMAGE: ghcr.io/changsongyang/rustdesk-server-s6
  GHCR_IMAGE_CLASSIC: ghcr.io/changsongyang/rustdesk-server
```

| 变量名 | 值 | 说明 |
|--------|-----|------|
| `CARGO_TERM_COLOR` | `always` | Cargo 构建时彩色输出 |
| `LATEST_TAG` | `latest` | Docker latest 标签名称 |
| `GHCR_IMAGE` | `ghcr.io/[用户名]/rustdesk-server-s6` | S6 版本 GitHub Container Registry 镜像名 |
| `GHCR_IMAGE_CLASSIC` | `ghcr.io/[用户名]/rustdesk-server` | Classic 版本 GHCR 镜像名 |

> **修改建议**：将 `GHCR_IMAGE` 和 `GHCR_IMAGE_CLASSIC` 中的 `changsongyang` 替换为你的 GitHub 用户名。

---

## 四、工作流触发条件

工作流在以下情况自动触发：

```yaml
on:
  workflow_dispatch:  # 手动触发（通过 GitHub UI）
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+'         # v1.2.3 格式
      - '[0-9]+.[0-9]+.[0-9]+'          # 1.2.3 格式
      - 'v[0-9]+.[0-9]+.[0-9]+-[0-9]+'  # v1.2.3-1 格式（预发布）
      - '[0-9]+.[0-9]+.[0-9]+-[0-9]+'   # 1.2.3-1 格式
```

**触发方式**：
1. **自动触发**：推送符合格式的 Git Tag（如 `git tag v1.0.0 && git push origin v1.0.0`）
2. **手动触发**：在 GitHub Actions 页面点击 "Run workflow"

---

## 五、权限配置

工作流需要以下权限：

```yaml
permissions:
  contents: read    # 读取仓库源码
  packages: write   # 推送镜像到 GitHub Container Registry
```

---

## 六、配置步骤

### 6.1 配置 Secrets

1. 进入 GitHub 仓库 → **Settings** → **Secrets and variables** → **Actions**
2. 点击 **New repository secret** 添加以下必需 Secrets：
   - `DOCKER_HUB_USERNAME`
   - `DOCKER_HUB_PASSWORD`
   - `DOCKER_IMAGE`
   - `DOCKER_IMAGE_CLASSIC`

### 6.2 修改环境变量

编辑 `build.yaml`，修改 `GHCR_IMAGE` 和 `GHCR_IMAGE_CLASSIC`：

```yaml
env:
  GHCR_IMAGE: ghcr.io/[你的GitHub用户名]/rustdesk-server-s6
  GHCR_IMAGE_CLASSIC: ghcr.io/[你的GitHub用户名]/rustdesk-server
```

### 6.3 启用 Windows 代码签名（可选）

将以下两行的 `if: false` 改为 `if: true`：

```yaml
# 第 126 行
- name: Sign exe files
  if: true  # 原为 if: false

# 第 155 行  
- name: Sign UI setup file
  if: true  # 原为 if: false
```

---

## 七、执行流程图

```
┌─────────────────────────────────────────────────────────────┐
│                    触发：Tag Push 或手动触发                │
└─────────────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         ▼                    ▼                    ▼
    ┌─────────┐         ┌─────────┐         ┌───────────┐
    │  build  │         │build-win│         │ deb-package│
    │(Linux)  │         │(Windows)│         │(Debian包) │
    └────┬────┘         └────┬────┘         └─────┬─────┘
         │                   │                     │
         ├───────────────────┼─────────────────────┤
         ▼                   ▼                     ▼
    ┌─────────────────────────────────────────────────────┐
    │                    release (GitHub Draft)          │
    └─────────────────────────────────────────────────────┘
         │
         ▼
    ┌─────────────┐           ┌───────────────────┐
    │   docker    │──────────▶│ docker-manifest   │
    │(S6 单架构)  │           │ (S6 多架构合并)    │
    └─────────────┘           └───────────────────┘
         │
         ▼
    ┌───────────────┐           ┌───────────────────────┐
    │docker-classic │──────────▶│docker-manifest-classic│
    │(Classic单架构)│           │ (Classic多架构合并)    │
    └───────────────┘           └───────────────────────┘
```

---

## 八、镜像标签策略

每个构建的 Docker 镜像会打上以下标签：

| 标签格式 | 说明 | 示例 |
|----------|------|------|
| `latest-{arch}` | 最新版本架构标签 | `latest-amd64` |
| `{version}-{arch}` | 完整版本架构标签 | `1.2.3-amd64` |
| `{major}-{arch}` | 主版本架构标签 | `1-amd64` |

**Manifest 合并后**：
- `:latest` → 多架构镜像
- `:1.2.3` → 多架构镜像（仅 Tag 触发时）
- `:1` → 多架构镜像（主版本）

---

## 九、关键技术要点

### 9.1 交叉编译

使用 `actions-rs/cargo` 配合 `use-cross: true` 实现跨平台编译：

```yaml
- name: Build
  uses: actions-rs/cargo@v1
  with:
    command: build
    args: --release --all-features --target=${{ matrix.job.target }}
    use-cross: true
```

### 9.2 多架构支持

支持以下架构：

| 架构名称 | Rust Target | Docker Platform |
|----------|-------------|-----------------|
| amd64 | `x86_64-unknown-linux-musl` | `linux/amd64` |
| arm64v8 | `aarch64-unknown-linux-musl` | `linux/arm64` |
| armv7 | `armv7-unknown-linux-musleabihf` | `linux/arm/v7` |
| i386 | `i686-unknown-linux-musl` | `linux/386` |

### 9.3 Artifacts 传递

通过 `actions/upload-artifact` 和 `actions/download-artifact` 在 jobs 间传递构建产物：

```yaml
# 上传
- name: Publish Artifacts
  uses: actions/upload-artifact@v4
  with:
    name: binaries-linux-${{ matrix.job.name }}
    path: target/${{ matrix.job.target }}/release/*

# 下载
- name: Download binaries
  uses: actions/download-artifact@v4
  with:
    name: binaries-linux-${{ matrix.job.name }}
    path: docker/rootfs/usr/bin
```

### 9.4 Manifest 合并

使用 `Noelware/docker-manifest-action` 创建多架构镜像清单：

```yaml
- name: Create and push manifest (:latest)
  uses: Noelware/docker-manifest-action@0.4.3
  with:
    base-image: ${{ secrets.DOCKER_IMAGE }}:${{ env.LATEST_TAG }}
    extra-images: ${{ secrets.DOCKER_IMAGE }}:${{ env.LATEST_TAG }}-amd64,...
    push: true
```

---

## 十、注意事项

1. **Docker Hub 令牌权限**：确保创建的 Access Token 具有 `read/write` 权限
2. **GHCR 权限**：工作流使用 `GITHUB_TOKEN` 自动授权，无需额外配置
3. **代码签名**：Windows 代码签名功能默认禁用，如需启用请配置相关 Secrets
4. **Draft Release**：发布的 Release 默认是 Draft 状态，需要手动确认后发布

---

## 十一、测试建议

配置完成后，建议先通过 **手动触发**（workflow_dispatch）测试工作流，确保所有配置正确无误后再进行 Tag 推送。

---

**文档版本**：v1.0  
**生成日期**：2026-06-10  
**适用文件**：`.github/workflows/build.yaml`

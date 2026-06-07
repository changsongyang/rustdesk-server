# RustDesk Pro Server 功能开发文档

## 目录

1. [版本对比概览](#1-版本对比概览)
2. [社区版功能说明](#2-社区版功能说明)
3. [商业版功能说明](#3-商业版功能说明)
4. [架构设计差异](#4-架构设计差异)
5. [开发流程](#5-开发流程)
6. [代码结构](#6-代码结构)
7. [功能开发指南](#7-功能开发指南)

---

## 1. 版本对比概览

### 1.1 功能矩阵

| 功能类别 | 社区版 | 商业版 | 说明 |
|----------|--------|--------|------|
| **核心功能** | | | |
| ID/Rendezvous Server (hbbs) | ✅ | ✅ | 基础连接服务 |
| Relay Server (hbbr) | ✅ | ✅ | 中继服务 |
| 端到端加密 | ✅ | ✅ | 安全通信 |
| **用户管理** | ❌ | ✅ | 用户 CRUD、认证授权 |
| **设备管理** | ❌ | ✅ | 设备注册、审批、分组 |
| **许可证管理** | ❌ | ✅ | 许可证生成、验证 |
| **审计日志** | ❌ | ✅ | 操作日志、审计追踪 |
| **监控告警** | ❌ | ✅ | Prometheus + Grafana |
| **API 接口** | ❌ | ✅ | RESTful API |
| **高可用** | ❌ | ✅ | 多实例部署 |

### 1.2 版本特性对比

| 特性 | 社区版 | 商业版 |
|------|--------|--------|
| 授权模式 | Apache 2.0 | 商业授权 |
| 用户数量 | 无限制 | 按许可证 |
| 设备数量 | 无限制 | 按许可证 |
| 支持服务 | 社区支持 | 企业支持 |
| 安全审计 | 基础 | 完整 |
| SLA 保障 | 无 | 可选 |

---

## 2. 社区版功能说明

### 2.1 核心组件

社区版包含三个核心组件：

#### 2.1.1 hbbs (ID/Rendezvous Server)

**职责**：
- 处理客户端注册
- 维护在线状态
- 建立点对点连接

**技术实现**：
```rust
// 简化的连接建立流程
pub async fn establish_connection(
    client_id: &str,
    target_id: &str,
) -> Result<Connection, Error> {
    // 1. 查询目标设备在线状态
    let target_info = lookup_device(target_id).await?;
    
    // 2. 尝试直接连接
    if let Ok(conn) = direct_connect(&target_info).await {
        return Ok(conn);
    }
    
    // 3. 回退到中继连接
    relay_connect(&target_info).await
}
```

#### 2.1.2 hbbr (Relay Server)

**职责**：
- 中转无法直连的流量
- 加密数据传输
- 负载均衡

**技术实现**：
```rust
// 中继流量转发
pub async fn relay_traffic(
    from: &mut TcpStream,
    to: &mut TcpStream,
) -> Result<(), Error> {
    let (mut from_rx, mut from_tx) = from.split();
    let (mut to_rx, mut to_tx) = to.split();
    
    // 双向转发
    let forward1 = tokio::spawn(async move {
        io::copy(&mut from_rx, &mut to_tx).await
    });
    
    let forward2 = tokio::spawn(async move {
        io::copy(&mut to_rx, &mut from_tx).await
    });
    
    tokio::try_join!(forward1, forward2)?;
    Ok(())
}
```

#### 2.1.3 rustdesk-utils

**职责**：
- CLI 工具集
- 密钥生成
- 配置管理

### 2.2 架构特点

```
┌─────────────────────────────────────────────────────┐
│              社区版架构                              │
├─────────────────────────────────────────────────────┤
│                                                     │
│   ┌──────────────┐      ┌──────────────┐           │
│   │     hbbs     │◄────►│     hbbr     │           │
│   │  (ID Server) │      │ (Relay Server)│           │
│   └──────┬───────┘      └──────┬───────┘           │
│          │                     │                    │
│          ▼                     ▼                    │
│   ┌───────────────────────────────────────┐        │
│   │           SQLite Database             │        │
│   │  - 设备注册信息                        │        │
│   │  - 连接日志                           │        │
│   └───────────────────────────────────────┘        │
│                                                     │
└─────────────────────────────────────────────────────┘
```

---

## 3. 商业版功能说明

### 3.1 新增功能模块

#### 3.1.1 用户管理系统

**功能特性**：
- 用户 CRUD 操作
- JWT 认证
- RBAC 角色权限

**角色定义**：
| 角色 | 权限 |
|------|------|
| Admin | 完整权限 |
| Operator | 操作权限 |
| Viewer | 只读权限 |

**数据模型**：
```rust
pub struct User {
    id: String,
    username: String,
    email: String,
    password_hash: String,
    role: Role,
    is_active: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}
```

#### 3.1.2 设备管理系统

**功能特性**：
- 设备注册与审批
- 设备状态管理
- 组织分组管理

**状态流转**：
```
注册 → 待审批 → 在线/离线/离开
           ↓
         拒绝
```

**数据模型**：
```rust
pub struct Device {
    id: String,
    name: String,
    device_id: String,
    status: DeviceStatus,
    user_id: Option<String>,
    organization_id: Option<String>,
    approval_status: ApprovalStatus,
    created_at: DateTime<Utc>,
    last_seen: DateTime<Utc>,
}
```

#### 3.1.3 许可证管理系统

**功能特性**：
- 许可证生成
- 许可证验证
- 授权级别管理

**许可证类型**：
| 类型 | 设备限制 | 有效期 |
|------|----------|--------|
| Basic | 10 | 1年 |
| Pro | 100 | 1年 |
| Enterprise | 无限 | 定制 |

**数据模型**：
```rust
pub struct License {
    id: String,
    license_key: String,
    license_type: LicenseType,
    max_devices: i32,
    valid_until: DateTime<Utc>,
    is_active: bool,
    created_at: DateTime<Utc>,
}
```

#### 3.1.4 审计日志系统

**功能特性**：
- 多类型日志记录
- 条件筛选查询
- 操作追踪

**日志类型**：
| 类型 | 说明 |
|------|------|
| login | 用户登录 |
| device_register | 设备注册 |
| device_approve | 设备审批 |
| config_update | 配置更新 |

**数据模型**：
```rust
pub struct AuditLog {
    id: String,
    log_type: AuditLogType,
    user_id: Option<String>,
    target_id: Option<String>,
    details: serde_json::Value,
    ip_address: String,
    created_at: DateTime<Utc>,
}
```

#### 3.1.5 API 服务

**RESTful API 设计**：

| 模块 | 端点 | 方法 | 认证 |
|------|------|------|------|
| 认证 | /api/auth/login | POST | 否 |
| 认证 | /api/auth/refresh | POST | 是 |
| 用户 | /api/users | GET | 是 |
| 用户 | /api/users/{id} | GET/PUT/DELETE | 是 |
| 设备 | /api/devices | GET/POST | 是 |
| 设备 | /api/devices/{id}/approve | POST | 是 |
| 许可证 | /api/license/generate | POST | 是 |
| 许可证 | /api/license/validate | POST | 否 |
| 审计 | /api/audit | GET | 是 |

### 3.2 商业版架构

```
┌─────────────────────────────────────────────────────────────────────┐
│                       商业版架构                                    │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│   ┌──────────────┐      ┌──────────────┐                           │
│   │     hbbs     │◄────►│     hbbr     │   社区版核心组件          │
│   │  (ID Server) │      │ (Relay Server)│                           │
│   └──────┬───────┘      └──────┬───────┘                           │
│          │                     │                                    │
│          └──────────┬──────────┘                                    │
│                     │                                              │
│                     ▼                                              │
│   ┌───────────────────────────────────────────────┐                │
│   │           商业版扩展层                          │                │
│   │  ┌──────────┐ ┌──────────┐ ┌──────────────┐   │                │
│   │  │ User     │ │ Device   │ │ License     │   │                │
│   │  │ Manager  │ │ Manager  │ │ Manager     │   │                │
│   │  └────┬─────┘ └────┬─────┘ └──────┬──────┘   │                │
│   │       │            │               │           │                │
│   │  ┌────▼─────┐ ┌────▼─────┐ ┌──────▼──────┐   │                │
│   │  │ Audit    │ │ Metrics  │ │    API      │   │                │
│   │  │ Logger   │ │Collector │ │  Gateway    │   │                │
│   │  └──────────┘ └──────────┘ └─────────────┘   │                │
│   └───────────────────────────────────────────────┘                │
│                           │                                        │
│                           ▼                                        │
│   ┌───────────────────────────────────────────────┐                │
│   │           SQLite Database (商业版)            │                │
│   │  - 用户表、设备表、许可证表、审计日志表        │                │
│   └───────────────────────────────────────────────┘                │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 4. 架构设计差异

### 4.1 目录结构对比

**社区版**：
```
rustdesk-server/
├── hbbs/           # ID/Rendezvous Server
├── hbbr/           # Relay Server
├── libs/           # 共享库
└── rustdesk-utils/ # CLI 工具
```

**商业版**：
```
rustdesk-server/
├── commercial/     # 商业版扩展（独立目录）
│   ├── src/
│   │   ├── audit/      # 审计日志
│   │   ├── device/     # 设备管理
│   │   ├── license/    # 许可证管理
│   │   ├── user/       # 用户管理
│   │   ├── web/        # API 服务
│   │   ├── cache.rs    # 缓存管理
│   │   ├── metrics.rs  # 监控指标
│   │   ├── main.rs     # 入口
│   │   └── lib.rs      # 库入口
│   ├── docker/         # Docker 配置
│   ├── monitoring/     # 监控配置
│   ├── docs/           # 文档
│   └── tests/          # 测试脚本
└── ... (社区版组件)
```

### 4.2 设计原则

| 原则 | 说明 |
|------|------|
| **隔离性** | 商业版与社区版代码完全隔离 |
| **扩展性** | 支持插件化扩展 |
| **兼容性** | 保持与社区版协议兼容 |
| **安全性** | 企业级安全标准 |

---

## 5. 开发流程

### 5.1 开发环境设置

```bash
# 克隆项目
git clone https://github.com/rustdesk/rustdesk-server.git
cd rustdesk-server/commercial

# 安装依赖
cargo build --release

# 运行测试
cargo test
```

### 5.2 功能开发流程

```
需求分析 → 设计文档 → 代码实现 → 单元测试 → 集成测试 → 代码审查 → 发布
```

### 5.3 版本控制

- **社区版分支**：`main`
- **商业版分支**：`commercial`
- **发布版本**：`v1.0.0`, `v1.1.0`...

---

## 6. 代码结构

### 6.1 模块划分

```
src/
├── audit/           # 审计日志
│   ├── errors.rs    # 错误定义
│   ├── logger.rs    # 日志记录器
│   ├── models.rs    # 数据模型
│   └── mod.rs       # 模块入口
├── device/          # 设备管理
│   ├── errors.rs
│   ├── manager.rs   # 业务逻辑
│   ├── models.rs
│   └── mod.rs
├── license/         # 许可证管理
│   ├── errors.rs
│   ├── manager.rs
│   ├── models.rs
│   └── mod.rs
├── user/            # 用户管理
│   ├── errors.rs
│   ├── manager.rs
│   ├── models.rs
│   └── mod.rs
├── web/             # Web API
│   ├── handlers.rs  # 请求处理
│   ├── middleware.rs # 中间件
│   ├── routes.rs    # 路由配置
│   └── mod.rs
├── cache.rs         # 缓存管理
├── metrics.rs       # 监控指标
├── main.rs          # 主入口
└── lib.rs           # 库入口
```

### 6.2 核心设计模式

#### 6.2.1 Manager 模式

```rust
pub struct UserManager {
    pool: SqlitePool,
}

impl UserManager {
    pub async fn new() -> Self {
        // 初始化数据库连接
    }
    
    pub async fn create_user(&self, user: CreateUserRequest) -> Result<User, UserError> {
        // 创建用户逻辑
    }
    
    pub async fn get_user(&self, id: &str) -> Result<User, UserError> {
        // 查询用户逻辑
    }
}
```

#### 6.2.2 错误处理模式

```rust
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("用户不存在: {0}")]
    NotFound(String),
    
    #[error("用户名已存在: {0}")]
    UsernameExists(String),
    
    #[error("邮箱已存在: {0}")]
    EmailExists(String),
    
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("密码验证失败")]
    InvalidPassword,
}
```

---

## 7. 功能开发指南

### 7.1 添加新功能模块

**步骤**：

1. **创建模块目录**：
```bash
mkdir -p src/new_feature
```

2. **创建文件**：
   - `models.rs` - 数据模型
   - `manager.rs` - 业务逻辑
   - `errors.rs` - 错误定义
   - `mod.rs` - 模块入口

3. **注册路由**：
```rust
// src/web/routes.rs
pub fn new_feature_routes() -> Router {
    Router::new()
        .route("/api/new_feature", get(get_handler))
        .route("/api/new_feature", post(create_handler))
}
```

4. **编写测试**：
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_create_new_feature() {
        // 测试逻辑
    }
}
```

### 7.2 API 设计规范

**命名规范**：
- 使用 RESTful 风格
- 资源名称使用复数形式
- 动词使用 HTTP 方法

**响应格式**：
```json
{
    "success": true,
    "data": {...},
    "message": "操作成功"
}
```

**错误响应**：
```json
{
    "success": false,
    "error": {
        "code": "NOT_FOUND",
        "message": "资源未找到"
    }
}
```

### 7.3 数据库迁移

```sql
-- scripts/migration_001.sql
CREATE TABLE IF NOT EXISTS new_feature (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

## 附录：功能开发检查清单

- [ ] 需求文档已编写
- [ ] 数据模型已设计
- [ ] API 接口已定义
- [ ] 错误处理已实现
- [ ] 单元测试已编写
- [ ] 集成测试已通过
- [ ] 代码审查已完成
- [ ] 文档已更新
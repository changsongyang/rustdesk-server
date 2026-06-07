# RustDesk Pro Server 技术架构文档

## 1. 概述

RustDesk Pro Server 是 RustDesk 远程桌面软件的企业级服务端解决方案，提供用户管理、设备管理、许可证管理、审计日志等企业级功能。

## 2. 系统架构

### 2.1 整体架构图

```
┌─────────────────────────────────────────────────────────────────────┐
│                      客户端层                                        │
│   RustDesk Client  |  Web Admin Panel  |  API Clients               │
└───────────────────────────┬─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      反向代理层                                      │
│                          Nginx                                      │
│              (负载均衡、SSL 终止、静态资源服务)                        │
└───────────────────────────┬─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      应用服务层                                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │                   RustDesk Pro Server                       │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────┐   │    │
│  │  │ HTTP API │ │ License  │ │   User   │ │   Device    │   │    │
│  │  │ (Axum)   │ │ Manager  │ │ Manager  │ │   Manager   │   │    │
│  │  └────┬─────┘ └────┬─────┘ └────┬─────┘ └──────┬──────┘   │    │
│  │       │            │            │               │           │    │
│  │  ┌────▼─────┐ ┌────▼─────┐ ┌────▼─────────────────────┐    │    │
│  │  │ Metrics  │ │  Cache   │ │         Audit Logger      │    │    │
│  │  │Collector │ │Manager   │ │                          │    │    │
│  │  └──────────┘ └──────────┘ └───────────────────────────┘    │    │
│  └─────────────────────────────────────────────────────────────┘    │
└───────────────────────────┬─────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      数据存储层                                      │
│                 SQLite 数据库 + 内存缓存                              │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 核心模块

| 模块 | 职责 | 技术实现 |
|------|------|----------|
| HTTP API | RESTful 接口服务 | Axum 框架 |
| License Manager | 许可证生成与验证 | Ed25519 数字签名 |
| User Manager | 用户 CRUD 与认证 | JWT 认证 |
| Device Manager | 设备管理与审批 | SQLite 持久化 |
| Audit Logger | 操作日志记录 | 结构化日志 |
| Metrics Collector | Prometheus 指标 | 自定义指标收集 |
| Cache Manager | 内存缓存 | LRU 缓存策略 |

## 3. 技术栈

| 分类 | 技术 | 版本 |
|------|------|------|
| 语言 | Rust | 2021 Edition |
| 框架 | Axum | 0.5 |
| 数据库 | SQLite | 3.x |
| ORM | SQLx | 0.6 |
| 加密 | Sodiumoxide | 0.2 |
| JSON | Serde | 1.0 |
| JWT | jsonwebtoken | 8.3 |
| 日志 | flexi_logger | 0.27 |

## 4. 数据模型

### 4.1 用户模型

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER | 主键 |
| username | VARCHAR(64) | 用户名 |
| email | VARCHAR(128) | 邮箱 |
| password_hash | VARCHAR(256) | 密码哈希 |
| role | VARCHAR(16) | 角色 (Admin/Operator/Viewer) |
| is_active | BOOLEAN | 是否激活 |
| created_at | TIMESTAMP | 创建时间 |
| updated_at | TIMESTAMP | 更新时间 |

### 4.2 设备模型

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER | 主键 |
| name | VARCHAR(128) | 设备名称 |
| device_id | VARCHAR(64) | 设备唯一标识 |
| status | VARCHAR(16) | 状态 (online/offline/away) |
| user_id | INTEGER | 关联用户 |
| organization_id | INTEGER | 组织ID |
| approval_status | VARCHAR(16) | 审批状态 |
| created_at | TIMESTAMP | 创建时间 |
| last_seen | TIMESTAMP | 最后在线时间 |

### 4.3 许可证模型

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER | 主键 |
| license_key | VARCHAR(256) | 许可证密钥 |
| license_type | VARCHAR(16) | 类型 (Basic/Pro/Enterprise) |
| max_devices | INTEGER | 最大设备数 |
| valid_until | TIMESTAMP | 有效期 |
| is_active | BOOLEAN | 是否激活 |
| created_at | TIMESTAMP | 创建时间 |

### 4.4 审计日志模型

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INTEGER | 主键 |
| log_type | VARCHAR(32) | 日志类型 |
| user_id | INTEGER | 操作用户 |
| target_id | VARCHAR(64) | 目标ID |
| details | TEXT | 详细信息(JSON) |
| ip_address | VARCHAR(45) | IP地址 |
| created_at | TIMESTAMP | 创建时间 |

## 5. API 设计

### 5.1 认证端点

| 方法 | 路径 | 描述 |
|------|------|------|
| POST | /api/auth/login | 用户登录 |
| POST | /api/auth/logout | 用户登出 |
| POST | /api/auth/refresh | 刷新令牌 |

### 5.2 用户管理端点

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | /api/users | 获取用户列表 |
| GET | /api/users/{id} | 获取用户详情 |
| POST | /api/users | 创建用户 |
| PUT | /api/users/{id} | 更新用户 |
| DELETE | /api/users/{id} | 删除用户 |

### 5.3 设备管理端点

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | /api/devices | 获取设备列表 |
| GET | /api/devices/{id} | 获取设备详情 |
| POST | /api/devices | 创建设备 |
| PUT | /api/devices/{id} | 更新设备 |
| DELETE | /api/devices/{id} | 删除设备 |
| POST | /api/devices/{id}/approve | 审批设备 |

### 5.4 审计日志端点

| 方法 | 路径 | 描述 |
|------|------|------|
| GET | /api/audit | 查询审计日志 |
| GET | /api/audit/{id} | 获取日志详情 |

### 5.5 许可证端点

| 方法 | 路径 | 描述 |
|------|------|------|
| POST | /api/license/generate | 生成许可证 |
| POST | /api/license/validate | 验证许可证 |
| GET | /api/license/status | 获取许可证状态 |

## 6. 安全设计

### 6.1 认证机制

- **JWT 令牌认证**：使用 RS256 算法签名
- **令牌有效期**：24 小时可配置
- **刷新令牌**：支持令牌刷新机制

### 6.2 授权机制

- **RBAC 角色模型**：Admin / Operator / Viewer
- **权限粒度**：细粒度资源访问控制

### 6.3 数据加密

- **密码存储**：bcrypt 哈希 (10 轮)
- **传输加密**：TLS 1.3
- **敏感数据**：AES-256 加密存储

### 6.4 安全防护

- **请求频率限制**：防止暴力破解
- **输入验证**：参数校验和过滤
- **SQL 注入防护**：ORM 参数化查询

## 7. 监控与告警

### 7.1 指标收集

| 指标类型 | 监控内容 |
|----------|----------|
| HTTP | 请求数、延迟、错误率 |
| 业务 | 活跃用户、设备数、会话数 |
| 数据库 | 查询数、查询延迟 |
| 缓存 | 命中率、未命中率 |

### 7.2 告警规则

| 告警项 | 阈值 |
|--------|------|
| HTTP 错误率 | > 5% |
| 请求延迟 P95 | > 500ms |
| 数据库连接数 | > 80% |
| 缓存命中率 | < 80% |

## 8. 部署架构

### 8.1 Docker 容器化

```yaml
services:
  rustdesk-pro:
    image: rustdesk-pro-server:latest
    ports:
      - "8080:8080"
    volumes:
      - rustdesk-data:/app/data
      - rustdesk-logs:/app/logs
    environment:
      - DATABASE_URL=sqlite:./data/rustdesk_pro.db
      - JWT_SECRET=your-secret-key
```

### 8.2 高可用部署

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Node 1    │    │   Node 2    │    │   Node 3    │
│ rustdesk-pro│    │ rustdesk-pro│    │ rustdesk-pro│
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                  │
       └────────┬─────────┴────────┬─────────┘
                │                  │
                ▼                  ▼
         ┌───────────┐      ┌───────────┐
         │  NFS/     │      │ Prometheus│
         │ Shared DB │      │  + Grafana│
         └───────────┘      └───────────┘
```

## 9. 扩展能力

### 9.1 水平扩展

- 支持多实例部署
- 共享数据库存储
- 负载均衡接入

### 9.2 功能扩展

- 插件化架构设计
- 支持自定义认证 Provider
- Webhook 事件通知

## 10. 版本历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0.0 | 2025-Q4 | 初始版本 |
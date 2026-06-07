# RustDesk Pro Server 代码规范文档

## 目录

1. [概述](#1-概述)
2. [命名规范](#2-命名规范)
3. [代码格式](#3-代码格式)
4. [注释规范](#4-注释规范)
5. [错误处理](#5-错误处理)
6. [设计模式](#6-设计模式)
7. [安全规范](#7-安全规范)
8. [测试规范](#8-测试规范)

---

## 1. 概述

本文档定义 RustDesk Pro Server 的代码规范，确保代码的一致性、可读性和可维护性。

---

## 2. 命名规范

### 2.1 文件命名

| 类型 | 格式 | 示例 |
|------|------|------|
| 模块文件 | snake_case | `user_manager.rs` |
| 测试文件 | `{name}_tests.rs` | `user_tests.rs` |
| 配置文件 | snake_case | `config.toml` |

### 2.2 变量命名

| 类型 | 格式 | 示例 |
|------|------|------|
| 普通变量 | snake_case | `user_name` |
| 常量 | UPPER_SNAKE_CASE | `MAX_USERS` |
| 静态变量 | UPPER_SNAKE_CASE | `GLOBAL_CONFIG` |
| 临时变量 | 简洁描述 | `i`, `buf`, `res` |

### 2.3 函数命名

| 类型 | 格式 | 示例 |
|------|------|------|
| 普通函数 | snake_case | `create_user()` |
| 构造函数 | `new()` | `UserManager::new()` |
| 转换函数 | `to_*()` | `to_string()` |
| 操作函数 | `*_mut()` | `update_user_mut()` |

### 2.4 类型命名

| 类型 | 格式 | 示例 |
|------|------|------|
| 结构体 | PascalCase | `UserManager` |
| 枚举 | PascalCase | `UserRole` |
| 特征 | PascalCase | `DatabaseAccess` |
| 模块 | snake_case | `user` |

### 2.5 宏命名

| 类型 | 格式 | 示例 |
|------|------|------|
| 声明宏 | snake_case | `debug_log!()` |
| 过程宏 | PascalCase | `#[derive(Debug)]` |

---

## 3. 代码格式

### 3.1 缩进

- 使用 4 空格缩进
- 不要使用 Tab

### 3.2 行长度

- 最大 100 字符
- 过长的行需要换行

### 3.3 空格

```rust
// 正确
let x = 1 + 2;
if x > 0 {
    println!("positive");
}

// 错误
let x=1+2;
if x>0{
    println!("positive");
}
```

### 3.4 空行

- 函数之间空一行
- 逻辑块之间空一行
- 导入语句后空一行

### 3.5 导入顺序

```rust
// 标准库
use std::collections::HashMap;
use std::sync::Arc;

// 外部依赖
use axum::{Router, Server};
use sqlx::SqlitePool;

// 内部模块
use crate::user::models::User;
use crate::utils::error::AppError;
```

### 3.6 使用 cargo fmt

```bash
# 格式化项目代码
cargo fmt

# 检查格式化
cargo fmt --check
```

---

## 4. 注释规范

### 4.1 模块注释

```rust
//! 用户管理模块
//! 
//! 提供用户 CRUD 操作、认证授权等功能。
//! 
//! # 功能特性
//! - 用户注册与登录
//! - JWT 认证
//! - RBAC 权限管理
```

### 4.2 函数注释

```rust
/// 创建新用户
/// 
/// # 参数
/// - `username`: 用户名
/// - `email`: 邮箱地址
/// - `password`: 密码
/// - `role`: 用户角色
/// 
/// # 返回值
/// 返回创建的用户信息
/// 
/// # 错误
/// - `UserError::UsernameExists`: 用户名已存在
/// - `UserError::EmailExists`: 邮箱已存在
pub async fn create_user(
    &self,
    username: &str,
    email: &str,
    password: &str,
    role: UserRole,
) -> Result<User, UserError> {
    // ...
}
```

### 4.3 代码注释

```rust
// 计算哈希值（使用 bcrypt，10 轮）
let hash = bcrypt::hash(password, bcrypt::DEFAULT_COST)?;

// 批量插入数据（使用事务提高性能）
let mut tx = self.pool.begin().await?;
for user in users {
    sqlx::query!("INSERT INTO users ...")
        .execute(&mut tx)
        .await?;
}
tx.commit().await?;
```

### 4.4 避免冗余注释

```rust
// 错误：冗余注释
let x = 5; // 将 x 设置为 5

// 正确：注释非直观的逻辑
// 使用指数退避策略重试，最多重试 5 次
for attempt in 0..5 {
    if let Ok(result) = operation().await {
        return Ok(result);
    }
    tokio::time::sleep(Duration::from_millis(100 * 2u64.pow(attempt))).await;
}
```

---

## 5. 错误处理

### 5.1 使用 thiserror

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum UserError {
    #[error("用户不存在: {0}")]
    NotFound(String),
    
    #[error("用户名已存在: {0}")]
    UsernameExists(String),
    
    #[error("数据库错误: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("密码验证失败")]
    InvalidPassword,
}
```

### 5.2 错误传播

```rust
// 正确：使用 ? 操作符传播错误
pub async fn get_user(&self, id: &str) -> Result<User, UserError> {
    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", id)
        .fetch_one(&self.pool)
        .await?;
    Ok(user)
}

// 错误：手动处理错误
pub async fn get_user(&self, id: &str) -> Result<User, UserError> {
    match sqlx::query_as!(User, "SELECT * FROM users WHERE id = ?", id)
        .fetch_one(&self.pool)
        .await
    {
        Ok(user) => Ok(user),
        Err(e) => Err(UserError::Database(e)),
    }
}
```

### 5.3 错误日志

```rust
// 使用 error! 宏记录错误日志
if let Err(e) = operation().await {
    error!("操作失败: {:?}", e);
    return Err(e);
}
```

---

## 6. 设计模式

### 6.1 Manager 模式

```rust
pub struct UserManager {
    pool: SqlitePool,
}

impl UserManager {
    pub async fn new() -> Self {
        let pool = SqlitePool::connect("sqlite:./data/db.sqlite").await?;
        Self { pool }
    }
    
    pub async fn create_user(&self, user: CreateUserRequest) -> Result<User, UserError> {
        // ...
    }
}
```

### 6.2 Builder 模式

```rust
pub struct UserBuilder {
    username: String,
    email: String,
    password: String,
    role: UserRole,
}

impl UserBuilder {
    pub fn new(username: &str, email: &str) -> Self {
        Self {
            username: username.to_string(),
            email: email.to_string(),
            password: String::new(),
            role: UserRole::Viewer,
        }
    }
    
    pub fn password(mut self, password: &str) -> Self {
        self.password = password.to_string();
        self
    }
    
    pub fn role(mut self, role: UserRole) -> Self {
        self.role = role;
        self
    }
    
    pub fn build(self) -> User {
        User {
            id: Uuid::new_v4().to_string(),
            username: self.username,
            email: self.email,
            password_hash: bcrypt::hash(&self.password, bcrypt::DEFAULT_COST).unwrap(),
            role: self.role,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

// 使用
let user = UserBuilder::new("admin", "admin@example.com")
    .password("password123")
    .role(UserRole::Admin)
    .build();
```

### 6.3 依赖注入

```rust
#[derive(Clone)]
pub struct AppState {
    user_manager: Arc<UserManager>,
    device_manager: Arc<DeviceManager>,
    license_manager: Arc<LicenseManager>,
}

// 处理器中获取状态
async fn get_user(
    Path(id): Path<String>,
    Extension(state): Extension<AppState>,
) -> Result<Json<User>, UserError> {
    let user = state.user_manager.get_user(&id).await?;
    Ok(Json(user))
}
```

---

## 7. 安全规范

### 7.1 密码处理

```rust
// 正确：使用 bcrypt 哈希
use bcrypt::{hash, verify, DEFAULT_COST};

// 存储密码
let hash = hash(password, DEFAULT_COST)?;

// 验证密码
if verify(password, &hash)? {
    // 验证成功
}

// 错误：明文存储密码
let password = "password123"; // 不要这样做！
```

### 7.2 SQL 注入防护

```rust
// 正确：使用参数化查询
let user = sqlx::query_as!(User, "SELECT * FROM users WHERE username = ?", username)
    .fetch_one(&self.pool)
    .await?;

// 错误：字符串拼接
let query = format!("SELECT * FROM users WHERE username = '{}'", username); // 危险！
```

### 7.3 JWT 安全

```rust
// 使用环境变量存储密钥
let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");

// 设置合理的过期时间
let expiration = chrono::Utc::now() + chrono::Duration::hours(24);
```

### 7.4 输入验证

```rust
// 使用 validator 库验证输入
use validator::Validate;

#[derive(Validate)]
struct CreateUserRequest {
    #[validate(length(min = 3, max = 64))]
    username: String,
    
    #[validate(email)]
    email: String,
    
    #[validate(length(min = 8))]
    password: String,
}

// 验证请求
let request: CreateUserRequest = serde_json::from_str(&body)?;
request.validate()?;
```

---

## 8. 测试规范

### 8.1 测试结构

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_create_user() {
        // 准备
        let manager = UserManager::new().await;
        
        // 执行
        let result = manager.create_user("test", "test@example.com", "password123", UserRole::Viewer).await;
        
        // 断言
        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.username, "test");
        assert_eq!(user.role, UserRole::Viewer);
    }
}
```

### 8.2 测试命名

| 类型 | 格式 | 示例 |
|------|------|------|
| 单元测试 | `test_` + 功能描述 | `test_create_user` |
| 集成测试 | `test_` + 场景描述 | `test_user_login_flow` |
| 边界测试 | `test_` + 边界条件 | `test_create_user_empty_username` |

### 8.3 测试覆盖率

```bash
# 运行测试并生成覆盖率报告
cargo tarpaulin --out Html

# 要求覆盖率达到 80%
```

### 8.4 测试隔离

```rust
#[tokio::test]
async fn test_create_and_get_user() {
    // 使用内存数据库进行测试
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    
    // 创建表
    sqlx::query!("CREATE TABLE users (...)").execute(&pool).await.unwrap();
    
    // 测试逻辑
    let manager = UserManager { pool };
    let user = manager.create_user("test", "test@example.com", "password123", UserRole::Viewer).await.unwrap();
    let fetched = manager.get_user(&user.id).await.unwrap();
    
    assert_eq!(user.id, fetched.id);
}
```

---

## 附录：代码审查检查清单

- [ ] 命名符合规范
- [ ] 代码格式正确（cargo fmt）
- [ ] 有适当的注释
- [ ] 错误处理正确
- [ ] 没有安全漏洞
- [ ] 有单元测试
- [ ] 测试覆盖关键路径
- [ ] 没有未使用的变量和导入
- [ ] 使用了适当的设计模式
- [ ] 符合团队架构规范
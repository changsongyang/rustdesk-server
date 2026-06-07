# RustDesk Pro Server API 接口文档

## 目录

1. [概述](#1-概述)
2. [认证方式](#2-认证方式)
3. [接口列表](#3-接口列表)
4. [数据模型](#4-数据模型)
5. [错误处理](#5-错误处理)
6. [示例代码](#6-示例代码)

---

## 1. 概述

本文档描述 RustDesk Pro Server 的 RESTful API 接口规范。

**基础 URL**: `http://localhost:8080/api`

**版本**: v1

---

## 2. 认证方式

### 2.1 JWT Token 认证

所有需要认证的接口需要在请求头中携带 JWT Token：

```bash
Authorization: Bearer <token>
```

### 2.2 登录获取 Token

```bash
POST /api/auth/login
Content-Type: application/json

{
    "username": "admin",
    "password": "admin123"
}
```

响应：
```json
{
    "success": true,
    "data": {
        "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
        "token_type": "Bearer",
        "expires_in": 86400
    }
}
```

---

## 3. 接口列表

### 3.1 认证接口

#### 3.1.1 登录

| 属性 | 值 |
|------|------|
| 方法 | POST |
| 路径 | /api/auth/login |
| 认证 | 否 |

**请求体**:
```json
{
    "username": "string (必填)",
    "password": "string (必填)"
}
```

**成功响应** (200):
```json
{
    "success": true,
    "data": {
        "access_token": "string",
        "token_type": "Bearer",
        "expires_in": 86400
    }
}
```

**失败响应** (401):
```json
{
    "success": false,
    "error": {
        "code": "INVALID_CREDENTIALS",
        "message": "用户名或密码错误"
    }
}
```

#### 3.1.2 刷新令牌

| 属性 | 值 |
|------|------|
| 方法 | POST |
| 路径 | /api/auth/refresh |
| 认证 | 是 |

**请求体**:
```json
{
    "refresh_token": "string (必填)"
}
```

**成功响应** (200):
```json
{
    "success": true,
    "data": {
        "access_token": "string",
        "token_type": "Bearer",
        "expires_in": 86400
    }
}
```

#### 3.1.3 登出

| 属性 | 值 |
|------|------|
| 方法 | POST |
| 路径 | /api/auth/logout |
| 认证 | 是 |

**成功响应** (200):
```json
{
    "success": true,
    "message": "登出成功"
}
```

---

### 3.2 用户管理接口

#### 3.2.1 获取用户列表

| 属性 | 值 |
|------|------|
| 方法 | GET |
| 路径 | /api/users |
| 认证 | 是 |

**查询参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| page | int | 否 | 页码，默认 1 |
| size | int | 否 | 每页数量，默认 20 |
| role | string | 否 | 角色筛选 |
| is_active | bool | 否 | 状态筛选 |

**成功响应** (200):
```json
{
    "success": true,
    "data": {
        "items": [
            {
                "id": "string",
                "username": "string",
                "email": "string",
                "role": "admin",
                "is_active": true,
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:00Z"
            }
        ],
        "total": 100,
        "page": 1,
        "size": 20
    }
}
```

#### 3.2.2 获取用户详情

| 属性 | 值 |
|------|------|
| 方法 | GET |
| 路径 | /api/users/{id} |
| 认证 | 是 |

**路径参数**:

| 参数 | 类型 | 说明 |
|------|------|------|
| id | string | 用户ID |

**成功响应** (200):
```json
{
    "success": true,
    "data": {
        "id": "string",
        "username": "string",
        "email": "string",
        "role": "admin",
        "is_active": true,
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    }
}
```

**失败响应** (404):
```json
{
    "success": false,
    "error": {
        "code": "NOT_FOUND",
        "message": "用户不存在"
    }
}
```

#### 3.2.3 创建用户

| 属性 | 值 |
|------|------|
| 方法 | POST |
| 路径 | /api/users |
| 认证 | 是 |

**请求体**:
```json
{
    "username": "string (必填)",
    "email": "string (必填)",
    "password": "string (必填)",
    "role": "string (选填，默认 viewer)"
}
```

**成功响应** (201):
```json
{
    "success": true,
    "data": {
        "id": "string",
        "username": "string",
        "email": "string",
        "role": "viewer",
        "is_active": true,
        "created_at": "2024-01-01T00:00:00Z"
    }
}
```

**失败响应** (409):
```json
{
    "success": false,
    "error": {
        "code": "CONFLICT",
        "message": "用户名已存在"
    }
}
```

#### 3.2.4 更新用户

| 属性 | 值 |
|------|------|
| 方法 | PUT |
| 路径 | /api/users/{id} |
| 认证 | 是 |

**请求体**:
```json
{
    "email": "string (选填)",
    "password": "string (选填)",
    "role": "string (选填)",
    "is_active": "bool (选填)"
}
```

**成功响应** (200):
```json
{
    "success": true,
    "data": {
        "id": "string",
        "username": "string",
        "email": "string",
        "role": "operator",
        "is_active": true,
        "updated_at": "2024-01-01T00:00:00Z"
    }
}
```

#### 3.2.5 删除用户

| 属性 | 值 |
|------|------|
| 方法 | DELETE |
| 路径 | /api/users/{id} |
| 认证 | 是 |

**成功响应** (200):
```json
{
    "success": true,
    "message": "删除成功"
}
```

---

### 3.3 设备管理接口

#### 3.3.1 获取设备列表

| 属性 | 值 |
|------|------|
| 方法 | GET |
| 路径 | /api/devices |
| 认证 | 是 |

**查询参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| page | int | 否 | 页码 |
| size | int | 否 | 每页数量 |
| status | string | 否 | 状态筛选 |
| approval_status | string | 否 | 审批状态筛选 |
| user_id | string | 否 | 用户ID筛选 |

**成功响应** (200):
```json
{
    "success": true,
    "data": {
        "items": [
            {
                "id": "string",
                "name": "string",
                "device_id": "string",
                "status": "online",
                "user_id": "string",
                "organization_id": "string",
                "approval_status": "approved",
                "created_at": "2024-01-01T00:00:00Z",
                "last_seen": "2024-01-01T00:00:00Z"
            }
        ],
        "total": 50,
        "page": 1,
        "size": 20
    }
}
```

#### 3.3.2 获取设备详情

| 属性 | 值 |
|------|------|
| 方法 | GET |
| 路径 | /api/devices/{id} |
| 认证 | 是 |

**成功响应** (200):
```json
{
    "success": true,
    "data": {
        "id": "string",
        "name": "string",
        "device_id": "string",
        "status": "online",
        "user_id": "string",
        "organization_id": "string",
        "approval_status": "approved",
        "created_at": "2024-01-01T00:00:00Z",
        "last_seen": "2024-01-01T00:00:00Z"
    }
}
```

#### 3.3.3 创建设备

| 属性 | 值 |
|------|------|
| 方法 | POST |
| 路径 | /api/devices |
| 认证 | 是 |

**请求体**:
```json
{
    "name": "string (必填)",
    "device_id": "string (必填)",
    "user_id": "string (选填)",
    "organization_id": "string (选填)"
}
```

**成功响应** (201):
```json
{
    "success": true,
    "data": {
        "id": "string",
        "name": "string",
        "device_id": "string",
        "status": "offline",
        "approval_status": "pending",
        "created_at": "2024-01-01T00:00:00Z"
    }
}
```

#### 3.3.4 更新设备

| 属性 | 值 |
|------|------|
| 方法 | PUT |
| 路径 | /api/devices/{id} |
| 认证 | 是 |

**请求体**:
```json
{
    "name": "string (选填)",
    "user_id": "string (选填)",
    "organization_id": "string (选填)"
}
```

#### 3.3.5 删除设备

| 属性 | 值 |
|------|------|
| 方法 | DELETE |
| 路径 | /api/devices/{id} |
| 认证 | 是 |

#### 3.3.6 审批设备

| 属性 | 值 |
|------|------|
| 方法 | POST |
| 路径 | /api/devices/{id}/approve |
| 认证 | 是 |

**请求体**:
```json
{
    "approved": "bool (必填)"
}
```

**成功响应** (200):
```json
{
    "success": true,
    "data": {
        "id": "string",
        "approval_status": "approved",
        "updated_at": "2024-01-01T00:00:00Z"
    }
}
```

---

### 3.4 许可证管理接口

#### 3.4.1 生成许可证

| 属性 | 值 |
|------|------|
| 方法 | POST |
| 路径 | /api/license/generate |
| 认证 | 是 |

**请求体**:
```json
{
    "license_type": "string (必填，basic/pro/enterprise)",
    "max_devices": "int (必填)",
    "valid_days": "int (必填，有效期天数)"
}
```

**成功响应** (201):
```json
{
    "success": true,
    "data": {
        "id": "string",
        "license_key": "RD-PRO-XXXXX-XXXXX-XXXXX",
        "license_type": "pro",
        "max_devices": 100,
        "valid_until": "2025-01-01T00:00:00Z",
        "is_active": true,
        "created_at": "2024-01-01T00:00:00Z"
    }
}
```

#### 3.4.2 验证许可证

| 属性 | 值 |
|------|------|
| 方法 | POST |
| 路径 | /api/license/validate |
| 认证 | 否 |

**请求体**:
```json
{
    "license_key": "string (必填)"
}
```

**成功响应** (200):
```json
{
    "success": true,
    "data": {
        "valid": true,
        "license_type": "pro",
        "max_devices": 100,
        "valid_until": "2025-01-01T00:00:00Z"
    }
}
```

**失败响应** (400):
```json
{
    "success": false,
    "error": {
        "code": "INVALID_LICENSE",
        "message": "许可证无效"
    }
}
```

#### 3.4.3 获取许可证状态

| 属性 | 值 |
|------|------|
| 方法 | GET |
| 路径 | /api/license/status |
| 认证 | 是 |

**成功响应** (200):
```json
{
    "success": true,
    "data": {
        "active": true,
        "license_type": "pro",
        "max_devices": 100,
        "used_devices": 15,
        "valid_until": "2025-01-01T00:00:00Z",
        "days_remaining": 365
    }
}
```

---

### 3.5 审计日志接口

#### 3.5.1 查询审计日志

| 属性 | 值 |
|------|------|
| 方法 | GET |
| 路径 | /api/audit |
| 认证 | 是 |

**查询参数**:

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| page | int | 否 | 页码 |
| size | int | 否 | 每页数量 |
| log_type | string | 否 | 日志类型 |
| user_id | string | 否 | 用户ID |
| start_time | string | 否 | 开始时间 |
| end_time | string | 否 | 结束时间 |

**成功响应** (200):
```json
{
    "success": true,
    "data": {
        "items": [
            {
                "id": "string",
                "log_type": "login",
                "user_id": "string",
                "target_id": "string",
                "details": "{...}",
                "ip_address": "string",
                "created_at": "2024-01-01T00:00:00Z"
            }
        ],
        "total": 1000,
        "page": 1,
        "size": 20
    }
}
```

#### 3.5.2 获取日志详情

| 属性 | 值 |
|------|------|
| 方法 | GET |
| 路径 | /api/audit/{id} |
| 认证 | 是 |

---

### 3.6 健康检查接口

#### 3.6.1 健康检查

| 属性 | 值 |
|------|------|
| 方法 | GET |
| 路径 | /health |
| 认证 | 否 |

**成功响应** (200):
```json
{
    "status": "ok",
    "version": "1.0.0",
    "timestamp": "2024-01-01T00:00:00Z"
}
```

---

## 4. 数据模型

### 4.1 User 模型

```json
{
    "id": "string",
    "username": "string",
    "email": "string",
    "role": "admin | operator | viewer",
    "is_active": "bool",
    "created_at": "ISO8601 timestamp",
    "updated_at": "ISO8601 timestamp"
}
```

### 4.2 Device 模型

```json
{
    "id": "string",
    "name": "string",
    "device_id": "string",
    "status": "online | offline | away",
    "user_id": "string | null",
    "organization_id": "string | null",
    "approval_status": "pending | approved | rejected",
    "created_at": "ISO8601 timestamp",
    "last_seen": "ISO8601 timestamp"
}
```

### 4.3 License 模型

```json
{
    "id": "string",
    "license_key": "string",
    "license_type": "basic | pro | enterprise",
    "max_devices": "int",
    "valid_until": "ISO8601 timestamp",
    "is_active": "bool",
    "created_at": "ISO8601 timestamp"
}
```

### 4.4 AuditLog 模型

```json
{
    "id": "string",
    "log_type": "string",
    "user_id": "string | null",
    "target_id": "string | null",
    "details": "json string",
    "ip_address": "string",
    "created_at": "ISO8601 timestamp"
}
```

---

## 5. 错误处理

### 5.1 错误响应格式

```json
{
    "success": false,
    "error": {
        "code": "string",
        "message": "string"
    }
}
```

### 5.2 错误码列表

| 错误码 | HTTP 状态码 | 说明 |
|--------|-------------|------|
| UNAUTHORIZED | 401 | 未认证 |
| FORBIDDEN | 403 | 无权限 |
| NOT_FOUND | 404 | 资源不存在 |
| CONFLICT | 409 | 资源冲突 |
| INVALID_CREDENTIALS | 401 | 凭据无效 |
| INVALID_LICENSE | 400 | 许可证无效 |
| VALIDATION_ERROR | 400 | 验证错误 |
| INTERNAL_ERROR | 500 | 服务器错误 |

---

## 6. 示例代码

### 6.1 Python 示例

```python
import requests

base_url = "http://localhost:8080/api"

# 登录
response = requests.post(
    f"{base_url}/auth/login",
    json={"username": "admin", "password": "admin123"}
)
token = response.json()["data"]["access_token"]
headers = {"Authorization": f"Bearer {token}"}

# 获取用户列表
users = requests.get(f"{base_url}/users", headers=headers).json()
print(users)

# 创建用户
new_user = requests.post(
    f"{base_url}/users",
    headers=headers,
    json={
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123",
        "role": "viewer"
    }
)
print(new_user.json())
```

### 6.2 JavaScript 示例

```javascript
const baseUrl = 'http://localhost:8080/api';

// 登录
const loginResponse = await fetch(`${baseUrl}/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
        username: 'admin',
        password: 'admin123'
    })
});
const { access_token } = await loginResponse.json();

const headers = {
    'Authorization': `Bearer ${access_token}`,
    'Content-Type': 'application/json'
};

// 获取设备列表
const devicesResponse = await fetch(`${baseUrl}/devices`, { headers });
const devices = await devicesResponse.json();
console.log(devices);
```

### 6.3 cURL 示例

```bash
# 登录
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"admin123"}'

# 获取用户列表（替换 <token>）
curl -X GET http://localhost:8080/api/users \
  -H "Authorization: Bearer <token>"

# 创建用户
curl -X POST http://localhost:8080/api/users \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "password123",
    "role": "viewer"
  }'
```
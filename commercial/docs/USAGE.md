# RustDesk Pro Server 使用文档

## 1. 快速开始

### 1.1 启动服务

```bash
# 使用 Docker Compose 启动
docker-compose -f docker/docker-compose.yml up -d

# 验证服务启动
curl http://localhost:8080/health
```

### 1.2 首次登录

默认管理员账户：
- **用户名**: admin
- **密码**: admin123
- **邮箱**: admin@rustdesk.local

## 2. API 使用指南

### 2.1 认证

#### 2.1.1 登录

```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "admin",
    "password": "admin123"
  }'
```

响应：
```json
{
  "access_token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "token_type": "Bearer",
  "expires_in": 86400
}
```

#### 2.1.2 使用令牌

```bash
curl -X GET http://localhost:8080/api/users \
  -H "Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
```

### 2.2 用户管理

#### 2.2.1 创建用户

```bash
curl -X POST http://localhost:8080/api/users \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john",
    "email": "john@example.com",
    "password": "password123",
    "role": "Operator"
  }'
```

#### 2.2.2 获取用户列表

```bash
curl -X GET http://localhost:8080/api/users \
  -H "Authorization: Bearer <token>"
```

#### 2.2.3 更新用户

```bash
curl -X PUT http://localhost:8080/api/users/1 \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "email": "john.updated@example.com",
    "role": "Admin"
  }'
```

#### 2.2.4 删除用户

```bash
curl -X DELETE http://localhost:8080/api/users/1 \
  -H "Authorization: Bearer <token>"
```

### 2.3 设备管理

#### 2.3.1 创建设备

```bash
curl -X POST http://localhost:8080/api/devices \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Desktop-PC",
    "device_id": "device-abc123",
    "user_id": 1,
    "organization_id": 1
  }'
```

#### 2.3.2 获取设备列表

```bash
curl -X GET "http://localhost:8080/api/devices?status=online" \
  -H "Authorization: Bearer <token>"
```

#### 2.3.3 审批设备

```bash
curl -X POST http://localhost:8080/api/devices/1/approve \
  -H "Authorization: Bearer <token>"
```

### 2.4 许可证管理

#### 2.4.1 生成许可证

```bash
curl -X POST http://localhost:8080/api/license/generate \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "license_type": "Pro",
    "max_devices": 100,
    "valid_days": 365
  }'
```

#### 2.4.2 验证许可证

```bash
curl -X POST http://localhost:8080/api/license/validate \
  -H "Content-Type: application/json" \
  -d '{
    "license_key": "RD-PRO-XXXXX-XXXXX-XXXXX"
  }'
```

### 2.5 审计日志

#### 2.5.1 查询日志

```bash
curl -X GET "http://localhost:8080/api/audit?type=login&page=1&size=20" \
  -H "Authorization: Bearer <token>"
```

## 3. 角色权限说明

### 3.1 角色列表

| 角色 | 权限描述 |
|------|----------|
| Admin | 完全权限，可管理所有资源 |
| Operator | 操作权限，可管理设备和会话 |
| Viewer | 只读权限，仅可查看数据 |

### 3.2 权限矩阵

| 资源 | Admin | Operator | Viewer |
|------|-------|----------|--------|
| 用户管理 | ✅ | ❌ | ❌ |
| 设备管理 | ✅ | ✅ | ❌ |
| 许可证管理 | ✅ | ❌ | ❌ |
| 审计日志 | ✅ | ✅ | ✅ |

## 4. 设备状态说明

### 4.1 状态列表

| 状态 | 说明 |
|------|------|
| online | 设备在线 |
| offline | 设备离线 |
| away | 设备离开 |
| pending | 待审批 |

### 4.2 审批流程

```
设备注册 → 待审批(pending) → 审批通过(online) / 拒绝(offline)
```

## 5. 许可证类型

### 5.1 许可证级别

| 类型 | 最大设备数 | 有效期 | 功能 |
|------|-----------|--------|------|
| Basic | 10 | 1年 | 基础功能 |
| Pro | 100 | 1年 | 高级功能 |
| Enterprise | 无限 | 定制 | 全部功能 |

## 6. 日志类型说明

### 6.1 审计日志类型

| 类型 | 说明 |
|------|------|
| login | 用户登录 |
| logout | 用户登出 |
| user_create | 创建用户 |
| user_update | 更新用户 |
| user_delete | 删除用户 |
| device_register | 设备注册 |
| device_approve | 设备审批 |
| device_delete | 删除设备 |
| license_generate | 生成许可证 |
| license_validate | 验证许可证 |
| config_update | 配置更新 |

## 7. API 错误码

### 7.1 认证错误

| 错误码 | 说明 |
|--------|------|
| 401 | 未授权 |
| 403 | 禁止访问 |
| 400 | 请求参数错误 |

### 7.2 业务错误

| 错误码 | 说明 |
|--------|------|
| 404 | 资源未找到 |
| 409 | 资源冲突 |
| 500 | 服务器错误 |

## 8. 使用示例

### 8.1 Python 客户端示例

```python
import requests

base_url = "http://localhost:8080"

# 登录
response = requests.post(
    f"{base_url}/api/auth/login",
    json={"username": "admin", "password": "admin123"}
)
token = response.json()["access_token"]

headers = {"Authorization": f"Bearer {token}"}

# 获取用户列表
users = requests.get(f"{base_url}/api/users", headers=headers).json()
print(users)

# 创建用户
new_user = requests.post(
    f"{base_url}/api/users",
    headers=headers,
    json={
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123",
        "role": "Viewer"
    }
)
print(new_user.json())
```

### 8.2 JavaScript 客户端示例

```javascript
const baseUrl = 'http://localhost:8080';

// 登录
const loginResponse = await fetch(`${baseUrl}/api/auth/login`, {
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
const devicesResponse = await fetch(`${baseUrl}/api/devices`, { headers });
const devices = await devicesResponse.json();
console.log(devices);
```

## 9. 最佳实践

### 9.1 安全实践

- 使用 HTTPS 传输
- 定期轮换 JWT_SECRET
- 使用强密码策略
- 限制 API 访问频率

### 9.2 性能优化

- 使用缓存减少数据库查询
- 分页查询大量数据
- 异步处理耗时操作

### 9.3 日志管理

- 定期清理旧日志
- 配置日志级别
- 集成日志收集系统

## 10. 常见问题

### 10.1 如何重置管理员密码？

```bash
# 通过 API 重置（需要现有令牌）
curl -X PUT http://localhost:8080/api/users/1 \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"password": "newpassword123"}'
```

### 10.2 如何查看设备在线状态？

```bash
curl -X GET "http://localhost:8080/api/devices?status=online" \
  -H "Authorization: Bearer <token>"
```

### 10.3 如何导出审计日志？

```bash
curl -X GET "http://localhost:8080/api/audit?page=1&size=100" \
  -H "Authorization: Bearer <token>" \
  -o audit_logs.json
```
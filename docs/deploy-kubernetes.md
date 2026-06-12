# RustDesk Server - Kubernetes 部署指南

## 概述

本文档提供 RustDesk Server 的 Kubernetes 部署指南，包含完整的资源配置、网络规则和生产级配置。

---

## 1. 环境准备

### 1.1 安装 kubectl

```bash
# 下载 kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"

# 验证二进制文件
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl.sha256"
echo "$(cat kubectl.sha256) kubectl" | sha256sum --check

# 安装 kubectl
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# 验证安装
kubectl version --client
```

### 1.2 配置 Kubernetes 集群

```bash
# 配置 kubeconfig
mkdir -p ~/.kube
cp /path/to/your/kubeconfig ~/.kube/config

# 验证集群连接
kubectl cluster-info

# 检查节点状态
kubectl get nodes
```

---

## 2. 创建 Namespace

```bash
# 创建 rustdesk namespace
kubectl create namespace rustdesk

# 验证创建
kubectl get namespaces
```

---

## 3. 社区版部署

### 3.1 创建 ConfigMap

```bash
cat > rustdesk-configmap.yaml << 'EOF'
apiVersion: v1
kind: ConfigMap
metadata:
  name: rustdesk-config
  namespace: rustdesk
data:
  RELAY_SERVER: "your-domain.com:21117"
  RUST_LOG: "info"
EOF

# 应用配置
kubectl apply -f rustdesk-configmap.yaml
```

### 3.2 创建 Secret（可选）

```bash
cat > rustdesk-secret.yaml << 'EOF'
apiVersion: v1
kind: Secret
metadata:
  name: rustdesk-secret
  namespace: rustdesk
type: Opaque
data:
  # Base64 encoded secret values
  # echo -n "your-secret" | base64
  API_KEY: ""
EOF

# 应用配置
kubectl apply -f rustdesk-secret.yaml
```

### 3.3 创建 PersistentVolumeClaim

```bash
cat > rustdesk-pvc.yaml << 'EOF'
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: rustdesk-data
  namespace: rustdesk
spec:
  accessModes:
    - ReadWriteOnce
  resources:
    requests:
      storage: 10Gi
EOF

# 应用配置
kubectl apply -f rustdesk-pvc.yaml
```

### 3.4 创建 hbbs Deployment

```bash
cat > hbbs-deployment.yaml << 'EOF'
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rustdesk-hbbs
  namespace: rustdesk
  labels:
    app: rustdesk-hbbs
spec:
  replicas: 1
  selector:
    matchLabels:
      app: rustdesk-hbbs
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    metadata:
      labels:
        app: rustdesk-hbbs
    spec:
      containers:
      - name: hbbs
        image: rustdesk/rustdesk-server:latest
        command: ["hbbs", "-r", "your-domain.com:21116"]
        ports:
        - containerPort: 21114
          name: heartbeat
        - containerPort: 21115
          name: api
        - containerPort: 21116
          name: rendezvous
        - containerPort: 21118
          name: extra1
        volumeMounts:
        - name: data
          mountPath: /data
        env:
        - name: RUST_LOG
          valueFrom:
            configMapKeyRef:
              name: rustdesk-config
              key: RUST_LOG
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /status
            port: 21115
          initialDelaySeconds: 40
          periodSeconds: 30
          timeoutSeconds: 10
        readinessProbe:
          httpGet:
            path: /status
            port: 21115
          initialDelaySeconds: 20
          periodSeconds: 10
          timeoutSeconds: 5
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: rustdesk-data
EOF

# 应用配置
kubectl apply -f hbbs-deployment.yaml
```

### 3.5 创建 hbbr Deployment

```bash
cat > hbbr-deployment.yaml << 'EOF'
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rustdesk-hbbr
  namespace: rustdesk
  labels:
    app: rustdesk-hbbr
spec:
  replicas: 1
  selector:
    matchLabels:
      app: rustdesk-hbbr
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    metadata:
      labels:
        app: rustdesk-hbbr
    spec:
      containers:
      - name: hbbr
        image: rustdesk/rustdesk-server:latest
        command: ["hbbr"]
        ports:
        - containerPort: 21117
          name: relay
        - containerPort: 21119
          name: extra2
        volumeMounts:
        - name: data
          mountPath: /data
        env:
        - name: RUST_LOG
          valueFrom:
            configMapKeyRef:
              name: rustdesk-config
              key: RUST_LOG
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /status
            port: 21117
          initialDelaySeconds: 40
          periodSeconds: 30
          timeoutSeconds: 10
        readinessProbe:
          httpGet:
            path: /status
            port: 21117
          initialDelaySeconds: 20
          periodSeconds: 10
          timeoutSeconds: 5
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: rustdesk-data
EOF

# 应用配置
kubectl apply -f hbbr-deployment.yaml
```

### 3.6 创建 Service

```bash
cat > rustdesk-service.yaml << 'EOF'
apiVersion: v1
kind: Service
metadata:
  name: rustdesk-hbbs
  namespace: rustdesk
spec:
  type: NodePort
  selector:
    app: rustdesk-hbbs
  ports:
  - name: heartbeat
    port: 21114
    targetPort: 21114
    nodePort: 30114
  - name: api
    port: 21115
    targetPort: 21115
    nodePort: 30115
  - name: rendezvous-tcp
    port: 21116
    targetPort: 21116
    nodePort: 30116
  - name: extra1
    port: 21118
    targetPort: 21118
    nodePort: 30118
---
apiVersion: v1
kind: Service
metadata:
  name: rustdesk-hbbr
  namespace: rustdesk
spec:
  type: NodePort
  selector:
    app: rustdesk-hbbr
  ports:
  - name: relay
    port: 21117
    targetPort: 21117
    nodePort: 30117
  - name: extra2
    port: 21119
    targetPort: 21119
    nodePort: 30119
EOF

# 应用配置
kubectl apply -f rustdesk-service.yaml
```

### 3.7 创建 Ingress（可选）

```bash
cat > rustdesk-ingress.yaml << 'EOF'
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: rustdesk-ingress
  namespace: rustdesk
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/proxy-read-timeout: "300"
spec:
  tls:
  - hosts:
    - your-domain.com
    secretName: rustdesk-tls
  rules:
  - host: your-domain.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: rustdesk-hbbs
            port:
              number: 21115
EOF

# 应用配置
kubectl apply -f rustdesk-ingress.yaml
```

---

## 4. 商业版部署

### 4.1 创建商业版 Deployment

```bash
cat > rustdesk-pro-deployment.yaml << 'EOF'
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rustdesk-pro
  namespace: rustdesk
  labels:
    app: rustdesk-pro
spec:
  replicas: 1
  selector:
    matchLabels:
      app: rustdesk-pro
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 0
  template:
    metadata:
      labels:
        app: rustdesk-pro
    spec:
      containers:
      - name: rustdesk-pro
        image: rustdesk/rustdesk-pro-server:latest
        ports:
        - containerPort: 21114
          name: heartbeat
        - containerPort: 21115
          name: api
        - containerPort: 21116
          name: rendezvous
        - containerPort: 21117
          name: relay
        - containerPort: 21118
          name: extra1
        - containerPort: 21119
          name: extra2
        - containerPort: 8000
          name: web
        volumeMounts:
        - name: data
          mountPath: /data
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "1Gi"
            cpu: "1"
        livenessProbe:
          httpGet:
            path: /status
            port: 21115
          initialDelaySeconds: 60
          periodSeconds: 30
          timeoutSeconds: 10
        readinessProbe:
          httpGet:
            path: /status
            port: 21115
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: rustdesk-data
EOF

# 应用配置
kubectl apply -f rustdesk-pro-deployment.yaml
```

### 4.2 创建商业版 Service

```bash
cat > rustdesk-pro-service.yaml << 'EOF'
apiVersion: v1
kind: Service
metadata:
  name: rustdesk-pro
  namespace: rustdesk
spec:
  type: NodePort
  selector:
    app: rustdesk-pro
  ports:
  - name: heartbeat
    port: 21114
    targetPort: 21114
    nodePort: 30114
  - name: api
    port: 21115
    targetPort: 21115
    nodePort: 30115
  - name: rendezvous-tcp
    port: 21116
    targetPort: 21116
    nodePort: 30116
  - name: relay
    port: 21117
    targetPort: 21117
    nodePort: 30117
  - name: extra1
    port: 21118
    targetPort: 21118
    nodePort: 30118
  - name: extra2
    port: 21119
    targetPort: 21119
    nodePort: 30119
  - name: web
    port: 8000
    targetPort: 8000
    nodePort: 30080
EOF

# 应用配置
kubectl apply -f rustdesk-pro-service.yaml
```

---

## 5. 部署验证

### 5.1 检查 Pod 状态

```bash
# 查看所有 Pod
kubectl get pods -n rustdesk

# 查看 Pod 详细信息
kubectl describe pod rustdesk-hbbs-xxx -n rustdesk

# 查看 Pod 日志
kubectl logs rustdesk-hbbs-xxx -n rustdesk

# 查看 Pod 日志（实时）
kubectl logs -f rustdesk-hbbs-xxx -n rustdesk
```

### 5.2 检查 Service 状态

```bash
# 查看所有 Service
kubectl get svc -n rustdesk

# 查看 Service 详细信息
kubectl describe svc rustdesk-hbbs -n rustdesk
```

### 5.3 检查 Ingress 状态

```bash
# 查看 Ingress
kubectl get ingress -n rustdesk

# 查看 Ingress 详细信息
kubectl describe ingress rustdesk-ingress -n rustdesk
```

### 5.4 验证服务可用性

```bash
# 获取 NodePort 端口
NODE_PORT=$(kubectl get svc rustdesk-hbbs -n rustdesk -o jsonpath='{.spec.ports[1].nodePort}')
NODE_IP=$(kubectl get nodes -o jsonpath='{.items[0].status.addresses[0].address}')

# 测试 API
curl http://${NODE_IP}:${NODE_PORT}/status
```

### 5.5 获取公钥

```bash
# 获取 Pod 名称
POD_NAME=$(kubectl get pods -n rustdesk -l app=rustdesk-hbbs -o jsonpath='{.items[0].metadata.name}')

# 执行命令获取公钥
kubectl exec -n rustdesk $POD_NAME -- cat /data/id_ed25519.pub
```

---

## 6. 资源管理

### 6.1 扩展副本数

```bash
# 扩展 hbbs 副本数
kubectl scale deployment rustdesk-hbbs --replicas=2 -n rustdesk

# 查看副本状态
kubectl get deployment rustdesk-hbbs -n rustdesk
```

### 6.2 更新镜像

```bash
# 更新镜像
kubectl set image deployment/rustdesk-hbbs hbbs=rustdesk/rustdesk-server:1.2.3 -n rustdesk

# 查看更新状态
kubectl rollout status deployment/rustdesk-hbbs -n rustdesk
```

### 6.3 回滚更新

```bash
# 查看历史版本
kubectl rollout history deployment/rustdesk-hbbs -n rustdesk

# 回滚到上一版本
kubectl rollout undo deployment/rustdesk-hbbs -n rustdesk

# 回滚到指定版本
kubectl rollout undo deployment/rustdesk-hbbs --to-revision=2 -n rustdesk
```

### 6.4 备份数据

```bash
# 获取 Pod 名称
POD_NAME=$(kubectl get pods -n rustdesk -l app=rustdesk-hbbs -o jsonpath='{.items[0].metadata.name}')

# 创建备份
kubectl exec -n rustdesk $POD_NAME -- tar -czvf /data/backup-$(date +%Y%m%d).tar.gz /data

# 复制备份到本地
kubectl cp rustdesk/$POD_NAME:/data/backup-$(date +%Y%m%d).tar.gz ./backup.tar.gz
```

---

## 7. 故障排除

### 7.1 常见问题

| 问题 | 原因 | 解决方案 |
|------|------|----------|
| Pod 状态为 Pending | 资源不足或调度问题 | 检查节点资源和调度器日志 |
| Pod 状态为 CrashLoopBackOff | 应用启动失败 | 查看 Pod 日志 |
| Pod 状态为 ImagePullBackOff | 镜像拉取失败 | 检查镜像名称和网络 |
| Service 无法访问 | 网络策略或防火墙 | 检查网络策略和安全组 |

### 7.2 日志分析

```bash
# 查看 Pod 日志
kubectl logs rustdesk-hbbs-xxx -n rustdesk

# 查看最近日志
kubectl logs --tail=100 rustdesk-hbbs-xxx -n rustdesk

# 查看容器日志（多容器 Pod）
kubectl logs rustdesk-hbbs-xxx -c hbbs -n rustdesk
```

### 7.3 调试命令

```bash
# 进入 Pod
kubectl exec -it rustdesk-hbbs-xxx -n rustdesk -- bash

# 检查环境变量
kubectl exec rustdesk-hbbs-xxx -n rustdesk -- env

# 检查网络连通性
kubectl exec rustdesk-hbbs-xxx -n rustdesk -- ping rustdesk-hbbr
```

---

## 8. 版本兼容性

| RustDesk Server 版本 | Kubernetes 版本要求 | 状态 |
|----------------------|-------------------|------|
| 1.2.0+ | 1.20+ | 支持 |
| 1.1.0+ | 1.18+ | 支持 |

---

**文档版本**: v1.0  
**适用产品**: RustDesk Server Community & Pro  
**最后更新**: 2026-06-12

#!/bin/bash
# RustDesk Server 一键部署脚本
# 支持: Docker, Docker Compose, Podman, Podman Compose, Kubernetes, 二进制

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 配置变量
DOMAIN=""
VERSION="latest"
DATA_DIR="/var/lib/rustdesk-server"
CONTAINER_NAME="rustdesk-server"

# 打印标题
print_title() {
    echo -e "${BLUE}==============================================${NC}"
    echo -e "${BLUE}      RustDesk Server 一键部署脚本${NC}"
    echo -e "${BLUE}==============================================${NC}"
}

# 打印信息
print_info() {
    echo -e "${GREEN}[INFO] $1${NC}"
}

# 打印警告
print_warn() {
    echo -e "${YELLOW}[WARN] $1${NC}"
}

# 打印错误
print_error() {
    echo -e "${RED}[ERROR] $1${NC}"
}

# 检查命令是否存在
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# 获取操作系统类型
get_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    elif [ -f /etc/redhat-release ]; then
        echo "centos"
    else
        echo "unknown"
    fi
}

# 检查是否为 root
check_root() {
    if [ "$(id -u)" != "0" ]; then
        print_error "请以 root 用户运行此脚本"
        exit 1
    fi
}

# 获取用户输入
get_input() {
    read -p "$1: " input
    echo "$input"
}

# 安装依赖
install_dependencies() {
    local os=$(get_os)
    print_info "正在安装依赖..."
    
    case "$os" in
        "debian"|"ubuntu")
            apt update -y
            apt install -y curl wget tar unzip
            ;;
        "centos"|"rhel"|"fedora")
            dnf install -y curl wget tar unzip
            ;;
        *)
            print_warn "无法识别操作系统，依赖安装可能不完整"
            ;;
    esac
}

# Docker 部署
deploy_docker() {
    print_info "开始 Docker 部署..."
    
    # 检查 Docker 是否安装
    if ! command_exists docker; then
        print_info "安装 Docker..."
        curl -fsSL https://get.docker.com | sh
        systemctl enable docker
        systemctl start docker
    fi
    
    # 创建数据目录
    mkdir -p "$DATA_DIR"
    
    # 停止并删除旧容器
    docker stop "$CONTAINER_NAME"-hbbs 2>/dev/null || true
    docker rm "$CONTAINER_NAME"-hbbs 2>/dev/null || true
    docker stop "$CONTAINER_NAME"-hbbr 2>/dev/null || true
    docker rm "$CONTAINER_NAME"-hbbr 2>/dev/null || true
    
    # 启动 hbbs
    print_info "启动 hbbs 服务..."
    docker run -d \
        --name "$CONTAINER_NAME"-hbbs \
        -p 21114:21114 \
        -p 21115:21115 \
        -p 21116:21116 \
        -p 21116:21116/udp \
        -v "$DATA_DIR":/data \
        rustdesk/rustdesk-server:"$VERSION" \
        hbbs -r "$DOMAIN":21116
    
    # 启动 hbbr
    print_info "启动 hbbr 服务..."
    docker run -d \
        --name "$CONTAINER_NAME"-hbbr \
        -p 21117:21117 \
        -v "$DATA_DIR":/data \
        rustdesk/rustdesk-server:"$VERSION" \
        hbbr
    
    # 验证
    verify_deployment docker
}

# Docker Compose 部署
deploy_docker_compose() {
    print_info "开始 Docker Compose 部署..."
    
    # 检查 Docker Compose 是否安装
    if ! command_exists docker-compose; then
        print_info "安装 Docker Compose..."
        curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
        chmod +x /usr/local/bin/docker-compose
    fi
    
    # 创建部署目录
    mkdir -p /opt/rustdesk-server
    cd /opt/rustdesk-server
    
    # 创建 docker-compose.yml
    cat > docker-compose.yml << EOF
version: '3.8'
services:
  hbbs:
    image: rustdesk/rustdesk-server:$VERSION
    command: hbbs -r $DOMAIN:21116
    ports:
      - "21114:21114"
      - "21115:21115"
      - "21116:21116"
      - "21116:21116/udp"
    volumes:
      - rustdesk-data:/data
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:21115/status"]
      interval: 30s
      timeout: 10s
      retries: 3

  hbbr:
    image: rustdesk/rustdesk-server:$VERSION
    command: hbbr
    ports:
      - "21117:21117"
    volumes:
      - rustdesk-data:/data
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:21117/status"]
      interval: 30s
      timeout: 10s
      retries: 3

volumes:
  rustdesk-data:
EOF
    
    # 启动服务
    docker-compose up -d
    
    # 验证
    verify_deployment docker-compose
}

# Podman 部署
deploy_podman() {
    print_info "开始 Podman 部署..."
    
    # 检查 Podman 是否安装
    if ! command_exists podman; then
        print_info "安装 Podman..."
        local os=$(get_os)
        case "$os" in
            "debian"|"ubuntu")
                apt update -y
                apt install -y podman
                ;;
            "centos"|"rhel"|"fedora")
                dnf install -y podman
                ;;
        esac
    fi
    
    # 创建数据目录
    mkdir -p "$DATA_DIR"
    
    # 停止并删除旧容器
    podman stop "$CONTAINER_NAME"-hbbs 2>/dev/null || true
    podman rm "$CONTAINER_NAME"-hbbs 2>/dev/null || true
    podman stop "$CONTAINER_NAME"-hbbr 2>/dev/null || true
    podman rm "$CONTAINER_NAME"-hbbr 2>/dev/null || true
    
    # 创建 Podman 网络
    podman network create rustdesk-net 2>/dev/null || true
    
    # 启动 hbbs
    print_info "启动 hbbs 服务..."
    podman run -d \
        --name "$CONTAINER_NAME"-hbbs \
        --network rustdesk-net \
        -p 21114:21114 \
        -p 21115:21115 \
        -p 21116:21116 \
        -p 21116:21116/udp \
        -v "$DATA_DIR":/data:Z \
        rustdesk/rustdesk-server:"$VERSION" \
        hbbs -r "$DOMAIN":21116
    
    # 启动 hbbr
    print_info "启动 hbbr 服务..."
    podman run -d \
        --name "$CONTAINER_NAME"-hbbr \
        --network rustdesk-net \
        -p 21117:21117 \
        -v "$DATA_DIR":/data:Z \
        rustdesk/rustdesk-server:"$VERSION" \
        hbbr
    
    # 验证
    verify_deployment podman
}

# Podman Compose 部署
deploy_podman_compose() {
    print_info "开始 Podman Compose 部署..."
    
    # 检查 Podman Compose 是否安装
    if ! command_exists podman-compose; then
        print_info "安装 Podman Compose..."
        pip3 install podman-compose
    fi
    
    # 创建部署目录
    mkdir -p /opt/rustdesk-server
    cd /opt/rustdesk-server
    
    # 创建 docker-compose.yml
    cat > docker-compose.yml << EOF
version: '3.8'
services:
  hbbs:
    image: rustdesk/rustdesk-server:$VERSION
    command: hbbs -r $DOMAIN:21116
    ports:
      - "21114:21114"
      - "21115:21115"
      - "21116:21116"
      - "21116:21116/udp"
    volumes:
      - rustdesk-data:/data
    restart: unless-stopped

  hbbr:
    image: rustdesk/rustdesk-server:$VERSION
    command: hbbr
    ports:
      - "21117:21117"
    volumes:
      - rustdesk-data:/data
    restart: unless-stopped

volumes:
  rustdesk-data:
EOF
    
    # 启动服务
    podman-compose up -d
    
    # 验证
    verify_deployment podman-compose
}

# Kubernetes 部署
deploy_kubernetes() {
    print_info "开始 Kubernetes 部署..."
    
    # 检查 kubectl 是否安装
    if ! command_exists kubectl; then
        print_info "安装 kubectl..."
        curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
        chmod +x kubectl
        mv kubectl /usr/local/bin/
    fi
    
    # 检查集群连接
    if ! kubectl cluster-info 2>/dev/null; then
        print_error "无法连接到 Kubernetes 集群"
        exit 1
    fi
    
    # 创建命名空间
    kubectl create namespace rustdesk 2>/dev/null || true
    
    # 创建配置文件
    mkdir -p /opt/rustdesk-server/k8s
    cd /opt/rustdesk-server/k8s
    
    # 创建 ConfigMap
    cat > configmap.yaml << EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: rustdesk-config
  namespace: rustdesk
data:
  RUST_LOG: "info"
  RELAY_SERVER: "$DOMAIN:21116"
EOF
    
    # 创建 PVC
    cat > pvc.yaml << EOF
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
    
    # 创建 hbbs Deployment
    cat > hbbs-deployment.yaml << EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rustdesk-hbbs
  namespace: rustdesk
spec:
  replicas: 1
  selector:
    matchLabels:
      app: rustdesk-hbbs
  template:
    metadata:
      labels:
        app: rustdesk-hbbs
    spec:
      containers:
      - name: hbbs
        image: rustdesk/rustdesk-server:$VERSION
        command: ["hbbs", "-r", "$DOMAIN:21116"]
        ports:
        - containerPort: 21114
        - containerPort: 21115
        - containerPort: 21116
        volumeMounts:
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: rustdesk-data
EOF
    
    # 创建 hbbr Deployment
    cat > hbbr-deployment.yaml << EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: rustdesk-hbbr
  namespace: rustdesk
spec:
  replicas: 1
  selector:
    matchLabels:
      app: rustdesk-hbbr
  template:
    metadata:
      labels:
        app: rustdesk-hbbr
    spec:
      containers:
      - name: hbbr
        image: rustdesk/rustdesk-server:$VERSION
        command: ["hbbr"]
        ports:
        - containerPort: 21117
        volumeMounts:
        - name: data
          mountPath: /data
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
      volumes:
      - name: data
        persistentVolumeClaim:
          claimName: rustdesk-data
EOF
    
    # 创建 Service
    cat > service.yaml << EOF
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
    nodePort: 32114
  - name: api
    port: 21115
    nodePort: 32115
  - name: rendezvous
    port: 21116
    nodePort: 32116
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
    nodePort: 32117
EOF
    
    # 应用配置
    kubectl apply -f configmap.yaml
    kubectl apply -f pvc.yaml
    kubectl apply -f hbbs-deployment.yaml
    kubectl apply -f hbbr-deployment.yaml
    kubectl apply -f service.yaml
    
    # 验证
    verify_deployment kubernetes
}

# 二进制部署
deploy_binary() {
    print_info "开始二进制部署..."
    
    local os=$(get_os)
    
    # 创建目录结构
    mkdir -p /opt/rustdesk-server/bin
    mkdir -p /var/lib/rustdesk-server
    
    # 下载二进制文件
    cd /opt/rustdesk-server/bin
    curl -LO "https://github.com/rustdesk/rustdesk-server/releases/download/v${VERSION#v}/rustdesk-server-linux-amd64.tar.gz"
    tar -xzvf rustdesk-server-linux-amd64.tar.gz
    
    # 设置权限
    chmod +x hbbs hbbr
    
    # 创建专用用户
    useradd -r -s /usr/sbin/nologin -d /var/lib/rustdesk-server rustdesk 2>/dev/null || true
    chown -R rustdesk:rustdesk /var/lib/rustdesk-server
    chown -R rustdesk:rustdesk /opt/rustdesk-server
    
    # 创建 systemd 服务
    cat > /etc/systemd/system/rustdesk-hbbs.service << EOF
[Unit]
Description=RustDesk Rendezvous Server
After=network.target

[Service]
Type=simple
User=rustdesk
Group=rustdesk
WorkingDirectory=/var/lib/rustdesk-server
ExecStart=/opt/rustdesk-server/bin/hbbs -r $DOMAIN:21116
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
    
    cat > /etc/systemd/system/rustdesk-hbbr.service << EOF
[Unit]
Description=RustDesk Relay Server
After=network.target

[Service]
Type=simple
User=rustdesk
Group=rustdesk
WorkingDirectory=/var/lib/rustdesk-server
ExecStart=/opt/rustdesk-server/bin/hbbr
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
EOF
    
    # 启动服务
    systemctl daemon-reload
    systemctl enable rustdesk-hbbs rustdesk-hbbr
    systemctl start rustdesk-hbbs rustdesk-hbbr
    
    # 配置防火墙
    case "$os" in
        "debian"|"ubuntu")
            ufw allow 21114/tcp
            ufw allow 21115/tcp
            ufw allow 21116/tcp
            ufw allow 21116/udp
            ufw allow 21117/tcp
            ;;
        "centos"|"rhel"|"fedora")
            firewall-cmd --add-port=21114/tcp --permanent
            firewall-cmd --add-port=21115/tcp --permanent
            firewall-cmd --add-port=21116/tcp --permanent
            firewall-cmd --add-port=21116/udp --permanent
            firewall-cmd --add-port=21117/tcp --permanent
            firewall-cmd --reload
            ;;
    esac
    
    # 验证
    verify_deployment binary
}

# 验证部署
verify_deployment() {
    local method=$1
    print_info "正在验证部署..."
    
    case "$method" in
        "docker")
            sleep 10
            docker ps | grep rustdesk
            ;;
        "docker-compose")
            sleep 10
            docker-compose ps
            ;;
        "podman")
            sleep 10
            podman ps | grep rustdesk
            ;;
        "podman-compose")
            sleep 10
            podman-compose ps
            ;;
        "kubernetes")
            sleep 30
            kubectl get pods -n rustdesk
            ;;
        "binary")
            sleep 5
            systemctl status rustdesk-hbbs
            ;;
    esac
    
    # 获取公钥
    print_info "获取服务器公钥..."
    if [ -f "$DATA_DIR/id_ed25519.pub" ]; then
        cat "$DATA_DIR/id_ed25519.pub"
    elif [ "$method" = "kubernetes" ]; then
        kubectl exec -n rustdesk $(kubectl get pods -n rustdesk -l app=rustdesk-hbbs -o name | head -n1) -- cat /data/id_ed25519.pub
    fi
    
    print_info "部署完成！"
    print_info "服务器配置信息:"
    print_info "  Relay Server: $DOMAIN:21116"
    print_info "  公钥已显示在上方，请在客户端配置中使用"
}

# 显示菜单
show_menu() {
    echo -e "\n${YELLOW}请选择部署方式:${NC}"
    echo "1. Docker 部署"
    echo "2. Docker Compose 部署"
    echo "3. Podman 部署"
    echo "4. Podman Compose 部署"
    echo "5. Kubernetes 部署"
    echo "6. 二进制部署"
    echo "7. 退出"
    read -p "请输入选项 (1-7): " choice
    
    case "$choice" in
        "1")
            deploy_docker
            ;;
        "2")
            deploy_docker_compose
            ;;
        "3")
            deploy_podman
            ;;
        "4")
            deploy_podman_compose
            ;;
        "5")
            deploy_kubernetes
            ;;
        "6")
            deploy_binary
            ;;
        "7")
            print_info "退出脚本"
            exit 0
            ;;
        *)
            print_error "无效选项"
            show_menu
            ;;
    esac
}

# 主函数
main() {
    print_title
    
    # 检查 root
    check_root
    
    # 获取配置
    DOMAIN=$(get_input "请输入服务器域名或IP地址")
    VERSION=$(get_input "请输入版本号 (默认 latest)")
    VERSION=${VERSION:-latest}
    
    # 安装依赖
    install_dependencies
    
    # 显示菜单
    show_menu
}

# 启动脚本
main "$@"

## kubernetes 本地架設流程

- 1. 安裝必要工具
    - kind or minikube等本地開發工具
    - docker
    - kubectl

- 2. 建置docker images
```bash
docker build -t notify-grpc-server:latest .
```

- 3. 建立群集
```bash
kind create cluster --config=kind-config.yaml
```

- 4. 載入image
```bash
kind load docker-image notify-grpc-server:latest
```

- 5. 建立 env
```bash
kubectl create configmap notify-server-env --from-env-file=.env
```
注意：如果想要使用本地的rabbitmq 或其他服務 需要將host或port轉成 自己本地的服務 並開 `service` 掛載椄口

- 6. 部署
```bash
kubectl apply -f k8s/
```


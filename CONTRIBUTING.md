# 贡献指南

主要代码位于 `rust-backend/`。

## 本地开发

```bash
cd rust-backend
cargo fmt
cargo test
cargo run
```

默认服务地址：

- `http://127.0.0.1:8787/`

## 提交前检查

请至少保证以下命令通过：

```bash
cd rust-backend
cargo fmt --check
cargo test
```

## 变更范围

- 页面层与接口实现：`rust-backend/src/`
- 静态资源：`public/`
- 容器与部署：根目录 `Dockerfile`、`docker-compose.yml`

## 问题反馈

- Issue：<https://github.com/KuekHaoYang/KVideo/issues>

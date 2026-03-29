# RVideo

实际服务代码在 `rust-backend/`，`public/` 下的静态资源会被 Rust 服务直接提供。

## 本地运行

```bash
cd rust-backend
cargo run
```

默认地址：

- `http://127.0.0.1:3000/`
- `http://127.0.0.1:3000/login`
- `http://127.0.0.1:3000/settings`
- `http://127.0.0.1:3000/premium`
- `http://127.0.0.1:3000/player?id=...&source=...`

默认管理员：

- 用户名：`admin`
- 密码：`Admin@1234`

## Docker

根目录已经收口成纯 Rust 版：

```bash
docker compose up -d
```

会启动：

- `rvideo`：Rust 服务
- `mysql`：MySQL 8

服务默认监听 `3000` 端口。

## 订阅配置

订阅源通过环境变量配置：

- `SUBSCRIPTION_SOURCES`

本地运行时可以这样传入：

```bash
cd rust-backend
SUBSCRIPTION_SOURCES="https://example.com/a.txt,https://example.com/b.json" cargo run
```

使用 `docker-compose` 时，在根目录 `docker-compose.yml` 的 `rvideo.environment` 里添加：

```yaml
- SUBSCRIPTION_SOURCES=https://example.com/a.txt,https://example.com/b.json
```

如果只有一个订阅地址，直接写一个即可。多个地址用英文逗号分隔。

## 项目结构

- `rust-backend/`：Axum + MySQL 的完整服务与页面层
- `public/`：`manifest.json`、`sw.js`、图标和占位资源

## 验证

当前仓库以 Rust 测试为准：

```bash
cd rust-backend
cargo fmt --check
cargo test
```

## 许可证

本项目采用 `Apache-2.0` 许可证，见 `LICENSE`。

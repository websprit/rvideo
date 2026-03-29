# Rust Backend

这是 RVideo 的 Rust 服务实现，覆盖鉴权、用户管理、数据同步、搜索流式接口、详情、代理、豆瓣接口与高级内容接口。

## 运行

```bash
cd rust-backend
cargo run
```

默认监听 `http://127.0.0.1:3000`。

除了 API 以外，服务本身也直接提供完整页面层：

- `http://127.0.0.1:3000/`：首页搜索，支持 SSE 流式搜索结果、搜索历史、来源/类型徽标筛选、普通/分组视图、排序切换、豆瓣推荐，以及搜索结果直接进入播放器并把分组结果里的组内来源带入播放器
- `http://127.0.0.1:3000/login`
- `http://127.0.0.1:3000/settings`：支持常用设置、观看历史 / 搜索历史开关、访问控制配置、结构化线路管理、源新增/编辑、订阅导入与同步、JSON 编辑、密码修改，以及清空已同步数据 / 清除所有数据
- `http://127.0.0.1:3000/premium`：支持 Premium 搜索与内容浏览双态、受全局搜索历史开关控制的 Premium 搜索历史、来源/类型徽标筛选、普通/分组视图、排序切换，以及搜索结果直接进入播放器并把分组结果里的组内来源带入播放器
- `http://127.0.0.1:3000/premium/settings`：支持结构化高级源管理和 JSON 编辑
- `http://127.0.0.1:3000/detail?id=...&source=...`
- `http://127.0.0.1:3000/player?id=...&source=...`：支持 HLS 播放、切集、来源切换、失败时自动切换下一来源、按片名记忆来源偏好、代理切换、按设置同步观看历史、收藏切换、倍速、续播恢复、自动跳过片头/片尾、广告过滤、播放器内偏好调整、页面 / 原始 / 代理 / 当前播放链接复制、全屏、画中画、Google Cast / 系统投屏 / 系统分享、复制播放地址、键盘快捷键，以及内嵌历史/收藏侧栏
- `http://127.0.0.1:3000/admin`

这些页面直接运行在服务内，继续使用同一套 JWT Cookie 和 `/api/*` 契约。页面壳会在启动时同步当前用户、环境订阅配置以及服务端用户数据到浏览器存储，并自动把启用中的订阅配置展开成 `sources / premiumSources`；本地关键数据变更也会自动写回服务端。同时也接入了 manifest、Service Worker 注册、图标静态资源、滚动位置记忆、环境广告关键词注入，以及基于 `settings.passwordAccess/accessPasswords` 或 `ACCESS_PASSWORD` 的访问密码门禁。当前已覆盖的核心链路包括：登录、首页搜索、搜索历史、首页来源/类型筛选、首页热门功能快捷入口、首页收藏/历史快速访问、首页侧栏批量管理、撤销、已选条目复制/分享/导出、分享包、分享链接、合并分享链接、系统分享、命名快照保存/重命名/克隆/恢复/合并/删除、快照分享包、快照分享链接、快照合并分享链接、快照导出/覆盖导入/合并导入、导入导出、实时筛选与排序持久化、分组结果视图、搜索结果直达播放器、分组来源透传到播放器、豆瓣推荐浏览、Premium 搜索与内容浏览、Premium 搜索历史、Premium 来源/类型筛选、Premium 分组结果视图、Premium 搜索结果直达播放器、Premium 分组来源透传到播放器、Premium 侧栏批量管理、撤销、已选条目复制/分享/导出、分享包、分享链接、合并分享链接、系统分享、命名快照保存/重命名/克隆/恢复/合并/删除、快照分享包、快照分享链接、快照合并分享链接、快照导出/覆盖导入/合并导入、导入导出、实时筛选与排序持久化、Premium 标签管理、标签导出/覆盖导入/合并导入、标签分享包、标签分享链接、标签合并分享链接、详情查看、播放器、HLS 主播放链路、来源切换、失败时自动切换下一来源、按片名记忆来源偏好、来源诊断复制与失败轨迹清理、重试与代理诊断、一键强制直连/代理、重置重试状态、播放诊断 JSON 复制与导出、播放书签保存/跳转/删除/清空、书签时间点链接复制、收藏联动、访问控制配置、访问密码门禁、广告过滤、播放器内偏好调整、页面/原始/代理/当前播放链接复制、设置、结构化线路管理、源新增/编辑、订阅导入与同步、清空已同步数据、清除所有数据、Premium 设置、管理员用户管理。

## 环境变量

- `HOST`：默认 `0.0.0.0`
- `PORT`：默认 `3000`
- `AUTH_SECRET`：JWT 密钥
- `MYSQL_HOST` / `MYSQL_PORT` / `MYSQL_USER` / `MYSQL_PASSWORD` / `MYSQL_DATABASE`
- `SUBSCRIPTION_SOURCES` 或 `NEXT_PUBLIC_SUBSCRIPTION_SOURCES`
- `AD_KEYWORDS_FILE`
- `AD_KEYWORDS` 或 `NEXT_PUBLIC_AD_KEYWORDS`
- `ACCESS_PASSWORD`：全局访问密码，启用后会通过 `kvideo_access_granted` Cookie 维持解锁状态
- `PERSIST_PASSWORD`：默认 `true`，设为 `false` 时访问密码只在当前浏览器会话内有效

## 订阅配置

订阅源通过 `SUBSCRIPTION_SOURCES` 环境变量传入。

示例：

```bash
SUBSCRIPTION_SOURCES="https://example.com/a.txt,https://example.com/b.json" cargo run
```

如果用 `docker-compose`，就在根目录 `docker-compose.yml` 里给 `rvideo.environment` 增加：

```yaml
- SUBSCRIPTION_SOURCES=https://example.com/a.txt,https://example.com/b.json
```

多个订阅地址用英文逗号分隔。

## Docker Compose

在仓库根目录运行：

```bash
docker compose up -d
```

该编排会启动 Rust 服务和 MySQL，服务地址为 `http://127.0.0.1:3000`。

## 许可证

本项目采用 `Apache-2.0` 许可证，见根目录 `LICENSE`。

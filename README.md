# VectorEngine TokenUse

批量查询 [VectorEngine](https://api.vectorengine.ai) token 用量的本地工具。

浏览器直接访问 `https://api.vectorengine.ai` 会遇到跨域限制。本项目通过一个本地代理解决此问题,粘贴一组 token 后批量展示每个 token 的余额、当日用量、最近请求明细等。

## 特性

- 多 token 批量粘贴、并发查询(可调并发数 1-20)
- 显示:令牌名称、总额、已用、剩余、使用率
- 每个 token 的最近请求明细(模型、提示/补全 token、用时)
- 日期区间查询(开始 / 结束)
- Token 列表自动保存到浏览器 localStorage,关闭页面不丢
- 数据查询结果实时刷新单行
- 一键重置(清结果 + 日期回到今天 + 控制项恢复默认)
- 明/暗模式自动适配

## 快速开始

### 1. 启动代理(选 Rust 版,推荐)

```bash
cd proxy-rust
cargo run --release
```

看到 `[proxy] VectorEngine proxy listening on http://127.0.0.1:8765` 即就绪。

### 2. 打开页面

浏览器访问 <http://localhost:8765/>

### 3. 粘贴 token,点「查询全部」

在弹出的「Token 列表」弹框里每行粘贴一个 token,点保存 → 主表格立即出现所有 token 行(状态"未查询")→ 点「查询全部」批量拉数据。

## 目录结构

```
useToken/
├── index.html              # 前端单文件应用
├── proxy-rust/             # Rust 版代理(已嵌入 index.html)
│   ├── Cargo.toml
│   ├── build.rs
│   ├── src/main.rs
│   └── README.md           # Rust 代理详细文档
└── .github/
    └── workflows/
        └── build.yml       # 多平台打包 CI
```

## 单文件分发

整个项目最终产物是**单个可执行文件**(已内嵌 `index.html`),无任何外部依赖:

- macOS / Linux: `vectorengine-proxy`
- Windows: `vectorengine-proxy.exe`

直接双击即可,启动后会自动打开浏览器。

## 多平台打包

推到 GitHub 后,`.github/workflows/build.yml` 会自动在 4 个 target 上编译 release 二进制:

- Linux x86_64
- macOS Intel
- macOS Apple Silicon
- Windows x86_64

在 GitHub repo 的 **Actions** 页可以下载所有产物。

### 本地交叉编译 Windows 包

详见 [proxy-rust/README.md](./proxy-rust/README.md#本地交叉编译-windows-包)。

## API 端点(代理)

| 本地路径 | 上游 |
|---|---|
| `GET /health` | 健康检查 |
| `GET /` 或 `/index.html` | 当前目录下的 `index.html` |
| `GET /api/usage?start_date=&end_date=` | `/v1/dashboard/billing/usage` |
| `GET /api/subscription` | `/v1/dashboard/billing/subscription` |
| `GET /api/log/token?key=&page=&page_size=&start_timestamp=&end_timestamp=` | `/api/log/token` |

## 注意事项

- `total_usage` 接口返回的是「美分」单位,前端已统一除以 100 换算成美元显示(常量 `USAGE_DIVISOR = 100` 在 `index.html` 中,如需调整改这一行)。
- `access_until == 0` 表示「永不过期」。
- 所有 token 全部保存在浏览器 localStorage,不上传任何服务器。

## ☁️ 在 GitHub Codespace 上跑(零本地环境)

仓库带 `.devcontainer/devcontainer.json`,可以一键在云端跑起来。**注意:这种模式下 token 会经过 GitHub 的服务器,适合临时演示/分享,不适合生产环境长期使用**。

启动步骤:

1. 打开 <https://github.com/KaenFelix/vectorengineUseToken>
2. 点 **Code** → **Codespaces** → **Create codespace on main**
3. 等 2-3 分钟(首次冷启动 + cargo build)
4. Codespace 自动启动代理,8765 端口自动转发,浏览器自动打开 <https://xxx-8765.app.github.dev/>

也可以手动启动:

```bash
cd proxy-rust
./target/release/vectorengine-proxy
```

免费额度:每月 120 core-hour,2 核 instance 约 60 小时,够个人临时使用。

## 三种使用方式对比

| 方式 | 适合场景 | token 安全性 |
|---|---|---|
| 下载 binary 双击(默认) | 自用、长期 | ⭐⭐⭐ token 不出本机 |
| GitHub Codespace | 给朋友演示、临时分享 | ⭐ token 经 GitHub 服务器 |
| 下载 Release 附件 | 装到别人机器上 | ⭐⭐⭐ 跟本地一样 |
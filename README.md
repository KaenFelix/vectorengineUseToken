# VectorEngine TokenUse

批量查询 [VectorEngine](https://api.vectorengine.ai) token 用量的工具。

浏览器直接访问 `https://api.vectorengine.ai` 会遇到跨域限制。本项目通过一个本地代理解决,粘贴一组 token 后批量展示每个 token 的余额、当日用量、最近请求明细等。

## 特性

- 多 token 批量粘贴、并发查询(并发数 1-20 可调)
- 每个 token 显示:令牌名称、总额、已用、剩余、使用率
- 每个 token 的最近请求明细(时间、模型、提示/补全 token、用时)
- 日期区间查询(开始 / 结束)
- 默认连本机代理,常用情况开箱即用
- 代理地址可配置(适合远程代理场景)
- Token 列表 + 代理地址都自动保存到浏览器 localStorage
- 数据查询结果实时刷新单行
- 一键重置(清结果 + 日期回到今天 + 控制项恢复默认)
- 明/暗模式自动适配

## 快速开始

### 方式 A:下载 binary 双击(最简单,token 不出本机)

去 [Releases](https://github.com/KaenFelix/vectorengineUseToken/releases) 下载对应平台的 binary:

- macOS / Linux: `vectorengine-proxy`
- Windows: `vectorengine-proxy.exe`

双击运行,**自动打开浏览器** 到 `http://localhost:8765/`。无需任何配置,粘贴 token 即可用。

### 方式 B:用 GitHub Pages 公开版(零下载)

1. 在自己电脑上跑代理(方式 A 的步骤 1-3,或 `cargo run --release`)
2. 浏览器打开 <https://KaenFelix.github.io/vectorengineUseToken/>
3. 页面**默认就连 `http://localhost:8765`**,直接粘贴 token 即可查询
4. 想换代理地址(例如远程代理)?点「Token 列表」→ 弹框底部「⚙ 代理设置」

> GitHub Pages 只托管静态前端,**拿不到你的 token**。token 只经过你自己的代理再发到 VectorEngine API。

### 方式 C:在 GitHub Codespaces 上跑(零本地环境)

适合给别人演示或自己临时用。**注意:这种模式下 token 会经过 GitHub 的服务器**。

1. 打开 <https://github.com/KaenFelix/vectorengineUseToken>
2. 点 **Code** → **Codespaces** → **Create codespace on main**
3. 等 2-3 分钟(首次冷启动 + cargo build)
4. 自动启动代理,8765 端口自动转发,浏览器自动打开

免费额度:每月 120 core-hour,2 核 instance 约 60 小时。

## 从源码启动代理

```bash
# 从源码
cd proxy-rust && cargo run --release

# 用下载的 binary
./vectorengine-proxy
```

看到 `[proxy] VectorEngine proxy listening on http://127.0.0.1:8765` 即就绪,浏览器自动打开。

## 使用流程

1. 打开页面(任选方式 A/B/C)
2. 点「Token 列表」粘贴你的 token(每行一个)
3. 点「查询全部」
4. 表格显示每个 token 的余额/用量,日志行展开看最近请求
5. 想换日期区间?改顶部日期组件 + 「查询全部」
6. 想重置?点「重置」按钮

## 目录结构

```
useToken/
├── index.html              # 前端单文件应用(也可部署到 GitHub Pages)
├── proxy-rust/             # Rust 版代理(已嵌入 index.html,单文件分发)
│   ├── Cargo.toml
│   ├── build.rs
│   ├── src/main.rs
│   └── README.md           # Rust 代理详细文档
├── .devcontainer/
│   └── devcontainer.json   # GitHub Codespaces 配置
└── .github/
    └── workflows/
        ├── build.yml       # 多平台打包 + Release
        └── pages.yml       # 部署 index.html 到 GitHub Pages
```

## 单文件分发原理

Rust 代理用 `include_bytes!` 在编译时把 `index.html` 嵌入 binary。运行时:

1. bind 8765 端口
2. 调用 `open` / `xdg-open` / `rundll32` 打开默认浏览器
3. `GET /` 和 `GET /index.html` 都返回内嵌的 HTML
4. 其他请求转发到 VectorEngine API

整个产物是单个 3MB 二进制,无任何外部文件依赖。

## 多平台打包 / Release

`.github/workflows/build.yml` 自动在 4 个 target 编译 release:

- Linux x86_64 (`x86_64-unknown-linux-gnu`)
- macOS Intel (`x86_64-apple-darwin`)
- macOS Apple Silicon (`aarch64-apple-darwin`)
- Windows (`x86_64-pc-windows-msvc`,静态链接 CRT,独立 .exe)

push tag(如 `v1.0.0`)→ 自动发布到 [Releases](https://github.com/KaenFelix/vectorengineUseToken/releases) 页面。

## GitHub Pages 部署

`.github/workflows/pages.yml` 在 push 到 main 时自动部署 `index.html` 到 GitHub Pages。

**首次启用需要手动操作一次:**

1. 打开 <https://github.com/KaenFelix/vectorengineUseToken/settings/pages>
2. **Source** 选 **GitHub Actions**(不是 "Deploy from a branch")
3. 保存

后续 push 到 main 自动部署。**首次部署后 DNS 生效需 5-10 分钟,期间访问可能 404**。

## 四种使用方式对比

| 方式 | 适合场景 | token 流向 | 上手难度 |
|---|---|---|---|
| 下载 binary 双击 | 自用、长期 | 不出本机 | ⭐ 最简单 |
| GitHub Pages + 本地代理 | 想用公开 URL,自己跑代理 | 经本地代理 | ⭐⭐ |
| GitHub Pages + 远程代理 | 共享一台代理给多人 | 经远程服务器(隐私差) | ⭐⭐ |
| GitHub Codespaces | 给朋友演示、临时分享 | 经 GitHub 服务器 | ⭐⭐ |

## API 端点(代理)

| 本地路径 | 上游 |
|---|---|
| `GET /health` | 健康检查 |
| `GET /` 或 `/index.html` | 内嵌的 index.html |
| `GET /api/usage?start_date=&end_date=` | `/v1/dashboard/billing/usage` |
| `GET /api/subscription` | `/v1/dashboard/billing/subscription` |
| `GET /api/log/token?key=&page=&page_size=&start_timestamp=&end_timestamp=` | `/api/log/token` |

## 注意事项

- `total_usage` 接口返回的是「美分」单位,前端已统一除以 100 换算成美元显示(常量 `USAGE_DIVISOR = 100` 在 `index.html` 中,如需调整改这一行)。
- `access_until == 0` 表示「永不过期」。
- 所有 token 全部保存在浏览器 localStorage,不上传任何服务器。
# VectorEngine Token Proxy (Rust)

本地 HTTP 代理,为浏览器前端(<code>../index.html</code>)解决跨域问题。

- **单一二进制**:**已内嵌 `index.html`**,拷一个文件到目标机器就能跑,无任何外部文件依赖
- **跨平台分发**:任意同架构机器直接双击
- **rustls TLS**:不依赖系统 OpenSSL,跨平台稳定
- **自动开浏览器**:启动后自动调起默认浏览器(macOS / Linux / Windows),无需复制 URL
- **Windows 静态链接**:`+crt-static` 让 .exe 不依赖 VC++ runtime,任何 Win10/11 直接跑

## 编译

需要 Rust 1.70+(`rustup update stable`)。

```bash
cargo build --release
```

产物:`target/release/vectorengine-proxy`(~3 MB)

## 运行

代理启动流程:
1. bind 8765 端口
2. 自动调起系统命令打开默认浏览器
3. 等待请求并 serve 内嵌的 HTML / 转发 API

```bash
# 从源码
cargo run --release

# 用下载的 binary
./vectorengine-proxy
```

按 `Ctrl-C` 停止。

启动日志示例:

```
[proxy] serving EMBEDDED index.html (single-file build)
[proxy] VectorEngine proxy listening on http://127.0.0.1:8765
[proxy] opened browser at http://127.0.0.1:8765
```

## 本地交叉编译

### Windows 包(macOS 上)

```bash
brew install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

产物:`target/x86_64-pc-windows-gnu/release/vectorengine-proxy.exe`

> ⚠️ `-gnu` 版本依赖运行时,若对方 Windows 缺 VC++ 运行库,改用 GitHub Actions 编译 `-msvc` 版本(自带 MSVC 链接器)。本地 Cargo.toml 已配 `[target.x86_64-pc-windows-msvc] rustflags = ["-C", "target-feature=+crt-static"]` 让 MSVC CRT 静态链接,目标机器无需装运行时。

### Linux 包

```bash
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
```

> macOS 交叉编译 Linux 通常需要装额外 linker;推荐用 GitHub Actions。

### macOS 跨架构(M 系列 → Intel)

```bash
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

## GitHub Actions 多平台打包

项目根的 `.github/workflows/build.yml` 会在 push 时自动编译 4 个平台的 release 版本,产物在 GitHub Actions 的 Artifacts 区域下载。

| Runner | Target | 产物 |
|---|---|---|
| ubuntu-latest | `x86_64-unknown-linux-gnu` | `vectorengine-proxy` |
| macos-latest | `x86_64-apple-darwin` | `vectorengine-proxy`(macOS Intel) |
| macos-latest | `aarch64-apple-darwin` | `vectorengine-proxy`(macOS Apple Silicon) |
| windows-latest | `x86_64-pc-windows-msvc` | `vectorengine-proxy.exe`(静态链接) |

**每个 artifact 只包含 1 个 binary**(`index.html` 已嵌入,无需额外文件),解压后双击即可。

`push tag` (如 `v1.0.0`) 还会触发 release job,把 4 个 binary 一起发布到 GitHub Releases 页面。

## 开发

```bash
cargo run            # debug 模式编译并运行
cargo check          # 只检查不编译(最快)
cargo watch -x run   # 保存代码自动重启(需 cargo install cargo-watch)
cargo clippy         # Lint
cargo fmt            # 格式化
```

修改 `index.html` 不会触发 binary 重编译 — `include_bytes!` 不知道,需要手动 `cargo build` 重新嵌入。

## 路由对照表

| 本地路径 | 上游 | 说明 |
|---|---|---|
| `GET /health` | - | 健康检查,返回 `{ok:true}` |
| `GET /` 或 `/index.html` | 内嵌的 index.html | 单文件分发 |
| `GET /api/usage?start_date=&end_date=` | `/v1/dashboard/billing/usage` | 透传 Authorization 头 |
| `GET /api/subscription` | `/v1/dashboard/billing/subscription` | 透传 Authorization 头 |
| `GET /api/log/token?key=&page=&page_size=&start_timestamp=&end_timestamp=` | `/api/log/token` | token 走 `key` 参数,无 Authorization 头 |
| 其他 | 404 | 不再 serve 外部静态文件,所有内容都在内嵌 HTML 里 |

## 依赖说明

| Crate | 作用 |
|---|---|
| `axum` 0.7 | Web 框架,基于 `tower` |
| `tokio` | 异步运行时 |
| `reqwest` 0.12 + `rustls-tls` | HTTP 客户端,纯 Rust TLS,无 OpenSSL 依赖 |
| `tower-http` | `CorsLayer` 中间件(允许任意 origin) |
| `serde` / `serde_json` | JSON 序列化 |

`build.rs` 在编译时校验 `../index.html` 存在,确保 `include_bytes!` 不失效。

## 常见问题

### 端口被占用

```
lsof -ti:8765 | xargs kill -9
```

### Linux 没浏览器,启动报错吗?

不会。`open_browser` 失败时只打印一行 `(browser auto-open skipped: <原因>)`,主流程继续 serve,curl/直连仍可用。

### 编译失败

```bash
cargo clean && cargo build --release
```

### 改 `index.html` 后 binary 没更新?

`include_bytes!` 不会感知到文件变化,手动 `touch` 一下 `src/main.rs` 或 `cargo clean` 后再 build。

### 改 `USAGE_DIVISOR` 之类的常量

所有金额换算逻辑都在 `../index.html` 顶部(JS 部分),Rust 代理不参与。
# VectorEngine Token Proxy (Rust)

本地 HTTP 代理,为浏览器前端(<code>../index.html</code>)解决跨域问题。功能与 <code>../proxy.py</code> 等价,优势是:

- **单一二进制**:无需 Python / Node / 任何运行时
- **跨平台分发**:拷一个文件到目标机器就能跑
- **rustls TLS**:不依赖系统 OpenSSL,跨平台稳定

## 编译

需要 Rust 1.70+(`rustup update stable`)。

```bash
cargo build --release
```

产物:`target/release/vectorengine-proxy`(~3 MB)

## 运行

代理会自动从 cwd、binary 所在目录、以及它们的祖先目录中查找 `index.html`,所以从哪个目录启动都行,只要项目里能向上找到 `index.html`。

```bash
# 方式 A:从本目录启动,代理自动向上找到 useToken/index.html
cargo run --release

# 方式 B:从 useToken/ 目录启动
cd ..
./proxy-rust/target/release/vectorengine-proxy
```

启动后浏览器访问 <http://127.0.0.1:8765/>。按 `Ctrl-C` 停止。

启动日志示例:

```
[proxy] serving static files from: /Volumes/970EVOP/ClifeCode/useToken
[proxy] VectorEngine proxy listening on http://127.0.0.1:8765
[proxy] 在浏览器打开上面的地址即可。Ctrl-C 停止。
```

## 本地交叉编译

### Windows 包(macOS 上)

```bash
brew install mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

产物:`target/x86_64-pc-windows-gnu/release/vectorengine-proxy.exe`

> ⚠️ `-gnu` 版本依赖运行时,若对方 Windows 缺 VC++ 运行库,改用 GitHub Actions 编译 `-msvc` 版本(自带 MSVC 链接器)。

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
| ubuntu-latest | `x86_64-unknown-linux-gnu` | `vectorengine-proxy-linux` |
| macos-latest | `x86_64-apple-darwin` | `vectorengine-proxy-macos-intel` |
| macos-latest | `aarch64-apple-darwin` | `vectorengine-proxy-macos-arm` |
| windows-latest | `x86_64-pc-windows-msvc` | `vectorengine-proxy-windows.exe` |

每个 artifact 包含 `vectorengine-proxy`(或 `.exe`)和 `index.html`,解压后双击 binary 即可,浏览器访问 <http://localhost:8765/>。

## 开发

```bash
cargo run            # debug 模式编译并运行
cargo check          # 只检查不编译(最快)
cargo watch -x run   # 保存代码自动重启(需 cargo install cargo-watch)
cargo clippy         # Lint
cargo fmt            # 格式化
```

修改 `index.html` **不需要重启代理**,代理每次请求都重新读磁盘文件。

## 路由对照表

| 本地路径 | 上游 | 说明 |
|---|---|---|
| `GET /health` | - | 健康检查,返回 `{ok:true}` |
| `GET /` 或 `/index.html` | 当前目录 `index.html` | 静态文件 |
| 其他 GET(非 `/api/*`) | 当前目录静态文件 | 自动 fallback,如 favicon.ico |
| `GET /api/usage?start_date=&end_date=` | `/v1/dashboard/billing/usage` | 透传 Authorization 头 |
| `GET /api/subscription` | `/v1/dashboard/billing/subscription` | 透传 Authorization 头 |
| `GET /api/log/token?key=&page=&page_size=&start_timestamp=&end_timestamp=` | `/api/log/token` | token 走 `key` 参数,无 Authorization 头 |

## 依赖说明

| Crate | 作用 |
|---|---|
| `axum` 0.7 | Web 框架,基于 `tower` |
| `tokio` | 异步运行时 |
| `reqwest` 0.12 + `rustls-tls` | HTTP 客户端,纯 Rust TLS,无 OpenSSL 依赖 |
| `tower-http` | `CorsLayer` + `ServeDir` 中间件 |
| `serde` / `serde_json` | JSON 序列化 |

## 常见问题

### 端口被占用

```
lsof -ti:8765 | xargs kill -9
```

### `index.html` 找不到

启动日志 `serving static files from: ...` 显示代理找到的静态目录。如果路径不对,确认:

1. cwd 或 binary 所在目录的祖先链中有 `index.html`
2. 或者把 `index.html` 复制到 binary 旁边

### 编译失败

```bash
cargo clean && cargo build --release
```

### 改 `USAGE_DIVISOR` 之类的常量

所有金额换算逻辑都在 `../index.html` 顶部(JS 部分),Rust 代理不参与。
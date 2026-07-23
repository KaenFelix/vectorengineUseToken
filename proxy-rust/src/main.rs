//! VectorEngine Token 本地代理 (Rust)
//!
//! 等价于 `proxy.py`,为浏览器解决跨域问题。
//! release 编译产物是单一二进制,无运行时依赖。

use axum::{
    body::Body,
    extract::Query,
    http::{HeaderMap, Response, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::{cors::CorsLayer, services::ServeDir};

const LISTEN_HOST: &str = "127.0.0.1";
const LISTEN_PORT: u16 = 8765;
const TARGET: &str = "https://api.vectorengine.ai";

// ---------- Handlers ----------

async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"ok": true, "service": "vectorengine-proxy"}))
}

#[derive(Deserialize)]
struct UsageQuery {
    start_date: String,
    end_date: String,
}

async fn api_usage(
    Query(q): Query<UsageQuery>,
    headers: HeaderMap,
) -> Response<Body> {
    let target = format!(
        "{}/v1/dashboard/billing/usage?start_date={}&end_date={}",
        TARGET, q.start_date, q.end_date
    );
    proxy(target, headers, true).await
}

async fn api_subscription(headers: HeaderMap) -> Response<Body> {
    let target = format!("{}/v1/dashboard/billing/subscription", TARGET);
    proxy(target, headers, true).await
}

#[derive(Deserialize)]
struct LogQuery {
    key: String,
    page: Option<u32>,
    page_size: Option<u32>,
    start_timestamp: Option<i64>,
    end_timestamp: Option<i64>,
}

async fn api_log(Query(q): Query<LogQuery>) -> Response<Body> {
    let target = format!(
        "{}/api/log/token?key={}&page={}&page_size={}&start_timestamp={}&end_timestamp={}",
        TARGET,
        urlencode(&q.key),
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(5),
        q.start_timestamp.unwrap_or(0),
        q.end_timestamp.unwrap_or(0),
    );
    proxy(target, HeaderMap::new(), false).await
}

// ---------- Core: forward a request to upstream ----------

async fn proxy(
    target: String,
    headers: HeaderMap,
    with_auth: bool,
) -> Response<Body> {
    let client = reqwest::Client::new();
    let mut req = client
        .get(&target)
        .header("accept", "application/json, text/plain, */*");
    if with_auth {
        if let Some(v) = headers.get("authorization").and_then(|h| h.to_str().ok()) {
            req = req.header("authorization", v);
        }
    }

    match req.send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let ctype = resp
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/json")
                .to_string();
            let body = resp.bytes().await.unwrap_or_default();
            Response::builder()
                .status(status)
                .header("content-type", ctype)
                .body(Body::from(body))
                .unwrap_or_else(|_| {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(Body::empty())
                        .unwrap()
                })
        }
        Err(e) => Response::builder()
            .status(StatusCode::BAD_GATEWAY)
            .header("content-type", "text/plain; charset=utf-8")
            .body(Body::from(format!("proxy error: {}", e)))
            .unwrap(),
    }
}

fn urlencode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() || "-_.~".contains(c) {
            out.push(c);
        } else {
            let mut buf = [0u8; 4];
            let s = c.encode_utf8(&mut buf);
            for b in s.bytes() {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

// ---------- main ----------

#[tokio::main]
async fn main() {
    let static_dir = find_static_dir();
    println!("[proxy] serving static files from: {}", static_dir.display());

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/usage", get(api_usage))
        .route("/api/subscription", get(api_subscription))
        .route("/api/log/token", get(api_log))
        // 任何非 /api/* 请求 → 当作静态文件
        .fallback_service(
            ServeDir::new(&static_dir)
                .append_index_html_on_directories(true)
                .fallback(ServeDir::new(&static_dir)),
        )
        .layer(CorsLayer::permissive());

    let addr: SocketAddr = format!("{}:{}", LISTEN_HOST, LISTEN_PORT)
        .parse()
        .expect("invalid listen address");

    println!("[proxy] VectorEngine proxy listening on http://{}", addr);
    println!("[proxy] 在浏览器打开上面的地址即可。Ctrl-C 停止。");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server crashed");
}

async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
    println!("\n[proxy] shutting down");
}

/// 自动定位包含 index.html 的目录:
/// 从当前工作目录、可执行文件所在目录分别出发,逐级向上找祖先目录,
/// 直到找到 index.html 为止。
fn find_static_dir() -> PathBuf {
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            roots.push(parent.to_path_buf());
        }
    }

    for root in roots {
        let mut current = Some(root);
        while let Some(c) = current {
            if c.join("index.html").is_file() {
                return c;
            }
            current = c.parent().map(|p| p.to_path_buf());
        }
    }

    // 真找不到:返回 cwd
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}
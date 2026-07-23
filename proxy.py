#!/usr/bin/env python3
"""
VectorEngine 本地代理 - 解决浏览器跨域问题
用法:
  python3 proxy.py
然后浏览器打开: http://localhost:8765/
"""
import http.server
import urllib.request
import urllib.error
import json
import os
import sys
from urllib.parse import urlparse, parse_qs

LISTEN_HOST = "127.0.0.1"
LISTEN_PORT = 8765
TARGET = "https://api.vectorengine.ai"
STATIC_DIR = os.path.dirname(os.path.abspath(__file__))


class ProxyHandler(http.server.BaseHTTPRequestHandler):
    def log_message(self, fmt, *args):
        sys.stderr.write("[proxy] " + (fmt % args) + "\n")

    def _send_json(self, code, obj):
        body = json.dumps(obj).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Access-Control-Allow-Origin", "*")
        self.end_headers()
        self.wfile.write(body)

    def _send_cors_preflight(self):
        self.send_response(204)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Authorization, Content-Type")
        self.end_headers()

    def do_OPTIONS(self):
        self._send_cors_preflight()

    def do_GET(self):
        parsed = urlparse(self.path)

        # 健康检查
        if parsed.path == "/health":
            self._send_json(200, {"ok": True, "service": "vectorengine-proxy"})
            return

        # 根路径 → 返回 index.html
        if parsed.path == "/" or parsed.path == "/index.html":
            self._serve_static("index.html")
            return

        # 路由: /api/usage?start_date=...&end_date=...
        if parsed.path == "/api/usage":
            qs = parse_qs(parsed.query)
            start = (qs.get("start_date") or [""])[0]
            end   = (qs.get("end_date")   or [""])[0]
            target = f"{TARGET}/v1/dashboard/billing/usage?start_date={start}&end_date={end}"
            self._proxy(target)
            return

        # 路由: /api/subscription
        if parsed.path == "/api/subscription":
            self._proxy(f"{TARGET}/v1/dashboard/billing/subscription")
            return

        # 路由: /api/log/token?key=...&page=...&page_size=...&start_timestamp=...&end_timestamp=...
        if parsed.path == "/api/log/token":
            qs = parsed.query  # 透传所有 query 参数
            target = f"{TARGET}/api/log/token"
            if qs:
                target += "?" + qs
            self._proxy(target, with_auth_header=False)
            return

        # 其他: 尝试作为静态文件(如 favicon.ico)
        rel = parsed.path.lstrip("/")
        if rel and not rel.startswith("api/"):
            self._serve_static(rel)
            return

        self._send_json(404, {"error": "not found", "path": parsed.path})

    def _serve_static(self, name):
        # 防路径穿越
        safe = os.path.normpath(name).lstrip(os.sep)
        full = os.path.join(STATIC_DIR, safe)
        if not full.startswith(STATIC_DIR) or not os.path.isfile(full):
            self.send_response(404); self.end_headers(); return
        ctype = "text/html; charset=utf-8" if safe.endswith(".html") else \
                "text/css"  if safe.endswith(".css")  else \
                "application/javascript" if safe.endswith(".js") else \
                "application/octet-stream"
        with open(full, "rb") as f:
            body = f.read()
        self.send_response(200)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        self.wfile.write(body)

    def _proxy(self, target, with_auth_header=True):
        req = urllib.request.Request(target, method="GET")
        if with_auth_header:
            auth = self.headers.get("Authorization", "")
            if auth:
                req.add_header("Authorization", auth)
        req.add_header("Accept", "application/json, text/plain, */*")

        try:
            with urllib.request.urlopen(req, timeout=30) as resp:
                body = resp.read()
                self.send_response(resp.status)
                self.send_header("Content-Type", resp.headers.get("Content-Type", "application/json"))
                self.send_header("Content-Length", str(len(body)))
                self.send_header("Access-Control-Allow-Origin", "*")
                self.end_headers()
                self.wfile.write(body)
        except urllib.error.HTTPError as e:
            try:    err_body = e.read()
            except: err_body = json.dumps({"error": str(e)}).encode()
            self.send_response(e.code)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(err_body)))
            self.send_header("Access-Control-Allow-Origin", "*")
            self.end_headers()
            self.wfile.write(err_body)
        except Exception as e:
            self._send_json(502, {"error": "proxy failure", "detail": str(e)})


def main():
    server = http.server.ThreadingHTTPServer((LISTEN_HOST, LISTEN_PORT), ProxyHandler)
    url = f"http://{LISTEN_HOST}:{LISTEN_PORT}/"
    print(f"[proxy] VectorEngine proxy listening on {url}")
    print(f"[proxy] 直接在浏览器打开上面的地址即可。Ctrl-C 停止。")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\n[proxy] shutting down")
        server.shutdown()


if __name__ == "__main__":
    main()
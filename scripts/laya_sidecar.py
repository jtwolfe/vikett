#!/usr/bin/env python3
"""Optional System One sidecar using the official PyTorch Laya package.

The Vikett engine is Rust. This process only answers POST /v1/systemone
the way EdgeJev does, so you can try `pip install laya` without an ONNX
export. EdgeJev INT8 (`edgejev serve --port 8009`) is the Linux CPU path
the docs recommend; use this sidecar only if you already have torch.

    pip install laya
    python scripts/laya_sidecar.py --port 8009
    cargo run --release -- tree -u "mute" -s desk --referee laya
"""

from __future__ import annotations

import argparse
import json
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer


def load_agent(name: str):
    import laya

    return laya.load(name)


class Handler(BaseHTTPRequestHandler):
    agent = None

    def log_message(self, fmt, *args):
        print("[laya]", fmt % args)

    def _send(self, code: int, body: dict):
        raw = json.dumps(body).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(raw)))
        self.end_headers()
        self.wfile.write(raw)

    def do_GET(self):
        if self.path in ("/health", "/v1/models"):
            self._send(200, {"ok": True, "model": "laya"})
            return
        self._send(404, {"error": "not found"})

    def do_POST(self):
        if self.path not in ("/v1/systemone", "/v1/system_one"):
            self._send(404, {"error": "not found"})
            return
        n = int(self.headers.get("Content-Length", "0"))
        payload = json.loads(self.rfile.read(n) or b"{}")
        state = payload.get("state")
        questions = payload.get("questions") or {}
        fn = getattr(self.agent, "system_one", None) or getattr(self.agent, "predict")
        result = fn(state, questions)
        if hasattr(result, "model_dump"):
            result = result.model_dump()
        elif not isinstance(result, dict):
            result = {"answers": getattr(result, "answers", result)}
        self._send(200, result if "answers" in result else {"answers": result})


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--port", type=int, default=8009)
    p.add_argument("--model", default="convaiinnovations/laya", help="HF id or subfolder")
    args = p.parse_args()
    Handler.agent = load_agent(args.model)
    httpd = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
    print(f"laya sidecar on http://127.0.0.1:{args.port}/v1/systemone")
    httpd.serve_forever()


if __name__ == "__main__":
    main()

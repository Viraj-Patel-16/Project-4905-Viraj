import os
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer

INSTANCE_NAME = os.environ.get("INSTANCE_NAME", "backend")
PORT = int(os.environ.get("PORT", "9090"))


class Handler(BaseHTTPRequestHandler):
    def _respond(self, body_note):
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.send_header("X-Backend-Instance", INSTANCE_NAME)
        self.end_headers()
        self.wfile.write(f"OK from {INSTANCE_NAME}: {body_note}\n".encode())

    def do_POST(self):
        length = int(self.headers.get("Content-Length", "0"))
        body = self.rfile.read(length).decode(errors="replace")
        print(f"[{INSTANCE_NAME}] POST {self.path} {body}", flush=True)
        self._respond(f"POST {self.path}")

    def do_GET(self):
        print(f"[{INSTANCE_NAME}] GET {self.path}", flush=True)
        self._respond(f"GET {self.path}")

    def log_message(self, *args):
        pass


if __name__ == "__main__":
    print(f"[{INSTANCE_NAME}] listening on 0.0.0.0:{PORT}", flush=True)
    ThreadingHTTPServer(("0.0.0.0", PORT), Handler).serve_forever()

import os
import socket
import threading

MODE = os.environ.get("MODE", "tcp").lower()
PORT = int(os.environ.get("PORT", "9092"))
INSTANCE_NAME = os.environ.get("INSTANCE_NAME", "l4-backend")


def handle_tcp_conn(conn, addr):
    with conn:
        buf = b""
        while True:
            data = conn.recv(4096)
            if not data:
                break
            buf += data
            while b"\n" in buf:
                line, buf = buf.split(b"\n", 1)
                print(f"[{INSTANCE_NAME}] TCP {line.decode(errors='replace')}", flush=True)
        if buf:
            print(f"[{INSTANCE_NAME}] TCP {buf.decode(errors='replace')}", flush=True)


def run_tcp():
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("0.0.0.0", PORT))
    s.listen()
    print(f"[{INSTANCE_NAME}] TCP listening on 0.0.0.0:{PORT}", flush=True)
    while True:
        conn, addr = s.accept()
        threading.Thread(target=handle_tcp_conn, args=(conn, addr), daemon=True).start()


def run_udp():
    s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    s.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    s.bind(("0.0.0.0", PORT))
    print(f"[{INSTANCE_NAME}] UDP listening on 0.0.0.0:{PORT}", flush=True)
    while True:
        data, addr = s.recvfrom(65535)
        print(f"[{INSTANCE_NAME}] UDP {data.decode(errors='replace')}", flush=True)


if __name__ == "__main__":
    if MODE == "udp":
        run_udp()
    else:
        run_tcp()

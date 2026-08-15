# COMP-4905-Viraj
Benchmarking framework for evaluating load balancing algorithms on Layer 4.

## Target Integration (NGINX/HAProxy)

The TUI `Target Config` screen now supports target presets for:

- `Generic`
- `NGINX`
- `HAProxy`

Workflow:

1. Open `Target Config` (`5` key).
2. Set `enabled: true`.
3. Select `target` as `NGINX` or `HAProxy`.
4. Choose `protocol` (`HTTP` / `TCP` / `UDP`).
5. Optionally edit `endpoint` and `http_path`.
6. Press `g` to generate events, export JSONL/summary, and send traffic.

Notes:

- For HTTP targets, events are sent as `POST` JSON payloads.
- For TCP/UDP targets, events are sent as raw JSON bytes (newline-delimited for TCP).
- Endpoint parsing is tolerant of scheme/path input and normalizes to the required socket address for TCP/UDP.
- A local HTTP receiver on `127.0.0.1:8080` can be auto-started when the app launches for local validation.
- Auto-receiver is disabled by default. To enable it, set `COMP4905_AUTO_RECEIVER=1` before running the app.

## Real Load Balancer Setup (NGINX)

For credible, end-to-end validation, the project includes a real NGINX load
balancer that receives generated traffic and distributes it across backend
applications, at both Layer 7 (HTTP) and Layer 4 (TCP/UDP):

```
L7  App ──POST http://127.0.0.1:8080/traffic──▶ NGINX http (:8080) ──▶ backend1 (:9090)
                                                                  └──▶ backend2 (:9091)

L4  App ──TCP  127.0.0.1:9000──▶ NGINX stream (:9000) ──▶ tcp-backend1 (:9092)
                                                     └──▶ tcp-backend2 (:9093)
    App ──UDP  127.0.0.1:9001──▶ NGINX stream (:9001) ──▶ udp-backend1 (:9094)
                                                     └──▶ udp-backend2 (:9095)
```

The app's default endpoints already match these ports, so no code change is
needed. Select `NGINX` as the target for accurate metadata.

**Protocol support by environment:**

| Protocol | Native Windows | Docker/Linux |
|----------|----------------|--------------|
| HTTP (L7) | Yes | Yes |
| TCP (L4)  | Yes | Yes |
| UDP (L4)  | No (Windows NGINX build has no UDP stream) | Yes |

### Option A — Docker (reproducible, full L4 + L7)

Requires Docker Desktop (WSL2 backend on Windows). This is the only option that
supports UDP load balancing.

```
docker compose up
```

This starts NGINX plus HTTP backends (`docker/backend.py`) and raw TCP/UDP
backends (`docker/raw_backend.py`), using `docker-compose.yml` and
`docker/nginx.conf`. NGINX balances HTTP on `:8080`, TCP on `:9000`, and UDP on
`:9001`.

### Option B — Native Windows (no Docker, HTTP + TCP only)

1. Download NGINX for Windows and extract it under `nginx/`.
2. Start the HTTP backends (from the repo root):
   ```
   $env:INSTANCE_NAME="backend1"; $env:PORT="9090"; Start-Process python -ArgumentList "docker\backend.py"
   $env:INSTANCE_NAME="backend2"; $env:PORT="9091"; Start-Process python -ArgumentList "docker\backend.py"
   ```
3. Start the raw TCP backends (for L4 TCP balancing):
   ```
   $env:MODE="tcp"; $env:INSTANCE_NAME="tcp-backend1"; $env:PORT="9092"; Start-Process python -ArgumentList "docker\raw_backend.py"
   $env:MODE="tcp"; $env:INSTANCE_NAME="tcp-backend2"; $env:PORT="9093"; Start-Process python -ArgumentList "docker\raw_backend.py"
   ```
4. Start NGINX with the provided config:
   ```
   $ngx="$PWD\nginx\nginx-1.27.4"
   Start-Process "$ngx\nginx.exe" -ArgumentList "-p","`"$ngx`"","-c","`"$PWD\nginx\comp4905.conf`"" -WorkingDirectory $ngx
   ```
5. Run the app, set target = `NGINX`, choose HTTP or TCP, and press `g`.

To stop NGINX: `& "$PWD\nginx\nginx-1.27.4\nginx.exe" -s stop` (run from that dir).

### Validating NGINX

```
# NGINX process and ports
Get-Process nginx
netstat -ano | Select-String ":8080.*LISTENING"

# L7 HTTP per-request log with the chosen backend (round-robin = load balancing)
Get-Content ".\nginx\nginx-1.27.4\logs\access.log" -Tail 10

# L4 TCP/UDP stream log with the chosen upstream backend
Get-Content ".\nginx\nginx-1.27.4\logs\stream_access.log" -Tail 10

# Watch traffic live while pressing 'g' in the app
Get-Content ".\nginx\nginx-1.27.4\logs\access.log" -Wait -Tail 5
```

Note: the NGINX binary distribution and logs are git-ignored; only the config
files (`nginx/comp4905.conf`, `docker/`, `docker-compose.yml`) are committed.

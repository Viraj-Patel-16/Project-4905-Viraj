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

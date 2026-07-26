pub mod app;
pub mod ui;

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::{self, Event, KeyEventKind};
use serde::Serialize;

use crate::{
    generator,
    model::{NetworkPayload, Task, TrackingMetadata, TrafficEvent},
    sender,
    sink::{ConsumerSink, JsonLinesSink},
    worker::spawn_worker,
};

use app::{App, Screen};

struct AutoHttpReceiver {
    stop: Arc<AtomicBool>,
    join_handle: Option<thread::JoinHandle<()>>,
}

impl AutoHttpReceiver {
    fn start_if_enabled() -> Option<Self> {
        if !auto_receiver_enabled() {
            return None;
        }

        let listener = match TcpListener::bind("127.0.0.1:8080") {
            Ok(listener) => listener,
            Err(_) => return None,
        };

        if listener.set_nonblocking(true).is_err() {
            return None;
        }

        let stop = Arc::new(AtomicBool::new(false));
        let stop_flag = Arc::clone(&stop);

        let join_handle = thread::spawn(move || {
            let mut read_buffer = [0_u8; 4096];
            let response = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nOK";

            while !stop_flag.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _addr)) => {
                        let _ = stream.read(&mut read_buffer);
                        let _ = stream.write_all(response);
                        let _ = stream.flush();
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(25));
                    }
                    Err(_) => break,
                }
            }
        });

        Some(Self {
            stop,
            join_handle: Some(join_handle),
        })
    }

    fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}

fn auto_receiver_enabled() -> bool {
    match std::env::var("COMP4905_AUTO_RECEIVER") {
        Ok(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            normalized != "0" && normalized != "false" && normalized != "off"
        }
        Err(_) => false,
    }
}

#[derive(Debug, Serialize)]
struct TenantTrafficSummary {
    tenant_id: String,
    event_count: usize,
    total_payload_bytes: u64,
}

#[derive(Debug, Serialize)]
struct TrafficGenerationSummary {
    generated_at_ms: u64,
    total_events: usize,
    total_payload_bytes: u64,
    first_timestamp_ms: Option<u64>,
    last_timestamp_ms: Option<u64>,
    tenant_summaries: Vec<TenantTrafficSummary>,
}

fn build_traffic_summary(events: &[TrafficEvent]) -> TrafficGenerationSummary {
    let mut by_tenant: BTreeMap<String, (usize, u64)> = BTreeMap::new();

    for event in events {
        let entry = by_tenant.entry(event.tenant_id.clone()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 += u64::from(event.payload_size_bytes);
    }

    let tenant_summaries = by_tenant
        .into_iter()
        .map(|(tenant_id, (event_count, total_payload_bytes))| TenantTrafficSummary {
            tenant_id,
            event_count,
            total_payload_bytes,
        })
        .collect();

    TrafficGenerationSummary {
        generated_at_ms: match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_millis() as u64,
            Err(_) => 0,
        },
        total_events: events.len(),
        total_payload_bytes: events
            .iter()
            .map(|event| u64::from(event.payload_size_bytes))
            .sum(),
        first_timestamp_ms: events.first().map(|event| event.timestamp_ms),
        last_timestamp_ms: events.last().map(|event| event.timestamp_ms),
        tenant_summaries,
    }
}

pub async fn run() -> std::io::Result<()> {
    ratatui::run(|terminal| {
        let mut app = App::default();
        let mut auto_receiver = AutoHttpReceiver::start_if_enabled();
        let worker = spawn_worker("preview_worker", 256);
        app.worker_preview.worker_id = worker.worker_id.clone();

        let mut request_sequence: u64 = 0;
        let mut last_submit_at = Instant::now();

        fn now_millis() -> u64 {
            match SystemTime::now().duration_since(UNIX_EPOCH) {
                Ok(duration) => duration.as_millis() as u64,
                Err(_) => 0,
            }
        }

        loop {
            if app.current_screen == Screen::Preview {
                if let Some(tenant) = app.tenants.get(app.selected_tenant) {
                    let interval_ms =
                        (1000_u64 / u64::from(tenant.requests_per_second.max(1))).clamp(20, 1000);

                    if last_submit_at.elapsed() >= Duration::from_millis(interval_ms) {
                        request_sequence += 1;
                        let payload = NetworkPayload::new(
                            vec![0_u8; tenant.payload_size_bytes as usize],
                            "application/octet-stream",
                        );
                        let metadata = TrackingMetadata::new(now_millis(), request_sequence);
                        let task = Task::new(&tenant.tenant_id, payload, metadata);

                        match worker.submit(task) {
                            Ok(()) => {
                                app.worker_preview.submitted_tasks += 1;
                                app.worker_preview.last_error = None;
                            }
                            Err(error) => {
                                app.worker_preview.last_error = Some(error.to_string());
                            }
                        }

                        last_submit_at = Instant::now();
                    }
                }
            } else {
                last_submit_at = Instant::now();
            }

            let snapshot = worker.snapshot_state();
            app.worker_preview.current_load = snapshot.current_load;
            app.worker_preview.is_free = snapshot.is_free;
            app.worker_preview.is_busy = snapshot.is_busy;
            app.worker_preview.processed_tasks = snapshot.processed_tasks;

            terminal.draw(|frame| ui::render(frame, &app))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        app.handle_key(key);
                    }
                }
            }

            if app.take_generate_export_request() {
                let output_path = "results/traffic_events.jsonl";
                let summary_output_path = "results/traffic_summary.json";
                let profiles = app.tenants.clone();
                let events = generator::generate(&profiles, app.target_config.system);

                let export_result = (|| -> anyhow::Result<()> {
                    let mut sink = JsonLinesSink::new(output_path)?;
                    for event in &events {
                        sink.consume(event)?;
                    }
                    sink.flush()?;

                    let summary = build_traffic_summary(&events);
                    if let Some(parent) = std::path::Path::new(summary_output_path).parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    let summary_json = serde_json::to_vec_pretty(&summary)?;
                    std::fs::write(summary_output_path, summary_json)?;
                    Ok(())
                })();

                match export_result {
                    Ok(()) => {
                        let send_report = sender::send_events(&events, &app.target_config);
                        app.set_send_report(
                            send_report.attempted,
                            send_report.succeeded,
                            send_report.failed,
                            send_report.last_error,
                        );
                        app.set_generation_result(events, output_path);
                    }
                    Err(error) => app.set_generation_error(error.to_string()),
                }
            }

            if app.should_quit {
                worker.close();
                if let Some(receiver) = auto_receiver.take() {
                    receiver.stop();
                }
                break Ok(());
            }
        }
    })
}

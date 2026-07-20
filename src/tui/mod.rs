pub mod app;
pub mod ui;

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crossterm::event::{self, Event, KeyEventKind};

use crate::{
    model::{NetworkPayload, Task, TrackingMetadata},
    worker::spawn_worker,
};

use app::{App, Screen};

pub async fn run() -> std::io::Result<()> {
    ratatui::run(|terminal| {
        let mut app = App::default();
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

            if app.should_quit {
                worker.close();
                break Ok(());
            }
        }
    })
}

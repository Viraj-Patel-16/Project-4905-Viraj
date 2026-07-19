pub mod app;
pub mod ui;

use crossterm::event::{self, Event, KeyEventKind};

use app::App;

pub fn run() -> std::io::Result<()> {
    ratatui::run(|terminal| {
        let mut app = App::default();

        loop {
            terminal.draw(|frame| ui::render(frame, &app))?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key);
                }
            }

            if app.should_quit {
                break Ok(());
            }
        }
    })
}

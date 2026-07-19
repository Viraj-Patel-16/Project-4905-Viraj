mod model;
mod sink;
mod tui;
mod worker;

fn main() -> anyhow::Result<()> {
    tui::run()?;
    Ok(())
}

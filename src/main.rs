mod generator;
mod model;
mod sender;
mod sink;
mod tui;
mod worker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tui::run().await?;
    Ok(())
}

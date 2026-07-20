mod model;
mod sink;
mod tui;
mod worker;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tui::run().await?;
    Ok(())
}

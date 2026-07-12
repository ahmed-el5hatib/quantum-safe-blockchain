use qsb_node::prelude::*;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!(target: "qsb", "Starting QSB single node...");
    Ok(())
}

mod server;
mod statics;

use clap::Parser;
use statics::{init, Args};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init(args)?;
    server::run().await?;
    Ok(())
}

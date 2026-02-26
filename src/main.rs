use clap::Parser;

mod server;
mod statics;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// VPS credentials in format veid1:key1,veid2:key2,...
    #[arg(long, value_delimiter = ',')]
    pub credentials: Vec<String>,

    /// Listen address
    #[arg(long, default_value = "127.0.0.1:3000")]
    pub listen_addr: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    statics::init(Args::parse())?;
    server::run().await?;
    Ok(())
}

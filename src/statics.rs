use anyhow::{Context, Result};
use std::sync::OnceLock;
use minijinja::Environment;
use clap::Parser;
use crate::server;

#[derive(Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Multiple VEIDs, separated by comma
    #[arg(long, value_delimiter = ',')]
    pub veids: Vec<String>,

    /// Multiple API Keys, corresponding to VEIDs
    #[arg(long, value_delimiter = ',')]
    pub api_keys: Vec<String>,
}

pub struct Config {
    pub credentials: Vec<(String, String)>,
    pub jinja: Environment<'static>,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init(args: Args) -> Result<()> {
    if args.veids.len() != args.api_keys.len() {
        anyhow::bail!(
            "The number of veids ({}) and api_keys ({}) must be the same.",
            args.veids.len(),
            args.api_keys.len()
        );
    }

    let credentials: Vec<(String, String)> = args
        .veids
        .into_iter()
        .zip(args.api_keys.into_iter())
        .collect();

    let mut jinja = Environment::new();
    jinja.add_filter("filesize", server::human_readable_size);
    jinja.add_template("info-page", include_str!("../templates/info-page.html"))
        .context("Failed to add template")?;

    let config = Config { credentials, jinja };
    CONFIG.set(config).map_err(|_| anyhow::anyhow!("Config already initialized"))?;
    Ok(())
}

pub fn cfg() -> Result<&'static Config> {
    CONFIG.get().context("get config error")
}

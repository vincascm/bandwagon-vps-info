use std::sync::OnceLock;

use anyhow::{Context, Result, anyhow};
use minijinja::Environment;

use crate::Args;

pub struct Config {
    pub credentials: Vec<(String, String)>,
    pub jinja: Environment<'static>,
    pub listen_addr: String,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn init(args: Args) -> Result<()> {
    let mut credentials = Vec::new();
    for i in args.credentials {
        let (veid, key) = i
            .split_once(':')
            .ok_or_else(|| anyhow!("Invalid credential format: {i}. Expected veid:key"))?;
        credentials.push((veid.to_string(), key.to_string()));
    }

    let mut jinja = Environment::new();
    jinja.add_filter("filesize", human_readable_size);
    jinja.add_template("info-page", include_str!("../templates/info-page.html"))?;

    let config = Config {
        credentials,
        jinja,
        listen_addr: args.listen_addr,
    };
    CONFIG
        .set(config)
        .map_err(|_| anyhow!("Config already initialized"))?;
    Ok(())
}

pub fn cfg() -> Result<&'static Config> {
    CONFIG.get().context("get config error")
}

fn human_readable_size(size: u64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{size:.2} {}", units[unit_idx])
}

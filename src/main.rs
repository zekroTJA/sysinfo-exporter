mod collector;
mod config;
mod metrics;

use anyhow::Result;
use clap::{Parser, arg, command};
use config::Config;
use metrics::Metrics;
use std::{path::PathBuf, time::Duration};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let cfg = Config::parse(cli.config)?;

    let log_level = cfg
        .loglevel
        .as_ref()
        .map(|l| l.parse())
        .transpose()?
        .unwrap_or(tracing::Level::INFO);

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_writer(std::io::stdout)
        .init();

    tracing::debug!("Config: {cfg:?}");

    let m = Metrics::new(cfg.endpoint, cfg.auth, cfg.labels)?;

    m.start_schedule(Duration::from_secs(cfg.interval_seconds.unwrap_or(15)))
        .await;

    Ok(())
}

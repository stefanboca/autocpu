use autocpu::{Args, Config};
use clap::Parser;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let args = Args::parse();

    match args.command {
        autocpu::Command::Daemon { config } => {
            let config = Config::load(&config)?;
            autocpu::daemon::run(config).await
        }
    }
}

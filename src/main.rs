use autocpu::{Args, Config};
use clap::Parser;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    match args.command {
        autocpu::Command::Daemon { config } => {
            let config = Config::load(&config)?;
            autocpu::daemon::run(config).await
        }
    }
}

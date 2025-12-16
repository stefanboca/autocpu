use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    Daemon {
        #[arg(short, long)]
        config: PathBuf,
    },
}

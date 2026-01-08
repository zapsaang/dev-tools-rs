mod commands;
mod utils;

use clap::{Parser, Subcommand};
use commands::{scc, ts, ucc};

#[derive(Parser)]
#[command(name = "dt")]
#[command(about = "All-in-one Developer Utility Tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// String Case Converter (字符串风格转换)
    Scc(scc::SccArgs),

    /// Universal Code Converter (编码/JWT/HTML 智能转换)
    Ucc(ucc::UccArgs),

    /// Timestamp Utility (时间格式处理)
    Ts(ts::TsArgs),
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scc(args) => scc::run(args),
        Commands::Ucc(args) => ucc::run(args),
        Commands::Ts(args) => ts::run(args),
    }
}

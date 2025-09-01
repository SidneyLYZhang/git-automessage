//     _         _        __  __
//    / \  _   _| |_ ___ |  \/  | ___  ___ ___  __ _  __ _  ___
//   / _ \| | | | __/ _ \| |\/| |/ _ \/ __/ __|/ _` |/ _` |/ _ \
//  / ___ \ |_| | || (_) | |  | |  __/\__ \__ \ (_| | (_| |  __/
// /_/   \_\__,_|\__\___/|_|  |_|\___||___/___/\__,_|\__, |\___|
//                                                   |___/
//
// Author: Sidney Zhang <zly@lyzhang.me>
// Date: 2025-08-05
// License: MIT
//
// A git commit message generator.

use anyhow::Result;
use clap::Parser;

mod changelog;
mod cli;
mod config;
mod git;
mod llm;

use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Commit(args) => {
            cli::handle_commit(args).await?;
        }
        Commands::Tag(args) => {
            cli::handle_tag(args).await?;
        }
        Commands::Changelog(args) => {
            cli::handle_changelog(args).await?;
        }
        Commands::Config(args) => {
            cli::handle_config(args).await?;
        }
    }

    Ok(())
}

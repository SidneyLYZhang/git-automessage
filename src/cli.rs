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
// CLI

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "git-automessage")]
#[command(author, version, about = "AI-powered git message generator written in Rust")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate commit message for staged changes
    Commit(CommitArgs),
    /// Generate tag message and optionally create tag
    Tag(TagArgs),
    /// Generate and append changelog for recent commits
    Changelog(ChangelogArgs),
}

#[derive(Args)]
pub struct CommitArgs {
    /// Create commit with generated message
    #[arg(long)]
    pub commit: bool,
    
    /// Custom prompt for message generation
    #[arg(long)]
    pub prompt: Option<String>,
    
    /// Maximum length for commit message
    #[arg(long, default_value = "72")]
    pub max_length: usize,
}

#[derive(Args)]
pub struct TagArgs {
    /// Tag name to create
    pub name: String,
    
    /// Create annotated tag with generated message
    #[arg(long)]
    pub annotated: bool,
    
    /// Custom prompt for tag message
    #[arg(long)]
    pub prompt: Option<String>,
    
    /// Reference to tag (commit SHA or branch)
    #[arg(long, default_value = "HEAD")]
    pub reference: String,
}

#[derive(Args)]
pub struct ChangelogArgs {
    /// Number of recent commits to include
    #[arg(long, default_value = "10")]
    pub commits: usize,
    
    /// Output file for changelog
    #[arg(long)]
    pub output: Option<String>,
    
    /// Append to existing changelog file
    #[arg(long)]
    pub append: bool,
    
    /// Tag range for changelog (e.g., v1.0.0..v1.1.0)
    #[arg(long)]
    pub range: Option<String>,
}

pub async fn handle_commit(args: CommitArgs) -> Result<()> {
    use crate::{git::GitRepo, llm::MessageGenerator};
    
    let repo = GitRepo::open()?;
    let generator = MessageGenerator::new()?;
    
    let staged_files = repo.get_staged_files()?;
    if staged_files.is_empty() {
        println!("No staged changes found. Please stage your changes first.");
        return Ok(());
    }
    
    let diff = repo.get_staged_diff()?;
    let message = generator.generate_commit_message(&diff, &staged_files, args.prompt.as_deref()).await?;
    
    if args.commit {
        repo.create_commit(&message)?;
        println!("Commit created successfully!");
    } else {
        println!("Generated commit message:\n{}\n", message);
        println!("Use --commit flag to create the commit automatically.");
    }
    
    Ok(())
}

pub async fn handle_tag(args: TagArgs) -> Result<()> {
    use crate::{git::GitRepo, llm::MessageGenerator};
    
    let repo = GitRepo::open()?;
    let generator = MessageGenerator::new()?;
    
    let commit_info = repo.get_commit_info(&args.reference)?;
    let message = generator.generate_tag_message(&args.name, &commit_info, args.prompt.as_deref()).await?;
    
    if args.annotated {
        repo.create_annotated_tag(&args.name, &message, &args.reference)?;
        println!("Annotated tag '{}' created successfully!", args.name);
    } else {
        println!("Generated tag message for '{}':\n{}\n", args.name, message);
        println!("Use --annotated flag to create the tag automatically.");
    }
    
    Ok(())
}

pub async fn handle_changelog(args: ChangelogArgs) -> Result<()> {
    use crate::{changelog::ChangelogGenerator, git::GitRepo};
    
    let repo = GitRepo::open()?;
    let generator = ChangelogGenerator::new()?;
    
    let commits = if let Some(range) = &args.range {
        repo.get_commits_in_range(range)?
    } else {
        repo.get_recent_commits(args.commits)?
    };
    
    let changelog = generator.generate_changelog(&commits).await?;
    
    if let Some(output_path) = &args.output {
        generator.write_changelog(&changelog, output_path, args.append)?;
        println!("Changelog written to {}", output_path);
    } else {
        println!("Generated changelog:\n{}\n", changelog);
    }
    
    Ok(())
}
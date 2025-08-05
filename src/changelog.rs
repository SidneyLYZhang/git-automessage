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
// A git changelog generator.


use anyhow::{Context, Result};
use chrono::Local;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use crate::git::CommitInfo;
use crate::llm::MessageGenerator;

pub struct ChangelogGenerator {
    llm: MessageGenerator,
}

impl ChangelogGenerator {
    pub fn new() -> Result<Self> {
        let llm = MessageGenerator::new()?;
        Ok(ChangelogGenerator { llm })
    }

    pub async fn generate_changelog(&self, commits: &[CommitInfo]) -> Result<String> {
        let summary = self.llm.generate_changelog_summary(commits).await?;
        
        let date = Local::now().format("%Y-%m-%d").to_string();
        let version = self.detect_version_from_commits(commits)?;
        
        let changelog = format!("## [{}] - {}\n\n{}", version, date, summary);
        
        Ok(changelog)
    }

    pub fn write_changelog(&self, content: &str, output_path: &str, append: bool) -> Result<()> {
        if append {
            self.append_to_changelog(content, output_path)?;
        } else {
            self.create_new_changelog(content, output_path)?;
        }
        Ok(())
    }

    fn create_new_changelog(&self, content: &str, output_path: &str) -> Result<()> {
        let mut file = File::create(output_path)?;
        
        let header = format!(
            "# Changelog\n\nAll notable changes to this project will be documented in this file.\n\nThe format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),\nand this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).\n\n"
        );
        
        write!(file, "{}{}", header, content)?;
        Ok(())
    }

    fn append_to_changelog(&self, content: &str, output_path: &str) -> Result<()> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(output_path)?;

        let mut existing_content = String::new();
        file.read_to_string(&mut existing_content)?;

        let new_content = if existing_content.contains("# Changelog") {
            // Insert after the header
            let lines: Vec<&str> = existing_content.lines().collect();
            let mut new_lines = Vec::new();
            
            let mut header_end = 0;
            for (i, line) in lines.iter().enumerate() {
                new_lines.push(*line);
                if line.trim().is_empty() && i > 0 && lines[i-1].starts_with('#') {
                    header_end = i + 1;
                    break;
                }
            }
            
            new_lines.push(content);
            new_lines.extend_from_slice(&lines[header_end..]);
            new_lines.join("\n")
        } else {
            // Create new changelog with existing content
            format!("{}", content)
        };

        // Write back the updated content
        let mut file = File::create(output_path)?;
        write!(file, "{}", new_content)?;
        
        Ok(())
    }

    fn detect_version_from_commits(&self, commits: &[CommitInfo]) -> Result<String> {
        // Simple version detection based on commit messages
        let mut version = "Unreleased".to_string();
        
        for commit in commits {
            let message = commit.message.to_lowercase();
            if message.contains("version") || message.contains("release") {
                // Try to extract version from commit message
                let words: Vec<&str> = message.split_whitespace().collect();
                for (i, word) in words.iter().enumerate() {
                    if word.contains("version") || word.contains("release") {
                        if i + 1 < words.len() {
                            let next_word = words[i + 1];
                            if next_word.starts_with('v') && next_word.len() > 1 {
                                version = next_word.trim_matches(|c: char| !c.is_alphanumeric()).to_string();
                                break;
                            }
                        }
                    }
                }
            }
        }

        if version == "Unreleased" {
            // Generate a version based on commit types
            let has_feat = commits.iter().any(|c| c.message.to_lowercase().starts_with("feat"));
            let has_fix = commits.iter().any(|c| c.message.to_lowercase().starts_with("fix"));
            
            if has_feat {
                version = "0.1.0".to_string();
            } else if has_fix {
                version = "0.0.1".to_string();
            } else {
                version = "0.0.1".to_string();
            }
        }

        Ok(version)
    }

    pub fn generate_default_changelog(&self, commits: &[CommitInfo]) -> Result<String> {
        let date = Local::now().format("%Y-%m-%d").to_string();
        let version = self.detect_version_from_commits(commits)?;
        
        let mut changelog = format!("## [{}] - {}\n\n", version, date);
        
        // Group commits by type
        let mut features = Vec::new();
        let mut fixes = Vec::new();
        let mut docs = Vec::new();
        let mut others = Vec::new();

        for commit in commits {
            let message = commit.message.trim();
            if message.to_lowercase().starts_with("feat") {
                features.push(format!("- {} ({}) - {}", &commit.sha[..8], message, commit.author));
            } else if message.to_lowercase().starts_with("fix") {
                fixes.push(format!("- {} ({}) - {}", &commit.sha[..8], message, commit.author));
            } else if message.to_lowercase().starts_with("docs") {
                docs.push(format!("- {} ({}) - {}", &commit.sha[..8], message, commit.author));
            } else {
                others.push(format!("- {} ({}) - {}", &commit.sha[..8], message, commit.author));
            }
        }

        if !features.is_empty() {
            changelog.push_str("### Added\n\n");
            changelog.push_str(&features.join("\n"));
            changelog.push_str("\n\n");
        }

        if !fixes.is_empty() {
            changelog.push_str("### Fixed\n\n");
            changelog.push_str(&fixes.join("\n"));
            changelog.push_str("\n\n");
        }

        if !docs.is_empty() {
            changelog.push_str("### Documentation\n\n");
            changelog.push_str(&docs.join("\n"));
            changelog.push_str("\n\n");
        }

        if !others.is_empty() {
            changelog.push_str("### Other Changes\n\n");
            changelog.push_str(&others.join("\n"));
            changelog.push_str("\n\n");
        }

        Ok(changelog)
    }
}
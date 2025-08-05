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
// Message generator using llm

use anyhow::Result;
use anyhow::Context;
use rig::{completion::Prompt, providers::openai};
use std::env;

pub struct MessageGenerator {
    client: openai::Client,
    model: String,
}

impl MessageGenerator {
    pub fn new() -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")
            .or_else(|_| env::var("RIG_API_KEY"))
            .context("OPENAI_API_KEY or RIG_API_KEY environment variable not set")?;
            
        let client = openai::Client::new(&api_key);
        let model = env::var("RIG_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

        Ok(MessageGenerator { client, model })
    }

    pub async fn generate_commit_message(
        &self,
        diff: &str,
        files: &[crate::git::StagedFile],
        custom_prompt: Option<&str>,
    ) -> Result<String> {
        let file_list: Vec<String> = files.iter()
            .map(|f| format!("{} ({})", f.path, f.status))
            .collect();

        let system_prompt = r#"You are a helpful assistant that generates concise and meaningful git commit messages.

Rules for commit messages:
1. Use the format: type(scope): description
2. Types: feat, fix, docs, style, refactor, test, chore
3. Keep the first line under 50 characters
4. Use present tense ("add" not "added")
5. Be specific about what changed
6. If there are breaking changes, add "BREAKING CHANGE:" in the body

Focus on the actual changes shown in the diff and file list."#;

        let user_prompt = if let Some(custom) = custom_prompt {
            format!("{}

Files changed: {:?}

Diff:\n{}", custom, file_list, diff)
        } else {
            format!("Generate a commit message for these changes:

Files changed: {:?}

Diff:\n{}", file_list, diff)
        };

        let completion = self.client
            .completion(&self.model)
            .preamble(system_prompt)
            .max_tokens(150)
            .temperature(0.7)
            .chat(&user_prompt, None)
            .await?;

        Ok(completion.trim().to_string())
    }

    pub async fn generate_tag_message(
        &self,
        tag_name: &str,
        commit_info: &crate::git::CommitInfo,
        custom_prompt: Option<&str>,
    ) -> Result<String> {
        let system_prompt = r#"You are a helpful assistant that generates informative git tag messages.

Rules for tag messages:
1. Start with a brief summary of the release
2. Include key changes and improvements
3. Mention any breaking changes
4. Keep it concise but informative
5. Use bullet points for multiple changes"#;

        let user_prompt = if let Some(custom) = custom_prompt {
            format!("{}

Tag name: {}
Commit SHA: {}
Commit message: {}
Author: {}
Date: {}
Files changed: {:?}", 
                custom, tag_name, commit_info.sha, commit_info.message, 
                commit_info.author, commit_info.date, commit_info.files_changed)
        } else {
            format!("Generate a tag message for version {} based on this commit:

Commit SHA: {}
Commit message: {}
Author: {}
Date: {}
Files changed: {:?}", 
                tag_name, commit_info.sha, commit_info.message, 
                commit_info.author, commit_info.date, commit_info.files_changed)
        };

        let completion = self.client
            .completion(&self.model)
            .preamble(system_prompt)
            .max_tokens(300)
            .temperature(0.7)
            .chat(&user_prompt, None)
            .await?;

        Ok(completion.trim().to_string())
    }

    pub async fn generate_changelog_summary(&self, commits: &[crate::git::CommitInfo]) -> Result<String> {
        let system_prompt = r#"You are a helpful assistant that generates changelog summaries from git commits.

Rules for changelog:
1. Group changes by type (Features, Fixes, Improvements, etc.)
2. Use bullet points for each change
3. Include commit SHA for reference
4. Keep descriptions concise but clear
5. Sort changes by importance
6. Use markdown format"#;

        let commit_details: Vec<String> = commits.iter()
            .map(|c| format!("- {}: {} ({}) by {}", 
                c.sha[..8].to_string(), c.message.trim(), c.date, c.author))
            .collect();

        let user_prompt = format!("Generate a changelog summary for these commits:\n\n{}", 
            commit_details.join("\n"));

        let completion = self.client
            .completion(&self.model)
            .preamble(system_prompt)
            .max_tokens(500)
            .temperature(0.7)
            .chat(&user_prompt, None)
            .await?;

        Ok(completion.trim().to_string())
    }
}
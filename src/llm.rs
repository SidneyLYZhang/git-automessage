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
use crate::config::Config;

pub struct MessageGenerator {
    client: openai::Client,
    model: String,
    config: Config,
}

impl MessageGenerator {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        config.validate()?;
        
        let client = openai::Client::new(&config.llm.api_key);
        
        Ok(MessageGenerator { 
            client, 
            model: config.llm.model.clone(),
            config,
        })
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

        let language_instruction = if self.config.language.starts_with("zh") {
            "请使用中文生成提交消息。"
        } else {
            "Please generate commit messages in English."
        };

        let system_prompt = format!(r#"You are a helpful assistant that generates concise and meaningful git commit messages.

Rules for commit messages:
1. Use the format: type(scope): description
2. Types: feat, fix, docs, style, refactor, test, chore
3. Keep the first line under 50 characters
4. Use present tense ("add" not "added")
5. Be specific about what changed
6. If there are breaking changes, add "BREAKING CHANGE:" in the body
7. {}

Focus on the actual changes shown in the diff and file list."#, language_instruction);

        let base_prompt = custom_prompt.unwrap_or(&self.config.prompt);
        let user_prompt = format!("{}

Files changed: {:?}

Diff:\n{}", base_prompt, file_list, diff);

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
        let language_instruction = if self.config.language.starts_with("zh") {
            "请使用中文生成标签消息。"
        } else {
            "Please generate tag messages in English."
        };

        let system_prompt = format!(r#"You are a helpful assistant that generates informative git tag messages.

Rules for tag messages:
1. Start with a brief summary of the release
2. Include key changes and improvements
3. Mention any breaking changes
4. Keep it concise but informative
5. Use bullet points for multiple changes
6. {}
"#, language_instruction);

        let base_prompt = custom_prompt.unwrap_or(&self.config.prompt);
        let user_prompt = format!("{}

Tag name: {}
Commit SHA: {}
Commit message: {}
Author: {}
Date: {}
Files changed: {:?}", 
            base_prompt, tag_name, commit_info.sha, commit_info.message, 
            commit_info.author, commit_info.date, commit_info.files_changed);

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
        let language_instruction = if self.config.language.starts_with("zh") {
            "请使用中文生成变更日志。"
        } else {
            "Please generate changelog in English."
        };

        let system_prompt = format!(r#"You are a helpful assistant that generates changelog summaries from git commits.

Rules for changelog:
1. Group changes by type (Features, Fixes, Improvements, etc.)
2. Use bullet points for each change
3. Include commit SHA for reference
4. Keep descriptions concise but clear
5. Sort changes by importance
6. Use markdown format
7. {}
"#, language_instruction);

        let commit_details: Vec<String> = commits.iter()
            .map(|c| format!("- {}: {} ({}) by {}", 
                c.sha[..8].to_string(), c.message.trim(), c.date, c.author))
            .collect();

        let base_prompt = &self.config.prompt;
        let user_prompt = format!("{}\n\n提交记录:\n{}", base_prompt, 
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
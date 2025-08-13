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

use anyhow::{Context, Result};
use rig::{agent::AgentBuilder, providers::openai};
use rig::client::CompletionClient;
use rig::completion::Prompt;
use std::time::Duration;
use tokio::time::timeout;
use crate::config::Config;

pub struct MessageGenerator {
    base_url: String,
    api_key: String,
    model: String,
}

impl MessageGenerator {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        
        Ok(MessageGenerator { 
            base_url: config.llm.base_url.clone(),
            api_key: config.llm.api_key.clone(),
            model: config.llm.model.clone(),
        })
    }

    pub async fn generate_message(
        &self,
        prompt: &str,
    ) -> Result<String> {
        const MAX_RETRIES: u32 = 3;
        
        let mut retries = 0;
        
        loop {
            match self.try_generate_message(prompt).await {
                Ok(response) => return Ok(response),
                Err(e) if retries < MAX_RETRIES => {
                    retries += 1;
                    eprintln!("生成消息失败 (尝试 {}/{}): {}", retries, MAX_RETRIES, e);
                    if retries < MAX_RETRIES {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn try_generate_message(&self, prompt: &str) -> Result<String> {
        timeout(
            Duration::from_secs(30),
            self.create_and_send_request(prompt)
        ).await
        .context("请求超时")?
        .context("生成消息失败")
    }

    async fn create_and_send_request(&self, prompt: &str) -> Result<String> {
        let client = openai::ClientBuilder::new(&self.api_key)
            .base_url(&self.base_url)
            .build()
            .context("创建LLM客户端失败")?;
            
        let model = client.completion_model(&self.model);
        let agent = AgentBuilder::new(model).build();
        
        let response = agent
            .prompt(prompt)
            .await
            .context("获取LLM响应失败")?;
            
        // 清理响应内容，移除可能的markdown代码块标记
        let cleaned_response = response
            .trim()
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
            
        Ok(cleaned_response.to_string())
    }

    /// 为变更日志生成摘要
    pub async fn generate_changelog_summary(&self, commits: &[super::git::CommitInfo]) -> Result<String> {
        let commit_descriptions: Vec<String> = commits
            .iter()
            .map(|c| format!("- {}: {} (by {})", &c.sha[..8], c.message, c.author))
            .collect();
        
        let prompt = format!(
            "基于以下git提交记录，生成一个简洁的变更日志摘要。请按以下格式组织内容：

### 新增功能
- 新增的功能描述

### 修复
- 修复的问题描述

### 改进
- 其他改进描述

请确保描述简洁明了，避免技术细节。

提交记录：
{}",
            commit_descriptions.join("\n")
        );
        
        self.generate_message(&prompt).await
    }

    /// 为暂存的更改生成提交消息
    pub async fn generate_commit_message(
        &self,
        diff: &str,
        staged_files: &[super::git::StagedFile],
        custom_prompt: Option<&str>,
    ) -> Result<String> {
        let files_list: Vec<String> = staged_files
            .iter()
            .map(|f| format!("{} ({})", f.path, f.status))
            .collect();

        let base_prompt = custom_prompt.unwrap_or(
            "基于以下代码更改生成一个简洁的git提交消息。请遵循常规提交规范（Conventional Commits）。

格式：<type>: <description>

类型包括：feat, fix, docs, style, refactor, test, chore

更改内容："
        );

        let prompt = format!(
            "{base_prompt}\n\n文件更改：\n{}\n\n代码差异：\n{diff}",
            files_list.join("\n")
        );

        self.generate_message(&prompt).await
    }

    /// 为标签生成消息
    pub async fn generate_tag_message(
        &self,
        tag_name: &str,
        commit_info: &super::git::CommitInfo,
        custom_prompt: Option<&str>,
    ) -> Result<String> {
        let base_prompt = custom_prompt.unwrap_or(
            "为git标签生成一个有意义的消息。消息应该简洁地描述这个标签代表的内容。"
        );

        let prompt = format!(
            "{base_prompt}\n\n标签名称：{tag_name}\n提交信息：{}\n作者：{}\n提交SHA：{}",
            commit_info.message,
            commit_info.author,
            &commit_info.sha[..8]
        );

        self.generate_message(&prompt).await
    }
}
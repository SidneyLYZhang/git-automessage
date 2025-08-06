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
use rig::{agent::AgentBuilder, providers::openai};
use rig::client::CompletionClient;
use rig::completion::Completion;
use crate::config::{Config, LLMProvider};

pub struct MessageGenerator {
    base_url: String,
    api_key: String,
    model: String,
    provider: LLMProvider,
}

impl MessageGenerator {
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        
        Ok(MessageGenerator { 
            base_url: config.llm.base_url.clone(),
            api_key: config.llm.api_key.clone(),
            model: config.llm.model.clone(),
            provider: config.llm.provider,
        })
    }

    pub async fn generate_message(
        &self,
        prompt: &str,
    ) -> Result<String> {
        let client = openai::ClientBuilder::new(&self.api_key)
            .base_url(&self.base_url)
            .build()?;
        let model = client.completion_model(&self.model);
        let agent = AgentBuilder::new(model).build();
        let history = vec![
            Message {
                role: Role::User,
                content: prompt.to_string(),
            }
        ];
        let completion = agent.completion(prompt, history).await?;
        Ok(completion)
    }
}
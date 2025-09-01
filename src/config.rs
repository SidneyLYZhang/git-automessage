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
// configuration

use anyhow::{Context, Result};
use dirs::config_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

impl LLMConfig {
    pub fn from_name(
        name: &str,
        base_url: Option<&str>,
        api_key: Option<&str>,
        model: Option<&str>,
    ) -> Self {
        let provider = match name {
            "openai" => LLMProvider::OpenAI,
            "deepseek" => LLMProvider::DeepSeek,
            "kimi" => LLMProvider::Kimi,
            "anthropic" => LLMProvider::Anthropic,
            "ollama" => LLMProvider::Ollama,
            _ => LLMProvider::OpenAI,
        };
        let base_url = match base_url {
            Some(url) => url.to_string(),
            None => match get_env_var("GAM_BASE_URL") {
                Some(_) => "ENV".to_string(),
                None => match name {
                    "openai" | "deepseek" | "kimi" | "anthropic" | "ollama" => {
                        "default_baseurl".to_string()
                    }
                    _ => input_info("base url"),
                },
            },
        };
        let api_key = match api_key {
            Some(key) => key.to_string(),
            None => match get_env_var("GAM_API_KEY") {
                Some(_) => "ENV".to_string(),
                None => input_info("api key"),
            },
        };
        let model = match model {
            Some(model) => model.to_string(),
            None => match get_env_var("GAM_MODEL") {
                Some(_) => "ENV".to_string(),
                None => input_info("model"),
            },
        };
        LLMConfig {
            provider,
            base_url,
            api_key,
            model,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LLMProvider {
    OpenAI,
    DeepSeek,
    Kimi,
    Anthropic,
    Ollama,
}

impl LLMProvider {
    fn list_providers() -> Vec<LLMProvider> {
        vec![
            LLMProvider::OpenAI,
            LLMProvider::DeepSeek,
            LLMProvider::Kimi,
            LLMProvider::Anthropic,
            LLMProvider::Ollama,
        ]
    }

    fn from_name(name: &str) -> Option<LLMProvider> {
        Self::list_providers()
            .iter()
            .find(|&&provider| provider.get_name() == name)
            .cloned()
    }

    fn get_name(&self) -> &str {
        match self {
            LLMProvider::OpenAI => "OpenAI",
            LLMProvider::DeepSeek => "DeepSeek",
            LLMProvider::Kimi => "Kimi",
            LLMProvider::Anthropic => "Anthropic",
            LLMProvider::Ollama => "Ollama",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub llm: LLMConfig,
    pub language: String,
    pub prompt: Option<String>,
    pub emoji: bool,
    pub multi_line: bool,
}

impl Default for Config {
    fn default() -> Self {
        println!("请选择一个LLM提供商(如需自定义请问输入custom):");
        for (i, provider) in LLMProvider::list_providers().iter().enumerate() {
            println!("{}. {}", i + 1, provider.get_name());
        }
        let provider_name = input_info("请输入提供商名称");
        let api_key = input_info("API Key");
        Self::with_provider(&provider_name, &api_key)
    }
}

impl Config {
    /// 创建一个新的配置实例，指定LLM提供商
    pub fn with_provider(provider: &str, api_key: &str) -> Self {
        Self {
            llm: LLMConfig::from_name(provider, None, Some(api_key), None),
            language: "zh-CN".to_string(),
            prompt: None,
            emoji: false,
            multi_line: false,
        }
    }
    /// 获取配置文件路径（根据操作系统）
    pub fn get_config_path() -> Result<PathBuf> {
        let config_dir = config_dir()
            .context("无法获取配置目录")?
            .join("git-automessage");
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        Ok(config_dir.join("config.yaml"))
    }

    /// 设置配置项
    pub fn set_config(&mut self, key: &str, value: &str) {
        match key {
            "llm.provider" => self.llm.provider = LLMProvider::from_name(value).unwrap(),
            "llm.base_url" => self.llm.base_url = value.to_string(),
            "llm.api_key" => self.llm.api_key = value.to_string(),
            "llm.model" => self.llm.model = value.to_string(),
            "language" => self.language = value.to_string(),
            "prompt" => self.prompt = Some(value.to_string()),
            "emoji" => self.emoji = value.parse().unwrap(),
            "multi_line" => self.multi_line = value.parse().unwrap(),
            _ => println!("Unknown key: {}", key),
        }
    }

    /// 从配置文件加载配置
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        if !config_path.exists() {
            let default_config = Self::default();
            default_config.save()?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&config_path)
            .with_context(|| format!("无法读取配置文件: {:?}", config_path))?;

        let config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("无法解析YAML配置文件: {:?}", config_path))?;

        Ok(config)
    }

    /// 保存配置到文件
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path()?;

        let content = serde_yaml::to_string(self).context("无法序列化配置到YAML")?;

        fs::write(&config_path, content)
            .with_context(|| format!("无法写入配置文件: {:?}", config_path))?;

        Ok(())
    }

    /// 创建示例配置文件
    pub fn create_config() -> Result<()> {
        let config_path = Self::get_config_path()?;

        if config_path.exists() {
            println!("配置文件已存在: {:?}", config_path);
            return Ok(());
        }

        let example_config = Self::default();
        example_config.save()?;

        println!("已创建配置文件: {:?}", config_path);
        println!("！如需要使用个人Prompt，请编辑配置文件并设置prompt字段。");
        println!("！如需要更改LLM提供商，请编辑配置文件并修改llm相关内容。");

        Ok(())
    }

    /// 验证配置是否完整
    pub fn validate(&self) -> Result<()> {
        match self.llm.api_key.as_str() {
            "ENV" => {
                if get_env_var("GAM_API_KEY").is_none() {
                    anyhow::bail!("API Key不能为空");
                }
            }
            "" => anyhow::bail!("API Key不能为空"),
            _ => {}
        }

        if self.llm.base_url.is_empty() {
            anyhow::bail!("Base URL不能为空");
        }

        if self.llm.model.is_empty() {
            anyhow::bail!("模型名称不能为空");
        }

        Ok(())
    }
}

fn input_info(info: &str) -> String {
    let mut txt = String::new();
    println!("Please input {}:", info);
    std::io::stdin()
        .read_line(&mut txt)
        .expect("failed to read line");
    txt.trim().to_string()
}

fn get_env_var(var_name: &str) -> Option<String> {
    match std::env::var(var_name) {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

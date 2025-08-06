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

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};
use dirs::config_dir;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMConfig {
    pub provider: LLMProvider,
    pub base_url: String,
    pub api_key: String,
    pub model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub llm: LLMConfig,
    pub language: String,
    pub prompt: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum LLMProvider {
    OpenAI,
    DeepSeek,
    Kimi,
    Anthropic,
    Ollama,
    Custom,
}

impl LLMProvider {
    pub fn get_provider_llm(&self) -> LLMConfig {
        let api_key = if self == &LLMProvider::Custom {
            "".to_string()
        } else {
            input_info("API KEY")
        };

        match self {
            LLMProvider::OpenAI => LLMConfig {
                provider: LLMProvider::OpenAI,
                base_url: "https://api.openai.com/v1".to_string(),
                api_key: api_key,
                model: "gpt-3.5-turbo".to_string(),
            },
            LLMProvider::DeepSeek => LLMConfig {
                provider: LLMProvider::DeepSeek,
                base_url: "https://api.deepseek.cn/v1".to_string(),
                api_key: api_key,
                model: "deepseek-chat".to_string(),
            },
            LLMProvider::Kimi => LLMConfig {
                provider: LLMProvider::Kimi,
                base_url: "https://api.kimi.ai/v1".to_string(),
                api_key: api_key,
                model: "kimi-k2-0711-preview".to_string(),
            },
            LLMProvider::Anthropic => LLMConfig {
                provider: LLMProvider::Anthropic,
                base_url: "https://api.anthropic.com/v1".to_string(),
                api_key: api_key,
                model: "claude-3.5-sonnet".to_string(),
            },
            LLMProvider::Ollama => {
                let model = input_info("ollama model");
                LLMConfig {
                    provider: LLMProvider::Ollama,
                    base_url: "http://localhost:11434".to_string(),
                    api_key: api_key,
                    model: model,
                }
            },
            LLMProvider::Custom => {
                let base_url = input_info("base url");
                let model = input_info("model");
                LLMConfig {
                    provider: LLMProvider::Custom,
                    base_url: base_url,
                    api_key: api_key,
                    model: model,
                }
            },
        }
    }

    fn list_providers() -> Vec<LLMProvider> {
        vec![
            LLMProvider::OpenAI,
            LLMProvider::DeepSeek,
            LLMProvider::Kimi,
            LLMProvider::Anthropic,
            LLMProvider::Ollama,
            LLMProvider::Custom,
        ]
    }

    fn from_name(name: &str) -> Option<LLMProvider> {
        Self::list_providers().iter().find(|&&provider| provider.get_name() == name).cloned()
    }

    fn get_name(&self) -> &str {
        match self {
            LLMProvider::OpenAI => "OpenAI",
            LLMProvider::DeepSeek => "DeepSeek",
            LLMProvider::Kimi => "Kimi",
            LLMProvider::Anthropic => "Anthropic",
            LLMProvider::Ollama => "Ollama",
            LLMProvider::Custom => "Custom",
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        println!("请选择一个LLM提供商:");
        for (i, provider) in LLMProvider::list_providers().iter().enumerate() {
            println!("{}. {}", i + 1, provider.get_name());
        }
        let provider_name = input_info("请输入提供商名称");
        let provider = LLMProvider::from_name(&provider_name)
                                                 .unwrap_or(LLMProvider::OpenAI);
        Self::with_provider(provider)
    }
}

impl Config {
    /// 创建一个新的配置实例，指定LLM提供商
    pub fn with_provider(provider: LLMProvider) -> Self {
        Self {
            llm: provider.get_provider_llm(),
            language: "zh-CN".to_string(),
            prompt: None,
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
        
        let content = serde_yaml::to_string(self)
            .context("无法序列化配置到YAML")?;
        
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
        if self.llm.api_key.is_empty() {
            anyhow::bail!("API密钥不能为空，请在配置文件中设置llm.api_key");
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
    std::io::stdin().read_line(&mut txt).expect("failed to read line");
    txt.trim().to_string()
}
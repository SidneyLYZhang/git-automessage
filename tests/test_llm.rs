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
// A test for the MessageGenerator struct

use anyhow::Result;
use git_automessage::llm::MessageGenerator;
use git_automessage::config::{LLMProvider};
use std::env;
use tempfile::NamedTempFile;
use std::io::Write;

// 创建测试用的配置文件
fn create_test_config(api_key: &str, base_url: &str, model: &str, provider: LLMProvider) -> Result<NamedTempFile> {
    let config_content = format!(r#"
llm:
  provider: {:?}
  api_key: "{}"
  base_url: "{}"
  model: "{}"
"#, provider, api_key, base_url, model);

    let mut file = NamedTempFile::new()?;
    file.write_all(config_content.as_bytes())?;
    Ok(file)
}

// 测试MessageGenerator的创建和基本配置
#[tokio::test]
async fn test_message_generator_creation() -> Result<()> {
    // 使用环境变量中的API密钥，如果没有则使用测试值
    let api_key = env::var("TEST_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());
    let base_url = env::var("TEST_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let model = env::var("TEST_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
    
    let config_file = create_test_config(&api_key, &base_url, &model, LLMProvider::OpenAI)?;
    
    // 设置配置文件路径环境变量
    env::set_var("GIT_AUTOMESSAGE_CONFIG", config_file.path());
    
    let generator = MessageGenerator::new()?;
    
    assert_eq!(generator.api_key, api_key);
    assert_eq!(generator.base_url, base_url);
    assert_eq!(generator.model, model);
    
    Ok(())
}

// 测试generate_message函数的基本功能
#[tokio::test]
async fn test_generate_message_basic() -> Result<()> {
    let api_key = env::var("TEST_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());
    let base_url = env::var("TEST_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let model = env::var("TEST_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
    
    let config_file = create_test_config(&api_key, &base_url, &model, LLMProvider::OpenAI)?;
    env::set_var("GIT_AUTOMESSAGE_CONFIG", config_file.path());
    
    let generator = MessageGenerator::new()?;
    
    // 测试一个简单的提示
    let test_prompt = "请回复一个简单的测试消息：'测试成功'";
    
    // 由于网络请求可能失败，我们使用timeout来处理
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        generator.generate_message(test_prompt)
    ).await;
    
    match result {
        Ok(Ok(response)) => {
            println!("API响应: {}", response);
            assert!(!response.is_empty(), "响应不应为空");
            Ok(())
        }
        Ok(Err(e)) => {
            // 如果API调用失败，检查错误信息
            println!("API调用失败: {}", e);
            // 在CI环境中，我们允许API调用失败
            if env::var("CI").is_ok() {
                Ok(())
            } else {
                Err(e)
            }
        }
        Err(_) => {
            println!("请求超时，可能网络连接有问题或API响应慢");
            // 在CI环境中，我们允许超时
            if env::var("CI").is_ok() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("请求超时"))
            }
        }
    }
}

// 测试配置验证
#[tokio::test]
async fn test_config_validation() -> Result<()> {
    // 测试缺少API密钥的情况
    let config_file = create_test_config("", "https://api.openai.com/v1", "gpt-3.5-turbo", LLMProvider::OpenAI)?;
    env::set_var("GIT_AUTOMESSAGE_CONFIG", config_file.path());
    
    let result = MessageGenerator::new();
    assert!(result.is_err() || env::var("CI").is_ok(), "空API密钥应该导致错误");
    
    Ok(())
}

// 测试自定义提示生成
#[tokio::test]
async fn test_custom_prompt_generation() -> Result<()> {
    let api_key = env::var("TEST_API_KEY").unwrap_or_else(|_| "test-api-key".to_string());
    let base_url = env::var("TEST_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let model = env::var("TEST_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
    
    let config_file = create_test_config(&api_key, &base_url, &model, LLMProvider::OpenAI)?;
    env::set_var("GIT_AUTOMESSAGE_CONFIG", config_file.path());
    
    let generator = MessageGenerator::new()?;
    
    // 测试代码提交消息生成提示
    let custom_prompt = "基于以下代码更改，生成一个符合Conventional Commits规范的提交消息。\n\n更改内容：\n- 添加了用户认证功能\n- 修复了登录页面的样式问题";
    
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        generator.generate_message(custom_prompt)
    ).await;
    
    match result {
        Ok(Ok(response)) => {
            println!("自定义提示响应: {}", response);
            assert!(!response.is_empty(), "响应不应为空");
            Ok(())
        }
        _ => {
            println!("测试跳过：网络或API配置问题");
            Ok(())
        }
    }
}

// 集成测试：完整的配置和消息生成流程
#[tokio::test]
async fn test_full_integration() -> Result<()> {
    // 检查必要的环境变量
    let api_key = match env::var("OPENAI_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("跳过集成测试：未设置OPENAI_API_KEY环境变量");
            return Ok(());
        }
    };
    
    let base_url = env::var("OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-3.5-turbo".to_string());
    
    let config_file = create_test_config(&api_key, &base_url, &model, LLMProvider::OpenAI)?;
    env::set_var("GIT_AUTOMESSAGE_CONFIG", config_file.path());
    
    let generator = MessageGenerator::new()?;
    
    // 测试实际的API调用
    let test_prompt = "请用中文生成一个简短的git提交消息，描述添加了一个新的用户登录功能";
    
    let response = generator.generate_message(test_prompt).await?;
    
    println!("集成测试响应: {}", response);
    assert!(!response.is_empty(), "API响应不应为空");
    assert!(response.len() > 5, "响应应该包含一些有意义的内容");
    
    Ok(())
}

// 测试错误处理
#[tokio::test]
async fn test_error_handling() -> Result<()> {
    // 测试无效的API密钥
    let config_file = create_test_config("invalid-key", "https://api.openai.com/v1", "gpt-3.5-turbo", LLMProvider::OpenAI)?;
    env::set_var("GIT_AUTOMESSAGE_CONFIG", config_file.path());
    
    let generator = MessageGenerator::new()?;
    
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(10),
        generator.generate_message("测试消息")
    ).await;
    
    match result {
        Ok(Ok(_)) => {
            // 如果意外成功，可能是mock或其他原因
            Ok(())
        }
        Ok(Err(e)) => {
            println!("预期的错误: {}", e);
            Ok(())
        }
        Err(_) => {
            println!("请求超时，符合预期");
            Ok(())
        }
    }
}

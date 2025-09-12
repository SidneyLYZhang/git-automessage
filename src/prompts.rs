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
// generate llm prompt

use crate::config::Config;

static GITMOJI_HELP: &str = include_str!("../prompts/GITMOJI");

static CONVENTIONAL_COMMIT_KEYWORDS: &str = "Do not preface the commit with anything, 
except for the conventional commit keywords: fix, feat, build, chore, ci, docs, style, refactor, perf, test.";

fn get_prompt_config() -> Config {
    let config = Config::load().map_err(|e| {
        eprintln!("Error loading config: {:?}", e);
    }).unwrap();
    config
}

pub struct Prompt {
    prompt: String
}

impl Prompt {
    pub fn from_str(prompt: &str) -> Self {
        Self { prompt: prompt.to_string() }
    }
    pub fn new(prompt_type: &str) -> Self {
        match prompt_type {
            "tag" => Self::tag_prompt(),
            "commit" => Self::commit_prompt(),
            "changelog" => Self::changelog_prompt(),
            _ => {
                Self::default_prompt()
            },
        }
    }
    pub fn get_prompt(&self) -> &str {
        &self.prompt
    }
    // 私有方法 ： default prompt
    fn default_prompt() -> Self {
        Self::from_str("")
    }
    // 私有方法 : tag massage prompt
    fn tag_prompt() -> Self {
        let config = get_prompt_config();
        let emoji = config.emoji;
        let mut prompt = "";
        Self::from_str("")
    }
    // 私有方法 : commit massage prompt
    fn commit_prompt() -> Self {
        let config = get_prompt_config();
        let emoji = config.emoji;
        let mut prompt = "";
        Self::from_str("")
    }
    // 私有方法 : changelog massage prompt
    fn changelog_prompt() -> Self {
        let config = get_prompt_config();
        let emoji = config.emoji;
        let mut prompt = "";
        Self::from_str("")
    }
}

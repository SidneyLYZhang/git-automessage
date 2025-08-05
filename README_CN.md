# Git AutoMessage

![Rust](https://img.shields.io/badge/Rust-1.70+-red.svg)
![License](https://img.shields.io/badge/License-MIT-blue.svg)

[【中文】](README_CN.md) [【English】](README.md)

使用大语言模型的 AI 驱动的 git 提交和标签消息生成器。

## 功能特性

- 🤖 **AI 驱动**：使用 OpenAI GPT 模型生成有意义的提交消息
- 📝 **智能提交消息**：分析暂存的更改并生成规范的提交消息
- 🏷️ **智能标签消息**：基于提交历史创建描述性的标签消息
- 📋 **自动更新日志**：生成并追加更新日志条目
- ⚙️ **可配置**：通过环境变量自定义提示和设置
- 🔧 **Git 集成**：与现有 git 工作流程无缝集成

## 安装

### 前置要求

- Rust 1.70 或更高版本
- Git 仓库
- OpenAI API 密钥

### 从源码构建

```bash
git clone <repository-url>
cd git-automessage
cargo build --release
```

二进制文件将位于 `target/release/git-automessage`。

### 环境设置

设置您的 OpenAI API 密钥：

```bash
# Linux/macOS
export OPENAI_API_KEY="your-openai-api-key"

# Windows
set OPENAI_API_KEY=your-openai-api-key
```

可选环境变量：
- `RIG_MODEL`：指定 AI 模型（默认："gpt-4o-mini"）
- `RIG_API_KEY`：`OPENAI_API_KEY` 的替代方案

## 使用方法

### 生成提交消息

```bash
# 为暂存的更改预览提交消息
git-automessage commit

# 使用生成的消息创建提交
git-automessage commit --commit

# 使用自定义提示
git-automessage commit --prompt "专注于性能改进"

# 设置最大消息长度
git-automessage commit --max-length 50 --commit
```

### 生成标签消息

```bash
# 预览标签消息
git-automessage tag v1.0.0

# 创建带注释的标签
git-automessage tag v1.0.0 --annotated

# 标记特定提交
git-automessage tag v1.0.0 --reference HEAD~5 --annotated

# 使用自定义提示
git-automessage tag v1.0.0 --prompt "专注于破坏性变更"
```

### 生成更新日志

```bash
# 为最近 10 次提交生成更新日志
git-automessage changelog

# 为特定提交范围生成更新日志
git-automessage changelog --range v0.9.0..v1.0.0

# 写入文件
git-automessage changelog --output CHANGELOG.md

# 追加到现有更新日志
git-automessage changelog --output CHANGELOG.md --append

# 自定义提交数量
git-automessage changelog --commits 20
```

## 工作流程示例

### 典型开发工作流程

```bash
# 1. 进行更改并暂存它们
git add .

# 2. 生成并创建提交
git-automessage commit --commit

# 3. 创建发布标签
git-automessage tag v1.0.0 --annotated

# 4. 更新更新日志
git-automessage changelog --output CHANGELOG.md --append --range v0.9.0..v1.0.0
```

### CI/CD 集成

```yaml
# .github/workflows/release.yml
- name: 生成更新日志
  run: |
    git-automessage changelog --output CHANGELOG.md --append --range ${{ github.event.release.tag_name }}^..${{ github.event.release.tag_name }}
  env:
    OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
```

## 配置

### 环境变量

| 变量 | 描述 | 默认值 |
|----------|-------------|---------|
| `OPENAI_API_KEY` | OpenAI API 密钥 | 必需 |
| `RIG_API_KEY` | 替代 API 密钥 | - |
| `RIG_MODEL` | 要使用的 AI 模型 | `gpt-4o-mini` |

### 自定义提示

您可以提供自定义提示以生成更具体的消息：

```bash
# 技术重点
git-automessage commit --prompt "专注于技术实现细节"

# 面向用户的功能
git-automessage tag v1.0.0 --prompt "突出面向用户的更改和改进"
```

## API 参考

### 命令结构

```
git-automessage <command> [options]
```

#### 命令

- `commit`：生成提交消息
- `tag`：生成标签消息
- `changelog`：生成更新日志条目

#### 全局选项

- `--help`：显示帮助信息
- `--version`：显示版本信息

## 开发

### 项目结构

```
src/
├── main.rs          # 应用程序入口点
├── cli.rs           # 命令行界面
├── git.rs           # Git 仓库操作
├── llm.rs           # AI 模型集成
└── changelog.rs     # 更新日志生成
```

### 测试

```bash
cargo test
```

### 贡献

1. Fork 仓库
2. 创建功能分支
3. 进行更改
4. 如适用，添加测试
5. 提交拉取请求

## 许可证

本项目采用 MIT 许可证 - 有关详细信息，请参阅 LICENSE 文件。

## 故障排除

### 常见问题

**未找到 API 密钥**
```
错误：未设置 OPENAI_API_KEY 或 RIG_API_KEY 环境变量
```
解决方案：在环境变量中设置您的 OpenAI API 密钥。

**未找到 Git 仓库**
```
错误：在 '.' 处找不到仓库
```
解决方案：在 git 仓库中运行命令。

**无暂存的更改**
```
未找到暂存的更改。请先暂存您的更改。
```
解决方案：在生成提交消息之前运行 `git add .` 暂存您的更改。

### 调试模式

启用调试日志：

```bash
RUST_LOG=debug git-automessage commit
```


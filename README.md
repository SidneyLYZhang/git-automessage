# Git AutoMessage

![Rust](https://img.shields.io/badge/Rust-1.70+-red.svg)
![License](https://img.shields.io/badge/License-MIT-blue.svg)

[【中文】](README_CN.md) [【English】](README.md)

AI-powered git commit and tag message generator using Large Language Models.

## Features

- 🤖 **AI-Powered**: Uses OpenAI GPT models to generate meaningful commit messages
- 📝 **Smart Commit Messages**: Analyzes staged changes and generates conventional commit messages
- 🏷️ **Intelligent Tag Messages**: Creates descriptive tag messages based on commit history
- 📋 **Automatic Changelog**: Generates and appends changelog entries
- ⚙️ **Configurable**: Custom prompts and settings via environment variables
- 🔧 **Git Integration**: Seamlessly integrates with existing git workflows

## Installation

### Build from source

#### Prerequisites

- Rust 1.70 or higher
- Git repository
- OpenAI API key

#### Build

```bash
git clone <repository-url>
cd git-automessage
cargo build --release
```

The binary will be available at `target/release/git-automessage`.

### Environment Setup

Set your OpenAI API key:

```bash
# Linux/macOS
export OPENAI_API_KEY="your-openai-api-key"

# Windows
set OPENAI_API_KEY=your-openai-api-key
```

Optional environment variables:
- `RIG_MODEL`: Specify the AI model (default: "gpt-4o-mini")
- `RIG_API_KEY`: Alternative to `OPENAI_API_KEY`

## Usage

### Generate Commit Messages

```bash
# Preview commit message for staged changes
git-automessage commit

# Create commit with generated message
git-automessage commit --commit

# Use custom prompt
git-automessage commit --prompt "Focus on performance improvements"

# Set max message length
git-automessage commit --max-length 50 --commit
```

### Generate Tag Messages

```bash
# Preview tag message
git-automessage tag v1.0.0

# Create annotated tag
git-automessage tag v1.0.0 --annotated

# Tag specific commit
git-automessage tag v1.0.0 --reference HEAD~5 --annotated

# Use custom prompt
git-automessage tag v1.0.0 --prompt "Focus on breaking changes"
```

### Generate Changelog

```bash
# Generate changelog for recent 10 commits
git-automessage changelog

# Generate changelog for specific commit range
git-automessage changelog --range v0.9.0..v1.0.0

# Write to file
git-automessage changelog --output CHANGELOG.md

# Append to existing changelog
git-automessage changelog --output CHANGELOG.md --append

# Custom number of commits
git-automessage changelog --commits 20
```

## Workflow Examples

### Typical Development Workflow

```bash
# 1. Make changes and stage them
git add .

# 2. Generate and create commit
git-automessage commit --commit

# 3. Create a release tag
git-automessage tag v1.0.0 --annotated

# 4. Update changelog
git-automessage changelog --output CHANGELOG.md --append --range v0.9.0..v1.0.0
```

### CI/CD Integration

```yaml
# .github/workflows/release.yml
- name: Generate Changelog
  run: |
    git-automessage changelog --output CHANGELOG.md --append --range ${{ github.event.release.tag_name }}^..${{ github.event.release.tag_name }}
  env:
    OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
```

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OPENAI_API_KEY` | OpenAI API key | Required |
| `RIG_API_KEY` | Alternative API key | - |
| `RIG_MODEL` | AI model to use | `gpt-4o-mini` |

### Custom Prompts

You can provide custom prompts for more specific message generation:

```bash
# Technical focus
git-automessage commit --prompt "Focus on technical implementation details"

# User-facing features
git-automessage tag v1.0.0 --prompt "Highlight user-facing changes and improvements"
```

## API Reference

### Command Structure

```
git-automessage <command> [options]
```

#### Commands

- `commit`: Generate commit messages
- `tag`: Generate tag messages
- `changelog`: Generate changelog entries

#### Global Options

- `--help`: Show help information
- `--version`: Show version information

## Development

### Project Structure

```
src/
├── main.rs          # Application entry point
├── cli.rs           # Command line interface
├── git.rs           # Git repository operations
├── llm.rs           # AI model integration
└── changelog.rs     # Changelog generation
```

### Testing

```bash
cargo test
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Troubleshooting

### Common Issues

**API Key Not Found**
```
Error: OPENAI_API_KEY or RIG_API_KEY environment variable not set
```
Solution: Set your OpenAI API key in the environment variables.

**Git Repository Not Found**
```
Error: could not find repository at '.'
```
Solution: Run the command from within a git repository.

**No Staged Changes**
```
No staged changes found. Please stage your changes first.
```
Solution: Run `git add .` to stage your changes before generating commit messages.

### Debug Mode

Enable debug logging:

```bash
RUST_LOG=debug git-automessage commit
```


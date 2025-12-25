# update-version

A CLI tool for updating version numbers across multiple file types in your project. Supports automatic version incrementing and git integration for commits, tags, and pushes.

## Features

- Update versions in multiple file formats simultaneously
- Auto-increment patch version when no version is specified
- Git integration: commit, tag, and push changes automatically
- Uses local git credentials (SSH keys, credential helpers)
- Recursive directory scanning

## Installation

### From crates.io (when published)

```bash
cargo install update-version
```

### From GitHub

```bash
cargo install --git https://github.com/Drew-Chase/update-version.git
```

### From Source

```bash
git clone https://github.com/Drew-Chase/update-version.git
cd update-version
cargo install --path .
```

## Usage

The binary is named `uv` for quick access.

```bash
# Set version to 1.2.3 in all supported files
uv 1.2.3

# Auto-increment patch version (1.2.3 -> 1.2.4)
uv

# Update only Cargo.toml files
uv -t toml 1.2.3

# Update and commit changes
uv -g commit 1.2.3

# Update, commit, and push
uv -g commit-push 1.2.3

# Update, commit, tag as v1.2.3, and push both
uv -g commit-push-tag 1.2.3

# Update in a specific directory
uv -p ./my-project 1.2.3
```

## Command Line Arguments

| Argument | Short | Long | Default | Description |
|----------|-------|------|---------|-------------|
| `VERSION` | - | - | - | The new version to set (e.g., `1.2.3`). If omitted, increments the patch version. |
| `-t` | `-t` | `--types` | `all` | File types to update. See [Supported Types](#supported-types). |
| `-g` | `-g` | `--git-mode` | `none` | Git operations to perform. See [Git Modes](#git-modes). |
| `-p` | `-p` | `--path` | `./` | Path to the project directory. |
| `-v` | `-v` | `--verbose` | `false` | Enable verbose/debug logging. |

## Supported Types

| Value | Files | Description |
|-------|-------|-------------|
| `all` | All below | Updates all supported file types (default) |
| `toml` | `Cargo.toml` | Rust package manifests |
| `package-json` | `package.json` | Node.js package manifests |
| `tauri-config` | `tauri.conf.json` | Tauri application config |

## Git Modes

| Value | Description |
|-------|-------------|
| `none` | No git operations (default) |
| `commit` | Stage changes and create a commit |
| `commit-push` | Commit and push to remote |
| `commit-tag` | Commit and create a version tag (e.g., `v1.2.3`) |
| `commit-push-tag` | Commit, tag, and push both to remote |

### Git Commit Format

Commits are created with the message:
```
chore: bump version to {version}
```

Tags are created as annotated tags with the format `v{version}` (e.g., `v1.2.3`).

### Git Authentication

The tool uses your local git credentials automatically:

1. **SSH Agent** - If running, SSH keys are used from the agent
2. **SSH Keys** - Checks `~/.ssh/` for `id_ed25519`, `id_rsa`, `id_ecdsa`
3. **Credential Helper** - Uses git's configured credential helper for HTTPS
4. **Default** - Falls back to default credentials

## Examples

### Update a Tauri + Rust Project

```bash
# Update all version files and push with a tag
uv -g commit-push-tag 2.0.0
```

This will update:
- `Cargo.toml` (Rust)
- `package.json` (Node.js/frontend)
- `tauri.conf.json` (Tauri config)

Then commit, tag as `v2.0.0`, and push everything.

### CI/CD Integration

```bash
# In your release workflow
uv -g commit-push-tag ${{ github.event.inputs.version }}
```

### Increment Patch Version

```bash
# If current version is 1.2.3, this sets it to 1.2.4
uv -g commit-push
```
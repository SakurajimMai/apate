# Rust 单程序多模式设计

## 目标

- 使用 Rust workspace 提供核心库和单一 `apate` 二进制。
- 同一个程序同时服务终端用户、脚本和 agent。
- GitHub Actions 构建 Windows/Linux 应用，并把产物发布到 GitHub Releases。

## 命令

- `apate inspect <path> [--json]`
- `apate masks [--json]`
- `apate disguise --input <path> (--one-key | --mask-file <path> | --kind exe|jpg|mp4|mov) [--recursive] [--no-rename] [--json] [--dry-run]`
- `apate reveal --input <path> [--recursive] [--no-rename] [--force] [--json] [--dry-run]`
- `apate tui`

无参数运行 `apate` 时直接进入 TUI，Windows 用户可以双击 `apate.exe` 使用；脚本和 agent 仍应调用明确子命令并使用 `--json`。

## TUI

TUI 使用标准输入输出实现，不引入复杂终端依赖。无参数默认进入 TUI，满足普通用户双击 exe 即可使用的需求；自动化场景通过显式子命令区分，不依赖无参数入口。

## 发布

GitHub Actions 在 `main` push 和 `v*` tag push 时构建：

- Windows: `apate-<ref>-windows-x86_64.zip`
- Linux: `apate-<ref>-linux-x86_64.tar.gz`

`main` push 会更新 `latest` 预发布 Release；`v*` tag 会创建正式 GitHub Release。Release Notes 从 `CHANGELOG.md` 的 `Unreleased` 段抽取，但压缩包只包含平台可执行文件。

## 安全策略

- 默认还原必须通过 `inspect_file`。
- 默认重命名不覆盖已有目标文件。
- dry-run 复用正式执行的 mask 校验和输出路径检查。

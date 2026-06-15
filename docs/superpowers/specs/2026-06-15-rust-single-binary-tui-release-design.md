# Rust 单程序多模式 TUI 发布设计

## 目标

apate Rust 版采用一个二进制 `apate`，同时服务普通用户、开发者和 agent：

- 普通用户可以运行 `apate tui` 进入终端菜单。
- 开发者和 agent 可以运行稳定子命令。
- GitHub Releases 只发布一个平台应用，不拆 CLI/GUI 两个程序。

## 模式

- `apate inspect <path> [--json]`
- `apate masks [--json]`
- `apate disguise --input <path> (--one-key | --mask-file <path> | --kind exe|jpg|mp4|mov) [--recursive] [--no-rename] [--json] [--dry-run]`
- `apate reveal --input <path> [--recursive] [--no-rename] [--force] [--json] [--dry-run]`
- `apate tui`

无参数时只提示如何进入 TUI 或直接使用子命令，不自动阻塞等待输入。

## TUI 边界

TUI 使用标准输入输出实现，不引入复杂终端依赖。当前行为是一次选择、一次执行、执行后退出，避免在自动化环境中挂住。

## 发布

GitHub Actions 在 `v*` tag 推送时构建：

- Windows: `apate-<tag>-windows-x86_64.zip`
- Linux: `apate-<tag>-linux-x86_64.tar.gz`

Release notes 从 `CHANGELOG.md` 的 `Unreleased` 段抽取。

## 兼容

Rust 核心库继续写入旧版 apate 文件格式，保持旧文件可还原。


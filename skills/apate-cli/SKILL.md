---
name: apate-cli
description: Use when an agent needs to inspect, disguise, reveal, batch process, or safely automate files with the Rust apate CLI.
---

# Apate CLI

## 核心原则

先检查，再写入；批量写入前先 dry-run；自动化场景优先解析 `--json`。

## 常用命令

| 目标 | 命令 |
| --- | --- |
| 查看内置面具 | `apate masks --json` |
| 检查文件 | `apate inspect <path> --json` |
| 简易伪装 | `apate disguise --input <path> --kind jpg --json` |
| 一键伪装 | `apate disguise --input <path> --one-key --json` |
| 自定义面具 | `apate disguise --input <path> --mask-file <mask> --json` |
| 还原文件 | `apate reveal --input <path> --json` |
| 用户交互 | `apate tui` |
| 批量目录 | 给 `disguise` 或 `reveal` 加 `--recursive` |

## Agent 工作流

1. 对用户给的目标路径先运行 `apate inspect <path> --json`。
2. 对任何批量 `disguise` 或 `reveal` 先加 `--dry-run --json`。
3. 确认 `results[*].ok` 全为 `true` 后，再去掉 `--dry-run` 执行。
4. 自动化里读取 JSON 字段：`ok`、`dry_run`、`results[*].code`、`results[*].message`。
5. 还原未识别文件时不要默认加 `--force`；只有用户明确接受风险才使用。
6. agent 默认不要使用 `apate tui`；只有用户明确要求交互式终端菜单时才进入 TUI。

## 命名行为

- `disguise` 默认追加面具扩展名，例如 `a.zip` 变成 `a.zip.jpg`。
- `reveal` 默认移除最后一个扩展名，例如 `a.zip.jpg` 变回 `a.zip`。
- 需要只改内容不改文件名时加 `--no-rename`。

## 常见错误

| 错误 | 正确做法 |
| --- | --- |
| 直接 reveal 用户路径 | 先 `inspect --json` |
| 批量目录直接写入 | 先 `--dry-run --json` |
| 解析普通文本输出 | 使用 `--json` |
| 未识别文件加 `--force` | 先让用户确认风险 |
| 假设文件名已改 | 读取 `results[*].output_path` |

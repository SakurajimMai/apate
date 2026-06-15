---
name: apate-cli
description: Use when an agent needs to inspect, disguise, reveal, batch process, or safely automate files with the Rust apate CLI/TUI single binary.
---

# Apate CLI

## 核心原则

先检查，再写入；批量写入前先 dry-run；自动化场景优先解析 `--json`。

Apate 用于文件格式伪装和混淆，常见目标是对抗百度网盘等网盘按扩展名、文件头、头尾签名或格式识别做的限制。它会加密恢复元数据，并混淆原始文件头和最后最多 128 KiB 尾部窗口；它不会全量加密文件内容，涉及隐私或敏感内容时仍应先用 zip/7z 等工具加密打包，再用 Apate 伪装外观。

伪装/还原只读写文件头、固定尾部窗口和少量恢复元数据，不复制完整 payload，因此可以用于超大文件。

`apate` 是一个单二进制多模式程序：

- 直接运行 `apate` 或双击 Windows `apate.exe`：进入交互菜单，面向普通用户。
- 运行 `apate tui`：显式进入同一个 TUI 菜单，面向临时人工操作。
- 运行 `apate <subcommand>`：进入 CLI 模式，面向脚本和 agent 自动化。

## 常用命令

| 目标 | 命令 |
| --- | --- |
| 查看内置面具 | `apate masks --json` |
| 检查文件 | `apate inspect <path> --json` |
| 简易伪装 | `apate disguise --input <path> --kind jpg --json` |
| 一键伪装 | `apate disguise --input <path> --one-key --json` |
| 自定义面具 | `apate disguise --input <path> --mask-file <mask> --json` |
| 还原文件 | `apate reveal --input <path> --json` |
| 用户交互 | `apate` 或 `apate tui` |
| 批量目录 | 给 `disguise` 或 `reveal` 加 `--recursive` |

## Agent 工作流

1. 对用户给的目标路径先运行 `apate inspect <path> --json`。
2. 对任何批量 `disguise` 或 `reveal` 先加 `--dry-run --json`。
3. 确认 `results[*].ok` 全为 `true` 后，再去掉 `--dry-run` 执行。
4. 自动化里读取 JSON 字段：`ok`、`dry_run`、`results[*].code`、`results[*].message`。
5. 还原未识别文件时不要默认加 `--force`；只有用户明确接受风险才使用。
6. agent 默认不要使用无参数 `apate` 或 `apate tui`；只有用户明确要求交互式终端菜单时才进入 TUI。
7. 需要指导普通用户时，可以告诉对方双击 `apate.exe` 进入菜单；需要自动化时必须给出明确 CLI 子命令。

## 命名行为

- `disguise` 默认替换最后一个扩展名，例如 `a.zip` 变成 `a.jpg`，`a.zip` 一键伪装变成 `a.mp4`。
- `reveal` 默认优先恢复伪装时记录的原扩展名，例如 `a.jpg` 变回 `a.zip`。
- 缺少原扩展名元数据的文件会退回到移除最后一个扩展名。
- 需要只改内容不改文件名时加 `--no-rename`。

## 常见错误

| 错误 | 正确做法 |
| --- | --- |
| 直接 reveal 用户路径 | 先 `inspect --json` |
| 批量目录直接写入 | 先 `--dry-run --json` |
| 解析普通文本输出 | 使用 `--json` |
| 未识别文件加 `--force` | 先让用户确认风险 |
| 假设文件名已改 | 读取 `results[*].output_path` |

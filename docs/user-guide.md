# Apate 使用指南

## 基本流程

1. 先用 `inspect` 检查目标文件。
2. 批量处理前先用 `--dry-run --json` 预检。
3. 确认输出路径不会冲突后再执行真实写入。

## 查看内置面具

```powershell
apate masks --json
```

## 伪装文件

```powershell
# 伪装为 JPG
apate disguise --input .\secret.zip --kind jpg --json

# 一键伪装为 MP4
apate disguise --input .\secret.zip --one-key --json

# 使用自定义 mask 文件
apate disguise --input .\secret.zip --mask-file .\mask.bin --json
```

默认会追加扩展名，例如 `secret.zip` -> `secret.zip.jpg`。只想改内容、不想改名时加 `--no-rename`。

## 还原文件

```powershell
apate reveal --input .\secret.zip.jpg --json
```

默认会移除最后一个扩展名，例如 `secret.zip.jpg` -> `secret.zip`。

如果目标文件名已经存在，命令会返回 `output_exists`，不会覆盖已有文件。

## 批量处理

目录输入必须加 `--recursive`。

```powershell
apate disguise --input .\files --kind mp4 --recursive --dry-run --json
apate disguise --input .\files --kind mp4 --recursive --json
```

## TUI

```powershell
apate tui
```

TUI 是标准输入输出菜单，适合人工临时操作。脚本和 agent 应优先使用 JSON 子命令。

## 常见错误

| code | 处理方式 |
| --- | --- |
| `not_disguised` | 文件未通过默认还原检查；确认风险后才考虑 `--force` |
| `output_exists` | 先移动、删除或备份目标文件 |
| `empty_mask` | 换一个非空 mask 文件 |
| `directory_requires_recursive` | 对目录输入加 `--recursive` |
| `invalid_arguments` | `--one-key`、`--kind`、`--mask-file` 只能选一个 |

## 数据安全

Apate 会就地改写文件。处理重要文件前请先备份，批量处理前先执行 dry-run。

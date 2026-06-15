# Apate 文档索引

面向不同读者的入口：

| 你是谁                        | 推荐阅读                                                    |
| ----------------------------- | ----------------------------------------------------------- |
| 想用 apate 伪装/还原文件      | [`user-guide.md`](./user-guide.md)                          |
| 想在脚本 / agent 里调用 CLI   | [`cli-reference.md`](./cli-reference.md) + [`../skills/apate-cli/SKILL.md`](../skills/apate-cli/SKILL.md) |
| 想把 `apate-core` 当库嵌入    | [`api-reference.md`](./api-reference.md)                    |
| 想了解整体设计                | [`ARCHITECTURE.md`](./ARCHITECTURE.md)                      |

## 文件清单

- [`ARCHITECTURE.md`](./ARCHITECTURE.md) —— workspace 拓扑、数据流、旧格式字节布局
- [`api-reference.md`](./api-reference.md) —— `apate-core` 的常量、类型、函数与示例
- [`cli-reference.md`](./cli-reference.md) —— `apate` 所有子命令、flag、JSON schema、错误码
- [`user-guide.md`](./user-guide.md) —— 三种伪装模式、批量、还原、FAQ、故障排查

## 旧版 WinForms

仓库根目录的 `apate/` 是历史 C#/WinForms 实现。`apate/Resources/mask.mp4`
仍然被 Rust CLI 的 `--one-key` 模式使用，以保持与旧版体验一致。**新功能请加在
`apate-core` / `apate-cli`**，旧版 UI 不再主动开发。

## 文档维护

- 改动 CLI flag / JSON shape 时同步更新 `cli-reference.md` 与
  `skills/apate-cli/SKILL.md`。
- 改动 `apate-core` 公开 API 时同步更新 `api-reference.md`。
- 改动文件字节布局（**极少见**）时同步更新 `ARCHITECTURE.md` 与
  `crates/apate-core/tests/legacy_format.rs`。
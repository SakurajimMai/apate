# Apate 文档索引

这里是 Rust 版 Apate 的文档入口。

| 主题 | 文档 |
| --- | --- |
| 总体架构 | [ARCHITECTURE.md](./ARCHITECTURE.md) |
| 核心 API | [api-reference.md](./api-reference.md) |
| CLI 参考 | [cli-reference.md](./cli-reference.md) |
| 用户指南 | [user-guide.md](./user-guide.md) |

## 文档维护

- 改动 `apate-core` 公共 API 后，同步更新 `api-reference.md`。
- 改动 CLI 参数、JSON 输出或错误码后，同步更新 `cli-reference.md`。
- 改动文件格式或安全链路后，同步更新 `ARCHITECTURE.md` 和 `crates/apate-core/tests/format_roundtrip.rs`。

# Changelog

## Unreleased

### Fixed

- 默认还原前会校验已知面具头，避免普通文件仅因尾部 4 字节像长度字段而被误判并写坏。
- 默认重命名前会拒绝覆盖已有目标文件，避免 `disguise` / `reveal` 静默覆盖用户数据。
- `disguise --dry-run --mask-file` 会复用正式执行的面具校验，空面具和超限面具不再被预检误报为成功。

### Added

- Rust workspace 重构。
- 单程序多模式入口 `apate`。
- `inspect`、`masks`、`disguise`、`reveal`、`tui` 子命令。
- 旧格式文件兼容测试。
- 面向 agent 的 `skills/apate-cli`。
- GitHub Actions 构建与 Release 流程。
- `docs/` 文档体系：`ARCHITECTURE.md`、`api-reference.md`、`cli-reference.md`、`user-guide.md`、`docs/README.md` 索引。

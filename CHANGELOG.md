# Changelog

## Unreleased

### Added

- Rust workspace：`apate-core` 和 `apate-cli`。
- 单程序多模式入口：`inspect`、`masks`、`disguise`、`reveal`、`tui`。
- 面向 agent 的 `skills/apate-cli`。
- GitHub Actions 多平台构建与 Release 发布流程。
- Rust-only 文档体系。

### Fixed

- `main` 分支构建产物会上传到 `latest` 预发布 Release，不再只停留在 Actions artifact。
- 默认还原前校验已知面具头，避免普通文件仅因尾部 4 字节像长度字段而被误判并写坏。
- 默认重命名前拒绝覆盖已有目标文件，避免 `disguise` / `reveal` 静默覆盖用户数据。
- `disguise --dry-run --mask-file` 复用正式执行的面具校验，空面具和超限面具不再被预检误报为成功。

### Removed

- 非 Rust 工程文件和非 Rust 文档入口。

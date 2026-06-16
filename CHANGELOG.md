# Changelog

## Unreleased

### Added

- Rust workspace：`apate-core` 和 `apate-cli`。
- 单程序多模式入口：`inspect`、`masks`、`disguise`、`reveal`、`tui`。
- 面向 agent 的 `skills/apate-cli`。
- Windows `apate.exe` 双击或无参数运行会打开拖拽 GUI，同时保留显式 `tui` 和 CLI 子命令模式。
- GUI 支持一次拖入多个文件，批量检查、伪装或还原，并汇总成功/失败数量。
- Android restore-only APK：手机端可选择通过 Apate 伪装的文件并还原，原地覆盖失败时提示另存。
- Android JNI 桥接 crate：复用 `apate-core` 处理文件描述符，不在 Kotlin 侧重新实现格式逻辑。
- GitHub Actions 多平台构建与 Release 发布流程。
- GitHub Actions 构建 Android APK 并把 APK 附件发布到 GitHub Releases；未配置 release keystore 时使用 debug 签名，方便用户手动下载安装。
- Rust-only 文档体系。
- 默认伪装命名会替换最后一个扩展名并记录原扩展名，例如 `secret.zip` -> `secret.jpg` -> `secret.zip`。
- 超大文件友好的伪装格式：只读写文件头、固定 128 KiB 尾部窗口和加密恢复元数据，不复制完整 payload。
- 使用 ChaCha20 加密恢复元数据，并混淆原始头尾窗口，降低原始格式被明文识别的概率。
- GUI 和文档说明 JPG 无法被图片查看器打开不代表源文件损坏；当前 JPG 模式是格式外观伪装，不是生成真实照片。
- `apate-core` 增加 stream/seekable API，支持路径以外的原地还原和另存恢复。

### Fixed

- `main` 分支构建产物会上传到 `latest` 预发布 Release，不再只停留在 Actions artifact。
- Android APK 不再因为缺少 release 签名 secrets 被跳过。
- Release 压缩包只包含对应平台可执行文件，不再把 `CHANGELOG.md` 打进附件目录。
- 默认还原前校验已知面具头，避免普通文件仅因尾部 4 字节像长度字段而被误判并写坏。
- 默认重命名前拒绝覆盖已有目标文件，避免 `disguise` / `reveal` 静默覆盖用户数据。
- `disguise --dry-run --mask-file` 复用正式执行的面具校验，空面具和超限面具不再被预检误报为成功。
- TUI 伪装复用 CLI 的重命名和输出冲突检查，避免只改写内容却不生成目标格式文件。
- Windows Release 构建目标改为 `x86_64-pc-windows-msvc`，生成更标准的 Windows 原生 exe。

### Removed

- 非 Rust 工程文件和非 Rust 文档入口。

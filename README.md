# Apate

Apate 是一个 Rust 文件格式伪装工具。它通过改写文件头和尾部元数据，把任意文件伪装成指定格式，并支持再还原为原始字节。

## 功能

- 单二进制入口：`inspect`、`masks`、`disguise`、`reveal`、`tui`。
- 支持内置 `exe`、`jpg`、`mp4`、`mov` 面具和自定义 `--mask-file`。
- 支持 `--dry-run --json`，适合脚本和 agent 自动化预检。
- 默认拒绝覆盖重命名目标，避免静默覆盖已有文件。
- 默认还原前会校验已知面具头，避免普通文件被误还原写坏。
- GitHub Actions 在 `main` 和 `v*` tag 上构建 Windows/Linux 产物；`v*` tag 会发布 GitHub Release。

## 构建与测试

```powershell
cargo test --workspace
cargo build --release --locked -p apate-cli
```

生成的二进制位于：

```text
target/release/apate.exe
```

Linux/macOS 下对应为：

```text
target/release/apate
```

## CLI 示例

```powershell
# 查看内置面具
cargo run -p apate-cli -- masks --json

# 检查文件是否可被默认还原
cargo run -p apate-cli -- inspect .\example.zip.mp4 --json

# 伪装为 JPG，默认追加 .jpg 扩展名
cargo run -p apate-cli -- disguise --input .\example.zip --kind jpg --json

# 一键伪装为 MP4
cargo run -p apate-cli -- disguise --input .\example.zip --one-key --json

# 批量处理前先 dry-run
cargo run -p apate-cli -- disguise --input .\files --kind mp4 --recursive --dry-run --json

# 还原文件，默认移除最后一个扩展名
cargo run -p apate-cli -- reveal --input .\example.zip.jpg --json

# 进入简易 TUI 菜单
cargo run -p apate-cli -- tui
```

## 项目结构

- `crates/apate-core`：文件伪装/还原算法、内置面具、输入文件收集。
- `crates/apate-cli`：单程序多模式 CLI/TUI 入口。
- `crates/apate-core/resources/mask.mp4`：`--one-key` 使用的内置 MP4 面具资源。
- `docs/`：架构、API、CLI、用户指南。
- `skills/apate-cli`：面向 agent 的 CLI 使用说明。

## 发布

`.github/workflows/release.yml` 会在 `main` push 和 `v*` tag push 时构建 Windows/Linux 产物。只有 `v*` tag 会创建 GitHub Release，并使用 `CHANGELOG.md` 的 `Unreleased` 段作为 Release Notes。

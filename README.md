# Apate

Apate 是一个 Rust 文件格式伪装工具。它通过改写文件头和尾部元数据，把任意文件伪装成图片、视频或可执行文件外观，并支持再还原为原始字节。主要用途是对抗百度网盘等网盘按文件格式、扩展名或文件头做的上传/分享封锁。

注意：Apate 做的是格式伪装和混淆，不是密码学加密。需要保护内容隐私时，建议先把文件打包成带密码的 zip/7z，再用 Apate 伪装成 jpg/mp4。

## 功能

- 单二进制多模式：双击/无参数运行进入交互菜单，`tui` 显式进入 TUI，子命令用于 CLI 自动化。
- 支持内置 `exe`、`jpg`、`mp4`、`mov` 面具和自定义 `--mask-file`。
- 支持 `--dry-run --json`，适合脚本和 agent 自动化预检。
- 默认替换最后一个扩展名，例如 `secret.zip` 伪装为 JPG 后得到 `secret.jpg`，一键 MP4 得到 `secret.mp4`。
- 默认拒绝覆盖重命名目标，避免静默覆盖已有文件。
- 默认还原前会校验已知面具头，避免普通文件被误还原写坏。
- GitHub Actions 在 `main` 和 `v*` tag 上构建 Windows/Linux 产物；`main` 会更新 `latest` 预发布 Release，`v*` tag 会发布正式 Release。

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

## 使用模式

- EXE 模式：Windows 上直接双击 `apate.exe`，或在终端运行 `apate`，会进入交互菜单。
- TUI 模式：运行 `apate tui`，显式进入同一个标准输入输出菜单。
- CLI 模式：运行 `apate inspect`、`apate masks`、`apate disguise`、`apate reveal` 等子命令，脚本和 agent 应优先搭配 `--json` 使用。

## CLI 示例

```powershell
# 查看内置面具
cargo run -p apate-cli -- masks --json

# 检查文件是否可被默认还原
cargo run -p apate-cli -- inspect .\example.zip.mp4 --json

# 伪装为 JPG，默认替换最后一个扩展名：example.zip -> example.jpg
cargo run -p apate-cli -- disguise --input .\example.zip --kind jpg --json

# 一键伪装为 MP4：example.zip -> example.mp4
cargo run -p apate-cli -- disguise --input .\example.zip --one-key --json

# 批量处理前先 dry-run
cargo run -p apate-cli -- disguise --input .\files --kind mp4 --recursive --dry-run --json

# 还原文件，默认恢复原扩展名：example.jpg -> example.zip
cargo run -p apate-cli -- reveal --input .\example.jpg --json

# 进入交互菜单；直接运行 apate 也会进入同一菜单
cargo run -p apate-cli -- tui
```

## 项目结构

- `crates/apate-core`：文件伪装/还原算法、内置面具、输入文件收集。
- `crates/apate-cli`：单程序多模式 CLI/TUI 入口。
- `crates/apate-core/resources/mask.mp4`：`--one-key` 使用的内置 MP4 面具资源。
- `docs/`：架构、API、CLI、用户指南。
- `skills/apate-cli`：面向 agent 的 CLI 使用说明。

## 发布

`.github/workflows/release.yml` 会在 `main` push 和 `v*` tag push 时构建 Windows/Linux 产物。压缩包只包含对应平台的可执行文件；`CHANGELOG.md` 只用于抽取 Release Notes，不会放进附件。`main` push 会把附件上传到 `latest` 预发布 Release；`v*` tag 会创建正式 GitHub Release。

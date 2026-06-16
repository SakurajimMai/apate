# Apate

Apate 是一个 Rust 文件格式伪装工具。它通过改写文件头、混淆固定大小尾部窗口、追加加密恢复元数据，把任意文件伪装成图片、视频或可执行文件外观，并支持再还原为原始字节。主要用途是对抗百度网盘等网盘按文件格式、扩展名、文件头或常见头尾签名做的上传/分享封锁。

注意：Apate 会用 ChaCha20 加密恢复元数据，并混淆原文件头和尾部窗口，但不会全量加密文件内容。需要保护内容隐私时，建议先把文件打包成带密码的 zip/7z，再用 Apate 伪装成 jpg/mp4。

## 功能

- 单二进制多模式：Windows 双击/无参数运行进入拖拽 GUI，`tui` 显式进入终端菜单，子命令用于 CLI 自动化。
- Android APK：手机端只做检查和还原，可直接恢复通过 Apate 伪装的文件；优先原地还原，文件提供方不支持时提示另存。
- 支持内置 `exe`、`jpg`、`mp4`、`mov` 面具和自定义 `--mask-file`。
- 支持 `--dry-run --json`，适合脚本和 agent 自动化预检。
- 支持超大文件：伪装/还原只读写文件头、最多 128 KiB 尾部窗口和少量元数据，不复制完整 payload。
- 恢复元数据使用 ChaCha20 加密保存，原扩展名、原始文件头和常见尾部签名不会以明文暴露。
- 默认替换最后一个扩展名，例如 `secret.zip` 伪装为 JPG 后得到 `secret.jpg`，一键 MP4 得到 `secret.mp4`。
- 默认拒绝覆盖重命名目标，避免静默覆盖已有文件。
- 默认还原前会校验已知面具头，避免普通文件被误还原写坏。
- GitHub Actions 在 `main` 和 `v*` tag 上构建 Windows/Linux 产物；`main` 会更新 `latest` 预发布 Release，`v*` tag 会发布正式 Release。
- GitHub Actions 同时构建 Android APK，并把 APK 与桌面附件一起上传到 Release；未配置 release 签名时会使用 debug 签名，便于用户手动下载安装。

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

- EXE/GUI 模式：Windows 上直接双击 `apate.exe`，会打开三栏拖拽窗口；可一次拖入多个文件，中间批量伪装，右侧批量还原。
- TUI 模式：运行 `apate tui`，显式进入标准输入输出菜单，适合临时终端操作。
- CLI 模式：运行 `apate inspect`、`apate masks`、`apate disguise`、`apate reveal` 等子命令，脚本和 agent 应优先搭配 `--json` 使用。
- Android 模式：安装 Release 里的 `apate-*-android.apk`，在手机上选择伪装文件并还原。第一版 APK 只支持还原，不支持手机端伪装。

## CLI 示例

```powershell
# 查看内置面具
cargo run -p apate-cli -- masks --json

# 检查文件是否可被默认还原
cargo run -p apate-cli -- inspect .\example.mp4 --json

# 伪装为 JPG，默认替换最后一个扩展名：example.zip -> example.jpg
cargo run -p apate-cli -- disguise --input .\example.zip --kind jpg --json

# 一键伪装为 MP4：example.zip -> example.mp4
cargo run -p apate-cli -- disguise --input .\example.zip --one-key --json

# 批量处理前先 dry-run
cargo run -p apate-cli -- disguise --input .\files --kind mp4 --recursive --dry-run --json

# 还原文件，默认恢复原扩展名：example.jpg -> example.zip
cargo run -p apate-cli -- reveal --input .\example.jpg --json

# 进入终端菜单；Windows 普通用户可直接双击 apate.exe 使用 GUI
cargo run -p apate-cli -- tui
```

### GUI 拖拽

Windows 上双击 release 附件里的 `apate.exe` 会打开拖拽窗口：

- 中间区域：按当前菜单选择的格式批量伪装，默认推荐 MP4，`secret.zip` 会变成 `secret.mp4`。
- 右侧区域：批量还原 Apate 文件，`secret.jpg` 会优先恢复成 `secret.zip`。
- 左侧区域：批量检查文件状态，并提示 JPG 打不开不等于源文件损坏。

一次拖入多个文件时，GUI 会逐个处理并在底部显示成功/失败数量；某个文件失败不会阻断其它文件。

注意：伪装成 `.jpg` 只是让文件头和扩展名呈现 JPG 外观，不等于生成真实照片。Windows 图片查看器打不开不代表原始文件损坏；只要用 Apate 还原后内容正常，源文件就是完整的。网盘场景默认优先用 MP4。

### Android APK

Android APK 面向只需要在手机上解开 Apate 文件的用户：

- 点击“选择文件”，从文件管理器、网盘目录或下载目录选择一个或多个 Apate 文件；
- APK 会先检查文件是否可还原，并显示建议恢复的文件名；
- 点击“还原”后优先尝试原地覆盖恢复，并尽量把文件名改回原扩展名；
- 如果 Android 文件提供方不允许原地写入、截断或重命名，APK 会弹出系统“另存为”面板，按原扩展名保存恢复文件。

APK 使用 Android 系统文件选择器，不申请全盘文件权限。只有用户主动选择的文件会被处理。

## 软件使用

> 完整教程见 [`docs/user-guide.md`](docs/user-guide.md)。这里给出一份快速指引。

### 三种伪装模式

```powershell
# 一键伪装为 MP4（适用最广）
apate disguise --input .\secret.zip --one-key --json

# 短面具伪装（只换 4–128 字节文件头）
apate disguise --input .\secret.zip --kind jpg --json
apate disguise --input .\secret.zip --kind exe --json
apate disguise --input .\secret.zip --kind mp4 --json
apate disguise --input .\secret.zip --kind mov --json

# 自定义 mask（用任意文件作为面具）
apate disguise --input .\secret.zip --mask-file .\my_mask.bin --json
```

### 还原

```powershell
apate reveal --input .\secret.jpg --json
```

默认会做两道安全检查：尾部 4 字节长度字段合法、文件头匹配已知面具。
任一不通过都会返回 `not_disguised` 并拒绝写入。

### 批量与预检

目录输入必须加 `--recursive`。批量处理前先 dry-run：

```powershell
apate disguise --input .\files --kind mp4 --recursive --dry-run --json
apate reveal --input .\disguised --recursive --dry-run --json
```

dry-run 模式下：

- 不写文件内容、不重命名；
- 仍会校验 mask（空 / 超限立即报错）；
- 仍会检查目标路径是否存在（已存在则返回 `output_exists`）；
- JSON 输出结构与正式执行完全一致，仅 `dry_run=true`。

### 默认命名规则

| 命令 | 默认行为 |
| --- | --- |
| `disguise --kind jpg` | `a.zip` → `a.jpg` |
| `disguise --one-key` | `a.zip` → `a.mp4` |
| `disguise --mask-file my.bin` | `a.zip` → `a.bin` |
| `disguise ... --no-rename` | 文件名不变 |
| `reveal` | `a.jpg` → `a.zip`（优先恢复记录的原扩展名）|
| `reveal --no-rename` | 文件名不变 |

### 常见错误处理

| code | 含义 | 处理 |
| --- | --- | --- |
| `not_disguised` | 文件未被识别为可默认还原 | 确认风险后才考虑 `--force` |
| `output_exists` | 重命名目标已存在 | 先移动 / 备份目标文件，或加 `--no-rename` |
| `empty_mask` | mask 为空 | 换一个非空文件作为 `--mask-file` |
| `mask_too_large` | mask 超过上限 | 截短 mask 或换内置短面具 |
| `directory_requires_recursive` | 目录输入未加 `--recursive` | 加 `--recursive` |
| `invalid_arguments` | `--one-key` / `--kind` / `--mask-file` 同时给多个 | 三选一 |

### 数据安全

- Apate **就地改写** 文件，没有撤销机制；
- 处理重要文件前请先备份；
- 批量前先 `--dry-run --json`；
- 写脚本时根据 `results[*].ok` 决定是否继续；
- Apate 加密的是恢复元数据与固定头尾窗口，不是完整内容保密层；要保护隐私请先用带密码的 zip / 7z 打包后再伪装。

## 项目结构

- `crates/apate-core`：文件伪装/还原算法、内置面具、输入文件收集。
- `crates/apate-cli`：单程序多模式 CLI/TUI 入口。
- `crates/apate-android-jni`：Android JNI 桥接层，把文件描述符交给 `apate-core` 还原。
- `android/`：Kotlin/Jetpack Compose 手机端还原 APK。
- `crates/apate-core/resources/mask.mp4`：`--one-key` 使用的内置 MP4 面具资源。
- `docs/`：架构、API、CLI、用户指南。
- `skills/apate-cli`：面向 agent 的 CLI 使用说明。

## 发布

`.github/workflows/release.yml` 会在 `main` push 和 `v*` tag push 时构建 Windows/Linux/Android 产物。压缩包只包含对应平台的可执行文件；`CHANGELOG.md` 只用于抽取 Release Notes，不会放进附件。`main` push 会把附件上传到 `latest` 预发布 Release；`v*` tag 会创建正式 GitHub Release。

Android APK 总会构建并上传到 Release。未配置签名密钥时，Gradle 会使用 debug 签名，用户可以手动下载并侧载安装；缺点是后续 APK 可能无法直接覆盖安装，用户需要先卸载已安装版本再安装新版。

如果希望用户后续能直接覆盖安装升级，可以在 GitHub Secrets 配置固定 release 签名：

- `ANDROID_KEYSTORE_BASE64`
- `ANDROID_KEYSTORE_PASSWORD`
- `ANDROID_KEY_ALIAS`
- `ANDROID_KEY_PASSWORD`

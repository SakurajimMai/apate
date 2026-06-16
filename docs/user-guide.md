# Apate 使用教程

本教程面向需要使用 apate 处理文件的人员。普通 Windows 用户可以直接双击 `apate.exe` 使用拖拽 GUI；开发者、脚本和 agent 使用 Rust 版 `apate` CLI。
Android 用户可以安装 Release 附件里的 APK，在手机上对通过 Apate 伪装的文件进行还原。

## 1. 它能做什么

Apate 通过改写文件头、混淆固定大小尾部窗口和加密恢复元数据，把任意文件的字节级外观变成另一种格式
（mp4 / jpg / exe / mov / 自定义 mask），用于对抗百度网盘等网盘按文件扩展名、
文件头、头尾签名或格式识别做的上传 / 分享封锁；并能在需要时 **1:1 还原** 回原始字节。

Apate 会加密恢复元数据，并混淆原始文件头和最后最多 128 KiB 尾部窗口；它不会全量加密文件内容。需要保护内容隐私时，建议先用 zip / 7z 等工具加密打包，再用 Apate 伪装成 jpg / mp4。

典型场景：

- 上传到只允许图片 / 视频的网盘，但原始是 zip / 7z 等压缩包；
- 通过按扩展名过滤的传输通道；
- 把可执行文件伪装成无害格式以避免被误报拦截；
- 备份 / 归档时把不同格式统一伪装成同一种外观。

Apate 只读写文件头、最后最多 128 KiB 尾部窗口和少量恢复元数据，不会复制完整 payload，因此适合超大文件。

## 2. 安装与首次运行

```powershell
# 从源码构建
cargo build --release --locked -p apate-cli

# 产物路径
# Windows: target\release\apate.exe
# Linux/macOS: target/release/apate
```

把二进制放到 `$PATH` 里的某个目录后即可直接调用 `apate`。验证安装：

```powershell
apate --version
apate masks --json
```

## 3. 新手 GUI 流程

Windows release 附件里的 `apate.exe` 可以直接双击打开。可以一次拖入一个或多个文件，窗口分为三块：

- 左侧绿色区域：批量检查文件是否像 Apate 伪装文件，并提示 JPG 打不开不等于源文件损坏；
- 中间黄色区域：把文件拖进去批量伪装，默认伪装为 MP4；
- 右侧紫色区域：把伪装后的文件拖进去批量还原。

菜单里的“选项”可以切换伪装格式：MP4、JPG、EXE、MOV。默认推荐 MP4，因为网盘对视频外观的兼容性通常比 JPG 更好。

GUI 和 CLI 使用同一套核心算法与命名规则：例如 `secret.zip` 默认伪装成 `secret.mp4` 或 `secret.jpg`，还原时优先恢复为 `secret.zip`。如果目标文件已经存在，GUI 也会拒绝覆盖。批量拖入时会逐个处理并汇总成功/失败数量；某个文件失败不会阻断其它文件。

## 4. CLI 基本流程

任何「修改文件」的操作都建议遵循三步：

1. `inspect`：先判断文件是不是由 Apate 伪装过的（通过默认校验后能直接还原）；
2. `--dry-run --json`：批量处理前先预检，确认输出路径、命名、mask 都没问题；
3. 去掉 `--dry-run` 真正执行。

跳过这三步而直接批量写入，是最常见的踩坑来源。

## 5. 三种伪装模式

### 5.1 一键伪装（推荐先用这个）

使用内置的完整 mp4 面具文件作为文件头。适用范围最广，对绝大多数「只能传视频」
的限制都能生效。

```powershell
apate disguise --input .\secret.zip --one-key --json
```

- 默认把 `secret.zip` 重命名为 `secret.mp4`；
- 文件头被替换成内置 mp4 完整结构。

### 5.2 短面具伪装

只把文件头换成另一种格式的「最小特征头」（4–128 字节）。优势是文件长度变化
最小；适用场景略窄，但对 EXE / JPG / MOV 这种「按头识别」的检查已经足够。

```powershell
apate disguise --input .\secret.zip --kind exe --json
apate disguise --input .\secret.zip --kind jpg --json
apate disguise --input .\secret.zip --kind mp4 --json   # 等价于一键但只换 32 字节头
apate disguise --input .\secret.zip --kind mov --json
```

- `--kind mp4` 和 `--one-key` 的区别：前者只写 32 字节 mp4 头，后者写完整 mp4 模板。
  简单限制场景两者都能过；遇到严格 mp4 解析时优先用 `--one-key`。

### 5.3 自定义 mask

用任意二进制文件作为面具（取 `--mask-file` 的全部字节）：

```powershell
apate disguise --input .\secret.zip --mask-file .\my_mask.bin --json
```

默认会按 mask 文件的扩展名替换原文件最后一个扩展名，例如 `secret.zip` 变成
`secret.bin`。

约束：

- mask 不能为空；
- mask 长度不能超过约 306 MB（`MAXIMUM_MASK_LENGTH = 2_147_483_647 / 7`）。
- 不论是否带 `--dry-run`，以上校验都会被强制执行。

## 6. 文件名变化规则

| 命令 | 默认行为 |
| --- | --- |
| `disguise --kind jpg` | `a.zip` → `a.jpg` |
| `disguise --one-key` | `a.zip` → `a.mp4` |
| `disguise --mask-file my.bin` | `a.zip` → `a.bin` |
| `disguise ... --no-rename` | 文件名不变 |
| `reveal` | `a.jpg` → `a.zip`（优先恢复记录的原扩展名）|
| `reveal --no-rename` | 文件名不变 |

缺少原扩展名元数据的文件会退回到移除最后一个扩展名。

## 7. 还原

```powershell
# 标准还原
apate reveal --input .\secret.jpg --json

# 不改名，只改内容
apate reveal --input .\secret.jpg --no-rename --json
```

默认安全检查：

- **文件头必须匹配已知面具**：与所有内置面具头（含一键伪装 mp4）逐个比对；
- **尾部 4 字节长度字段必须合法**。

任意一项不通过都会返回 `not_disguised`，不写任何字节。

只有完全确认要强行尝试（比如想抢救一个尾部字节看起来对但头被改过的文件）时才
使用 `--force`。`--force` 只跳过 mask 头校验，仍要求尾部 4 字节合法。

## 8. 批量处理

目录输入必须加 `--recursive`，否则命令会直接拒绝并返回
`directory_requires_recursive`。

```powershell
# 整个目录伪装为 mp4
apate disguise --input .\files --kind mp4 --recursive --json

# 先 dry-run 看一遍计划
apate disguise --input .\files --kind mp4 --recursive --dry-run --json

# 批量还原
apate reveal --input .\disguised --recursive --json
```

dry-run 模式下：

- 不会写入文件内容；
- 会校验 mask（空 / 超限会立即报错）；
- 会检查目标路径是否存在（已存在则返回 `output_exists`）；
- 输出 JSON 结构与正式执行完全一致，仅 `dry_run=true`。

## 9. 检查文件状态

```powershell
# 单个文件
apate inspect .\secret.jpg --json
```

返回：

```json
{
  "path": "secret.jpg",
  "disguised": true,
  "mask_length": 32,
  "payload_length": 1024
}
```

- `disguised = false` 时，这个文件无法被默认还原（可能根本不是 apate 伪装文件，
  或者用了自定义 mask）；
- `disguised = true` 时，可以放心调用 `reveal`。

## 10. TUI 交互模式

适合临时手动操作，不适合脚本：

```powershell
apate tui
```

菜单选项：

```
1) inspect
2) masks
3) disguise
4) reveal
0) exit
```

选 1–4 之后会提示输入路径与参数，操作完会回到菜单，选择 `0` 才退出。脚本和 agent
**不要使用 TUI**，统一用 `--json` 子命令。

## 11. Android 手机端还原

Release 附件里的 `apate-*-android.apk` 用于手机端还原。第一版 APK 是 restore-only：只检查和还原通过 Apate 伪装的文件，不在手机上执行伪装。

使用流程：

1. 安装 APK。
2. 点击“选择文件”，从文件管理器、网盘目录或下载目录选择一个或多个文件。
3. APK 会先检查文件是否为 Apate 文件，并显示建议恢复的文件名。
4. 点击“还原”。
5. 如果系统允许原地写入，文件会直接恢复，并尽量改回原扩展名；如果不允许，APK 会弹出系统“另存为”面板。

Android 版不申请全盘文件权限，只处理用户通过系统选择器主动交给它的文件。不同文件来源对原地覆盖和重命名的支持不一致：本地文件管理器通常更容易成功，部分网盘或相册提供方可能只允许读或只允许另存。若内容已恢复但改名失败，手动把扩展名改回原格式即可。

## 12. 错误处理速查

| code | 含义 | 处理方式 |
| --- | --- | --- |
| `ok` | 成功 | — |
| `dry_run` | 仅预检记录 | 确认结果后去掉 `--dry-run` 重跑 |
| `io_error` | 文件系统错误 | 检查权限、磁盘空间、文件占用 |
| `empty_mask` | mask 为空 | 换一个非空文件作为 `--mask-file` |
| `mask_too_large` | mask 超过上限 | 截短 mask 或换内置短面具 |
| `not_disguised` | 文件未被识别为可默认还原 | 确认风险后才考虑 `--force` |
| `output_exists` | 目标路径已存在 | 先移动 / 备份目标文件，或加 `--no-rename` |
| `invalid_arguments` | 参数组合无效 | `--one-key` / `--kind` / `--mask-file` 三选一 |
| `missing_path` | 输入路径不存在 | 检查路径拼写 |
| `directory_requires_recursive` | 目录输入未加 `--recursive` | 加 `--recursive` |

## 13. 故障排查

### 「`not_disguised`：文件不是有效的 Apate 伪装文件」

可能原因：

1. 从未用 apate 伪装过这个文件；
2. 文件被外部工具修改过头部或尾部；
3. 用了 `--mask-file my.bin` 自定义 mask，事后又丢失了 `my.bin`，无法还原
   （自定义 mask 不在白名单里）；
4. 仅尾部 4 字节碰巧是合法长度但头不是任何已知面具（这是默认检查会拒绝的情况）。

可以加 `--force` 强行尝试，但结果大概率损坏。

### 「`output_exists`：输出路径已存在」

apate 默认拒绝覆盖已有文件以防误删。处理：

1. 手工删除或重命名已存在的目标；
2. 加 `--no-rename` 跳过 rename（但内容仍会被改写）；
3. 确认要覆盖后再重跑。

### dry-run 通过但正式执行失败

dry-run 会复用 mask 校验和路径检查，所以这种情况只可能来自 IO 层面的差异：
权限、文件被其他进程占用、磁盘空间不足等。检查环境后再重跑。

### 伪装成 JPG 后图片查看器打不开

这不代表原始文件损坏。

Apate 当前的 JPG 伪装是格式外观伪装：它会改写文件头、扩展名和恢复元数据，让网盘或传输通道看到 JPG 外观；它不会把压缩包真的转换成一张可显示的照片。因此 Windows 图片查看器提示“无效的位图文件或不支持格式”是正常现象。

判断文件是否损坏只看还原结果：如果 `apate reveal --input .\secret.jpg --json` 后恢复出 `secret.zip`，并且压缩包能正常打开，说明原文件没有损坏。若确实需要“能被图片查看器打开的真实图片外壳”，那属于另一种载体格式策略，不是当前默认短面具模式。

### 还原后文件打不开

最常见原因：

1. 伪装后文件被二次修改；
2. 用了 `--force` 对非伪装文件强行还原；
3. 伪装文件缺少原扩展名元数据，只能退回到移除最后一个扩展名；
4. 自定义 mask 与原 mask 不一致。

恢复策略：保持原始文件未被破坏的前提下，重新 `reveal` 应能回到原始字节。

## 14. 最佳实践

- **始终先 inspect 再 reveal**：避免对非伪装文件写入；
- **始终先 dry-run 再批量写入**：避免对目录误改；
- **始终确保目标路径无冲突**：避免 `output_exists` 中断批量；
- **敏感内容先加密再伪装**：Apate 解决格式封锁，不提供完整内容保密；
- **agent / 脚本只解析 `--json`**：人类可读输出格式可能调整；
- **备份原始数据**：apate 就地改写，无法撤销。

## 15. 数据安全

Apate 会就地改写文件，无回收站机制。建议：

- 处理重要文件前先做一次完整备份；
- 批量处理前先 `--dry-run --json`；
- 写脚本时根据 `results[*].ok` 决定是否继续；
- 遇到 `output_exists` 时先停下来确认，再决定是否覆盖。

## 16. 进阶：读 `BatchOutput`

`disguise` / `reveal` 共用以下结构：

```json
{
  "ok": true,
  "dry_run": false,
  "results": [
    {
      "action": "disguise",
      "path": "a.zip",
      "output_path": "a.jpg",
      "ok": true,
      "code": "ok",
      "message": "处理成功"
    }
  ]
}
```

字段：

- `ok`（顶层）：所有 `results[*].ok` 的 AND。批量里只要有一个失败就是 `false`，
  进程退出码也是 `1`。
- `dry_run`：当前批次是否为预检。
- `results[*].path`：输入路径。
- `results[*].output_path`：重命名后的目标路径；失败 / `--no-rename` 时为 `null`。
- `results[*].code`：见上文错误处理速查表。
- `results[*].message`：人类可读说明。

完整 JSON schema 与错误码定义见 [`cli-reference.md`](./cli-reference.md)。

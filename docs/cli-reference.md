# apate CLI 参考

二进制名：`apate`

## 全局约定

- 除 `tui` 外，子命令都支持 `--json`。
- 批量命令在 JSON 模式下即使失败也会向 stdout 输出可解析 JSON，并以退出码 `1` 结束。
- `disguise` 和 `reveal` 默认会重命名文件；需要只改内容时使用 `--no-rename`。
- 默认重命名目标已存在时返回 `output_exists`，不会覆盖已有文件。
- `--dry-run` 不写文件，但会校验 mask 和输出路径。
- Windows 交互终端里直接运行 `apate` 会进入拖拽 GUI；管道输入或非 Windows 环境会回退到 TUI。
- 默认命名会隐藏原扩展名：`secret.zip` 伪装为 JPG 后得到 `secret.jpg`，还原时再恢复为 `secret.zip`。

## `apate inspect <PATH> [--json]`

只读检查文件是否可被默认还原。

JSON 输出：

```json
{
  "path": "example.zip.mp4",
  "disguised": true,
  "mask_length": 32,
  "payload_length": 1024
}
```

## `apate masks [--json]`

列出内置短面具。

```json
{
  "masks": [
    { "kind": "exe", "extension": ".exe", "length": 128 },
    { "kind": "jpg", "extension": ".jpg", "length": 4 },
    { "kind": "mp4", "extension": ".mp4", "length": 32 },
    { "kind": "mov", "extension": ".mov", "length": 4 }
  ]
}
```

## `apate disguise [flags]`

必填：

- `--input <PATH>`
- `--one-key`、`--kind <exe|jpg|mp4|mov>`、`--mask-file <PATH>` 三选一

可选：

- `--recursive`
- `--no-rename`
- `--dry-run`
- `--json`

示例：

```powershell
apate disguise --input .\a.zip --kind jpg --json
apate disguise --input .\a.zip --one-key --json
apate disguise --input .\a.zip --mask-file .\mask.bin --json
apate disguise --input .\files --kind mp4 --recursive --dry-run --json
```

默认命名：

- `--kind jpg`：`a.zip` -> `a.jpg`
- `--one-key`：`a.zip` -> `a.mp4`
- `--mask-file mask.bin`：`a.zip` -> `a.bin`

## `apate reveal [flags]`

必填：

- `--input <PATH>`

可选：

- `--recursive`
- `--no-rename`
- `--force`
- `--dry-run`
- `--json`

默认命名：

- `a.jpg` -> `a.zip`（新格式会记录原扩展名）
- 没有原扩展名元数据的文件会退回到移除最后一个扩展名

安全说明：

- 默认会先调用 `inspect_file`，未识别文件返回 `not_disguised`。
- `--force` 只在明确接受风险时使用。

## `apate` GUI

Windows 普通用户可以直接双击 `apate.exe`，或在交互终端无参数运行 `apate`。GUI 提供三块拖拽区域：

- 左侧：检查文件状态；
- 中间：按当前菜单选择的格式伪装，默认 MP4；
- 右侧：还原 Apate 文件。

GUI 复用 CLI 的核心算法、默认命名和输出冲突检查。

## `apate tui`

标准输入输出菜单，不支持 `--json`。适合临时终端操作；脚本和 agent 不应使用 TUI。

菜单提供：

1. `inspect`
2. `masks`
3. `disguise`
4. `reveal`
0. `exit`

## Batch JSON

`disguise` 和 `reveal` 共用结构：

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

## 错误码

| code | 含义 |
| --- | --- |
| `ok` | 处理成功 |
| `dry_run` | 预检记录 |
| `io_error` | 文件系统错误 |
| `empty_mask` | mask 为空 |
| `mask_too_large` | mask 超过上限 |
| `not_disguised` | 未识别为可默认还原的伪装文件 |
| `output_exists` | 输出路径已存在 |
| `invalid_arguments` | 参数组合无效 |
| `missing_path` | 输入路径不存在 |
| `directory_requires_recursive` | 目录输入需要 `--recursive` |

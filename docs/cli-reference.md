# apate CLI 参考

二进制名：`apate`

## 全局约定

- 除 `tui` 外，子命令都支持 `--json`。
- 批量命令在 JSON 模式下即使失败也会向 stdout 输出可解析 JSON，并以退出码 `1` 结束。
- `disguise` 和 `reveal` 默认会重命名文件；需要只改内容时使用 `--no-rename`。
- 默认重命名目标已存在时返回 `output_exists`，不会覆盖已有文件。
- `--dry-run` 不写文件，但会校验 mask 和输出路径。

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

- `--kind jpg`：`a.zip` -> `a.zip.jpg`
- `--one-key`：`a.zip` -> `a.zip.mp4`
- `--mask-file mask.bin`：`a.zip` -> `a.zip.bin`

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

- `a.zip.jpg` -> `a.zip`

安全说明：

- 默认会先调用 `inspect_file`，未识别文件返回 `not_disguised`。
- `--force` 只在明确接受风险时使用。

## `apate tui`

标准输入输出菜单，不支持 `--json`。菜单提供：

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
      "output_path": "a.zip.jpg",
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

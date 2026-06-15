# `apate` CLI 参考

> 路径：`crates/apate-cli/src/main.rs`
> 依赖：`apate-core`、`clap = "4"`（derive, std, help, usage, error-context）、
> `serde`、`serde_json`
> 二进制名：`apate`

CLI 是 `apate-core` 的用户面。所有子命令都接受 `--json`（除 `tui`），输出结构稳定，
便于 agent / 脚本解析。

## 全局约定

- 不传子命令时打印顶层 help 并以退出码 `0` 退出。
- 任意子命令出错时把 `ApateError` 写到 stderr 并以退出码 `1` 退出。
- 批量子命令（`disguise` / `reveal`）在 **JSON 模式** 下始终写合法 JSON 到 stdout，
  即便 `ok=false`；退出码仍为 `1` 但 JSON 仍可被解析。
- 命名行为：默认修改文件名（追加面具扩展名或去除最后一个扩展名）；加 `--no-rename`
  可只改内容。

## 子命令索引

| 子命令      | 用途                                  | 是否支持 `--json` |
| ----------- | ------------------------------------- | ----------------- |
| `inspect`   | 判定文件是否为旧格式伪装文件          | 是                |
| `masks`     | 列出所有内置面具                      | 是                |
| `disguise`  | 伪装文件或目录                        | 是                |
| `reveal`    | 还原文件或目录                        | 是                |
| `tui`       | 极简 stdin/stdout 交互菜单            | **否**（传了会报错） |

---

## `apate inspect <PATH> [--json]`

判断 `<PATH>` 是否为旧格式伪装文件。**只读**。

### 参数

| 位置 / flag | 类型      | 说明                                       |
| ----------- | --------- | ------------------------------------------ |
| `<PATH>`    | 必填位置  | 待检查文件路径                             |
| `--json`    | flag      | 以 JSON 输出，否则人类可读                 |

### 退出码

- `0`：成功（不论是否识别为伪装文件）
- `1`：IO 错误

### 人类可读输出

```
example.zip.mp4: 旧格式伪装文件
```

### JSON 输出

```json
{
  "path": "example.zip.mp4",
  "disguised": true,
  "mask_length": 32,
  "payload_length": 17
}
```

字段说明：

| 字段            | 类型              | 说明                                       |
| --------------- | ----------------- | ------------------------------------------ |
| `path`          | string            | 输入路径（字符串形式）                     |
| `disguised`     | bool              | 是否被识别为旧格式伪装文件                 |
| `mask_length`   | number \| null    | 面具字节数；`disguised=false` 时为 `null` |
| `payload_length`| number \| null    | 原文件剩余字节数；同上                     |

---

## `apate masks [--json]`

列出所有内置面具及其字节长度。**只读**。

### 参数

| flag     | 说明                              |
| -------- | --------------------------------- |
| `--json` | 以 JSON 输出                       |

### 人类可读输出

```
exe    .exe    128 bytes
jpg    .jpg    4 bytes
mp4    .mp4    32 bytes
mov    .mov    4 bytes
```

### JSON 输出

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

---

## `apate disguise [flags]`

把真身文件伪装成另一种格式。**会就地修改文件**，并（默认）改名。

### flag 总览

| flag          | 取值                | 说明                                                       |
| ------------- | ------------------- | ---------------------------------------------------------- |
| `--input`     | 必填路径            | 单文件或目录                                               |
| `--one-key`   | flag                | 一键伪装（固定 mp4，依赖 `apate/Resources/mask.mp4`）      |
| `--kind`      | `exe\|jpg\|mp4\|mov` | 使用对应内置面具头                                        |
| `--mask-file` | 路径                | 自定义面具文件（读取全部字节）                             |
| `--recursive` | flag                | 当 `--input` 是目录时递归处理                              |
| `--no-rename` | flag                | 只改内容，不改文件名                                       |
| `--dry-run`   | flag                | 仅打印计划，不写文件、不改名                               |
| `--json`      | flag                | 以 JSON 输出 batch 结果                                    |

### 互斥规则

`--one-key` / `--kind` / `--mask-file` 必须 **恰好选择其一**；选 0 个或选 ≥2 个
都会报错并以退出码 `1` 退出：

```
必须且只能选择一种面具来源: --one-key、--kind 或 --mask-file
```

### 默认命名行为

- `disguise` 默认追加面具扩展名：
  - `a.zip` + `--kind jpg` → `a.zip.jpg`
  - `a.zip` + `--one-key` → `a.zip.mp4`
  - `a.zip` + `--mask-file my.bin` → `a.zip.bin`（取 mask 文件的扩展名）
- 加 `--no-rename` 则只改内容不重命名。

### JSON 输出（`BatchOutput`）

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

字段说明：

| 字段          | 类型             | 说明                                                                 |
| ------------- | ---------------- | -------------------------------------------------------------------- |
| `ok`          | bool             | 顶层状态：所有 `results[*].ok` 的 AND                                |
| `dry_run`     | bool             | 是否为 dry-run（dry-run 时所有结果 `ok=true`，`code="dry_run"`）     |
| `results`     | 数组             | 单文件处理记录                                                       |
| `action`      | string           | 固定为 `"disguise"` 或 `"reveal"`                                    |
| `path`        | string           | 输入路径                                                             |
| `output_path` | string \| null   | 重命名后的目标路径；`--no-rename` 或失败时为 `null`                  |
| `ok`          | bool             | 单文件级成功标记                                                     |
| `code`        | string           | `ok` / `dry_run` / `io_error` / `empty_mask` / `mask_too_large` / `not_disguised` / `missing_path` / `directory_requires_recursive` |
| `message`     | string           | 人类可读消息                                                         |

### 退出码

- `0`：所有文件成功
- `1`：至少一个文件失败（`BatchOutput.ok == false`）

### 例子

```powershell
# 简易伪装为 JPG
apate disguise --input .\a.zip --kind jpg --json

# 一键 mp4 面具
apate disguise --input .\a.zip --one-key --json

# 自定义面具
apate disguise --input .\a.zip --mask-file .\my.bin --json

# 批量目录（先 dry-run）
apate disguise --input .\files --kind mp4 --recursive --dry-run --json

# 只改内容不改文件名
apate disguise --input .\a.zip --kind jpg --no-rename --json
```

---

## `apate reveal [flags]`

把旧格式伪装文件还原为原始字节。**会就地修改文件**，并（默认）改回原始文件名。

### flag 总览

| flag          | 类型     | 说明                                                       |
| ------------- | -------- | ---------------------------------------------------------- |
| `--input`     | 必填路径 | 单文件或目录                                               |
| `--recursive` | flag     | 当 `--input` 是目录时递归处理                              |
| `--no-rename` | flag     | 只改内容，不动文件名                                       |
| `--force`     | flag     | 跳过 `inspect` 安全检查；**只在用户明确接受风险时使用**    |
| `--dry-run`   | flag     | 仅打印计划，不写文件、不改名                               |
| `--json`      | flag     | 以 JSON 输出 batch 结果                                    |

### 默认命名行为

- `reveal` 默认移除最后一个扩展名：
  - `a.zip.jpg` → `a.zip`
- 加 `--no-rename` 则只改内容。

### 安全检查

默认会先 `inspect_file`：如果目标不被识别为旧格式伪装文件，返回 `NotDisguised`
（`code="not_disguised"`）并不写文件。`--force` 会跳过这一步，**对非伪装文件执行
还原会得到损坏数据**，使用前请确认风险。

### JSON 输出

与 `disguise` 共享 `BatchOutput` 形态，仅 `action` 字段变为 `"reveal"`。

### 退出码

同 `disguise`。

### 例子

```powershell
# 还原（默认改回原文件名）
apate reveal --input .\a.zip.jpg --json

# 不改名
apate reveal --input .\a.zip.jpg --no-rename --json

# 批量目录
apate reveal --input .\disguised --recursive --dry-run --json
```

---

## `apate tui`

极简交互菜单，进入后选择 1–4 进入一次性提示模式（inspect / masks / disguise /
reveal），完成后回到 shell。

- 输入 `0` / `q` / `quit` / `exit` 立即退出。
- 不接受 `--json`；传了会立即报错并以退出码 `1` 退出：

  ```
  tui 模式不支持 --json
  ```

### TUI 中的可选面具

```
输入面具类型(exe/jpg/mp4/mov/onekey):
```

- `exe` / `jpg` / `mp4` / `mov` —— 使用 `apate-core` 内置面具头
- `onekey` —— 一键伪装，使用 `apate/Resources/mask.mp4`

---

## 错误码速查

CLI 把 `ApateError` 映射成以下 `code` 字符串，便于 agent 直接匹配：

| `code`                          | 含义                          |
| ------------------------------- | ----------------------------- |
| `ok`                            | 处理成功                      |
| `dry_run`                       | 仅 dry-run 记录，未实际写入   |
| `io_error`                      | 文件系统读写错误              |
| `empty_mask`                    | 面具字节为空                  |
| `mask_too_large`                | 面具超过最大允许字节数        |
| `not_disguised`                 | 目标文件不是旧格式伪装文件    |
| `missing_path`                  | 输入路径不存在                |
| `directory_requires_recursive`  | 输入是目录但未传 `--recursive` |

## agent / 脚本最佳实践

来自 `skills/apate-cli/SKILL.md`：

1. 先 `inspect --json` 再决定是否 `reveal`；
2. 批量目录先 `--dry-run --json`；
3. 解析 `--json` 输出，不要解析人类可读文本；
4. 未识别文件不要默认加 `--force`；
5. agent 默认不进 `tui`。
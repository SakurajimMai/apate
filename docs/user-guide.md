# Apate 使用指南

> 这是一份面向终端用户的指南。如果你打算让 agent 自动化调用，请改读
> [`cli-reference.md`](./cli-reference.md) 与 [`skills/apate-cli/SKILL.md`](../skills/apate-cli/SKILL.md)。

## 它能做什么

apate 可以把任意文件的字节级外观改写成另一种格式的「面具」，从而在不支持
文件名扩展名限制、或者只检查文件头的场景下绕过限制。它也可以把伪装文件
**1:1 还原**回原始字节，不依赖你记住当初用了什么面具。

## 三种伪装模式

### 1. 一键伪装（推荐先试这个）

使用预置的 mp4 面具文件，适用范围最广，几乎所有「不允许上传 zip」的场景都能绕过。

```powershell
apate disguise --input .\secret.zip --one-key --json
```

效果：`secret.zip` → `secret.zip.mp4`，文件头看起来像一个完整的 mp4。

### 2. 面具伪装

使用「仅文件头」的内置面具（4–128 字节）。适用于 EXE / JPG / MOV 这种「只要头对」
就能绕过限制的场景。

```powershell
# EXE 面具
apate disguise --input .\secret.zip --kind exe --json

# JPG 面具（伪装后看起来像 JPEG）
apate disguise --input .\secret.zip --kind jpg --json

# MOV 面具
apate disguise --input .\secret.zip --kind mov --json

# 自定义面具（任意二进制文件，文件头按你给的算）
apate disguise --input .\secret.zip --mask-file .\my.bin --json
```

### 3. 简易伪装

「面具伪装」中只填文件头的那一类。EXE 的简易伪装建议真身文件不超过 2 GB。
其他格式的简易伪装适配场景较少，不建议使用。

## 还原

```powershell
# 默认会移除最后一个扩展名：a.zip.jpg → a.zip
apate reveal --input .\a.zip.jpg --json

# 只改内容不改文件名
apate reveal --input .\a.zip.jpg --no-rename --json
```

> ⚠️ `reveal` 默认会先做 `inspect` 检查。如果目标文件不是旧格式伪装文件，
> 会报 `not_disguised` 并拒绝写入。只有当你完全确认风险时才使用 `--force`。

## 批量处理

`--input` 既可以是单个文件，也可以是目录。目录模式必须加 `--recursive`：

```powershell
# 批量伪装：先 dry-run 看一遍计划
apate disguise --input .\files --kind mp4 --recursive --dry-run --json

# 确认无误后去掉 --dry-run
apate disguise --input .\files --kind mp4 --recursive --json

# 批量还原
apate reveal --input .\disguised --recursive --json
```

dry-run 的输出与正式执行的 JSON 结构完全一致，仅 `dry_run=true` 且不会动文件。

## 交互模式

不想记命令的话，可以：

```powershell
apate tui
```

进入菜单后选 1–4，按提示输入路径与面具即可。`tui` 是一次性提示模式，不是常驻界面。

## 输出格式

每个非 `tui` 子命令都支持 `--json`。脚本/agent 必须使用 `--json`；人类阅读可以省略。

```json
{
  "ok": true,
  "dry_run": false,
  "results": [
    {
      "action": "disguise",
      "path": "secret.zip",
      "output_path": "secret.zip.mp4",
      "ok": true,
      "code": "ok",
      "message": "处理成功"
    }
  ]
}
```

详细字段定义见 [`cli-reference.md`](./cli-reference.md)。

## 文件名变化规则

| 操作                            | 默认行为                              |
| ------------------------------- | ------------------------------------- |
| `disguise --kind jpg`           | `a.zip` → `a.zip.jpg`                 |
| `disguise --one-key`            | `a.zip` → `a.zip.mp4`                 |
| `disguise --mask-file my.bin`   | `a.zip` → `a.zip.bin`                 |
| `disguise --no-rename ...`      | 文件名不变                            |
| `reveal`                        | `a.zip.jpg` → `a.zip`（去最后一个扩展名）|
| `reveal --no-rename ...`        | 文件名不变                            |

## 故障排查

### 「文件不是有效的旧格式伪装文件」

`reveal` 默认会先 `inspect`。说明这个文件从来没用过 apate 伪装过，
或者已经在外部被改过。如果你完全确认要强行还原（结果可能损坏），加 `--force`。

### 「不支持非递归处理文件夹」

你把目录直接传给了 `disguise` / `reveal`。加 `--recursive`：

```powershell
apate disguise --input .\folder --kind jpg --recursive --json
```

### 「路径不存在」

`--input` 指向了不存在的文件/目录。先检查路径是否存在、相对/绝对路径是否正确。

### 还原后文件打不开

最常见原因：

1. 还原前文件被外部工具二次改写过；
2. 用了 `--force` 对非伪装文件强行还原；
3. 伪装后又被改名或移动过，导致 `reveal` 找错了文件。

恢复策略：原始文件 + apate 还原链路都没有损坏的话，重新执行一次 `reveal`
就能回到原始字节。

## FAQ

### 1. `copy /b a.jpg + b.zip` 跟 apate 一样吗？

技术上不一样。`copy /b` 直接拼接、不加密原文件头；apate 把原文件头**反转并加密
（倒序）**后再追加，因此从伪装文件里读不出原始格式。

### 2. 一键伪装为什么只有 mp4？

mp4 适用范围最广。如果需要其他格式，用面具伪装 + `--kind exe`/`--kind jpg`/
`--kind mov` 或者自定义面具 `--mask-file`。

### 3. 能不能批量还原一个混合目录？

可以，`apate reveal --input <dir> --recursive --dry-run --json` 会列出每个文件
的处理结果；非伪装文件会被单独标记 `code="not_disguised"`。

### 4. 还原后文件名不对怎么办？

`reveal` 默认只去掉最后一个扩展名。如果你的命名是 `a.zip.png.jpg`，它会变回
`a.zip.png`；如果你本来就想还原成 `a.zip`，先去掉 `.png` 再 `reveal`，或者
加 `--no-rename` 之后手工改名。

### 5. apate 跟杀毒软件冲突吗？

apate 不打包、不自启动、不联网，本身就是普通的文件读写工具。但被伪装的文件
可能携带任何内容，杀软仍会按内容判定。

## 数据安全与免责

- 使用前请务必做好数据备份。
- 本软件仅供学习交流，不得用于商业用途。
- 不得用于任何非法用途。用户使用本软件导致的任何后果均由用户本人承担。
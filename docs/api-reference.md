# apate-core API 参考

路径：`crates/apate-core/src/lib.rs`

`apate-core` 是 Rust 核心库，负责字节级伪装/还原算法，不依赖 CLI 或终端 UI。

## 常量

- `MASK_LENGTH_INDICATOR_LENGTH: u64 = 4`
- `MAXIMUM_MASK_LENGTH: u64 = 2_147_483_647 / 7`

## 类型

### `MaskKind`

```rust
pub enum MaskKind {
    Exe,
    Jpg,
    Mp4,
    Mov,
}
```

### `BuiltinMask`

```rust
pub struct BuiltinMask {
    pub kind: MaskKind,
    pub name: &'static str,
    pub extension: &'static str,
    pub bytes: &'static [u8],
}
```

### `Inspection`

```rust
pub struct Inspection {
    pub disguised: bool,
    pub mask_length: Option<u32>,
    pub payload_length: Option<u64>,
}
```

### `ApateError`

| 变体 | 含义 |
| --- | --- |
| `Io` | 文件系统读写错误 |
| `EmptyMask` | 面具字节为空 |
| `MaskTooLarge` | 面具超过最大长度 |
| `NotDisguised` | 文件未被识别为可默认还原的伪装文件 |
| `OutputExists` | 输出路径已存在 |
| `InvalidArguments` | 调用方传入语义错误的参数 |
| `MissingPath` | 输入路径不存在 |
| `DirectoryRequiresRecursive` | 输入是目录但未启用递归 |

## 函数

### `builtin_masks() -> &'static [BuiltinMask]`

返回所有内置短面具。

### `builtin_mask(kind: MaskKind) -> BuiltinMask`

按类型返回单个内置面具。

### `one_key_mask() -> &'static [u8]`

返回 `--one-key` 使用的内置 MP4 面具。资源位于 `crates/apate-core/resources/mask.mp4`。

### `validate_mask(mask: &[u8]) -> Result<()>`

校验面具非空且不超过 `MAXIMUM_MASK_LENGTH`。

### `disguise_file(path, mask) -> Result<()>`

就地把文件伪装成指定面具格式。

流程：

1. 校验 mask。
2. 读取原文件前 `min(file_len, mask.len())` 字节。
3. 从文件头写入 mask。
4. 在文件尾部追加倒序原文件头。
5. 追加 4 字节 little-endian 面具长度。

### `inspect_file(path) -> Result<Inspection>`

只读检查文件是否可被默认还原。它会同时检查：

1. 尾部 4 字节面具长度字段合法。
2. 文件头匹配内置面具或 `one_key_mask()`。

任意一项不满足时返回 `Inspection { disguised: false, ... }`。

### `reveal_file(path, force) -> Result<()>`

就地还原伪装文件。

- `force=false`：先执行 `inspect_file` 安全检查。
- `force=true`：跳过已知面具头检查，但仍依赖合法尾部长度字段。

### `collect_input_files(path, recursive) -> Result<Vec<PathBuf>>`

把单文件或目录解析为待处理文件列表。目录输入必须设置 `recursive=true`。

## 示例

```rust
use apate_core::{builtin_mask, disguise_file, inspect_file, reveal_file, MaskKind};

let path = "secret.zip";
disguise_file(path, builtin_mask(MaskKind::Jpg).bytes)?;

let inspection = inspect_file(path)?;
assert!(inspection.disguised);

reveal_file(path, false)?;
```

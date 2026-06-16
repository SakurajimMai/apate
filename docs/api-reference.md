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

### `SeekableFile`

```rust
pub trait SeekableFile: Read + Write + Seek {
    fn set_target_len(&mut self, len: u64) -> io::Result<()>;
}
```

用于把还原逻辑从路径 API 抽出来。`fs::File` 和测试用 `Cursor<Vec<u8>>` 已实现该 trait；Android JNI 可以把文件描述符转换为 `File` 后复用同一套还原逻辑。

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
3. 读取原文件最后最多 128 KiB 尾部窗口。
4. 用 ChaCha20 混淆尾部窗口，再从文件头写入 mask。
5. 追加 ChaCha20 加密恢复元数据和 4 字节 little-endian 面具长度。

该流程不会复制完整文件内容，适合超大文件；处理时间主要取决于 mask 长度、128 KiB 尾部窗口和恢复元数据大小。

### `inspect_file(path) -> Result<Inspection>`

只读检查文件是否可被默认还原。它会同时检查：

1. 尾部 4 字节面具长度字段合法。
2. 文件头匹配内置面具或 `one_key_mask()`。

任意一项不满足时返回 `Inspection { disguised: false, ... }`。

### `inspect_reader(reader) -> Result<Inspection>`

对任意 `Read + Seek` 输入执行只读检查。Android 和其它非路径调用方可用它检查文件描述符、内存缓冲或其它 seekable 输入。

### `original_extension(path) -> Result<Option<String>>`

读取伪装文件加密恢复元数据里的原扩展名。存在扩展名时返回例如 `Some("zip")`，用于 CLI 默认把 `secret.jpg` 还原为 `secret.zip`。缺少扩展名元数据的文件返回 `None`，调用方可以退回到移除最后一个扩展名的命名策略。

### `original_extension_reader(reader) -> Result<Option<String>>`

对任意 `Read + Seek` 输入读取原扩展名元数据。

### `reveal_file(path, force) -> Result<()>`

就地还原伪装文件。

- `force=false`：先执行 `inspect_file` 安全检查。
- `force=true`：跳过已知面具头检查，但仍依赖合法尾部长度字段。

### `reveal_seekable(file, force) -> Result<()>`

对实现 `SeekableFile` 的读写对象执行原地还原。它与 `reveal_file` 使用同一套校验和字节恢复逻辑，只是不要求调用方传入文件路径。

### `restore_to_writer(input, output, force) -> Result<Option<String>>`

从 `Read + Seek` 输入中读取伪装文件，把恢复后的原始字节写入 `Write` 输出，并返回原扩展名。该函数不会修改输入文件，适合 Android 文件提供方不支持原地覆盖时的“另存为” fallback。

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

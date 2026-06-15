# `apate-core` API 参考

> 路径：`crates/apate-core/src/lib.rs`
> 依赖：`std` + `thiserror = "2"`
> 适用版本：`0.1.0`（与 workspace 同步）

`apate-core` 是 apate 的算法核心，仅暴露纯函数与几个枚举/结构体，刻意不依赖
任何 IO 框架，以便在 CLI、GUI、agent runtime 中复用。

## 常量

### `MASK_LENGTH_INDICATOR_LENGTH: u64 = 4`

文件尾部用于记录面具长度的字段宽度，固定为 4 字节（小端 i32）。

### `MAXIMUM_MASK_LENGTH: u64 = 2_147_483_647 / 7`

面具字节数上限。约 `306_784_778`，主要防止 i32 与运算溢出与保证 `reveal` 的内存安全。

## 类型

### `enum MaskKind`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskKind {
    Exe,
    Jpg,
    Mp4,
    Mov,
}
```

内置面具的分类。`Clone + Copy` 可放心按值传递。

### `struct BuiltinMask`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinMask {
    pub kind: MaskKind,
    pub name: &'static str,        // 短名："exe" / "jpg" / "mp4" / "mov"
    pub extension: &'static str,   // 形如 ".mp4"（含前导点）
    pub bytes: &'static [u8],      // 实际面具字节
}
```

`builtin_masks()` 返回值中的元素类型。`Copy` 是安全的，因为字节切片是 `&'static`。

### `struct Inspection`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inspection {
    pub disguised: bool,           // 是否被识别为旧格式伪装文件
    pub mask_length: Option<u32>,  // 仅在 disguised=true 时有意义
    pub payload_length: Option<u64>,
}
```

`inspect_file` 的返回值。

### `enum ApateError`

```rust
#[derive(Debug, thiserror::Error)]
pub enum ApateError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("面具不能为空")]
    EmptyMask,

    #[error("面具文件过大: {length} 字节，最大允许 {max} 字节")]
    MaskTooLarge { length: u64, max: u64 },

    #[error("文件不是有效的旧格式伪装文件")]
    NotDisguised,

    #[error("路径不存在: {0}")]
    MissingPath(PathBuf),

    #[error("不支持非递归处理文件夹: {0}")]
    DirectoryRequiresRecursive(PathBuf),
}
```

`Result<T> = std::result::Result<T, ApateError>` 是 crate 的统一结果类型。

| 变体                              | 触发条件                                          |
| --------------------------------- | ------------------------------------------------- |
| `Io`                              | 文件打开、读写、rename 等任何 `std::io` 错误      |
| `EmptyMask`                       | `disguise_file` 收到空 mask                        |
| `MaskTooLarge`                    | mask 字节数 > `MAXIMUM_MASK_LENGTH`               |
| `NotDisguised`                    | 长度字段非正数 / 文件长度不足 / `inspect` 未识别   |
| `MissingPath`                     | `collect_input_files` 收到不存在的路径            |
| `DirectoryRequiresRecursive`      | `collect_input_files` 收到目录但 `recursive=false` |

### `type Result<T> = std::result::Result<T, ApateError>`

## 函数

### `builtin_masks() -> &'static [BuiltinMask]`

返回所有内置面具。迭代顺序与 `MaskKind` 声明顺序一致：`Exe, Jpg, Mp4, Mov`。

### `builtin_mask(kind: MaskKind) -> BuiltinMask`

按枚举值取单个内置面具。若未来枚举值与 `BUILTIN_MASKS` 不一致会触发 panic（设计上不允许）。

### `disguise_file(path, mask) -> Result<()>`

把 `path` 指向的文件就地改写为旧格式伪装文件。

- **参数**：
  - `path: impl AsRef<Path>` —— 任意可借用为 `Path` 的输入。
  - `mask: &[u8]` —— 面具字节，必须非空且不超过 `MAXIMUM_MASK_LENGTH`。
- **前置校验**：`validate_mask` 会拒绝空 mask 与过大 mask。
- **副作用**：
  1. 读取原文件前 `min(file_len, mask.len())` 字节；
  2. 文件指针归零，写入 mask；
  3. 文件指针置尾，追加 **倒序** 的原文件头；
  4. 追加 `mask.len() as i32` 的 little-endian 表示；
  5. `flush()`。
- **错误**：空 mask → `EmptyMask`；过大 mask → `MaskTooLarge`；其余走 `Io`。
- **注意**：不会修改文件名，命名由调用方（如 `apate-cli`）负责。

### `reveal_file(path, force) -> Result<()>`

把旧格式伪装文件就地还原为原始字节。

- **参数**：
  - `path: impl AsRef<Path>`
  - `force: bool` —— 若为 `false` 且 `inspect_file` 判定 `disguised=false`，直接返回
    `NotDisguised`；为 `true` 时跳过该安全检查（CLI 对应 `--force`）。
- **步骤**：
  1. 调 `inspect_file` 读取尾部 4 字节；
  2. 计算 `payload_length = file_len - 4 - mask_head_length`；
  3. 当 `mask_head_length <= payload_length` 时，从 `(file_len - 4 - mask_head_length)`
     处读取倒序原文件头；否则从 `mask_head_length` 处读取 `payload_length` 字节
     （处理「原文件比面具还短」的情况）；
  4. 截断尾部，倒序写回头部。
- **错误**：任何长度字段非法、文件过短 → `NotDisguised`；其余 → `Io`。

### `inspect_file(path) -> Result<Inspection>`

只读探测文件是否为旧格式伪装文件，并返回面具长度与负载长度。
**不会修改文件内容**，适合放在 `reveal_file` 之前做安全检查。

### `collect_input_files(path, recursive) -> Result<Vec<PathBuf>>`

把 `path` 解析为「待处理文件列表」：

- `path` 指向文件 → 返回 `[path]`；
- `path` 指向目录且 `recursive=true` → 递归收集所有文件并按字典序排序；
- `path` 指向目录且 `recursive=false` → `DirectoryRequiresRecursive`；
- `path` 不存在 → `MissingPath`。

## 内部函数（不导出但写在源码里）

- `validate_mask(&[u8]) -> Result<()>`：空 / 过大 mask 的统一校验。
- `read_mask_length(&mut File, file_length) -> Result<u32>`：从文件末尾读取 4 字节
  little-endian i32，并检查非正数和文件长度下限。
- `collect_directory_files(&Path, &mut Vec<PathBuf>) -> Result<()>`：`collect_input_files`
  的递归驱动器。

> 内部函数不在 API 契约内，未来可能改动；公开 API 仅包含本文件列出的常量、类型与函数。

## 用法示例

### 一次性伪装 + 还原

```rust
use apate_core::{builtin_mask, disguise_file, inspect_file, reveal_file, MaskKind};

let path = "secret.zip";

// 1. 伪装
disguise_file(path, builtin_mask(MaskKind::Jpg).bytes)?;

// 2. 检查
let inspection = inspect_file(path)?;
assert!(inspection.disguised);
assert_eq!(inspection.mask_length, Some(4));

// 3. 还原
reveal_file(path, /* force */ false)?;
```

### 批量递归处理目录

```rust
use apate_core::{builtin_mask, collect_input_files, disguise_file, MaskKind};

let files = collect_input_files("./to_disguise", true)?;
for path in files {
    disguise_file(&path, builtin_mask(MaskKind::Mp4).bytes)?;
}
```

### 自定义 mask

```rust
use apate_core::disguise_file;
use std::fs;

let custom = fs::read("my_mask.bin")?;
disguise_file("payload.zip", &custom)?;
```

> ⚠️ 不要在外侧自己拼接 mask 字节：参考 [`ARCHITECTURE.md`](./ARCHITECTURE.md#文件格式旧版-apate-格式) 中
> 的字节布局。任何对该格式的修改都必须同步更新 `legacy_format.rs` 的回归测试。

## 兼容性与稳定性

- 本 crate 处于 `0.1.0`，接口可能在重大重构时变化。一旦引入 1.0，将走 semver。
- 文件格式层面（字节布局、4 字节长度字段）属于 **跨实现契约**，即使 crate 接口
  调整也不允许变更，除非同时更新：
  - `crates/apate-core/tests/legacy_format.rs`
  - 旧版 WinForms 实现 `apate/ApateUI.cs` 中的对应算法路径。
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

pub const MASK_LENGTH_INDICATOR_LENGTH: u64 = 4;
pub const MAXIMUM_MASK_LENGTH: u64 = 2_147_483_647 / 7;

const EXTENSION_FOOTER_MAGIC: &[u8; 8] = b"APATE2EX";
const EXTENSION_LENGTH_FIELD_LENGTH: u64 = 2;
const EXTENSION_FOOTER_MAGIC_LENGTH: u64 = EXTENSION_FOOTER_MAGIC.len() as u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaskKind {
    Exe,
    Jpg,
    Mp4,
    Mov,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinMask {
    pub kind: MaskKind,
    pub name: &'static str,
    pub extension: &'static str,
    pub bytes: &'static [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Inspection {
    pub disguised: bool,
    pub mask_length: Option<u32>,
    pub payload_length: Option<u64>,
}

#[derive(Debug, thiserror::Error)]
pub enum ApateError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("面具不能为空")]
    EmptyMask,
    #[error("面具文件过大: {length} 字节，最大允许 {max} 字节")]
    MaskTooLarge { length: u64, max: u64 },
    #[error("文件不是有效的 apate 伪装文件")]
    NotDisguised,
    #[error("输出路径已存在: {0}")]
    OutputExists(PathBuf),
    #[error("参数无效: {0}")]
    InvalidArguments(String),
    #[error("路径不存在: {0}")]
    MissingPath(PathBuf),
    #[error("不支持非递归处理文件夹: {0}")]
    DirectoryRequiresRecursive(PathBuf),
}

pub type Result<T> = std::result::Result<T, ApateError>;

const JPG_HEAD: &[u8] = &[0xff, 0xd8, 0xff, 0xe1];
const MOV_HEAD: &[u8] = b"moov";
const MP4_HEAD: &[u8] = &[
    0x00, 0x00, 0x00, 0x20, 0x66, 0x74, 0x79, 0x70, 0x69, 0x73, 0x6f, 0x6d, 0x00, 0x00, 0x02, 0x00,
    0x69, 0x73, 0x6f, 0x6d, 0x69, 0x73, 0x6f, 0x32, 0x61, 0x76, 0x63, 0x31, 0x6d, 0x70, 0x34, 0x31,
];
const EXE_HEAD: &[u8] = &[
    0x4d, 0x5a, 0x90, 0x00, 0x03, 0x00, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00,
    0xb8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00,
    0x00, 0x0e, 0x1f, 0xba, 0x0e, 0x00, 0xb4, 0x09, 0xcd, 0x21, 0xb8, 0x01, 0x4c, 0xcd, 0x21, 0x54,
    0x68, 0x69, 0x73, 0x20, 0x70, 0x72, 0x6f, 0x67, 0x72, 0x61, 0x6d, 0x20, 0x63, 0x61, 0x6e, 0x6e,
    0x6f, 0x74, 0x20, 0x62, 0x65, 0x20, 0x72, 0x75, 0x6e, 0x20, 0x69, 0x6e, 0x20, 0x44, 0x4f, 0x53,
    0x20, 0x6d, 0x6f, 0x64, 0x65, 0x2e, 0x0d, 0x0d, 0x0a, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00,
];

const BUILTIN_MASKS: &[BuiltinMask] = &[
    BuiltinMask {
        kind: MaskKind::Exe,
        name: "exe",
        extension: ".exe",
        bytes: EXE_HEAD,
    },
    BuiltinMask {
        kind: MaskKind::Jpg,
        name: "jpg",
        extension: ".jpg",
        bytes: JPG_HEAD,
    },
    BuiltinMask {
        kind: MaskKind::Mp4,
        name: "mp4",
        extension: ".mp4",
        bytes: MP4_HEAD,
    },
    BuiltinMask {
        kind: MaskKind::Mov,
        name: "mov",
        extension: ".mov",
        bytes: MOV_HEAD,
    },
];

const ONE_KEY_MASK: &[u8] = include_bytes!("../resources/mask.mp4");

pub fn builtin_masks() -> &'static [BuiltinMask] {
    BUILTIN_MASKS
}

pub fn one_key_mask() -> &'static [u8] {
    ONE_KEY_MASK
}

pub fn builtin_mask(kind: MaskKind) -> BuiltinMask {
    *BUILTIN_MASKS
        .iter()
        .find(|mask| mask.kind == kind)
        .expect("内置面具枚举必须完整")
}

pub fn disguise_file(path: impl AsRef<Path>, mask: &[u8]) -> Result<()> {
    validate_mask(mask)?;

    let path = path.as_ref();
    let original_extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().into_owned())
        .unwrap_or_default();
    validate_extension_text(&original_extension)?;
    let original_extension = original_extension.as_bytes();
    if original_extension.len() > u16::MAX as usize {
        return Err(ApateError::InvalidArguments("文件扩展名过长".to_string()));
    }

    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let file_length = file.metadata()?.len();
    let original_head_length = file_length.min(mask.len() as u64) as usize;
    let mut original_head = vec![0_u8; original_head_length];
    file.read_exact(&mut original_head)?;

    file.seek(SeekFrom::Start(0))?;
    file.write_all(mask)?;
    file.seek(SeekFrom::End(0))?;
    original_head.reverse();
    file.write_all(&original_head)?;
    file.write_all(original_extension)?;
    file.write_all(&(original_extension.len() as u16).to_le_bytes())?;
    file.write_all(EXTENSION_FOOTER_MAGIC)?;
    file.write_all(&(mask.len() as i32).to_le_bytes())?;
    file.flush()?;

    Ok(())
}

pub fn reveal_file(path: impl AsRef<Path>, force: bool) -> Result<()> {
    let path = path.as_ref();
    let inspection = inspect_file(path)?;
    if !force && !inspection.disguised {
        return Err(ApateError::NotDisguised);
    }

    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let disguised_length = file.metadata()?.len();
    let mask_head_length = read_mask_length(&mut file, disguised_length)?;
    let extension_footer = read_extension_footer(&mut file, disguised_length, mask_head_length)?;
    let payload_length = disguised_length
        .checked_sub(MASK_LENGTH_INDICATOR_LENGTH)
        .and_then(|length| length.checked_sub(extension_footer.byte_length))
        .and_then(|length| length.checked_sub(mask_head_length as u64))
        .ok_or(ApateError::NotDisguised)?;

    let original_head_length;
    if mask_head_length as u64 <= payload_length {
        file.seek(SeekFrom::Start(
            disguised_length
                - MASK_LENGTH_INDICATOR_LENGTH
                - extension_footer.byte_length
                - mask_head_length as u64,
        ))?;
        original_head_length = mask_head_length as usize;
    } else {
        file.seek(SeekFrom::Start(mask_head_length as u64))?;
        original_head_length = payload_length as usize;
    }

    let mut original_head = vec![0_u8; original_head_length];
    file.read_exact(&mut original_head)?;
    file.set_len(
        disguised_length
            - mask_head_length as u64
            - extension_footer.byte_length
            - MASK_LENGTH_INDICATOR_LENGTH,
    )?;
    file.seek(SeekFrom::Start(0))?;
    original_head.reverse();
    file.write_all(&original_head)?;
    file.flush()?;

    Ok(())
}

pub fn inspect_file(path: impl AsRef<Path>) -> Result<Inspection> {
    let path = path.as_ref();
    let mut file = OpenOptions::new().read(true).open(path)?;
    let file_length = file.metadata()?.len();
    if file_length < MASK_LENGTH_INDICATOR_LENGTH {
        return Ok(Inspection {
            disguised: false,
            mask_length: None,
            payload_length: None,
        });
    }

    let mask_length = match read_mask_length(&mut file, file_length) {
        Ok(mask_length) => mask_length,
        Err(ApateError::NotDisguised) => {
            return Ok(Inspection {
                disguised: false,
                mask_length: None,
                payload_length: None,
            });
        }
        Err(error) => return Err(error),
    };
    let extension_footer = read_extension_footer(&mut file, file_length, mask_length)?;

    if !has_known_mask_header(&mut file, mask_length)? {
        return Ok(Inspection {
            disguised: false,
            mask_length: None,
            payload_length: None,
        });
    }

    let payload_length = file_length
        .checked_sub(MASK_LENGTH_INDICATOR_LENGTH)
        .and_then(|length| length.checked_sub(extension_footer.byte_length))
        .and_then(|length| length.checked_sub(mask_length as u64))
        .ok_or(ApateError::NotDisguised)?;
    Ok(Inspection {
        disguised: true,
        mask_length: Some(mask_length),
        payload_length: Some(payload_length),
    })
}

pub fn original_extension(path: impl AsRef<Path>) -> Result<Option<String>> {
    let path = path.as_ref();
    let mut file = OpenOptions::new().read(true).open(path)?;
    let file_length = file.metadata()?.len();
    let mask_length = read_mask_length(&mut file, file_length)?;
    let extension_footer = read_extension_footer(&mut file, file_length, mask_length)?;
    Ok(extension_footer.original_extension)
}

pub fn collect_input_files(path: impl AsRef<Path>, recursive: bool) -> Result<Vec<PathBuf>> {
    let path = path.as_ref();
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.exists() {
        return Err(ApateError::MissingPath(path.to_path_buf()));
    }
    if !recursive {
        return Err(ApateError::DirectoryRequiresRecursive(path.to_path_buf()));
    }

    let mut files = Vec::new();
    collect_directory_files(path, &mut files)?;
    files.sort();
    Ok(files)
}

pub fn validate_mask(mask: &[u8]) -> Result<()> {
    if mask.is_empty() {
        return Err(ApateError::EmptyMask);
    }
    if mask.len() as u64 > MAXIMUM_MASK_LENGTH {
        return Err(ApateError::MaskTooLarge {
            length: mask.len() as u64,
            max: MAXIMUM_MASK_LENGTH,
        });
    }
    Ok(())
}

fn has_known_mask_header(file: &mut fs::File, mask_length: u32) -> Result<bool> {
    let mask_length = mask_length as usize;
    for mask in builtin_masks()
        .iter()
        .map(|mask| mask.bytes)
        .chain([ONE_KEY_MASK])
    {
        if mask.len() != mask_length {
            continue;
        }

        let mut head = vec![0_u8; mask_length];
        file.seek(SeekFrom::Start(0))?;
        file.read_exact(&mut head)?;
        if head == mask {
            return Ok(true);
        }
    }

    Ok(false)
}

fn read_mask_length(file: &mut fs::File, file_length: u64) -> Result<u32> {
    if file_length < MASK_LENGTH_INDICATOR_LENGTH {
        return Err(ApateError::NotDisguised);
    }
    file.seek(SeekFrom::Start(file_length - MASK_LENGTH_INDICATOR_LENGTH))?;
    let mut length_bytes = [0_u8; 4];
    file.read_exact(&mut length_bytes)?;
    let signed_length = i32::from_le_bytes(length_bytes);
    if signed_length <= 0 {
        return Err(ApateError::NotDisguised);
    }
    let mask_length = signed_length as u32;
    if mask_length as u64 > MAXIMUM_MASK_LENGTH {
        return Err(ApateError::NotDisguised);
    }
    let minimum_length = MASK_LENGTH_INDICATOR_LENGTH + mask_length as u64;
    if file_length < minimum_length {
        return Err(ApateError::NotDisguised);
    }
    Ok(mask_length)
}

struct ExtensionFooter {
    original_extension: Option<String>,
    byte_length: u64,
}

fn read_extension_footer(
    file: &mut fs::File,
    file_length: u64,
    mask_length: u32,
) -> Result<ExtensionFooter> {
    let minimum_v2_length = MASK_LENGTH_INDICATOR_LENGTH
        + EXTENSION_FOOTER_MAGIC_LENGTH
        + EXTENSION_LENGTH_FIELD_LENGTH;
    if file_length < minimum_v2_length {
        return Ok(ExtensionFooter {
            original_extension: None,
            byte_length: 0,
        });
    }

    let magic_start = file_length - MASK_LENGTH_INDICATOR_LENGTH - EXTENSION_FOOTER_MAGIC_LENGTH;
    file.seek(SeekFrom::Start(magic_start))?;
    let mut magic = [0_u8; EXTENSION_FOOTER_MAGIC.len()];
    file.read_exact(&mut magic)?;
    if &magic != EXTENSION_FOOTER_MAGIC {
        return Ok(ExtensionFooter {
            original_extension: None,
            byte_length: 0,
        });
    }

    let Some(length_start) = magic_start.checked_sub(EXTENSION_LENGTH_FIELD_LENGTH) else {
        return Ok(ExtensionFooter {
            original_extension: None,
            byte_length: 0,
        });
    };
    file.seek(SeekFrom::Start(length_start))?;
    let mut length_bytes = [0_u8; 2];
    file.read_exact(&mut length_bytes)?;
    let extension_length = u16::from_le_bytes(length_bytes) as u64;
    let footer_length =
        extension_length + EXTENSION_LENGTH_FIELD_LENGTH + EXTENSION_FOOTER_MAGIC_LENGTH;

    if file_length < MASK_LENGTH_INDICATOR_LENGTH + footer_length + mask_length as u64 {
        return Ok(ExtensionFooter {
            original_extension: None,
            byte_length: 0,
        });
    }

    let Some(extension_start) = length_start.checked_sub(extension_length) else {
        return Ok(ExtensionFooter {
            original_extension: None,
            byte_length: 0,
        });
    };
    file.seek(SeekFrom::Start(extension_start))?;
    let mut extension = vec![0_u8; extension_length as usize];
    file.read_exact(&mut extension)?;
    let Ok(extension) = String::from_utf8(extension) else {
        return Ok(ExtensionFooter {
            original_extension: None,
            byte_length: 0,
        });
    };
    if validate_extension_text(&extension).is_err() {
        return Ok(ExtensionFooter {
            original_extension: None,
            byte_length: 0,
        });
    }

    Ok(ExtensionFooter {
        original_extension: Some(extension),
        byte_length: footer_length,
    })
}

fn validate_extension_text(extension: &str) -> Result<()> {
    let valid = extension
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if valid {
        Ok(())
    } else {
        Err(ApateError::InvalidArguments(
            "文件扩展名只能包含 ASCII 字母、数字、- 或 _".to_string(),
        ))
    }
}

fn collect_directory_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            files.push(entry_path);
        } else if metadata.is_dir() {
            collect_directory_files(&entry_path, files)?;
        }
    }
    Ok(())
}

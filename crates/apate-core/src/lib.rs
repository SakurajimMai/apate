use chacha20::ChaCha20;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use std::fs::{self, OpenOptions};
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const MASK_LENGTH_INDICATOR_LENGTH: u64 = 4;
pub const MAXIMUM_MASK_LENGTH: u64 = 2_147_483_647 / 7;

const ENCRYPTED_FOOTER_MAGIC: &[u8; 8] = b"APATE3EN";
const ENCRYPTED_METADATA_MAGIC: &[u8; 6] = b"APMD01";
const PLAIN_EXTENSION_FOOTER_MAGIC: &[u8; 8] = b"APATE2EX";
const METADATA_LENGTH_FIELD_LENGTH: u64 = 4;
const METADATA_NONCE_FIELD_LENGTH: u64 = 8;
const EXTENSION_LENGTH_FIELD_LENGTH: u64 = 2;
const ENCRYPTED_FOOTER_MAGIC_LENGTH: u64 = ENCRYPTED_FOOTER_MAGIC.len() as u64;
const PLAIN_EXTENSION_FOOTER_MAGIC_LENGTH: u64 = PLAIN_EXTENSION_FOOTER_MAGIC.len() as u64;
const OBFUSCATED_TAIL_WINDOW: usize = 128 * 1024;
const METADATA_CIPHER_CONTEXT: &[u8] = b"apate-metadata-v1";
const TAIL_CIPHER_CONTEXT: &[u8] = b"apate-tail-window-v1";
const APATE_INTERNAL_KEY: [u8; 32] = [
    0x41, 0x70, 0x61, 0x74, 0x65, 0x2d, 0x72, 0x75, 0x73, 0x74, 0x2d, 0x66, 0x6f, 0x72, 0x6d, 0x61,
    0x74, 0x2d, 0x6d, 0x61, 0x73, 0x6b, 0x2d, 0x76, 0x33, 0x2d, 0x63, 0x68, 0x61, 0x63, 0x68, 0x61,
];

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

pub trait SeekableFile: Read + Write + Seek {
    fn set_target_len(&mut self, len: u64) -> io::Result<()>;
}

impl SeekableFile for fs::File {
    fn set_target_len(&mut self, len: u64) -> io::Result<()> {
        fs::File::set_len(self, len)
    }
}

impl SeekableFile for Cursor<Vec<u8>> {
    fn set_target_len(&mut self, len: u64) -> io::Result<()> {
        let len = usize::try_from(len).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "目标长度超过当前平台可寻址内存",
            )
        })?;
        self.get_mut().resize(len, 0);
        if self.position() > len as u64 {
            self.set_position(len as u64);
        }
        Ok(())
    }
}

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
    let tail_start = file_length.saturating_sub(OBFUSCATED_TAIL_WINDOW as u64);
    let tail_length = file_length.saturating_sub(tail_start) as usize;
    let mut original_tail = vec![0_u8; tail_length];
    file.seek(SeekFrom::Start(tail_start))?;
    file.read_exact(&mut original_tail)?;
    let encrypted_metadata = build_encrypted_metadata(
        file_length,
        &original_head,
        &original_tail,
        original_extension,
        mask.len(),
    )?;

    obfuscate_tail_window(
        &mut file,
        tail_start,
        tail_length,
        encrypted_metadata.nonce,
        mask.len() as u64,
    )?;
    file.seek(SeekFrom::Start(0))?;
    file.write_all(mask)?;
    file.seek(SeekFrom::End(0))?;
    file.write_all(&encrypted_metadata.ciphertext)?;
    file.write_all(&(encrypted_metadata.ciphertext.len() as u32).to_le_bytes())?;
    file.write_all(&encrypted_metadata.nonce.to_le_bytes())?;
    file.write_all(ENCRYPTED_FOOTER_MAGIC)?;
    file.write_all(&(mask.len() as i32).to_le_bytes())?;
    file.flush()?;

    Ok(())
}

pub fn reveal_file(path: impl AsRef<Path>, force: bool) -> Result<()> {
    let path = path.as_ref();
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    reveal_seekable(&mut file, force)
}

pub fn reveal_seekable(file: &mut impl SeekableFile, force: bool) -> Result<()> {
    let disguised_length = stream_len(file)?;
    if !force {
        let inspection = inspect_seekable(file, disguised_length)?;
        if !inspection.disguised {
            return Err(ApateError::NotDisguised);
        }
    }

    let mask_head_length = read_mask_length(file, disguised_length)?;
    let restore_metadata = read_restore_metadata(file, disguised_length, mask_head_length)?;
    restore_in_place(file, disguised_length, mask_head_length, restore_metadata)
}

pub fn restore_to_writer(
    mut input: impl Read + Seek,
    output: &mut impl Write,
    force: bool,
) -> Result<Option<String>> {
    let disguised_length = stream_len(&mut input)?;
    if !force {
        let inspection = inspect_seekable(&mut input, disguised_length)?;
        if !inspection.disguised {
            return Err(ApateError::NotDisguised);
        }
    }

    let mask_head_length = read_mask_length(&mut input, disguised_length)?;
    let restore_metadata = read_restore_metadata(&mut input, disguised_length, mask_head_length)?;
    let original_extension = restore_metadata.original_extension().map(ToOwned::to_owned);
    write_restored_to_output(
        &mut input,
        output,
        disguised_length,
        mask_head_length,
        &restore_metadata,
    )?;
    output.flush()?;
    Ok(original_extension)
}

fn restore_in_place(
    file: &mut impl SeekableFile,
    disguised_length: u64,
    mask_head_length: u32,
    restore_metadata: RestoreMetadata,
) -> Result<()> {
    match restore_metadata {
        RestoreMetadata::Encrypted {
            original_head,
            original_file_length,
            original_tail,
            ..
        } => {
            validate_encrypted_restore_parts(
                original_file_length,
                mask_head_length,
                &original_head,
                &original_tail,
            )?;

            file.set_target_len(original_file_length)?;
            let tail_start = original_file_length.saturating_sub(original_tail.len() as u64);
            file.seek(SeekFrom::Start(tail_start))?;
            file.write_all(&original_tail)?;
            file.seek(SeekFrom::Start(0))?;
            file.write_all(&original_head)?;
            file.flush()?;
        }
        RestoreMetadata::Plain { byte_length, .. } => {
            let (payload_length, mut original_head) =
                plain_original_head(file, disguised_length, byte_length, mask_head_length)?;
            file.set_target_len(payload_length)?;
            file.seek(SeekFrom::Start(0))?;
            original_head.reverse();
            file.write_all(&original_head)?;
            file.flush()?;
        }
    }

    Ok(())
}

fn write_restored_to_output(
    input: &mut (impl Read + Seek),
    output: &mut impl Write,
    disguised_length: u64,
    mask_head_length: u32,
    restore_metadata: &RestoreMetadata,
) -> Result<()> {
    match restore_metadata {
        RestoreMetadata::Encrypted {
            original_head,
            original_file_length,
            original_tail,
            ..
        } => {
            validate_encrypted_restore_parts(
                *original_file_length,
                mask_head_length,
                original_head,
                original_tail,
            )?;
            write_encrypted_restored_to_output(
                input,
                output,
                *original_file_length,
                original_head,
                original_tail,
            )?;
        }
        RestoreMetadata::Plain { byte_length, .. } => {
            let (payload_length, mut original_head) =
                plain_original_head(input, disguised_length, *byte_length, mask_head_length)?;
            original_head.reverse();
            output.write_all(&original_head)?;
            if mask_head_length as u64 <= payload_length {
                copy_range(
                    input,
                    output,
                    mask_head_length as u64,
                    payload_length - mask_head_length as u64,
                )?;
            }
        }
    }

    Ok(())
}

pub fn inspect_file(path: impl AsRef<Path>) -> Result<Inspection> {
    let path = path.as_ref();
    let mut file = OpenOptions::new().read(true).open(path)?;
    inspect_reader(&mut file)
}

pub fn inspect_reader(mut reader: impl Read + Seek) -> Result<Inspection> {
    let file_length = stream_len(&mut reader)?;
    inspect_seekable(&mut reader, file_length)
}

fn inspect_seekable(file: &mut (impl Read + Seek), file_length: u64) -> Result<Inspection> {
    if file_length < MASK_LENGTH_INDICATOR_LENGTH {
        return Ok(Inspection {
            disguised: false,
            mask_length: None,
            payload_length: None,
        });
    }

    let mask_length = match read_mask_length(file, file_length) {
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
    if !has_known_mask_header(file, mask_length)? {
        return Ok(Inspection {
            disguised: false,
            mask_length: None,
            payload_length: None,
        });
    }
    let restore_metadata = read_restore_metadata(file, file_length, mask_length)?;

    let payload_length = match restore_metadata {
        RestoreMetadata::Encrypted {
            original_file_length,
            ..
        } => original_file_length,
        RestoreMetadata::Plain { byte_length, .. } => {
            plain_payload_length(file_length, byte_length, mask_length)?
        }
    };
    Ok(Inspection {
        disguised: true,
        mask_length: Some(mask_length),
        payload_length: Some(payload_length),
    })
}

pub fn original_extension(path: impl AsRef<Path>) -> Result<Option<String>> {
    let path = path.as_ref();
    let mut file = OpenOptions::new().read(true).open(path)?;
    original_extension_reader(&mut file)
}

pub fn original_extension_reader(mut reader: impl Read + Seek) -> Result<Option<String>> {
    let file_length = stream_len(&mut reader)?;
    let mask_length = read_mask_length(&mut reader, file_length)?;
    let restore_metadata = read_restore_metadata(&mut reader, file_length, mask_length)?;
    Ok(restore_metadata.original_extension().map(ToOwned::to_owned))
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

fn stream_len(file: &mut impl Seek) -> Result<u64> {
    let position = file.stream_position()?;
    let len = file.seek(SeekFrom::End(0))?;
    file.seek(SeekFrom::Start(position))?;
    Ok(len)
}

fn has_known_mask_header(file: &mut (impl Read + Seek), mask_length: u32) -> Result<bool> {
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

fn read_mask_length(file: &mut (impl Read + Seek), file_length: u64) -> Result<u32> {
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

struct EncryptedMetadata {
    ciphertext: Vec<u8>,
    nonce: u64,
}

enum RestoreMetadata {
    Encrypted {
        original_extension: Option<String>,
        original_file_length: u64,
        original_head: Vec<u8>,
        original_tail: Vec<u8>,
    },
    Plain {
        original_extension: Option<String>,
        byte_length: u64,
    },
}

impl RestoreMetadata {
    fn original_extension(&self) -> Option<&str> {
        match self {
            Self::Encrypted {
                original_extension, ..
            }
            | Self::Plain {
                original_extension, ..
            } => original_extension.as_deref(),
        }
    }
}

fn plain_payload_length(file_length: u64, metadata_length: u64, mask_length: u32) -> Result<u64> {
    file_length
        .checked_sub(MASK_LENGTH_INDICATOR_LENGTH)
        .and_then(|length| length.checked_sub(metadata_length))
        .and_then(|length| length.checked_sub(mask_length as u64))
        .ok_or(ApateError::NotDisguised)
}

fn plain_original_head(
    file: &mut (impl Read + Seek),
    disguised_length: u64,
    metadata_length: u64,
    mask_length: u32,
) -> Result<(u64, Vec<u8>)> {
    let payload_length = plain_payload_length(disguised_length, metadata_length, mask_length)?;
    let original_head_length;
    if mask_length as u64 <= payload_length {
        file.seek(SeekFrom::Start(
            disguised_length - MASK_LENGTH_INDICATOR_LENGTH - metadata_length - mask_length as u64,
        ))?;
        original_head_length = mask_length as usize;
    } else {
        file.seek(SeekFrom::Start(mask_length as u64))?;
        original_head_length = payload_length as usize;
    }

    let mut original_head = vec![0_u8; original_head_length];
    file.read_exact(&mut original_head)?;
    Ok((payload_length, original_head))
}

fn validate_encrypted_restore_parts(
    original_file_length: u64,
    mask_head_length: u32,
    original_head: &[u8],
    original_tail: &[u8],
) -> Result<()> {
    if original_head.len() as u64 != original_file_length.min(mask_head_length as u64) {
        return Err(ApateError::NotDisguised);
    }
    if original_tail.len()
        != original_file_length
            .min(OBFUSCATED_TAIL_WINDOW as u64)
            .try_into()
            .map_err(|_| ApateError::NotDisguised)?
    {
        return Err(ApateError::NotDisguised);
    }
    Ok(())
}

fn write_encrypted_restored_to_output(
    input: &mut (impl Read + Seek),
    output: &mut impl Write,
    original_file_length: u64,
    original_head: &[u8],
    original_tail: &[u8],
) -> Result<()> {
    output.write_all(original_head)?;

    let middle_start = original_head.len() as u64;
    let tail_start = original_file_length.saturating_sub(original_tail.len() as u64);
    if tail_start > middle_start {
        copy_range(input, output, middle_start, tail_start - middle_start)?;
    }

    if tail_start < middle_start {
        let overlap_start = (middle_start - tail_start) as usize;
        output.write_all(&original_tail[overlap_start..])?;
    } else {
        output.write_all(original_tail)?;
    }

    Ok(())
}

fn copy_range(
    input: &mut (impl Read + Seek),
    output: &mut impl Write,
    start: u64,
    mut length: u64,
) -> Result<()> {
    input.seek(SeekFrom::Start(start))?;
    let mut buffer = vec![0_u8; 64 * 1024];
    while length > 0 {
        let read_len = buffer.len().min(length as usize);
        let read = input.read(&mut buffer[..read_len])?;
        if read == 0 {
            return Err(ApateError::NotDisguised);
        }
        output.write_all(&buffer[..read])?;
        length -= read as u64;
    }
    Ok(())
}

fn build_encrypted_metadata(
    original_file_length: u64,
    original_head: &[u8],
    original_tail: &[u8],
    original_extension: &[u8],
    mask_length: usize,
) -> Result<EncryptedMetadata> {
    if original_head.len() as u64 != original_file_length.min(mask_length as u64) {
        return Err(ApateError::NotDisguised);
    }
    if original_head.len() > u32::MAX as usize {
        return Err(ApateError::MaskTooLarge {
            length: original_head.len() as u64,
            max: u32::MAX as u64,
        });
    }
    if original_tail.len() as u64 != original_file_length.min(OBFUSCATED_TAIL_WINDOW as u64) {
        return Err(ApateError::NotDisguised);
    }
    if original_tail.len() > u32::MAX as usize {
        return Err(ApateError::InvalidArguments(
            "原始尾部窗口过长，无法写入加密尾部".to_string(),
        ));
    }

    let mut plaintext = Vec::with_capacity(
        ENCRYPTED_METADATA_MAGIC.len()
            + 8
            + 4
            + 4
            + 2
            + original_extension.len()
            + original_head.len()
            + original_tail.len(),
    );
    plaintext.extend_from_slice(ENCRYPTED_METADATA_MAGIC);
    plaintext.extend_from_slice(&original_file_length.to_le_bytes());
    plaintext.extend_from_slice(&(original_head.len() as u32).to_le_bytes());
    plaintext.extend_from_slice(&(original_tail.len() as u32).to_le_bytes());
    plaintext.extend_from_slice(&(original_extension.len() as u16).to_le_bytes());
    plaintext.extend_from_slice(original_extension);
    plaintext.extend_from_slice(original_head);
    plaintext.extend_from_slice(original_tail);

    if plaintext.len() > u32::MAX as usize {
        return Err(ApateError::InvalidArguments(
            "恢复元数据过长，无法写入加密尾部".to_string(),
        ));
    }

    let nonce = metadata_nonce(
        original_file_length,
        mask_length as u64,
        original_head,
        original_tail,
    );
    apply_chacha20(
        &mut plaintext,
        METADATA_CIPHER_CONTEXT,
        nonce,
        mask_length as u64,
    );

    Ok(EncryptedMetadata {
        ciphertext: plaintext,
        nonce,
    })
}

fn read_restore_metadata(
    file: &mut (impl Read + Seek),
    file_length: u64,
    mask_length: u32,
) -> Result<RestoreMetadata> {
    if let Some(metadata) = read_encrypted_metadata(file, file_length, mask_length)? {
        return Ok(metadata);
    }

    read_plain_extension_footer(file, file_length, mask_length)
}

fn read_encrypted_metadata(
    file: &mut (impl Read + Seek),
    file_length: u64,
    mask_length: u32,
) -> Result<Option<RestoreMetadata>> {
    let fixed_length = METADATA_LENGTH_FIELD_LENGTH
        + METADATA_NONCE_FIELD_LENGTH
        + ENCRYPTED_FOOTER_MAGIC_LENGTH
        + MASK_LENGTH_INDICATOR_LENGTH;
    if file_length < fixed_length {
        return Ok(None);
    }

    let magic_start = file_length - MASK_LENGTH_INDICATOR_LENGTH - ENCRYPTED_FOOTER_MAGIC_LENGTH;
    file.seek(SeekFrom::Start(magic_start))?;
    let mut magic = [0_u8; ENCRYPTED_FOOTER_MAGIC.len()];
    file.read_exact(&mut magic)?;
    if &magic != ENCRYPTED_FOOTER_MAGIC {
        return Ok(None);
    }

    let nonce_start = magic_start
        .checked_sub(METADATA_NONCE_FIELD_LENGTH)
        .ok_or(ApateError::NotDisguised)?;
    file.seek(SeekFrom::Start(nonce_start))?;
    let mut nonce_bytes = [0_u8; 8];
    file.read_exact(&mut nonce_bytes)?;
    let nonce = u64::from_le_bytes(nonce_bytes);

    let metadata_length_start = nonce_start
        .checked_sub(METADATA_LENGTH_FIELD_LENGTH)
        .ok_or(ApateError::NotDisguised)?;
    file.seek(SeekFrom::Start(metadata_length_start))?;
    let mut metadata_length_bytes = [0_u8; 4];
    file.read_exact(&mut metadata_length_bytes)?;
    let metadata_length = u32::from_le_bytes(metadata_length_bytes) as u64;
    if metadata_length == 0 {
        return Err(ApateError::NotDisguised);
    }

    let metadata_start = metadata_length_start
        .checked_sub(metadata_length)
        .ok_or(ApateError::NotDisguised)?;
    let byte_length = metadata_length
        + METADATA_LENGTH_FIELD_LENGTH
        + METADATA_NONCE_FIELD_LENGTH
        + ENCRYPTED_FOOTER_MAGIC_LENGTH;
    if file_length < MASK_LENGTH_INDICATOR_LENGTH + byte_length + mask_length as u64 {
        return Err(ApateError::NotDisguised);
    }

    file.seek(SeekFrom::Start(metadata_start))?;
    let mut plaintext = vec![0_u8; metadata_length as usize];
    file.read_exact(&mut plaintext)?;
    apply_chacha20(
        &mut plaintext,
        METADATA_CIPHER_CONTEXT,
        nonce,
        mask_length as u64,
    );
    let (original_file_length, original_head, original_tail, original_extension) =
        parse_encrypted_metadata(&plaintext, mask_length)?;

    Ok(Some(RestoreMetadata::Encrypted {
        original_extension,
        original_file_length,
        original_head,
        original_tail,
    }))
}

fn parse_encrypted_metadata(
    plaintext: &[u8],
    mask_length: u32,
) -> Result<(u64, Vec<u8>, Vec<u8>, Option<String>)> {
    let minimum_length = ENCRYPTED_METADATA_MAGIC.len() + 8 + 4 + 4 + 2;
    if plaintext.len() < minimum_length {
        return Err(ApateError::NotDisguised);
    }
    if &plaintext[..ENCRYPTED_METADATA_MAGIC.len()] != ENCRYPTED_METADATA_MAGIC {
        return Err(ApateError::NotDisguised);
    }

    let mut cursor = ENCRYPTED_METADATA_MAGIC.len();
    let original_file_length = read_u64_le(plaintext, &mut cursor)?;
    let original_head_length = read_u32_le(plaintext, &mut cursor)? as usize;
    let original_tail_length = read_u32_le(plaintext, &mut cursor)? as usize;
    let extension_length = read_u16_le(plaintext, &mut cursor)? as usize;

    let extension_end = cursor
        .checked_add(extension_length)
        .ok_or(ApateError::NotDisguised)?;
    if extension_end > plaintext.len() {
        return Err(ApateError::NotDisguised);
    }
    let extension = &plaintext[cursor..extension_end];
    cursor = extension_end;

    let head_end = cursor
        .checked_add(original_head_length)
        .ok_or(ApateError::NotDisguised)?;
    if head_end > plaintext.len() {
        return Err(ApateError::NotDisguised);
    }
    let tail_end = head_end
        .checked_add(original_tail_length)
        .ok_or(ApateError::NotDisguised)?;
    if tail_end != plaintext.len() {
        return Err(ApateError::NotDisguised);
    }
    if original_head_length as u64 != original_file_length.min(mask_length as u64) {
        return Err(ApateError::NotDisguised);
    }
    if original_tail_length as u64 != original_file_length.min(OBFUSCATED_TAIL_WINDOW as u64) {
        return Err(ApateError::NotDisguised);
    }

    let extension = if extension.is_empty() {
        None
    } else {
        let extension =
            String::from_utf8(extension.to_vec()).map_err(|_| ApateError::NotDisguised)?;
        if validate_extension_text(&extension).is_err() {
            return Err(ApateError::NotDisguised);
        }
        Some(extension)
    };

    Ok((
        original_file_length,
        plaintext[cursor..head_end].to_vec(),
        plaintext[head_end..tail_end].to_vec(),
        extension,
    ))
}

fn read_u16_le(bytes: &[u8], cursor: &mut usize) -> Result<u16> {
    let end = cursor.checked_add(2).ok_or(ApateError::NotDisguised)?;
    let value = bytes
        .get(*cursor..end)
        .ok_or(ApateError::NotDisguised)?
        .try_into()
        .map(u16::from_le_bytes)
        .map_err(|_| ApateError::NotDisguised)?;
    *cursor = end;
    Ok(value)
}

fn read_u32_le(bytes: &[u8], cursor: &mut usize) -> Result<u32> {
    let end = cursor.checked_add(4).ok_or(ApateError::NotDisguised)?;
    let value = bytes
        .get(*cursor..end)
        .ok_or(ApateError::NotDisguised)?
        .try_into()
        .map(u32::from_le_bytes)
        .map_err(|_| ApateError::NotDisguised)?;
    *cursor = end;
    Ok(value)
}

fn read_u64_le(bytes: &[u8], cursor: &mut usize) -> Result<u64> {
    let end = cursor.checked_add(8).ok_or(ApateError::NotDisguised)?;
    let value = bytes
        .get(*cursor..end)
        .ok_or(ApateError::NotDisguised)?
        .try_into()
        .map(u64::from_le_bytes)
        .map_err(|_| ApateError::NotDisguised)?;
    *cursor = end;
    Ok(value)
}

fn obfuscate_tail_window(
    file: &mut fs::File,
    tail_start: u64,
    tail_length: usize,
    nonce: u64,
    mask_length: u64,
) -> Result<()> {
    if tail_length == 0 {
        return Ok(());
    }

    let mut tail = vec![0_u8; tail_length];
    file.seek(SeekFrom::Start(tail_start))?;
    file.read_exact(&mut tail)?;
    apply_chacha20(&mut tail, TAIL_CIPHER_CONTEXT, nonce, mask_length);
    file.seek(SeekFrom::Start(tail_start))?;
    file.write_all(&tail)?;

    Ok(())
}

fn metadata_nonce(
    original_file_length: u64,
    mask_length: u64,
    original_head: &[u8],
    original_tail: &[u8],
) -> u64 {
    let random_part = random_u64().unwrap_or(0);
    let time_part = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(0);
    let head_part = original_head
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            hash ^ (*byte as u64).wrapping_mul(0x0000_0100_0000_01b3)
        });
    let tail_part = original_tail
        .iter()
        .rev()
        .take(256)
        .fold(0x9e37_79b9_7f4a_7c15_u64, |hash, byte| {
            hash.rotate_left(5) ^ *byte as u64
        });

    random_part
        ^ time_part
        ^ original_file_length.rotate_left(17)
        ^ mask_length.rotate_left(41)
        ^ splitmix64(head_part)
        ^ splitmix64(tail_part)
}

fn random_u64() -> Option<u64> {
    let mut bytes = [0_u8; 8];
    getrandom::getrandom(&mut bytes).ok()?;
    Some(u64::from_le_bytes(bytes))
}

fn apply_chacha20(bytes: &mut [u8], context: &[u8], nonce: u64, mask_length: u64) {
    let key = derive_chacha20_key(context, nonce, mask_length);
    let nonce_bytes = derive_chacha20_nonce(context, nonce, mask_length);
    let mut cipher = ChaCha20::new(&key.into(), &nonce_bytes.into());
    cipher.apply_keystream(bytes);
}

fn derive_chacha20_key(context: &[u8], nonce: u64, mask_length: u64) -> [u8; 32] {
    let mut state = 0x243f_6a88_85a3_08d3 ^ nonce ^ mask_length.rotate_left(13);
    for byte in context {
        state = splitmix64(state ^ *byte as u64);
    }
    for byte in APATE_INTERNAL_KEY {
        state = splitmix64(state ^ byte as u64);
    }

    let mut key = [0_u8; 32];
    for chunk in key.chunks_exact_mut(8) {
        state = splitmix64(state.wrapping_add(0x9e37_79b9_7f4a_7c15));
        chunk.copy_from_slice(&state.to_le_bytes());
    }
    key
}

fn derive_chacha20_nonce(context: &[u8], nonce: u64, mask_length: u64) -> [u8; 12] {
    let mut state = 0x1319_8a2e_0370_7344 ^ nonce.rotate_left(29) ^ mask_length.rotate_left(7);
    for byte in context.iter().rev() {
        state = splitmix64(state ^ *byte as u64);
    }
    for byte in APATE_INTERNAL_KEY.iter().rev() {
        state = splitmix64(state ^ *byte as u64);
    }

    let mut nonce_bytes = [0_u8; 12];
    let first = splitmix64(state).to_le_bytes();
    let second = splitmix64(state ^ 0xa409_3822_299f_31d0).to_le_bytes();
    nonce_bytes[..8].copy_from_slice(&first);
    nonce_bytes[8..].copy_from_slice(&second[..4]);
    nonce_bytes
}

fn splitmix64(value: u64) -> u64 {
    let mut z = value;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

fn read_plain_extension_footer(
    file: &mut (impl Read + Seek),
    file_length: u64,
    mask_length: u32,
) -> Result<RestoreMetadata> {
    let minimum_v2_length = MASK_LENGTH_INDICATOR_LENGTH
        + PLAIN_EXTENSION_FOOTER_MAGIC_LENGTH
        + EXTENSION_LENGTH_FIELD_LENGTH;
    if file_length < minimum_v2_length {
        return Ok(RestoreMetadata::Plain {
            original_extension: None,
            byte_length: 0,
        });
    }

    let magic_start =
        file_length - MASK_LENGTH_INDICATOR_LENGTH - PLAIN_EXTENSION_FOOTER_MAGIC_LENGTH;
    file.seek(SeekFrom::Start(magic_start))?;
    let mut magic = [0_u8; PLAIN_EXTENSION_FOOTER_MAGIC.len()];
    file.read_exact(&mut magic)?;
    if &magic != PLAIN_EXTENSION_FOOTER_MAGIC {
        return Ok(RestoreMetadata::Plain {
            original_extension: None,
            byte_length: 0,
        });
    }

    let Some(length_start) = magic_start.checked_sub(EXTENSION_LENGTH_FIELD_LENGTH) else {
        return Ok(RestoreMetadata::Plain {
            original_extension: None,
            byte_length: 0,
        });
    };
    file.seek(SeekFrom::Start(length_start))?;
    let mut length_bytes = [0_u8; 2];
    file.read_exact(&mut length_bytes)?;
    let extension_length = u16::from_le_bytes(length_bytes) as u64;
    let footer_length =
        extension_length + EXTENSION_LENGTH_FIELD_LENGTH + PLAIN_EXTENSION_FOOTER_MAGIC_LENGTH;

    if file_length < MASK_LENGTH_INDICATOR_LENGTH + footer_length + mask_length as u64 {
        return Ok(RestoreMetadata::Plain {
            original_extension: None,
            byte_length: 0,
        });
    }

    let Some(extension_start) = length_start.checked_sub(extension_length) else {
        return Ok(RestoreMetadata::Plain {
            original_extension: None,
            byte_length: 0,
        });
    };
    file.seek(SeekFrom::Start(extension_start))?;
    let mut extension = vec![0_u8; extension_length as usize];
    file.read_exact(&mut extension)?;
    let Ok(extension) = String::from_utf8(extension) else {
        return Ok(RestoreMetadata::Plain {
            original_extension: None,
            byte_length: 0,
        });
    };
    if validate_extension_text(&extension).is_err() {
        return Ok(RestoreMetadata::Plain {
            original_extension: None,
            byte_length: 0,
        });
    }

    Ok(RestoreMetadata::Plain {
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

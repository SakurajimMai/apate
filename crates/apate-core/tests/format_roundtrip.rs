use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use apate_core::{
    ApateError, MASK_LENGTH_INDICATOR_LENGTH, MaskKind, builtin_mask, collect_input_files,
    disguise_file, inspect_file, original_extension, reveal_file,
};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new() -> Self {
        let nonce = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("apate-core-test-{nanos}-{nonce}"));
        fs::create_dir(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn disguise_appends_little_endian_mask_length_and_reversed_original_header() {
    let dir = TestDir::new();
    let file = dir.path().join("payload.bin");
    fs::write(&file, b"abcdef").unwrap();

    disguise_file(&file, b"XYZ").unwrap();

    let bytes = fs::read(&file).unwrap();
    assert_eq!(&bytes[..3], b"XYZ");
    assert_eq!(&bytes[6..9], b"cba");
    assert_eq!(&bytes[9..12], b"bin");
    assert_eq!(&bytes[12..14], &3_u16.to_le_bytes());
    assert_eq!(&bytes[14..22], b"APATE2EX");
    assert_eq!(&bytes[22..26], &3_i32.to_le_bytes());
    assert_eq!(
        bytes.len(),
        6 + 3 + 3 + 2 + 8 + MASK_LENGTH_INDICATOR_LENGTH as usize
    );
}

#[test]
fn reveal_restores_file_when_mask_is_shorter_than_original_file() {
    let dir = TestDir::new();
    let file = dir.path().join("payload.bin");
    let original = b"abcdef0123456789";
    fs::write(&file, original).unwrap();

    disguise_file(&file, builtin_mask(MaskKind::Jpg).bytes).unwrap();
    reveal_file(&file, false).unwrap();

    assert_eq!(fs::read(&file).unwrap(), original);
}

#[test]
fn disguise_records_original_extension_for_default_reveal_name() {
    let dir = TestDir::new();
    let file = dir.path().join("payload.zip");
    fs::write(&file, b"abcdef0123456789").unwrap();

    disguise_file(&file, builtin_mask(MaskKind::Jpg).bytes).unwrap();

    assert_eq!(original_extension(&file).unwrap(), Some("zip".to_string()));
}

#[test]
fn reveal_restores_file_when_mask_is_longer_than_original_file() {
    let dir = TestDir::new();
    let file = dir.path().join("tiny.bin");
    let original = b"tiny";
    fs::write(&file, original).unwrap();

    disguise_file(&file, builtin_mask(MaskKind::Exe).bytes).unwrap();
    reveal_file(&file, false).unwrap();

    assert_eq!(fs::read(&file).unwrap(), original);
}

#[test]
fn original_extension_is_none_for_old_metadata_without_extension() {
    let dir = TestDir::new();
    let file = dir.path().join("old-format.bin");
    let mut bytes = Vec::new();
    bytes.extend_from_slice(builtin_mask(MaskKind::Jpg).bytes);
    bytes.extend_from_slice(b"ef0123456789");
    bytes.extend_from_slice(b"dcba");
    bytes.extend_from_slice(&4_i32.to_le_bytes());
    fs::write(&file, bytes).unwrap();

    assert_eq!(original_extension(&file).unwrap(), None);
    reveal_file(&file, false).unwrap();
    assert_eq!(fs::read(&file).unwrap(), b"abcdef0123456789");
}

#[test]
fn inspect_rejects_plain_file_and_accepts_apate_disguised_file() {
    let dir = TestDir::new();
    let file = dir.path().join("plain.bin");
    fs::write(&file, b"plain").unwrap();

    let plain = inspect_file(&file).unwrap();
    assert!(!plain.disguised);
    assert_eq!(plain.mask_length, None);

    disguise_file(&file, builtin_mask(MaskKind::Mov).bytes).unwrap();
    let disguised = inspect_file(&file).unwrap();

    assert!(disguised.disguised);
    assert_eq!(disguised.mask_length, Some(4));
    assert_eq!(disguised.payload_length, Some(5));
}

#[test]
fn default_reveal_rejects_plain_file_with_plausible_length_trailer() {
    let dir = TestDir::new();
    let file = dir.path().join("plain.bin");
    fs::write(&file, b"hello\x01\0\0\0").unwrap();
    let original = fs::read(&file).unwrap();

    let inspection = inspect_file(&file).unwrap();
    assert!(!inspection.disguised);
    assert_eq!(inspection.mask_length, None);

    let error = reveal_file(&file, false).unwrap_err();
    assert!(matches!(error, ApateError::NotDisguised));
    assert_eq!(fs::read(&file).unwrap(), original);
}

#[test]
fn force_reveal_restores_custom_mask_file() {
    let dir = TestDir::new();
    let file = dir.path().join("custom-mask.bin");
    let original = b"abcdef0123456789";
    fs::write(&file, original).unwrap();

    disguise_file(&file, b"XYZ").unwrap();
    reveal_file(&file, true).unwrap();

    assert_eq!(fs::read(&file).unwrap(), original);
}

#[test]
fn builtin_masks_match_simple_headers() {
    assert_eq!(builtin_mask(MaskKind::Jpg).bytes, &[0xff, 0xd8, 0xff, 0xe1]);
    assert_eq!(builtin_mask(MaskKind::Mov).bytes, b"moov");
    assert_eq!(&builtin_mask(MaskKind::Mp4).bytes[..8], b"\0\0\0 ftyp");
    assert_eq!(&builtin_mask(MaskKind::Exe).bytes[..2], b"MZ");
}

#[test]
fn collect_input_files_supports_single_files_and_recursive_directories() {
    let dir = TestDir::new();
    let nested = dir.path().join("nested");
    fs::create_dir(&nested).unwrap();
    let first = dir.path().join("a.bin");
    let second = nested.join("b.bin");
    fs::write(&first, b"a").unwrap();
    fs::write(&second, b"b").unwrap();

    assert_eq!(
        collect_input_files(&first, false).unwrap(),
        vec![first.clone()]
    );

    let mut files = collect_input_files(dir.path(), true).unwrap();
    files.sort();
    assert_eq!(files, vec![first, second]);
}

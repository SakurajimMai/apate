use apate_core::{ApateError, Inspection};
use serde::Serialize;

#[cfg(unix)]
use std::fs::File;
#[cfg(unix)]
use std::os::fd::FromRawFd;

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::{jint, jstring};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeResponse {
    ok: bool,
    code: &'static str,
    message: String,
    disguised: Option<bool>,
    mask_length: Option<u32>,
    payload_length: Option<u64>,
    original_extension: Option<String>,
}

impl NativeResponse {
    #[cfg_attr(not(unix), allow(dead_code))]
    fn ok_inspection(inspection: Inspection, original_extension: Option<String>) -> Self {
        Self {
            ok: true,
            code: "ok",
            message: "处理成功".to_string(),
            disguised: Some(inspection.disguised),
            mask_length: inspection.mask_length,
            payload_length: inspection.payload_length,
            original_extension,
        }
    }

    #[cfg_attr(not(unix), allow(dead_code))]
    fn ok_restore(original_extension: Option<String>) -> Self {
        Self {
            ok: true,
            code: "ok",
            message: "处理成功".to_string(),
            disguised: None,
            mask_length: None,
            payload_length: None,
            original_extension,
        }
    }

    fn failure(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            ok: false,
            code,
            message: message.into(),
            disguised: None,
            mask_length: None,
            payload_length: None,
            original_extension: None,
        }
    }

    #[cfg_attr(not(unix), allow(dead_code))]
    fn from_error(error: ApateError) -> Self {
        Self::failure(error_code(&error), error.to_string())
    }

    fn json(&self) -> String {
        serde_json::to_string(self).expect("NativeResponse 必须可序列化")
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_moe_sakurajimamai_apate_NativeBridge_inspectFd(
    env: JNIEnv,
    _class: JClass,
    fd: jint,
) -> jstring {
    response_to_jstring(env, inspect_fd(fd))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_moe_sakurajimamai_apate_NativeBridge_revealInPlaceFd(
    env: JNIEnv,
    _class: JClass,
    fd: jint,
) -> jstring {
    response_to_jstring(env, reveal_in_place_fd(fd))
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_moe_sakurajimamai_apate_NativeBridge_restoreToFd(
    env: JNIEnv,
    _class: JClass,
    input_fd: jint,
    output_fd: jint,
) -> jstring {
    response_to_jstring(env, restore_to_fd(input_fd, output_fd))
}

fn response_to_jstring(env: JNIEnv, response: NativeResponse) -> jstring {
    match env.new_string(response.json()) {
        Ok(value) => value.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

#[cfg(unix)]
fn inspect_fd(fd: jint) -> NativeResponse {
    let mut file = unsafe { File::from_raw_fd(fd) };
    let inspection = match apate_core::inspect_reader(&mut file) {
        Ok(inspection) => inspection,
        Err(error) => return NativeResponse::from_error(error),
    };
    let original_extension = if inspection.disguised {
        match apate_core::original_extension_reader(&mut file) {
            Ok(extension) => extension,
            Err(error) => return NativeResponse::from_error(error),
        }
    } else {
        None
    };
    NativeResponse::ok_inspection(inspection, original_extension)
}

#[cfg(not(unix))]
fn inspect_fd(_fd: jint) -> NativeResponse {
    NativeResponse::failure("unsupported_platform", "当前平台不支持 Android fd")
}

#[cfg(unix)]
fn reveal_in_place_fd(fd: jint) -> NativeResponse {
    let mut file = unsafe { File::from_raw_fd(fd) };
    let original_extension = match apate_core::original_extension_reader(&mut file) {
        Ok(extension) => extension,
        Err(error) => return NativeResponse::from_error(error),
    };
    match apate_core::reveal_seekable(&mut file, false) {
        Ok(()) => NativeResponse::ok_restore(original_extension),
        Err(error) => NativeResponse::from_error(error),
    }
}

#[cfg(not(unix))]
fn reveal_in_place_fd(_fd: jint) -> NativeResponse {
    NativeResponse::failure("unsupported_platform", "当前平台不支持 Android fd")
}

#[cfg(unix)]
fn restore_to_fd(input_fd: jint, output_fd: jint) -> NativeResponse {
    let input = unsafe { File::from_raw_fd(input_fd) };
    let mut output = unsafe { File::from_raw_fd(output_fd) };
    match apate_core::restore_to_writer(input, &mut output, false) {
        Ok(original_extension) => NativeResponse::ok_restore(original_extension),
        Err(error) => NativeResponse::from_error(error),
    }
}

#[cfg(not(unix))]
fn restore_to_fd(_input_fd: jint, _output_fd: jint) -> NativeResponse {
    NativeResponse::failure("unsupported_platform", "当前平台不支持 Android fd")
}

#[cfg_attr(not(unix), allow(dead_code))]
fn error_code(error: &ApateError) -> &'static str {
    match error {
        ApateError::Io(_) => "io_error",
        ApateError::EmptyMask => "empty_mask",
        ApateError::MaskTooLarge { .. } => "mask_too_large",
        ApateError::NotDisguised => "not_disguised",
        ApateError::OutputExists(_) => "output_exists",
        ApateError::InvalidArguments(_) => "invalid_arguments",
        ApateError::MissingPath(_) => "missing_path",
        ApateError::DirectoryRequiresRecursive(_) => "directory_requires_recursive",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn serializes_successful_inspection_for_kotlin() {
        let response = NativeResponse::ok_inspection(
            Inspection {
                disguised: true,
                mask_length: Some(4),
                payload_length: Some(16),
            },
            Some("zip".to_string()),
        );

        let json: Value = serde_json::from_str(&response.json()).unwrap();

        assert_eq!(json["ok"], true);
        assert_eq!(json["code"], "ok");
        assert_eq!(json["disguised"], true);
        assert_eq!(json["maskLength"], 4);
        assert_eq!(json["payloadLength"], 16);
        assert_eq!(json["originalExtension"], "zip");
    }

    #[test]
    fn maps_not_disguised_error_for_android_ui() {
        let response = NativeResponse::from_error(ApateError::NotDisguised);
        let json: Value = serde_json::from_str(&response.json()).unwrap();

        assert_eq!(json["ok"], false);
        assert_eq!(json["code"], "not_disguised");
        assert!(json["message"].as_str().unwrap().contains("apate"));
    }
}

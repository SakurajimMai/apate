use std::ffi::OsString;
use std::fmt;
use std::mem;
use std::os::windows::ffi::OsStringExt;
use std::path::{Path, PathBuf};
use std::ptr;

use apate_core::{MaskKind, disguise_file, inspect_file, one_key_mask, reveal_file};
use windows_sys::Win32::Foundation::{COLORREF, HWND, LPARAM, LRESULT, POINT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, CLIP_DEFAULT_PRECIS, CreateFontW, CreateSolidBrush, DEFAULT_CHARSET, DEFAULT_PITCH,
    DEFAULT_QUALITY, DT_CENTER, DT_SINGLELINE, DT_VCENTER, DT_WORDBREAK, DeleteObject, DrawTextW,
    EndPaint, FF_DONTCARE, FW_BOLD, FW_NORMAL, FillRect, HBRUSH, HDC, HGDIOBJ, InvalidateRect,
    OUT_DEFAULT_PRECIS, PAINTSTRUCT, SelectObject, SetBkMode, SetTextColor, TRANSPARENT,
    UpdateWindow,
};
use windows_sys::Win32::System::Console::FreeConsole;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Shell::{
    DragAcceptFiles, DragFinish, DragQueryFileW, DragQueryPoint, HDROP,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateMenu, CreateWindowExW,
    DefWindowProcW, DispatchMessageW, GWLP_USERDATA, GetClientRect, GetMessageW, GetWindowLongPtrW,
    HMENU, IDC_ARROW, LoadCursorW, MB_ICONERROR, MB_ICONINFORMATION, MB_OK, MF_POPUP, MF_STRING,
    MSG, MessageBoxW, PostQuitMessage, RegisterClassW, SW_SHOW, SetMenu, SetWindowLongPtrW,
    ShowWindow, TranslateMessage, WM_COMMAND, WM_CREATE, WM_DESTROY, WM_DROPFILES, WM_NCCREATE,
    WM_PAINT, WNDCLASSW, WS_EX_ACCEPTFILES, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

use crate::{
    builtin_selected_mask, disguise_output_path, display_path, ensure_output_available,
    rename_if_needed, reveal_output_path,
};

const IDM_MASK_MP4: usize = 101;
const IDM_MASK_JPG: usize = 102;
const IDM_MASK_EXE: usize = 103;
const IDM_MASK_MOV: usize = 104;
const IDM_HELP_USAGE: usize = 201;
const IDM_HELP_JPG: usize = 202;

const COLOR_HINT: COLORREF = rgb(142, 232, 140);
const COLOR_DISGUISE: COLORREF = rgb(255, 211, 0);
const COLOR_REVEAL: COLORREF = rgb(223, 112, 224);
const COLOR_STATUS: COLORREF = rgb(245, 245, 245);

const fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    red as u32 | ((green as u32) << 8) | ((blue as u32) << 16)
}

pub(crate) fn run_gui() -> Result<(), GuiError> {
    let instance = unsafe { GetModuleHandleW(ptr::null()) };
    if instance.is_null() {
        return Err(GuiError::windows("获取程序实例失败".to_string()));
    }

    let class_name = wide("ApateDragWindow");
    let window_title = wide("Apate");
    let mut state = Box::new(GuiState::default());

    let window_class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        hInstance: instance,
        hCursor: unsafe { LoadCursorW(ptr::null_mut(), IDC_ARROW) },
        hbrBackground: ptr::null_mut(),
        lpszClassName: class_name.as_ptr(),
        ..unsafe { mem::zeroed() }
    };

    if unsafe { RegisterClassW(&window_class) } == 0 {
        return Err(GuiError::windows("注册窗口类失败".to_string()));
    }

    let hwnd = unsafe {
        CreateWindowExW(
            WS_EX_ACCEPTFILES,
            class_name.as_ptr(),
            window_title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1040,
            640,
            ptr::null_mut(),
            ptr::null_mut(),
            instance,
            state.as_mut() as *mut GuiState as *mut _,
        )
    };
    if hwnd.is_null() {
        return Err(GuiError::windows("创建窗口失败".to_string()));
    }

    mem::forget(state);
    unsafe {
        FreeConsole();
        SetMenu(hwnd, create_menu());
        DragAcceptFiles(hwnd, 1);
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }

    let mut message = unsafe { mem::zeroed::<MSG>() };
    while unsafe { GetMessageW(&mut message, ptr::null_mut(), 0, 0) } > 0 {
        unsafe {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    Ok(())
}

#[derive(Debug)]
pub(crate) struct GuiError {
    message: String,
}

impl GuiError {
    fn windows(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for GuiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for GuiError {}

struct GuiState {
    disguise_mask: GuiMask,
    status: String,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            disguise_mask: GuiMask::OneKeyMp4,
            status: "可一次拖入多个文件：中间区域批量伪装，右侧区域批量还原，左侧区域批量检查。"
                .to_string(),
        }
    }
}

#[derive(Clone, Copy)]
enum GuiMask {
    OneKeyMp4,
    Jpg,
    Exe,
    Mov,
}

impl GuiMask {
    fn label(self) -> &'static str {
        match self {
            Self::OneKeyMp4 => "MP4",
            Self::Jpg => "JPG",
            Self::Exe => "EXE",
            Self::Mov => "MOV",
        }
    }
}

#[derive(Clone, Copy)]
enum DropZone {
    Hint,
    Disguise,
    Reveal,
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_NCCREATE => {
            let create = lparam as *const CREATESTRUCTW;
            let state = unsafe { (*create).lpCreateParams as *mut GuiState };
            unsafe { SetWindowLongPtrW(hwnd, GWLP_USERDATA, state as isize) };
            1
        }
        WM_CREATE => 0,
        WM_COMMAND => {
            let command_id = wparam & 0xffff;
            if let Some(state) = unsafe { state_mut(hwnd) } {
                match command_id {
                    IDM_MASK_MP4 => state.disguise_mask = GuiMask::OneKeyMp4,
                    IDM_MASK_JPG => state.disguise_mask = GuiMask::Jpg,
                    IDM_MASK_EXE => state.disguise_mask = GuiMask::Exe,
                    IDM_MASK_MOV => state.disguise_mask = GuiMask::Mov,
                    IDM_HELP_USAGE => show_info(
                        hwnd,
                        "使用方式",
                        "把文件拖到中间区域：伪装为当前选择的格式。\n把文件拖到右侧区域：还原 Apate 文件。\n默认推荐 MP4，更适合网盘场景。",
                    ),
                    IDM_HELP_JPG => show_info(
                        hwnd,
                        "JPG 打不开说明",
                        "伪装成 .jpg 不等于生成真实照片。\n图片查看器打不开不代表原文件损坏；只要还原后内容一致，原文件就是正常的。\n如果需要可预览的图片外壳，应使用真实图片作为外层载体，这属于另一个格式策略。",
                    ),
                    _ => {}
                }
                invalidate(hwnd);
            }
            0
        }
        WM_DROPFILES => {
            let drop = wparam as HDROP;
            let paths = unsafe { dropped_files(drop) };
            let point = unsafe { drop_point(drop) };
            unsafe { DragFinish(drop) };

            if !paths.is_empty() {
                let zone = drop_zone(hwnd, point);
                if let Some(state) = unsafe { state_mut(hwnd) } {
                    let summary = process_paths_for_gui(&paths, zone, state.disguise_mask);
                    state.status = summary.message.clone();
                    if summary.fail_count > 0 {
                        let mut detail = summary.message.clone();
                        if let Some(first_error) = summary.first_error {
                            detail.push_str("\n");
                            detail.push_str(&format!("首个错误：{first_error}"));
                        }
                        show_error(hwnd, "批量处理结果", &detail);
                    }
                    invalidate(hwnd);
                }
            }
            0
        }
        WM_PAINT => {
            if let Some(state) = unsafe { state_mut(hwnd) } {
                unsafe { paint(hwnd, state) };
            }
            0
        }
        WM_DESTROY => {
            let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut GuiState };
            if !state.is_null() {
                unsafe {
                    drop(Box::from_raw(state));
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                }
            }
            unsafe { PostQuitMessage(0) };
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, message, wparam, lparam) },
    }
}

fn inspect_for_gui(path: &Path) -> Result<String, String> {
    let inspection = inspect_file(path).map_err(|error| error.to_string())?;
    if inspection.disguised {
        Ok(format!("已识别为 Apate 文件：{}", display_path(path)))
    } else {
        Ok(format!("未识别为 Apate 文件：{}", display_path(path)))
    }
}

fn disguise_for_gui(path: &Path, mask: GuiMask) -> Result<String, String> {
    let selected = match mask {
        GuiMask::OneKeyMp4 => crate::SelectedMask {
            bytes: one_key_mask().to_vec(),
            extension: ".mp4".to_string(),
        },
        GuiMask::Jpg => builtin_selected_mask(MaskKind::Jpg),
        GuiMask::Exe => builtin_selected_mask(MaskKind::Exe),
        GuiMask::Mov => builtin_selected_mask(MaskKind::Mov),
    };
    let output_path = disguise_output_path(path, &selected.extension);
    ensure_output_available(Some(&output_path)).map_err(|error| error.to_string())?;
    disguise_file(path, &selected.bytes).map_err(|error| error.to_string())?;
    rename_if_needed(path, Some(&output_path)).map_err(|error| error.to_string())?;
    Ok(format!(
        "完成：{} -> {}",
        display_path(path),
        display_path(&output_path)
    ))
}

fn reveal_for_gui(path: &Path) -> Result<String, String> {
    let output_path = reveal_output_path(path).map_err(|error| error.to_string())?;
    if let Some(output_path) = output_path.as_deref() {
        ensure_output_available(Some(output_path)).map_err(|error| error.to_string())?;
    }
    reveal_file(path, false).map_err(|error| error.to_string())?;
    rename_if_needed(path, output_path.as_deref()).map_err(|error| error.to_string())?;
    let output = output_path.as_deref().unwrap_or(path);
    Ok(format!(
        "完成：{} -> {}",
        display_path(path),
        display_path(output)
    ))
}

struct GuiBatchSummary {
    fail_count: usize,
    message: String,
    first_error: Option<String>,
}

fn process_paths_for_gui(paths: &[PathBuf], zone: DropZone, mask: GuiMask) -> GuiBatchSummary {
    let mut fail_count = 0;
    let mut first_error = None;

    for path in paths {
        let result = match zone {
            DropZone::Hint => inspect_for_gui(path),
            DropZone::Disguise => disguise_for_gui(path, mask),
            DropZone::Reveal => reveal_for_gui(path),
        };
        match result {
            Ok(_) => {}
            Err(error) => {
                fail_count += 1;
                if first_error.is_none() {
                    first_error = Some(format!("{}：{error}", display_path(path)));
                }
            }
        }
    }

    let action = zone.action_label();
    let message = if paths.is_empty() {
        "没有可处理的文件".to_string()
    } else {
        let ok_count = paths.len().saturating_sub(fail_count);
        format!("批量{action}完成：成功 {ok_count} 个，失败 {fail_count} 个")
    };

    GuiBatchSummary {
        fail_count,
        message,
        first_error,
    }
}

unsafe fn paint(hwnd: HWND, state: &GuiState) {
    let mut paint = unsafe { mem::zeroed::<PAINTSTRUCT>() };
    let hdc = unsafe { BeginPaint(hwnd, &mut paint) };
    let mut client = unsafe { mem::zeroed::<RECT>() };
    unsafe { GetClientRect(hwnd, &mut client) };

    let status_height = 42;
    let gap = 12;
    let width = client.right - client.left;
    let height = client.bottom - client.top - status_height;
    let column = (width - gap * 2) / 3;

    let hint = RECT {
        left: 0,
        top: 0,
        right: column,
        bottom: height,
    };
    let disguise = RECT {
        left: column + gap,
        top: 0,
        right: column * 2 + gap,
        bottom: height,
    };
    let reveal = RECT {
        left: column * 2 + gap * 2,
        top: 0,
        right: width,
        bottom: height,
    };
    let status = RECT {
        left: 0,
        top: height,
        right: width,
        bottom: client.bottom,
    };

    unsafe {
        draw_panel(
            hdc,
            hint,
            COLOR_HINT,
            "提示\nJPG 打不开不等于损坏\n还原后正常即可",
            26,
            FW_BOLD,
            DT_CENTER | DT_VCENTER | DT_WORDBREAK,
        );
        draw_panel(
            hdc,
            disguise,
            COLOR_DISGUISE,
            &format!("拖入\n进行伪装\n当前：{}", state.disguise_mask.label()),
            32,
            FW_BOLD,
            DT_CENTER | DT_VCENTER | DT_WORDBREAK,
        );
        draw_panel(
            hdc,
            reveal,
            COLOR_REVEAL,
            "拖入\n进行还原",
            34,
            FW_BOLD,
            DT_CENTER | DT_VCENTER | DT_WORDBREAK,
        );
        draw_status(hdc, status, &state.status);
        EndPaint(hwnd, &paint);
    }
}

unsafe fn draw_panel(
    hdc: HDC,
    rect: RECT,
    color: COLORREF,
    text: &str,
    font_size: i32,
    weight: u32,
    format: u32,
) {
    let brush = unsafe { CreateSolidBrush(color) };
    unsafe {
        FillRect(hdc, &rect, brush);
        DeleteObject(brush);
        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, rgb(0, 0, 0));
    }

    let mut text_rect = rect;
    text_rect.top += (rect.bottom - rect.top) / 3;
    draw_text(hdc, &mut text_rect, text, font_size, weight, format);
}

unsafe fn draw_status(hdc: HDC, rect: RECT, text: &str) {
    let brush: HBRUSH = unsafe { CreateSolidBrush(COLOR_STATUS) };
    unsafe {
        FillRect(hdc, &rect, brush);
        DeleteObject(brush);
        SetBkMode(hdc, TRANSPARENT as i32);
        SetTextColor(hdc, rgb(0, 0, 0));
    }

    let mut text_rect = rect;
    text_rect.left += 8;
    draw_text(
        hdc,
        &mut text_rect,
        text,
        20,
        FW_NORMAL,
        DT_VCENTER | DT_SINGLELINE,
    );
}

fn draw_text(hdc: HDC, rect: &mut RECT, text: &str, font_size: i32, weight: u32, format: u32) {
    let face = wide("Microsoft YaHei UI");
    let text = wide(text);
    unsafe {
        let font = CreateFontW(
            -font_size,
            0,
            0,
            0,
            weight as i32,
            0,
            0,
            0,
            DEFAULT_CHARSET as u32,
            OUT_DEFAULT_PRECIS as u32,
            CLIP_DEFAULT_PRECIS as u32,
            DEFAULT_QUALITY as u32,
            (DEFAULT_PITCH | FF_DONTCARE) as u32,
            face.as_ptr(),
        );
        let previous = if font.is_null() {
            ptr::null_mut()
        } else {
            SelectObject(hdc, font as HGDIOBJ)
        };
        DrawTextW(hdc, text.as_ptr(), -1, rect, format);
        if !previous.is_null() {
            SelectObject(hdc, previous);
        }
        if !font.is_null() {
            DeleteObject(font as HGDIOBJ);
        }
    }
}

fn drop_zone(hwnd: HWND, point: POINT) -> DropZone {
    let mut client = unsafe { mem::zeroed::<RECT>() };
    unsafe { GetClientRect(hwnd, &mut client) };
    let width = client.right - client.left;
    if point.x < width / 3 {
        DropZone::Hint
    } else if point.x < width * 2 / 3 {
        DropZone::Disguise
    } else {
        DropZone::Reveal
    }
}

impl DropZone {
    fn action_label(self) -> &'static str {
        match self {
            Self::Hint => "检查",
            Self::Disguise => "伪装",
            Self::Reveal => "还原",
        }
    }
}

unsafe fn state_mut(hwnd: HWND) -> Option<&'static mut GuiState> {
    let state = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut GuiState };
    if state.is_null() {
        None
    } else {
        Some(unsafe { &mut *state })
    }
}

unsafe fn dropped_files(drop: HDROP) -> Vec<PathBuf> {
    let count = unsafe { DragQueryFileW(drop, u32::MAX, ptr::null_mut(), 0) };
    if count == 0 {
        return Vec::new();
    }

    let mut paths = Vec::with_capacity(count as usize);
    for index in 0..count {
        let length = unsafe { DragQueryFileW(drop, index, ptr::null_mut(), 0) };
        if length == 0 {
            continue;
        }
        let mut buffer = vec![0_u16; length as usize + 1];
        unsafe { DragQueryFileW(drop, index, buffer.as_mut_ptr(), buffer.len() as u32) };
        buffer.truncate(length as usize);
        paths.push(PathBuf::from(OsString::from_wide(&buffer)));
    }

    paths
}

unsafe fn drop_point(drop: HDROP) -> POINT {
    let mut point = POINT { x: 0, y: 0 };
    unsafe { DragQueryPoint(drop, &mut point) };
    point
}

fn create_menu() -> HMENU {
    unsafe {
        let menu = CreateMenu();
        let option = CreateMenu();
        let help = CreateMenu();

        AppendMenuW(
            option,
            MF_STRING,
            IDM_MASK_MP4,
            wide("默认伪装为 MP4").as_ptr(),
        );
        AppendMenuW(option, MF_STRING, IDM_MASK_JPG, wide("伪装为 JPG").as_ptr());
        AppendMenuW(option, MF_STRING, IDM_MASK_EXE, wide("伪装为 EXE").as_ptr());
        AppendMenuW(option, MF_STRING, IDM_MASK_MOV, wide("伪装为 MOV").as_ptr());
        AppendMenuW(menu, MF_POPUP, option as usize, wide("选项").as_ptr());

        AppendMenuW(help, MF_STRING, IDM_HELP_USAGE, wide("使用说明").as_ptr());
        AppendMenuW(
            help,
            MF_STRING,
            IDM_HELP_JPG,
            wide("JPG 无法打开说明").as_ptr(),
        );
        AppendMenuW(menu, MF_POPUP, help as usize, wide("帮助").as_ptr());
        menu
    }
}

fn show_info(hwnd: HWND, title: &str, message: &str) {
    let title = wide(title);
    let message = wide(message);
    unsafe {
        MessageBoxW(
            hwnd,
            message.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

fn show_error(hwnd: HWND, title: &str, message: &str) {
    let title = wide(title);
    let message = wide(message);
    unsafe {
        MessageBoxW(hwnd, message.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
    }
}

fn invalidate(hwnd: HWND) {
    unsafe {
        InvalidateRect(hwnd, ptr::null(), 1);
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

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
            let path = std::env::temp_dir().join(format!("apate-gui-test-{nanos}-{nonce}"));
            fs::create_dir(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn gui_batch_disguises_all_dropped_files() {
        let dir = TestDir::new();
        let first = dir.path().join("one.zip");
        let second = dir.path().join("two.zip");
        fs::write(&first, b"first payload").unwrap();
        fs::write(&second, b"second payload").unwrap();

        let summary = process_paths_for_gui(
            &[first.clone(), second.clone()],
            DropZone::Disguise,
            GuiMask::Jpg,
        );

        assert_eq!(summary.fail_count, 0);
        assert!(!first.exists());
        assert!(!second.exists());
        assert!(dir.path().join("one.jpg").exists());
        assert!(dir.path().join("two.jpg").exists());
        assert!(summary.message.contains("成功 2 个"));
    }

    #[test]
    fn gui_batch_reveals_all_dropped_files() {
        let dir = TestDir::new();
        let first = dir.path().join("one.zip");
        let second = dir.path().join("two.zip");
        let first_original = b"first payload";
        let second_original = b"second payload";
        fs::write(&first, first_original).unwrap();
        fs::write(&second, second_original).unwrap();
        process_paths_for_gui(
            &[first.clone(), second.clone()],
            DropZone::Disguise,
            GuiMask::Jpg,
        );
        let first_disguised = dir.path().join("one.jpg");
        let second_disguised = dir.path().join("two.jpg");

        let summary = process_paths_for_gui(
            &[first_disguised.clone(), second_disguised.clone()],
            DropZone::Reveal,
            GuiMask::Jpg,
        );

        assert_eq!(summary.fail_count, 0);
        assert!(first.exists());
        assert!(second.exists());
        assert!(!first_disguised.exists());
        assert!(!second_disguised.exists());
        assert_eq!(fs::read(&first).unwrap(), first_original);
        assert_eq!(fs::read(&second).unwrap(), second_original);
    }

    #[test]
    fn gui_batch_continues_after_one_file_fails() {
        let dir = TestDir::new();
        let ok = dir.path().join("ok.zip");
        let conflict_source = dir.path().join("conflict.zip");
        let conflict_target = dir.path().join("conflict.jpg");
        fs::write(&ok, b"ok payload").unwrap();
        fs::write(&conflict_source, b"conflict payload").unwrap();
        fs::write(&conflict_target, b"existing").unwrap();

        let summary = process_paths_for_gui(
            &[ok.clone(), conflict_source.clone()],
            DropZone::Disguise,
            GuiMask::Jpg,
        );

        assert_eq!(summary.fail_count, 1);
        assert!(!ok.exists());
        assert!(dir.path().join("ok.jpg").exists());
        assert!(conflict_source.exists());
        assert_eq!(fs::read(&conflict_target).unwrap(), b"existing");
        assert!(summary.message.contains("成功 1 个"));
        assert!(summary.message.contains("失败 1 个"));
    }
}

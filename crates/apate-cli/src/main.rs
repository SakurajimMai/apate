use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use apate_core::{
    ApateError, MaskKind, builtin_mask, builtin_masks, collect_input_files, disguise_file,
    inspect_file, one_key_mask, reveal_file, validate_mask,
};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

fn main() {
    let cli = Cli::parse();
    let result = match cli.command {
        Some(Command::Inspect(args)) => inspect_command(args),
        Some(Command::Masks(args)) => masks_command(args),
        Some(Command::Disguise(args)) => disguise_command(args),
        Some(Command::Reveal(args)) => reveal_command(args),
        Some(Command::Tui(args)) => tui_command(args),
        None => {
            print_top_level_help();
            Ok(())
        }
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

#[derive(Debug, Parser)]
#[command(name = "apate", version, about = "Rust 版文件格式伪装工具")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Inspect(InspectArgs),
    Masks(JsonArgs),
    Disguise(DisguiseArgs),
    Reveal(RevealArgs),
    Tui(TuiArgs),
}

#[derive(Debug, Args, Default)]
struct TuiArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct JsonArgs {
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct InspectArgs {
    path: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Args)]
struct DisguiseArgs {
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    one_key: bool,
    #[arg(long, value_enum)]
    kind: Option<CliMaskKind>,
    #[arg(long)]
    mask_file: Option<PathBuf>,
    #[arg(long)]
    recursive: bool,
    #[arg(long)]
    no_rename: bool,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Args)]
struct RevealArgs {
    #[arg(long)]
    input: PathBuf,
    #[arg(long)]
    recursive: bool,
    #[arg(long)]
    no_rename: bool,
    #[arg(long)]
    force: bool,
    #[arg(long)]
    json: bool,
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliMaskKind {
    Exe,
    Jpg,
    Mp4,
    Mov,
}

#[derive(Debug, Serialize)]
struct MasksOutput {
    masks: Vec<MaskOutput>,
}

#[derive(Debug, Serialize)]
struct MaskOutput {
    kind: &'static str,
    extension: &'static str,
    length: usize,
}

#[derive(Debug, Serialize)]
struct InspectOutput {
    path: String,
    disguised: bool,
    mask_length: Option<u32>,
    payload_length: Option<u64>,
}

#[derive(Debug, Serialize)]
struct BatchOutput {
    ok: bool,
    dry_run: bool,
    results: Vec<ActionOutput>,
}

#[derive(Debug, Serialize)]
struct ActionOutput {
    action: &'static str,
    path: String,
    output_path: Option<String>,
    ok: bool,
    code: &'static str,
    message: String,
}

struct SelectedMask {
    bytes: Vec<u8>,
    extension: String,
}

fn inspect_command(args: InspectArgs) -> Result<(), String> {
    let inspection = inspect_file(&args.path).map_err(|error| error.to_string())?;
    let output = InspectOutput {
        path: display_path(&args.path),
        disguised: inspection.disguised,
        mask_length: inspection.mask_length,
        payload_length: inspection.payload_length,
    };
    if args.json {
        print_json(&output)
    } else {
        println!(
            "{}: {}",
            output.path,
            if output.disguised {
                "旧格式伪装文件"
            } else {
                "未识别为旧格式伪装文件"
            }
        );
        Ok(())
    }
}

fn masks_command(args: JsonArgs) -> Result<(), String> {
    let masks = builtin_masks()
        .iter()
        .map(|mask| MaskOutput {
            kind: mask.name,
            extension: mask.extension,
            length: mask.bytes.len(),
        })
        .collect::<Vec<_>>();
    if args.json {
        print_json(&MasksOutput { masks })
    } else {
        for mask in masks {
            println!("{}\t{}\t{} bytes", mask.kind, mask.extension, mask.length);
        }
        Ok(())
    }
}

fn disguise_command(args: DisguiseArgs) -> Result<(), String> {
    let selected_mask = match select_mask_checked(&args) {
        Ok(selected_mask) => selected_mask,
        Err(error) => {
            return emit_batch(
                args.json,
                args.dry_run,
                vec![ActionOutput::failure("disguise", &args.input, None, error)],
            );
        }
    };
    let files =
        collect_input_files(&args.input, args.recursive).map_err(|error| error.to_string())?;
    let mut results = Vec::new();

    for path in files {
        let output_path = if args.no_rename {
            None
        } else {
            Some(PathBuf::from(format!(
                "{}{}",
                path.to_string_lossy(),
                selected_mask.extension
            )))
        };

        if let Err(error) = ensure_output_available(output_path.as_deref()) {
            results.push(ActionOutput::failure(
                "disguise",
                &path,
                output_path.as_deref(),
                error,
            ));
            continue;
        }

        if args.dry_run {
            results.push(ActionOutput::success(
                "disguise",
                &path,
                output_path.as_deref(),
                "dry_run",
            ));
            continue;
        }

        match disguise_file(&path, &selected_mask.bytes)
            .and_then(|_| rename_if_needed(&path, output_path.as_deref()))
        {
            Ok(()) => results.push(ActionOutput::success(
                "disguise",
                &path,
                output_path.as_deref(),
                "ok",
            )),
            Err(error) => results.push(ActionOutput::failure(
                "disguise",
                &path,
                output_path.as_deref(),
                error,
            )),
        }
    }

    emit_batch(args.json, args.dry_run, results)
}

fn reveal_command(args: RevealArgs) -> Result<(), String> {
    let files =
        collect_input_files(&args.input, args.recursive).map_err(|error| error.to_string())?;
    let mut results = Vec::new();

    for path in files {
        let output_path = if args.no_rename {
            None
        } else {
            path.file_stem()
                .map(|stem| path.with_file_name(stem))
                .filter(|renamed| renamed != &path)
        };

        if let Err(error) = ensure_output_available(output_path.as_deref()) {
            results.push(ActionOutput::failure(
                "reveal",
                &path,
                output_path.as_deref(),
                error,
            ));
            continue;
        }

        if args.dry_run {
            results.push(ActionOutput::success(
                "reveal",
                &path,
                output_path.as_deref(),
                "dry_run",
            ));
            continue;
        }

        match reveal_file(&path, args.force)
            .and_then(|_| rename_if_needed(&path, output_path.as_deref()))
        {
            Ok(()) => results.push(ActionOutput::success(
                "reveal",
                &path,
                output_path.as_deref(),
                "ok",
            )),
            Err(error) => results.push(ActionOutput::failure(
                "reveal",
                &path,
                output_path.as_deref(),
                error,
            )),
        }
    }

    emit_batch(args.json, args.dry_run, results)
}

fn tui_command(args: TuiArgs) -> Result<(), String> {
    if args.json {
        return Err("tui 模式不支持 --json".to_string());
    }
    run_tui()
}

#[allow(dead_code)]
fn select_mask(args: &DisguiseArgs) -> Result<SelectedMask, String> {
    let selected_count =
        args.one_key as usize + args.kind.is_some() as usize + args.mask_file.is_some() as usize;
    if selected_count != 1 {
        return Err("必须且只能选择一种面具来源: --one-key、--kind 或 --mask-file".to_string());
    }

    if args.one_key {
        let mask = builtin_mask(MaskKind::Mp4);
        return Ok(SelectedMask {
            bytes: include_bytes!("../../../apate/Resources/mask.mp4").to_vec(),
            extension: mask.extension.to_string(),
        });
    }

    if let Some(kind) = args.kind {
        let mask = builtin_mask(kind.into());
        return Ok(SelectedMask {
            bytes: mask.bytes.to_vec(),
            extension: mask.extension.to_string(),
        });
    }

    let mask_file = args
        .mask_file
        .as_ref()
        .expect("mask_file 已在数量检查中确认");
    let bytes = fs::read(mask_file).map_err(|error| error.to_string())?;
    let extension = mask_file
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()))
        .unwrap_or_default();
    Ok(SelectedMask { bytes, extension })
}

fn select_mask_checked(args: &DisguiseArgs) -> apate_core::Result<SelectedMask> {
    let selected_count =
        args.one_key as usize + args.kind.is_some() as usize + args.mask_file.is_some() as usize;
    if selected_count != 1 {
        return Err(ApateError::InvalidArguments(
            "必须且只能选择一种面具来源: --one-key、--kind 或 --mask-file".to_string(),
        ));
    }

    if args.one_key {
        let mask = builtin_mask(MaskKind::Mp4);
        return Ok(SelectedMask {
            bytes: one_key_mask().to_vec(),
            extension: mask.extension.to_string(),
        });
    }

    if let Some(kind) = args.kind {
        let mask = builtin_mask(kind.into());
        return Ok(SelectedMask {
            bytes: mask.bytes.to_vec(),
            extension: mask.extension.to_string(),
        });
    }

    let mask_file = args
        .mask_file
        .as_ref()
        .expect("mask_file 已在数量检查中确认");
    let bytes = fs::read(mask_file)?;
    validate_mask(&bytes)?;
    let extension = mask_file
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()))
        .unwrap_or_default();
    Ok(SelectedMask { bytes, extension })
}

fn rename_if_needed(path: &Path, output_path: Option<&Path>) -> apate_core::Result<()> {
    if let Some(output_path) = output_path {
        ensure_output_available(Some(output_path))?;
        fs::rename(path, output_path)?;
    }
    Ok(())
}

fn ensure_output_available(output_path: Option<&Path>) -> apate_core::Result<()> {
    if let Some(output_path) = output_path {
        if output_path.exists() {
            return Err(ApateError::OutputExists(output_path.to_path_buf()));
        }
    }
    Ok(())
}

fn print_top_level_help() {
    println!("apate TUI 模式已提供。");
    println!("运行 `apate tui` 进入交互菜单。");
    println!("运行 `apate <subcommand>` 使用直接命令模式。");
}

fn run_tui() -> Result<(), String> {
    let stdin = io::stdin();
    let mut input = String::new();
    loop {
        println!("apate TUI 模式");
        println!("1) inspect");
        println!("2) masks");
        println!("3) disguise");
        println!("4) reveal");
        println!("0) exit");
        println!("输入数字后回车");
        print!("选择: ");
        io::stdout().flush().map_err(|error| error.to_string())?;
        input.clear();
        stdin
            .read_line(&mut input)
            .map_err(|error| error.to_string())?;
        match input.trim() {
            "1" => {
                run_tui_inspect(&stdin)?;
                return Ok(());
            }
            "2" => {
                run_tui_masks()?;
                return Ok(());
            }
            "3" => {
                run_tui_disguise(&stdin)?;
                return Ok(());
            }
            "4" => {
                run_tui_reveal(&stdin)?;
                return Ok(());
            }
            "0" | "q" | "quit" | "exit" => return Ok(()),
            _ => println!("无效选择"),
        }
    }
}

fn run_tui_masks() -> Result<(), String> {
    for mask in builtin_masks() {
        println!(
            "{}\t{}\t{} bytes",
            mask.name,
            mask.extension,
            mask.bytes.len()
        );
    }
    Ok(())
}

fn run_tui_inspect(stdin: &io::Stdin) -> Result<(), String> {
    let path = prompt_path(stdin, "输入要检查的文件路径: ")?;
    let inspection = inspect_file(&path).map_err(|error| error.to_string())?;
    println!(
        "{}",
        if inspection.disguised {
            "旧格式伪装文件"
        } else {
            "未识别为旧格式伪装文件"
        }
    );
    Ok(())
}

fn run_tui_disguise(stdin: &io::Stdin) -> Result<(), String> {
    let path = prompt_path(stdin, "输入要伪装的文件路径: ")?;
    let kind = prompt_line(stdin, "输入面具类型(exe/jpg/mp4/mov/onekey): ")?;
    let selected_mask = match kind.as_str() {
        "onekey" => SelectedMask {
            bytes: include_bytes!("../../../apate/Resources/mask.mp4").to_vec(),
            extension: ".mp4".to_string(),
        },
        "exe" => builtin_selected_mask(MaskKind::Exe),
        "jpg" => builtin_selected_mask(MaskKind::Jpg),
        "mp4" => builtin_selected_mask(MaskKind::Mp4),
        "mov" => builtin_selected_mask(MaskKind::Mov),
        _ => return Err("未知面具类型".to_string()),
    };
    disguise_file(&path, &selected_mask.bytes).map_err(|error| error.to_string())?;
    println!("伪装完成");
    Ok(())
}

fn run_tui_reveal(stdin: &io::Stdin) -> Result<(), String> {
    let path = prompt_path(stdin, "输入要还原的文件路径: ")?;
    reveal_file(&path, false).map_err(|error| error.to_string())?;
    println!("还原完成");
    Ok(())
}

fn prompt_path(stdin: &io::Stdin, prompt: &str) -> Result<PathBuf, String> {
    let path = prompt_line(stdin, prompt)?;
    Ok(PathBuf::from(path))
}

fn prompt_line(stdin: &io::Stdin, prompt: &str) -> Result<String, String> {
    let mut input = String::new();
    print!("{prompt}");
    io::stdout().flush().map_err(|error| error.to_string())?;
    stdin
        .read_line(&mut input)
        .map_err(|error| error.to_string())?;
    Ok(input.trim().to_string())
}

fn builtin_selected_mask(kind: MaskKind) -> SelectedMask {
    let mask = builtin_mask(kind);
    SelectedMask {
        bytes: mask.bytes.to_vec(),
        extension: mask.extension.to_string(),
    }
}

fn emit_batch(json: bool, dry_run: bool, results: Vec<ActionOutput>) -> Result<(), String> {
    let ok = results.iter().all(|result| result.ok);
    let output = BatchOutput {
        ok,
        dry_run,
        results,
    };
    if json {
        print_json(&output)?;
    } else {
        for result in &output.results {
            println!(
                "{}\t{}\t{}",
                if result.ok { "ok" } else { "fail" },
                result.action,
                result.path
            );
        }
    }
    if ok {
        Ok(())
    } else {
        Err("部分文件处理失败".to_string())
    }
}

fn print_json(output: &impl Serialize) -> Result<(), String> {
    serde_json::to_writer_pretty(std::io::stdout(), output).map_err(|error| error.to_string())?;
    println!();
    Ok(())
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

impl ActionOutput {
    fn success(
        action: &'static str,
        path: &Path,
        output_path: Option<&Path>,
        code: &'static str,
    ) -> Self {
        Self {
            action,
            path: display_path(path),
            output_path: output_path.map(display_path),
            ok: true,
            code,
            message: "处理成功".to_string(),
        }
    }

    fn failure(
        action: &'static str,
        path: &Path,
        output_path: Option<&Path>,
        error: ApateError,
    ) -> Self {
        Self {
            action,
            path: display_path(path),
            output_path: output_path.map(display_path),
            ok: false,
            code: error_code(&error),
            message: error.to_string(),
        }
    }
}

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

impl From<CliMaskKind> for MaskKind {
    fn from(kind: CliMaskKind) -> Self {
        match kind {
            CliMaskKind::Exe => MaskKind::Exe,
            CliMaskKind::Jpg => MaskKind::Jpg,
            CliMaskKind::Mp4 => MaskKind::Mp4,
            CliMaskKind::Mov => MaskKind::Mov,
        }
    }
}

## 📑工具简介  
  apate是一款能够简洁、快速地对文件进行格式伪装的工具，可以在某些情况下绕过限制。  
  开源项目主页：[**_Github: SakurajimMai/apate_**](https://github.com/SakurajimMai/apate)  
  
## ⭐软件特性  
  1. 支持超大文件，可以做到瞬间伪装/还原完毕，无需任何等待。  
  2. 针对文件的还原做了优化，无需知道文件的伪装面具即可一键还原。  
  3. 针对真身文件的原始文件头做了加密处理，不易被检测出原始格式。  
  4. 支持批量拖拽，支持文件夹拖拽。  
  
## 📥下载方法
  1.安装运行环境：根据自己的操作系统，安装.NET桌面运行时6.0：[**_64位安装包_**](https://dotnet.microsoft.com/zh-cn/download/dotnet/thank-you/runtime-desktop-6.0.16-windows-x64-installer) 或者 [**_32位安装包_**](https://dotnet.microsoft.com/zh-cn/download/dotnet/thank-you/runtime-desktop-6.0.16-windows-x86-installer)  
  2.下载apate：最新版v1.4.2：[**_from Github_**](https://github.com/SakurajimMai/apate/releases) 或者 [**_from 蓝奏云_**](https://wwve.lanzoup.com/iEaSU0ymznza)  
  
## 📗使用说明  
  1. 一键伪装  
  使用预置面具文件，对真身文件进行伪装。伪装后，真身文件看起来与面具文件一样。适用大部分应用场景。  
  2. 面具伪装  
  使用自定义面具文件，对真身文件进行伪装。伪装后，真身文件看起来与面具文件一样。适用范围取决于面具文件的选择，建议在一键伪装失效时尝试使用。  
  3. 简易伪装  
  不使用面具文件，而是使用指定格式的二进制特征文件头，对真身文件进行伪装。伪装后，真身文件对于操作系统来说已经是指定格式，只是无法被双击执行或播放。对于EXE的简易伪装，建议真身文件不超过2G。其他格式的简易伪装适配场景较少，不建议使用。  
  
## ❗注意事项  
  1. 使用前请务必做好数据备份。  
  2. 本软件不得用于商业用途，仅做学习交流。  
  3. 本软件不得用于非法用途，用户使用本软件导致的任何后果均由用户本人承担，软件作者不承担任何责任。  
  
## 🙋FAQ  
### 1. copy /b a.jpg+b.zip 原理是这个吗？  
  技术上不同，但有类似之处。比copy命令伪装更快速、还原更方便，具体特性请参阅[软件特性](#软件特性)章节。  
### 2. 一键伪装只支持单一的mp4格式，建议增加更多选项  
  由于mp4格式适用范围最广，所以暂不考虑增加其他选项。如果需要使用其他格式，可以使用面具伪装模式，自定义面具文件。  
  
## 🆕更新记录  
  ### v1.4.2  
    bugfix: 优化界面布局，修复DPI改变时界面布局混乱的bug。  

## 🦀Rust 重构版

本仓库已开始提供 Rust workspace，用于替代原 WinForms 实现，并保留旧格式兼容能力。

### 结构

- `crates/apate-core`：旧格式伪装/还原算法、内置文件头、文件扫描。
- `crates/apate-cli`：单程序多模式入口，支持 `inspect`、`masks`、`disguise`、`reveal`、`tui`。
- `skills/apate-cli`：面向 agent 的 CLI 使用 skill。
- `docs/`：面向用户的文档（架构、API、CLI、用户指南）。详见 [`docs/README.md`](docs/README.md)。

### 构建与测试

```powershell
cargo test --workspace
cargo run -p apate-cli -- masks --json
cargo run -p apate-cli -- tui
```

### CLI 示例

```powershell
# 查看内置面具
cargo run -p apate-cli -- masks --json

# 检查文件是否为旧格式伪装文件
cargo run -p apate-cli -- inspect .\example.zip.mp4 --json

# 简易伪装为 JPG，默认会追加 .jpg 扩展名
cargo run -p apate-cli -- disguise --input .\example.zip --kind jpg --json

# 一键 MP4 面具伪装
cargo run -p apate-cli -- disguise --input .\example.zip --one-key --json

# 批量处理前先 dry-run
cargo run -p apate-cli -- disguise --input .\files --kind mp4 --recursive --dry-run --json

# 还原文件，默认会移除最后一个扩展名
cargo run -p apate-cli -- reveal --input .\example.zip.jpg --json

# 进入 TUI
cargo run -p apate-cli -- tui
```

### 旧格式兼容

Rust 版默认继续写入旧版 apate 格式：覆盖文件头，在尾部追加倒序原文件头，再追加 4 字节 little-endian 面具长度。这样旧版生成的伪装文件可以被 Rust 版还原，Rust 版生成的文件也保持同一格式语义。

### 发布

GitHub Actions 会在 `v*` tag 推送时构建 Windows 和 Linux 产物，并把 `CHANGELOG.md` 的 `Unreleased` 段作为 Release Notes 发布到 GitHub Releases。

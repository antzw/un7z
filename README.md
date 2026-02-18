# un7z

现代化的批量递归解压工具，支持多种分卷压缩格式，提供美观的命令行界面和实时进度显示。

## 🌟 版本说明

### Rust 版本（推荐）🦀
- ✅ **单个二进制文件** - 编译后直接运行，无需依赖
- ✅ **实时进度条** - 使用 `indicatif` 库，流畅更新
- ✅ **现代化界面** - 彩色输出、动画效果、多进度条
- ✅ **高性能** - Rust 实现，响应迅速
- ✅ **详细的命令行参数** - 支持更多选项

### Bash 版本（经典）📜
- 📦 纯 Shell 脚本，无需编译
- 📦 轻量级，适合简单场景
- 📦 兼容性好，只要 bash 就能运行

## ✨ 特性

### 通用特性
- 🔍 递归扫描子文件夹中的压缩包
- 📦 支持多种格式：7z/zip 分卷、rar 分卷、tar.gz/tgz
- 🎯 解压前完整列出归档，支持选择编号或范围（如 `1,3,5-7`）
- ✅ CRC/测试失败自动记录并跳过
- 📝 生成清晰的失败日志

### Rust 版本额外特性
- 🎨 **美观的彩色输出** - 使用 `console` 和 `termcolor`
- 📊 **实时多进度条** - 同时显示多个文件进度
- ⚡ **快速扫描** - 异步文件系统遍历
- 🔧 **丰富的命令行选项** - `--all`, `--test`, `--password`, `--verbose`
- 📈 **智能 ETA 计算** - 预估剩余时间

## 📋 依赖

### Rust 版本
**构建依赖：**
- Rust 工具链 (`cargo`, `rustc`)

**运行时依赖：**
- `7zz`（7-Zip）- 用于 7z/zip 分卷
- `unrar` - 用于 RAR 分卷
- `tar`, `gzip` - 用于 tar.gz

安装构建工具：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

安装运行时依赖：
```bash
# macOS
brew install p7zip unrar

# Ubuntu/Debian
sudo apt install p7zip-full unrar

# Arch Linux
sudo pacman -S p7zip unrar
```

### Bash 版本
- `bash`（macOS 自带 3.2 也可运行）
- `find`, `tar`
- `7zz`（7-Zip）
- `unrar`

> macOS 可用 Homebrew 安装：`brew install p7zip unrar`

## 📖 使用方法

### Rust 版本（推荐）

#### 编译
```bash
# 克隆或下载项目
cd un7z

# 编译发布版本
cargo build --release

# 编译后的二进制文件位于 target/release/un7z
```

#### 安装
```bash
# 安装到系统路径（可选）
cargo install --path .

# 或者手动复制
sudo cp target/release/un7z /usr/local/bin/
```

#### 命令行选项
```bash
un7z [OPTIONS]

选项：
  -d, --dir <DIR>          扫描目录（默认：当前目录）
  -a, --all                解压所有找到的压缩包（不询问）
  -t, --test              解压前进行完整性测试
  -p, --password <PWD>    加密压缩包的密码
  -v, --verbose           详细输出
  -h, --help              显示帮助信息
  -V, --version           显示版本信息
```

#### 使用示例

**交互式选择：**
```bash
# 在当前目录查找并交互式选择
un7z

# 在指定目录查找
un7z -d ~/Downloads/archives

# 带密码解压
un7z --password "mypassword"

# 解压前测试完整性
un7z --test
```

**自动解压所有：**
```bash
# 扫描并解压所有压缩包（不询问）
un7z --all

# 等同于
un7z -a
```

**组合使用：**
```bash
# 扫描指定目录，测试完整性，使用密码
un7z -d ~/Downloads -t -p "secret123"

# 详细模式
un7z --verbose
```

### Bash 版本

#### 运行
```bash
# 在脚本所在目录执行
./un7z.bash

# 或者（如果保留了原 un7z）
./un7z
```

#### 交互流程
1. 递归扫描压缩包并列出
2. 选择要解压的编号（如 `1,3,5-7`），或直接回车使用 `all`
3. 选择是否进行完整性测试（y/N）
4. 输入密码（没有则回车）
5. 显示单行进度条，逐个解压

## 📁 支持的文件类型

| 类型 | 模式 | 说明 |
|------|------|------|
| 7z 分卷 | `*.7z.001` | 7-Zip 分卷压缩 |
| ZIP 分卷 | `*.zip.001` | ZIP 分卷压缩 |
| RAR 分卷 | `*.part01.rar` | WinRAR 分卷（仅处理起始卷） |
| RAR 分卷 | `*.part001.rar` | WinRAR 分卷（仅处理起始卷） |
| Tar Gzip | `*.tar.gz` | Gzip 压缩的 Tar |
| Tar Gzip | `*.tgz` | Gzip 压缩的 Tar（简写） |

## 📝 输出与日志

### 日志文件
- `failed.log` - 解压失败的压缩包列表（便于重试）

### 退出状态
- `0` - 成功完成
- `1` - 出现错误

## 🆚 版本对比

| 特性 | Rust 版本 | Bash 版本 |
|------|-----------|-----------|
| 依赖 | 仅构建时需要 Rust | 需要 bash |
| 性能 | ⚡ 快 | 🐌 较慢 |
| 进度显示 | 📊 实时多进度条 | 📊 单文件进度条 |
| 界面 | 🎨 彩色现代化 | 📝 简洁文本 |
| 分发 | 📦 单个二进制 | 📜 脚本文件 |
| 安装 | 需要编译 | 直接运行 |

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📄 许可

MIT License

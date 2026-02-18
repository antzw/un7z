# 安装说明

## 前提条件

你的系统上需要安装 Rust 编译器。如果没有，请先安装：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

安装完成后，需要重启终端或者运行：

```bash
source ~/.cargo/env
```

## 安装运行时依赖

无论使用哪个版本，都需要安装解压工具：

**macOS:**
```bash
brew install p7zip unrar
```

**Ubuntu/Debian:**
```bash
sudo apt install p7zip-full unrar
```

**Arch Linux:**
```bash
sudo pacman -S p7zip unrar
```

## 编译 Rust 版本

### 方法 1: 使用安装脚本（推荐）

```bash
./build.sh install
```

这将：
1. 编译项目
2. 安装到 `/usr/local/bin/un7z`
3. 你可以在任何地方直接运行 `un7z`

### 方法 2: 只编译不安装

```bash
./build.sh
```

编译后的二进制文件位于：`target/release/un7z`

你可以直接运行：

```bash
./target/release/un7z --help
```

或者手动复制到其他位置：

```bash
cp target/release/un7z ~/bin/
# 或者
sudo cp target/release/un7z /usr/local/bin/
```

### 方法 3: 使用 cargo（如果你安装了 Rust）

```bash
# 安装到 ~/.cargo/bin/
cargo install --path .

# 确保 ~/.cargo/bin 在 PATH 中
export PATH="$HOME/.cargo/bin:$PATH"
```

## 验证安装

```bash
# 检查版本
un7z --version

# 查看帮助
un7z --help
```

## 故障排除

### cargo: command not found

**原因**: Rust 没有安装或 PATH 没有配置

**解决**:
```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 配置 PATH（如果还没有）
source ~/.cargo/env

# 或者在 ~/.bashrc 或 ~/.zshrc 中添加
echo 'source ~/.cargo/env' >> ~/.bashrc  # Bash
echo 'source ~/.cargo/env' >> ~/.zshrc   # Zsh
```

### 编译错误：linker `cc` not found

**原因**: 系统缺少 C 编译器

**解决**:

**macOS:**
```bash
xcode-select --install
```

**Ubuntu/Debian:**
```bash
sudo apt install build-essential
```

**Arch Linux:**
```bash
sudo pacman -S base-devel
```

### 运行时错误：7zz: command not found

**原因**: 没有安装 7-Zip

**解决**:
```bash
# macOS
brew install p7zip

# Ubuntu/Debian
sudo apt install p7zip-full

# Arch Linux
sudo pacman -S p7zip
```

### 运行时错误：unrar: command not found

**原因**: 没有安装 unrar

**解决**:
```bash
# macOS
brew install unrar

# Ubuntu/Debian
sudo apt install unrar

# Arch Linux
sudo pacman -S unrar
```

## 使用 Bash 版本（备用）

如果你不想安装 Rust，可以直接使用 Bash 版本：

```bash
./un7z.bash
```

Bash 版本不需要编译，但功能相对简单。

## 下一步

安装成功后，请查看：

- **QUICKSTART.md** - 快速开始指南
- **README.md** - 完整文档

开始使用：

```bash
# 交互式选择解压
un7z

# 查看所有选项
un7z --help

# 批量解压所有文件
un7z --all
```

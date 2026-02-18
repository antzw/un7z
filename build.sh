#!/bin/bash

# un7z Rust 版本编译和安装脚本
# 用法: ./build.sh [install]

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}╔══════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   un7z Rust 版本编译脚本            ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════╝${NC}"
echo ""

# 检查 Rust 是否安装
if ! command -v cargo &> /dev/null; then
    echo -e "${YELLOW}⚠ 未检测到 Rust/Cargo${NC}"
    echo ""
    echo "请先安装 Rust 工具链："
    echo ""
    echo -e "  ${GREEN}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
    echo ""
    echo "安装后需要重启终端或运行："
    echo -e "  ${GREEN}source ~/.cargo/env${NC}"
    echo ""
    exit 1
fi

echo -e "${GREEN}✓${NC} 检测到 Cargo: $(cargo --version)"
echo ""

# 编译项目
echo -e "${BLUE}→${NC} 开始编译..."
echo ""

cargo build --release

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✓${NC} 编译成功！"
    echo ""
    echo -e "二进制文件位置: ${YELLOW}target/release/un7z${NC}"
    echo ""

    # 如果参数是 install，安装到系统路径
    if [ "$1" = "install" ]; then
        echo -e "${BLUE}→${NC} 安装到 /usr/local/bin/..."

        if [ -w /usr/local/bin ]; then
            cp target/release/un7z /usr/local/bin/
            echo -e "${GREEN}✓${NC} 安装成功！"
            echo ""
            echo -e "现在可以直接运行: ${GREEN}un7z${NC}"
        else
            echo -e "${YELLOW}⚠ 需要管理员权限${NC}"
            sudo cp target/release/un7z /usr/local/bin/
            echo -e "${GREEN}✓${NC} 安装成功！"
            echo ""
            echo -e "现在可以直接运行: ${GREEN}un7z${NC}"
        fi
    else
        echo "提示: 运行 ${YELLOW}./build.sh install${NC} 可安装到系统路径"
    fi

    echo ""
    echo -e "${BLUE}══════════════════════════════════════${NC}"
    echo -e "快速测试:"
    echo -e "  ${GREEN}./target/release/un7z --help${NC}"
    echo -e "  ${GREEN}./target/release/un7z --version${NC}"
    echo -e "${BLUE}══════════════════════════════════════${NC}"
else
    echo ""
    echo -e "${RED}✗${NC} 编译失败"
    exit 1
fi

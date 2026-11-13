#!/usr/bin/env bash
# macOS 安装脚本：编译 → 装到 PATH → 注册 launchd 开机自启
#
#   ./scripts/install-macos.sh              # 自动选择网卡
#   ./scripts/install-macos.sh en1          # 指定网卡（ifconfig -l 查看，有线口常见 en1）
#   ./scripts/install-macos.sh --uninstall  # 停止并移除开机自启（保留二进制和配置）
set -euo pipefail

LABEL="local.upc.login"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${BIN_DIR:-/usr/local/bin}"
BIN="$BIN_DIR/upc-login"
CONFIG="$HOME/.config/upc-login/config.toml"
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
LOG="$HOME/Library/Logs/upc-login.log"

uninstall() {
    launchctl bootout "gui/$UID/$LABEL" 2>/dev/null || true
    rm -f "$PLIST"
    echo "已停止并移除开机自启。二进制（${BIN}）和配置（${CONFIG}）保留，需要时手动删除。"
}

if [ "${1:-}" = "--uninstall" ]; then
    uninstall
    exit 0
fi

IFACE="${1:-}"

command -v cargo >/dev/null 2>&1 || {
    echo "未找到 cargo，先装 Rust 工具链："
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo "  source \"\$HOME/.cargo/env\""
    exit 1
}

echo "==> 编译"
(cd "$ROOT" && cargo build --release)

echo "==> 安装二进制到 $BIN"
if [ -w "$BIN_DIR" ]; then
    install -m 755 "$ROOT/target/release/upc-login" "$BIN"
else
    sudo install -m 755 "$ROOT/target/release/upc-login" "$BIN"
fi

if [ ! -f "$CONFIG" ]; then
    echo "==> 生成配置文件 $CONFIG"
    UPC_CONFIG="$CONFIG" "$BIN" config --init
    echo
    echo "请先填入学号、密码和运营商（service: cmcc/unicom/ctcc/default/local），"
    echo "然后重新运行本脚本以启用开机自启。"
    exit 0
fi

if [ -n "$IFACE" ]; then
    ARGS="<string>$BIN</string><string>daemon</string><string>-i</string><string>$IFACE</string>"
else
    ARGS="<string>$BIN</string><string>daemon</string>"
fi

echo "==> 写入 $PLIST"
mkdir -p "$(dirname "$PLIST")" "$(dirname "$LOG")"
cat >"$PLIST" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>$LABEL</string>
    <key>ProgramArguments</key>
    <array>$ARGS</array>
    <key>EnvironmentVariables</key>
    <dict>
        <key>UPC_CONFIG</key>
        <string>$CONFIG</string>
    </dict>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>ThrottleInterval</key>
    <integer>30</integer>
    <key>ProcessType</key>
    <string>Background</string>
    <key>StandardOutPath</key>
    <string>$LOG</string>
    <key>StandardErrorPath</key>
    <string>$LOG</string>
</dict>
</plist>
PLIST

# 先卸载旧实例再加载，改过配置后也能直接重跑本脚本
launchctl bootout "gui/$UID/$LABEL" 2>/dev/null || true
launchctl bootstrap "gui/$UID" "$PLIST"
launchctl kickstart -k "gui/$UID/$LABEL"

echo
echo "✅ 已启动。日志: $LOG"
echo "   查看: tail -f \"$LOG\""
echo "   停止: launchctl bootout gui/$UID/$LABEL"
echo "   改配置后重启: launchctl kickstart -k gui/$UID/$LABEL"
echo
echo "首次运行时 macOS 可能弹出\"本地网络\"权限请求，请允许终端访问本地网络。"

# UPC 校园网登录工具

中国石油大学（华东）校园网命令行登录工具，使用 Rust 实现，基于锐捷 ePortal 认证协议，理论上也适用于其他使用相同系统的校园网。

![校园网登录界面](校园网登录界面.png)

## 功能

- 登录 / 登出校园网
- 配置文件保存账号信息（TOML）
- **按线路状态判断**：只有被认证网关拦截时才登录，已在线时不重复登录
- **支持指定网卡并绑定其 IPv4 作为请求源地址**（多网卡机器必需，否则会被判「认证失败」）
- 守护进程模式：自动检测网络并重连，指数退避（最长 300 秒）
- 彩色终端输出 + 进度指示
- 支持 SIGTERM/SIGINT 优雅退出

## 编译

```bash
cargo build --release
```

编译产物位于 `target/release/upc-login`。

## 配置文件

程序按以下顺序查找 `config.toml`：

1. 环境变量 `UPC_CONFIG` 指定的路径
2. **当前工作目录下的 `config.toml`**（推荐，仓库根目录已有模板）
3. 可执行文件同目录的 `config.toml`
4. `%APPDATA%\upc-login\config.toml`（Linux：`~/.config/upc-login/config.toml`）

```toml
[auth]
username = ""       # 学号 / 校园卡号
password = ""       # 校园网密码
service = "cmcc"    # cmcc 移动 | unicom 联通 | ctcc 电信 | default 校园网 | local 本地

[network]
interface = ""      # 留空自动检测；也可填网卡名，如 "WLAN"、"以太网 2"

[daemon]
check_interval = 30 # 守护进程检测间隔（秒）
max_retries = 3     # 连续失败多少次后开始指数退避
```

命令行参数优先级高于配置文件。

> ⚠️ 配置文件包含明文密码，已加入 `.gitignore`，请勿提交或分享。

## 子命令

```
upc-login login    登录校园网
upc-login logout   登出校园网
upc-login daemon   守护进程模式（自动检测并重连）
upc-login config   管理配置文件
```

## 使用

### 登录

```bash
# 命令行参数
./target/release/upc-login login -u 学号 -p 密码 -s cmcc

# 指定网卡（会绑定该网卡的 IPv4 作为请求源地址）
./target/release/upc-login login -i WLAN -u 学号 -p 密码 -s cmcc

# 使用配置文件（无需每次输入参数）
./target/release/upc-login login
```

### 登出

```bash
./target/release/upc-login logout
./target/release/upc-login logout -i WLAN
```

### 配置文件

```bash
./target/release/upc-login config --init   # 在当前目录生成模板
./target/release/upc-login config          # 查看当前生效的配置文件
```

### 守护进程

```bash
./target/release/upc-login daemon                  # 参数取自配置文件
./target/release/upc-login daemon -i WLAN -t 60 -m 5
```

守护进程定期检测线路状态，被拦截时自动登录；连续失败超过 `max_retries` 后间隔翻倍（上限 300 秒）。

## 开机自启（systemd）

```ini
[Unit]
Description = UPC campus network login daemon
After = network.target syslog.target
Wants = network.target

[Service]
Type = simple
WorkingDirectory = /path/to/upc-login
ExecStart = /path/to/upc-login daemon -i wlp3s0

[Install]
WantedBy = multi-user.target
```

将 `WorkingDirectory` 指向存放 `config.toml` 的目录，然后：

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now upc.service
sudo systemctl status upc.service
```

## 输出示例

```
本地地址: 172.24.132.4 (已绑定为请求源地址)
username: 学号
service : cmcc
🔑 登录中...
✅ 登录成功: 登录成功
```

已在线：

```
✅ 已登录，无需重复登录
```

登录失败（退出码 1）：

```
❌ 登录失败: 用户名或密码错误
```

## 常见问题

**「认证失败」**：锐捷 BRAS 会校验 `queryString` 里的 `wlanuserip` 与请求源 IP 是否一致。请用 `-i <网卡名>` 指定实际要认证的网卡，程序会绑定该网卡的 IPv4 作为源地址。

**「已登录，无需重复登录」**：程序用外网探针（`generate_204`）判断线路状态，表示当前出口已经在线，无需认证。

**网卡名怎么填**：Windows 用 `Get-NetAdapter` 查看（如「WLAN」「以太网 2」），Linux 用 `ip link`（如 `wlp3s0`、`eth0`）。

## 说明

- 自动检测本机 IP，并优先选择物理网卡的 IPv4（跳过 `127.` 与 `169.254.` 开头的地址）
- HTTP 请求由内置的极简 HTTP/1.1 客户端完成，可精确绑定源地址（规避 reqwest 在 Windows 上 `local_address` 报 `os error 10049` 的问题）
- 如有问题欢迎联系：`tianyuyang666@gmail.com`

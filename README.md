# UPC 校园网登录工具

中国石油大学（华东）校园网命令行登录工具，使用 Rust 实现。基于锐捷 ePortal 认证协议，理论上也适用于其他使用相同系统的校园网。

![校园网登录界面](校园网登录界面.png)

## 功能

- 登录/登出校园网
- 配置文件保存账号信息（TOML）
- 守护进程模式：自动检测网络并重连
- 指数退避重连策略
- 彩色终端输出 + 进度指示
- 支持 SIGTERM/SIGINT 优雅退出

## 依赖

- Rust 1.70+

## 编译

```bash
cargo build --release
```

编译产物位于 `target/release/upc-login`。

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

# 指定网卡
./target/release/upc-login login -i wlp3s0 -u 学号 -p 密码 -s cmcc

# 使用配置文件（无需每次输入参数）
./target/release/upc-login login
```

### 登出

```bash
./target/release/upc-login logout

# 指定网卡
./target/release/upc-login logout -i wlp3s0
```

### 配置文件

```bash
# 生成模板配置文件
./target/release/upc-login config --init

# 查看当前配置
./target/release/upc-login config
```

配置文件路径：`~/.config/upc-login/config.toml`

```toml
[auth]
username = ""
password = ""
service = "cmcc"          # cmcc | unicom | ctcc | default | local

[network]
interface = ""             # 留空则自动检测

[daemon]
check_interval = 30        # 检测间隔（秒）
max_retries = 3            # 连续失败最大重试次数
```

命令行参数优先级高于配置文件。

### 守护进程

```bash
# 使用配置文件中的参数
./target/release/upc-login daemon

# 指定网卡和检测间隔
./target/release/upc-login daemon -i eth0 -t 60 -m 5
```

守护进程会定期检测网络状态，断网时自动重连。连续失败后会使用指数退避策略（间隔翻倍，最长300秒）。

## 开机自启（systemd）

```bash
sudo vim /etc/systemd/system/upc.service
```

```ini
[Unit]
Description = UPC campus network login daemon
After = network.target syslog.target
Wants = network.target

[Service]
Type = simple
ExecStart = /path/to/upc-login daemon -i eth0

[Install]
WantedBy = multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl start upc.service
sudo systemctl enable upc.service   # 开机自启
sudo systemctl status upc.service   # 查看状态
```

## 输出示例

登录成功：

```
username: 学号
service : cmcc
🔑 登录中...
✅ 登录成功: 登录成功
```

已登录：

```
✅ 已登录，无需重复登录
```

登录失败：

```
❌ 登录失败: 用户不存在或者密码错误!
```

## 说明

- 自动检测本机 IP，智能选择 LAN/WLAN 认证服务器
- 守护进程模式需要在配置文件中设置账号信息。
- 如有问题欢迎联系：`tianyuyang666@gmail.com`

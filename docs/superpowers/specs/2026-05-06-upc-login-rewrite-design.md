# UPC 校园网登录工具 - Rust 重写设计

## 概述

重写中国石油大学（华东）校园网登录工具，基于锐捷 ePortal 认证协议，增加配置文件、自动重连、守护进程、终端美化功能。

## 项目结构

```
src/
├── main.rs          # 入口 + clap 子命令定义
├── config.rs        # 配置文件读写 (TOML)
├── network.rs       # 网络检测、IP获取、URL判断
├── portal.rs        # ePortal 协议：登录/登出/queryString获取
├── daemon.rs        # 守护进程：循环检测+自动重连
├── transcode.rs     # URL双编码转义
└── output.rs        # 彩色输出美化
```

## 子命令设计

| 命令 | 说明 | 示例 |
|------|------|------|
| `upc-login login` | 登录 | `upc-login login -u 学号 -p 密码 -s cmcc` |
| `upc-login logout` | 登出 | `upc-login logout` |
| `upc-login daemon` | 守护进程模式 | `upc-login daemon -i eth0` |
| `upc-login config` | 生成/查看配置文件 | `upc-login config --init` |

命令行参数覆盖配置文件中的同名项。

## 配置文件

路径：`~/.config/upc-login/config.toml`

```toml
[auth]
username = ""
password = ""
service = "cmcc"

[network]
interface = ""

[daemon]
check_interval = 30
max_retries = 3
```

- `upc-login config --init`：生成模板配置文件
- `upc-login config`：显示当前配置

## 守护进程模式

- 定期（默认30秒）访问 `lan.upc.edu.cn` 检测网络连通性
- 检测到需要登录时（收到重定向到 ePortal），自动执行登录
- 连续失败超过 `max_retries` 次后，休眠间隔加倍（指数退避），最长300秒
- 支持 `SIGTERM`/`SIGINT` 优雅退出
- 日志输出当前状态（时间戳 + 事件描述）

## 终端美化

- `colored` crate：成功绿色、失败红色、警告黄色
- `indicatif` crate：登录时显示 spinner
- 状态提示使用 emoji 前缀：🔑 登录中 / ✅ 登录成功 / ❌ 登录失败

## 核心协议

保持现有 ePortal 协议逻辑：

1. 访问 `lan.upc.edu.cn` 或 `wlan.upc.edu.cn`，获取重定向 Location
2. 从 Location 中提取 queryString
3. 对 queryString 进行双编码转义（transcode）
4. POST 到 `/eportal/InterFace.do?method=login` 进行登录
5. 解析 JSON 响应，输出结果

URL选择：
- IP 以 `10.` 开头 → LAN (`lan.upc.edu.cn`)
- 其他 → WLAN (`wlan.upc.edu.cn`)

## 依赖

```toml
reqwest = { version = "0.12", features = ["blocking"] }
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"
anyhow = "1"
colored = "2"
indicatif = "0.17"
local-ip-address = "0.6"
dirs = "5"
signal-hook = "0.3"
```

## 错误处理

- 使用 `anyhow` 统一错误处理
- 网络超时、DNS解析失败等给出友好提示
- 守护进程模式下错误不退出，记录日志后继续

## 兼容性

- 保持与现有版本相同的协议行为
- 支持 systemd 服务部署（守护进程模式更适合）
- 配置文件为可选，不配置时仍可纯命令行使用

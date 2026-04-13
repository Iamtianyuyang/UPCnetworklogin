# UPC 校园网登录工具

中国石油大学（华东）校园网命令行登录工具，支持 C 和 Python 两个版本。基于锐捷 ePortal 认证协议，理论上也适用于其他使用相同系统的校园网。

![校园网登录界面](校园网登录界面.png)

## 目录结构

```
.
├── c/              # C 语言版本
│   ├── main.c
│   └── Makefile
├── python/         # Python 版本
│   ├── main.py
│   ├── function.py
│   ├── get_opt.py
│   └── my_info.py
└── README.md
```

## 参数说明

两个版本使用相同的命令行参数：

```
-h, --help          显示帮助信息
-V, --version       显示版本号
-u, --username      校园卡学号
-p, --password      校园卡密码
-s, --service       运营商: cmcc | unicom | ctcc | default | local
```

---

## C 版本

### 依赖

- `gcc`
- `libcurl`

### 编译

```bash
cd c
make
```

### 使用

```bash
./login -u 学号 -p 密码 -s cmcc
```

### 开机自启（systemd）

```bash
sudo vim /etc/systemd/system/upc.service
```

```ini
[Unit]
Description = UPC campus network login
After = network.target syslog.target
Wants = network.target

[Service]
Type = simple
ExecStart = /path/to/login -u 学号 -p 密码 -s cmcc

[Install]
WantedBy = multi-user.target
```

```bash
sudo systemctl daemon-reload
sudo systemctl start upc.service
sudo systemctl enable upc.service   # 开机自启
sudo systemctl status upc.service   # 查看状态
```

---

## Python 版本

### 依赖

- Python 3.10+
- `pycurl`

```bash
pip install pycurl
```

### 使用

```bash
cd python
python main.py -u 学号 -p 密码 -s cmcc
```

---

## 输出示例

登录成功：

```
username : 学号
service  : cmcc
logging in...
NOTICE:
    success
    登录成功
```

登录失败（密码错误）：

```
NOTICE:
    fail
    用户不存在或者密码错误!
```

登录失败（运营商不匹配）：

```
NOTICE:
    fail
    您未绑定服务对应的运营商!
```

---

## 说明

- 默认连接有线校园网（`lan.upc.edu.cn`）。如需无线校园网认证，将代码中的 `lan.upc.edu.cn` 改为 `wlan.upc.edu.cn`。
- 如需定时检测网络状态，可配合 `ping` 命令使用：
  ```bash
  ping -c 1 www.baidu.com || ./login -u 学号 -p 密码 -s cmcc
  ```
- 如有问题欢迎联系：`tianyuyang666@gmail.com`

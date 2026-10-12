use anyhow::{bail, Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

const CONFIG_TEMPLATE: &str = r#"# upc-login 配置文件
# 本文件包含明文密码，请勿提交到 git 或分享给别人。

[auth]
username = ""       # 学号 / 校园卡号
password = ""       # 校园网密码
service = "cmcc"    # 运营商: cmcc 移动 | unicom 联通 | ctcc 电信 | default 校园网 | local 本地

[network]
interface = ""      # 留空自动检测；也可填网卡名，如 "WLAN"、"以太网 2"

[daemon]
check_interval = 30 # 守护进程检测间隔（秒）
max_retries = 3     # 连续失败多少次后开始指数退避
"#;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub network: NetworkConfig,
    #[serde(default)]
    pub daemon: DaemonConfig,
}

#[derive(Debug, Deserialize, Default)]
pub struct AuthConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub service: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct NetworkConfig {
    pub interface: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DaemonConfig {
    pub check_interval: Option<u64>,
    pub max_retries: Option<u32>,
}

fn user_config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("upc-login").join("config.toml")
}

/// 配置文件查找顺序：
/// 1. 环境变量 UPC_CONFIG
/// 2. 当前工作目录下的 config.toml（项目/程序目录，推荐）
/// 3. 可执行文件同目录的 config.toml
/// 4. 用户配置目录 %APPDATA%\upc-login\config.toml（Linux: ~/.config/upc-login/）
pub fn config_path() -> PathBuf {
    if let Ok(p) = std::env::var("UPC_CONFIG") {
        if !p.trim().is_empty() {
            return PathBuf::from(p);
        }
    }

    let local = PathBuf::from("config.toml");
    if local.exists() {
        return local;
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let beside_exe = dir.join("config.toml");
            if beside_exe.exists() {
                return beside_exe;
            }
        }
    }

    user_config_path()
}

/// 读取一个值，空字符串视为未填写
fn non_empty(value: Option<String>) -> Option<String> {
    value.filter(|s| !s.trim().is_empty())
}

pub fn load() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("读取配置文件失败: {}", path.display()))?;

    let mut config: Config = toml::from_str(&content)
        .with_context(|| format!("解析配置文件失败: {}", path.display()))?;

    config.auth.username = non_empty(config.auth.username);
    config.auth.password = non_empty(config.auth.password);
    config.auth.service = non_empty(config.auth.service);
    config.network.interface = non_empty(config.network.interface);

    Ok(config)
}

pub fn init_config() -> Result<()> {
    let path = match std::env::current_dir() {
        Ok(dir) => dir.join("config.toml"),
        Err(_) => user_config_path(),
    };

    if path.exists() {
        bail!("配置文件已存在: {}", path.display());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, CONFIG_TEMPLATE)?;

    println!("{} 配置文件已生成: {}", "✅".green(), path.display());
    println!("请编辑该文件填入学号、密码与运营商");

    Ok(())
}

pub fn show_config() -> Result<()> {
    let path = config_path();

    if !path.exists() {
        println!("配置文件不存在: {}", path.display());
        println!("运行 `upc-login config --init` 生成配置文件");
        return Ok(());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("读取配置文件失败: {}", path.display()))?;

    println!("配置文件路径: {}", path.display());
    println!("---");
    print!("{}", content);

    Ok(())
}

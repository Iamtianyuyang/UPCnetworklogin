use anyhow::{bail, Context, Result};
use colored::Colorize;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

const CONFIG_TEMPLATE: &str = r#"[auth]
username = ""
password = ""
service = "cmcc"

[network]
interface = ""

[daemon]
check_interval = 30
max_retries = 3
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

fn config_path() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("upc-login").join("config.toml")
}

pub fn load() -> Result<Config> {
    let path = config_path();
    if !path.exists() {
        return Ok(Config::default());
    }

    let content = fs::read_to_string(&path)
        .with_context(|| format!("读取配置文件失败: {}", path.display()))?;

    let config: Config = toml::from_str(&content)
        .with_context(|| format!("解析配置文件失败: {}", path.display()))?;

    Ok(config)
}

pub fn init_config() -> Result<()> {
    let path = config_path();

    if path.exists() {
        bail!("配置文件已存在: {}", path.display());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&path, CONFIG_TEMPLATE)?;

    println!(
        "{} 配置文件已生成: {}",
        "✅".green(),
        path.display()
    );
    println!("请编辑配置文件填入账号信息");

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

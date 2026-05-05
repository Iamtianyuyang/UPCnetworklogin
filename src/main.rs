use anyhow::Result;
use clap::{Parser, Subcommand};

mod config;
mod daemon;
mod network;
mod output;
mod portal;
mod transcode;

const VERSION: &str = "2.0.0";

#[derive(Parser)]
#[command(name = "upc-login", version = VERSION, about = "UPC 校园网登录工具")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Login {
        #[arg(short, long)]
        username: Option<String>,

        #[arg(short, long)]
        password: Option<String>,

        #[arg(short, long)]
        service: Option<String>,

        #[arg(short = 'i', long)]
        interface: Option<String>,
    },

    Logout {
        #[arg(short = 'i', long)]
        interface: Option<String>,
    },

    Daemon {
        #[arg(short = 'i', long)]
        interface: Option<String>,

        #[arg(short = 't', long, default_value = "30")]
        interval: u64,

        #[arg(short = 'm', long, default_value = "3")]
        max_retries: u32,
    },

    Config {
        #[arg(long)]
        init: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login {
            username,
            password,
            service,
            interface,
        } => {
            let cfg = config::load()?;
            let username = username
                .or(cfg.auth.username)
                .ok_or_else(|| anyhow::anyhow!("请输入学号 (-u) 或在配置文件中设置"))?;
            let password = password
                .or(cfg.auth.password)
                .ok_or_else(|| anyhow::anyhow!("请输入密码 (-p) 或在配置文件中设置"))?;
            let service = service
                .or(cfg.auth.service)
                .ok_or_else(|| anyhow::anyhow!("请输入运营商 (-s) 或在配置文件中设置"))?;
            let interface = interface.or(cfg.network.interface);

            let base_url = network::resolve_base_url(interface.as_deref())?;
            let client = portal::build_client()?;

            output::print_login_header(&username, &service);

            let info = match portal::get_portal_info(&client, base_url) {
                Ok(info) => info,
                Err(e) => {
                    if e.to_string().contains("already logged in") {
                        crate::output::print_already_logged_in();
                        return Ok(());
                    }
                    return Err(e);
                }
            };
            let encoded = transcode::transcode(&info.query_string);

            let pb = output::print_logging_in();
            portal::login(
                &client,
                &info.base_url,
                &encoded,
                &username,
                &password,
                &service,
            )?;
            pb.finish_and_clear();
        }
        Commands::Logout { interface } => {
            let cfg = config::load()?;
            let interface = interface.or(cfg.network.interface);

            let base_url = network::resolve_base_url(interface.as_deref())?;
            let client = portal::build_client()?;

            let actual_base = match portal::get_portal_info(&client, base_url) {
                Ok(info) => info.base_url,
                Err(e) => {
                    if e.to_string().contains("already logged in") {
                        base_url.to_string()
                    } else {
                        return Err(e);
                    }
                }
            };

            output::print_logout();
            portal::logout(&client, &actual_base)?;
        }
        Commands::Daemon {
            interface,
            interval,
            max_retries,
        } => {
            let cfg = config::load()?;
            let interface = interface.or(cfg.network.interface);
            let interval = if interval == 30 {
                cfg.daemon.check_interval.unwrap_or(30)
            } else {
                interval
            };
            let max_retries = if max_retries == 3 {
                cfg.daemon.max_retries.unwrap_or(3)
            } else {
                max_retries
            };

            let base_url = network::resolve_base_url(interface.as_deref())?;
            let username = cfg
                .auth
                .username
                .ok_or_else(|| anyhow::anyhow!("守护进程模式需要在配置文件中设置学号"))?;
            let password = cfg
                .auth
                .password
                .ok_or_else(|| anyhow::anyhow!("守护进程模式需要在配置文件中设置密码"))?;
            let service = cfg
                .auth
                .service
                .ok_or_else(|| anyhow::anyhow!("守护进程模式需要在配置文件中设置运营商"))?;

            daemon::run(
                base_url,
                &username,
                &password,
                &service,
                interval,
                max_retries,
            )?;
        }
        Commands::Config { init } => {
            if init {
                config::init_config()?;
            } else {
                config::show_config()?;
            }
        }
    }

    Ok(())
}

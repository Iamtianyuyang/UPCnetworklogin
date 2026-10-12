use anyhow::Result;
use clap::{Parser, Subcommand};

mod config;
mod daemon;
mod network;
mod output;
mod portal;
mod transcode;

const VERSION: &str = "2.1.0";

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

        #[arg(short = 't', long)]
        interval: Option<u64>,

        #[arg(short = 'm', long)]
        max_retries: Option<u32>,
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
                .ok_or_else(|| anyhow::anyhow!("请输入学号 (-u) 或在 config.toml 中设置"))?;
            let password = password
                .or(cfg.auth.password)
                .ok_or_else(|| anyhow::anyhow!("请输入密码 (-p) 或在 config.toml 中设置"))?;
            let service = service
                .or(cfg.auth.service)
                .ok_or_else(|| anyhow::anyhow!("请输入运营商 (-s) 或在 config.toml 中设置"))?;
            let interface = interface
                .or(cfg.network.interface)
                .filter(|s| !s.trim().is_empty());

            let local_ip = network::resolve_local_ip(interface.as_deref())?;
            let session = portal::Session::new(local_ip);

            output::print_login_header(&username, &service);

            let info = match portal::discover(&session)? {
                Some(info) => info,
                None => {
                    output::print_already_logged_in();
                    return Ok(());
                }
            };
            let encoded = transcode::transcode(&info.query_string);

            let pb = output::print_logging_in();
            let ok = portal::login(
                &session,
                &info.base_url,
                &encoded,
                &username,
                &password,
                &service,
            )?;
            pb.finish_and_clear();

            if !ok {
                std::process::exit(1);
            }
        }
        Commands::Logout { interface } => {
            let cfg = config::load()?;
            let interface = interface
                .or(cfg.network.interface)
                .filter(|s| !s.trim().is_empty());

            let local_ip = network::resolve_local_ip(interface.as_deref())?;
            let session = portal::Session::new(local_ip);

            let base_url = match portal::discover(&session)? {
                Some(info) => info.base_url,
                None => portal::DEFAULT_PORTAL.to_string(),
            };

            output::print_logout();
            if !portal::logout(&session, &base_url)? {
                std::process::exit(1);
            }
        }
        Commands::Daemon {
            interface,
            interval,
            max_retries,
        } => {
            let cfg = config::load()?;
            let interface = interface
                .or(cfg.network.interface)
                .filter(|s| !s.trim().is_empty());
            let interval = interval.or(cfg.daemon.check_interval).unwrap_or(30);
            let max_retries = max_retries.or(cfg.daemon.max_retries).unwrap_or(3);

            let _ = network::resolve_local_ip(interface.as_deref())?;

            let username = cfg
                .auth
                .username
                .ok_or_else(|| anyhow::anyhow!("守护进程模式需要在 config.toml 中设置学号"))?;
            let password = cfg
                .auth
                .password
                .ok_or_else(|| anyhow::anyhow!("守护进程模式需要在 config.toml 中设置密码"))?;
            let service = cfg
                .auth
                .service
                .ok_or_else(|| anyhow::anyhow!("守护进程模式需要在 config.toml 中设置运营商"))?;

            daemon::run(&username, &password, &service, interface, interval, max_retries)?;
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

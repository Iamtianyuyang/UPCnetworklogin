use anyhow::Result;

use std::net::IpAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::network;
use crate::output;
use crate::portal;

pub fn run(
    username: &str,
    password: &str,
    service: &str,
    interface: Option<String>,
    interval: u64,
    max_retries: u32,
) -> Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc_handler(r);

    output::print_daemon_status("守护进程已启动");
    output::print_daemon_status(&format!(
        "检测间隔: {}s, 最大重试: {}次",
        interval, max_retries
    ));

    let mut consecutive_failures: u32 = 0;
    let mut current_interval = interval;
    let mut last_local_ip: Option<IpAddr> = None;

    while running.load(Ordering::SeqCst) {
        // 每轮都重新解析本地地址：无线重连后 IP 可能变化，绑着旧地址会永远连不上
        let local_ip = match network::local_ip(interface.as_deref()) {
            Ok(ip) => ip,
            Err(e) => {
                consecutive_failures += 1;
                output::print_daemon_error(&format!("获取本地地址失败: {}", e));
                apply_backoff(
                    &mut consecutive_failures,
                    &mut current_interval,
                    max_retries,
                    interval,
                );
                sleep_with_interrupt(running.clone(), Duration::from_secs(current_interval));
                continue;
            }
        };

        if local_ip != last_local_ip {
            match local_ip {
                Some(ip) => output::print_daemon_status(&format!("当前本地地址: {}", ip)),
                None => output::print_daemon_status("当前本地地址: 系统默认出口"),
            }
            last_local_ip = local_ip;
        }

        let session = portal::Session::new(local_ip);

        match portal::discover(&session) {
            Ok(Some(info)) => {
                output::print_daemon_status("检测到需要登录，正在自动登录...");

                let encoded = crate::transcode::transcode(&info.query_string);

                match portal::login(
                    &session,
                    &info.base_url,
                    &encoded,
                    username,
                    password,
                    service,
                ) {
                    Ok(true) => {
                        consecutive_failures = 0;
                        current_interval = interval;
                    }
                    Ok(false) => {
                        consecutive_failures += 1;
                        output::print_daemon_error("登录被服务端拒绝");
                        apply_backoff(
                            &mut consecutive_failures,
                            &mut current_interval,
                            max_retries,
                            interval,
                        );
                    }
                    Err(e) => {
                        consecutive_failures += 1;
                        output::print_daemon_error(&format!("登录失败: {}", e));
                        apply_backoff(
                            &mut consecutive_failures,
                            &mut current_interval,
                            max_retries,
                            interval,
                        );
                    }
                }
            }
            Ok(None) => {
                if consecutive_failures > 0 {
                    output::print_daemon_status("网络已恢复");
                }
                consecutive_failures = 0;
                current_interval = interval;
            }
            Err(e) => {
                consecutive_failures += 1;
                output::print_daemon_error(&format!("网络检测失败: {}", e));
                apply_backoff(
                    &mut consecutive_failures,
                    &mut current_interval,
                    max_retries,
                    interval,
                );
            }
        }

        sleep_with_interrupt(running.clone(), Duration::from_secs(current_interval));
    }

    output::print_daemon_status("守护进程已停止");
    Ok(())
}

fn apply_backoff(failures: &mut u32, current_interval: &mut u64, max_retries: u32, base_interval: u64) {
    if *failures >= max_retries {
        let backoff = (*current_interval * 2).min(300).max(base_interval);
        output::print_daemon_error(&format!("连续失败 {} 次，退避至 {}s", failures, backoff));
        *current_interval = backoff;
    }
}

fn ctrlc_handler(running: Arc<AtomicBool>) {
    // Unix 下响应 SIGTERM；Windows 上无效（仅 Ctrl+C 生效）
    let _ = signal_hook::flag::register(signal_hook::consts::SIGTERM, running.clone());
    let r2 = running.clone();
    ctrlc::set_handler(move || {
        r2.store(false, Ordering::SeqCst);
    })
    .ok();
}

fn sleep_with_interrupt(running: Arc<AtomicBool>, duration: Duration) {
    let start = std::time::Instant::now();
    while running.load(Ordering::SeqCst) && start.elapsed() < duration {
        thread::sleep(Duration::from_millis(500));
    }
}

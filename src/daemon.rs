use anyhow::Result;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::output;
use crate::portal;

pub fn run(
    base_url: &str,
    username: &str,
    password: &str,
    service: &str,
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

    while running.load(Ordering::SeqCst) {
        let client = match portal::build_client() {
            Ok(c) => c,
            Err(e) => {
                output::print_daemon_error(&format!("创建HTTP客户端失败: {}", e));
                sleep_with_interrupt(running.clone(), Duration::from_secs(current_interval));
                continue;
            }
        };

        match portal::check_needs_login(&client, base_url) {
            Ok(true) => {
                output::print_daemon_status("检测到需要登录，正在自动登录...");

                let info = match portal::get_portal_info(&client, base_url) {
                    Ok(i) => i,
                    Err(e) => {
                        output::print_daemon_error(&format!("获取登录参数失败: {}", e));
                        consecutive_failures += 1;
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

                let encoded = crate::transcode::transcode(&info.query_string);

                match portal::login(
                    &client,
                    &info.base_url,
                    &encoded,
                    username,
                    password,
                    service,
                ) {
                    Ok(_) => {
                        consecutive_failures = 0;
                        current_interval = interval;
                    }
                    Err(e) => {
                        output::print_daemon_error(&format!("登录失败: {}", e));
                        consecutive_failures += 1;
                        apply_backoff(
                            &mut consecutive_failures,
                            &mut current_interval,
                            max_retries,
                            interval,
                        );
                    }
                }
            }
            Ok(false) => {
                if consecutive_failures > 0 {
                    output::print_daemon_status("网络已恢复");
                }
                consecutive_failures = 0;
                current_interval = interval;
            }
            Err(e) => {
                output::print_daemon_error(&format!("网络检测失败: {}", e));
                consecutive_failures += 1;
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

fn apply_backoff(failures: &mut u32, current_interval: &mut u64, max_retries: u32, _base_interval: u64) {
    if *failures >= max_retries {
        let backoff = (*current_interval * 2).min(300);
        output::print_daemon_error(&format!(
            "连续失败 {} 次，退避至 {}s",
            failures, backoff
        ));
        *current_interval = backoff;
    }
}

fn ctrlc_handler(running: Arc<AtomicBool>) {
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

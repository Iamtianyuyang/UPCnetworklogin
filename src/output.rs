use colored::Colorize;
use indicatif::ProgressBar;
use std::time::Duration;

pub fn print_login_header(username: &str, service: &str) {
    println!("{} {}", "username:".bright_blue(), username);
    println!("{} {}", "service :".bright_blue(), service);
}

pub fn print_logging_in() -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        indicatif::ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message("🔑 登录中...");
    pb.enable_steady_tick(Duration::from_millis(120));
    pb
}

pub fn print_login_success(message: &str) {
    let text = if message.trim().is_empty() {
        "登录成功".to_string()
    } else {
        format!("登录成功: {}", message)
    };
    println!("{} {}", "✅".green(), text.green());
}

pub fn print_login_failure(message: &str) {
    eprintln!("{} {}", "❌".red(), format!("登录失败: {}", message).red());
}

pub fn print_already_logged_in() {
    println!("{} {}", "✅".green(), "已登录，无需重复登录".green());
}

pub fn print_logout() {
    println!("{}", "🔌 正在登出...".yellow());
}

pub fn print_logout_success(message: &str) {
    let text = if message.trim().is_empty() {
        "登出成功".to_string()
    } else {
        format!("登出成功: {}", message)
    };
    println!("{} {}", "✅".green(), text.green());
}

pub fn print_logout_failure(message: &str) {
    eprintln!("{} {}", "❌".red(), format!("登出失败: {}", message).red());
}

pub fn print_logout_not_needed(message: &str) {
    println!(
        "{} {}",
        "ℹ️".bright_blue(),
        format!("无需登出: {}", message).bright_blue()
    );
}

pub fn print_daemon_status(msg: &str) {
    let now = chrono_free_timestamp();
    println!("[{}] {}", now, msg);
}

pub fn print_daemon_error(msg: &str) {
    let now = chrono_free_timestamp();
    eprintln!("[{}] {} {}", now, "⚠️".yellow(), msg.yellow());
}

fn chrono_free_timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // 守护进程日志用本机时区（macOS/Linux 的 localtime），而不是写死的 UTC+8
    #[cfg(unix)]
    if let Some(local) = local_hms(secs) {
        return local;
    }

    // 兵底：没有 localtime 的平台按 UTC+8 换算
    let h = ((secs / 3600) + 8) % 24;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

#[cfg(unix)]
fn local_hms(secs: u64) -> Option<String> {
    let time = secs as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    if unsafe { libc::localtime_r(&time, &mut tm) }.is_null() {
        return None;
    }
    Some(format!(
        "{:02}:{:02}:{:02}",
        tm.tm_hour, tm.tm_min, tm.tm_sec
    ))
}

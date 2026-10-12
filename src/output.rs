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
    eprintln!(
        "{} {}",
        "❌".red(),
        format!("登录失败: {}", message).red()
    );
}

pub fn print_already_logged_in() {
    println!(
        "{} {}",
        "✅".green(),
        "已登录，无需重复登录".green()
    );
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
    eprintln!(
        "{} {}",
        "❌".red(),
        format!("登出失败: {}", message).red()
    );
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
    let now = std::time::SystemTime::now();
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let h = ((secs / 3600) + 8) % 24;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

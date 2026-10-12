use anyhow::{bail, Result};
use colored::Colorize;
use local_ip_address::list_afinet_netifas;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};

fn is_usable_v4(v4: &Ipv4Addr) -> bool {
    !v4.is_loopback() && !v4.is_link_local() && !v4.is_unspecified()
}

/// 取指定网卡的地址，优先返回 IPv4（Windows 上枚举顺序可能先给出 IPv6 / link-local）
pub fn get_interface_ip(name: &str) -> Result<IpAddr> {
    let interfaces = list_afinet_netifas()?;
    let mut fallback_v6: Option<IpAddr> = None;
    for (iface, ip) in &interfaces {
        if iface != name {
            continue;
        }
        match ip {
            IpAddr::V4(v4) => return Ok(IpAddr::V4(*v4)),
            IpAddr::V6(_) => {
                if fallback_v6.is_none() {
                    fallback_v6 = Some(*ip);
                }
            }
        }
    }
    match fallback_v6 {
        Some(ip) => Ok(ip),
        None => bail!("未找到网卡: {}", name),
    }
}

/// 自动挑选可用 IPv4：跳过 127. 与 169.254.，网卡名看起来像物理网卡时优先。
/// Windows 的接口友好名是 "WLAN" / "以太网"，因此比较时统一转小写。
pub fn find_local_ip() -> Option<IpAddr> {
    let interfaces = list_afinet_netifas().ok()?;
    let mut best: Option<IpAddr> = None;
    for (iface, ip) in &interfaces {
        if let IpAddr::V4(v4) = ip {
            if !is_usable_v4(v4) {
                continue;
            }
            if best.is_none() {
                best = Some(IpAddr::V4(*v4));
            }
            let name = iface.to_lowercase();
            if name.starts_with("eth")
                || name.starts_with("en")
                || name.starts_with("wlan")
                || name.starts_with("wlp")
            {
                return Some(IpAddr::V4(*v4));
            }
        }
    }
    best
}

/// 系统默认出口对应的本地地址（UDP connect 不实际发包，只做一次路由查询）
pub fn default_route_ip() -> Option<IpAddr> {
    let sock = UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect("223.5.5.5:80").ok()?;
    sock.local_addr().ok().map(|addr| addr.ip())
}

/// 解析本次要使用的本地地址，并在"指定网卡 != 系统出口"时给出可执行的提示。
///
/// 说明：锐捷 BRAS 会校验 queryString 里的 wlanuserip 与请求源 IP 是否一致，
/// 而 Windows 上无法为单个进程可靠地指定出口（reqwest 绑定源地址会报 os error 10049），
/// 因此这里不改路由，只做提示，由系统路由决定出口。
/// 纯逻辑：解析要使用的本地地址（不做任何打印，供守护进程每轮调用）
pub fn local_ip(interface: Option<&str>) -> Result<Option<IpAddr>> {
    match interface {
        Some(name) => {
            let ip = get_interface_ip(name)?;
            // 只有 169.254.x（或 fe80::）说明该网卡没有拿到有效地址：网线没插/无线没连上
            if let IpAddr::V4(v4) = ip {
                if v4.is_link_local() {
                    bail!(
                        "网卡 '{}' 未连接，只有 link-local 地址 {} —— 请确认已插网线或已连上无线，或去掉 -i 让程序自动选择",
                        name,
                        ip
                    );
                }
            }
            Ok(Some(ip))
        }
        None => Ok(find_local_ip()),
    }
}

/// 解析本地地址并打印诊断信息（命令行一次性使用）
pub fn resolve_local_ip(interface: Option<&str>) -> Result<Option<IpAddr>> {
    let ip = local_ip(interface)?;

    match ip {
        Some(local) => {
            println!("本地地址: {} {}", local, "(已绑定为请求源地址)".dimmed());
            if local.is_ipv4() {
                if let Some(out) = default_route_ip() {
                    if out != local {
                        println!(
                            "{} 系统默认出口是 {}，本程序按绑定地址从 {} 发出请求",
                            "ℹ️".bright_blue(),
                            out,
                            local
                        );
                    }
                }
            }
        }
        None => {
            println!("{} 未找到可用 IPv4，将使用系统默认出口", "⚠️".yellow());
        }
    }

    Ok(ip)
}

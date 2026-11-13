use anyhow::{bail, Result};
use colored::Colorize;
use local_ip_address::list_afinet_netifas;
use std::net::{IpAddr, Ipv4Addr, UdpSocket};

fn is_usable_v4(v4: &Ipv4Addr) -> bool {
    !v4.is_loopback() && !v4.is_link_local() && !v4.is_unspecified()
}

/// 虚拟/隧道网卡：macOS 的 VPN(utun*)、AirDrop 与接力(awdl*, llw*)、
/// 虚拟机(bridge*, vmnet*)、隧道(gif*, stf*)、网络共享(anpi*)、iOS USB(ipsec*)，
/// Linux 的 docker*, tun*, tap*。这些地址不能用来给校园网认证。
fn is_virtual(name: &str) -> bool {
    const PREFIXES: [&str; 13] = [
        "utun", "awdl", "llw", "bridge", "vmnet", "gif", "stf", "anpi", "ipsec", "lo", "docker",
        "tap", "tun",
    ];
    let name = name.to_lowercase();
    PREFIXES.iter().any(|p| name.starts_with(p))
}

/// 名字看起来像物理网卡：macOS 是 en0/en1，Linux 是 eth0/enp3s0/wlp2s0，
/// Windows 是 "WLAN"/"以太网 2"/"Ethernet"。比较前统一转小写。
fn looks_physical(name: &str) -> bool {
    const PREFIXES: [&str; 7] = ["en", "eth", "wl", "以太", "ethernet", "wi-fi", "wifi"];
    let name = name.to_lowercase();
    PREFIXES.iter().any(|p| name.starts_with(p))
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

/// 反查某个地址属于哪块网卡（仅用于输出提示）
pub fn interface_name(ip: IpAddr) -> Option<String> {
    list_afinet_netifas()
        .ok()?
        .into_iter()
        .find(|(_, addr)| *addr == ip)
        .map(|(name, _)| name)
}

/// 自动挑选可用 IPv4：
/// 1. 优先跟随系统默认路由（UDP connect 只查路由表）—— Mac 上的 en0/en1、
///    Windows 上的"以太网 2"都靠这个认出来，比按名字猜准；
/// 2. 默认出口挂在虚拟网卡上（开着 VPN）时，退而挑名字像物理网卡的第一个地址；
/// 3. 都没有就拿第一个非虚拟、非 127./169.254. 的地址。
pub fn find_local_ip() -> Option<IpAddr> {
    let interfaces = list_afinet_netifas().ok()?;

    if let Some(ip) = default_route_ip() {
        if let IpAddr::V4(v4) = ip {
            let on_virtual = interfaces
                .iter()
                .find(|(_, addr)| *addr == ip)
                .map(|(name, _)| is_virtual(name))
                .unwrap_or(false);
            if is_usable_v4(&v4) && !on_virtual {
                return Some(ip);
            }
        }
    }

    let mut best_physical: Option<IpAddr> = None;
    let mut best_any: Option<IpAddr> = None;
    for (name, ip) in &interfaces {
        let IpAddr::V4(v4) = ip else { continue };
        if !is_usable_v4(v4) {
            continue;
        }
        if best_any.is_none() && !is_virtual(name) {
            best_any = Some(*ip);
        }
        if best_physical.is_none() && looks_physical(name) {
            best_physical = Some(*ip);
        }
    }
    best_physical.or(best_any)
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
/// 因此这里不改路由，只绑定源地址，由系统路由决定出口。
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
            let iface = interface_name(local)
                .map(|name| format!(" [{}]", name))
                .unwrap_or_default();
            println!(
                "本地地址: {}{} {}",
                local,
                iface,
                "(已绑定为请求源地址)".dimmed()
            );
            if local.is_ipv4() {
                if let Some(out) = default_route_ip() {
                    if out != local {
                        let out_iface = interface_name(out)
                            .map(|name| format!(" [{}]", name))
                            .unwrap_or_default();
                        println!(
                            "{} 系统默认出口是 {}{}，本程序按绑定地址从 {} 发出请求",
                            "ℹ️".bright_blue(),
                            out,
                            out_iface,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_interfaces_are_recognised() {
        for name in [
            "utun4",
            "awdl0",
            "llw0",
            "bridge100",
            "vmnet1",
            "gif0",
            "stf0",
            "anpi1",
            "utun2",
        ] {
            assert!(is_virtual(name), "{} 应被判为虚拟网卡", name);
        }
        assert!(!is_virtual("en0"));
        assert!(!is_virtual("以太网 2"));
    }

    #[test]
    fn physical_names_are_recognised() {
        for name in [
            "en0",
            "en1",
            "eth0",
            "enp3s0",
            "wlp2s0",
            "WLAN",
            "以太网 2",
            "Ethernet",
        ] {
            assert!(looks_physical(name), "{} 应是物理网卡名", name);
        }
        assert!(!looks_physical("utun4"));
        assert!(!looks_physical("bridge100"));
    }
}

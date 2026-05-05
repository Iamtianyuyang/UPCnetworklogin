use anyhow::{bail, Result};
use local_ip_address::list_afinet_netifas;
use std::net::IpAddr;

const LAN_URL: &str = "http://lan.upc.edu.cn";
const WLAN_URL: &str = "http://wlan.upc.edu.cn";

pub fn get_interface_ip(name: &str) -> Result<IpAddr> {
    let interfaces = list_afinet_netifas()?;
    for (iface, ip) in &interfaces {
        if iface == name {
            return Ok(*ip);
        }
    }
    bail!("未找到网卡: {}", name)
}

fn detect_url(ip: IpAddr) -> &'static str {
    if ip.to_string().starts_with("10.") {
        LAN_URL
    } else {
        WLAN_URL
    }
}

fn find_default_ip() -> Option<IpAddr> {
    let interfaces = list_afinet_netifas().ok()?;
    let mut best: Option<IpAddr> = None;
    for (iface, ip) in &interfaces {
        let ip_str = ip.to_string();
        if ip_str.starts_with("127.") || ip_str == "::1" {
            continue;
        }
        if let IpAddr::V4(v4) = ip {
            if best.is_none() {
                best = Some(IpAddr::V4(*v4));
            }
            if iface.starts_with("eth") || iface.starts_with("en") || iface.starts_with("wlan") || iface.starts_with("wlp") {
                return Some(IpAddr::V4(*v4));
            }
        }
    }
    best
}

pub fn resolve_base_url(interface: Option<&str>) -> Result<&'static str> {
    match interface {
        Some(name) => {
            let ip = get_interface_ip(name)?;
            println!("interface: {} ({})", name, ip);
            Ok(detect_url(ip))
        }
        None => {
            match find_default_ip() {
                Some(ip) => {
                    println!("auto-detected ip: {}", ip);
                    Ok(detect_url(ip))
                }
                None => Ok(LAN_URL),
            }
        }
    }
}

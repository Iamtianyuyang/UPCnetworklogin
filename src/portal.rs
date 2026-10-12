use anyhow::{bail, Result};
use socket2::{Domain, Protocol, Socket, Type};
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

const UA: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36";

/// 未被拦截时的兜底门户地址（登出时没有 queryString 可用）
pub const DEFAULT_PORTAL: &str = "http://wlan.upc.edu.cn";

/// 在线探针：被 BRAS 拦截时返回带 wlanuserip 的 302，已在线时返回 204/200 且无 Location。
/// 用外网地址而非门户域名做探针，是因为门户域名通常整段免认证、永远不会被拦截。
const PROBES: [&str; 3] = [
    "http://connect.rom.miui.com/generate_204",
    "http://www.baidu.com/",
    "http://lan.upc.edu.cn/",
];

const TIMEOUT: Duration = Duration::from_secs(8);
const MAX_RESPONSE: usize = 1 << 20;

pub struct PortalInfo {
    pub base_url: String,
    pub query_string: String,
}

pub struct Response {
    pub status: u16,
    pub location: Option<String>,
    pub body: String,
}

/// 一次会话。可绑定本地源地址 —— 锐捷 BRAS 会校验 queryString 里的 wlanuserip
/// 与请求源 IP 是否一致，多网卡机器上必须绑定，否则登录会被判"认证失败"。
pub struct Session {
    local_ip: Option<IpAddr>,
}

impl Session {
    pub fn new(local_ip: Option<IpAddr>) -> Self {
        Session { local_ip }
    }

    fn connect(&self, host: &str, port: u16) -> Result<TcpStream> {
        let addrs: Vec<SocketAddr> = (host, port)
            .to_socket_addrs()
            .map_err(|e| anyhow::anyhow!("解析 {} 失败: {}", host, e))?
            .collect();

        // 绑定源地址时目标必须是同族地址
        let target = match self.local_ip {
            Some(IpAddr::V4(_)) => addrs.iter().find(|a| a.is_ipv4()).copied(),
            Some(IpAddr::V6(_)) => addrs.iter().find(|a| a.is_ipv6()).copied(),
            None => addrs.first().copied(),
        }
        .ok_or_else(|| anyhow::anyhow!("无法解析 {} 的地址", host))?;

        let stream = match self.local_ip {
            Some(local) => {
                let domain = if local.is_ipv4() { Domain::IPV4 } else { Domain::IPV6 };
                let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;
                socket.bind(&SocketAddr::new(local, 0).into())?;
                socket.connect_timeout(&target.into(), TIMEOUT)?;
                let stream: TcpStream = socket.into();
                stream
            }
            None => TcpStream::connect_timeout(&target, TIMEOUT)?,
        };

        stream.set_read_timeout(Some(TIMEOUT))?;
        stream.set_write_timeout(Some(TIMEOUT))?;
        Ok(stream)
    }

    /// 极简 HTTP/1.1 明文请求（Connection: close，整包读到 EOF）
    pub fn request(&self, method: &str, url: &str, body: Option<&str>) -> Result<Response> {
        let rest = url
            .strip_prefix("http://")
            .ok_or_else(|| anyhow::anyhow!("仅支持 http:// 地址: {}", url))?;
        let (authority, path) = match rest.find('/') {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => (rest, "/"),
        };
        let (host, port) = match authority.rsplit_once(':') {
            Some((h, p)) => (h, p.parse::<u16>().unwrap_or(80)),
            None => (authority, 80u16),
        };

        let mut stream = self.connect(host, port)?;

        let mut req = String::new();
        req.push_str(&format!("{} {} HTTP/1.1\r\n", method, path));
        req.push_str(&format!("Host: {}\r\n", authority));
        req.push_str(&format!("User-Agent: {}\r\n", UA));
        req.push_str("Accept: */*\r\n");
        if let Some(b) = body {
            req.push_str("Content-Type: application/x-www-form-urlencoded; charset=UTF-8\r\n");
            req.push_str(&format!("Content-Length: {}\r\n", b.as_bytes().len()));
        }
        req.push_str("Connection: close\r\n\r\n");
        if let Some(b) = body {
            req.push_str(b);
        }

        stream.write_all(req.as_bytes())?;
        stream.flush()?;

        let mut raw = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            let n = stream.read(&mut buf)?;
            if n == 0 {
                break;
            }
            raw.extend_from_slice(&buf[..n]);
            if raw.len() > MAX_RESPONSE {
                break;
            }
        }

        parse_response(&raw)
    }
}

fn parse_response(raw: &[u8]) -> Result<Response> {
    let text = String::from_utf8_lossy(raw);
    let (head, body) = match text.find("\r\n\r\n") {
        Some(idx) => (&text[..idx], text[idx + 4..].to_string()),
        None => (text.as_ref(), String::new()),
    };

    let mut lines = head.split("\r\n");
    let status_line = lines.next().unwrap_or("");
    let status = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse::<u16>().ok())
        .unwrap_or(0);
    if status == 0 {
        bail!("无法解析响应: {}", status_line);
    }

    let mut location = None;
    for line in lines {
        if let Some((k, v)) = line.split_once(':') {
            if k.eq_ignore_ascii_case("location") {
                location = Some(v.trim().to_string());
            }
        }
    }

    Ok(Response { status, location, body })
}

fn extract_base_url(location: &str) -> Option<String> {
    let url = location.split('?').next()?;
    let scheme_end = url.find("://")?;
    let after_scheme = &url[scheme_end + 3..];
    let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
    Some(format!("{}://{}", &url[..scheme_end], &after_scheme[..host_end]))
}

/// 探测当前线路状态：
/// - Some(...) 表示被 BRAS 拦截、需要登录，且已拿到可直接使用的 queryString
/// - None     表示已在线，无需登录
pub fn discover(session: &Session) -> Result<Option<PortalInfo>> {
    let mut last_err = None;

    for probe in PROBES {
        match session.request("GET", probe, None) {
            Ok(resp) => {
                if resp.status >= 400 {
                    last_err = Some(anyhow::anyhow!("探针 {} 返回 HTTP {}", probe, resp.status));
                    continue;
                }
                if let Some(location) = resp.location.as_deref() {
                    if let Some((_, query)) = location.split_once('?') {
                        if query.contains("userip") || query.contains("userIndex") {
                            let actual = extract_base_url(location)
                                .unwrap_or_else(|| DEFAULT_PORTAL.to_string());
                            return Ok(Some(PortalInfo {
                                base_url: actual,
                                query_string: query.to_string(),
                            }));
                        }
                    }
                }
                return Ok(None);
            }
            Err(e) => last_err = Some(e),
        }
    }

    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("无法连接认证探针")))
}

/// application/x-www-form-urlencoded 值编码（避免密码里的 & = + % 空格或中文破坏请求）
fn form_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// 登录，返回是否成功（调用方据此决定退出码与守护进程的重试计数）
pub fn login(
    session: &Session,
    base_url: &str,
    query_string: &str,
    username: &str,
    password: &str,
    service: &str,
) -> Result<bool> {
    // queryString 已是锐捷要求的双层编码结果，不能再编码；其余字段按表单规则编码
    let post_data = format!(
        "userId={}&password={}&service={}&queryString={}&operatorPwd=&operatorUserId=&validcode=&passwordEncrypt=false",
        form_encode(username),
        form_encode(password),
        form_encode(service),
        query_string
    );

    let resp = session.request(
        "POST",
        &format!("{}/eportal/InterFace.do?method=login", base_url),
        Some(&post_data),
    )?;

    let body: serde_json::Value = serde_json::from_str(&resp.body).map_err(|_| {
        anyhow::anyhow!(
            "登录响应非JSON: {}",
            resp.body.chars().take(200).collect::<String>()
        )
    })?;

    let result = body.get("result").and_then(|v| v.as_str()).unwrap_or("未知");
    let message = body.get("message").and_then(|v| v.as_str()).unwrap_or("未知");

    if result == "success" {
        crate::output::print_login_success(message);
        Ok(true)
    } else {
        crate::output::print_login_failure(message);
        if message.contains("认证失败") {
            eprintln!(
                "   提示: 该响应通常表示 queryString 里的 wlanuserip 与请求源 IP 不一致，"
            );
            eprintln!(
                "   请确认已用 -i 指定目标网卡（程序会绑定其 IPv4 地址作为源地址）。"
            );
        }
        Ok(false)
    }
}

/// 登出，返回是否成功
pub fn logout(session: &Session, base_url: &str) -> Result<bool> {
    let resp = session.request(
        "POST",
        &format!("{}/eportal/InterFace.do?method=logout", base_url),
        Some(""),
    )?;

    let body: serde_json::Value = serde_json::from_str(&resp.body).map_err(|_| {
        anyhow::anyhow!(
            "登出响应非JSON: {}",
            resp.body.chars().take(200).collect::<String>()
        )
    })?;

    let result = body.get("result").and_then(|v| v.as_str()).unwrap_or("未知");
    let message = body.get("message").and_then(|v| v.as_str()).unwrap_or("未知");

    if result == "success" {
        crate::output::print_logout_success(message);
        Ok(true)
    } else if message.contains("不在线") {
        // 该线路本就没有 ePortal 会话（例如运营商直连端口），目标状态已满足
        crate::output::print_logout_not_needed(message);
        Ok(true)
    } else {
        crate::output::print_logout_failure(message);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn form_encode_escapes_reserved_characters() {
        assert_eq!(form_encode("a&b=c+d"), "a%26b%3Dc%2Bd");
        assert_eq!(form_encode("p@ss word"), "p%40ss%20word");
        assert_eq!(form_encode("100%"), "100%25");
        assert_eq!(form_encode("中文"), "%E4%B8%AD%E6%96%87");
    }

    #[test]
    fn form_encode_keeps_unreserved_characters() {
        assert_eq!(form_encode("BY2101040110"), "BY2101040110");
        assert_eq!(form_encode("a-b_c.d~e"), "a-b_c.d~e");
    }

    #[test]
    fn extract_base_url_strips_path_and_query() {
        assert_eq!(
            extract_base_url("http://wlan.upc.edu.cn/eportal/index.jsp?wlanuserip=1.2.3.4"),
            Some("http://wlan.upc.edu.cn".to_string())
        );
        assert_eq!(
            extract_base_url("http://10.0.0.1:8080/eportal/index.jsp?a=b"),
            Some("http://10.0.0.1:8080".to_string())
        );
    }

    #[test]
    fn parse_response_reads_status_and_location() {
        let raw = b"HTTP/1.0 302 Moved Temporarily\r\nServer: NetEngine Server 1.0\r\nLocation: http://wlan.upc.edu.cn/eportal/index.jsp?wlanuserip=172.24.132.4\r\n\r\n";
        let resp = parse_response(raw).unwrap();
        assert_eq!(resp.status, 302);
        assert!(resp
            .location
            .unwrap()
            .contains("wlanuserip=172.24.132.4"));
    }
}

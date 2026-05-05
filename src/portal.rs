use anyhow::{bail, Result};
use reqwest::blocking::Client;
use std::time::Duration;

pub fn build_client() -> Result<Client> {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(10))
        .build()?;
    Ok(client)
}

pub struct PortalInfo {
    pub base_url: String,
    pub query_string: String,
}

fn extract_base_url(location: &str) -> Option<String> {
    let url = location.split('?').next()?;
    let scheme_end = url.find("://")?;
    let after_scheme = &url[scheme_end + 3..];
    let host_end = after_scheme.find('/').unwrap_or(after_scheme.len());
    Some(format!("{}://{}", &url[..scheme_end], &after_scheme[..host_end]))
}

pub fn get_portal_info(client: &Client, base_url: &str) -> Result<PortalInfo> {
    let resp = client
        .get(format!("{}/", base_url))
        .header(
            "Accept",
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        )
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
        )
        .send()?;

    let location = resp
        .headers()
        .get("Location")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if location.is_empty() {
        bail!("未收到重定向，无法连接到认证服务器");
    }

    if let Some(query_start) = location.find('?') {
        let query = &location[query_start + 1..];
        if !query.is_empty() {
            let actual_base = extract_base_url(location)
                .unwrap_or_else(|| base_url.to_string());
            return Ok(PortalInfo {
                base_url: actual_base,
                query_string: query.to_string(),
            });
        }
    }

    if location.contains("success") {
        bail!("already logged in");
    }

    bail!("未知的重定向目标: {}", location)
}

pub fn check_needs_login(client: &Client, base_url: &str) -> Result<bool> {
    match get_portal_info(client, base_url) {
        Ok(_) => Ok(true),
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("already logged in") {
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}

pub fn login(
    client: &Client,
    base_url: &str,
    query_string: &str,
    username: &str,
    password: &str,
    service: &str,
) -> Result<()> {
    let post_data = format!(
        "userId={}&password={}&service={}&queryString={}&operatorPwd=&operatorUserId=&validcode=&passwordEncrypt=false",
        username, password, service, query_string
    );

    let resp = client
        .post(format!(
            "{}/eportal/InterFace.do?method=login",
            base_url
        ))
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
        )
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .header("Accept", "*/*")
        .body(post_data)
        .send()?;

    let body = resp.text()?;
    let body: serde_json::Value = serde_json::from_str(&body)
        .map_err(|_| anyhow::anyhow!("登录响应非JSON: {}", body.chars().take(200).collect::<String>()))?;

    let result = body
        .get("result")
        .and_then(|v| v.as_str())
        .unwrap_or("未知");
    let message = body
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("未知");

    if result == "success" {
        crate::output::print_login_success(message);
    } else {
        crate::output::print_login_failure(message);
    }

    Ok(())
}

pub fn logout(client: &Client, base_url: &str) -> Result<()> {
    let resp = client
        .post(format!(
            "{}/eportal/InterFace.do?method=logout",
            base_url
        ))
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36",
        )
        .header(
            "Content-Type",
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .send()?;

    let body = resp.text()?;
    let body: serde_json::Value = serde_json::from_str(&body)
        .map_err(|_| anyhow::anyhow!("登出响应非JSON: {}", body.chars().take(200).collect::<String>()))?;

    let result = body
        .get("result")
        .and_then(|v| v.as_str())
        .unwrap_or("未知");
    let message = body
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("未知");

    if result == "success" {
        crate::output::print_logout_success(message);
    } else {
        crate::output::print_logout_failure(message);
    }

    Ok(())
}

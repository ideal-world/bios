//! Http request helper
//!
//! Http请求辅助操作
use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

use itertools::Itertools;
use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    web::poem::{http::header::FORWARDED, Request},
};

pub const REMOTE_ADDR: &str = "remote-addr";

/// Add ip to context
pub async fn add_ip(ip: Option<String>, ctx: &TardisContext) -> TardisResult<()> {
    if let Some(ip) = ip {
        ctx.add_ext(REMOTE_ADDR, &ip).await?;
    }
    Ok(())
}

/// Try to set real ip from request to context
pub async fn try_set_real_ip_from_req_to_ctx(request: &Request, ctx: &TardisContext) -> TardisResult<()> {
    ctx.add_ext(REMOTE_ADDR, &try_get_real_ip_from_req(request).await?.unwrap_or_default()).await?;
    Ok(())
}

/// Parse the Forwarded header to get the IP
///
/// Forwarded format： `Forwarded: by=<identifier>; for=<identifier><,for=<identifier>>; host=<host>; proto=<http|https>`
///
/// ```
/// use bios_basic::helper::request_helper::parse_forwarded_ip;
/// assert_eq!(parse_forwarded_ip("Forwarded: for=192.168.0.11; proto=http").unwrap().to_string(),"192.168.0.11");
/// assert_eq!(parse_forwarded_ip("Forwarded: for=192.168.0.9, 192.168.0.11; proto=http").unwrap().to_string(), "192.168.0.9");
/// assert_eq!(parse_forwarded_ip("Forwarded: proto=http; for=192.168.0.12").unwrap().to_string(), "192.168.0.12");
/// assert_eq!(parse_forwarded_ip("Forwarded: for=192.168.0.10").unwrap().to_string(), "192.168.0.10");
/// assert_eq!(parse_forwarded_ip("Forwarded: proto=http; for=192.168.0"), None);
/// ```
pub fn parse_forwarded_ip(forwarded_value: &str) -> Option<IpAddr> {
    forwarded_value.strip_prefix("Forwarded: ").and_then(|forwarded_value| {
        forwarded_value
            .split(';')
            .find(|part| part.trim().starts_with("for="))
            .and_then(|part| part.trim()[4..].split(',').next().and_then(|ip_str| IpAddr::from_str(ip_str).ok()))
    })
}

/// Convert IPv4-mapped IPv6 to ipv4
/// e.g
/// ::ffff:192.168.0.11 => 192.168.0.11
/// ```
/// use std::{
/// net::IpAddr,
/// str::FromStr,
/// };
/// use bios_basic::helper::request_helper::mapped_ipv6_to_ipv4;
/// assert_eq!(mapped_ipv6_to_ipv4(IpAddr::from_str("::ffff:192.168.0.11").unwrap()),IpAddr::from_str("192.168.0.11").unwrap() );
/// assert_eq!(mapped_ipv6_to_ipv4(IpAddr::from_str("::ffff:c0a8:000b").unwrap()),IpAddr::from_str("192.168.0.11").unwrap() );
/// assert_eq!(mapped_ipv6_to_ipv4(IpAddr::from_str("192.168.0.11").unwrap()),IpAddr::from_str("192.168.0.11").unwrap());
/// assert_eq!(mapped_ipv6_to_ipv4(IpAddr::from_str("fd00::1a2b:3c4d:5e6f:7a8b").unwrap()),IpAddr::from_str("fd00::1a2b:3c4d:5e6f:7a8b").unwrap());
/// assert_eq!(mapped_ipv6_to_ipv4(IpAddr::from_str("::1").unwrap()),IpAddr::from_str("::1").unwrap());
/// ```
pub fn mapped_ipv6_to_ipv4(ip_addr: IpAddr) -> IpAddr {
    match ip_addr {
        IpAddr::V6(ip_addr) => {
            let segments = ip_addr.segments();
            if segments[0] == 0 && segments[1] == 0 && segments[2] == 0 && segments[3] == 0 && segments[4] == 0 && segments[5] == 0xffff {
                // 提取并返回 IPv4 地址部分
                IpAddr::V4(Ipv4Addr::new(ip_addr.octets()[12], ip_addr.octets()[13], ip_addr.octets()[14], ip_addr.octets()[15]))
            } else {
                IpAddr::V6(ip_addr)
            }
        }
        IpAddr::V4(ip_addr) => IpAddr::V4(ip_addr),
    }
}

/// Try to get real ip from request
///
/// This method only parses the main request headers and cannot guarantee that the real IP can be obtained.
pub async fn try_get_real_ip_from_req(request: &Request) -> TardisResult<Option<String>> {
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Forwarded
    if let Some(forwarded_header) = request.headers().get(FORWARDED) {
        if let Ok(forwarded_value) = forwarded_header.to_str() {
            if let Some(ip) = parse_forwarded_ip(forwarded_value.trim()) {
                return Ok(Some(mapped_ipv6_to_ipv4(ip).to_string()));
            }
        }
    }
    if let Some(xff_header) = request.headers().get("X-Forwarded-For") {
        if let Ok(xff_value) = xff_header.to_str() {
            if let Some(ip) = xff_value.split(',').next().and_then(|s| IpAddr::from_str(s.trim()).ok()) {
                return Ok(Some(mapped_ipv6_to_ipv4(ip).to_string()));
            }
        }
    }
    if let Some(xrp_header) = request.headers().get("X-Real-IP") {
        if let Ok(xrp_value) = xrp_header.to_str() {
            if let Some(ip) = xrp_value.split(',').next().and_then(|s| IpAddr::from_str(s.trim()).ok()) {
                return Ok(Some(mapped_ipv6_to_ipv4(ip).to_string()));
            }
        }
    }
    Ok(request.remote_addr().as_socket_addr().map(|addr| mapped_ipv6_to_ipv4(addr.ip()).to_string()))
}

/// Get real ip from context
pub async fn get_real_ip_from_ctx(ctx: &TardisContext) -> TardisResult<Option<String>> {
    ctx.get_ext(REMOTE_ADDR).await
}

/// Sort query string and convert to lowercase
pub fn sort_query(query: &str) -> String {
    if query.is_empty() {
        return "".to_string();
    }
    query.split('&').sorted_by(|a, b| Ord::cmp(&a.to_lowercase(), &b.to_lowercase())).join("&")
}

/// Sort query and convert to lowercase
pub fn sort_hashmap_query(query: HashMap<String, String>) -> String {
    if query.is_empty() {
        return "".to_string();
    }
    query.iter().map(|a| format!("{}={}", a.0, a.1)).sorted_by(|a, b| Ord::cmp(&a.to_lowercase(), &b.to_lowercase())).join("&")
}

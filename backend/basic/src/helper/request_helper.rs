//! Http request helper
use std::{collections::HashMap, net::IpAddr, str::FromStr};

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
    forwarded_value
        .strip_prefix("Forwarded: ")
        .and_then(|forwarded_value| {
            forwarded_value
                .split(';')
                .find(|part| part.trim().starts_with("for="))
                .and_then(|part| part.trim()[4..].split(',').next().and_then(|ip_str| IpAddr::from_str(ip_str).ok()))
        })
}

/// Try to get real ip from request
///
/// This method only parses the main request headers and cannot guarantee that the real IP can be obtained.
pub async fn try_get_real_ip_from_req(request: &Request) -> TardisResult<Option<String>> {
    // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Forwarded
    if let Some(forwarded_header) = request.headers().get(FORWARDED) {
        if let Ok(forwarded_value) = forwarded_header.to_str() {
            if let Some(ip) = parse_forwarded_ip(forwarded_value.trim()) {
                return Ok(Some(ip.to_string()));
            }
        }
    }
    if let Some(xff_header) = request.headers().get("X-Forwarded-For") {
        if let Ok(xff_value) = xff_header.to_str() {
            if let Some(ip) = xff_value.split(',').next().and_then(|s| IpAddr::from_str(s.trim()).ok()) {
                return Ok(Some(ip.to_string()));
            }
        }
    }
    if let Some(xrp_header) = request.headers().get("X-Real-IP") {
        if let Ok(xrp_value) = xrp_header.to_str() {
            if let Some(ip) = xrp_value.split(',').next().and_then(|s| IpAddr::from_str(s.trim()).ok()) {
                return Ok(Some(ip.to_string()));
            }
        }
    }
    Ok(request.remote_addr().as_socket_addr().map(|addr| addr.ip().to_string()))
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

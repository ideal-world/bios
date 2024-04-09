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

/// Try to get real ip from request
pub async fn try_get_real_ip_from_req(request: &Request) -> TardisResult<Option<String>> {
    fn parse_forwarded_ip(field: &str) -> Option<IpAddr> {
        if let Some(pos) = field.find("for=") {
            let ip_str = &field[pos + 4..];
            if let Ok(ip) = IpAddr::from_str(ip_str) {
                return Some(ip);
            }
        }
        None
    }
    if let Some(forwarded_header) = request.headers().get(FORWARDED) {
        if let Ok(forwarded_value) = forwarded_header.to_str() {
            for field in forwarded_value.split(',') {
                if let Some(ip) = parse_forwarded_ip(field.trim()) {
                    return Ok(Some(ip.to_string()));
                }
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
    if let Some(xff_header) = request.headers().get("X-Real-IP") {
        if let Ok(xff_value) = xff_header.to_str() {
            if let Some(ip) = xff_value.split(',').next().and_then(|s| IpAddr::from_str(s.trim()).ok()) {
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

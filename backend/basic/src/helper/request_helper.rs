use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    web::poem::Request,
};

pub const REMOTE_ADDR: &str = "remote-addr";

// Add ip to context
pub async fn add_ip(ip: Option<String>, ctx: &TardisContext) -> TardisResult<()> {
    if let Some(ip) = ip {
        ctx.add_ext(REMOTE_ADDR, &ip).await?;
    }
    Ok(())
}

pub async fn add_remote_ip(request: &Request, ctx: &TardisContext) -> TardisResult<()> {
    ctx.add_ext(REMOTE_ADDR, &get_ip(request).await?.unwrap_or_default()).await?;
    Ok(())
}

pub async fn get_ip(request: &Request) -> TardisResult<Option<String>> {
    let back_ip = request.remote_addr().as_socket_addr().map(|socket_addr| socket_addr.ip().to_string());
    let ip = if let Some(real_ips) = request.headers().get("X-Forwarded-For") {
        if let Some(real_ip) = real_ips.to_str().ok().and_then(|ips| ips.split(',').collect::<Vec<_>>().first().map(|ip| ip.to_string())) {
            Some(real_ip)
        } else {
            back_ip
        }
    } else {
        back_ip
    };
    Ok(ip)
}

pub async fn get_remote_ip(ctx: &TardisContext) -> TardisResult<Option<String>> {
    ctx.get_ext(REMOTE_ADDR).await
}

use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    web::poem::Request,
};

pub const REMOTE_ADDR: &str = "remote-addr";

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
    if let Some(socert_addr) = request.remote_addr().as_socket_addr() {
        Ok(Some(socert_addr.ip().to_string()))
    } else {
        Ok(None)
    }
}

pub async fn get_remote_ip(ctx: &TardisContext) -> TardisResult<Option<String>> {
    Ok(ctx.get_ext(REMOTE_ADDR).await?)
}

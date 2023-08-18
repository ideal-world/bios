use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    web::poem::Request,
};

pub const REMOTE_ADDR: &str = "remote-addr";

pub async fn add_remote_ip(request: &Request, ctx: &TardisContext) -> TardisResult<()> {
    if let Some(add) = request.remote_addr().as_socket_addr() {
        ctx.add_ext(REMOTE_ADDR, &add.ip().to_string()).await?;
    }
    Ok(())
}

pub async fn get_remote_ip(ctx: &TardisContext) -> TardisResult<Option<String>> {
    Ok(ctx.get_ext(REMOTE_ADDR).await?)
}

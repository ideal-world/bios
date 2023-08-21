use tardis::{
    basic::{dto::TardisContext, result::TardisResult},
    web::poem::Request,
};

pub const REMOTE_ADDR: &str = "remote-addr";

pub async fn add_remote_ip(request: &Request, ctx: &TardisContext) -> TardisResult<()> {
    ctx.add_ext(REMOTE_ADDR, &request.remote_addr().to_string()).await?;
    Ok(())
}

pub async fn get_remote_ip(ctx: &TardisContext) -> TardisResult<Option<String>> {
    Ok(ctx.get_ext(REMOTE_ADDR).await?)
}

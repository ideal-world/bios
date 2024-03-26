use crate::rbum::rbum_config::RbumConfigApi;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    web::poem::Request,
    TardisFuns, TardisFunsInst,
};

pub async fn unsafe_fill_ctx(request: &Request, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()> {
    let bios_ctx = if let Some(bios_ctx) = request.header(&funs.rbum_head_key_bios_ctx()).or_else(|| request.header(&funs.rbum_head_key_bios_ctx().to_lowercase())) {
        TardisFuns::json.str_to_obj::<TardisContext>(&TardisFuns::crypto.base64.decode_to_string(bios_ctx)?)?
    } else {
        return Err(TardisError::unauthorized(
            &format!("[Basic] Request is not legal, missing header [{}]", funs.rbum_head_key_bios_ctx()),
            "401-auth-req-ak-not-exist",
        ));
    };

    if bios_ctx.own_paths.contains(&ctx.own_paths) {
        let mut roles = bios_ctx.roles.clone();
        for role in bios_ctx.roles.clone() {
            if role.contains(':') {
                let extend_role = role.split(':').collect::<Vec<_>>()[0];
                roles.push(extend_role.to_string());
            }
        }
        ctx.owner = bios_ctx.owner.clone();
        ctx.roles = roles;
        ctx.groups = bios_ctx.groups;
        ctx.own_paths = bios_ctx.own_paths;

        Ok(())
    } else {
        Err(TardisError::forbidden(
            &format!("[Basic] Request is not legal from head [{}]", funs.rbum_head_key_bios_ctx()),
            "403-auth-req-permission-denied",
        ))
    }
}

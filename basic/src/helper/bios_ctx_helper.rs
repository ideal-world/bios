use crate::rbum::rbum_config::RbumConfigApi;
use tardis::{
    basic::{dto::TardisContext, error::TardisError, result::TardisResult},
    web::poem::Request,
    TardisFuns, TardisFunsInst,
};

fn unsafe_fill_ctx<F>(request: &Request, f: F, check: bool, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()>
where
    F: FnOnce(TardisContext, &mut TardisContext),
{
    if check && !ctx.owner.is_empty() {
        return Ok(());
    }
    let bios_ctx = if let Some(bios_ctx) = request.header(&funs.rbum_head_key_bios_ctx()).or_else(|| request.header(&funs.rbum_head_key_bios_ctx().to_lowercase())) {
        TardisFuns::json.str_to_obj::<TardisContext>(&TardisFuns::crypto.base64.decode_to_string(bios_ctx)?)?
    } else {
        return Err(TardisError::unauthorized(
            &format!("[Basic] Request is not legal, missing header [{}]", funs.rbum_head_key_bios_ctx()),
            "401-auth-req-ak-not-exist",
        ));
    };

    if bios_ctx.own_paths.contains(&ctx.own_paths) {
        f(bios_ctx, ctx);

        Ok(())
    } else {
        Err(TardisError::forbidden(
            &format!("[Basic] Request is not legal from head [{}]", funs.rbum_head_key_bios_ctx()),
            "403-auth-req-permission-denied",
        ))
    }
}

// xxx_check_own function will check the owner is empty or not.
pub fn check_own_fill_ctx(request: &Request, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()> {
    unsafe_fill_ctx(
        request,
        |bios_ctx, ctx| {
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
        },
        true,
        funs,
        ctx,
    )
}

pub fn unsfae_fill_ctx(request: &Request, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()> {
    unsafe_fill_ctx(
        request,
        |bios_ctx, ctx| {
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
        },
        false,
        funs,
        ctx,
    )
}

pub fn unsfae_fill_owner_only(request: &Request, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()> {
    unsafe_fill_ctx(
        request,
        |bios_ctx, ctx| {
            ctx.owner = bios_ctx.owner.clone();
        },
        false,
        funs,
        ctx,
    )
}

pub fn unsfae_fill_own_paths_only(request: &Request, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()> {
    unsafe_fill_ctx(
        request,
        |bios_ctx, ctx| {
            ctx.own_paths = bios_ctx.own_paths;
        },
        false,
        funs,
        ctx,
    )
}

pub fn unsfae_fill_roles_only(request: &Request, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()> {
    unsafe_fill_ctx(
        request,
        |bios_ctx, ctx| {
            let mut roles = bios_ctx.roles.clone();
            for role in bios_ctx.roles.clone() {
                if role.contains(':') {
                    let extend_role = role.split(':').collect::<Vec<_>>()[0];
                    roles.push(extend_role.to_string());
                }
            }
            ctx.roles = roles;
        },
        false,
        funs,
        ctx,
    )
}

pub fn unsfae_fill_groups_only(request: &Request, funs: &TardisFunsInst, ctx: &mut TardisContext) -> TardisResult<()> {
    unsafe_fill_ctx(
        request,
        |bios_ctx, ctx| {
            ctx.groups = bios_ctx.groups;
        },
        false,
        funs,
        ctx,
    )
}

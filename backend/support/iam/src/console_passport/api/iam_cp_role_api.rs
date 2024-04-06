use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::basic::dto::iam_filer_dto::IamRoleFilterReq;
use crate::basic::dto::iam_role_dto::IamRoleBoneResp;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_constants;
use bios_basic::helper::request_helper::add_remote_ip;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::poem::Request;

#[derive(Clone, Default)]
pub struct IamCpRoleApi;

#[poem_openapi::OpenApi(prefix_path = "/cp", tag = "bios_basic::ApiTag::Passport")]
impl IamCpRoleApi {
    /// Find Role By CTX
    #[oai(path = "/", method = "get")]
    async fn find_by_ctx(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Vec<IamRoleBoneResp>> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let roles = ctx.0.roles.clone();
        let result = IamRoleServ::do_find_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(roles),
                    enabled: Some(true),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(
            result
                .into_iter()
                .map(|item| IamRoleBoneResp {
                    id: item.id,
                    name: item.name,
                    code: item.code,
                    kind: item.kind,
                    scope_level: item.scope_level,
                    icon: item.icon,
                    in_base: item.in_base,
                    in_embed: item.in_embed,
                    extend_role_id: item.extend_role_id,
                })
                .collect(),
        )
    }
}

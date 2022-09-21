use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamRoleFilterReq;
use crate::basic::dto::iam_role_dto::IamRoleBoneResp;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_constants;
use crate::iam_constants::RBUM_SCOPE_LEVEL_APP;

pub struct IamCcRoleApi;

/// Common Console Role API
#[poem_openapi::OpenApi(prefix_path = "/cc/role", tag = "crate::iam_enumeration::Tag::Common")]
impl IamCcRoleApi {
    /// Find Roles
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamRoleBoneResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::paginate_items(
            &IamRoleFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    enabled: Some(true),
                    scope_level: if IamAppServ::is_app_level_by_ctx(&ctx.0) { Some(RBUM_SCOPE_LEVEL_APP) } else { None },
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    ..Default::default()
                },
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(TardisPage {
            page_size: result.page_size,
            page_number: result.page_number,
            total_size: result.total_size,
            records: result
                .records
                .into_iter()
                .map(|item| IamRoleBoneResp {
                    id: item.id,
                    name: item.name,
                    icon: item.icon,
                })
                .collect(),
        })
    }

    /// Find pub Rel Res By Role Id
    #[oai(path = "/:id/pub_res", method = "get")]
    async fn find_rel_res_with_pub(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<RbumRelBoneResp>> {
        let mut ctx = ctx.0;
        ctx.own_paths = "".to_string();
        let funs = iam_constants::get_tardis_inst();
        let result = IamRoleServ::find_simple_rels(
            &id.0,
            desc_by_create.0,
            desc_by_update.0,
            Some(vec![
                RbumScopeLevelKind::Root.to_int().try_into().unwrap(),
                RbumScopeLevelKind::L1.to_int().try_into().unwrap(),
                RbumScopeLevelKind::L2.to_int().try_into().unwrap(),
                RbumScopeLevelKind::L3.to_int().try_into().unwrap(),
            ]),
            &funs,
            &ctx,
        )
        .await?;
        TardisResp::ok(result)
    }
}

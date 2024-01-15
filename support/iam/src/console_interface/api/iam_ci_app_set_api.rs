use bios_basic::rbum::dto::rbum_set_dto::RbumSetTreeResp;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetItemServ;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumSetItemFilterReq, RbumSetTreeFilterReq, RbumBasicFilterReq};
use bios_basic::rbum::dto::rbum_set_item_dto::RbumSetItemDetailResp;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;

use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_constants;
use crate::iam_enumeration::IamSetKind;
use bios_basic::helper::request_helper::add_remote_ip;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCiAppSetApi;

/// Interface Console App Set API
///
#[poem_openapi::OpenApi(prefix_path = "/ci/apps", tag = "bios_basic::ApiTag::Interface")]
impl IamCiAppSetApi {
    /// Find App Set Items (App Or Account)
    #[oai(path = "/item", method = "get")]
    async fn find_items(
        &self,
        cate_ids: Query<Option<String>>,
        item_ids: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<RbumSetItemDetailResp>> {
        let funs = iam_constants::get_tardis_inst();
        let ctx = IamCertServ::use_sys_or_tenant_ctx_unsafe(ctx.0)?;
        add_remote_ip(request, &ctx).await?;
        let set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let result = RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: false,
                    ..Default::default()
                },
                rel_rbum_item_disabled: Some(false),
                rel_rbum_set_id: Some(set_id.clone()),
                rel_rbum_set_cate_ids: cate_ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                rel_rbum_item_ids: item_ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }
}

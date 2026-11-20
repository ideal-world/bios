use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::basic::dto::iam_filer_dto::IamPublishSystemFilterReq;
use crate::basic::dto::iam_publish_system_dto::IamPublishSystemDetailResp;
use crate::basic::serv::iam_publish_system_serv::IamPublishSystemServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCiPublishSystemApi;

/// Interface Console Publish System API
/// 接口控制台发布系统API
#[poem_openapi::OpenApi(prefix_path = "/ci/publish_system", tag = "bios_basic::ApiTag::Interface")]
impl IamCiPublishSystemApi {
    /// Get Publish System By Id
    /// 根据ID获取发布系统
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamPublishSystemDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let result = IamPublishSystemServ::get_item(
            &id.0,
            &IamPublishSystemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}

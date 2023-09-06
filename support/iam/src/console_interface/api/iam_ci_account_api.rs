use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, param::Query};
use tardis::web::web_resp::{TardisApiResult, TardisResp};
use tardis::TardisFuns;

use crate::basic::dto::iam_account_dto::IamAccountDetailAggResp;
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_key_cache_serv::IamIdentCacheServ;
use crate::iam_constants;
use bios_basic::helper::request_helper::add_remote_ip;
use tardis::web::poem::Request;
#[derive(Clone, Default)]
pub struct IamCiAccountApi;

/// Interface Console Account API	接口控制台帐户API
#[poem_openapi::OpenApi(prefix_path = "/ci/account", tag = "bios_basic::ApiTag::Interface")]
impl IamCiAccountApi {
    /// Get Context By Account Id	根据帐户Id获取上下文
    #[oai(path = "/:id/ctx", method = "get")]
    async fn get_account_context(&self, id: Path<String>, app_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<String> {
        add_remote_ip(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let mut ctx_resp = IamIdentCacheServ::get_account_context(&id.0, &app_id.0.unwrap_or((&"").to_string()), &funs).await?;
        ctx_resp.own_paths = ctx.0.own_paths;
        TardisResp::ok(TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&ctx_resp).unwrap_or_default()))
    }

    //// Get Account By Account Id	通过帐户Id获取帐户
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamAccountDetailAggResp> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        add_remote_ip(request, &ctx).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamAccountServ::get_account_detail_aggs(
            &id.0,
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            true,
            true,
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }
}

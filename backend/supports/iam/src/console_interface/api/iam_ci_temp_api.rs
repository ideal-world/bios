use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use serde::{Deserialize, Serialize};
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::console_interface::serv::iam_ci_temp_serv::IamCiTempServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCiTempApi;

/// 临时产品访问授权
///
/// 账号侧记录可临时进入的产品，接口清单全系统共用一份。
#[poem_openapi::OpenApi(prefix_path = "/ci/temp", tag = "bios_basic::ApiTag::Interface")]
impl IamCiTempApi {
    /// 赋予或覆盖账号对一个产品的临时访问
    #[oai(path = "/product", method = "put")]
    async fn grant_product(&self, grant_req: Json<IamCiTempProductGrantReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        IamCiTempServ::grant_product(&grant_req.0.account_id, &grant_req.0.product_id, &grant_req.0.tenant_id, grant_req.0.expire_at, &funs).await?;
        TardisResp::ok(Void {})
    }

    /// 回收账号对一个产品的临时访问。不改全局临时接口清单。
    #[oai(path = "/product/:account_id/:product_id", method = "delete")]
    async fn revoke_product(&self, account_id: Path<String>, product_id: Path<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        IamCiTempServ::revoke_product(&account_id.0, &product_id.0, &funs).await?;
        TardisResp::ok(Void {})
    }

    /// 整体替换全系统可临时访问的接口。空列表表示清空。
    #[oai(path = "/api", method = "put")]
    async fn replace_apis(&self, api_req: Json<IamCiTempApiReplaceReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        IamCiTempServ::replace_apis(&api_req.0.apis, &funs).await?;
        TardisResp::ok(Void {})
    }
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCiTempProductGrantReq {
    pub account_id: String,
    pub product_id: String,
    pub tenant_id: String,
    /// Unix 秒。`0` 表示不过期。
    pub expire_at: i64,
}

#[derive(poem_openapi::Object, Serialize, Deserialize, Debug)]
pub struct IamCiTempApiReplaceReq {
    /// `{res_uri}##{res_action}`，与正式资源标识一致，可带 `*`、`**`。
    pub apis: Vec<String>,
}

use bios_basic::helper::request_helper::add_remote_ip;
use tardis::basic::dto::TardisContext;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_open_dto::{IamOpenAddProductReq, IamOpenBindAkProductReq};
use crate::basic::serv::iam_open_serv::IamOpenServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCiOpenApi;

/// # Interface Console Manage Open API
///
#[poem_openapi::OpenApi(prefix_path = "/ci/open", tag = "bios_basic::ApiTag::Interface")]
impl IamCiOpenApi {
    /// Add product / 添加产品
    #[oai(path = "/add_product", method = "post")]
    async fn add_product(&self, add_req: Json<IamOpenAddProductReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamOpenServ::add_product(&add_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Cert bind product_and_spec / 凭证绑定产品和规格
    #[oai(path = "/:id/bind_cert_product_and_spec", method = "post")]
    async fn bind_cert_product_and_spec(&self, id: Path<String>, bind_req: Json<IamOpenBindAkProductReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamOpenServ::bind_cert_product_and_spec(&id.0, &bind_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Refresh cumulative number of api calls / 刷新API累计调用数 (定时任务)
    #[oai(path = "/refresh_cert_cumulative_count", method = "post")]
    async fn refresh_cert_cumulative_count(&self, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        let ctx = TardisContext::default();
        funs.begin().await?;
        IamOpenServ::refresh_cert_cumulative_count(&funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }
}

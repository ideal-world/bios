use bios_basic::helper::request_helper::add_remote_ip;
use bios_basic::rbum::helper::rbum_scope_helper::check_without_owner_and_unsafe_fill_ctx;
use tardis::basic::dto::TardisContext;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_open_dto::{IamOpenAddOrModifyProductReq, IamOpenAkSkAddReq, IamOpenAkSkResp, IamOpenBindAkProductReq, IamOpenRuleResp};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_open_serv::IamOpenServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCiOpenApi;

/// # Interface Console Manage Open API
///
#[poem_openapi::OpenApi(prefix_path = "/ci/open", tag = "bios_basic::ApiTag::Interface")]
impl IamCiOpenApi {
    /// Add product / 添加产品
    #[oai(path = "/add_or_modify_product", method = "post")]
    async fn add_or_modify_product(&self, req: Json<IamOpenAddOrModifyProductReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        add_remote_ip(request, &ctx.0).await?;
        funs.begin().await?;
        IamOpenServ::add_or_modify_product(&req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Cert bind product_and_spec / 凭证绑定产品和规格
    #[oai(path = "/:id/bind_cert_product_and_spec", method = "post")]
    async fn bind_cert_product_and_spec(
        &self,
        id: Path<String>,
        bind_req: Json<IamOpenBindAkProductReq>,
        mut ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        add_remote_ip(request, &ctx.0).await?;
        funs.begin().await?;
        IamOpenServ::bind_cert_product_and_spec(&id.0, &bind_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Add aksk Cert by open platform / 生成AKSK通过开放平台
    #[oai(path = "/aksk", method = "post")]
    async fn add_aksk(&self, add_req: Json<IamOpenAkSkAddReq>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamOpenAkSkResp> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, Some(add_req.tenant_id.clone()))?;
        add_remote_ip(request, &ctx).await?;
        funs.begin().await?;
        let result = IamOpenServ::general_cert(add_req.0, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Get account rule info / 获取账号规则信息
    #[oai(path = "/", method = "get")]
    async fn get_rule_info(&self, cert_id: Query<String>, mut ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamOpenRuleResp> {
        let mut funs = iam_constants::get_tardis_inst();
        check_without_owner_and_unsafe_fill_ctx(request, &funs, &mut ctx.0)?;
        add_remote_ip(request, &ctx.0).await?;
        funs.begin().await?;
        let result = IamOpenServ::get_rule_info(cert_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Refresh cumulative number of api calls / 刷新API累计调用数 (定时任务)
    #[oai(path = "/refresh_cert_cumulative_count", method = "post")]
    async fn refresh_cert_cumulative_count(&self, _request: &Request) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        let ctx = TardisContext::default();
        funs.begin().await?;
        IamOpenServ::refresh_cert_cumulative_count(&funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }
}

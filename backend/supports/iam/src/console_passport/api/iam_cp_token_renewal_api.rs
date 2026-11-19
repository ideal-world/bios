use bios_basic::rbum::helper::rbum_event_helper;
use tardis::basic::result::TardisResult;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::{param::Path, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};
use tardis::TardisFunsInst;

use crate::basic::dto::iam_token_renewal_dto::{
    IamTokenRenewalExchangeReq, IamTokenRenewalExchangeResp, IamTokenRenewalRegisterReq, IamTokenRenewalRegisterResp, IamTokenRenewalRevokeReq,
};
use crate::basic::serv::iam_oauth2_task_grant_serv::validate_task_service_token;
use crate::basic::serv::iam_token_renewal_serv::IamTokenRenewalServ;
use crate::iam_constants;

/// 工作流运行级凭证续期注册（用户上下文）。
///
/// 生产链路中受信宿主把**本次被密封的凭证值本身**作为 `Bios-Token`（可选 `Bios-App`）经
/// 网关调用本端点；IAM 侧经 `TardisContextExtractor` 取得该凭证的主体上下文，授权因此绑定在
/// 被续期凭证的主体上，而不是宿主服务主体。
#[derive(Clone, Default)]
pub struct IamCpTokenRenewalApi;

#[poem_openapi::OpenApi(prefix_path = "/cp/token-renewals", tag = "bios_basic::ApiTag::Passport")]
impl IamCpTokenRenewalApi {
    /// 为本次 run 登记续期授权；capability 仅本次响应可见一次。
    #[oai(path = "/", method = "put")]
    async fn register(&self, req: Json<IamTokenRenewalRegisterReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamTokenRenewalRegisterResp> {
        let mut funs = iam_constants::get_tardis_inst();
        require_renewal_service_token(&funs, request)?;
        funs.begin().await?;
        let resp = IamTokenRenewalServ::register(&req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(notify_events) = rbum_event_helper::get_notify_event_with_ctx(&ctx.0).await? {
            rbum_event_helper::try_notifies(notify_events, &iam_constants::get_tardis_inst(), &ctx.0).await?;
        }
        TardisResp::ok(resp)
    }
}

/// 续期兑换/吊销接口不使用用户登录态；服务凭据与 task-grants 兑换共用同一部署级服务 token。
#[derive(Clone, Default)]
pub struct IamTokenRenewalRuntimeApi;

#[poem_openapi::OpenApi(prefix_path = "/internal/token-renewals", tag = "bios_basic::ApiTag::Interface")]
impl IamTokenRenewalRuntimeApi {
    #[oai(path = "/:credential_id/exchange", method = "post")]
    async fn exchange(&self, credential_id: Path<String>, req: Json<IamTokenRenewalExchangeReq>, request: &Request) -> TardisApiResult<IamTokenRenewalExchangeResp> {
        let funs = iam_constants::get_tardis_inst();
        require_renewal_service_token(&funs, request)?;
        TardisResp::ok(IamTokenRenewalServ::exchange(&credential_id.0, &req.0, &funs).await?)
    }

    #[oai(path = "/:credential_id", method = "delete")]
    async fn revoke(&self, credential_id: Path<String>, req: Json<IamTokenRenewalRevokeReq>, request: &Request) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        require_renewal_service_token(&funs, request)?;
        IamTokenRenewalServ::revoke(&credential_id.0, &req.0, &funs).await?;
        TardisResp::ok(Void {})
    }
}

/// 服务 token 守卫照抄 IamTaskRuntimeApi：env `BIOS_OAUTH2_TASK_SERVICE_TOKEN` 优先，
/// 否则取 `oauth2_task_service_token` 配置；必须在任何数据库访问之前完成。
fn require_renewal_service_token(funs: &TardisFunsInst, request: &Request) -> TardisResult<()> {
    let configured = std::env::var("BIOS_OAUTH2_TASK_SERVICE_TOKEN").unwrap_or_else(|_| funs.conf::<crate::iam_config::IamConfig>().oauth2_task_service_token.clone());
    validate_task_service_token(&configured, request.headers().get("X-Bios-Task-Service-Token").and_then(|v| v.to_str().ok()))
}

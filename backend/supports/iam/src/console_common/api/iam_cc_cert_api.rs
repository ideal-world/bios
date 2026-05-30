use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use crate::basic::dto::iam_cert_dto::IamCcThirdPartyCertExpiryNotifyResp;
use crate::console_common::serv::iam_cc_cert_expiry_notify_serv::IamCcCertExpiryNotifyServ;
use crate::iam_constants;

#[derive(Clone, Default)]
pub struct IamCcCertApi;

/// Common Console Cert API
/// 通用控制台凭证API
#[poem_openapi::OpenApi(prefix_path = "/cc/cert", tag = "bios_basic::ApiTag::Common")]
impl IamCcCertApi {
    /// [定时脚本] 扫描即将到期的三方凭证，在到期前 14 / 7 / 3 / 1 天向账号绑定的手机号发送短信提醒。
    /// 同一账号同一天仅发送一条；多张凭证同时命中时取剩余天数最小者。
    ///
    /// 短信模板变量：`end_time`（到期时间）、`remaining_days`（剩余天数）、`username`（账号名称）。
    /// 需在配置中设置 `third_party_cert_expiry_reach_msg_signature_id` 与 `third_party_cert_expiry_reach_msg_template_id`。
    #[oai(path = "/script/third-party-expiry-notify", method = "post")]
    async fn third_party_expiry_notify(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamCcThirdPartyCertExpiryNotifyResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamCcCertExpiryNotifyServ::notify_expiring_third_party_certs(&funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// [定时脚本] 扫描今天已到期的三方凭证，向账号绑定的手机号发送短信提醒。
    /// 同一账号同一天仅发送一条；多张凭证同时命中时取 `end_time` 最早者。
    ///
    /// 短信模板变量：`end_time`（到期时间）、`username`（账号名称）。
    /// 需在配置中设置 `third_party_cert_expired_reach_msg_signature_id` 与 `third_party_cert_expired_reach_msg_template_id`。
    #[oai(path = "/script/third-party-expired-today-notify", method = "post")]
    async fn third_party_expired_today_notify(&self, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamCcThirdPartyCertExpiryNotifyResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamCcCertExpiryNotifyServ::notify_expired_today_third_party_certs(&funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}

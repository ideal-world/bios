use crate::iam_config::IamConfig;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

pub struct SmsClient;

impl SmsClient {
    pub async fn send_vcode(phone: &str, vcode: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let conf = funs.conf::<IamConfig>();
        let ctx_base64 = &TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx)?);
        match funs
            .web_client()
            .put_str_to_str(
                &format!("{}/{}/{}/{}", conf.sms_base_url, conf.sms_path, phone, vcode),
                "",
                Some(vec![(
                    TardisFuns::fw_config().web_server.context_conf.context_header_name.to_string(),
                    ctx_base64.to_string(),
                )]),
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(funs.err().unauthorized("send_code", "activate", "send sms error", "403-iam-cert-valid")),
        }
    }

    pub async fn send_pwd(phone: &str, pwd: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let conf = funs.conf::<IamConfig>();
        let ctx_base64 = &TardisFuns::crypto.base64.encode(&TardisFuns::json.obj_to_string(&ctx)?);
        match funs
            .web_client()
            .put_str_to_str(
                &format!("{}/{}/{}/{}", conf.sms_base_url, conf.sms_pwd_path, phone, pwd),
                "",
                Some(vec![(
                    TardisFuns::fw_config().web_server.context_conf.context_header_name.to_string(),
                    ctx_base64.to_string(),
                )]),
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(funs.err().unauthorized("send_pwd", "activate", "send sms error", "403-iam-cert-valid")),
        }
    }
}

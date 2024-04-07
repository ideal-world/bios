use crate::iam_config::IamConfig;
use crate::iam_constants;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::{tokio, TardisFuns, TardisFunsInst};

pub struct SmsClient;

impl SmsClient {
    pub async fn async_send_vcode(phone: &str, vcode: &str, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let phone_clone = phone.to_string();
        let vcode_clone = vcode.to_string();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = iam_constants::get_tardis_inst();
                    let _ = SmsClient::send_vcode(&phone_clone, &vcode_clone, &funs, &ctx_clone).await;
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await
    }

    pub async fn send_vcode(phone: &str, vcode: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let conf = funs.conf::<IamConfig>();
        let ctx_base64 = TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&ctx)?);
        let fw_config = TardisFuns::fw_config();
        let web_server_config = fw_config.web_server();
        let header_name = web_server_config.context_conf.context_header_name.to_string();
        match funs.web_client().put_str_to_str(&format!("{}/{}/{}/{}", conf.sms_base_url, conf.sms_path, phone, vcode), "", vec![(header_name, ctx_base64)]).await {
            Ok(_) => Ok(()),
            Err(_) => Err(funs.err().unauthorized("send_code", "activate", "send sms error", "403-iam-cert-valid")),
        }
    }

    pub async fn async_send_pwd(phone: &str, pwd: &str, _funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let ctx_clone = ctx.clone();
        let phone_clone = phone.to_string();
        let pwd_clone = pwd.to_string();
        ctx.add_async_task(Box::new(|| {
            Box::pin(async move {
                let task_handle = tokio::spawn(async move {
                    let funs = iam_constants::get_tardis_inst();
                    let _ = SmsClient::send_pwd(&phone_clone, &pwd_clone, &funs, &ctx_clone).await;
                });
                task_handle.await.unwrap();
                Ok(())
            })
        }))
        .await?;
        Ok(())
    }

    pub async fn send_pwd(phone: &str, pwd: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let conf = funs.conf::<IamConfig>();
        let ctx_base64 = TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&ctx)?);
        let fw_config = TardisFuns::fw_config();
        let web_server_config = fw_config.web_server();
        match funs
            .web_client()
            .put_str_to_str(
                &format!("{}/{}/{}/{}", conf.sms_base_url, conf.sms_pwd_path, phone, pwd),
                "",
                vec![(web_server_config.context_conf.context_header_name.clone(), ctx_base64.to_string())],
            )
            .await
        {
            Ok(_) => Ok(()),
            Err(_) => Err(funs.err().unauthorized("send_pwd", "activate", "send sms error", "403-iam-cert-valid")),
        }
    }
}

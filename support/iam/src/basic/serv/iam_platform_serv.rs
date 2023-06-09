use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

use crate::basic::dto::iam_cert_conf_dto::{IamCertConfMailVCodeAddOrModifyReq, IamCertConfPhoneVCodeAddOrModifyReq};
use crate::basic::dto::iam_filer_dto::IamConfigFilterReq;
use crate::basic::dto::iam_platform_dto::{IamPlatformConfigReq, IamPlatformConfigResp};
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;

use crate::iam_config::IamConfig;
use crate::iam_enumeration::IamCertKernelKind;

use super::clients::spi_log_client::{LogParamTag, SpiLogClient};
use super::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use super::iam_config_serv::IamConfigServ;

pub struct IamPlatformServ;

impl IamPlatformServ {
    pub async fn modify_platform_config_agg(modify_req: &IamPlatformConfigReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if modify_req.cert_conf_by_user_pwd.is_none() && modify_req.cert_conf_by_phone_vcode.is_none() && modify_req.cert_conf_by_mail_vcode.is_none() && modify_req.config.is_none() {
            return Ok(());
        }

        let mut log_tasks = vec![];
        if modify_req.cert_conf_by_phone_vcode.is_some() {
            log_tasks.push(("修改认证方式为手机号".to_string(), "ModifyCertifiedWay".to_string()));
        }
        if modify_req.cert_conf_by_mail_vcode.is_some() {
            log_tasks.push(("修改认证方式为邮箱".to_string(), "ModifyCertifiedWay".to_string()));
        }
        for (op_describe, op_kind) in log_tasks {
            let _ = SpiLogClient::add_ctx_task(LogParamTag::SecurityAlarm, None, op_describe, Some(op_kind), ctx).await;
        }
        // Init cert conf
        let cert_confs = IamCertServ::find_cert_conf(true, Some("".to_string()), None, None, funs, ctx).await?;

        if let Some(cert_conf_by_user_pwd) = &modify_req.cert_conf_by_user_pwd {
            if let Some(cert_conf_by_user_pwd_id) = cert_confs.iter().find(|r| r.kind == IamCertKernelKind::UserPwd.to_string()).map(|r| r.id.clone()) {
                IamCertUserPwdServ::modify_cert_conf(&cert_conf_by_user_pwd_id, cert_conf_by_user_pwd, funs, ctx).await?;
            }
        }

        if let Some(cert_conf_by_phone_vcode) = modify_req.cert_conf_by_phone_vcode {
            if let Some(cert_conf_by_phone_vcode_id) = cert_confs.iter().find(|r| r.kind == IamCertKernelKind::PhoneVCode.to_string()).map(|r| r.id.clone()) {
                if !cert_conf_by_phone_vcode {
                    IamCertServ::disable_cert_conf(&cert_conf_by_phone_vcode_id, funs, ctx).await?;
                }
            } else if cert_conf_by_phone_vcode {
                IamCertPhoneVCodeServ::add_or_enable_cert_conf(&IamCertConfPhoneVCodeAddOrModifyReq { ak_note: None, ak_rule: None }, Some("".to_string()), funs, ctx).await?;
            }
        }

        if let Some(cert_conf_by_mail_vcode) = modify_req.cert_conf_by_mail_vcode {
            if let Some(cert_conf_by_mail_vcode_id) = cert_confs.iter().find(|r| r.kind == IamCertKernelKind::MailVCode.to_string()).map(|r| r.id.clone()) {
                if !cert_conf_by_mail_vcode {
                    IamCertServ::disable_cert_conf(&cert_conf_by_mail_vcode_id, funs, ctx).await?;
                }
            } else if cert_conf_by_mail_vcode {
                IamCertMailVCodeServ::add_or_enable_cert_conf(&IamCertConfMailVCodeAddOrModifyReq { ak_note: None, ak_rule: None }, Some("".to_string()), funs, ctx).await?;
            }
        }
        if let Some(config) = &modify_req.config {
            IamConfigServ::add_or_modify_batch("", config.to_vec(), funs, ctx).await?;
        }
        Ok(())
    }

    pub async fn get_platform_config_agg(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamPlatformConfigResp> {
        let cert_confs = IamCertServ::find_cert_conf(true, Some("".to_string()), None, None, funs, ctx).await?;
        let cert_conf_by_user_pwd = match cert_confs.iter().find(|r| r.kind == IamCertKernelKind::UserPwd.to_string()) {
            Some(conf) => conf,
            None => {
                return Err(funs.err().not_found("iam_platform_serv", "get_platform_config_agg", "not found cert config", "404-iam-cert-conf-not-exist"));
            }
        };
        let config = IamConfigServ::find_rbums(
            &IamConfigFilterReq {
                rel_item_id: Some("".to_string()),
                ..Default::default()
            },
            Some(true),
            None,
            funs,
            ctx,
        )
        .await?;
        let platform = IamPlatformConfigResp {
            cert_conf_by_user_pwd: TardisFuns::json.str_to_obj(&cert_conf_by_user_pwd.ext)?,
            cert_conf_by_phone_vcode: cert_confs.iter().any(|r| r.kind == IamCertKernelKind::PhoneVCode.to_string()),
            cert_conf_by_mail_vcode: cert_confs.iter().any(|r| r.kind == IamCertKernelKind::MailVCode.to_string()),
            config,
            strict_security_mode: funs.conf::<IamConfig>().strict_security_mode,
        };

        Ok(platform)
    }
}

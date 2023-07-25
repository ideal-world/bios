use bios_basic::process::task_processor::TaskProcessor;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Path;
use tardis::web::poem_openapi::{param::Query, payload::Json};
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_cert_conf_dto::{IamCertConfLdapAddOrModifyReq, IamCertConfLdapResp};
use crate::basic::serv::iam_account_serv::IamAccountServ;
use bios_basic::rbum::dto::rbum_cert_dto::{RbumCertSummaryResp, RbumCertSummaryWithSkResp};
use bios_basic::rbum::dto::rbum_filer_dto::RbumCertFilterReq;
use bios_basic::rbum::helper::rbum_scope_helper::get_max_level_id_by_context;

use crate::basic::dto::iam_cert_dto::{IamCertUserPwdRestReq, IamThirdIntegrationConfigDto, IamThirdIntegrationSyncAddReq, IamThirdIntegrationSyncStatusDto};
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamCertExtKind, IamCertKernelKind};

#[derive(Clone, Default)]
pub struct IamCsCertApi;

/// System Console Cert API
#[poem_openapi::OpenApi(prefix_path = "/cs/cert", tag = "bios_basic::ApiTag::System")]
impl IamCsCertApi {
    /// Rest Password By Account Id
    #[oai(path = "/user-pwd", method = "put")]
    async fn rest_password(
        &self,
        account_id: Query<String>,
        tenant_id: Query<Option<String>>,
        modify_req: Json<IamCertUserPwdRestReq>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Void> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let ctx = IamAccountServ::is_global_account_context(&account_id.0, &funs, &ctx).await?;
        let rbum_cert_conf_id = IamCertServ::get_cert_conf_id_by_kind(IamCertKernelKind::UserPwd.to_string().as_str(), get_max_level_id_by_context(&ctx), &funs).await?;
        IamCertUserPwdServ::reset_sk(&modify_req.0, &account_id.0, &rbum_cert_conf_id, &funs, &ctx).await?;
        funs.commit().await?;
        ctx.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Find Certs By Account Id
    #[oai(path = "/", method = "get")]
    async fn find_certs(&self, account_id: Query<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<Vec<RbumCertSummaryResp>> {
        let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
        let funs = iam_constants::get_tardis_inst();
        let rbum_certs = IamCertServ::find_certs(
            &RbumCertFilterReq {
                rel_rbum_id: Some(account_id.0.to_string()),
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx,
        )
        .await?;
        ctx.execute_task().await?;
        TardisResp::ok(rbum_certs)
    }

    /// Get UserPwd Certs By Account Id
    #[oai(path = "/userpwd", method = "get")]
    async fn get_user_pwd_cert(&self, account_id: Query<String>, tenant_id: Query<Option<String>>, ctx: TardisContextExtractor) -> TardisApiResult<RbumCertSummaryWithSkResp> {
        let funs = iam_constants::get_tardis_inst();
        let resp = IamCertServ::get_kernel_cert(&account_id.0, &IamCertKernelKind::UserPwd, &funs, &ctx.0).await;
        ctx.0.execute_task().await?;
        let rbum_cert = if let Ok(resp) = resp {
            resp
        } else {
            let ctx = IamCertServ::try_use_tenant_ctx(ctx.0, tenant_id.0)?;
            IamCertServ::get_kernel_cert(&account_id.0, &IamCertKernelKind::UserPwd, &funs, &ctx).await?
        };
        TardisResp::ok(rbum_cert)
    }

    /// Delete Cert Conf By Id
    #[oai(path = "/conf/:id", method = "delete")]
    async fn delete_cert_conf(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertServ::delete_cert_conf(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    /// Force Delete Cert And Cert-Conf By Conf Id
    #[oai(path = "/conf/force/:id", method = "delete")]
    async fn delete_cert_and_conf(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Option<String>> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertServ::delete_cert_and_conf_by_conf_id(&id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }

    ///Add Or Modify Sync Config
    #[oai(path = "/sync", method = "put")]
    async fn add_or_modify_sync_third_integration_config(&self, req: Json<Vec<IamThirdIntegrationSyncAddReq>>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = iam_constants::get_tardis_inst();
        IamCertServ::add_or_modify_sync_third_integration_config(req.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void {})
    }

    ///Get Sync Config
    #[oai(path = "/sync", method = "get")]
    async fn get_sync_third_integration_config(&self, ctx: TardisContextExtractor) -> TardisApiResult<Option<Vec<IamThirdIntegrationConfigDto>>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCertServ::get_sync_third_integration_config(&funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    ///Get sync status
    ///
    /// 获取最新的同步状态,如果返回为空，那么就是没有同步记录。
    /// 当total=failed+success被认为同步完成
    #[oai(path = "/sync/:task_id/status", method = "get")]
    async fn get_third_intg_sync_status(&self,task_id: Path<String>) -> TardisApiResult<Option<IamThirdIntegrationSyncStatusDto>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCertServ::get_third_intg_sync_status(&task_id.0,&funs).await?;
        TardisResp::ok(result)
    }

    ///Manual sync
    ///
    /// 手动触发第三方集成同步，如果有其他同步正在进行中，那么就会返回错误。
    #[oai(path = "/sync", method = "post")]
    async fn third_integration_sync(&self, account_sync_from: Json<IamCertExtKind>, ctx: TardisContextExtractor) -> TardisApiResult<Option<String>> {
        let funs = iam_constants::get_tardis_inst();
        IamCertServ::third_integration_sync(
            Some(IamThirdIntegrationConfigDto {
                account_sync_from: account_sync_from.0,
                account_sync_cron: None,
                account_way_to_add: Default::default(),
                account_way_to_delete: Default::default(),
            }),
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        if let Some(task_id) = TaskProcessor::get_task_id_with_ctx(&ctx.0).await? {
            TardisResp::accepted(Some(task_id))
        } else {
            TardisResp::ok(None)
        }
    }
}

#[derive(Clone, Default)]
pub struct IamCsCertConfigLdapApi;
/// System Console Cert Config LDAP API
#[cfg(feature = "ldap_client")]
#[poem_openapi::OpenApi(prefix_path = "/cs/ldap", tag = "bios_basic::ApiTag::System")]
impl IamCsCertConfigLdapApi {
    /// Add Ldap Cert Conf
    #[oai(path = "/", method = "post")]
    async fn add_ldap_cert(&self, add_req: Json<IamCertConfLdapAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let resp = IamCertLdapServ::add_cert_conf(&add_req.0, None, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(resp)
    }
    /// Modify Ldap Cert Conf
    #[oai(path = "/:id", method = "put")]
    async fn modify_ldap_cert(&self, id: Path<String>, modify_req: Json<IamCertConfLdapAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCertLdapServ::modify_cert_conf(&id.0, &modify_req.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
    /// Get Ldap Cert Conf
    #[oai(path = "/", method = "get")]
    async fn get_ldap_cert(&self, ctx: TardisContextExtractor) -> TardisApiResult<Option<IamCertConfLdapResp>> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let resp = IamCertLdapServ::get_cert_conf_by_ctx(&funs, &ctx.0).await?;
        funs.commit().await?;
        TardisResp::ok(resp)
    }
}

use bios_basic::helper::request_helper::try_set_real_ip_from_req_to_ctx;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Json;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountAddByLdapResp, IamAccountBoneResp, IamAccountDetailAggResp, IamAccountExtSysBatchAddReq, IamAccountExtSysResp};
use crate::basic::dto::iam_filer_dto::{IamAccountFilterReq, IamRoleFilterReq};
use crate::basic::serv::iam_account_serv::IamAccountServ;
#[cfg(feature = "ldap_client")]
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRelKind;

#[derive(Clone, Default)]
pub struct IamCcAccountApi;
#[derive(Clone, Default)]
pub struct IamCcAccountLdapApi;

/// Common Console Account API
/// 通用控制台账号API
#[poem_openapi::OpenApi(prefix_path = "/cc/account", tag = "bios_basic::ApiTag::Common")]
impl IamCcAccountApi {
    /// Find Accounts
    /// 查找账号
    #[oai(path = "/", method = "get")]
    #[allow(clippy::too_many_arguments)]
    async fn paginate(
        &self,
        ids: Query<Option<String>>,
        name: Query<Option<String>>,
        role_id: Query<Option<String>>,
        role_code: Query<Option<String>>,
        app_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<TardisPage<IamAccountBoneResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let role_id = if let Some(role_code) = role_code.0 {
            IamRoleServ::find_id_items(
                &IamRoleFilterReq {
                    basic: RbumBasicFilterReq {
                        code: Some(role_code),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                None,
                None,
                &funs,
                &ctx.0,
            )
            .await?
            .pop()
        } else if let Some(role_id) = role_id.0 {
            Some(role_id)
        } else {
            None
        };
        let rel = role_id.map(|role_id| RbumItemRelFilterReq {
            rel_by_from: true,
            tag: Some(IamRelKind::IamAccountRole.to_string()),
            from_rbum_kind: Some(RbumRelFromKind::Item),
            rel_item_id: Some(role_id),
            // todo 开放人员的 own_paths 限制
            // own_paths: Some(ctx.0.clone().own_paths),
            ..Default::default()
        });
        let rel2 = app_id.0.map(|app_id| RbumItemRelFilterReq {
            rel_by_from: true,
            tag: Some(IamRelKind::IamAccountApp.to_string()),
            from_rbum_kind: Some(RbumRelFromKind::Item),
            rel_item_id: Some(app_id),
            // todo 开放人员的 own_paths 限制
            // own_paths: Some(ctx.0.clone().own_paths),
            ..Default::default()
        });
        let result = IamAccountServ::paginate_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ids: ids.0.map(|ids| ids.split(',').map(|id| id.to_string()).collect::<Vec<String>>()),
                    name: name.0,
                    enabled: Some(true),
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    ..Default::default()
                },
                rel,
                rel2,
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(TardisPage {
            page_size: result.page_size,
            page_number: result.page_number,
            total_size: result.total_size,
            records: result
                .records
                .into_iter()
                .map(|item| IamAccountBoneResp {
                    id: item.id,
                    name: item.name,
                    icon: item.icon,
                })
                .collect(),
        })
    }

    /// Get Account
    /// 获取账号
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<IamAccountDetailAggResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
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
            false,
            true,
            false,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Account Name By Ids
    /// 根据账号ID查找账号名称
    ///
    /// Return format: ["<id>,<name>,<icon>"]
    /// 返回格式：["<id>,<name>,<icon>"]
    #[oai(path = "/name", method = "get")]
    async fn find_name_by_ids(
        &self,
        // Account Ids, multiple ids separated by ,
        // 账号ID，多个ID以,分隔
        ids: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ids = ids.0.split(',').map(|s| s.to_string()).collect();
        let result = IamAccountServ::find_name_by_ids(ids, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Account online By Ids
    /// 根据账号ID查找账号在线状态
    ///
    /// Return format: ["<id>,<online -> true or false>"]
    /// 返回格式：["<id>,<online -> true or false>"]
    #[oai(path = "/online", method = "get")]
    async fn find_account_online_by_ids(
        &self,
        // Account Ids, multiple ids separated by ,
        // 账号ID，多个ID以,分隔
        ids: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ids = ids.0.split(',').map(|s| s.to_string()).collect();
        let result = IamAccountServ::find_account_online_by_ids(ids, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Find Account lock state By Ids
    /// 根据账号ID查找账号锁定状态
    ///
    /// Return format: ["<id>,<state>"]
    /// 返回格式：["<id>,<state>"]
    #[oai(path = "/lock/state", method = "get")]
    async fn find_account_lock_state_by_ids(
        &self,
        // Account Ids, multiple ids separated by ,
        // 账号ID，多个ID以,分隔
        ids: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<String>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let ids = ids.0.split(',').map(|s| s.to_string()).collect();
        let result = IamAccountServ::find_account_lock_state_by_ids(ids, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}

/// Common Console Account LDAP API
/// 通用控制台账号LDAP API
#[cfg(feature = "ldap_client")]
#[poem_openapi::OpenApi(prefix_path = "/cc/account/ldap", tag = "bios_basic::ApiTag::Common")]
impl IamCcAccountLdapApi {
    /// Find Accounts by LDAP
    /// 根据LDAP查找账号
    #[oai(path = "/", method = "get")]
    async fn find_from_ldap(
        &self,
        name: Query<String>,
        tenant_id: Query<Option<String>>,
        code: Query<String>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<Vec<IamAccountExtSysResp>> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let funs = iam_constants::get_tardis_inst();
        let result = IamCertLdapServ::search_accounts(&name.0, tenant_id.0, &code.0, &funs, &ctx.0).await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }

    /// Add Account by LDAP
    /// 通过LDAP添加账号
    #[oai(path = "/", method = "put")]
    async fn add_account_from_ldap(
        &self,
        add_req: Json<IamAccountExtSysBatchAddReq>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
        request: &Request,
    ) -> TardisApiResult<IamAccountAddByLdapResp> {
        try_set_real_ip_from_req_to_ctx(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamCertLdapServ::batch_get_or_add_account_without_verify(add_req.0, tenant_id.0, &funs, &ctx.0).await?;
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}

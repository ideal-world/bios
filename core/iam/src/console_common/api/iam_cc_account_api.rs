use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::web::Json;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_account_dto::{IamAccountAddByLdapResp, IamAccountBoneResp, IamAccountExtSysBatchAddReq, IamAccountExtSysResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_account_serv::IamAccountServ;
#[cfg(feature = "ldap_client")]
use crate::basic::serv::iam_cert_ldap_serv::IamCertLdapServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRelKind;

pub struct IamCcAccountApi;
pub struct IamCcAccountLdapApi;

/// Common Console Account API
#[poem_openapi::OpenApi(prefix_path = "/cc/account", tag = "bios_basic::ApiTag::Common")]
impl IamCcAccountApi {
    /// Find Accounts
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        role_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamAccountBoneResp>> {
        let funs = iam_constants::get_tardis_inst();
        let rel = role_id.0.map(|role_id| RbumItemRelFilterReq {
            rel_by_from: true,
            tag: Some(IamRelKind::IamAccountRole.to_string()),
            from_rbum_kind: Some(RbumRelFromKind::Item),
            rel_item_id: Some(role_id),
            ..Default::default()
        });
        let result = IamAccountServ::paginate_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    enabled: Some(true),
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    ..Default::default()
                },
                rel,
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

    /// Find Account Name By Ids
    ///
    /// Return format: ["<id>,<name>"]
    #[oai(path = "/name", method = "get")]
    async fn find_name_by_ids(
        &self,
        // Account Ids, multiple ids separated by ,
        ids: Query<String>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<String>> {
        let funs = iam_constants::get_tardis_inst();
        let ids = ids.0.split(',').map(|s| s.to_string()).collect();
        let result = IamAccountServ::find_name_by_ids(ids, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }
}

/// Common Console Account LDAP API
#[cfg(feature = "ldap_client")]
#[poem_openapi::OpenApi(prefix_path = "/cc/account/ldap", tag = "bios_basic::ApiTag::Common")]
impl IamCcAccountLdapApi {
    /// Find Accounts by LDAP
    #[oai(path = "/", method = "get")]
    async fn find_from_ldap(
        &self,
        name: Query<String>,
        tenant_id: Query<Option<String>>,
        code: Query<String>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<Vec<IamAccountExtSysResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCertLdapServ::search_accounts(&name.0, tenant_id.0, &code.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }

    /// Add Account by LDAP
    #[oai(path = "/", method = "put")]
    async fn add_account_from_ldap(
        &self,
        add_req: Json<IamAccountExtSysBatchAddReq>,
        tenant_id: Query<Option<String>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<IamAccountAddByLdapResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamCertLdapServ::batch_get_or_add_account_without_verify(add_req.0, tenant_id.0, &funs, &ctx.0).await?;
        TardisResp::ok(result)
    }
}

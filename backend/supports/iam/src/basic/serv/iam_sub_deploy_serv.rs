use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use bios_basic::rbum::dto::rbum_cert_dto::RbumCertSummaryWithSkResp;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq, RbumSetCateFilterReq, RbumSetFilterReq, RbumSetItemFilterReq};
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggAddReq;
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelAddReq;
use bios_basic::rbum::dto::rbum_set_cate_dto::RbumSetCateDetailResp;
use bios_basic::rbum::dto::rbum_set_dto::{RbumSetAddReq, RbumSetDetailResp};
use bios_basic::rbum::rbum_enumeration::{RbumRelFromKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use bios_basic::rbum::serv::rbum_set_serv::{RbumSetCateServ, RbumSetItemServ, RbumSetServ};
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm::prelude::Expr;
use tardis::db::sea_orm::sea_query::{Query, SelectStatement};
use tardis::db::sea_orm::{self, *};
use tardis::{chrono, serde_json, TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::{iam_sub_deploy, iam_sub_deploy_host, iam_sub_deploy_license};
use crate::basic::dto::iam_account_dto::{IamAccountAddReq, IamAccountDetailResp, IamAccountModifyReq};
use crate::basic::dto::iam_app_dto::{IamAppAddReq, IamAppKind, IamAppModifyReq};
use crate::basic::dto::iam_config_dto::{IamConfigAggOrModifyReq, IamConfigDetailResp};
use crate::basic::dto::iam_filer_dto::{
    IamAccountFilterReq, IamAppFilterReq, IamConfigFilterReq, IamResFilterReq, IamRoleFilterReq, IamSubDeployFilterReq, IamSubDeployHostFilterReq, IamSubDeployLicenseFilterReq,
};
use crate::basic::dto::iam_res_dto::{IamResAddReq, IamResAggAddReq, IamResDetailResp, IamResModifyReq};
use crate::basic::dto::iam_role_dto::IamRoleAddReq;
use crate::basic::dto::iam_set_dto::{IamSetItemAddReq, IamSetItemAggAddReq};
use crate::basic::dto::iam_sub_deploy_dto::{
    IamSubDeployAddReq, IamSubDeployDetailResp, IamSubDeployModifyReq, IamSubDeployOneExportAggResp, IamSubDeployOneImportReq, IamSubDeploySummaryResp,
    IamSubDeployTowExportAggResp, IamSubDeployTowImportReq,
};
use crate::basic::dto::iam_sub_deploy_host_dto::{IamSubDeployHostAddReq, IamSubDeployHostDetailResp, IamSubDeployHostModifyReq};
use crate::basic::dto::iam_sub_deploy_license_dto::{IamSubDeployLicenseAddReq, IamSubDeployLicenseDetailResp, IamSubDeployLicenseModifyReq};
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::iam_config::{IamBasicInfoManager, IamConfig};
use crate::iam_constants::RBUM_ITEM_ID_SUB_ROLE_LEN;
use crate::iam_enumeration::{IamAccountLogoutTypeKind, IamCertKernelKind, IamConfigDataTypeKind, IamConfigKind, IamRelKind, IamRoleKind, IamSetKind, IamSubDeployHostKind};

use super::clients::iam_search_client::IamSearchClient;
use super::iam_account_serv::IamAccountServ;
use super::iam_app_serv::IamAppServ;
use super::iam_cert_serv::IamCertServ;
use super::iam_config_serv::IamConfigServ;
use super::iam_res_serv::IamResServ;
use super::iam_role_serv::IamRoleServ;
use super::iam_set_serv::IamSetServ;

pub struct IamSubDeployServ;
pub struct IamSubDeployHostServ;
pub struct IamSubDeployLicenseServ;

#[async_trait]
impl RbumItemCrudOperation<iam_sub_deploy::ActiveModel, IamSubDeployAddReq, IamSubDeployModifyReq, IamSubDeploySummaryResp, IamSubDeployDetailResp, IamSubDeployFilterReq>
    for IamSubDeployServ
{
    fn get_ext_table_name() -> &'static str {
        iam_sub_deploy::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.kind_sub_deploy_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone()))
    }

    async fn package_item_add(add_req: &IamSubDeployAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        let id = if let Some(extend_sub_deploy_id) = &add_req.extend_sub_deploy_id {
            if extend_sub_deploy_id.is_empty() {
                None
            } else {
                Some(TrimString::from(format!(
                    "{}:{}",
                    extend_sub_deploy_id.clone(),
                    TardisFuns::field.nanoid_len(RBUM_ITEM_ID_SUB_ROLE_LEN as usize)
                )))
            }
        } else {
            None
        };
        Ok(RbumItemKernelAddReq {
            id,
            code: None,
            name: add_req.name.clone().unwrap_or(TrimString::from("")),
            scope_level: add_req.scope_level.clone(),
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamSubDeployAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_sub_deploy::ActiveModel> {
        Ok(iam_sub_deploy::ActiveModel {
            id: Set(id.to_string()),
            province: Set(add_req.province.clone()),
            access_url: Set(add_req.access_url.clone().unwrap_or("".to_string())),
            note: Set(add_req.note.clone().unwrap_or("".to_string())),
            extend_sub_deploy_id: Set(add_req.extend_sub_deploy_id.clone().unwrap_or("".to_string())),
            ..Default::default()
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamSubDeployModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
        if modify_req.name.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemKernelModifyReq {
            code: None,
            name: modify_req.name.clone(),
            scope_level: modify_req.scope_level.clone(),
            disabled: modify_req.disabled,
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &IamSubDeployModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<iam_sub_deploy::ActiveModel>> {
        if modify_req.province.is_none() && modify_req.access_url.is_none() && modify_req.note.is_none() {
            return Ok(None);
        }
        let mut iam_sub_deploy = iam_sub_deploy::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(province) = &modify_req.province {
            iam_sub_deploy.province = Set(province.to_string());
        }
        if let Some(access_url) = &modify_req.access_url {
            iam_sub_deploy.access_url = Set(access_url.to_string());
        }
        if let Some(note) = &modify_req.note {
            iam_sub_deploy.note = Set(note.to_string());
        }
        Ok(Some(iam_sub_deploy))
    }

    async fn before_delete_item(id: &str, funs: &TardisFunsInst, _ctx: &TardisContext) -> TardisResult<Option<IamSubDeployDetailResp>> {
        if funs
            .db()
            .count(Query::select().column(iam_sub_deploy::Column::Id).from(iam_sub_deploy::Entity).and_where(Expr::col(iam_sub_deploy::Column::ExtendSubDeployId).eq(id)))
            .await?
            > 0
        {
            return Err(funs.err().conflict(
                &Self::get_obj_name(),
                "delete",
                &format!("can not delete {}.{} when there are associated by extend_sub_deploy_id", Self::get_obj_name(), id),
                "409-iam-sub-deploy-delete-conflict",
            ));
        }
        Ok(None)
    }
    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamSubDeployFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_sub_deploy::Entity, iam_sub_deploy::Column::Province));
        query.column((iam_sub_deploy::Entity, iam_sub_deploy::Column::AccessUrl));
        query.column((iam_sub_deploy::Entity, iam_sub_deploy::Column::Note));
        query.column((iam_sub_deploy::Entity, iam_sub_deploy::Column::ExtendSubDeployId));
        if let Some(province) = &filter.province {
            query.and_where(Expr::col(iam_sub_deploy::Column::Province).eq(province));
        }
        if let Some(access_url) = &filter.access_url {
            query.and_where(Expr::col(iam_sub_deploy::Column::AccessUrl).like(access_url));
        }
        if let Some(extend_sub_deploy_id) = &filter.extend_sub_deploy_id {
            query.and_where(Expr::col(iam_sub_deploy::Column::ExtendSubDeployId).eq(extend_sub_deploy_id));
        } else {
            query.and_where(Expr::col(iam_sub_deploy::Column::ExtendSubDeployId).ne(""));
        }
        Ok(())
    }
}

impl IamSubDeployServ {
    pub async fn delete_item_with_ext_rel(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let iam_sub_deploy_host_ids = IamSubDeployHostServ::find_id_rbums(
            &IamSubDeployHostFilterReq {
                basic: Default::default(),
                sub_deploy_id: Some(id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for iam_sub_deploy_host_id in iam_sub_deploy_host_ids {
            IamSubDeployHostServ::delete_rbum(&iam_sub_deploy_host_id, funs, ctx).await?;
        }
        let iam_sub_deploy_license_ids = IamSubDeployLicenseServ::find_id_rbums(
            &IamSubDeployLicenseFilterReq {
                basic: Default::default(),
                sub_deploy_id: Some(id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        for iam_sub_deploy_license_id in iam_sub_deploy_license_ids {
            IamSubDeployLicenseServ::delete_rbum(&iam_sub_deploy_license_id, funs, ctx).await?;
        }
        Self::delete_item_with_all_rels(id, funs, ctx).await?;
        Ok(())
    }

    pub(crate) async fn add_rel_account(id: &str, account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::add_simple_rel(&IamRelKind::IamSubDeployAccount, id, account_id, None, None, true, false, funs, ctx).await
    }

    pub(crate) async fn delete_rel_account(id: &str, account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamSubDeployAccount, id, account_id, funs, ctx).await
    }

    pub(crate) async fn add_rel_auth_account(id: &str, account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::add_simple_rel(&IamRelKind::IamSubDeployAuthAccount, id, account_id, None, None, true, false, funs, ctx).await
    }

    pub(crate) async fn delete_rel_auth_account(id: &str, account_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamSubDeployAuthAccount, id, account_id, funs, ctx).await
    }

    pub(crate) async fn add_rel_org(id: &str, org_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::add_simple_rel(&IamRelKind::IamSubDeployOrg, id, org_id, None, None, true, true, funs, ctx).await
    }

    pub(crate) async fn delete_rel_org(id: &str, org_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamSubDeployOrg, id, org_id, funs, ctx).await
    }

    pub(crate) async fn add_rel_apps(id: &str, apps_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::add_simple_rel(&IamRelKind::IamSubDeployApps, id, apps_id, None, None, true, true, funs, ctx).await
    }

    pub(crate) async fn delete_rel_apps(id: &str, apps_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamSubDeployApps, id, apps_id, funs, ctx).await
    }

    pub(crate) async fn add_rel_other(id: &str, apps_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::add_simple_rel(&IamRelKind::IamSubDeployRel, id, apps_id, None, None, true, true, funs, ctx).await
    }

    pub(crate) async fn delete_rel_other(id: &str, apps_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamSubDeployRel, id, apps_id, funs, ctx).await
    }

    pub(crate) async fn exist_sub_deploy_rels(rel_kind: &IamRelKind, id: &str, to_rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        IamRelServ::exist_rels(rel_kind, id, to_rbum_item_id, funs, ctx).await
    }

    pub(crate) async fn exist_to_rel(rel_kind: &IamRelKind, to_rbum_item_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<bool> {
        IamRelServ::exist_to_rel(rel_kind, to_rbum_item_id, funs, ctx).await
    }

    pub(crate) async fn find_rel_id_by_sub_deploy_id(rel_kind: &IamRelKind, id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        IamRelServ::find_from_id_rels(rel_kind, true, id, None, None, funs, ctx).await
    }

    pub(crate) async fn find_sub_deploy_id_by_rel_id(rel_kind: &IamRelKind, id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Vec<String>> {
        IamRelServ::find_to_id_rels(rel_kind, id, None, None, funs, ctx).await
    }

    pub(crate) async fn one_deploy_export(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamSubDeployOneExportAggResp> {
        let sub_deploy = Self::get_item(
            id,
            &IamSubDeployFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;

        let accounts = IamAccountServ::find_detail_items(
            &IamAccountFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel: Some(RbumItemRelFilterReq {
                    rel_by_from: false,
                    tag: Some(IamRelKind::IamSubDeployAccount.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    rel_item_ids: Some(vec![sub_deploy.id.to_string()]),
                    // own_paths: Some(ctx.own_paths.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        let account_ids = accounts.iter().map(|account| account.id.clone()).collect::<Vec<_>>();

        let (orgs_set, orgs_set_cate) = Self::export_orgs(id, funs, ctx).await?;
        let (apps_set, apps_set_cate) = Self::export_apps(id, funs, ctx).await?;
        let (res_set, res_set_cate, res_items, res_set_item, res_api_map, res_role_map) = Self::export_res(funs, ctx).await?;
        Ok(IamSubDeployOneExportAggResp {
            accounts: Some(accounts),
            account_cert: Some(Self::export_account_cert(account_ids.clone(), funs, ctx).await?),
            iam_config: Some(Self::export_iam_config(funs, ctx).await?),
            account_role: Some(Self::export_account_tenant_role(account_ids.clone(), funs, ctx).await?),
            account_org: Some(Self::export_account_org(account_ids.clone(), funs, ctx).await?),
            account_apps: Some(Self::export_account_apps(account_ids.clone(), funs, ctx).await?),
            res_set: Some(res_set),
            res_set_cate: Some(res_set_cate),
            res_set_item: Some(res_set_item),
            res_items: Some(res_items),
            res_api: Some(res_api_map),
            res_role: Some(res_role_map),
            org_set: Some(orgs_set),
            org_set_cate: Some(orgs_set_cate),
            apps_set: Some(apps_set),
            apps_set_cate: Some(apps_set_cate),
        })
    }

    async fn export_res(
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<(
        RbumSetDetailResp,
        Vec<RbumSetCateDetailResp>,
        Vec<IamResDetailResp>,
        HashMap<String, Vec<String>>,
        HashMap<String, Vec<String>>,
        HashMap<String, Vec<String>>,
    )> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        let res_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Res, &funs, &global_ctx).await?;
        let res_set = RbumSetServ::get_rbum(
            &res_set_id,
            &RbumSetFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            &global_ctx,
        )
        .await?;
        let res_set_cate = RbumSetCateServ::find_detail_rbums(
            &RbumSetCateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_rbum_set_id: Some(res_set_id.clone()),
                ..Default::default()
            },
            None,
            None,
            funs,
            &global_ctx,
        )
        .await?;

        let res_items = IamResServ::find_detail_items(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            Some(false),
            None,
            funs,
            &global_ctx,
        )
        .await?;
        let mut res_role_map: HashMap<String, Vec<String>> = HashMap::new();
        let mut res_api_map = HashMap::new();
        let mut res_set_item = HashMap::new();
        for res in res_items.clone().into_iter() {
            let apis_res_id = IamResServ::find_to_id_rel_roles(&IamRelKind::IamResApi, &res.id.clone(), None, None, funs, &global_ctx).await?;
            res_api_map.insert(res.id.clone(), apis_res_id);
            let roles = IamResServ::find_from_id_rel_roles(&IamRelKind::IamResRole, true, &res.id.clone(), None, None, funs, &global_ctx).await?;
            res_role_map.insert(res.id.clone(), roles);
            let res_cate_ids = RbumSetItemServ::find_rbums(
                &RbumSetItemFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    rel_rbum_set_id: Some(res_set_id.clone()),
                    rel_rbum_item_ids: Some(vec![res.id.to_string()]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                &global_ctx,
            )
            .await?
            .iter()
            .map(|org| org.rel_rbum_set_cate_id.clone().unwrap_or_default())
            .collect::<Vec<_>>();
            res_set_item.insert(res.id.clone(), res_cate_ids);
        }
        Ok((res_set, res_set_cate, res_items, res_set_item, res_api_map, res_role_map))
    }

    async fn export_apps(sub_deploy_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<(RbumSetDetailResp, Vec<RbumSetCateDetailResp>)> {
        let sub_deploy_apps_ids = Self::find_rel_id_by_sub_deploy_id(&IamRelKind::IamSubDeployApps, sub_deploy_id, funs, ctx).await?;

        let apps_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let apps_set = RbumSetServ::get_rbum(
            &apps_set_id,
            &RbumSetFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let apps_auth = RbumSetCateServ::find_detail_rbums(
            &RbumSetCateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ids: Some(sub_deploy_apps_ids),
                    ..Default::default()
                },
                rel_rbum_set_id: Some(apps_set_id.clone()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        let apps_auth_sys_code = apps_auth.iter().map(|app| app.sys_code.clone()).collect::<Vec<_>>();
        let last_apps_cate = RbumSetCateServ::find_detail_rbums(
            &RbumSetCateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                sys_codes: Some(apps_auth_sys_code),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndParent),
                rel_rbum_set_id: Some(apps_set_id),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        Ok((apps_set, last_apps_cate))
    }

    async fn export_orgs(sub_deploy_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<(RbumSetDetailResp, Vec<RbumSetCateDetailResp>)> {
        let sub_deploy_org_ids: Vec<String> = Self::find_rel_id_by_sub_deploy_id(&IamRelKind::IamSubDeployOrg, sub_deploy_id, funs, ctx).await?;

        let org_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        let org_set = RbumSetServ::get_rbum(
            &org_set_id,
            &RbumSetFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let org_auth = RbumSetCateServ::find_detail_rbums(
            &RbumSetCateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ids: Some(sub_deploy_org_ids),
                    ..Default::default()
                },
                rel_rbum_set_id: Some(org_set_id.clone()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        let org_auth_sys_code = org_auth.iter().map(|app| app.sys_code.clone()).collect::<Vec<_>>();
        let last_org_cate = RbumSetCateServ::find_detail_rbums(
            &RbumSetCateFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                sys_codes: Some(org_auth_sys_code),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndParent),
                rel_rbum_set_id: Some(org_set_id),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        Ok((org_set, last_org_cate))
    }

    async fn export_iam_config(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, Vec<IamConfigDetailResp>>> {
        let mut iam_conifg = HashMap::new();
        let base_iam_config = IamConfigServ::find_detail_rbums(
            &IamConfigFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_item_id: Some("".to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        iam_conifg.insert("".to_string(), base_iam_config);
        let tenant_iam_config = IamConfigServ::find_detail_rbums(
            &IamConfigFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                rel_item_id: Some(ctx.own_paths.clone()),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        iam_conifg.insert(ctx.own_paths.clone(), tenant_iam_config);
        Ok(iam_conifg)
    }

    async fn export_account_cert(account_ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, Vec<RbumCertSummaryWithSkResp>>> {
        let mut account_cert = HashMap::new();
        for account_id in account_ids {
            let mut certs = vec![];
            let pwd_cert = IamCertServ::get_kernel_cert(&account_id, &IamCertKernelKind::UserPwd, funs, ctx).await;
            if let Ok(pwd_cert) = pwd_cert {
                certs.push(pwd_cert);
            }
            let phone_cert = IamCertServ::get_kernel_cert(&account_id, &IamCertKernelKind::PhoneVCode, funs, ctx).await;
            if let Ok(phone_cert) = phone_cert {
                certs.push(phone_cert);
            }
            let email_cert = IamCertServ::get_kernel_cert(&account_id, &IamCertKernelKind::MailVCode, funs, ctx).await;
            if let Ok(email_cert) = email_cert {
                certs.push(email_cert);
            }
            account_cert.insert(account_id.to_string(), certs);
        }
        Ok(account_cert)
    }

    async fn export_account_tenant_role(account_ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, Vec<String>>> {
        let mut account_role = HashMap::new();
        for account_id in account_ids {
            let role_ids = IamRoleServ::find_id_items(
                &&IamRoleFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    kind: Some(IamRoleKind::Tenant),
                    in_base: Some(false),
                    in_embed: Some(true),
                    rel: Some(RbumItemRelFilterReq {
                        rel_by_from: false,
                        tag: Some(IamRelKind::IamAccountRole.to_string()),
                        from_rbum_kind: Some(RbumRelFromKind::Item),
                        rel_item_ids: Some(vec![account_id.to_string()]),
                        own_paths: Some(ctx.own_paths.clone()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?;
            account_role.insert(account_id.to_string(), role_ids);
        }
        Ok(account_role)
    }

    async fn export_account_org(account_ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, Vec<String>>> {
        let mut account_role = HashMap::new();
        let org_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Org, &funs, &ctx).await?;
        for account_id in account_ids {
            let org_ids = RbumSetItemServ::find_rbums(
                &RbumSetItemFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    rel_rbum_set_id: Some(org_set_id.clone()),
                    rel_rbum_item_ids: Some(vec![account_id.to_string()]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
            .iter()
            .map(|org| org.rel_rbum_set_cate_id.clone().unwrap_or_default())
            .collect::<Vec<_>>();
            account_role.insert(account_id.to_string(), org_ids);
        }
        Ok(account_role)
    }

    async fn export_account_apps(account_ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, Vec<String>>> {
        let mut account_role = HashMap::new();
        let apps_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;

        for account_id in account_ids {
            let apps_ids = RbumSetItemServ::find_rbums(
                &RbumSetItemFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    rel_rbum_set_id: Some(apps_set_id.clone()),
                    rel_rbum_item_ids: Some(vec![account_id.to_string()]),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
            .iter()
            .map(|apps| apps.rel_rbum_set_cate_id.clone().unwrap_or_default())
            .collect::<Vec<_>>();
            account_role.insert(account_id.to_string(), apps_ids);
        }
        Ok(account_role)
    }

    pub(crate) async fn one_deploy_import(import_req: IamSubDeployOneImportReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let app_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        // 同步项目
        if let Some(apps) = import_req.app {
            for app in apps {
                let app_ctx = IamCertServ::use_app_ctx(ctx.clone(), &app.id)?;
                if IamAppServ::count_items(
                    &IamAppFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ids: Some(vec![app.id.clone()]),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    ctx,
                )
                .await?
                    > 0
                {
                    let _ = IamAppServ::modify_item(
                        &app.id,
                        &mut IamAppModifyReq {
                            name: Some(TrimString::from(app.name.clone())),
                            description: app.description.clone(),
                            scope_level: Some(app.scope_level.clone()),
                            disabled: Some(app.disabled.clone()),
                            icon: Some(app.icon.clone()),
                            sort: Some(app.sort.clone()),
                            contact_phone: Some(app.contact_phone.clone()),
                        },
                        funs,
                        &app_ctx,
                    )
                    .await;
                } else {
                    let _ = IamAppServ::add_item(
                        &mut IamAppAddReq {
                            id: Some(TrimString::from(app.id.clone())),
                            name: TrimString::from(app.name),
                            description: app.description,
                            scope_level: Some(app.scope_level),
                            disabled: Some(app.disabled),
                            icon: Some(app.icon),
                            sort: Some(app.sort),
                            contact_phone: Some(app.contact_phone),
                            kind: Some(app.kind.clone()),
                            sync_apps_group: Some(app.kind == IamAppKind::Product),
                        },
                        funs,
                        &app_ctx,
                    )
                    .await;
                }
                // 同步项目的项目组
                if let Some(app_apps) = import_req.app_apps.clone() {
                    if let Some(app_set_cate) = app_apps.get(&app.id) {
                        let set_items = IamSetServ::find_set_items(Some(app_set_id.clone()), None, Some(app.id.to_owned()), None, true, Some(true), funs, &ctx).await?;
                        for set_item in set_items {
                            let _ = IamSetServ::delete_set_item(&set_item.id, funs, &ctx).await;
                        }
                        for cate_id in app_set_cate {
                            let _ = IamSetServ::add_set_item(
                                &IamSetItemAddReq {
                                    set_id: app_set_id.clone(),
                                    set_cate_id: cate_id.to_string(),
                                    sort: 0,
                                    rel_rbum_item_id: app.id.to_string(),
                                },
                                funs,
                                &ctx,
                            )
                            .await;
                        }
                    }
                }
                // 同步项目的用户
                if let Some(app_account) = import_req.app_account.clone() {
                    if let Some(account_ids) = app_account.get(&app.id) {
                        for account in IamAppServ::find_rel_account(&app.id, funs, ctx).await? {
                            let _ = IamRelServ::delete_simple_rel(&IamRelKind::IamAccountApp, &account.rel_id, &app.id, funs, &app_ctx).await;
                        }
                        for account_id in account_ids {
                            let _ = IamRelServ::add_simple_rel(&IamRelKind::IamAccountApp, account_id, &app.id, None, None, true, false, funs, &app_ctx).await;
                        }
                    }
                }
                // 同步项目的角色
                if let Some(app_role) = import_req.app_role.clone() {
                    // todo 删除不存在的角色
                    if let Some(app_role_vec) = app_role.get(&app.id) {
                        for role in app_role_vec {
                            if IamRoleServ::count_items(
                                &IamRoleFilterReq {
                                    basic: RbumBasicFilterReq {
                                        with_sub_own_paths: true,
                                        ids: Some(vec![role.id.clone()]),
                                        ..Default::default()
                                    },
                                    kind: Some(IamRoleKind::App),
                                    ..Default::default()
                                },
                                funs,
                                &app_ctx,
                            )
                            .await?
                                == 0
                            {
                                let _ = IamRoleServ::add_item(
                                    &mut IamRoleAddReq {
                                        id: Some(TrimString::from(role.id.clone())),
                                        code: Some(TrimString::from(role.code.clone())),
                                        name: TrimString::from(role.name.clone()),
                                        kind: Some(role.kind.clone()),
                                        scope_level: Some(role.scope_level.clone()),
                                        disabled: Some(role.disabled.clone()),
                                        icon: Some(role.icon.clone()),
                                        sort: Some(role.sort.clone()),
                                        extend_role_id: Some(role.extend_role_id.clone()),
                                        in_embed: Some(role.in_embed.clone()),
                                        in_base: Some(role.in_base.clone()),
                                    },
                                    funs,
                                    &app_ctx,
                                )
                                .await;
                            }
                            // 同步角色的用户
                            if let Some(app_role_account) = import_req.app_role_account.clone() {
                                for account in IamAppServ::find_rel_account(&app.id, funs, ctx).await? {
                                    IamRelServ::delete_simple_rel(&IamRelKind::IamAccountRole, &account.rel_id, &role.id, funs, &app_ctx).await?;
                                }
                                if let Some(role_account_ids) = app_role_account.get(&role.id) {
                                    for account_id in role_account_ids {
                                        let _ = IamRelServ::add_simple_rel(&IamRelKind::IamAccountRole, account_id, &role.id, None, None, true, false, funs, &app_ctx).await;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // 同步项目绑定工作流模板
        if let Some(app_flow_template) = import_req.app_flow_template.clone() {
            for (app_id, rel_template_id) in app_flow_template {
                let app_ctx = IamCertServ::use_app_ctx(ctx.clone(), &app_id)?;
                if !rel_template_id.is_empty()
                    && RbumRelServ::find_from_simple_rels("FlowAppTemplate", &RbumRelFromKind::Item, true, &app_id, None, None, funs, &app_ctx).await?.is_empty()
                {
                    RbumRelServ::add_rel(
                        &mut RbumRelAggAddReq {
                            rel: RbumRelAddReq {
                                tag: "FlowAppTemplate".to_string(),
                                note: None,
                                from_rbum_kind: RbumRelFromKind::Item,
                                from_rbum_id: app_id.clone(),
                                to_rbum_item_id: rel_template_id.clone(),
                                to_own_paths: app_ctx.own_paths.clone(),
                                ext: None,
                                to_is_outside: true,
                                disabled: None,
                            },
                            attrs: vec![],
                            envs: vec![],
                        },
                        funs,
                        &app_ctx,
                    )
                    .await?;
                }
            }
        }
        Ok(())
    }

    pub(crate) async fn sub_deploy_export(
        _start_time: Option<DateTime<Utc>>,
        _end_time: Option<DateTime<Utc>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<IamSubDeployTowExportAggResp> {
        let mut app_apps = HashMap::new();
        let mut app_role = HashMap::new();
        let mut app_account = HashMap::new();
        let mut app_role_account = HashMap::new();
        let app_set_id = IamSetServ::get_default_set_id_by_ctx(&IamSetKind::Apps, &funs, &ctx).await?;
        let app_vec = IamAppServ::find_detail_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            &funs,
            &ctx,
        )
        .await?;
        for app in app_vec.clone() {
            let app_ctx = IamCertServ::use_app_ctx(ctx.clone(), &app.id)?;
            let app_id = app.id.clone();
            let app_set_cate = RbumSetItemServ::find_detail_rbums(
                &RbumSetItemFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    rel_rbum_item_ids: Some(vec![app_id.to_string()]),
                    rel_rbum_set_id: Some(app_set_id.clone()),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?
            .iter()
            .map(|cate| cate.rel_rbum_set_cate_id.clone().unwrap_or_default())
            .collect::<Vec<_>>();
            app_apps.insert(app_id.clone(), app_set_cate);
            let app_role_vec = IamRoleServ::find_detail_items(
                &IamRoleFilterReq {
                    basic: RbumBasicFilterReq { ..Default::default() },
                    kind: Some(IamRoleKind::App),
                    ..Default::default()
                },
                None,
                None,
                funs,
                &app_ctx,
            )
            .await?;
            app_role.insert(app_id.clone(), app_role_vec.clone());
            for role in app_role_vec {
                let role_account_ids = IamRoleServ::find_id_rel_accounts(&role.id, None, None, funs, &app_ctx).await?;
                app_role_account.insert(role.id.clone(), role_account_ids);
            }
            let app_account_vec = IamAppServ::find_rel_account(&app_id.clone(), funs, &app_ctx).await?.iter().map(|r| r.rel_id.clone()).collect::<Vec<_>>();
            app_account.insert(app_id.clone(), app_account_vec);
        }
        Ok(IamSubDeployTowExportAggResp {
            app: Some(app_vec),
            app_apps: Some(app_apps),
            app_role: Some(app_role),
            app_account: Some(app_account),
            app_role_account: Some(app_role_account),
        })
    }

    pub(crate) async fn sub_deploy_import(import_req: IamSubDeployTowImportReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let _ = Self::import_org(import_req.org_set.clone(), import_req.org_set_cate, funs, ctx).await;
        let _ = Self::import_apps(import_req.apps_set.clone(), import_req.apps_set_cate, funs, ctx).await;
        let _ = Self::import_iam_config(import_req.iam_config, funs, ctx).await;
        let _ = Self::import_account(
            import_req.accounts,
            import_req.account_cert,
            import_req.account_role,
            import_req.account_org,
            import_req.account_apps,
            import_req.org_set,
            import_req.apps_set,
            funs,
            ctx,
        )
        .await;
        let _ = Self::import_res(
            import_req.res_set,
            import_req.res_set_cate,
            import_req.res_items,
            import_req.res_set_item,
            import_req.res_api,
            import_req.res_role,
            funs,
            ctx,
        )
        .await;
        Ok(())
    }

    async fn import_iam_config(iam_config: Option<HashMap<String, Vec<IamConfigDetailResp>>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(iam_config) = iam_config {
            for (rel_item_id, configs) in iam_config {
                let iam_config_reqs = configs
                    .iter()
                    .map(|config| IamConfigAggOrModifyReq {
                        name: Some(config.name.clone()),
                        data_type: IamConfigDataTypeKind::parse(&config.data_type.clone()).unwrap_or(IamConfigDataTypeKind::Number),
                        note: Some(config.note.clone()),
                        value1: Some(config.value1.clone()),
                        value2: Some(config.value2.clone()),
                        ext: Some(config.ext.clone()),
                        disabled: Some(config.disabled),
                        code: IamConfigKind::parse(&config.code.clone()).unwrap_or(IamConfigKind::TokenExpire),
                    })
                    .collect::<Vec<_>>();
                let mock_ctx = TardisContext {
                    own_paths: rel_item_id.clone(),
                    ..ctx.clone()
                };
                IamConfigServ::add_or_modify_batch(&rel_item_id, iam_config_reqs, funs, &mock_ctx).await?;
            }
        }
        Ok(())
    }

    async fn import_res(
        res_set: Option<RbumSetDetailResp>,
        res_set_cate: Option<Vec<RbumSetCateDetailResp>>,
        res_items: Option<Vec<IamResDetailResp>>,
        res_set_item: Option<HashMap<String, Vec<String>>>,
        res_api: Option<HashMap<String, Vec<String>>>,
        res_role: Option<HashMap<String, Vec<String>>>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(),
            ..ctx.clone()
        };
        if let Some(res_set) = res_set {
            if RbumSetServ::count_rbums(
                &RbumSetFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ids: Some(vec![res_set.id.clone()]),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                &global_ctx,
            )
            .await?
                == 0
            {
                RbumSetServ::add_rbum(
                    &mut RbumSetAddReq {
                        id: Some(TrimString::from(res_set.id.clone())),
                        code: TrimString::from(res_set.code),
                        kind: TrimString::from(res_set.kind),
                        name: TrimString::from(res_set.name),
                        note: Some(res_set.note),
                        icon: Some(res_set.icon),
                        sort: Some(res_set.sort),
                        ext: Some(res_set.ext),
                        scope_level: Some(res_set.scope_level),
                        disabled: Some(res_set.disabled),
                    },
                    funs,
                    &global_ctx,
                )
                .await?;
            }
            if let Some(res_set_cate) = res_set_cate {
                let new_res_set_cate_ids = res_set_cate.iter().map(|cate| cate.id.clone()).collect::<Vec<_>>();
                let old_res_set_cate = RbumSetCateServ::find_detail_rbums(
                    &RbumSetCateFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        rel_rbum_set_id: Some(res_set.id.clone()),
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    &global_ctx,
                )
                .await?;
                let delete_old_res_set_cate = old_res_set_cate.iter().filter(|cate| !new_res_set_cate_ids.contains(&cate.id)).collect::<Vec<_>>();
                for cate in res_set_cate {
                    if RbumSetCateServ::count_rbums(
                        &RbumSetCateFilterReq {
                            basic: RbumBasicFilterReq {
                                with_sub_own_paths: true,
                                ids: Some(vec![cate.id.clone()]),
                                ..Default::default()
                            },
                            rel_rbum_set_id: Some(res_set.id.clone()),
                            ..Default::default()
                        },
                        funs,
                        &global_ctx,
                    )
                    .await?
                        == 0
                    {
                        funs.db()
                            .insert_one(
                                bios_basic::rbum::domain::rbum_set_cate::ActiveModel {
                                    id: Set(cate.id.clone()),
                                    sys_code: Set(cate.sys_code.clone()),
                                    bus_code: Set(cate.bus_code.to_string()),
                                    name: Set(cate.name.to_string()),
                                    icon: Set(cate.icon),
                                    sort: Set(cate.sort),
                                    ext: Set(cate.ext),
                                    rel_rbum_set_id: Set(res_set.id.to_string()),
                                    scope_level: Set(cate.scope_level.to_int()),
                                    ..Default::default()
                                },
                                &global_ctx,
                            )
                            .await?;
                    }
                }
                for cate in delete_old_res_set_cate {
                    let _ = RbumSetCateServ::delete_with_all_rels(&cate.id, funs, &global_ctx).await?;
                }
            }
            if let Some(res_items) = res_items {
                let old_res_item_ids = IamResServ::find_id_items(
                    &IamResFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            own_paths: Some("".to_string()),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?;
                let new_res_ids = res_items.iter().map(|res| res.id.clone()).collect::<Vec<_>>();
                let delete_old_res_ids = old_res_item_ids.iter().filter(|res_id| !new_res_ids.contains(&res_id)).collect::<Vec<_>>();
                for res_item in res_items.clone() {
                    let bind_api_res = if let Some(res_api) = res_api.clone() {
                        if res_api.contains_key(&res_item.id.clone()) {
                            Some(res_api[&res_item.id.clone()].clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if old_res_item_ids.contains(&res_item.id.clone()) {
                        let _ = IamResServ::modify_item(
                            &res_item.id.clone(),
                            &mut IamResModifyReq {
                                name: Some(TrimString(res_item.name)),
                                code: Some(TrimString(res_item.code)),
                                method: Some(TrimString(res_item.method)),
                                icon: Some(res_item.icon),
                                sort: Some(res_item.sort),
                                hide: Some(res_item.hide),
                                action: Some(res_item.action),
                                ext: Some(res_item.ext),
                                scope_level: Some(res_item.scope_level),
                                disabled: Some(res_item.disabled),
                                crypto_req: Some(res_item.crypto_req),
                                crypto_resp: Some(res_item.crypto_resp),
                                double_auth: Some(res_item.double_auth),
                                double_auth_msg: Some(res_item.double_auth_msg),
                                need_login: Some(res_item.need_login),
                                bind_api_res,
                                bind_data_guards: None, // @TODO 需支持导入数据权限
                            },
                            funs,
                            &global_ctx,
                        )
                        .await?;
                    } else {
                        if let Some(res_set_item) = res_set_item.clone() {
                            if res_set_item.contains_key(&res_item.id.clone()) && !res_set_item[&res_item.id].is_empty() {
                                let set_cate_id = res_set_item[&res_item.id][0].clone();
                                let _ = IamResServ::add_res_agg(
                                    &mut IamResAggAddReq {
                                        res: IamResAddReq {
                                            id: Some(TrimString(res_item.id.clone())),
                                            code: TrimString(res_item.code),
                                            name: TrimString(res_item.name),
                                            kind: res_item.kind,
                                            icon: Some(res_item.icon),
                                            sort: Some(res_item.sort),
                                            method: Some(TrimString(res_item.method)),
                                            hide: Some(res_item.hide),
                                            action: Some(res_item.action),
                                            ext: Some(res_item.ext),
                                            scope_level: Some(res_item.scope_level),
                                            disabled: Some(res_item.disabled),
                                            crypto_req: Some(res_item.crypto_req),
                                            crypto_resp: Some(res_item.crypto_resp),
                                            double_auth: Some(res_item.double_auth),
                                            double_auth_msg: Some(res_item.double_auth_msg),
                                            need_login: Some(res_item.need_login),
                                            bind_api_res,
                                            bind_data_guards: None, // @TODO 需支持导入数据权限
                                        },
                                        set: IamSetItemAggAddReq { set_cate_id },
                                    },
                                    &res_set.id,
                                    funs,
                                    &global_ctx,
                                )
                                .await?;
                            }
                        }
                    }
                    if let Some(res_role) = res_role.clone() {
                        if res_role.contains_key(&res_item.id.clone()) && !res_role[&res_item.id.clone()].is_empty() {
                            for role_id in res_role[&res_item.id.clone()].clone() {
                                if IamRoleServ::count_items(
                                    &IamRoleFilterReq {
                                        basic: RbumBasicFilterReq {
                                            with_sub_own_paths: true,
                                            ids: Some(vec![role_id.clone()]),
                                            ..Default::default()
                                        },
                                        ..Default::default()
                                    },
                                    funs,
                                    &global_ctx,
                                )
                                .await?
                                    > 0
                                {
                                    let _ = IamRelServ::add_simple_rel(&IamRelKind::IamResRole, &res_item.id, &role_id, None, None, true, false, funs, &global_ctx).await?;
                                }
                            }
                        }
                    }
                }
                for delete_res_id in delete_old_res_ids {
                    let _ = IamResServ::delete_item_with_all_rels(&delete_res_id, funs, &global_ctx).await?;
                }
            }
        }
        Ok(())
    }

    async fn import_org(org_set: Option<RbumSetDetailResp>, org_set_cate: Option<Vec<RbumSetCateDetailResp>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(org_set) = org_set {
            if RbumSetServ::count_rbums(
                &RbumSetFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ids: Some(vec![org_set.id.clone()]),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
                == 0
            {
                RbumSetServ::add_rbum(
                    &mut RbumSetAddReq {
                        id: Some(TrimString::from(org_set.id.clone())),
                        code: TrimString::from(org_set.code),
                        kind: TrimString::from(org_set.kind),
                        name: TrimString::from(org_set.name),
                        note: Some(org_set.note),
                        icon: Some(org_set.icon),
                        sort: Some(org_set.sort),
                        ext: Some(org_set.ext),
                        scope_level: Some(org_set.scope_level),
                        disabled: Some(org_set.disabled),
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
            if let Some(org_set_cate) = org_set_cate {
                let new_org_set_cate_ids = org_set_cate.iter().map(|cate| cate.id.clone()).collect::<Vec<_>>();
                let old_org_set_cate = RbumSetCateServ::find_detail_rbums(
                    &RbumSetCateFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        rel_rbum_set_id: Some(org_set.id.clone()),
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?;
                let delete_old_org_set_cate = old_org_set_cate.iter().filter(|cate| !new_org_set_cate_ids.contains(&cate.id)).collect::<Vec<_>>();
                for cate in org_set_cate {
                    if RbumSetCateServ::count_rbums(
                        &RbumSetCateFilterReq {
                            basic: RbumBasicFilterReq {
                                with_sub_own_paths: true,
                                ids: Some(vec![cate.id.clone()]),
                                ..Default::default()
                            },
                            rel_rbum_set_id: Some(org_set.id.clone()),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await?
                        == 0
                    {
                        funs.db()
                            .insert_one(
                                bios_basic::rbum::domain::rbum_set_cate::ActiveModel {
                                    id: Set(cate.id.clone()),
                                    sys_code: Set(cate.sys_code.clone()),
                                    bus_code: Set(cate.bus_code.to_string()),
                                    name: Set(cate.name.to_string()),
                                    icon: Set(cate.icon),
                                    sort: Set(cate.sort),
                                    ext: Set(cate.ext),
                                    rel_rbum_set_id: Set(org_set.id.to_string()),
                                    scope_level: Set(cate.scope_level.to_int()),
                                    ..Default::default()
                                },
                                ctx,
                            )
                            .await?;
                    }
                }
                for cate in delete_old_org_set_cate {
                    RbumSetCateServ::delete_with_all_rels(&cate.id, funs, ctx).await?;
                }
            }
        }
        Ok(())
    }

    async fn import_apps(apps_set: Option<RbumSetDetailResp>, apps_set_cate: Option<Vec<RbumSetCateDetailResp>>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(apps_set) = apps_set {
            if RbumSetServ::count_rbums(
                &RbumSetFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ids: Some(vec![apps_set.id.clone()]),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?
                == 0
            {
                RbumSetServ::add_rbum(
                    &mut RbumSetAddReq {
                        id: Some(TrimString::from(apps_set.id.clone())),
                        code: TrimString::from(apps_set.code),
                        kind: TrimString::from(apps_set.kind),
                        name: TrimString::from(apps_set.name),
                        note: Some(apps_set.note),
                        icon: Some(apps_set.icon),
                        sort: Some(apps_set.sort),
                        ext: Some(apps_set.ext),
                        scope_level: Some(apps_set.scope_level),
                        disabled: Some(apps_set.disabled),
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
            if let Some(apps_set_cate) = apps_set_cate {
                let new_apps_set_cate_ids = apps_set_cate.iter().map(|cate| cate.id.clone()).collect::<Vec<_>>();
                let old_apps_set_cate = RbumSetCateServ::find_detail_rbums(
                    &RbumSetCateFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            ..Default::default()
                        },
                        rel_rbum_set_id: Some(apps_set.id.clone()),
                        ..Default::default()
                    },
                    None,
                    None,
                    funs,
                    ctx,
                )
                .await?;
                let delete_old_apps_set_cate = old_apps_set_cate.iter().filter(|cate| !new_apps_set_cate_ids.contains(&cate.id)).collect::<Vec<_>>();
                for cate in apps_set_cate {
                    if RbumSetCateServ::count_rbums(
                        &RbumSetCateFilterReq {
                            basic: RbumBasicFilterReq {
                                with_sub_own_paths: true,
                                ids: Some(vec![cate.id.clone()]),
                                own_paths: Some("".to_string()),
                                ..Default::default()
                            },
                            rel_rbum_set_id: Some(apps_set.id.clone()),
                            ..Default::default()
                        },
                        funs,
                        ctx,
                    )
                    .await?
                        == 0
                    {
                        funs.db()
                            .insert_one(
                                bios_basic::rbum::domain::rbum_set_cate::ActiveModel {
                                    id: Set(cate.id.clone()),
                                    sys_code: Set(cate.sys_code.clone()),
                                    bus_code: Set(cate.bus_code.to_string()),
                                    name: Set(cate.name.to_string()),
                                    icon: Set(cate.icon),
                                    sort: Set(cate.sort),
                                    ext: Set(cate.ext),
                                    rel_rbum_set_id: Set(cate.rel_rbum_set_id.to_string()),
                                    scope_level: Set(cate.scope_level.to_int()),
                                    ..Default::default()
                                },
                                ctx,
                            )
                            .await?;
                    }
                }
                for cate in delete_old_apps_set_cate {
                    RbumSetCateServ::delete_with_all_rels(&cate.id, funs, ctx).await?;
                }
            }
        }
        Ok(())
    }

    async fn import_account(
        accounts: Option<Vec<IamAccountDetailResp>>,
        account_cert: Option<HashMap<String, Vec<RbumCertSummaryWithSkResp>>>,
        account_role: Option<HashMap<String, Vec<String>>>,
        account_org: Option<HashMap<String, Vec<String>>>,
        account_apps: Option<HashMap<String, Vec<String>>>,
        org_set: Option<RbumSetDetailResp>,
        apps_set: Option<RbumSetDetailResp>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if let Some(accounts) = accounts {
            // let new_account_ids = accounts.iter().map(|account| account.id.clone()).collect::<Vec<_>>();
            // let old_accounts = IamAccountServ::find_items(
            //     &IamAccountFilterReq {
            //         basic: RbumBasicFilterReq {
            //             with_sub_own_paths: true,
            //             own_paths: Some("".to_string()),
            //             ..Default::default()
            //         },
            //         ..Default::default()
            //     },
            //     None,
            //     None,
            //     funs,
            //     ctx,
            // )
            // .await?;
            // let delete_old_accounts = old_accounts.iter().filter(|account| !new_account_ids.contains(&account.id)).collect::<Vec<_>>();
            for account in accounts {
                let account_ctx = TardisContext {
                    own_paths: account.own_paths,
                    ..ctx.clone()
                };
                if IamAccountServ::count_items(
                    &IamAccountFilterReq {
                        basic: RbumBasicFilterReq {
                            with_sub_own_paths: true,
                            own_paths: Some("".to_string()),
                            ids: Some(vec![account.id.clone()]),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    funs,
                    &account_ctx,
                )
                .await?
                    == 0
                {
                    IamAccountServ::add_item(
                        &mut IamAccountAddReq {
                            id: Some(TrimString::from(account.id.clone())),
                            name: TrimString::from(account.name.clone()),
                            scope_level: Some(account.scope_level),
                            logout_type: Some(IamAccountLogoutTypeKind::from_str(account.logout_type.as_str()).unwrap_or(IamAccountLogoutTypeKind::NotLogout)),
                            labor_type: Some(account.labor_type),
                            temporary: Some(account.temporary),
                            lock_status: Some(account.lock_status),
                            status: Some(account.status),
                            icon: Some(account.icon),
                            disabled: Some(account.disabled),
                        },
                        funs,
                        &account_ctx,
                    )
                    .await?;
                } else {
                    IamAccountServ::modify_item(
                        &account.id,
                        &mut IamAccountModifyReq {
                            name: Some(TrimString::from(account.name.clone())),
                            scope_level: Some(account.scope_level),
                            disabled: Some(account.disabled),
                            logout_type: Some(IamAccountLogoutTypeKind::from_str(account.logout_type.as_str()).unwrap_or(IamAccountLogoutTypeKind::NotLogout)),
                            labor_type: Some(account.labor_type),
                            temporary: Some(account.temporary),
                            lock_status: Some(account.lock_status),
                            status: Some(account.status),
                            icon: Some(account.icon),
                            is_auto: None,
                        },
                        &funs,
                        &account_ctx,
                    )
                    .await?
                }
                if let Some(ref account_cert) = account_cert {
                    if let Some(certs) = account_cert.get(&account.id) {
                        for cert in certs {
                            if funs
                                .db()
                                .count(
                                    Query::select()
                                        .column(bios_basic::rbum::domain::rbum_cert::Column::Id)
                                        .from(bios_basic::rbum::domain::rbum_cert::Entity)
                                        .and_where(Expr::col(bios_basic::rbum::domain::rbum_cert::Column::Id).eq(cert.id.clone())),
                                )
                                .await?
                                > 0
                            {
                                continue;
                            }
                            let cert_ctx = TardisContext {
                                own_paths: cert.own_paths.clone(),
                                owner: cert.owner.clone(),
                                ..account_ctx.clone()
                            };
                            funs.db()
                                .insert_one(
                                    bios_basic::rbum::domain::rbum_cert::ActiveModel {
                                        id: Set(cert.id.clone()),
                                        kind: Set(cert.kind.to_string()),
                                        ext: Set(cert.ext.clone()),
                                        ak: Set(cert.ak.clone()),
                                        sk: Set(cert.sk.clone()),
                                        sk_invisible: Set(cert.sk_invisible.clone()),
                                        supplier: Set(cert.supplier.clone()),
                                        start_time: Set(cert.start_time.clone()),
                                        end_time: Set(cert.end_time.clone()),
                                        conn_uri: Set(cert.conn_uri.clone()),
                                        status: Set(cert.status.to_int()),
                                        rel_rbum_cert_conf_id: Set(cert.rel_rbum_cert_conf_id.clone().unwrap_or_default()),
                                        rel_rbum_kind: Set(cert.rel_rbum_kind.to_int()),
                                        rel_rbum_id: Set(cert.rel_rbum_id.clone()),
                                        own_paths: Set(cert.own_paths.clone()),
                                        owner: Set(cert.owner.clone()),
                                        create_time: Set(cert.create_time.clone()),
                                        update_time: Set(cert.update_time.clone()),
                                        create_by: Set(cert.owner.clone()),
                                        update_by: Set(cert.owner.clone()),
                                    },
                                    &cert_ctx,
                                )
                                .await?;
                        }
                    }
                }
                if let Some(ref account_role) = account_role {
                    if let Some(role_ids) = account_role.get(&account.id) {
                        for role_id in role_ids {
                            if IamRoleServ::count_items(
                                &IamRoleFilterReq {
                                    basic: RbumBasicFilterReq {
                                        with_sub_own_paths: true,
                                        ids: Some(vec![role_id.clone()]),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                },
                                funs,
                                ctx,
                            )
                            .await?
                                == 0
                            {
                                continue;
                            }
                            let _ = IamRelServ::add_simple_rel(&IamRelKind::IamAccountRole, &account.id, &role_id, None, None, true, false, funs, ctx).await;
                        }
                    }
                }
                if let Some(org_set) = org_set.clone() {
                    if let Some(ref account_org) = account_org {
                        if let Some(org_ids) = account_org.get(&account.id) {
                            for org_id in org_ids {
                                let (cate_ids, cate_codes) = if org_id.is_empty() {
                                    (None, Some(vec!["".to_string()]))
                                } else {
                                    (Some(vec![org_id.to_string()]), None)
                                };
                                // 不存在 cate 节点，则跳过
                                if !org_id.is_empty() {
                                    if RbumSetCateServ::count_rbums(
                                        &RbumSetCateFilterReq {
                                            basic: RbumBasicFilterReq {
                                                with_sub_own_paths: true,
                                                ids: Some(vec![org_id.clone()]),
                                                own_paths: Some("".to_string()),
                                                ..Default::default()
                                            },
                                            rel_rbum_set_id: Some(org_set.id.clone()),
                                            ..Default::default()
                                        },
                                        funs,
                                        ctx,
                                    )
                                    .await?
                                        == 0
                                    {
                                        continue;
                                    }
                                }
                                // 已经存在该节点，跳过
                                if RbumSetItemServ::exist_rbum(
                                    &RbumSetItemFilterReq {
                                        basic: RbumBasicFilterReq {
                                            with_sub_own_paths: true,
                                            own_paths: Some("".to_owned()),
                                            ..Default::default()
                                        },
                                        rel_rbum_set_cate_ids: cate_ids,
                                        rel_rbum_set_cate_sys_codes: cate_codes,
                                        rel_rbum_item_ids: Some(vec![account.id.clone()]),
                                        rel_rbum_set_id: Some(org_set.id.clone()),
                                        ..Default::default()
                                    },
                                    funs,
                                    ctx,
                                )
                                .await?
                                {
                                    continue;
                                }
                                IamSetServ::add_set_item(
                                    &IamSetItemAddReq {
                                        set_id: org_set.id.clone(),
                                        set_cate_id: org_id.clone(),
                                        sort: 0,
                                        rel_rbum_item_id: account.id.clone(),
                                    },
                                    funs,
                                    ctx,
                                )
                                .await?;
                            }
                        }
                    }
                }
                if let Some(apps_set) = apps_set.clone() {
                    if let Some(ref account_apps) = account_apps {
                        if let Some(app_ids) = account_apps.get(&account.id) {
                            for app_id in app_ids {
                                let (cate_ids, cate_codes) = if app_id.is_empty() {
                                    (None, Some(vec!["".to_string()]))
                                } else {
                                    (Some(vec![app_id.to_string()]), None)
                                };
                                // 不存在 cate 节点，则跳过
                                if !app_id.is_empty() {
                                    if RbumSetCateServ::count_rbums(
                                        &RbumSetCateFilterReq {
                                            basic: RbumBasicFilterReq {
                                                with_sub_own_paths: true,
                                                ids: Some(vec![app_id.clone()]),
                                                own_paths: Some("".to_string()),
                                                ..Default::default()
                                            },
                                            rel_rbum_set_id: Some(apps_set.id.clone()),
                                            ..Default::default()
                                        },
                                        funs,
                                        ctx,
                                    )
                                    .await?
                                        == 0
                                    {
                                        continue;
                                    }
                                }
                                // 已经存在该节点，跳过
                                if RbumSetItemServ::exist_rbum(
                                    &RbumSetItemFilterReq {
                                        basic: RbumBasicFilterReq {
                                            with_sub_own_paths: true,
                                            own_paths: Some("".to_owned()),
                                            ..Default::default()
                                        },
                                        rel_rbum_set_cate_ids: cate_ids,
                                        rel_rbum_set_cate_sys_codes: cate_codes,
                                        rel_rbum_item_ids: Some(vec![account.id.clone()]),
                                        rel_rbum_set_id: Some(apps_set.id.clone()),
                                        ..Default::default()
                                    },
                                    funs,
                                    ctx,
                                )
                                .await?
                                {
                                    continue;
                                }
                                IamSetServ::add_set_item(
                                    &IamSetItemAddReq {
                                        set_id: apps_set.id.clone(),
                                        set_cate_id: app_id.clone(),
                                        sort: 0,
                                        rel_rbum_item_id: account.id.clone(),
                                    },
                                    funs,
                                    ctx,
                                )
                                .await?;
                            }
                        }
                    }
                }
                let _ = IamSearchClient::async_add_or_modify_account_search(&account.id, Box::new(false), "", funs, ctx).await?;
            }
            // todo remove
            // for account in delete_old_accounts {
            //     let account_ctx = TardisContext {
            //         own_paths: account.own_paths.clone(),
            //         ..ctx.clone()
            //     };
            //     IamAccountServ::modify_item(
            //         &account.id,
            //         &mut IamAccountModifyReq {
            //             name: None,
            //             scope_level: None,
            //             disabled: Some(true),
            //             logout_type: None,
            //             labor_type: None,
            //             temporary: None,
            //             lock_status: None,
            //             status: None,
            //             is_auto: None,
            //             icon: None,
            //         },
            //         funs,
            //         &account_ctx,
            //     )
            //     .await?;
            //     IamSearchClient::async_add_or_modify_account_search(&account.id, Box::new(true), "", &funs, &ctx).await?;
            // }
        }
        Ok(())
    }
}

#[async_trait]
impl
    RbumCrudOperation<
        iam_sub_deploy_host::ActiveModel,
        IamSubDeployHostAddReq,
        IamSubDeployHostModifyReq,
        IamSubDeployHostDetailResp,
        IamSubDeployHostDetailResp,
        IamSubDeployHostFilterReq,
    > for IamSubDeployHostServ
{
    fn get_table_name() -> &'static str {
        iam_sub_deploy_host::Entity.table_name()
    }

    async fn before_add_rbum(add_req: &mut IamSubDeployHostAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if IamSubDeployServ::count_items(
            &IamSubDeployFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ids: Some(vec![add_req.sub_deploy_id.to_string()]),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
            == 0
        {
            return Err(funs.err().not_found(&Self::get_obj_name(), "add", "sub_deploy_id not found", "404-iam-sub-deploy-license-sub-deploy-id"));
        }
        Ok(())
    }

    async fn package_add(add_req: &IamSubDeployHostAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_sub_deploy_host::ActiveModel> {
        Ok(iam_sub_deploy_host::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            name: Set(add_req.name.clone().unwrap_or(TrimString::from("")).to_string()),
            sub_deploy_id: Set(add_req.sub_deploy_id.to_string()),
            host: Set(add_req.host.to_string()),
            host_type: Set(add_req.host_type.clone().unwrap_or(IamSubDeployHostKind::IamSubDeployHostWhite).to_string()),
            note: Set(add_req.note.clone().unwrap_or("".to_string())),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &IamSubDeployHostModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_sub_deploy_host::ActiveModel> {
        let mut iam_sub_deploy_host = iam_sub_deploy_host::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(name) = &modify_req.name {
            iam_sub_deploy_host.name = Set(name.to_string());
        }
        if let Some(host) = &modify_req.host {
            iam_sub_deploy_host.host = Set(host.to_string());
        }
        if let Some(note) = &modify_req.note {
            iam_sub_deploy_host.note = Set(note.to_string());
        }
        Ok(iam_sub_deploy_host)
    }

    async fn package_query(_: bool, filter: &IamSubDeployHostFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::Id),
                (iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::Name),
                (iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::SubDeployId),
                (iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::Host),
                (iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::HostType),
                (iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::Note),
                (iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::OwnPaths),
                (iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::Owner),
                (iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::CreateTime),
                (iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::UpdateTime),
                (iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::CreateBy),
                (iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::UpdateBy),
            ])
            .from(iam_sub_deploy_host::Entity);
        if let Some(sub_deploy_id) = &filter.sub_deploy_id {
            query.and_where(Expr::col((iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::SubDeployId)).eq(sub_deploy_id.to_string()));
        }
        if let Some(host) = &filter.host {
            query.and_where(Expr::col((iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::Host)).like(host));
        }
        if let Some(host_type) = &filter.host_type {
            query.and_where(Expr::col((iam_sub_deploy_host::Entity, iam_sub_deploy_host::Column::HostType)).eq(host_type.to_string()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, true, false, ctx);
        Ok(query)
    }
}

#[async_trait]
impl
    RbumCrudOperation<
        iam_sub_deploy_license::ActiveModel,
        IamSubDeployLicenseAddReq,
        IamSubDeployLicenseModifyReq,
        IamSubDeployLicenseDetailResp,
        IamSubDeployLicenseDetailResp,
        IamSubDeployLicenseFilterReq,
    > for IamSubDeployLicenseServ
{
    fn get_table_name() -> &'static str {
        iam_sub_deploy_license::Entity.table_name()
    }

    async fn before_add_rbum(add_req: &mut IamSubDeployLicenseAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if IamSubDeployServ::count_items(
            &IamSubDeployFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ids: Some(vec![add_req.sub_deploy_id.to_string()]),
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
            == 0
        {
            return Err(funs.err().not_found(&Self::get_obj_name(), "add", "sub_deploy_id not found", "404-iam-sub-deploy-license-sub-deploy-id"));
        }
        Ok(())
    }

    async fn package_add(add_req: &IamSubDeployLicenseAddReq, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_sub_deploy_license::ActiveModel> {
        // 开始时间没有值时，默认值为当前时间
        let start_time = add_req.start_time.unwrap_or(Utc::now());
        // 结束时间没有值时，默认值为当前时间+一年
        let end_time = add_req.end_time.unwrap_or_else(|| {
            let mut end_time = start_time.clone();
            end_time = end_time + chrono::Duration::days(365);
            end_time
        });
        // 结束时间小于开始时间时，抛出异常
        if end_time < start_time {
            return Err(funs.err().bad_request(&Self::get_obj_name(), "add", "start time must be less than end time", "400-iam-sub-deploy-license-time"));
        }
        Ok(iam_sub_deploy_license::ActiveModel {
            id: Set(TardisFuns::field.nanoid()),
            name: Set(add_req.name.clone().unwrap_or(TrimString::from("")).to_string()),
            sub_deploy_id: Set(add_req.sub_deploy_id.to_string()),
            license: Set(format!("sk-{}", TardisFuns::field.nanoid())),
            start_time: Set(start_time),
            end_time: Set(end_time),
            ..Default::default()
        })
    }

    async fn package_modify(id: &str, modify_req: &IamSubDeployLicenseModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_sub_deploy_license::ActiveModel> {
        let mut iam_sub_deploy_license = iam_sub_deploy_license::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        iam_sub_deploy_license.name = Set(modify_req.name.to_string());
        Ok(iam_sub_deploy_license)
    }

    async fn package_query(_: bool, filter: &IamSubDeployLicenseFilterReq, _: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<SelectStatement> {
        let mut query = Query::select();
        query
            .columns(vec![
                (iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::Id),
                (iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::Name),
                (iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::License),
                (iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::SubDeployId),
                (iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::StartTime),
                (iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::EndTime),
                (iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::OwnPaths),
                (iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::Owner),
                (iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::CreateTime),
                (iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::UpdateTime),
                (iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::CreateBy),
                (iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::UpdateBy),
            ])
            .from(iam_sub_deploy_license::Entity);

        if let Some(sub_deploy_id) = &filter.sub_deploy_id {
            query.and_where(Expr::col((iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::SubDeployId)).eq(sub_deploy_id.to_string()));
        }
        query.with_filter(Self::get_table_name(), &filter.basic, true, false, ctx);
        Ok(query)
    }
}

impl IamSubDeployLicenseServ {
    /// Show license
    ///
    /// 显示license
    pub async fn show_license(id: &str, filter: &IamSubDeployLicenseFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        #[derive(sea_orm::FromQueryResult)]
        struct LicenseResp {
            pub license: String,
        }
        let mut query = Query::select();
        query
            .column((iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::License))
            .from(iam_sub_deploy_license::Entity)
            .and_where(Expr::col((iam_sub_deploy_license::Entity, iam_sub_deploy_license::Column::Id)).eq(id))
            .with_filter(Self::get_table_name(), &filter.basic, false, false, ctx);
        let license_resp = funs.db().get_dto::<LicenseResp>(&query).await?;
        if let Some(license_resp) = license_resp {
            Ok(license_resp.license)
        } else {
            Err(funs.err().not_found(&Self::get_obj_name(), "show_license", "not found license record", "404-iam-sub-deploy-license-not-exist"))
        }
    }

    /// Generate license file
    ///
    /// 生成license文件
    pub async fn generate_license(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let iam_sub_deploy_license = IamSubDeployLicenseServ::get_rbum(
            id,
            &IamSubDeployLicenseFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;
        let white_host = IamSubDeployHostServ::find_rbums(
            &IamSubDeployHostFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                sub_deploy_id: Some(iam_sub_deploy_license.sub_deploy_id),
                host_type: Some(IamSubDeployHostKind::IamSubDeployHostWhite),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await?;

        let license = Self::show_license(
            id,
            &IamSubDeployLicenseFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?;

        let pri_key = TardisFuns::crypto.sm2.new_private_key_from_str(&funs.conf::<IamConfig>().crypto_pri_key)?;
        // let pub_key = TardisFuns::crypto.sm2.new_public_key(&pri_key)?;
        let license_data = if white_host.is_empty() {
            serde_json::json!({
                "mid_info": license,
                // "start_time": iam_sub_deploy_license.start_time,
                "expire": iam_sub_deploy_license.end_time,
            })
        } else {
            serde_json::json!({
                "mid_info": license,
                // "start_time": iam_sub_deploy_license.start_time,
                "expire": iam_sub_deploy_license.end_time,
                "ip_white_list": white_host.iter().map(|host| host.host.clone()).collect::<Vec<String>>(),
            })
        };

        let info = serde_json::to_string(&license_data).unwrap_or_default();
        let signature = pri_key.sign(&info).unwrap_or_default();
        let certification = serde_json::json!({
            "info": info,
            "signature": signature,
        });
        let cert_base64 = TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&certification)?);
        // 80 char per line
        let cert_base64_chunks: Vec<_> = cert_base64.as_bytes().chunks(80).map(|chunk| std::str::from_utf8(chunk).unwrap()).collect();
        let cert_base64 = cert_base64_chunks.join("\n");
        Ok(format!("-----BEGIN BIOS-CERTIFICATION-----\n{cert_base64}\n-----END BIOS-CERTIFICATION-----"))
    }
}

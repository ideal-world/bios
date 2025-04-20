use async_trait::async_trait;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_crud_serv::{RbumCrudOperation, RbumCrudQueryPackage};
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
use crate::basic::dto::iam_filer_dto::{IamSubDeployFilterReq, IamSubDeployHostFilterReq, IamSubDeployLicenseFilterReq};
use crate::basic::dto::iam_sub_deploy_dto::{
    IamSubDeployAddReq, IamSubDeployDetailResp, IamSubDeployModifyReq, IamSubDeployOneExportAggResp, IamSubDeployOneImportReq, IamSubDeploySummaryResp,
    IamSubDeployTowExportAggResp, IamSubDeployTowImportReq,
};
use crate::basic::dto::iam_sub_deploy_host_dto::{IamSubDeployHostAddReq, IamSubDeployHostDetailResp, IamSubDeployHostModifyReq};
use crate::basic::dto::iam_sub_deploy_license_dto::{IamSubDeployLicenseAddReq, IamSubDeployLicenseDetailResp, IamSubDeployLicenseModifyReq};
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::iam_config::IamBasicInfoManager;
use crate::iam_enumeration::{IamRelKind, IamSubDeployHostKind};

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
        Ok(RbumItemKernelAddReq {
            id: None,
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

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamSubDeployFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_sub_deploy::Entity, iam_sub_deploy::Column::Province));
        query.column((iam_sub_deploy::Entity, iam_sub_deploy::Column::AccessUrl));
        query.column((iam_sub_deploy::Entity, iam_sub_deploy::Column::Note));
        if let Some(province) = &filter.province {
            query.and_where(Expr::col(iam_sub_deploy::Column::Province).eq(province));
        }
        if let Some(access_url) = &filter.access_url {
            query.and_where(Expr::col(iam_sub_deploy::Column::AccessUrl).like(access_url));
        }
        Ok(())
    }
}

impl IamSubDeployServ {
    pub async fn save() {}

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
        let mut export = IamSubDeployOneExportAggResp {
            account: None,
            org: None,
            apps: None,
            role: None,       // Updated from todo!() to None
            iam_config: None, // Updated from todo!() to None
            cert: None,       // Updated from todo!() to None
        };
        Ok(export)
    }

    pub(crate) async fn one_deploy_import(import_req: IamSubDeployOneImportReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        Ok(())
    }

    pub(crate) async fn sub_deploy_export(
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<IamSubDeployTowExportAggResp> {
        let mut export = IamSubDeployTowExportAggResp {
            account: None,
            org: None,
            apps: None,       // Updated from todo!() to None
            role: None,       // Updated from todo!() to None
            iam_config: None, // Updated from todo!() to None
        };
        Ok(export)
    }

    pub(crate) async fn sub_deploy_import(import_req: IamSubDeployTowImportReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
        // let private_key = TardisFuns::crypto.sm2.new_private_key_from_str(&license)?;

        let license_data = serde_json::json!({
            "license": license,
            "start_time": iam_sub_deploy_license.start_time,
            "end_time": iam_sub_deploy_license.end_time,
            "allowed_hosts": white_host.iter().map(|host| host.host.clone()).collect::<Vec<String>>(),
        });
        Ok(TardisFuns::crypto.base64.encode(TardisFuns::json.obj_to_string(&license_data)?))
    }
}

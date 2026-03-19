use async_trait::async_trait;
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::{dto::TardisContext, result::TardisResult};
use tardis::db::sea_orm::sea_query::{Expr, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::clients::iam_search_client::IamSearchClient;
use crate::iam_enumeration::{IamRelKind, IamThirdPartyAppStatusKind};
use crate::basic::{
    domain::iam_third_party_app,
    dto::{
        iam_filer_dto::IamThirdPartyAppFilterReq,
        iam_third_party_app_dto::{
            IamThirdPartyAppAddReq, IamThirdPartyAppDetailResp, IamThirdPartyAppModifyReq, IamThirdPartyAppSummaryResp,
        },
    },
};
use crate::iam_config::IamBasicInfoManager;

pub struct IamThirdPartyAppServ;

#[async_trait]
impl RbumItemCrudOperation<
    iam_third_party_app::ActiveModel,
    IamThirdPartyAppAddReq,
    IamThirdPartyAppModifyReq,
    IamThirdPartyAppSummaryResp,
    IamThirdPartyAppDetailResp,
    IamThirdPartyAppFilterReq,
> for IamThirdPartyAppServ
{
    fn get_ext_table_name() -> &'static str {
        iam_third_party_app::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.kind_third_party_app_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone()))
    }

    async fn package_item_add(
        add_req: &IamThirdPartyAppAddReq,
        _: &TardisFunsInst,
        _: &TardisContext,
    ) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: add_req.id.clone(),
            name: add_req.name.clone(),
            scope_level: add_req.scope_level.clone(),
            ..Default::default()
        })
    }

    async fn package_ext_add(
        id: &str,
        add_req: &IamThirdPartyAppAddReq,
        _: &TardisFunsInst,
        _: &TardisContext,
    ) -> TardisResult<iam_third_party_app::ActiveModel> {
        Ok(iam_third_party_app::ActiveModel {
            id: Set(id.to_string()),
            external_id: Set(add_req.external_id.clone()),
            description: Set(add_req.description.clone()),
            icon: Set(add_req.icon.clone().unwrap_or_default()),
            link_url: Set(add_req.link_url.to_string()),
            status: Set(add_req.status.as_ref().map_or(0, |s| s.to_int())),
            sort: Set(add_req.sort.unwrap_or(0)),
            ..Default::default()
        })
    }

    async fn package_item_modify(
        _: &str,
        modify_req: &IamThirdPartyAppModifyReq,
        _: &TardisFunsInst,
        _: &TardisContext,
    ) -> TardisResult<Option<RbumItemKernelModifyReq>> {
        if modify_req.name.is_none() && modify_req.scope_level.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemKernelModifyReq {
            code: None,
            name: modify_req.name.clone(),
            scope_level: modify_req.scope_level.clone(),
            disabled: None,
        }))
    }

    async fn package_ext_modify(
        id: &str,
        modify_req: &IamThirdPartyAppModifyReq,
        _: &TardisFunsInst,
        _: &TardisContext,
    ) -> TardisResult<Option<iam_third_party_app::ActiveModel>> {
        if modify_req.description.is_none()
            && modify_req.external_id.is_none()
            && modify_req.icon.is_none()
            && modify_req.link_url.is_none()
            && modify_req.status.is_none()
            && modify_req.sort.is_none()
        {
            return Ok(None);
        }
        let mut model = iam_third_party_app::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(description) = &modify_req.description {
            model.description = Set(Some(description.clone()));
        }
        if modify_req.external_id.is_some() {
            model.external_id = Set(modify_req.external_id.clone());
        }
        if let Some(icon) = &modify_req.icon {
            model.icon = Set(icon.clone());
        }
        if let Some(link_url) = &modify_req.link_url {
            model.link_url = Set(link_url.to_string());
        }
        if let Some(status) = &modify_req.status {
            model.status = Set(status.to_int());
        }
        if let Some(sort) = &modify_req.sort {
            model.sort = Set(*sort);
        }
        Ok(Some(model))
    }

    async fn package_ext_query(
        query: &mut SelectStatement,
        _: bool,
        filter: &IamThirdPartyAppFilterReq,
        _: &TardisFunsInst,
        _: &TardisContext,
    ) -> TardisResult<()> {
        query.column((iam_third_party_app::Entity, iam_third_party_app::Column::ExternalId));
        query.column((iam_third_party_app::Entity, iam_third_party_app::Column::Description));
        query.column((iam_third_party_app::Entity, iam_third_party_app::Column::Icon));
        query.column((iam_third_party_app::Entity, iam_third_party_app::Column::LinkUrl));
        query.column((iam_third_party_app::Entity, iam_third_party_app::Column::Status));
        query.column((iam_third_party_app::Entity, iam_third_party_app::Column::Sort));
        if let Some(external_id) = &filter.external_id {
            query.and_where(Expr::col((iam_third_party_app::Entity, iam_third_party_app::Column::ExternalId)).eq(external_id.as_str()));
        }
        if let Some(status) = &filter.status {
            query.and_where(Expr::col((iam_third_party_app::Entity, iam_third_party_app::Column::Status)).eq(status.to_int()));
        }
        if let Some(sort) = &filter.sort {
            query.and_where(Expr::col((iam_third_party_app::Entity, iam_third_party_app::Column::Sort)).eq(*sort));
        }
        Ok(())
    }
}

impl IamThirdPartyAppServ {
    /// 根据外部ID获取第三方应用
    pub async fn get_item_by_external_id(
        external_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Option<IamThirdPartyAppDetailResp>> {
        let page = Self::paginate_items(
            &IamThirdPartyAppFilterReq {
                external_id: Some(external_id.to_string()),
                ..Default::default()
            },
            1,
            1,
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        if let Some(summary) = page.records.into_iter().next() {
            let detail = Self::get_item(&summary.id, &IamThirdPartyAppFilterReq::default(), funs, ctx).await?;
            Ok(Some(detail))
        } else {
            Ok(None)
        }
    }

    /// 根据外部ID删除第三方应用
    pub async fn delete_item_by_external_id(
        external_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        if let Some(item) = Self::get_item_by_external_id(external_id, funs, ctx).await? {
            Self::delete_item_with_all_rels(&item.id, funs, ctx).await?;
        }
        Ok(())
    }

    /// 绑定账号到第三方应用
    pub async fn add_rel_account(
        third_party_app_id: &str,
        account_id: &str,
        ignore_exist_error: bool,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        IamRelServ::add_simple_rel(
            &IamRelKind::IamThirdPartyAppAccount,
            account_id,
            third_party_app_id,
            None,
            None,
            ignore_exist_error,
            false,
            funs,
            ctx,
        )
        .await
    }

    /// 解绑账号与第三方应用
    pub async fn delete_rel_account(
        third_party_app_id: &str,
        account_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        IamRelServ::delete_simple_rel(&IamRelKind::IamThirdPartyAppAccount, account_id, third_party_app_id, funs, ctx).await?;
        IamSearchClient::sync_add_or_modify_account_search(account_id, Box::new(true), "", funs, ctx).await?;
        Ok(())
    }

    /// 查询第三方应用绑定的账号列表
    pub async fn find_rel_account(
        third_party_app_id: &str,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_to_simple_rels(
            &IamRelKind::IamThirdPartyAppAccount,
            third_party_app_id,
            None,
            None,
            funs,
            ctx,
        )
        .await
    }

    /// 分页查询第三方应用绑定的账号
    pub async fn paginate_rel_account(
        third_party_app_id: &str,
        page_number: u32,
        page_size: u32,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        IamRelServ::paginate_to_simple_rels(
            &IamRelKind::IamThirdPartyAppAccount,
            third_party_app_id,
            page_number,
            page_size,
            desc_by_create,
            desc_by_update,
            funs,
            ctx,
        )
        .await
    }
}

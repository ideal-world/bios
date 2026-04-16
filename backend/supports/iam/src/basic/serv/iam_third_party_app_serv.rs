use async_trait::async_trait;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::dto::rbum_rel_dto::{RbumRelBoneResp, RbumRelModifyReq, RbumRelSimpleFindReq};
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelServ;
use tardis::serde_json::{json, Value as JsonValue};
use tardis::basic::{dto::TardisContext, result::TardisResult};
use tardis::db::sea_orm::sea_query::{Expr, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::clients::iam_search_client::IamSearchClient;
use crate::iam_enumeration::IamRelKind;
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
        .await?;
        IamSearchClient::sync_add_or_modify_account_search(account_id, Box::new(true), "", funs, ctx).await?;
        Ok(())
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
    ) -> TardisResult<Vec<RbumRelAggResp>> {
        IamRelServ::find_to_rels(
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

    /// 查询账号关联的所有第三方应用
    /// visible: None-不筛选；Some(true)-筛选ext.visible为true或ext为null；Some(false)-筛选ext.visible为false
    pub async fn find_rel_third_party_app(
        account_id: &str,
        visible: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<IamThirdPartyAppSummaryResp>> {
        let global_ctx = TardisContext {
            own_paths: "".to_string(), // 查询所有关联的第三方应用，不受租户/部门等维度限制
            ..ctx.clone()
        };
        let rels = IamRelServ::find_from_simple_rels(
            &IamRelKind::IamThirdPartyAppAccount,
            true,
            account_id,
            None,
            None,
            funs,
            &global_ctx,
        )
        .await?;
        let app_ids: Vec<String> = rels
            .into_iter()
            .filter(|r| {
                match visible {
                    None => true,
                    Some(true) => {
                        // 筛选 ext.visible 为 true 或 ext 为 null/空
                        r.ext.is_empty()
                            || tardis::serde_json::from_str::<JsonValue>(&r.ext)
                                .ok()
                                .and_then(|v| v.get("visible").and_then(|x| x.as_bool()))
                                .unwrap_or(true) // ext 中无 visible 时视为可见
                    }
                    Some(false) => {
                        // 筛选 ext.visible 为 false
                        tardis::serde_json::from_str::<JsonValue>(&r.ext)
                            .ok()
                            .and_then(|v| v.get("visible").and_then(|x| x.as_bool()))
                            == Some(false)
                    }
                }
            })
            .map(|r| r.rel_id)
            .collect();
        let len = app_ids.len();
        if len == 0 {
            return Ok(vec![]);
        }
        let page = Self::paginate_items(
            &IamThirdPartyAppFilterReq {
                basic: RbumBasicFilterReq {
                    ids: Some(app_ids),
                    with_sub_own_paths: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            1,
            len as u32,
            None,
            None,
            funs,
            ctx,
        )
        .await?;
        Ok(page.records)
    }

    /// 批量修改当前账号关联的第三方应用是否展示
    /// 通过 rel 表的 ext 字段存储：true 为展示，false 为隐藏
    pub async fn batch_modify_rel_display(
        account_id: &str,
        items: &[crate::basic::dto::iam_third_party_app_dto::IamThirdPartyAppDisplayModifyItem],
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<()> {
        for item in items {
            let rel_ids = RbumRelServ::find_rel_ids(
                &RbumRelSimpleFindReq {
                    tag: Some(IamRelKind::IamThirdPartyAppAccount.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    from_rbum_id: Some(account_id.to_string()),
                    to_rbum_item_id: Some(item.app_id.clone()),
                    from_own_paths: None,
                    to_rbum_own_paths: None,
                },
                funs,
                ctx,
            )
            .await?;
            if let Some(rel_id) = rel_ids.into_iter().next() {
                let ext = json!({ "visible": item.visible }).to_string();
                RbumRelServ::modify_rbum(
                    &rel_id,
                    &mut RbumRelModifyReq {
                        tag: None,
                        note: None,
                        ext: Some(ext),
                        disabled: None,
                    },
                    funs,
                    ctx,
                )
                .await?;
            }
        }
        Ok(())
    }
}

use async_trait::async_trait;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{Expr, SelectStatement};
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemModifyReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_res;
use crate::basic::dto::iam_filer_dto::IamResFilterReq;
use crate::basic::dto::iam_res_dto::{IamResAddReq, IamResAggAddReq, IamResDetailResp, IamResModifyReq, IamResSummaryResp};
use crate::basic::dto::iam_set_dto::IamSetItemAddReq;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_config::IamBasicInfoManager;
use crate::iam_enumeration::IamRelKind;

pub struct IamResServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_res::ActiveModel, IamResAddReq, IamResModifyReq, IamResSummaryResp, IamResDetailResp, IamResFilterReq> for IamResServ {
    fn get_ext_table_name() -> &'static str {
        iam_res::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        IamBasicInfoManager::get().kind_res_id
    }

    fn get_rbum_domain_id() -> String {
        IamBasicInfoManager::get().domain_iam_id
    }

    async fn package_item_add(add_req: &IamResAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: None,
            code: Some(add_req.code.clone()),
            name: add_req.name.clone(),
            disabled: add_req.disabled,
            scope_level: add_req.scope_level.clone(),
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamResAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<iam_res::ActiveModel> {
        Ok(iam_res::ActiveModel {
            id: Set(id.to_string()),
            kind: Set(add_req.kind.to_int()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            method: Set(add_req.method.as_ref().unwrap_or(&TrimString("".to_string())).to_string()),
            hide: Set(add_req.hide.unwrap_or(false)),
            action: Set(add_req.action.as_ref().unwrap_or(&"".to_string()).to_string()),
            ..Default::default()
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamResModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
        if modify_req.code.is_none() && modify_req.name.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemModifyReq {
            code: modify_req.code.clone(),
            name: modify_req.name.clone(),
            scope_level: modify_req.scope_level.clone(),
            disabled: modify_req.disabled,
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &IamResModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<iam_res::ActiveModel>> {
        if modify_req.icon.is_none() && modify_req.sort.is_none() && modify_req.method.is_none() && modify_req.hide.is_none() && modify_req.action.is_none() {
            return Ok(None);
        }
        let mut iam_res = iam_res::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            iam_res.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            iam_res.sort = Set(sort);
        }
        if let Some(method) = &modify_req.method {
            iam_res.method = Set(method.0.to_string());
        }
        if let Some(hide) = modify_req.hide {
            iam_res.hide = Set(hide);
        }
        if let Some(action) = &modify_req.action {
            iam_res.action = Set(action.to_string());
        }
        Ok(Some(iam_res))
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamResFilterReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_res::Entity, iam_res::Column::Kind));
        query.column((iam_res::Entity, iam_res::Column::Icon));
        query.column((iam_res::Entity, iam_res::Column::Sort));
        query.column((iam_res::Entity, iam_res::Column::Method));
        query.column((iam_res::Entity, iam_res::Column::Hide));
        query.column((iam_res::Entity, iam_res::Column::Action));
        if let Some(kind) = &filter.kind {
            query.and_where(Expr::col(iam_res::Column::Kind).eq(kind.to_int()));
        }
        if let Some(method) = &filter.method {
            query.and_where(Expr::col(iam_res::Column::Method).eq(method.as_str()));
        }
        Ok(())
    }
}

impl<'a> IamResServ {
    pub async fn find_simple_rel_roles(
        res_id: &str,
        with_sub: bool,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_from_simple_rels(IamRelKind::IamResRole, with_sub, res_id, desc_by_create, desc_by_update, funs, cxt).await
    }

    pub async fn paginate_simple_rel_roles(
        res_id: &str,
        with_sub: bool,
        page_number: u64,
        page_size: u64,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        IamRelServ::paginate_from_simple_rels(IamRelKind::IamResRole, with_sub, res_id, page_number, page_size, desc_by_create, desc_by_update, funs, cxt).await
    }
}

impl<'a> IamResServ {
    pub async fn add_agg_res(add_req: &mut IamResAggAddReq, set_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let res_id = Self::add_item(&mut add_req.res, funs, cxt).await?;
        IamSetServ::add_set_item(
            &mut IamSetItemAddReq {
                set_id: set_id.to_string(),
                set_cate_id: add_req.set.set_cate_id.to_string(),
                sort: 0,
                rel_rbum_item_id: res_id.clone(),
            },
            funs,
            cxt,
        )
        .await?;
        Ok(res_id)
    }
}

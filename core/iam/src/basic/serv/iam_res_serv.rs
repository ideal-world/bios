use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use bios_basic::rbum::rbum_config::RbumConfigApi;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::RbumSetItemServ;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::{Expr, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetItemFilterReq};
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemModifyReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_res;
use crate::basic::dto::iam_filer_dto::IamResFilterReq;
use crate::basic::dto::iam_res_dto::{IamResAddReq, IamResAggAddReq, IamResDetailResp, IamResModifyReq, IamResSummaryResp};
use crate::basic::dto::iam_set_dto::IamSetItemAddReq;
use crate::basic::serv::iam_key_cache_serv::IamResCacheServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_config::IamBasicInfoManager;
use crate::iam_enumeration::{IamRelKind, IamResKind};

use super::iam_account_serv::IamAccountServ;
use super::iam_cert_serv::IamCertServ;
use super::iam_role_serv::IamRoleServ;

pub struct IamResServ;

#[async_trait]
impl RbumItemCrudOperation<iam_res::ActiveModel, IamResAddReq, IamResModifyReq, IamResSummaryResp, IamResDetailResp, IamResFilterReq> for IamResServ {
    fn get_ext_table_name() -> &'static str {
        iam_res::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        IamBasicInfoManager::get_config(|conf| conf.kind_res_id.clone())
    }

    fn get_rbum_domain_id() -> String {
        IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone())
    }

    async fn package_item_add(add_req: &IamResAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: None,
            code: Some(add_req.code.clone()),
            name: add_req.name.clone(),
            disabled: add_req.disabled,
            scope_level: add_req.scope_level.clone(),
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamResAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<iam_res::ActiveModel> {
        Ok(iam_res::ActiveModel {
            id: Set(id.to_string()),
            kind: Set(add_req.kind.to_int()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            method: Set(add_req.method.as_ref().unwrap_or(&TrimString("*".to_string())).to_string()),
            hide: Set(add_req.hide.unwrap_or(false)),
            action: Set(add_req.action.as_ref().unwrap_or(&"".to_string()).to_string()),
            ..Default::default()
        })
    }

    async fn before_add_item(add_req: &mut IamResAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        add_req.encoding();
        Ok(())
    }

    async fn after_add_item(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let res = Self::peek_item(
            id,
            &IamResFilterReq {
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
        if res.kind == IamResKind::Api {
            IamResCacheServ::add_res(&res.code, &res.method, funs).await?;
        }
        Ok(())
    }

    async fn package_item_modify(_: &str, modify_req: &IamResModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
        if modify_req.name.is_none() && modify_req.scope_level.is_none() && modify_req.disabled.is_none() {
            return Ok(None);
        }
        Ok(Some(RbumItemModifyReq {
            code: None,
            name: modify_req.name.clone(),
            scope_level: modify_req.scope_level.clone(),
            disabled: modify_req.disabled,
        }))
    }

    async fn package_ext_modify(id: &str, modify_req: &IamResModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<iam_res::ActiveModel>> {
        if modify_req.icon.is_none() && modify_req.sort.is_none() && modify_req.hide.is_none() && modify_req.action.is_none() {
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
        if let Some(hide) = modify_req.hide {
            iam_res.hide = Set(hide);
        }
        if let Some(action) = &modify_req.action {
            iam_res.action = Set(action.to_string());
        }
        Ok(Some(iam_res))
    }

    async fn after_modify_item(id: &str, modify_req: &mut IamResModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(disabled) = modify_req.disabled {
            let res = Self::peek_item(
                id,
                &IamResFilterReq {
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
            if res.kind == IamResKind::Api {
                if disabled {
                    IamResCacheServ::delete_res(&res.code, &res.method, funs).await?;
                } else {
                    IamResCacheServ::add_res(&res.code, &res.method, funs).await?;
                }
            }
        }
        Ok(())
    }

    async fn before_delete_item(id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamResDetailResp>> {
        Ok(Some(
            Self::get_item(
                id,
                &IamResFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?,
        ))
    }

    async fn after_delete_item(_: &str, deleted_item: &Option<IamResDetailResp>, funs: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        if let Some(deleted_item) = deleted_item {
            if deleted_item.kind == IamResKind::Api {
                IamResCacheServ::delete_res(&deleted_item.code, &deleted_item.method, funs).await?;
            }
            Ok(())
        } else {
            Err(funs.err().not_found(&Self::get_obj_name(), "delete", "not found resource", "404-iam-res-not-exist"))
        }
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamResFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_res::Entity, iam_res::Column::Kind));
        query.column((iam_res::Entity, iam_res::Column::Icon));
        query.column((iam_res::Entity, iam_res::Column::Sort));
        query.column((iam_res::Entity, iam_res::Column::Method));
        query.column((iam_res::Entity, iam_res::Column::Hide));
        query.column((iam_res::Entity, iam_res::Column::Action));
        if let Some(kind) = &filter.kind {
            query.and_where(Expr::col(iam_res::Column::Kind).eq(kind.to_int()));
        }
        Ok(())
    }

    async fn peek_item(id: &str, filter: &IamResFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamResSummaryResp> {
        let res = Self::do_peek_item(id, filter, funs, ctx).await?;
        Ok(res.decoding())
    }

    async fn get_item(id: &str, filter: &IamResFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<IamResDetailResp> {
        let res = Self::do_get_item(id, filter, funs, ctx).await?;
        Ok(res.decoding())
    }

    async fn paginate_items(
        filter: &IamResFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<IamResSummaryResp>> {
        let mut res = Self::do_paginate_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        res.records = res.records.into_iter().map(|r| r.decoding()).collect();
        Ok(res)
    }

    async fn paginate_detail_items(
        filter: &IamResFilterReq,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<IamResDetailResp>> {
        let mut res = Self::do_paginate_detail_items(filter, page_number, page_size, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        res.records = res.records.into_iter().map(|r| r.decoding()).collect();
        Ok(res)
    }

    async fn find_one_item(filter: &IamResFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamResSummaryResp>> {
        let res = Self::do_find_one_item(filter, funs, ctx).await?;
        if let Some(r) = res {
            Ok(Some(r.decoding()))
        } else {
            Ok(None)
        }
    }

    async fn find_items(
        filter: &IamResFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<IamResSummaryResp>> {
        let res = Self::do_find_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        Ok(res.into_iter().map(|r| r.decoding()).collect())
    }

    async fn find_one_detail_item(filter: &IamResFilterReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<Option<IamResDetailResp>> {
        let res = Self::do_find_one_detail_item(filter, funs, ctx).await?;
        if let Some(r) = res {
            Ok(Some(r.decoding()))
        } else {
            Ok(None)
        }
    }

    async fn find_detail_items(
        filter: &IamResFilterReq,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<IamResDetailResp>> {
        let res = Self::do_find_detail_items(filter, desc_sort_by_create, desc_sort_by_update, funs, ctx).await?;
        Ok(res.into_iter().map(|r| r.decoding()).collect())
    }
}

impl IamResServ {
    pub async fn find_from_id_rel_roles(
        rel_kind: &IamRelKind,
        with_sub: bool,
        res_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        IamRelServ::find_from_id_rels(rel_kind, with_sub, res_id, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn find_to_id_rel_roles(
        rel_kind: &IamRelKind,
        res_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<String>> {
        IamRelServ::find_to_id_rels(rel_kind, res_id, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn find_from_simple_rel_roles(
        rel_kind: &IamRelKind,
        with_sub: bool,
        res_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_from_simple_rels(rel_kind, with_sub, res_id, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn find_to_simple_rel_roles(
        rel_kind: &IamRelKind,
        res_id: &str,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_to_simple_rels(rel_kind, res_id, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn paginate_from_simple_rel_roles(
        rel_kind: &IamRelKind,
        res_id: &str,
        with_sub: bool,
        page_number: u64,
        page_size: u64,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        IamRelServ::paginate_from_simple_rels(rel_kind, with_sub, res_id, page_number, page_size, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn paginate_to_simple_rel_roles(
        rel_kind: &IamRelKind,
        res_id: &str,
        page_number: u64,
        page_size: u64,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelBoneResp>> {
        IamRelServ::paginate_to_simple_rels(rel_kind, res_id, page_number, page_size, desc_by_create, desc_by_update, funs, ctx).await
    }

    pub async fn add_res_agg(add_req: &mut IamResAggAddReq, set_id: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        if add_req.res.kind == IamResKind::Menu {
            let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
            // todo: check menu cate
            let menu_ids = &Self::find_id_items(
                &IamResFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    kind: Some(IamResKind::Menu),
                    ..Default::default()
                },
                None,
                None,
                funs,
                ctx,
            )
            .await?;
            let count = RbumSetItemServ::count_rbums(
                &RbumSetItemFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ..Default::default()
                    },
                    sys_code_query_kind: Some(RbumSetCateLevelQueryKind::Sub),
                    rel_rbum_set_cate_sys_codes: Some(vec![String::from_utf8(vec![b'0'; set_cate_sys_code_node_len])?]),
                    rel_rbum_item_ids: Some(menu_ids.iter().map(|id| id.to_string()).collect()),
                    rel_rbum_set_id: Some(set_id.to_string()),
                    rel_rbum_set_cate_ids: Some(vec![add_req.set.set_cate_id.to_string()]),
                    ..Default::default()
                },
                funs,
                ctx,
            )
            .await?;
            if count > 0 {
                return Err(funs.err().bad_request(&Self::get_obj_name(), "add", "conflict error", "409-iam-cate-menu-conflict"));
            }
        }
        let res_id = Self::add_item(&mut add_req.res, funs, ctx).await?;
        IamSetServ::add_set_item(
            &IamSetItemAddReq {
                set_id: set_id.to_string(),
                set_cate_id: add_req.set.set_cate_id.to_string(),
                sort: 0,
                rel_rbum_item_id: res_id.clone(),
            },
            funs,
            ctx,
        )
        .await?;
        Ok(res_id)
    }

    pub async fn get_res_by_app(app_ids: Vec<String>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<HashMap<String, Vec<IamResSummaryResp>>> {
        let raw_roles = IamAccountServ::find_simple_rel_roles(&ctx.owner, true, Some(true), None, funs, ctx).await?;
        let mut roles: Vec<RbumRelBoneResp> = vec![];
        let mut result = HashMap::new();
        for role in raw_roles {
            if !IamRoleServ::is_disabled(&role.rel_id, funs).await? {
                roles.push(role)
            }
        }
        let global_ctx = IamCertServ::use_sys_ctx_unsafe(ctx.clone())?;
        for app_id in app_ids {
            let mut res_ids = HashSet::new();
            let app_ctx = IamCertServ::try_use_app_ctx(ctx.clone(), Some(app_id.clone()))?;
            let app_role_ids =
                roles.iter().filter(|r| r.rel_own_paths == app_ctx.own_paths || r.rel_own_paths == ctx.own_paths).map(|r| r.rel_id.to_string()).collect::<Vec<String>>();
            // todo default empty res
            res_ids.insert("".to_string());
            for role_id in app_role_ids {
                let rel_res_ids = IamRelServ::find_to_id_rels(&IamRelKind::IamResRole, &role_id, None, None, funs, &global_ctx).await?;
                res_ids.extend(rel_res_ids.into_iter());
            }
            let res = Self::find_items(
                &IamResFilterReq {
                    basic: RbumBasicFilterReq {
                        with_sub_own_paths: true,
                        ids: Some(res_ids.into_iter().collect()),
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
            result.insert(app_id, res);
        }
        Ok(result)
    }
}

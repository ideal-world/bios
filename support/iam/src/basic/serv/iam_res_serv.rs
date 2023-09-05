use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use bios_basic::rbum::rbum_config::RbumConfigApi;
use bios_basic::rbum::rbum_enumeration::RbumSetCateLevelQueryKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::{RbumSetCateServ, RbumSetItemServ};
use ldap3::log::warn;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::{Expr, SelectStatement};
use tardis::db::sea_orm::*;
use tardis::futures::future::BoxFuture;
use tardis::futures::FutureExt;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetItemFilterReq};
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::dto::rbum_set_cate_dto::RbumSetCateAddReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_res;
use crate::basic::dto::iam_filer_dto::IamResFilterReq;
use crate::basic::dto::iam_res_dto::{IamResAddReq, IamResAggAddReq, IamResDetailResp, IamResModifyReq, IamResSummaryResp, JsonMenu, MenuItem};
use crate::basic::dto::iam_set_dto::{IamSetItemAddReq, IamSetItemAggAddReq};
use crate::basic::serv::iam_key_cache_serv::IamResCacheServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_set_serv::IamSetServ;
use crate::iam_config::IamBasicInfoManager;
use crate::iam_constants;
use crate::iam_enumeration::{IamRelKind, IamResKind, IamSetCateKind};

use super::clients::iam_log_client::{IamLogClient, LogParamTag};
use super::iam_account_serv::IamAccountServ;
use super::iam_cert_serv::IamCertServ;
use super::iam_key_cache_serv::IamCacheResRelAddOrModifyReq;
use super::iam_role_serv::IamRoleServ;

pub struct IamResServ;

pub struct IamMenuServ;

#[async_trait]
impl RbumItemCrudOperation<iam_res::ActiveModel, IamResAddReq, IamResModifyReq, IamResSummaryResp, IamResDetailResp, IamResFilterReq> for IamResServ {
    fn get_ext_table_name() -> &'static str {
        iam_res::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.kind_res_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(IamBasicInfoManager::get_config(|conf| conf.domain_iam_id.clone()))
    }

    async fn package_item_add(add_req: &IamResAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            code: Some(add_req.code.clone()),
            name: add_req.name.clone(),
            disabled: add_req.disabled,
            scope_level: add_req.scope_level.clone(),
            ..Default::default()
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
            crypto_req: Set(add_req.crypto_req.unwrap_or(false)),
            crypto_resp: Set(add_req.crypto_resp.unwrap_or(false)),
            double_auth: Set(add_req.double_auth.unwrap_or(false)),
            double_auth_msg: Set(add_req.double_auth_msg.as_ref().unwrap_or(&"".to_string()).to_string()),
            need_login: Set(add_req.need_login.unwrap_or(false)),
            ..Default::default()
        })
    }

    async fn before_add_item(add_req: &mut IamResAddReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        add_req.encoding();
        Ok(())
    }

    async fn after_add_item(id: &str, _: &mut IamResAddReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
            IamResCacheServ::add_res(&res.code, &res.method, res.crypto_req, res.crypto_resp, res.double_auth, res.need_login, funs).await?;
        }
        let (op_describe, op_kind) = match res.kind {
            IamResKind::Menu => ("添加目录页面".to_string(), "AddContentPageaspersonal".to_string()),
            IamResKind::Api => ("添加API".to_string(), "AddApi".to_string()),
            IamResKind::Ele => ("添加目录页面按钮".to_string(), "AddContentPageButton".to_string()),
        };
        if !op_describe.is_empty() {
            let _ = IamLogClient::add_ctx_task(LogParamTag::IamRes, Some(id.to_string()), op_describe, Some(op_kind), ctx).await;
        }

        Ok(())
    }

    async fn package_item_modify(_: &str, modify_req: &IamResModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
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

    async fn package_ext_modify(id: &str, modify_req: &IamResModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<iam_res::ActiveModel>> {
        if modify_req.icon.is_none()
            && modify_req.sort.is_none()
            && modify_req.hide.is_none()
            && modify_req.action.is_none()
            && modify_req.crypto_req.is_none()
            && modify_req.crypto_resp.is_none()
            && modify_req.double_auth.is_none()
        {
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
        if let Some(crypto_req) = modify_req.crypto_req {
            iam_res.crypto_req = Set(crypto_req);
        }
        if let Some(crypto_resp) = modify_req.crypto_resp {
            iam_res.crypto_resp = Set(crypto_resp);
        }
        if let Some(double_auth) = modify_req.double_auth {
            iam_res.double_auth = Set(double_auth);
        }
        if let Some(double_auth_msg) = &modify_req.double_auth_msg {
            iam_res.double_auth_msg = Set(double_auth_msg.to_string());
        }
        if let Some(need_login) = modify_req.need_login {
            iam_res.need_login = Set(need_login);
        }
        Ok(Some(iam_res))
    }

    async fn after_modify_item(id: &str, modify_req: &mut IamResModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
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
        if modify_req.crypto_req.is_some() || modify_req.crypto_resp.is_some() || modify_req.double_auth.is_some() {
            IamResCacheServ::add_or_modify_res_rel(
                &res.code,
                &res.method,
                &IamCacheResRelAddOrModifyReq {
                    st: None,
                    et: None,
                    accounts: vec![],
                    roles: vec![],
                    groups: vec![],
                    apps: vec![],
                    tenants: vec![],
                    need_crypto_req: modify_req.crypto_req,
                    need_crypto_resp: modify_req.crypto_resp,
                    need_double_auth: modify_req.double_auth,
                    need_login: modify_req.need_login,
                },
                funs,
            )
            .await?;
        }
        if let Some(disabled) = modify_req.disabled {
            if res.kind == IamResKind::Api {
                if disabled {
                    IamResCacheServ::delete_res(&res.code, &res.method, funs).await?;
                } else {
                    IamResCacheServ::add_res(&res.code, &res.method, res.crypto_req, res.crypto_resp, res.double_auth, res.need_login, funs).await?;
                }
            }
        }

        let (op_describe, op_kind) = match res.kind {
            IamResKind::Menu => ("编辑目录页面".to_string(), "ModifyContentPage".to_string()),
            IamResKind::Api => ("编辑API".to_string(), "ModifyApi".to_string()),
            IamResKind::Ele => ("".to_string(), "".to_string()),
        };
        if !op_describe.is_empty() {
            let _ = IamLogClient::add_ctx_task(LogParamTag::IamRes, Some(id.to_string()), op_describe, Some(op_kind), ctx).await;
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

    async fn after_delete_item(_: &str, deleted_item: &Option<IamResDetailResp>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        if let Some(deleted_item) = deleted_item {
            if deleted_item.kind == IamResKind::Api {
                IamResCacheServ::delete_res(&deleted_item.code, &deleted_item.method, funs).await?;
            }
            let (op_describe, op_kind) = match deleted_item.kind {
                IamResKind::Menu => ("删除目录页面".to_string(), "DeleteContentPageAsPersonal".to_string()),
                IamResKind::Api => ("删除API".to_string(), "DeleteApi".to_string()),
                IamResKind::Ele => ("移除目录页面按钮".to_string(), "RemoveContentPageButton".to_string()),
            };
            if !op_describe.is_empty() {
                let _ = IamLogClient::add_ctx_task(LogParamTag::IamRes, Some(deleted_item.id.to_string()), op_describe, Some(op_kind), ctx).await;
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
        query.column((iam_res::Entity, iam_res::Column::CryptoReq));
        query.column((iam_res::Entity, iam_res::Column::CryptoResp));
        query.column((iam_res::Entity, iam_res::Column::DoubleAuth));
        query.column((iam_res::Entity, iam_res::Column::DoubleAuthMsg));
        query.column((iam_res::Entity, iam_res::Column::NeedLogin));
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
        page_number: u32,
        page_size: u32,
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
        page_number: u32,
        page_size: u32,
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
        page_number: u32,
        page_size: u32,
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
        page_number: u32,
        page_size: u32,
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
                if role_id.contains(":") {
                    let extend_role_id = role_id.split(":").collect::<Vec<&str>>()[0];
                    let rel_res_ids = IamRelServ::find_to_id_rels(&IamRelKind::IamResRole, &extend_role_id, None, None, funs, &global_ctx).await?;
                    res_ids.extend(rel_res_ids.into_iter());
                }
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

impl IamMenuServ {
    pub fn parse_menu<'a>(set_id: &'a str, parent_cate_id: &'a str, json_menu: JsonMenu, funs: &'a TardisFunsInst, ctx: &'a TardisContext) -> BoxFuture<'a, TardisResult<String>> {
        async move {
            let new_cate_id = Self::add_cate_menu(
                set_id,
                parent_cate_id,
                &json_menu.name,
                &json_menu.bus_code,
                &IamSetCateKind::parse(&json_menu.ext)?,
                funs,
                ctx,
            )
            .await?;
            if let Some(items) = json_menu.items {
                for item in items {
                    Self::parse_item(set_id, &new_cate_id, item, funs, ctx).await?;
                }
            };
            if let Some(children_menus) = json_menu.children {
                for children_menu in children_menus {
                    Self::parse_menu(set_id, &new_cate_id, children_menu, funs, ctx).await?;
                }
            };
            Ok(new_cate_id)
        }
        .boxed()
    }

    async fn add_cate_menu<'a>(
        set_id: &str,
        parent_cate_menu_id: &str,
        name: &str,
        bus_code: &str,
        ext: &IamSetCateKind,
        funs: &TardisFunsInst,
        ctx: &TardisContext,
    ) -> TardisResult<String> {
        RbumSetCateServ::add_rbum(
            &mut RbumSetCateAddReq {
                name: TrimString(name.to_string()),
                bus_code: TrimString(bus_code.to_string()),
                icon: None,
                sort: None,
                ext: Some(ext.to_string()),
                rbum_parent_cate_id: Some(parent_cate_menu_id.to_string()),
                rel_rbum_set_id: set_id.to_string(),
                scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_GLOBAL),
            },
            funs,
            ctx,
        )
        .await
    }

    async fn parse_item(set_id: &str, cate_menu_id: &str, item: MenuItem, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        let id = match &item.kind as &str {
            "Menu" => Self::add_menu_res(set_id, cate_menu_id, &item.name, &item.code, funs, ctx).await?,
            "Ele" => Self::add_ele_res(set_id, cate_menu_id, &item.name, &item.code, funs, ctx).await?,
            _ => {
                warn!("item({},{}) have unsupported kind {} !", &item.name, &item.code, &item.kind);
                "".to_string()
            }
        };
        Ok(id)
    }
    async fn add_menu_res<'a>(set_id: &str, cate_menu_id: &str, name: &str, code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        IamResServ::add_res_agg(
            &mut IamResAggAddReq {
                res: IamResAddReq {
                    code: TrimString(code.to_string()),
                    name: TrimString(name.to_string()),
                    kind: IamResKind::Menu,
                    icon: None,
                    sort: None,
                    method: None,
                    hide: None,
                    action: None,
                    scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_GLOBAL),
                    disabled: None,
                    crypto_req: None,
                    crypto_resp: None,
                    double_auth: None,
                    double_auth_msg: None,
                    need_login: None,
                },
                set: IamSetItemAggAddReq {
                    set_cate_id: cate_menu_id.to_string(),
                },
            },
            set_id,
            funs,
            ctx,
        )
        .await
    }

    async fn add_ele_res<'a>(set_id: &str, cate_menu_id: &str, name: &str, code: &str, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<String> {
        IamResServ::add_res_agg(
            &mut IamResAggAddReq {
                res: IamResAddReq {
                    code: TrimString(code.to_string()),
                    name: TrimString(name.to_string()),
                    kind: IamResKind::Ele,
                    icon: None,
                    sort: None,
                    method: None,
                    hide: None,
                    action: None,
                    scope_level: Some(iam_constants::RBUM_SCOPE_LEVEL_GLOBAL),
                    disabled: None,
                    crypto_req: None,
                    crypto_resp: None,
                    double_auth: None,
                    double_auth_msg: None,
                    need_login: None,
                },
                set: IamSetItemAggAddReq {
                    set_cate_id: cate_menu_id.to_string(),
                },
            },
            set_id,
            funs,
            ctx,
        )
        .await
    }
}

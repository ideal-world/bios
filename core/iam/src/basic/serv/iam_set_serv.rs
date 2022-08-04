use std::collections::HashMap;

use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::{TardisFuns, TardisFunsInst};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetItemFilterReq, RbumSetTreeFilterReq};
use bios_basic::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateModifyReq};
use bios_basic::rbum::dto::rbum_set_dto::{RbumSetAddReq, RbumSetPathResp, RbumSetTreeResp};
use bios_basic::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemDetailResp, RbumSetItemModifyReq};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::rbum_config::RbumConfigApi;
use bios_basic::rbum::rbum_enumeration::{RbumScopeLevelKind, RbumSetCateLevelQueryKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::{RbumSetCateServ, RbumSetItemServ, RbumSetServ};

use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq};
use crate::iam_config::IamBasicConfigApi;
use crate::iam_constants::{RBUM_SCOPE_LEVEL_APP, RBUM_SCOPE_LEVEL_TENANT};
use crate::iam_enumeration::IamSetKind;

const SET_AND_ITEM_SPLIT_FLAG: &str = ":";

pub struct IamSetServ;

impl<'a> IamSetServ {
    pub async fn init_set(
        set_kind: IamSetKind,
        scope_level: RbumScopeLevelKind,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<(String, Option<(String, String)>)> {
        let code = Self::get_default_code(&set_kind, &ctx.own_paths);
        let set_id = RbumSetServ::add_rbum(
            &mut RbumSetAddReq {
                code: TrimString(code.clone()),
                kind: TrimString(set_kind.to_string()),
                name: TrimString(code),
                note: None,
                icon: None,
                sort: None,
                ext: None,
                scope_level: Some(scope_level.clone()),
                disabled: None,
            },
            funs,
            ctx,
        )
        .await?;
        let cates = if set_kind == IamSetKind::Res {
            let cate_menu_id = RbumSetCateServ::add_rbum(
                &mut RbumSetCateAddReq {
                    name: TrimString("Menus".to_string()),
                    bus_code: TrimString("__menus__".to_string()),
                    icon: None,
                    sort: None,
                    ext: None,
                    rbum_parent_cate_id: None,
                    rel_rbum_set_id: set_id.clone(),
                    scope_level: Some(scope_level.clone()),
                },
                funs,
                ctx,
            )
            .await?;
            let cate_api_id = RbumSetCateServ::add_rbum(
                &mut RbumSetCateAddReq {
                    name: TrimString("Apis".to_string()),
                    bus_code: TrimString("__apis__".to_string()),
                    icon: None,
                    sort: None,
                    ext: None,
                    rbum_parent_cate_id: None,
                    rel_rbum_set_id: set_id.clone(),
                    scope_level: Some(scope_level.clone()),
                },
                funs,
                ctx,
            )
            .await?;
            Some((cate_menu_id, cate_api_id))
        } else {
            None
        };
        Ok((set_id, cates))
    }

    pub async fn get_default_set_id_by_ctx(kind: &IamSetKind, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<String> {
        Self::get_set_id_by_code(&Self::get_default_code(kind, &ctx.own_paths), true, funs, ctx).await
    }

    pub async fn get_set_id_by_code(code: &str, with_sub: bool, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<String> {
        RbumSetServ::get_rbum_set_id_by_code(code, with_sub, funs, ctx)
            .await?
            .ok_or_else(|| funs.err().not_found("iam_set", "get_id", &format!("not found set by code {}", code), "404-rbum-set-code-not-exist"))
    }

    pub fn get_default_org_code_by_system() -> String {
        Self::get_default_code(&IamSetKind::Org, "")
    }

    pub fn get_default_org_code_by_tenant(funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<String> {
        if let Some(own_paths) = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_TENANT.to_int(), &ctx.own_paths) {
            Ok(Self::get_default_code(&IamSetKind::Org, &own_paths))
        } else {
            Err(funs.err().not_found("iam_set", "get_org_code", "not found tenant own_paths", "404-rbum-set-code-not-exist"))
        }
    }

    pub fn get_default_org_code_by_app(funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<String> {
        if let Some(own_paths) = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_APP.to_int(), &ctx.own_paths) {
            Ok(Self::get_default_code(&IamSetKind::Org, &own_paths))
        } else {
            Err(funs.err().not_found("iam_set", "get_org_code", "not found app own_paths", "404-rbum-set-code-not-exist"))
        }
    }

    pub fn get_default_code(kind: &IamSetKind, own_paths: &str) -> String {
        format!("{}:{}", own_paths, kind.to_string().to_lowercase())
    }

    pub async fn add_set_cate(set_id: &str, add_req: &IamSetCateAddReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<String> {
        RbumSetCateServ::add_rbum(
            &mut RbumSetCateAddReq {
                name: add_req.name.clone(),
                bus_code: add_req.bus_code.as_ref().unwrap_or(&TrimString("".to_string())).clone(),
                icon: add_req.icon.clone(),
                sort: add_req.sort,
                ext: add_req.ext.clone(),
                rbum_parent_cate_id: add_req.rbum_parent_cate_id.clone(),
                rel_rbum_set_id: set_id.to_string(),
                scope_level: add_req.scope_level.clone(),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn modify_set_cate(set_cate_id: &str, modify_req: &IamSetCateModifyReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
        RbumSetCateServ::modify_rbum(
            set_cate_id,
            &mut RbumSetCateModifyReq {
                bus_code: modify_req.bus_code.clone(),
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                sort: modify_req.sort,
                ext: modify_req.ext.clone(),
                scope_level: modify_req.scope_level.clone(),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn delete_set_cate(set_cate_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<u64> {
        RbumSetCateServ::delete_rbum(set_cate_id, funs, ctx).await
    }

    pub async fn get_tree(set_id: &str, filter: &mut RbumSetTreeFilterReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<Vec<RbumSetTreeResp>> {
        filter.rel_rbum_item_domain_ids = Some(vec![funs.iam_basic_domain_iam_id()]);
        RbumSetServ::get_tree(set_id, filter, funs, ctx).await
    }

    pub async fn get_tree_with_auth_by_account(set_id: &str, account_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<Vec<RbumSetTreeResp>> {
        let account_rel_sys_codes = Self::get_tree(
            set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                rel_rbum_item_ids: Some(vec![account_id.to_string()]),
                rel_rbum_item_kind_ids: Some(vec![funs.iam_basic_kind_account_id()]),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await?
        .into_iter()
        .filter(|cate| !cate.rbum_set_items.is_empty())
        .map(|cate| cate.sys_code)
        .collect::<Vec<String>>();
        if account_rel_sys_codes.is_empty() {
            return Ok(vec![]);
        }
        Self::get_tree(
            set_id,
            &mut RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_codes: Some(account_rel_sys_codes),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn get_menu_tree(set_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<Vec<RbumSetTreeResp>> {
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let menu_sys_code = String::from_utf8(vec![b'0'; set_cate_sys_code_node_len])?;
        Self::get_tree_with_sys_code(set_id, &menu_sys_code, funs, ctx).await
    }

    pub async fn get_api_tree(set_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<Vec<RbumSetTreeResp>> {
        let set_cate_sys_code_node_len = funs.rbum_conf_set_cate_sys_code_node_len();
        let api_sys_code = TardisFuns::field.incr_by_base36(&String::from_utf8(vec![b'0'; set_cate_sys_code_node_len])?).unwrap();
        Self::get_tree_with_sys_code(set_id, &api_sys_code, funs, ctx).await
    }

    async fn get_tree_with_sys_code(set_id: &str, filter_sys_code: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<Vec<RbumSetTreeResp>> {
        RbumSetServ::get_tree(
            set_id,
            &RbumSetTreeFilterReq {
                fetch_cate_item: true,
                sys_codes: Some(vec![filter_sys_code.to_string()]),
                sys_code_query_kind: Some(RbumSetCateLevelQueryKind::CurrentAndSub),
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn add_set_item(add_req: &IamSetItemAddReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<String> {
        RbumSetItemServ::add_rbum(
            &mut RbumSetItemAddReq {
                sort: add_req.sort,
                rel_rbum_set_id: add_req.set_id.clone(),
                rel_rbum_set_cate_id: add_req.set_cate_id.clone(),
                rel_rbum_item_id: add_req.rel_rbum_item_id.clone(),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn modify_set_item(set_item_id: &str, modify_req: &mut RbumSetItemModifyReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
        RbumSetItemServ::modify_rbum(set_item_id, modify_req, funs, ctx).await
    }

    pub async fn delete_set_item(set_item_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<u64> {
        RbumSetItemServ::delete_rbum(set_item_id, funs, ctx).await
    }

    pub async fn find_set_items(
        set_id: Option<String>,
        set_cate_id: Option<String>,
        item_id: Option<String>,
        with_sub: bool,
        funs: &TardisFunsInst<'a>,
        ctx: &TardisContext,
    ) -> TardisResult<Vec<RbumSetItemDetailResp>> {
        RbumSetItemServ::find_detail_rbums(
            &RbumSetItemFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                rel_rbum_set_id: set_id.clone(),
                rel_rbum_set_cate_ids: set_cate_id.map(|r| vec![r]),
                rel_rbum_item_ids: item_id.map(|i| vec![i]),
                ..Default::default()
            },
            None,
            None,
            funs,
            ctx,
        )
        .await
    }

    pub async fn find_set_paths(set_item_id: &str, set_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<Vec<Vec<RbumSetPathResp>>> {
        RbumSetItemServ::find_set_paths(set_item_id, set_id, funs, ctx).await
    }

    pub async fn find_flat_set_items(set_id: &str, item_id: &str, with_sub: bool, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
        let items = Self::find_set_items(Some(set_id.to_string()), None, Some(item_id.to_string()), with_sub, funs, ctx).await?;
        let items = items
            .into_iter()
            .map(|item| {
                (
                    format!("{}{}{}", item.rel_rbum_set_id, SET_AND_ITEM_SPLIT_FLAG, item.rel_rbum_set_cate_sys_code),
                    item.rel_rbum_set_cate_name,
                )
            })
            .collect();
        Ok(items)
    }

    pub async fn check_scope(app_id: &str, account_id: &str, set_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<bool> {
        RbumSetItemServ::check_a_is_parent_or_sibling_of_b(account_id, app_id, set_id, funs, ctx).await
    }
}

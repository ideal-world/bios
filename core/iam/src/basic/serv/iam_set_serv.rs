use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetItemFilterReq};
use bios_basic::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateModifyReq, RbumSetTreeResp};
use bios_basic::rbum::dto::rbum_set_dto::RbumSetAddReq;
use bios_basic::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemModifyReq, RbumSetItemSummaryResp};
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::{RbumSetCateServ, RbumSetItemServ, RbumSetServ};

use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq};

pub struct IamSetServ;

impl<'a> IamSetServ {
    pub async fn init_set(is_org: bool, scope_level: RbumScopeLevelKind, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let code = if is_org { Self::get_default_org_code_by_cxt(cxt) } else { Self::get_default_res_code_by_cxt(cxt) };
        let set_id = RbumSetServ::add_rbum(
            &mut RbumSetAddReq {
                code: TrimString(code.clone()),
                name: TrimString(code),
                note: None,
                icon: None,
                sort: None,
                ext: None,
                scope_level: Some(scope_level.clone()),
                disabled: None,
            },
            funs,
            cxt,
        )
        .await?;
        if !is_org {
            RbumSetCateServ::add_rbum(
                &mut RbumSetCateAddReq {
                    name: TrimString("Menus".to_string()),
                    bus_code: TrimString("".to_string()),
                    icon: None,
                    sort: None,
                    ext: None,
                    rbum_parent_cate_id: None,
                    rel_rbum_set_id: set_id.clone(),
                    scope_level: Some(scope_level.clone()),
                },
                funs,
                cxt,
            )
            .await?;
            RbumSetCateServ::add_rbum(
                &mut RbumSetCateAddReq {
                    name: TrimString("Apis".to_string()),
                    bus_code: TrimString("".to_string()),
                    icon: None,
                    sort: None,
                    ext: None,
                    rbum_parent_cate_id: None,
                    rel_rbum_set_id: set_id.clone(),
                    scope_level: Some(scope_level.clone()),
                },
                funs,
                cxt,
            )
            .await?;
        }
        Ok(set_id)
    }

    pub async fn get_default_set_id_by_cxt(is_org: bool, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let code = if is_org { Self::get_default_org_code_by_cxt(cxt) } else { Self::get_default_res_code_by_cxt(cxt) };
        Self::get_set_id_by_code(&code, funs, cxt).await
    }

    pub async fn get_set_id_by_code(code: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let resp = RbumSetServ::find_rbums(
            &RbumBasicFilterReq {
                code: Some(code.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            cxt,
        )
        .await?;
        let id = resp.get(0).map(|x| x.id.clone()).ok_or_else(|| TardisError::NotFound(format!("set {} not found", code)))?;
        Ok(id)
    }

    pub async fn add_set_cate(set_id: &str, add_req: &IamSetCateAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        RbumSetCateServ::add_rbum(
            &mut RbumSetCateAddReq {
                name: add_req.name.clone(),
                bus_code: add_req.bus_code.clone(),
                icon: add_req.icon.clone(),
                sort: add_req.sort,
                ext: add_req.ext.clone(),
                rbum_parent_cate_id: add_req.rbum_parent_cate_id.clone(),
                rel_rbum_set_id: set_id.to_string(),
                scope_level: add_req.scope_level.clone(),
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn modify_set_cate(set_cate_id: &str, modify_req: &IamSetCateModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
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
            cxt,
        )
        .await
    }

    pub async fn delete_set_cate(set_cate_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        RbumSetCateServ::delete_rbum(set_cate_id, funs, cxt).await
    }

    pub async fn find_set_cates(set_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<Vec<RbumSetTreeResp>> {
        RbumSetServ::get_tree_all(set_id, funs, cxt).await
    }

    pub async fn add_set_item(add_req: &IamSetItemAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        RbumSetItemServ::add_rbum(
            &mut RbumSetItemAddReq {
                sort: add_req.sort,
                rel_rbum_set_id: add_req.set_id.clone(),
                rel_rbum_set_cate_id: add_req.set_cate_id.clone(),
                rel_rbum_item_id: add_req.rel_rbum_item_id.clone(),
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn modify_set_item(set_item_id: &str, modify_req: &mut RbumSetItemModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        RbumSetItemServ::modify_rbum(set_item_id, modify_req, funs, cxt).await
    }

    pub async fn delete_set_item(set_item_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        RbumSetItemServ::delete_rbum(set_item_id, funs, cxt).await
    }

    pub async fn find_set_items(set_id: Option<String>, set_cate_id: Option<String>, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<Vec<RbumSetItemSummaryResp>> {
        RbumSetItemServ::find_rbums(
            &RbumSetItemFilterReq {
                basic: Default::default(),
                rel_rbum_set_id: set_id.clone(),
                rel_rbum_set_cate_id: set_cate_id.clone(),
                rel_rbum_item_id: None,
            },
            None,
            None,
            funs,
            cxt,
        )
        .await
    }

    pub fn get_default_res_code_by_cxt(cxt: &TardisContext) -> String {
        format!("{}:{}", cxt.own_paths, "res")
    }

   pub fn get_default_org_code_by_cxt(cxt: &TardisContext) -> String {
        format!("{}:{}", cxt.own_paths, "org")
    }
}

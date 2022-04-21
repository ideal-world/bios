use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumSetItemFilterReq};
use bios_basic::rbum::dto::rbum_set_cate_dto::{RbumSetCateAddReq, RbumSetCateModifyReq, RbumSetCateSummaryWithPidResp};
use bios_basic::rbum::dto::rbum_set_dto::RbumSetAddReq;
use bios_basic::rbum::dto::rbum_set_item_dto::{RbumSetItemAddReq, RbumSetItemDetailResp, RbumSetItemModifyReq};
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_set_serv::{RbumSetCateServ, RbumSetItemServ, RbumSetServ};

use crate::basic::dto::iam_set_dto::{IamSetCateAddReq, IamSetCateModifyReq, IamSetItemAddReq};

pub struct IamSetServ;

impl<'a> IamSetServ {
    pub async fn init_set(is_org: bool, scope_level: RbumScopeLevelKind, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let code = if is_org { Self::get_org_code(cxt) } else { Self::get_http_res_code(cxt) };
        RbumSetServ::add_rbum(
            &mut RbumSetAddReq {
                code: TrimString(code.clone()),
                name: TrimString(code),
                note: None,
                icon: None,
                sort: None,
                ext: None,
                scope_level,
                disabled: None,
            },
            funs,
            cxt,
        )
        .await
    }

    async fn get_set(is_org: bool, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let code = if is_org { Self::get_org_code(cxt) } else { Self::get_http_res_code(cxt) };
        let resp = RbumSetServ::find_rbums(
            &RbumBasicFilterReq {
                code: Some(code.clone()),
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

    pub async fn add_set_cate(add_req: &IamSetCateAddReq, is_org: bool, scope_level: RbumScopeLevelKind, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let id = Self::get_set(is_org, funs, cxt).await?;
        RbumSetCateServ::add_rbum(
            &mut RbumSetCateAddReq {
                bus_code: add_req.bus_code.clone(),
                name: add_req.name.clone(),
                icon: add_req.icon.clone(),
                sort: add_req.sort,
                ext: add_req.ext.clone(),
                rbum_parent_cate_id: add_req.rbum_parent_cate_id.clone(),
                rel_rbum_set_id: id,
                scope_level,
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn modify_set_cate(
        set_cate_id: &str,
        modify_req: &IamSetCateModifyReq,
        scope_level: Option<RbumScopeLevelKind>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<()> {
        RbumSetCateServ::modify_rbum(
            set_cate_id,
            &mut RbumSetCateModifyReq {
                bus_code: modify_req.bus_code.clone(),
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                sort: modify_req.sort,
                ext: modify_req.ext.clone(),
                scope_level,
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn delete_set_cate(set_cate_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        RbumSetCateServ::delete_rbum(set_cate_id, funs, cxt).await
    }

    pub async fn find_set_cates(is_org: bool, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<Vec<RbumSetCateSummaryWithPidResp>> {
        let id = Self::get_set(is_org, funs, cxt).await?;
        RbumSetServ::get_tree_all(&id, funs, cxt).await
    }

    pub async fn add_set_item(set_cate_id: &str, add_req: &IamSetItemAddReq, is_org: bool, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let id = Self::get_set(is_org, funs, cxt).await?;
        RbumSetItemServ::add_rbum(
            &mut RbumSetItemAddReq {
                sort: add_req.sort,
                rel_rbum_set_id: id,
                rel_rbum_set_cate_id: set_cate_id.to_string(),
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

    pub async fn find_set_items(set_cate_id: &str, is_org: bool, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<Vec<RbumSetItemDetailResp>> {
        let id = Self::get_set(is_org, funs, cxt).await?;
        RbumSetItemServ::find_rbums(
            &RbumSetItemFilterReq {
                basic: Default::default(),
                rel_rbum_set_id: Some(id),
                rel_rbum_set_cate_id: Some(set_cate_id.to_string()),
                rel_rbum_item_id: None,
            },
            None,
            None,
            funs,
            cxt,
        )
        .await
    }

    fn get_http_res_code(cxt: &TardisContext) -> String {
        format!("{}:{}", cxt.own_paths, "http_res")
    }

    fn get_org_code(cxt: &TardisContext) -> String {
        format!("{}:{}", cxt.own_paths, "org")
    }
}

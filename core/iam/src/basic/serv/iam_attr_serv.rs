use std::collections::HashMap;

use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemAttrFilterReq, RbumKindAttrFilterReq};
use bios_basic::rbum::dto::rbum_item_attr_dto::{RbumItemAttrAddReq, RbumItemAttrDetailResp, RbumItemAttrModifyReq, RbumItemAttrSummaryResp, RbumItemAttrsAddOrModifyReq};
use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrAddReq, RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemAttrServ;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindAttrServ;

use crate::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use crate::iam_config::IamBasicInfoManager;

pub struct IamAttrServ;

impl<'a> IamAttrServ {
    pub async fn add_account_attr(add_req: &IamKindAttrAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        RbumKindAttrServ::add_rbum(
            &mut RbumKindAttrAddReq {
                name: add_req.name.clone(),
                label: add_req.label.clone(),
                note: add_req.note.clone(),
                sort: add_req.sort,
                main_column: add_req.main_column,
                position: add_req.position,
                capacity: add_req.capacity,
                overload: add_req.overload,
                idx: add_req.idx,
                data_type: add_req.data_type.clone(),
                widget_type: add_req.widget_type.clone(),
                default_value: add_req.default_value.clone(),
                options: add_req.options.clone(),
                required: add_req.required,
                min_length: add_req.min_length,
                max_length: add_req.max_length,
                action: add_req.action.clone(),
                ext: add_req.ext.clone(),
                rel_rbum_kind_id: IamBasicInfoManager::get().kind_account_id,
                scope_level: add_req.scope_level.clone(),
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn modify_account_attr(id: &str, modify_req: &mut RbumKindAttrModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        RbumKindAttrServ::modify_rbum(id, modify_req, funs, cxt).await
    }

    pub async fn get_account_attr(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<RbumKindAttrDetailResp> {
        RbumKindAttrServ::get_rbum(
            id,
            &RbumKindAttrFilterReq {
                basic: RbumBasicFilterReq {
                    rbum_kind_id: Some(IamBasicInfoManager::get().kind_account_id),
                    ..Default::default()
                },
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn find_account_attrs(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<Vec<RbumKindAttrSummaryResp>> {
        RbumKindAttrServ::find_rbums(
            &RbumKindAttrFilterReq {
                basic: RbumBasicFilterReq {
                    rbum_kind_id: Some(IamBasicInfoManager::get().kind_account_id),
                    desc_by_sort: Some(true),
                    ..Default::default()
                },
            },
            None,
            None,
            funs,
            cxt,
        )
        .await
    }

    pub async fn delete_account_attr(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        RbumKindAttrServ::delete_rbum(id, funs, cxt).await
    }

    pub async fn find_account_ext_attr_values(
        rel_account_id: &str,
        with_sub_own_paths: bool,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumItemAttrSummaryResp>> {
        RbumItemAttrServ::find_rbums(
            &RbumItemAttrFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths,
                    ..Default::default()
                },
                rel_rbum_item_id: Some(rel_account_id.to_string()),
                ..Default::default()
            },
            None,
            None,
            funs,
            cxt,
        )
        .await
    }

    pub async fn add_or_modify_account_attr_values(rel_account_id: &str, values: HashMap<String, String>, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        RbumItemAttrServ::add_or_modify_item_attrs(
            &RbumItemAttrsAddOrModifyReq {
                values,
                rel_rbum_item_id: rel_account_id.to_string(),
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn add_account_attr_value(value: String, attr_id: &str, account_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        RbumItemAttrServ::add_rbum(
            &mut RbumItemAttrAddReq {
                value,
                rel_rbum_item_id: account_id.to_string(),
                rel_rbum_kind_attr_id: attr_id.to_string(),
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn modify_account_attr_value(attr_value_id: &str, value: String, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        RbumItemAttrServ::modify_rbum(attr_value_id, &mut RbumItemAttrModifyReq { value }, funs, cxt).await
    }

    pub async fn get_account_attr_value(attr_value_id: &str, with_sub_own_paths: bool, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<RbumItemAttrDetailResp> {
        RbumItemAttrServ::get_rbum(
            attr_value_id,
            &RbumItemAttrFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn delete_account_attr_value(attr_value_id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        RbumItemAttrServ::delete_rbum(attr_value_id, funs, cxt).await
    }
}

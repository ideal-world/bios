use std::collections::HashMap;

use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::TardisFunsInst;

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemAttrFilterReq, RbumKindAttrFilterReq};
use bios_basic::rbum::dto::rbum_item_attr_dto::{RbumItemAttrAddReq, RbumItemAttrDetailResp, RbumItemAttrModifyReq, RbumItemAttrsAddOrModifyReq};
use bios_basic::rbum::dto::rbum_kind_attr_dto::{RbumKindAttrAddReq, RbumKindAttrDetailResp, RbumKindAttrModifyReq, RbumKindAttrSummaryResp};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemAttrServ;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindAttrServ;

use crate::basic::dto::iam_attr_dto::IamKindAttrAddReq;
use crate::iam_config::IamBasicConfigApi;

pub struct IamAttrServ;

const ACCOUNT_IDX_MAIN_COLUMN_NAMES: [&str; 3] = ["ext1_idx", "ext2_idx", "ext3_idx"];
const ACCOUNT_NO_IDX_MAIN_COLUMN_NAMES: [&str; 6] = ["ext4", "ext5", "ext6", "ext7", "ext8", "ext9"];

impl<'a> IamAttrServ {
    pub async fn add_account_attr(add_req: &IamKindAttrAddReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<String> {
        let idx = add_req.idx.unwrap_or(false);
        let main_column = add_req.main_column.unwrap_or(false);
        let name = add_req.name.0.as_str();
        if idx && !main_column {
            return Err(funs.err().bad_request("account_attr", "add", "only the main table columns support indexes"));
        }
        if idx && !ACCOUNT_IDX_MAIN_COLUMN_NAMES.contains(&name) {
            return Err(funs.err().bad_request("account_attr", "add", "index column name is invalid"));
        }
        if main_column && !(ACCOUNT_IDX_MAIN_COLUMN_NAMES.contains(&name) || ACCOUNT_NO_IDX_MAIN_COLUMN_NAMES.contains(&name)) {
            return Err(funs.err().bad_request("account_attr", "add", "main column name is invalid"));
        }
        RbumKindAttrServ::add_rbum(
            &mut RbumKindAttrAddReq {
                name: add_req.name.clone(),
                module: None,
                label: add_req.label.clone(),
                note: add_req.note.clone(),
                sort: add_req.sort,
                main_column: add_req.main_column,
                position: add_req.position,
                capacity: add_req.capacity,
                overload: add_req.overload,
                hide: None,
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
                rel_rbum_kind_id: funs.iam_basic_kind_account_id(),
                scope_level: add_req.scope_level.clone(),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn modify_account_attr(id: &str, modify_req: &mut RbumKindAttrModifyReq, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
        RbumKindAttrServ::modify_rbum(id, modify_req, funs, ctx).await
    }

    pub async fn get_account_attr(id: &str, with_sub: bool, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<RbumKindAttrDetailResp> {
        RbumKindAttrServ::get_rbum(
            id,
            &RbumKindAttrFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with_sub,
                    rbum_kind_id: Some(funs.iam_basic_kind_account_id()),
                    ..Default::default()
                },
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn find_account_attrs(funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<Vec<RbumKindAttrSummaryResp>> {
        RbumKindAttrServ::find_rbums(
            &RbumKindAttrFilterReq {
                basic: RbumBasicFilterReq {
                    rbum_kind_id: Some(funs.iam_basic_kind_account_id()),
                    desc_by_sort: Some(true),
                    ..Default::default()
                },
            },
            None,
            None,
            funs,
            ctx,
        )
        .await
    }

    pub async fn delete_account_attr(id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<u64> {
        RbumKindAttrServ::delete_rbum(id, funs, ctx).await
    }

    pub async fn find_account_attr_values(account_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<HashMap<String, String>> {
        RbumItemAttrServ::find_item_attr_values(account_id, funs, ctx).await
    }

    pub async fn add_or_modify_account_attr_values(rel_account_id: &str, values: HashMap<String, String>, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
        RbumItemAttrServ::add_or_modify_item_attrs(
            &RbumItemAttrsAddOrModifyReq {
                values,
                rel_rbum_item_id: rel_account_id.to_string(),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn add_account_attr_value(value: String, attr_id: &str, account_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<String> {
        RbumItemAttrServ::add_rbum(
            &mut RbumItemAttrAddReq {
                value,
                rel_rbum_item_id: account_id.to_string(),
                rel_rbum_kind_attr_id: attr_id.to_string(),
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn modify_account_attr_value(attr_value_id: &str, value: String, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<()> {
        RbumItemAttrServ::modify_rbum(attr_value_id, &mut RbumItemAttrModifyReq { value }, funs, ctx).await
    }

    pub async fn get_account_attr_value(attr_value_id: &str, with_sub: bool, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<RbumItemAttrDetailResp> {
        RbumItemAttrServ::get_rbum(
            attr_value_id,
            &RbumItemAttrFilterReq {
                basic: RbumBasicFilterReq {
                    with_sub_own_paths: with_sub,
                    ..Default::default()
                },
                ..Default::default()
            },
            funs,
            ctx,
        )
        .await
    }

    pub async fn delete_account_attr_value(attr_value_id: &str, funs: &TardisFunsInst<'a>, ctx: &TardisContext) -> TardisResult<u64> {
        RbumItemAttrServ::delete_rbum(attr_value_id, funs, ctx).await
    }
}

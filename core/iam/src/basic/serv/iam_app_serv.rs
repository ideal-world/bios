use async_trait::async_trait;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{Expr, SelectStatement};
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemModifyReq};
use bios_basic::rbum::helper::rbum_scope_helper;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_app;
use crate::basic::dto::iam_app_dto::{IamAppAddReq, IamAppDetailResp, IamAppModifyReq, IamAppSummaryResp};
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::iam_config::IamBasicInfoManager;
use crate::iam_constants::{RBUM_ITEM_ID_APP_LEN, RBUM_SCOPE_LEVEL_APP};

pub struct IamAppServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_app::ActiveModel, IamAppAddReq, IamAppModifyReq, IamAppSummaryResp, IamAppDetailResp, IamAppFilterReq> for IamAppServ {
    fn get_ext_table_name() -> &'static str {
        iam_app::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        IamBasicInfoManager::get().kind_app_id.clone()
    }

    fn get_rbum_domain_id() -> String {
        IamBasicInfoManager::get().domain_iam_id.clone()
    }

    async fn package_item_add(add_req: &IamAppAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<RbumItemAddReq> {
        Ok(RbumItemAddReq {
            id: Some(TrimString(IamAppServ::get_new_id())),
            code: None,
            name: add_req.name.clone(),
            disabled: add_req.disabled,
            rel_rbum_kind_id: "".to_string(),
            rel_rbum_domain_id: "".to_string(),
            scope_level: add_req.scope_level.clone(),
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamAppAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<iam_app::ActiveModel> {
        Ok(iam_app::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            sort: Set(add_req.sort.unwrap_or(0)),
            contact_phone: Set(add_req.contact_phone.as_ref().unwrap_or(&"".to_string()).to_string()),
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamAppModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
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

    async fn package_ext_modify(id: &str, modify_req: &IamAppModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<iam_app::ActiveModel>> {
        if modify_req.icon.is_none() && modify_req.sort.is_none() && modify_req.contact_phone.is_none() {
            return Ok(None);
        }
        let mut iam_app = iam_app::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            iam_app.icon = Set(icon.to_string());
        }
        if let Some(sort) = modify_req.sort {
            iam_app.sort = Set(sort);
        }
        if let Some(contact_phone) = &modify_req.contact_phone {
            iam_app.contact_phone = Set(contact_phone.to_string());
        }
        Ok(Some(iam_app))
    }

    async fn package_item_query(query: &mut SelectStatement, _: bool, filter: &IamAppFilterReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_app::Entity, iam_app::Column::ContactPhone));
        query.column((iam_app::Entity, iam_app::Column::Icon));
        query.column((iam_app::Entity, iam_app::Column::Sort));
        if let Some(contact_phone) = &filter.contact_phone {
            query.and_where(Expr::col(iam_app::Column::ContactPhone).eq(contact_phone.as_str()));
        }
        Ok(())
    }
}

impl IamAppServ {
    pub fn get_new_id() -> String {
        TardisFuns::field.nanoid_len(RBUM_ITEM_ID_APP_LEN as usize)
    }

    pub fn get_id_by_cxt(cxt: &TardisContext) -> TardisResult<String> {
        if let Some(id) = rbum_scope_helper::get_path_item(RBUM_SCOPE_LEVEL_APP.to_int(), &cxt.own_paths) {
            Ok(id)
        } else {
            Err(TardisError::Unauthorized(format!("app id not found in tardis content {}", cxt.own_paths)))
        }
    }
}

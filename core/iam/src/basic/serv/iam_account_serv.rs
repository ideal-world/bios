use async_trait::async_trait;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{Expr, SelectStatement};
use tardis::web::web_resp::TardisPage;

use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemModifyReq};
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_account;
use crate::basic::dto::iam_account_dto::{IamAccountAddReq, IamAccountDetailResp, IamAccountModifyReq, IamAccountSelfModifyReq, IamAccountSummaryResp};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::iam_config::{IamBasicInfoManager, IamConfig};
use crate::iam_enumeration::IAMRelKind;

pub struct IamAccountServ;

#[async_trait]
impl<'a> RbumItemCrudOperation<'a, iam_account::ActiveModel, IamAccountAddReq, IamAccountModifyReq, IamAccountSummaryResp, IamAccountDetailResp, IamAccountFilterReq>
    for IamAccountServ
{
    fn get_ext_table_name() -> &'static str {
        iam_account::Entity.table_name()
    }

    fn get_rbum_kind_id() -> String {
        IamBasicInfoManager::get().kind_account_id
    }

    fn get_rbum_domain_id() -> String {
        IamBasicInfoManager::get().domain_iam_id
    }

    async fn package_item_add(add_req: &IamAccountAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            id: add_req.id.clone(),
            code: None,
            name: add_req.name.clone(),
            disabled: add_req.disabled,
            scope_level: add_req.scope_level.clone(),
        })
    }

    async fn package_ext_add(id: &str, add_req: &IamAccountAddReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<iam_account::ActiveModel> {
        Ok(iam_account::ActiveModel {
            id: Set(id.to_string()),
            icon: Set(add_req.icon.as_ref().unwrap_or(&"".to_string()).to_string()),
            ext1_idx: Set("".to_string()),
            ext2_idx: Set("".to_string()),
            ext3_idx: Set("".to_string()),
            ext4: Set("".to_string()),
            ext5: Set("".to_string()),
            ext6: Set("".to_string()),
            ext7: Set("".to_string()),
            ext8: Set("".to_string()),
            ext9: Set("".to_string()),
            ..Default::default()
        })
    }

    async fn package_item_modify(_: &str, modify_req: &IamAccountModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<RbumItemModifyReq>> {
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

    async fn package_ext_modify(id: &str, modify_req: &IamAccountModifyReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<Option<iam_account::ActiveModel>> {
        if modify_req.icon.is_none() {
            return Ok(None);
        }
        let mut iam_account = iam_account::ActiveModel {
            id: Set(id.to_string()),
            ..Default::default()
        };
        if let Some(icon) = &modify_req.icon {
            iam_account.icon = Set(icon.to_string());
        }
        Ok(Some(iam_account))
    }

    async fn after_modify_item(id: &str, _: &mut IamAccountModifyReq, funs: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        Self::delete_cache(id, funs).await?;
        Ok(())
    }

    async fn after_delete_item(id: &str, funs: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        Self::delete_cache(id, funs).await?;
        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, filter: &IamAccountFilterReq, _: &TardisFunsInst<'a>, _: &TardisContext) -> TardisResult<()> {
        query.column((iam_account::Entity, iam_account::Column::Icon));
        query.column((iam_account::Entity, iam_account::Column::Ext1Idx));
        query.column((iam_account::Entity, iam_account::Column::Ext2Idx));
        query.column((iam_account::Entity, iam_account::Column::Ext3Idx));
        query.column((iam_account::Entity, iam_account::Column::Ext4));
        query.column((iam_account::Entity, iam_account::Column::Ext5));
        query.column((iam_account::Entity, iam_account::Column::Ext6));
        query.column((iam_account::Entity, iam_account::Column::Ext7));
        query.column((iam_account::Entity, iam_account::Column::Ext8));
        query.column((iam_account::Entity, iam_account::Column::Ext9));
        if let Some(icon) = &filter.icon {
            query.and_where(Expr::col(iam_account::Column::Icon).eq(icon.as_str()));
        }
        Ok(())
    }
}

impl<'a> IamAccountServ {
    pub async fn self_modify_account(modify_req: &mut IamAccountSelfModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamAccountServ::modify_item(
            &cxt.owner,
            &mut IamAccountModifyReq {
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                disabled: modify_req.disabled,
                scope_level: None,
            },
            funs,
            cxt,
        )
        .await
    }

    pub async fn paginate_rel_roles(
        account_id: &str,
        page_number: u64,
        page_size: u64,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumRelAggResp>> {
        IamRelServ::paginate_from_rels(
            IAMRelKind::IamAccountRole,
            false,
            account_id,
            page_number,
            page_size,
            desc_by_create,
            desc_by_update,
            funs,
            cxt,
        )
        .await
    }

    pub async fn delete_cache(account_id: &str, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        // TODO change cert role group
        let tokens = funs.cache().hgetall(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        for (token, _) in tokens.iter() {
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await?;
        }
        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str()).await?;
        Ok(())
    }
}

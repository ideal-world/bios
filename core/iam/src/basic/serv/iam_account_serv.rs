use async_trait::async_trait;
use tardis::basic::dto::{TardisContext, TardisFunsInst};
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::*;
use tardis::db::sea_query::{Expr, SelectStatement};

use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemModifyReq};
use bios_basic::rbum::dto::rbum_rel_dto::RbumRelBoneResp;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::domain::iam_account;
use crate::basic::dto::iam_account_dto::{
    IamAccountAddReq, IamAccountAggAddReq, IamAccountAggModifyReq, IamAccountDetailResp, IamAccountModifyReq, IamAccountSelfModifyReq, IamAccountSummaryResp,
};
use crate::basic::dto::iam_cert_dto::{IamMailVCodeCertAddReq, IamPhoneVCodeCertAddReq, IamUserPwdCertAddReq};
use crate::basic::dto::iam_filer_dto::IamAccountFilterReq;
use crate::basic::serv::iam_attr_serv::IamAttrServ;
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_role_serv::IamRoleServ;
use crate::iam_config::{IamBasicInfoManager, IamConfig};
use crate::iam_enumeration::{IamCertKind, IamRelKind};

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
    pub async fn add_account_agg(add_req: &IamAccountAggAddReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<String> {
        let attrs = IamAttrServ::find_account_attrs(funs, cxt).await?;
        if attrs.iter().any(|i| i.required && !add_req.exts.contains_key(&i.name)) {
            return Err(TardisError::BadRequest("Missing required field".to_string()));
        }
        let account_id = IamAccountServ::add_item(
            &mut IamAccountAddReq {
                id: add_req.id.clone(),
                name: add_req.name.clone(),
                scope_level: add_req.scope_level.clone(),
                disabled: add_req.disabled,
                icon: add_req.icon.clone(),
            },
            funs,
            cxt,
        )
        .await?;
        if let Some(cert_conf_id) = IamCertServ::get_cert_conf_id_opt_by_code(&IamCertKind::UserPwd.to_string(), Some(cxt.own_paths.clone()), funs).await? {
            IamCertUserPwdServ::add_cert(
                &IamUserPwdCertAddReq {
                    ak: add_req.cert_user_name.clone(),
                    sk: add_req.cert_password.clone(),
                },
                &account_id,
                Some(cert_conf_id),
                funs,
                cxt,
            )
            .await?;
        }
        if let Some(cert_phone) = &add_req.cert_phone {
            if let Some(cert_conf_id) = IamCertServ::get_cert_conf_id_opt_by_code(&IamCertKind::PhoneVCode.to_string(), Some(cxt.own_paths.clone()), funs).await? {
                IamCertPhoneVCodeServ::add_cert(
                    &IamPhoneVCodeCertAddReq {
                        phone: TrimString(cert_phone.to_string()),
                    },
                    &account_id,
                    &cert_conf_id,
                    funs,
                    cxt,
                )
                .await?;
            }
        }
        if let Some(cert_mail) = &add_req.cert_mail {
            if let Some(cert_conf_id) = IamCertServ::get_cert_conf_id_opt_by_code(&IamCertKind::MailVCode.to_string(), Some(cxt.own_paths.clone()), funs).await? {
                IamCertMailVCodeServ::add_cert(&IamMailVCodeCertAddReq { mail: cert_mail.to_string() }, &account_id, &cert_conf_id, funs, cxt).await?;
            }
        }
        if let Some(role_ids) = &add_req.role_ids {
            for role_id in role_ids {
                IamRoleServ::add_rel_account(role_id, &account_id, funs, cxt).await?;
            }
        }
        IamAttrServ::add_or_modify_account_attr_values(&account_id, add_req.exts.clone(), funs, cxt).await?;
        Ok(account_id)
    }

    pub async fn modify_account_agg(id: &str, modify_req: &IamAccountAggModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamAccountServ::modify_item(
            id,
            &mut IamAccountModifyReq {
                name: modify_req.name.clone(),
                scope_level: modify_req.scope_level.clone(),
                disabled: modify_req.disabled,
                icon: modify_req.icon.clone(),
            },
            funs,
            cxt,
        )
        .await?;
        if let Some(input_role_ids) = &modify_req.role_ids {
            let stored_roles = Self::find_simple_rel_roles(id, true, None, None, funs, cxt).await?;
            let stored_role_ids: Vec<String> = stored_roles.into_iter().map(|r| r.rel_id).collect();
            for input_role_id in input_role_ids {
                if !stored_role_ids.contains(input_role_id) {
                    IamRoleServ::add_rel_account(input_role_id, id, funs, cxt).await?;
                }
            }
            for stored_role_id in stored_role_ids {
                if !input_role_ids.contains(&stored_role_id) {
                    IamRoleServ::delete_rel_account(&stored_role_id, id, funs, cxt).await?;
                }
            }
        }
        IamAttrServ::add_or_modify_account_attr_values(id, modify_req.exts.clone(), funs, cxt).await?;
        Ok(())
    }

    pub async fn self_modify_account(modify_req: &mut IamAccountSelfModifyReq, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        let id = &cxt.owner;
        IamAccountServ::modify_item(
            id,
            &mut IamAccountModifyReq {
                name: modify_req.name.clone(),
                icon: modify_req.icon.clone(),
                disabled: modify_req.disabled,
                scope_level: None,
            },
            funs,
            cxt,
        )
        .await?;
        IamAttrServ::add_or_modify_account_attr_values(id, modify_req.exts.clone(), funs, cxt).await?;
        Ok(())
    }

    pub async fn find_simple_rel_roles(
        account_id: &str,
        with_sub: bool,
        desc_by_create: Option<bool>,
        desc_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<Vec<RbumRelBoneResp>> {
        IamRelServ::find_from_simple_rels(IamRelKind::IamAccountRole, with_sub, account_id, desc_by_create, desc_by_update, funs, cxt).await
    }

    pub async fn delete_cache(account_id: &str, funs: &TardisFunsInst<'a>) -> TardisResult<()> {
        let tokens = funs.cache().hgetall(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        for (token, _) in tokens.iter() {
            funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_token_info_, token).as_str()).await?;
        }
        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_rel_, account_id).as_str()).await?;
        funs.cache().del(format!("{}{}", funs.conf::<IamConfig>().cache_key_account_info_, account_id).as_str()).await?;
        Ok(())
    }
}

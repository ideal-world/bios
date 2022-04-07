use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::reldb_client::TardisRelDBlConnection;
use tardis::web::web_resp::TardisPage;
use tardis::TardisFuns;

use bios_basic::rbum::dto::filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::dto::rbum_cert_conf_dto::{RbumCertConfDetailResp, RbumCertConfSummaryResp};
use bios_basic::rbum::serv::rbum_cert_serv::{RbumCertConfServ, RbumCertServ};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;

use crate::basic::constants;
use crate::basic::dto::iam_cert_conf_dto::{IamMailVCodeCertConfAddOrModifyReq, IamPhoneVCodeCertConfAddOrModifyReq, IamTokenCertConfAddReq, IamUserPwdCertConfAddOrModifyReq};
use crate::basic::enumeration::{IAMRelKind, IamCertTokenKind};
use crate::basic::serv::iam_cert_mail_vcode_serv::IamCertMailVCodeServ;
use crate::basic::serv::iam_cert_phone_vcode_serv::IamCertPhoneVCodeServ;
use crate::basic::serv::iam_cert_token_serv::IamCertTokenServ;
use crate::basic::serv::iam_cert_user_pwd_serv::IamCertUserPwdServ;
use crate::basic::serv::iam_rel_serv::IamRelServ;

pub struct IamCertServ;

impl<'a> IamCertServ {
    pub fn get_new_pwd() -> String {
        TardisFuns::field.nanoid_len(10)
    }

    pub async fn init_global_ident_conf(funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<()> {
        IamCertUserPwdServ::add_cert_conf(
            &mut IamUserPwdCertConfAddOrModifyReq {
                ak_note: None,
                ak_rule: None,
                sk_note: None,
                sk_rule: None,
                repeatable: Some(true),
                expire_sec: None,
            },
            None,
            db,
            cxt,
        )
        .await?;

        IamCertMailVCodeServ::add_cert_conf(&mut IamMailVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }, None, db, cxt).await?;

        IamCertPhoneVCodeServ::add_cert_conf(&mut IamPhoneVCodeCertConfAddOrModifyReq { ak_note: None, ak_rule: None }, None, db, cxt).await?;

        IamCertTokenServ::add_cert_conf(
            &mut IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenDefault.to_string()),
                coexist_num: constants::RBUM_CERT_CONF_TOKEN_DEFAULT_COEXIST_NUM,
                expire_sec: Some(constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenDefault,
            None,
            db,
            cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &mut IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenPc.to_string()),
                coexist_num: 1,
                expire_sec: Some(constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPc,
            None,
            db,
            cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &mut IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenPhone.to_string()),
                coexist_num: 1,
                expire_sec: Some(constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPhone,
            None,
            db,
            cxt,
        )
        .await?;

        IamCertTokenServ::add_cert_conf(
            &mut IamTokenCertConfAddReq {
                name: TrimString(IamCertTokenKind::TokenPad.to_string()),
                coexist_num: 1,
                expire_sec: Some(constants::RBUM_CERT_CONF_TOKEN_EXPIRE_SEC),
            },
            IamCertTokenKind::TokenPad,
            None,
            db,
            cxt,
        )
        .await?;

        Ok(())
    }

    pub async fn get_cert_conf(id: &str, rbum_item_id: Option<String>, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<RbumCertConfDetailResp> {
        RbumCertConfServ::get_rbum(
            id,
            &RbumBasicFilterReq {
                rbum_domain_id: Some(constants::get_rbum_basic_info().domain_iam_id.to_string()),
                rbum_item_id,
                ..Default::default()
            },
            db,
            cxt,
        )
        .await
    }

    pub async fn paginate_cert_conf(
        q_name: Option<String>,
        rbum_item_id: Option<String>,
        page_number: u64,
        page_size: u64,
        desc_sort_by_create: Option<bool>,
        desc_sort_by_update: Option<bool>,
        funs: &TardisFunsInst<'a>,
        cxt: &TardisContext,
    ) -> TardisResult<TardisPage<RbumCertConfSummaryResp>> {
        RbumCertConfServ::paginate_rbums(
            &RbumBasicFilterReq {
                name: q_name,
                rbum_domain_id: Some(constants::get_rbum_basic_info().domain_iam_id.to_string()),
                rbum_item_id,
                ..Default::default()
            },
            page_number,
            page_size,
            desc_sort_by_create,
            desc_sort_by_update,
            db,
            cxt,
        )
        .await
    }

    pub async fn delete_cert_conf(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        RbumCertConfServ::delete_rbum(id, db, cxt).await
    }

    pub async fn delete_cert(id: &str, funs: &TardisFunsInst<'a>, cxt: &TardisContext) -> TardisResult<u64> {
        RbumCertServ::delete_rbum(id, db, cxt).await
    }

    pub async fn get_id_by_code(code: &str, rel_iam_tenant_id: Option<&str>, funs: &TardisFunsInst<'a>) -> TardisResult<String> {
        RbumCertConfServ::get_rbum_cert_conf_id_by_code(code, &constants::get_rbum_basic_info().domain_iam_id, rel_iam_tenant_id.unwrap_or(""), db)
            .await?
            .ok_or_else(|| TardisError::NotFound(format!("cert config code {} not found", code)))
    }

    pub async fn get_tardis_context(
        iam_tenant_id: Option<&str>,
        iam_app_id: Option<&str>,
        ak: &str,
        account_id: &str,
        token: Option<&str>,
        token_kind: Option<&str>,
        funs: &TardisFunsInst<'a>,
    ) -> TardisResult<TardisContext> {
        let own_paths = if let Some(iam_tenant_id) = iam_tenant_id {
            if let Some(iam_app_id) = iam_app_id {
                format!("{}/{}", iam_tenant_id, iam_app_id)
            } else {
                iam_tenant_id.to_string()
            }
        } else {
            "".to_string()
        };
        let mut context = TardisContext {
            own_paths,
            ak: ak.to_string(),
            account_id: account_id.to_string(),
            token: token.unwrap_or("").to_string(),
            token_kind: token_kind.unwrap_or("").to_string(),
            roles: vec![],
            // TODO
            groups: vec![],
        };
        let roles = IamRelServ::paginate_to_rels(IAMRelKind::IamRoleAccount, account_id, 1, u64::MAX, Some(true), None, db, &context)
            .await?
            .records
            .iter()
            .map(|i| i.rel.id.to_string())
            .collect::<Vec<String>>();
        context.roles = roles;
        Ok(context)
    }
}

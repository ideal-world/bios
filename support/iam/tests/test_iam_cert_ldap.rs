use crate::test_basic;
use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfLdapAddOrModifyReq;
use bios_iam::basic::serv::iam_cert_ldap_serv::{AccountFieldMap, IamCertLdapServ, OrgFieldMap};
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::iam_constants;
use ldap3::log::info;
use std::env;
use tardis::basic::dto::TardisContext;

pub async fn test(admin_ctx: &TardisContext, tenant1_admin_context: &TardisContext, tenant2_admin_context: &TardisContext) -> () {
    let mut funs = iam_constants::get_tardis_inst();
    // funs.begin().await.unwrap();
    info!("【test ldap conf curd】");
    let ldap_cert_conf = IamCertLdapServ::get_cert_conf_by_ctx(&funs, admin_ctx).await.unwrap();
    assert!(ldap_cert_conf.is_none());

    let conf_ldap_add_or_modify_req = test_basic::gen_test_ldap_conf();
    let err_req_param = IamCertConfLdapAddOrModifyReq {
        port: Some(293u16),
        ..conf_ldap_add_or_modify_req.clone()
    };
    assert!(IamCertLdapServ::add_cert_conf(&err_req_param, None, &funs, admin_ctx).await.is_err());

    let ldap_cert_conf_id = IamCertLdapServ::add_cert_conf(&conf_ldap_add_or_modify_req, None, &funs, admin_ctx).await.unwrap();
    let ldap_cert_conf = IamCertLdapServ::get_cert_conf(&ldap_cert_conf_id, &funs, admin_ctx).await.unwrap();

    info!("【test ldap sync function】");
    let conf_id = IamCertServ::get_cert_conf_id_by_kind("Ldap", None, &funs).await.unwrap();
    assert_eq!(conf_id, ldap_cert_conf_id);
    // IamCertLdapServ::iam_sync_ldap_user_to_iam(&conf_id, &funs, &admin_ctx).await.unwrap();
    // funs.commit().await.unwrap();
}

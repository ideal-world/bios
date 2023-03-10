use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfLdapAddOrModifyReq;
use bios_iam::basic::serv::iam_cert_ldap_serv::{AccountFieldMap, IamCertLdapServ, OrgFieldMap};
use bios_iam::basic::serv::iam_cert_serv::IamCertServ;
use bios_iam::iam_constants;
use ldap3::log::info;
use std::env;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use crate::test_basic;

pub async fn test(admin_ctx: &TardisContext, tenant1_admin_context: &TardisContext, tenant2_admin_context: &TardisContext) -> () {
    let funs = iam_constants::get_tardis_inst();

    info!("【test ldap conf curd】");
    let ldap_cert_conf = IamCertLdapServ::get_cert_conf_by_ctx(&funs, admin_ctx).await.unwrap();
    assert!(ldap_cert_conf.is_none());

    let ldap_cert_conf = IamCertLdapServ::add_cert_conf(&test_basic::gen_test_ldap_conf(), None, &funs, admin_ctx).await.unwrap();

    info!("【test ldap sync function】");
    let conf_id = IamCertServ::get_cert_conf_id_by_kind("Ldap", None, &funs).await.unwrap();
    IamCertLdapServ::iam_sync_ldap_user_to_iam(&conf_id, &funs, &admin_ctx);
    ()
}



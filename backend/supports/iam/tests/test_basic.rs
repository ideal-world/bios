use std::env;
use tardis::basic::field::TrimString;

use bios_iam::basic::dto::iam_cert_conf_dto::IamCertConfLdapAddOrModifyReq;
use bios_iam::basic::serv::iam_cert_ldap_serv::AccountFieldMap;
use tardis::basic::result::TardisResult;
use tardis::testcontainers::core::{ExecCommand, WaitFor};
use tardis::testcontainers::runners::AsyncRunner;
use tardis::testcontainers::GenericImage;
use tardis::testcontainers::{ContainerAsync, ImageExt};
use tardis::TardisFuns;

const BASE_LDIF: &str = "dn: cn=Barbara,dc=test,dc=com
objectClass: inetOrgPerson
cn: Barbara
sn: Jensen
displayName: Barbara Jensen
title: the world's most famous mythical manager
mail: bjensen@test.com
uid: bjensen
userpassword: 123456

dn: cn=testUser,dc=test,dc=com
objectClass: inetOrgPerson
cn: testUser
sn: user
displayName: testUser
title: the world's most famous mythical manager
mail: testUser@test.com
uid: tuser
userpassword: 123456

dn: cn=testUser1,dc=test,dc=com
objectClass: inetOrgPerson
cn: testUser1
sn: user1
displayName: testUser1
title: the world's most famous mythical manager
mail: testUser1@test.com
uid: tuser1
userpassword: 123456

dn: cn=testUser2,dc=test,dc=com
objectClass: inetOrgPerson
cn: testUser2
sn: user2
displayName: testUser2
title: the world's most famous mythical manager
mail: testUser2@test.com
uid: tuser2
userpassword: 123456";
//if BASE_LDIF change,LDAP_ACCOUNT_NUB must change too
pub const LDAP_ACCOUNT_NUB: u64 = 4;
pub struct LifeHold {
    pub ldap: ContainerAsync<GenericImage>,
}

pub async fn init() -> TardisResult<LifeHold> {
    let ldap_container = get_ldap_container().await?;

    TardisFuns::init(Some("tests/config")).await?;
    // TardisFuns::init("core/iam/tests/config").await?;

    Ok(LifeHold { ldap: ldap_container })
}

async fn get_ldap_container() -> TardisResult<ContainerAsync<GenericImage>> {
    const ORGANISATION: &str = "test";
    const ADMIN_PASSWORD: &str = "123456";
    let domain: String = format!("{}.com", ORGANISATION);

    let ldap_container = GenericImage::new("osixia/openldap", "latest")
        .with_wait_for(WaitFor::message_on_stdout("First start is done..."))
        .with_env_var("LDAP_ORGANISATION", ORGANISATION)
        .with_env_var("LDAP_DOMAIN", domain)
        .with_env_var("LDAP_ADMIN_PASSWORD", ADMIN_PASSWORD)
        .start()
        .await
        .expect("ldap container start");

    let port = ldap_container.get_host_port_ipv4(389).await?;
    let url = "ldap://127.0.0.1".to_string();
    let base_dn = format!("DC={ORGANISATION},DC=com");
    let admin_dn = format!("CN=admin,{base_dn}");

    ldap_container.exec(ExecCommand::new(vec![format!("echo \"{BASE_LDIF}\" > /home/base.ldif",)]));
    ldap_container.exec(
        ExecCommand::new(vec![format!("ldapadd -x -H ldap://127.0.0.1  -D \"{admin_dn}\" -w {ADMIN_PASSWORD} -f /home/base.ldif ")])
            .with_container_ready_conditions(vec![WaitFor::millis(5)]),
    );

    env::set_var("TARDIS_FW.LDAP.PORT", port.to_string());
    env::set_var("TARDIS_FW.LDAP.URL", url);
    env::set_var("TARDIS_FW.LDAP.BASE_DN", base_dn);
    env::set_var("TARDIS_FW.LDAP.ADMIN_DN", admin_dn);
    env::set_var("TARDIS_FW.LDAP.ADMIN_CN", "admin");
    env::set_var("TARDIS_FW.LDAP.ADMIN_PASSWORD", ADMIN_PASSWORD);
    Ok(ldap_container)
}

//生成测试通用ldap 配置
pub fn gen_test_ldap_conf() -> IamCertConfLdapAddOrModifyReq {
    IamCertConfLdapAddOrModifyReq {
        supplier: Some(TrimString("TEST".to_string())),
        name: "testLdap".to_string(),
        conn_uri: env::var("TARDIS_FW.LDAP.URL").unwrap(),
        is_tls: false,
        principal: TrimString(env::var("TARDIS_FW.LDAP.ADMIN_DN").unwrap()),
        credentials: TrimString(env::var("TARDIS_FW.LDAP.ADMIN_PASSWORD").unwrap()),
        base_dn: env::var("TARDIS_FW.LDAP.BASE_DN").unwrap_or("".to_string()),
        port: Some(env::var("TARDIS_FW.LDAP.PORT").unwrap().parse().unwrap()),
        account_unique_id: "dn".to_string(),
        timeout: None,
        account_field_map: AccountFieldMap {
            search_base_filter: Some("objectClass=person".to_string()),
            field_user_name: "cn".to_string(),
            field_display_name: "displayName".to_string(),
            field_mobile: "mobile".to_string(),
            field_email: "email".to_string(),
            field_user_name_remarks: "".to_string(),
            field_display_name_remarks: "".to_string(),
            field_mobile_remarks: "".to_string(),
            field_email_remarks: "".to_string(),
            field_labor_type: "".to_string(),
            field_labor_type_remarks: "".to_string(),
            field_labor_type_map: None,
        },
        // org_unique_id: "ou".to_string(),
        // org_field_map: OrgFieldMap {
        //     search_base_filter: Some("objectClass=organizationalUnit".to_string()),
        //     field_dept_id: "ou".to_string(),
        //     field_dept_name: "ou".to_string(),
        //     field_parent_dept_id: "".to_string(),
        //     field_dept_id_remarks: "".to_string(),
        //     field_dept_name_remarks: "".to_string(),
        //     field_parent_dept_id_remarks: "".to_string(),
        // },
    }
}

use std::env;

use tardis::basic::result::TardisResult;
use tardis::test::test_container::TardisTestContainer;
use tardis::testcontainers::clients::Cli;
use tardis::testcontainers::core::{ExecCommand, WaitFor};
use tardis::testcontainers::images::generic::GenericImage;
use tardis::testcontainers::images::redis::Redis;
use tardis::testcontainers::Container;
use tardis::TardisFuns;

pub struct LifeHold<'a> {
    pub mysql: Container<'a, GenericImage>,
    pub redis: Container<'a, Redis>,
    pub rabbit: Container<'a, GenericImage>,
    pub ldap: Container<'a, GenericImage>,
}

pub async fn init(docker: &'_ Cli) -> TardisResult<LifeHold<'_>> {
    let mysql_container = TardisTestContainer::mysql_custom(None, docker);
    let port = mysql_container.get_host_port_ipv4(3306);
    let url = format!("mysql://root:123456@localhost:{}/test", port);
    env::set_var("TARDIS_FW.DB.URL", url);

    let redis_container = TardisTestContainer::redis_custom(docker);
    let port = redis_container.get_host_port_ipv4(6379);
    let url = format!("redis://127.0.0.1:{}/0", port);
    env::set_var("TARDIS_FW.CACHE.URL", url);

    let rabbit_container = TardisTestContainer::rabbit_custom(docker);
    let port = rabbit_container.get_host_port_ipv4(5672);
    let url = format!("amqp://guest:guest@127.0.0.1:{}/%2f", port);
    env::set_var("TARDIS_FW.MQ.URL", url);

    let ldap_container = get_ldap_container(docker).await;

    env::set_var("RUST_LOG", "debug,test_iam_serv=trace,bios_iam=trace,sqlx::query=off");
    TardisFuns::init("tests/config").await?;
    // TardisFuns::init("core/iam/tests/config").await?;

    Ok(LifeHold {
        mysql: mysql_container,
        redis: redis_container,
        rabbit: rabbit_container,
        ldap: ldap_container,
    })
}

async fn get_ldap_container<'a>(docker: &'a Cli) -> Container<'a, GenericImage> {
    const ORGANISATION: &str = "test";
    const ADMIN_PASSWORD: &str = "123456";
    let domain: String = format!("{}.com", ORGANISATION);

    let ldap_container = docker.run(
        GenericImage::new("osixia/openldap", "latest")
            .with_env_var("LDAP_ORGANISATION", ORGANISATION)
            .with_env_var("LDAP_DOMAIN", domain)
            .with_env_var("LDAP_ADMIN_PASSWORD", ADMIN_PASSWORD)
            .with_wait_for(WaitFor::message_on_stdout("First start is done...")),
    );

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
sn: user1
displayName: testUser1
title: the world's most famous mythical manager
mail: testUser@test.com
uid: tuser1
userpassword: 123456";
    let port = ldap_container.get_host_port_ipv4(389);
    let url = format!("ldap://localhost:{}", port);
    let base_dn = format!("DC={},DC=com", ORGANISATION);
    let admin_dn = format!("CN=admin,{}", base_dn);

    ldap_container.exec(ExecCommand {
        cmd: format!("echo \"{}\" > /home/base.ldif", BASE_LDIF),
        ready_conditions: vec![],
    });
    ldap_container.exec(ExecCommand {
        cmd: format!("ldapadd -x -H ldap://localhost  -D \"{}\" -w {} -f /home/base.ldif ", admin_dn, ADMIN_PASSWORD),
        ready_conditions: vec![WaitFor::millis(5)],
    });

    env::set_var("TARDIS_FW.LDAP.URL", url);
    env::set_var("TARDIS_FW.LDAP.BASE_DN", base_dn);
    env::set_var("TARDIS_FW.LDAP.ADMIN_DN", admin_dn);
    env::set_var("TARDIS_FW.LDAP.ADMIN_CN", "admin");
    env::set_var("TARDIS_FW.LDAP.ADMIN_PASSWORD", ADMIN_PASSWORD);
    ldap_container
}

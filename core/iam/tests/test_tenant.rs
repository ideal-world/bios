use tardis::basic::result::TardisResult;
use tardis::tokio;
use tardis::TardisFuns;

use bios_basic::rbum::dto::rbum_item_dto::{RbumItemAddReq, RbumItemModifyReq};
use bios_basic::rbum::enumeration::RbumScopeKind;
use bios_iam::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantModifyReq};
use bios_iam::console_system::serv::iam_cs_tenant_serv;

mod test_basic;

#[tokio::test]
async fn test_tenant() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = test_basic::init(&docker).await?;

    let context = bios_basic::rbum::initializer::get_sys_admin_context().await;

    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;
    // add_iam_tenant
    let id = iam_cs_tenant_serv::add_iam_tenant(
        &IamCsTenantAddReq {
            basic: RbumItemAddReq {
                scope_kind: RbumScopeKind::APP,
                disabled: false,
                name: "测试记录".to_string(),
                uri_part: "".to_string(),
                icon: "".to_string(),
                sort: 0,
                rel_rbum_domain_id: "iam".to_string(),
            },
        },
        &tx,
        &context,
    )
    .await?;
    tx.commit().await?;

    // peek_iam_tenant
    let tenant = iam_cs_tenant_serv::peek_iam_tenant(&id, &TardisFuns::reldb().conn(), &context).await?;
    assert_eq!(tenant.basic.id, id);
    assert_eq!(tenant.basic.name, "测试记录");

    let mut tx = TardisFuns::reldb().conn();
    tx.begin().await?;
    // modify_iam_tenant
    iam_cs_tenant_serv::modify_iam_tenant(
        &id,
        &IamCsTenantModifyReq {
            basic: RbumItemModifyReq {
                name: Some("测试记录2".to_string()),
                uri_part: None,
                icon: None,
                sort: None,
                scope_kind: None,
                disabled: None,
            },
        },
        &tx,
        &context,
    )
    .await?;
    tx.commit().await?;

    // get_iam_tenant
    let tenant = iam_cs_tenant_serv::get_iam_tenant(&id, &TardisFuns::reldb().conn(), &context).await?;
    assert_eq!(tenant.basic.id, id);
    assert_eq!(tenant.basic.name, "测试记录2");

    // find_iam_tenants
    let tenants = iam_cs_tenant_serv::find_iam_tenants(1, 2, None, &TardisFuns::reldb().conn(), &context).await?;
    assert_eq!(tenants.page_number, 1);
    assert_eq!(tenants.page_size, 2);
    assert_eq!(tenants.total_size, 2);

    Ok(())
}

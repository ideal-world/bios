use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_item_dto::RbumItemAddReq;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::rbum_config::RbumConfig;
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemServ;
use bios_basic::rbum::serv::rbum_kind_serv::RbumKindServ;
use bios_basic::test::init_rbum_test_container;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::{testcontainers, tokio, TardisFuns};

const RBUM_KIND_SCHEME_IAM_TENANT: &str = "iam-tenant";
const RBUM_KIND_SCHEME_IAM_APP: &str = "iam-app";
const RBUM_KIND_SCHEME_IAM_ACCOUNT: &str = "iam-account";
const RBUM_ITEM_NAME_DEFAULT_TENANT: &str = "system";
const RBUM_ITEM_NAME_DEFAULT_APP: &str = "iam";
const RBUM_ITEM_NAME_DEFAULT_ACCOUNT: &str = "sys_admin";

mod test_rbum_cert;
mod test_rbum_domain;
mod test_rbum_event;
mod test_rbum_item;
mod test_rbum_kind;
mod test_rbum_rel;
mod test_rbum_set;
mod test_scope;

#[tokio::test]
async fn test_rbum() -> TardisResult<()> {
    let docker = testcontainers::clients::Cli::default();
    let _x = init_rbum_test_container::init(&docker, None).await?;
    let ctx = init_test_data().await?;
    test_scope::test().await?;
    test_rbum_domain::test(&ctx).await?;
    test_rbum_kind::test(&ctx).await?;
    test_rbum_item::test(&ctx).await?;
    test_rbum_cert::test(&ctx).await?;
    test_rbum_rel::test(&ctx).await?;
    test_rbum_set::test(&ctx).await?;
    test_rbum_event::test().await?;
    Ok(())
}

pub async fn init_test_data() -> TardisResult<TardisContext> {
    bios_basic::rbum::rbum_initializer::init("", RbumConfig::default()).await?;

    let mut funs = TardisFuns::inst_with_db_conn("".to_string(), None);

    funs.mq().subscribe("rbum::entity_deleted", |(_, _)| async { Ok(()) }).await?;

    funs.begin().await?;

    let ctx = TardisContext {
        own_paths: "".to_string(),
        owner: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        ..Default::default()
    };

    let kind_tenant_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(RBUM_KIND_SCHEME_IAM_TENANT.to_string()),
            name: TrimString(RBUM_KIND_SCHEME_IAM_TENANT.to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some(RBUM_KIND_SCHEME_IAM_TENANT.to_string().to_lowercase()),
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &ctx,
    )
    .await?;

    let kind_app_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(RBUM_KIND_SCHEME_IAM_APP.to_string()),
            name: TrimString(RBUM_KIND_SCHEME_IAM_APP.to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some(RBUM_KIND_SCHEME_IAM_APP.to_string().to_lowercase()),
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &ctx,
    )
    .await?;

    let kind_account_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(RBUM_KIND_SCHEME_IAM_ACCOUNT.to_string()),
            name: TrimString(RBUM_KIND_SCHEME_IAM_ACCOUNT.to_string()),
            note: None,
            icon: None,
            sort: None,
            ext_table_name: Some(RBUM_KIND_SCHEME_IAM_ACCOUNT.to_string().to_lowercase()),
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &ctx,
    )
    .await?;

    let domain_iam_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("iam".to_string()),
            name: TrimString("IAM".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &ctx,
    )
    .await?;

    let tenant_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: None,
            name: TrimString(RBUM_ITEM_NAME_DEFAULT_TENANT.to_string()),
            disabled: None,
            rel_rbum_kind_id: kind_tenant_id.clone(),
            rel_rbum_domain_id: domain_iam_id.clone(),
            scope_level: Some(RbumScopeLevelKind::L2),
            id: Some(TrimString(TardisFuns::field.nanoid_len(4))),
        },
        &funs,
        &ctx,
    )
    .await?;

    let app_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: None,
            name: TrimString(RBUM_ITEM_NAME_DEFAULT_APP.to_string()),
            disabled: None,
            rel_rbum_kind_id: kind_app_id.clone(),
            rel_rbum_domain_id: domain_iam_id.clone(),
            scope_level: Some(RbumScopeLevelKind::L2),
            id: Some(TrimString(TardisFuns::field.nanoid_len(4))),
        },
        &funs,
        &ctx,
    )
    .await?;

    let account_id = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            code: None,
            name: TrimString(RBUM_ITEM_NAME_DEFAULT_ACCOUNT.to_string()),
            disabled: None,
            rel_rbum_kind_id: kind_account_id.clone(),
            rel_rbum_domain_id: domain_iam_id.clone(),
            scope_level: Some(RbumScopeLevelKind::Root),
            id: None,
        },
        &funs,
        &ctx,
    )
    .await?;

    funs.commit().await?;
    Ok(TardisContext {
        own_paths: format!("{}/{}", tenant_id, app_id),
        owner: account_id.to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        ..Default::default()
    })
}

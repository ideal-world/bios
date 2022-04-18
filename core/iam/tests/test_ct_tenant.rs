use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_iam::console_tenant::dto::iam_ct_tenant_dto::IamCtTenantModifyReq;
use bios_iam::console_tenant::serv::iam_ct_tenant_serv::IamCtTenantServ;
use bios_iam::iam_constants;

pub async fn test(context1: &TardisContext, _: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ct_tenant】 : Modify Current Tenant");
    IamCtTenantServ::modify_tenant(
        &mut IamCtTenantModifyReq {
            name: Some(TrimString("NT".to_string())),
            icon: None,
            sort: None,
            contact_phone: Some("1333333".to_string()),
            scope_level: None,
            disabled: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ct_tenant】 : Get Current Tenant");
    let tenant = IamCtTenantServ::get_tenant(&funs, context1).await?;
    assert_eq!(tenant.name, "NT");
    assert_eq!(tenant.contact_phone, "1333333");
    assert!(!tenant.disabled);

    funs.rollback().await?;

    Ok(())
}

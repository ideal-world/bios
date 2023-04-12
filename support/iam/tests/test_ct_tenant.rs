use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::log::info;

use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_filter_dto::IamTenantFilterReq;
use bios_iam::basic::dto::iam_tenant_dto::IamTenantModifyReq;
use bios_iam::basic::serv::iam_tenant_serv::IamTenantServ;
use bios_iam::iam_constants;

pub async fn test(context1: &TardisContext, _: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    funs.begin().await?;

    info!("【test_ct_tenant】 : Modify Current Tenant");
    IamTenantServ::modify_item(
        &IamTenantServ::get_id_by_ctx(context1, &funs)?,
        &mut IamTenantModifyReq {
            name: Some(TrimString("NT".to_string())),
            icon: None,
            sort: None,
            contact_phone: Some("1333333".to_string()),
            scope_level: None,
            disabled: None,
            note: None,
            account_self_reg: None,
        },
        &funs,
        context1,
    )
    .await?;

    info!("【test_ct_tenant】 : Get Current Tenant");
    let tenant = IamTenantServ::get_item(&IamTenantServ::get_id_by_ctx(context1, &funs)?, &IamTenantFilterReq::default(), &funs, context1).await?;
    assert_eq!(tenant.name, "NT");
    assert_eq!(tenant.contact_phone, "1333333");
    assert!(!tenant.disabled);

    funs.rollback().await?;

    Ok(())
}

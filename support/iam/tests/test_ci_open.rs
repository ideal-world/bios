use bios_iam::basic::dto::iam_open_dto::{IamOpenAddProductReq, IamOpenAddSpecReq, IamOpenAkSkAddReq, IamOpenBindAkProductReq};
use bios_iam::basic::serv::iam_open_serv::IamOpenServ;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::chrono::Utc;
use tardis::log::info;

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_iam::basic::dto::iam_app_dto::{IamAppAggAddReq, IamAppModifyReq};
use bios_iam::basic::dto::iam_filer_dto::IamAppFilterReq;
use bios_iam::basic::serv::iam_app_serv::IamAppServ;
use bios_iam::iam_constants;

pub async fn test(context1: &TardisContext) -> TardisResult<()> {
    let mut funs = iam_constants::get_tardis_inst();
    
    let product_code = "product1".to_string();
    let spec1_code = "spec1".to_string();
    let spec2_code = "spec2".to_string();
    info!("【test_ci_open】 : Add Product");
    IamOpenServ::add_product(&IamOpenAddProductReq {
        code: TrimString(product_code.clone()),
        name:TrimString("测试产品".to_string()),
        icon:None,
        scope_level:None,
        disabled:None,
        specifications:vec![
            IamOpenAddSpecReq {
                code: TrimString(spec1_code.clone()),
                name: TrimString("测试规格1".to_string()),
                icon: None,
                url: None,
                scope_level:None,
                disabled:None,
            },
            IamOpenAddSpecReq {
                code: TrimString(spec2_code.clone()),
                name: TrimString("测试规格2".to_string()),
                icon: None,
                url: None,
                scope_level:None,
                disabled:None,
            },
        ],
    }, &funs, context1).await?;
    info!("【test_ci_open】 : Apply ak/sk");
    let cert_resp = IamOpenServ::general_cert(IamOpenAkSkAddReq {
        tenant_id: context1.own_paths.clone(),
        app_id: None,
    }, &funs, context1).await?;
    let cert_id = cert_resp.id;
    IamOpenServ::bind_cert_product_and_spec(&cert_id, &IamOpenBindAkProductReq {
        product_code: product_code.clone(),
        spec_code: spec1_code.clone(),
        start_time: Some(Utc::now()),
        end_time: Some(Utc::now()),
        api_call_frequency: Some(500),
        api_call_count:Some(10000),
    }, &funs, context1).await?;

    Ok(())
}

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::OpenApi;
use tardis::web::web_resp::{TardisApiResult, TardisResp};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use crate::basic::dto::iam_tenant_dto::{IamTenantBoneResp, IamTenantSummaryResp};
use crate::basic::serv::iam_cert_serv::IamCertServ;
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::iam_constants;

pub struct IamCpTenantApi;

/// Passport Console Tenant API
#[OpenApi(prefix_path = "/cp/tenant", tag = "crate::iam_enumeration::Tag::Passport")]
impl IamCpTenantApi {
    /// Get Current Tenant
    #[oai(path = "/", method = "get")]
    async fn get(&self, ctx: TardisContextExtractor) -> TardisApiResult<Option<IamTenantSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        if ctx.0.own_paths.is_empty() {
            return TardisResp::ok(None);
        }
        let result = IamTenantServ::peek_item(&IamTenantServ::get_id_by_ctx(&ctx.0, &funs)?, &IamTenantFilterReq::default(), &funs, &ctx.0).await?;
        TardisResp::ok(Some(result))
    }

    /// Find Tenants
    #[oai(path = "/all", method = "get")]
    async fn find(&self) -> TardisApiResult<Vec<IamTenantBoneResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::find_items(
            &IamTenantFilterReq {
                basic: RbumBasicFilterReq {
                    ignore_scope: true,
                    own_paths: Some("".to_string()),
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                ..Default::default()
            },
            None,
            None,
            &funs,
            &IamCertServ::get_anonymous_context(),
        )
        .await?;
        let result = result
            .into_iter()
            .map(|i| IamTenantBoneResp {
                id: i.id,
                name: i.name,
                icon: i.icon,
            })
            .collect();
        TardisResp::ok(result)
    }
}

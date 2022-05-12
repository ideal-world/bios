use crate::basic::dto::iam_filer_dto::IamTenantFilterReq;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::basic::dto::iam_tenant_dto::{IamTenantDetailResp, IamTenantModifyReq, IamTenantSummaryResp};
use crate::basic::serv::iam_tenant_serv::IamTenantServ;
use crate::console_system::dto::iam_cs_tenant_dto::{IamCsTenantAddReq, IamCsTenantModifyReq};
use crate::console_system::serv::iam_cs_tenant_serv::IamCsTenantServ;
use crate::iam_constants;

pub struct IamCsTenantApi;

/// System Console Tenant API
#[OpenApi(prefix_path = "/cs/tenant", tag = "crate::iam_enumeration::Tag::System")]
impl IamCsTenantApi {
    /// Add Tenant
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamCsTenantAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamCsTenantServ::add_tenant(&mut add_req.0, &funs, &cxt.0).await?.1;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Tenant By Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, modify_req: Json<IamCsTenantModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamTenantServ::modify_item(
            &id.0,
            &mut IamTenantModifyReq {
                name: None,
                icon: None,
                sort: None,
                contact_phone: None,
                disabled: modify_req.0.disabled,
                scope_level: None,
            },
            &funs,
            &cxt.0,
        )
        .await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Tenant By Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamTenantDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::get_item(&id.0, &IamTenantFilterReq::default(), &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Tenants
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        q_id: Query<Option<String>>,
        q_name: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamTenantSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamTenantServ::paginate_items(
            &IamTenantFilterReq {
                basic: RbumBasicFilterReq {
                    ids: q_id.0.map(|id| vec![id]),
                    name: q_name.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &cxt.0,
        )
        .await?;
        TardisResp::ok(result)
    }
}

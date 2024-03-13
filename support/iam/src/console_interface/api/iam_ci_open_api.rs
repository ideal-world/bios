use bios_basic::helper::request_helper::add_remote_ip;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem::Request;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisResp, Void};

use crate::basic::dto::iam_open_dto::IamOpenAddProductReq;
use crate::basic::dto::iam_res_dto::IamResAddReq;
use crate::basic::serv::iam_rel_serv::IamRelServ;
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamRelKind, IamResKind};
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

#[derive(Clone, Default)]
pub struct IamCiOpenApi;

/// # Interface Console Manage Open API
///
#[poem_openapi::OpenApi(prefix_path = "/ci/open", tag = "bios_basic::ApiTag::Interface")]
impl IamCiOpenApi {
    /// Add product
    #[oai(path = "/add_product", method = "post")]
    async fn add_product(&self, add_req: Json<IamOpenAddProductReq>, ctx: TardisContextExtractor, request: &Request) -> TardisApiResult<Void> {
        add_remote_ip(request, &ctx.0).await?;
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let product_id = IamResServ::add_item(&mut IamResAddReq {
            code: add_req.0.code,
            name: add_req.0.name,
            kind: IamResKind::Product,
            scope_level: add_req.0.scope_level,
            disabled: add_req.0.disabled,
            ..Default::default()
        }, &funs, &ctx.0).await?;
        for spec in add_req.0.specifications {
            let spec_id = IamResServ::add_item(&mut IamResAddReq {
                code: spec.code,
                name: spec.name,
                kind: IamResKind::Spec,
                scope_level: spec.scope_level,
                disabled: spec.disabled,
                ..Default::default()
            }, &funs, &ctx.0).await?;
            IamRelServ::add_simple_rel(&IamRelKind::IamProductSpec, &product_id, &spec_id, None, None, false, false, &funs, &ctx.0).await?;
        }
        funs.commit().await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(Void{})
    }
}

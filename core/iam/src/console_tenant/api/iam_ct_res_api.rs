use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;

use crate::basic::dto::iam_res_dto::{IamResDetailResp, IamResSummaryResp};
use crate::console_tenant::dto::iam_ct_res_dto::{IamCtResAddReq, IamCtResModifyReq};
use crate::console_tenant::serv::iam_ct_res_serv::IamCtResServ;
use crate::iam_constants;
use crate::iam_enumeration::IamResKind;

pub struct IamCtResApi;

/// Tenant Console Res API
#[OpenApi(prefix_path = "/ct/res", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtResApi {
    /// Add Res
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamCtResAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamCtResServ::add_res(&mut add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Res By Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamCtResModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCtResServ::modify_res(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Res By Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamResDetailResp> {
        let result = IamCtResServ::get_res(&id.0, &iam_constants::get_tardis_inst(), &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Res
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        q_kind: Query<IamResKind>,
        q_id: Query<Option<String>>,
        q_name: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamResSummaryResp>> {
        let result = IamCtResServ::paginate_res(
            q_kind.0,
            q_id.0,
            q_name.0,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &iam_constants::get_tardis_inst(),
            &cxt.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Delete Res By Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamCtResServ::delete_res(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Rel Roles By Res Id
    #[oai(path = "/:id/roles", method = "get")]
    async fn paginate_rel_roles(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<RbumRelAggResp>> {
        let result = IamCtResServ::paginate_rel_roles(
            &id.0,
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &iam_constants::get_tardis_inst(),
            &cxt.0,
        )
        .await?;
        TardisResp::ok(result)
    }
}

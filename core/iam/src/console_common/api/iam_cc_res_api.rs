use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAggResp;
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_filer_dto::IamResFilterReq;
use crate::basic::dto::iam_res_dto::{IamResAddReq, IamResDetailResp, IamResModifyReq, IamResSummaryResp};
use crate::basic::serv::iam_res_serv::IamResServ;
use crate::iam_constants;
use crate::iam_enumeration::{IamRelKind, IamResKind};

pub struct IamCcResApi;

/// Common Console Res API
#[OpenApi(prefix_path = "/cc/res", tag = "crate::iam_enumeration::Tag::Common")]
impl IamCcResApi {
    /// Add Res
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamResAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamResServ::add_item(&mut add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify Res By Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut modify_req: Json<IamResModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamResServ::modify_item(&id.0, &mut modify_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Get Res By Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamResDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::get_item(&id.0, &IamResFilterReq::default(), &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Res
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        kind: Query<IamResKind>,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        role_id: Query<Option<String>>,
        with_sub: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamResSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let rel = role_id.0.map(|role_id| RbumItemRelFilterReq {
            rel_by_from: true,
            tag: Some(IamRelKind::IamResRole.to_string()),
            from_rbum_kind: Some(RbumRelFromKind::Item),
            rel_item_id: Some(role_id),
        });
        let result = IamResServ::paginate_items(
            &IamResFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    with_sub_own_paths: with_sub.0.unwrap_or(false),
                    ..Default::default()
                },
                rel,
                kind: Some(kind.0),
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

    /// Delete Res By Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamResServ::delete_item(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }

    /// Find Rel Roles By Res Id
    #[oai(path = "/:id/roles", method = "get")]
    async fn find_rel_roles(
        &self,
        id: Path<String>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<Vec<RbumRelAggResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamResServ::find_rel_roles(&id.0, false, desc_by_create.0, desc_by_update.0, &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }
}

use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi::{param::Path, param::Query, payload::Json, OpenApi};
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;

use crate::basic::dto::iam_app_dto::{IamAppDetailResp, IamAppModifyReq, IamAppSummaryResp};
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::console_tenant::dto::iam_ct_app_dto::{IamCtAppAddReq, IamCtAppModifyReq};
use crate::console_tenant::serv::iam_ct_app_serv::IamCtAppServ;
use crate::iam_constants;

pub struct IamCtAppApi;

/// Tenant Console App API
#[OpenApi(prefix_path = "/ct/app", tag = "crate::iam_enumeration::Tag::Tenant")]
impl IamCtAppApi {
    /// Add App
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_req: Json<IamCtAppAddReq>, cxt: TardisContextExtractor) -> TardisApiResult<String> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        let result = IamCtAppServ::add_app(&mut add_req.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(result)
    }

    /// Modify App By Id
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, modify_req: Json<IamCtAppModifyReq>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAppServ::modify_item(
            &id.0,
            &mut IamAppModifyReq {
                name: modify_req.0.name.clone(),
                icon: modify_req.0.icon.clone(),
                sort: modify_req.0.sort,
                contact_phone: modify_req.0.contact_phone.clone(),
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

    /// Get App By Id
    #[oai(path = "/:id", method = "get")]
    async fn get(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<IamAppDetailResp> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAppServ::get_item(&id.0, &IamAppFilterReq::default(), &funs, &cxt.0).await?;
        TardisResp::ok(result)
    }

    /// Find Apps
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u64>,
        page_size: Query<u64>,
        cxt: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamAppSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();
        let result = IamAppServ::paginate_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    with_sub_own_paths: true,
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

    /// Delete App By Id
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, cxt: TardisContextExtractor) -> TardisApiResult<Void> {
        let mut funs = iam_constants::get_tardis_inst();
        funs.begin().await?;
        IamAppServ::delete_item_with_all_rels(&id.0, &funs, &cxt.0).await?;
        funs.commit().await?;
        TardisResp::ok(Void {})
    }
}

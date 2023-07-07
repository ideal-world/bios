use crate::basic::dto::iam_app_dto::IamAppSummaryResp;
use crate::basic::dto::iam_filer_dto::IamAppFilterReq;
use crate::basic::serv::iam_app_serv::IamAppServ;
use crate::iam_constants;
use crate::iam_enumeration::IamRelKind;
use bios_basic::rbum::dto::rbum_filer_dto::{RbumBasicFilterReq, RbumItemRelFilterReq};
use bios_basic::rbum::rbum_enumeration::RbumRelFromKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::Query;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp};

#[derive(Clone, Default)]
pub struct IamCpAppApi;

/// Passport Console App API
#[poem_openapi::OpenApi(prefix_path = "/cp/app", tag = "bios_basic::ApiTag::Passport")]
impl IamCpAppApi {
    /// Find Apps by ctx.owner(account_id)
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        name: Query<Option<String>>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<IamAppSummaryResp>> {
        let funs = iam_constants::get_tardis_inst();

        let result = IamAppServ::paginate_items(
            &IamAppFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    with_sub_own_paths: true,
                    enabled: Some(true),
                    ..Default::default()
                },
                rel: Some(RbumItemRelFilterReq {
                    rel_by_from: false,
                    is_left: false,
                    tag: Some(IamRelKind::IamAccountApp.to_string()),
                    from_rbum_kind: Some(RbumRelFromKind::Item),
                    rel_item_id: Some(ctx.0.owner.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        ctx.0.execute_task().await?;
        TardisResp::ok(result)
    }
}

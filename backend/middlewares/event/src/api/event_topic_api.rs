use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::event_dto::{EventTopicAddOrModifyReq, EventTopicFilterReq, EventTopicInfoResp};
use crate::event_constants::get_tardis_inst;
use crate::serv::event_topic_serv::EventDefServ;
#[derive(Clone)]
pub struct EventTopicApi;

/// Event Topic API
/// 
/// 事件主题API
#[poem_openapi::OpenApi(prefix_path = "/topic")]
impl EventTopicApi {
    /// Add Event Definition
    /// 
    /// 添加事件主题
    #[oai(path = "/", method = "post")]
    async fn add(&self, mut add_or_modify_req: Json<EventTopicAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let funs = get_tardis_inst();
        let id = EventDefServ::add_item(&mut add_or_modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(id)
    }

    /// Modify Event Definition
    /// 
    /// 修改事件主题
    #[oai(path = "/:id", method = "put")]
    async fn modify(&self, id: Path<String>, mut add_or_modify_req: Json<EventTopicAddOrModifyReq>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        EventDefServ::modify_item(&id.0, &mut add_or_modify_req.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Event Definition
    /// 
    /// 删除事件主题
    #[oai(path = "/:id", method = "delete")]
    async fn delete(&self, id: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        EventDefServ::delete_item(&id.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Event Definitions
    /// 
    /// 查找事件主题
    #[oai(path = "/", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        code: Query<Option<String>>,
        name: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<EventTopicInfoResp>> {
        let funs = get_tardis_inst();
        let result = EventDefServ::paginate_items(
            &EventTopicFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    name: name.0,
                    code: code.0,
                    ..Default::default()
                },
            },
            page_number.0,
            page_size.0,
            desc_by_create.0,
            desc_by_update.0,
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }
}

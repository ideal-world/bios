use asteroid_mq::prelude::TopicCode;
use bios_basic::rbum::dto::rbum_filer_dto::RbumBasicFilterReq;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::web::context_extractor::TardisContextExtractor;
use tardis::web::poem_openapi;
use tardis::web::poem_openapi::param::{Path, Query};
use tardis::web::poem_openapi::payload::Json;
use tardis::web::web_resp::{TardisApiResult, TardisPage, TardisResp, Void};

use crate::dto::event_dto::{EventTopicConfig, EventTopicFilterReq, EventTopicInfoResp, SetTopicAuth};
use crate::event_constants::get_tardis_inst;
use crate::serv::event_topic_serv::EventTopicServ;
#[derive(Clone)]
pub struct EventTopicApi;

/// Event Topic API
///
/// 事件主题API
#[poem_openapi::OpenApi(prefix_path = "/ci/topic")]
impl EventTopicApi {
    /// Add Event Definition
    ///
    /// 添加事件主题
    #[oai(path = "/", method = "post")]
    async fn add(&self, add_or_modify_req: Json<EventTopicConfig>, ctx: TardisContextExtractor) -> TardisApiResult<String> {
        let funs = get_tardis_inst();
        let mut add_or_modify_req = add_or_modify_req.0.into_rbum_req();
        let id = EventTopicServ::add_item(&mut add_or_modify_req, &funs, &ctx.0).await?;
        TardisResp::ok(id)
    }

    /// Add Event Definition
    ///
    /// 添加事件主题
    #[oai(path = "/", method = "get")]
    async fn get_by_code(&self, topic_code: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<Option<EventTopicInfoResp>> {
        let funs = get_tardis_inst();
        let result = EventTopicServ::find_one_item(
            &EventTopicFilterReq {
                basic: Default::default(),
                topic_code: Some(topic_code.0),
            },
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(result)
    }

    /// Modify Event Definition
    ///
    /// 修改事件主题
    #[oai(path = "/", method = "put")]
    async fn modify(&self, id: Query<String>, add_or_modify_req: Json<EventTopicConfig>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        let mut add_or_modify_req = add_or_modify_req.0.into_rbum_req();
        EventTopicServ::modify_item(&id.0, &mut add_or_modify_req, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Delete Event Definition
    ///
    /// 删除事件主题
    #[oai(path = "/", method = "delete")]
    async fn delete(&self, id: Query<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        EventTopicServ::delete_item(&id.0, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }

    /// Find Event Definitions
    ///
    /// 查找事件主题
    #[oai(path = "/paged", method = "get")]
    async fn paginate(
        &self,
        id: Query<Option<String>>,
        topic_code: Query<Option<String>>,
        page_number: Query<u32>,
        page_size: Query<u32>,
        desc_by_create: Query<Option<bool>>,
        desc_by_update: Query<Option<bool>>,
        ctx: TardisContextExtractor,
    ) -> TardisApiResult<TardisPage<EventTopicInfoResp>> {
        let funs = get_tardis_inst();
        let result = EventTopicServ::paginate_items(
            &EventTopicFilterReq {
                basic: RbumBasicFilterReq {
                    ids: id.0.map(|id| vec![id]),
                    // name: topic_code.as_ref().map(|code| format_code(&code)),
                    // code: topic_code.as_ref().map(|code| format_code(&code)),
                    ..Default::default()
                },
                topic_code: topic_code.0,
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

    /// Register user to topic
    ///
    /// 注册用户到主题
    #[oai(path = "/:topic_code/register", method = "put")]
    async fn register(&self, topic_code: Path<String>, read: Query<bool>, write: Query<bool>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        EventTopicServ::register_user(
            SetTopicAuth {
                topic: topic_code.0,
                read: read.0,
                write: write.0,
            },
            &funs,
            &ctx.0,
        )
        .await?;
        TardisResp::ok(Void {})
    }

    /// Register user to topic
    ///
    /// 注销用户
    #[oai(path = "/:topic_code/unregister", method = "delete")]
    async fn unregister(&self, topic_code: Path<String>, ctx: TardisContextExtractor) -> TardisApiResult<Void> {
        let funs = get_tardis_inst();
        EventTopicServ::unregister_user(TopicCode::new(topic_code.0), &ctx.0.ak, &funs, &ctx.0).await?;
        TardisResp::ok(Void {})
    }
}

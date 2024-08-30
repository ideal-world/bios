use async_trait::async_trait;
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use bios_sdk_invoke::clients::event_client::mq_node;
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::SelectStatement;
use tardis::db::sea_orm::{EntityName, Set};
use tardis::TardisFunsInst;

use crate::domain::event_topic;
use crate::dto::event_dto::{EventTopicAddOrModifyReq, EventTopicFilterReq, EventTopicInfoResp};
use crate::event_config::EventInfoManager;

pub struct EventDefServ;

#[async_trait]
impl RbumItemCrudOperation<event_topic::ActiveModel, EventTopicAddOrModifyReq, EventTopicAddOrModifyReq, EventTopicInfoResp, EventTopicInfoResp, EventTopicFilterReq>
    for EventDefServ
{
    fn get_ext_table_name() -> &'static str {
        event_topic::Entity.table_name()
    }

    fn get_rbum_kind_id() -> Option<String> {
        Some(EventInfoManager::get_config(|conf| conf.kind_id.clone()))
    }

    fn get_rbum_domain_id() -> Option<String> {
        Some(EventInfoManager::get_config(|conf| conf.domain_id.clone()))
    }

    async fn package_item_add(add_req: &EventTopicAddOrModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<RbumItemKernelAddReq> {
        Ok(RbumItemKernelAddReq {
            code: Some(add_req.code.clone().into()),
            name: add_req.name.clone().into(),
            scope_level: Some(RbumScopeLevelKind::Root),
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &EventTopicAddOrModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<event_topic::ActiveModel> {
        Ok(event_topic::ActiveModel {
            id: Set(id.to_string()),
            blocking: Set(add_req.blocking),
            overflow_policy: Set(add_req.overflow_policy.clone()),
            overflow_size: Set(add_req.overflow_size),
            topic_code: Set(add_req.code.clone()),
            ..Default::default()
        })
    }

    async fn after_add_item(id: &str, add_req: &mut EventTopicAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let _key = add_req.code.to_string();
        let value = Self::get_item(id, &EventTopicFilterReq::default(), funs, ctx).await?;
        mq_node().new_topic(value.into_topic_config()).await.map_err(|e| TardisError::internal_error(&e.to_string(), "event-fail-to-create-topic"))?;
        Ok(())
    }

    async fn package_item_modify(_: &str, modify_req: &EventTopicAddOrModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
        Ok(Some(RbumItemKernelModifyReq {
            code: Some(modify_req.code.clone().into()),
            name: Some(modify_req.name.clone().into()),
            ..Default::default()
        }))
    }

    async fn package_ext_modify(_: &str, modify_req: &EventTopicAddOrModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<event_topic::ActiveModel>> {
        let event_topic = event_topic::ActiveModel {
            blocking: Set(modify_req.blocking),
            overflow_policy: Set(modify_req.overflow_policy.clone()),
            overflow_size: Set(modify_req.overflow_size),
            topic_code: Set(modify_req.code.clone()),
            ..Default::default()
        };
        Ok(Some(event_topic))
    }

    // async fn after_modify_item(id: &str, modify_req: &mut EventTopicAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    //     Ok(())
    // }

    // async fn after_delete_item(id: &str, _: &Option<EventTopicInfoResp>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    //     Ok(())
    // }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, _: &EventTopicFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query
            .column((event_topic::Entity, event_topic::Column::Blocking))
            .column((event_topic::Entity, event_topic::Column::OverflowPolicy))
            .column((event_topic::Entity, event_topic::Column::OverflowSize))
            .column((event_topic::Entity, event_topic::Column::TopicCode));
        Ok(())
    }
}

impl EventDefServ {
    pub async fn init(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        // let defs = Self::find_items(&EventTopicFilterReq::default(), None, None, funs, ctx).await?;

        Ok(())
    }
}

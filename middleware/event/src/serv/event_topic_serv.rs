use async_trait::async_trait;
use bios_basic::rbum::dto::rbum_item_dto::{RbumItemKernelAddReq, RbumItemKernelModifyReq};
use bios_basic::rbum::rbum_enumeration::RbumScopeLevelKind;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemCrudOperation;
use tardis::basic::dto::TardisContext;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm::sea_query::SelectStatement;
use tardis::db::sea_orm::{EntityName, Set};
use tardis::{tardis_static, TardisFunsInst};

use crate::domain::event_topic;
use crate::dto::event_dto::{EventTopicAddOrModifyReq, EventTopicFilterReq, EventTopicInfoResp};
use crate::event_config::EventInfoManager;

use tardis::cluster::cluster_hashmap::ClusterStaticHashMap;

tardis_static! {
    pub(crate) topics: ClusterStaticHashMap<String, EventTopicInfoResp> = ClusterStaticHashMap::new("bios/event/topics");
}

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
            code: Some(add_req.code.clone()),
            name: add_req.name.clone(),
            scope_level: Some(RbumScopeLevelKind::Root),
            ..Default::default()
        })
    }

    async fn package_ext_add(id: &str, add_req: &EventTopicAddOrModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<event_topic::ActiveModel> {
        Ok(event_topic::ActiveModel {
            id: Set(id.to_string()),
            save_message: Set(add_req.save_message),
            need_mgr: Set(add_req.need_mgr),
            queue_size: Set(add_req.queue_size),
            use_sk: Set(add_req.use_sk.as_deref().unwrap_or("").to_string()),
            mgr_sk: Set(add_req.mgr_sk.as_deref().unwrap_or("").to_string()),
            ..Default::default()
        })
    }

    async fn after_add_item(id: &str, add_req: &mut EventTopicAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let key = add_req.code.to_string();
        let value = Self::get_item(id, &EventTopicFilterReq::default(), funs, ctx).await?;
        topics().insert(key, value).await?;
        Ok(())
    }

    async fn package_item_modify(_: &str, modify_req: &EventTopicAddOrModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<RbumItemKernelModifyReq>> {
        Ok(Some(RbumItemKernelModifyReq {
            code: Some(modify_req.code.clone()),
            name: Some(modify_req.name.clone()),
            ..Default::default()
        }))
    }

    async fn package_ext_modify(_: &str, modify_req: &EventTopicAddOrModifyReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<Option<event_topic::ActiveModel>> {
        let event_topic = event_topic::ActiveModel {
            save_message: Set(modify_req.save_message),
            need_mgr: Set(modify_req.need_mgr),
            queue_size: Set(modify_req.queue_size),
            use_sk: Set(modify_req.use_sk.as_deref().unwrap_or("").to_string()),
            mgr_sk: Set(modify_req.mgr_sk.as_deref().unwrap_or("").to_string()),
            ..Default::default()
        };
        Ok(Some(event_topic))
    }

    async fn after_modify_item(id: &str, modify_req: &mut EventTopicAddOrModifyReq, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let key = modify_req.code.to_string();
        let value = Self::get_item(id, &EventTopicFilterReq::default(), funs, ctx).await?;
        topics().insert(key, value).await?;
        Ok(())
    }

    async fn after_delete_item(id: &str, _: &Option<EventTopicInfoResp>, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let item = Self::get_item(id, &EventTopicFilterReq::default(), funs, ctx).await?;
        topics().remove(item.code).await?;
        Ok(())
    }

    async fn package_ext_query(query: &mut SelectStatement, _: bool, _: &EventTopicFilterReq, _: &TardisFunsInst, _: &TardisContext) -> TardisResult<()> {
        query
            .column((event_topic::Entity, event_topic::Column::SaveMessage))
            .column((event_topic::Entity, event_topic::Column::NeedMgr))
            .column((event_topic::Entity, event_topic::Column::QueueSize))
            .column((event_topic::Entity, event_topic::Column::UseSk))
            .column((event_topic::Entity, event_topic::Column::MgrSk));
        Ok(())
    }
}

impl EventDefServ {
    pub async fn init(funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
        let defs = Self::find_items(&EventTopicFilterReq::default(), None, None, funs, ctx).await?;
        let mut cache_defs = topics().local().write().await;
        for def in defs {
            cache_defs.insert(def.code.to_string(), def);
        }
        Ok(())
    }
}

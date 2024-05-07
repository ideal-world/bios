use crate::dto::*;
use crate::fill_by_add_req;
use crate::fill_by_mod_req;
use tardis::chrono::{self, DateTime, Utc};
use tardis::db::sea_orm;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "reach_trigger_instance_config")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[tardis_entity(custom_type = "string")]
    pub id: Nanoid,
    /// 所有者路径
    #[fill_ctx(fill = "own_paths")]
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub own_paths: String,
    /// 所有者
    #[fill_ctx]
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub owner: String,
    /// 创建时间
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub create_time: DateTime<Utc>,
    /// 更新时间
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub update_time: DateTime<Utc>,
    #[tardis_entity(custom_type = "string", custom_len = "512")]
    /// 关联的触发场景id
    pub rel_reach_trigger_scene_id: String,
    #[tardis_entity(custom_type = "string", custom_len = "512")]
    /// 关联的触达通道
    pub rel_reach_channel: ReachChannelKind,
    #[tardis_entity(custom_type = "string", custom_len = "512")]
    /// 关联资源项id
    pub rel_item_id: String,
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    /// 接收组编码
    pub receive_group_code: String,
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    /// 接收组名称
    pub receive_group_name: String,
}

impl From<&ReachTriggerInstanceConfigAddReq> for ActiveModel {
    fn from(value: &ReachTriggerInstanceConfigAddReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            create_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(value => {
            rel_reach_trigger_scene_id,
            rel_reach_channel,
            rel_item_id,
            receive_group_code,
            receive_group_name,
        } model);
        model
    }
}

impl From<&ReachTriggerInstanceConfigModifyReq> for ActiveModel {
    fn from(value: &ReachTriggerInstanceConfigModifyReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_mod_req!(value => {
            rel_reach_trigger_scene_id,
            rel_reach_channel,
            rel_item_id,
            receive_group_code,
            receive_group_name,
        } model);
        model
    }
}

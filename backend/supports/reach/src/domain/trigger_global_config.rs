use crate::dto::*;
use crate::fill_by_add_req;
use crate::fill_by_mod_req;
use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, DateTime, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;

use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "reach_trigger_global_config")]
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
    /// 关联的触发场景id
    #[tardis_entity(custom_type = "string", custom_len = "512")]
    pub rel_reach_trigger_scene_id: String,
    /// 关联的触达通道
    #[tardis_entity(custom_type = "string", custom_len = "512")]
    pub rel_reach_channel: ReachChannelKind,
    /// 用户触达消息签名Id
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub rel_reach_msg_signature_id: String,
    /// 用户触达消息模板Id
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub rel_reach_msg_template_id: String,
}

impl From<&ReachTriggerGlobalConfigAddReq> for ActiveModel {
    fn from(value: &ReachTriggerGlobalConfigAddReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            create_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(value => {
            rel_reach_trigger_scene_id,
            rel_reach_channel,
            rel_reach_msg_signature_id,
            rel_reach_msg_template_id,
        } model);
        model
    }
}

impl From<&ReachTriggerGlobalConfigModifyReq> for ActiveModel {
    fn from(value: &ReachTriggerGlobalConfigModifyReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_mod_req!(value => {
            rel_reach_trigger_scene_id,
            rel_reach_channel,
            rel_reach_msg_signature_id,
            rel_reach_msg_template_id,
        } model);
        model
    }
}

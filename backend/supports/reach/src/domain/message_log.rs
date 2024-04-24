use tardis::chrono::{self, DateTime, Utc};
use tardis::db::sea_orm;

use crate::dto::*;
use crate::fill_by_add_req;
use tardis::db::sea_orm::*;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "reach_msg_log")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[tardis_entity(custom_type = "string")]
    pub id: Nanoid,
    /// 所有者路径
    #[fill_ctx(fill = "own_paths")]
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub own_paths: String,
    /// 所有者
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    #[fill_ctx]
    pub owner: String,
    /// 创建时间
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub create_time: DateTime<Utc>,
    /// 更新时间
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub update_time: DateTime<Utc>,
    /// 关联接收人Id
    pub rel_account_id: String,
    /// 免扰时间，HH::MM-HH:MM
    pub dnd_time: String,
    /// 免扰策略
    pub dnd_strategy: ReachDndStrategyKind,
    /// 开始时间
    pub start_time: chrono::DateTime<Utc>,
    /// 结束时间
    pub end_time: chrono::DateTime<Utc>,
    /// 完成时间
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub finish_time: Option<chrono::DateTime<Utc>>,
    /// 是否失败
    pub failure: bool,
    /// 失败原因
    pub fail_message: String,
    /// 用户触达消息Id
    pub rel_reach_message_id: String,
}

impl From<&ReachMsgLogAddReq> for ActiveModel {
    fn from(value: &ReachMsgLogAddReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            create_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(
            value => {
                rel_account_id,
                dnd_time,
                dnd_strategy,
                start_time,
                end_time,
                failure,
                fail_message,
                rel_reach_message_id,
            } model
        );

        model
    }
}

impl From<&ReachMsgLogModifyReq> for ActiveModel {
    fn from(value: &ReachMsgLogModifyReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_add_req!(
            value => {
                rel_account_id,
                dnd_time,
                dnd_strategy,
                start_time,
                end_time,
                failure,
                fail_message,
            } model
        );
        model
    }
}

use tardis::chrono::{DateTime, Utc};
use tardis::db::sea_orm;

use tardis::db::sea_orm::*;

use crate::dto::*;
use crate::{fill_by_add_req, fill_by_mod_req};
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
/// 用户触达消息
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "reach_message")]
pub struct Model {
    #[sea_orm(primary_key)]
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
    /// 发件人，可随意填写，分号分隔
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub from_res: String,
    /// 关联的触达通道
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub rel_reach_channel: ReachChannelKind,
    /// 用户触达接收类型
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub receive_kind: ReachReceiveKind,
    /// 接收主体，分号分隔
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub to_res_ids: String,
    /// 用户触达签名Id
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub rel_reach_msg_signature_id: String,
    /// 用户触达模板Id
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub rel_reach_msg_template_id: String,
    /// 替换参数，例如：{name1:value1,name2:value2}
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub content_replace: String,
    /// 触达状态
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub reach_status: ReachStatusKind,
}

impl From<&ReachMessageAddReq> for ActiveModel {
    fn from(value: &ReachMessageAddReq) -> Self {
        let mut model = ActiveModel { ..Default::default() };
        fill_by_add_req! {
            value => {
                from_res,
                rel_reach_channel,
                receive_kind,
                to_res_ids,
                rel_reach_msg_signature_id,
                rel_reach_msg_template_id,
                reach_status,
                content_replace,
            } model
        };
        model
    }
}

impl From<&ReachMessageModifyReq> for ActiveModel {
    fn from(value: &ReachMessageModifyReq) -> Self {
        let mut model = ActiveModel { ..Default::default() };
        fill_by_mod_req! {
            value => {
                from_res,
                rel_reach_channel,
                receive_kind,
                to_res_ids,
                rel_reach_msg_signature_id,
                rel_reach_msg_template_id,
                reach_status,
                content_replace,
            } model
        };
        model
    }
}

use crate::dto::*;
use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, DateTime, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;

use tardis::db::sea_orm::sea_query::{ColumnDef, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

use crate::fill_by_mod_req;
use tardis::{TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation};
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateEntity, TardisEmptyBehavior, TardisEmptyRelation)]
#[sea_orm(table_name = "reach_msg_signature")]
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
    /// 名称
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub name: String,
    /// 说明
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub note: String,
    /// 内容
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub content: String,
    /// 来源
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub source: String,
    /// 关联的触达通道
    #[tardis_entity(custom_type = "string", custom_len = "255")]
    pub rel_reach_channel: ReachChannelKind,
}

impl From<&ReachMsgSignatureAddReq> for ActiveModel {
    fn from(value: &ReachMsgSignatureAddReq) -> Self {
        ActiveModel {
            name: Set(value.name.clone()),
            note: Set(value.note.clone()),
            content: Set(value.content.clone()),
            source: Set(value.source.clone()),
            rel_reach_channel: Set(value.rel_reach_channel),
            create_time: Set(chrono::Utc::now()),
            update_time: Set(chrono::Utc::now()),
            ..Default::default()
        }
    }
}

impl From<&ReachMsgSignatureModifyReq> for ActiveModel {
    fn from(value: &ReachMsgSignatureModifyReq) -> Self {
        let mut model = ActiveModel {
            update_time: Set(chrono::Utc::now()),
            ..Default::default()
        };
        fill_by_mod_req!(
            value => {
                name,
                note,
                content,
                source,
                rel_reach_channel: Copy
            } model
        );
        model
    }
}

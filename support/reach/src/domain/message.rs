use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;

use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

use crate::dto::{ReachChannelKind, ReachMessageAddReq, ReachMessageModifyReq, ReachReceiveKind, ReachStatusKind};
use crate::{fill_by_add_req, fill_by_mod_req};

/// 用户触达消息
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "reach_message")]
pub struct Model {
    #[sea_orm(primary_key, generator = "uuid")]
    pub id: String,
    /// 所有者路径
    #[sea_orm(column_name = "own_paths", column_type = "Char(Some(255))")]
    pub own_paths: String,
    /// 发件人，可随意填写，分号分隔
    #[sea_orm(column_name = "from_res", column_type = "Char(Some(255))")]
    pub from_res: String,
    /// 关联的触达通道
    #[sea_orm(column_name = "rel_reach_channel", column_type = "Char(Some(255))")]
    pub rel_reach_channel: ReachChannelKind,
    /// 用户触达接收类型
    #[sea_orm(column_name = "receive_kind", column_type = "Char(Some(255))")]
    pub receive_kind: ReachReceiveKind,
    /// 接收主体，分号分隔
    #[sea_orm(column_name = "to_res_ids", column_type = "Char(Some(255))")]
    pub to_res_ids: String,
    /// 用户触达签名Id
    #[sea_orm(column_name = "rel_reach_msg_signature_id", column_type = "Char(Some(255))")]
    pub rel_reach_msg_signature_id: String,
    /// 用户触达模板Id
    #[sea_orm(column_name = "rel_reach_msg_template_id", column_type = "Char(Some(255))")]
    pub rel_reach_msg_template_id: String,
    /// 替换参数，例如：{name1:value1,name2:value2}
    #[sea_orm(column_name = "content_replace", column_type = "Char(Some(255))")]
    pub content_replace: String,
    /// 触达状态
    #[sea_orm(column_name = "reach_status", column_type = "Char(Some(255))")]
    pub reach_status: ReachStatusKind,
}
impl ActiveModelBehavior for ActiveModel {}

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

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
        }
    }
    fn create_table_statement(_db: DbBackend) -> TableCreateStatement {
        let mut builder = Table::create();

        builder
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::FromRes).not_null().string())
            .col(ColumnDef::new(Column::RelReachChannel).not_null().string())
            .col(ColumnDef::new(Column::ReceiveKind).not_null().string())
            .col(ColumnDef::new(Column::ToResIds).not_null().string())
            .col(ColumnDef::new(Column::RelReachMsgSignatureId).not_null().string())
            .col(ColumnDef::new(Column::RelReachMsgTemplateId).not_null().string())
            .col(ColumnDef::new(Column::ContentReplace).not_null().string())
            .col(ColumnDef::new(Column::ReachStatus).not_null().string());
        builder.to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        vec![]
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

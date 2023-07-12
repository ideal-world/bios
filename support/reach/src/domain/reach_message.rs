// /**
//  * 用户触达消息
//  *
//  * @author gudaoxuri
//  */
// @EqualsAndHashCode(callSuper = true)
// @Entity
// @Table(name = "reach_message")
// @Data
// @SuperBuilder
// @NoArgsConstructor
// @AllArgsConstructor
// public class ReachMessage extends RbumItemExtEntity {

//     @Column(nullable = false, columnDefinition = "varchar(2000) comment '发件人，可随意填写，分号分隔'")
//     private String fromRes;

//     @Column(nullable = false, columnDefinition = "varchar(255) comment '关联的触达通道'")
//     @Enumerated(EnumType.STRING)
//     private ReachChannelKind relReachChannel;

//     @Column(nullable = false, columnDefinition = "varchar(255) comment '用户触达接收类型'")
//     @Enumerated(EnumType.STRING)
//     private ReachReceiveKind receiveKind;

//     @Column(nullable = false, columnDefinition = "varchar(2000) comment '接收主体，分号分隔'")
//     private String toResIds;

//     @Column(nullable = false, columnDefinition = "varchar(255) comment '用户触达签名Id'")
//     private String relReachMsgSignatureId;

//     @Column(nullable = false, columnDefinition = "varchar(255) comment '用户触达模板Id'")
//     private String relReachMsgTemplateId;

//     @Column(nullable = false, columnDefinition = "varchar(255) comment '替换参数，例如：{name1:value1,name2:value2}'")
//     private String contentReplace;

//     @Column(nullable = false, columnDefinition = "varchar(255) comment '触达状态'")
//     @Enumerated(EnumType.STRING)
//     private ReachStatusKind reachStatus;

// }

use tardis::basic::dto::TardisContext;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::Uuid;
use tardis::db::sea_orm::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

use crate::dto::ReachStatusKind;

/// 用户触达消息
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "reach_message")]
pub struct Model {
    #[sea_orm(primary_key, generator = "uuid")]
    pub id: Uuid,
    /// 所有者路径
    #[sea_orm(column_name = "own_paths", column_type="Char(Some(255))")]
    pub own_paths: String,
    /// 发件人，可随意填写，分号分隔
    #[sea_orm(column_name = "from_res", column_type="Char(Some(255))")]
    pub from_res: String,
    /// 关联的触达通道
    #[sea_orm(column_name = "rel_reach_channel", column_type="Char(Some(255))")]
    pub rel_reach_channel: String,
    /// 用户触达接收类型
    #[sea_orm(column_name = "receive_kind", column_type="Char(Some(255))")]
    pub receive_kind: String,
    /// 接收主体，分号分隔
    #[sea_orm(column_name = "to_res_ids", column_type="Char(Some(255))")]
    pub to_res_ids: String,
    /// 用户触达签名Id
    #[sea_orm(column_name = "rel_reach_msg_signature_id", column_type="Char(Some(255))")]
    pub rel_reach_msg_signature_id: String,
    /// 用户触达模板Id
    #[sea_orm(column_name = "rel_reach_msg_template_id", column_type="Char(Some(255))")]
    pub rel_reach_msg_template_id: String,
    /// 替换参数，例如：{name1:value1,name2:value2}
    #[sea_orm(column_name = "content_replace", column_type="Char(Some(255))")]
    pub content_replace: String,
    /// 触达状态
    #[sea_orm(column_name = "reach_status", column_type="Char(Some(255))")]
    pub reach_status: ReachStatusKind,
}
impl ActiveModelBehavior for ActiveModel {}

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
        }
    }
    fn create_table_statement(db: DbBackend) -> TableCreateStatement {
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
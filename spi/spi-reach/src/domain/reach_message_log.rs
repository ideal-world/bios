/* /**
 * 用户触达消息日志
 *
 * TO ES
 *
 * @author gudaoxuri
 */
@Entity
@Table(name = "reach_msg_log")
@Data
@SuperBuilder
@NoArgsConstructor
@AllArgsConstructor
public class ReachMsgLog extends RbumBasicEntity {

    @Column(nullable = false, columnDefinition = "varchar(255) comment '关联接收人Id'")
    private String relAccountId;

    @Column(nullable = false, columnDefinition = "varchar(255) comment '免扰时间，HH::MM-HH:MM'")
    private String dndTime;

    @Column(nullable = false, columnDefinition = "varchar(255) comment '免扰策略'")
    @Enumerated(EnumType.STRING)
    private ReachDndStrategyKind dndStrategy;

    /**
     * 对于 {@link ReachChannelKind#INBOX} 类型，在开始时间之前不会显示，
     * 对其它类型而言如存在开始时间且视为定时消息的发送时间
     */
    @Column(columnDefinition = "timestamp null comment '开始时间'")
    private Date startTime;

    /**
     * 对于 {@link ReachChannelKind#INBOX} 类型，在结束时间之后不会显示
     */
    @Column(columnDefinition = "timestamp null comment '结束时间'")
    private Date endTime;

    @Column(columnDefinition = "timestamp default CURRENT_TIMESTAMP comment '完成时间'",
            insertable = false, updatable = false)
    protected Date finishTime;

    @Column(nullable = false, columnDefinition = "tinyint(1) comment '是否失败'")
    private Boolean failure;

    @Column(nullable = false, columnDefinition = "varchar(2000) comment '失败原因'")
    private String failMessage;

    @Column(nullable = false,
            columnDefinition = "varchar(255) comment '用户触达消息Id'")
    private String relReachMessageId;

} */

use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::Uuid;
use tardis::db::sea_orm::sea_query::{ColumnDef, Index, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;

use crate::dto::ReachStatusKind;
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "reach_msg_log")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, generator = "uuid")]
    pub id: Uuid,
    /// 关联接收人Id
    pub rel_account_id: String,
    /// 免扰时间，HH::MM-HH:MM
    pub dnd_time: String,
    /// 免扰策略
    pub dnd_strategy: String,
    /// 开始时间
    pub start_time: Option<chrono::DateTime<Utc>>,
    /// 结束时间
    pub end_time: Option<chrono::DateTime<Utc>>,
    /// 完成时间
    pub finish_time: Option<chrono::DateTime<Utc>>,
    /// 是否失败
    pub failure: bool,
    /// 失败原因
    pub fail_message: String,
    /// 用户触达消息Id
    pub rel_reach_message_id: String,

    pub own_paths: String,
    pub owner: String,
    pub create_time: chrono::DateTime<Utc>,
    pub update_time: chrono::DateTime<Utc>,
}

impl ActiveModelBehavior for ActiveModel {}

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
            self.owner = Set(ctx.owner.to_string());
        }
    }

    fn create_table_statement(db:DbBackend) -> TableCreateStatement {
        let mut builder = Table::create();
        builder.table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            // Specific
            .col(ColumnDef::new(Column::RelAccountId).not_null().string())
            .col(ColumnDef::new(Column::DndTime).not_null().string())
            .col(ColumnDef::new(Column::DndStrategy).not_null().string())
            .col(ColumnDef::new(Column::StartTime).timestamp())
            .col(ColumnDef::new(Column::EndTime).timestamp())
            .col(ColumnDef::new(Column::FinishTime).timestamp())
            .col(ColumnDef::new(Column::Failure).not_null().boolean())
            .col(ColumnDef::new(Column::FailMessage).not_null().string())
            .col(ColumnDef::new(Column::RelReachMessageId).not_null().string())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string());
        if db == DatabaseBackend::Postgres {
            builder
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone());
        } else {
            builder
                .engine("InnoDB")
                .character_set("utf8mb4")
                .collate("utf8mb4_0900_as_cs")
                .col(ColumnDef::new(Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp())
                .col(ColumnDef::new(Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).timestamp());
        }
        builder.to_owned()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
use crate::dto::flow_inst_dto::{FlowInstTransitionInfo, FlowOperationContext};
use tardis::basic::dto::TardisContext;
use tardis::chrono::Utc;
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::Json;
use tardis::db::sea_orm::sea_query::{ColumnDef, Index};
use tardis::db::sea_orm::sea_query::{IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::{chrono, TardisCreateEntity, TardisCreateIndex, TardisEmptyBehavior, TardisEmptyRelation};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, TardisCreateIndex)]
#[sea_orm(table_name = "flow_inst")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[index(index_id = "idx_001")]
    pub rel_flow_model_id: String,
    #[index(index_id = "idx_002")]
    pub rel_res_id: String,

    #[index(index_id = "idx_003")]
    pub current_state_id: String,
    pub current_vars: Option<Json>,

    pub create_vars: Option<Json>,
    pub create_ctx: FlowOperationContext,
    #[index(index_id = "idx_004")]
    #[sea_orm(extra = "DEFAULT CURRENT_TIMESTAMP")]
    pub create_time: chrono::DateTime<Utc>,

    pub finish_ctx: Option<FlowOperationContext>,
    #[index(index_id = "idx_005")]
    pub finish_time: Option<chrono::DateTime<Utc>>,
    pub finish_abort: Option<bool>,
    pub output_message: Option<String>,

    #[index(index_id = "idx_006", full_text)]
    #[sea_orm(column_type = "JsonBinary")]
    pub transitions: Option<Json>,

    pub own_paths: String,
}

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
            .col(ColumnDef::new(Column::RelFlowModelId).not_null().string())
            .col(ColumnDef::new(Column::RelResId).not_null().string())
            .col(ColumnDef::new(Column::CurrentStateId).not_null().string())
            .col(ColumnDef::new(Column::CurrentVars).json_binary())
            .col(ColumnDef::new(Column::CreateVars).json_binary())
            .col(ColumnDef::new(Column::CreateCtx).not_null().json_binary())
            .col(ColumnDef::new(Column::CreateTime).not_null().date_time().extra("DEFAULT CURRENT_TIMESTAMP".to_string()).timestamp_with_time_zone())
            .col(ColumnDef::new(Column::FinishCtx).json_binary())
            .col(ColumnDef::new(Column::FinishTime).date_time())
            .col(ColumnDef::new(Column::FinishAbort).boolean())
            .col(ColumnDef::new(Column::OutputMessage).string())
            .col(ColumnDef::new(Column::Transitions).json_binary())
            .col(ColumnDef::new(Column::OwnPaths).not_null().string());
        builder.to_owned()
    }

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        tardis_create_index_statement()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

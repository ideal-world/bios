use tardis::basic::dto::TardisContext;
use tardis::chrono::{self, Utc};
use tardis::db::reldb_client::TardisActiveModel;
use tardis::db::sea_orm;
use tardis::db::sea_orm::prelude::*;
use tardis::db::sea_orm::sea_query::{ColumnDef, IndexCreateStatement, Table, TableCreateStatement};
use tardis::db::sea_orm::*;
use tardis::TardisCreateIndex;

/// Resource domain model
/// 
/// 资源域模型
///
/// The resource domain is the unit of ownership of the resource, indicating the owner of the resource.
/// Each resource is required to belong to a resource domain.
/// 
/// 资源域是资源的归属单位，表示资源的所有者。每个资源都要求归属于一个资源域。
///
/// E.g. All menu resources are provided by IAM components, and all IaaS resources are provided by CMDB components.
/// IAM components and CMDB components are resource domains.
/// 
/// 例如：所有菜单资源由IAM组件提供，所有IaaS资源由CMDB组件提供。IAM组件和CMDB组件是资源域。
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, TardisCreateIndex)]
#[sea_orm(table_name = "rbum_domain")]
pub struct Model {
    /// Resource domain id
    /// 
    /// 资源域id
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    /// Resource domain code
    /// 
    /// 资源域编码
    /// 
    /// Global unique
    /// 
    /// 全局唯一
    /// 
    /// Which is required to conform to the host specification in the uri, matching the regular: ^[a-z0-9-.]+$.
    /// 
    /// 需要符合uri中的host规范，匹配正则：^[a-z0-9-.]+$。
    #[index(unique)]
    pub code: String,
    /// Resource domain name
    /// 
    /// 资源域名称
    pub name: String,
    /// Resource domain note
    /// 
    /// 资源域备注
    pub note: String,
    /// Resource domain icon
    /// 
    /// 资源域图标
    pub icon: String,
    /// Resource domain sort
    /// 
    /// 资源域排序
    pub sort: i64,

    pub scope_level: i16,

    #[index]
    pub own_paths: String,
    pub owner: String,
    pub create_time: chrono::DateTime<Utc>,
    pub update_time: chrono::DateTime<Utc>,
    pub create_by: String,
    pub update_by: String,
}

impl TardisActiveModel for ActiveModel {
    fn fill_ctx(&mut self, ctx: &TardisContext, is_insert: bool) {
        if is_insert {
            self.own_paths = Set(ctx.own_paths.to_string());
            self.owner = Set(ctx.owner.to_string());
            self.create_by = Set(ctx.owner.to_string());
        }
        self.update_by = Set(ctx.owner.to_string());
    }

    fn create_table_statement(db: DbBackend) -> TableCreateStatement {
        let mut builder = Table::create();
        builder
            .table(Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(Column::Id).not_null().string().primary_key())
            // Specific
            .col(ColumnDef::new(Column::Code).not_null().string())
            .col(ColumnDef::new(Column::Name).not_null().string())
            .col(ColumnDef::new(Column::Note).not_null().string())
            .col(ColumnDef::new(Column::Icon).not_null().string())
            .col(ColumnDef::new(Column::Sort).not_null().big_integer())
            // Basic
            .col(ColumnDef::new(Column::OwnPaths).not_null().string())
            .col(ColumnDef::new(Column::Owner).not_null().string())
            // With Scope
            .col(ColumnDef::new(Column::ScopeLevel).not_null().small_integer())
            .col(ColumnDef::new(Column::CreateBy).not_null().string())
            .col(ColumnDef::new(Column::UpdateBy).not_null().string());
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

    fn create_index_statement() -> Vec<IndexCreateStatement> {
        tardis_create_index_statement()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

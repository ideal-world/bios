use std::fmt::Debug;
use std::process::id;

use async_trait::async_trait;
use sea_orm::{ActiveModelBehavior, ColumnTrait, ConnectionTrait, DatabaseConnection, DbErr, EntityTrait, FromQueryResult, IntoActiveModel, StatementBuilder};
use sea_query::{backend, Alias, Expr, IntoTableRef, JoinType, Query};
use tardis::basic::dto::TardisContext;
use tardis::basic::error::TardisError;
use tardis::basic::result::TardisResult;
use tardis::TardisFuns;

pub mod rbum_item;
pub mod rbum_kind;
pub mod rbum_kind_attr;

pub struct BiosRelDBClient {}

impl BiosRelDBClient {
    // pub async fn build_detail_resp<S,T,C>(main_table: T,main_columns:C) ->
    // TardisResult<S>
    //     where
    //         S: sea_query::SelectStatement,
    // T:IntoTableRef,
    // C:ColumnTrait,
    // {
    //     let creator_table = Alias::new("creator");
    //     let updater_table = Alias::new("updater");
    //     let rel_app_table = Alias::new("relApp");
    //     let rel_tenant_table = Alias::new("relTenant");
    //
    //    let mut query = Query::select();
    //     query.expr_as(Expr::tbl(creator_table.clone(), rbum_item::Column::Name), Alias::new("creator_name"))
    //         .expr_as(Expr::tbl(updater_table.clone(), rbum_item::Column::Name), Alias::new("updater_name"))
    //         .expr_as(Expr::tbl(rel_app_table.clone(), rbum_item::Column::Name), Alias::new("rel_app_name"))
    //         .expr_as(Expr::tbl(rel_tenant_table.clone(), rbum_item::Column::Name), Alias::new("rel_tenant_name"))
    //         .from(main_table)
    //         .join_as(
    //             JoinType::LeftJoin,
    //             rbum_item::Entity,
    //             creator_table.clone(),
    //             Expr::tbl(creator_table, rbum_item::Column::Id).equals(main_table, main_columns::CreatorId),
    //         )
    //         .join_as(
    //             JoinType::LeftJoin,
    //             rbum_item::Entity,
    //             updater_table.clone(),
    //             Expr::tbl(updater_table, rbum_item::Column::Id).equals(main_table, rbum_kind::Column::UpdaterId),
    //         )
    //         .join_as(
    //             JoinType::LeftJoin,
    //             rbum_item::Entity,
    //             rel_app_table.clone(),
    //             Expr::tbl(rel_app_table, rbum_item::Column::Id).equals(rbum_kind::Entity, rbum_kind::Column::RelAppId),
    //         )
    //         .join_as(
    //             JoinType::LeftJoin,
    //             rbum_item::Entity,
    //             rel_tenant_table.clone(),
    //             Expr::tbl(rel_tenant_table, rbum_item::Column::Id).equals(rbum_kind::Entity, rbum_kind::Column::RelTenantId),
    //         );
    //     Ok(query)
    // }

    pub async fn get<'a, S, C, D>(statement: &S, db: &'a C, cxt: &TardisContext) -> TardisResult<Option<D>>
    where
        S: StatementBuilder,
        C: ConnectionTrait,
        D: FromQueryResult,
    {
        let result = D::find_by_statement(TardisFuns::reldb().backend().build(statement)).one(db).await;
        match result {
            Ok(r) => TardisResult::Ok(r),
            Err(err) => TardisResult::Err(TardisError::from(err)),
        }
    }
}

#[async_trait]
pub trait BiosSeaORMExtend: Clone + Debug {
    type Entity: EntityTrait;

    async fn insert_with_cxt<'a, C>(mut self, db: &'a C, cxt: &TardisContext) -> Result<<<Self as sea_orm::ActiveModelTrait>::Entity as EntityTrait>::Model, DbErr>
    where
        <<Self as sea_orm::ActiveModelTrait>::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        Self: ActiveModelBehavior + 'a,
        C: ConnectionTrait,
    {
        self.insert_cxt(cxt);
        let am = ActiveModelBehavior::before_save(self, true)?;
        let model = <<Self as sea_orm::ActiveModelTrait>::Entity as EntityTrait>::insert(am).exec_with_returning(db).await?;
        Self::after_save(model, true)
    }

    async fn update_with_cxt<'a, C>(mut self, db: &'a C, cxt: &TardisContext) -> Result<<<Self as sea_orm::ActiveModelTrait>::Entity as EntityTrait>::Model, DbErr>
    where
        <<Self as sea_orm::ActiveModelTrait>::Entity as EntityTrait>::Model: IntoActiveModel<Self>,
        Self: ActiveModelBehavior + 'a,
        C: ConnectionTrait,
    {
        self.update_cxt(cxt);
        let am = ActiveModelBehavior::before_save(self, false)?;
        let model: <<Self as sea_orm::ActiveModelTrait>::Entity as EntityTrait>::Model = <Self as sea_orm::ActiveModelTrait>::Entity::update(am).exec(db).await?;
        Self::after_save(model, false)
    }

    fn insert_cxt(&mut self, cxt: &TardisContext);
    fn update_cxt(&mut self, cxt: &TardisContext);
}

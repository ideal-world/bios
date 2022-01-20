/*
 * Copyright 2022. the original author or authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{ColumnDef, Table, TableCreateStatement};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelBehavior, DbBackend};

use crate::db::domain::bios_db_config;
use crate::BIOSFuns;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "bios_config")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(indexed)]
    pub k: String,
    #[sea_orm(column_type = "Text")]
    pub v: String,
    pub creator: String,
    pub updater: String,
    pub create_time: DateTime,
    pub update_time: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(BIOSFuns::field.uuid_str()),
            ..ActiveModelTrait::default()
        }
    }
}

pub fn create_table_statement(db_type: DbBackend) -> TableCreateStatement {
    match db_type {
        DbBackend::MySql => Table::create()
            .table(bios_db_config::Entity.table_ref())
            .if_not_exists()
            .col(ColumnDef::new(bios_db_config::Column::Id).not_null().string().primary_key())
            .col(ColumnDef::new(bios_db_config::Column::K).not_null().string())
            .col(ColumnDef::new(bios_db_config::Column::V).not_null().text())
            .col(ColumnDef::new(bios_db_config::Column::Creator).not_null().string())
            .col(ColumnDef::new(bios_db_config::Column::Updater).not_null().string())
            .col(ColumnDef::new(bios_db_config::Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
            .col(ColumnDef::new(bios_db_config::Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).date_time())
            .to_owned(),
        DbBackend::Postgres => {
            Table::create()
                .table(bios_db_config::Entity.table_ref())
                .if_not_exists()
                .col(ColumnDef::new(bios_db_config::Column::Id).not_null().string().primary_key())
                .col(ColumnDef::new(bios_db_config::Column::K).not_null().string())
                .col(ColumnDef::new(bios_db_config::Column::V).not_null().text())
                .col(ColumnDef::new(bios_db_config::Column::Creator).not_null().string())
                .col(ColumnDef::new(bios_db_config::Column::Updater).not_null().string())
                .col(ColumnDef::new(bios_db_config::Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
                // TODO update time
                .col(ColumnDef::new(bios_db_config::Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
                .to_owned()
        }
        DbBackend::Sqlite =>
        // TODO
        {
            Table::create()
                .table(bios_db_config::Entity.table_ref())
                .if_not_exists()
                .col(ColumnDef::new(bios_db_config::Column::Id).not_null().string().primary_key())
                .col(ColumnDef::new(bios_db_config::Column::K).not_null().string())
                .col(ColumnDef::new(bios_db_config::Column::V).not_null().text())
                .col(ColumnDef::new(bios_db_config::Column::Creator).not_null().string())
                .col(ColumnDef::new(bios_db_config::Column::Updater).not_null().string())
                .col(ColumnDef::new(bios_db_config::Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
                .col(ColumnDef::new(bios_db_config::Column::UpdateTime).extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string()).date_time())
                .to_owned()
        }
    }
}

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

use crate::db::domain::bios_db_del_record;
use crate::BIOSFuns;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "bios_del_record")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    #[sea_orm(indexed)]
    pub entity_name: String,
    #[sea_orm(indexed)]
    pub record_id: String,
    #[sea_orm(column_type = "Text")]
    pub content: String,
    pub creator: String,
    pub create_time: DateTime,
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

pub fn create_table_statement(_: DbBackend) -> TableCreateStatement {
    Table::create()
        .table(bios_db_del_record::Entity.table_ref())
        .if_not_exists()
        .col(ColumnDef::new(bios_db_del_record::Column::Id).not_null().string().primary_key())
        .col(ColumnDef::new(bios_db_del_record::Column::EntityName).not_null().string())
        .col(ColumnDef::new(bios_db_del_record::Column::RecordId).not_null().string())
        .col(ColumnDef::new(bios_db_del_record::Column::Content).not_null().text())
        .col(ColumnDef::new(bios_db_del_record::Column::Creator).not_null().string())
        .col(ColumnDef::new(bios_db_del_record::Column::CreateTime).extra("DEFAULT CURRENT_TIMESTAMP".to_string()).date_time())
        .to_owned()
}

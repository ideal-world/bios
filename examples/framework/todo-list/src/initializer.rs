/*
 * Copyright 2021. gudaoxuri
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
use sea_query::{ColumnDef, Query, Table};
use sqlx::Connection;

use bios_framework::basic::error::BIOSResult;
use bios_framework::db::reldb_client::SqlBuilderProcess;
use bios_framework::BIOSFuns;

use crate::domain::{Category, Item};

pub async fn init() -> BIOSResult<()> {
    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    BIOSFuns::reldb()
        .exec(
            &Table::create()
                .table(Category::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Category::Id)
                        .big_integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(Category::Name).not_null().string())
                .done(),
            Some(&mut tx),
        )
        .await?;

    BIOSFuns::reldb()
        .exec(
            &Table::create()
                .table(Item::Table)
                .if_not_exists()
                .col(
                    ColumnDef::new(Item::Id)
                        .big_integer()
                        .not_null()
                        .auto_increment()
                        .primary_key(),
                )
                .col(ColumnDef::new(Item::Content).not_null().string())
                .col(ColumnDef::new(Item::Creator).not_null().string())
                .col(
                    ColumnDef::new(Item::CreateTime)
                        .extra("DEFAULT CURRENT_TIMESTAMP".to_string())
                        .timestamp(),
                )
                .col(
                    ColumnDef::new(Item::UpdateTime)
                        .extra("DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP".to_string())
                        .timestamp(),
                )
                .col(ColumnDef::new(Item::CategoryId).not_null().big_integer())
                .done(),
            Some(&mut tx),
        )
        .await?;
    if BIOSFuns::reldb()
        .count(
            &Query::select()
                .columns(vec![Category::Id])
                .from(Category::Table)
                .done(),
            Some(&mut tx),
        )
        .await?
        != 0
    {
        tx.commit().await?;
        return Ok(());
    }

    let category_id = BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(Category::Table)
                .columns(vec![Category::Name])
                .values_panic(vec!["默认分类".into()])
                .done(),
            Some(&mut tx),
        )
        .await?
        .last_insert_id();
    BIOSFuns::reldb()
        .exec(
            &Query::insert()
                .into_table(Item::Table)
                .columns(vec![Item::Content, Item::Creator, Item::CategoryId])
                .values_panic(vec!["默认任务".into(), "admin".into(), category_id.into()])
                .done(),
            Some(&mut tx),
        )
        .await?;
    tx.commit().await?;
    Ok(())
}

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
use bios_framework::basic::error::BIOSResult;
use bios_framework::db::reldb_client::BIOSDB;
use rbatis::crud::{CRUDMut, CRUD};
use rbatis::executor::Executor;

use crate::domain::{Category, Item};

pub async fn init() -> BIOSResult<()> {
    BIOSDB
        .exec(
            r#"
create table if not exists `todo_category` (
  `id` bigint auto_increment primary key,
  `name` varchar(10) not null comment 'category name'
) comment 'Category';
    "#,
            &vec![],
        )
        .await?;

    BIOSDB
        .exec(
            r#"
create table if not exists `todo_item` (
  `id` bigint auto_increment primary key,
  `content` varchar(255) not null comment 'item content',
  `creator` varchar(10) not null comment 'item creator',
  `create_time` timestamp default CURRENT_TIMESTAMP null comment 'item create time',
  `update_time` timestamp default CURRENT_TIMESTAMP null on update CURRENT_TIMESTAMP comment 'item update time',
  `category_id` bigint not null comment 'item related category'
) comment 'Item';
    "#,
            &vec![],
        )
        .await?;

    if !BIOSDB.fetch_list::<Category>().await?.is_empty() {
        return Ok(());
    }
    let mut tx = BIOSDB.acquire_begin().await?;
    let id = tx
        .save(&Category {
            id: None,
            name: Some("默认分类".to_string()),
        })
        .await?
        .last_insert_id
        .unwrap();
    tx.save(&Item {
        id: None,
        content: Some("默认任务".to_string()),
        creator: Some("admin".to_string()),
        create_time: None,
        update_time: None,
        category_id: Some(id),
    })
    .await?;
    tx.commit().await?;
    Ok(())
}

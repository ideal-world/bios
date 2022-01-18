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

use actix_web::{delete, get, post, put, HttpRequest};
use sea_query::{Alias, Expr, Query};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::{DateTime, Utc};
use sqlx::Connection;
use validator::Validate;

use bios::basic::dto::BIOSResp;
use bios::db::reldb_client::SqlBuilderProcess;
use bios::web::resp_handler::BIOSResponse;
use bios::web::validate::json::Json;
use bios::web::validate::query::Query as VQuery;
use bios::BIOSFuns;

use crate::domain::{Category, Item};

#[get("/categories")]
pub async fn list_categories(query: VQuery<CategoryListReq>) -> BIOSResponse {
    let sql_builder = Query::select()
        .columns(vec![Category::Id, Category::Name])
        .from(Category::Table)
        .and_where_option(if query.name.as_ref().is_some() {
            Some(Expr::col(Category::Name).like(query.name.as_ref().unwrap().as_str()))
        } else {
            None
        })
        .done();
    let categories = BIOSFuns::reldb().fetch_all::<CategoryResp>(&sql_builder, None).await?;
    BIOSResp::ok(categories, None)
}

#[post("/category")]
pub async fn add_category(category: Json<CategoryAddOrModifyReq>) -> BIOSResponse {
    let sql_builder = Query::insert().into_table(Category::Table).columns(vec![Category::Name]).values_panic(vec![category.name.clone().into()]).done();
    let result = BIOSFuns::reldb().exec(&sql_builder, None).await?;
    let id = result.last_insert_id();
    BIOSResp::ok(id, None)
}

#[put("/category/{id}")]
pub async fn modify_category(req: HttpRequest, category: Json<CategoryAddOrModifyReq>) -> BIOSResponse {
    let id: i64 = req.match_info().get("id").unwrap().parse()?;
    let sql_builder = Query::update().table(Category::Table).values(vec![(Category::Name, category.name.clone().into())]).and_where(Expr::col(Category::Id).eq(id)).done();
    BIOSFuns::reldb().exec(&sql_builder, None).await?;
    BIOSResp::ok("", None)
}

#[delete("/category/{id}")]
pub async fn delete_category(req: HttpRequest) -> BIOSResponse {
    let id: i64 = req.match_info().get("id").unwrap().parse()?;
    let mut conn = BIOSFuns::reldb().conn().await;
    let mut tx = conn.begin().await?;

    BIOSFuns::reldb().exec(&Query::delete().from_table(Category::Table).and_where(Expr::col(Category::Id).eq(id)).done(), Some(&mut tx)).await?;
    BIOSFuns::reldb().exec(&Query::delete().from_table(Item::Table).and_where(Expr::col(Item::CategoryId).eq(id)).done(), Some(&mut tx)).await?;

    tx.commit().await?;
    BIOSResp::ok("", None)
}

#[derive(Deserialize, Validate)]
pub struct CategoryListReq {
    #[validate(length(min = 2, max = 10))]
    name: Option<String>,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct CategoryAddOrModifyReq {
    #[validate(length(min = 2, max = 10))]
    name: String,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct CategoryResp {
    id: i64,
    name: String,
}

// -----------------------

#[get("/items")]
pub async fn page_items(query: VQuery<ItemPageReq>) -> BIOSResponse {
    let sql_builder = Query::select()
        .columns(vec![
            (Item::Table, Item::Id),
            (Item::Table, Item::Content),
            (Item::Table, Item::Creator),
            (Item::Table, Item::CreateTime),
            (Item::Table, Item::UpdateTime),
        ])
        .expr_as(Expr::col((Category::Table, Category::Name)), Alias::new("category_name"))
        .from(Item::Table)
        .left_join(Category::Table, Expr::tbl(Category::Table, Category::Id).equals(Item::Table, Item::CategoryId))
        .and_where_option(if query.content.as_ref().is_some() {
            Some(Expr::col(Item::Content).like(query.content.as_ref().unwrap().as_str()))
        } else {
            None
        })
        .and_where_option(if query.category_id.is_some() {
            Some(Expr::col(Item::Creator).eq(query.category_id.unwrap()))
        } else {
            None
        })
        .done();
    let items = BIOSFuns::reldb().pagination::<ItemResp>(&sql_builder, query.page_number, query.page_size, None).await?;
    BIOSResp::ok(items, None)
}

#[post("/item")]
pub async fn add_item(item: Json<ItemAddReq>) -> BIOSResponse {
    let sql_builder = Query::insert()
        .into_table(Item::Table)
        .columns(vec![Item::Content, Item::Creator, Item::CategoryId])
        .values_panic(vec![item.content.clone().into(), item.creator.clone().into(), item.category_id.into()])
        .done();
    let result = BIOSFuns::reldb().exec(&sql_builder, None).await?;
    let id = result.last_insert_id();
    BIOSResp::ok(id, None)
}

#[put("/item/{id}")]
pub async fn modify_item(req: HttpRequest, item: Json<ItemModifyReq>) -> BIOSResponse {
    let id: i64 = req.match_info().get("id").unwrap().parse()?;
    let mut values = Vec::new();
    if item.content.as_ref().is_some() {
        values.push((Item::Content, item.content.as_ref().unwrap().as_str().into()));
    }
    if item.creator.as_ref().is_some() {
        values.push((Item::Creator, item.creator.as_ref().unwrap().as_str().into()));
    }
    let sql_builder = Query::update().table(Item::Table).values(values).and_where(Expr::col(Item::Id).eq(id)).done();
    let result = BIOSFuns::reldb().exec(&sql_builder, None).await?;
    if result.rows_affected() != 0 {
        BIOSResp::ok("", None)
    } else {
        BIOSResp::error("404", "", None)
    }
}

#[delete("/item/{id}")]
pub async fn delete_item(req: HttpRequest) -> BIOSResponse {
    let id: i64 = req.match_info().get("id").unwrap().parse()?;
    BIOSFuns::reldb().exec(&Query::delete().from_table(Item::Table).and_where(Expr::col(Item::Id).eq(id)).done(), None).await?;
    BIOSResp::ok("", None)
}

#[derive(Deserialize, Validate)]
pub struct ItemPageReq {
    #[validate(length(min = 2, max = 255))]
    content: Option<String>,
    category_id: Option<i64>,
    page_number: u64,
    page_size: u64,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct ItemAddReq {
    #[validate(length(min = 2, max = 255))]
    pub content: String,
    #[validate(length(min = 2, max = 10))]
    pub creator: String,
    pub category_id: i64,
}

#[derive(Deserialize, Serialize, Validate)]
pub struct ItemModifyReq {
    #[validate(length(min = 2, max = 255))]
    pub content: Option<String>,
    #[validate(length(min = 2, max = 10))]
    pub creator: Option<String>,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct ItemResp {
    pub id: i64,
    pub content: String,
    pub creator: String,
    pub update_time: DateTime<Utc>,
    pub category_name: String,
}

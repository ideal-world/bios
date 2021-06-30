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

use actix_web::{delete, get, post, put, HttpRequest};
use actix_web_validator::{Json, Query};
use rbatis::crud::{CRUDMut, CRUD};
use rbatis::plugin::page::PageRequest;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

use bios_framework::db::reldb_client::BIOSDB;
use bios_framework::web::resp_handler::{BIOSResp, BIOSRespHelper};

use crate::domain::{Category, Item};

#[get("/categories")]
pub async fn list_categories(query: Query<CategoryListReq>) -> BIOSResp {
    let select = BIOSDB.new_wrapper().do_if(query.name.is_some(), |w| {
        w.like("name", query.name.as_ref().unwrap())
    });
    let categories: Vec<CategoryResp> = BIOSDB
        .fetch_list_by_wrapper::<Category>(&select)
        .await?
        .iter()
        .map(|category| CategoryResp {
            id: category.id.unwrap(),
            name: category.name.as_ref().unwrap().to_string(),
        })
        .collect();
    BIOSRespHelper::ok(categories)
}

#[post("/category")]
pub async fn add_category(category: Json<CategoryAddOrModifyReq>) -> BIOSResp {
    BIOSDB
        .save(&Category {
            id: None,
            name: Some(category.name.to_string()),
        })
        .await?;
    BIOSRespHelper::ok("")
}

#[put("/category/{id}")]
pub async fn modify_category(req: HttpRequest, category: Json<CategoryAddOrModifyReq>) -> BIOSResp {
    let id: i64 = req.match_info().get("id").unwrap().parse()?;
    BIOSDB
        .update_by_column(
            "id",
            &mut Category {
                id: Some(id),
                name: Some(category.name.to_string()),
            },
        )
        .await?;
    BIOSRespHelper::ok("")
}

#[delete("/category/{id}")]
pub async fn delete_category(req: HttpRequest) -> BIOSResp {
    let id: i64 = req.match_info().get("id").unwrap().parse()?;
    let mut tx = BIOSDB.acquire_begin().await?;
    tx.remove_by_column::<Item, i64>("category_id", &id).await?;
    tx.remove_by_column::<Category, i64>("id", &id).await?;
    tx.commit().await?;
    BIOSRespHelper::ok("")
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

#[derive(Deserialize, Serialize)]
pub struct CategoryResp {
    id: i64,
    name: String,
}

// -----------------------

#[get("/items")]
pub async fn page_items(query: Query<ItemPageReq>) -> BIOSResp {
    let (mut sql, mut args) = ("".to_owned(), Vec::<Value>::new());
    if let Some(content) = &query.content {
        sql += " and i.content like %?%";
        args.push(Value::from(content.to_string()));
    }
    if let Some(category_id) = &query.category_id {
        sql += " and i.category_id = ?";
        args.push(Value::from(*category_id));
    }
    let categories = BIOSDB
        .fetch_page::<ItemResp>(
            &format!(
                r#"
    select i.id, i.content, i.creator, i.update_time, c.name as category_name from todo_item i
      left join todo_category c on c.id = i.category_id
      where 1 = 1{}
    "#,
                sql
            ),
            &args,
            &PageRequest::new(
                query.page_number.unwrap_or(1),
                query.page_size.unwrap_or(10),
            ),
        )
        .await?;
    BIOSRespHelper::ok(categories)
}

#[post("/item")]
pub async fn add_item(item: Json<ItemAddReq>) -> BIOSResp {
    BIOSDB
        .save(&Item {
            id: None,
            content: Some(item.content.to_string()),
            creator: Some(item.creator.to_string()),
            create_time: None,
            update_time: None,
            category_id: Some(item.category_id),
        })
        .await?;
    BIOSRespHelper::ok("")
}

#[put("/item/{id}")]
pub async fn modify_item(req: HttpRequest, item: Json<ItemModifyReq>) -> BIOSResp {
    let id: i64 = req.match_info().get("id").unwrap().parse()?;
    if BIOSDB
        .update_by_column(
            "id",
            &mut Item {
                id: Some(id),
                content: item.content.clone(),
                creator: item.creator.clone(),
                create_time: None,
                update_time: None,
                category_id: None,
            },
        )
        .await?
        != 0
    {
        BIOSRespHelper::ok("")
    } else {
        BIOSRespHelper::bus_err("404", "")
    }
}

#[delete("/item/{id}")]
pub async fn delete_item(req: HttpRequest) -> BIOSResp {
    let id: i64 = req.match_info().get("id").unwrap().parse()?;
    BIOSDB.remove_by_column::<Item, i64>("id", &id).await?;
    BIOSRespHelper::ok("")
}

#[derive(Deserialize, Validate)]
pub struct ItemPageReq {
    #[validate(length(min = 2, max = 255))]
    content: Option<String>,
    category_id: Option<i64>,
    page_number: Option<u64>,
    page_size: Option<u64>,
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

#[derive(Deserialize, Serialize)]
pub struct ItemResp {
    pub id: i64,
    pub content: String,
    pub creator: String,
    pub update_time: String,
    pub category_name: String,
}

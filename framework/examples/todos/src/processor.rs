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

use poem_openapi::param::Query;
use poem_openapi::{param::Path, payload::Json, Object, OpenApi};
use sea_orm::sea_query::Expr;
use sea_orm::ActiveValue::Set;
use sea_orm::*;
use serde::{Deserialize, Serialize};

use bios::db::reldb_client::BIOSSeaORMExtend;
use bios::web::web_resp::{BIOSPage, BIOSResp};
use bios::BIOSFuns;

use crate::domain::todos;

#[derive(Object, FromQueryResult, Serialize, Deserialize, Debug)]
struct TodoDetailResp {
    id: i32,
    description: String,
    done: bool,
}

#[derive(Object, Serialize, Deserialize, Debug)]
struct TodoAddReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    description: String,
    done: bool,
}

#[derive(Object, Serialize, Deserialize, Debug)]
struct TodoModifyReq {
    #[oai(validator(min_length = "2", max_length = "255"))]
    description: Option<String>,
    done: Option<bool>,
}

pub struct TodoApi;

#[OpenApi]
impl TodoApi {
    #[oai(path = "/todo", method = "post")]
    async fn add(&self, todo_add_req: Json<TodoAddReq>) -> BIOSResp<i32> {
        let todo = todos::ActiveModel {
            description: Set(todo_add_req.description.to_string()),
            done: Set(todo_add_req.done),
            ..Default::default()
        }
        .insert(BIOSFuns::reldb().conn())
        .await
        .unwrap();
        BIOSResp::ok(todo.id)
    }

    #[oai(path = "/todo/:id", method = "get")]
    async fn get(&self, id: Path<i32>) -> BIOSResp<TodoDetailResp> {
        let todo_detail_resp = todos::Entity::find()
            .filter(todos::Column::Id.eq(id.0))
            .select_only()
            .column(todos::Column::Id)
            .column(todos::Column::Description)
            .column(todos::Column::Done)
            .into_model::<TodoDetailResp>()
            .one(BIOSFuns::reldb().conn())
            .await
            .unwrap()
            .unwrap();
        BIOSResp::ok(todo_detail_resp)
    }

    #[oai(path = "/todo", method = "get")]
    async fn get_all(&self, page_number: Query<usize>, page_size: Query<usize>) -> BIOSResp<BIOSPage<TodoDetailResp>> {
        let result = todos::Entity::find()
            .select_only()
            .column(todos::Column::Id)
            .column(todos::Column::Description)
            .column(todos::Column::Done)
            .order_by_desc(todos::Column::Id)
            .into_model::<TodoDetailResp>()
            .paginate(BIOSFuns::reldb().conn(), page_size.0);
        BIOSResp::ok(BIOSPage {
            page_size: page_size.0,
            page_number: result.num_pages().await.unwrap(),
            total_size: result.num_items().await.unwrap(),
            records: result.fetch_page(page_number.0 - 1).await.unwrap(),
        })
    }

    #[oai(path = "/todo/:id", method = "delete")]
    async fn delete(&self, id: Path<i32>) -> BIOSResp<usize> {
        let delete_num = todos::Entity::find().filter(todos::Column::Id.eq(id.0)).soft_delete(BIOSFuns::reldb().conn(), "").await.unwrap();
        BIOSResp::ok(delete_num)
    }

    #[oai(path = "/todo/:id", method = "put")]
    async fn update(&self, id: Path<i32>, todo_modify_req: Json<TodoModifyReq>) -> BIOSResp<usize> {
        let mut update = todos::Entity::update_many().filter(todos::Column::Id.eq(id.0));
        if let Some(description) = &todo_modify_req.description {
            update = update.col_expr(todos::Column::Description, Expr::value(description.clone()));
        }
        if let Some(done) = todo_modify_req.done {
            update = update.col_expr(todos::Column::Done, Expr::value(done));
        }
        let update_num = update.exec(BIOSFuns::reldb().conn()).await.unwrap().rows_affected;
        BIOSResp::ok(update_num as usize)
    }
}

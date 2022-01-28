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
use sea_orm::ActiveModelBehavior;
use sea_orm::ActiveValue::Set;

use bios::BIOSFuns;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "test_tenant")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_one = "super::tenant_conf::Entity")]
    TenantConfig,
    #[sea_orm(has_many = "super::app::Entity")]
    App,
}

impl Related<super::tenant_conf::Entity> for super::tenant::Entity {
    fn to() -> RelationDef {
        Relation::TenantConfig.def()
    }
}

impl Related<super::app::Entity> for super::tenant::Entity {
    fn to() -> RelationDef {
        Relation::App.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(BIOSFuns::field.uuid_str()),
            ..ActiveModelTrait::default()
        }
    }
}

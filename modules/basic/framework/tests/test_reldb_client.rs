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

// https://github.com/SeaQL/sea-orm

use std::time::Duration;

use log::{info, log};
use sea_orm::entity::*;
use sea_orm::query::*;
use sea_orm::sea_query::Expr;
use sea_orm::ActiveValue::Set;
pub use sea_orm::FromQueryResult;
use sea_orm::{QueryFilter, QueryOrder};
use testcontainers::clients;
use tokio::time::sleep;

use bios::basic::result::BIOSResult;
use bios::db::domain::{bios_db_config, bios_db_del_record};
use bios::db::reldb_client::BIOSRelDBClient;
use bios::db::reldb_client::BIOSSeaORMExtend;
use bios::test::test_container::BIOSTestContainer;
use bios::BIOSFuns;

#[tokio::test]
async fn test_reldb_client() -> BIOSResult<()> {
    BIOSFuns::init_log()?;

    let docker = clients::Cli::default();
    let mysql_container = BIOSTestContainer::mysql_custom(None, &docker);
    let port = mysql_container.get_host_port(3306).expect("Test port acquisition error");
    let url = format!("mysql://root:123456@localhost:{}/test", port);

    let client = BIOSRelDBClient::init(&url, 10, 5, None, None).await?;

    test_basic(&client).await?;
    test_rel(&client).await?;
    test_transaction(&client).await?;
    test_advanced_query(&client).await?;

    Ok(())
}

async fn test_advanced_query(client: &BIOSRelDBClient) -> BIOSResult<()> {
    // Prepare data
    entities::app_account_rel::Entity::delete_many().exec(client.conn()).await?;
    entities::account::Entity::delete_many().exec(client.conn()).await?;
    entities::app::Entity::delete_many().exec(client.conn()).await?;
    entities::tenant_conf::Entity::delete_many().exec(client.conn()).await?;
    entities::tenant::Entity::delete_many().exec(client.conn()).await?;

    let tenant = entities::tenant::ActiveModel {
        name: Set("tenant1".to_owned()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;

    entities::app::ActiveModel {
        name: Set("app1".to_owned()),
        tenant_id: Set(tenant.id.clone()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;

    let app = entities::app::ActiveModel {
        name: Set("app2".to_owned()),
        tenant_id: Set(tenant.id.clone()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;

    let account = entities::account::ActiveModel {
        name: Set("account1".to_owned()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;
    entities::app_account_rel::ActiveModel {
        app_id: Set(app.id.to_owned()),
        account_id: Set(account.id.to_owned()),
    }
    .insert(client.conn())
    .await?;

    // Select to DTO
    #[derive(Debug, FromQueryResult)]
    struct SelectResult {
        name: String,
        aa_id: String,
    }

    let select_result = entities::tenant::Entity::find()
        .select_only()
        .column(entities::tenant::Column::Name)
        .column_as(entities::tenant::Column::Id, "aa_id")
        .into_model::<SelectResult>()
        .one(client.conn())
        .await?
        .unwrap();
    assert_eq!(select_result.aa_id, tenant.id);
    assert_eq!(select_result.name, "tenant1");

    // AND Condition
    let apps =
        entities::app::Entity::find().filter(Condition::all().add(entities::app::Column::Id.eq("__")).add(entities::app::Column::Name.like("%app%"))).all(client.conn()).await?;
    assert_eq!(apps.len(), 0);

    // OR Condition
    let apps =
        entities::app::Entity::find().filter(Condition::any().add(entities::app::Column::Id.eq("__")).add(entities::app::Column::Name.like("%app%"))).all(client.conn()).await?;
    assert_eq!(apps.len(), 2);

    // Group By
    let apps = entities::app::Entity::find()
        .select_only()
        .column(entities::app::Column::Name)
        .column_as(entities::app::Column::Id.count(), "count")
        .group_by(entities::app::Column::Name)
        .into_json()
        .all(client.conn())
        .await?;
    assert_eq!(apps[0]["count"], 1);

    // Join
    let tenants = entities::tenant::Entity::find()
        .select_only()
        .column(entities::tenant::Column::Name)
        .column_as(entities::tenant_conf::Column::Name, "conf_name")
        .left_join(entities::tenant_conf::Entity)
        .into_json()
        .all(client.conn())
        .await?;
    assert_eq!(tenants.len(), 1);
    let tenants = entities::tenant::Entity::find()
        .select_only()
        .column(entities::tenant::Column::Name)
        .column_as(entities::tenant_conf::Column::Name, "conf_name")
        .inner_join(entities::tenant_conf::Entity)
        .into_json()
        .all(client.conn())
        .await?;
    assert_eq!(tenants.len(), 0);

    let apps = entities::app::Entity::find()
        .select_only()
        .column(entities::app::Column::Name)
        .column_as(entities::tenant::Column::Name, "tenant_name")
        .left_join(entities::tenant::Entity)
        .filter(entities::tenant::Column::Name.contains("tenant"))
        .into_json()
        .all(client.conn())
        .await?;
    assert_eq!(apps.len(), 2);
    assert_eq!(apps[0]["tenant_name"], "tenant1");

    let apps = entities::app::Entity::find()
        .select_only()
        .column(entities::app::Column::Name)
        .column_as(entities::tenant::Column::Name, "tenant_name")
        .join(
            JoinType::LeftJoin,
            // construct `RelationDef` on the fly
            entities::app::Entity::belongs_to(entities::tenant::Entity).from(entities::app::Column::TenantId).to(entities::tenant::Column::Id).into(),
        )
        .filter(entities::tenant::Column::Name.contains("tenant"))
        .into_json()
        .all(client.conn())
        .await?;
    assert_eq!(apps.len(), 2);
    assert_eq!(apps[0]["tenant_name"], "tenant1");

    Ok(())
}

async fn test_transaction(client: &BIOSRelDBClient) -> BIOSResult<()> {
    // Normal transaction
    let tx = client.conn().begin().await?;

    let config = bios_db_config::ActiveModel {
        k: Set("kn".to_owned()),
        v: Set("vn".to_owned()),
        creator: Set("admin".to_owned()),
        updater: Set("admin".to_owned()),
        ..Default::default()
    }
    .insert(&tx)
    .await?;

    let conf = bios_db_config::Entity::find_by_id(config.id.clone()).one(client.conn()).await?;
    assert_eq!(conf, None);
    let conf = bios_db_config::Entity::find_by_id(config.id.clone()).one(&tx).await?.unwrap();
    assert_eq!(conf.k, "kn");

    tx.commit().await?;

    let conf = bios_db_config::Entity::find_by_id(config.id.clone()).one(client.conn()).await?.unwrap();
    assert_eq!(conf.k, "kn");

    // Rollback transaction

    let tx = client.conn().begin().await?;

    let config = bios_db_config::ActiveModel {
        k: Set("ke".to_owned()),
        v: Set("ve".to_owned()),
        creator: Set("admin".to_owned()),
        updater: Set("admin".to_owned()),
        ..Default::default()
    }
    .insert(&tx)
    .await?;

    tx.rollback().await?;

    let conf = bios_db_config::Entity::find_by_id(config.id.clone()).one(client.conn()).await?;
    assert_eq!(conf, None);

    Ok(())
}

async fn test_rel(client: &BIOSRelDBClient) -> BIOSResult<()> {
    client.create_table_from_entity(entities::tenant::Entity).await?;
    client.create_table_from_entity(entities::tenant_conf::Entity).await?;
    client.create_table_from_entity(entities::app::Entity).await?;
    client.create_table_from_entity(entities::account::Entity).await?;
    client.create_table_from_entity(entities::app_account_rel::Entity).await?;

    entities::tenant::ActiveModel {
        name: Set("tenant1".to_owned()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;

    let tenant = entities::tenant::Entity::find().one(client.conn()).await?.unwrap();
    let config = tenant.find_related(entities::tenant_conf::Entity).one(client.conn()).await?;
    // Not Exists
    assert_eq!(config, None);

    entities::tenant_conf::ActiveModel {
        name: Set("conf1".to_owned()),
        tenant_id: Set(tenant.id.clone()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;

    entities::app::ActiveModel {
        name: Set("app1".to_owned()),
        tenant_id: Set(tenant.id.clone()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;

    entities::app::ActiveModel {
        name: Set("app2".to_owned()),
        tenant_id: Set(tenant.id.clone()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;

    let tenant = entities::tenant::Entity::find_by_id(tenant.id.clone()).one(client.conn()).await?.unwrap();

    info!("----------------- One To One -----------------");
    let config = tenant.find_related(entities::tenant_conf::Entity).one(client.conn()).await?.unwrap();
    assert_eq!(config.name, "conf1");
    let tenant = config.find_related(entities::tenant::Entity).one(client.conn()).await?.unwrap();
    assert_eq!(tenant.name, "tenant1");

    info!("----------------- One To Many -----------------");
    let apps = tenant.find_related(entities::app::Entity).all(client.conn()).await?;
    assert_eq!(apps.len(), 2);
    info!("----------------- Many To One -----------------");
    let tenant = apps[0].find_related(entities::tenant::Entity).one(client.conn()).await?.unwrap();
    assert_eq!(tenant.name, "tenant1");

    info!("----------------- Many To Many -----------------");
    let accounts = apps[0].find_related(entities::account::Entity).all(client.conn()).await?;
    assert_eq!(accounts.len(), 0);

    let account = entities::account::ActiveModel {
        name: Set("account1".to_owned()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;
    entities::app_account_rel::ActiveModel {
        app_id: Set(apps[0].id.to_owned()),
        account_id: Set(account.id.to_owned()),
    }
    .insert(client.conn())
    .await?;

    let accounts = apps[0].find_related(entities::account::Entity).all(client.conn()).await?;
    assert_eq!(accounts.len(), 1);

    Ok(())
}

async fn test_basic(client: &BIOSRelDBClient) -> BIOSResult<()> {
    // Insert
    bios_db_config::ActiveModel {
        k: Set("k1".to_owned()),
        v: Set("v1".to_owned()),
        creator: Set("admin".to_owned()),
        updater: Set("admin".to_owned()),
        ..Default::default()
    }
    .insert(client.conn())
    .await?;

    let conf2 = bios_db_config::ActiveModel {
        k: Set("k2".to_owned()),
        v: Set("v2".to_owned()),
        creator: Set("admin".to_owned()),
        updater: Set("admin".to_owned()),
        ..Default::default()
    };
    let insert_result = bios_db_config::Entity::insert(conf2).exec(client.conn()).await?;

    // Find One
    let conf2 = bios_db_config::Entity::find_by_id(insert_result.last_insert_id.clone()).one(client.conn()).await?.unwrap();
    assert_eq!(conf2.k, "k2");
    assert_eq!(conf2.create_time, conf2.update_time);

    // Update One
    sleep(Duration::from_millis(1100)).await;
    let mut conf2: bios_db_config::ActiveModel = conf2.into();
    conf2.v = Set("v2更新".to_owned());
    conf2.update(client.conn()).await?;
    let conf2 = bios_db_config::Entity::find_by_id(insert_result.last_insert_id.clone()).one(client.conn()).await?.unwrap();
    assert_eq!(conf2.v, "v2更新");
    assert_ne!(conf2.create_time, conf2.update_time);

    // Update Many
    bios_db_config::Entity::update_many()
        .col_expr(bios_db_config::Column::V, Expr::value("v1更新"))
        .filter(bios_db_config::Column::Id.ne(insert_result.last_insert_id))
        .exec(client.conn())
        .await?;

    // Find Many
    let confs = bios_db_config::Entity::find().filter(bios_db_config::Column::K.contains("k")).order_by_desc(bios_db_config::Column::K).all(client.conn()).await?;
    assert_eq!(confs.len(), 2);
    assert_eq!(confs[0].k, "k2");
    assert_eq!(confs[1].k, "k1");
    assert_eq!(confs[0].v, "v2更新");
    assert_eq!(confs[1].v, "v1更新");

    // Page
    let conf_page = bios_db_config::Entity::find().filter(bios_db_config::Column::K.contains("k1")).order_by_desc(bios_db_config::Column::K).paginate(client.conn(), 1);
    assert_eq!(conf_page.num_pages().await.unwrap(), 1);
    assert_eq!(conf_page.cur_page(), 0);
    let confs = conf_page.fetch_page(0).await?;
    assert_eq!(confs.len(), 1);
    assert_eq!(confs[0].k, "k1");
    assert_eq!(confs[0].v, "v1更新");

    // Exists TODO https://github.com/SeaQL/sea-orm/issues/408

    // Soft Delete
    bios_db_config::Entity::find().soft_delete(client.conn(), "admin").await?;
    let dels = bios_db_del_record::Entity::find().all(client.conn()).await?;
    assert_eq!(dels.len(), 2);
    assert_eq!(dels[0].entity_name, "bios_config");

    // Delete
    let delete_result = bios_db_del_record::Entity::delete_many().filter(bios_db_del_record::Column::Id.eq(dels[0].id.clone())).exec(client.conn()).await?;
    assert_eq!(delete_result.rows_affected, 1);

    // Count
    let count = bios_db_del_record::Entity::find().count(client.conn()).await?;
    assert_eq!(count, 1);

    Ok(())
}

pub mod entities {

    pub mod tenant {
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
    }

    pub mod tenant_conf {
        use sea_orm::entity::prelude::*;
        use sea_orm::ActiveModelBehavior;
        use sea_orm::ActiveValue::Set;

        use bios::BIOSFuns;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "test_tenant_conf")]
        pub struct Model {
            #[sea_orm(primary_key, auto_increment = false)]
            pub id: String,
            pub name: String,
            pub tenant_id: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {
            #[sea_orm(belongs_to = "super::tenant::Entity", from = "Column::TenantId", to = "super::tenant::Column::Id")]
            Tenant,
        }

        impl Related<super::tenant::Entity> for super::tenant_conf::Entity {
            fn to() -> RelationDef {
                Relation::Tenant.def()
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
    }

    pub mod app {
        use sea_orm::entity::prelude::*;
        use sea_orm::ActiveModelBehavior;
        use sea_orm::ActiveValue::Set;

        use bios::BIOSFuns;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "test_app")]
        pub struct Model {
            #[sea_orm(primary_key, auto_increment = false)]
            pub id: String,
            pub name: String,
            pub tenant_id: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {
            #[sea_orm(belongs_to = "super::tenant::Entity", from = "Column::TenantId", to = "super::tenant::Column::Id")]
            Tenant,
        }

        impl Related<super::tenant::Entity> for super::app::Entity {
            fn to() -> RelationDef {
                Relation::Tenant.def()
            }
        }

        impl Related<super::account::Entity> for super::app::Entity {
            fn to() -> RelationDef {
                super::app_account_rel::Relation::Account.def()
            }

            fn via() -> Option<RelationDef> {
                Some(super::app_account_rel::Relation::App.def().rev())
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
    }

    pub mod account {
        use sea_orm::entity::prelude::*;
        use sea_orm::ActiveModelBehavior;
        use sea_orm::ActiveValue::Set;

        use bios::BIOSFuns;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "test_account")]
        pub struct Model {
            #[sea_orm(primary_key, auto_increment = false)]
            pub id: String,
            pub name: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {}

        impl Related<super::app::Entity> for super::account::Entity {
            fn to() -> RelationDef {
                super::app_account_rel::Relation::App.def()
            }

            fn via() -> Option<RelationDef> {
                Some(super::app_account_rel::Relation::Account.def().rev())
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
    }

    pub mod app_account_rel {
        use sea_orm::entity::prelude::*;
        use sea_orm::ActiveModelBehavior;

        #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
        #[sea_orm(table_name = "test_app_account_rel")]
        pub struct Model {
            #[sea_orm(primary_key, auto_increment = false)]
            pub app_id: String,
            #[sea_orm(primary_key, auto_increment = false)]
            pub account_id: String,
        }

        #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
        pub enum Relation {
            #[sea_orm(
                belongs_to = "super::app::Entity",
                from = "Column::AppId",
                to = "super::app::Column::Id",
                on_update = "Cascade",
                on_delete = "Cascade"
            )]
            App,
            #[sea_orm(
                belongs_to = "super::account::Entity",
                from = "Column::AccountId",
                to = "super::account::Column::Id",
                on_update = "Cascade",
                on_delete = "Cascade"
            )]
            Account,
        }

        impl ActiveModelBehavior for ActiveModel {}
    }
}

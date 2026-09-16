use std::collections::HashMap;
use std::env;
use std::time::Duration;

use bios_basic::rbum::domain::rbum_rel_attr;
use bios_basic::rbum::dto::rbum_domain_dto::RbumDomainAddReq;
use bios_basic::rbum::dto::rbum_filer_dto::RbumRelExtFilterReq;
use bios_basic::rbum::dto::rbum_item_dto::RbumItemAddReq;
use bios_basic::rbum::dto::rbum_kind_attr_dto::RbumKindAttrAddReq;
use bios_basic::rbum::dto::rbum_kind_dto::RbumKindAddReq;
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAttrAggAddReq;
use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumScopeLevelKind, RbumWidgetTypeKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_domain_serv::RbumDomainServ;
use bios_basic::rbum::serv::rbum_item_serv::RbumItemServ;
use bios_basic::rbum::serv::rbum_kind_serv::{RbumKindAttrServ, RbumKindServ};
use bios_basic::rbum::serv::rbum_rel_serv::RbumRelAttrServ;
use bios_basic::spi::dto::spi_bs_dto::SpiBsAddReq;
use bios_basic::spi::spi_initializer;
use bios_basic::test::init_test_container;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_plugin::dto::plugin_api_dto::PluginApiAddOrModifyReq;
use bios_spi_plugin::dto::plugin_bs_dto::{PluginBsAddReq, PluginBsInfoResp};
use bios_spi_plugin::dto::plugin_kind_dto::PluginKindAddAggReq;
use bios_spi_plugin::plugin_config::PluginConfig;
use bios_spi_plugin::plugin_constants::DOMAIN_CODE;
use bios_spi_plugin::plugin_enumeration::PluginApiMethodKind;
use bios_spi_plugin::plugin_initializer;
use bios_spi_plugin::serv::plugin_rel_serv::PluginRelServ;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::db::sea_orm;
use tardis::db::sea_orm::sea_query::{Expr, Query};
use tardis::db::sea_orm::Set;
use tardis::serde_json::{self, json};
use tardis::tokio::time::sleep;
use tardis::web::web_resp::{TardisPage, Void};
use tardis::{tokio, TardisFuns, TardisFunsInst};
mod test_plugin_exec;

#[tokio::test]
async fn test_plugin() -> TardisResult<()> {
    env::set_var("RUST_LOG", "debug,test_plugin=trace,sqlx::query=off");

    let _x = init_test_container::init(None).await?;
    init_data().await?;

    Ok(())
}

async fn init_data() -> TardisResult<()> {
    let web_server = TardisFuns::web_server();
    // Initialize SPI plugin
    plugin_initializer::init(&web_server).await.unwrap();

    tokio::spawn(async move {
        web_server.start().await.unwrap();
    });

    sleep(Duration::from_millis(500)).await;

    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    funs.mq()
        .subscribe(&funs.conf::<PluginConfig>().mq_topic_event_plugin_delete, |(_, msg)| async move {
            let msg: serde_json::Value = serde_json::from_str(&msg).unwrap_or_default();
            let rel_id = msg.get("rel_id").and_then(|v| v.as_str()).unwrap_or_default();
            println!("Received plugin delete event for rel_id: {}", rel_id);
            assert_eq!(rel_id, "1");
            Ok(())
        })
        .await?;
    sleep(Duration::from_millis(500)).await;
    funs.mq()
        .publish(
            &funs.conf::<PluginConfig>().mq_topic_event_plugin_delete,
            json!({ "rel_id": "1".to_string() }).to_string(),
            &HashMap::new(),
        )
        .await?;

    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };
    let app_kind_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString("iam-app".to_string()),
            name: TrimString("iam-app".to_string()),
            note: None,
            icon: None,
            sort: None,
            module: None,
            ext_table_name: Some("iam_app".to_lowercase()),
            scope_level: Some(RbumScopeLevelKind::Root),
            parent_id: None,
        },
        &funs,
        &ctx,
    )
    .await?;
    let iam_domain_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString("iam".to_string()),
            name: TrimString("iam".to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &ctx,
    )
    .await?;
    spi_initializer::add_kind("gitlib", &funs, &ctx).await?;
    let kind_id = RbumKindServ::get_rbum_kind_id_by_code("gitlib", &funs).await?.unwrap();
    let kind_attr_1 = RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("url".to_string()),
            module: None,
            label: "url".to_string(),
            note: None,
            sort: None,
            main_column: None,
            position: None,
            capacity: None,
            overload: None,
            hide: None,
            secret: None,
            show_by_conds: None,
            idx: None,
            data_type: RbumDataTypeKind::String,
            widget_type: RbumWidgetTypeKind::Input,
            widget_columns: None,
            default_value: None,
            dyn_default_value: None,
            options: None,
            dyn_options: None,
            required: None,
            min_length: None,
            max_length: None,
            parent_attr_name: None,
            action: None,
            ext: None,
            rel_rbum_kind_id: kind_id.clone(),
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &ctx,
    )
    .await?;
    let kind_attr_2 = RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("ak".to_string()),
            module: None,
            label: "ak".to_string(),
            note: None,
            sort: None,
            main_column: None,
            position: None,
            capacity: None,
            overload: None,
            hide: None,
            secret: None,
            show_by_conds: None,
            idx: None,
            data_type: RbumDataTypeKind::String,
            widget_type: RbumWidgetTypeKind::Input,
            widget_columns: None,
            default_value: None,
            dyn_default_value: None,
            options: None,
            dyn_options: None,
            required: None,
            min_length: None,
            max_length: None,
            parent_attr_name: None,
            action: None,
            ext: None,
            rel_rbum_kind_id: kind_id.clone(),
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &ctx,
    )
    .await?;
    let kind_attr_3 = RbumKindAttrServ::add_rbum(
        &mut RbumKindAttrAddReq {
            name: TrimString("sk".to_string()),
            module: None,
            label: "sk".to_string(),
            note: None,
            sort: None,
            main_column: None,
            position: None,
            capacity: None,
            overload: None,
            hide: None,
            secret: Some(true),
            show_by_conds: None,
            idx: None,
            data_type: RbumDataTypeKind::String,
            widget_type: RbumWidgetTypeKind::Input,
            widget_columns: None,
            default_value: None,
            dyn_default_value: None,
            options: None,
            dyn_options: None,
            required: None,
            min_length: None,
            max_length: None,
            parent_attr_name: None,
            action: None,
            ext: None,
            rel_rbum_kind_id: kind_id.clone(),
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        &funs,
        &ctx,
    )
    .await?;
    let _ = RbumItemServ::add_rbum(
        &mut RbumItemAddReq {
            id: Some("app001".into()),
            code: Some("app001".into()),
            name: "app001".into(),
            rel_rbum_kind_id: app_kind_id,
            rel_rbum_domain_id: iam_domain_id,
            scope_level: Some(RbumScopeLevelKind::Root),
            disabled: None,
        },
        &funs,
        &ctx,
    )
    .await?;
    let base_url = format!("http://127.0.0.1:8080/{}", DOMAIN_CODE);
    let mut client = TestHttpClient::new(base_url.clone());

    client.set_auth(&ctx)?;

    let bs_id: String = client
        .post(
            "/ci/manage/bs",
            &SpiBsAddReq {
                name: TrimString("test-spi".to_string()),
                kind_id: TrimString(kind_id.clone()),
                conn_uri: base_url.to_string(),
                ak: TrimString("minioadmin".to_string()),
                sk: TrimString("minioadmin".to_string()),
                ext: r#"{"region":"us-east-1"}"#.to_string(),
                private: false,
                disabled: None,
            },
        )
        .await;
    let attrs: Vec<RbumRelAttrAggAddReq> = vec![
        RbumRelAttrAggAddReq {
            is_from: true,
            value: "http://xxx".to_string(),
            name: Some("url".to_string()),
            record_only: true,
            rel_rbum_kind_attr_id: Some(kind_attr_1),
        },
        RbumRelAttrAggAddReq {
            is_from: true,
            value: "ak123".to_string(),
            name: Some("ak".to_string()),
            record_only: true,
            rel_rbum_kind_attr_id: Some(kind_attr_2),
        },
        RbumRelAttrAggAddReq {
            is_from: true,
            value: "sk123".to_string(),
            name: Some("sk".to_string()),
            record_only: true,
            rel_rbum_kind_attr_id: Some(kind_attr_3),
        },
    ];
    let _: String = client
        .put(
            "/ci/spi/plugin/api",
            &PluginApiAddOrModifyReq {
                code: TrimString("test-api".to_string()),
                name: TrimString("test-api".to_string()),
                kind_id: TrimString(kind_id.clone()),
                callback: "".to_string(),
                content_type: "".to_string(),
                timeout: 0,
                ext: "".to_string(),
                http_method: PluginApiMethodKind::DELETE,
                kind: "".to_string(),
                path_and_query: "ci/spi/plugin/test/exec/:msg".to_string(),
                save_message: true,
            },
        )
        .await;
    client.set_auth(&TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    })?;
    let rel_id: String = client
        .put(
            &format!("/ci/manage/bs/rel",),
            &PluginBsAddReq {
                attrs: Some(attrs.clone()),
                rel_id: None,
                bs_id: bs_id.clone(),
                app_tenant_id: "app001".to_string(),
                name: "test-rel".to_string(),
            },
        )
        .await;
    let rel_id: String = client
        .put(
            &format!("/ci/manage/bs/rel",),
            &PluginBsAddReq {
                attrs: Some(attrs),
                rel_id: Some(rel_id.clone()),
                bs_id: bs_id.clone(),
                app_tenant_id: "app001".to_string(),
                name: "test-rel".to_string(),
            },
        )
        .await;
    let bs_resp: PluginBsInfoResp = client.get(&format!("/ci/manage/bs/rel/{}", rel_id)).await;
    assert_eq!(bs_resp.name, "test-spi");
    assert_eq!(bs_resp.kind_code, "gitlib");
    assert!(bs_resp.rel.is_some());
    assert!(bs_resp.rel.as_ref().unwrap().attrs.len() == 3);
    assert!(bs_resp.rel.as_ref().unwrap().attrs.iter().any(|attr| attr.name == "url" && attr.value == "http://xxx"));
    assert!(bs_resp.rel.as_ref().unwrap().attrs.iter().any(|attr| attr.name == "ak" && attr.value == "ak123"));
    assert!(bs_resp.rel.as_ref().unwrap().attrs.iter().any(|attr| attr.name == "sk" && attr.value == "sk123"));
    let bs_hide_secret: PluginBsInfoResp = client.get(&format!("/ci/manage/bs/rel/hide/secret/{}", rel_id)).await;
    assert_eq!(bs_hide_secret.name, "test-spi");
    assert_eq!(bs_hide_secret.kind_code, "gitlib");
    assert!(bs_hide_secret.rel.is_some());
    assert!(bs_hide_secret.rel.as_ref().unwrap().attrs.len() == 2);
    assert!(bs_hide_secret.rel.as_ref().unwrap().attrs.iter().any(|attr| attr.name == "url" && attr.value == "http://xxx"));
    assert!(bs_hide_secret.rel.as_ref().unwrap().attrs.iter().any(|attr| attr.name == "ak" && attr.value == "ak123"));

    // Simulate the legacy plugin data which is stored in plaintext, then encrypt it by the migration
    //
    // 模拟明文存储的老插件数据，并通过迁移加密
    let migrate_ctx = TardisContext {
        own_paths: "t1/app001".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "app001".to_string(),
        ..Default::default()
    };
    let funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let secret_attr_id = RbumRelAttrServ::find_rbums(
        &RbumRelExtFilterReq {
            rel_rbum_rel_id: Some(rel_id.clone()),
            ..Default::default()
        },
        None,
        None,
        &funs,
        &migrate_ctx,
    )
    .await?
    .into_iter()
    .find(|attr| attr.name == "sk")
    .expect("The sensitive attribute should exist")
    .id;
    funs.db()
        .update_one(
            rbum_rel_attr::ActiveModel {
                id: Set(secret_attr_id.clone()),
                value: Set("sk123".to_string()),
                ..Default::default()
            },
            &migrate_ctx,
        )
        .await?;
    assert_eq!(find_stored_attr_value(&secret_attr_id, &funs).await?, "sk123");

    // The dry run only counts and returns the samples
    //
    // 预演只统计并返回样例
    let dry_run_resp = PluginRelServ::migrate_plaintext_secret_attrs(100, true, &funs, &migrate_ctx).await?;
    assert!(dry_run_resp.dry_run);
    assert!(dry_run_resp.total_size >= 1);
    assert_eq!(dry_run_resp.processed_size, 0);
    assert_eq!(find_stored_attr_value(&secret_attr_id, &funs).await?, "sk123");

    // Encrypt the legacy plaintext data
    //
    // 加密存量明文数据
    let migrate_resp = PluginRelServ::migrate_plaintext_secret_attrs(100, false, &funs, &migrate_ctx).await?;
    assert!(migrate_resp.processed_size >= 1);
    assert!(find_stored_attr_value(&secret_attr_id, &funs).await?.starts_with("ENC(sm4v1:"));

    // The reading paths return the plaintext after the migration
    //
    // 迁移后读取路径仍返回明文
    let bs_resp_after_migrate: PluginBsInfoResp = client.get(&format!("/ci/manage/bs/rel/{}", rel_id)).await;
    assert!(bs_resp_after_migrate
        .rel
        .as_ref()
        .unwrap()
        .attrs
        .iter()
        .any(|attr| attr.name == "sk" && attr.value == "sk123"));

    // Migrating again is idempotent
    //
    // 重复迁移幂等
    let migrate_again_resp = PluginRelServ::migrate_plaintext_secret_attrs(100, false, &funs, &migrate_ctx).await?;
    assert_eq!(migrate_again_resp.total_size, 0);
    assert_eq!(migrate_again_resp.processed_size, 0);

    // The dry run of the decryption only counts and returns the samples
    //
    // 解密预演只统计并返回样例
    let decrypt_dry_run_resp = PluginRelServ::decrypt_secret_attrs(100, true, &funs, &migrate_ctx).await?;
    assert!(decrypt_dry_run_resp.dry_run);
    assert!(decrypt_dry_run_resp.total_size >= 1);
    assert_eq!(decrypt_dry_run_resp.processed_size, 0);
    assert!(find_stored_attr_value(&secret_attr_id, &funs).await?.starts_with("ENC(sm4v1:"));

    // Decrypt the legacy data (rollback)
    //
    // 解密存量数据（回退）
    let decrypt_resp = PluginRelServ::decrypt_secret_attrs(100, false, &funs, &migrate_ctx).await?;
    assert!(decrypt_resp.processed_size >= 1);
    assert_eq!(decrypt_resp.failed_size, 0);
    assert_eq!(find_stored_attr_value(&secret_attr_id, &funs).await?, "sk123");

    // The reading paths return the plaintext after the decryption
    //
    // 解密后读取路径仍返回明文
    let bs_resp_after_decrypt: PluginBsInfoResp = client.get(&format!("/ci/manage/bs/rel/{}", rel_id)).await;
    assert!(bs_resp_after_decrypt.rel.as_ref().unwrap().attrs.iter().any(|attr| attr.name == "sk" && attr.value == "sk123"));

    // Decrypting again is idempotent
    //
    // 重复解密幂等
    let decrypt_again_resp = PluginRelServ::decrypt_secret_attrs(100, false, &funs, &migrate_ctx).await?;
    assert_eq!(decrypt_again_resp.total_size, 0);
    assert_eq!(decrypt_again_resp.processed_size, 0);

    // Encrypt the data again, and the final stored value is the ciphertext
    //
    // 再次加密，最终以密文存储
    let encrypt_again_resp = PluginRelServ::migrate_plaintext_secret_attrs(100, false, &funs, &migrate_ctx).await?;
    assert!(encrypt_again_resp.processed_size >= 1);
    assert!(find_stored_attr_value(&secret_attr_id, &funs).await?.starts_with("ENC(sm4v1:"));

    let _: Void = client
        .put(
            &format!("/ci/kind/agg",),
            &PluginKindAddAggReq {
                attrs: None,
                bs_id,
                app_tenant_id: "app001".to_string(),
                kind_id: kind_id.clone(),
                rel_id: Some(rel_id),
                bs_rel: None,
                ignore_exist: None,
            },
        )
        .await;
    let _: TardisPage<PluginBsInfoResp> = client.get(&format!("/ci/manage/bs/rel?app_tenant_id=app001&page_number=1&page_size=10",)).await;
    test_plugin_exec::test(&mut client).await?;
    Ok(())
}

/// Find the stored value of the relationship attribute, without decryption
///
/// 查询关联属性的存储值，不做解密
async fn find_stored_attr_value(id: &str, funs: &TardisFunsInst) -> TardisResult<String> {
    #[derive(Debug, sea_orm::FromQueryResult)]
    struct ValueResp {
        pub value: String,
    }
    Ok(funs
        .db()
        .get_dto::<ValueResp>(Query::select().column(rbum_rel_attr::Column::Value).from(rbum_rel_attr::Entity).and_where(Expr::col(rbum_rel_attr::Column::Id).eq(id)))
        .await?
        .expect("The relationship attribute should exist")
        .value)
}

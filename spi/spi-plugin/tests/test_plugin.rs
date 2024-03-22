use std::env;
use std::time::Duration;

use bios_basic::rbum::dto::rbum_kind_attr_dto::RbumKindAttrAddReq;
use bios_basic::rbum::dto::rbum_rel_agg_dto::RbumRelAttrAggAddReq;
use bios_basic::rbum::rbum_enumeration::{RbumDataTypeKind, RbumScopeLevelKind, RbumWidgetTypeKind};
use bios_basic::rbum::serv::rbum_crud_serv::RbumCrudOperation;
use bios_basic::rbum::serv::rbum_kind_serv::{RbumKindAttrServ, RbumKindServ};
use bios_basic::spi::dto::spi_bs_dto::SpiBsAddReq;
use bios_basic::spi::spi_initializer;
use bios_basic::test::init_rbum_test_container;
use bios_basic::test::test_http_client::TestHttpClient;
use bios_spi_plugin::dto::plugin_api_dto::PluginApiAddOrModifyReq;
use bios_spi_plugin::dto::plugin_bs_dto::PluginBsAddReq;
use bios_spi_plugin::plugin_constants::DOMAIN_CODE;
use bios_spi_plugin::plugin_enumeration::PluginApiMethodKind;
use bios_spi_plugin::plugin_initializer;
use tardis::basic::dto::TardisContext;
use tardis::basic::field::TrimString;
use tardis::basic::result::TardisResult;
use tardis::tokio::time::sleep;
use tardis::{testcontainers, tokio, TardisFuns};
mod test_plugin_exec;

#[tokio::test]
async fn test_plugin() -> TardisResult<()> {
    env::set_var("RUST_LOG", "debug,test_plugin=trace,sqlx::query=off");

    let docker = testcontainers::clients::Cli::default();
    let _x = init_rbum_test_container::init(&docker, None).await?;
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
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };
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
    let base_url = format!("https://127.0.0.1:8080/{}", DOMAIN_CODE);
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
            name: "url".to_string(),
            record_only: true,
            rel_rbum_kind_attr_id: kind_attr_1,
        },
        RbumRelAttrAggAddReq {
            is_from: true,
            value: "ak123".to_string(),
            name: "ak".to_string(),
            record_only: true,
            rel_rbum_kind_attr_id: kind_attr_2,
        },
        RbumRelAttrAggAddReq {
            is_from: true,
            value: "sk123".to_string(),
            name: "sk".to_string(),
            record_only: true,
            rel_rbum_kind_attr_id: kind_attr_3,
        },
    ];
    let _: String = client
        .put(
            "/ci/spi/plugin/api",
            &PluginApiAddOrModifyReq {
                code: TrimString("test-api".to_string()),
                name: TrimString("test-api".to_string()),
                kind_id: TrimString(kind_id),
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
    let _: String = client.put(&format!("/ci/manage/bs/{}/rel/app001", bs_id), &PluginBsAddReq { attrs: Some(attrs) }).await;
    test_plugin_exec::test(&mut client).await?;

    Ok(())
}

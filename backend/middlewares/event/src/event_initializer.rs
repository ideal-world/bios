use std::vec;

use asteroid_mq::{prelude::TopicConfig, protocol::topic::durable_message::LoadTopic};
use bios_basic::rbum::{
    dto::{rbum_domain_dto::RbumDomainAddReq, rbum_kind_dto::RbumKindAddReq},
    rbum_enumeration::RbumScopeLevelKind,
    serv::{rbum_crud_serv::RbumCrudOperation, rbum_domain_serv::RbumDomainServ, rbum_kind_serv::RbumKindServ},
};
use bios_sdk_invoke::clients::event_client::SPI_RPC_TOPIC;
use tardis::{
    basic::{dto::TardisContext, field::TrimString, result::TardisResult},
    db::reldb_client::TardisActiveModel,
    web::web_server::TardisWebServer,
    TardisFuns, TardisFunsInst,
};

use crate::{
    api::{event_listener_api, event_proc_api, event_topic_api},
    domain::event_topic,
    event_config::{EventConfig, EventInfo, EventInfoManager},
    event_constants::{DOMAIN_CODE, KIND_CODE},
};

pub async fn init(web_server: &TardisWebServer) -> TardisResult<()> {
    let mut funs = TardisFuns::inst_with_db_conn(DOMAIN_CODE.to_string(), None);
    let config = funs.conf::<EventConfig>();

    init_api(web_server).await?;
    let ctx = TardisContext {
        own_paths: "".to_string(),
        ak: "".to_string(),
        roles: vec![],
        groups: vec![],
        owner: "".to_string(),
        ..Default::default()
    };
    funs.begin().await?;
    init_db(DOMAIN_CODE.to_string(), KIND_CODE.to_string(), &funs, &ctx).await?;

    funs.commit().await?;
    if config.enable {
        init_mq_cluster(&config.svc).await?;
    }
    Ok(())
}

async fn init_db(domain_code: String, kind_code: String, funs: &TardisFunsInst, ctx: &TardisContext) -> TardisResult<()> {
    if let Some(domain_id) = RbumDomainServ::get_rbum_domain_id_by_code(&domain_code, funs).await? {
        let kind_id = RbumKindServ::get_rbum_kind_id_by_code(&kind_code, funs).await?.expect("missing event kind");
        EventInfoManager::set(EventInfo { kind_id, domain_id })?;
        return Ok(());
    }

    // Initialize event component RBUM item table and indexs
    funs.db().init(event_topic::ActiveModel::init(TardisFuns::reldb().backend(), None, TardisFuns::reldb().compatible_type())).await?;
    // funs.db()
    //     .init(event_persistent::ActiveModel::init(
    //         TardisFuns::reldb().backend(),
    //         None,
    //         TardisFuns::reldb().compatible_type(),
    //     ))
    //     .await?;
    // Initialize event component RBUM domain data
    let domain_id = RbumDomainServ::add_rbum(
        &mut RbumDomainAddReq {
            code: TrimString(domain_code.to_string()),
            name: TrimString(domain_code.to_string()),
            note: None,
            icon: None,
            sort: None,
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        funs,
        ctx,
    )
    .await?;
    // Initialize event component RBUM kind data
    let kind_id = RbumKindServ::add_rbum(
        &mut RbumKindAddReq {
            code: TrimString(kind_code.to_string()),
            name: TrimString(kind_code.to_string()),
            note: None,
            icon: None,
            sort: None,
            module: None,
            ext_table_name: Some("mq_topic".to_lowercase()),
            scope_level: Some(RbumScopeLevelKind::Root),
        },
        funs,
        ctx,
    )
    .await?;
    EventInfoManager::set(EventInfo { kind_id, domain_id })?;
    Ok(())
}

async fn init_api(web_server: &TardisWebServer) -> TardisResult<()> {
    web_server
        .add_module(
            DOMAIN_CODE,
            (event_topic_api::EventTopicApi, event_proc_api::EventProcApi, event_listener_api::EventListenerApi),
        )
        .await;
    Ok(())
}

async fn init_mq_cluster(svc_name: &str) -> TardisResult<()> {
    use bios_sdk_invoke::clients::event_client::mq_error;
    let mq_node = init_mq_node(svc_name).await;
    mq_node
        .load_topic(LoadTopic {
            config: TopicConfig {
                code: SPI_RPC_TOPIC,
                overflow_config: None,
                blocking: false,
            },
            queue: vec![],
        })
        .await
        .map_err(mq_error)?;
    Ok(())
}

pub async fn init_mq_node(svc_name: &str) -> asteroid_mq::prelude::Node {
    if let Some(node) = TardisFuns::store().get_singleton::<asteroid_mq::prelude::Node>() {
        node
    } else {
        let cluster_provider = asteroid_mq::protocol::cluster::k8s::K8sClusterProvider::new(svc_name.to_string(), asteroid_mq::DEFAULT_TCP_PORT).await;
        let uid = std::env::var("POD_UID").expect("POD_UID is required");
        let node = cluster_provider.run(uid).await.expect("failed to run k8s cluster");
        TardisFuns::store().insert_singleton(node.clone());
        node
    }
}
